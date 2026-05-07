use std::cell::RefCell;
use std::fmt::Write;
use std::rc::Rc;

// Re-export for external use
use serde::{Serialize, Deserialize, Serializer, Deserializer};
pub use crate::ubc::UbcError;

/// Result of a single step operation (for step_members reporting)
#[derive(Debug, Clone, PartialEq)]
pub enum StepResult {
    NoOp,
    MadeProgress,
    NewChildren,
    Blocked,
}

impl StepResult {
    pub fn is_terminal(&self) -> bool {
        matches!(self, StepResult::NoOp)
    }
}

/// Shared state for all FIRs
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

/// Search direction for anchored searches
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

// Forward decl — structs use FirRef, enum wraps structs
pub type FirRef = Rc<RefCell<dyn Steppable>>;

/// Statement: name -> body
#[derive(Debug, Clone)]
pub struct StatementFir {
    pub name: Option<String>,
    pub body: FirRef,
    pub state: Nyes,
}

impl StatementFir {
    pub fn new(name: Option<String>, body: FirRef) -> Self {
        Self {
            name,
            body,
            state: Nyes::Embryonic,
        }
    }

    pub fn anonymous(body: FirRef) -> Self {
        Self::new(None, body)
    }
}

// ==================== FIR Struct Definitions ====================

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ConstantIntFir {
    pub value: i64,
    pub state: Nyes,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NkFir {
    pub reason: String,
    pub state: Nyes,
}

#[derive(Debug, Clone)]
pub struct BinaryOpFir {
    pub op: String,
    pub left: FirRef,
    pub right: FirRef,
    pub state: Nyes,
}

#[derive(Debug, Clone)]
pub struct UnaryOpFir {
    pub op: String,
    pub expr: FirRef,
    pub state: Nyes,
}

#[derive(Debug, Clone)]
pub struct SearchFir {
    pub pattern: String,
    pub direction: SearchDirection,
    pub anchored: bool,
    pub anchor: Option<FirRef>,
    pub target: Option<FirRef>,
    pub state: Nyes,
}

#[derive(Debug, Clone)]
pub struct IndexFir {
    pub offset: i32,
    pub anchored: bool,
    pub anchor: Option<FirRef>,
    pub state: Nyes,
}

#[derive(Debug, Clone)]
pub struct HeadTailFir {
    pub is_head: bool,
    pub anchored: bool,
    pub anchor: Option<FirRef>,
    pub state: Nyes,
}

#[derive(Debug, Clone)]
pub struct StayFoolishFir {
    pub expr: FirRef,
    pub state: Nyes,
}

#[derive(Debug, Clone)]
pub struct StayFullyFoolishFir {
    pub expr: FirRef,
    pub state: Nyes,
}

#[derive(Debug, Clone)]
pub struct ConcatenationFir {
    pub elements: Vec<FirRef>,
    pub merged: Option<FirRef>,
    pub state: Nyes,
}

#[derive(Debug, Clone)]
pub struct NormalBraneFir {
    pub characterizations: Vec<String>,
    pub statements: Vec<StatementFir>,
    pub state: Nyes,
}

// ==================== Steppable Trait ====================

/// Trait for all FIRs — the OOP interface.
/// step_one() returns None if self was mutated in-place,
/// or Some(replacement_fir) if self should be replaced entirely.
pub trait Steppable: std::fmt::Debug {
    fn step_one(&mut self, scope: &crate::ubc::Scope) -> Result<Option<Fir>, UbcError>;

    fn state(&self) -> Nyes;
    fn set_state(&mut self, state: Nyes);

    fn children_mut(&mut self) -> Vec<&mut FirRef>;

    /// Step all children in-place. If a child returns Some(replacement), replace it.
    fn step_members(&mut self, scope: &crate::ubc::Scope) -> Result<(), UbcError> {
        for child in self.children_mut() {
            let repl = child.borrow_mut().step_one(scope)?;
            if let Some(repl_fir) = repl {
                *child = fir_to_ref(repl_fir);
            }
        }
        Ok(())
    }

    fn format(&self, buf: &mut String, depth: usize) -> std::fmt::Result;

    /// Clone this Steppable into a Fir enum (bypasses dyn Clone limitation).
    fn clone_into_fir(&self) -> Fir;

    /// Return variant identifier (for type-based dispatch on trait objects).
    fn fir_variant(&self) -> &'static str { "" }

    // --- Accessors (defaults return None/empty — overridden by Fir impl) ---
    fn as_int(&self) -> Option<i64> { None }
    fn binary_values(&self) -> Option<(i64, i64)> { None }
    fn search_target_ref(&self) -> Option<FirRef> { None }
    fn search_pattern(&self) -> Option<String> { None }
    fn search_anchored(&self) -> bool { false }
    fn search_anchor_ref(&self) -> Option<FirRef> { None }
    fn index_offset(&self) -> i32 { 0 }
    fn index_anchored(&self) -> bool { false }
    fn index_anchor_ref(&self) -> Option<FirRef> { None }
    fn headtail_is_head(&self) -> bool { false }
    fn headtail_anchor_ref(&self) -> Option<FirRef> { None }
    fn left_state(&self) -> Nyes { Nyes::Nk }
    fn right_state(&self) -> Nyes { Nyes::Nk }
    fn expr_state(&self) -> Nyes { Nyes::Nk }
    fn unary_value(&self) -> Option<i64> { None }
    fn concat_merged_ref(&self) -> Option<FirRef> { None }
    fn concat_elements(&self) -> Vec<FirRef> { vec![] }
    fn normal_brane_statements(&self) -> Vec<StatementFir> { vec![] }
    fn as_brane_statements(&self) -> Option<Vec<StatementFir>> { None }
    fn brane_statement_at(&self, _idx: usize) -> Option<FirRef> { None }
    fn brane_statement_count(&self) -> usize { 0 }

    // Mutation methods (defaults: no-op)
    fn set_binary_operands(&mut self, _left: Fir, _right: Fir) {}
    fn set_unary_expr(&mut self, _expr: Fir) {}
    fn set_search_target(&mut self, _target: FirRef) {}
    fn set_search_target_direct(&mut self, _target: Option<FirRef>) {}
    fn set_concat_merged(&mut self, _merged: FirRef) {}
    fn set_concat_merged_direct(&mut self, _merged: Option<FirRef>) {}
}

// ==================== Fir Enum ====================

/// Clone a FirRef into a Fir enum value.
pub(crate) fn clone_steppable(fir: &FirRef) -> Fir {
    fir.borrow().clone_into_fir()
}

/// Wrap a Fir into a FirRef (Rc<RefCell<dyn Steppable>>).
pub(crate) fn fir_to_ref(fir: Fir) -> FirRef {
    Rc::new(RefCell::new(fir))
}

#[derive(Debug, Clone)]
pub enum Fir {
    ConstantInt(Box<ConstantIntFir>),
    Nk(Box<NkFir>),
    BinaryOp(Box<BinaryOpFir>),
    UnaryOp(Box<UnaryOpFir>),
    Search(Box<SearchFir>),
    Index(Box<IndexFir>),
    HeadTail(Box<HeadTailFir>),
    StayFoolish(Box<StayFoolishFir>),
    StayFullyFoolish(Box<StayFullyFoolishFir>),
    Concatenation(Box<ConcatenationFir>),
    NormalBrane(Box<NormalBraneFir>),
}

impl Fir {
    pub fn into_ref(self) -> FirRef {
        Rc::new(RefCell::new(self))
    }
}

// ==================== Fir: Steppable (dispatches to inner struct) ====================

impl Steppable for Fir {
    fn step_one(&mut self, scope: &crate::ubc::Scope) -> Result<Option<Fir>, UbcError> {
        match self {
            Fir::ConstantInt(inner) => inner.step_one(scope),
            Fir::Nk(inner) => inner.step_one(scope),
            Fir::BinaryOp(inner) => inner.step_one(scope),
            Fir::UnaryOp(inner) => inner.step_one(scope),
            Fir::Search(inner) => inner.step_one(scope),
            Fir::Index(inner) => inner.step_one(scope),
            Fir::HeadTail(inner) => inner.step_one(scope),
            Fir::StayFoolish(inner) => inner.step_one(scope),
            Fir::StayFullyFoolish(inner) => inner.step_one(scope),
            Fir::Concatenation(inner) => inner.step_one(scope),
            Fir::NormalBrane(inner) => inner.step_one(scope),
        }
    }

    fn state(&self) -> Nyes {
        match self {
            Fir::ConstantInt(i) => i.state(),
            Fir::Nk(i) => i.state(),
            Fir::BinaryOp(i) => i.state(),
            Fir::UnaryOp(i) => i.state(),
            Fir::Search(i) => i.state(),
            Fir::Index(i) => i.state(),
            Fir::HeadTail(i) => i.state(),
            Fir::StayFoolish(i) => i.state(),
            Fir::StayFullyFoolish(i) => i.state(),
            Fir::Concatenation(i) => i.state(),
            Fir::NormalBrane(i) => i.state(),
        }
    }

    fn set_state(&mut self, s: Nyes) {
        match self {
            Fir::ConstantInt(i) => i.set_state(s),
            Fir::Nk(i) => i.set_state(s),
            Fir::BinaryOp(i) => i.set_state(s),
            Fir::UnaryOp(i) => i.set_state(s),
            Fir::Search(i) => i.set_state(s),
            Fir::Index(i) => i.set_state(s),
            Fir::HeadTail(i) => i.set_state(s),
            Fir::StayFoolish(i) => i.set_state(s),
            Fir::StayFullyFoolish(i) => i.set_state(s),
            Fir::Concatenation(i) => i.set_state(s),
            Fir::NormalBrane(i) => i.set_state(s),
        }
    }

    fn children_mut(&mut self) -> Vec<&mut FirRef> {
        match self {
            Fir::ConstantInt(i) => i.children_mut(),
            Fir::Nk(i) => i.children_mut(),
            Fir::BinaryOp(i) => i.children_mut(),
            Fir::UnaryOp(i) => i.children_mut(),
            Fir::Search(i) => i.children_mut(),
            Fir::Index(i) => i.children_mut(),
            Fir::HeadTail(i) => i.children_mut(),
            Fir::StayFoolish(i) => i.children_mut(),
            Fir::StayFullyFoolish(i) => i.children_mut(),
            Fir::Concatenation(i) => i.children_mut(),
            Fir::NormalBrane(i) => i.children_mut(),
        }
    }

    fn format(&self, buf: &mut String, depth: usize) -> std::fmt::Result {
        match self {
            Fir::ConstantInt(i) => i.format(buf, depth),
            Fir::Nk(i) => i.format(buf, depth),
            Fir::BinaryOp(i) => i.format(buf, depth),
            Fir::UnaryOp(i) => i.format(buf, depth),
            Fir::Search(i) => i.format(buf, depth),
            Fir::Index(i) => i.format(buf, depth),
            Fir::HeadTail(i) => i.format(buf, depth),
            Fir::StayFoolish(i) => i.format(buf, depth),
            Fir::StayFullyFoolish(i) => i.format(buf, depth),
            Fir::Concatenation(i) => i.format(buf, depth),
            Fir::NormalBrane(i) => i.format(buf, depth),
        }
    }

    fn clone_into_fir(&self) -> Fir {
        self.clone()
    }

    fn fir_variant(&self) -> &'static str {
        match self {
            Fir::ConstantInt(_) => "ConstantInt",
            Fir::Nk(_) => "Nk",
            Fir::BinaryOp(_) => "BinaryOp",
            Fir::UnaryOp(_) => "UnaryOp",
            Fir::Search(_) => "Search",
            Fir::Index(_) => "Index",
            Fir::HeadTail(_) => "HeadTail",
            Fir::StayFoolish(_) => "StayFoolish",
            Fir::StayFullyFoolish(_) => "StayFullyFoolish",
            Fir::Concatenation(_) => "Concatenation",
            Fir::NormalBrane(_) => "NormalBrane",
        }
    }

    // Accessor overrides
    fn as_int(&self) -> Option<i64> {
        if let Fir::ConstantInt(inner) = self { Some(inner.value) } else { None }
    }

    fn binary_values(&self) -> Option<(i64, i64)> {
        if let Fir::BinaryOp(inner) = self {
            let l = inner.left.borrow().as_int()?;
            let r = inner.right.borrow().as_int()?;
            Some((l, r))
        } else { None }
    }

    fn search_target_ref(&self) -> Option<FirRef> {
        if let Fir::Search(inner) = self { inner.target.clone() } else { None }
    }

    fn search_pattern(&self) -> Option<String> {
        if let Fir::Search(inner) = self { Some(inner.pattern.clone()) } else { None }
    }

    fn search_anchored(&self) -> bool {
        if let Fir::Search(inner) = self { inner.anchored } else { false }
    }

    fn search_anchor_ref(&self) -> Option<FirRef> {
        if let Fir::Search(inner) = self { inner.anchor.clone() } else { None }
    }

    fn index_offset(&self) -> i32 {
        if let Fir::Index(inner) = self { inner.offset } else { 0 }
    }

    fn index_anchored(&self) -> bool {
        if let Fir::Index(inner) = self { inner.anchored } else { false }
    }

    fn index_anchor_ref(&self) -> Option<FirRef> {
        if let Fir::Index(inner) = self { inner.anchor.clone() } else { None }
    }

    fn headtail_is_head(&self) -> bool {
        if let Fir::HeadTail(inner) = self { inner.is_head } else { false }
    }

    fn headtail_anchor_ref(&self) -> Option<FirRef> {
        if let Fir::HeadTail(inner) = self { inner.anchor.clone() } else { None }
    }

    fn left_state(&self) -> Nyes {
        if let Fir::BinaryOp(inner) = self { inner.left.borrow().state() } else { Nyes::Nk }
    }

    fn right_state(&self) -> Nyes {
        if let Fir::BinaryOp(inner) = self { inner.right.borrow().state() } else { Nyes::Nk }
    }

    fn expr_state(&self) -> Nyes {
        if let Fir::UnaryOp(inner) = self { inner.expr.borrow().state() } else { Nyes::Nk }
    }

    fn unary_value(&self) -> Option<i64> {
        if let Fir::UnaryOp(inner) = self { inner.expr.borrow().as_int() } else { None }
    }

    fn concat_merged_ref(&self) -> Option<FirRef> {
        if let Fir::Concatenation(inner) = self { inner.merged.clone() } else { None }
    }

    fn concat_elements(&self) -> Vec<FirRef> {
        if let Fir::Concatenation(inner) = self {
            inner.elements.iter().map(Rc::clone).collect()
        } else { vec![] }
    }

    fn normal_brane_statements(&self) -> Vec<StatementFir> {
        if let Fir::NormalBrane(inner) = self { inner.statements.clone() } else { vec![] }
    }

    fn as_brane_statements(&self) -> Option<Vec<StatementFir>> {
        if let Fir::NormalBrane(inner) = self { Some(inner.statements.clone()) } else { None }
    }

    fn brane_statement_at(&self, idx: usize) -> Option<FirRef> {
        if let Fir::NormalBrane(inner) = self {
            inner.statements.get(idx).map(|s| Rc::clone(&s.body))
        } else { None }
    }

    fn brane_statement_count(&self) -> usize {
        if let Fir::NormalBrane(inner) = self { inner.statements.len() } else { 0 }
    }

    fn set_binary_operands(&mut self, left: Fir, right: Fir) {
        if let Fir::BinaryOp(inner) = self {
            inner.left = fir_to_ref(left);
            inner.right = fir_to_ref(right);
        }
    }

    fn set_unary_expr(&mut self, expr: Fir) {
        if let Fir::UnaryOp(inner) = self {
            inner.expr = fir_to_ref(expr);
        }
    }

    fn set_search_target(&mut self, target: FirRef) {
        if let Fir::Search(inner) = self {
            inner.target = Some(target);
        }
    }

    fn set_search_target_direct(&mut self, target: Option<FirRef>) {
        if let Fir::Search(inner) = self {
            inner.target = target;
        }
    }

    fn set_concat_merged(&mut self, merged: FirRef) {
        if let Fir::Concatenation(inner) = self {
            inner.merged = Some(Rc::clone(&merged));
        }
    }

    fn set_concat_merged_direct(&mut self, merged: Option<FirRef>) {
        if let Fir::Concatenation(inner) = self {
            inner.merged = merged;
        }
    }
}

// Mutation methods on Fir (not on trait — enum knows its own shape)
impl Fir {
    pub fn set_stay_expr(&mut self, expr: Fir) {
        if let Fir::StayFoolish(inner) = self {
            inner.expr = fir_to_ref(expr);
        }
    }

    pub fn normal_brane_set_statements(&mut self, statements: Vec<StatementFir>) {
        if let Fir::NormalBrane(inner) = self {
            inner.statements = statements;
        }
    }
}

// ==================== Struct Steppable Implementations ====================

impl Steppable for ConstantIntFir {
    fn step_one(&mut self, _: &crate::ubc::Scope) -> Result<Option<Fir>, UbcError> {
        Ok(None)
    }
    fn state(&self) -> Nyes { self.state }
    fn set_state(&mut self, s: Nyes) { self.state = s; }
    fn children_mut(&mut self) -> Vec<&mut FirRef> { vec![] }
    fn format(&self, buf: &mut String, depth: usize) -> std::fmt::Result {
        writeln!(buf, "{}Int({}) [{}]", "  ".repeat(depth), self.value, self.state)
    }
    fn clone_into_fir(&self) -> Fir { Fir::ConstantInt(Box::new(self.clone())) }
    fn fir_variant(&self) -> &'static str { "ConstantInt" }
}

impl Steppable for NkFir {
    fn step_one(&mut self, _: &crate::ubc::Scope) -> Result<Option<Fir>, UbcError> {
        Ok(None)
    }
    fn state(&self) -> Nyes { self.state }
    fn set_state(&mut self, s: Nyes) { self.state = s; }
    fn children_mut(&mut self) -> Vec<&mut FirRef> { vec![] }
    fn format(&self, buf: &mut String, depth: usize) -> std::fmt::Result {
        writeln!(buf, "{}??? ({}) [{}]", "  ".repeat(depth), self.reason, self.state)
    }
    fn clone_into_fir(&self) -> Fir { Fir::Nk(Box::new(self.clone())) }
    fn fir_variant(&self) -> &'static str { "Nk" }
}

impl Steppable for BinaryOpFir {
    fn step_one(&mut self, scope: &crate::ubc::Scope) -> Result<Option<Fir>, UbcError> {
        // Step children in-place
        self.step_members(scope)?;

        let ls = self.left.borrow().state();
        let rs = self.right.borrow().state();

        if ls == Nyes::Nk || rs == Nyes::Nk {
            self.state = Nyes::Nk;
            return Ok(None);
        }

        if (ls == Nyes::Constant || ls == Nyes::Independent)
            && (rs == Nyes::Constant || rs == Nyes::Independent)
        {
            if let Some((l, r)) = self.binary_values_own() {
                return Ok(Some(crate::ubc::compute_binary(&self.op, l, r)?));
            }
        }

        if ls.is_constanic() && rs.is_constanic() {
            self.state = Nyes::Woconstanic;
        }
        Ok(None)
    }
    fn state(&self) -> Nyes { self.state }
    fn set_state(&mut self, s: Nyes) { self.state = s; }
    fn children_mut(&mut self) -> Vec<&mut FirRef> {
        vec![&mut self.left, &mut self.right]
    }
    fn format(&self, buf: &mut String, depth: usize) -> std::fmt::Result {
        let indent = "  ".repeat(depth);
        writeln!(buf,"{}BinaryOp({}) [{}]", indent, self.op, self.state)?;
        self.left.borrow().format(buf,depth + 1)?;
        self.right.borrow().format(buf,depth + 1)?;
        Ok(())
    }
    fn clone_into_fir(&self) -> Fir { Fir::BinaryOp(Box::new(self.clone())) }
    fn fir_variant(&self) -> &'static str { "BinaryOp" }
    fn binary_values(&self) -> Option<(i64, i64)> { self.binary_values_own() }
}

impl BinaryOpFir {
    fn binary_values_own(&self) -> Option<(i64, i64)> {
        let l = self.left.borrow().as_int()?;
        let r = self.right.borrow().as_int()?;
        Some((l, r))
    }
}

impl Steppable for UnaryOpFir {
    fn step_one(&mut self, scope: &crate::ubc::Scope) -> Result<Option<Fir>, UbcError> {
        self.step_members(scope)?;
        let es = self.expr.borrow().state();

        match es {
            Nyes::Nk => { self.state = Nyes::Nk; Ok(None) }
            Nyes::Constant | Nyes::Independent => {
                if let Some(val) = self.expr.borrow().as_int() {
                    return Ok(Some(crate::ubc::compute_unary(&self.op, val)?));
                }
                Ok(None)
            }
            _ => { self.state = Nyes::Woconstanic; Ok(None) }
        }
    }
    fn state(&self) -> Nyes { self.state }
    fn set_state(&mut self, s: Nyes) { self.state = s; }
    fn children_mut(&mut self) -> Vec<&mut FirRef> { vec![&mut self.expr] }
    fn format(&self, buf: &mut String, depth: usize) -> std::fmt::Result {
        let indent = "  ".repeat(depth);
        writeln!(buf,"{}UnaryOp({}) [{}]", indent, self.op, self.state)?;
        self.expr.borrow().format(buf,depth + 1)
    }
    fn clone_into_fir(&self) -> Fir { Fir::UnaryOp(Box::new(self.clone())) }
    fn fir_variant(&self) -> &'static str { "UnaryOp" }
    fn unary_value(&self) -> Option<i64> { self.expr.borrow().as_int() }
    fn expr_state(&self) -> Nyes { self.expr.borrow().state() }
}

impl Steppable for SearchFir {
    fn step_one(&mut self, scope: &crate::ubc::Scope) -> Result<Option<Fir>, UbcError> {
        if self.state.is_constanic() {
            return Ok(None);
        }
        if self.anchored {
            self.step_anchored(scope)
        } else {
            self.step_unanchored(scope)
        }
    }
    fn state(&self) -> Nyes { self.state }
    fn set_state(&mut self, s: Nyes) { self.state = s; }
    fn children_mut(&mut self) -> Vec<&mut FirRef> { vec![] }
    fn format(&self, buf: &mut String, depth: usize) -> std::fmt::Result {
        let anchor_str = if self.anchored { "ANCHORED" } else { "FREE" };
        let indent = "  ".repeat(depth);
        writeln!(buf,"{}Search(pattern='{}', dir={}, {}) [{}]",
            indent, self.pattern, self.direction, anchor_str, self.state)?;
        if let Some(ref target) = self.target {
            target.borrow().format(buf,depth + 1)
        } else {
            Ok(())
        }
    }
    fn search_pattern(&self) -> Option<String> { Some(self.pattern.clone()) }
    fn search_anchored(&self) -> bool { self.anchored }
    fn search_target_ref(&self) -> Option<FirRef> { self.target.clone() }
    fn search_anchor_ref(&self) -> Option<FirRef> { self.anchor.clone() }
    fn clone_into_fir(&self) -> Fir { Fir::Search(Box::new(self.clone())) }
    fn fir_variant(&self) -> &'static str { "Search" }
}

impl SearchFir {
    fn step_unanchored(&mut self, scope: &crate::ubc::Scope) -> Result<Option<Fir>, UbcError> {
        match scope.search(&self.pattern) {
            Some(found) => {
                let stripped_fir = match found.borrow().fir_variant() {
                    "StayFullyFoolish" | "StayFoolish" => {
                        let found_fir = clone_steppable(&found);
                        match found_fir {
                            Fir::StayFullyFoolish(inner) => clone_steppable(&inner.expr),
                            Fir::StayFoolish(inner) => clone_steppable(&inner.expr),
                            _ => found_fir,
                        }
                    }
                    _ => clone_steppable(&found),
                };
                if scope.block_brane_searches() && matches!(stripped_fir, Fir::NormalBrane(_)) {
                    self.state = Nyes::Econstanic;
                } else {
                    let mut found_rc: FirRef = fir_to_ref(stripped_fir);
                    crate::ubc::run_to_completion_with_scope(&mut found_rc, scope)?;
                    self.target = Some(Rc::clone(&found_rc));
                    let cs = found_rc.borrow().state();
                    if cs == Nyes::Constant || cs == Nyes::Independent {
                        self.state = Nyes::Constant;
                        return Ok(self.target.take().map(|t| clone_steppable(&t)));
                    } else if cs.is_constanic() {
                        self.short_circuit_self();
                        self.state = Nyes::Woconstanic;
                    } else {
                        self.state = Nyes::Woconstanic;
                    }
                }
            }
            None => {
                self.state = Nyes::Econstanic;
            }
        }
        Ok(None)
    }

    fn step_anchored(&mut self, scope: &crate::ubc::Scope) -> Result<Option<Fir>, UbcError> {
        if let Some(anchor_rc) = self.anchor.clone() {
            let mut anchor = Rc::clone(&anchor_rc);
            crate::ubc::run_to_completion_with_scope(&mut anchor, scope)?;
            let resolved = crate::ubc::resolve_to_value(&anchor);
            let anchor_state = resolved.borrow().state();
            match anchor_state {
                Nyes::Nk => { self.state = Nyes::Nk; }
                Nyes::Constant | Nyes::Independent => {
                    match crate::search::search_in_brane(&resolved, &self.pattern) {
                        None => { self.state = Nyes::Nk; }
                        Some(found) => {
                            let cloned = crate::ubc::constanic_clone(&found, false);
                            self.target = Some(cloned);
                            let cs = self.target.as_ref().unwrap().borrow().state();
                            if cs == Nyes::Constant || cs == Nyes::Independent {
                                self.state = Nyes::Constant;
                                return Ok(self.target.take().map(|t| clone_steppable(&t)));
                            } else if cs.is_constanic() {
                                self.short_circuit_self();
                                self.state = Nyes::Woconstanic;
                            }
                        }
                    }
                }
                _ => { self.state = Nyes::Nk; }
            }
        } else {
            self.state = Nyes::Econstanic;
        }
        Ok(None)
    }

    /// Follow WOCONSTANIC chain in-place.
    fn short_circuit_self(&mut self) {
        let mut current = self.target.clone();
        loop {
            let local_rc = match current {
                Some(rc) => rc,
                None => break,
            };
            let state = local_rc.borrow().state();
            if state != Nyes::Woconstanic {
                self.target = Some(local_rc);
                break;
            }
            // Follow chain
            let next = if local_rc.borrow().fir_variant() == "Search" {
                local_rc.borrow().search_target_ref()
            } else {
                None
            };
            current = next;
        }
    }
}

impl Steppable for IndexFir {
    fn step_one(&mut self, scope: &crate::ubc::Scope) -> Result<Option<Fir>, UbcError> {
        if self.state.is_constanic() {
            return Ok(None);
        }
        if self.anchored {
            self.step_anchored(scope)
        } else {
            self.step_unanchored(scope)
        }
    }
    fn state(&self) -> Nyes { self.state }
    fn set_state(&mut self, s: Nyes) { self.state = s; }
    fn children_mut(&mut self) -> Vec<&mut FirRef> { vec![] }
    fn format(&self, buf: &mut String, depth: usize) -> std::fmt::Result {
        let anchor_str = if self.anchored { "ANCHORED" } else { "FREE" };
        let indent = "  ".repeat(depth);
        writeln!(buf, "{}Index(offset={}, {}) [{}]", indent, self.offset, anchor_str, self.state)
    }
    fn index_offset(&self) -> i32 { self.offset }
    fn index_anchored(&self) -> bool { self.anchored }
    fn index_anchor_ref(&self) -> Option<FirRef> { self.anchor.clone() }
    fn clone_into_fir(&self) -> Fir { Fir::Index(Box::new(self.clone())) }
    fn fir_variant(&self) -> &'static str { "Index" }
}

impl IndexFir {
    fn step_anchored(&mut self, scope: &crate::ubc::Scope) -> Result<Option<Fir>, UbcError> {
        if let Some(anchor_rc) = self.anchor.clone() {
            let mut anchor = Rc::clone(&anchor_rc);
            crate::ubc::run_to_completion_with_scope(&mut anchor, scope)?;
            let resolved = crate::ubc::resolve_to_value(&anchor);
            match resolved.borrow().state() {
                Nyes::Nk => { self.state = Nyes::Nk; }
                Nyes::Constant | Nyes::Independent => {
                    match crate::search::index_in_brane(&resolved, self.offset) {
                        None => { self.state = Nyes::Nk; }
                        Some(found) => {
                            let cloned = crate::ubc::constanic_clone(&found, false);
                            return Ok(Some(clone_steppable(&cloned)));
                        }
                    }
                }
                _ => { self.state = Nyes::Nk; }
            }
        }
        Ok(None)
    }

    fn step_unanchored(&mut self, scope: &crate::ubc::Scope) -> Result<Option<Fir>, UbcError> {
        if let (Some(brane), Some(stmt_idx)) = (scope.current_brane(), scope.current_stmt_idx()) {
            let target = stmt_idx as i32 + self.offset;
            match crate::search::index_in_brane(&brane, target) {
                None => { self.state = Nyes::Nk; }
                Some(found) => {
                    let cloned = crate::ubc::constanic_clone(&found, false);
                    return Ok(Some(clone_steppable(&cloned)));
                }
            }
        } else {
            self.state = Nyes::Econstanic;
        }
        Ok(None)
    }
}

impl Steppable for HeadTailFir {
    fn step_one(&mut self, scope: &crate::ubc::Scope) -> Result<Option<Fir>, UbcError> {
        if self.state.is_constanic() {
            return Ok(None);
        }
        if let Some(anchor_rc) = self.anchor.clone() {
            let mut anchor = Rc::clone(&anchor_rc);
            crate::ubc::run_to_completion_with_scope(&mut anchor, scope)?;
            let resolved = crate::ubc::resolve_to_value(&anchor);
            match resolved.borrow().state() {
                Nyes::Nk => { self.state = Nyes::Nk; }
                Nyes::Constant | Nyes::Independent => {
                    let found = if self.is_head {
                        crate::search::head_of_brane(&resolved)
                    } else {
                        crate::search::tail_of_brane(&resolved)
                    };
                    match found {
                        None => { self.state = Nyes::Nk; }
                        Some(f) => {
                            let cloned = crate::ubc::constanic_clone(&f, false);
                            return Ok(Some(clone_steppable(&cloned)));
                        }
                    }
                }
                _ => { self.state = Nyes::Nk; }
            }
        }
        Ok(None)
    }
    fn state(&self) -> Nyes { self.state }
    fn set_state(&mut self, s: Nyes) { self.state = s; }
    fn children_mut(&mut self) -> Vec<&mut FirRef> { vec![] }
    fn format(&self, buf: &mut String, depth: usize) -> std::fmt::Result {
        let ht = if self.is_head { "HEAD" } else { "TAIL" };
        let anchor_str = if self.anchored { "ANCHORED" } else { "FREE" };
        let indent = "  ".repeat(depth);
        writeln!(buf, "{}HeadTail({}, {}) [{}]", indent, ht, anchor_str, self.state)
    }
    fn headtail_is_head(&self) -> bool { self.is_head }
    fn headtail_anchor_ref(&self) -> Option<FirRef> { self.anchor.clone() }
    fn clone_into_fir(&self) -> Fir { Fir::HeadTail(Box::new(self.clone())) }
    fn fir_variant(&self) -> &'static str { "HeadTail" }
}

impl Steppable for StayFoolishFir {
    fn step_one(&mut self, scope: &crate::ubc::Scope) -> Result<Option<Fir>, UbcError> {
        if self.state.is_constanic() {
            return Ok(None);
        }
        let stepped = crate::ubc::step_except_brane_searches(&self.expr, scope)?;
        self.expr = fir_to_ref(stepped);
        let es = self.expr.borrow().state();
        if es == Nyes::Constant || es == Nyes::Independent {
            self.state = Nyes::Constant;
        } else {
            self.state = Nyes::Woconstanic;
        }
        Ok(None)
    }
    fn state(&self) -> Nyes { self.state }
    fn set_state(&mut self, s: Nyes) { self.state = s; }
    fn children_mut(&mut self) -> Vec<&mut FirRef> { vec![&mut self.expr] }
    fn format(&self, buf: &mut String, depth: usize) -> std::fmt::Result {
        let indent = "  ".repeat(depth);
        writeln!(buf,"{}StayFoolish [{}]", indent, self.state)?;
        self.expr.borrow().format(buf,depth + 1)
    }
    fn clone_into_fir(&self) -> Fir { Fir::StayFoolish(Box::new(self.clone())) }
    fn fir_variant(&self) -> &'static str { "StayFoolish" }
}

impl Steppable for StayFullyFoolishFir {
    fn step_one(&mut self, _: &crate::ubc::Scope) -> Result<Option<Fir>, UbcError> {
        Ok(None)
    }
    fn state(&self) -> Nyes { self.state }
    fn set_state(&mut self, s: Nyes) { self.state = s; }
    fn children_mut(&mut self) -> Vec<&mut FirRef> { vec![&mut self.expr] }
    fn format(&self, buf: &mut String, depth: usize) -> std::fmt::Result {
        let indent = "  ".repeat(depth);
        writeln!(buf,"{}StayFullyFoolish [{}]", indent, self.state)?;
        self.expr.borrow().format(buf,depth + 1)
    }
    fn clone_into_fir(&self) -> Fir { Fir::StayFullyFoolish(Box::new(self.clone())) }
    fn fir_variant(&self) -> &'static str { "StayFullyFoolish" }
}

impl Steppable for ConcatenationFir {
    fn step_one(&mut self, scope: &crate::ubc::Scope) -> Result<Option<Fir>, UbcError> {
        // If we already have a merged brane, step it
        if let Some(merged_rc) = self.merged.clone() {
            let mut merged = Rc::clone(&merged_rc);
            crate::ubc::run_to_completion_with_scope(&mut merged, scope)?;
            if merged.borrow().state().is_constanic() {
                return Ok(Some(clone_steppable(&merged)));
            }
            self.merged = Some(Rc::clone(&merged));
            self.state = Nyes::Braning;
            return Ok(None);
        }

        // Step each element and build merged brane
        let stepped: Vec<Fir> = self.elements.iter()
            .map(|e| crate::ubc::step_boxed(e, scope))
            .collect::<Result<_, _>>()?;

        if stepped.iter().any(|e| e.state() == Nyes::Nk) {
            self.state = Nyes::Nk;
            return Ok(None);
        }

        let merged_statements: Vec<StatementFir> = stepped.iter()
            .flat_map(|elem| {
                let elem_ref: FirRef = fir_to_ref(elem.clone());
                let resolved = crate::ubc::resolve_to_value(&elem_ref);
                if resolved.borrow().fir_variant() == "NormalBrane" {
                    let resolved_fir = clone_steppable(&resolved);
                    if let Fir::NormalBrane(inner) = resolved_fir {
                        inner.statements.iter().map(|stmt| {
                            StatementFir {
                                name: stmt.name.clone(),
                                body: fir_to_ref(clone_steppable(&stmt.body)),
                                state: if stmt.body.borrow().state().is_constanic() {
                                    stmt.body.borrow().state()
                                } else {
                                    Nyes::Embryonic
                                },
                            }
                        }).collect::<Vec<_>>()
                    } else { vec![] }
                } else { vec![] }
            })
            .collect();

        let brane_state = crate::ubc::compute_brane_state(&merged_statements);
        let merged = Fir::NormalBrane(Box::new(NormalBraneFir {
            characterizations: vec![],
            statements: merged_statements,
            state: brane_state,
        }));
        self.merged = Some(fir_to_ref(merged));
        self.state = Nyes::Braning;
        Ok(None)
    }
    fn state(&self) -> Nyes { self.state }
    fn set_state(&mut self, s: Nyes) { self.state = s; }
    fn children_mut(&mut self) -> Vec<&mut FirRef> {
        self.elements.iter_mut().collect()
    }
    fn format(&self, buf: &mut String, depth: usize) -> std::fmt::Result {
        let indent = "  ".repeat(depth);
        writeln!(buf,"{}Concatenation(elements={}) [{}]", indent, self.elements.len(), self.state)?;
        for elem in &self.elements {
            elem.borrow().format(buf,depth + 1)?;
        }
        Ok(())
    }
    fn concat_merged_ref(&self) -> Option<FirRef> { self.merged.clone() }
    fn clone_into_fir(&self) -> Fir { Fir::Concatenation(Box::new(self.clone())) }
    fn fir_variant(&self) -> &'static str { "Concatenation" }
    fn concat_elements(&self) -> Vec<FirRef> {
        self.elements.iter().map(Rc::clone).collect()
    }
}

impl Steppable for NormalBraneFir {
    fn step_one(&mut self, scope: &crate::ubc::Scope) -> Result<Option<Fir>, UbcError> {
        match self.state {
            Nyes::Prembrionic => {
                self.state = Nyes::Embryonic;
                Ok(None)
            }
            Nyes::Embryonic => {
                self.state = Nyes::Braning;
                Ok(None)
            }
            Nyes::Braning | Nyes::Woconstanic => {
                crate::ubc::re_step_brane_bodies(self, scope)?;
                Ok(None)
            }
            _ => Ok(None),
        }
    }
    fn state(&self) -> Nyes { self.state }
    fn set_state(&mut self, s: Nyes) { self.state = s; }
    fn children_mut(&mut self) -> Vec<&mut FirRef> {
        self.statements.iter_mut().map(|s| &mut s.body).collect()
    }
    fn format(&self, buf: &mut String, depth: usize) -> std::fmt::Result {
        let chars = if self.characterizations.is_empty() {
            String::new()
        } else {
            format!("{}'", self.characterizations.join(" "))
        };
        let indent = "  ".repeat(depth);
        writeln!(buf,"{}{}Brane [{}]", indent, chars, self.state)?;
        for stmt in &self.statements {
            if let Some(ref name) = stmt.name {
                writeln!(buf,"{}{} = ", "  ".repeat(depth + 1), name)?;
            }
            stmt.body.borrow().format(buf,depth + 1)?;
        }
        Ok(())
    }
    fn normal_brane_statements(&self) -> Vec<StatementFir> { self.statements.clone() }
    fn as_brane_statements(&self) -> Option<Vec<StatementFir>> { Some(self.statements.clone()) }
    fn brane_statement_count(&self) -> usize { self.statements.len() }
    fn clone_into_fir(&self) -> Fir { Fir::NormalBrane(Box::new(self.clone())) }
    fn fir_variant(&self) -> &'static str { "NormalBrane" }
}

// ==================== Helper: extract struct from enum ====================

impl Fir {
    pub fn as_const_int(&self) -> Option<&ConstantIntFir> {
        if let Fir::ConstantInt(inner) = self { Some(inner) } else { None }
    }
    pub fn as_nk(&self) -> Option<&NkFir> {
        if let Fir::Nk(inner) = self { Some(inner) } else { None }
    }
    pub fn as_binary_op(&self) -> Option<&BinaryOpFir> {
        if let Fir::BinaryOp(inner) = self { Some(inner) } else { None }
    }
    pub fn as_unary_op(&self) -> Option<&UnaryOpFir> {
        if let Fir::UnaryOp(inner) = self { Some(inner) } else { None }
    }
    pub fn as_search(&self) -> Option<&SearchFir> {
        if let Fir::Search(inner) = self { Some(inner) } else { None }
    }
    pub fn as_index(&self) -> Option<&IndexFir> {
        if let Fir::Index(inner) = self { Some(inner) } else { None }
    }
    pub fn as_head_tail(&self) -> Option<&HeadTailFir> {
        if let Fir::HeadTail(inner) = self { Some(inner) } else { None }
    }
    pub fn as_stay_foolish(&self) -> Option<&StayFoolishFir> {
        if let Fir::StayFoolish(inner) = self { Some(inner) } else { None }
    }
    pub fn as_stay_fully_foolish(&self) -> Option<&StayFullyFoolishFir> {
        if let Fir::StayFullyFoolish(inner) = self { Some(inner) } else { None }
    }
    pub fn as_concatenation(&self) -> Option<&ConcatenationFir> {
        if let Fir::Concatenation(inner) = self { Some(inner) } else { None }
    }
    pub fn as_normal_brane(&self) -> Option<&NormalBraneFir> {
        if let Fir::NormalBrane(inner) = self { Some(inner) } else { None }
    }
}

// ==================== Manual Serde for Fir ====================

fn fir_to_json(fir: &Fir) -> serde_json::Value {
    use serde_json::{Map, Value};
    match fir {
        Fir::ConstantInt(inner) => {
            let mut m = Map::new();
            m.insert("type".into(), Value::String("ConstantInt".into()));
            m.insert("value".into(), Value::Number(inner.value.into()));
            m.insert("state".into(), serde_json::to_value(&inner.state).unwrap());
            Value::Object(m)
        }
        Fir::Nk(inner) => {
            let mut m = Map::new();
            m.insert("type".into(), Value::String("Nk".into()));
            m.insert("reason".into(), Value::String(inner.reason.clone()));
            m.insert("state".into(), serde_json::to_value(&inner.state).unwrap());
            Value::Object(m)
        }
        Fir::BinaryOp(inner) => {
            let mut m = Map::new();
            m.insert("type".into(), Value::String("BinaryOp".into()));
            m.insert("op".into(), Value::String(inner.op.clone()));
            m.insert("left".into(), fir_to_json(&clone_steppable(&inner.left)));
            m.insert("right".into(), fir_to_json(&clone_steppable(&inner.right)));
            m.insert("state".into(), serde_json::to_value(&inner.state).unwrap());
            Value::Object(m)
        }
        Fir::UnaryOp(inner) => {
            let mut m = Map::new();
            m.insert("type".into(), Value::String("UnaryOp".into()));
            m.insert("op".into(), Value::String(inner.op.clone()));
            m.insert("expr".into(), fir_to_json(&clone_steppable(&inner.expr)));
            m.insert("state".into(), serde_json::to_value(&inner.state).unwrap());
            Value::Object(m)
        }
        Fir::Search(inner) => {
            let mut m = Map::new();
            m.insert("type".into(), Value::String("Search".into()));
            m.insert("pattern".into(), Value::String(inner.pattern.clone()));
            m.insert("direction".into(), serde_json::to_value(&inner.direction).unwrap());
            m.insert("anchored".into(), Value::Bool(inner.anchored));
            m.insert("state".into(), serde_json::to_value(&inner.state).unwrap());
            if let Some(ref a) = inner.anchor { m.insert("anchor".into(), fir_to_json(&clone_steppable(a))); }
            if let Some(ref t) = inner.target { m.insert("target".into(), fir_to_json(&clone_steppable(t))); }
            Value::Object(m)
        }
        Fir::Index(inner) => {
            let mut m = Map::new();
            m.insert("type".into(), Value::String("Index".into()));
            m.insert("offset".into(), Value::Number(inner.offset.into()));
            m.insert("anchored".into(), Value::Bool(inner.anchored));
            m.insert("state".into(), serde_json::to_value(&inner.state).unwrap());
            if let Some(ref a) = inner.anchor { m.insert("anchor".into(), fir_to_json(&clone_steppable(a))); }
            Value::Object(m)
        }
        Fir::HeadTail(inner) => {
            let mut m = Map::new();
            m.insert("type".into(), Value::String("HeadTail".into()));
            m.insert("is_head".into(), Value::Bool(inner.is_head));
            m.insert("anchored".into(), Value::Bool(inner.anchored));
            m.insert("state".into(), serde_json::to_value(&inner.state).unwrap());
            if let Some(ref a) = inner.anchor { m.insert("anchor".into(), fir_to_json(&clone_steppable(a))); }
            Value::Object(m)
        }
        Fir::StayFoolish(inner) => {
            let mut m = Map::new();
            m.insert("type".into(), Value::String("StayFoolish".into()));
            m.insert("expr".into(), fir_to_json(&clone_steppable(&inner.expr)));
            m.insert("state".into(), serde_json::to_value(&inner.state).unwrap());
            Value::Object(m)
        }
        Fir::StayFullyFoolish(inner) => {
            let mut m = Map::new();
            m.insert("type".into(), Value::String("StayFullyFoolish".into()));
            m.insert("expr".into(), fir_to_json(&clone_steppable(&inner.expr)));
            m.insert("state".into(), serde_json::to_value(&inner.state).unwrap());
            Value::Object(m)
        }
        Fir::Concatenation(inner) => {
            let mut m = Map::new();
            m.insert("type".into(), Value::String("Concatenation".into()));
            m.insert("elements".into(), Value::Array(inner.elements.iter().map(|e| fir_to_json(&clone_steppable(e))).collect()));
            m.insert("state".into(), serde_json::to_value(&inner.state).unwrap());
            if let Some(ref mg) = inner.merged { m.insert("merged".into(), fir_to_json(&clone_steppable(mg))); }
            Value::Object(m)
        }
        Fir::NormalBrane(inner) => {
            let mut m = Map::new();
            m.insert("type".into(), Value::String("NormalBrane".into()));
            m.insert("characterizations".into(), Value::Array(inner.characterizations.iter().map(|c| Value::String(c.clone())).collect()));
            let stmts: Vec<Value> = inner.statements.iter().map(|s| {
                let mut sm = Map::new();
                sm.insert("body".into(), fir_to_json(&clone_steppable(&s.body)));
                sm.insert("state".into(), serde_json::to_value(&s.state).unwrap());
                if let Some(ref n) = s.name { sm.insert("name".into(), Value::String(n.clone())); }
                Value::Object(sm)
            }).collect();
            m.insert("statements".into(), Value::Array(stmts));
            m.insert("state".into(), serde_json::to_value(&inner.state).unwrap());
            Value::Object(m)
        }
    }
}

impl Serialize for Fir {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        fir_to_json(self).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Fir {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(deserializer)?;
        let obj = value.as_object().ok_or_else(|| serde::de::Error::custom("expected JSON object"))?;
        let type_name = obj.get("type").and_then(|v| v.as_str()).ok_or_else(|| serde::de::Error::custom("missing type field"))?;
        let state = obj.get("state").and_then(|v| serde_json::from_value::<Nyes>(v.clone()).ok()).unwrap_or(Nyes::Embryonic);
        match type_name {
            "ConstantInt" => {
                let value = obj.get("value").and_then(|v| v.as_i64()).unwrap_or(0);
                Ok(Fir::ConstantInt(Box::new(ConstantIntFir { value, state })))
            }
            "Nk" => {
                let reason = obj.get("reason").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
                Ok(Fir::Nk(Box::new(NkFir { reason, state })))
            }
            _ => Err(serde::de::Error::custom(format!("unsupported Fir type for deserialization: {}", type_name))),
        }
    }
}
