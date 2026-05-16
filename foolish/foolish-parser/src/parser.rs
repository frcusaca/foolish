use crate::ast::*;
use crate::lexer::Lexer;
use crate::token::{Token, TokenAndLocation};

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("expected {expected}, found {found} at line {line}, column {col}")]
    UnexpectedToken {
        expected: &'static str,
        found: String,
        line: u32,
        col: u32,
    },
    #[error("syntax error at line {line}, column {col}: {message}")]
    Syntax {
        message: String,
        line: u32,
        col: u32,
    },
    #[error("unexpected end of input at line {line}, column {col}")]
    Eof { line: u32, col: u32 },
}

type Result<T, E = ParseError> = std::result::Result<T, E>;

pub struct Parser {
    tokens: Vec<TokenAndLocation>,
    pos: usize,
}

pub fn parse(source: &str) -> Result<Vec<Astn>> {
    let tokens = Lexer::new(source).tokenize();
    let mut p = Parser { tokens, pos: 0 };
    p.parse_program()
}

impl Parser {
    fn loc(&self) -> (u32, u32) {
        self.current().map(|t| (t.line, t.column)).unwrap_or((0, 0))
    }

    fn current(&self) -> Option<&TokenAndLocation> {
        self.tokens.get(self.pos)
    }

    fn peek_token(&self) -> Option<&Token> {
        self.current().map(|t| &t.token)
    }

    fn advance(&mut self) -> Option<TokenAndLocation> {
        let tok = self.current().cloned();
        if tok.is_some() {
            self.pos += 1;
        }
        tok
    }

    fn expect(&mut self, expected: &Token) -> Result<TokenAndLocation> {
        let cur = self.current().cloned();
        match cur {
            Some(t) if t.token == *expected => self.advance().ok_or_else(|| ParseError::Eof { line: t.line, col: t.column }),
            Some(t) => Err(ParseError::UnexpectedToken {
                expected: "<token>",
                found: format!("{:?}", t.token),
                line: t.line,
                col: t.column,
            }),
            None => Err(ParseError::Eof { line: 0, col: 0 }),
        }
    }

    fn eat(&mut self, token: &Token) -> bool {
        if self.peek_token() == Some(token) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn at_eof(&self) -> bool {
        matches!(self.peek_token(), Some(&Token::Eof))
    }

    fn skip_comments(&mut self) {
        while matches!(self.peek_token(), Some(Token::LineComment | Token::BlockComment(_))) {
            self.advance();
        }
    }

    // --- program ---
    fn parse_program(&mut self) -> Result<Vec<Astn>> {
        if matches!(self.peek_token(), Some(Token::Shebang(_))) {
            self.advance();
        }
        self.skip_comments();
        let mut branes = Vec::new();
        while !self.at_eof() {
            self.skip_comments();
            if self.at_eof() { break; }
            branes.push(self.parse_brane()?);
        }
        Ok(branes)
    }

    // --- brane ---
    fn parse_brane(&mut self) -> Result<Astn> {
        if self.peek_token() == Some(&Token::LBracket) {
            return self.parse_detach_brane();
        }
        if self.peek_token() == Some(&Token::Up) {
            self.advance();
            return Ok(Astn::UpwardSearch);
        }
        let chars = self.parse_characterizations();
        self.expect(&Token::LBrace)?;
        let stmts = self.parse_brane_body()?;
        self.expect(&Token::RBrace)?;
        Ok(Astn::Brane {
            characterizations: chars,
            statements: stmts,
        })
    }

    fn parse_brane_with_chars(&mut self, chars: Vec<String>) -> Result<Astn> {
        self.expect(&Token::LBrace)?;
        let stmts = self.parse_brane_body()?;
        self.expect(&Token::RBrace)?;
        Ok(Astn::Brane {
            characterizations: chars,
            statements: stmts,
        })
    }

    // --- detach_brane: [ stmt_list ] brane? ---
    fn parse_detach_brane(&mut self) -> Result<Astn> {
        self.expect(&Token::LBracket)?;
        let mut stmts = Vec::new();
        while !self.at_eof() && self.peek_token() != Some(&Token::RBracket) {
            self.skip_comments();
            if self.peek_token() == Some(&Token::RBracket) { break; }
            stmts.push(self.parse_detach_stmt()?);
            self.skip_comments();
            if self.eat(&Token::Semicolon) || self.eat(&Token::Comma) {
                self.skip_comments();
            }
        }
        self.expect(&Token::RBracket)?;
        let body = if self.peek_token() == Some(&Token::LBrace) {
            Some(Box::new(self.parse_brane()?))
        } else {
            None
        };
        Ok(Astn::DetachmentBrane {
            statements: stmts,
            body,
        })
    }

    fn parse_detach_stmt(&mut self) -> Result<Astn> {
        let chars = self.parse_characterizations();
        let ident = self.parse_identifier()?;
        if self.peek_token() == Some(&Token::Assign) {
            self.advance();
            let expr = self.parse_expr()?;
            Ok(Astn::Assignment {
                characterizations: chars,
                identifier: ident,
                operator: AssignmentOperator::Assign,
                expr: Box::new(expr),
            })
        } else {
            Ok(Astn::Identifier {
                characterizations: chars,
                id: ident,
            })
        }
    }

    // --- characterizations: identifier'?* ---
    fn parse_characterizations(&mut self) -> Vec<String> {
        let mut chars = Vec::new();
        loop {
            if let Some(Token::Ident(name)) = self.peek_token() {
                let next_pos = self.pos + 1;
                if self.tokens.get(next_pos).map(|t| &t.token) == Some(&Token::Apostrophe) {
                    chars.push(name.clone());
                    self.advance(); // ident
                    self.advance(); // apostrophe
                    continue;
                }
            }
            break;
        }
        chars
    }

    // --- brane body: stmt* stmt_body? ---
    fn parse_brane_body(&mut self) -> Result<Vec<Astn>> {
        let mut stmts = Vec::new();
        while !self.at_eof() && self.peek_token() != Some(&Token::RBrace) {
            self.skip_comments();
            if self.at_eof() || self.peek_token() == Some(&Token::RBrace) {
                break;
            }
            stmts.push(self.parse_stmt_body()?);
            // Consume separators after stmt_body
            self.skip_comments();
            if self.peek_token().map(|t| matches!(t, Token::Semicolon | Token::Comma)).unwrap_or(false) {
                loop {
                    self.advance(); // semicolon/comma
                    self.skip_comments();
                    if self.peek_token().map(|t| matches!(t, Token::Semicolon | Token::Comma)).unwrap_or(false) {
                        continue;
                    }
                    break;
                }
            }
        }
        Ok(stmts)
    }

    // --- stmt_body: assignment | expr ---
    fn parse_stmt_body(&mut self) -> Result<Astn> {
        if self.is_assignment_start() {
            return self.parse_assignment();
        }
        self.parse_expr()
    }

    fn is_assignment_start(&self) -> bool {
        let mut pos = self.pos;
        // Skip characterizations
        loop {
            match self.tokens.get(pos) {
                Some(t) if matches!(t.token, Token::Ident(_)) => {
                    if self.tokens.get(pos + 1).map(|t| &t.token) == Some(&Token::Apostrophe) {
                        pos += 2;
                        continue;
                    }
                    break;
                }
                _ => return false,
            }
        }
        if !matches!(self.tokens.get(pos), Some(t) if matches!(t.token, Token::Ident(_))) {
            return false;
        }
        pos += 1;
        matches!(
            self.tokens.get(pos).map(|t| &t.token),
            Some(Token::Assign | Token::LtEqGt | Token::LtLtEqGtGt)
        )
    }

    // --- assignment ---
    fn parse_assignment(&mut self) -> Result<Astn> {
        let chars = self.parse_characterizations();
        let ident_tok = self.advance();
        let ident = match ident_tok {
            Some(TokenAndLocation { token: Token::Ident(s), .. }) => s,
            Some(t) => return Err(ParseError::UnexpectedToken {
                expected: "identifier",
                found: format!("{:?}", t.token),
                line: t.line,
                col: t.column,
            }),
            None => return Err(ParseError::Eof { line: 0, col: 0 }),
        };

        let op = match self.peek_token() {
            Some(Token::LtLtEqGtGt) => { self.advance(); AssignmentOperator::SFF },
            Some(Token::LtEqGt) => { self.advance(); AssignmentOperator::SF },
            Some(Token::Assign) => {
                self.advance();
                match self.peek_token() {
                    Some(Token::Dollar) => {
                        self.advance();
                        let inner = self.parse_expr()?;
                        return Ok(Astn::Assignment {
                            characterizations: chars,
                            identifier: ident,
                            operator: AssignmentOperator::Assign,
                            expr: Box::new(Astn::BinaryOp {
                                op: "$".into(),
                                left: Box::new(Astn::UnanchoredSeek { offset: -1 }),
                                right: Box::new(inner),
                            }),
                        });
                    }
                    Some(Token::Caret) => {
                        self.advance();
                        let inner = self.parse_expr()?;
                        return Ok(Astn::Assignment {
                            characterizations: chars,
                            identifier: ident,
                            operator: AssignmentOperator::Assign,
                            expr: Box::new(Astn::BinaryOp {
                                op: "^".into(),
                                left: Box::new(Astn::UnanchoredSeek { offset: -1 }),
                                right: Box::new(inner),
                            }),
                        });
                    }
                    _ => AssignmentOperator::Assign,
                }
            }
            _ => return self.parse_expr(),
        };

        let expr = self.parse_expr()?;
        Ok(Astn::Assignment {
            characterizations: chars,
            identifier: ident,
            operator: op,
            expr: Box::new(expr),
        })
    }

    // --- expr: addExpr | ifExpr | concatenation ---
    fn parse_expr(&mut self) -> Result<Astn> {
        if self.peek_token() == Some(&Token::If) {
            return self.parse_if_expr();
        }
        let mut first = self.parse_add_expr()?;
        while self.is_concatenation_continuation() {
            let second = self.parse_concatenation_element()?;
            if let Astn::Concatenation { mut elements } = first {
                elements.push(second);
                first = Astn::Concatenation { elements };
            } else {
                first = Astn::Concatenation {
                    elements: vec![first, second],
                };
            }
        }
        Ok(first)
    }

    fn is_concatenation_continuation(&self) -> bool {
        let current_token = match self.current() {
            Some(t) => t.clone(),
            None => return false,
        };
        match current_token.token {
            Token::LBrace | Token::Ident(_) | Token::Up => {
                let Some(prev_idx) = self.pos.checked_sub(1) else {
                    return false;
                };
                self.tokens.get(prev_idx)
                    .map(|t| t.line == current_token.line)
                    .unwrap_or(false)
            }
            _ => false,
        }
    }

    fn parse_concatenation_element(&mut self) -> Result<Astn> {
        if self.peek_token() == Some(&Token::Up) {
            return self.parse_brane();
        }
        if self.peek_token() == Some(&Token::LBracket) {
            return self.parse_brane();
        }
        if self.peek_token() == Some(&Token::LBrace) {
            return self.parse_brane();
        }
        self.parse_postfix_expr()
    }

    // --- addExpr ---
    fn parse_add_expr(&mut self) -> Result<Astn> {
        let mut left = self.parse_mul_expr()?;
        loop {
            match self.peek_token() {
                Some(Token::Plus) => {
                    self.advance();
                    let right = self.parse_mul_expr()?;
                    left = Astn::BinaryOp {
                        op: "+".into(),
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                Some(Token::Minus) => {
                    self.advance();
                    let right = self.parse_mul_expr()?;
                    left = Astn::BinaryOp {
                        op: "-".into(),
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }
        Ok(left)
    }

    // --- mulExpr ---
    fn parse_mul_expr(&mut self) -> Result<Astn> {
        let mut left = self.parse_unary_expr()?;
        loop {
            match self.peek_token() {
                Some(Token::Mul) => {
                    self.advance();
                    let right = self.parse_unary_expr()?;
                    left = Astn::BinaryOp {
                        op: "*".into(),
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                Some(Token::Div) => {
                    self.advance();
                    let right = self.parse_unary_expr()?;
                    left = Astn::BinaryOp {
                        op: "/".into(),
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }
        Ok(left)
    }

    // --- unaryExpr ---
    fn parse_unary_expr(&mut self) -> Result<Astn> {
        match self.peek_token() {
            Some(Token::Plus) => {
                self.advance();
                let expr = self.parse_postfix_expr()?;
                Ok(Astn::UnaryOp { op: "+".into(), expr: Box::new(expr) })
            }
            Some(Token::Minus) => {
                self.advance();
                let expr = self.parse_postfix_expr()?;
                Ok(Astn::UnaryOp { op: "-".into(), expr: Box::new(expr) })
            }
            Some(Token::Mul) => {
                self.advance();
                let expr = self.parse_postfix_expr()?;
                Ok(Astn::UnaryOp { op: "*".into(), expr: Box::new(expr) })
            }
            _ => self.parse_postfix_expr(),
        }
    }

    // --- postfixExpr: primary (postfix_op)* ---
    fn parse_postfix_expr(&mut self) -> Result<Astn> {
        let mut expr = self.parse_primary()?;
        loop {
            match self.peek_token() {
                Some(Token::Dot) => {
                    self.advance();
                    let coord = self.parse_identifier_or_regexp()?;
                    expr = Astn::DotSearch {
                        anchor: Box::new(expr),
                        coordinate: coord,
                    };
                }
                Some(Token::Question) => {
                    self.advance();
                    let pattern = self.parse_regexp_pattern()?;
                    expr = Astn::RegexpSearch {
                        anchor: Box::new(expr),
                        operator: SearchOperator::RegexpLocal,
                        pattern,
                    };
                }
                Some(Token::Tilde) => {
                    self.advance();
                    let pattern = self.parse_regexp_pattern()?;
                    expr = Astn::RegexpSearch {
                        anchor: Box::new(expr),
                        operator: SearchOperator::RegexpForward,
                        pattern,
                    };
                }
                Some(Token::Hash) => {
                    self.advance();
                    let offset = self.parse_seek_index()?;
                    expr = Astn::Seek {
                        anchor: Box::new(expr),
                        offset,
                    };
                }
                Some(Token::Caret) => {
                    self.advance();
                    expr = Astn::HeadTail {
                        is_head: true,
                        anchor: Box::new(expr),
                    };
                }
                Some(Token::Dollar) => {
                    self.advance();
                    expr = Astn::HeadTail {
                        is_head: false,
                        anchor: Box::new(expr),
                    };
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    // --- regexp_expression ---
    fn parse_regexp_pattern(&mut self) -> Result<String> {
        let mut pattern = String::new();
        while !self.at_eof() {
            match self.peek_token() {
                Some(
                    Token::Semicolon | Token::Comma | Token::RBrace
                    | Token::RParen | Token::RBracket | Token::Eof
                    | Token::LineComment,
                ) => break,
                Some(Token::Ident(s)) => {
                    pattern.push_str(s);
                    self.advance();
                }
                Some(Token::Integer(n)) => {
                    pattern.push_str(&n.to_string());
                    self.advance();
                }
                Some(Token::LParen) => {
                    pattern.push('(');
                    self.advance();
                    loop {
                        if self.peek_token() == Some(&Token::RParen) {
                            pattern.push(')');
                            self.advance();
                            break;
                        }
                        if self.at_eof() { break; }
                        if let Some(Token::Ident(s)) = self.peek_token() {
                            pattern.push_str(s);
                        } else if let Some(Token::Integer(n)) = self.peek_token() {
                            pattern.push_str(&n.to_string());
                        } else if let Some(t) = self.peek_token() {
                            pattern.push_str(&t.to_string());
                        }
                        self.advance();
                    }
                }
                Some(Token::LBrace) => {
                    pattern.push('{');
                    self.advance();
                    loop {
                        if self.peek_token() == Some(&Token::RBrace) {
                            pattern.push('}');
                            self.advance();
                            break;
                        }
                        if self.at_eof() { break; }
                        if let Some(t) = self.peek_token() {
                            pattern.push_str(&t.to_string());
                        }
                        self.advance();
                    }
                }
                Some(Token::LBracket) => {
                    pattern.push('[');
                    self.advance();
                    loop {
                        if self.peek_token() == Some(&Token::RBracket) {
                            pattern.push(']');
                            self.advance();
                            break;
                        }
                        if self.at_eof() { break; }
                        if let Some(t) = self.peek_token() {
                            pattern.push_str(&t.to_string());
                        }
                        self.advance();
                    }
                }
                Some(Token::Apostrophe) => {
                    pattern.push('\'');
                    self.advance();
                }
                Some(t) => {
                    // Operators and other tokens that can be part of regex patterns
                    pattern.push_str(&t.to_string());
                    self.advance();
                }
                None => break,
            }
        }
        Ok(pattern)
    }

    fn parse_identifier_or_regexp(&mut self) -> Result<String> {
        let chars = self.parse_characterizations();
        let id = self.parse_identifier()?;
        let mut coord = chars.join("'");
        coord.push_str(&id);
        Ok(coord)
    }

    fn parse_identifier(&mut self) -> Result<String> {
        match self.advance() {
            Some(TokenAndLocation { token: Token::Ident(s), .. }) => Ok(s),
            Some(t) => Err(ParseError::UnexpectedToken {
                expected: "identifier",
                found: format!("{:?}", t.token),
                line: t.line,
                col: t.column,
            }),
            None => Err(ParseError::Eof { line: 0, col: 0 }),
        }
    }

    fn parse_seek_index(&mut self) -> Result<i32> {
        let mut neg = 1i32;
        if self.eat(&Token::Minus) {
            neg = -1;
        }
        match self.advance() {
            Some(TokenAndLocation { token: Token::Integer(n), .. }) => Ok(neg * n as i32),
            Some(t) => Err(ParseError::UnexpectedToken {
                expected: "integer",
                found: format!("{:?}", t.token),
                line: t.line,
                col: t.column,
            }),
            None => Err(ParseError::Eof { line: 0, col: 0 }),
        }
    }

    // --- primary ---
    fn parse_primary(&mut self) -> Result<Astn> {
        let token_clone = self.peek_token().cloned();
        match token_clone {
            Some(Token::LBrace) => self.parse_brane(),
            Some(Token::LBracket) => self.parse_detach_brane(),
            Some(Token::Up) => { self.advance(); Ok(Astn::UpwardSearch) },
            Some(Token::LParen) => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(&Token::RParen)?;
                Ok(expr)
            }
            Some(Token::LtLt) => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(&Token::GtGt)?;
                Ok(Astn::StayFullyFoolish { expr: Box::new(expr) })
            }
            Some(Token::Lt) => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(&Token::Gt)?;
                Ok(Astn::StayFoolish { expr: Box::new(expr) })
            }
            Some(Token::Integer(n)) => {
                self.advance();
                Ok(Astn::IntLit(n))
            }
            Some(Token::Unknown) => {
                self.advance();
                Ok(Astn::UnknownLit)
            }
            Some(Token::Hash) => {
                self.advance();
                let offset = self.parse_seek_index()?;
                if offset >= 0 {
                    return Err(ParseError::Syntax {
                        message: "Unanchored seek must be negative".into(),
                        line: self.loc().0,
                        col: self.loc().1,
                    });
                }
                Ok(Astn::UnanchoredSeek { offset })
            }
            Some(Token::Ident(_)) => {
                let chars = self.parse_characterizations();
                if !chars.is_empty() && self.peek_token() == Some(&Token::LBrace) {
                    // name'{ ... } is a characterized brane
                    return self.parse_brane_with_chars(chars);
                }
                let id = self.parse_identifier()?;
                Ok(Astn::Identifier {
                    characterizations: chars,
                    id,
                })
            }
            Some(tok) => Err(ParseError::UnexpectedToken {
                expected: "primary expression",
                found: format!("{:?}", tok),
                line: self.loc().0,
                col: self.loc().1,
            }),
            None => Err(ParseError::Eof { line: 0, col: 0 }),
        }
    }

    // --- ifExpr ---
    fn parse_if_expr(&mut self) -> Result<Astn> {
        self.expect(&Token::If)?;
        let condition = self.parse_expr()?;
        self.expect(&Token::Then)?;
        let then_body = self.parse_expr()?;
        let mut elif_clauses = Vec::new();
        loop {
            if self.peek_token() == Some(&Token::Elif) {
                self.advance();
                let cond = self.parse_expr()?;
                self.expect(&Token::Then)?;
                let body = self.parse_expr()?;
                elif_clauses.push((Box::new(cond), Box::new(body)));
            } else {
                break;
            }
        }
        let else_body = if self.peek_token() == Some(&Token::Else) {
            self.advance();
            Some(Box::new(self.parse_expr()?))
        } else {
            None
        };
        if self.peek_token() == Some(&Token::Fi) {
            self.advance();
        }
        Ok(Astn::IfExpr {
            condition: Box::new(condition),
            then_body: Box::new(then_body),
            elif_clauses,
            else_body,
        })
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::LBrace => write!(f, "{{"),
            Token::RBrace => write!(f, "}}"),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::LBracket => write!(f, "["),
            Token::RBracket => write!(f, "]"),
            Token::Semicolon => write!(f, ";"),
            Token::Comma => write!(f, ","),
            Token::Assign => write!(f, "="),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Mul => write!(f, "*"),
            Token::Div => write!(f, "/"),
            Token::Dot => write!(f, "."),
            Token::DotDot => write!(f, ".."),
            Token::Caret => write!(f, "^"),
            Token::Dollar => write!(f, "$"),
            Token::Question => write!(f, "?"),
            Token::QuestionQuestion => write!(f, "??"),
            Token::Tilde => write!(f, "~"),
            Token::TildeTilde => write!(f, "~~"),
            Token::Hash => write!(f, "#"),
            Token::Lt => write!(f, "<"),
            Token::Gt => write!(f, ">"),
            Token::LtEqGt => write!(f, "<=>"),
            Token::LtLt => write!(f, "<<"),
            Token::GtGt => write!(f, ">>"),
            Token::LtLtEqGtGt => write!(f, "<<=>>>"),
            Token::Apostrophe => write!(f, "'"),
            Token::Integer(n) => write!(f, "{}", n),
            Token::Ident(s) => write!(f, "{}", s),
            Token::Shebang(s) => write!(f, "#!{}", s),
            Token::LineComment => write!(f, "!!"),
            Token::BlockComment(s) => write!(f, "!!!{}!!!", s),
            Token::Unknown => write!(f, "???"),
            Token::Up => write!(f, "↑"),
            Token::If => write!(f, "if"),
            Token::Then => write!(f, "then"),
            Token::Elif => write!(f, "elif"),
            Token::Else => write!(f, "else"),
            Token::Fi => write!(f, "fi"),
            Token::Eof => write!(f, "<EOF>"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_single(source: &str) -> Result<Astn> {
        parse(source).map(|branes| branes.into_iter().next().unwrap())
    }

    #[test]
    fn parses_empty_brane() {
        let ast = parse_single("{}").unwrap();
        match ast {
            Astn::Brane { statements, .. } => assert!(statements.is_empty()),
            _ => panic!("expected brane"),
        }
    }

    #[test]
    fn parses_integer_literal() {
        let ast = parse_single("{42;}").unwrap();
        match ast {
            Astn::Brane { statements, .. } => {
                assert_eq!(statements.len(), 1);
                assert_eq!(statements[0], Astn::IntLit(42));
            }
            _ => panic!("expected brane"),
        }
    }

    #[test]
    fn parses_addition() {
        let ast = parse_single("{3 + 4;}").unwrap();
        match ast {
            Astn::Brane { statements, .. } => {
                assert_eq!(statements.len(), 1);
                match &statements[0] {
                    Astn::BinaryOp { op, .. } => assert_eq!(op, "+"),
                    _ => panic!("expected binary op"),
                }
            }
            _ => panic!("expected brane"),
        }
    }

    #[test]
    fn parses_assignment() {
        let ast = parse_single("{x = 42;}").unwrap();
        match ast {
            Astn::Brane { statements, .. } => {
                assert_eq!(statements.len(), 1);
                match &statements[0] {
                    Astn::Assignment { identifier, .. } => assert_eq!(identifier, "x"),
                    _ => panic!("expected assignment"),
                }
            }
            _ => panic!("expected brane"),
        }
    }

    #[test]
    fn parses_bare_identifier() {
        let ast = parse_single("{x;}").unwrap();
        match ast {
            Astn::Brane { statements, .. } => {
                assert_eq!(statements.len(), 1);
                match &statements[0] {
                    Astn::Identifier { id, .. } => assert_eq!(id, "x"),
                    _ => panic!("expected identifier"),
                }
            }
            _ => panic!("expected brane"),
        }
    }

    #[test]
    fn parses_nested_brane() {
        let ast = parse_single("{ {1; 2;}; }").unwrap();
        match ast {
            Astn::Brane { statements, .. } => {
                assert_eq!(statements.len(), 1);
                assert!(matches!(&statements[0], Astn::Brane { .. }));
            }
            _ => panic!("expected brane"),
        }
    }

    #[test]
    fn parses_dot_search() {
        let ast = parse_single("{x.y;}").unwrap();
        match ast {
            Astn::Brane { statements, .. } => {
                assert_eq!(statements.len(), 1);
                assert!(matches!(&statements[0], Astn::DotSearch { .. }));
            }
            _ => panic!("expected brane"),
        }
    }

    #[test]
    fn parses_head_tail() {
        let ast = parse_single("{x^;}").unwrap();
        match ast {
            Astn::Brane { statements, .. } => {
                assert!(matches!(&statements[0], Astn::HeadTail { is_head: true, .. }));
            }
            _ => panic!("expected brane"),
        }
    }

    #[test]
    fn parses_shebang() {
        let asts = parse("#!/usr/bin/env foolish\n{x = 1;}").unwrap();
        assert_eq!(asts.len(), 1);
    }

    #[test]
    fn parses_unanchored_seek() {
        let ast = parse_single("{a = 1; b = #-1;}").unwrap();
        match ast {
            Astn::Brane { statements, .. } => {
                assert_eq!(statements.len(), 2);
            }
            _ => panic!("expected brane"),
        }
    }

    #[test]
    fn parses_regexp_search() {
        let ast = parse_single("{brn?pattern;}").unwrap();
        match ast {
            Astn::Brane { statements, .. } => {
                assert!(matches!(&statements[0], Astn::RegexpSearch { .. }));
            }
            _ => panic!("expected brane"),
        }
    }

    #[test]
    fn parses_characterizations() {
        let ast = parse_single("{outer'{ x = 1; };}").unwrap();
        match ast {
            Astn::Brane { statements, .. } => {
                match &statements[0] {
                    Astn::Brane { characterizations, .. } => {
                        assert_eq!(characterizations, &["outer".to_string()]);
                    }
                    _ => panic!("expected characterized brane"),
                }
            }
            _ => panic!("expected brane"),
        }
    }

    #[test]
    fn parses_concatenation() {
        let ast = parse_single("{c = {a=1} {b=2};}").unwrap();
        match ast {
            Astn::Brane { statements, .. } => {
                match &statements[0] {
                    Astn::Assignment { expr, .. } => {
                        assert!(matches!(&**expr, Astn::Concatenation { .. }));
                    }
                    _ => panic!("expected assignment with concatenation"),
                }
            }
            _ => panic!("expected brane"),
        }
    }
}
