use crate::token::{Token, TokenAndLocation};

pub struct Lexer {
    chars: Vec<char>,
    pos: usize,
    line: u32,
    column: u32,
}

/// Canonical identifier separator: U+02CD (modifier letter low line)
const SEP: char = '\u{02CD}';

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn tokenize(&mut self) -> Vec<TokenAndLocation> {
        let mut tokens = Vec::new();
        loop {
            self.skip_whitespace();
            if self.pos >= self.chars.len() {
                tokens.push(TokenAndLocation::new(Token::Eof, self.line, self.column));
                break;
            }

            let (token, _skip_leading_space) = self.next_token();
            tokens.push(token);
        }
        tokens
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.chars.len() {
            match self.chars[self.pos] {
                ' ' | '\t' => self.pos += 1,
                '\n' => {
                    self.pos += 1;
                    self.line += 1;
                    self.column = 1;
                }
                '\r' => {
                    self.pos += 1;
                    // Don't increment line if followed by \n
                    if self.pos < self.chars.len() && self.chars[self.pos] == '\n' {
                        self.pos += 1;
                    }
                    self.line += 1;
                    self.column = 1;
                }
                _ => break,
            }
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek_at(&self, offset: usize) -> Option<char> {
        self.chars.get(self.pos + offset).copied()
    }

    fn advance(&mut self) -> char {
        let c = self.chars[self.pos];
        self.pos += 1;
        self.column += 1;
        c
    }

    fn is_letter(c: char) -> bool {
        c.is_alphabetic()
    }

    fn is_id_sep(c: char) -> bool {
        c == '_' || c == SEP || c == '\u{202F}' // narrow no-break space
    }

    fn is_id_char(c: char) -> bool {
        Self::is_letter(c) || c.is_ascii_digit() || Self::is_id_sep(c)
    }

    fn record_pos(&self) -> (u32, u32) {
        (self.line, self.column)
    }

    fn make_token(&self, token: Token) -> TokenAndLocation {
        TokenAndLocation::new(token, self.line, self.column)
    }

    fn next_token(&mut self) -> (TokenAndLocation, bool) {
        let c = self.peek().unwrap(); // skip_whitespace guarantees not EOF

        // Comments
        if c == '!' && self.peek_at(1) == Some('!') {
            if self.peek_at(2) == Some('!') {
                // Block comment !!! ... !!!
                return self.block_comment();
            } else {
                // Line comment !!
                return self.line_comment();
            }
        }

        // Shebang
        if c == '#' && self.peek_at(1) == Some('!') {
            return self.shebang();
        }

        // Unknown literal ???
        if c == '?'
            && self.peek_at(1) == Some('?')
            && self.peek_at(2) == Some('?')
            && self.peek_at(3) != Some('?')
        {
            self.advance();
            self.advance();
            self.advance();
            return (self.make_token(Token::Unknown), false);
        }

        // Multi-char operators: <<=>>>
        if c == '<' {
            return self.lt_token();
        }

        // Multi-char: >>
        if c == '>' && self.peek_at(1) == Some('>') {
            self.advance();
            self.advance();
            return (self.make_token(Token::GtGt), false);
        }

        // ~~
        if c == '~' && self.peek_at(1) == Some('~') {
            self.advance();
            self.advance();
            return (self.make_token(Token::TildeTilde), false);
        }
        // ~=
        if c == '~' && self.peek_at(1) == Some('=') {
            self.advance();
            self.advance();
            return (self.make_token(Token::TildeEquals), false);
        }

        // ??
        if c == '?' && self.peek_at(1) == Some('?') {
            self.advance();
            self.advance();
            return (self.make_token(Token::QuestionQuestion), false);
        }
        // ?=
        if c == '?' && self.peek_at(1) == Some('=') {
            self.advance();
            self.advance();
            return (self.make_token(Token::QuestionEquals), false);
        }

        // ..
        if c == '.' && self.peek_at(1) == Some('.') {
            self.advance();
            self.advance();
            return (self.make_token(Token::DotDot), false);
        }

        // Single char tokens
        match c {
            '{' => {
                self.advance();
                return (self.make_token(Token::LBrace), false);
            }
            '}' => {
                self.advance();
                return (self.make_token(Token::RBrace), false);
            }
            '(' => {
                self.advance();
                return (self.make_token(Token::LParen), false);
            }
            ')' => {
                self.advance();
                return (self.make_token(Token::RParen), false);
            }
            '[' => {
                self.advance();
                return (self.make_token(Token::LBracket), false);
            }
            ']' => {
                self.advance();
                return (self.make_token(Token::RBracket), false);
            }
            ';' => {
                self.advance();
                return (self.make_token(Token::Semicolon), false);
            }
            ',' => {
                self.advance();
                return (self.make_token(Token::Comma), false);
            }
            '=' => {
                self.advance();
                return (self.make_token(Token::Assign), false);
            }
            '+' => {
                self.advance();
                return (self.make_token(Token::Plus), false);
            }
            '-' => {
                self.advance();
                return (self.make_token(Token::Minus), false);
            }
            '*' => {
                self.advance();
                return (self.make_token(Token::Mul), false);
            }
            '/' => {
                self.advance();
                return (self.make_token(Token::Div), false);
            }
            '.' => {
                self.advance();
                return (self.make_token(Token::Dot), false);
            }
            '^' => {
                self.advance();
                return (self.make_token(Token::Caret), false);
            }
            '$' => {
                self.advance();
                return (self.make_token(Token::Dollar), false);
            }
            '?' => {
                self.advance();
                return (self.make_token(Token::Question), false);
            }
            '~' => {
                self.advance();
                return (self.make_token(Token::Tilde), false);
            }
            '#' => {
                self.advance();
                return (self.make_token(Token::Hash), false);
            }
            '>' => {
                self.advance();
                return (self.make_token(Token::Gt), false);
            }
            '\'' => {
                self.advance();
                return (self.make_token(Token::Apostrophe), false);
            }

            // Upward arrow ↑
            '\u{2191}' => {
                self.advance();
                return (self.make_token(Token::Up), false);
            }

            _ => {}
        }

        // Integer
        if c.is_ascii_digit() {
            return self.integer();
        }

        // Identifier / keyword
        if Lexer::is_letter(c) {
            return self.identifier();
        }

        // Fallback: skip unknown character
        self.advance();
        (self.make_token(Token::LineComment), false) // effectively skip
    }

    fn block_comment(&mut self) -> (TokenAndLocation, bool) {
        let (line, column) = self.record_pos();
        // Skip !!!
        self.advance();
        self.advance();
        self.advance();
        let mut body = String::new();
        loop {
            if self.pos >= self.chars.len() {
                break;
            }
            if self.peek_at(0) == Some('!')
                && self.peek_at(1) == Some('!')
                && self.peek_at(2) == Some('!')
            {
                self.advance();
                self.advance();
                self.advance();
                break;
            }
            let c = self.advance();
            body.push(c);
        }
        (
            TokenAndLocation::new(Token::BlockComment(body.trim().to_string()), line, column),
            false,
        )
    }

    fn line_comment(&mut self) -> (TokenAndLocation, bool) {
        let (line, column) = self.record_pos();
        // Skip !!
        self.advance();
        self.advance();
        // Read until newline
        while self.pos < self.chars.len() && self.peek() != Some('\n') && self.peek() != Some('\r')
        {
            self.advance();
        }
        (
            TokenAndLocation::new(Token::LineComment, line, column),
            false,
        )
    }

    fn shebang(&mut self) -> (TokenAndLocation, bool) {
        let (line, column) = self.record_pos();
        self.advance(); // #
        self.advance(); // !
        let mut body = String::new();
        while self.pos < self.chars.len() && self.peek() != Some('\n') && self.peek() != Some('\r')
        {
            body.push(self.advance());
        }
        (
            TokenAndLocation::new(Token::Shebang(body.trim().to_string()), line, column),
            false,
        )
    }

    fn lt_token(&mut self) -> (TokenAndLocation, bool) {
        let (line, column) = self.record_pos();
        self.advance(); // <

        // <<=>>>
        if self.peek_at(0) == Some('<')
            && self.peek_at(1) == Some('=')
            && self.peek_at(2) == Some('>')
            && self.peek_at(3) == Some('>')
            && self.peek_at(4) == Some('>')
        {
            for _ in 0..5 {
                self.advance();
            }
            return (
                TokenAndLocation::new(Token::LtLtEqGtGt, line, column),
                false,
            );
        }

        // <=>
        if self.peek_at(0) == Some('=') && self.peek_at(1) == Some('>') {
            self.advance();
            self.advance();
            return (TokenAndLocation::new(Token::LtEqGt, line, column), false);
        }

        // <<
        if self.peek_at(0) == Some('<') {
            self.advance();
            return (TokenAndLocation::new(Token::LtLt, line, column), false);
        }

        (TokenAndLocation::new(Token::Lt, line, column), false)
    }

    fn integer(&mut self) -> (TokenAndLocation, bool) {
        let (line, column) = self.record_pos();
        let mut num = String::new();
        while self.pos < self.chars.len()
            && self.peek().map(|c| c.is_ascii_digit()).unwrap_or(false)
        {
            num.push(self.advance());
        }
        let value = num.parse().unwrap_or(u64::MAX);
        (
            TokenAndLocation::new(Token::Integer(value), line, column),
            false,
        )
    }

    fn identifier(&mut self) -> (TokenAndLocation, bool) {
        let (line, column) = self.record_pos();
        let mut s = String::new();
        while self.pos < self.chars.len() {
            let c = self.peek().unwrap();
            if Lexer::is_id_char(c) {
                if Lexer::is_id_sep(c) {
                    s.push(SEP);
                } else {
                    s.push(c);
                }
                self.advance();
            } else {
                break;
            }
        }

        // Check for keywords
        match s.as_str() {
            "if" => (TokenAndLocation::new(Token::If, line, column), false),
            "then" => (TokenAndLocation::new(Token::Then, line, column), false),
            "elif" => (TokenAndLocation::new(Token::Elif, line, column), false),
            "else" => (TokenAndLocation::new(Token::Else, line, column), false),
            "fi" => (TokenAndLocation::new(Token::Fi, line, column), false),
            _ => (TokenAndLocation::new(Token::Ident(s), line, column), false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Token::*;

    fn tokens(input: &str) -> Vec<Token> {
        Lexer::new(input)
            .tokenize()
            .into_iter()
            .map(|t| t.token)
            .collect()
    }

    #[test]
    fn lex_integer() {
        assert_eq!(tokens("42"), vec![Integer(42), Eof]);
    }

    #[test]
    fn lex_identifiers() {
        assert_eq!(tokens("x"), vec![Ident("x".into()), Eof]);
        assert_eq!(tokens("my_var"), vec![Ident("my\u{02CD}var".into()), Eof]);
    }

    #[test]
    fn lex_operators() {
        assert_eq!(tokens("+ - * /"), vec![Plus, Minus, Mul, Div, Eof]);
    }

    #[test]
    fn lex_line_comment() {
        let t = tokens("!! this is a comment");
        assert!(t.contains(&LineComment));
    }

    #[test]
    fn lex_block_comment() {
        let t = tokens("!!! block !!!");
        assert!(t.iter().any(|tok| matches!(tok, BlockComment(_))));
    }

    #[test]
    fn lex_unknown() {
        assert_eq!(tokens("???"), vec![Unknown, Eof]);
    }

    #[test]
    fn lex_shebang() {
        let t = tokens("#!/usr/bin/env foolish");
        assert!(t.iter().any(|tok| matches!(tok, Shebang(_))));
    }

    #[test]
    fn lex_brane() {
        let t = tokens("{x = 42;}");
        assert!(t.contains(&LBrace));
        assert!(t.contains(&RBrace));
    }

    #[test]
    fn lex_tilde_equals() {
        assert_eq!(tokens("~="), vec![TildeEquals, Eof]);
    }

    #[test]
    fn lex_question_equals() {
        assert_eq!(tokens("?="), vec![QuestionEquals, Eof]);
    }

    #[test]
    fn lex_tilde_standalone() {
        assert_eq!(tokens("~"), vec![Tilde, Eof]);
    }

    #[test]
    fn lex_question_standalone() {
        assert_eq!(tokens("?"), vec![Question, Eof]);
    }
}
