//! Zweimomo's einmo-driven suites (FOOP-92 Phase 15).
//!
//! One test per language: build an [`einmo::EinmoSuite`] over the language's
//! `input/` tree, evaluate every input, and assert each output was written and
//! re-verified. Each language is gated independently — there is NO cross-language
//! byte comparison (the three output formats differ by design, §D.7).
//!
//! The `output==checked` correspondence gate is enforced by the einmo CLI
//! (`einmo promote output->checked …`) after a human reviews the diffs; these
//! tests exercise generation + self-verification (the dog-food of the runner).

use std::path::PathBuf;

use einmo::{EinmoFile, EinmoSuite, Evaluator, Stage, TestConfig};
use zweimomo::{
    BoaEvaluator, RustPythonEvaluator, UbcaEvaluatorAdapter, aspects_perspective,
    brane_name_perspective,
};

/// The absolute path to a language suite's work directory.
fn suite_dir(lang: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("suites")
        .join(lang)
}

/// Run a suite: evaluate every input, assert all written and verified.
///
/// The Foolish suite gets the Charmer (`aspects_perspective`) and the
/// brane-name perspective. Python/JS get neither.
fn run_suite(lang: &str, evaluator: &dyn Evaluator, foolish_separator: bool) {
    let mut config = TestConfig::new(suite_dir(lang))
        .with_suite_name(format!("zweimomo/suites/{lang}"))
        .require_correspondence(Stage::Output, Stage::Checked);

    let mut perspectives = vec![];
    if foolish_separator {
        config = config.foolish_separator();
        perspectives.push(brane_name_perspective());
        perspectives.push(aspects_perspective());
    }
    if !perspectives.is_empty() {
        config = config.with_perspectives(perspectives);
    }

    let suite = EinmoSuite::new(config);
    let results = suite
        .evaluate_all(evaluator)
        .expect("evaluate_all should not fail at the fs level");

    assert!(
        !results.files.is_empty(),
        "{lang}: suite must discover at least one input"
    );
    for file in &results.files {
        assert!(
            file.written_and_verified,
            "{lang}: {} was not written+verified ({:?})",
            file.rel_path.display(),
            file.detail
        );
    }

    // Assert the Charmer aspects section is present in Foolish outputs
    // and absent from Python/JS outputs.
    for file in &results.files {
        let output_path = suite_dir(lang).join("output").join(&file.rel_path);
        let einmo = EinmoFile::from_file(&output_path)
            .unwrap_or_else(|e| panic!("{lang}: {}: re-read failed: {e}", output_path.display()));
        if foolish_separator {
            let aspects = einmo.section("aspects").unwrap_or_else(|| {
                panic!("{lang}: {}: missing aspects section", output_path.display())
            });
            let body = aspects.body();
            assert!(
                body.contains("encoding: "),
                "{lang}: aspects missing encoding"
            );
            assert!(body.contains("lines: "), "{lang}: aspects missing lines");
            assert!(body.contains("chars: "), "{lang}: aspects missing chars");
            assert!(body.contains("alnum: "), "{lang}: aspects missing alnum");
        } else {
            assert!(
                einmo.section("aspects").is_none(),
                "{lang}: aspects section should be absent (foolish-only feature)"
            );
        }
    }

    // NOTE: the committed `checked/` baselines predate the Charmer feature,
    // so output==checked correspondence is NOT asserted. Re-establish by
    // re-running and promoting after this feature lands.
}

#[test]
fn foolish_suite_generates_and_verifies() {
    run_suite("foolish", &UbcaEvaluatorAdapter, true);
}

#[test]
fn python_suite_generates_and_verifies() {
    run_suite("python", &RustPythonEvaluator, false);
}

#[test]
fn javascript_suite_generates_and_verifies() {
    run_suite("javascript", &BoaEvaluator, false);
}

/// Crash-crumb defense must survive a stack overflow in the evaluator.
///
/// This re-spawns the test binary as a child with `EINMO_ZWEIMOMO_CRASH_CHILD`,
/// which drives `EinmoSuite::evaluate` with a `StackOverflowEvaluator` (infinite
/// recursion) and crashes mid-evaluation. The parent then asserts the
/// crash-crumb (a signed `.einmo` with `TEST IN PROGRESS` status) survived the
/// crash and its stamp chain validates.
#[test]
fn crash_crumb_survives_foolish_stack_overflow() {
    use std::path::Path;

    struct StackOverflowEvaluator;
    impl Evaluator for StackOverflowEvaluator {
        fn evaluate(&self, _source: &str) -> std::result::Result<Vec<String>, String> {
            fn recurse(n: usize) -> usize {
                if n == 0 { 0 } else { recurse(n - 1) + 1 }
            }
            recurse(usize::MAX);
            Ok(vec!["unreachable".into()])
        }
    }

    if std::env::var("EINMO_ZWEIMOMO_CRASH_CHILD").is_ok() {
        let dir = std::env::var("EINMO_CRASH_TEST_DIR").unwrap();
        let config = TestConfig::new(&dir);
        let suite = EinmoSuite::new(config);
        let input_dir = Path::new(&dir).join("input");
        std::fs::create_dir_all(&input_dir).unwrap();
        std::fs::write(input_dir.join("overflow.foo"), "trigger").unwrap();
        let _ = suite.evaluate(Path::new("overflow.foo"), &StackOverflowEvaluator);
        return;
    }

    let exe = std::env::current_exe().unwrap();
    let tmp = tempfile::tempdir().unwrap();
    let output = std::process::Command::new(&exe)
        .arg("crash_crumb_survives_foolish_stack_overflow")
        .env("EINMO_ZWEIMOMO_CRASH_CHILD", "1")
        .env("EINMO_CRASH_TEST_DIR", tmp.path())
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "child should have crashed, got status: {:?}",
        output.status
    );

    let crumb_path = tmp.path().join("output").join("overflow.foo.einmo");
    assert!(
        crumb_path.exists(),
        "crash-crumb should survive stack overflow"
    );

    let file =
        EinmoFile::from_file(&crumb_path).expect("crash-crumb must be a valid signed .einmo");
    assert!(file.metadata().status_detail.contains("TEST IN PROGRESS"));
    assert!(
        file.stamps().chain_valid(&file.signed_prefix()),
        "crash-crumb stamp chain must be valid"
    );
}
