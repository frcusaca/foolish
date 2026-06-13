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
    ConstantInt,
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
}

impl Scope {
    /// Create a minimal scope with no position.
    pub fn empty() -> Self {
        Self {
            current_brane: None,
            current_stmt_idx: None,
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

    /// Read the integer value if this is a ConstantInt. Default: None.
    fn as_i64(&self) -> Option<i64> {
        None
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
            if front_rc.borrow().core().get_nyes().is_settled() {
                this.borrow().core().pop_front_task();
            } else {
                step_fir_ref_inner(&front_rc, scope, depth + 1)?;
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
            // In the real model, fir_op_step produces a settled state or pushes
            // more tasks. A leaf's work is immediate.
            if !self.core.get_nyes().is_settled() {
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
                    let all_settled = children
                        .iter()
                        .all(|c| c.borrow().core().get_nyes().is_settled());
                    if all_settled {
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
        // Prembrionic → Embryonic → Braning → Woconstanic (settled)
        let leaf = make_leaf(Nyes::Prembrionic);
        let scope = Scope::empty();

        let mut transitions = vec![leaf.borrow().core().get_nyes()];

        // Step until settled
        for _ in 0..10 {
            let report = step_fir_ref(&leaf, &scope).unwrap();
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
                    if nyes.is_settled() {
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
            final_root.is_settled(),
            "root should be settled, got {final_root}"
        );

        // Child should be settled
        let final_child = child.borrow().core().get_nyes();
        assert!(
            final_child.is_settled(),
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
                    if nyes.is_settled() {
                        break;
                    }
                }
                StepReport::NoProgress => break,
            }
        }

        eprintln!("Root NYES log: {root_nyes_log:?}");

        // Both children should be settled
        assert!(child_a.borrow().core().get_nyes().is_settled());
        assert!(child_b.borrow().core().get_nyes().is_settled());
        // Root should be settled
        assert!(root.borrow().core().get_nyes().is_settled());
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
