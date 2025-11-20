use crate::ast::*;
use crate::parser::{parse_program, ParseError};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Brane(Vec<Value>),
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UbcError {
    Parse(ParseError),
    UnknownIdentifier(String),
    DivisionByZero,
    UnsupportedOperation(String),
}

impl fmt::Display for UbcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UbcError::Parse(err) => write!(f, "{}", err),
            UbcError::UnknownIdentifier(name) => {
                write!(f, "Unknown identifier: {}", name)
            }
            UbcError::DivisionByZero => write!(f, "Division by zero"),
            UbcError::UnsupportedOperation(op) => write!(f, "Unsupported operation: {}", op),
        }
    }
}

impl std::error::Error for UbcError {}

impl From<ParseError> for UbcError {
    fn from(err: ParseError) -> Self {
        UbcError::Parse(err)
    }
}

pub struct Ubc {
    statements: Vec<Statement>,
    index: usize,
    scopes: Vec<HashMap<String, Value>>,
    result: Option<Value>,
    complete: bool,
}

impl Ubc {
    pub fn from_source(source: &str) -> Result<Self, UbcError> {
        let program = parse_program(source)?;
        Ok(Self::new(program))
    }

    pub fn new(program: Program) -> Self {
        Self {
            statements: program.brane.statements,
            index: 0,
            scopes: vec![HashMap::new()],
            result: None,
            complete: false,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.complete
    }

    pub fn result(&self) -> Option<&Value> {
        self.result.as_ref()
    }

    pub fn step(&mut self) -> Result<bool, UbcError> {
        if self.complete {
            return Ok(false);
        }
        if self.index >= self.statements.len() {
            self.complete = true;
            return Ok(false);
        }
        let stmt = self.statements[self.index].clone();
        self.index += 1;
        let value = self.execute_statement(&stmt)?;
        self.result = Some(value);
        if self.index >= self.statements.len() {
            self.complete = true;
        }
        Ok(true)
    }

    pub fn run_to_completion(&mut self) -> Result<Option<&Value>, UbcError> {
        while self.step()? {}
        Ok(self.result())
    }

    fn execute_statement(&mut self, stmt: &Statement) -> Result<Value, UbcError> {
        match stmt {
            Statement::Expr(expr) => self.evaluate_expr(expr),
            Statement::Assign { target, expr } => {
                let value = self.evaluate_expr(expr)?;
                if let Some(scope) = self.scopes.last_mut() {
                    scope.insert(target.clone(), value.clone());
                }
                Ok(value)
            }
        }
    }

    fn evaluate_expr(&mut self, expr: &Expr) -> Result<Value, UbcError> {
        match expr {
            Expr::Int(value) => Ok(Value::Int(*value)),
            Expr::Identifier(name) => self.lookup(name),
            Expr::Unary { op, expr } => {
                let value = self.evaluate_expr(expr)?;
                match (op, value) {
                    (UnaryOp::Negate, Value::Int(v)) => Ok(Value::Int(-v)),
                    _ => Err(UbcError::UnsupportedOperation("unary op".into())),
                }
            }
            Expr::Binary { op, left, right } => {
                let l = self.evaluate_expr(left)?;
                let r = self.evaluate_expr(right)?;
                match (op, l, r) {
                    (BinaryOp::Add, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
                    (BinaryOp::Subtract, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
                    (BinaryOp::Multiply, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
                    (BinaryOp::Divide, Value::Int(a), Value::Int(b)) => {
                        if b == 0 {
                            Err(UbcError::DivisionByZero)
                        } else {
                            Ok(Value::Int(a / b))
                        }
                    }
                    _ => Err(UbcError::UnsupportedOperation("binary op".into())),
                }
            }
            Expr::Brane(brane) => self.evaluate_brane(brane),
        }
    }

    fn evaluate_brane(&mut self, brane: &Brane) -> Result<Value, UbcError> {
        self.scopes.push(HashMap::new());
        let mut values = Vec::new();
        for stmt in &brane.statements {
            let value = self.execute_statement(stmt)?;
            values.push(value);
        }
        self.scopes.pop();
        if values.is_empty() {
            Ok(Value::None)
        } else {
            Ok(Value::Brane(values))
        }
    }

    fn lookup(&self, name: &str) -> Result<Value, UbcError> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Ok(value.clone());
            }
        }
        Err(UbcError::UnknownIdentifier(name.to_string()))
    }
}
