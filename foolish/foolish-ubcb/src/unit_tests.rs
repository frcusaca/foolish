use super::*;
use foolish_core::{Fir, FirRef, Nyes, clone_steppable, fir_to_ref, ubc, Compiler};

#[test]
fn test_literals() {
    let mut engine = UbcbEngine::new();
    let result = engine.evaluate("{1; 2; 3}").unwrap();
    assert_eq!(result.statements.len(), 3);
    assert_eq!(result.brane_state, Nyes::Constant);
    for stmt in &result.statements {
        assert_eq!(stmt.state, Nyes::Independent);
    }
}

#[test]
fn test_identification() {
    let mut engine = UbcbEngine::new();
    let result = engine.evaluate("{x = 42}").unwrap();
    assert_eq!(result.statements.len(), 1);
    assert_eq!(result.statements[0].name, Some("x".to_string()));
    assert_eq!(result.statements[0].state, Nyes::Independent);
    assert_eq!(result.brane_state, Nyes::Constant);
}

#[test]
fn test_identification_and_use() {
    let mut engine = UbcbEngine::new();
    let result = engine.evaluate("{x = 42; y = x}").unwrap();
    assert_eq!(result.statements.len(), 2);
    assert_eq!(result.statements[0].state, Nyes::Independent);
    assert_eq!(result.statements[1].state, Nyes::Independent);
    assert_eq!(result.brane_state, Nyes::Constant);
}

#[test]
fn test_arithmetic() {
    let mut engine = UbcbEngine::new();
    let result = engine.evaluate("{1 + 2}").unwrap();
    assert_eq!(result.statements.len(), 1);
    assert_eq!(result.statements[0].state, Nyes::Constant);
    assert_eq!(result.brane_state, Nyes::Constant);
}

#[test]
fn test_named_arithmetic() {
    let mut engine = UbcbEngine::new();
    let result = engine.evaluate("{x = 1 + 2}").unwrap();
    assert_eq!(result.statements.len(), 1);
    assert_eq!(result.statements[0].name, Some("x".to_string()));
    assert_eq!(result.statements[0].state, Nyes::Constant);
    assert_eq!(result.brane_state, Nyes::Constant);
}

#[test]
fn test_chained() {
    let mut engine = UbcbEngine::new();
    let result = engine.evaluate("{x = 42; y = x + 8}").unwrap();
    assert_eq!(result.statements.len(), 2);
    assert_eq!(result.statements[0].state, Nyes::Independent);
    assert_eq!(result.statements[1].state, Nyes::Constant);
    assert_eq!(result.brane_state, Nyes::Constant);
}

#[test]
fn test_chained_two_hops() {
    let mut engine = UbcbEngine::new();
    let result = engine.evaluate("{x = 42; y = x; z = y + 1}").unwrap();
    assert_eq!(result.statements.len(), 3);
    assert_eq!(result.brane_state, Nyes::Constant);
}

#[test]
fn cross_validate_literals() {
    cross_validate("{1; 2; 3}");
}

#[test]
fn cross_validate_identification() {
    cross_validate("{x = 42}");
}

#[test]
fn cross_validate_identification_and_use() {
    cross_validate("{x = 42; y = x}");
}

#[test]
fn cross_validate_arithmetic() {
    cross_validate("{1 + 2}");
}

#[test]
fn cross_validate_named_arithmetic() {
    cross_validate("{x = 1 + 2}");
}

#[test]
fn cross_validate_chained() {
    cross_validate("{x = 42; y = x + 8}");
}

#[test]
fn cross_validate_chained_two_hops() {
    cross_validate("{x = 42; y = x; z = y + 1}");
}

#[test]
fn cross_validate_empty_brane() {
    cross_validate("{}");
}

#[test]
fn cross_validate_multiplication() {
    cross_validate("{3 * 4}");
}

#[test]
fn cross_validate_division() {
    cross_validate("{20 / 4}");
}

#[test]
fn cross_validate_subtraction() {
    cross_validate("{10 - 3}");
}

#[test]
fn cross_validate_unary_minus() {
    cross_validate("{-42}");
}

fn cross_validate(source: &str) {
    let firs_a = Compiler::compile(source).unwrap();
    let fir_ref: FirRef = fir_to_ref(firs_a[0].clone());
    let mut fir_a = fir_to_ref(clone_steppable(&fir_ref));
    ubc::run_to_completion(&mut fir_a).unwrap();

    let mut engine = UbcbEngine::new();
    let result_b = engine.evaluate(source).unwrap();

    let state_a = fir_a.borrow().state();
    assert_eq!(state_a, result_b.brane_state,
        "Brane state mismatch for '{}': UBC={}, UBCb={}", source, state_a, result_b.brane_state);

    if let Fir::NormalBrane(brane_a) = clone_steppable(&fir_a) {
        let stmts_a = brane_a.statements();
        assert_eq!(stmts_a.len(), result_b.statements.len(),
            "Statement count mismatch for '{}': UBC={}, UBCb={}", source, stmts_a.len(), result_b.statements.len());
        for (i, (sa, sb)) in stmts_a.iter().zip(result_b.statements.iter()).enumerate() {
            assert_eq!(sa.state(), sb.state,
                "Statement {} state mismatch for '{}': UBC={}, UBCb={}", i, source, sa.state(), sb.state);
        }
    }
}
