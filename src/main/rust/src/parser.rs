use std::fmt;

use antlr_rust::common_token_stream::CommonTokenStream;
use antlr_rust::error_strategy::BailErrorStrategy;
use antlr_rust::errors::ANTLRError;
use antlr_rust::token::Token;
use antlr_rust::token_factory::CommonTokenFactory;
use antlr_rust::tree::{ParseTree, TerminalNode, Tree};
use antlr_rust::InputStream;
use antlr_rust::TidExt;

use crate::antlr::foolishlexer::FoolishLexer;
use crate::antlr::foolishparser::{
    self, AddExprContextAll, AddExprContextAttrs, AssignmentContextAll, AssignmentContextAttrs,
    BraneContextAll, BraneContextAttrs, BranesContextAll, BranesContextAttrs,
    CharacterizableContextAll, CharacterizableContextAttrs, Characterizable_identifierContextAll,
    Characterizable_identifierContextAttrs, ExprContextAll, ExprContextAttrs, LiteralContextAll,
    LiteralContextAttrs, MulExprContextAll, MulExprContextAttrs, PostfixExprContextAll,
    PostfixExprContextAttrs, PrimaryContextAll, PrimaryContextAttrs, ProgramContextAll,
    ProgramContextAttrs, Standard_braneContextAll, Standard_braneContextAttrs, StmtContextAll,
    StmtContextAttrs, UnaryExprContextAll, UnaryExprContextAttrs,
};

use crate::ast::{BinaryOp, Brane, Expr, Program, Statement, UnaryOp};

#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    Syntax {
        line: usize,
        column: usize,
        message: String,
    },
    Unsupported(String),
    Internal(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::Syntax {
                line,
                column,
                message,
            } => {
                write!(f, "{}:{} {}", line, column, message)
            }
            ParseError::Unsupported(message) => write!(f, "Unsupported construct: {}", message),
            ParseError::Internal(message) => write!(f, "Internal parser error: {}", message),
        }
    }
}

impl std::error::Error for ParseError {}

impl From<ANTLRError> for ParseError {
    fn from(err: ANTLRError) -> Self {
        let message = format!("{:?}", err);
        if let Some(token) = err.get_offending_token() {
            let line = token.line.max(0) as usize;
            let column = token.column.max(0) as usize;
            ParseError::Syntax {
                line,
                column,
                message,
            }
        } else {
            ParseError::Internal(message)
        }
    }
}

pub fn parse_program(source: &str) -> Result<Program, ParseError> {
    let tf = CommonTokenFactory::default();
    let input = InputStream::new(source);
    let lexer = FoolishLexer::new_with_token_factory(input, &tf);
    let token_stream = CommonTokenStream::new(lexer);
    let mut parser =
        foolishparser::FoolishParser::with_strategy(token_stream, BailErrorStrategy::new());
    let tree = parser.program().map_err(ParseError::from)?;
    build_program(&tree)
}

fn build_program<'input>(ctx: &ProgramContextAll<'input>) -> Result<Program, ParseError> {
    let branes_ctx = ctx
        .branes()
        .ok_or_else(|| ParseError::Unsupported("expected brane".into()))?;
    let brane = build_branes(&branes_ctx)?;
    Ok(Program { brane })
}

fn build_branes<'input>(ctx: &BranesContextAll<'input>) -> Result<Brane, ParseError> {
    let branes = ctx.brane_all();
    if branes.len() != 1 {
        return Err(ParseError::Unsupported(
            "multiple branes are not currently supported".into(),
        ));
    }
    build_brane(&branes[0])
}

fn build_brane<'input>(ctx: &BraneContextAll<'input>) -> Result<Brane, ParseError> {
    if let Some(standard) = ctx.standard_brane() {
        build_standard_brane(&standard)
    } else {
        Err(ParseError::Unsupported(
            "only standard branes are supported".into(),
        ))
    }
}

fn build_standard_brane<'input>(
    ctx: &Standard_braneContextAll<'input>,
) -> Result<Brane, ParseError> {
    let mut statements = Vec::new();
    for stmt_ctx in ctx.stmt_all() {
        statements.push(build_statement(&stmt_ctx)?);
    }
    Ok(Brane { statements })
}

fn build_statement<'input>(ctx: &StmtContextAll<'input>) -> Result<Statement, ParseError> {
    if let Some(assign_ctx) = ctx.assignment() {
        build_assignment(&assign_ctx)
    } else if let Some(expr_ctx) = ctx.expr() {
        Ok(Statement::Expr(build_expr(&expr_ctx)?))
    } else {
        Err(ParseError::Unsupported("empty statement".into()))
    }
}

fn build_assignment<'input>(ctx: &AssignmentContextAll<'input>) -> Result<Statement, ParseError> {
    let target_ctx = ctx
        .characterizable_identifier()
        .ok_or_else(|| ParseError::Unsupported("missing assignment target".into()))?;
    let target = extract_identifier(&target_ctx)?;
    let expr_ctx = ctx
        .expr()
        .ok_or_else(|| ParseError::Unsupported("missing assignment expression".into()))?;
    let expr = build_expr(&expr_ctx)?;
    Ok(Statement::Assign { target, expr })
}

fn build_expr<'input>(ctx: &ExprContextAll<'input>) -> Result<Expr, ParseError> {
    if let Some(add_ctx) = ctx.addExpr() {
        build_add_expr(&add_ctx)
    } else if let Some(brane_ctx) = ctx.branes() {
        let brane = build_branes(&brane_ctx)?;
        Ok(Expr::Brane(Box::new(brane)))
    } else {
        Err(ParseError::Unsupported(
            "if expressions are not supported".into(),
        ))
    }
}

fn build_add_expr<'input>(ctx: &AddExprContextAll<'input>) -> Result<Expr, ParseError> {
    let terms = ctx.mulExpr_all();
    let first = terms
        .get(0)
        .ok_or_else(|| ParseError::Unsupported("missing expression".into()))?;
    let mut expr = build_mul_expr(first)?;

    for index in 1..terms.len() {
        let operator_child_index = index * 2 - 1;
        let operator_node = ctx
            .get_child(operator_child_index)
            .ok_or_else(|| ParseError::Unsupported("expected operator".into()))?
            .downcast_rc::<TerminalNode<'input, foolishparser::FoolishParserContextType>>()
            .map_err(|_| ParseError::Unsupported("expected operator token".into()))?;
        let token_type = operator_node.symbol.get_token_type();
        let op = match token_type {
            foolishparser::PLUS => BinaryOp::Add,
            foolishparser::MINUS => BinaryOp::Subtract,
            _ => {
                return Err(ParseError::Unsupported(format!(
                    "unsupported additive operator token {}",
                    token_type
                )))
            }
        };
        let right = build_mul_expr(&terms[index])?;
        expr = Expr::Binary {
            op,
            left: Box::new(expr),
            right: Box::new(right),
        };
    }

    Ok(expr)
}

fn build_mul_expr<'input>(ctx: &MulExprContextAll<'input>) -> Result<Expr, ParseError> {
    let factors = ctx.unaryExpr_all();
    let first = factors
        .get(0)
        .ok_or_else(|| ParseError::Unsupported("missing expression".into()))?;
    let mut expr = build_unary_expr(first)?;

    for index in 1..factors.len() {
        let operator_child_index = index * 2 - 1;
        let operator_node = ctx
            .get_child(operator_child_index)
            .ok_or_else(|| ParseError::Unsupported("expected operator".into()))?
            .downcast_rc::<TerminalNode<'input, foolishparser::FoolishParserContextType>>()
            .map_err(|_| ParseError::Unsupported("expected operator token".into()))?;
        let token_type = operator_node.symbol.get_token_type();
        let op = match token_type {
            foolishparser::MUL => BinaryOp::Multiply,
            foolishparser::DIV => BinaryOp::Divide,
            _ => {
                return Err(ParseError::Unsupported(format!(
                    "unsupported multiplicative operator token {}",
                    token_type
                )))
            }
        };
        let right = build_unary_expr(&factors[index])?;
        expr = Expr::Binary {
            op,
            left: Box::new(expr),
            right: Box::new(right),
        };
    }

    Ok(expr)
}

fn build_unary_expr<'input>(ctx: &UnaryExprContextAll<'input>) -> Result<Expr, ParseError> {
    let postfix_ctx = ctx
        .postfixExpr()
        .ok_or_else(|| ParseError::Unsupported("missing expression".into()))?;
    let mut expr = build_postfix_expr(&postfix_ctx)?;

    if ctx.MINUS().is_some() {
        expr = Expr::Unary {
            op: UnaryOp::Negate,
            expr: Box::new(expr),
        };
    } else if ctx.MUL().is_some() {
        return Err(ParseError::Unsupported("unary '*' is not supported".into()));
    }

    Ok(expr)
}

fn build_postfix_expr<'input>(ctx: &PostfixExprContextAll<'input>) -> Result<Expr, ParseError> {
    if !ctx.DOT_all().is_empty() {
        return Err(ParseError::Unsupported(
            "member access is not supported".into(),
        ));
    }
    let primary_ctx = ctx
        .primary()
        .ok_or_else(|| ParseError::Unsupported("missing primary expression".into()))?;
    build_primary(&primary_ctx)
}

fn build_primary<'input>(ctx: &PrimaryContextAll<'input>) -> Result<Expr, ParseError> {
    if let Some(characterizable_ctx) = ctx.characterizable() {
        build_characterizable(&characterizable_ctx)
    } else if let Some(expr_ctx) = ctx.expr() {
        build_expr(&expr_ctx)
    } else if ctx.UNKNOWN().is_some() {
        Err(ParseError::Unsupported("unknown literal".into()))
    } else {
        Err(ParseError::Unsupported("unrecognized primary".into()))
    }
}

fn build_characterizable<'input>(
    ctx: &CharacterizableContextAll<'input>,
) -> Result<Expr, ParseError> {
    if ctx.APOSTROPHE().is_some() {
        return Err(ParseError::Unsupported(
            "prefixed characterizable values".into(),
        ));
    }

    if let Some(identifier_ctx) = ctx.characterizable_identifier() {
        Ok(Expr::Identifier(extract_identifier(&identifier_ctx)?))
    } else if let Some(literal_ctx) = ctx.literal() {
        build_literal(&literal_ctx)
    } else if let Some(brane_ctx) = ctx.brane() {
        let brane = build_brane(&brane_ctx)?;
        Ok(Expr::Brane(Box::new(brane)))
    } else {
        Err(ParseError::Unsupported(
            "unsupported characterizable".into(),
        ))
    }
}

fn build_literal<'input>(ctx: &LiteralContextAll<'input>) -> Result<Expr, ParseError> {
    let token = ctx
        .INTEGER()
        .ok_or_else(|| ParseError::Unsupported("missing integer literal".into()))?;
    let text = token.symbol.get_text().to_owned();
    let value = text
        .parse::<i64>()
        .map_err(|_| ParseError::Unsupported(format!("invalid integer literal '{}'", text)))?;
    Ok(Expr::Int(value))
}

fn extract_identifier<'input>(
    ctx: &Characterizable_identifierContextAll<'input>,
) -> Result<String, ParseError> {
    if ctx.APOSTROPHE().is_some() {
        return Err(ParseError::Unsupported(
            "apostrophe-delimited identifiers are not supported".into(),
        ));
    }
    Ok(ctx.get_text())
}
