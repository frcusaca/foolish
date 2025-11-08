use foolish_rust::ast::{BinaryOp, Expr, Statement};
use foolish_rust::parse_program;

#[test]
fn parses_simple_program() {
    let source = "{ x = 1 + 2 * 3; y = x - 4; y; }";
    let program = parse_program(source).expect("failed to parse");
    assert_eq!(program.brane.statements.len(), 3);

    match &program.brane.statements[0] {
        Statement::Assign { target, expr } => {
            assert_eq!(target, "x");
            match expr {
                Expr::Binary { op, left, right } => {
                    assert_eq!(*op, BinaryOp::Add);
                    assert!(matches!(**left, Expr::Int(1)));
                    match &**right {
                        Expr::Binary { op, left, right } => {
                            assert_eq!(*op, BinaryOp::Multiply);
                            assert!(matches!(**left, Expr::Int(2)));
                            assert!(matches!(**right, Expr::Int(3)));
                        }
                        other => panic!("unexpected right expression: {:?}", other),
                    }
                }
                other => panic!("unexpected expr: {:?}", other),
            }
        }
        other => panic!("unexpected statement: {:?}", other),
    }
}
