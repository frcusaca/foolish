//! ProtoBrane — the shared field-holder for every UBCa FIR node.
//!
//! All topology code lives as inherent methods on this struct (written once,
//! not overridable). Every FIR kind contains one of these as a field named
//! `core`.

use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::rc::{Rc, Weak};

use foolish_core::fir::Nyes;

use crate::fir_trait::{Fir, FirRef};

/// Shared field-holder. Every FIR kind contains one of these as `core`.
///
/// Stores children in two provenance-separated stores:
/// - `foolish_children`: parse-derived, fixed-shape (immutable topology)
/// - `ubc_children`: computation-derived, mutable (search results, etc.)
///
/// Plus evaluation state (`nyes`), a task queue (`tasks`), and a parent
/// back-link (`parent`).
///
/// Interior mutability: `nyes`, `tasks`, and `ubc_children` use `Cell`/`RefCell`
/// so that `fir_op_step` can take `&self`. This allows `step_fir_ref_inner` to
/// hold a *shared* borrow during stepping — multiple shared borrows are
/// permitted, so walking the parent chain and borrowing siblings (which may be
/// the node being stepped) does not panic.
pub struct ProtoBrane {
    /// Parse-time children. FIXED: the vector never grows or shrinks and no Rc
    /// slot is ever re-seated once built. The referenced FIR structs DO step
    /// and compute in place (interior evolution), but the topology is frozen.
    foolish_children: Vec<FirRef>,

    /// Compute-time children. FIR created during evaluation — search results,
    /// operator results, concatenation result branes. May expand and shrink.
    /// Order is significant (snapshot-visible).
    ubc_children: RefCell<Vec<FirRef>>,

    /// Evaluation state (NYES) of THIS node. Single source of truth.
    /// Uses `Cell` for interior mutability — `fir_op_step` takes `&self`.
    nyes: Cell<Nyes>,

    /// Task list for NYES-driven stepping. Children to drain to constanic;
    /// the node's own `fir_op_step` runs when it empties.
    tasks: RefCell<VecDeque<FirRef>>,

    /// Weak back-link to the parent FIR node. NON-optional: the root node's
    /// parent is a Weak pointing at itself (detected via `is_root()`).
    /// IMMUTABLE after construction.
    parent: Weak<RefCell<dyn Fir>>,
}

impl ProtoBrane {
    /// Create a new ProtoBrane.
    ///
    /// `parent` is the Weak back-link to the parent node. For root nodes,
    /// pass a self-Weak (constructed via `Rc::new_cyclic`).
    pub fn new(foolish_children: Vec<FirRef>, parent: Weak<RefCell<dyn Fir>>, nyes: Nyes) -> Self {
        Self {
            foolish_children,
            ubc_children: RefCell::new(Vec::new()),
            nyes: Cell::new(nyes),
            tasks: RefCell::new(VecDeque::new()),
            parent,
        }
    }

    // --- read-only child access (no RefMut guards returned) ---

    /// Parse-time children (immutable topology).
    pub fn foolish_children(&self) -> &[FirRef] {
        &self.foolish_children
    }

    /// Compute-time children (mutable during evaluation).
    /// Returns a cloned Vec — callers store it in a local before indexing.
    pub fn ubc_children(&self) -> Vec<FirRef> {
        self.ubc_children.borrow().clone()
    }

    /// All children in render order: ubc first (result=), then foolish.
    pub fn all_children(&self) -> Vec<FirRef> {
        let mut all = self.ubc_children.borrow().clone();
        all.extend(self.foolish_children.iter().cloned());
        all
    }

    // --- ubc mutation (the ONLY public topology mutators) ---

    /// Push a compute-time child.
    pub fn push_ubc_child(&self, child: FirRef) {
        self.ubc_children.borrow_mut().push(child);
    }

    /// Clear all compute-time children.
    pub fn clear_ubc_children(&self) {
        self.ubc_children.borrow_mut().clear();
    }

    // --- task queue (stepping internals) ---

    /// Peek at the front task without removing it.
    pub(crate) fn front_task(&self) -> Option<FirRef> {
        self.tasks.borrow().front().cloned()
    }

    /// Pop the front task (called when it reaches a settled state).
    pub(crate) fn pop_front_task(&self) {
        self.tasks.borrow_mut().pop_front();
    }

    /// Push a task to the back of the queue.
    #[allow(dead_code)]
    pub(crate) fn push_task(&self, t: FirRef) {
        self.tasks.borrow_mut().push_back(t);
    }

    // --- parent / root ---

    /// Upgrade the parent Weak. Returns None only during teardown.
    pub fn parent(&self) -> Option<FirRef> {
        self.parent.upgrade()
    }

    /// Root iff parent upgrades to a node that is pointer-equal to self.
    /// Caller passes `self_rc` because ProtoBrane has no back-pointer.
    pub fn is_root(&self, self_rc: &FirRef) -> bool {
        self.parent
            .upgrade()
            .is_some_and(|p| Rc::ptr_eq(&p, self_rc))
    }

    // --- nyes (interior mutability via Cell) ---

    /// Get the current NYES state.
    pub fn get_nyes(&self) -> Nyes {
        self.nyes.get()
    }

    /// Set the NYES state. Only `fir_op_step` and the builder call this.
    #[allow(dead_code)]
    pub(crate) fn set_nyes(&self, n: Nyes) {
        self.nyes.set(n);
    }
}

impl std::fmt::Debug for ProtoBrane {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProtoBrane")
            .field("foolish_children", &self.foolish_children.len())
            .field("ubc_children", &self.ubc_children.borrow().len())
            .field("nyes", &self.nyes.get())
            .field("tasks", &self.tasks.borrow().len())
            .field(
                "parent",
                &if self.parent.upgrade().is_some() {
                    "Some(…)"
                } else {
                    "None (dropped)"
                },
            )
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fir_trait::tests::{make_leaf, make_root_brane};

    #[test]
    fn proto_brane_basic_accessors() {
        let child = make_leaf(Nyes::Constant);
        let root = make_root_brane(vec![Rc::clone(&child)]);

        let borrowed = root.borrow();
        let core = borrowed.core();
        assert_eq!(core.foolish_children().len(), 1);
        assert_eq!(core.ubc_children().len(), 0);
        assert_eq!(core.all_children().len(), 1);
        assert_eq!(core.get_nyes(), Nyes::Prembrionic);
    }

    #[test]
    fn proto_brane_ubc_children_mutation() {
        let root = make_root_brane(vec![]);
        let extra = make_leaf(Nyes::Constant);

        root.borrow().core().push_ubc_child(Rc::clone(&extra));
        assert_eq!(root.borrow().core().ubc_children().len(), 1);
        assert_eq!(root.borrow().core().all_children().len(), 1);

        root.borrow().core().clear_ubc_children();
        assert_eq!(root.borrow().core().ubc_children().len(), 0);
    }

    #[test]
    fn proto_brane_is_root() {
        let root = make_root_brane(vec![]);
        assert!(root.borrow().core().is_root(&root));

        let child = make_leaf(Nyes::Constant);
        // make_leaf is self-rooting, so it's root of itself
        assert!(child.borrow().core().is_root(&child));
        // But it's NOT root of the actual root brane
        assert!(!child.borrow().core().is_root(&root));
    }

    #[test]
    fn proto_brane_parent_link() {
        let root = make_root_brane(vec![]);
        // Root's parent is a self-Weak
        let parent = root.borrow().core().parent().unwrap();
        assert!(Rc::ptr_eq(&parent, &root));
    }

    #[test]
    fn proto_brane_nyes_contract() {
        let root = make_root_brane(vec![]);
        assert_eq!(root.borrow().core().get_nyes(), Nyes::Prembrionic);

        root.borrow().core().set_nyes(Nyes::Braning);
        assert_eq!(root.borrow().core().get_nyes(), Nyes::Braning);
    }
}
