//! `.einmo` containment envelope parse and serialize (FOOP-54 §4.1–4.6).
//!
//! An `.einmo` file is a header line, a metadata section, body sections separated by a
//! configurable separator, a STAMPS section (JSON lines), and an optional advisory trailer.
//!
//! The separator is configurable per suite (default `①\n`; Foolish suites use `!!\n`).
//! Serialization refuses (hard error) if any section's content contains the separator.

use std::collections::HashMap;

use crate::signature::Stamps;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors from `.einmo` envelope parse/serialize operations.
#[derive(Debug, thiserror::Error)]
pub enum EinmoError {
    #[error("invalid UTF-8 in .einmo file")]
    InvalidUtf8(#[from] std::str::Utf8Error),
    #[error("header line malformed: {0}")]
    HeaderMalformed(String),
    #[error("unsupported format version {0}")]
    UnsupportedVersion(u32),
    #[error("metadata section missing newline after header")]
    MetadataMissingNewline,
    #[error("metadata key '{0}' missing or out of expected order")]
    MetadataKeyMissing(String),
    #[error("metadata key '{0}' appears more than once")]
    MetadataKeyDuplicate(String),
    #[error("duplicate section name '{0}'")]
    DuplicateSection(String),
    #[error("section '{0}' not found")]
    SectionNotFound(String),
    #[error("sections mismatch: declared {declared:?}, found {found:?}")]
    SectionsMismatch {
        declared: Vec<String>,
        found: Vec<String>,
    },
    #[error("separator collision: section '{section}' contains the separator sequence")]
    SeparatorCollision { section: String },
    #[error("missing separator line after header")]
    MissingSeparator,
    #[error("stamps parse error: {0}")]
    StampsParse(#[from] crate::signature::SignatureError),
}

// ---------------------------------------------------------------------------
// EinmoFile
// ---------------------------------------------------------------------------

/// Parsed `.einmo` containment envelope.
///
/// Fields are private; access via `EinmoFileRef` trait methods.
#[derive(Debug)]
pub struct EinmoFile {
    // Header
    format_version: u32,
    encoding: String,
    separator: Vec<u8>,

    // Metadata (kept individually for byte-stable serialization)
    test: String,
    suite: String,
    producer: String,
    producer_diff: Option<String>,
    generated: String,
    status: String,
    status_detail: String,
    reference: Option<String>,
    sections_list: Vec<String>,

    // Body sections (name → raw bytes)
    sections: HashMap<String, Vec<u8>>,

    // STAMPS
    stamps: Stamps,

    // Advisory lines (after STAMPS, before verification)
    advisory_lines: Vec<String>,
}

/// Accessors for `EinmoFile`.
impl EinmoFile {
    /// The `.einmo` format version.
    pub fn format_version(&self) -> u32 {
        self.format_version
    }

    /// The encoding declared in the header.
    pub fn encoding(&self) -> &str {
        &self.encoding
    }

    /// The raw separator bytes (e.g. `①\n` or `!!\n`).
    pub fn separator(&self) -> &[u8] {
        &self.separator
    }

    /// Test name (mirror-relative input path).
    pub fn test(&self) -> &str {
        &self.test
    }

    /// Suite identity.
    pub fn suite(&self) -> &str {
        &self.suite
    }

    /// Commit SHA of the producing tree.
    pub fn producer(&self) -> &str {
        &self.producer
    }

    /// SHA of `git diff` when dirty; `None` when tree is clean.
    pub fn producer_diff(&self) -> Option<&str> {
        self.producer_diff.as_deref()
    }

    /// ISO8601 UTC timestamp of generation.
    pub fn generated(&self) -> &str {
        &self.generated
    }

    /// Evaluation status: `normal`, `input-error`, or `output-error`.
    pub fn status(&self) -> &str {
        &self.status
    }

    /// Status detail (free text; empty when `status` is `normal`).
    pub fn status_detail(&self) -> &str {
        &self.status_detail
    }

    /// Reference test name (mirror-relative name of the reference test).
    ///
    /// Present only for dependent einmos (tests with `++` in their name).
    pub fn reference(&self) -> Option<&str> {
        self.reference.as_deref()
    }

    /// The declared section order.
    pub fn sections_list(&self) -> &[String] {
        &self.sections_list
    }

    /// Get a body section by name (e.g. `"INPUT"`, `"OUTPUT"`, `"COMMENTS"`).
    pub fn section(&self, name: &str) -> Option<&[u8]> {
        self.sections.get(name).map(Vec::as_slice)
    }

    /// The parsed stamps.
    pub fn stamps(&self) -> &Stamps {
        &self.stamps
    }

    /// Advisory lines (`# flagged: <reason> <ISO8601>`), stripped before verification.
    pub fn advisory_lines(&self) -> &[String] {
        &self.advisory_lines
    }

    pub fn with_advisory_line(mut self, line: impl Into<String>) -> Self {
        self.advisory_lines.push(line.into());
        self
    }
}

// ---------------------------------------------------------------------------
// Builder
// ---------------------------------------------------------------------------

impl EinmoFile {
    /// Create a new `EinmoFile` via a builder.
    pub fn builder(test: impl Into<String>, suite: impl Into<String>) -> EinmoFileBuilder {
        EinmoFileBuilder {
            test: test.into(),
            suite: suite.into(),
            producer: String::new(),
            producer_diff: None,
            generated: String::new(),
            status: "normal".into(),
            status_detail: String::new(),
            reference: None,
            sections: HashMap::new(),
            stamps: Stamps::new(Vec::new()),
            advisory_lines: Vec::new(),
            encoding: "utf-8".into(),
            separator: DEFAULT_SEPARATOR.as_bytes().to_vec(),
        }
    }
}

/// Builder for constructing an [`EinmoFile`].
pub struct EinmoFileBuilder {
    test: String,
    suite: String,
    producer: String,
    producer_diff: Option<String>,
    generated: String,
    status: String,
    status_detail: String,
    reference: Option<String>,
    sections: HashMap<String, Vec<u8>>,
    stamps: Stamps,
    advisory_lines: Vec<String>,
    encoding: String,
    separator: Vec<u8>,
}

impl EinmoFileBuilder {
    pub fn producer(mut self, producer: impl Into<String>) -> Self {
        self.producer = producer.into();
        self
    }

    pub fn producer_diff(mut self, diff: impl Into<String>) -> Self {
        self.producer_diff = Some(diff.into());
        self
    }

    pub fn generated(mut self, generated: impl Into<String>) -> Self {
        self.generated = generated.into();
        self
    }

    pub fn status(mut self, status: impl Into<String>) -> Self {
        self.status = status.into();
        self
    }

    pub fn status_detail(mut self, detail: impl Into<String>) -> Self {
        self.status_detail = detail.into();
        self
    }

    pub fn reference(mut self, reference: impl Into<String>) -> Self {
        self.reference = Some(reference.into());
        self
    }

    pub fn encoding(mut self, encoding: impl Into<String>) -> Self {
        self.encoding = encoding.into();
        self
    }

    pub fn separator(mut self, separator: &[u8]) -> Self {
        self.separator = separator.to_vec();
        self
    }

    pub fn section(mut self, name: impl Into<String>, content: Vec<u8>) -> Self {
        self.sections.insert(name.into(), content);
        self
    }

    pub fn stamps(mut self, stamps: Stamps) -> Self {
        self.stamps = stamps;
        self
    }

    pub fn advisory_line(mut self, line: impl Into<String>) -> Self {
        self.advisory_lines.push(line.into());
        self
    }

    /// Build the `EinmoFile`.
    ///
    /// The `sections_list` is derived from the sections added via [`section`],
    /// with STAMPS always last. The order is: sections added in insertion order,
    /// then STAMPS.
    pub fn build(self) -> EinmoFile {
        let mut sections_list: Vec<String> = self
            .sections
            .keys()
            .filter(|k| k.as_str() != "STAMPS")
            .cloned()
            .collect();
        sections_list.push("STAMPS".into());

        let mut sections = self.sections;
        sections.insert("STAMPS".into(), self.stamps.serialize());

        EinmoFile {
            format_version: 1,
            encoding: self.encoding,
            separator: self.separator,
            test: self.test,
            suite: self.suite,
            producer: self.producer,
            producer_diff: self.producer_diff,
            generated: self.generated,
            status: self.status,
            status_detail: self.status_detail,
            reference: self.reference,
            sections_list,
            sections,
            stamps: self.stamps,
            advisory_lines: self.advisory_lines,
        }
    }
}

// ---------------------------------------------------------------------------
// Parse
// ---------------------------------------------------------------------------

/// Default separator: `①` (U+2460) + LF.
pub const DEFAULT_SEPARATOR: &str = "①\n";

impl EinmoFile {
    /// Read and verify a `.einmo` file from disk (verify-on-inspect invariant).
    ///
    /// Reads the file, parses the envelope, and verifies all cryptographic stamps.
    /// Returns `Err` if any stamp fails verification.
    pub fn from_file(path: &std::path::Path) -> Result<Self, EinmoError> {
        let bytes = std::fs::read(path).map_err(|e| {
            EinmoError::HeaderMalformed(format!("I/O error reading {}: {e}", path.display()))
        })?;
        let einmo = Self::parse(&bytes)?;
        let signed_bytes = einmo.signed_bytes()?;
        let (_compiled_sk, compiled_vk) = crate::signature::compiled_keypair()
            .map_err(|e| EinmoError::HeaderMalformed(format!("compiled keypair: {e}")))?;
        let verification =
            crate::signature::verify_all_stamps(&signed_bytes, einmo.stamps(), &compiled_vk);
        for (stamp, status) in &verification {
            if let crate::signature::StampStatus::Invalid(reason) = status {
                return Err(EinmoError::HeaderMalformed(format!(
                    "stamp '{}' invalid: {reason}",
                    stamp.key()
                )));
            }
        }
        Ok(einmo)
    }

    /// Parse a `.einmo` envelope from raw bytes.
    ///
    /// Returns `Err` on format violations. The parser is strict but does not verify
    /// cryptographic stamps — use `verify` for that.
    pub fn parse(bytes: &[u8]) -> Result<Self, EinmoError> {
        let text = std::str::from_utf8(bytes)?;

        // --- Header line (line 1) ---
        let (header, rest) = split_at_first_newline(text)?;

        if !header.starts_with("#einmo ") {
            return Err(EinmoError::HeaderMalformed(
                "expected '#einmo ' prefix".into(),
            ));
        }

        let header_body = &header[7..]; // skip "#einmo "
        let mut format_version = None;
        let mut encoding = None;
        let mut separator_escaped = None;

        for field in header_body.split_whitespace() {
            if let Some(val) = field.strip_prefix("encoding=") {
                encoding = Some(val.to_string());
            } else if let Some(val) = field.strip_prefix("separator=") {
                separator_escaped = Some(val.to_string());
            } else if format_version.is_none() {
                format_version =
                    Some(field.parse::<u32>().map_err(|_| {
                        EinmoError::HeaderMalformed(format!("bad version: {field}"))
                    })?);
            }
        }

        let format_version = format_version
            .ok_or_else(|| EinmoError::HeaderMalformed("missing format version".into()))?;
        if format_version != 1 {
            return Err(EinmoError::UnsupportedVersion(format_version));
        }
        let encoding = encoding.unwrap_or_else(|| "utf-8".into());
        let separator_escaped = separator_escaped.unwrap_or_else(|| "①\\n".into());
        let separator = unescape_separator(&separator_escaped);

        // --- Split remaining content on separator ---
        // rest is everything after the header's trailing newline.
        let rest_bytes = rest.as_bytes();
        let sep_len = separator.len();

        let mut sections_content: Vec<Vec<u8>> = Vec::new();
        let mut pos = 0;

        while pos <= rest_bytes.len() {
            if let Some(found) = find_separator(rest_bytes, &separator, pos) {
                sections_content.push(rest_bytes[pos..found].to_vec());
                pos = found + sep_len;
            } else {
                break;
            }
        }

        // The content after the last separator is the final section (STAMPS).
        let final_content = &rest_bytes[pos..];
        sections_content.push(final_content.to_vec());

        if sections_content.len() < 2 {
            return Err(EinmoError::MissingSeparator);
        }

        // sections_content[0] = metadata, sections_content[1..] = body sections
        let metadata_bytes = &sections_content[0];
        let metadata_str = std::str::from_utf8(metadata_bytes)?;
        let body_sections = &sections_content[1..];

        // --- Parse metadata ---
        let metadata_lines: Vec<&str> = metadata_str.lines().collect();
        let mut m_test = None;
        let mut m_suite = None;
        let mut m_producer = None;
        let mut m_producer_diff = None;
        let mut m_generated = None;
        let mut m_status = None;
        let mut m_status_detail = None;
        let mut m_reference = None;
        let mut m_sections = None;

        for line in &metadata_lines {
            let (key, value) = line.split_once(':').ok_or_else(|| {
                EinmoError::HeaderMalformed(format!("metadata line missing ':': {line}"))
            })?;
            let key = key.trim();
            let value = value.trim();

            match key {
                "test" => {
                    if m_test.is_some() {
                        return Err(EinmoError::MetadataKeyDuplicate("test".into()));
                    }
                    m_test = Some(value.to_string());
                }
                "suite" => {
                    if m_suite.is_some() {
                        return Err(EinmoError::MetadataKeyDuplicate("suite".into()));
                    }
                    m_suite = Some(value.to_string());
                }
                "producer" => {
                    if m_producer.is_some() {
                        return Err(EinmoError::MetadataKeyDuplicate("producer".into()));
                    }
                    m_producer = Some(value.to_string());
                }
                "producer-diff" => {
                    if m_producer_diff.is_some() {
                        return Err(EinmoError::MetadataKeyDuplicate("producer-diff".into()));
                    }
                    m_producer_diff = Some(value.to_string());
                }
                "generated" => {
                    if m_generated.is_some() {
                        return Err(EinmoError::MetadataKeyDuplicate("generated".into()));
                    }
                    m_generated = Some(value.to_string());
                }
                "status" => {
                    if m_status.is_some() {
                        return Err(EinmoError::MetadataKeyDuplicate("status".into()));
                    }
                    m_status = Some(value.to_string());
                }
                "status-detail" => {
                    if m_status_detail.is_some() {
                        return Err(EinmoError::MetadataKeyDuplicate("status-detail".into()));
                    }
                    m_status_detail = Some(value.to_string());
                }
                "reference" => {
                    if m_reference.is_some() {
                        return Err(EinmoError::MetadataKeyDuplicate("reference".into()));
                    }
                    m_reference = Some(value.to_string());
                }
                "sections" => {
                    if m_sections.is_some() {
                        return Err(EinmoError::MetadataKeyDuplicate("sections".into()));
                    }
                    m_sections = Some(
                        value
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect::<Vec<_>>(),
                    );
                }
                _ => {} // ignore unknown keys
            }
        }

        let test = m_test.ok_or_else(|| EinmoError::MetadataKeyMissing("test".into()))?;
        let suite = m_suite.ok_or_else(|| EinmoError::MetadataKeyMissing("suite".into()))?;
        let producer =
            m_producer.ok_or_else(|| EinmoError::MetadataKeyMissing("producer".into()))?;
        let generated =
            m_generated.ok_or_else(|| EinmoError::MetadataKeyMissing("generated".into()))?;
        let status = m_status.ok_or_else(|| EinmoError::MetadataKeyMissing("status".into()))?;
        let status_detail = m_status_detail.unwrap_or_default();
        let sections_list =
            m_sections.ok_or_else(|| EinmoError::MetadataKeyMissing("sections".into()))?;

        if !sections_list.contains(&"STAMPS".to_string()) {
            return Err(EinmoError::HeaderMalformed(
                "sections must include STAMPS".into(),
            ));
        }

        // --- Map body sections ---
        let mut sections: HashMap<String, Vec<u8>> = HashMap::new();
        for (i, sec_name) in sections_list.iter().enumerate() {
            if i < body_sections.len() {
                sections.insert(sec_name.clone(), body_sections[i].clone());
            } else {
                sections.insert(sec_name.clone(), Vec::new());
            }
        }

        // Validate that all declared sections are present
        let mut missing: Vec<String> = Vec::new();
        for sec in &sections_list {
            if !sections.contains_key(sec) {
                missing.push(sec.clone());
            }
        }
        if !missing.is_empty() {
            return Err(EinmoError::SectionsMismatch {
                declared: sections_list.clone(),
                found: sections.keys().cloned().collect(),
            });
        }

        // --- Parse advisory lines from STAMPS content, then strip them ---
        let stamps_raw = sections
            .get("STAMPS")
            .ok_or_else(|| EinmoError::SectionNotFound("STAMPS".into()))?;
        let stamps_content = std::str::from_utf8(stamps_raw).unwrap_or("");
        let advisory_lines: Vec<String> = stamps_content
            .lines()
            .filter(|line| line.starts_with("# flagged:"))
            .map(String::from)
            .collect();

        if !advisory_lines.is_empty() {
            let cleaned: String = stamps_content
                .lines()
                .filter(|line| !line.starts_with("# flagged:"))
                .collect::<Vec<_>>()
                .join("\n");
            sections.insert("STAMPS".to_string(), cleaned.into_bytes());
        }

        let stamps_bytes = sections
            .get("STAMPS")
            .ok_or_else(|| EinmoError::SectionNotFound("STAMPS".into()))?;
        let stamps = Stamps::parse(stamps_bytes)?;

        // Validate actual section count
        let actual_section_count = body_sections.len();
        let expected_section_count = sections_list.len();
        if actual_section_count != expected_section_count {
            let found_names: Vec<String> =
                sections_list[..actual_section_count.min(sections_list.len())].to_vec();
            return Err(EinmoError::SectionsMismatch {
                declared: sections_list.clone(),
                found: found_names,
            });
        }

        Ok(Self {
            format_version,
            encoding,
            separator,
            test,
            suite,
            producer,
            producer_diff: m_producer_diff,
            generated,
            status,
            status_detail,
            reference: m_reference,
            sections_list,
            sections,
            stamps,
            advisory_lines,
        })
    }
}

// ---------------------------------------------------------------------------
// Serialize
// ---------------------------------------------------------------------------

impl EinmoFile {
    /// Serialize the envelope to bytes (LF-only; byte-exact roundtrip with `parse`).
    ///
    /// Returns `Err` if any section's content contains the separator sequence
    /// (the collision rule — the suite must configure a different separator).
    pub fn serialize(&self) -> Result<Vec<u8>, EinmoError> {
        // Check separator collision on all body sections.
        for name in &self.sections_list {
            let content = self.sections.get(name).map(Vec::as_slice).unwrap_or(&[]);
            if find_separator(content, &self.separator, 0).is_some() {
                return Err(EinmoError::SeparatorCollision {
                    section: name.clone(),
                });
            }
        }

        // Also check metadata fields for separator collision.
        let mut meta_check: Vec<&[u8]> = vec![
            self.test.as_bytes(),
            self.suite.as_bytes(),
            self.producer.as_bytes(),
            self.generated.as_bytes(),
            self.status.as_bytes(),
            self.status_detail.as_bytes(),
        ];
        if let Some(ref reference) = self.reference {
            meta_check.push(reference.as_bytes());
        }
        for field_bytes in &meta_check {
            if find_separator(field_bytes, &self.separator, 0).is_some() {
                return Err(EinmoError::SeparatorCollision {
                    section: "metadata".into(),
                });
            }
        }

        let mut out = Vec::new();

        // Header line
        out.extend_from_slice(b"#einmo ");
        out.extend_from_slice(self.format_version.to_string().as_bytes());
        out.extend_from_slice(b" encoding=");
        out.extend_from_slice(self.encoding.as_bytes());
        out.extend_from_slice(b" separator=");
        out.extend_from_slice(escape_separator(&self.separator).as_bytes());
        out.push(b'\n');

        // Metadata section
        write_metadata_line(&mut out, "test", &self.test);
        write_metadata_line(&mut out, "suite", &self.suite);
        write_metadata_line(&mut out, "producer", &self.producer);
        if let Some(ref diff) = self.producer_diff {
            write_metadata_line(&mut out, "producer-diff", diff);
        }
        write_metadata_line(&mut out, "generated", &self.generated);
        write_metadata_line(&mut out, "status", &self.status);
        write_metadata_line(&mut out, "status-detail", &self.status_detail);
        if let Some(ref reference) = self.reference {
            write_metadata_line(&mut out, "reference", reference);
        }
        let sections_csv = self.sections_list.join(", ");
        write_metadata_line(&mut out, "sections", &sections_csv);

        // Body sections
        for name in &self.sections_list {
            out.extend_from_slice(&self.separator);
            let content = self.sections.get(name).map(Vec::as_slice).unwrap_or(&[]);
            out.extend_from_slice(content);
        }

        // Advisory lines (after STAMPS, outside signed content)
        if !self.advisory_lines.is_empty() {
            for line in &self.advisory_lines {
                out.push(b'\n');
                out.extend_from_slice(line.as_bytes());
            }
        }

        Ok(out)
    }

    /// Replace the stamps in this file, updating the STAMPS section content.
    pub fn with_stamps(mut self, stamps: Stamps) -> Result<Self, EinmoError> {
        self.sections
            .insert("STAMPS".to_string(), stamps.serialize());
        self.stamps = stamps;
        Ok(self)
    }

    /// Compute the bytes that are signed (everything up to but not including
    /// the STAMPS section separator).
    pub fn signed_bytes(&self) -> Result<Vec<u8>, EinmoError> {
        // Re-serialize everything before the STAMPS separator.
        // This is the header + metadata + all body sections except STAMPS.
        let stamps_idx = self
            .sections_list
            .iter()
            .position(|s| s == "STAMPS")
            .ok_or_else(|| EinmoError::SectionNotFound("STAMPS".into()))?;

        let mut out = Vec::new();

        // Header
        out.extend_from_slice(b"#einmo ");
        out.extend_from_slice(self.format_version.to_string().as_bytes());
        out.extend_from_slice(b" encoding=");
        out.extend_from_slice(self.encoding.as_bytes());
        out.extend_from_slice(b" separator=");
        out.extend_from_slice(escape_separator(&self.separator).as_bytes());
        out.push(b'\n');

        // Metadata
        write_metadata_line(&mut out, "test", &self.test);
        write_metadata_line(&mut out, "suite", &self.suite);
        write_metadata_line(&mut out, "producer", &self.producer);
        if let Some(ref diff) = self.producer_diff {
            write_metadata_line(&mut out, "producer-diff", diff);
        }
        write_metadata_line(&mut out, "generated", &self.generated);
        write_metadata_line(&mut out, "status", &self.status);
        write_metadata_line(&mut out, "status-detail", &self.status_detail);
        if let Some(ref reference) = self.reference {
            write_metadata_line(&mut out, "reference", reference);
        }
        let sections_csv = self.sections_list.join(", ");
        write_metadata_line(&mut out, "sections", &sections_csv);

        // Body sections up to (but not including) STAMPS
        for (i, name) in self.sections_list.iter().enumerate() {
            if i >= stamps_idx {
                break;
            }
            out.extend_from_slice(&self.separator);
            let content = self.sections.get(name).map(Vec::as_slice).unwrap_or(&[]);
            out.extend_from_slice(content);
        }

        Ok(out)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Split text at the first `\n`, returning (line_without_newline, rest_after_newline).
fn split_at_first_newline(text: &str) -> Result<(&str, &str), EinmoError> {
    match text.find('\n') {
        Some(pos) => Ok((&text[..pos], &text[pos + 1..])),
        None => Err(EinmoError::HeaderMalformed(
            "header line not terminated by newline".into(),
        )),
    }
}
/// Find the first occurrence of `sep` in `data` starting at `start`.
fn find_separator(data: &[u8], sep: &[u8], start: usize) -> Option<usize> {
    if sep.is_empty() {
        return None;
    }
    data[start..]
        .windows(sep.len())
        .position(|w| w == sep)
        .map(|i| start + i)
}
/// Escape a separator for the header line.
///
/// Operates at the char level (not byte level) so multi-byte UTF-8 chars
/// like ① (U+2460) roundtrip correctly.
fn escape_separator(sep: &[u8]) -> String {
    let s = String::from_utf8_lossy(sep);
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\n' => out.push_str("\\n"),
            '\\' => out.push_str("\\\\"),
            _ => out.push(c),
        }
    }
    out
}

/// Unescape a separator from the header line.
///
/// `\\n` → `\n`, `\\\\` → `\\`.
fn unescape_separator(escaped: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(escaped.len());
    let mut chars = escaped.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push(b'\n'),
                Some('\\') => out.push(b'\\'),
                Some(other) => {
                    // Unknown escape: pass through literally
                    out.push(b'\\');
                    for b in other.to_string().as_bytes() {
                        out.push(*b);
                    }
                }
                None => out.push(b'\\'),
            }
        } else {
            for b in c.to_string().as_bytes() {
                out.push(*b);
            }
        }
    }
    out
}

/// Write a metadata line: `key: value\n`.
fn write_metadata_line(out: &mut Vec<u8>, key: &str, value: &str) {
    out.extend_from_slice(key.as_bytes());
    out.extend_from_slice(b": ");
    out.extend_from_slice(value.as_bytes());
    out.push(b'\n');
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signature::{Stamp, Stamps};

    // -----------------------------------------------------------------------
    // Test helpers
    // -----------------------------------------------------------------------

    /// Build a minimal `.einmo` file from parts (default separator).
    fn build_einmo(test: &str, input: &str, output: &str, stamps: &Stamps) -> Vec<u8> {
        build_einmo_with_opts(test, input, output, stamps, DEFAULT_SEPARATOR, None, None)
    }

    /// Build a `.einmo` file with custom separator and optional extras.
    fn build_einmo_with_opts(
        test: &str,
        input: &str,
        output: &str,
        stamps: &Stamps,
        separator: &str,
        advisory: Option<&str>,
        perspectives: Option<&[(&str, &str)]>,
    ) -> Vec<u8> {
        let mut sections = vec!["INPUT", "OUTPUT", "COMMENTS", "STAMPS"];
        if let Some(ps) = perspectives {
            for (name, _) in ps {
                sections.insert(2, name);
            }
        }
        let sections_csv = sections.join(", ");

        let mut out = String::new();

        // Header
        let escaped = escape_separator(separator.as_bytes());
        out.push_str(&format!("#einmo 1 encoding=utf-8 separator={escaped}\n"));

        // Metadata — must match write_metadata_line canonical format ("key: value\n")
        out.push_str(&format!("test: {test}\n"));
        out.push_str("suite: test-suite\n");
        out.push_str("producer: abc123\n");
        out.push_str("generated: 2026-07-03T15:30:45Z\n");
        out.push_str("status: normal\n");
        out.push_str("status-detail: \n");
        out.push_str(&format!("sections: {sections_csv}\n"));

        // Sections
        out.push_str(separator);
        out.push_str(input);
        out.push_str(separator);
        out.push_str(output);
        if let Some(ps) = perspectives {
            for (_, body) in ps {
                out.push_str(separator);
                out.push_str(body);
            }
        }
        out.push_str(separator);
        // empty COMMENTS
        out.push_str(separator);
        out.push_str(&String::from_utf8(stamps.serialize()).unwrap());

        if let Some(adv) = advisory {
            out.push('\n');
            out.push_str(adv);
        }

        out.into_bytes()
    }

    fn make_stamp(key: &str, signs: &str) -> Stamp {
        Stamp::new_for_test(
            key,
            "aabbccdd",
            signs,
            "eeff0011",
            "test",
            "2026-07-03T15:30:45Z",
        )
    }

    fn sample_stamps() -> Stamps {
        Stamps::new(vec![
            make_stamp("compiled", "pubkey:configured"),
            make_stamp("configured", "pubkey:stage:output"),
            make_stamp("stage:output", "prior-bytes"),
        ])
    }

    // -----------------------------------------------------------------------
    // Roundtrip
    // -----------------------------------------------------------------------

    #[test]
    fn parse_roundtrip_byte_exact() {
        let stamps = sample_stamps();
        let original = build_einmo("test1.foo", "hello input", "hello output", &stamps);

        let parsed = EinmoFile::parse(&original).expect("parse should succeed");
        let reserialized = parsed.serialize().expect("serialize should succeed");

        assert_eq!(
            original,
            reserialized,
            "roundtrip must be byte-exact\n--- original ---\n{}\n--- reserialized ---\n{}",
            String::from_utf8_lossy(&original),
            String::from_utf8_lossy(&reserialized),
        );
    }

    #[test]
    fn roundtrip_with_multiple_outputs() {
        let sep = DEFAULT_SEPARATOR;
        let escaped = escape_separator(sep.as_bytes());
        let stamps = sample_stamps();
        let stamps_str = String::from_utf8(stamps.serialize()).unwrap();

        let mut content = String::new();
        content.push_str(&format!("#einmo 1 encoding=utf-8 separator={escaped}\n"));
        content.push_str("test: multi.out\n");
        content.push_str("suite: test-suite\n");
        content.push_str("producer: abc123\n");
        content.push_str("generated: 2026-07-03T15:30:45Z\n");
        content.push_str("status: normal\n");
        content.push_str("status-detail: \n");
        content.push_str("sections: INPUT, OUTPUT, OUTPUT[1], COMMENTS, STAMPS\n");
        content.push_str(sep);
        content.push_str("input body");
        content.push_str(sep);
        content.push_str("output 0");
        content.push_str(sep);
        content.push_str("output 1");
        content.push_str(sep);
        // empty COMMENTS
        content.push_str(sep);
        content.push_str(&stamps_str);
        let bytes = content.into_bytes();

        let parsed = EinmoFile::parse(&bytes).expect("parse should succeed");
        assert_eq!(parsed.section("OUTPUT"), Some(b"output 0".as_slice()));
        assert_eq!(parsed.section("OUTPUT[1]"), Some(b"output 1".as_slice()));
        assert_eq!(parsed.sections_list().len(), 5);

        let reserialized = parsed.serialize().expect("serialize should succeed");
        assert_eq!(bytes, reserialized);
    }

    // -----------------------------------------------------------------------
    // Custom separator
    // -----------------------------------------------------------------------

    #[test]
    fn custom_separator_roundtrip() {
        let stamps = sample_stamps();
        let original = build_einmo_with_opts(
            "custom.test",
            "some input",
            "some output",
            &stamps,
            "!!\n", // Foolish separator
            None,
            None,
        );

        let parsed = EinmoFile::parse(&original).expect("parse should succeed");
        assert_eq!(parsed.separator(), b"!!\n");
        let reserialized = parsed.serialize().expect("serialize should succeed");
        assert_eq!(original, reserialized);
    }

    // -----------------------------------------------------------------------
    // Separator collision
    // -----------------------------------------------------------------------

    #[test]
    fn separator_collision_refuses_serialize() {
        let stamps = sample_stamps();
        let mut sections: HashMap<String, Vec<u8>> = HashMap::new();
        sections.insert("INPUT".into(), b"hello \xe2\x91\xa0\n world".to_vec());
        sections.insert("OUTPUT".into(), b"output".to_vec());
        sections.insert("COMMENTS".into(), Vec::new());
        sections.insert("STAMPS".into(), stamps.serialize());

        let file = EinmoFile {
            format_version: 1,
            encoding: "utf-8".into(),
            separator: b"\xe2\x91\xa0\n".to_vec(),
            test: "collide.test".into(),
            suite: "test-suite".into(),
            producer: "abc123".into(),
            producer_diff: None,
            generated: "2026-07-03T15:30:45Z".into(),
            status: "normal".into(),
            status_detail: String::new(),
            reference: None,
            sections_list: vec![
                "INPUT".into(),
                "OUTPUT".into(),
                "COMMENTS".into(),
                "STAMPS".into(),
            ],
            sections,
            stamps,
            advisory_lines: Vec::new(),
        };

        let result = file.serialize();
        assert!(
            result.is_err(),
            "serialize should refuse on separator collision"
        );
        assert!(matches!(
            result.unwrap_err(),
            EinmoError::SeparatorCollision { .. }
        ));
    }

    // -----------------------------------------------------------------------
    // Missing sections
    // -----------------------------------------------------------------------

    #[test]
    fn missing_section_error() {
        let stamps = sample_stamps();
        let sep = DEFAULT_SEPARATOR;
        let escaped = escape_separator(sep.as_bytes());
        let stamps_str = String::from_utf8(stamps.serialize()).unwrap();

        let mut content = String::new();
        content.push_str(&format!("#einmo 1 encoding=utf-8 separator={escaped}\n"));
        content.push_str("test: missing.test\n");
        content.push_str("suite: test-suite\n");
        content.push_str("producer: abc123\n");
        content.push_str("generated: 2026-07-03T15:30:45Z\n");
        content.push_str("status: normal\n");
        content.push_str("status-detail: \n");
        content.push_str("sections: INPUT, OUTPUT, STAMPS\n");
        content.push_str(sep);
        content.push_str("output body");
        content.push_str(sep);
        content.push_str(&stamps_str);
        let bytes = content.into_bytes();

        let result = EinmoFile::parse(&bytes);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            EinmoError::SectionsMismatch { .. }
        ));
    }

    // -----------------------------------------------------------------------
    // Multiple OUTPUT sections
    // -----------------------------------------------------------------------

    #[test]
    fn multiple_output_sections_roundtrip() {
        let stamps = sample_stamps();
        let sep = DEFAULT_SEPARATOR;
        let escaped = escape_separator(sep.as_bytes());
        let stamps_str = String::from_utf8(stamps.serialize()).unwrap();

        let mut content = String::new();
        content.push_str(&format!("#einmo 1 encoding=utf-8 separator={escaped}\n"));
        content.push_str("test: multi-outputs.test\n");
        content.push_str("suite: test-suite\n");
        content.push_str("producer: abc123\n");
        content.push_str("generated: 2026-07-03T15:30:45Z\n");
        content.push_str("status: normal\n");
        content.push_str("status-detail: \n");
        content.push_str("sections: INPUT, OUTPUT, OUTPUT[1], OUTPUT[2], COMMENTS, STAMPS\n");
        content.push_str(sep);
        content.push_str("input text");
        content.push_str(sep);
        content.push_str("first output");
        content.push_str(sep);
        content.push_str("second output");
        content.push_str(sep);
        content.push_str("third output");
        content.push_str(sep);
        content.push_str(sep);
        content.push_str(&stamps_str);
        let bytes = content.into_bytes();

        let parsed = EinmoFile::parse(&bytes).expect("parse should succeed");
        assert_eq!(parsed.section("OUTPUT"), Some(b"first output".as_slice()));
        assert_eq!(
            parsed.section("OUTPUT[1]"),
            Some(b"second output".as_slice())
        );
        assert_eq!(
            parsed.section("OUTPUT[2]"),
            Some(b"third output".as_slice())
        );

        let reserialized = parsed.serialize().expect("serialize should succeed");
        assert_eq!(bytes, reserialized);
    }

    // -----------------------------------------------------------------------
    // Perspective sections
    // -----------------------------------------------------------------------

    #[test]
    fn perspective_sections_roundtrip() {
        let stamps = sample_stamps();
        let original = build_einmo_with_opts(
            "perspective.test",
            "{a=1,b=2,c=3}",
            "result",
            &stamps,
            DEFAULT_SEPARATOR,
            None,
            Some(&[("names-perspective", "{a=???,b=???,c=???}")]),
        );

        let parsed = EinmoFile::parse(&original).expect("parse should succeed");
        assert_eq!(
            parsed.section("names-perspective"),
            Some(b"{a=???,b=???,c=???}".as_slice())
        );
        assert!(
            parsed
                .sections_list()
                .contains(&"names-perspective".to_string())
        );

        let reserialized = parsed.serialize().expect("serialize should succeed");
        assert_eq!(original, reserialized);
    }

    // -----------------------------------------------------------------------
    // Status / status-detail
    // -----------------------------------------------------------------------

    #[test]
    fn status_roundtrip() {
        let stamps = sample_stamps();
        let sep = DEFAULT_SEPARATOR;
        let escaped = escape_separator(sep.as_bytes());
        let stamps_str = String::from_utf8(stamps.serialize()).unwrap();

        let mut content = String::new();
        content.push_str(&format!("#einmo 1 encoding=utf-8 separator={escaped}\n"));
        content.push_str("test: err.test\n");
        content.push_str("suite: test-suite\n");
        content.push_str("producer: abc123\n");
        content.push_str("generated: 2026-07-03T15:30:45Z\n");
        content.push_str("status: input-error\n");
        content.push_str("status-detail: parse failed at line 5\n");
        content.push_str("sections: INPUT, OUTPUT, COMMENTS, STAMPS\n");
        content.push_str(sep);
        content.push_str("bad input");
        content.push_str(sep);
        content.push_str("error output");
        content.push_str(sep);
        content.push_str(sep);
        content.push_str(&stamps_str);
        let bytes = content.into_bytes();

        let parsed = EinmoFile::parse(&bytes).expect("parse should succeed");
        assert_eq!(parsed.status(), "input-error");
        assert_eq!(parsed.status_detail(), "parse failed at line 5");

        let reserialized = parsed.serialize().expect("serialize should succeed");
        assert_eq!(bytes, reserialized);
    }

    #[test]
    fn empty_status_detail_roundtrip() {
        let stamps = sample_stamps();
        let sep = DEFAULT_SEPARATOR;
        let escaped = escape_separator(sep.as_bytes());
        let stamps_str = String::from_utf8(stamps.serialize()).unwrap();

        let content = format!(
            "#einmo 1 encoding=utf-8 separator={escaped}\n\
             test: empty-detail.test\n\
             suite: test-suite\n\
             producer: abc123\n\
             generated: 2026-07-03T15:30:45Z\n\
             status: normal\n\
             status-detail: \n\
             sections: INPUT, OUTPUT, COMMENTS, STAMPS\n\
             {sep}input\n\
             {sep}output\n\
             {sep}\n\
             {sep}{stamps_str}"
        );
        let bytes = content.into_bytes();

        let parsed = EinmoFile::parse(&bytes).expect("parse should succeed");
        assert_eq!(parsed.status_detail(), "");
        assert_eq!(parsed.status(), "normal");

        let reserialized = parsed.serialize().expect("serialize should succeed");
        assert_eq!(bytes, reserialized);
    }

    // -----------------------------------------------------------------------
    // Advisory line
    // -----------------------------------------------------------------------

    #[test]
    fn advisory_line_roundtrip_and_excluded_from_stamps() {
        let stamps = sample_stamps();
        let original = build_einmo_with_opts(
            "flagged.test",
            "input data",
            "output data",
            &stamps,
            DEFAULT_SEPARATOR,
            Some("# flagged: out of date 2026-07-04T10:00:00Z"),
            None,
        );

        let parsed = EinmoFile::parse(&original).expect("parse should succeed");
        assert_eq!(parsed.advisory_lines().len(), 1);
        assert!(
            parsed.advisory_lines()[0].contains("flagged:"),
            "advisory should contain flagged marker"
        );
        // Stamps themselves should not include the advisory line
        assert_eq!(parsed.stamps().len(), 3);

        let reserialized = parsed.serialize().expect("serialize should succeed");
        assert_eq!(original, reserialized);
    }

    #[test]
    fn advisory_excluded_from_signed_bytes() {
        let stamps = sample_stamps();
        let with_advisory = build_einmo_with_opts(
            "signed.test",
            "input",
            "output",
            &stamps,
            DEFAULT_SEPARATOR,
            Some("# flagged: some reason 2026-07-04T10:00:00Z"),
            None,
        );
        let without_advisory = build_einmo("signed.test", "input", "output", &stamps);

        let parsed_with = EinmoFile::parse(&with_advisory).expect("parse");
        let parsed_without = EinmoFile::parse(&without_advisory).expect("parse");

        let signed_with = parsed_with.signed_bytes().expect("signed_bytes");
        let signed_without = parsed_without.signed_bytes().expect("signed_bytes");

        // Signed bytes should be identical regardless of advisory presence
        assert_eq!(
            signed_with, signed_without,
            "advisory lines must not affect signed bytes"
        );
    }

    // -----------------------------------------------------------------------
    // Header-line errors
    // -----------------------------------------------------------------------

    #[test]
    fn header_missing_prefix() {
        let bytes = b"not an einmo file\nrest\n";
        let result = EinmoFile::parse(bytes);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            EinmoError::HeaderMalformed(_)
        ));
    }

    #[test]
    fn header_missing_newline() {
        let bytes = b"#einmo 1 encoding=utf-8 separator=\\n";
        let result = EinmoFile::parse(bytes);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            EinmoError::HeaderMalformed(_)
        ));
    }

    #[test]
    fn header_bad_version() {
        let bytes = b"#einmo abc encoding=utf-8 separator=\\n\nrest\n";
        let result = EinmoFile::parse(bytes);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            EinmoError::HeaderMalformed(_)
        ));
    }

    #[test]
    fn header_unsupported_version() {
        let bytes = b"#einmo 99 encoding=utf-8 separator=\\n\nrest\n";
        let result = EinmoFile::parse(bytes);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            EinmoError::UnsupportedVersion(99)
        ));
    }

    // -----------------------------------------------------------------------
    // Empty sections
    // -----------------------------------------------------------------------

    #[test]
    fn empty_comments_roundtrip() {
        let stamps = sample_stamps();
        let original = build_einmo("empty-comments.test", "input", "output", &stamps);

        let parsed = EinmoFile::parse(&original).expect("parse should succeed");
        // COMMENTS should be empty
        assert_eq!(parsed.section("COMMENTS"), Some(b"".as_slice()));

        let reserialized = parsed.serialize().expect("serialize should succeed");
        assert_eq!(original, reserialized);
    }

    #[test]
    fn empty_output_roundtrip() {
        let stamps = sample_stamps();
        let sep = DEFAULT_SEPARATOR;
        let escaped = escape_separator(sep.as_bytes());
        let stamps_str = String::from_utf8(stamps.serialize()).unwrap();

        let mut content = String::new();
        content.push_str(&format!("#einmo 1 encoding=utf-8 separator={escaped}\n"));
        content.push_str("test: empty-output.test\n");
        content.push_str("suite: test-suite\n");
        content.push_str("producer: abc123\n");
        content.push_str("generated: 2026-07-03T15:30:45Z\n");
        content.push_str("status: normal\n");
        content.push_str("status-detail: \n");
        content.push_str("sections: INPUT, OUTPUT, COMMENTS, STAMPS\n");
        content.push_str(sep);
        content.push_str("input");
        content.push_str(sep);
        content.push_str(sep);
        content.push_str(sep);
        content.push_str(&stamps_str);
        let bytes = content.into_bytes();

        let parsed = EinmoFile::parse(&bytes).expect("parse should succeed");
        assert_eq!(parsed.section("OUTPUT"), Some(b"".as_slice()));
        assert_eq!(parsed.section("COMMENTS"), Some(b"".as_slice()));

        let reserialized = parsed.serialize().expect("serialize should succeed");
        assert_eq!(bytes, reserialized);
    }

    // -----------------------------------------------------------------------
    // Metadata parsing
    // -----------------------------------------------------------------------

    #[test]
    fn metadata_fields_parsed() {
        let stamps = sample_stamps();
        let original = build_einmo("meta.test", "input", "output", &stamps);

        let parsed = EinmoFile::parse(&original).expect("parse should succeed");
        assert_eq!(parsed.test(), "meta.test");
        assert_eq!(parsed.suite(), "test-suite");
        assert_eq!(parsed.producer(), "abc123");
        assert_eq!(parsed.producer_diff(), None);
        assert_eq!(parsed.generated(), "2026-07-03T15:30:45Z");
        assert_eq!(parsed.format_version(), 1);
        assert_eq!(parsed.encoding(), "utf-8");
    }

    #[test]
    fn producer_diff_roundtrip() {
        let stamps = sample_stamps();
        let sep = DEFAULT_SEPARATOR;
        let escaped = escape_separator(sep.as_bytes());
        let stamps_str = String::from_utf8(stamps.serialize()).unwrap();

        let content = format!(
            "#einmo 1 encoding=utf-8 separator={escaped}\n\
             test: dirty.test\n\
             suite: test-suite\n\
             producer: abc123\n\
             producer-diff: sha256:9f2c8a\n\
             generated: 2026-07-03T15:30:45Z\n\
             status: normal\n\
             status-detail: \n\
             sections: INPUT, OUTPUT, COMMENTS, STAMPS\n\
             {sep}input\n\
             {sep}output\n\
             {sep}\n\
             {sep}{stamps_str}"
        );
        let bytes = content.into_bytes();

        let parsed = EinmoFile::parse(&bytes).expect("parse should succeed");
        assert_eq!(parsed.producer_diff(), Some("sha256:9f2c8a"));

        let reserialized = parsed.serialize().expect("serialize should succeed");
        assert_eq!(bytes, reserialized);
    }

    // -----------------------------------------------------------------------
    // Signed bytes
    // -----------------------------------------------------------------------

    #[test]
    fn signed_bytes_excludes_stamps() {
        let stamps = sample_stamps();
        let original = build_einmo("signed.test", "hello", "world", &stamps);

        let parsed = EinmoFile::parse(&original).expect("parse should succeed");
        let signed = parsed.signed_bytes().expect("signed_bytes should succeed");

        // Signed bytes should NOT contain any stamp JSON
        let signed_str = String::from_utf8_lossy(&signed);
        assert!(
            !signed_str.contains("\"key\":\"compiled\""),
            "signed bytes must not include stamps"
        );
        assert!(
            signed_str.contains("hello"),
            "signed bytes must include INPUT"
        );
        assert!(
            signed_str.contains("world"),
            "signed bytes must include OUTPUT"
        );
    }

    // -----------------------------------------------------------------------
    // Separator escape/unescape
    // -----------------------------------------------------------------------

    #[test]
    fn separator_escape_roundtrip() {
        let cases: &[&[u8]] = &[b"\xe2\x91\xa0\n", b"!!\n", b"\n", b"abc", b"a\\b\n"];
        for &sep in cases {
            let escaped = escape_separator(sep);
            let unescaped = unescape_separator(&escaped);
            assert_eq!(
                sep,
                unescaped.as_slice(),
                "escape/unescape roundtrip failed for {sep:?} (escaped: {escaped:?})"
            );
        }
    }

    // -----------------------------------------------------------------------
    // Stamps parsing
    // -----------------------------------------------------------------------

    #[test]
    fn stamps_parsed_from_envelope() {
        let stamps = sample_stamps();
        let original = build_einmo("stamps.test", "input", "output", &stamps);

        let parsed = EinmoFile::parse(&original).expect("parse should succeed");
        assert_eq!(parsed.stamps().len(), 3);
        assert_eq!(parsed.stamps().entries()[0].key(), "compiled");
        assert_eq!(parsed.stamps().entries()[1].key(), "configured");
        assert_eq!(parsed.stamps().entries()[2].key(), "stage:output");
    }

    // -----------------------------------------------------------------------
    // Duplicate metadata key
    // -----------------------------------------------------------------------

    #[test]
    fn duplicate_metadata_key_errors() {
        let stamps = sample_stamps();
        let sep = DEFAULT_SEPARATOR;
        let escaped = escape_separator(sep.as_bytes());
        let stamps_str = String::from_utf8(stamps.serialize()).unwrap();

        let mut content = String::new();
        content.push_str(&format!("#einmo 1 encoding=utf-8 separator={escaped}\n"));
        content.push_str("test: dup.test\n");
        content.push_str("test: dup2.test\n");
        content.push_str("suite: test-suite\n");
        content.push_str("producer: abc123\n");
        content.push_str("generated: 2026-07-03T15:30:45Z\n");
        content.push_str("status: normal\n");
        content.push_str("status-detail: \n");
        content.push_str("sections: INPUT, OUTPUT, COMMENTS, STAMPS\n");
        content.push_str(sep);
        content.push_str("input");
        content.push_str(sep);
        content.push_str("output");
        content.push_str(sep);
        content.push_str(sep);
        content.push_str(&stamps_str);
        let bytes = content.into_bytes();

        let result = EinmoFile::parse(&bytes);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            EinmoError::MetadataKeyDuplicate(k) if k == "test"
        ));
    }

    // -----------------------------------------------------------------------
    // Missing metadata key
    // -----------------------------------------------------------------------

    #[test]
    fn missing_metadata_key_errors() {
        let stamps = sample_stamps();
        let sep = DEFAULT_SEPARATOR;
        let escaped = escape_separator(sep.as_bytes());
        let stamps_str = String::from_utf8(stamps.serialize()).unwrap();

        let mut content = String::new();
        content.push_str(&format!("#einmo 1 encoding=utf-8 separator={escaped}\n"));
        content.push_str("suite: test-suite\n");
        content.push_str("producer: abc123\n");
        content.push_str("generated: 2026-07-03T15:30:45Z\n");
        content.push_str("status: normal\n");
        content.push_str("status-detail: \n");
        content.push_str("sections: INPUT, OUTPUT, COMMENTS, STAMPS\n");
        content.push_str(sep);
        content.push_str("input");
        content.push_str(sep);
        content.push_str("output");
        content.push_str(sep);
        content.push_str(sep);
        content.push_str(&stamps_str);
        let bytes = content.into_bytes();

        let result = EinmoFile::parse(&bytes);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            EinmoError::MetadataKeyMissing(k) if k == "test"
        ));
    }
}
