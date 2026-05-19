use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::FirRef;
use crate::sequencer::format_fir_simple_with_indent;

/// Trait for evaluators that can evaluate a Foolish source file.
/// Returns the evaluated FIRs (final brane and statements).
pub trait Evaluator {
    /// Evaluate source code and return the final evaluated FIRs.
    fn evaluate(&self, source: &str) -> Result<Vec<FirRef>, String>;
}

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

/// Orchestrator for snapshot-based approval testing.
///
/// Scans `snapshot_tests/input/` for `.foo` source files and compares
/// evaluation output against approved snapshots in `snapshot_tests/approved/`.
/// Supports parallel evaluation via Rayon with sequential insta assertion (thread-safety).
pub struct SnapshotSuite {
    input_dir: PathBuf,
    approved_dir: PathBuf,
}

impl SnapshotSuite {
    /// Create a new suite for a pair of input/approved directories.
    ///
    /// Discovers `.foo` files in `input_dir` (excluding `.foo.disabled`).
    /// Returns error if extraneous approved snapshots exist without matching inputs.
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

    /// Evaluate a single `.foo` file using the provided evaluator and return formatted output.
    ///
    /// Flow: read source → sign input → evaluate → humanize sequence → format output.
    pub fn evaluate(&self, path: &Path, evaluator: &dyn Evaluator) -> Result<String, String> {
        let source = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

        let firs = evaluator.evaluate(&source)?;
        let sig = crate::signature::sign_input_line(&source);
        let mut lines = vec![sig];
        lines.push(format!("INPUT: {}", source.lines().next().unwrap_or(&source)));
        for (i, fir_ref) in firs.iter().enumerate() {
            let fir = crate::clone_steppable(fir_ref);
            lines.push(format!("[{}] RESULT:", i));
            lines.push(crate::Sequencer::format(&fir));
        }
        Ok(lines.join("\n"))
    }

    /// Run all snapshot tests with parallel evaluation.
    ///
    /// Phase 1 (parallel): Evaluate all `.foo` files using Rayon threads.
    /// Returns a vector of (test_name, result) for sequential insta assertion.
    pub fn evaluate_all(&self, threads: usize, evaluator: &dyn Evaluator) -> Vec<(String, Result<String, String>)> {
        let files = self.discover();

        if files.is_empty() {
            return Vec::new();
        }

        #[cfg(feature = "parallel-snapshot")]
        {
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
                        (name, self.evaluate(path.as_path(), evaluator))
                    })
                    .collect()
            })
        }

        #[cfg(not(feature = "parallel-snapshot"))]
        {
            let _ = threads;
            files.iter()
                .map(|path| {
                    let name = path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("")
                        .to_string();
                    (name, self.evaluate(path.as_path(), evaluator))
                })
                .collect()
        }
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
