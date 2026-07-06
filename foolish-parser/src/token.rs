#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    LBrace,
    RBrace,
    LParen,
    RParen,
    LBracket,
    RBracket,
    Semicolon,
    Comma,

    Assign,           // =
    Plus,             // +
    Minus,            // -
    Mul,              // *
    Div,              // /
    Dot,              // .
    DotDot,           // ..
    Caret,            // ^
    Dollar,           // $
    Question,         // ?
    QuestionQuestion, // ??
    QuestionEquals,   // ?=
    Tilde,            // ~
    TildeTilde,       // ~~
    TildeEquals,      // ~=
    Hash,             // #
    Ampersand,        // &

    Lt,         // <
    Gt,         // >
    LtEqGt,     // <=>
    LtLt,       // <<
    GtGt,       // >>
    LtLtEqGtGt, // <<=>>>

    Apostrophe, // '

    Integer(u64),
    Ident(String),
    Shebang(String),
    LineComment,
    BlockComment(String),
    Unknown, // ???
    Up,      // ↑

    If,
    Then,
    Elif,
    Else,
    Fi,

    Eof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenAndLocation {
    pub token: Token,
    pub line: u32,
    pub column: u32,
}

impl TokenAndLocation {
    pub fn new(token: Token, line: u32, column: u32) -> Self {
        Self {
            token,
            line,
            column,
        }
    }
}
