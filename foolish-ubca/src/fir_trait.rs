//! Fir trait — the dyn-dispatch surface for UBCa FIR nodes.
//!
//! Each FIR kind implements this trait with its own `fir_op_step` (combining
//! work) and `kind()`. Shared stepping logic lives in [`FirRefExt::step`], an
//! extension method on the `FirRef` handle rather than a method on the inner
//! `dyn Fir` — this is critical for borrow discipline (transient borrows of the
//! pointee dropped before recursion).

use std::cell::RefCell;
use std::rc::Rc;

use foolish_core::fir::Nyes;

use crate::proto_brane::ProtoBrane;

/// A FIR reference: `Rc<RefCell<dyn Fir>>`.
pub type FirRef = Rc<RefCell<dyn Fir>>;

/// Step report: either no progress (outer loop may stop), or progress with
/// the node's current NYES after the action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepReport {
    /// No progress was made — the outer loop may stop.
    NoProgress,
    /// Progress was made; the node's current NYES is reported.
    Progress(Nyes),
}

/// Identifies the kind of FIR node (for dispatch and debugging).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FirKind {
    Brane,
    Statement,
    Operator,
    Search,
    Index,
    StayFoolish,
    StayFullyFoolish,
    Concatenation,
    /// Internal storage brane for ConcatBrane — holds constanic-cloned statements.
    /// Transparent: inherits all defaults, BraneFir-shaped stepping.
    ConcatHelper,
    IndepInt,
    Nk,
    /// Placeholder for test stubs before real kinds are implemented.
    Unknown,
    /// Strong reference to an original statement, created alongside search results.
    /// Born CONSTANT, immutable, no stepping. Bookkeeping for contexted search (FOOP-23 C2).
    FoolRef,
    /// Creation dot (⬤ / {*}). Born Independent, identity via Rc::ptr_eq.
    /// FOOP-33 Phase 2.
    Creation,
    /// A `system.foo` comparison operator (`'lt`/`'gt`/`'le`/`'ge`/`'eq`).
    /// Two SFF-marked operand lookups, one Rust comparison, a `'True`/`'False`
    /// result. FOOP-33 §5.0; see `system_foo::ComparisonFir`.
    Comparison,
    /// A `system.foo` arithmetic operator (`'mod`). Two SFF-marked operand
    /// lookups, one Rust arithmetic operation, an integer result. FOOP-55 §1.
    Modulo,
    /// A `system.foo` boolean OR operator (`'or`). Two SFF-marked operand
    /// lookups, boolean logic on `'True`/`'False` creations. FOOP-55 §2.
    Or,
    /// A `system.foo` order-statistic operator (`'min_int_val`,
    /// `'max_int_val`). Unlike the operators above it takes **no fixed
    /// operands**: declared as a BRANE wrapping the FIR, it is spliced into a
    /// concatenation and folds the integers preceding it, selecting by index
    /// into their ascending sort. FOOP-55 §7.
    Extremum,
}

/// Minimal Scope stub — enough for compilation. The real Scope comes later
/// (FOOP-62 §10).
#[derive(Debug, Clone)]
pub struct Scope {
    pub current_brane: Option<FirRef>,
    pub current_statement: Option<FirRef>,
    pub has_ancestral_sfm: bool,
}

impl Scope {
    pub fn empty() -> Self {
        Self {
            current_brane: None,
            current_statement: None,
            has_ancestral_sfm: false,
        }
    }

    pub fn with_ancestral_sfm(&self, has_ancestral_sfm: bool) -> Self {
        Self {
            has_ancestral_sfm,
            ..self.clone()
        }
    }

    pub fn get_my_statement(&self) -> Option<FirRef> {
        self.current_statement.clone()
    }

    pub fn get_my_brane(&self) -> Option<FirRef> {
        self.current_brane.clone()
    }
}

/// UBCa evaluation error type.
#[derive(Debug, thiserror::Error)]
pub enum UbcError {
    /// An evaluation error with a human-readable message.
    #[error("ubca evaluation error: {0}")]
    Eval(String),

    /// The FVM's own state or construction violates an invariant it relies on.
    ///
    /// Distinct from [`UbcError::Eval`]: an `Eval` error is a *program* being
    /// unevaluable (a legitimate outcome — division by zero, an unresolvable
    /// name). An `InternalConsistency` error means the **interpreter** is
    /// broken — a FIR was built or mutated into a shape the evaluator's rules
    /// say cannot exist. Continuing past one silently produces wrong results,
    /// so callers should halt rather than recover.
    ///
    /// Example: an SFF body carrying a search that is not ECONSTANIC, i.e. a
    /// body that is supposed to be constanic-unevaluated but contains a search
    /// that can still run.
    #[error("ubca INTERNAL CONSISTENCY error: {0}")]
    InternalConsistency(String),
}

/// The dyn-dispatch trait for UBCa FIR nodes.
///
/// Every FIR kind contains a `ProtoBrane` (accessed via `core()`)
/// and implements `fir_op_step` with its own combining work. Stepping itself
/// is the shared [`FirRefExt::step`] extension method — kinds differ through
/// construction-time state and `fir_op_step`, not by overriding the step.
pub trait Fir: std::fmt::Debug {
    /// Read-only access to the shared field-holder.
    /// ProtoBrane uses interior mutability, so `&ProtoBrane` suffices for
    /// all mutations (nyes via Cell, tasks/ubc_children via RefCell).
    fn core(&self) -> &ProtoBrane;

    /// The node's OWN combining work, run once child tasks are drained.
    /// Reads neighbors into locals (borrows dropped) before writing.
    /// May push result tasks and advance the node's own nyes via
    /// `core().set_nyes(…)`.
    fn fir_op_step(&self, scope: &Scope) -> Result<(), UbcError>;

    /// Identify the kind of FIR node.
    fn kind(&self) -> FirKind;

    /// Read the integer value. Default: look through `settled_result` for resolved
    /// results. IndepIntFir overrides to return `Some(value)`.
    fn as_i64(&self) -> Option<i64> {
        self.settled_result().and_then(|c| c.borrow().as_i64())
    }

    // ── Accessors for proto_to_core_fir bridge ──

    /// NkFir reason. Default: None.
    fn as_nk_reason(&self) -> Option<&str> {
        None
    }

    /// OperatorFir op name. Default: None.
    fn as_op_name(&self) -> Option<&str> {
        None
    }

    /// StatementFir identifier. Default: None.
    fn as_stmt_identifier(&self) -> Option<&crate::identifier::Identifier> {
        None
    }

    /// StatementFir's searchable name — the full characterized LHS as one string
    /// (e.g. `"tag'x"`), what every name-search matches against. Default method:
    /// derived from `as_stmt_identifier()`, so it is `None` for any non-Statement FIR
    /// and for the whole-brane root convention. See `Identifier::searchable_name`.
    fn as_stmt_searchable_name(&self) -> Option<&str> {
        self.as_stmt_identifier().map(|id| id.searchable_name())
    }

    /// StatementFir line number. Default: None.
    fn as_stmt_line_number(&self) -> Option<usize> {
        None
    }

    /// SearchFir pattern. Default: None.
    fn as_search_pattern(&self) -> Option<&str> {
        None
    }

    /// SearchFir anchored flag. Default: false.
    fn as_search_anchored(&self) -> bool {
        false
    }

    /// IndexFir offset. Default: 0.
    fn as_index_offset(&self) -> i32 {
        0
    }

    /// IndexFir anchored flag. Default: false.
    fn as_index_anchored(&self) -> bool {
        false
    }

    /// BraneFir characterizations. Default: empty.
    fn as_brane_characterizations(&self) -> &[String] {
        &[]
    }

    /// The name a creation reports for itself, if any. Default: `None`.
    ///
    /// Only [`CreationFir`](crate::fir_kinds::CreationFir) overrides this, by
    /// delegating to its own `get_display_name`. See that method for the
    /// two-condition rule (FOOP-33): a creation names itself only when (1) it
    /// is being viewed from a statement OTHER than its own defining
    /// statement, AND (2) that defining statement's name is
    /// null-characterized.
    ///
    /// `self_ref` must be the `FirRef` wrapping `self` — the parent chain is
    /// reached through it, exactly as [`Fir::_get_my_brane`] does.
    /// `viewed_from` is the statement currently being rendered (the one
    /// whose body led the caller here); `None` if there is no such statement
    /// in scope, which conservatively means "not a reference elsewhere" and
    /// so the name is never shown.
    fn as_creation_display_name(
        &self,
        _self_ref: &FirRef,
        _viewed_from: Option<&FirRef>,
    ) -> Option<String> {
        None
    }

    /// SF inner search pattern — set when a search resolved through SF
    /// re-evaluation. Used by the humanizer to preserve the search wrapper.
    fn as_sf_inner_pattern(&self) -> Option<String> {
        None
    }

    /// SearchFir is_value_search flag. Default: false.
    fn as_search_is_value(&self) -> bool {
        false
    }

    /// FoolRefFir referent. Default: None.
    fn as_fool_ref_referent(&self) -> Option<&FirRef> {
        None
    }

    /// Whether this search/index is contexted (&-prefixed). Default: false.
    fn as_search_contexted(&self) -> bool {
        false
    }

    /// FOOP-33 §4 — set by `ConcatenationFir`'s merge-collision check
    /// (`apply_null_const_rule_to_merged_stmt`) on a `StatementFir` it just
    /// cloned into the merge, when that clone conflicts with an
    /// already-merged null-characterized statement of the same name.
    /// Default: no-op (every non-`StatementFir` kind ignores this — only a
    /// `StatementFir` HAS an `nf_reason` to substitute via `settled_result`).
    /// `&self` + interior mutability, matching every other post-construction
    /// mutation in this crate (`RefCell`/`Cell` fields, never `&mut self`
    /// through a `dyn Fir` trait object).
    fn set_nf_reason(&self, _reason: String) {}

    fn _get_my_statement(&self, self_ref: &FirRef) -> FirRef {
        match self.kind() {
            FirKind::Statement => Rc::clone(self_ref),
            _ => match self.core().parent() {
                Some(p) => {
                    if Rc::ptr_eq(&p, self_ref) {
                        Rc::clone(self_ref)
                    } else {
                        p.borrow()._get_my_statement(&p)
                    }
                }
                None => Rc::clone(self_ref),
            },
        }
    }

    /// Iterative parent-walk. Climbs `.parent()` until a brane-like kind is found
    /// (capability: `stmt_count().is_some()`). Returns the brane that owns `self`,
    /// or `None` at the root.
    ///
    /// **"Home brane of a FIR" ≡ "brane of a FIR"** — these two phrases mean
    /// exactly the same thing (per FOOP-23 §Terminology). Use "home brane"
    /// when a second brane is also under discussion and the specific one must
    /// be named; use "brane of" otherwise.
    ///
    /// Two-mechanism asymmetry: `_get_my_brane` (underscore) climbs the parent
    /// chain; `get_my_brane` on [`Scope`] reads the scope cache.
    fn _get_my_brane(&self, self_ref: &FirRef) -> Option<FirRef> {
        match self.core().parent() {
            Some(p) => {
                if Rc::ptr_eq(&p, self_ref) {
                    return None;
                }
                if p.borrow().is_brane_like() {
                    Some(p)
                } else {
                    p.borrow()._get_my_brane(&p)
                }
            }
            None => None,
        }
    }

    fn set_contexted(&mut self, _contexted: bool) {}

    /// Immediate brane search — find a named statement before `self` in the home brane.
    ///
    /// Call chain: `StatementFir::_ib_search` → `_get_my_brane(self_ref)` →
    /// `brane._search_brane(name, line_number-1, 0)`.
    /// Scope-cached twin: `ib_search(scope, name)` reads [`Scope::current_statement`]
    /// instead of walking parents.
    fn _ib_search(&self, self_ref: &FirRef, name: &str) -> Option<(FirRef, Nyes)> {
        let stmt = self._get_my_statement(self_ref);
        let borrowed = stmt.borrow();
        borrowed._ib_search(&stmt, name)
    }

    fn ib_search(&self, scope: &Scope, name: &str) -> Option<(FirRef, Nyes)> {
        let stmt = scope.get_my_statement()?;
        let borrowed = stmt.borrow();
        borrowed._ib_search(&stmt, name)
    }

    /// Ancestral brane search — find a named statement in ancestor branes.
    ///
    /// Call chain: `_get_my_brane(self_ref)` → `brane._ab_search(brane, name)` →
    /// recurses up the parent chain.
    /// Scope-cached twin: `ab_search(scope, name)` reads [`Scope::current_brane`]
    /// instead of walking parents.
    fn _ab_search(&self, self_ref: &FirRef, name: &str) -> Option<(FirRef, Nyes)> {
        let brane = self._get_my_brane(self_ref)?;
        let borrowed = brane.borrow();
        borrowed._ab_search(&brane, name)
    }

    fn ab_search(&self, scope: &Scope, name: &str) -> Option<(FirRef, Nyes)> {
        let brane = scope.get_my_brane()?;
        let borrowed = brane.borrow();
        borrowed._ab_search(&brane, name)
    }

    fn _search_brane(
        &self,
        _expression: &str,
        _starting_index: usize,
        _ending_index: usize,
    ) -> Option<(usize, FirRef, Nyes)> {
        None
    }

    // ── Capability methods (FOOP-13 A2) ──

    /// Number of statements this FIR presents as a brane. `None` = not brane-like.
    /// An `Extremum` FIR's sort index and `system.foo` name (FOOP-55 §7).
    /// `None` for every other kind.
    fn as_extremum_config(&self) -> Option<(i64, &'static str)> {
        None
    }

    fn stmt_count(&self) -> Option<usize> {
        None
    }

    /// The statement at a global index, per the Equivalence Law.
    fn stmt_at(&self, _idx: usize) -> Option<FirRef> {
        None
    }

    /// The settled result this FIR resolves to, if any.
    /// CONTRACT: applies the constanic gate ITSELF — pre-constanic always answers None.
    fn settled_result(&self) -> Option<FirRef> {
        if !self.core().get_nyes().is_constanic() {
            return None;
        }
        self.core().ubc_children().into_iter().next()
    }

    /// Whether this FIR is brane-like (has statements to iterate).
    fn is_brane_like(&self) -> bool {
        self.stmt_count().is_some()
    }

    /// Whether this FIR is a **search kind** — one that resolves by *looking
    /// something up* rather than by combining already-present operands.
    ///
    /// Today: [`FirKind::Search`] (name/value/regexp/dot searches) and
    /// [`FirKind::Index`] (`#N`, `^`/`$` head-tail). These are exactly the
    /// kinds `compiler::build_fir`'s `under_sff` rule targets — under an SFF
    /// marker they are constructed ECONSTANIC so they never run.
    ///
    /// One predicate rather than open-coded `matches!` at each site, so a
    /// future search kind is added in one place.
    fn is_search_kind(&self) -> bool {
        matches!(self.kind(), FirKind::Search | FirKind::Index)
    }

    // ── UBCA debugging facilities ──

    /// Peek at the front of the task queue without removing it.
    ///
    /// Used by `step_until` utilities to check which FIR is next in line
    /// for stepping — equivalent to a debugger breakpoint on the job queue.
    fn debug_front_task(&self) -> Option<FirRef> {
        self.core().front_task()
    }
}

/// Guard against runaway recursion on pathologically deep trees.
const MAX_DEPTH: usize = 100;

/// Behaviors a [`FirRef`] performs on itself: resolving to a value and stepping.
///
/// # Why an extension trait
///
/// [`FirRef`] is a type *alias* for the foreign type `Rc<RefCell<dyn Fir>>`, so
/// the orphan rule forbids an inherent `impl FirRef { … }`. To give these
/// operations `self`-method call syntax (`node.value()`, `root.step(&scope)`)
/// we attach them to the `Rc` handle through an extension trait — the same
/// pattern this crate already uses for [`crate::nyes_ext::NyesExt`], which bolts
/// predicates onto the foreign `Nyes` enum.
///
/// # Borrow discipline
///
/// These methods take `&self` (the `Rc` handle), *not* `&self` of the inner
/// `dyn Fir`. Each `RefCell` borrow of the pointee is opened and dropped within
/// a single statement before any recursion or `fir_op_step` call. This is the
/// invariant that keeps stepping panic-free: a borrow of the inner FIR must
/// never be held across a recursive step, or the nested `borrow`/`borrow_mut`
/// would alias and panic. Because the receiver is the pointer, not a borrow of
/// the pointee, that discipline is preserved exactly as it was when these were
/// free functions over `&FirRef`.
pub trait FirRefExt {
    /// Returns the deepest resolved value this FIR represents.
    ///
    /// Delegates to [`Fir::settled_result`] to obtain the constanic-gated
    /// result child. If a result exists, recurses to unwrap nested wrappers.
    /// For pre-constanic FIRs or FIRs with no settled result, returns a clone
    /// of `self`.
    #[must_use]
    fn value(&self) -> FirRef;

    /// Performs ONE stepping action (check-then-act) and reports progress.
    ///
    /// The outer driver loops `root.step(&scope)` until the node is constanic.
    /// See the trait-level note on borrow discipline.
    fn step(&self, scope: &Scope) -> Result<StepReport, UbcError>;
}

impl FirRefExt for FirRef {
    fn value(&self) -> FirRef {
        let child = self.borrow().settled_result();
        match child {
            Some(c) => c.value(),
            None => Rc::clone(self),
        }
    }

    fn step(&self, scope: &Scope) -> Result<StepReport, UbcError> {
        step_inner(self, scope, 0)
    }
}

/// Recursion companion for [`FirRefExt::step`], carrying the depth counter.
///
/// Kept as a private free function rather than a trait method so the internal
/// `depth` parameter never leaks into the public `step` signature.
fn step_inner(this: &FirRef, scope: &Scope, depth: usize) -> Result<StepReport, UbcError> {
    if depth > MAX_DEPTH {
        return Ok(StepReport::NoProgress);
    }
    let front = this.borrow().core().front_task();

    match front {
        Some(front_rc) => {
            if front_rc.borrow().core().get_nyes().is_constanic() {
                this.borrow().core().pop_front_task();
            } else {
                let this_kind = this.borrow().kind();
                let mut child_scope = if this_kind == FirKind::StayFoolish {
                    scope.with_ancestral_sfm(true)
                } else {
                    scope.clone()
                };
                if this_kind == FirKind::Statement {
                    child_scope.current_statement = Some(Rc::clone(this));
                }
                if this.borrow().is_brane_like() {
                    child_scope.current_brane = Some(Rc::clone(this));
                }
                step_inner(&front_rc, &child_scope, depth + 1)?;
            }
            Ok(StepReport::Progress(this.borrow().core().get_nyes()))
        }
        None => {
            this.borrow().fir_op_step(scope)?;
            let nyes = this.borrow().core().get_nyes();
            Ok(StepReport::Progress(nyes))
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::proto_brane::ProtoBrane;
    use std::rc::Weak;

    // --- Test FIR kinds ---

    /// A leaf FIR that transitions Prembrionic → Embryonic → Braning → Woconstanic.
    #[derive(Debug)]
    pub(crate) struct LeafFir {
        pub(crate) core: ProtoBrane,
    }

    impl Fir for LeafFir {
        fn core(&self) -> &ProtoBrane {
            &self.core
        }
        fn fir_op_step(&self, _scope: &Scope) -> Result<(), UbcError> {
            // Leaf: no children, no tasks — advance directly to terminal.
            // In the real model, fir_op_step produces a constanic state or pushes
            // more tasks. A leaf's work is immediate.
            if !self.core.get_nyes().is_constanic() {
                self.core.set_nyes(Nyes::Woconstanic);
            }
            Ok(())
        }
        fn kind(&self) -> FirKind {
            FirKind::Unknown
        }
    }

    /// A brane FIR that steps Prembrionic → Embryonic → Braning, then
    /// drains its child tasks, then classifies.
    ///
    /// Test-only stand-in for `fir_kinds::BraneFir`, named distinctly to avoid
    /// colliding with the production type.
    #[derive(Debug)]
    pub(crate) struct TestBraneFir {
        pub(crate) core: ProtoBrane,
    }

    impl Fir for TestBraneFir {
        fn core(&self) -> &ProtoBrane {
            &self.core
        }
        fn fir_op_step(&self, _scope: &Scope) -> Result<(), UbcError> {
            let nyes = self.core.get_nyes();
            match nyes {
                Nyes::Prembrionic => {
                    // Build task queue from foolish_children, advance to Braning
                    self.core.set_nyes(Nyes::Braning);
                    let children: Vec<FirRef> = self.core.foolish_children().to_vec();
                    for child in children {
                        self.core.push_task(child);
                    }
                }
                Nyes::Braning | Nyes::Woconstanic => {
                    // Classification: check children's states
                    let children = self.core.foolish_children().to_vec();
                    let all_constanic = children
                        .iter()
                        .all(|c| c.borrow().core().get_nyes().is_constanic());
                    if all_constanic {
                        let any_nk = children
                            .iter()
                            .any(|c| c.borrow().core().get_nyes() == Nyes::Nk);
                        if any_nk {
                            self.core.set_nyes(Nyes::Nk);
                        } else {
                            self.core.set_nyes(Nyes::Woconstanic);
                        }
                    }
                }
                _ => {}
            }
            Ok(())
        }
        fn kind(&self) -> FirKind {
            FirKind::Brane
        }
    }

    /// Create a self-rooting leaf (for use as a standalone test node).
    /// Uses `Rc::new_cyclic` on the concrete `LeafFir` type so the parent
    /// Weak is a self-pointer (unsized coercion to FirRef at the let binding).
    pub(crate) fn make_leaf(nyes: Nyes) -> FirRef {
        Rc::new_cyclic(|me: &Weak<RefCell<LeafFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(LeafFir {
                core: ProtoBrane::new(vec![], parent, nyes),
            })
        })
    }

    /// Create a self-rooting brane (for use as a standalone test node).
    pub(crate) fn make_brane(children: Vec<FirRef>) -> FirRef {
        Rc::new_cyclic(|me: &Weak<RefCell<TestBraneFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(TestBraneFir {
                core: ProtoBrane::new(children, parent, Nyes::Prembrionic),
            })
        })
    }

    /// Build a root brane with children. Children are re-parented to the root.
    /// Uses `Rc::new_cyclic` on concrete `TestBraneFir`; the `FirRef` type
    /// annotation triggers unsized coercion.
    pub(crate) fn make_root_brane(children: Vec<FirRef>) -> FirRef {
        Rc::new_cyclic(|me: &Weak<RefCell<TestBraneFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            // Re-parent all children to this root
            // (children were created self-rooting; re-wire them)
            for child in &children {
                // We can't mutate the child's ProtoBrane parent (it's immutable
                // after construction). But the tests only step downward, so the
                // incorrect parent on children is harmless for these tests.
                let _ = child;
            }
            RefCell::new(TestBraneFir {
                core: ProtoBrane::new(children, parent, Nyes::Prembrionic),
            })
        })
    }

    #[test]
    fn step_report_equality() {
        assert_eq!(StepReport::NoProgress, StepReport::NoProgress);
        assert_eq!(
            StepReport::Progress(Nyes::Braning),
            StepReport::Progress(Nyes::Braning)
        );
        assert_ne!(
            StepReport::Progress(Nyes::Braning),
            StepReport::Progress(Nyes::Constant)
        );
    }

    #[test]
    fn step_leaf_through_nyes_transitions() {
        // A leaf starts at Prembrionic and should step through:
        // Prembrionic → Embryonic → Braning → Woconstanic (constanic)
        let leaf = make_leaf(Nyes::Prembrionic);
        let scope = Scope::empty();

        let mut transitions = vec![leaf.borrow().core().get_nyes()];

        // Step until settled
        for _ in 0..10 {
            let report = leaf.step(&scope).unwrap();
            match report {
                StepReport::Progress(nyes) => {
                    transitions.push(nyes);
                    if nyes.is_constanic() {
                        break;
                    }
                }
                StepReport::NoProgress => break,
            }
        }

        eprintln!("Leaf NYES transitions: {transitions:?}");
        assert_eq!(transitions, vec![Nyes::Prembrionic, Nyes::Woconstanic,]);
    }

    #[test]
    fn step_brane_drains_children_then_classifies() {
        // Create a child leaf (starts Prembrionic)
        let child = make_leaf(Nyes::Prembrionic);

        // Create a root brane containing that child
        let root = make_root_brane(vec![Rc::clone(&child)]);
        let scope = Scope::empty();

        let mut root_transitions = vec![root.borrow().core().get_nyes()];
        let mut child_transitions = vec![child.borrow().core().get_nyes()];

        // Step the root — this should recurse into the child via the task queue
        for _ in 0..20 {
            let report = root.step(&scope).unwrap();
            match report {
                StepReport::Progress(nyes) => {
                    root_transitions.push(nyes);
                    // Track child's state too
                    child_transitions.push(child.borrow().core().get_nyes());
                    if nyes.is_constanic() {
                        break;
                    }
                }
                StepReport::NoProgress => break,
            }
        }

        eprintln!("Root NYES transitions: {root_transitions:?}");
        eprintln!("Child NYES transitions: {child_transitions:?}");

        // Root should eventually reach Woconstanic (child settled as Woconstanic)
        let final_root = root.borrow().core().get_nyes();
        assert!(
            final_root.is_constanic(),
            "root should be settled, got {final_root}"
        );

        // Child should be settled
        let final_child = child.borrow().core().get_nyes();
        assert!(
            final_child.is_constanic(),
            "child should be settled, got {final_child}"
        );
    }

    #[test]
    fn step_brane_with_two_children_drains_in_order() {
        let child_a = make_leaf(Nyes::Prembrionic);
        let child_b = make_leaf(Nyes::Prembrionic);

        let root = make_root_brane(vec![Rc::clone(&child_a), Rc::clone(&child_b)]);
        let scope = Scope::empty();

        let mut root_nyes_log = vec![];

        for i in 0..30 {
            let report = root.step(&scope).unwrap();
            match report {
                StepReport::Progress(nyes) => {
                    root_nyes_log.push((i, nyes));
                    if nyes.is_constanic() {
                        break;
                    }
                }
                StepReport::NoProgress => break,
            }
        }

        eprintln!("Root NYES log: {root_nyes_log:?}");

        // Both children should be settled
        assert!(child_a.borrow().core().get_nyes().is_constanic());
        assert!(child_b.borrow().core().get_nyes().is_constanic());
        // Root should be settled
        assert!(root.borrow().core().get_nyes().is_constanic());
    }

    #[test]
    fn step_already_settled_returns_progress() {
        let leaf = make_leaf(Nyes::Constant);
        let scope = Scope::empty();

        let report = leaf.step(&scope).unwrap();
        // A settled leaf with empty task queue: fir_op_step runs (no-op for Constant)
        // and returns Progress(Constant)
        assert_eq!(report, StepReport::Progress(Nyes::Constant));
    }

    #[test]
    fn step_fir_ref_transient_borrow_safety() {
        // This test verifies the transient-borrow discipline: we can step a
        // node that is referenced by another node without RefCell panics.
        let inner = make_leaf(Nyes::Prembrionic);
        let outer = make_brane(vec![Rc::clone(&inner)]);
        let scope = Scope::empty();

        // Step outer — it will recurse into inner through the task queue
        // If borrow discipline were wrong (nested borrow_mut), this would panic
        let result = outer.step(&scope);
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod get_value_tests {
    use super::*;
    use crate::fir_kinds::{
        BraneFir, ConcatenationFir, IndepIntFir, IndexFir, NkFir, OperatorFir, SearchFir,
        StatementFir, StayFoolishFir, StayFullyFoolishFir,
    };
    use foolish_core::fir::Nyes;
    use std::rc::Weak;

    fn make_ci(v: i64) -> FirRef {
        Rc::new_cyclic(|me: &Weak<RefCell<IndepIntFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(IndepIntFir {
                core: ProtoBrane::new(vec![], parent, Nyes::Prembrionic),
                value: v,
            })
        })
    }

    fn make_nk_node(reason: &str) -> FirRef {
        Rc::new_cyclic(|me: &Weak<RefCell<NkFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(NkFir {
                core: ProtoBrane::new(vec![], parent, Nyes::Prembrionic),
                reason: reason.to_owned(),
            })
        })
    }

    fn make_op(op: &str, operands: Vec<FirRef>) -> FirRef {
        Rc::new_cyclic(|me: &Weak<RefCell<OperatorFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(OperatorFir {
                core: ProtoBrane::new(operands, parent, Nyes::Prembrionic),
                op: op.to_owned(),
            })
        })
    }

    fn make_stmt(name: &str, line: usize, body: FirRef) -> FirRef {
        Rc::new_cyclic(|me: &Weak<RefCell<StatementFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            let self_weak: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(StatementFir {
                core: ProtoBrane::new(vec![body], parent, Nyes::Prembrionic),
                identifier: crate::identifier::Identifier::from_parts(vec![], name),
                line_number: line,
                self_weak,
                nf_reason: RefCell::new(None),
            })
        })
    }

    fn make_brn(children: Vec<FirRef>) -> FirRef {
        Rc::new_cyclic(|me: &Weak<RefCell<BraneFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(BraneFir {
                core: ProtoBrane::new(children, parent, Nyes::Prembrionic),
                characterizations: crate::identifier::Characterizations::default(),
            })
        })
    }

    fn make_search(pattern: &str, anchored: bool, children: Vec<FirRef>) -> FirRef {
        Rc::new_cyclic(|me: &Weak<RefCell<SearchFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(SearchFir {
                core: ProtoBrane::new(children, parent, Nyes::Prembrionic),
                pattern: pattern.to_owned(),
                anchored,
                forward: false,
                sf_inner_pattern: RefCell::new(None),
                is_value_search: false,
                contexted: false,
            })
        })
    }

    fn make_index(offset: i32, anchored: bool, children: Vec<FirRef>) -> FirRef {
        Rc::new_cyclic(|me: &Weak<RefCell<IndexFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(IndexFir {
                core: ProtoBrane::new(children, parent, Nyes::Prembrionic),
                offset,
                anchored,
                contexted: false,
            })
        })
    }

    fn make_sf(expr: FirRef) -> FirRef {
        Rc::new_cyclic(|me: &Weak<RefCell<StayFoolishFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(StayFoolishFir {
                core: ProtoBrane::new(vec![expr], parent, Nyes::Prembrionic),
            })
        })
    }

    fn make_sff(expr: FirRef) -> FirRef {
        Rc::new_cyclic(|me: &Weak<RefCell<StayFullyFoolishFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(StayFullyFoolishFir {
                core: ProtoBrane::new(vec![expr], parent, Nyes::Prembrionic),
            })
        })
    }

    fn make_cat(elements: Vec<FirRef>) -> FirRef {
        Rc::new_cyclic(|me: &Weak<RefCell<ConcatenationFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(ConcatenationFir {
                core: ProtoBrane::new(elements, parent, Nyes::Prembrionic),
                _helpers_populated: std::cell::Cell::new(false),
            })
        })
    }

    fn settle(node: &FirRef) {
        let scope = Scope::empty();
        for _ in 0..50 {
            let report = node.step(&scope).unwrap();
            if let StepReport::Progress(nyes) = report
                && nyes.is_constanic()
            {
                return;
            }
        }
        panic!("did not settle within 50 steps");
    }

    // ── 1. IndepInt ──────────────────────────────────────────────────────

    #[test]
    fn get_value_constant_int_returns_self() {
        let ci = make_ci(42);
        settle(&ci);
        assert_eq!(ci.borrow().core().get_nyes(), Nyes::Constant);
        assert!(ci.borrow().core().ubc_children().is_empty());

        let result = ci.value();
        assert!(Rc::ptr_eq(&result, &ci));
        assert_eq!(result.borrow().kind(), FirKind::IndepInt);
        assert_eq!(result.borrow().as_i64(), Some(42));
    }

    // ── 2. Nk ───────────────────────────────────────────────────────────────

    #[test]
    fn get_value_nk_returns_self() {
        let nk = make_nk_node("unbound");
        settle(&nk);
        assert_eq!(nk.borrow().core().get_nyes(), Nyes::Nk);
        assert!(nk.borrow().core().ubc_children().is_empty());

        let result = nk.value();
        assert!(Rc::ptr_eq(&result, &nk));
        assert_eq!(result.borrow().kind(), FirKind::Nk);
    }

    // ── 3. OperatorFir settled ──────────────────────────────────────────────

    #[test]
    fn get_value_operator_settled_returns_result() {
        let a = make_ci(3);
        let b = make_ci(5);
        let op = make_op("+", vec![Rc::clone(&a), Rc::clone(&b)]);
        settle(&op);
        assert_eq!(op.borrow().core().get_nyes(), Nyes::Constant);
        assert_eq!(op.borrow().core().ubc_children().len(), 1);

        let result = op.value();
        assert!(!Rc::ptr_eq(&result, &op));
        assert_eq!(result.borrow().kind(), FirKind::IndepInt);
        assert_eq!(result.borrow().as_i64(), Some(8));
    }

    // ── 4. OperatorFir not settled ──────────────────────────────────────────

    #[test]
    fn get_value_operator_not_settled_returns_self() {
        let a = make_ci(3);
        let b = make_ci(5);
        let op = make_op("+", vec![Rc::clone(&a), Rc::clone(&b)]);
        assert!(!op.borrow().core().get_nyes().is_constanic());

        let result = op.value();
        assert!(Rc::ptr_eq(&result, &op));
    }

    // ── 5. SearchFir settled with result ────────────────────────────────────

    #[test]
    fn get_value_search_settled_with_result_returns_body() {
        let val = make_ci(42);
        let stmt = make_stmt("x", 0, Rc::clone(&val));
        let brane = make_brn(vec![Rc::clone(&stmt)]);
        let search = make_search("^x$", true, vec![Rc::clone(&brane)]);
        settle(&search);
        assert_eq!(search.borrow().core().get_nyes(), Nyes::Constant);
        assert_eq!(search.borrow().core().ubc_children().len(), 2);

        let result = search.value();
        assert_eq!(result.borrow().kind(), FirKind::IndepInt);
        assert_eq!(result.borrow().as_i64(), Some(42));
    }

    // ── 6. SearchFir settled no result (Econstanic) ─────────────────────────

    #[test]
    fn get_value_search_settled_no_result_returns_self() {
        // A non-anchored search with a self-rooting parent won't find an
        // enclosing brane → steps to Econstanic with no ubc_children.
        let search = make_search("^nonexistent$", false, vec![]);
        settle(&search);
        assert_eq!(search.borrow().core().get_nyes(), Nyes::Econstanic);
        assert!(search.borrow().core().ubc_children().is_empty());

        let result = search.value();
        assert!(Rc::ptr_eq(&result, &search));
    }

    // ── 7. SearchFir not settled ────────────────────────────────────────────

    #[test]
    fn get_value_search_not_settled_returns_self() {
        let search = make_search("^x$", false, vec![]);
        assert!(!search.borrow().core().get_nyes().is_constanic());

        let result = search.value();
        assert!(Rc::ptr_eq(&result, &search));
    }

    // ── 8. IndexFir settled ─────────────────────────────────────────────────

    #[test]
    fn get_value_index_settled_returns_element() {
        let val_a = make_ci(10);
        let val_b = make_ci(20);
        let val_c = make_ci(30);
        let stmt_a = make_stmt("a", 1, Rc::clone(&val_a));
        let stmt_b = make_stmt("b", 2, Rc::clone(&val_b));
        let stmt_c = make_stmt("c", 3, Rc::clone(&val_c));
        let brane = make_brn(vec![
            Rc::clone(&stmt_a),
            Rc::clone(&stmt_b),
            Rc::clone(&stmt_c),
        ]);
        let idx = make_index(1, true, vec![Rc::clone(&brane)]);
        settle(&idx);
        assert_eq!(idx.borrow().core().get_nyes(), Nyes::Constant);
        assert_eq!(idx.borrow().core().ubc_children().len(), 2);

        let result = idx.value();
        assert_eq!(result.borrow().kind(), FirKind::IndepInt);
        assert_eq!(result.borrow().as_i64(), Some(20));
    }

    // ── 9. IndexFir not settled ─────────────────────────────────────────────

    #[test]
    fn get_value_index_not_settled_returns_self() {
        let brane = make_brn(vec![]);
        let idx = make_index(0, true, vec![Rc::clone(&brane)]);
        assert!(!idx.borrow().core().get_nyes().is_constanic());

        let result = idx.value();
        assert!(Rc::ptr_eq(&result, &idx));
    }

    // ── 10. BraneFir ────────────────────────────────────────────────────────

    #[test]
    fn get_value_brane_returns_self() {
        let a = make_ci(10);
        let b = make_ci(20);
        let brane = make_brn(vec![Rc::clone(&a), Rc::clone(&b)]);
        settle(&brane);
        assert!(brane.borrow().core().get_nyes().is_constanic());
        assert!(brane.borrow().core().ubc_children().is_empty());

        let result = brane.value();
        assert!(Rc::ptr_eq(&result, &brane));
        assert_eq!(result.borrow().kind(), FirKind::Brane);
    }

    // ── 11. StatementFir ────────────────────────────────────────────────────

    #[test]
    fn get_value_statement_returns_self() {
        let body = make_ci(42);
        let stmt = make_stmt("x", 1, Rc::clone(&body));
        settle(&stmt);
        assert!(stmt.borrow().core().get_nyes().is_constanic());
        assert!(stmt.borrow().core().ubc_children().is_empty());

        let result = stmt.value();
        assert!(Rc::ptr_eq(&result, &stmt));
        assert_eq!(result.borrow().kind(), FirKind::Statement);
    }

    // ── 12. ConcatenationFir settled ────────────────────────────────────────

    #[test]
    fn get_value_concatenation_settled_returns_self() {
        let a = make_ci(1);
        let b = make_ci(2);
        let stmt_a = make_stmt("a", 1, Rc::clone(&a));
        let stmt_b = make_stmt("b", 2, Rc::clone(&b));
        let brane1 = make_brn(vec![Rc::clone(&stmt_a)]);
        let brane2 = make_brn(vec![Rc::clone(&stmt_b)]);
        let cat = make_cat(vec![Rc::clone(&brane1), Rc::clone(&brane2)]);
        settle(&cat);
        assert_eq!(cat.borrow().core().get_nyes(), Nyes::Constant);
        assert_eq!(cat.borrow().core().ubc_children().len(), 1);

        // ConcatBrane IS its own value — settled_result() returns None
        let result = cat.value();
        assert!(Rc::ptr_eq(&result, &cat));
        assert_eq!(result.borrow().kind(), FirKind::Concatenation);
        assert_eq!(result.borrow().stmt_count(), Some(2));
    }

    // ── 13. ConcatenationFir not settled ────────────────────────────────────

    #[test]
    fn get_value_concatenation_not_settled_returns_self() {
        let brane = make_brn(vec![]);
        let cat = make_cat(vec![Rc::clone(&brane)]);
        assert!(!cat.borrow().core().get_nyes().is_constanic());

        let result = cat.value();
        assert!(Rc::ptr_eq(&result, &cat));
    }

    // ── 14. StayFoolishFir ──────────────────────────────────────────────────

    #[test]
    fn get_value_stay_foolish_returns_inner_value() {
        let body = make_ci(42);
        let sf = make_sf(Rc::clone(&body));
        settle(&sf);
        assert!(sf.borrow().core().get_nyes().is_constanic());
        // SF constanic-clones the expr result into ubc_children
        assert_eq!(sf.borrow().core().ubc_children().len(), 1);

        let result = sf.value();
        assert!(!Rc::ptr_eq(&result, &sf));
        assert_eq!(result.borrow().kind(), FirKind::IndepInt);
        assert_eq!(result.borrow().as_i64(), Some(42));
    }

    // ── 15. StayFullyFoolishFir ─────────────────────────────────────────────

    #[test]
    fn get_value_stay_fully_foolish_returns_inner_value() {
        let body = make_ci(42);
        let sff = make_sff(Rc::clone(&body));
        settle(&sff);
        assert!(sff.borrow().core().get_nyes().is_constanic());
        assert_eq!(sff.borrow().core().ubc_children().len(), 1);

        let result = sff.value();
        assert!(!Rc::ptr_eq(&result, &sff));
        assert_eq!(result.borrow().as_i64(), Some(42));
    }

    #[test]
    fn get_my_statement_returns_self_if_statement() {
        let ci = make_ci(42);
        let stmt = make_stmt("x", 10, ci);
        let result = stmt.borrow()._get_my_statement(&stmt);
        assert!(Rc::ptr_eq(&result, &stmt));
    }

    #[test]
    fn get_my_statement_climbs_to_parent_statement() {
        use crate::compiler::Compiler;
        let root = Compiler::compile("{x = 1 + 2;}").unwrap().pop().unwrap();
        let stmts = root.borrow().core().foolish_children().to_vec();
        let stmt = &stmts[0];
        let body = stmt
            .borrow()
            .core()
            .foolish_children()
            .first()
            .unwrap()
            .clone();
        let result = body.borrow()._get_my_statement(&body);
        assert!(Rc::ptr_eq(&result, stmt));
    }

    #[test]
    fn get_my_brane_returns_parent_brane() {
        use crate::compiler::Compiler;
        let root = Compiler::compile("{x = 1;}").unwrap().pop().unwrap();
        let stmts = root.borrow().core().foolish_children().to_vec();
        let stmt = &stmts[0];
        let result = stmt.borrow()._get_my_brane(stmt);
        assert!(result.is_some());
        assert!(Rc::ptr_eq(&result.unwrap(), &root));
    }

    #[test]
    fn get_my_brane_climbs_through_operator() {
        use crate::compiler::Compiler;
        let root = Compiler::compile("{x = 1 + 2;}").unwrap().pop().unwrap();
        let stmts = root.borrow().core().foolish_children().to_vec();
        let stmt = &stmts[0];
        let body = stmt
            .borrow()
            .core()
            .foolish_children()
            .first()
            .unwrap()
            .clone();
        let result = body.borrow()._get_my_brane(&body);
        assert!(result.is_some());
        assert!(Rc::ptr_eq(&result.unwrap(), &root));
    }

    #[test]
    fn get_my_brane_returns_self_for_root_brane() {
        use crate::compiler::Compiler;
        let root = Compiler::compile("{x = 1;}").unwrap().pop().unwrap();
        let result = root.borrow()._get_my_brane(&root);
        assert!(result.is_none());
    }

    #[test]
    fn ib_search_finds_variable_in_same_brane() {
        use crate::compiler::Compiler;
        let root = Compiler::compile("{x = 1; y = x;}").unwrap().pop().unwrap();
        let stmts = root.borrow().core().foolish_children().to_vec();
        let y_stmt = &stmts[1]; // y = x
        let result = y_stmt.borrow()._ib_search(y_stmt, "x");
        assert!(result.is_some());
        let (stmt, _nyes) = result.unwrap();
        assert_eq!(stmt.borrow().kind(), FirKind::Statement);
        assert_eq!(stmt.borrow().as_stmt_searchable_name(), Some("x"));
    }

    #[test]
    fn ib_search_returns_none_for_missing_name() {
        use crate::compiler::Compiler;
        let root = Compiler::compile("{x = 1;}").unwrap().pop().unwrap();
        let stmts = root.borrow().core().foolish_children().to_vec();
        let x_stmt = &stmts[0];
        let result = x_stmt.borrow()._ib_search(x_stmt, "missing");
        assert!(result.is_none());
    }

    #[test]
    fn ab_search_finds_in_ancestor_brane() {
        use crate::compiler::Compiler;
        let root = Compiler::compile("{x = 1; inner = {y = x;};}")
            .unwrap()
            .pop()
            .unwrap();
        let stmts = root.borrow().core().foolish_children().to_vec();
        let inner_stmt = &stmts[1]; // inner = {y = x;}
        let inner_brane = inner_stmt
            .borrow()
            .core()
            .foolish_children()
            .first()
            .unwrap()
            .clone();
        let inner_stmts = inner_brane.borrow().core().foolish_children().to_vec();
        let y_stmt = &inner_stmts[0]; // y = x
        let result = y_stmt.borrow()._ib_search(y_stmt, "x");
        assert!(result.is_none(), "x should not be found in inner brane");
        let result = inner_brane.borrow()._ab_search(&inner_brane, "x");
        assert!(result.is_some(), "x should be found via ab_search");
        let (stmt, _nyes) = result.unwrap();
        assert_eq!(stmt.borrow().kind(), FirKind::Statement);
        assert_eq!(stmt.borrow().as_stmt_searchable_name(), Some("x"));
    }

    #[test]
    fn ab_search_returns_none_when_not_found() {
        use crate::compiler::Compiler;
        let root = Compiler::compile("{x = 1; inner = {y = x;};}")
            .unwrap()
            .pop()
            .unwrap();
        let stmts = root.borrow().core().foolish_children().to_vec();
        let inner_stmt = &stmts[1]; // inner = {y = x;}
        let inner_brane = inner_stmt
            .borrow()
            .core()
            .foolish_children()
            .first()
            .unwrap()
            .clone();
        let result = inner_brane.borrow()._ab_search(&inner_brane, "nonexistent");
        assert!(result.is_none(), "nonexistent should not be found anywhere");
    }

    /// Regression for the index-0 self-hit bug (FOOP-13): a statement that is
    /// the FIRST statement in its brane (`line_number == 0`) must not find
    /// itself via its own backward IB search. `_ib_search` computes the
    /// backward-scan end as `self.line_number.saturating_sub(1)`; for
    /// `line_number == 0` this SATURATES to `0` rather than representing "no
    /// preceding statements", so the scan range `[0, 0]` wrongly includes the
    /// statement's own slot. Left unfixed, a bare self-referential search at
    /// index 0 (e.g. `{a = a + 1;}`) recurses forever: `handle_found` clones
    /// the found statement's body — which still contains the same unresolved
    /// self-search — pushes the clone as a fresh task, the clone re-searches,
    /// finds the SAME original statement again, and clones again without
    /// bound (proved via `debug_front_task` depth tracing during triage:
    /// nesting depth climbs unboundedly, e.g. 1,2,4,5,7,9,11,13... with no
    /// terminal state).
    #[test]
    fn ib_search_at_index_zero_does_not_find_self() {
        use crate::compiler::Compiler;
        let root = Compiler::compile("{a = a + 1;}").unwrap().pop().unwrap();
        let stmts = root.borrow().core().foolish_children().to_vec();
        let a_stmt = &stmts[0]; // a = a + 1, line_number == 0
        assert_eq!(a_stmt.borrow().as_stmt_line_number(), Some(0));

        let result = a_stmt.borrow()._ib_search(a_stmt, "a");
        assert!(
            result.is_none(),
            "BUG: a statement at index 0 of its brane must not find itself \
             via backward IB search — got a hit instead of None"
        );
    }

    /// End-to-end companion to `ib_search_at_index_zero_does_not_find_self`:
    /// exercises the SAME off-by-one through the actual runtime path
    /// (`SearchFir::ib_search_with_engine`, which computes its scan end as
    /// `idx.saturating_sub(1)` and feeds `BraneNavigator::set_range` — the
    /// scope-cached twin of `_ib_search`, used by real stepping). Before the
    /// fix this program never settles (steps forever in BRANING); after the
    /// fix `a`'s self-search is correctly absent from its own brane, falls
    /// through, and the statement settles.
    #[test]
    fn statement_at_index_zero_settles_without_self_reference_hang() {
        use crate::compiler::Compiler;
        let root = Compiler::compile("{a = a + 1;}").unwrap().pop().unwrap();
        let scope = Scope::empty();

        let mut settled = false;
        for _ in 0..500 {
            if root.borrow().core().get_nyes().is_constanic() {
                settled = true;
                break;
            }
            let _ = root.step(&scope).unwrap();
        }

        assert!(
            settled,
            "BUG: {{a = a + 1;}} must settle (a's self-search absent, falls \
             through to unanchored-miss ECONSTANIC per FOOP-43 / or NK if no \
             fallback exists) — it must NOT hang forever due to a's own \
             statement finding itself at index 0"
        );
    }
}
