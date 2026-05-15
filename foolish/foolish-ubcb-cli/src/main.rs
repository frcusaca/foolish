use std::io::Write;
use std::path::PathBuf;

use anyhow::Context;
use clap::{Parser, Subcommand};
use foolish_core::{Sequencer, clone_steppable, FirRef};
use foolish_ubcb::{UbcbEngine, EvaluationResult};

#[derive(Parser)]
#[command(name = "foolish-ubcb-cli")]
#[command(about = "Foolish UBCb CLI — message-passing brane computer")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Evaluate .foo source and print result
    Run {
        /// Path to .foo source file
        file: PathBuf,
        /// Show NYES states alongside values
        #[arg(long)]
        states: bool,
    },
    /// Interactive REPL
    Repl,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Run { file, states } => cmd_run(&file, states),
        Commands::Repl => cmd_repl(),
    }
}

fn cmd_run(file: &PathBuf, states: bool) -> anyhow::Result<()> {
    let source = std::fs::read_to_string(file)
        .with_context(|| format!("Failed to read {}", file.display()))?;
    let mut engine = UbcbEngine::new();
    let result = engine.evaluate(&source)
        .with_context(|| "Evaluation failed")?;
    for line in format_result(&result, states) {
        println!("{line}");
    }
    Ok(())
}

fn cmd_repl() -> anyhow::Result<()> {
    println!("Foolish UBCb REPL — type {{ to start a brane, evaluated to completion");
    let mut engine = UbcbEngine::new();
    let mut buf = String::new();
    let mut depth = 0i32;
    loop {
        let prompt = if depth > 0 { ".. " } else { "> " };
        print!("{}", prompt);
        std::io::stdout().flush()?;

        let mut line = String::new();
        match std::io::stdin().read_line(&mut line) {
            Ok(0) => { println!(); return Ok(()); }
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => { println!(); return Ok(()); }
            Ok(_) => {}
            Err(e) => return Err(e.into()),
        }

        for c in line.chars() {
            match c {
                '{' => depth += 1,
                '}' => depth -= 1,
                _ => {}
            }
        }
        buf.push_str(&line);

        if depth <= 0 && !buf.trim().is_empty() {
            match engine.evaluate(&buf) {
                Ok(result) => {
                    for line in format_result(&result, false) {
                        println!("=> {line}");
                    }
                }
                Err(e) => eprintln!("Error: {e}"),
            }
            buf.clear();
        }
    }
}

fn format_result(result: &EvaluationResult, states: bool) -> Vec<String> {
    if result.statements.is_empty() {
        return vec!["{}".to_string()];
    }

    if result.statements.len() == 1 {
        let fmt = fmt_stmt(&result.statements[0], states);
        return vec![format!("{{{};}}", fmt)];
    }

    let stmts: Vec<String> = result.statements.iter()
        .map(|s| fmt_stmt(s, states))
        .collect();

    vec![format!("{{{};}}", stmts.join("; "))]
}

fn fmt_stmt(stmt: &foolish_ubcb::StatementResult, states: bool) -> String {
    let value = fmt_fir_inline(&stmt.fir, states);
    match &stmt.name {
        Some(name) => format!("{name} = {value}"),
        None => value,
    }
}

fn fmt_fir_inline(fir: &FirRef, states: bool) -> String {
    if fir.borrow().fir_variant() == "Search" {
        let pattern = fir.borrow().search_pattern().unwrap_or_default();
        let state = fir.borrow().state();
        let direction = match &pattern {
            p if p.starts_with('~') || p.starts_with("^~") || p.ends_with('~') => "FORWARD",
            _ => "BACKWARD",
        };
        let anchor = fir.borrow().search_anchor_ref();
        let anchor_str = anchor.as_ref().map(|a| fmt_anchor(a));
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

// ─────────────────────────────────────────────────────────────────────────────
// Approval Test Framework
// ─────────────────────────────────────────────────────────────────────────────
//
//  What is Approval Testing?
//  -------------------------
//  Approval Testing (also known as Golden Master Testing, Snapshot Testing, or
//  Characterization Testing) is a software testing technique where a system's
//  output is captured, reviewed, and "approved" by a human before being saved
//  as the canonical reference ("golden master").  On every subsequent test run,
//  the system's fresh output is compared byte-for-byte against this approved
//  snapshot to detect unintended regressions.
//
//  Core Functions & Related Names
//  ------------------------------
//  ┌─────────────────────────────────────────────────────────────────────────┐
//  │  ApprovalSuite::new(input_dir, output_dir, input_re, output_re)        │
//  │      Construct a suite from two directories and regex name patterns.   │
//  │                                                                        │
//  │  ApprovalSuite::list_tests() -> SuiteListing                           │
//  │      Discover .foo inputs and .foo.approved outputs, pair them by      │
//  │      name, and report orphaned or extraneous files as warnings.        │
//  │                                                                        │
//  │  ApprovalSuite::run_one(&name, evaluator) -> ApprovalResult            │
//  │      Evaluate a single test: read the input .foo, run the evaluator,   │
//  │      compare against the .foo.approved golden master.                  │
//  │                                                                        │
//  │  ApprovalSuite::run_all(evaluator) -> SuiteResult                      │
//  │      Evaluate every paired input/output across the suite.              │
//  │                                                                        │
//  │  ApprovalSuite::run_pattern(&regex, evaluator) -> SuiteResult          │
//  │      Like run_all but only execute tests whose names match the regex.  │
//  │                                                                        │
//  │  TestStatus::Pass                                                      │
//  │      Output matched the approved golden master exactly.                │
//  │                                                                        │
//  │  TestStatus::Mismatch { expected, actual, received_path }              │
//  │      Output diverged.  The .foo.received file is written so that a     │
//  │      human can diff it against the .foo.approved and decide how to     │
//  │      resolve the discrepancy.                                          │
//  │                                                                        │
//  │  TestStatus::MissingApproved(path)                                     │
//  │      An input exists but has no corresponding golden master yet.       │
//  │                                                                        │
//  │  TestStatus::Error(msg)                                                │
//  │      I/O or evaluator failure.                                         │
//  └─────────────────────────────────────────────────────────────────────────┘
//
//  Workflow (Human-Centric)
//  ------------------------
//  1. RUN    — Execute the code to generate output.
//  2. VERIFY — Inspect the output manually (it must be human-readable).
//  3. APPROVE — Save the verified output as the ".foo.approved" golden master.
//  4. COMPARE — Future test runs compare new output with the approved file.
//
//  Why It Works Well for Foolish
//  ------------------------------
//  Foolish program output is already a serializable text representation of
//  evaluated branes.  A human can read Int(42), Search(pattern='^a$', ...),
//  and brane contents like {{a; b;}} and immediately judge correctness.
//  This makes the golden-master comparison both machine-exact and
//  human-reviewable — the best of both worlds.
//
//  Caveat — "Pouring Concrete"
//  -----------------------------
//  Approval tests treat ANY change as a failure, even intended ones.
//  When refactoring is intentional, review the .foo.received diff carefully
//  and update the .foo.approved file if the new output is correct.

#[cfg(test)]
mod approval {
    use std::path::{Path, PathBuf};
    use std::fs;
    use std::collections::HashSet;
    use regex::Regex;

    /// The core orchestrator for approval (golden-master) testing.
    ///
    /// Scans two directories — `approval_test_input/` for `.foo` source files
    /// and `approval_test_output/` for `.foo.approved` golden masters — pairs
    /// them by name, runs the UBCb evaluator on each input, and compares
    /// output against the approved snapshot.
    pub struct ApprovalSuite {
        input_dir:  PathBuf,
        output_dir: PathBuf,
        input_pattern:  Regex,
        output_pattern: Regex,
    }

    impl ApprovalSuite {
        /// Create a new suite.
        ///
        /// `input_pattern`  — regex to extract test names from `.foo` files
        ///                     (e.g. `(.*?)\\.foo$`)
        /// `output_pattern` — regex to extract test names from `.foo.approved`
        ///                     (e.g. `(.*?)\\.foo\\.approved$`)
        pub fn new(
            input_dir:  impl Into<PathBuf>,
            output_dir: impl Into<PathBuf>,
            input_pattern:  &str,
            output_pattern: &str,
        ) -> Self {
            Self {
                input_dir:  input_dir.into(),
                output_dir: output_dir.into(),
                input_pattern:  Regex::new(input_pattern)
                    .unwrap_or_else(|e| panic!("invalid input pattern: {e}")),
                output_pattern: Regex::new(output_pattern)
                    .unwrap_or_else(|e| panic!("invalid output pattern: {e}")),
            }
        }

        fn extract_name(&self, file_name: &str, re: &Regex) -> Option<String> {
            re.captures(file_name)
                .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
        }

        fn collect_names(&self, dir: &Path, re: &Regex) -> HashSet<String> {
            let mut names = HashSet::new();
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    if let Some(file_name) = entry.file_name().to_str() {
                        if let Some(name) = self.extract_name(file_name, re) {
                            names.insert(name);
                        }
                    }
                }
            }
            names
        }

        /// Scan both directories and return paired, orphaned, and extraneous
        /// test names.  This is the "linting" pass that alerts you when an
        /// input has no approved output (or vice-versa).
        pub fn list_tests(&self) -> SuiteListing {
            let input_names  = self.collect_names(&self.input_dir,  &self.input_pattern);
            let output_names = self.collect_names(&self.output_dir, &self.output_pattern);

            let mut orphan_inputs  = Vec::new();
            let mut orphan_outputs = Vec::new();
            let mut extraneous_outputs = Vec::new();
            let mut paired: Vec<String> = Vec::new();

            for name in &input_names {
                if output_names.contains(name.as_str()) {
                    paired.push(name.clone());
                } else {
                    orphan_inputs.push(name.clone());
                }
            }

            for name in &output_names {
                if input_names.contains(name.as_str()) {
                    continue;
                }
                let file = self.output_dir.join(format!("{name}.foo.approved"));
                let file_name = file
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("");
                if file_name.ends_with(".foo.received") {
                    continue;
                }
                extraneous_outputs.push(name.clone());
            }

            let mut ignored: Vec<String> = Vec::new();
            if let Ok(entries) = fs::read_dir(&self.output_dir) {
                for entry in entries.flatten() {
                    if let Some(file_name) = entry.file_name().to_str() {
                        if !self.output_pattern.is_match(file_name)
                            && !file_name.ends_with(".foo.received")
                        {
                            ignored.push(file_name.to_string());
                        }
                    }
                }
            }

            paired.sort();
            orphan_inputs.sort();
            orphan_outputs.sort();
            extraneous_outputs.sort();
            ignored.sort();

            SuiteListing {
                paired,
                orphan_inputs,
                orphan_outputs,
                extraneous_outputs,
                ignored,
            }
        }

        /// Evaluate a single test by name.
        fn run_single(
            &self,
            name: &str,
            evaluator: impl Fn(&str) -> Vec<String>,
        ) -> ApprovalResult {
            let input_path = self.input_dir.join(format!("{name}.foo"));
            let source = match fs::read_to_string(&input_path) {
                Ok(s) => s,
                Err(e) => {
                    return ApprovalResult {
                        name: name.to_string(),
                        status: TestStatus::Error(format!("Failed to read input: {e}")),
                    };
                }
            };

            let approved_path = self.output_dir.join(format!("{name}.foo.approved"));
            let received_path = self.output_dir.join(format!("{name}.foo.received"));

            let expected = match fs::read_to_string(&approved_path) {
                Ok(s) => s,
                Err(_) => {
                    return ApprovalResult {
                        name: name.to_string(),
                        status: TestStatus::MissingApproved(approved_path.display().to_string()),
                    };
                }
            };

            let actual = evaluator(&source);
            let actual_str = actual.join("\n");

            if expected.trim() == actual_str.trim() {
                let _ = fs::remove_file(&received_path);
                ApprovalResult {
                    name: name.to_string(),
                    status: TestStatus::Pass,
                }
            } else {
                match fs::write(&received_path, &actual_str) {
                    Ok(_) => ApprovalResult {
                        name: name.to_string(),
                        status: TestStatus::Mismatch {
                            expected: expected.trim().to_string(),
                            actual: actual_str.trim().to_string(),
                            received_path: received_path.display().to_string(),
                        },
                    },
                    Err(e) => ApprovalResult {
                        name: name.to_string(),
                        status: TestStatus::Error(format!("Failed to write received: {e}")),
                    },
                }
            }
        }

        /// Run a single named test.  Useful for debugging one failure.
        pub fn run_one(
            &self,
            name: &str,
            evaluator: impl Fn(&str) -> Vec<String>,
        ) -> ApprovalResult {
            self.run_single(name, evaluator)
        }

        /// Run only the tests whose names match the given regex.
        ///
        /// Example: `run_pattern(&Regex::new("search_.*"), evaluator)`
        /// runs `search_constanic_anchored`, `search_resolved_simple`, etc.
        pub fn run_pattern(
            &self,
            pattern: &Regex,
            evaluator: impl Fn(&str) -> Vec<String>,
        ) -> SuiteResult {
            let listing = self.list_tests();
            let mut results: Vec<ApprovalResult> = Vec::new();
            let mut passed = 0;
            let mut failed = 0;

            for name in &listing.paired {
                if !pattern.is_match(name) {
                    continue;
                }
                let result = self.run_single(name, &evaluator);
                match result.status {
                    TestStatus::Pass => passed += 1,
                    _ => failed += 1,
                }
                results.push(result);
            }

            SuiteResult {
                passed,
                failed,
                warnings: listing,
                results,
            }
        }

        /// Run every paired test in the suite.
        pub fn run_all(
            &self,
            evaluator: impl Fn(&str) -> Vec<String>,
        ) -> SuiteResult {
            let listing = self.list_tests();
            let mut results: Vec<ApprovalResult> = Vec::new();
            let mut passed = 0;
            let mut failed = 0;

            for name in &listing.paired {
                let result = self.run_single(name, &evaluator);
                match result.status {
                    TestStatus::Pass => passed += 1,
                    _ => failed += 1,
                }
                results.push(result);
            }

            SuiteResult {
                passed,
                failed,
                warnings: listing,
                results,
            }
        }
    }

    /// Result of a single approval test.
    pub enum TestStatus {
        Pass,
        Mismatch {
            expected:      String,
            actual:        String,
            received_path: String,
        },
        MissingApproved(String),
        Error(String),
    }

    /// One test's outcome.
    pub struct ApprovalResult {
        pub name:   String,
        pub status: TestStatus,
    }

    impl ApprovalResult {
        pub fn is_pass(&self) -> bool {
            matches!(self.status, TestStatus::Pass)
        }
    }

    /// Discovered test file layout (paired vs. orphaned).
    pub struct SuiteListing {
        pub paired:             Vec<String>,
        pub orphan_inputs:      Vec<String>,
        pub orphan_outputs:     Vec<String>,
        pub extraneous_outputs: Vec<String>,
        pub ignored:            Vec<String>,
    }

    impl SuiteListing {
        pub fn has_warnings(&self) -> bool {
            !self.orphan_inputs.is_empty()
                || !self.orphan_outputs.is_empty()
                || !self.extraneous_outputs.is_empty()
        }
    }

    /// Aggregate result of a suite run.
    pub struct SuiteResult {
        pub passed:   usize,
        pub failed:   usize,
        pub warnings: SuiteListing,
        pub results:  Vec<ApprovalResult>,
    }

    impl SuiteResult {
        pub fn is_ok(&self) -> bool {
            self.failed == 0
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Test functions — wired through ApprovalSuite
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod approval_tests {
    use super::approval::ApprovalSuite;

    fn make_suite() -> ApprovalSuite {
        let input_dir  = format!("{}/approval_test_input",  env!("CARGO_MANIFEST_DIR"));
        let output_dir = format!("{}/approval_test_output", env!("CARGO_MANIFEST_DIR"));
        ApprovalSuite::new(
            &input_dir,
            &output_dir,
            r"(.*?)\.foo$",
            r"(.*?)\.foo\.approved$",
        )
    }

    fn make_suite_states() -> ApprovalSuite {
        let input_dir  = format!("{}/approval_test_input", env!("CARGO_MANIFEST_DIR"));
        let output_dir = format!("{}/approval_test_output_states", env!("CARGO_MANIFEST_DIR"));
        ApprovalSuite::new(
            &input_dir,
            &output_dir,
            r"(.*?)\.foo$",
            r"(.*?)\.foo\.approved$",
        )
    }

    fn ubcb_evaluator(states: bool) -> impl Fn(&str) -> Vec<String> {
        move |source: &str| {
            let mut engine = foolish_ubcb::UbcbEngine::new();
            match engine.evaluate(source) {
                Ok(r) => super::format_result(&r, states),
                Err(e) => vec![format!("ERROR: {e}")],
            }
        }
    }

    #[test]
    fn ubcb_approval_suite() {
        let suite = make_suite();
        let result = suite.run_all(ubcb_evaluator(false));

        if result.is_ok() {
            println!("All {} approval tests passed.", result.passed);
        } else {
            let mut msg = format!(
                "\n{} approval test(s) passed, {} failed:\n",
                result.passed, result.failed
            );
            for r in &result.results {
                if !r.is_pass() {
                    msg.push_str(&format!("\nFAIL: {}\n", r.name));
                    match &r.status {
                        super::approval::TestStatus::Mismatch { expected, actual, received_path } => {
                            msg.push_str(&format!("  expected ≠ actual (received: {})\n", received_path));
                            msg.push_str(&format!("  expected:\n    {}\n", expected));
                            msg.push_str(&format!("  actual:\n    {}\n", actual));
                        }
                        super::approval::TestStatus::MissingApproved(path) => {
                            msg.push_str(&format!("  no approved file: {}\n", path));
                        }
                        super::approval::TestStatus::Error(e) => {
                            msg.push_str(&format!("  error: {}\n", e));
                        }
                        super::approval::TestStatus::Pass => unreachable!(),
                    }
                }
            }
            panic!("{msg}");
        }
    }

    #[test]
    fn ubcb_approval_suite_states() {
        let suite = make_suite_states();
        let result = suite.run_all(ubcb_evaluator(true));

        if result.is_ok() {
            println!("All {} states approval tests passed.", result.passed);
        } else {
            let mut msg = format!(
                "\n{} states approval test(s) passed, {} failed:\n",
                result.passed, result.failed
            );
            for r in &result.results {
                if !r.is_pass() {
                    msg.push_str(&format!("\nFAIL: {}\n", r.name));
                    match &r.status {
                        super::approval::TestStatus::Mismatch { expected, actual, received_path } => {
                            msg.push_str(&format!("  expected ≠ actual (received: {})\n", received_path));
                        }
                        super::approval::TestStatus::MissingApproved(path) => {
                            msg.push_str(&format!("  no approved file: {}\n", path));
                        }
                        super::approval::TestStatus::Error(e) => {
                            msg.push_str(&format!("  error: {}\n", e));
                        }
                        super::approval::TestStatus::Pass => unreachable!(),
                    }
                }
            }
            panic!("{msg}");
        }
    }
}
