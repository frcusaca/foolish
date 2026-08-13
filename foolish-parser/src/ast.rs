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

    /// Regex search: anchor?pattern or anchor~pattern.
    /// `anchor: None` is a bare **unanchored** form — it searches the current brane
    /// (IB), then climbs ancestors (AB), per FOOP-23 Part B / AGENTS.md "Contextless
    /// Anchored Searches"; an unanchored miss settles ECONSTANIC, not NK.
    ///
    /// **Both directions have an unanchored form** (FOOP-55 §6). They scan the *same*
    /// candidate window — the home brane's statements **before** the searching one,
    /// `[0, my_index-1]` — in opposite directions:
    /// - `?pattern` (`RegexpLocal`) walks it backward, finding the **nearest preceding**
    ///   match.
    /// - `~pattern` (`RegexpForward`) walks it forward, finding the **earliest** match.
    ///
    /// This does not contradict FOOP-23 §Specification A.1 ("Foolish cannot look forward
    /// in its own brane"): that concerns looking *ahead* into statements which have not
    /// settled. The window ends before the searching statement, so it contains only
    /// statements FIFO draining has already settled — the searching statement itself
    /// never matches, and nothing after it is a candidate.
    RegexpSearch {
        anchor: Option<Box<Astn>>,
        operator: SearchOperator,
        pattern: String,
    },

    /// `anchor@` — project a search result's POSITION (FOOP-55 §8).
    ///
    /// A **continuation**: `anchor` must BE a search, since only a search
    /// produces a position. `@` yields the found statement's index, or `-1`
    /// when the search `candidates_exhausted()` — which is what lets a default
    /// branch fall out of arithmetic, since `@+1` maps a miss to index 0.
    ///
    /// The position is **shallow**: `{b = ?hello_world; {a = b@+1}}` reports
    /// `b`'s own position, not `?hello_world`'s. A value chases through a
    /// reference; a position does not, because a position is meaningful only
    /// relative to one brane.
    SearchPosition {
        anchor: Box<Astn>,
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
