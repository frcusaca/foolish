#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub brane: Brane,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Brane {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Expr(Expr),
    Assign { target: Identifier, expr: Expr },
}

pub type Identifier = String;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Int(i64),
    Identifier(Identifier),
    Brane(Box<Brane>),
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Negate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
}
