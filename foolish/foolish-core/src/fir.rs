use std::cell::RefCell;
use std::rc::Rc;

pub type FirRef = Rc<RefCell<Fir>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Nyes {
    #[serde(rename = "PREMBRIONIC")]
    Prembrionic,
    #[serde(rename = "EMBRYONIC")]
    Embryonic,
    #[serde(rename = "BRANING")]
    Braning,
    #[serde(rename = "ECONSTANIC")]
    Econstanic,
    #[serde(rename = "WOCONSTANIC")]
    Woconstanic,
    #[serde(rename = "CONSTANT")]
    Constant,
    #[serde(rename = "INDEPENDENT")]
    Independent,
    #[serde(rename = "NK")]
    Nk,
}

impl Nyes {
    pub fn is_constanic(&self) -> bool {
        matches!(
            self,
            Nyes::Econstanic | Nyes::Woconstanic | Nyes::Constant | Nyes::Independent
        )
    }

    pub fn is_nye(&self) -> bool {
        matches!(self, Nyes::Prembrionic | Nyes::Embryonic | Nyes::Braning)
    }
}

impl std::fmt::Display for Nyes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Nyes::Prembrionic => write!(f, "PREMBRIONIC"),
            Nyes::Embryonic => write!(f, "EMBRYONIC"),
            Nyes::Braning => write!(f, "BRANING"),
            Nyes::Econstanic => write!(f, "ECONSTANIC"),
            Nyes::Woconstanic => write!(f, "WOCONSTANIC"),
            Nyes::Constant => write!(f, "CONSTANT"),
            Nyes::Independent => write!(f, "INDEPENDENT"),
            Nyes::Nk => write!(f, "NK"),
        }
    }
}

/// Statement: name -> body
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StatementFir {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub body: Fir,
    pub state: Nyes,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Fir {
    #[serde(rename = "CONSTANT_INT")]
    ConstantInt {
        value: i64,
        state: Nyes,
    },

    #[serde(rename = "NK")]
    Nk {
        reason: String,
        state: Nyes,
    },

    #[serde(rename = "NORMAL_BRANE")]
    NormalBrane {
        #[serde(skip_serializing_if = "Vec::is_empty", default)]
        characterizations: Vec<String>,
        statements: Vec<StatementFir>,
        state: Nyes,
    },

    #[serde(rename = "BINARY_OP")]
    BinaryOp {
        op: String,
        left: Box<Fir>,
        right: Box<Fir>,
        state: Nyes,
    },

    #[serde(rename = "UNARY_OP")]
    UnaryOp {
        op: String,
        expr: Box<Fir>,
        state: Nyes,
    },

    #[serde(rename = "SEARCH")]
    Search {
        pattern: String,
        #[serde(default)]
        direction: SearchDirection,
        #[serde(default)]
        anchored: bool,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        anchor: Option<Box<Fir>>,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        target: Option<Box<Fir>>,
        state: Nyes,
    },

    #[serde(rename = "INDEX")]
    Index {
        offset: i32,
        #[serde(default)]
        anchored: bool,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        anchor: Option<Box<Fir>>,
        state: Nyes,
    },

    #[serde(rename = "HEAD_TAIL")]
    HeadTail {
        is_head: bool,
        #[serde(default)]
        anchored: bool,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        anchor: Option<Box<Fir>>,
        state: Nyes,
    },

    #[serde(rename = "CONCATENATION")]
    Concatenation {
        elements: Vec<Fir>,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        merged: Option<Box<Fir>>,
        state: Nyes,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SearchDirection {
    Forward,
    Backward,
}

impl Default for SearchDirection {
    fn default() -> Self {
        SearchDirection::Backward
    }
}

impl std::fmt::Display for SearchDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchDirection::Forward => write!(f, "FORWARD"),
            SearchDirection::Backward => write!(f, "BACKWARD"),
        }
    }
}

impl Fir {
    pub fn state(&self) -> Nyes {
        match self {
            Fir::ConstantInt { state, .. } => *state,
            Fir::Nk { state, .. } => *state,
            Fir::NormalBrane { state, .. } => *state,
            Fir::BinaryOp { state, .. } => *state,
            Fir::UnaryOp { state, .. } => *state,
            Fir::Search { state, .. } => *state,
            Fir::Index { state, .. } => *state,
            Fir::HeadTail { state, .. } => *state,
            Fir::Concatenation { state, .. } => *state,
        }
    }

    pub fn set_state(&mut self, state: Nyes) {
        match self {
            Fir::ConstantInt { state: s, .. } => *s = state,
            Fir::Nk { state: s, .. } => *s = state,
            Fir::NormalBrane { state: s, .. } => *s = state,
            Fir::BinaryOp { state: s, .. } => *s = state,
            Fir::UnaryOp { state: s, .. } => *s = state,
            Fir::Search { state: s, .. } => *s = state,
            Fir::Index { state: s, .. } => *s = state,
            Fir::HeadTail { state: s, .. } => *s = state,
            Fir::Concatenation { state: s, .. } => *s = state,
        }
    }

    pub fn is_constanic(&self) -> bool {
        self.state().is_constanic()
    }

    pub fn statements(&self) -> Vec<StatementFir> {
        match self {
            Fir::NormalBrane { statements, .. } => statements.clone(),
            _ => vec![],
        }
    }

    pub fn left_state(&self) -> Nyes {
        match self {
            Fir::BinaryOp { left, .. } => left.state(),
            _ => Nyes::Embryonic,
        }
    }

    pub fn right_state(&self) -> Nyes {
        match self {
            Fir::BinaryOp { right, .. } => right.state(),
            _ => Nyes::Embryonic,
        }
    }

    pub fn step_left(&mut self) -> Result<(), crate::ubc::UbcError> {
        if let Fir::BinaryOp { left, .. } = self {
            let inner: Fir = (**left).clone();
            let mut left_ref = Rc::new(RefCell::new(inner));
            crate::ubc::run_to_completion(&mut left_ref)?;
            *left = Box::new(left_ref.borrow().clone());
        }
        Ok(())
    }

    pub fn step_right(&mut self) -> Result<(), crate::ubc::UbcError> {
        if let Fir::BinaryOp { right, .. } = self {
            let inner: Fir = (**right).clone();
            let mut right_ref = Rc::new(RefCell::new(inner));
            crate::ubc::run_to_completion(&mut right_ref)?;
            *right = Box::new(right_ref.borrow().clone());
        }
        Ok(())
    }

    pub fn binary_values(&self) -> Option<(i64, i64)> {
        if let Fir::BinaryOp { left, right, .. } = self {
            let l = left.as_int()?;
            let r = right.as_int()?;
            Some((l, r))
        } else {
            None
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        match self {
            Fir::ConstantInt { value, .. } => Some(*value),
            _ => None,
        }
    }

    pub fn expr_state(&self) -> Nyes {
        match self {
            Fir::UnaryOp { expr, .. } => expr.state(),
            _ => Nyes::Embryonic,
        }
    }

    pub fn step_expr(&mut self) -> Result<(), crate::ubc::UbcError> {
        if let Fir::UnaryOp { expr, .. } = self {
            let inner: Fir = (**expr).clone();
            let mut expr_ref = Rc::new(RefCell::new(inner));
            crate::ubc::run_to_completion(&mut expr_ref)?;
            *expr = Box::new(expr_ref.borrow().clone());
        }
        Ok(())
    }

    pub fn unary_value(&self) -> Option<i64> {
        if let Fir::UnaryOp { expr, .. } = self {
            expr.as_int()
        } else {
            None
        }
    }

    pub fn search_anchored(&self) -> bool {
        match self {
            Fir::Search { anchored, .. } => *anchored,
            _ => false,
        }
    }

    pub fn search_pattern(&self) -> String {
        match self {
            Fir::Search { pattern, .. } => pattern.clone(),
            _ => String::new(),
        }
    }

    pub fn search_anchor_ref(&self) -> Option<FirRef> {
        match self {
            Fir::Search { anchor: Some(a), .. } => Some(Rc::new(RefCell::new((**a).clone()))),
            _ => None,
        }
    }

    pub fn search_target_ref(&self) -> Option<FirRef> {
        match self {
            Fir::Search { target: Some(t), .. } => Some(Rc::new(RefCell::new((**t).clone()))),
            _ => None,
        }
    }

    pub fn set_search_target(&mut self, target: FirRef) {
        if let Fir::Search { target: t, .. } = self {
            *t = Some(Box::new(target.borrow().clone()));
        }
    }

    pub fn set_search_target_direct(&mut self, target: Option<FirRef>) {
        if let Fir::Search { target: t, .. } = self {
            *t = target.map(|r| Box::new(r.borrow().clone()));
        }
    }

    pub fn index_anchored(&self) -> bool {
        match self {
            Fir::Index { anchored, .. } => *anchored,
            _ => false,
        }
    }

    pub fn index_offset(&self) -> i32 {
        match self {
            Fir::Index { offset, .. } => *offset,
            _ => 0,
        }
    }

    pub fn index_anchor_ref(&self) -> Option<FirRef> {
        match self {
            Fir::Index { anchor: Some(a), .. } => Some(Rc::new(RefCell::new((**a).clone()))),
            _ => None,
        }
    }

    pub fn set_index_target(&mut self, target: FirRef) {
        let found = target.borrow().clone();
        *self = found;
    }

    pub fn headtail_is_head(&self) -> bool {
        match self {
            Fir::HeadTail { is_head, .. } => *is_head,
            _ => false,
        }
    }

    pub fn headtail_anchor_ref(&self) -> Option<FirRef> {
        match self {
            Fir::HeadTail { anchor: Some(a), .. } => Some(Rc::new(RefCell::new((**a).clone()))),
            _ => None,
        }
    }

    pub fn set_headtail_target(&mut self, target: FirRef) {
        let found = target.borrow().clone();
        *self = found;
    }

    pub fn concat_elements(&self) -> Vec<Fir> {
        match self {
            Fir::Concatenation { elements, .. } => elements.clone(),
            _ => vec![],
        }
    }

    pub fn set_concat_merged(&mut self, merged: FirRef) {
        if let Fir::Concatenation { merged: m, .. } = self {
            *m = Some(Box::new(merged.borrow().clone()));
        }
    }

    pub fn normal_brane_statements(&mut self, statements: Vec<StatementFir>) {
        if let Fir::NormalBrane { statements: s, .. } = self {
            *s = statements;
        }
    }

    pub fn set_binary_operands(&mut self, left: Fir, right: Fir) {
        if let Fir::BinaryOp { left: l, right: r, .. } = self {
            *l = Box::new(left);
            *r = Box::new(right);
        }
    }

    pub fn set_unary_expr(&mut self, expr: Fir) {
        if let Fir::UnaryOp { expr: e, .. } = self {
            *e = Box::new(expr);
        }
    }
}

impl StatementFir {
    pub fn new(name: Option<String>, body: Fir) -> Self {
        Self {
            name,
            body,
            state: Nyes::Embryonic,
        }
    }

    pub fn anonymous(body: Fir) -> Self {
        Self::new(None, body)
    }
}
