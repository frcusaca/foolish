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
    foolish_children: Vec<FirRef>,
    ubc_children: RefCell<Vec<FirRef>>,
    nyes: Cell<Nyes>,
    tasks: RefCell<VecDeque<FirRef>>,
    parent: Weak<RefCell<dyn Fir>>,
    alarm_reason: RefCell<Option<String>>,
}

impl ProtoBrane {
    /// Create a new ProtoBrane.
    ///
    /// `parent` is the Weak back-link to the parent node. For root nodes,
    /// pass a self-Weak (constructed via `Rc::new_cyclic`).
    pub fn new(foolish_children: Vec<FirRef>, parent: Weak<RefCell<dyn Fir>>, nyes: Nyes) -> Self {
        let mut proto = Self {
            foolish_children: Vec::with_capacity(foolish_children.len()),
            ubc_children: RefCell::new(Vec::new()),
            nyes: Cell::new(nyes),
            tasks: RefCell::new(VecDeque::new()),
            parent,
            alarm_reason: RefCell::new(None),
        };
        // Route the initial children through the one setter, so any
        // per-child handling added there applies uniformly whether a caller
        // passes a full Vec up front or pushes incrementally.
        for child in foolish_children {
            proto.push_foolish_child(child);
        }
        proto
    }

    pub fn set_alarm_reason(&self, reason: String) {
        *self.alarm_reason.borrow_mut() = Some(reason);
    }

    pub fn alarm_reason(&self) -> Option<String> {
        self.alarm_reason.borrow().clone()
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

    /// All children in render order: ubc first (evaluator renders as `result=`),
    /// then foolish.
    pub fn all_children(&self) -> Vec<FirRef> {
        let mut all = self.ubc_children.borrow().clone();
        all.extend(self.foolish_children.iter().cloned());
        all
    }

    // --- foolish mutation (construction-time only) ---

    /// Push a parse-time child onto the foolish store.
    ///
    /// `&mut self` by design: `foolish_children` is a plain `Vec` (no interior
    /// mutability), so this is reachable ONLY while the `ProtoBrane` is still
    /// owned — i.e. during construction, before the FIR is wrapped in
    /// `Rc<RefCell<…>>` and goes live. That preserves the "immutable topology"
    /// promise above: the store may be *built up*, but is fixed once stepping
    /// can observe it.
    ///
    /// Prefer this over passing a fully-formed `Vec` to [`ProtoBrane::new`] when
    /// the caller needs to act on each child as it is added — e.g. Rust-side
    /// construction that must mark children before they are enqueued. (Building
    /// from parsed Foolish source does not need this: `compiler::build_fir`'s
    /// `under_sff` flag already sets descendant searches to ECONSTANIC
    /// recursively at construction, which is the correct rule for that path.)
    pub fn push_foolish_child(&mut self, child: FirRef) {
        self.foolish_children.push(child);
    }

    /// Push a parse-time child under an SF/SFF marker, sanity-checking that the
    /// SFF construction rule already took effect.
    ///
    /// `compiler::build_fir`'s `under_sff` flag sets descendant SEARCH FIRs to
    /// ECONSTANIC *recursively* at construction, so that an SFF body is
    /// constanic-unevaluated and its searches never run. That rule is applied
    /// on the way *down* (during `build_fir`), well before the finished child
    /// reaches this push. This method verifies the outcome on the way *back
    /// up*: every search-like descendant of `child` must already be constanic.
    ///
    /// **Panics** (unconditionally — not a `debug_assert!`, so it fires in
    /// release too) on violation. A violation means the FVM's own construction
    /// is internally inconsistent: an SFF body carrying a search that can still
    /// run. Stepping it would evaluate a body that is supposed to be
    /// constanic-unevaluated, silently producing wrong results, so halting is
    /// the correct failure mode. This matches how the codebase already treats
    /// broken internal invariants (`_decide_nyes_due_to_children`'s
    /// `unreachable!`, the pre-constanic-search-candidate guard in
    /// `fir_kinds.rs`).
    ///
    /// The panic message names the same condition as
    /// [`UbcError::InternalConsistency`] — "the interpreter is broken", as
    /// distinct from "this program is unevaluable".
    pub fn push_foolish_child_sff_marked(&mut self, child: FirRef) {
        if let Some(offender) = Self::sift_for_first_non_econstanic_descendent_search(&child) {
            let (kind, nyes) = {
                let b = offender.borrow();
                (b.kind(), b.core().get_nyes())
            };
            panic!(
                "ubca INTERNAL CONSISTENCY error: SFF-marked child has a \
                 descendant {kind:?} search at {nyes:?}, expected ECONSTANIC. \
                 The `under_sff` construction rule (compiler::build_fir) did \
                 not reach it. An SFF body must be constanic-unevaluated — \
                 every descendant search kind must be built ECONSTANIC so it \
                 never runs. Refusing to continue: stepping this body would \
                 evaluate a search that must not run."
            );
        }
        self.push_foolish_child(child);
    }

    /// The first descendant search kind (see [`Fir::is_search_kind`] — the
    /// kinds `build_fir`'s `under_sff` rule targets) that is **not** exactly
    /// `Nyes::Econstanic`, or `None` if every one of them is.
    ///
    /// Returns the offending node (not just a bool) so the caller's panic can
    /// name its kind and actual NYES.
    ///
    /// **Naming**: `sift_*`, not `search_*`. In this codebase "search" means
    /// the *Foolish language* feature (`?`/`~`/`.`/`#`, `SearchFir`, the
    /// `ContextfulSearch` engine and its NYES rules). A `sift_*` function is
    /// an ordinary Rust-side walk over the FIR tree with no Foolish search
    /// semantics — no anchoring, no NYES effects, no ECONSTANIC/NK outcome.
    /// Keeping the prefixes distinct prevents reading interpreter plumbing as
    /// language behaviour.
    ///
    /// Deliberately checks `== Econstanic`, not `is_constanic()`: the SFF rule
    /// is that these searches are *built* ECONSTANIC and never run. A search
    /// sitting at CONSTANT or NK under an SFF marker would mean it *did* run
    /// (or was mis-constructed) — exactly what this guard exists to catch.
    ///
    /// Walks the foolish store only — the parse-time topology, which is all
    /// that exists at construction time (`ubc_children` is empty until
    /// stepping begins).
    fn sift_for_first_non_econstanic_descendent_search(node: &FirRef) -> Option<FirRef> {
        let borrowed = node.borrow();
        if borrowed.is_search_kind() && borrowed.core().get_nyes() != Nyes::Econstanic {
            return Some(Rc::clone(node));
        }
        let children: Vec<FirRef> = borrowed.core().foolish_children().to_vec();
        drop(borrowed);
        children
            .iter()
            .find_map(Self::sift_for_first_non_econstanic_descendent_search)
    }

    /// Read a single parse-time child by index.
    ///
    /// Convenience over `foolish_children()[i]` for callers that want a cloned
    /// handle without holding a borrow of the slice across other borrows.
    pub fn get_foolish_child(&self, index: usize) -> Option<FirRef> {
        self.foolish_children.get(index).cloned()
    }

    // --- ubc mutation (the ONLY public topology mutators) ---

    /// Push a compute-time child AND enqueue it as a task (FIFO).
    /// New additions always go to the end of the job queue.
    pub fn push_ubc_child(&self, child: FirRef) {
        self.ubc_children.borrow_mut().push(Rc::clone(&child));
        if !child.borrow().core().get_nyes().is_constanic() {
            self.tasks.borrow_mut().push_back(child);
        }
    }

    /// Push the SINGULAR result of a search FIR (search / Index).
    ///
    /// SINGULAR-RESULT INVARIANT (FOOP-62): every search FIR we currently implement produces
    /// at most ONE result, so its `ubc_children` holds at most one entry.
    /// [`Fir::settled_result`] reads this single entry as the resolved value.
    /// (Multi-result searches are a future extension that will hold more children.)
    /// This is the result-pushing path for search kinds; it runtime-verifies the invariant.
    pub fn push_search_result(&self, result: FirRef) {
        debug_assert!(
            self.ubc_children.borrow().is_empty(),
            "search FIR already has a result; existing searches are singular-result \
             (ubc_children must be <= 1)"
        );
        self.push_ubc_child(result);
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

    /// Pop the front task (called when it reaches a constanic state).
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

    /// Clone the parent Weak. Used by fir_op_step for constanic-clone
    /// of results pushed to ubc_children.
    pub fn parent_weak(&self) -> Weak<RefCell<dyn Fir>> {
        self.parent.clone()
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

    /// Set the NYES state.
    ///
    /// OWNERSHIP CONTRACT (FOOP-62 #10): a FIR owns its own nyes — nyes must NOT be changed
    /// from outside the FIR. The ONLY sanctioned writers are:
    ///
    /// 1. a FIR on ITSELF, inside its own `fir_op_step` (`self.core.set_nyes(...)`); and
    /// 2. construction — `ProtoBrane::new(.., nyes)` (builders, and the constanic-clone path,
    ///    which legitimately sets the clone's nyes at construction via `Nyes::transform_for_clone`).
    ///
    /// No code may call `set_nyes` on a node it does not own. (`pub(crate)` is the tightest the
    /// type system allows here since each FIR kind is a sibling type reaching its own `core`.)
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
    #[should_panic(expected = "INTERNAL CONSISTENCY error")]
    fn push_foolish_child_sff_marked_rejects_unmarked_descendant_search() {
        // Proves the guard is not vacuous: an SFF body whose descendant search
        // was NOT built ECONSTANIC (i.e. the `under_sff` rule failed to reach
        // it) must trip the debug_assert rather than being silently stored.
        use crate::compiler::Compiler;

        // Build `{x = 1; y = x;}` WITHOUT any SFF marker, so `y`'s search is
        // Prembrionic — exactly the mis-constructed shape the guard catches.
        let root = Compiler::compile("{x = 1; y = x;}").unwrap().pop().unwrap();
        let parent = root.borrow().core().parent_weak();
        let mut core = ProtoBrane::new(vec![], parent, Nyes::Prembrionic);
        core.push_foolish_child_sff_marked(root);
    }

    #[test]
    #[should_panic(expected = "INTERNAL CONSISTENCY error")]
    fn push_foolish_child_sff_marked_rejects_a_constant_descendant_search() {
        // "All else should fail, INCLUDING constanic searches." A CONSTANT
        // search is the least obvious violation — it *looks* fine (settled,
        // constanic) — but under an SFF marker it means the search actually
        // RAN, which is precisely what SFF forbids. Only ECONSTANIC passes.
        use crate::compiler::Compiler;

        let root = Compiler::compile("{x = 1; y = x;}").unwrap().pop().unwrap();
        // Force every descendant search to CONSTANT, simulating "it ran".
        fn force_searches_constant(node: &FirRef) {
            let b = node.borrow();
            if b.is_search_kind() {
                b.core().set_nyes(Nyes::Constant);
            }
            let kids: Vec<FirRef> = b.core().foolish_children().to_vec();
            drop(b);
            for k in &kids {
                force_searches_constant(k);
            }
        }
        force_searches_constant(&root);

        let parent = root.borrow().core().parent_weak();
        let mut core = ProtoBrane::new(vec![], parent, Nyes::Prembrionic);
        core.push_foolish_child_sff_marked(root);
    }

    #[test]
    fn push_foolish_child_sff_marked_accepts_a_properly_marked_body() {
        // The positive case: a real `<<…>>` body, built through the compiler's
        // `under_sff` path, passes the guard. (Compiling this at all already
        // exercises the guard via build_fir's SFF arm; asserting the shape here
        // documents why it passes.)
        use crate::compiler::Compiler;

        let root = Compiler::compile("{a = <<x>>;}").unwrap().pop().unwrap();
        // Walk to the SFF body's descendant search and confirm it is ECONSTANIC.
        fn any_search_is_econstanic(node: &FirRef) -> bool {
            let b = node.borrow();
            if b.is_search_kind() {
                return b.core().get_nyes() == Nyes::Econstanic;
            }
            let kids: Vec<FirRef> = b.core().foolish_children().to_vec();
            drop(b);
            kids.iter().any(any_search_is_econstanic)
        }
        assert!(
            any_search_is_econstanic(&root),
            "an SFF body's descendant search should be built ECONSTANIC"
        );
    }

    #[test]
    fn push_foolish_child_appends_in_order_matching_the_vec_constructor() {
        // Incremental pushes and a full-Vec `new` must produce the same store,
        // since `new` routes its initial children through `push_foolish_child`.
        let a = make_leaf(Nyes::Constant);
        let b = make_leaf(Nyes::Prembrionic);

        // Path 1: hand the whole Vec to the constructor.
        let via_vec = make_root_brane(vec![Rc::clone(&a), Rc::clone(&b)]);
        let via_vec_borrowed = via_vec.borrow();
        let via_vec_core = via_vec_borrowed.core();

        // Path 2: start empty, push incrementally.
        let via_push = make_root_brane(vec![]);
        // `push_foolish_child` takes `&mut self` by design — reachable only
        // while the ProtoBrane is still owned, not through an Rc<RefCell<…>>.
        // Build a standalone ProtoBrane to exercise it directly.
        let mut standalone = ProtoBrane::new(
            vec![],
            via_push.borrow().core().parent_weak(),
            Nyes::Prembrionic,
        );
        standalone.push_foolish_child(Rc::clone(&a));
        standalone.push_foolish_child(Rc::clone(&b));

        // Both paths agree on length and order.
        assert_eq!(via_vec_core.foolish_children().len(), 2);
        assert_eq!(standalone.foolish_children().len(), 2);
        assert!(Rc::ptr_eq(&standalone.get_foolish_child(0).unwrap(), &a));
        assert!(Rc::ptr_eq(&standalone.get_foolish_child(1).unwrap(), &b));
        assert!(Rc::ptr_eq(&via_vec_core.get_foolish_child(0).unwrap(), &a));
        assert!(Rc::ptr_eq(&via_vec_core.get_foolish_child(1).unwrap(), &b));

        // Out-of-range reads answer None rather than panicking.
        assert!(standalone.get_foolish_child(2).is_none());
        assert!(via_vec_core.get_foolish_child(2).is_none());
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
