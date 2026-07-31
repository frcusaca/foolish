use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchOperator {
    Head,          // ^
    Tail,          // $
    RegexpLocal,   // ?
    RegexpForward, // ~
    ValueLocal,    // ?=  (backward value search)
    ValueForward,  // ~=  (forward value search)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignmentOperator {
    Assign, // =
    SF,     // <=>
    SFF,    // <<=>>>
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Astn {
    IntLit(u64),
    UnknownLit, // ???
    Creation,   // ⬤

    Identifier {
        characterizations: Vec<String>,
        id: String,
    },

    Brane {
        characterizations: Vec<String>,
        statements: Vec<Astn>,
    },

    Assignment {
        characterizations: Vec<String>,
        identifier: String,
        operator: AssignmentOperator,
        expr: Box<Astn>,
    },

    BinaryOp {
        op: String,
        left: Box<Astn>,
        right: Box<Astn>,
    },

    UnaryOp {
        op: String,
        expr: Box<Astn>,
    },

    Concatenation {
        elements: Vec<Astn>,
    },

    /// Dot search: anchor.coordinate
    DotSearch {
        anchor: Box<Astn>,
        coordinate: String,
    },

    /// Regex search: anchor?pattern or anchor~pattern
    RegexpSearch {
        anchor: Box<Astn>,
        operator: SearchOperator,
        pattern: String,
    },

    /// Value search: anchor~=value, anchor?=value, ?=value
    /// Combined: anchor~name=value, anchor?name=value, ?name=value
    ValueSearch {
        anchor: Option<Box<Astn>>,
        forward: bool,
        name_pattern: Option<String>,
        value_pattern: Box<Astn>,
    },

    /// Indexed access: anchor#N or anchor#-N
    Seek {
        anchor: Box<Astn>,
        offset: i32,
    },

    /// Head/tail: anchor^ or anchor$
    HeadTail {
        is_head: bool,
        anchor: Box<Astn>,
    },

    /// Unanchored backward seek: #-N
    UnanchoredSeek {
        offset: i32, // negative value
    },

    /// Upward search: ↑
    UpwardSearch,

    /// SF marker: <expr>
    StayFoolish {
        expr: Box<Astn>,
    },

    /// SFF marker: <<expr>>
    StayFullyFoolish {
        expr: Box<Astn>,
    },

    /// Detachment brane: [...]{...}
    DetachmentBrane {
        statements: Vec<Astn>,
        body: Option<Box<Astn>>,
    },

    /// if-then-elif-else-fi (parsed but rejected at compile time)
    IfExpr {
        condition: Box<Astn>,
        then_body: Box<Astn>,
        elif_clauses: Vec<(Box<Astn>, Box<Astn>)>,
        else_body: Option<Box<Astn>>,
    },

    /// Contexted search: &?name, &~name, &#N, &^, &$, &~=v, &?=v
    ContextedSearch {
        inner: Box<Astn>,
    },

    /// Deferred/not yet implemented
    NotImplemented(String),
}

impl Astn {
    pub fn is_constanic(&self) -> bool {
        matches!(self, Astn::IntLit(_) | Astn::UnknownLit)
    }
}

impl fmt::Display for Astn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Astn::IntLit(n) => write!(f, "{}", n),
            Astn::UnknownLit => write!(f, "???"),
            Astn::Creation => write!(f, "\u{2B24}"),
            Astn::Identifier { id, .. } => write!(f, "{}", id),
            Astn::Brane { statements, .. } => {
                write!(f, "{{")?;
                for (i, s) in statements.iter().enumerate() {
                    if i > 0 {
                        write!(f, "; ")?;
                    }
                    write!(f, "{:?}", s)?;
                }
                write!(f, "}}")
            }
            Astn::BinaryOp { op, left, right } => write!(f, "({} {} {})", left, op, right),
            Astn::UnaryOp { op, expr } => write!(f, "{}{}", op, expr),
            _ => write!(f, "{:?}", self),
        }
    }
}
