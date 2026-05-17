use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use rayon::prelude::*;

use foolish_core::{SequenceableFir, HumanizingSequencer, clone_steppable, FirRef};
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
                write!(f, "PENDING: {} \u{2014} no approved snapshot", name)?;
                if !actual.is_empty() {
                    write!(f, " (actual output: {})", actual)?;
                }
                Ok(())
            }
            TestFailure::Mismatch { name, expected, actual } => {
                write!(
                    f,
                    "MISMATCH: {} \u{2014} output differs from approved snapshot\n  expected: {}\n  actual:   {}",
                    name, expected, actual
                )
            }
            TestFailure::Error { name, message } => {
                write!(f, "ERROR: {} \u{2014} {}", name, message)
            }
        }
    }
}

/// Orchestrator for snapshot-based approval testing of UBCb evaluation.
///
/// Scans `snapshot_tests/input/` for `.foo` source files and compares
/// UBCb evaluation output against approved snapshots in `snapshot_tests/approved/`.
/// Supports parallel evaluation via Rayon with sequential insta assertion (thread-safety).
pub struct SnapshotSuite {
    input_dir: PathBuf,
    approved_dir: PathBuf,
}

impl SnapshotSuite {
    /// Create a new suite for a pair of input/approved directories.
    ///
    /// Discovers `.foo` files in `input_dir` (excluding `.foo.disabled`).
    /// Panics if extraneous approved snapshots exist without matching inputs.
    pub fn new(input_dir: impl Into<PathBuf>, approved_dir: impl Into<PathBuf>) -> Result<Self, SnapshotSuiteError> {
        let input_dir = input_dir.into();
        let approved_dir = approved_dir.into();
        let suite = Self { input_dir, approved_dir };

        if suite.approved_dir.exists() {
            let snapshot_names: HashSet<String> = match fs::read_dir(&suite.approved_dir) {
                Ok(entries) => entries
                    .flatten()
                    .filter(|e| e.path().extension().is_some_and(|ext| ext == "snap"))
                    .filter_map(|e| {
                        e.file_name()
                            .to_str()
                            .and_then(|s| s.strip_suffix(".snap"))
                            .and_then(|s| {
                                let without_foo = s.strip_suffix(".foo").unwrap_or(s);
                                let without_states = without_foo.strip_suffix("_states").unwrap_or(without_foo);
                                Some(without_states.to_string())
                            })
                    })
                    .collect(),
                Err(_) => HashSet::new(),
            };

            let input_names = suite.input_names();
            let mut extraneous = Vec::new();

            for name in &snapshot_names {
                if !input_names.contains(name) {
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

    /// Return sorted list of input names that have no corresponding approved snapshot.
    pub fn get_missing_snapshots(&self) -> Vec<String> {
        let input_names = self.input_names();
        let mut missing = Vec::new();

        for name in &input_names {
            let snap_path = self.approved_dir.join(format!("{}.foo.snap", name));
            if !snap_path.exists() {
                missing.push(name.clone());
            }
        }
        missing.sort();
        missing
    }

    /// Return sorted list of approved snapshot names that have no corresponding input.
    pub fn get_missing_inputs(&self) -> Vec<String> {
        let input_names = self.input_names();
        let mut missing = Vec::new();

        if let Ok(entries) = fs::read_dir(&self.approved_dir) {
            for entry in entries.flatten() {
                let fname = entry.file_name().to_string_lossy().to_string();
                if let Some(rest) = fname.strip_suffix(".snap")
                    .and_then(|s| s.strip_suffix(".foo"))
                {
                    let normal_name = rest.strip_suffix("_states").unwrap_or(rest);
                    if !input_names.contains(normal_name) {
                        missing.push(normal_name.to_string());
                    }
                }
            }
        }
        missing.sort();
        missing
    }
}

/// Format an EvaluationResult as a multiline string using HumanizingSequencer.
pub fn format_result(result: &EvaluationResult, states: bool) -> String {
    if result.statements.is_empty() {
        return "{}".to_string();
    }

    let stmts: Vec<String> = result.statements.iter()
        .map(|s| format!("  {};", fmt_stmt(s, 2, states)))
        .collect();

    format!("\n{}\n}}", stmts.join("\n"))
}

/// Format a single statement result using HumanizingSequencer.
fn fmt_stmt(stmt: &StatementResult, indent: usize, states: bool) -> String {
    let value = fmt_fir_inline(&stmt.fir, indent, states);
    match &stmt.name {
        Some(name) => format!("{name} = {value}"),
        None => value,
    }
}

/// Format a FIR inline using HumanizingSequencer.
fn fmt_fir_inline(fir: &FirRef, indent: usize, states: bool) -> String {
    let seq = SequenceableFir::from(clone_steppable(fir));
    let sequencer = HumanizingSequencer::new(seq);
    let output = sequencer.format_with_indent(indent);
    if states {
        format!("{} [{}]", output, fir.borrow().state())
    } else {
        output
    }
}

#[cfg(test)]
mod approval_tests {
    use super::*;

    fn suite() -> SnapshotSuite {
        SnapshotSuite::new(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("snapshot_tests").join("input"),
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("snapshot_tests").join("approved"),
        ).expect("SnapshotSuite initialization failed")
    }

    #[test]
    fn approval_all() {
        let evaluations = suite().evaluate_all(num_cpus::get(), false);
        let mut settings = insta::Settings::clone_current();
        settings.set_snapshot_path(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("snapshot_tests").join("approved"),
        );
        settings.set_prepend_module_to_snapshot(false);
        settings.set_omit_expression(true);
        settings.bind(|| {
            for (name, result) in evaluations {
                match result {
                    Ok(output) => {
                        insta::assert_snapshot!(format!("{}.foo", name), output);
                    }
                    Err(msg) => {
                        panic!("Evaluation error for {}: {}", name, msg);
                    }
                }
            }
        });
    }

    #[test]
    fn approval_all_states() {
        let evaluations = suite().evaluate_all(num_cpus::get(), true);
        let mut settings = insta::Settings::clone_current();
        settings.set_snapshot_path(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("snapshot_tests").join("approved"),
        );
        settings.set_prepend_module_to_snapshot(false);
        settings.set_omit_expression(true);
        settings.bind(|| {
            for (name, result) in evaluations {
                match result {
                    Ok(output) => {
                        insta::assert_snapshot!(format!("{}_states.foo", name), output);
                    }
                    Err(msg) => {
                        panic!("Evaluation error for {}: {}", name, msg);
                    }
                }
            }
        });
    }
}
