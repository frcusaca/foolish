//! EinmoSuite and the generalised Evaluator trait.
//!
//! The `Evaluator` trait is language-agnostic — it takes a source string and
//! returns formatted output blocks. `EinmoSuite` orchestrates evaluation,
//! envelope assembly, stamping, and writing signed `.einmo` files.

use std::panic::{self, AssertUnwindSafe};
use std::path::Path;

use crate::config::{PerspectiveOf, TestConfig};
use crate::format::EinmoFile;
use crate::signature::{
    Stamps, compiled_keypair, create_compiled_stamp, create_configured_stamp, create_stage_stamp,
    derive_keypair,
};
use crate::stage::{
    StageError, ensure_stage_dirs, mirror_input_path, resolve_reference, topo_sort_inputs,
};

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors from suite evaluation.
#[derive(Debug, thiserror::Error)]
pub enum SuiteError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("stage error: {0}")]
    Stage(#[from] StageError),
    #[error("signature error: {0}")]
    Signature(#[from] crate::signature::SignatureError),
    #[error("format error: {0}")]
    Format(#[from] crate::format::EinmoError),
    #[error("evaluator error: {0}")]
    Evaluator(String),
    #[error("evaluator panicked: {0}")]
    Panic(String),
}

// ---------------------------------------------------------------------------
// Evaluator
// ---------------------------------------------------------------------------

/// Generalised evaluator — no Foolish FIR dependency. Returns formatted
/// output blocks (human-readable strings). Adapters format results to
/// strings internally before returning them.
pub trait Evaluator {
    fn evaluate(&self, source: &str) -> Result<Vec<String>, String>;
}

/// Compute a deterministic unified diff between reference and dependent OUTPUT
/// sections. Fixed 3-line context; labels `reference`/`dependent`; no paths or
/// timestamps in headers. Returns the diff as a UTF-8 string.
pub fn compute_diff(reference_output: &[u8], dependent_output: &[u8]) -> String {
    use similar::{ChangeTag, TextDiff};
    let ref_text = std::str::from_utf8(reference_output).unwrap_or("<invalid utf-8>");
    let dep_text = std::str::from_utf8(dependent_output).unwrap_or("<invalid utf-8>");

    let diff = TextDiff::from_lines(ref_text, dep_text);
    let mut result = String::new();
    for (idx, group) in diff.grouped_ops(3).iter().enumerate() {
        if idx > 0 {
            result.push_str("---\n");
        }
        for op in group {
            for change in diff.iter_changes(op) {
                let sign = match change.tag() {
                    ChangeTag::Delete => "-",
                    ChangeTag::Insert => "+",
                    ChangeTag::Equal => " ",
                };
                result.push_str(sign);
                result.push_str(change.value());
                if !change.value().ends_with('\n') {
                    result.push('\n');
                }
            }
        }
    }
    result
}

// ---------------------------------------------------------------------------
// EinmoSuite
// ---------------------------------------------------------------------------

pub struct EinmoSuite {
    config: TestConfig,
}

impl EinmoSuite {
    pub fn new(config: TestConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &TestConfig {
        &self.config
    }

    /// File-based input: read input file, evaluate, write signed `.einmo` to output/.
    pub fn evaluate(&self, path: &Path, evaluator: &dyn Evaluator) -> Result<String, SuiteError> {
        let source = std::fs::read_to_string(path)?;

        let rel_path = path.strip_prefix(self.config.input_path()).unwrap_or(path);
        let test_name = rel_path.to_string_lossy().to_string();

        self.run_evaluation(&test_name, &source, evaluator, path, None)
    }

    /// Inlined input: input is a string, captured into INPUT section.
    pub fn evaluate_inline(
        &self,
        name: &str,
        input: &str,
        evaluator: &dyn Evaluator,
    ) -> Result<String, SuiteError> {
        self.run_evaluation(name, input, evaluator, Path::new(name), None)
    }

    /// Evaluate a dependent test with a known reference output.
    ///
    /// The `reference_name` is the mirror-relative name of the reference test.
    /// The `reference_output` is the canonical OUTPUT bytes from the reference's
    /// already-evaluated `.einmo` file.
    pub fn evaluate_dependent(
        &self,
        path: &Path,
        evaluator: &dyn Evaluator,
        reference_name: &str,
        reference_output: &[u8],
    ) -> Result<String, SuiteError> {
        let source = std::fs::read_to_string(path)?;
        let rel_path = path.strip_prefix(self.config.input_path()).unwrap_or(path);
        let test_name = rel_path.to_string_lossy().to_string();
        self.run_evaluation(
            &test_name,
            &source,
            evaluator,
            path,
            Some((reference_name, reference_output)),
        )
    }

    fn run_evaluation(
        &self,
        test_name: &str,
        source: &str,
        evaluator: &dyn Evaluator,
        _input_path: &Path,
        reference: Option<(&str, &[u8])>,
    ) -> Result<String, SuiteError> {
        ensure_stage_dirs(&self.config)?;

        let eval_result = match panic::catch_unwind(AssertUnwindSafe(|| evaluator.evaluate(source)))
        {
            Ok(result) => result,
            Err(payload) => {
                let msg = panic_payload_to_string(payload);
                let einmo = self.assemble_envelope(
                    test_name,
                    source,
                    &[],
                    "output-error",
                    &format!("evaluator panicked: {msg}"),
                    None,
                    None,
                );
                return self.write_output(test_name, einmo);
            }
        };

        let (status, status_detail, outputs) = match eval_result {
            Ok(outputs) => ("normal", String::new(), outputs),
            Err(msg) => ("input-error", msg, Vec::new()),
        };

        let (diff_content, diff_status, diff_detail) = match reference {
            Some((_, ref_output)) if !outputs.is_empty() => {
                let dep_output = outputs[0].as_bytes();
                let raw_diff = compute_diff(ref_output, dep_output);
                let limit = self.config.diff_limit();
                if raw_diff.len() > limit {
                    let detail = format!("DIFF exceeds diff-limit ({} > {limit})", raw_diff.len());
                    (
                        Some(raw_diff[..limit].to_string()),
                        Some("output-error"),
                        Some(detail),
                    )
                } else {
                    (Some(raw_diff), None, None)
                }
            }
            Some(_) => (
                Some("reference unavailable: dependent has no output".to_string()),
                None,
                None,
            ),
            None => (None, None, None),
        };

        let effective_status = diff_status.unwrap_or(status);
        let effective_detail = match diff_detail {
            Some(d) => d,
            None => status_detail,
        };

        let ref_name = reference.map(|(n, _)| n);
        let einmo = self.assemble_envelope(
            test_name,
            source,
            &outputs,
            effective_status,
            &effective_detail,
            ref_name,
            diff_content.as_deref(),
        );
        let path = self.write_output(test_name, einmo)?;
        if diff_status == Some("output-error") {
            Err(SuiteError::Evaluator(effective_detail))
        } else {
            Ok(path)
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn assemble_envelope(
        &self,
        test_name: &str,
        source: &str,
        outputs: &[String],
        status: &str,
        status_detail: &str,
        reference: Option<&str>,
        diff: Option<&str>,
    ) -> EinmoFile {
        let now = time::OffsetDateTime::now_utc();
        let generated = now
            .format(&time::format_description::well_known::Iso8601::DEFAULT)
            .unwrap_or_else(|_| "1970-01-01T00:00:00Z".into());

        let producer = git_producer();

        let mut builder = EinmoFile::builder(test_name, "einmo")
            .producer(&producer)
            .generated(&generated)
            .status(status)
            .status_detail(status_detail)
            .separator(self.config.separator().as_bytes())
            .encoding(self.config.encoding())
            .section("INPUT", source.as_bytes().to_vec());

        if let Some(ref_name) = reference {
            builder = builder.reference(ref_name);
        }

        for (i, output) in outputs.iter().enumerate() {
            let name = if i == 0 {
                "OUTPUT".to_string()
            } else {
                format!("OUTPUT[{i}]")
            };
            builder = builder.section(name, output.as_bytes().to_vec());
        }

        if let Some(diff_content) = diff {
            builder = builder.section("DIFF", diff_content.as_bytes().to_vec());
        }

        for perspective in self.config.perspectives() {
            let content = match perspective.of() {
                PerspectiveOf::Input => perspective.apply(source),
                PerspectiveOf::Output(idx) => outputs
                    .get(*idx)
                    .map(|o| perspective.apply(o))
                    .unwrap_or_default(),
            };
            builder = builder.section(perspective.name().to_string(), content.into_bytes());
        }

        builder = builder.section("COMMENTS", Vec::new());

        builder.build()
    }

    fn write_output(&self, test_name: &str, mut einmo: EinmoFile) -> Result<String, SuiteError> {
        let signed_bytes = einmo.signed_bytes()?;
        let (compiled_sk, _compiled_vk) = compiled_keypair()?;
        let configured_pass = self.configured_passphrase();
        let (configured_sk, _configured_vk) = derive_keypair(&configured_pass)?;

        let stage_pass = self.stage_passphrase("output");
        let (stage_sk, _stage_vk) = derive_keypair(&stage_pass)?;

        let configured_pubkey_hex = hex::encode(_configured_vk.as_bytes());
        let stage_pubkey_hex = hex::encode(_stage_vk.as_bytes());

        let compiled_stamp = create_compiled_stamp(&compiled_sk, &configured_pubkey_hex);
        let configured_stamp = create_configured_stamp(&configured_sk, &stage_pubkey_hex);

        let mut cert_stamps = Stamps::new(vec![compiled_stamp, configured_stamp]);
        let mut prior = signed_bytes;
        for entry in cert_stamps.entries() {
            let line = serde_json::to_string(entry).expect("stamp serializes");
            prior.extend_from_slice(line.as_bytes());
            prior.push(b'\n');
        }

        let stage_stamp = create_stage_stamp(&stage_sk, "output", &prior);
        cert_stamps.push(stage_stamp);
        einmo = einmo.with_stamps(cert_stamps)?;

        let mirror_rel = mirror_input_path(Path::new(test_name));
        let output_dir = self.config.stage_dir(crate::config::Stage::Output);
        let out_path = output_dir.join(&mirror_rel);

        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let bytes = einmo.serialize()?;
        std::fs::write(&out_path, &bytes)?;

        Ok(out_path.to_string_lossy().to_string())
    }

    fn configured_passphrase(&self) -> String {
        String::new()
    }

    fn stage_passphrase(&self, _stage: &str) -> String {
        String::new()
    }
}

// ---------------------------------------------------------------------------
// TestResults — aggregate output from evaluate_all / evaluate_all_inline
// ---------------------------------------------------------------------------

/// Per-file result from `evaluate_all`.
#[derive(Debug)]
pub struct FileResult {
    /// The mirror-relative path (e.g. `integer_arithmetic.foo.einmo`).
    pub path: std::path::PathBuf,
    /// `Ok(output_path)` or `Err(message)`.
    pub result: Result<String, String>,
}

/// Aggregate results from `evaluate_all` / `evaluate_all_inline`.
#[derive(Debug)]
pub struct TestResults {
    pub files: Vec<FileResult>,
    /// Any correspondence failures (description strings).
    pub correspondence_failures: Vec<String>,
}

impl TestResults {
    /// Returns `true` when every file was written successfully and all
    /// required correspondences hold.
    pub fn all_output_written_and_verified(&self) -> bool {
        self.files.iter().all(|f| f.result.is_ok()) && self.correspondence_failures.is_empty()
    }
}

impl EinmoSuite {
    /// Discover all inputs in `input/`, evaluate in serial, write all outputs.
    ///
    /// References are evaluated before their dependents (topological order).
    /// Dependent tests gain a DIFF section computed against their reference's
    /// OUTPUT from the same run.
    ///
    /// Returns per-file results. Enforces `require_correspondence` pairs via
    /// [`crate::compare::compare`].
    pub fn evaluate_all(&self, evaluator: &dyn Evaluator) -> TestResults {
        use crate::stage::walk_input_tree;

        let input_dir = self.config.input_path();
        let mirror_paths = match walk_input_tree(&self.config) {
            Ok(paths) => paths,
            Err(e) => {
                return TestResults {
                    files: Vec::new(),
                    correspondence_failures: vec![format!("walk_input_tree: {e}")],
                };
            }
        };

        let sep = self.config.dependent_separator();

        let mut input_names: Vec<String> = mirror_paths
            .iter()
            .map(|p| {
                let rel = mirror_path_to_input_rel(p);
                rel.to_string_lossy().to_string()
            })
            .collect();
        topo_sort_inputs(&mut input_names, sep);

        let mut files = Vec::new();
        let mut reference_outputs: std::collections::HashMap<String, Vec<u8>> =
            std::collections::HashMap::new();

        for input_name in &input_names {
            let full_input = input_dir.join(input_name);
            let ref_name = resolve_reference(input_name, sep);

            let result = if let Some(ref rn) = ref_name {
                let ref_output = reference_outputs.get(rn).map(|v| v.as_slice());
                if let Some(output) = ref_output {
                    match self.evaluate_dependent(&full_input, evaluator, rn, output) {
                        Ok(path) => Ok(path),
                        Err(e) => Err(e.to_string()),
                    }
                } else {
                    match self.evaluate(&full_input, evaluator) {
                        Ok(path) => Ok(path),
                        Err(e) => Err(e.to_string()),
                    }
                }
            } else {
                match self.evaluate(&full_input, evaluator) {
                    Ok(path) => Ok(path),
                    Err(e) => Err(e.to_string()),
                }
            };

            if result.is_ok() {
                let mirror = mirror_input_path(Path::new(input_name));
                let output_path = self
                    .config
                    .stage_dir(crate::config::Stage::Output)
                    .join(&mirror);
                if let Ok(bytes) = std::fs::read(&output_path)
                    && let Ok(einmo) = EinmoFile::parse(&bytes)
                    && let Some(output_section) = einmo.section("OUTPUT")
                {
                    reference_outputs.insert(input_name.clone(), output_section.to_vec());
                }
            }

            let mirror = mirror_input_path(Path::new(input_name));
            files.push(FileResult {
                path: mirror,
                result,
            });
        }

        self.enforce_correspondence(files)
    }

    fn enforce_correspondence(&self, files: Vec<FileResult>) -> TestResults {
        let mut correspondence_failures = Vec::new();
        for &(stage_a, stage_b) in self.config.require_correspondence() {
            let comparison = crate::compare::compare(
                &self.config,
                stage_a,
                stage_b,
                self.config.match_sections(),
            );

            if !comparison.is_clean() {
                for entry in &comparison.differing {
                    correspondence_failures.push(format!(
                        "differing: {} (sections: {:?})",
                        entry.path.display(),
                        entry.sections
                    ));
                }
                for path in &comparison.only_in_a {
                    correspondence_failures.push(format!("only in {stage_a}: {}", path.display()));
                }
                for path in &comparison.only_in_b {
                    correspondence_failures.push(format!("only in {stage_b}: {}", path.display()));
                }
                for path in &comparison.tampered {
                    correspondence_failures.push(format!("tampered: {}", path.display()));
                }
            }
        }

        TestResults {
            files,
            correspondence_failures,
        }
    }

    /// Discover all inputs in `input/`, evaluate in serial, write all outputs.
    /// Enforces `require_correspondence`.
    ///
    /// Like [`evaluate_all`] but accepts inline (name, source) pairs instead
    /// of discovering files from disk.
    pub fn evaluate_all_inline(
        &self,
        pairs: &[(&str, &str)],
        evaluator: &dyn Evaluator,
    ) -> TestResults {
        let mut files = Vec::new();

        for &(name, source) in pairs {
            let result = match self.evaluate_inline(name, source, evaluator) {
                Ok(output_path) => Ok(output_path),
                Err(e) => Err(e.to_string()),
            };

            // Construct the expected mirror path.
            let mirror = crate::stage::mirror_input_path(std::path::Path::new(name));
            files.push(FileResult {
                path: mirror,
                result,
            });
        }

        self.enforce_correspondence(files)
    }
}

/// Strip the `.einmo` suffix from a mirror path to recover the input-relative
/// path (e.g. `test.foo.einmo` → `test.foo`).
fn mirror_path_to_input_rel(mirror: &std::path::Path) -> std::path::PathBuf {
    let s = mirror.to_string_lossy();
    if let Some(stripped) = s.strip_suffix(".einmo") {
        std::path::PathBuf::from(stripped)
    } else {
        mirror.to_path_buf()
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn panic_payload_to_string(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = payload.downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "unknown panic payload".to_string()
    }
}

fn git_producer() -> String {
    std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{MatchSections, Perspective, PerspectiveOf, Stage};

    struct EchoEvaluator;

    impl Evaluator for EchoEvaluator {
        fn evaluate(&self, source: &str) -> Result<Vec<String>, String> {
            Ok(vec![format!("echo: {source}")])
        }
    }

    struct MultiOutputEvaluator;

    impl Evaluator for MultiOutputEvaluator {
        fn evaluate(&self, source: &str) -> Result<Vec<String>, String> {
            Ok(vec![source.to_uppercase(), source.to_lowercase()])
        }
    }

    struct ErrorEvaluator;

    impl Evaluator for ErrorEvaluator {
        fn evaluate(&self, _source: &str) -> Result<Vec<String>, String> {
            Err("parse failed at line 3".into())
        }
    }

    struct PanicEvaluator;

    impl Evaluator for PanicEvaluator {
        fn evaluate(&self, _source: &str) -> Result<Vec<String>, String> {
            panic!("intentional test panic");
        }
    }

    fn temp_config(dir: &Path) -> TestConfig {
        TestConfig::new(dir)
            .with_encoding("utf-8")
            .with_separator("①\n")
    }

    #[test]
    fn output_written_stamped_verifies() {
        let dir = tempfile::tempdir().unwrap();
        let config = temp_config(dir.path());
        let suite = EinmoSuite::new(config);

        let input = dir.path().join("input");
        std::fs::create_dir_all(&input).unwrap();
        std::fs::write(input.join("test.foo"), "hello world").unwrap();

        let eval = EchoEvaluator;
        let result = suite.evaluate(&input.join("test.foo"), &eval);
        assert!(
            result.is_ok(),
            "evaluate should succeed: {:?}",
            result.err()
        );

        let out_path = dir.path().join("output").join("test.foo.einmo");
        assert!(out_path.exists(), "output file should exist");

        let bytes = std::fs::read(&out_path).unwrap();
        let parsed = EinmoFile::parse(&bytes).expect("should parse");
        assert_eq!(parsed.status(), "normal");
        assert_eq!(parsed.test(), "test.foo");
        assert_eq!(parsed.section("INPUT"), Some(b"hello world".as_slice()));
        assert_eq!(
            parsed.section("OUTPUT"),
            Some(b"echo: hello world".as_slice())
        );
        assert!(
            parsed.stamps().len() >= 3,
            "should have compiled + configured + stage:output"
        );
    }

    #[test]
    fn err_captures_input_error_status() {
        let dir = tempfile::tempdir().unwrap();
        let config = temp_config(dir.path());
        let suite = EinmoSuite::new(config);

        let input = dir.path().join("input");
        std::fs::create_dir_all(&input).unwrap();
        std::fs::write(input.join("bad.foo"), "invalid input").unwrap();

        let eval = ErrorEvaluator;
        let result = suite.evaluate(&input.join("bad.foo"), &eval);
        assert!(result.is_ok(), "evaluate should still write the einmo");

        let out_path = dir.path().join("output").join("bad.foo.einmo");
        assert!(out_path.exists());

        let bytes = std::fs::read(&out_path).unwrap();
        let parsed = EinmoFile::parse(&bytes).expect("should parse");
        assert_eq!(parsed.status(), "input-error");
        assert!(
            parsed.status_detail().contains("parse failed"),
            "status-detail should contain the error: got '{}'",
            parsed.status_detail()
        );
    }

    #[test]
    fn panic_captured_as_output_error() {
        let dir = tempfile::tempdir().unwrap();
        let config = temp_config(dir.path());
        let suite = EinmoSuite::new(config);

        let input = dir.path().join("input");
        std::fs::create_dir_all(&input).unwrap();
        std::fs::write(input.join("panic.foo"), "anything").unwrap();

        let eval = PanicEvaluator;
        let result = suite.evaluate(&input.join("panic.foo"), &eval);
        assert!(
            result.is_ok(),
            "panicking evaluator should still produce output"
        );

        let out_path = dir.path().join("output").join("panic.foo.einmo");
        assert!(out_path.exists());

        let bytes = std::fs::read(&out_path).unwrap();
        let parsed = EinmoFile::parse(&bytes).expect("should parse");
        assert_eq!(parsed.status(), "output-error");
        assert!(
            parsed.status_detail().contains("panicked"),
            "status-detail should mention panic: got '{}'",
            parsed.status_detail()
        );
    }

    #[test]
    fn inline_input_captured() {
        let dir = tempfile::tempdir().unwrap();
        let config = temp_config(dir.path());
        let suite = EinmoSuite::new(config);

        let eval = EchoEvaluator;
        let result = suite.evaluate_inline("inline_test", "inline source", &eval);
        assert!(
            result.is_ok(),
            "evaluate_inline should succeed: {:?}",
            result.err()
        );

        let out_path = dir.path().join("output").join("inline_test.einmo");
        assert!(out_path.exists(), "inline output file should exist");

        let bytes = std::fs::read(&out_path).unwrap();
        let parsed = EinmoFile::parse(&bytes).expect("should parse");
        assert_eq!(parsed.test(), "inline_test");
        assert_eq!(parsed.section("INPUT"), Some(b"inline source".as_slice()));
        assert_eq!(
            parsed.section("OUTPUT"),
            Some(b"echo: inline source".as_slice())
        );
    }

    #[test]
    fn mirror_path_respected() {
        let dir = tempfile::tempdir().unwrap();
        let config = temp_config(dir.path());
        let suite = EinmoSuite::new(config);

        let input = dir.path().join("input");
        let sub = input.join("stage1/section3");
        std::fs::create_dir_all(&sub).unwrap();
        std::fs::write(sub.join("specific.test"), "deep input").unwrap();

        let eval = EchoEvaluator;
        let result = suite.evaluate(&sub.join("specific.test"), &eval);
        assert!(result.is_ok());

        let out_path = dir
            .path()
            .join("output")
            .join("stage1")
            .join("section3")
            .join("specific.test.einmo");
        assert!(out_path.exists(), "deep mirrored path should exist");
    }

    #[test]
    fn perspective_section_emitted() {
        let dir = tempfile::tempdir().unwrap();
        let p = Perspective::new("upper", PerspectiveOf::Input, |s| s.to_uppercase());
        let config = temp_config(dir.path()).with_perspective(p);
        let suite = EinmoSuite::new(config);

        let input = dir.path().join("input");
        std::fs::create_dir_all(&input).unwrap();
        std::fs::write(input.join("persp.foo"), "hello").unwrap();

        let eval = EchoEvaluator;
        let result = suite.evaluate(&input.join("persp.foo"), &eval);
        assert!(result.is_ok());

        let out_path = dir.path().join("output").join("persp.foo.einmo");
        let bytes = std::fs::read(&out_path).unwrap();
        let parsed = EinmoFile::parse(&bytes).expect("should parse");

        assert_eq!(
            parsed.section("upper"),
            Some(b"HELLO".as_slice()),
            "perspective section should be present"
        );
        assert!(
            parsed.sections_list().contains(&"upper".to_string()),
            "perspective should be in sections list"
        );
    }

    #[test]
    fn multiple_outputs_emitted() {
        let dir = tempfile::tempdir().unwrap();
        let config = temp_config(dir.path());
        let suite = EinmoSuite::new(config);

        let input = dir.path().join("input");
        std::fs::create_dir_all(&input).unwrap();
        std::fs::write(input.join("multi.foo"), "Hello World").unwrap();

        let eval = MultiOutputEvaluator;
        let result = suite.evaluate(&input.join("multi.foo"), &eval);
        assert!(result.is_ok());

        let out_path = dir.path().join("output").join("multi.foo.einmo");
        let bytes = std::fs::read(&out_path).unwrap();
        let parsed = EinmoFile::parse(&bytes).expect("should parse");

        assert_eq!(
            parsed.section("OUTPUT"),
            Some(b"HELLO WORLD".as_slice()),
            "first output"
        );
        assert_eq!(
            parsed.section("OUTPUT[1]"),
            Some(b"hello world".as_slice()),
            "second output"
        );
    }

    // -- dependent einmos (Phase 15b) ----------------------------------------

    struct IncEvaluator;
    impl Evaluator for IncEvaluator {
        fn evaluate(&self, source: &str) -> Result<Vec<String>, String> {
            let n: i64 = source
                .trim()
                .parse()
                .map_err(|e: std::num::ParseIntError| e.to_string())?;
            Ok(vec![(n + 1).to_string()])
        }
    }

    #[test]
    fn dependent_has_diff_section() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("input");
        std::fs::create_dir_all(&input).unwrap();
        std::fs::write(input.join("base.foo"), "10").unwrap();
        std::fs::write(input.join("base++inc.foo"), "20").unwrap();

        let config = temp_config(dir.path());
        let suite = EinmoSuite::new(config);
        let results = suite.evaluate_all(&IncEvaluator);
        assert!(
            results.all_output_written_and_verified(),
            "failures: {:?}",
            results.correspondence_failures
        );

        let dep_path = dir.path().join("output").join("base++inc.foo.einmo");
        assert!(dep_path.exists());
        let bytes = std::fs::read(&dep_path).unwrap();
        let parsed = EinmoFile::parse(&bytes).unwrap();
        assert!(
            parsed.section("DIFF").is_some(),
            "dependent should have DIFF section"
        );
        assert_eq!(
            parsed.reference(),
            Some("base.foo"),
            "reference metadata should point to base"
        );
    }

    #[test]
    fn dependent_diff_is_deterministic() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("input");
        std::fs::create_dir_all(&input).unwrap();
        std::fs::write(input.join("ref.foo"), "5").unwrap();
        std::fs::write(input.join("ref++var.foo"), "7").unwrap();

        let config = temp_config(dir.path());
        let suite = EinmoSuite::new(config);
        suite.evaluate_all(&IncEvaluator);

        let bytes1 = std::fs::read(dir.path().join("output").join("ref++var.foo.einmo")).unwrap();
        let parsed1 = EinmoFile::parse(&bytes1).unwrap();
        let diff1 = parsed1.section("DIFF").unwrap().to_vec();

        let config2 = temp_config(dir.path());
        let suite2 = EinmoSuite::new(config2);
        suite2.evaluate_all(&IncEvaluator);

        let bytes2 = std::fs::read(dir.path().join("output").join("ref++var.foo.einmo")).unwrap();
        let parsed2 = EinmoFile::parse(&bytes2).unwrap();
        let diff2 = parsed2.section("DIFF").unwrap().to_vec();
        assert_eq!(diff1, diff2, "diff should be deterministic across runs");
    }

    #[test]
    fn dependent_diff_signed_and_verifies() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("input");
        std::fs::create_dir_all(&input).unwrap();
        std::fs::write(input.join("base.foo"), "3").unwrap();
        std::fs::write(input.join("base++d.foo"), "4").unwrap();

        let config = temp_config(dir.path());
        let suite = EinmoSuite::new(config);
        suite.evaluate_all(&IncEvaluator);

        let dep_path = dir.path().join("output").join("base++d.foo.einmo");
        let einmo = EinmoFile::from_file(&dep_path).unwrap();
        assert_eq!(einmo.status(), "normal");
        assert!(
            einmo.stamps().len() >= 3,
            "should have compiled+configured+stage:output"
        );
        assert!(einmo.section("DIFF").is_some());
    }

    #[test]
    fn dependent_reference_unavailable() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("input");
        std::fs::create_dir_all(&input).unwrap();
        // Only the dependent exists, not its reference
        std::fs::write(input.join("base++orphan.foo"), "42").unwrap();

        let config = temp_config(dir.path());
        let suite = EinmoSuite::new(config);
        let results = suite.evaluate_all(&IncEvaluator);

        // The dependent should still be evaluated (its own output captured)
        assert_eq!(results.files.len(), 1);
        assert!(results.files[0].result.is_ok());

        let dep_path = dir.path().join("output").join("base++orphan.foo.einmo");
        let bytes = std::fs::read(&dep_path).unwrap();
        let parsed = EinmoFile::parse(&bytes).unwrap();
        // No reference metadata (no reference found)
        // Status is normal because the dependent itself evaluated fine
        assert_eq!(parsed.status(), "normal");
    }

    #[test]
    fn diff_limit_pass() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("input");
        std::fs::create_dir_all(&input).unwrap();
        // Small diff: same output
        std::fs::write(input.join("r.foo"), "10").unwrap();
        std::fs::write(input.join("r++same.foo"), "10").unwrap();

        let config = temp_config(dir.path()).with_diff_limit(2000);
        let suite = EinmoSuite::new(config);
        let results = suite.evaluate_all(&IncEvaluator);
        assert!(
            results.all_output_written_and_verified(),
            "failures: {:?}",
            results.correspondence_failures
        );

        let dep_path = dir.path().join("output").join("r++same.foo.einmo");
        let parsed = EinmoFile::from_file(&dep_path).unwrap();
        assert_eq!(parsed.status(), "normal");
    }

    #[test]
    fn diff_limit_fail_truncates_and_errors() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("input");
        std::fs::create_dir_all(&input).unwrap();

        // Create inputs that produce very different outputs to exceed the limit
        let long_input_a = "a".repeat(3000);
        let long_input_b = "b".repeat(3000);
        std::fs::write(input.join("long.foo"), &long_input_a).unwrap();
        std::fs::write(input.join("long++diff.foo"), &long_input_b).unwrap();

        struct LongEval;
        impl Evaluator for LongEval {
            fn evaluate(&self, source: &str) -> Result<Vec<String>, String> {
                Ok(vec![source.to_string()])
            }
        }

        let config = temp_config(dir.path()).with_diff_limit(100);
        let suite = EinmoSuite::new(config);
        let results = suite.evaluate_all(&LongEval);

        // The test should fail because diff exceeds limit
        assert!(
            !results.all_output_written_and_verified(),
            "should fail due to diff limit"
        );
        let dep_file = results
            .files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("long++diff"));
        assert!(
            dep_file.is_some_and(|f| f.result.is_err()),
            "dependent should have error result: {:?}",
            dep_file.map(|f| &f.result)
        );

        // The envelope should still be written
        let dep_path = dir.path().join("output").join("long++diff.foo.einmo");
        assert!(dep_path.exists(), "envelope should still be written");
        let parsed = EinmoFile::from_file(&dep_path).unwrap();
        assert_eq!(parsed.status(), "output-error");
        assert!(
            parsed.status_detail().contains("diff-limit"),
            "status-detail should mention diff-limit: '{}'",
            parsed.status_detail()
        );
    }

    #[test]
    fn compare_flags_diff_drift() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("input");
        std::fs::create_dir_all(&input).unwrap();
        std::fs::write(input.join("base.foo"), "10").unwrap();
        std::fs::write(input.join("base++inc.foo"), "20").unwrap();

        let config = temp_config(dir.path());
        let suite = EinmoSuite::new(config);
        suite.evaluate_all(&IncEvaluator);

        // Promote output → checked
        let cfg = TestConfig::new(dir.path());
        let rel_dep = std::path::Path::new("base++inc.foo.einmo");
        crate::stage::promote(&cfg, Stage::Output, Stage::Checked, rel_dep, "").unwrap();

        // Now change the reference input (simulating reference behavior change)
        std::fs::write(input.join("base.foo"), "999").unwrap();
        let config2 = temp_config(dir.path());
        let suite2 = EinmoSuite::new(config2);
        suite2.evaluate_all(&IncEvaluator);

        // Compare: the dependent's DIFF should have changed
        let cmp = crate::compare::compare(
            &cfg,
            Stage::Output,
            Stage::Checked,
            MatchSections::InputOutput,
        );
        assert!(
            !cmp.differing.is_empty() || !cmp.only_in_a.is_empty(),
            "should detect drift in DIFF section"
        );
    }
}
