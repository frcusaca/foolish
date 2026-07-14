//! The `.einmo` containment envelope: parse and serialize (FOOP-92 §4).
//!
//! A `.einmo` file is a **header line**, then **sections** separated by a
//! configurable **separator string**, in the order declared by the metadata
//! section, ending with the JSON STAMPS section and an optional unsigned
//! advisory line.
//!
//! ```text
//! #einmo 1 encoding=utf-8 separator=①\n
//! <metadata>
//! ①
//! <INPUT body>
//! ①
//! <OUTPUT body>            (one per evaluator chunk: OUTPUT, OUTPUT[1], …)
//! ①
//! <perspective bodies…>    (zero or more, named in metadata)
//! ①
//! <COMMENTS body>
//! ①
//! <STAMPS — one JSON object per line>
//! # flagged: <reason> <ISO8601>   (optional, unsigned advisory)
//! ```
//!
//! Einmo is language-agnostic: bodies are opaque text. The **collision rule**
//! (§4.1) refuses to serialize any section whose content contains the
//! configured separator — parsing then stays trivially byte-exact.

use crate::error::{EinmoError, Result};
use crate::signature::{StampCheck, Stamps, derive_keypair};

/// The envelope format version emitted by this build.
const FORMAT_VERSION: u32 = 1;

/// The default section separator: `①` (U+2460) followed by LF.
pub(crate) const DEFAULT_SEPARATOR: &str = "①\n";

/// The Foolish-suite separator: `!!` (a Foolish line comment) followed by LF.
pub(crate) const FOOLISH_SEPARATOR: &str = "!!\n";

/// The metadata `status` field — whether the *harness* ran normally.
///
/// An *expected* error result (a division-by-zero alarm, "infinite loop
/// detected") is [`Status::Normal`]; `status` marks harness abnormality, not
/// the program-under-test's.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// The evaluator ran to completion (even if the program produced an error
    /// *value*). Promotable all the way to `verified/`.
    Normal,
    /// The evaluator could not parse/accept the input.
    InputError,
    /// Evaluation failed abnormally (panic, crash, harness fault).
    OutputError,
}

impl Status {
    fn as_str(self) -> &'static str {
        match self {
            Status::Normal => "normal",
            Status::InputError => "input-error",
            Status::OutputError => "output-error",
        }
    }

    fn parse(s: &str) -> Result<Self> {
        match s {
            "normal" => Ok(Status::Normal),
            "input-error" => Ok(Status::InputError),
            "output-error" => Ok(Status::OutputError),
            other => Err(EinmoError::Parse(format!("unknown status {other:?}"))),
        }
    }
}

/// A named section body (INPUT, OUTPUT[i], a perspective, COMMENTS, DIFF).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    name: String,
    body: String,
}

impl Section {
    /// Construct a section from its name and opaque body text.
    #[must_use]
    pub fn new(name: impl Into<String>, body: impl Into<String>) -> Self {
        Section {
            name: name.into(),
            body: body.into(),
        }
    }

    /// The section name as it appears in the metadata `sections:` list.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The opaque section body.
    #[must_use]
    pub fn body(&self) -> &str {
        &self.body
    }
}

/// The metadata section — key/value lines in a fixed, byte-stable order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Metadata {
    /// Test name = mirror-relative input path.
    pub test: String,
    /// Suite identity.
    pub suite: String,
    /// Commit SHA of the producing tree.
    pub producer: String,
    /// SHA of `git diff` when the tree is dirty; empty when clean (omitted).
    pub producer_diff: String,
    /// ISO-8601 UTC generation time.
    pub generated: String,
    /// Harness status.
    pub status: Status,
    /// Free-text specifics when `status != normal` (may be multi-line — stored
    /// single-line-escaped in the envelope).
    pub status_detail: String,
    /// For dependent einmos (§4.7): the reference's mirror-relative name.
    /// Empty when this is not a dependent.
    pub reference: String,
    /// The ordered section names (INPUT, OUTPUT, …, COMMENTS, STAMPS).
    pub sections: Vec<String>,
}

impl Metadata {
    /// Serialize to the fixed-order key/value block (no trailing separator).
    fn serialize(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("test: {}\n", self.test));
        out.push_str(&format!("suite: {}\n", self.suite));
        out.push_str(&format!("producer: {}\n", self.producer));
        if !self.producer_diff.is_empty() {
            out.push_str(&format!("producer-diff: {}\n", self.producer_diff));
        }
        out.push_str(&format!("generated: {}\n", self.generated));
        out.push_str(&format!("status: {}\n", self.status.as_str()));
        out.push_str(&format!(
            "status-detail: {}\n",
            escape_line(&self.status_detail)
        ));
        if !self.reference.is_empty() {
            out.push_str(&format!("reference: {}\n", self.reference));
        }
        out.push_str(&format!("sections: {}", self.sections.join(", ")));
        out
    }

    fn parse(block: &str) -> Result<Self> {
        let mut test = None;
        let mut suite = None;
        let mut producer = None;
        let mut producer_diff = String::new();
        let mut generated = None;
        let mut status = None;
        let mut status_detail = String::new();
        let mut reference = String::new();
        let mut sections = None;

        for line in block.lines() {
            let Some((key, value)) = line.split_once(':') else {
                continue;
            };
            let value = value.trim_start_matches(' ');
            match key {
                "test" => test = Some(value.to_string()),
                "suite" => suite = Some(value.to_string()),
                "producer" => producer = Some(value.to_string()),
                "producer-diff" => producer_diff = value.to_string(),
                "generated" => generated = Some(value.to_string()),
                "status" => status = Some(Status::parse(value)?),
                "status-detail" => status_detail = unescape_line(value),
                "reference" => reference = value.to_string(),
                "sections" => {
                    sections = Some(
                        value
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect::<Vec<_>>(),
                    );
                }
                _ => {}
            }
        }

        Ok(Metadata {
            test: test.ok_or_else(|| EinmoError::Parse("missing metadata `test`".into()))?,
            suite: suite.ok_or_else(|| EinmoError::Parse("missing metadata `suite`".into()))?,
            producer: producer
                .ok_or_else(|| EinmoError::Parse("missing metadata `producer`".into()))?,
            producer_diff,
            generated: generated
                .ok_or_else(|| EinmoError::Parse("missing metadata `generated`".into()))?,
            status: status.ok_or_else(|| EinmoError::Parse("missing metadata `status`".into()))?,
            status_detail,
            reference,
            sections: sections
                .ok_or_else(|| EinmoError::Parse("missing metadata `sections`".into()))?,
        })
    }
}

/// Escape a possibly-multi-line value onto a single metadata line.
fn escape_line(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

/// Inverse of [`escape_line`].
fn unescape_line(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('r') => out.push('\r'),
                Some('\\') => out.push('\\'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// A parsed `.einmo` file: header, metadata, ordered bodies, stamps, and an
/// optional unsigned advisory line.
///
/// Construct via [`EinmoFile::new`] (in-memory) or parse via
/// [`EinmoFile::parse`]. Reading from disk with verify-on-inspect lives in the
/// `verify` module ([`EinmoFile::from_file`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EinmoFile {
    encoding: String,
    separator: String,
    metadata: Metadata,
    /// Body sections in file order: INPUT, OUTPUT…, perspectives, COMMENTS,
    /// (DIFF). Excludes the metadata and STAMPS sections.
    sections: Vec<Section>,
    stamps: Stamps,
    /// The unsigned `# flagged: …` advisory line, if present (without the LF).
    advisory: Option<String>,
}

impl EinmoFile {
    /// Assemble an in-memory envelope from its parts.
    ///
    /// `sections` are the body sections in file order (INPUT first). The
    /// metadata `sections:` list is not auto-derived here — the suite builder
    /// sets it — but [`EinmoFile::serialize`] validates that the declared list
    /// matches the actual bodies plus the trailing STAMPS.
    #[must_use]
    pub fn new(
        encoding: impl Into<String>,
        separator: impl Into<String>,
        metadata: Metadata,
        sections: Vec<Section>,
        stamps: Stamps,
    ) -> Self {
        EinmoFile {
            encoding: encoding.into(),
            separator: separator.into(),
            metadata,
            sections,
            stamps,
            advisory: None,
        }
    }

    /// The file's metadata.
    #[must_use]
    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    /// The body sections in file order.
    #[must_use]
    pub fn sections(&self) -> &[Section] {
        &self.sections
    }

    /// The stamp chain.
    #[must_use]
    pub fn stamps(&self) -> &Stamps {
        &self.stamps
    }

    /// The configured section separator.
    #[must_use]
    pub fn separator(&self) -> &str {
        &self.separator
    }

    /// The unsigned advisory line, if the file was flagged.
    #[must_use]
    pub fn advisory(&self) -> Option<&str> {
        self.advisory.as_deref()
    }

    /// Look up a body section by name.
    #[must_use]
    pub fn section(&self, name: &str) -> Option<&Section> {
        self.sections.iter().find(|s| s.name() == name)
    }

    /// Replace the stamp chain (used by transitions after appending a stamp).
    pub(crate) fn set_stamps(&mut self, stamps: Stamps) {
        self.stamps = stamps;
    }

    /// Attach (or replace) the unsigned advisory line.
    pub(crate) fn set_advisory(&mut self, advisory: impl Into<String>) {
        self.advisory = Some(advisory.into());
    }

    /// The exact file bytes **before the first stamp line** — the message the
    /// generation chain and every stage stamp cover.
    ///
    /// This is: header line + LF, then each section (metadata, bodies) joined
    /// by the separator, then the separator that introduces the STAMPS section.
    #[must_use]
    pub fn signed_prefix(&self) -> Vec<u8> {
        let mut out = self.header_line();
        out.push('\n');
        out.push_str(&self.metadata.serialize());
        out.push_str(&self.separator);
        for section in &self.sections {
            out.push_str(&section.body);
            out.push_str(&self.separator);
        }
        // The final separator above introduces the STAMPS section.
        out.into_bytes()
    }

    fn header_line(&self) -> String {
        format!(
            "#einmo {FORMAT_VERSION} encoding={} separator={}",
            self.encoding,
            escape_separator(&self.separator)
        )
    }

    /// Verify every stamp against this file's own signed prefix.
    #[must_use]
    pub fn verify_all(&self) -> Vec<StampCheck> {
        self.stamps.verify_chain(&self.signed_prefix())
    }

    /// Append a `stage_key` stamp signed by the key derived from `passphrase`.
    ///
    /// The caller must have already verified the chain (verify-on-inspect);
    /// this method assumes the existing stamps are valid and appends over all
    /// current file bytes. Returns the hex pubkey of the appended stamp so the
    /// caller can warn on a non-human (computer-key) verified attestation.
    pub(crate) fn append_stage_stamp(&mut self, stage_key: &str, passphrase: &str) -> String {
        let (signing, verifying) = derive_keypair(passphrase);
        let prefix = self.stamps.prefix_for_next_stamp(&self.signed_prefix());
        self.stamps.append_stage(stage_key, &signing, &prefix);
        hex::encode(verifying.to_bytes())
    }

    /// `true` iff the whole stamp chain verifies.
    #[must_use]
    pub fn chain_valid(&self) -> bool {
        self.stamps.chain_valid(&self.signed_prefix())
    }

    /// Serialize to the byte-exact `.einmo` file form.
    ///
    /// # Errors
    ///
    /// Returns [`EinmoError::SeparatorCollision`] if any section body (or the
    /// metadata) contains the configured separator — the suite must configure a
    /// different separator (§4.1).
    pub fn serialize(&self) -> Result<Vec<u8>> {
        // Collision rule: no signed section may contain the separator sequence.
        let meta = self.metadata.serialize();
        if meta.contains(&self.separator) {
            return Err(EinmoError::SeparatorCollision {
                section: "metadata".into(),
            });
        }
        for section in &self.sections {
            if section.body.contains(&self.separator) {
                return Err(EinmoError::SeparatorCollision {
                    section: section.name.clone(),
                });
            }
        }

        let mut out = self.signed_prefix();
        out.extend_from_slice(self.stamps.serialize().as_bytes());
        if let Some(advisory) = &self.advisory {
            out.push(b'\n');
            out.extend_from_slice(advisory.as_bytes());
        }
        Ok(out)
    }

    /// Parse a `.einmo` file from bytes.
    ///
    /// This does **not** verify signatures; use `EinmoFile::from_file` (the
    /// `verify` module) for verify-on-inspect.
    ///
    /// # Errors
    ///
    /// Returns [`EinmoError::Parse`] on a malformed header, missing section, or
    /// bad UTF-8.
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        let text =
            std::str::from_utf8(bytes).map_err(|e| EinmoError::Parse(format!("bad utf-8: {e}")))?;

        let (header, rest) = text
            .split_once('\n')
            .ok_or_else(|| EinmoError::Parse("missing header line".into()))?;
        let (encoding, separator) = parse_header(header)?;

        // Split off the optional advisory line, which sits after the STAMPS
        // section and is not part of the separated body.
        let (main, advisory) = split_advisory(rest);

        let parts: Vec<&str> = main.split(&separator).collect();
        if parts.len() < 3 {
            return Err(EinmoError::Parse(
                "too few sections (need metadata, ≥1 body, STAMPS)".into(),
            ));
        }

        let metadata = Metadata::parse(parts[0])?;
        // The last part is STAMPS; everything between is body sections.
        let stamps = Stamps::parse(parts[parts.len() - 1])?;

        // Body section names come from metadata.sections minus the trailing
        // STAMPS entry. There must be exactly one body part per declared name.
        let declared_bodies: Vec<&String> = metadata
            .sections
            .iter()
            .filter(|s| s.as_str() != "STAMPS")
            .collect();
        let body_parts = &parts[1..parts.len() - 1];
        if body_parts.len() != declared_bodies.len() {
            return Err(EinmoError::Parse(format!(
                "declared {} body sections but found {}",
                declared_bodies.len(),
                body_parts.len()
            )));
        }
        let sections = declared_bodies
            .iter()
            .zip(body_parts.iter())
            .map(|(name, body)| Section::new((*name).clone(), (*body).to_string()))
            .collect();

        Ok(EinmoFile {
            encoding,
            separator,
            metadata,
            sections,
            stamps,
            advisory: advisory.map(str::to_string),
        })
    }
}

/// Render a separator for the header line, escaping the trailing LF.
fn escape_separator(sep: &str) -> String {
    sep.replace('\\', "\\\\").replace('\n', "\\n")
}

/// Inverse of [`escape_separator`].
fn unescape_separator(s: &str) -> String {
    unescape_line(s)
}

/// Parse the `#einmo <version> encoding=<enc> separator=<escaped>` header.
fn parse_header(header: &str) -> Result<(String, String)> {
    let mut parts = header.split_whitespace();
    let magic = parts
        .next()
        .ok_or_else(|| EinmoError::Parse("empty header".into()))?;
    if magic != "#einmo" {
        return Err(EinmoError::Parse(format!("bad header magic {magic:?}")));
    }
    let version = parts
        .next()
        .ok_or_else(|| EinmoError::Parse("header missing version".into()))?;
    if version != FORMAT_VERSION.to_string() {
        return Err(EinmoError::Parse(format!(
            "unsupported format version {version:?}"
        )));
    }
    let mut encoding = None;
    let mut separator = None;
    for kv in parts {
        if let Some(v) = kv.strip_prefix("encoding=") {
            encoding = Some(v.to_string());
        } else if let Some(v) = kv.strip_prefix("separator=") {
            separator = Some(unescape_separator(v));
        }
    }
    Ok((
        encoding.ok_or_else(|| EinmoError::Parse("header missing encoding=".into()))?,
        separator.ok_or_else(|| EinmoError::Parse("header missing separator=".into()))?,
    ))
}

/// Split the trailing unsigned advisory line off the main (separated) content.
///
/// The advisory, if present, is the final line beginning with `# flagged:`.
/// Everything up to it is the separated body; the advisory is returned
/// separately so verification never sees it.
fn split_advisory(main: &str) -> (&str, Option<&str>) {
    // The advisory is the last line and starts with "# flagged:".
    if let Some(idx) = main.rfind("\n# flagged:") {
        let (body, adv) = main.split_at(idx);
        // adv starts with "\n# flagged:…"; strip the leading LF.
        (body, Some(adv.trim_start_matches('\n')))
    } else if main.starts_with("# flagged:") {
        // Degenerate: advisory with no preceding content.
        ("", Some(main))
    } else {
        (main, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signature::derive_keypair;

    fn sample_metadata(sections: Vec<&str>) -> Metadata {
        Metadata {
            test: "arith/simple.foo".into(),
            suite: "zweimomo/suites/foolish".into(),
            producer: "abc1234".into(),
            producer_diff: String::new(),
            generated: "2026-07-11T07:00:00Z".into(),
            status: Status::Normal,
            status_detail: String::new(),
            reference: String::new(),
            sections: sections.into_iter().map(String::from).collect(),
        }
    }

    fn signed_file(separator: &str, bodies: Vec<Section>) -> EinmoFile {
        // Build declared sections = body names + STAMPS.
        let mut section_names: Vec<String> = bodies.iter().map(|s| s.name().to_string()).collect();
        section_names.push("STAMPS".into());
        let mut meta = sample_metadata(vec![]);
        meta.sections = section_names;

        let mut file = EinmoFile::new("utf-8", separator, meta, bodies, Stamps::new());
        // Stamp it so it round-trips through a real chain.
        let (configured, _) = derive_keypair("cfg");
        let (stage, _) = derive_keypair("");
        let stamps = Stamps::generate(&file.signed_prefix(), &configured, &stage);
        file.set_stamps(stamps);
        file
    }

    #[test]
    fn roundtrip_byte_exact_default_separator() {
        let file = signed_file(
            DEFAULT_SEPARATOR,
            vec![
                Section::new("INPUT", "{2 + 3;}"),
                Section::new("OUTPUT", "5"),
                Section::new("COMMENTS", ""),
            ],
        );
        let bytes = file.serialize().expect("serialize");
        let parsed = EinmoFile::parse(&bytes).expect("parse");
        assert_eq!(file, parsed);
        assert_eq!(bytes, parsed.serialize().unwrap(), "byte-exact roundtrip");
    }

    #[test]
    fn roundtrip_foolish_separator() {
        let file = signed_file(
            FOOLISH_SEPARATOR,
            vec![
                Section::new("INPUT", "{x = 42; y = x + 8; y;}"),
                Section::new("OUTPUT", "y = Int(50)"),
                Section::new("COMMENTS", "reviewed"),
            ],
        );
        let bytes = file.serialize().expect("serialize");
        let parsed = EinmoFile::parse(&bytes).expect("parse");
        assert_eq!(file, parsed);
    }

    #[test]
    fn generated_chain_verifies_after_roundtrip() {
        let file = signed_file(
            DEFAULT_SEPARATOR,
            vec![
                Section::new("INPUT", "{5;}"),
                Section::new("OUTPUT", "5"),
                Section::new("COMMENTS", ""),
            ],
        );
        assert!(file.chain_valid());
        let bytes = file.serialize().unwrap();
        let parsed = EinmoFile::parse(&bytes).unwrap();
        assert!(parsed.chain_valid(), "chain must verify after parse");
    }

    #[test]
    fn separator_collision_refused() {
        // Body contains the default separator sequence.
        let file = signed_file(
            DEFAULT_SEPARATOR,
            vec![
                Section::new("INPUT", "contains ①\n a separator"),
                Section::new("OUTPUT", "x"),
                Section::new("COMMENTS", ""),
            ],
        );
        let err = file.serialize().unwrap_err();
        assert!(matches!(err, EinmoError::SeparatorCollision { .. }));
    }

    #[test]
    fn multiple_output_sections() {
        let file = signed_file(
            DEFAULT_SEPARATOR,
            vec![
                Section::new("INPUT", "{1;} {2;}"),
                Section::new("OUTPUT", "1"),
                Section::new("OUTPUT[1]", "2"),
                Section::new("COMMENTS", ""),
            ],
        );
        let bytes = file.serialize().unwrap();
        let parsed = EinmoFile::parse(&bytes).unwrap();
        assert_eq!(parsed.section("OUTPUT[1]").unwrap().body(), "2");
    }

    #[test]
    fn perspective_section_roundtrips() {
        let file = signed_file(
            DEFAULT_SEPARATOR,
            vec![
                Section::new("INPUT", "{a=1,b=2}"),
                Section::new("OUTPUT", "..."),
                Section::new("names-perspective", "{a=???,b=???}"),
                Section::new("COMMENTS", ""),
            ],
        );
        let bytes = file.serialize().unwrap();
        let parsed = EinmoFile::parse(&bytes).unwrap();
        assert_eq!(
            parsed.section("names-perspective").unwrap().body(),
            "{a=???,b=???}"
        );
    }

    #[test]
    fn status_and_detail_roundtrip_multiline() {
        let mut file = signed_file(
            DEFAULT_SEPARATOR,
            vec![
                Section::new("INPUT", "bad input"),
                Section::new("OUTPUT", ""),
                Section::new("COMMENTS", ""),
            ],
        );
        // Reassemble with an error status and multi-line detail.
        let mut meta = file.metadata().clone();
        meta.status = Status::InputError;
        meta.status_detail = "line 1 of detail\nline 2 of detail".into();
        let mut rebuilt = EinmoFile::new(
            file.encoding.clone(),
            file.separator.clone(),
            meta,
            file.sections().to_vec(),
            Stamps::new(),
        );
        let (configured, _) = derive_keypair("cfg");
        let (stage, _) = derive_keypair("");
        rebuilt.set_stamps(Stamps::generate(
            &rebuilt.signed_prefix(),
            &configured,
            &stage,
        ));
        let bytes = rebuilt.serialize().unwrap();
        let parsed = EinmoFile::parse(&bytes).unwrap();
        assert_eq!(parsed.metadata().status, Status::InputError);
        assert_eq!(
            parsed.metadata().status_detail,
            "line 1 of detail\nline 2 of detail"
        );
        assert!(parsed.chain_valid());
        file.set_advisory("unused"); // exercise setter
    }

    #[test]
    fn advisory_line_excluded_from_signed_bytes() {
        let mut file = signed_file(
            DEFAULT_SEPARATOR,
            vec![
                Section::new("INPUT", "{5;}"),
                Section::new("OUTPUT", "5"),
                Section::new("COMMENTS", ""),
            ],
        );
        // Chain valid before flagging.
        assert!(file.chain_valid());
        file.set_advisory("# flagged: regenerate 2026-07-11T07:00:00Z");
        let bytes = file.serialize().unwrap();
        let parsed = EinmoFile::parse(&bytes).unwrap();
        // Advisory parsed off; signed content unchanged so chain still valid.
        assert_eq!(
            parsed.advisory(),
            Some("# flagged: regenerate 2026-07-11T07:00:00Z")
        );
        assert!(
            parsed.chain_valid(),
            "advisory must not be part of signed bytes"
        );
    }

    #[test]
    fn malformed_header_errors() {
        assert!(EinmoFile::parse(b"no newline at all").is_err());
        assert!(EinmoFile::parse(b"#wrong 1 encoding=utf-8 separator=X\nbody").is_err());
        assert!(EinmoFile::parse(b"#einmo 9 encoding=utf-8 separator=X\nbody").is_err());
    }

    #[test]
    fn missing_sections_errors() {
        // Header + only one part after → too few sections.
        let bad = b"#einmo 1 encoding=utf-8 separator=X\njust-metadata";
        assert!(EinmoFile::parse(bad).is_err());
    }

    #[test]
    fn cr_in_status_detail_roundtrips() {
        let file = signed_file(
            DEFAULT_SEPARATOR,
            vec![
                Section::new("INPUT", "bad input"),
                Section::new("OUTPUT", ""),
                Section::new("COMMENTS", ""),
            ],
        );
        let mut meta = file.metadata().clone();
        meta.status = Status::OutputError;
        meta.status_detail = "line 1\r\nline 2".into();
        let mut rebuilt = EinmoFile::new(
            file.encoding.clone(),
            file.separator.clone(),
            meta,
            file.sections().to_vec(),
            Stamps::new(),
        );
        let (configured, _) = derive_keypair("cfg");
        let (stage, _) = derive_keypair("");
        rebuilt.set_stamps(Stamps::generate(
            &rebuilt.signed_prefix(),
            &configured,
            &stage,
        ));
        let bytes = rebuilt.serialize().unwrap();
        let parsed = EinmoFile::parse(&bytes).unwrap();
        assert_eq!(parsed.metadata().status_detail, "line 1\r\nline 2");
        assert!(parsed.chain_valid());
    }
}
