use crate::fir::AlarmSink;
use crate::*;
use std::rc::Rc;

#[test]
fn foop9_binary_compiles_to_operator() {
    let firs = Compiler::compile("{1 + 2}").unwrap();
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
    if let Fir::NormalBrane(nb) = &recovered {
        assert_eq!(nb.statements.len(), 1);
        assert!(matches!(
            nb.statements[0].body.borrow().fir_variant(),
            "Operator"
        ));
    } else {
        panic!("Expected NormalBrane");
    }
    let mut ref_fir = fir_to_ref(recovered);
    ubc::run_to_completion(&mut ref_fir).unwrap();
    assert!(ref_fir.borrow().state().is_constanic());
}

#[test]
fn foop9_chained_operators() {
    let firs = Compiler::compile("{1 + 2 * 3}").unwrap();
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
    let output = run_foo("{x=5, y=7, z=#-2 + #-1;}");
    assert!(
        output.contains("Int(12)"),
        "Expected z=12 but got: {}",
        output
    );
}

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
    let firs = Compiler::compile("{10 / 0}").unwrap();
    let mut fir_ref = fir_to_ref(firs[0].clone());
    ubc::run_to_completion(&mut fir_ref).unwrap();
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
    let scope = ubc::Scope::new();
    scope.emit(crate::fir::Alarm {
        level: crate::fir::AlarmLevel::Info,
        code: "TEST".to_string(),
        message: "Test message".to_string(),
        source: crate::fir::AlarmSource::Compiler,
    });
}

#[test]
fn test_negative_seek_oob_returns_nk() {
    // b#-4 on a 3-element brane should return NK, not clamp to element 0.
    // This is the symmetric counterpart to b#3 on a 3-element brane returning NK.
    let output = run_foo("{b = {10; 20; 30}; oob = b#-4;}");
    assert!(
        output.contains("NK"),
        "Expected b#-4 on 3-element brane to produce NK, but got:\n{}",
        output
    );
    assert!(
        !output.contains("oob = \n  Int(10)"),
        "b#-4 should NOT resolve to Int(10) (the first element). Got:\n{}",
        output
    );
}

#[test]
fn test_negative_seek_valid_boundary() {
    // b#-3 on a 3-element brane should return the first element (index 0).
    // b#-1 should return the last element (index 2).
    let output = run_foo("{b = {10; 20; 30}; first = b#-3; last = b#-1;}");
    assert!(
        output.contains("first = \n  Int(10)"),
        "b#-3 should be Int(10), got:\n{}",
        output
    );
    assert!(
        output.contains("last = \n  Int(30)"),
        "b#-1 should be Int(30), got:\n{}",
        output
    );
}

#[test]
fn test_search_concat_precedence_isolation() {
    let output_paren =
        run_foo("{target = {a=1; c=2; c={a=1, b=2, c=3}}; b1 = {x=10}; result = b1 (target.c);}");
    let output_no_paren =
        run_foo("{target = {a=1; c=2; c={a=1, b=2, c=3}}; b1 = {x=10}; result = b1 target.c;}");
    eprintln!("=== With parentheses ===\n{}", output_paren);
    eprintln!("=== Without parentheses ===\n{}", output_no_paren);
}

#[test]
fn test_dot_search_returns_last_match_backward() {
    let output = run_foo("{dup = {a=1; a=2; a=3}; result = dup.a;}");
    assert!(
        output.contains("result = \n  Int(3)"),
        "Backward dot search should return last match (a=3), got:\n{}",
        output
    );
}

#[test]
fn test_concat_of_brane_and_dot_search_result() {
    let output =
        run_foo("{target = {a=1; c=2; c={a=1, b=2, c=3}}; b1 = {x=10}; result = b1 target.c;}");
    assert!(
        output.contains("x = \n    Int(10)")
            && output.contains("a = \n    Int(1)")
            && output.contains("b = \n    Int(2)")
            && output.contains("c = \n    Int(3)"),
        "result should be concatenation of b1 and target.c brane, got:\n{}",
        output
    );
}

#[test]
fn test_bug_a_forward_ref_across_two_brane_boundaries() {
    // Bug A: Forward reference resolves across two brane boundaries.
    // Input: {nested = {inner = {val = x}}; x = 42;}
    // x is defined AFTER nested AND inside a nested nested brane, so val should NOT resolve.
    let output = run_foo("{nested = {inner = {val = x}}; x = 42;}");
    eprintln!("=== Bug A output ===\n{}", output);
    // val should NOT resolve to Int(42) - x is blocked by 2 brane boundaries and defined AFTER
    assert!(
        !output.contains("val = \n      Int(42)"),
        "Bug A: val should NOT resolve to Int(42) across two brane boundaries and forward ref. Got:\n{}",
        output
    );
}

#[test]
fn test_bug_c_parent_scope_resolves_in_nested_brane() {
    // Bug C: Search for `sum` fails inside nested brane despite being in parent scope.
    // Input: {a = 10; b = 20; sum = a + b; nested = {inner = sum / 2}; result = nested.inner;}
    // sum is defined BEFORE nested, so it should resolve.
    let output = run_foo(
        "{a = 10; b = 20; sum = a + b; nested = {inner = sum / 2}; result = nested.inner;}",
    );
    eprintln!("=== Bug C output ===\n{}", output);
    // inner should resolve to Int(15) since sum=30 and 30/2=15
    assert!(
        output.contains("inner = \n    Int(15)"),
        "Bug C: inner should resolve to Int(15) (sum=30, 30/2=15). Got:\n{}",
        output
    );
}

fn run_foo(source: &str) -> String {
    let firs = Compiler::compile(source)
        .unwrap_or_else(|e| panic!("Failed to compile '{}': {}", source, e));
    let mut lines = vec![format!(
        "INPUT: {}",
        source.lines().next().unwrap_or(source)
    )];
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
