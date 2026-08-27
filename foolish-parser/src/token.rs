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
    /// `@` — projects a search result's POSITION (FOOP-55 §8).
    At, // @
    Ampersand,        // &

    Lt,     // <
    Gt,     // >
    LtEqGt, // <=>
    LtLt,   // <<
    GtGt,   // >>
    /// `<<<` — UFM (Unstay Foolishness Mark) opener, FOOP-55 Phase 3J.
    LtLtLt, // <<<
    /// `>>>` — UFM closer.
    GtGtGt, // >>>
    LtLtEqGtGt, // <<=>>>

    Apostrophe, // '
    Backtick,   // `

    Creation, // ⬤ (U+2B24)

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
    /// True when whitespace (space, tab, or newline) immediately preceded
    /// this token.
    ///
    /// FOOP-75 §5: a space terminates a search-specification sequence, so the
    /// parser needs *adjacency* — and `column` cannot supply it.
    /// `Lexer::skip_whitespace` advances `pos` past spaces and tabs without
    /// incrementing `column`, so `column` counts consumed non-whitespace
    /// characters since the line started, not a character offset. Measured
    /// consequence on jia@dc6db093: `"{a =$ b}"`, `"{a = $ b}"` and
    /// `"{a =   $ b}"` lexed to byte-identical token streams.
    ///
    /// Fixing `column` to count whitespace was rejected: it would change every
    /// existing parse-error message's reported column for no gain, and it
    /// conflates "where is this for a human" with "was this adjacent".
    pub preceded_by_space: bool,
}

impl TokenAndLocation {
    pub fn new(token: Token, line: u32, column: u32, preceded_by_space: bool) -> Self {
        Self {
            token,
            line,
            column,
            preceded_by_space,
        }
    }
}
