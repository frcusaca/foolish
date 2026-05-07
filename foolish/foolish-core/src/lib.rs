pub mod fir;
pub mod serialization;
pub mod compiler;
pub mod ubc;
pub mod search;
pub mod sequencer;

pub use fir::{Fir, FirRef, Nyes, SearchDirection, StatementFir, StepResult, Steppable,
    clone_steppable, fir_to_ref};
pub use serialization::{fir_from_json, fir_to_json, FirSerializer, JsonSerializer};
pub use compiler::Compiler;
pub use ubc::{UbcError, Scope, constanic_clone, resolve_to_value, run_to_completion, run_to_completion_with_scope, short_circuit, step_boxed};
pub use sequencer::Sequencer;

#[cfg(test)]
mod approval_tests {
    use super::*;
    use std::path::Path;

    fn test_resources_dir() -> &'static str {
        env!("CARGO_MANIFEST_DIR")
    }

    fn run_foo(source: &str) -> String {
        let firs = Compiler::compile(source)
            .unwrap_or_else(|e| panic!("Failed to compile '{}': {}", source, e));
        let mut lines = vec![format!("INPUT: {}", source.lines().next().unwrap_or(source))];
        for (i, fir) in firs.iter().enumerate() {
            lines.push(format!("[{}] PARSED:", i));
            lines.push(Sequencer::format(fir));
            // Evaluate
            let mut fir_ref = fir_to_ref(fir.clone());
            let result = ubc::run_to_completion(&mut fir_ref);
            let final_fir = clone_steppable(&fir_ref);
            lines.push("RESULT:".to_string());
            lines.push(Sequencer::format(&final_fir));
            if let Err(e) = result {
                lines.push(format!("ERROR: {}", e));
            }
        }
        lines.join("\n")
    }

    fn approval_test(name: &str) {
        let input_dir = format!("{}/../../test-resources/org/foolish/fvm/inputs", test_resources_dir());
        let input_path = Path::new(&input_dir).join(format!("{}.foo", name));
        let source = std::fs::read_to_string(&input_path)
            .expect(&format!("test input file required: {}", input_path.display()));
        let output = run_foo(&source);
        insta::assert_snapshot!(name, output);
    }

    #[test]
    fn empty_brane() { approval_test("emptyBraneIsApproved"); }

    #[test]
    fn chained_arithmetic() { approval_test("chainedArithmeticIsApproved"); }

    #[test]
    fn anchored_search_on_constant() { approval_test("anchoredSearchOnConstant"); }

    #[test]
    fn scope_resolution() {
        let output = run_foo("{ x = 42; y = x + 8; }");
        insta::assert_snapshot!("scope_resolution", output);
    }

    #[test]
    fn concatenation_basic() {
        let output = run_foo("{ a={p=1;q=2}; b={r=3}; c = a b; }");
        insta::assert_snapshot!("concatenation_basic", output);
    }

    #[test]
    fn concat_shadow_scope() { approval_test("concat_shadow_scope"); }

    #[test]
    fn concat_search_failures() { approval_test("concat_search_failures"); }

    #[test]
    fn sff_basic() {
        let output = run_foo("{a=1,b=2; c=<<a+b>>; c; c;}");
        insta::assert_snapshot!("sff_basic", output);
    }

    #[test]
    fn sff_nested() {
        let output = run_foo("{a=1,b=2; c=<<a+<<b>>>>; c; c;}");
        insta::assert_snapshot!("sff_nested", output);
    }

    #[test]
    fn sf_brane_blocking() {
        let output = run_foo("{a={x=1}; b=<a>; a=42; d=<a>;}");
        insta::assert_snapshot!("sf_brane_blocking", output);
    }

    #[test]
    fn unanchored_seek() {
        let output = run_foo("{a=1,b=2,c=#-1;}");
        insta::assert_snapshot!("unanchored_seek", output);
    }

    #[test]
    fn anchored_seek_positive_negative() {
        // a#1 should get 1 (second element), a#-1 should get 3 (last element)
        let output = run_foo("{a={1,2,3}; a#1; a#-1;}");
        insta::assert_snapshot!("anchored_seek_positive_negative", output);
    }

    #[test]
    fn sff_in_binary_op() {
        // SFF content resolves when used in arithmetic via strip_sf_wrapper
        let output = run_foo("{a=<<3>>; a + 7;}");
        insta::assert_snapshot!("sff_in_binary_op", output);
    }

    #[test]
    fn sf_non_brane_resolves() {
        // SF should resolve non-brane searches normally
        let output = run_foo("{x=42; b=<x>; b;}");
        insta::assert_snapshot!("sf_non_brane_resolves", output);
    }

    #[test]
    fn seek_negative_clamping() {
        // #-99 from stmt 2 in 3-stmt brane: 2+(-99)=-97, clamps to 0 (a=1)
        let output = run_foo("{a=1,b=2; c=#-99;}");
        insta::assert_snapshot!("seek_negative_clamping", output);
    }

    #[test]
    fn seek_beyond_start() {
        // Seek beyond brane start should be NK
        let output = run_foo("{a=1; c=#-99;}");
        insta::assert_snapshot!("seek_beyond_start", output);
    }
}
