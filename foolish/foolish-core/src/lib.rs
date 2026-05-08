pub mod fir;
pub mod serialization;
pub mod compiler;
pub mod ubc;
pub mod search;
pub mod sequencer;

pub use fir::{Fir, FirRef, Nyes, SearchDirection, StatementFir, StepResult, Steppable, OperatorFir,
    clone_steppable, fir_to_ref};
pub use serialization::{fir_from_json, fir_to_json, FirSerializer, JsonSerializer};
pub use compiler::Compiler;
pub use ubc::{UbcError, Scope, constanic_clone, resolve_to_value, run_to_completion, run_to_completion_with_scope, short_circuit, step_boxed, compute_operator};
pub use sequencer::Sequencer;

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn foop9_binary_compiles_to_operator() {
        let firs = Compiler::compile("{1 + 2}").unwrap();
        // Top-level is a brane, operator is inside the brane's statement body
        if let Fir::NormalBrane(nb) = &firs[0] {
            assert_eq!(nb.statements.len(), 1);
            let body = &nb.statements[0].body;
            assert!(matches!(body.borrow().fir_variant(), "Operator"));
            if let Fir::Operator(inner) = clone_steppable(body) {
                assert_eq!(inner.op, "+");
                assert_eq!(inner.operands.len(), 2);
            }
        } else {
            panic!("Expected NormalBrane");
        }
    }

    #[test]
    fn foop9_unary_compiles_to_operator() {
        let firs = Compiler::compile("{-42}").unwrap();
        if let Fir::NormalBrane(nb) = &firs[0] {
            assert_eq!(nb.statements.len(), 1);
            let body = &nb.statements[0].body;
            if let Fir::Operator(inner) = clone_steppable(body) {
                assert_eq!(inner.op, "-");
                assert_eq!(inner.operands.len(), 1);
            }
        } else {
            panic!("Expected NormalBrane");
        }
    }

    #[test]
    fn foop9_operator_json_roundtrip() {
        let firs = Compiler::compile("{5 + 3}").unwrap();
        let json = fir_to_json(&firs[0]).unwrap();
        let recovered = fir_from_json(&json).unwrap();
        // The recovered FIR should be a brane containing an operator
        if let Fir::NormalBrane(nb) = &recovered {
            assert_eq!(nb.statements.len(), 1);
            assert!(matches!(nb.statements[0].body.borrow().fir_variant(), "Operator"));
        } else {
            panic!("Expected NormalBrane");
        }
        // Verify evaluation works after roundtrip
        let mut ref_fir = fir_to_ref(recovered);
        ubc::run_to_completion(&mut ref_fir).unwrap();
        assert!(ref_fir.borrow().state().is_constanic());
    }

    #[test]
    fn foop9_chained_operators() {
        let firs = Compiler::compile("{1 + 2 * 3}").unwrap();
        // Count operator nodes inside the brane
        let count = count_operators(&firs[0]);
        assert_eq!(count, 2);
    }

    fn count_operators(fir: &Fir) -> usize {
        let mut count = 0;
        match fir {
            Fir::Operator(inner) => {
                count += 1;
                for op in &inner.operands {
                    let op_fir = clone_steppable(op);
                    count += count_operators(&op_fir);
                }
            }
            Fir::NormalBrane(nb) => {
                for stmt in &nb.statements {
                    let body = clone_steppable(&stmt.body);
                    count += count_operators(&body);
                }
            }
            _ => {}
        }
        count
    }

    #[test]
    fn foop9_operator_search_transparency_regression() {
        // #-2 + #-1 must resolve to 12 (search transparency)
        let output = run_foo("{x=5, y=7, z=#-2 + #-1;}");
        assert!(output.contains("Int(12)"), "Expected z=12 but got: {}", output);
    }

    fn run_foo(source: &str) -> String {
        let firs = Compiler::compile(source)
            .unwrap_or_else(|e| panic!("Failed to compile '{}': {}", source, e));
        let mut lines = vec![format!("INPUT: {}", source.lines().next().unwrap_or(source))];
        for (i, fir) in firs.iter().enumerate() {
            lines.push(format!("[{}] PARSED:", i));
            lines.push(Sequencer::format(fir));
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
}

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

    #[test]
    fn foop9_operator_search_transparency() {
        // #-2 + #-1: operators must be transparent to search, not a search boundary.
        // #-1 finds y=7, #-2 finds x=5, result is 12.
        // If operator were a search boundary, #-1 would find #-2, giving 10.
        let output = run_foo("{x=5, y=7, z=#-2 + #-1;}");
        insta::assert_snapshot!("foop9_operator_search_transparency", output);
    }

    #[test]
    fn foop9_unary_operator() {
        // -42 compiles to OperatorFir("-", [Const(42)])
        let output = run_foo("{a=-42;}");
        insta::assert_snapshot!("foop9_unary_operator", output);
    }
}
