pub mod fir;
pub mod serialization;
pub mod compiler;
pub mod ubc;
pub mod search;
pub mod sequencer;
pub mod snapshot_suite;
pub mod signature;

pub use fir::{Fir, FirRef, Nyes, SearchDirection, StatementFir, StepResult, Steppable, OperatorFir,
    clone_steppable, fir_to_ref, FirQueryable, StatementSimple};
pub use serialization::{fir_from_json, fir_to_json, FirSerializer, JsonSerializer};
pub use compiler::Compiler;
pub use ubc::{UbcError, Scope, constanic_clone, resolve_to_value, run_to_completion, run_to_completion_with_scope, short_circuit, step_boxed, compute_operator};
pub use sequencer::{Sequencer, HumanizingSequencerRef};
pub use snapshot_suite::{SnapshotSuite, SnapshotSuiteError, TestFailure, Evaluator};
pub use signature::{derive_keypair, sign_content, verify_signature};

/// UBC evaluator adapter for SnapshotSuite.
///
/// Compiles source via `Compiler`, evaluates to completion with `run_to_completion`,
/// and formats the result using `Sequencer` matching the existing inline test format.
pub struct UbcEvaluator;

impl Evaluator for UbcEvaluator {
    fn evaluate(&self, source: &str) -> Result<String, String> {
        let firs = Compiler::compile(source)
            .map_err(|e| format!("Compilation failed: {}", e))?;
        let mut lines =
            vec![format!("INPUT: {}", source.lines().next().unwrap_or(source))];
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
        Ok(lines.join("\n"))
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use std::rc::Rc;
    use crate::fir::AlarmSink;

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

    // ===== FOOP-12: Alarm System Tests =====

    #[test]
    fn foop12_alarm_level_display() {
        assert_eq!(format!("{}", crate::fir::AlarmLevel::Info), "INFO");
        assert_eq!(format!("{}", crate::fir::AlarmLevel::Warn), "WARN");
        assert_eq!(format!("{}", crate::fir::AlarmLevel::Mild), "MILD");
        assert_eq!(format!("{}", crate::fir::AlarmLevel::Panic), "PANIC");
    }

    #[test]
    fn foop12_alarm_level_serialization() {
        let info = serde_json::to_string(&crate::fir::AlarmLevel::Info).unwrap();
        assert_eq!(info, "\"Info\"");
        let warn: crate::fir::AlarmLevel = serde_json::from_str("\"Warn\"").unwrap();
        assert_eq!(warn, crate::fir::AlarmLevel::Warn);
    }

    #[test]
    fn foop12_alarm_display() {
        let alarm = crate::fir::Alarm {
            level: crate::fir::AlarmLevel::Mild,
            code: "DIV-BY-ZERO".to_string(),
            message: "Division by zero produces NK".to_string(),
            source: crate::fir::AlarmSource::Evaluator,
        };
        let display = format!("{}", alarm);
        assert!(display.contains("[MILD]"));
        assert!(display.contains("DIV-BY-ZERO"));
        assert!(display.contains("Division by zero produces NK"));
    }

    #[test]
    fn foop12_vec_alarm_sink() {
        let sink = crate::fir::VecAlarmSink::new();
        sink.record(crate::fir::Alarm {
            level: crate::fir::AlarmLevel::Info,
            code: "TEST".to_string(),
            message: "Test alarm".to_string(),
            source: crate::fir::AlarmSource::Compiler,
        });
        let alarms = sink.get_alarms();
        assert_eq!(alarms.len(), 1);
        assert_eq!(alarms[0].code, "TEST");
    }

    #[test]
    fn foop12_nkfir_alarm_roundtrip() {
        let fir = Fir::Nk(Box::new(crate::fir::NkFir {
            reason: "division by zero".to_string(),
            state: Nyes::Nk,
            alarm: Some(crate::fir::Alarm {
                level: crate::fir::AlarmLevel::Mild,
                code: "DIV-BY-ZERO".to_string(),
                message: "Division by zero produces NK".to_string(),
                source: crate::fir::AlarmSource::Evaluator,
            }),
        }));
        let json = fir_to_json(&fir).unwrap();
        assert!(json.to_string().contains("DIV-BY-ZERO"));
        let recovered = fir_from_json(&json).unwrap();
        if let Fir::Nk(inner) = &recovered {
            assert!(inner.alarm.is_some());
            assert_eq!(inner.alarm.as_ref().unwrap().code, "DIV-BY-ZERO");
        } else {
            panic!("Expected NkFir");
        }
    }

    #[test]
    fn foop12_division_by_zero_alarm() {
        // Division by zero must produce NK with DIV-BY-ZERO alarm
        let firs = Compiler::compile("{10 / 0}").unwrap();
        let mut fir_ref = fir_to_ref(firs[0].clone());
        ubc::run_to_completion(&mut fir_ref).unwrap();
        // The result should be a brane containing NK with an alarm
        let final_fir = clone_steppable(&fir_ref);
        if let Fir::NormalBrane(nb) = &final_fir {
            let body = clone_steppable(&nb.statements[0].body);
            if let Fir::Nk(inner) = &body {
                assert!(inner.alarm.is_some());
                assert_eq!(inner.alarm.as_ref().unwrap().code, "DIV-BY-ZERO");
            } else {
                panic!("Expected NkFir for division by zero");
            }
        } else {
            panic!("Expected NormalBrane");
        }
    }

    #[test]
    fn foop12_unknown_literal_no_alarm() {
        // ??? literal produces NK without alarm
        let firs = Compiler::compile("{???}").unwrap();
        if let Fir::NormalBrane(nb) = &firs[0] {
            let body = clone_steppable(&nb.statements[0].body);
            if let Fir::Nk(inner) = &body {
                assert!(inner.alarm.is_none());
            } else {
                panic!("Expected NkFir");
            }
        } else {
            panic!("Expected NormalBrane");
        }
    }

    #[test]
    fn foop12_scope_emit_alarm() {
        let sink = Rc::new(crate::fir::VecAlarmSink::new());
        let scope = ubc::Scope::new().with_alarms(sink.clone());
        scope.emit(crate::fir::Alarm {
            level: crate::fir::AlarmLevel::Warn,
            code: "TEST".to_string(),
            message: "Test message".to_string(),
            source: crate::fir::AlarmSource::Compiler,
        });
        let alarms = sink.get_alarms();
        assert_eq!(alarms.len(), 1);
        assert_eq!(alarms[0].level, crate::fir::AlarmLevel::Warn);
    }

    #[test]
    fn foop12_scope_without_sink_no_panic() {
        // Emitting to a scope without a sink should not panic
        let scope = ubc::Scope::new();
        scope.emit(crate::fir::Alarm {
            level: crate::fir::AlarmLevel::Info,
            code: "TEST".to_string(),
            message: "Test message".to_string(),
            source: crate::fir::AlarmSource::Compiler,
        });
        // No panic = test passes
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

    // ===== Arithmetic Tests =====

    #[test]
    fn simple_addition() {
        let output = run_foo("{3 + 4;}");
        insta::assert_snapshot!("simple_addition", output);
    }

    #[test]
    fn simple_subtraction() {
        let output = run_foo("{10 - 3;}");
        insta::assert_snapshot!("simple_subtraction", output);
    }

    #[test]
    fn simple_multiplication() {
        let output = run_foo("{6 * 7;}");
        insta::assert_snapshot!("simple_multiplication", output);
    }

    #[test]
    fn simple_division() {
        let output = run_foo("{15 / 3;}");
        insta::assert_snapshot!("simple_division", output);
    }

    #[test]
    fn zero_division() {
        // Division by zero produces NK with DIV-BY-ZERO alarm
        let output = run_foo("{10 / 0;}");
        insta::assert_snapshot!("zero_division", output);
    }

    #[test]
    fn operator_precedence() {
        // 2 + 3 * 4 - 5: * has higher precedence
        let output = run_foo("{2 + 3 * 4 - 5;}");
        insta::assert_snapshot!("operator_precedence", output);
    }

    #[test]
    fn nested_arithmetic() {
        let output = run_foo("{((2 + 3) * (4 - 1)) / 5;}");
        insta::assert_snapshot!("nested_arithmetic", output);
    }

    #[test]
    fn negative_result() {
        let output = run_foo("{5 - 10;}");
        insta::assert_snapshot!("negative_result", output);
    }

    #[test]
    fn mixed_operators() {
        let output = run_foo("{10 + 5 - 3 * 2;}");
        insta::assert_snapshot!("mixed_operators", output);
    }

    #[test]
    fn simple_integer() {
        let output = run_foo("{5;}");
        insta::assert_snapshot!("simple_integer", output);
    }

    #[test]
    fn multiple_expressions() {
        let output = run_foo("{1; 2; 3;}");
        insta::assert_snapshot!("multiple_expressions", output);
    }

    #[test]
    fn multiple_arithmetic_expressions() {
        let output = run_foo("{5 + 3; 10 - 4; 2 * 6;}");
        insta::assert_snapshot!("multiple_arithmetic_expressions", output);
    }

    #[test]
    fn mixed_expressions() {
        let output = run_foo("{42; (3 + 4) * 2; -15; 100 / 5;}");
        insta::assert_snapshot!("mixed_expressions", output);
    }

    #[test]
    fn single_parenthesized_expression() {
        let output = run_foo("{((((5))));}");
        insta::assert_snapshot!("single_parenthesized_expression", output);
    }

    #[test]
    fn simple_unary_minus() {
        let output = run_foo("{-42;}");
        insta::assert_snapshot!("simple_unary_minus", output);
    }

    // ===== Identifier / Scope Tests =====

    #[test]
    fn simple_identifier() {
        let output = run_foo("{x = 42; x;}");
        insta::assert_snapshot!("simple_identifier", output);
    }

    #[test]
    fn identifier_in_expression() {
        let output = run_foo("{x = 10; y = 20; x + y;}");
        insta::assert_snapshot!("identifier_in_expression", output);
    }

    #[test]
    fn identifier_shadowing() {
        let output = run_foo("{x = 10; x; x = 20; x;}");
        insta::assert_snapshot!("identifier_shadowing", output);
    }

    #[test]
    fn multiple_identifiers() {
        let output = run_foo("{x = 5; y = 3; z = 2; x * y + z;}");
        insta::assert_snapshot!("multiple_identifiers", output);
    }

    #[test]
    fn undeclared_identifier() {
        let output = run_foo("{x = non_existent;}");
        insta::assert_snapshot!("undeclared_identifier", output);
    }

    #[test]
    fn chained_undeclared() {
        // Chain of undeclared references
        let output = run_foo("{bad = undeclared; y = bad; z = y;}");
        insta::assert_snapshot!("chained_undeclared", output);
    }

    // ===== Nested Brane Tests =====

    #[test]
    fn nested_branes() {
        let output = run_foo("{5; {10; 15}; 20;}");
        insta::assert_snapshot!("nested_branes", output);
    }

    #[test]
    fn deeply_nested_branes() {
        let output = run_foo("{{{1;}; 2}; 3;}");
        insta::assert_snapshot!("deeply_nested_branes", output);
    }

    #[test]
    fn nested_branes_with_arithmetic() {
        let output = run_foo("{2 + 3; {4 * 5; {6 - 1}; 7 + 8}; 9 / 3;}");
        insta::assert_snapshot!("nested_branes_with_arithmetic", output);
    }

    #[test]
    fn level_skipping_search_found() {
        let output = run_foo("{x = 42; y = x; nested = {z = x};}");
        insta::assert_snapshot!("level_skipping_search_found", output);
    }

    #[test]
    fn level_skipping_search_not_found() {
        let output = run_foo("{x = undeclared; outer = {inner = {y = missing};};}");
        insta::assert_snapshot!("level_skipping_search_not_found", output);
    }

    #[test]
    fn nested_brane_boundary() {
        // Seek inside nested brane respects boundary
        let output = run_foo("{a = 1; b = {c = #-1; d = 2; e = #-1}; f = #-1;}");
        insta::assert_snapshot!("nested_brane_boundary", output);
    }

    // ===== Search / Regex Tests =====

    #[test]
    fn simple_regex_search() {
        let output = run_foo("{x = 5; result = {y = 1;}?y;}");
        insta::assert_snapshot!("simple_regex_search", output);
    }

    #[test]
    fn regex_search_pattern() {
        let output = run_foo("{result = {alice = 1; bob = 2; charlie = 3;}?(a.*);}");
        insta::assert_snapshot!("regex_search_pattern", output);
    }

    #[test]
    fn regex_search_not_found() {
        let output = run_foo("{result = {x = 100; y = 200;}?notfound;}");
        insta::assert_snapshot!("regex_search_not_found", output);
    }

    #[test]
    fn regex_search_anchor_start() {
        let output = run_foo("{result = {alice = 1; adam = 2; bob = 3;}?^a;}");
        insta::assert_snapshot!("regex_search_anchor_start", output);
    }

    #[test]
    fn regex_search_anchor_end() {
        let output = run_foo("{result = {alice = 1; charlie = 2; bob = 3;}?e$;}");
        insta::assert_snapshot!("regex_search_anchor_end", output);
    }

    #[test]
    fn assignment_anchor_search() {
        let output = run_foo("{brn = {alice = 2; bob = 3; charlie = 4}; result1 = brn?a.*; result2 = brn?b.*;}");
        insta::assert_snapshot!("assignment_anchor_search", output);
    }

    #[test]
    fn search_pattern_basics() {
        let output = run_foo("{simple = {x=10; y=20; z=30;}?y; notfound = {a=1; b=2;}?missing;}");
        insta::assert_snapshot!("search_pattern_basics", output);
    }

    // ===== Head/Tail (One-shot) Tests =====

    #[test]
    fn head_tail_basic() {
        let output = run_foo("{x = {10; 20; 30}^; y = {10; 20; 30}$;}");
        insta::assert_snapshot!("head_tail_basic", output);
    }

    #[test]
    fn head_tail_empty_brane() {
        let output = run_foo("{e = {}^; f = {}$;}");
        insta::assert_snapshot!("head_tail_empty_brane", output);
    }

    #[test]
    fn head_tail_on_named_brane() {
        let output = run_foo("{brane = {1; 2; 3}; val = brane^; last = brane$;}");
        insta::assert_snapshot!("head_tail_on_named_brane", output);
    }

    #[test]
    fn head_tail_nested_brane() {
        let output = run_foo("{b = {{1; 2}; {3; 4}}$; c = b^; d = b$;}");
        insta::assert_snapshot!("head_tail_nested_brane", output);
    }

    // ===== Offset / Seek Tests =====

    #[test]
    fn offset_access_forward() {
        let output = run_foo("{data = {a=10; b=20; c=30; d=40; e=50}; first = data#0; second = data#1;}");
        insta::assert_snapshot!("offset_access_forward", output);
    }

    #[test]
    fn offset_access_backward() {
        let output = run_foo("{data = {a=10; b=20; c=30; d=40; e=50}; last = data#-1; second_last = data#-2;}");
        insta::assert_snapshot!("offset_access_backward", output);
    }

    #[test]
    fn offset_access_out_of_bounds() {
        let output = run_foo("{data = {a=10; b=20; c=30; d=40; e=50}; oob = data#5; oob_neg = data#-6;}");
        insta::assert_snapshot!("offset_access_out_of_bounds", output);
    }

    #[test]
    fn offset_access_empty_brane() {
        let output = run_foo("{empty = {}; result = empty#0;}");
        insta::assert_snapshot!("offset_access_empty_brane", output);
    }

    #[test]
    fn unanchored_seek_basic() {
        let output = run_foo("{a = 1; b = 2; c = #-1 + #-2;}");
        insta::assert_snapshot!("unanchored_seek_basic", output);
    }

    #[test]
    fn unanchored_seek_chain() {
        let output = run_foo("{val1 = 3; val2 = 4; val3 = 5; sum = #-1 + #-2 + #-3;}");
        insta::assert_snapshot!("unanchored_seek_chain", output);
    }

    #[test]
    fn unanchored_seek_with_head_tail() {
        let output = run_foo("{a = 1; b = {10; 20; 30}; c = #-1; d = #-1$;}");
        insta::assert_snapshot!("unanchored_seek_with_head_tail", output);
    }

    // ===== Anchored Search Edge Cases =====

    #[test]
    fn anchored_search_fails_on_constant() {
        let output = run_foo("{b = {x = 100; y = 200}; notFound = b?γ;}");
        insta::assert_snapshot!("anchored_search_fails_on_constant", output);
    }

    #[test]
    fn anchored_search_on_constanic() {
        let output = run_foo("{emptyBrane = {}; chained = emptyBrane^;}");
        insta::assert_snapshot!("anchored_search_on_constanic", output);
    }

    // ===== Concatenation Tests =====

    #[test]
    fn concatenation_inline_branes() {
        let output = run_foo("{c = {a=1, b=2, c=3}{e=4, f=5, g=6};}");
        insta::assert_snapshot!("concatenation_inline_branes", output);
    }

    #[test]
    fn concatenation_references() {
        let output = run_foo("{b1={a=1, b=2, c=3}; b2={e=4, f=5, g=6}; c = b1 b2;}");
        insta::assert_snapshot!("concatenation_references", output);
    }

    #[test]
    fn concatenation_mixed() {
        let output = run_foo("{b1={a=1, b=2, c=3}; c = {x=10} b1 {y=20};}");
        insta::assert_snapshot!("concatenation_mixed", output);
    }

    #[test]
    fn concatenation_three_way() {
        let output = run_foo("{b1={a=1, b=2}; b2={c=3, d=4}; b3={e=5, f=6}; c = b1 b2 b3;}");
        insta::assert_snapshot!("concatenation_three_way", output);
    }

    // ===== Alarm System Tests =====

    #[test]
    fn alarm_division_by_zero_in_brane() {
        // Multiple divisions including by zero
        let output = run_foo("{a = 10 / 2; b = 10 / 0; c = 20 / 4;}");
        insta::assert_snapshot!("alarm_division_by_zero_in_brane", output);
    }

    #[test]
    fn alarm_unknown_literal_no_alarm() {
        let output = run_foo("{x = ???; y = 42;}");
        insta::assert_snapshot!("alarm_unknown_literal_no_alarm", output);
    }

    #[test]
    fn alarm_nested_division_by_zero() {
        let output = run_foo("{outer = {inner = 5 / 0};}");
        insta::assert_snapshot!("alarm_nested_division_by_zero", output);
    }

    // ===== Operator Search Transparency Tests =====

    #[test]
    fn operator_transparency_deep_chain() {
        // Deep unanchored seeks through operator nodes
        let output = run_foo("{a=1, b=2, c=3, d=4, e=5; result = #-1 + #-2 + #-3 + #-4 + #-5;}");
        insta::assert_snapshot!("operator_transparency_deep_chain", output);
    }

    #[test]
    fn operator_transparency_mixed_ops() {
        let output = run_foo("{a=1, b=2, c=3; result = #-1 * #-2 + #-3;}");
        insta::assert_snapshot!("operator_transparency_mixed_ops", output);
    }

    #[test]
    fn operator_transparency_unary_in_brane() {
        let output = run_foo("{a=5, b=-a;}");
        insta::assert_snapshot!("operator_transparency_unary_in_brane", output);
    }

    // ===== Complex Integration Tests =====

    #[test]
    fn complex_negative_results() {
        let output = run_foo("{a = 5 - 10; b = 3 * 2; c = 15 + 7;}");
        insta::assert_snapshot!("complex_negative_results", output);
    }

    #[test]
    fn complex_nested_scope() {
        let output = run_foo("{a = 10; b = 20; outer = {c = 30; ac = a + c; inner = {a = 40; abc = a + b + c}; ab = a + b};}");
        insta::assert_snapshot!("complex_nested_scope", output);
    }

    #[test]
    fn complex_search_and_concatenation() {
        let output = run_foo("{target = {a=1; c=2; c={a=1, b=2, c=3}}; b1 = {x=10}; result = b1 target.c;}");
        insta::assert_snapshot!("complex_search_and_concatenation", output);
    }

    #[test]
    fn complex_sff_in_nested_brane() {
        let output = run_foo("{a=1, b=2; inner = {c = <<a+b>>; c}; inner;}");
        insta::assert_snapshot!("complex_sff_in_nested_brane", output);
    }

    #[test]
    fn complex_sf_in_expression() {
        let output = run_foo("{x=10; y=<x>; z=y + 5;}");
        insta::assert_snapshot!("complex_sf_in_expression", output);
    }

    #[test]
    fn complex_multiple_seeks_in_brane() {
        let output = run_foo("{a=1, b=2, c=3, d=4, e=5; s1 = #-1; s2 = #-2; s3 = #-3;}");
        insta::assert_snapshot!("complex_multiple_seeks_in_brane", output);
    }

    #[test]
    fn complex_brane_with_operations_and_search() {
        let output = run_foo("{x=10; y=20; z=30; sum = x + y + z; avg = sum / 3;}");
        insta::assert_snapshot!("complex_brane_with_operations_and_search", output);
    }

    // ===== File-based Tests from Input Directory =====

    #[test]
    fn file_shebang() { approval_test("shebang"); }

    #[test]
    fn file_simple_addition() { approval_test("simpleAdditionIsApproved"); }

    #[test]
    fn file_simple_integer() { approval_test("simpleIntegerIsApproved"); }

    #[test]
    fn file_simple_division() { approval_test("simpleDivisionIsApproved"); }

    #[test]
    fn file_zero_division() { approval_test("zeroDivisionIsApproved"); }

    #[test]
    fn file_simple_subtraction() { approval_test("simpleSubtractionIsApproved"); }

    #[test]
    fn file_simple_multiplication() { approval_test("simpleMultiplicationIsApproved"); }

    #[test]
    fn file_simple_unary_minus() { approval_test("simpleUnaryMinusIsApproved"); }

    #[test]
    fn file_operator_precedence() { approval_test("operatorPrecedenceIsApproved"); }

    #[test]
    fn file_nested_arithmetic() { approval_test("nestedArithmeticIsApproved"); }

    #[test]
    fn file_single_expression() { approval_test("singleExpressionIsApproved"); }

    #[test]
    fn file_multiple_expressions() { approval_test("multipleExpressionsIsApproved"); }

    #[test]
    fn file_multiple_arithmetic() { approval_test("multipleArithmeticExpressionsIsApproved"); }

    #[test]
    fn file_mixed_operators() { approval_test("mixedOperatorsIsApproved"); }

    #[test]
    fn file_mixed_expressions() { approval_test("mixedExpressionsIsApproved"); }

    #[test]
    fn file_negative_results() { approval_test("negativeResultsIsApproved"); }

    #[test]
    fn file_simple_identifier() { approval_test("simpleIdentifierIsApproved"); }

    #[test]
    fn file_identifier_in_expression() { approval_test("identifierInExpressionIsApproved"); }

    #[test]
    fn file_identifier_shadowing() { approval_test("identifierShadowingIsApproved"); }

    #[test]
    fn file_multiple_identifiers() { approval_test("multipleIdentifiersIsApproved"); }

    #[test]
    fn file_nested_branes() { approval_test("nestedBranesIsApproved"); }

    #[test]
    fn file_deeply_nested_branes() { approval_test("deeplyNestedBranesIsApproved"); }

    #[test]
    fn file_nested_branes_with_arithmetic() { approval_test("nestedBranesWithArithmeticIsApproved"); }

    #[test]
    fn file_simple_regex_search() { approval_test("simpleRegexSearchIsApproved"); }

    #[test]
    fn file_regex_search_with_pattern() { approval_test("regexSearchWithPatternIsApproved"); }

    #[test]
    fn file_regex_search_not_found() { approval_test("regexSearchNotFoundIsApproved"); }

    #[test]
    fn file_search_pattern_basics() { approval_test("searchPatternBasicsIsApproved"); }

    #[test]
    fn file_search_regex_patterns() { approval_test("searchRegexPatternsIsApproved"); }

    #[test]
    fn file_search_localized_vs_globalized() { approval_test("searchLocalizedVsGlobalizedIsApproved"); }

    #[test]
    fn file_assignment_anchor() { approval_test("assignmentAnchor"); }

    #[test]
    fn file_constantic_rendering() { approval_test("constanticRendering"); }

    #[test]
    fn file_anchored_search_fails_on_constant() { approval_test("anchoredSearchFailsOnConstant"); }

    #[test]
    fn file_anchored_search_on_constanic() { approval_test("anchoredSearchOnConstanic"); }

    #[test]
    fn file_offset_access() { approval_test("offsetAccess"); }

    #[test]
    fn file_unanchored_seek_basic() { approval_test("unanchoredSeekBasic"); }

    #[test]
    fn file_one_shot_search() { approval_test("oneShotSearchIsApproved"); }

    #[test]
    fn file_level_skipping_found() { approval_test("levelSkippingSearchFound"); }

    #[test]
    fn file_level_skipping_constanic() { approval_test("levelSkippingSearchConstanic"); }

    #[test]
    fn file_level_skipping_not_found() { approval_test("levelSkippingSearchNotFound"); }

    #[test]
    fn file_concatenation_basics() { approval_test("concatenationBasics"); }

    #[test]
    fn file_concatenation_search() { approval_test("concatenationSearch"); }

    #[test]
    fn file_test_unanchored_oneshot() { approval_test("test_unanchored_oneshot"); }

    #[test]
    fn file_test_tilde() { approval_test("testTilde"); }

    #[test]
    fn file_test_nested_brane_boundary() { approval_test("test_nested_brane_boundary"); }

    #[test]
    fn file_test_syntax() { approval_test("test_syntax"); }

    #[test]
    fn file_comment_ends_statement() { approval_test("commentEndsStatement"); }

    // NOTE: identifierSeparators.foo has unsupported characters (², ₀, ©, §, ™)
    // that the parser doesn't accept in identifiers yet. Deferred to future phase.
    // fn file_identifier_separators() { approval_test("identifierSeparators"); }

    #[test]
    fn file_regex_search_shadowy() { approval_test("regexSearchShadowy"); }

    #[test]
    fn file_nested_scope_identifier() { approval_test("nestedScopeIdentifierIsApproved"); }

    #[test]
    fn file_nested_scope_shadowing() { approval_test("nestedScopeShadowingIsApproved"); }

    #[test]
    fn file_complex_identifier_scope() { approval_test("complexIdentifierScopeIsApproved"); }

    #[test]
    fn file_four_level_nested_branes() { approval_test("fourLevelNestedBranesWithNamesIsApproved"); }

    #[test]
    fn file_very_deep_nesting() { approval_test("veryDeepNestingIsApproved"); }

    // ===== Additional Edge Case Tests =====

    #[test]
    fn large_numbers() {
        let output = run_foo("{a = 1000000; b = 999999; c = a - b;}");
        insta::assert_snapshot!("large_numbers", output);
    }

    #[test]
    fn division_exact_and_remainder() {
        let output = run_foo("{a = 10 / 3; b = 9 / 3;}");
        insta::assert_snapshot!("division_exact_and_remainder", output);
    }

    #[test]
    fn brane_with_single_value_head_tail() {
        let output = run_foo("{b = {42}; h = b^; t = b$;}");
        insta::assert_snapshot!("brane_with_single_value_head_tail", output);
    }

    #[test]
    fn sff_resolves_on_each_use() {
        // SFF should re-resolve each time it's accessed
        let output = run_foo("{a=1; b=2; s=<<a+b>>; a=10; s;}");
        insta::assert_snapshot!("sff_resolves_on_each_use", output);
    }

    #[test]
    fn sf_blocks_brane_at_assignment_time() {
        // SF captures the brane at assignment time
        let output = run_foo("{a={x=1, y=2}; s=<a>; a=99; s;}");
        insta::assert_snapshot!("sf_blocks_brane_at_assignment_time", output);
    }

    #[test]
    fn concatenation_with_search_result() {
        let output = run_foo("{target = {a=1, b=2, c={x=10}}; result = {y=5} target.c;}");
        insta::assert_snapshot!("concatenation_with_search_result", output);
    }

    #[test]
    fn seek_in_nested_result_after_concatenation() {
        let output = run_foo("{OB1={a=1; message=2; only=1}; OB2={which_1 = #-1 + 1}; OB = OB1 OB2;}");
        insta::assert_snapshot!("seek_in_nested_result_after_concatenation", output);
    }

    #[test]
    fn chained_unary_operators() {
        let output = run_foo("{a = -(-5); b = -(-(-3));}");
        insta::assert_snapshot!("chained_unary_operators", output);
    }

    #[test]
    fn nested_search_in_brane() {
        let output = run_foo("{x = 42; b = {y = x + 1};}");
        insta::assert_snapshot!("nested_search_in_brane", output);
    }

    #[test]
    fn multiple_concatenation_in_sequence() {
        let output = run_foo("{a={1;2}; b={3;4}; c={5;6}; r1 = a b; r2 = b c; r3 = a c;}");
        insta::assert_snapshot!("multiple_concatenation_in_sequence", output);
    }

    #[test]
    fn search_through_concatenation() {
        let output = run_foo("{b={x=10}; c = b {y=20}; result = c.x + c.y;}");
        insta::assert_snapshot!("search_through_concatenation", output);
    }

    #[test]
    fn unicode_identifiers_basic() {
        let output = run_foo("{x = 1; π = 3; Б = 7; sum = x + π + Б;}");
        insta::assert_snapshot!("unicode_identifiers_basic", output);
    }

    #[test]
    fn seek_zero_offset() {
        let output = run_foo("{b = {10; 20; 30}; first = b#0;}");
        insta::assert_snapshot!("seek_zero_offset", output);
    }

    #[test]
    fn unanchored_seek_large_negative() {
        let output = run_foo("{a=1; b=2; c=3; d=#-100;}");
        insta::assert_snapshot!("unanchored_seek_large_negative", output);
    }

    #[test]
    fn brane_with_operations_and_underscores() {
        let output = run_foo("{my_var = 10; another_var = 20; result = my_var + another_var;}");
        insta::assert_snapshot!("brane_with_operations_and_underscores", output);
    }

    // ===== Deep Scope Resolution Tests =====

    #[test]
    fn forward_reference_basic() {
        // Forward reference: y is resolved after x is defined
        let output = run_foo("{y = x; x = 42;}");
        insta::assert_snapshot!("forward_reference_basic", output);
    }

    #[test]
    fn forward_reference_in_nested_brane() {
        let output = run_foo("{outer = {val = x}; x = 100;}");
        insta::assert_snapshot!("forward_reference_in_nested_brane", output);
    }

    #[test]
    fn scope_shadowing_multiple_levels() {
        let output = run_foo("{x = 1; b = {x = 2; c = {x = 3; val = x}; val2 = x}; val3 = x;}");
        insta::assert_snapshot!("scope_shadowing_multiple_levels", output);
    }

    #[test]
    fn cross_scope_reference_chain() {
        let output = run_foo("{a = 1; b = {c = a + 1; d = c + 1};}");
        insta::assert_snapshot!("cross_scope_reference_chain", output);
    }

    // ===== Complex Concatenation Tests =====

    #[test]
    fn concatenation_of_empty_branes() {
        let output = run_foo("{a = {}; b = {}; c = a b;}");
        insta::assert_snapshot!("concatenation_of_empty_branes", output);
    }

    #[test]
    fn concatenation_with_single_element() {
        let output = run_foo("{a = {x=1}; b = {y=2}; c = a b;}");
        insta::assert_snapshot!("concatenation_with_single_element", output);
    }

    #[test]
    fn concatenation_repeated_reference() {
        let output = run_foo("{a = {x=1}; c = a a;}");
        insta::assert_snapshot!("concatenation_repeated_reference", output);
    }

    #[test]
    fn concatenation_with_unresolved_search() {
        let output = run_foo("{a = {x=ref}; b = {y=2}; c = a b;}");
        insta::assert_snapshot!("concatenation_with_unresolved_search", output);
    }

    // ===== Operator Edge Cases =====

    #[test]
    fn operator_with_zero_operands_edge() {
        let output = run_foo("{a = 0 * 100; b = 0 + 999;}");
        insta::assert_snapshot!("operator_with_zero_operands_edge", output);
    }

    #[test]
    fn division_by_zero_in_nested_brane() {
        let output = run_foo("{outer = {inner = {a = 5 / 0; b = 10 / 2};};}");
        insta::assert_snapshot!("division_by_zero_in_nested_brane", output);
    }

    #[test]
    fn operator_chain_with_division_by_zero() {
        let output = run_foo("{a = 10 / 0 * 5;}");
        insta::assert_snapshot!("operator_chain_with_division_by_zero", output);
    }

    #[test]
    fn operator_with_unary_and_binary() {
        let output = run_foo("{a = -3 * 4; b = -3 + 4;}");
        insta::assert_snapshot!("operator_with_unary_and_binary", output);
    }

    #[test]
    fn operator_in_nested_brane_with_scope() {
        let output = run_foo("{x = 10; b = {y = x * 2 + 3};}");
        insta::assert_snapshot!("operator_in_nested_brane_with_scope", output);
    }

    // ===== Head/Tail Edge Cases =====

    #[test]
    fn head_tail_on_two_element_brane() {
        let output = run_foo("{b = {10; 20}; h = b^; t = b$;}");
        insta::assert_snapshot!("head_tail_on_two_element_brane", output);
    }

    #[test]
    fn head_tail_on_search_result() {
        let output = run_foo("{b = {x = {1; 2; 3}}; h = b.x^; t = b.x$;}");
        insta::assert_snapshot!("head_tail_on_search_result", output);
    }

    #[test]
    fn head_tail_chained_on_nested() {
        let output = run_foo("{b = {{{1; 2}; 3}; 4}; h = b^; hh = b^^;}");
        insta::assert_snapshot!("head_tail_chained_on_nested", output);
    }

    // ===== Seek Edge Cases =====

    #[test]
    fn seek_from_first_statement() {
        let output = run_foo("{a = 1; b = #-1;}");
        insta::assert_snapshot!("seek_from_first_statement", output);
    }

    #[test]
    fn seek_across_nested_brane_boundary() {
        let output = run_foo("{a = 1; b = 2; inner = {x = #-1}; c = #-1;}");
        insta::assert_snapshot!("seek_across_nested_brane_boundary", output);
    }

    #[test]
    fn anchored_seek_positive_boundary() {
        let output = run_foo("{b = {10; 20; 30}; first = b#0; second = b#1; third = b#2; oob = b#3;}");
        insta::assert_snapshot!("anchored_seek_positive_boundary", output);
    }

    #[test]
    fn anchored_seek_negative_boundary() {
        let output = run_foo("{b = {10; 20; 30}; last = b#-1; second = b#-2; first = b#-3; oob = b#-4;}");
        insta::assert_snapshot!("anchored_seek_negative_boundary", output);
    }

    // ===== SFF/SF Interaction Tests =====

    #[test]
    fn sff_vs_sf_timing_difference() {
        // SFF re-evaluates, SF is fixed
        let output = run_foo("{x = 1; sf = <x>; sff = <<x>>; x = 10; sf; sff;}");
        insta::assert_snapshot!("sff_vs_sf_timing_difference", output);
    }

    #[test]
    fn sff_in_assignment_chain() {
        let output = run_foo("{a = 1; b = 2; c = <<a + b>>; a = 100; c;}");
        insta::assert_snapshot!("sff_in_assignment_chain", output);
    }

    #[test]
    fn sf_of_sff() {
        let output = run_foo("{a = 1; b = 2; sff = <<a + b>>; sf = <sff>; a = 10; sf; sff;}");
        insta::assert_snapshot!("sf_of_sff", output);
    }

    // ===== Search Pattern Edge Cases =====

    #[test]
    fn search_pattern_with_complex_regex() {
        let output = run_foo("{b = {tmp_a = 1; result = 2; tmp_b = 3; output = 4}; r1 = b?tmp_.*; r2 = b?result;}");
        insta::assert_snapshot!("search_pattern_with_complex_regex", output);
    }

    #[test]
    fn search_pattern_matching_nested_brane() {
        let output = run_foo("{b = {a = {x = 1}; b = 2}; r = b?a;}");
        insta::assert_snapshot!("search_pattern_matching_nested_brane", output);
    }

    #[test]
    fn search_with_multiple_matches() {
        let output = run_foo("{b = {a1 = 1; a2 = 2; a3 = 3}; r = b?a.*;}");
        insta::assert_snapshot!("search_with_multiple_matches", output);
    }

    // ===== Alarm System Integration Tests =====

    #[test]
    fn alarm_multiple_divisions_by_zero() {
        let output = run_foo("{a = 1 / 0; b = 2 / 0; c = 3 / 0;}");
        insta::assert_snapshot!("alarm_multiple_divisions_by_zero", output);
    }

    #[test]
    fn alarm_division_by_zero_deeply_nested() {
        let output = run_foo("{l1 = {l2 = {l3 = {bad = 1 / 0; good = 42};};};}");
        insta::assert_snapshot!("alarm_division_by_zero_deeply_nested", output);
    }

    #[test]
    fn alarm_mixed_alarms_and_normals() {
        let output = run_foo("{good = 42; bad = 1 / 0; also_good = 10 + 20;}");
        insta::assert_snapshot!("alarm_mixed_alarms_and_normals", output);
    }

    // ===== Complex Integration Tests =====

    #[test]
    fn complex_full_program_with_all_features() {
        let output = run_foo("{a = 10; b = 20; sum = a + b; nested = {inner = sum / 2}; result = nested.inner;}");
        insta::assert_snapshot!("complex_full_program_with_all_features", output);
    }

    #[test]
    fn complex_search_concat_and_seeks() {
        let output = run_foo("{data = {x=1, y=2, z=3}; copy = data {w=4}; last = copy#-1; first = copy#0;}");
        insta::assert_snapshot!("complex_search_concat_and_seeks", output);
    }

    #[test]
    fn complex_sff_with_nested_scope() {
        let output = run_foo("{x = 5; y = 10; inner = {calc = <<x + y>>; doubled = calc * 2};}");
        insta::assert_snapshot!("complex_sff_with_nested_scope", output);
    }

    #[test]
    fn complex_unanchored_seeks_with_operations() {
        let output = run_foo("{a=10, b=20, c=30, result=#-1 + #-2, result2=#-1 * #-2, result3=#-1 - #-2;}");
        insta::assert_snapshot!("complex_unanchored_seeks_with_operations", output);
    }

    #[test]
    fn complex_forward_refs_in_nested_branes() {
        let output = run_foo("{nested = {inner = {val = x}}; x = 42;}");
        insta::assert_snapshot!("complex_forward_refs_in_nested_branes", output);
    }

    #[test]
    fn complex_brane_with_all_operator_types() {
        let output = run_foo("{add = 1 + 2; sub = 10 - 3; mul = 4 * 5; div = 20 / 4; neg = -7;}");
        insta::assert_snapshot!("complex_brane_with_all_operator_types", output);
    }

    #[test]
    fn complex_concat_with_operations() {
        let output = run_foo("{a = {x=1+2}; b = {y=3*4}; c = a b;}");
        insta::assert_snapshot!("complex_concat_with_operations", output);
    }

    // ===== Named Brane Tests =====

    #[test]
    fn named_brane_basic() {
        let output = run_foo("{x = 10; myname'{val = x};}");
        insta::assert_snapshot!("named_brane_basic", output);
    }

    #[test]
    fn named_brane_with_search() {
        let output = run_foo("{x = 42; outer'{inner = {y = x}; val = inner.y};}");
        insta::assert_snapshot!("named_brane_with_search", output);
    }

    #[test]
    fn named_brane_shadowing() {
        let output = run_foo("{x = 100; inner'{x = 200; x}; x;}");
        insta::assert_snapshot!("named_brane_shadowing", output);
    }

    // ===== Regression Tests =====

    #[test]
    fn regression_regression_disappearing_brane() {
        // Tests that brane statements are preserved
        let output = run_foo("{a = 1; b = 2; c = 3;}");
        insta::assert_snapshot!("regression_regression_disappearing_brane", output);
    }

    #[test]
    fn regression_deep_nesting_does_not_lose_values() {
        let output = run_foo("{{{{x = 42; x};};};}");
        insta::assert_snapshot!("regression_deep_nesting_does_not_lose_values", output);
    }

    #[test]
    fn regression_operator_does_not_block_search() {
        // Verify operator search transparency holds for complex expressions
        let output = run_foo("{a=1, b=2, c=3, d=4, e=5, f=6; expr = #-1 + #-2 + #-3 + #-4 + #-5 + #-6;}");
        insta::assert_snapshot!("regression_operator_does_not_block_search", output);
    }

    // ===== Cross Validation Tests =====

    #[test]
    fn crossval_simple_addition() { approval_test("simpleAdditionIsApproved"); }

    #[test]
    fn crossval_simple_subtraction() { approval_test("simpleSubtractionIsApproved"); }

    #[test]
    fn crossval_simple_multiplication() { approval_test("simpleMultiplicationIsApproved"); }

    #[test]
    fn crossval_simple_division() { approval_test("simpleDivisionIsApproved"); }

    #[test]
    fn crossval_simple_integer() { approval_test("simpleIntegerIsApproved"); }

    #[test]
    fn crossval_empty_brane() { approval_test("emptyBraneIsApproved"); }

    #[test]
    fn crossval_chained_arithmetic() { approval_test("chainedArithmeticIsApproved"); }

    #[test]
    fn crossval_simple_identifier() { approval_test("simpleIdentifierIsApproved"); }

    #[test]
    fn crossval_identifier_shadowing() { approval_test("identifierShadowingIsApproved"); }

    #[test]
    fn crossval_nested_branes() { approval_test("nestedBranesIsApproved"); }

    #[test]
    fn crossval_zero_division() { approval_test("zeroDivisionIsApproved"); }

    #[test]
    fn crossval_simple_unary_minus() { approval_test("simpleUnaryMinusIsApproved"); }

    #[test]
    fn crossval_simple_regex_search() { approval_test("simpleRegexSearchIsApproved"); }

    #[test]
    fn crossval_anchored_search_on_constant() { approval_test("anchoredSearchOnConstant"); }

    #[test]
    fn file_complex_arithmetic() { approval_test("complexArithmeticIsApproved"); }

    // ===== Regression Test: Parse Error Recovery =====

    #[test]
    fn regression_disappearing_brane_statements() {
        // This tests the syntax "d =$ 4" which the parser accepts as tail-of-dollar
        // The =$ prefix followed by a value is parsed as $ (tail) of the following expression.
        let output = run_foo("{a = 1; b = 2; d =$ 4; e = 5; f = 6; g = 7;}");
        insta::assert_snapshot!("regression_disappearing_brane_statements", output);
    }
}

// ============================================================================
// Sequencer Tests: Builders + HumanizingSequencerRef
// ============================================================================

#[cfg(test)]
mod sequencer_tests {
    use super::*;
    use crate::fir::{
        ConstantIntFirBuilder, NkFirBuilder, OperatorFirBuilder, SearchFirBuilder,
        IndexFirBuilder, HeadTailFirBuilder, StayFoolishFirBuilder, StayFullyFoolishFirBuilder,
        ConcatenationFirBuilder, NormalBraneFirBuilder, Nyes, SearchDirection, Alarm, AlarmLevel, AlarmSource,
    };
    use crate::HumanizingSequencerRef;
    use crate::sequencer::format_fir_simple;

    fn format_fir_ref(fir: &Fir) -> String {
        HumanizingSequencerRef::new(fir).format_for_snap_test()
    }

    // ── FirQueryable variant checks ─────────────────────────────────────

    #[test]
    fn test_hs_variant_all_types() {
        assert_eq!(ConstantIntFirBuilder::new(42).build().hs_variant(), "ConstantInt");
        assert_eq!(NkFirBuilder::new("unknown").build().hs_variant(), "Nk");
        assert_eq!(OperatorFirBuilder::new("+").build().hs_variant(), "Operator");
        assert_eq!(SearchFirBuilder::new("x").build().hs_variant(), "Search");
        assert_eq!(IndexFirBuilder::new(1).build().hs_variant(), "Index");
        assert_eq!(HeadTailFirBuilder::new(true).build().hs_variant(), "HeadTail");
        assert_eq!(StayFoolishFirBuilder::new(ConstantIntFirBuilder::new(1).build()).build().hs_variant(), "StayFoolish");
        assert_eq!(StayFullyFoolishFirBuilder::new(ConstantIntFirBuilder::new(1).build()).build().hs_variant(), "StayFullyFoolish");
        assert_eq!(ConcatenationFirBuilder::new().build().hs_variant(), "Concatenation");
        assert_eq!(NormalBraneFirBuilder::new().build().hs_variant(), "NormalBrane");
    }

    // ── HumanizingSequencerRef formatting ────────────────────────────────

    #[test]
    fn test_format_empty_brane() {
        let brane = NormalBraneFirBuilder::new().state(Nyes::Constant).build();
        assert_eq!(format_fir_simple(&brane), "Brane{}");
    }

    #[test]
    fn test_format_constant_int() {
        let c = ConstantIntFirBuilder::new(42).build();
        assert_eq!(format_fir_simple(&c), "Int(42)");
    }

    #[test]
    fn test_format_named_statement() {
        let body = ConstantIntFirBuilder::new(42).build();
        let brane = NormalBraneFirBuilder::new()
            .statement(Some("x".into()), body)
            .build();
        let s = format_fir_simple(&brane);
        assert!(s.contains("x = Int(42)"), "Expected 'x = Int(42)' in: {}", s);
    }

    #[test]
    fn test_format_multi_statement() {
        let s = vec![
            (Some("a".into()), ConstantIntFirBuilder::new(1).build()),
            (Some("b".into()), ConstantIntFirBuilder::new(2).build()),
        ];
        let brane = NormalBraneFirBuilder::new().statements(s).build();
        let out = format_fir_simple(&brane);
        assert!(out.contains("a = Int(1)"), "Expected 'a = Int(1)' in: {}", out);
        assert!(out.contains("b = Int(2)"), "Expected 'b = Int(2)' in: {}", out);
        assert!(out.contains(";"), "Expected semicolon separator in: {}", out);
    }

    #[test]
    fn test_format_search() {
        let search = SearchFirBuilder::new("x")
            .direction(SearchDirection::Backward)
            .state(Nyes::Econstanic)
            .build();
        let s = format_fir_simple(&search);
        assert!(s.starts_with("Search("), "Expected Search( in: {}", s);
        assert!(s.contains("pattern='x'"), "Expected pattern='x' in: {}", s);
    }

    #[test]
    fn test_format_nk() {
        let nk = NkFirBuilder::new("unknown").build();
        let s = format_fir_simple(&nk);
        assert!(s.starts_with("NK("), "Expected NK( in: {}", s);
    }

    #[test]
    fn test_format_nk_with_alarm() {
        let alarm = Alarm {
            level: AlarmLevel::Mild,
            code: "TEST".to_string(),
            message: "test alarm".to_string(),
            source: AlarmSource::Evaluator,
        };
        let nk = NkFirBuilder::new("div-by-zero").alarm(alarm).build();
        let s = format_fir_simple(&nk);
        assert!(s.starts_with("NK("), "Expected NK( in: {}", s);
        assert!(s.contains("test alarm"), "Expected alarm message in: {}", s);
    }

    #[test]
    fn test_format_operator() {
        let op = OperatorFirBuilder::new("+")
            .operand(ConstantIntFirBuilder::new(1).build())
            .operand(ConstantIntFirBuilder::new(2).build())
            .state(Nyes::Constant)
            .build();
        let s = format_fir_simple(&op);
        assert!(s.contains("Operator(op='+'"), "Expected Operator(op='+' in: {}", s);
        assert!(s.contains("Int(1)"), "Expected Int(1) in: {}", s);
        assert!(s.contains("Int(2)"), "Expected Int(2) in: {}", s);
    }

    #[test]
    fn test_format_concatenation() {
        let conc = ConcatenationFirBuilder::new()
            .element(ConstantIntFirBuilder::new(1).build())
            .element(ConstantIntFirBuilder::new(2).build())
            .state(Nyes::Constant)
            .build();
        let s = format_fir_simple(&conc);
        assert!(s.starts_with("Concatenation("), "Expected Concatenation( in: {}", s);
        assert!(s.contains("elements=2"), "Expected elements=2 in: {}", s);
    }

    #[test]
    fn test_format_concatenation_merged() {
        let conc = ConcatenationFirBuilder::new()
            .element(ConstantIntFirBuilder::new(1).build())
            .merged(ConstantIntFirBuilder::new(99).build())
            .build();
        let s = format_fir_simple(&conc);
        assert!(s.contains("merged="), "Expected merged= in: {}", s);
    }

    #[test]
    fn test_format_index() {
        let idx = IndexFirBuilder::new(1).build();
        let s = format_fir_simple(&idx);
        assert!(s.contains("Index(offset=1, FREE)"), "Expected 'Index(offset=1, FREE)' in: {}", s);
    }

    #[test]
    fn test_format_index_anchored() {
        let idx = IndexFirBuilder::new(0).anchored(true).build();
        let s = format_fir_simple(&idx);
        assert!(s.contains("Index(offset=0, ANCHORED)"), "Expected 'Index(offset=0, ANCHORED)' in: {}", s);
    }

    #[test]
    fn test_format_headtail_head() {
        let ht = HeadTailFirBuilder::new(true).build();
        let s = format_fir_simple(&ht);
        assert!(s.contains("HeadTail(HEAD, FREE)"), "Expected 'HeadTail(HEAD, FREE)' in: {}", s);
    }

    #[test]
    fn test_format_headtail_tail_anchored() {
        let ht = HeadTailFirBuilder::new(false).anchored(true).build();
        let s = format_fir_simple(&ht);
        assert!(s.contains("HeadTail(TAIL, ANCHORED)"), "Expected 'HeadTail(TAIL, ANCHORED)' in: {}", s);
    }

    #[test]
    fn test_format_stay_foolish() {
        let sf = StayFoolishFirBuilder::new(ConstantIntFirBuilder::new(1).build()).build();
        let s = format_fir_simple(&sf);
        assert!(s.starts_with("StayFoolish("), "Expected StayFoolish( in: {}", s);
        assert!(s.contains("Int(1)"), "Expected Int(1) in: {}", s);
    }

    #[test]
    fn test_format_stay_fully_foolish() {
        let sff = StayFullyFoolishFirBuilder::new(ConstantIntFirBuilder::new(2).build()).build();
        let s = format_fir_simple(&sff);
        assert!(s.starts_with("StayFullyFoolish("), "Expected StayFullyFoolish( in: {}", s);
        assert!(s.contains("Int(2)"), "Expected Int(2) in: {}", s);
    }

    // ── Integration: Compile → FirQueryable → Format ─────────────────────

    #[test]
    fn test_integration_compile_format() {
        let firs = Compiler::compile("{x = 1 + 2}").unwrap();
        let formatted = format_fir_simple(&firs[0]);

        assert!(formatted.contains("x ="), "Expected 'x =' in: {}", formatted);
        assert!(formatted.contains("Operator(op='+'"), "Expected operator in: {}", formatted);
        assert!(formatted.contains("Int(1)"), "Expected Int(1) in: {}", formatted);
        assert!(formatted.contains("Int(2)"), "Expected Int(2) in: {}", formatted);
    }

    #[test]
    fn test_integration_multi_statement_roundtrip() {
        let firs = Compiler::compile("{a = 1; b = 2; c = 3}").unwrap();
        let formatted = format_fir_simple(&firs[0]);

        assert!(formatted.contains("a = Int(1)"), "Expected 'a = Int(1)' in: {}", formatted);
        assert!(formatted.contains("b = Int(2)"), "Expected 'b = Int(2)' in: {}", formatted);
        assert!(formatted.contains("c = Int(3)"), "Expected 'c = Int(3)' in: {}", formatted);
    }

    #[test]
    fn test_sequencer_ref_format_constant() {
        let fir = ConstantIntFirBuilder::new(42).build();
        let ref_seq = HumanizingSequencerRef::new(&fir);
        let out = ref_seq.format_for_snap_test();
        assert!(out.contains("Int(42)"), "Expected Int(42) in: {}", out);
    }

    #[test]
    fn test_sequencer_format_with_header() {
        let fir = ConstantIntFirBuilder::new(42).build();
        let out = Sequencer::format_with_header("{42}", &fir, 0);
        assert!(out.contains("INPUT:"));
        assert!(out.contains("STEPS:"));
    }
}

#[cfg(test)]
mod ubc_approval_tests {
    use super::*;
    use std::path::PathBuf;

    fn suite() -> SnapshotSuite {
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        SnapshotSuite::new(
            manifest.join("snapshot_tests").join("input"),
            manifest.join("snapshot_tests").join("approved"),
        )
        .expect("SnapshotSuite initialization failed")
    }

    #[test]
    fn approval_all() {
        let suite = suite();
        let evaluations = suite.evaluate_all(num_cpus::get(), &UbcEvaluator);
        let mut settings = insta::Settings::clone_current();
        settings.set_snapshot_path(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("snapshot_tests")
                .join("approved"),
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
}
