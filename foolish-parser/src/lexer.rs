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
            // FOOP-75 §5.3: whether whitespace was consumed here is the ONLY
            // record that it existed — `column` does not count it. Stamp the
            // answer onto the token that follows, since this is the one place
            // that knows. Every `make_token` construction site defaults the
            // flag to false; it is corrected here.
            let had_space = self.skip_whitespace();
            if self.pos >= self.chars.len() {
                tokens.push(TokenAndLocation::new(
                    Token::Eof,
                    self.line,
                    self.column,
                    had_space,
                ));
                break;
            }

            let (mut token, _skip_leading_space) = self.next_token();
            token.preceded_by_space = had_space;
            tokens.push(token);
        }
        tokens
    }

    /// Consume whitespace, reporting whether any was consumed.
    ///
    /// The boolean is FOOP-75 §5.3's adjacency signal — see
    /// [`TokenAndLocation::preceded_by_space`]. Note this function
    /// deliberately does NOT increment `column` for spaces and tabs; that
    /// long-standing behavior is what makes the boolean necessary.
    fn skip_whitespace(&mut self) -> bool {
        let start = self.pos;
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
        self.pos != start
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
        TokenAndLocation::new(token, self.line, self.column, false)
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
                // Check for {*} creation alias: { immediately followed by *}
                // with no interior whitespace. This is a parser-level recognition,
                // but we detect it at the character level to distinguish from { * }.
                if self.peek_at(1) == Some('*') && self.peek_at(2) == Some('}') {
                    self.advance();
                    self.advance();
                    self.advance();
                    return (self.make_token(Token::Creation), false);
                }
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
            '&' => {
                self.advance();
                return (self.make_token(Token::Ampersand), false);
            }
            '`' => {
                self.advance();
                return (self.make_token(Token::Backtick), false);
            }

            // Upward arrow ↑
            '\u{2191}' => {
                self.advance();
                return (self.make_token(Token::Up), false);
            }

            // Creation dot ⬤ (U+2B24)
            '\u{2B24}' => {
                self.advance();
                return (self.make_token(Token::Creation), false);
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
            TokenAndLocation::new(
                Token::BlockComment(body.trim().to_string()),
                line,
                column,
                false,
            ),
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
            TokenAndLocation::new(Token::LineComment, line, column, false),
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
            TokenAndLocation::new(Token::Shebang(body.trim().to_string()), line, column, false),
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
                TokenAndLocation::new(Token::LtLtEqGtGt, line, column, false),
                false,
            );
        }

        // <=>
        if self.peek_at(0) == Some('=') && self.peek_at(1) == Some('>') {
            self.advance();
            self.advance();
            return (
                TokenAndLocation::new(Token::LtEqGt, line, column, false),
                false,
            );
        }

        // <<
        if self.peek_at(0) == Some('<') {
            self.advance();
            return (
                TokenAndLocation::new(Token::LtLt, line, column, false),
                false,
            );
        }

        (TokenAndLocation::new(Token::Lt, line, column, false), false)
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
            TokenAndLocation::new(Token::Integer(value), line, column, false),
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
            "if" => (TokenAndLocation::new(Token::If, line, column, false), false),
            "then" => (
                TokenAndLocation::new(Token::Then, line, column, false),
                false,
            ),
            "elif" => (
                TokenAndLocation::new(Token::Elif, line, column, false),
                false,
            ),
            "else" => (
                TokenAndLocation::new(Token::Else, line, column, false),
                false,
            ),
            "fi" => (TokenAndLocation::new(Token::Fi, line, column, false), false),
            _ => (
                TokenAndLocation::new(Token::Ident(s), line, column, false),
                false,
            ),
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

    #[test]
    fn lex_ampersand() {
        assert_eq!(tokens("&"), vec![Ampersand, Eof]);
    }

    #[test]
    fn lex_backtick() {
        assert_eq!(tokens("`"), vec![Backtick, Eof]);
    }

    /// FOOP-75 §5.3: the lexer must record whether whitespace preceded each
    /// token, because `column` cannot answer the adjacency question — it does
    /// not count skipped whitespace (`skip_whitespace` advances `pos` without
    /// bumping `column`).
    ///
    /// Regression guard for the exact defect measured on jia@dc6db093:
    /// `"{a =$ b}"` and `"{a = $ b}"` produced byte-identical token streams,
    /// which made FOOP-75 §5's space rule unimplementable.
    #[test]
    fn foop75_lexer_records_preceding_space() {
        let toks = Lexer::new("{a =$ b}").tokenize();
        let dollar = toks.iter().find(|t| t.token == Dollar).expect("has $");
        assert!(
            !dollar.preceded_by_space,
            "`=$`: the $ is adjacent to the ="
        );

        let toks = Lexer::new("{a = $ b}").tokenize();
        let dollar = toks.iter().find(|t| t.token == Dollar).expect("has $");
        assert!(
            dollar.preceded_by_space,
            "`= $`: the $ is NOT adjacent to the ="
        );

        let toks = Lexer::new("{a =   $ b}").tokenize();
        let dollar = toks.iter().find(|t| t.token == Dollar).expect("has $");
        assert!(
            dollar.preceded_by_space,
            "`=   $`: multiple spaces, still not adjacent"
        );
    }

    /// FOOP-75 §5.3: tabs and newlines count as space for adjacency.
    #[test]
    fn foop75_lexer_counts_tab_and_newline_as_space() {
        for src in ["{a =\t$ b}", "{a =\n$ b}"] {
            let toks = Lexer::new(src).tokenize();
            let dollar = toks.iter().find(|t| t.token == Dollar).unwrap();
            assert!(
                dollar.preceded_by_space,
                "tab/newline must count as space: {src:?}"
            );
        }
    }

    /// FOOP-75 §5.3: the very first token of a source has nothing before it.
    /// It is not "preceded by space" — there is no space, there is nothing.
    #[test]
    fn foop75_lexer_first_token_is_not_preceded_by_space() {
        let toks = Lexer::new("{a}").tokenize();
        assert!(
            !toks[0].preceded_by_space,
            "the first token has no preceding whitespace"
        );
        // ...but a source that OPENS with whitespace does flag its first token.
        let toks = Lexer::new("  {a}").tokenize();
        assert!(
            toks[0].preceded_by_space,
            "leading whitespace flags the first token"
        );
    }
}
