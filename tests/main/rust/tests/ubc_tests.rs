use foolish_rust::{Ubc, Value};

fn run(source: &str) -> Value {
    let mut ubc = Ubc::from_source(source).expect("failed to create UBC");
    ubc.run_to_completion()
        .expect("execution error")
        .cloned()
        .unwrap_or(Value::None)
}

#[test]
fn evaluates_arithmetic_expressions() {
    let result = run("{ 1 + 2 * 3; }");
    assert_eq!(result, Value::Int(7));
}

#[test]
fn supports_assignments_and_references() {
    let result = run("{ x = 10; y = x + 5; y; }");
    assert_eq!(result, Value::Int(15));
}

#[test]
fn evaluates_nested_branes() {
    let result = run("{ inner = { 1; 2; }; inner; }");
    match result {
        Value::Brane(values) => {
            assert_eq!(values.len(), 2);
            assert_eq!(values[0], Value::Int(1));
            assert_eq!(values[1], Value::Int(2));
        }
        other => panic!("expected brane value, got {:?}", other),
    }
}

#[test]
fn detects_unknown_identifiers() {
    let mut ubc = Ubc::from_source("{ x + 1; }").expect("parsed");
    let err = ubc.step().expect_err("expected error");
    match err {
        foolish_rust::UbcError::UnknownIdentifier(name) => assert_eq!(name, "x"),
        other => panic!("unexpected error: {:?}", other),
    }
}
