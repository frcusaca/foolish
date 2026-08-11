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
            Some(t) if t.token == *expected => self.advance().ok_or(ParseError::Eof {
                line: t.line,
                col: t.column,
            }),
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
        while matches!(
            self.peek_token(),
            Some(Token::LineComment | Token::BlockComment(_))
        ) {
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
            if self.at_eof() {
                break;
            }
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
            if self.peek_token() == Some(&Token::RBracket) {
                break;
            }
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
        // Handle leading apostrophe: 'name is a null-characterized name
        if self.peek_token() == Some(&Token::Apostrophe) {
            chars.push(String::new());
            self.advance();
        }
        loop {
            match self.peek_token() {
                Some(Token::Ident(name)) => {
                    if self.tokens.get(self.pos + 1).map(|t| &t.token) == Some(&Token::Apostrophe) {
                        chars.push(name.clone());
                        self.advance();
                        self.advance();
                        continue;
                    }
                    break;
                }
                Some(Token::Apostrophe) => {
                    chars.push(String::new());
                    self.advance();
                    continue;
                }
                _ => break,
            }
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
            if self
                .peek_token()
                .map(|t| matches!(t, Token::Semicolon | Token::Comma))
                .unwrap_or(false)
            {
                loop {
                    self.advance(); // semicolon/comma
                    self.skip_comments();
                    if self
                        .peek_token()
                        .map(|t| matches!(t, Token::Semicolon | Token::Comma))
                        .unwrap_or(false)
                    {
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
        // Handle leading apostrophe: 'name
        if self.tokens.get(pos).map(|t| &t.token) == Some(&Token::Apostrophe) {
            pos += 1;
        }
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
                // '' — consecutive apostrophe (null characterization)
                Some(t) if t.token == Token::Apostrophe => {
                    pos += 1;
                    continue;
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

    /// Is the current token the start of an **attached search** (FOOP-75 §1)?
    ///
    /// Two conditions, both required:
    /// - the token begins a search operator — `^ $ ~ ? .`, including the
    ///   value forms `~=` and `?=`;
    /// - it is **adjacent** to the `=` just consumed, i.e. no whitespace
    ///   intervenes (FOOP-75 §5.1(1)).
    ///
    /// Adjacency comes from [`TokenAndLocation::preceded_by_space`], not from
    /// `column` — see FOOP-75 §5.2/§5.3 for why `column` cannot answer this.
    ///
    /// # Two operators are deliberately excluded
    ///
    /// **`&`** (§5.4): a cursor-source modifier, not a search operator in its
    /// own right; what position a contexted search would read when anchored
    /// on a whole RHS is undecided.
    ///
    /// **`#`**: alone among the trigger set, `#` also *begins a complete
    /// standalone expression* — [`Astn::UnanchoredSeek`]. Every other trigger
    /// (`$ ^ ~ ? . ~= ?=`) is a suffix operator that cannot start an
    /// expression, so no ambiguity arises for them. For `#` it does:
    ///
    /// ```foolish
    /// z = #-2 + #-1     !! a SUM of two unanchored seeks  ← the real meaning
    /// z =#-2 + #-1      !! or an attached `#-2` over RHS `#-1`?
    /// ```
    ///
    /// The einmo corpus settles it: **every** `=#` occurrence is an
    /// unanchored seek (`foop/9/operator_search_transparency.foo:1`,
    /// `misc/complex_unanchored_seeks_with_operations.foo:1`,
    /// `misc/unanchored_seek.foo`, `misc/seek_beyond_start.foo`,
    /// `misc/seek_negative_clamping.foo`, `misc/unanchored_seek_large_negative.foo`,
    /// `foop/42/…hfs.foo:37,69`) — none is an attached search. Claiming `#`
    /// changed `z=#-2 + #-1` from `BinaryOp(+, #-2, #-1)` into
    /// `Seek { anchor: #-1, offset: -2 }`, regressing two baselines.
    ///
    /// The positional index remains reachable attached in a CHAIN, where no
    /// ambiguity exists because a suffix operator opens the run:
    /// `A =$#1 B` is fine. Only a run *starting* with `#` is excluded.
    fn at_attached_search(&self) -> bool {
        let Some(tok) = self.current() else {
            return false;
        };
        if tok.preceded_by_space {
            return false;
        }
        matches!(
            tok.token,
            Token::Caret
                | Token::Dollar
                | Token::Tilde
                | Token::TildeEquals
                | Token::Question
                | Token::QuestionEquals
                | Token::Dot
        )
    }

    /// Index of the first token after the attached-search run, or `None` when
    /// this is not an attached search after all.
    ///
    /// FOOP-75 §5.1(2)/(3): the run is the maximal sequence of **adjacent**
    /// tokens (a search specification may not contain spaces — AGENTS.md
    /// §Searches), and it **must** end with a space followed by an RHS.
    ///
    /// A run that ends at `;`, `,`, `}`, `)`, `]` or EOF is **not** an
    /// attached search — it is an ordinary RHS that merely begins with a
    /// search-operator character. This distinction is load-bearing, because
    /// `#` also begins [`Astn::UnanchoredSeek`], a legitimate standalone
    /// expression:
    ///
    /// ```foolish
    /// seek_unanchored = #-2;   !! an unanchored seek, NOT an attached search
    /// tail_of_b       =$ b;    !! an attached search
    /// ```
    ///
    /// Returning `None` here (rather than an error) lets the caller fall
    /// through to ordinary expression parsing, which is what makes the
    /// attached-search path a strict addition.
    fn attached_search_run_end(&self) -> Option<usize> {
        let mut i = self.pos + 1;
        while let Some(tok) = self.tokens.get(i) {
            // A space ends the run (§5.1(2)) — and so does a statement or
            // grouping terminator, which ends the STATEMENT and therefore
            // cannot be part of its attached search. Without the second
            // condition the scan runs through `;` into the next statement,
            // because a `;` is not itself preceded by a space:
            //     seek_unanchored=#-2;
            //     sf_target={a=1;b=2};
            // would take `#-2;` as the run and `sf_target` as its RHS.
            if tok.preceded_by_space
                || matches!(
                    tok.token,
                    Token::Eof
                        | Token::Semicolon
                        | Token::Comma
                        | Token::RBrace
                        | Token::RParen
                        | Token::RBracket
                )
            {
                break;
            }
            i += 1;
        }
        match self.tokens.get(i).map(|t| &t.token) {
            None
            | Some(
                Token::Eof
                | Token::Semicolon
                | Token::Comma
                | Token::RBrace
                | Token::RParen
                | Token::RBracket,
            ) => None,
            _ => Some(i),
        }
    }

    // --- assignment ---
    fn parse_assignment(&mut self) -> Result<Astn> {
        let chars = self.parse_characterizations();
        let ident_tok = self.advance();
        let ident = match ident_tok {
            Some(TokenAndLocation {
                token: Token::Ident(s),
                ..
            }) => s,
            Some(t) => {
                return Err(ParseError::UnexpectedToken {
                    expected: "identifier",
                    found: format!("{:?}", t.token),
                    line: t.line,
                    col: t.column,
                });
            }
            None => return Err(ParseError::Eof { line: 0, col: 0 }),
        };

        let op = match self.peek_token() {
            Some(Token::LtLtEqGtGt) => {
                self.advance();
                AssignmentOperator::SFF
            }
            Some(Token::LtEqGt) => {
                self.advance();
                AssignmentOperator::SF
            }
            Some(Token::Assign) => {
                self.advance();
                // FOOP-75 §1/§2: an ATTACHED SEARCH — a search written
                // immediately after the `=` with no intervening space.
                // `A =SPEC B` is defined as `A = B SPEC`.
                //
                // This replaces the two bespoke `=$` / `=^` branches that used
                // to live here (which built a synthetic
                // `UnanchoredSeek { offset: -1 }` left operand). Those were
                // measurably wrong: `=$` yielded the whole brane rather than
                // its tail, and `=^` had no evaluator arm at all and leaked
                // `Op^(...)` into rendered output. Routing every operator
                // through the ordinary postfix path dissolves both — FOOP-75 §7.
                if let Some(rhs_start) = self
                    .at_attached_search()
                    .then(|| self.attached_search_run_end())
                    .flatten()
                {
                    let suffix_start = self.pos;

                    // Parse the RHS first, then rewind and replay the recorded
                    // suffix against it using the SAME routine the postfix
                    // spelling uses — this is what guarantees §2 tree identity.
                    self.pos = rhs_start;
                    let rhs = self.parse_expr()?;
                    let after_rhs = self.pos;

                    self.pos = suffix_start;
                    let expr = self.apply_search_suffixes(rhs)?;
                    if self.pos != rhs_start {
                        // The suffix parser and the run scanner disagree on
                        // where the specification ends. This happens for the
                        // greedy scanners (`.`, and `?`/`~` patterns), which
                        // run past a space into the RHS — the FOOP-75 §6
                        // limitation. Refuse rather than mis-parse silently.
                        let (line, col) = self.loc();
                        return Err(ParseError::Syntax {
                            message: "attached search specification is ambiguous: \
                                      its pattern or coordinate runs past the \
                                      terminating space (FOOP-75 §6) — write the \
                                      postfix form, or parenthesize the pattern"
                                .into(),
                            line,
                            col,
                        });
                    }
                    self.pos = after_rhs;

                    return Ok(Astn::Assignment {
                        characterizations: chars,
                        identifier: ident,
                        operator: AssignmentOperator::Assign,
                        expr: Box::new(expr),
                    });
                }
                AssignmentOperator::Assign
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
            Token::LBrace
            | Token::LParen
            | Token::Ident(_)
            | Token::Up
            | Token::LtLt
            | Token::Lt => {
                let Some(prev_idx) = self.pos.checked_sub(1) else {
                    return false;
                };
                self.tokens
                    .get(prev_idx)
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

    // --- arithExpr: arithmetic only, NO search suffixes ---
    // Used for value patterns (spec: "value_pattern := arith_expr", no trailing
    // search suffixes).  Calls parse_primary() directly instead of
    // parse_postfix_expr(), so `~=`/`?=`/`&`/`.`/`?`/`~`/`#`/`^`/`$` are NOT consumed.
    fn parse_arith_expr(&mut self) -> Result<Astn> {
        let mut left = self.parse_arith_mul_expr()?;
        loop {
            match self.peek_token() {
                Some(Token::Plus) => {
                    self.advance();
                    let right = self.parse_arith_mul_expr()?;
                    left = Astn::BinaryOp {
                        op: "+".into(),
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                Some(Token::Minus) => {
                    self.advance();
                    let right = self.parse_arith_mul_expr()?;
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

    fn parse_arith_mul_expr(&mut self) -> Result<Astn> {
        let mut left = self.parse_arith_unary_expr()?;
        loop {
            match self.peek_token() {
                Some(Token::Mul) => {
                    self.advance();
                    let right = self.parse_arith_unary_expr()?;
                    left = Astn::BinaryOp {
                        op: "*".into(),
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                Some(Token::Div) => {
                    self.advance();
                    let right = self.parse_arith_unary_expr()?;
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

    fn parse_arith_unary_expr(&mut self) -> Result<Astn> {
        match self.peek_token() {
            Some(Token::Plus) => {
                self.advance();
                let expr = self.parse_primary()?;
                Ok(Astn::UnaryOp {
                    op: "+".into(),
                    expr: Box::new(expr),
                })
            }
            Some(Token::Minus) => {
                self.advance();
                let expr = self.parse_primary()?;
                Ok(Astn::UnaryOp {
                    op: "-".into(),
                    expr: Box::new(expr),
                })
            }
            _ => self.parse_primary(),
        }
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
                Ok(Astn::UnaryOp {
                    op: "+".into(),
                    expr: Box::new(expr),
                })
            }
            Some(Token::Minus) => {
                self.advance();
                let expr = self.parse_postfix_expr()?;
                Ok(Astn::UnaryOp {
                    op: "-".into(),
                    expr: Box::new(expr),
                })
            }
            Some(Token::Mul) => {
                self.advance();
                let expr = self.parse_postfix_expr()?;
                Ok(Astn::UnaryOp {
                    op: "*".into(),
                    expr: Box::new(expr),
                })
            }
            _ => self.parse_postfix_expr(),
        }
    }

    // --- postfixExpr: primary (postfix_op)* ---
    fn parse_postfix_expr(&mut self) -> Result<Astn> {
        let expr = self.parse_primary()?;
        self.apply_search_suffixes(expr)
    }

    /// Apply a run of search suffixes to an already-parsed anchor expression.
    ///
    /// This is the suffix half of [`Self::parse_postfix_expr`], factored out so
    /// FOOP-75's **attached searches** can replay the *same* code path against
    /// the RHS of an assignment. That reuse is what makes FOOP-75 §2's tree
    /// identity (`A =SPEC B` builds the same tree as `A = B SPEC`) true by
    /// construction rather than by two implementations happening to agree.
    ///
    /// Chained suffixes build a left-nested spine through each node's `anchor`,
    /// so the leftmost suffix in the source is the innermost node.
    fn apply_search_suffixes(&mut self, anchor: Astn) -> Result<Astn> {
        let mut expr = anchor;
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
                    if self.eat(&Token::Assign) {
                        let value_pattern = self.parse_arith_expr()?;
                        expr = Astn::ValueSearch {
                            anchor: Some(Box::new(expr)),
                            forward: false,
                            name_pattern: Some(pattern),
                            value_pattern: Box::new(value_pattern),
                        };
                    } else {
                        expr = Astn::RegexpSearch {
                            anchor: Some(Box::new(expr)),
                            operator: SearchOperator::RegexpLocal,
                            pattern,
                        };
                    }
                }
                Some(Token::QuestionEquals) => {
                    self.advance();
                    let value_pattern = self.parse_arith_expr()?;
                    expr = Astn::ValueSearch {
                        anchor: Some(Box::new(expr)),
                        forward: false,
                        name_pattern: None,
                        value_pattern: Box::new(value_pattern),
                    };
                }
                Some(Token::Tilde) => {
                    self.advance();
                    let pattern = self.parse_regexp_pattern()?;
                    if self.eat(&Token::Assign) {
                        let value_pattern = self.parse_arith_expr()?;
                        expr = Astn::ValueSearch {
                            anchor: Some(Box::new(expr)),
                            forward: true,
                            name_pattern: Some(pattern),
                            value_pattern: Box::new(value_pattern),
                        };
                    } else {
                        expr = Astn::RegexpSearch {
                            anchor: Some(Box::new(expr)),
                            operator: SearchOperator::RegexpForward,
                            pattern,
                        };
                    }
                }
                Some(Token::TildeEquals) => {
                    self.advance();
                    let value_pattern = self.parse_arith_expr()?;
                    expr = Astn::ValueSearch {
                        anchor: Some(Box::new(expr)),
                        forward: true,
                        name_pattern: None,
                        value_pattern: Box::new(value_pattern),
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
                Some(Token::Ampersand) => {
                    self.advance();
                    let inner = match self.peek_token() {
                        Some(Token::Question) => {
                            self.advance();
                            let pattern = self.parse_regexp_pattern()?;
                            if self.eat(&Token::Assign) {
                                let value_pattern = self.parse_arith_expr()?;
                                Astn::ValueSearch {
                                    anchor: Some(Box::new(expr)),
                                    forward: false,
                                    name_pattern: Some(pattern),
                                    value_pattern: Box::new(value_pattern),
                                }
                            } else {
                                Astn::RegexpSearch {
                                    anchor: Some(Box::new(expr)),
                                    operator: SearchOperator::RegexpLocal,
                                    pattern,
                                }
                            }
                        }
                        Some(Token::Tilde) => {
                            self.advance();
                            let pattern = self.parse_regexp_pattern()?;
                            if self.eat(&Token::Assign) {
                                let value_pattern = self.parse_arith_expr()?;
                                Astn::ValueSearch {
                                    anchor: Some(Box::new(expr)),
                                    forward: true,
                                    name_pattern: Some(pattern),
                                    value_pattern: Box::new(value_pattern),
                                }
                            } else {
                                Astn::RegexpSearch {
                                    anchor: Some(Box::new(expr)),
                                    operator: SearchOperator::RegexpForward,
                                    pattern,
                                }
                            }
                        }
                        Some(Token::Hash) => {
                            self.advance();
                            let offset = self.parse_seek_index()?;
                            Astn::Seek {
                                anchor: Box::new(expr),
                                offset,
                            }
                        }
                        Some(Token::Caret) => {
                            self.advance();
                            Astn::HeadTail {
                                is_head: true,
                                anchor: Box::new(expr),
                            }
                        }
                        Some(Token::Dollar) => {
                            self.advance();
                            Astn::HeadTail {
                                is_head: false,
                                anchor: Box::new(expr),
                            }
                        }
                        Some(Token::TildeEquals) => {
                            self.advance();
                            let value_pattern = self.parse_arith_expr()?;
                            Astn::ValueSearch {
                                anchor: Some(Box::new(expr)),
                                forward: true,
                                name_pattern: None,
                                value_pattern: Box::new(value_pattern),
                            }
                        }
                        Some(Token::QuestionEquals) => {
                            self.advance();
                            let value_pattern = self.parse_arith_expr()?;
                            Astn::ValueSearch {
                                anchor: Some(Box::new(expr)),
                                forward: false,
                                name_pattern: None,
                                value_pattern: Box::new(value_pattern),
                            }
                        }
                        Some(Token::Dot) => {
                            return Err(ParseError::Syntax {
                                message: "&. is not a valid operator".into(),
                                line: self.loc().0,
                                col: self.loc().1,
                            });
                        }
                        _ => {
                            return Err(ParseError::Syntax {
                                message: "expected search operator after &".into(),
                                line: self.loc().0,
                                col: self.loc().1,
                            });
                        }
                    };
                    expr = Astn::ContextedSearch {
                        inner: Box::new(inner),
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
                    Token::Semicolon
                    | Token::Comma
                    | Token::RBrace
                    | Token::RParen
                    | Token::RBracket
                    | Token::Eof
                    | Token::LineComment
                    | Token::Assign
                    | Token::Ampersand,
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
                        if self.at_eof() {
                            break;
                        }
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
                        if self.at_eof() {
                            break;
                        }
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
                        if self.at_eof() {
                            break;
                        }
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
        // Each characterization component gets a trailing `'`, not a `'`
        // BETWEEN components (`join` loses a leading null characterization:
        // `'b` parses to chars=[""], and `[""].join("'")` is `""`, silently
        // dropping the null-characterization marker entirely). Matches
        // `Identifier::from_parts`'s `characterization_string` construction
        // (`foolish-ubca/src/identifier.rs`), the authoritative algorithm.
        let mut coord: String = chars.iter().map(|c| format!("{c}'")).collect();
        coord.push_str(&id);
        Ok(coord)
    }

    fn parse_identifier(&mut self) -> Result<String> {
        match self.advance() {
            Some(TokenAndLocation {
                token: Token::Ident(s),
                ..
            }) => Ok(s),
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
            Some(TokenAndLocation {
                token: Token::Integer(n),
                ..
            }) => Ok(neg * n as i32),
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
            Some(Token::Up) => {
                self.advance();
                Ok(Astn::UpwardSearch)
            }
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
                Ok(Astn::StayFullyFoolish {
                    expr: Box::new(expr),
                })
            }
            Some(Token::Lt) => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(&Token::Gt)?;
                Ok(Astn::StayFoolish {
                    expr: Box::new(expr),
                })
            }
            Some(Token::Integer(n)) => {
                self.advance();
                Ok(Astn::IntLit(n))
            }
            Some(Token::Unknown) => {
                self.advance();
                Ok(Astn::UnknownLit)
            }
            Some(Token::Creation) => {
                self.advance();
                Ok(Astn::Creation)
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
            Some(Token::QuestionEquals) => {
                self.advance();
                let value_pattern = self.parse_arith_expr()?;
                Ok(Astn::ValueSearch {
                    anchor: None,
                    forward: false,
                    name_pattern: None,
                    value_pattern: Box::new(value_pattern),
                })
            }
            Some(Token::Question) => {
                self.advance();
                let pattern = self.parse_regexp_pattern()?;
                if self.eat(&Token::Assign) {
                    let value_pattern = self.parse_arith_expr()?;
                    Ok(Astn::ValueSearch {
                        anchor: None,
                        forward: false,
                        name_pattern: Some(pattern),
                        value_pattern: Box::new(value_pattern),
                    })
                } else {
                    // Bare unanchored backward search: searches the current brane (IB),
                    // then climbs ancestor branes (AB) — mirrors the unanchored
                    // ValueSearch arm just above, NOT a literal empty-brane anchor.
                    Ok(Astn::RegexpSearch {
                        anchor: None,
                        operator: SearchOperator::RegexpLocal,
                        pattern,
                    })
                }
            }
            Some(Token::Tilde) => {
                self.advance();
                let pattern = self.parse_regexp_pattern()?;
                if self.eat(&Token::Assign) {
                    let value_pattern = self.parse_arith_expr()?;
                    Ok(Astn::ValueSearch {
                        anchor: None,
                        forward: true,
                        name_pattern: Some(pattern),
                        value_pattern: Box::new(value_pattern),
                    })
                } else {
                    // Bare unanchored FORWARD search (FOOP-55 §6), the twin of the
                    // `?` arm above. Both scan the same candidate window — the home
                    // brane's statements BEFORE this one — but in opposite
                    // directions: `?` finds the nearest preceding match, `~` the
                    // earliest. This does NOT look forward into unsettled
                    // statements (FOOP-23 §A.1); the window ends before the
                    // searching statement.
                    Ok(Astn::RegexpSearch {
                        anchor: None,
                        operator: SearchOperator::RegexpForward,
                        pattern,
                    })
                }
            }
            Some(Token::Apostrophe) => {
                let chars = self.parse_characterizations();
                let id = self.parse_identifier()?;
                Ok(Astn::Identifier {
                    characterizations: chars,
                    id,
                })
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
            Some(Token::Ampersand) => Err(ParseError::Syntax {
                message: "& requires a preceding expression".into(),
                line: self.loc().0,
                col: self.loc().1,
            }),
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
            Token::QuestionEquals => write!(f, "?="),
            Token::Tilde => write!(f, "~"),
            Token::TildeTilde => write!(f, "~~"),
            Token::TildeEquals => write!(f, "~="),
            Token::Hash => write!(f, "#"),
            Token::Ampersand => write!(f, "&"),
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
            Token::Creation => write!(f, "\u{2B24}"),
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

    // ── FOOP-75: Assignment Attached Searches ──────────────────────────
    // An ATTACHED SEARCH is a search written immediately after a statement's
    // `=`, with no intervening space, terminated by a space:
    //     A =$ B      is defined as      A = B$
    // See docs/foop/FOOP-75.md §2 (the rewrite) and §5 (the space rule).

    /// FOOP-75 §2: `LHS =SEARCH_SPEC RHS` builds the SAME tree as
    /// `LHS = RHS SEARCH_SPEC`. This single property IS the parse-direction
    /// specification — if it holds for an operator, that operator needs no
    /// further parse test. `Astn` derives `PartialEq`, so it is assertable
    /// directly rather than approximated by structural spot-checks.
    ///
    /// `?x` / `~x` (bare name patterns) are absent deliberately — their
    /// pattern scanner runs past the terminating space into the RHS, so they
    /// are refused rather than mis-parsed. See
    /// `foop75_pins_bare_name_pattern_limitation` (§6). `.x` IS covered here:
    /// its coordinate scanner stops at the space, so the identity holds.
    #[test]
    fn foop75_attached_search_builds_same_tree_as_postfix() {
        let pairs = [
            ("{B={1,2,3}; A =$ B;}", "{B={1,2,3}; A = B$;}"),
            ("{B={1,2,3}; A =^ B;}", "{B={1,2,3}; A = B^;}"),
            ("{B={1,2,3}; A =.x B;}", "{B={1,2,3}; A = B.x;}"),
            ("{B={1,2,3}; A =~=5 B;}", "{B={1,2,3}; A = B~=5;}"),
            ("{B={1,2,3}; A =?=5 B;}", "{B={1,2,3}; A = B?=5;}"),
        ];
        for (attached, postfix) in pairs {
            let a = parse_single(attached)
                .unwrap_or_else(|e| panic!("attached form failed to parse: {attached}: {e:?}"));
            let p = parse_single(postfix)
                .unwrap_or_else(|e| panic!("postfix form failed to parse: {postfix}: {e:?}"));
            assert_eq!(
                a, p,
                "trees differ:\n  attached: {attached}\n  postfix:  {postfix}"
            );
        }
    }

    /// FOOP-75 §3: a CHAIN of attached searches builds the same left-nested
    /// spine as the equivalent postfix chain. The leftmost search in the
    /// attached sequence is the INNERMOST node of the spine.
    #[test]
    fn foop75_attached_search_chains_match_postfix_chains() {
        // Note every chain here OPENS with a suffix operator (`$`/`^`), never
        // with `#`. A run starting with `#` is not an attached search at all
        // -- see `at_attached_search` -- but `#` is perfectly usable *inside*
        // a chain, which is what these pin.
        let pairs = [
            ("{B={1,2,3}; A =$#1 B;}", "{B={1,2,3}; A = B$#1;}"),
            ("{B={1,2,3}; A =^#1 B;}", "{B={1,2,3}; A = B^#1;}"),
            ("{B={1,2,3}; A =$#-2 B;}", "{B={1,2,3}; A = B$#-2;}"),
            // From the corpus: test-resources/.../test_syntax.foo:4 already
            // contains `c =$#-1;` -- an attached chain written before this
            // FOOP existed. Found by FOOP-75's Phase 1 survey.
            ("{a=1; b=2; c =$#-1 b;}", "{a=1; b=2; c = b$#-1;}"),
        ];
        for (attached, postfix) in pairs {
            let a = parse_single(attached).expect(attached);
            let p = parse_single(postfix).expect(postfix);
            assert_eq!(a, p, "chain trees differ: {attached} vs {postfix}");
        }
    }

    /// PINS A KNOWN LIMITATION — not desired behavior. FOOP-75 §6.
    ///
    /// `parse_regexp_pattern` breaks on `;`, `,`, `}`, `)`, `]`, EOF, line
    /// comment, `=` and `&` — but **not on a space**. So in `A =?x B` the
    /// pattern scanner runs past the terminating space and swallows `B`,
    /// yielding the pattern `"xB"`.
    ///
    /// FOOP-75 §5 says a space terminates a search specification; making the
    /// pattern scanner honor that is §6.2's job, and it changes the meaning of
    /// existing programs (the §6.3 survey found 3 such lines), so it is NOT
    /// done here. Until then the attached form is **refused** — the run
    /// scanner and the suffix parser disagree on where the specification ends,
    /// and refusing beats silently mis-parsing.
    ///
    /// Note `.` is NOT affected: its coordinate scanner stops at the space, so
    /// `A =.x B` works and is covered by the tree-identity test above.
    ///
    /// If you are reading this because the test failed: you are changing
    /// pattern-boundary semantics. Confirm the §6.3 survey was done and the
    /// change is intended — do not "fix" this test.
    #[test]
    fn foop75_pins_bare_name_pattern_limitation() {
        for src in ["{B={1,2,3}; A =?x B;}", "{B={1,2,3}; A =~x B;}"] {
            let attached = parse_single(src);
            assert!(
                attached.is_err(),
                "§6: a bare name-pattern attached search must be refused, not \
                 silently mis-parsed; got {attached:?} for {src}"
            );
        }
    }

    /// FOOP-75 §5.1(1): attachment requires ADJACENCY. `= $` (a space after
    /// the `=`) has NO attached search -- the `$` belongs to the RHS.
    ///
    /// This is what the Phase 2 lexer change (`preceded_by_space`) exists to
    /// make decidable: on jia@dc6db093 these two lexed identically.
    #[test]
    fn foop75_space_after_equals_means_no_attached_search() {
        let attached = parse_single("{B={1,2,3}; A =$ B;}").expect("attached form parses");
        match parse_single("{B={1,2,3}; A = $ B;}") {
            Ok(t) => assert_ne!(
                t, attached,
                "`= $` must NOT be treated as an attached search (§5.1(1))"
            ),
            Err(_) => { /* also acceptable: `= $` may simply be a parse error */ }
        }
    }

    /// FOOP-75 §5.1(3): an attached search MUST be terminated by a space
    /// followed by the RHS it applies to. A run that instead ends at `;`,
    /// `,`, `}`, `)` or EOF is **not an attached search at all** — it is an
    /// ordinary RHS that merely starts with a search-operator character.
    ///
    /// This distinction is load-bearing rather than pedantic: `#` also
    /// begins an UNANCHORED SEEK, which is a legitimate standalone RHS.
    /// `seek_unanchored = #-2;` appears in the einmo corpus
    /// (`foop/42/…hfs.foo:37`) and must keep parsing as `UnanchoredSeek`.
    /// An earlier draft of this FOOP raised a parse error here and broke
    /// that baseline — the suite caught it.
    #[test]
    fn foop75_unterminated_run_is_not_an_attached_search() {
        // `#-2` as an entire RHS is an unanchored seek, not an attached search.
        let t = parse_single("{a=1; b=2; seek = #-2;}").expect("unanchored seek still parses");
        match t {
            Astn::Brane { statements, .. } => match &statements[2] {
                Astn::Assignment { expr, .. } => assert!(
                    matches!(**expr, Astn::UnanchoredSeek { offset: -2 }),
                    "expected UnanchoredSeek, got {expr:?}"
                ),
                other => panic!("expected assignment, got {other:?}"),
            },
            other => panic!("expected brane, got {other:?}"),
        }

        // The same shape written adjacent to the `=` is likewise NOT attached
        // (nothing follows the run but `;`), so it parses as the seek too.
        let t = parse_single("{a=1; b=2; seek=#-2;}").expect("adjacent unanchored seek parses");
        match t {
            Astn::Brane { statements, .. } => match &statements[2] {
                Astn::Assignment { expr, .. } => assert!(
                    matches!(**expr, Astn::UnanchoredSeek { offset: -2 }),
                    "expected UnanchoredSeek, got {expr:?}"
                ),
                other => panic!("expected assignment, got {other:?}"),
            },
            other => panic!("expected brane, got {other:?}"),
        }

        // `A =$;` has no RHS for the `$` to apply to; it is not attached, and
        // the ordinary parser then rejects a bare `$` as a primary expression.
        assert!(
            parse_single("{B={1,2,3}; A =$;}").is_err(),
            "a bare `$` with no RHS is not a valid expression"
        );
    }

    /// FOOP-75 §5.1(2): the attached-search run stops at a STATEMENT
    /// terminator as well as at a space.
    ///
    /// Regression guard for a bug the einmo suite caught (and the
    /// single-statement unit tests did not): a `;` is not itself preceded by
    /// a space, so a run scan that breaks only on spaces walks straight
    /// through it into the following statement. In
    /// `foop/42/…hfs.foo:37-38` that made
    ///
    /// ```foolish
    /// seek_unanchored=#-2;
    /// sf_target={a=1;b=2};
    /// ```
    ///
    /// read as "attached search `#-2;` applied to RHS `sf_target`", failing
    /// the whole file. `#-2` here is an UNANCHORED SEEK and the statement
    /// ends at the `;`.
    #[test]
    fn foop75_attached_run_stops_at_statement_terminator() {
        let src = "{index_brane={1,2};\n\
                   index_out_of_bounds=index_brane#99;\n\
                   seek_unanchored=#-2;\n\
                   sf_target={a=1;b=2};\n\
                   }";
        let t = parse_single(src).expect("multi-statement seek must parse");
        match t {
            Astn::Brane { statements, .. } => {
                assert_eq!(statements.len(), 4, "all four statements survive");
                match &statements[2] {
                    Astn::Assignment {
                        expr, identifier, ..
                    } => {
                        assert_eq!(identifier, "seekˍunanchored");
                        assert!(
                            matches!(**expr, Astn::UnanchoredSeek { offset: -2 }),
                            "expected UnanchoredSeek, got {expr:?}"
                        );
                    }
                    other => panic!("expected assignment, got {other:?}"),
                }
            }
            other => panic!("expected brane, got {other:?}"),
        }
    }

    /// `#` is NOT an attached-search trigger — see `at_attached_search`.
    ///
    /// Alone among the trigger set, `#` also begins a complete standalone
    /// expression (`UnanchoredSeek`), so `z=#-2 + #-1` is ambiguous. The
    /// einmo corpus settles it: every `=#` occurrence is a seek, none is an
    /// attached search.
    ///
    /// Regression guard: claiming `#` turned `z=#-2 + #-1` into
    /// `Seek { anchor: #-1, offset: -2 }` (ANCHORED, stuck at BRANING),
    /// breaking `foop/9/operator_search_transparency` and
    /// `misc/complex_unanchored_seeks_with_operations`.
    #[test]
    fn foop75_hash_is_not_an_attached_search_trigger() {
        // Adjacent `=#` with a following operator is a SUM of two seeks.
        let t = parse_single("{x=5, y=7, z=#-2 + #-1;}").expect("seek arithmetic parses");
        match t {
            Astn::Brane { statements, .. } => match &statements[2] {
                Astn::Assignment { expr, .. } => match &**expr {
                    Astn::BinaryOp { op, left, right } => {
                        assert_eq!(op, "+");
                        assert!(matches!(**left, Astn::UnanchoredSeek { offset: -2 }));
                        assert!(matches!(**right, Astn::UnanchoredSeek { offset: -1 }));
                    }
                    other => panic!("expected BinaryOp(+), got {other:?}"),
                },
                other => panic!("expected assignment, got {other:?}"),
            },
            other => panic!("expected brane, got {other:?}"),
        }

        // Spaced and adjacent forms agree, because neither is attached.
        assert_eq!(
            parse_single("{x=5, y=7, z=#-2 + #-1;}").unwrap(),
            parse_single("{x=5, y=7, z= #-2 + #-1;}").unwrap(),
            "`=#` and `= #` must parse identically -- `#` is not a trigger"
        );
    }

    /// FOOP-75 §5.4: `&` is NOT in the trigger set. `&` is a cursor-source
    /// modifier, not a search operator in its own right (AGENTS.md §Searches
    /// group 2), and what position a contexted search would read from when
    /// anchored on a whole RHS has no obviously correct answer.
    #[test]
    fn foop75_ampersand_is_not_an_attached_search_trigger() {
        let attached_like = parse_single("{B={1,2,3}; A =&?x B;}");
        let postfix = parse_single("{B={1,2,3}; A = B&?x;}");
        if let (Ok(a), Ok(p)) = (attached_like, postfix) {
            assert_ne!(
                a, p,
                "`=&?x` must not be rewritten as an attached search (§5.4)"
            );
        }
    }

    /// FOOP-75 §2: an ordinary assignment is untouched. The attached-search
    /// path must be a strict addition -- everything that parsed before parses
    /// identically.
    #[test]
    fn foop75_ordinary_assignment_unaffected() {
        let t = parse_single("{a = 1; b = a + 2;}").expect("ordinary assignment parses");
        match t {
            Astn::Brane { statements, .. } => {
                assert_eq!(statements.len(), 2);
                assert!(matches!(statements[0], Astn::Assignment { .. }));
            }
            other => panic!("expected brane, got {other:?}"),
        }
    }

    #[test]
    fn brane_literal_dollar_reads_the_whole_literals_tail() {
        // FOOP-33 Phase 6 research task ("$ vs concatenation precedence"):
        // the SETTLED syntax (§5.0's evening revision) is a brane LITERAL
        // with 'lt as a comma-separated member -- NOT postfix-concatenation
        // (`{1,3}'lt$`, from the superseded historical prose, does NOT even
        // parse as intended -- see git history for that investigation).
        // {1, 2, 'lt}$ must parse as ({1, 2, 'lt})$ -- $ (tail) applied to
        // the WHOLE brane literal -- not to 'lt alone.
        let ast = parse_single("{r = {1, 2, 'lt}$;}").unwrap();
        match ast {
            Astn::Brane { statements, .. } => match &statements[0] {
                Astn::Assignment { expr, .. } => match &**expr {
                    Astn::HeadTail {
                        is_head: false,
                        anchor,
                    } => match &**anchor {
                        Astn::Brane { statements, .. } => {
                            assert_eq!(statements.len(), 3);
                            assert!(matches!(statements[0], Astn::IntLit(1)));
                            assert!(matches!(statements[1], Astn::IntLit(2)));
                            assert!(
                                matches!(&statements[2], Astn::Identifier { id, .. } if id == "lt")
                            );
                        }
                        other => panic!(
                            "expected the $ anchor to be the WHOLE brane literal, got {other:?}"
                        ),
                    },
                    other => panic!("expected HeadTail (tail search), got {other:?}"),
                },
                other => panic!("expected assignment, got {other:?}"),
            },
            _ => panic!("expected brane"),
        }
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
    fn parses_dot_search_coordinate_preserves_null_characterization() {
        // `x.'y` must produce coordinate `"'y"`, not `"y"` -- the leading
        // apostrophe (null characterization) was previously lost because
        // `parse_identifier_or_regexp` used `chars.join("'")`, which puts `'`
        // BETWEEN elements. For chars=[""] (what a leading apostrophe parses
        // to), `[""].join("'")` is `""`, silently dropping the marker. Fixed
        // to match `Identifier::from_parts`'s per-component-suffix algorithm
        // (`foolish-ubca/src/identifier.rs`): each component gets a trailing
        // `'`, so `[""]` becomes `"'"`, giving coordinate `"'y"`.
        let ast = parse_single("{x.'y;}").unwrap();
        match ast {
            Astn::Brane { statements, .. } => {
                assert_eq!(statements.len(), 1);
                match &statements[0] {
                    Astn::DotSearch { coordinate, .. } => {
                        assert_eq!(coordinate, "'y");
                    }
                    other => panic!("expected DotSearch, got {other:?}"),
                }
            }
            _ => panic!("expected brane"),
        }
    }

    #[test]
    fn parses_dot_search_coordinate_with_named_characterization() {
        // `x.a'y` (a NON-null characterization) must produce `"a'y"` --
        // exercises the multi-component join path, not just the empty-string
        // edge case.
        let ast = parse_single("{x.a'y;}").unwrap();
        match ast {
            Astn::Brane { statements, .. } => match &statements[0] {
                Astn::DotSearch { coordinate, .. } => {
                    assert_eq!(coordinate, "a'y");
                }
                other => panic!("expected DotSearch, got {other:?}"),
            },
            _ => panic!("expected brane"),
        }
    }

    #[test]
    fn parses_head_tail() {
        let ast = parse_single("{x^;}").unwrap();
        match ast {
            Astn::Brane { statements, .. } => {
                assert!(matches!(
                    &statements[0],
                    Astn::HeadTail { is_head: true, .. }
                ));
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
            Astn::Brane { statements, .. } => match &statements[0] {
                Astn::Brane {
                    characterizations, ..
                } => {
                    assert_eq!(characterizations, &["outer".to_string()]);
                }
                _ => panic!("expected characterized brane"),
            },
            _ => panic!("expected brane"),
        }
    }

    #[test]
    fn parses_concatenation() {
        let ast = parse_single("{c = {a=1} {b=2};}").unwrap();
        match ast {
            Astn::Brane { statements, .. } => match &statements[0] {
                Astn::Assignment { expr, .. } => {
                    assert!(matches!(&**expr, Astn::Concatenation { .. }));
                }
                _ => panic!("expected assignment with concatenation"),
            },
            _ => panic!("expected brane"),
        }
    }

    #[test]
    fn parses_parenthesized_concatenation() {
        let ast = parse_single("{r = b1(target.c);}").unwrap();
        match ast {
            Astn::Brane { statements, .. } => match &statements[0] {
                Astn::Assignment { expr, .. } => {
                    assert!(matches!(&**expr, Astn::Concatenation { .. }));
                }
                _ => panic!("expected assignment with concatenation"),
            },
            _ => panic!("expected brane"),
        }
    }

    #[test]
    fn parses_value_search_forward() {
        let ast = parse_single("{a~=10;}").unwrap();
        match ast {
            Astn::Brane { statements, .. } => match &statements[0] {
                Astn::ValueSearch {
                    forward,
                    name_pattern,
                    ..
                } => {
                    assert!(*forward);
                    assert!(name_pattern.is_none());
                }
                other => panic!("expected ValueSearch, got {:?}", other),
            },
            _ => panic!("expected brane"),
        }
    }

    #[test]
    fn parses_value_search_backward() {
        let ast = parse_single("{a?=10;}").unwrap();
        match ast {
            Astn::Brane { statements, .. } => match &statements[0] {
                Astn::ValueSearch {
                    forward,
                    name_pattern,
                    ..
                } => {
                    assert!(!*forward);
                    assert!(name_pattern.is_none());
                }
                other => panic!("expected ValueSearch, got {:?}", other),
            },
            _ => panic!("expected brane"),
        }
    }

    #[test]
    fn parses_regexp_search_bare_unanchored() {
        // Bare `?pattern` (nothing before the `?`) must carry `anchor: None` — a
        // real "no anchor" AST shape, not a hardcoded empty Brane{} literal (the
        // FOOP-33 regression this test pins). See `parses_regexp_search` above
        // for the anchored form (`brn?pattern`), which is unaffected.
        let ast = parse_single("{found = ?pattern;}").unwrap();
        match ast {
            Astn::Brane { statements, .. } => match &statements[0] {
                Astn::Assignment { expr, .. } => match &**expr {
                    Astn::RegexpSearch {
                        anchor,
                        operator,
                        pattern,
                    } => {
                        assert!(anchor.is_none());
                        assert_eq!(*operator, SearchOperator::RegexpLocal);
                        assert_eq!(pattern, "pattern");
                    }
                    other => panic!("expected RegexpSearch, got {:?}", other),
                },
                _ => panic!("expected assignment"),
            },
            _ => panic!("expected brane"),
        }
    }

    #[test]
    fn parses_value_search_unanchored() {
        let ast = parse_single("{found = ?=3;}").unwrap();
        match ast {
            Astn::Brane { statements, .. } => match &statements[0] {
                Astn::Assignment { expr, .. } => match &**expr {
                    Astn::ValueSearch {
                        anchor,
                        forward,
                        name_pattern,
                        ..
                    } => {
                        assert!(anchor.is_none());
                        assert!(!forward);
                        assert!(name_pattern.is_none());
                    }
                    other => panic!("expected ValueSearch, got {:?}", other),
                },
                _ => panic!("expected assignment"),
            },
            _ => panic!("expected brane"),
        }
    }

    #[test]
    fn parses_value_search_combined_name_value() {
        let ast = parse_single("{a~name=10;}").unwrap();
        match ast {
            Astn::Brane { statements, .. } => match &statements[0] {
                Astn::ValueSearch {
                    forward,
                    name_pattern,
                    ..
                } => {
                    assert!(*forward);
                    assert_eq!(name_pattern.as_deref(), Some("name"));
                }
                other => panic!("expected ValueSearch, got {:?}", other),
            },
            _ => panic!("expected brane"),
        }
    }

    #[test]
    fn parses_star_brane_as_creation() {
        let ast = parse_single("{x = {*};}").unwrap();
        match ast {
            Astn::Brane { statements, .. } => {
                assert_eq!(statements.len(), 1);
                match &statements[0] {
                    Astn::Assignment { expr, .. } => {
                        assert!(matches!(**expr, Astn::Creation));
                    }
                    other => panic!("expected Assignment, got {:?}", other),
                }
            }
            _ => panic!("expected brane"),
        }
    }

    #[test]
    fn parses_unicode_creation() {
        let ast = parse_single("{x = \u{2B24};}").unwrap();
        match ast {
            Astn::Brane { statements, .. } => {
                assert_eq!(statements.len(), 1);
                match &statements[0] {
                    Astn::Assignment { expr, .. } => {
                        assert!(matches!(**expr, Astn::Creation));
                    }
                    other => panic!("expected Assignment, got {:?}", other),
                }
            }
            _ => panic!("expected brane"),
        }
    }
}
