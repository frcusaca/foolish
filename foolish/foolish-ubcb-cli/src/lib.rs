use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use rayon::prelude::*;

use foolish_core::{Sequencer, clone_steppable, FirRef};
use foolish_ubcb::{UbcbEngine, EvaluationResult, StatementResult};

/// Error type for SnapshotSuite initialization failures.
#[derive(Debug)]
pub enum SnapshotSuiteError {
    /// Approved output files exist without corresponding input files.
    ExtraneousOutputs { files: Vec<String> },
}

impl std::fmt::Display for SnapshotSuiteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SnapshotSuiteError::ExtraneousOutputs { files } => {
                writeln!(f, "Extraneous approved output files (no matching .foo input):")?;
                for file in files {
                    writeln!(f, "  - {}", file)?;
                }
                write!(f, "Remove these files or add matching .foo inputs.")
            }
        }
    }
}

impl std::error::Error for SnapshotSuiteError {}

/// Failure type for individual snapshot tests.
#[derive(Debug)]
pub enum TestFailure {
    /// No approved snapshot exists yet.
    Pending { name: String, actual: String },
    /// Output differs from approved snapshot.
    Mismatch {
        name: String,
        expected: String,
        actual: String,
    },
    /// Evaluation or I/O error.
    Error { name: String, message: String },
}

impl std::fmt::Display for TestFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestFailure::Pending { name, actual } => {
                write!(f, "PENDING: {} — no approved snapshot", name)?;
                if !actual.is_empty() {
                    write!(f, " (actual output: {})", actual)?;
                }
                Ok(())
            }
            TestFailure::Mismatch { name, expected, actual } => {
                write!(
                    f,
                    "MISMATCH: {} — output differs from approved snapshot\n  expected: {}\n  actual:   {}",
                    name, expected, actual
                )
            }
            TestFailure::Error { name, message } => {
                write!(f, "ERROR: {} — {}", name, message)
            }
        }
    }
}

/// Orchestrator for snapshot-based approval testing of UBCb evaluation.
///
/// Scans `approval_test_input/` for `.foo` source files and compares
/// UBCb evaluation output against insta snapshots. Supports parallel
/// evaluation via Rayon with sequential insta assertion (thread-safety).
pub struct SnapshotSuite {
    input_dir: PathBuf,
}

impl SnapshotSuite {
    /// Create a new suite.
    ///
    /// Discovers `.foo` files in `input_dir` (excluding `.foo.disabled`).
    /// Panics if extraneous insta snapshots exist without matching inputs.
    pub fn new(input_dir: impl Into<PathBuf>) -> Result<Self, SnapshotSuiteError> {
        let input_dir = input_dir.into();
        let suite = Self { input_dir };

        // Validate: no extraneous snapshots
        let snapshots_dir = suite.snapshots_dir();
        if snapshots_dir.exists() {
            let snapshot_names: HashSet<String> = match fs::read_dir(&snapshots_dir) {
                Ok(entries) => entries
                    .flatten()
                    .filter(|e| e.path().extension().is_some_and(|ext| ext == "snap"))
                    .filter_map(|e| {
                        e.file_name()
                            .to_str()
                            .map(|s| s.replace(&format!("{}__approval_tests__", suite.snap_prefix()), ""))
                            .map(|s| s.trim_end_matches(".snap").to_string())
                    })
                    .collect(),
                Err(_) => HashSet::new(),
            };

            let input_names = suite.input_names();
            let mut extraneous = Vec::new();

            for name in &snapshot_names {
                // Check both normal and states variants
                let normal_name = name.trim_end_matches("_states");
                if !input_names.contains(normal_name) {
                    extraneous.push(name.clone());
                }
            }

            if !extraneous.is_empty() {
                extraneous.sort();
                return Err(SnapshotSuiteError::ExtraneousOutputs { files: extraneous });
            }
        }

        Ok(suite)
    }

    fn snap_prefix(&self) -> &str {
        "foolish_ubcb_cli"
    }

    fn snapshots_dir(&self) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("snapshots")
    }

    /// Extract test names from `.foo` files in input directory.
    fn input_names(&self) -> HashSet<String> {
        let mut names = HashSet::new();
        if let Ok(entries) = fs::read_dir(&self.input_dir) {
            for entry in entries.flatten() {
                if let Some(file_name) = entry.file_name().to_str() {
                    // Exclude .foo.disabled files
                    if file_name.ends_with(".foo.disabled") {
                        continue;
                    }
                    if let Some(stem) = file_name.strip_suffix(".foo") {
                        names.insert(stem.to_string());
                    }
                }
            }
        }
        names
    }

    /// Discover and return sorted list of `.foo` input files.
    pub fn discover(&self) -> Vec<PathBuf> {
        let mut files = Vec::new();
        if let Ok(entries) = fs::read_dir(&self.input_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension().and_then(|e| e.to_str())
                    && ext == "foo" && !path.file_name().unwrap_or_default().to_str().unwrap_or("").ends_with(".foo.disabled")
                {
                    files.push(path);
                }
            }
        }
        files.sort();
        files
    }

    /// Evaluate a single `.foo` file and return formatted output.
    pub fn evaluate(&self, path: &Path, with_states: bool) -> Result<String, String> {
        let source = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

        let mut engine = UbcbEngine::new();
        let result = engine.evaluate(&source)
            .map_err(|e| format!("Evaluation failed: {}", e))?;

        Ok(format_result(&result, with_states))
    }

    /// Run all snapshot tests with parallel evaluation.
    ///
    /// Phase 1 (parallel): Evaluate all `.foo` files using Rayon threads.
    /// Returns a vector of (test_name, result) for sequential insta assertion.
    pub fn evaluate_all(&self, threads: usize, with_states: bool) -> Vec<(String, Result<String, String>)> {
        let files = self.discover();

        if files.is_empty() {
            return Vec::new();
        }

        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(threads.max(1))
            .build()
            .expect("Failed to create Rayon thread pool");

        pool.install(|| {
            files.par_iter()
                .map(move |path: &PathBuf| {
                    let name = path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("")
                        .to_string();
                    (name, self.evaluate(path.as_path(), with_states))
                })
                .collect()
        })
    }
}

/// Format an EvaluationResult as a single-line string.
pub fn format_result(result: &EvaluationResult, states: bool) -> String {
    if result.statements.is_empty() {
        return "{}".to_string();
    }

    if result.statements.len() == 1 {
        let fmt = fmt_stmt(&result.statements[0], states);
        return format!("{{{};}}", fmt);
    }

    let stmts: Vec<String> = result.statements.iter()
        .map(|s| fmt_stmt(s, states))
        .collect();

    format!("{{{};}}", stmts.join("; "))
}

/// Format a single statement result.
fn fmt_stmt(stmt: &StatementResult, states: bool) -> String {
    let value = fmt_fir_inline(&stmt.fir, states);
    match &stmt.name {
        Some(name) => format!("{name} = {value}"),
        None => value,
    }
}

/// Format a FIR inline for display.
fn fmt_fir_inline(fir: &FirRef, states: bool) -> String {
    if fir.borrow().fir_variant() == "Search" {
        let pattern = fir.borrow().search_pattern().unwrap_or_default();
        let state = fir.borrow().state();
        let direction = match &pattern {
            p if p.starts_with('~') || p.starts_with("^~") || p.ends_with('~') => "FORWARD",
            _ => "BACKWARD",
        };
        let anchor = fir.borrow().search_anchor_ref();
        let anchor_str = anchor.as_ref().map(fmt_anchor);
        let target = fir.borrow().search_target_ref();
        let target_fmt = target.as_ref().map(|t| fmt_fir_inline(t, states));

        let anchor_part = anchor_str.as_deref()
            .map(|a| format!(", anchor=\"{}\"", a))
            .unwrap_or_default();
        let s = if let Some(ref t) = target_fmt {
            format!("Search(result={}, pattern='{}', direction={}{})",
                t, pattern, direction, anchor_part)
        } else {
            format!("Search(pattern='{}', direction={}{})",
                pattern, direction, anchor_part)
        };

        if states {
            format!("{} [{}]", s, state)
        } else {
            s
        }
    } else {
        let raw = Sequencer::format(&clone_steppable(fir));
        if states { raw } else { strip_nyes_tag(&raw) }
    }
}

/// Format a search anchor for display.
fn fmt_anchor(anchor: &FirRef) -> String {
    let v = anchor.borrow().fir_variant();
    if v == "Search" {
        let p = anchor.borrow().search_pattern().unwrap_or_default();
        p.strip_prefix('^').unwrap_or(&p)
            .strip_suffix('$')
            .unwrap_or(&p)
            .to_string()
    } else if v == "ConstantInt" {
        format!("{}", anchor.borrow().as_int().unwrap_or(0))
    } else {
        let raw = Sequencer::format(&clone_steppable(anchor));
        strip_nyes_tag(&raw)
    }
}

/// Strip the NYES state tag from formatted output.
fn strip_nyes_tag(s: &str) -> String {
    let trimmed = s.trim();
    if let Some(pos) = trimmed.rfind(" [") {
        let before = &trimmed[..pos];
        let after = &trimmed[pos + 2..];
        if after.strip_suffix(']').is_some() {
            return before.trim().to_string();
        }
    }
    s.to_string()
}

#[cfg(test)]
mod approval_tests {
    use super::*;

    fn suite() -> SnapshotSuite {
        SnapshotSuite::new(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("approval_test_input"),
        ).expect("SnapshotSuite initialization failed")
    }

    #[test]
    fn approval_all() {
        let evaluations = suite().evaluate_all(num_cpus::get(), false);
        for (name, result) in evaluations {
            match result {
                Ok(output) => {
                    insta::assert_snapshot!(name.as_str(), output);
                }
                Err(msg) => {
                    panic!("Evaluation error for {}: {}", name, msg);
                }
            }
        }
    }

    #[test]
    fn approval_all_states() {
        let evaluations = suite().evaluate_all(num_cpus::get(), true);
        for (name, result) in evaluations {
            match result {
                Ok(output) => {
                    insta::assert_snapshot!(format!("{}_states", name), output);
                }
                Err(msg) => {
                    panic!("Evaluation error for {}: {}", name, msg);
                }
            }
        }
    }
}
