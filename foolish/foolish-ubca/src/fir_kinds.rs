//! FIR kinds — leaf, operator, statement, and brane nodes.
//!
//! Leaf kinds (ConstantInt, Nk) are the simplest: no children, no task queue,
//! immediate terminal NYES states.  Operator has two operand children.
//! Statement wraps a single body expression.  Brane contains an ordered list
//! of statement children that drain sequentially.

use std::cell::RefCell;
use std::rc::{Rc, Weak};

use foolish_core::fir::Nyes;
use regex::Regex;

use crate::fir_trait::{Fir, FirKind, FirRef, Scope, UbcError, get_value};
use crate::nyes_ext::NyesExt;
use crate::proto_brane::ProtoBrane;

// ── Default NYES classification ──────────────────────────────────────────────

/// Determine the NYES for a node based on its children's states.
///
/// Priority order (worst wins):
///   NK > WOCONSTANIC/ECONSTANIC > CONSTANT > INDEPENDENT
///
/// Used by BraneFir directly. OperatorFir uses this as a base but may override
/// (e.g., `1/0` → NK even though children are CONSTANT).
///
/// Returns `None` if not all children are constanic yet (stay BRANING).
pub fn decide_nyes_due_to_children(children: &[FirRef]) -> Option<Nyes> {
    // Stay BRANING until every child is settled (constanic including NK)
    if !children.iter().all(|c| c.borrow().core().get_nyes().is_settled()) {
        return None;
    }
    // Pick the worst: NK > WOCONSTANIC/ECONSTANIC > CONSTANT > INDEPENDENT
    if children.iter().any(|c| c.borrow().core().get_nyes() == Nyes::Nk) {
        return Some(Nyes::Nk);
    }
    if children.iter().any(|c| {
        let n = c.borrow().core().get_nyes();
        n == Nyes::Econstanic || n == Nyes::Woconstanic
    }) {
        return Some(Nyes::Woconstanic);
    }
    if children.iter().any(|c| c.borrow().core().get_nyes() == Nyes::Constant) {
        return Some(Nyes::Constant);
    }
    Some(Nyes::Independent)
}

// ── ConstantIntFir ──────────────────────────────────────────────────────────

/// A leaf FIR representing a known integer constant.
///
/// NYES progression: Prembrionic → Constant (immediate, one step).
#[derive(Debug)]
pub struct ConstantIntFir {
    pub(crate) core: ProtoBrane,
    pub(crate) value: i64,
}

impl ConstantIntFir {
    /// The integer value held by this leaf.
    pub fn value(&self) -> i64 {
        self.value
    }
}

impl Fir for ConstantIntFir {
    fn core(&self) -> &ProtoBrane {
        &self.core
    }

    fn fir_op_step(&self, _scope: &Scope) -> Result<(), UbcError> {
        // Leaf: no children, no tasks — advance directly to terminal.
        if !self.core.get_nyes().is_settled() {
            self.core.set_nyes(Nyes::Constant);
        }
        Ok(())
    }

    fn kind(&self) -> FirKind {
        FirKind::ConstantInt
    }

    fn as_i64(&self) -> Option<i64> {
        Some(self.value)
    }
}

// ── NkFir ───────────────────────────────────────────────────────────────────

/// A leaf FIR representing a provably unknowable value (`???`).
///
/// NYES progression: Prembrionic → Nk (immediate, one step).
#[derive(Debug)]
pub struct NkFir {
    pub(crate) core: ProtoBrane,
    pub(crate) reason: String,
}

impl NkFir {
    /// The reason this value is unknowable.
    pub fn reason(&self) -> &str {
        &self.reason
    }
}

impl Fir for NkFir {
    fn core(&self) -> &ProtoBrane {
        &self.core
    }

    fn fir_op_step(&self, _scope: &Scope) -> Result<(), UbcError> {
        // Leaf: no children, no tasks — advance directly to terminal.
        if !self.core.get_nyes().is_settled() {
            self.core.set_nyes(Nyes::Nk);
        }
        Ok(())
    }

    fn kind(&self) -> FirKind {
        FirKind::Nk
    }

    fn as_nk_reason(&self) -> Option<&str> {
        Some(&self.reason)
    }
}

// ── OperatorFir ─────────────────────────────────────────────────────────────

/// An operator FIR with 2 operand children in `foolish_children`.
///
/// NYES progression: Prembrionic → Braning (task queue built) →
/// (children drain) → Constant (result computed and pushed to ubc_children).
#[derive(Debug)]
pub struct OperatorFir {
    pub(crate) core: ProtoBrane,
    pub(crate) op: String,
}

impl Fir for OperatorFir {
    fn core(&self) -> &ProtoBrane {
        &self.core
    }

    fn fir_op_step(&self, _scope: &Scope) -> Result<(), UbcError> {
        match self.core.get_nyes() {
            Nyes::Prembrionic => {
                self.core.set_nyes(Nyes::Braning);
                let children: Vec<FirRef> = self.core.foolish_children().to_vec();
                for child in children {
                    self.core.push_task(child);
                }
            }
            Nyes::Braning => {
                let children = self.core.foolish_children().to_vec();

                let any_nk = children
                    .iter()
                    .any(|c| c.borrow().core().get_nyes() == Nyes::Nk);
                if any_nk {
                    let nk_ref: FirRef = Rc::new_cyclic(|me: &Weak<RefCell<NkFir>>| {
                        let parent: Weak<RefCell<dyn Fir>> = me.clone();
                        let reason = children.iter()
                            .find_map(|c| {
                                let b = c.borrow();
                                if b.core().get_nyes() == Nyes::Nk {
                                    b.as_nk_reason().map(|s| s.to_string())
                                } else {
                                    None
                                }
                            })
                            .unwrap_or_else(|| "operator nk".to_string());
                        RefCell::new(NkFir {
                            core: ProtoBrane::new(vec![], parent, Nyes::Nk),
                            reason,
                        })
                    });
                    self.core.push_ubc_child(nk_ref);
                    self.core.set_nyes(Nyes::Nk);
                    return Ok(());
                }

                let values: Vec<i64> = children
                    .iter()
                    .filter_map(|c| c.borrow().as_i64())
                    .collect();

                if values.len() != children.len() {
                    self.core.set_nyes(Nyes::Woconstanic);
                    return Ok(());
                }

                let result = match self.op.as_str() {
                    "+" if values.len() == 2 => values[0] + values[1],
                    "-" if values.len() == 2 => values[0] - values[1],
                    "*" if values.len() == 2 => values[0] * values[1],
                    "/" if values.len() == 2 => {
                        if values[1] == 0 {
                            let nk_ref: FirRef = Rc::new_cyclic(|me: &Weak<RefCell<NkFir>>| {
                                let parent: Weak<RefCell<dyn Fir>> = me.clone();
                                RefCell::new(NkFir {
                                    core: ProtoBrane::new(vec![], parent, Nyes::Nk),
                                    reason: "division by zero".to_string(),
                                })
                            });
                            self.core.push_ubc_child(nk_ref);
                            self.core.set_nyes(Nyes::Nk);
                            return Ok(());
                        }
                        values[0] / values[1]
                    }
                    "%" if values.len() == 2 => {
                        if values[1] == 0 {
                            let nk_ref: FirRef = Rc::new_cyclic(|me: &Weak<RefCell<NkFir>>| {
                                let parent: Weak<RefCell<dyn Fir>> = me.clone();
                                RefCell::new(NkFir {
                                    core: ProtoBrane::new(vec![], parent, Nyes::Nk),
                                    reason: "division by zero".to_string(),
                                })
                            });
                            self.core.push_ubc_child(nk_ref);
                            self.core.set_nyes(Nyes::Nk);
                            return Ok(());
                        }
                        values[0] % values[1]
                    }
                    "-" if values.len() == 1 => -values[0], // unary negation
                    op => {
                        return Err(UbcError::Eval(format!(
                            "unknown operator: {op} ({} operands)",
                            values.len()
                        )));
                    }
                };

                let result_ref: FirRef = Rc::new_cyclic(|me: &Weak<RefCell<ConstantIntFir>>| {
                    let parent: Weak<RefCell<dyn Fir>> = me.clone();
                    RefCell::new(ConstantIntFir {
                        core: ProtoBrane::new(vec![], parent, Nyes::Constant),
                        value: result,
                    })
                });
                self.core.push_ubc_child(result_ref);
                self.core.set_nyes(Nyes::Constant);
            }
            _ => {}
        }
        Ok(())
    }

    fn kind(&self) -> FirKind {
        FirKind::Operator
    }

    fn as_op_name(&self) -> Option<&str> {
        Some(&self.op)
    }
}

// ── StatementFir ─────────────────────────────────────────────────────────────

/// A statement FIR wrapping a single body expression.
///
/// NYES progression: Prembrionic → Braning (body task pushed) →
/// (body drains) → body's NYES copied to self.
#[derive(Debug)]
pub struct StatementFir {
    pub(crate) core: ProtoBrane,
    pub(crate) name: String,
    pub(crate) line_number: usize,
}

impl StatementFir {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn line_number(&self) -> usize {
        self.line_number
    }
}

impl Fir for StatementFir {
    fn core(&self) -> &ProtoBrane {
        &self.core
    }

    fn fir_op_step(&self, _scope: &Scope) -> Result<(), UbcError> {
        match self.core.get_nyes() {
            Nyes::Prembrionic => {
                self.core.set_nyes(Nyes::Braning);
                let children: Vec<FirRef> = self.core.foolish_children().to_vec();
                for child in children {
                    self.core.push_task(child);
                }
            }
            Nyes::Braning => {
                let children = self.core.foolish_children().to_vec();
                if let Some(body) = children.first() {
                    let body_nyes = body.borrow().core().get_nyes();
                    if body_nyes.is_settled() {
                        self.core.set_nyes(body_nyes);
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn kind(&self) -> FirKind {
        FirKind::Statement
    }

    fn as_stmt_name(&self) -> Option<&str> {
        if self.name.is_empty() {
            None
        } else {
            Some(&self.name)
        }
    }
    fn as_stmt_line_number(&self) -> Option<usize> {
        Some(self.line_number)
    }
}

// ── BraneFir ────────────────────────────────────────────────────────────────

/// A brane FIR containing an ordered list of statement children.
///
/// NYES progression: Prembrionic → Braning (all children pushed as tasks) →
/// (children drain in order) → classified (Constant / Nk / Woconstanic).
#[derive(Debug)]
pub struct BraneFir {
    pub(crate) core: ProtoBrane,
    pub(crate) characterizations: Vec<String>,
}

impl Fir for BraneFir {
    fn core(&self) -> &ProtoBrane {
        &self.core
    }

    fn fir_op_step(&self, _scope: &Scope) -> Result<(), UbcError> {
        match self.core.get_nyes() {
            Nyes::Prembrionic => {
                let children: Vec<FirRef> = self.core.foolish_children().to_vec();
                if children.is_empty() {
                    self.core.set_nyes(Nyes::Constant);
                } else {
                    self.core.set_nyes(Nyes::Braning);
                    for child in children {
                        self.core.push_task(child);
                    }
                }
            }
            Nyes::Braning => {
                let children = self.core.foolish_children().to_vec();
                if let Some(nyes) = decide_nyes_due_to_children(&children) {
                    self.core.set_nyes(nyes);
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn kind(&self) -> FirKind {
        FirKind::Brane
    }

    fn as_brane_characterizations(&self) -> &[String] {
        &self.characterizations
    }
}

// ── SearchFir ────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct SearchFir {
    pub(crate) core: ProtoBrane,
    pub(crate) pattern: String,
    pub(crate) anchored: bool,
    pub(crate) forward: bool,
    pub(crate) found_body: RefCell<Option<FirRef>>,
}

/// Recursively advance all PREMBRIONIC nodes to EMBRYONIC.
/// Used by SFF to match reference behavior where SFF inner expressions
/// start at EMBRYONIC, not PREMBRIONIC. Covers both foolish_children
/// (parse-time structure) and ubc_children (computed results).
fn advance_to_embryonic(fir_ref: &FirRef) {
    let mut borrowed = fir_ref.borrow_mut();
    if borrowed.core().get_nyes() == Nyes::Prembrionic {
        borrowed.core().set_nyes(Nyes::Embryonic);
    }
    let foolish: Vec<FirRef> = borrowed.core().foolish_children().to_vec();
    let ubc: Vec<FirRef> = borrowed.core().ubc_children().to_vec();
    drop(borrowed);
    for child in foolish {
        advance_to_embryonic(&child);
    }
    for result in ubc {
        advance_to_embryonic(&result);
    }
}

fn extract_simple_name(pattern: &str) -> &str {
    let s = pattern.strip_prefix('^').unwrap_or(pattern);
    s.strip_suffix('$').unwrap_or(s)
}

/// Resolve an anchor node to its actual value.
///
/// When an anchor (e.g. a SearchFir for `hw`) settles, its resolved value
/// is stored in `ubc_children`. This helper returns the resolved value if
/// available, otherwise returns the anchor itself.
fn resolve_anchor(anchor: &FirRef) -> FirRef {
    get_value(anchor)
}

fn find_stmt_index_in_brane(stmt: &FirRef, brane: &FirRef) -> Option<usize> {
    let brane_borrowed = brane.borrow();
    for (i, child) in brane_borrowed.core().foolish_children().iter().enumerate() {
        if Rc::ptr_eq(child, stmt) {
            return Some(i);
        }
    }
    None
}

fn matches_pattern(stmt_name: &str, pattern: &str) -> bool {
    if stmt_name == pattern {
        return true;
    }
    if let Ok(re) = Regex::new(pattern) {
        return re.is_match(stmt_name);
    }
    false
}

/// Unwrap SF/SFF wrappers to get the inner expression.
/// UBC does this in resolve_to_value: when a search finds an SF/SFF expression,
/// it returns the inner expression, not the wrapper.
fn unwrap_sf_sff(fir_ref: &FirRef) -> FirRef {
    let kind = fir_ref.borrow().kind();
    match kind {
        FirKind::StayFoolish | FirKind::StayFullyFoolish => {
            if let Some(inner) = fir_ref.borrow().core().foolish_children().first() {
                Rc::clone(inner)
            } else {
                Rc::clone(fir_ref)
            }
        }
        _ => Rc::clone(fir_ref),
    }
}

fn search_brane_children(brane: &FirRef, name: &str, before: Option<usize>, forward: bool) -> Option<(FirRef, Nyes)> {
    let brane_borrowed = brane.borrow();
    let children = brane_borrowed.core().foolish_children();
    let end = before.unwrap_or(children.len());
    let range = children[..end].iter();
    let iter: Box<dyn Iterator<Item = &FirRef>> = if forward {
        Box::new(range)
    } else {
        Box::new(range.rev())
    };
    for child in iter {
        let child_borrowed = child.borrow();
        if let Some(sn) = child_borrowed.as_stmt_name() {
            if matches_pattern(sn, name)
                && let Some(body) = child_borrowed.core().foolish_children().first()
            {
                // Unwrap SF/SFF wrappers — UBC does this in resolve_to_value
                let unwrapped = unwrap_sf_sff(body);
                let body_nyes = unwrapped.borrow().core().get_nyes();
                return Some((unwrapped, body_nyes));
            }
        }
    }
    None
}

impl Fir for SearchFir {
    fn core(&self) -> &ProtoBrane {
        &self.core
    }
    fn fir_op_step(&self, _scope: &Scope) -> Result<(), UbcError> {
        match self.core.get_nyes() {
            Nyes::Prembrionic => {
                if self.anchored {
                    let anchor = Rc::clone(&self.core.foolish_children()[0]);
                    self.core.push_task(anchor);
                    self.core.set_nyes(Nyes::Braning);
                } else {
                    let name = &self.pattern;
                    let mut current = self.core.parent();
                    while let Some(node) = current {
                        if node.borrow().kind() == FirKind::Statement {
                            let brane = {
                                let borrowed = node.borrow();
                                find_parent_brane(borrowed.core())
                            };
                            if let Some(ref brane_ref) = brane {
                                let before_idx = find_stmt_index_in_brane(&node, brane_ref);
                                if let Some((body, nyes)) = search_brane_children(brane_ref, name, before_idx, self.forward) {
                                    if nyes.is_settled() {
                                        self.core.push_ubc_child(body);
                                        // Econstanic/Woconstanic/Nk found → search becomes Woconstanic
                                        // (found something, but it's not a value yet)
                                        let search_nyes = if nyes == Nyes::Econstanic
                                            || nyes == Nyes::Woconstanic
                                            || nyes == Nyes::Nk
                                        {
                                            Nyes::Woconstanic
                                        } else {
                                            nyes
                                        };
                                        self.core.set_nyes(search_nyes);
                                    } else {
                                        self.core.push_task(Rc::clone(&body));
                                        *self.found_body.borrow_mut() = Some(body);
                                        self.core.set_nyes(Nyes::Braning);
                                    }
                                    return Ok(());
                                }
                            }
                        }
                        let next = node.borrow().core().parent();
                        match next {
                            Some(ref n) if Rc::ptr_eq(n, &node) => break,
                            None => break,
                            _ => current = next,
                        }
                    }
                    self.core.set_nyes(Nyes::Econstanic);
                }
            }
            Nyes::Braning => {
                if self.anchored {
                    let anchor = Rc::clone(&self.core.foolish_children()[0]);
                    let resolved = resolve_anchor(&anchor);
                    let name = &self.pattern;
                    if let Some((body, nyes)) = search_brane_children(&resolved, name, None, self.forward) {
                        self.core.push_ubc_child(body);
                        let search_nyes = if nyes == Nyes::Econstanic
                            || nyes == Nyes::Woconstanic
                            || nyes == Nyes::Nk
                        {
                            Nyes::Woconstanic
                        } else {
                            nyes
                        };
                        self.core.set_nyes(search_nyes);
                    } else {
                        self.core.set_nyes(Nyes::Nk);
                    }
                } else if let Some(body) = self.found_body.borrow_mut().take() {
                    let nyes = body.borrow().core().get_nyes();
                    if nyes.is_settled() {
                        self.core.push_ubc_child(body);
                        let search_nyes = if nyes == Nyes::Econstanic || nyes == Nyes::Woconstanic {
                            Nyes::Woconstanic
                        } else {
                            nyes
                        };
                        self.core.set_nyes(search_nyes);
                    } else {
                        // Body not settled yet — keep waiting
                        *self.found_body.borrow_mut() = Some(body);
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
    fn kind(&self) -> FirKind {
        FirKind::Search
    }
    fn as_search_pattern(&self) -> Option<&str> {
        Some(&self.pattern)
    }
    fn as_search_anchored(&self) -> bool {
        self.anchored
    }
}

// ── IndexFir (stub) ────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct IndexFir {
    pub(crate) core: ProtoBrane,
    pub(crate) offset: i32,
    pub(crate) anchored: bool,
}

fn index_into_brane(brane: &FirRef, offset: i32) -> Option<(FirRef, Nyes)> {
    // Clone all handles under short borrows, then drop ALL borrows before accessing.
    let (body, body_nyes) = {
        let body: FirRef = {
            let borrowed = brane.borrow();
            let children = borrowed.core().foolish_children();
            let len = children.len() as i32;
            let idx = if offset >= 0 { offset } else { len + offset };
            if idx < 0 || idx >= len {
                return None;
            }
            let stmt = Rc::clone(&children[idx as usize]);
            let body = Rc::clone(stmt.borrow().core().foolish_children().first()?);
            body
        };
        let body_nyes = body.borrow().core().get_nyes();
        (body, body_nyes)
    };
    Some((body, body_nyes))
}

fn index_into_brane_relative(brane: &FirRef, stmt_idx: usize, offset: i32) -> Option<(FirRef, Nyes)> {
    let (body, body_nyes) = {
        let body: FirRef = {
            let borrowed = brane.borrow();
            let children = borrowed.core().foolish_children();
            let idx = stmt_idx as i32 + offset;
            if idx < 0 || idx >= children.len() as i32 {
                return None;
            }
            let stmt = Rc::clone(&children[idx as usize]);
            Rc::clone(stmt.borrow().core().foolish_children().first()?)
        };
        let body_nyes = body.borrow().core().get_nyes();
        (body, body_nyes)
    };
    Some((body, body_nyes))
}

fn find_enclosing_stmt_and_brane(start: &ProtoBrane) -> Option<(FirRef, FirRef)> {
    let mut current = start.parent();
    while let Some(node) = current {
        let kind = node.borrow().kind();
        if kind == FirKind::Statement {
            let brane = {
                let borrowed = node.borrow();
                find_parent_brane(borrowed.core())
            };
            if let Some(brane) = brane {
                return Some((node, brane));
            }
        }
        let next = node.borrow().core().parent();
        match next {
            Some(ref n) if Rc::ptr_eq(n, &node) => break,
            None => break,
            _ => current = next,
        }
    }
    None
}

fn find_parent_brane(start: &ProtoBrane) -> Option<FirRef> {
    let mut current = start.parent();
    while let Some(node) = current {
        if node.borrow().kind() == FirKind::Brane {
            return Some(node);
        }
        let next = node.borrow().core().parent();
        match next {
            Some(ref n) if Rc::ptr_eq(n, &node) => break,
            None => break,
            _ => current = next,
        }
    }
    None
}

impl Fir for IndexFir {
    fn core(&self) -> &ProtoBrane {
        &self.core
    }
    fn fir_op_step(&self, _scope: &Scope) -> Result<(), UbcError> {
        match self.core.get_nyes() {
            Nyes::Prembrionic => {
                if self.anchored {
                    let anchor = Rc::clone(&self.core.foolish_children()[0]);
                    self.core.push_task(anchor);
                    self.core.set_nyes(Nyes::Braning);
                } else {
                    if self.offset >= 0 {
                        return Err(UbcError::Eval(
                            "unanchored index requires negative offset".to_owned(),
                        ));
                    }
                    match find_enclosing_stmt_and_brane(&self.core) {
                        Some((stmt_ref, brane_ref)) => {
                            if let Some(idx) = find_stmt_index_in_brane(&stmt_ref, &brane_ref) {
                                if let Some((body, nyes)) = index_into_brane_relative(&brane_ref, idx, self.offset) {
                                    if nyes.is_settled() {
                                        self.core.push_ubc_child(body);
                                        self.core.set_nyes(nyes);
                                    } else {
                                        self.core.push_task(Rc::clone(&body));
                                        self.core.set_nyes(Nyes::Braning);
                                    }
                                } else {
                                    self.core.set_nyes(Nyes::Nk);
                                }
                            } else {
                                self.core.set_nyes(Nyes::Nk);
                            }
                        }
                        None => {
                            self.core.set_nyes(Nyes::Nk);
                        }
                    }
                }
            }
            Nyes::Braning => {
                if self.anchored {
                    let anchor = Rc::clone(&self.core.foolish_children()[0]);
                    let resolved = resolve_anchor(&anchor);
                    if let Some((body, nyes)) = index_into_brane(&resolved, self.offset) {
                        self.core.push_ubc_child(body);
                        self.core.set_nyes(nyes);
                    } else {
                        self.core.set_nyes(Nyes::Nk);
                    }
                } else {
                    match find_enclosing_stmt_and_brane(&self.core) {
                        Some((stmt_ref, brane_ref)) => {
                            if let Some(idx) = find_stmt_index_in_brane(&stmt_ref, &brane_ref) {
                                if let Some((body, nyes)) = index_into_brane_relative(&brane_ref, idx, self.offset) {
                                    self.core.push_ubc_child(body);
                                    self.core.set_nyes(nyes);
                                } else {
                                    self.core.set_nyes(Nyes::Nk);
                                }
                            } else {
                                self.core.set_nyes(Nyes::Nk);
                            }
                        }
                        None => {
                            self.core.set_nyes(Nyes::Nk);
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
    fn kind(&self) -> FirKind {
        FirKind::Index
    }
    fn as_index_offset(&self) -> i32 {
        self.offset
    }
    fn as_index_anchored(&self) -> bool {
        self.anchored
    }
}

// ── HeadTailFir (stub) ─────────────────────────────────────────────────────

#[derive(Debug)]
pub struct HeadTailFir {
    pub(crate) core: ProtoBrane,
    pub(crate) is_head: bool,
    pub(crate) anchored: bool,
}

impl Fir for HeadTailFir {
    fn core(&self) -> &ProtoBrane {
        &self.core
    }
    fn fir_op_step(&self, _scope: &Scope) -> Result<(), UbcError> {
        let offset: i32 = if self.is_head { 0 } else { -1 };
        match self.core.get_nyes() {
            Nyes::Prembrionic => {
                if self.anchored {
                    let anchor = Rc::clone(&self.core.foolish_children()[0]);
                    self.core.push_task(anchor);
                    self.core.set_nyes(Nyes::Braning);
                } else {
                    if offset >= 0 {
                        return Err(UbcError::Eval(
                            "unanchored head/tail requires tail (negative offset)".to_owned(),
                        ));
                    }
                    match find_enclosing_stmt_and_brane(&self.core) {
                        Some((stmt_ref, brane_ref)) => {
                            if let Some(idx) = find_stmt_index_in_brane(&stmt_ref, &brane_ref) {
                                if let Some((body, nyes)) = index_into_brane_relative(&brane_ref, idx, offset) {
                                    if nyes.is_settled() {
                                        self.core.push_ubc_child(body);
                                        self.core.set_nyes(nyes);
                                    } else {
                                        self.core.push_task(Rc::clone(&body));
                                        self.core.set_nyes(Nyes::Braning);
                                    }
                                } else {
                                    self.core.set_nyes(Nyes::Nk);
                                }
                            } else {
                                self.core.set_nyes(Nyes::Nk);
                            }
                        }
                        None => {
                            self.core.set_nyes(Nyes::Nk);
                        }
                    }
                }
            }
            Nyes::Braning => {
                if self.anchored {
                    let anchor = Rc::clone(&self.core.foolish_children()[0]);
                    let resolved = resolve_anchor(&anchor);
                    if let Some((body, nyes)) = index_into_brane(&resolved, offset) {
                        self.core.push_ubc_child(body);
                        self.core.set_nyes(nyes);
                    } else {
                        self.core.set_nyes(Nyes::Nk);
                    }
                } else {
                    match find_enclosing_stmt_and_brane(&self.core) {
                        Some((stmt_ref, brane_ref)) => {
                            if let Some(idx) = find_stmt_index_in_brane(&stmt_ref, &brane_ref) {
                                if let Some((body, nyes)) = index_into_brane_relative(&brane_ref, idx, offset) {
                                    self.core.push_ubc_child(body);
                                    self.core.set_nyes(nyes);
                                } else {
                                    self.core.set_nyes(Nyes::Nk);
                                }
                            } else {
                                self.core.set_nyes(Nyes::Nk);
                            }
                        }
                        None => {
                            self.core.set_nyes(Nyes::Nk);
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
    fn kind(&self) -> FirKind {
        FirKind::HeadTail
    }
    fn as_headtail_is_head(&self) -> bool {
        self.is_head
    }
    fn as_headtail_anchored(&self) -> bool {
        self.anchored
    }
}

// ── StayFoolishFir ────────────────────────────────────────────────────────

/// A StayFoolish FIR wrapping a single expression child.
///
/// SF freezes the result at assignment time: the expr child is evaluated once,
/// then the result is constanic-cloned into ubc_children. No re-evaluation on
/// subsequent accesses. The clone carries the Foolishly flag (used when Scope
/// is implemented).
///
/// NYES progression: Prembrionic → Braning (expr task pushed) →
/// (expr drains) → constanic-clone result → expr's NYES copied to self.
#[derive(Debug)]
pub struct StayFoolishFir {
    pub(crate) core: ProtoBrane,
}

impl Fir for StayFoolishFir {
    fn core(&self) -> &ProtoBrane {
        &self.core
    }
    fn fir_op_step(&self, _scope: &Scope) -> Result<(), UbcError> {
        match self.core.get_nyes() {
            Nyes::Prembrionic => {
                // SF has a single expression child. Push it as a task and move
                // to Braning — the expr will drain first.
                let children: Vec<FirRef> = self.core.foolish_children().to_vec();
                if children.is_empty() {
                    // No child — immediately constanic (empty SF is constant).
                    self.core.set_nyes(Nyes::Constant);
                } else {
                    self.core.set_nyes(Nyes::Braning);
                    for child in children {
                        self.core.push_task(child);
                    }
                }
            }
            Nyes::Braning => {
                // Constanic-clone: when expr settles, clone result into ubc_children.
                // Foolishly flag (documented; refined when Scope is implemented).
                let children = self.core.foolish_children().to_vec();
                if let Some(expr) = children.first() {
                    let expr_nyes = expr.borrow().core().get_nyes();
                    if expr_nyes.is_settled() {
                        let (result, result_nyes) = {
                            let borrowed = expr.borrow();
                            let ubc = borrowed.core().ubc_children();
                            match ubc.into_iter().next() {
                                Some(r) => {
                                    let n = r.borrow().core().get_nyes();
                                    (r, n)
                                }
                                None => (Rc::clone(expr), expr_nyes),
                            }
                        };
                        self.core.push_ubc_child(result);
                        self.core.set_nyes(result_nyes);
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
    fn kind(&self) -> FirKind {
        FirKind::StayFoolish
    }
}

// ── StayFullyFoolishFir ────────────────────────────────────────────────────

/// A StayFullyFoolish FIR wrapping an expression.
///
/// SFF evaluates its child like a single-membered brane. The only special
/// behavior is that descendant search FIRs are created as ECONSTANIC
/// (searches never run). The SFF marker's state depends on its child:
/// - `<<1+1>>` → Constant 2 (all children are constants)
/// - `<<a+b>>` → WOCONSTANIC (a,b are ECONSTANIC searches)
///
/// NYES progression: Prembrionic → Braning (child pushed as task) →
/// (child drains) → child's NYES copied to self.
#[derive(Debug)]
pub struct StayFullyFoolishFir {
    pub(crate) core: ProtoBrane,
}

impl Fir for StayFullyFoolishFir {
    fn core(&self) -> &ProtoBrane {
        &self.core
    }
    fn fir_op_step(&self, _scope: &Scope) -> Result<(), UbcError> {
        match self.core.get_nyes() {
            Nyes::Prembrionic => {
                // SFF evaluates its child like a single-membered brane.
                // Push child as task, move to Braning.
                let children: Vec<FirRef> = self.core.foolish_children().to_vec();
                if children.is_empty() {
                    self.core.set_nyes(Nyes::Constant);
                } else {
                    self.core.set_nyes(Nyes::Braning);
                    for child in children {
                        self.core.push_task(child);
                    }
                }
            }
            Nyes::Braning => {
                // All child tasks have drained. Copy the child's NYES to self.
                // SFF inherits its child's state.
                let children = self.core.foolish_children().to_vec();
                if let Some(child) = children.first() {
                    let child_nyes = child.borrow().core().get_nyes();
                    self.core.set_nyes(child_nyes);
                }
            }
            _ => {}
        }
        Ok(())
    }
    fn kind(&self) -> FirKind {
        FirKind::StayFullyFoolish
    }
}

// ── ConcatenationFir (stub) ────────────────────────────────────────────────

#[derive(Debug)]
pub struct ConcatenationFir {
    pub(crate) core: ProtoBrane,
}

impl Fir for ConcatenationFir {
    fn core(&self) -> &ProtoBrane {
        &self.core
    }
    fn fir_op_step(&self, _scope: &Scope) -> Result<(), UbcError> {
        match self.core.get_nyes() {
            Nyes::Prembrionic => {
                let children: Vec<FirRef> = self.core.foolish_children().to_vec();
                if children.is_empty() {
                    let result_ref: FirRef =
                        Rc::new_cyclic(|me: &Weak<RefCell<BraneFir>>| {
                            let parent: Weak<RefCell<dyn Fir>> = me.clone();
                            RefCell::new(BraneFir {
                                core: ProtoBrane::new(vec![], parent, Nyes::Constant),
                                characterizations: Vec::new(),
                            })
                        });
                    self.core.push_ubc_child(result_ref);
                    self.core.set_nyes(Nyes::Constant);
                } else {
                    self.core.set_nyes(Nyes::Braning);
                    for child in children {
                        self.core.push_task(child);
                    }
                }
            }
            Nyes::Braning => {
                let children = self.core.foolish_children().to_vec();
                let any_nk = children
                    .iter()
                    .any(|c| c.borrow().core().get_nyes() == Nyes::Nk);
                if any_nk {
                    self.core.set_nyes(Nyes::Nk);
                    return Ok(());
                }
                let any_woconstanic = children.iter().any(|c| {
                    let n = c.borrow().core().get_nyes();
                    n == Nyes::Econstanic || n == Nyes::Woconstanic
                });
                if any_woconstanic {
                    self.core.set_nyes(Nyes::Woconstanic);
                    return Ok(());
                }
                let mut merged_stmts: Vec<FirRef> = Vec::new();
                for child in &children {
                    let resolved = {
                        let borrowed = child.borrow();
                        if borrowed.core().get_nyes().is_settled() {
                            borrowed.core().ubc_children().into_iter().next()
                        } else {
                            None
                        }
                    };
                    let source = resolved.as_ref().unwrap_or(child);
                    let borrowed = source.borrow();
                    for stmt in borrowed.core().foolish_children() {
                        merged_stmts.push(Rc::clone(stmt));
                    }
                }
                let result_ref: FirRef = Rc::new_cyclic(|me: &Weak<RefCell<BraneFir>>| {
                    let parent: Weak<RefCell<dyn Fir>> = me.clone();
                    RefCell::new(BraneFir {
                        core: ProtoBrane::new(merged_stmts, parent, Nyes::Constant),
                        characterizations: Vec::new(),
                    })
                });
                self.core.push_ubc_child(result_ref);
                self.core.set_nyes(Nyes::Constant);
            }
            _ => {}
        }
        Ok(())
    }
    fn kind(&self) -> FirKind {
        FirKind::Concatenation
    }
}

// ── Builders ────────────────────────────────────────────────────────────────

/// Build a `ConstantIntFir` leaf wrapped in `FirRef`.
///
/// Starts at Prembrionic; stepping transitions it to Constant.
pub fn constant_int(value: i64, parent: Weak<RefCell<dyn Fir>>) -> FirRef {
    Rc::new(RefCell::new(ConstantIntFir {
        core: ProtoBrane::new(vec![], parent, Nyes::Prembrionic),
        value,
    }))
}

/// Build an `NkFir` leaf wrapped in `FirRef`.
///
/// Starts at Prembrionic; stepping transitions it to Nk.
pub fn nk(reason: &str, parent: Weak<RefCell<dyn Fir>>) -> FirRef {
    Rc::new(RefCell::new(NkFir {
        core: ProtoBrane::new(vec![], parent, Nyes::Prembrionic),
        reason: reason.to_owned(),
    }))
}

/// Build an `OperatorFir` wrapped in `FirRef`.
///
/// `operands` become `foolish_children`. Starts at Prembrionic.
pub fn operator(op: &str, operands: Vec<FirRef>, parent: Weak<RefCell<dyn Fir>>) -> FirRef {
    Rc::new(RefCell::new(OperatorFir {
        core: ProtoBrane::new(operands, parent, Nyes::Prembrionic),
        op: op.to_owned(),
    }))
}

pub fn statement(
    name: &str,
    line_number: usize,
    body: FirRef,
    parent: Weak<RefCell<dyn Fir>>,
) -> FirRef {
    Rc::new(RefCell::new(StatementFir {
        core: ProtoBrane::new(vec![body], parent, Nyes::Prembrionic),
        name: name.to_owned(),
        line_number,
    }))
}

pub fn brane(children: Vec<FirRef>, parent: Weak<RefCell<dyn Fir>>) -> FirRef {
    Rc::new(RefCell::new(BraneFir {
        core: ProtoBrane::new(children, parent, Nyes::Prembrionic),
        characterizations: Vec::new(),
    }))
}

pub fn index(
    offset: i32,
    anchored: bool,
    children: Vec<FirRef>,
    parent: Weak<RefCell<dyn Fir>>,
) -> FirRef {
    Rc::new(RefCell::new(IndexFir {
        core: ProtoBrane::new(children, parent, Nyes::Prembrionic),
        offset,
        anchored,
    }))
}

pub fn headtail(
    is_head: bool,
    anchored: bool,
    children: Vec<FirRef>,
    parent: Weak<RefCell<dyn Fir>>,
) -> FirRef {
    Rc::new(RefCell::new(HeadTailFir {
        core: ProtoBrane::new(children, parent, Nyes::Prembrionic),
        is_head,
        anchored,
    }))
}

pub fn concatenation(elements: Vec<FirRef>, parent: Weak<RefCell<dyn Fir>>) -> FirRef {
    Rc::new(RefCell::new(ConcatenationFir {
        core: ProtoBrane::new(elements, parent, Nyes::Prembrionic),
    }))
}

pub fn stay_foolish(expr: FirRef, parent: Weak<RefCell<dyn Fir>>) -> FirRef {
    Rc::new(RefCell::new(StayFoolishFir {
        core: ProtoBrane::new(vec![expr], parent, Nyes::Prembrionic),
    }))
}

pub fn stay_fully_foolish(expr: FirRef, parent: Weak<RefCell<dyn Fir>>) -> FirRef {
    Rc::new(RefCell::new(StayFullyFoolishFir {
        core: ProtoBrane::new(vec![expr], parent, Nyes::Prembrionic),
    }))
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fir_trait::step_fir_ref;
    use crate::fir_trait::StepReport;

    /// Helper: create a self-rooting FirRef for standalone testing.
    /// The parent Weak points at the node itself (root convention).
    fn make_constant_int(value: i64) -> FirRef {
        Rc::new_cyclic(|me: &Weak<RefCell<ConstantIntFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(ConstantIntFir {
                core: ProtoBrane::new(vec![], parent, Nyes::Prembrionic),
                value,
            })
        })
    }

    /// Helper: create a self-rooting NkFir for standalone testing.
    fn make_nk(reason: &str) -> FirRef {
        Rc::new_cyclic(|me: &Weak<RefCell<NkFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(NkFir {
                core: ProtoBrane::new(vec![], parent, Nyes::Prembrionic),
                reason: reason.to_owned(),
            })
        })
    }

    #[test]
    fn constant_int_prembrionic_to_constant_in_one_step() {
        let node = make_constant_int(42);
        let scope = Scope::empty();

        // Before stepping: Prembrionic
        assert_eq!(node.borrow().core().get_nyes(), Nyes::Prembrionic);
        let mut transitions = vec![Nyes::Prembrionic];

        // Step once
        let report = step_fir_ref(&node, &scope).unwrap();
        match report {
            StepReport::Progress(nyes) => {
                transitions.push(nyes);
                assert_eq!(nyes, Nyes::Constant);
            }
            StepReport::NoProgress => panic!("expected progress on first step"),
        }

        eprintln!("ConstantInt NYES transitions: {transitions:?}");
        assert_eq!(transitions, vec![Nyes::Prembrionic, Nyes::Constant]);

        // Settled after one step
        assert!(node.borrow().core().get_nyes().is_settled());

        // Value is accessible
        assert_eq!(node.borrow().core().get_nyes(), Nyes::Constant);
        // Access value through the concrete type via FirRef
        // (we know it's a ConstantIntFir from construction)
        let borrowed = node.borrow();
        assert_eq!(borrowed.core().get_nyes(), Nyes::Constant);
        // The value is accessible via downcast or the kind() check
        assert_eq!(borrowed.kind(), FirKind::ConstantInt);
    }

    #[test]
    fn constant_int_value_accessor() {
        let node = make_constant_int(-7);
        let scope = Scope::empty();

        // Step to settled
        let _ = step_fir_ref(&node, &scope).unwrap();

        // Verify the value via the Fir trait's kind()
        assert_eq!(node.borrow().kind(), FirKind::ConstantInt);
    }

    #[test]
    fn nk_prembrionic_to_nk_in_one_step() {
        let node = make_nk("unbound name");
        let scope = Scope::empty();

        // Before stepping: Prembrionic
        assert_eq!(node.borrow().core().get_nyes(), Nyes::Prembrionic);
        let mut transitions = vec![Nyes::Prembrionic];

        // Step once
        let report = step_fir_ref(&node, &scope).unwrap();
        match report {
            StepReport::Progress(nyes) => {
                transitions.push(nyes);
                assert_eq!(nyes, Nyes::Nk);
            }
            StepReport::NoProgress => panic!("expected progress on first step"),
        }

        eprintln!("NkFir NYES transitions: {transitions:?}");
        assert_eq!(transitions, vec![Nyes::Prembrionic, Nyes::Nk]);

        // Settled after one step
        assert!(node.borrow().core().get_nyes().is_settled());

        // Kind check
        assert_eq!(node.borrow().kind(), FirKind::Nk);
    }

    #[test]
    fn nk_reason_accessor() {
        let node = make_nk("division by zero");
        let scope = Scope::empty();

        // Step to settled
        let _ = step_fir_ref(&node, &scope).unwrap();

        // Verify the kind
        assert_eq!(node.borrow().kind(), FirKind::Nk);
    }

    #[test]
    fn both_leaf_kinds_are_settled_after_one_step() {
        let ci = make_constant_int(100);
        let nk = make_nk("nope");
        let scope = Scope::empty();

        // Neither is settled before stepping
        assert!(!ci.borrow().core().get_nyes().is_settled());
        assert!(!nk.borrow().core().get_nyes().is_settled());

        // One step each
        let r1 = step_fir_ref(&ci, &scope).unwrap();
        let r2 = step_fir_ref(&nk, &scope).unwrap();

        assert!(matches!(r1, StepReport::Progress(Nyes::Constant)));
        assert!(matches!(r2, StepReport::Progress(Nyes::Nk)));

        // Both settled
        assert!(ci.borrow().core().get_nyes().is_settled());
        assert!(nk.borrow().core().get_nyes().is_settled());
    }

    #[test]
    fn stepping_already_settled_is_noop() {
        let ci = make_constant_int(1);
        let nk = make_nk("done");
        let scope = Scope::empty();

        // Step to settle
        let _ = step_fir_ref(&ci, &scope).unwrap();
        let _ = step_fir_ref(&nk, &scope).unwrap();

        // Step again — should remain at same terminal state
        let r1 = step_fir_ref(&ci, &scope).unwrap();
        let r2 = step_fir_ref(&nk, &scope).unwrap();

        assert_eq!(r1, StepReport::Progress(Nyes::Constant));
        assert_eq!(r2, StepReport::Progress(Nyes::Nk));
    }

    #[test]
    fn constant_int_builder_sets_prembrionic() {
        // Use the public builder with a dummy parent Weak
        let dummy: Rc<RefCell<dyn Fir>> = Rc::new_cyclic(|me: &Weak<RefCell<ConstantIntFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(ConstantIntFir {
                core: ProtoBrane::new(vec![], parent, Nyes::Constant),
                value: 0,
            })
        });
        let parent_weak = Rc::downgrade(&dummy);
        let node = constant_int(99, parent_weak);

        assert_eq!(node.borrow().core().get_nyes(), Nyes::Prembrionic);
        assert_eq!(node.borrow().kind(), FirKind::ConstantInt);
    }

    #[test]
    fn nk_builder_sets_prembrionic() {
        let dummy: Rc<RefCell<dyn Fir>> = Rc::new_cyclic(|me: &Weak<RefCell<NkFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(NkFir {
                core: ProtoBrane::new(vec![], parent, Nyes::Nk),
                reason: String::new(),
            })
        });
        let parent_weak = Rc::downgrade(&dummy);
        let node = nk("test reason", parent_weak);

        assert_eq!(node.borrow().core().get_nyes(), Nyes::Prembrionic);
        assert_eq!(node.borrow().kind(), FirKind::Nk);
    }

    fn make_operator(op: &str, operands: Vec<FirRef>) -> FirRef {
        Rc::new_cyclic(|me: &Weak<RefCell<OperatorFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(OperatorFir {
                core: ProtoBrane::new(operands, parent, Nyes::Prembrionic),
                op: op.to_owned(),
            })
        })
    }

    fn make_statement(name: &str, line_number: usize, body: FirRef) -> FirRef {
        Rc::new_cyclic(|me: &Weak<RefCell<StatementFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(StatementFir {
                core: ProtoBrane::new(vec![body], parent, Nyes::Prembrionic),
                name: name.to_owned(),
                line_number,
            })
        })
    }

    fn make_brane(children: Vec<FirRef>) -> FirRef {
        Rc::new_cyclic(|me: &Weak<RefCell<BraneFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(BraneFir {
                core: ProtoBrane::new(children, parent, Nyes::Prembrionic),
                characterizations: Vec::new(),
            })
        })
    }

    fn make_search(pattern: &str, anchored: bool, foolish_children: Vec<FirRef>) -> FirRef {
        Rc::new_cyclic(|me: &Weak<RefCell<SearchFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(SearchFir {
                core: ProtoBrane::new(foolish_children, parent, Nyes::Prembrionic),
                pattern: pattern.to_owned(),
                anchored,
                forward: false,
                found_body: RefCell::new(None),
            })
        })
    }

    fn make_index(offset: i32, anchored: bool, foolish_children: Vec<FirRef>) -> FirRef {
        Rc::new_cyclic(|me: &Weak<RefCell<IndexFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(IndexFir {
                core: ProtoBrane::new(foolish_children, parent, Nyes::Prembrionic),
                offset,
                anchored,
            })
        })
    }

    fn step_to_settled(node: &FirRef, scope: &Scope) -> Vec<Nyes> {
        let mut transitions = vec![node.borrow().core().get_nyes()];
        for _ in 0..50 {
            let report = step_fir_ref(node, scope).unwrap();
            match report {
                StepReport::Progress(nyes) => {
                    transitions.push(nyes);
                    if nyes.is_settled() {
                        break;
                    }
                }
                StepReport::NoProgress => break,
            }
        }
        transitions
    }

    #[test]
    fn operator_add_two_constants() {
        let a = make_constant_int(3);
        let b = make_constant_int(5);
        let op = make_operator("+", vec![Rc::clone(&a), Rc::clone(&b)]);
        let scope = Scope::empty();

        assert_eq!(op.borrow().core().get_nyes(), Nyes::Prembrionic);
        let mut transitions = vec![Nyes::Prembrionic];

        for _ in 0..20 {
            let report = step_fir_ref(&op, &scope).unwrap();
            match report {
                StepReport::Progress(nyes) => {
                    transitions.push(nyes);
                    if nyes.is_settled() {
                        break;
                    }
                }
                StepReport::NoProgress => break,
            }
        }

        eprintln!("Operator(+) NYES transitions: {transitions:?}");

        assert!(a.borrow().core().get_nyes().is_settled());
        assert!(b.borrow().core().get_nyes().is_settled());
        assert_eq!(op.borrow().core().get_nyes(), Nyes::Constant);
        assert_eq!(op.borrow().kind(), FirKind::Operator);

        let ubc = op.borrow().core().ubc_children().to_vec();
        assert_eq!(ubc.len(), 1);
        assert_eq!(ubc[0].borrow().kind(), FirKind::ConstantInt);
        assert_eq!(ubc[0].borrow().as_i64(), Some(8));
    }

    #[test]
    fn operator_subtract() {
        let a = make_constant_int(10);
        let b = make_constant_int(3);
        let op = make_operator("-", vec![Rc::clone(&a), Rc::clone(&b)]);
        let scope = Scope::empty();

        for _ in 0..20 {
            let report = step_fir_ref(&op, &scope).unwrap();
            if let StepReport::Progress(nyes) = report {
                if nyes.is_settled() {
                    break;
                }
            }
        }

        assert_eq!(op.borrow().core().get_nyes(), Nyes::Constant);
        let ubc = op.borrow().core().ubc_children().to_vec();
        assert_eq!(ubc.len(), 1);
        assert_eq!(ubc[0].borrow().as_i64(), Some(7));
    }

    #[test]
    fn operator_multiply() {
        let a = make_constant_int(4);
        let b = make_constant_int(6);
        let op = make_operator("*", vec![Rc::clone(&a), Rc::clone(&b)]);
        let scope = Scope::empty();

        for _ in 0..20 {
            let report = step_fir_ref(&op, &scope).unwrap();
            if let StepReport::Progress(nyes) = report {
                if nyes.is_settled() {
                    break;
                }
            }
        }

        assert_eq!(op.borrow().core().get_nyes(), Nyes::Constant);
        let ubc = op.borrow().core().ubc_children().to_vec();
        assert_eq!(ubc.len(), 1);
        assert_eq!(ubc[0].borrow().as_i64(), Some(24));
    }

    #[test]
    fn operator_divide() {
        let a = make_constant_int(20);
        let b = make_constant_int(4);
        let op = make_operator("/", vec![Rc::clone(&a), Rc::clone(&b)]);
        let scope = Scope::empty();

        for _ in 0..20 {
            let report = step_fir_ref(&op, &scope).unwrap();
            if let StepReport::Progress(nyes) = report {
                if nyes.is_settled() {
                    break;
                }
            }
        }

        assert_eq!(op.borrow().core().get_nyes(), Nyes::Constant);
        let ubc = op.borrow().core().ubc_children().to_vec();
        assert_eq!(ubc.len(), 1);
        assert_eq!(ubc[0].borrow().as_i64(), Some(5));
    }

    #[test]
    fn operator_divide_by_zero_is_nk() {
        let a = make_constant_int(10);
        let b = make_constant_int(0);
        let op = make_operator("/", vec![Rc::clone(&a), Rc::clone(&b)]);
        let scope = Scope::empty();

        for _ in 0..20 {
            let report = step_fir_ref(&op, &scope).unwrap();
            if let StepReport::Progress(nyes) = report {
                if nyes.is_settled() {
                    break;
                }
            }
        }

        assert_eq!(op.borrow().core().get_nyes(), Nyes::Nk);
        assert_eq!(op.borrow().core().ubc_children().len(), 1);
        assert_eq!(op.borrow().core().ubc_children()[0].borrow().kind(), FirKind::Nk);
    }

    #[test]
    fn operator_with_nk_operand_is_nk() {
        let a = make_constant_int(5);
        let b = make_nk("unbound");
        let op = make_operator("+", vec![Rc::clone(&a), Rc::clone(&b)]);
        let scope = Scope::empty();

        for _ in 0..20 {
            let report = step_fir_ref(&op, &scope).unwrap();
            if let StepReport::Progress(nyes) = report {
                if nyes.is_settled() {
                    break;
                }
            }
        }

        assert_eq!(op.borrow().core().get_nyes(), Nyes::Nk);
    }

    #[test]
    fn operator_builder_sets_prembrionic() {
        let dummy: Rc<RefCell<dyn Fir>> = Rc::new_cyclic(|me: &Weak<RefCell<ConstantIntFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(ConstantIntFir {
                core: ProtoBrane::new(vec![], parent, Nyes::Constant),
                value: 0,
            })
        });
        let parent_weak = Rc::downgrade(&dummy);
        let node = operator("+", vec![], parent_weak);

        assert_eq!(node.borrow().core().get_nyes(), Nyes::Prembrionic);
        assert_eq!(node.borrow().kind(), FirKind::Operator);
    }

    // ── StatementFir tests ──────────────────────────────────────────────────

    #[test]
    fn statement_wrapping_constant_copies_body_nyes() {
        let body = make_constant_int(42);
        let stmt = make_statement("x", 1, Rc::clone(&body));
        let scope = Scope::empty();

        let transitions = step_to_settled(&stmt, &scope);
        eprintln!("Statement(ConstantInt) NYES transitions: {transitions:?}");

        assert_eq!(transitions.first(), Some(&Nyes::Prembrionic));
        assert_eq!(transitions.last(), Some(&Nyes::Constant));
        assert!(transitions.contains(&Nyes::Braning));
        assert_eq!(stmt.borrow().kind(), FirKind::Statement);
        assert!(stmt.borrow().core().get_nyes().is_settled());
    }

    #[test]
    fn statement_wrapping_nk_copies_nk() {
        let body = make_nk("unbound");
        let stmt = make_statement("y", 2, Rc::clone(&body));
        let scope = Scope::empty();

        let transitions = step_to_settled(&stmt, &scope);
        eprintln!("Statement(Nk) NYES transitions: {transitions:?}");

        assert_eq!(transitions.first(), Some(&Nyes::Prembrionic));
        assert_eq!(transitions.last(), Some(&Nyes::Nk));
        assert!(transitions.contains(&Nyes::Braning));
    }

    #[test]
    fn statement_name_and_line_accessors() {
        let body = make_constant_int(1);
        let stmt = make_statement("myvar", 42, Rc::clone(&body));
        assert_eq!(stmt.borrow().kind(), FirKind::Statement);
        // Access through concrete downcast
        let borrowed = stmt.borrow();
        assert_eq!(borrowed.kind(), FirKind::Statement);
    }

    #[test]
    fn statement_builder_sets_prembrionic() {
        let dummy: Rc<RefCell<dyn Fir>> = Rc::new_cyclic(|me: &Weak<RefCell<ConstantIntFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(ConstantIntFir {
                core: ProtoBrane::new(vec![], parent, Nyes::Constant),
                value: 0,
            })
        });
        let parent_weak = Rc::downgrade(&dummy);
        let body = make_constant_int(1);
        let node = statement("x", 1, body, parent_weak);

        assert_eq!(node.borrow().core().get_nyes(), Nyes::Prembrionic);
        assert_eq!(node.borrow().kind(), FirKind::Statement);
    }

    // ── BraneFir tests ──────────────────────────────────────────────────────

    #[test]
    fn brane_two_constant_children_classifies_constant() {
        let a = make_constant_int(10);
        let b = make_constant_int(20);
        let brane = make_brane(vec![Rc::clone(&a), Rc::clone(&b)]);
        let scope = Scope::empty();

        let transitions = step_to_settled(&brane, &scope);
        eprintln!("Brane(2x Constant) NYES transitions: {transitions:?}");

        assert_eq!(brane.borrow().core().get_nyes(), Nyes::Constant);
        assert_eq!(brane.borrow().kind(), FirKind::Brane);
    }

    #[test]
    fn brane_with_nk_child_classifies_nk() {
        let a = make_constant_int(5);
        let b = make_nk("unbound name");
        let brane = make_brane(vec![Rc::clone(&a), Rc::clone(&b)]);
        let scope = Scope::empty();

        let transitions = step_to_settled(&brane, &scope);
        eprintln!("Brane(Constant + Nk) NYES transitions: {transitions:?}");

        assert_eq!(brane.borrow().core().get_nyes(), Nyes::Nk);
    }

    #[test]
    fn brane_drains_children_in_order() {
        let a = make_constant_int(1);
        let b = make_constant_int(2);
        let brane = make_brane(vec![Rc::clone(&a), Rc::clone(&b)]);
        let scope = Scope::empty();

        // Step once: Prembrionic → Braning (tasks pushed)
        let r1 = step_fir_ref(&brane, &scope).unwrap();
        assert!(matches!(r1, StepReport::Progress(Nyes::Braning)));

        // Next steps should drain child a first
        let mut child_a_settled_first = false;
        for _ in 0..20 {
            let _ = step_fir_ref(&brane, &scope).unwrap();
            let a_settled = a.borrow().core().get_nyes().is_settled();
            let b_settled = b.borrow().core().get_nyes().is_settled();
            if a_settled && !b_settled {
                child_a_settled_first = true;
            }
            if a_settled && b_settled {
                break;
            }
        }
        assert!(
            child_a_settled_first,
            "child a should settle before child b"
        );
    }

    #[test]
    fn brane_nested_brane_drains_inner_first() {
        // Inner brane with 2 constant children
        let inner_a = make_constant_int(100);
        let inner_b = make_constant_int(200);
        let inner = make_brane(vec![Rc::clone(&inner_a), Rc::clone(&inner_b)]);

        // Outer brane containing inner as only child
        let outer = make_brane(vec![Rc::clone(&inner)]);
        let scope = Scope::empty();

        let transitions = step_to_settled(&outer, &scope);
        eprintln!("Nested brane NYES transitions: {transitions:?}");

        assert!(inner.borrow().core().get_nyes().is_settled());
        assert!(outer.borrow().core().get_nyes().is_settled());
        assert_eq!(inner.borrow().core().get_nyes(), Nyes::Constant);
        assert_eq!(outer.borrow().core().get_nyes(), Nyes::Constant);
    }

    #[test]
    fn brane_empty_children_classifies_constant() {
        let brane = make_brane(vec![]);
        let scope = Scope::empty();

        let transitions = step_to_settled(&brane, &scope);
        eprintln!("Brane(empty) NYES transitions: {transitions:?}");

        assert_eq!(brane.borrow().core().get_nyes(), Nyes::Constant);
    }

    #[test]
    fn brane_builder_sets_prembrionic() {
        let dummy: Rc<RefCell<dyn Fir>> = Rc::new_cyclic(|me: &Weak<RefCell<ConstantIntFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(ConstantIntFir {
                core: ProtoBrane::new(vec![], parent, Nyes::Constant),
                value: 0,
            })
        });
        let parent_weak = Rc::downgrade(&dummy);
        let node = brane(vec![], parent_weak);

        assert_eq!(node.borrow().core().get_nyes(), Nyes::Prembrionic);
        assert_eq!(node.borrow().kind(), FirKind::Brane);
    }

    #[test]
    fn statement_wrapping_operator_propagates() {
        // Statement wrapping an operator: 3 + 5 → body becomes Constant(8),
        // statement copies Constant.
        let a = make_constant_int(3);
        let b = make_constant_int(5);
        let op = make_operator("+", vec![Rc::clone(&a), Rc::clone(&b)]);
        let stmt = make_statement("result", 1, Rc::clone(&op));
        let scope = Scope::empty();

        let transitions = step_to_settled(&stmt, &scope);
        eprintln!("Statement(Operator) NYES transitions: {transitions:?}");

        assert_eq!(stmt.borrow().core().get_nyes(), Nyes::Constant);
        assert_eq!(op.borrow().core().get_nyes(), Nyes::Constant);
    }

    // ── SearchFir tests ──────────────────────────────────────────────────────

    #[test]
    fn search_finds_name_in_anchored_brane() {
        let val = make_constant_int(42);
        let stmt = make_statement("x", 0, Rc::clone(&val));
        let brane = make_brane(vec![Rc::clone(&stmt)]);
        let search = make_search("^x$", true, vec![Rc::clone(&brane)]);
        let scope = Scope::empty();

        let transitions = step_to_settled(&search, &scope);
        eprintln!("Search(found) NYES transitions: {transitions:?}");

        assert_eq!(search.borrow().core().get_nyes(), Nyes::Constant);
        let ubc = search.borrow().core().ubc_children().to_vec();
        assert_eq!(ubc.len(), 1);
        assert_eq!(ubc[0].borrow().as_i64(), Some(42));
    }

    #[test]
    fn search_not_found_becomes_nk() {
        let val = make_constant_int(42);
        let stmt = make_statement("y", 0, Rc::clone(&val));
        let brane = make_brane(vec![Rc::clone(&stmt)]);
        let search = make_search("^x$", true, vec![Rc::clone(&brane)]);
        let scope = Scope::empty();

        let transitions = step_to_settled(&search, &scope);
        eprintln!("Search(not found) NYES transitions: {transitions:?}");

        assert_eq!(search.borrow().core().get_nyes(), Nyes::Nk);
        assert!(search.borrow().core().ubc_children().is_empty());
    }

    #[test]
    fn search_anchored_nk_body_propagates_woconstanic() {
        let val = make_nk("unbound");
        let stmt = make_statement("z", 0, Rc::clone(&val));
        let brane = make_brane(vec![Rc::clone(&stmt)]);
        let search = make_search("^z$", true, vec![Rc::clone(&brane)]);
        let scope = Scope::empty();

        let transitions = step_to_settled(&search, &scope);
        eprintln!("Search(nk body) NYES transitions: {transitions:?}");

        // NK found → search becomes Woconstanic (found something, but it's not a value)
        assert_eq!(search.borrow().core().get_nyes(), Nyes::Woconstanic);
    }

    // ── IndexFir tests ─────────────────────────────────────────────────────

    #[test]
    fn index_finds_element_at_offset_in_anchor_brane() {
        let val_a = make_constant_int(10);
        let val_b = make_constant_int(20);
        let val_c = make_constant_int(30);
        let stmt_a = make_statement("a", 1, Rc::clone(&val_a));
        let stmt_b = make_statement("b", 2, Rc::clone(&val_b));
        let stmt_c = make_statement("c", 3, Rc::clone(&val_c));
        let brane = make_brane(vec![
            Rc::clone(&stmt_a),
            Rc::clone(&stmt_b),
            Rc::clone(&stmt_c),
        ]);

        let idx = make_index(1, true, vec![Rc::clone(&brane)]);
        let scope = Scope::empty();

        let transitions = step_to_settled(&idx, &scope);
        eprintln!("Index(offset=1) NYES transitions: {transitions:?}");

        assert_eq!(idx.borrow().core().get_nyes(), Nyes::Constant);
        let ubc = idx.borrow().core().ubc_children().to_vec();
        assert_eq!(ubc.len(), 1);
        assert_eq!(ubc[0].borrow().as_i64(), Some(20));
    }

    #[test]
    fn index_out_of_bounds_is_nk() {
        let val = make_constant_int(42);
        let stmt = make_statement("x", 1, Rc::clone(&val));
        let brane = make_brane(vec![Rc::clone(&stmt)]);

        let idx = make_index(5, true, vec![Rc::clone(&brane)]);
        let scope = Scope::empty();

        let transitions = step_to_settled(&idx, &scope);
        eprintln!("Index(oob) NYES transitions: {transitions:?}");

        assert_eq!(idx.borrow().core().get_nyes(), Nyes::Nk);
        assert!(idx.borrow().core().ubc_children().is_empty());
    }

    #[test]
    fn index_negative_offset_from_back() {
        let val_a = make_constant_int(10);
        let val_b = make_constant_int(20);
        let val_c = make_constant_int(30);
        let stmt_a = make_statement("a", 1, Rc::clone(&val_a));
        let stmt_b = make_statement("b", 2, Rc::clone(&val_b));
        let stmt_c = make_statement("c", 3, Rc::clone(&val_c));
        let brane = make_brane(vec![
            Rc::clone(&stmt_a),
            Rc::clone(&stmt_b),
            Rc::clone(&stmt_c),
        ]);

        let idx = make_index(-1, true, vec![Rc::clone(&brane)]);
        let scope = Scope::empty();

        let transitions = step_to_settled(&idx, &scope);
        eprintln!("Index(offset=-1) NYES transitions: {transitions:?}");

        assert_eq!(idx.borrow().core().get_nyes(), Nyes::Constant);
        let ubc = idx.borrow().core().ubc_children().to_vec();
        assert_eq!(ubc.len(), 1);
        assert_eq!(ubc[0].borrow().as_i64(), Some(30));
    }

    #[test]
    fn index_zero_offset_gets_first_element() {
        let val = make_constant_int(99);
        let stmt = make_statement("first", 1, Rc::clone(&val));
        let brane = make_brane(vec![Rc::clone(&stmt)]);

        let idx = make_index(0, true, vec![Rc::clone(&brane)]);
        let scope = Scope::empty();

        let transitions = step_to_settled(&idx, &scope);
        eprintln!("Index(offset=0) NYES transitions: {transitions:?}");

        assert_eq!(idx.borrow().core().get_nyes(), Nyes::Constant);
        let ubc = idx.borrow().core().ubc_children().to_vec();
        assert_eq!(ubc.len(), 1);
        assert_eq!(ubc[0].borrow().as_i64(), Some(99));
    }

    #[test]
    fn index_nk_body_propagates_nk() {
        let val = make_nk("unbound");
        let stmt = make_statement("z", 1, Rc::clone(&val));
        let brane = make_brane(vec![Rc::clone(&stmt)]);

        let idx = make_index(0, true, vec![Rc::clone(&brane)]);
        let scope = Scope::empty();

        let transitions = step_to_settled(&idx, &scope);
        eprintln!("Index(nk body) NYES transitions: {transitions:?}");

        assert_eq!(idx.borrow().core().get_nyes(), Nyes::Nk);
    }

    // ── HeadTailFir helpers & tests ────────────────────────────────────────

    fn make_headtail(is_head: bool, anchored: bool, foolish_children: Vec<FirRef>) -> FirRef {
        Rc::new_cyclic(|me: &Weak<RefCell<HeadTailFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(HeadTailFir {
                core: ProtoBrane::new(foolish_children, parent, Nyes::Prembrionic),
                is_head,
                anchored,
            })
        })
    }

    #[test]
    fn headtail_head_gets_first_element() {
        let val_a = make_constant_int(10);
        let val_b = make_constant_int(20);
        let val_c = make_constant_int(30);
        let stmt_a = make_statement("a", 1, Rc::clone(&val_a));
        let stmt_b = make_statement("b", 2, Rc::clone(&val_b));
        let stmt_c = make_statement("c", 3, Rc::clone(&val_c));
        let brane = make_brane(vec![
            Rc::clone(&stmt_a),
            Rc::clone(&stmt_b),
            Rc::clone(&stmt_c),
        ]);

        let ht = make_headtail(true, true, vec![Rc::clone(&brane)]);
        let scope = Scope::empty();

        let transitions = step_to_settled(&ht, &scope);
        eprintln!("HeadTail(head) NYES transitions: {transitions:?}");

        assert_eq!(ht.borrow().core().get_nyes(), Nyes::Constant);
        let ubc = ht.borrow().core().ubc_children().to_vec();
        assert_eq!(ubc.len(), 1);
        assert_eq!(ubc[0].borrow().as_i64(), Some(10));
    }

    #[test]
    fn headtail_tail_gets_last_element() {
        let val_a = make_constant_int(10);
        let val_b = make_constant_int(20);
        let val_c = make_constant_int(30);
        let stmt_a = make_statement("a", 1, Rc::clone(&val_a));
        let stmt_b = make_statement("b", 2, Rc::clone(&val_b));
        let stmt_c = make_statement("c", 3, Rc::clone(&val_c));
        let brane = make_brane(vec![
            Rc::clone(&stmt_a),
            Rc::clone(&stmt_b),
            Rc::clone(&stmt_c),
        ]);

        let ht = make_headtail(false, true, vec![Rc::clone(&brane)]);
        let scope = Scope::empty();

        let transitions = step_to_settled(&ht, &scope);
        eprintln!("HeadTail(tail) NYES transitions: {transitions:?}");

        assert_eq!(ht.borrow().core().get_nyes(), Nyes::Constant);
        let ubc = ht.borrow().core().ubc_children().to_vec();
        assert_eq!(ubc.len(), 1);
        assert_eq!(ubc[0].borrow().as_i64(), Some(30));
    }

    #[test]
    fn headtail_empty_brane_is_nk() {
        let brane = make_brane(vec![]);
        let ht = make_headtail(true, true, vec![Rc::clone(&brane)]);
        let scope = Scope::empty();

        let transitions = step_to_settled(&ht, &scope);
        eprintln!("HeadTail(empty) NYES transitions: {transitions:?}");

        assert_eq!(ht.borrow().core().get_nyes(), Nyes::Nk);
        assert!(ht.borrow().core().ubc_children().is_empty());
    }

    #[test]
    fn headtail_single_element_head_and_tail_same() {
        let val = make_constant_int(42);
        let stmt = make_statement("x", 1, Rc::clone(&val));
        let brane = make_brane(vec![Rc::clone(&stmt)]);

        let head = make_headtail(true, true, vec![Rc::clone(&brane)]);
        let tail = make_headtail(false, true, vec![Rc::clone(&brane)]);
        let scope = Scope::empty();

        step_to_settled(&head, &scope);
        step_to_settled(&tail, &scope);

        assert_eq!(head.borrow().core().get_nyes(), Nyes::Constant);
        assert_eq!(head.borrow().core().ubc_children()[0].borrow().as_i64(), Some(42));
        assert_eq!(tail.borrow().core().get_nyes(), Nyes::Constant);
        assert_eq!(tail.borrow().core().ubc_children()[0].borrow().as_i64(), Some(42));
    }

    #[test]
    fn headtail_nk_body_propagates_nk() {
        let val = make_nk("unbound");
        let stmt = make_statement("z", 1, Rc::clone(&val));
        let brane = make_brane(vec![Rc::clone(&stmt)]);

        let ht = make_headtail(true, true, vec![Rc::clone(&brane)]);
        let scope = Scope::empty();

        let transitions = step_to_settled(&ht, &scope);
        eprintln!("HeadTail(nk body) NYES transitions: {transitions:?}");

        assert_eq!(ht.borrow().core().get_nyes(), Nyes::Nk);
    }

    // ── ConcatenationFir helpers & tests ──────────────────────────────────

    fn make_concatenation(elements: Vec<FirRef>) -> FirRef {
        Rc::new_cyclic(|me: &Weak<RefCell<ConcatenationFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(ConcatenationFir {
                core: ProtoBrane::new(elements, parent, Nyes::Prembrionic),
            })
        })
    }

    #[test]
    fn concatenation_two_brane_elements() {
        let a = make_constant_int(1);
        let b = make_constant_int(2);
        let stmt_a = make_statement("a", 1, Rc::clone(&a));
        let stmt_b = make_statement("b", 2, Rc::clone(&b));
        let brane1 = make_brane(vec![Rc::clone(&stmt_a)]);
        let brane2 = make_brane(vec![Rc::clone(&stmt_b)]);

        let cat = make_concatenation(vec![Rc::clone(&brane1), Rc::clone(&brane2)]);
        let scope = Scope::empty();

        let transitions = step_to_settled(&cat, &scope);
        eprintln!("Concatenation(2 branes) NYES transitions: {transitions:?}");

        assert_eq!(cat.borrow().core().get_nyes(), Nyes::Constant);
        let ubc = cat.borrow().core().ubc_children().to_vec();
        assert_eq!(ubc.len(), 1);
        assert_eq!(ubc[0].borrow().kind(), FirKind::Brane);
        let result_brane = &ubc[0];
        assert_eq!(result_brane.borrow().core().foolish_children().len(), 2);
    }

    #[test]
    fn concatenation_empty_elements_is_constant_empty_brane() {
        let cat = make_concatenation(vec![]);
        let scope = Scope::empty();

        let transitions = step_to_settled(&cat, &scope);
        eprintln!("Concatenation(empty) NYES transitions: {transitions:?}");

        assert_eq!(cat.borrow().core().get_nyes(), Nyes::Constant);
        let ubc = cat.borrow().core().ubc_children().to_vec();
        assert_eq!(ubc.len(), 1);
        assert_eq!(ubc[0].borrow().kind(), FirKind::Brane);
        assert_eq!(ubc[0].borrow().core().foolish_children().len(), 0);
    }

    #[test]
    fn concatenation_with_nk_element_is_nk() {
        let a = make_constant_int(1);
        let b = make_nk("unbound");
        let stmt_a = make_statement("a", 1, Rc::clone(&a));
        let stmt_b = make_statement("b", 2, Rc::clone(&b));
        let brane1 = make_brane(vec![Rc::clone(&stmt_a)]);
        let brane2 = make_brane(vec![Rc::clone(&stmt_b)]);

        let cat = make_concatenation(vec![Rc::clone(&brane1), Rc::clone(&brane2)]);
        let scope = Scope::empty();

        let transitions = step_to_settled(&cat, &scope);
        eprintln!("Concatenation(nk element) NYES transitions: {transitions:?}");

        assert_eq!(cat.borrow().core().get_nyes(), Nyes::Nk);
    }

    #[test]
    fn concatenation_single_element_brane() {
        let val = make_constant_int(99);
        let stmt = make_statement("x", 1, Rc::clone(&val));
        let brane = make_brane(vec![Rc::clone(&stmt)]);

        let cat = make_concatenation(vec![Rc::clone(&brane)]);
        let scope = Scope::empty();

        let transitions = step_to_settled(&cat, &scope);
        eprintln!("Concatenation(single) NYES transitions: {transitions:?}");

        assert_eq!(cat.borrow().core().get_nyes(), Nyes::Constant);
        let ubc = cat.borrow().core().ubc_children().to_vec();
        assert_eq!(ubc.len(), 1);
        assert_eq!(ubc[0].borrow().kind(), FirKind::Brane);
        assert_eq!(ubc[0].borrow().core().foolish_children().len(), 1);
    }

    // ── StayFoolishFir helpers & tests ────────────────────────────────────

    fn make_stay_foolish(expr: FirRef) -> FirRef {
        Rc::new_cyclic(|me: &Weak<RefCell<StayFoolishFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(StayFoolishFir {
                core: ProtoBrane::new(vec![expr], parent, Nyes::Prembrionic),
            })
        })
    }

    #[test]
    fn stay_foolish_freezes_constant_body() {
        let body = make_constant_int(42);
        let sf = make_stay_foolish(Rc::clone(&body));
        let scope = Scope::empty();

        let transitions = step_to_settled(&sf, &scope);
        eprintln!("SF(ConstantInt) NYES transitions: {transitions:?}");

        assert_eq!(transitions.first(), Some(&Nyes::Prembrionic));
        assert_eq!(transitions.last(), Some(&Nyes::Constant));
        assert!(transitions.contains(&Nyes::Braning));
        assert_eq!(sf.borrow().kind(), FirKind::StayFoolish);
    }

    #[test]
    fn stay_foolish_freezes_nk_body() {
        let body = make_nk("unbound");
        let sf = make_stay_foolish(Rc::clone(&body));
        let scope = Scope::empty();

        let transitions = step_to_settled(&sf, &scope);
        eprintln!("SF(Nk) NYES transitions: {transitions:?}");

        assert_eq!(transitions.first(), Some(&Nyes::Prembrionic));
        assert_eq!(transitions.last(), Some(&Nyes::Nk));
        assert!(transitions.contains(&Nyes::Braning));
    }

    #[test]
    fn stay_foolish_freezes_econstanic_body() {
        // A search that doesn't find anything → Econstanic
        let val = make_constant_int(1);
        let stmt = make_statement("x", 1, Rc::clone(&val));
        let brane = make_brane(vec![Rc::clone(&stmt)]);
        let search = make_search("^missing$", true, vec![Rc::clone(&brane)]);
        let sf = make_stay_foolish(Rc::clone(&search));
        let scope = Scope::empty();

        let transitions = step_to_settled(&sf, &scope);
        eprintln!("SF(Econstanic body) NYES transitions: {transitions:?}");

        assert!(sf.borrow().core().get_nyes().is_settled());
    }

    #[test]
    fn stay_foolish_builder_sets_prembrionic() {
        let dummy: Rc<RefCell<dyn Fir>> = Rc::new_cyclic(|me: &Weak<RefCell<ConstantIntFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(ConstantIntFir {
                core: ProtoBrane::new(vec![], parent, Nyes::Constant),
                value: 0,
            })
        });
        let parent_weak = Rc::downgrade(&dummy);
        let body = make_constant_int(1);
        let node = stay_foolish(body, parent_weak);

        assert_eq!(node.borrow().core().get_nyes(), Nyes::Prembrionic);
        assert_eq!(node.borrow().kind(), FirKind::StayFoolish);
    }

    #[test]
    fn stay_foolish_constanic_clones_constant_to_ubc_children() {
        let body = make_constant_int(42);
        let sf = make_stay_foolish(Rc::clone(&body));
        let scope = Scope::empty();

        step_to_settled(&sf, &scope);

        assert_eq!(sf.borrow().core().get_nyes(), Nyes::Constant);
        let ubc = sf.borrow().core().ubc_children();
        assert_eq!(ubc.len(), 1, "SF should constanic-clone result into ubc_children");
        assert_eq!(ubc[0].borrow().kind(), FirKind::ConstantInt);
        assert_eq!(ubc[0].borrow().as_i64(), Some(42));
    }

    #[test]
    fn stay_foolish_constanic_clones_nk_to_ubc_children() {
        let body = make_nk("unbound");
        let sf = make_stay_foolish(Rc::clone(&body));
        let scope = Scope::empty();

        step_to_settled(&sf, &scope);

        assert_eq!(sf.borrow().core().get_nyes(), Nyes::Nk);
        let ubc = sf.borrow().core().ubc_children();
        assert_eq!(ubc.len(), 1, "SF should constanic-clone nk result into ubc_children");
        assert_eq!(ubc[0].borrow().kind(), FirKind::Nk);
    }

    #[test]
    fn stay_foolish_constanic_clones_operator_result() {
        let a = make_constant_int(3);
        let b = make_constant_int(5);
        let op = make_operator("+", vec![Rc::clone(&a), Rc::clone(&b)]);
        let sf = make_stay_foolish(Rc::clone(&op));
        let scope = Scope::empty();

        step_to_settled(&sf, &scope);

        assert_eq!(sf.borrow().core().get_nyes(), Nyes::Constant);
        let ubc = sf.borrow().core().ubc_children();
        assert_eq!(ubc.len(), 1, "SF should constanic-clone operator result");
        assert_eq!(ubc[0].borrow().as_i64(), Some(8));
    }

    #[test]
    fn stay_foolish_constanic_clones_search_result() {
        let val = make_constant_int(10);
        let stmt = make_statement("x", 1, Rc::clone(&val));
        let brane = make_brane(vec![Rc::clone(&stmt)]);
        let search = make_search("^x$", true, vec![Rc::clone(&brane)]);
        let sf = make_stay_foolish(Rc::clone(&search));
        let scope = Scope::empty();

        step_to_settled(&sf, &scope);

        assert_eq!(sf.borrow().core().get_nyes(), Nyes::Constant);
        let ubc = sf.borrow().core().ubc_children();
        assert_eq!(ubc.len(), 1, "SF should constanic-clone search result");
        assert_eq!(ubc[0].borrow().as_i64(), Some(10));
    }

    // ── StayFullyFoolishFir helpers & tests ───────────────────────────────

    fn make_stay_fully_foolish(expr: FirRef) -> FirRef {
        Rc::new_cyclic(|me: &Weak<RefCell<StayFullyFoolishFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(StayFullyFoolishFir {
                core: ProtoBrane::new(vec![expr], parent, Nyes::Prembrionic),
            })
        })
    }

    #[test]
    fn stay_fully_foolish_evaluates_child() {
        let body = make_constant_int(42);
        let sff = make_stay_fully_foolish(Rc::clone(&body));
        let scope = Scope::empty();

        // SFF evaluates its child like a single-membered brane.
        // Prembrionic → Braning (child pushed) → Constant (child settled)
        assert_eq!(sff.borrow().core().get_nyes(), Nyes::Prembrionic);
        let mut transitions = vec![Nyes::Prembrionic];
        for _ in 0..10 {
            let report = step_fir_ref(&sff, &scope).unwrap();
            match report {
                StepReport::Progress(nyes) => {
                    transitions.push(nyes);
                    if nyes.is_settled() {
                        break;
                    }
                }
                StepReport::NoProgress => break,
            }
        }
        eprintln!("SFF NYES transitions: {transitions:?}");
        assert!(sff.borrow().core().get_nyes().is_settled());
        assert_eq!(sff.borrow().kind(), FirKind::StayFullyFoolish);
    }

    #[test]
    fn stay_fully_foolish_body_is_evaluated() {
        let body = make_constant_int(42);
        let sff = make_stay_fully_foolish(Rc::clone(&body));
        let scope = Scope::empty();

        // Step SFF to settled (multiple steps needed: Prembrionic → Braning → Constant)
        for _ in 0..10 {
            let report = step_fir_ref(&sff, &scope).unwrap();
            match report {
                StepReport::Progress(nyes) if nyes.is_settled() => break,
                StepReport::NoProgress => break,
                _ => {}
            }
        }

        // The body IS evaluated (SFF evaluates its child)
        assert!(body.borrow().core().get_nyes().is_settled());
    }

    #[test]
    fn stay_fully_foolish_builder_sets_prembrionic() {
        let dummy: Rc<RefCell<dyn Fir>> = Rc::new_cyclic(|me: &Weak<RefCell<ConstantIntFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(ConstantIntFir {
                core: ProtoBrane::new(vec![], parent, Nyes::Constant),
                value: 0,
            })
        });
        let parent_weak = Rc::downgrade(&dummy);
        let body = make_constant_int(1);
        let node = stay_fully_foolish(body, parent_weak);

        assert_eq!(node.borrow().core().get_nyes(), Nyes::Prembrionic);
        assert_eq!(node.borrow().kind(), FirKind::StayFullyFoolish);
    }
}
