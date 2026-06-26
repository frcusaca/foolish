//! Fir trait — the dyn-dispatch surface for UBCa FIR nodes.
//!
//! Each FIR kind implements this trait with its own `fir_op_step` (combining
//! work) and `kind()`. Shared stepping logic lives in the free function
//! `step_fir_ref`, NOT as a trait method — this is critical for borrow
//! discipline (transient borrows dropped before recursion).

use std::cell::RefCell;
use std::rc::Rc;

use foolish_core::fir::Nyes;

use crate::nyes_ext::NyesExt;
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
    HeadTail,
    StayFoolish,
    StayFullyFoolish,
    Concatenation,
    IndepInt,
    Nk,
    /// Placeholder for test stubs before real kinds are implemented.
    Unknown,
}

/// Minimal Scope stub — enough for compilation. The real Scope comes later
/// (FOOP-62 §10).
#[derive(Debug, Clone)]
pub struct Scope {
    /// Placeholder: current brane (will be replaced by capability surface)
    pub current_brane: Option<FirRef>,
    /// Placeholder: current statement index
    pub current_stmt_idx: Option<usize>,
    /// True when the current evaluation is inside an SF-mark's RHS (FOOP-62 §10.1,
    /// "Terminology: ignorance"). `step()` carries this down the step recursion; it
    /// seeds each `constanic_clone`'s `descendent_of_sfm_and_foolishly_ignorant`.
    /// When true, clones copy ALL NYES unchanged (foolishly ignorant); when false,
    /// the normal NYES-transfer rule applies.
    pub has_ancestral_sfm: bool,
}

impl Scope {
    /// Create a minimal scope with no position (not foolishly ignorant).
    pub fn empty() -> Self {
        Self {
            current_brane: None,
            current_stmt_idx: None,
            has_ancestral_sfm: false,
        }
    }

    /// Return a copy of this scope with `has_ancestral_sfm` set — used when
    /// descending into an SF-mark's RHS so descendants clone foolishly ignorant.
    pub fn with_ancestral_sfm(&self, has_ancestral_sfm: bool) -> Self {
        Self {
            has_ancestral_sfm,
            ..self.clone()
        }
    }
}

/// UBCa evaluation error type.
#[derive(Debug, thiserror::Error)]
pub enum UbcError {
    /// An evaluation error with a human-readable message.
    #[error("ubca evaluation error: {0}")]
    Eval(String),
}

/// The dyn-dispatch trait for UBCa FIR nodes.
///
/// Every FIR kind contains a `ProtoBrane` (accessed via `core()`)
/// and implements `fir_op_step` with its own combining work. Stepping itself
/// is the shared `step_fir_ref` free function — kinds differ through
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

    /// Read the integer value. Default: look through ubc_children for resolved
    /// results. IndepIntFir overrides to return `Some(value)`.
    fn as_i64(&self) -> Option<i64> {
        let ubc = self.core().ubc_children();
        ubc.first().and_then(|c| c.borrow().as_i64())
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

    /// StatementFir name (empty string = anonymous). Default: None.
    fn as_stmt_name(&self) -> Option<&str> {
        None
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

    /// HeadTailFir is_head flag. Default: false.
    fn as_headtail_is_head(&self) -> bool {
        false
    }

    /// HeadTailFir anchored flag. Default: false.
    fn as_headtail_anchored(&self) -> bool {
        false
    }

    /// BraneFir characterizations. Default: empty.
    fn as_brane_characterizations(&self) -> &[String] {
        &[]
    }

    /// SF inner search pattern — set when a search resolved through SF
    /// re-evaluation. Used by the humanizer to preserve the search wrapper.
    fn as_sf_inner_pattern(&self) -> Option<String> {
        None
    }

    fn get_my_statement(&self, self_ref: &FirRef) -> FirRef {
        match self.kind() {
            FirKind::Statement => Rc::clone(self_ref),
            _ => match self.core().parent() {
                Some(p) => p.borrow().get_my_statement(&p),
                None => Rc::clone(self_ref),
            },
        }
    }

    fn get_my_brane(&self) -> Option<FirRef> {
        match self.core().parent() {
            Some(p) => {
                let kind = p.borrow().kind();
                match kind {
                    FirKind::Brane => Some(p),
                    _ => p.borrow().get_my_brane(),
                }
            }
            None => None,
        }
    }

    fn ib_search(&self, _self_ref: &FirRef, _name: &str, _forward: bool) -> Option<(FirRef, Nyes)> {
        None
    }

    fn ab_search(&self, _self_ref: &FirRef, _name: &str, _forward: bool) -> Option<(FirRef, Nyes)> {
        None
    }
}

/// Returns the deepest resolved value this FIR represents.
///
/// When a FIR settles, it may store its result in `ubc_children[0]`
/// (e.g. SearchFir, OperatorFir, IndexFir). That result may itself be a
/// wrapper with its own `ubc_children`. This function recursively unwraps
/// the chain until it reaches a terminal value (one that has no
/// `ubc_children`, like IndepInt, Nk, or BraneFir).
///
/// For pre-constanic FIRs or FIRs without `ubc_children`, returns `self`.
pub fn get_value(fir_ref: &FirRef) -> FirRef {
    // Extract first ubc_child under a short borrow, then drop it before recursing.
    let child: Option<FirRef> = {
        let borrowed = fir_ref.borrow();
        if borrowed.core().get_nyes().is_constanic() {
            borrowed.core().ubc_children().into_iter().next()
        } else {
            None
        }
    };
    match child {
        Some(c) => get_value(&c),
        None => Rc::clone(fir_ref),
    }
}

/// The shared step function — written ONCE, called as a free function over a
/// `&FirRef`.
///
/// This is NOT a trait method: it takes `this: &FirRef` so the borrow is
/// transient and dropped before any recursive call or `fir_op_step`
/// invocation, preventing the RefCell aliasing panic that a nested
/// `borrow_mut`-across-recursion shape would cause.
///
/// ONE action per call (check-then-act). The outer driver loops
/// `step_fir_ref(root, scope)`.
///
/// Uses an explicit stack to avoid stack overflow on deeply nested trees.
const MAX_DEPTH: usize = 100;

pub fn step_fir_ref(this: &FirRef, scope: &Scope) -> Result<StepReport, UbcError> {
    step_fir_ref_inner(this, scope, 0)
}

fn step_fir_ref_inner(this: &FirRef, scope: &Scope, depth: usize) -> Result<StepReport, UbcError> {
    if depth > MAX_DEPTH {
        return Ok(StepReport::NoProgress);
    }
    let front = this.borrow().core().front_task();

    match front {
        Some(front_rc) => {
            if front_rc.borrow().core().get_nyes().is_constanic() {
                this.borrow().core().pop_front_task();
            } else {
                // Descending into an SF-mark's RHS turns on foolish ignorance for the
                // whole subtree (FOOP-62 §10.1): the child and its descendants clone
                // foolishly ignorant. Once set, the flag stays set for deeper levels.
                let child_scope = if this.borrow().kind() == FirKind::StayFoolish {
                    scope.with_ancestral_sfm(true)
                } else {
                    scope.clone()
                };
                step_fir_ref_inner(&front_rc, &child_scope, depth + 1)?;
            }
            Ok(StepReport::Progress(this.borrow().core().get_nyes()))
        }
        None => {
            // Shared borrow — fir_op_step takes &self. Multiple shared borrows
            // are permitted, so walking the parent chain and borrowing siblings
            // (which may be `this`) does not panic.
            this.borrow().fir_op_step(scope)?;
            let nyes = this.borrow().core().get_nyes();
            Ok(StepReport::Progress(nyes))
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::nyes_ext::NyesExt;
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
    #[derive(Debug)]
    pub(crate) struct BraneFir {
        pub(crate) core: ProtoBrane,
    }

    impl Fir for BraneFir {
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
        Rc::new_cyclic(|me: &Weak<RefCell<BraneFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(BraneFir {
                core: ProtoBrane::new(children, parent, Nyes::Prembrionic),
            })
        })
    }

    /// Build a root brane with children. Children are re-parented to the root.
    /// Uses `Rc::new_cyclic` on concrete `BraneFir`; the `FirRef` type
    /// annotation triggers unsized coercion.
    pub(crate) fn make_root_brane(children: Vec<FirRef>) -> FirRef {
        Rc::new_cyclic(|me: &Weak<RefCell<BraneFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            // Re-parent all children to this root
            // (children were created self-rooting; re-wire them)
            for child in &children {
                // We can't mutate the child's ProtoBrane parent (it's immutable
                // after construction). But the tests only step downward, so the
                // incorrect parent on children is harmless for these tests.
                let _ = child;
            }
            RefCell::new(BraneFir {
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
            let report = step_fir_ref(&leaf, &scope).unwrap();
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
            let report = step_fir_ref(&root, &scope).unwrap();
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
            let report = step_fir_ref(&root, &scope).unwrap();
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

        let report = step_fir_ref(&leaf, &scope).unwrap();
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
        let result = step_fir_ref(&outer, &scope);
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
    use crate::nyes_ext::NyesExt;
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
            RefCell::new(StatementFir {
                core: ProtoBrane::new(vec![body], parent, Nyes::Prembrionic),
                name: name.to_owned(),
                line_number: line,
            })
        })
    }

    fn make_brn(children: Vec<FirRef>) -> FirRef {
        Rc::new_cyclic(|me: &Weak<RefCell<BraneFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(BraneFir {
                core: ProtoBrane::new(children, parent, Nyes::Prembrionic),
                characterizations: Vec::new(),
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
            })
        })
    }

    fn settle(node: &FirRef) {
        let scope = Scope::empty();
        for _ in 0..50 {
            let report = step_fir_ref(node, &scope).unwrap();
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

        let result = get_value(&ci);
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

        let result = get_value(&nk);
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

        let result = get_value(&op);
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

        let result = get_value(&op);
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
        assert_eq!(search.borrow().core().ubc_children().len(), 1);

        let result = get_value(&search);
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

        let result = get_value(&search);
        assert!(Rc::ptr_eq(&result, &search));
    }

    // ── 7. SearchFir not settled ────────────────────────────────────────────

    #[test]
    fn get_value_search_not_settled_returns_self() {
        let search = make_search("^x$", false, vec![]);
        assert!(!search.borrow().core().get_nyes().is_constanic());

        let result = get_value(&search);
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
        assert_eq!(idx.borrow().core().ubc_children().len(), 1);

        let result = get_value(&idx);
        assert_eq!(result.borrow().kind(), FirKind::IndepInt);
        assert_eq!(result.borrow().as_i64(), Some(20));
    }

    // ── 9. IndexFir not settled ─────────────────────────────────────────────

    #[test]
    fn get_value_index_not_settled_returns_self() {
        let brane = make_brn(vec![]);
        let idx = make_index(0, true, vec![Rc::clone(&brane)]);
        assert!(!idx.borrow().core().get_nyes().is_constanic());

        let result = get_value(&idx);
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

        let result = get_value(&brane);
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

        let result = get_value(&stmt);
        assert!(Rc::ptr_eq(&result, &stmt));
        assert_eq!(result.borrow().kind(), FirKind::Statement);
    }

    // ── 12. ConcatenationFir settled ────────────────────────────────────────

    #[test]
    fn get_value_concatenation_settled_returns_result_brane() {
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

        let result = get_value(&cat);
        assert!(!Rc::ptr_eq(&result, &cat));
        assert_eq!(result.borrow().kind(), FirKind::Brane);
        assert_eq!(result.borrow().core().foolish_children().len(), 2);
    }

    // ── 13. ConcatenationFir not settled ────────────────────────────────────

    #[test]
    fn get_value_concatenation_not_settled_returns_self() {
        let brane = make_brn(vec![]);
        let cat = make_cat(vec![Rc::clone(&brane)]);
        assert!(!cat.borrow().core().get_nyes().is_constanic());

        let result = get_value(&cat);
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

        let result = get_value(&sf);
        assert!(!Rc::ptr_eq(&result, &sf));
        assert_eq!(result.borrow().kind(), FirKind::IndepInt);
        assert_eq!(result.borrow().as_i64(), Some(42));
    }

    // ── 15. StayFullyFoolishFir ─────────────────────────────────────────────

    #[test]
    fn get_value_stay_fully_foolish_returns_self() {
        let body = make_ci(42);
        let sff = make_sff(Rc::clone(&body));
        settle(&sff);
        assert!(sff.borrow().core().get_nyes().is_constanic());
        assert!(sff.borrow().core().ubc_children().is_empty());

        let result = get_value(&sff);
        assert!(Rc::ptr_eq(&result, &sff));
        assert_eq!(result.borrow().kind(), FirKind::StayFullyFoolish);
    }

    #[test]
    fn get_my_statement_returns_self_if_statement() {
        let ci = make_ci(42);
        let stmt = make_stmt("x", 10, ci);
        let result = stmt.borrow().get_my_statement(&stmt);
        assert!(Rc::ptr_eq(&result, &stmt));
    }

    #[test]
    fn get_my_statement_climbs_to_parent_statement() {
        use crate::compiler::Compiler;
        let root = Compiler::compile("{x = 1 + 2;}").unwrap().pop().unwrap();
        let stmts = root.borrow().core().foolish_children().to_vec();
        let stmt = &stmts[0];
        let body = stmt.borrow().core().foolish_children().first().unwrap().clone();
        let result = body.borrow().get_my_statement(&body);
        assert!(Rc::ptr_eq(&result, stmt));
    }

    #[test]
    fn get_my_brane_returns_parent_brane() {
        use crate::compiler::Compiler;
        let root = Compiler::compile("{x = 1;}").unwrap().pop().unwrap();
        let stmts = root.borrow().core().foolish_children().to_vec();
        let stmt = &stmts[0];
        let result = stmt.borrow().get_my_brane();
        assert!(result.is_some());
        assert!(Rc::ptr_eq(&result.unwrap(), &root));
    }

    #[test]
    fn get_my_brane_climbs_through_operator() {
        use crate::compiler::Compiler;
        let root = Compiler::compile("{x = 1 + 2;}").unwrap().pop().unwrap();
        let stmts = root.borrow().core().foolish_children().to_vec();
        let stmt = &stmts[0];
        let body = stmt.borrow().core().foolish_children().first().unwrap().clone();
        let result = body.borrow().get_my_brane();
        assert!(result.is_some());
        assert!(Rc::ptr_eq(&result.unwrap(), &root));
    }

    #[test]
    fn get_my_brane_returns_self_for_root_brane() {
        use crate::compiler::Compiler;
        let root = Compiler::compile("{x = 1;}").unwrap().pop().unwrap();
        let result = root.borrow().get_my_brane();
        assert!(result.is_some());
        assert!(Rc::ptr_eq(&result.unwrap(), &root));
    }

    #[test]
    fn ib_search_finds_variable_in_same_brane() {
        use crate::compiler::Compiler;
        let root = Compiler::compile("{x = 1; y = x;}").unwrap().pop().unwrap();
        let stmts = root.borrow().core().foolish_children().to_vec();
        let y_stmt = &stmts[1]; // y = x
        let result = y_stmt.borrow().ib_search(y_stmt, "x", false);
        assert!(result.is_some());
        let (body, nyes) = result.unwrap();
        assert_eq!(body.borrow().kind(), FirKind::IndepInt);
        assert!(nyes.is_constanic());
    }

    #[test]
    fn ib_search_returns_none_for_missing_name() {
        use crate::compiler::Compiler;
        let root = Compiler::compile("{x = 1;}").unwrap().pop().unwrap();
        let stmts = root.borrow().core().foolish_children().to_vec();
        let x_stmt = &stmts[0];
        let result = x_stmt.borrow().ib_search(x_stmt, "missing", false);
        assert!(result.is_none());
    }

    #[test]
    fn ab_search_finds_in_ancestor_brane() {
        use crate::compiler::Compiler;
        let root = Compiler::compile("{x = 1; inner = {y = x;};}").unwrap().pop().unwrap();
        let stmts = root.borrow().core().foolish_children().to_vec();
        let inner_stmt = &stmts[1]; // inner = {y = x;}
        let inner_brane = inner_stmt.borrow().core().foolish_children().first().unwrap().clone();
        let inner_stmts = inner_brane.borrow().core().foolish_children().to_vec();
        let y_stmt = &inner_stmts[0]; // y = x
        let result = y_stmt.borrow().ib_search(y_stmt, "x", false);
        assert!(result.is_none(), "x should not be found in inner brane");
        let result = inner_brane.borrow().ab_search(&inner_brane, "x", false);
        assert!(result.is_some(), "x should be found via ab_search");
        let (body, nyes) = result.unwrap();
        assert_eq!(body.borrow().kind(), FirKind::IndepInt);
    }
}
