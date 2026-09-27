//! `FVMStorage` — the arena-backed FIR store.
//!
//! Every FIR node lives in a `u32`-indexed arena slot, addressed through the
//! validated handle type [`FirPointer`] rather than a `Rc<RefCell<dyn Fir>>`
//! with a `Weak` parent back-pointer. A `&mut FVMStorage` borrow is the sole
//! exclusivity check for mutation — no per-node interior mutability, no
//! runtime borrow panics. See `docs/foop/FOOP-16.md` §Specification for the
//! full design rationale.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};

use foolish_core::fir::Nyes;

use crate::nyes_ext::NyesExt;

use crate::identifier::{Characterizations, Identifier};

/// Randomized per-[`FVMStorage`] instance stamp.
///
/// Minted once per [`FVMStorage::new`] call so a [`FirPointer`] from one arena
/// instance can never silently validate against a different arena instance —
/// see FOOP-16.md §Specification "Arena allocation and expansion" for why this
/// field is randomized while `index`/`generation` are sequential.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct ArenaId(u64);

impl ArenaId {
    /// Mints a fresh, effectively-unique stamp.
    ///
    /// A process-wide monotonic counter, not a random number generator: this
    /// crate has no existing dependency on `rand`, and cross-process/cross-run
    /// collision resistance is not the property being protected — the only
    /// requirement is that two [`FVMStorage`] instances alive in the SAME
    /// process never compare equal. A monotonic counter guarantees that
    /// exactly, with no new dependency.
    fn mint() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        Self(NEXT.fetch_add(1, Ordering::Relaxed))
    }
}

/// A validated handle into one specific [`FVMStorage`] arena.
///
/// Cannot be constructed, incremented, or otherwise fabricated outside this
/// module — only ever handed out by an [`FVMStorage`] method that just
/// finished allocating or validating it. Bounds-checking and validity-checking
/// happen once, centrally, inside `FVMStorage`; callers never re-derive or
/// assume validity. See FOOP-16.md §Specification "`FirPointer` — a validated
/// arena handle" for the full rationale (this directly implements
/// `rust_instructions.md` §1b.4, "make illegal states unrepresentable").
///
/// Internal-only by design: never serialized, persisted, or exposed outside
/// the lifetime of the single `FVMStorage` that minted it.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct FirPointer {
    arena: ArenaId,
    index: u32,
    generation: u32,
}

/// One arena slot: the stored payload plus its generation counter.
///
/// `generation` is not load-bearing for safety today — slots are never
/// reused/reclaimed, so every generation value is currently `0` — but is
/// carried so a future slot-reuse scheme does not need to change
/// `FirPointer`'s shape.
struct Slot {
    payload: ProtoBrane,
    parent: FirPointer,
    /// Parse-time children — fixed topology, set once at construction.
    foolish_children: Vec<FirPointer>,
    generation: u32,
}

/// Per-node payload stored in each arena slot.
///
/// Tree structure (`parent`, `foolish_children`) lives on [`Slot`] itself,
/// not here — the arena, not each node, owns topology — so [`FirCursor`]/
/// [`FirCursorMut`] have one place to read and write it.
#[derive(Debug, Clone)]
pub(crate) struct ProtoBrane {
    spec: FirSpec,
    nyes: Nyes,
    /// Compute-time children (search results, resolved references — as opposed to `foolish_children`'s
    /// fixed parse-time topology). A plain `Vec`: the arena's `&mut FVMStorage` borrow is the only
    /// exclusivity check a mutator needs, so no interior mutability is required here.
    ubc_children: Vec<FirPointer>,
    /// Task queue driving this node's stepping.
    tasks: VecDeque<FirPointer>,
    alarm_reason: Option<String>,
    /// The **unsteppable cause** (FOOP-86 §6.4): set only on **brane-like**
    /// nodes, naming the statement whose presence made the rest of this
    /// brane unsteppable — a null-characterized name given a meaning it
    /// cannot have in this brane's context (§6.2's four routes). `None` in
    /// the common case; once set, terminal.
    ///
    /// It lives on the BRANE, not on the offending statement, and that is
    /// the whole point (§6.4): the statement itself is fine — `3` in
    /// `'K = 3` is an honest `IndepInt`/`Independent` and must not be
    /// marked NK — while the brane is the thing that could not finish its
    /// work. Recording it on the statement (the superseded `nf_reason`)
    /// put the fault exactly where readers resolve to it, which is what
    /// leaked NK into every later reader of the name.
    ///
    /// Stores the CAUSE, not the boundary (§6.6 Q-A): the rendering reads
    /// its name for the annotation, and the first unsteppable statement is
    /// simply the next one.
    unsteppable_cause: Option<FirPointer>,
    /// Mirrors `ConcatenationFir::_helpers_populated`. Applies ONLY to `FirSpec::Concatenation` nodes. A
    /// monotonic one-way gate distinct from "`ubc_children` is non-empty": the real field must flip `true`
    /// even on the ZERO-LINES-TO-MERGE path (`populate_concat_helpers` pushes no helper at all when every
    /// element resolves to an empty brane), so the SECOND `Braning` re-entry can settle from the (empty)
    /// helper set rather than re-attempting the merge forever. `false` for every other kind, always.
    helpers_populated: bool,
}

impl ProtoBrane {
    /// No caller yet — kept as the symmetric counterpart to [`Self::set_nyes`] for code that already holds
    /// an `&ProtoBrane` (e.g. inside a `with_mut`/`get_mut` closure) and would otherwise have to route back
    /// through `FVMStorage` just to read what it already has in hand.
    #[expect(dead_code, reason = "no caller yet — symmetric counterpart to set_nyes")]
    pub(crate) fn get_nyes(&self) -> Nyes {
        self.nyes
    }

    /// A FIR owns its own `nyes`; it must never be changed from outside the FIR. `pub(crate)`, not `pub`,
    /// is the enforcement: only a node's own `fir_op_step` or its own construction may call this.
    pub(crate) fn set_nyes(&mut self, n: Nyes) {
        self.nyes = n;
    }

    /// No-op except on `FirSpec::Concatenation`.
    pub(crate) fn set_concat_rendering_aid(&mut self, aid: ConcatRenderingAid) {
        if let FirSpec::Concatenation { rendering_aid, .. } = &mut self.spec {
            *rendering_aid = aid;
        }
    }

    /// No-op except on `FirSpec::Search`/`FirSpec::Index`.
    pub(crate) fn set_contexted(&mut self, value: bool) {
        match &mut self.spec {
            FirSpec::Search { contexted, .. } | FirSpec::Index { contexted, .. } => {
                *contexted = value;
            }
            _ => {}
        }
    }

    pub(crate) fn ubc_children(&self) -> &[FirPointer] {
        &self.ubc_children
    }

    /// Takes the child's current `Nyes` as a parameter, rather than looking it up itself, because
    /// `ProtoBrane` cannot reach across arena slots to read another node's state — the caller
    /// ([`FirCursorMut::push_ubc_child`]) already has `&FVMStorage` access to read it first.
    pub(crate) fn push_ubc_child(&mut self, child: FirPointer, child_nyes: Nyes) {
        self.ubc_children.push(child);
        if !child_nyes.is_constanic() {
            self.tasks.push_back(child);
        }
    }

    /// A search settles with at most one result ever pushed to `ubc_children` (the singular-result
    /// invariant); a second push indicates a search re-resolving after already settling, a logic error
    /// rather than a legitimate re-evaluation.
    pub(crate) fn push_search_result(&mut self, result: FirPointer, result_nyes: Nyes) {
        debug_assert!(
            self.ubc_children.is_empty(),
            "search FIR already has a result; existing searches are singular-result \
             (ubc_children must be <= 1)"
        );
        self.push_ubc_child(result, result_nyes);
    }

    pub(crate) fn clear_ubc_children(&mut self) {
        self.ubc_children.clear();
    }

    pub(crate) fn front_task(&self) -> Option<FirPointer> {
        self.tasks.front().copied()
    }

    pub(crate) fn pop_front_task(&mut self) {
        self.tasks.pop_front();
    }

    pub(crate) fn push_task(&mut self, t: FirPointer) {
        self.tasks.push_back(t);
    }

    pub(crate) fn set_alarm_reason(&mut self, reason: String) {
        self.alarm_reason = Some(reason);
    }

    pub(crate) fn alarm_reason(&self) -> Option<&str> {
        self.alarm_reason.as_deref()
    }

    /// `None` unless this brane was halted by an unsteppable statement
    /// (FOOP-86 §6.4); then, the statement that caused it.
    pub(crate) fn unsteppable_cause(&self) -> Option<FirPointer> {
        self.unsteppable_cause
    }

    /// Terminal and first-writer-wins: only the FIRST unsteppable statement is recorded (§6.4 — the brane
    /// halts there, so no later one is ever reached anyway, but a merge-time route could try twice).
    pub(crate) fn set_unsteppable_cause(&mut self, cause: FirPointer) {
        if self.unsteppable_cause.is_none() {
            self.unsteppable_cause = Some(cause);
        }
    }

    /// See the `helpers_populated` field's own doc comment for why this
    /// cannot be inferred from `ubc_children`'s emptiness.
    pub(crate) fn helpers_populated(&self) -> bool {
        self.helpers_populated
    }

    /// One-way — never called with `false`.
    pub(crate) fn set_helpers_populated(&mut self) {
        self.helpers_populated = true;
    }
}

/// The arena. Owns every node reachable from any [`FirPointer`] it minted.
///
/// `slots` is a plain growable `Vec`, not a fixed-capacity buffer (see
/// FOOP-16.md §Specification "Arena allocation and expansion"): allocation is
/// a bump allocator (`make_my_child` pushes a new `Slot`, the new pointer's
/// `index` is `slots.len() - 1` at push time), and there is no free-list reuse
/// in this FOOP's scope — a slot, once allocated, is never reclaimed.
pub struct FVMStorage {
    arena_id: ArenaId,
    slots: Vec<Slot>,
}

/// How a concatenation was spelled in source. Affects SEQUENCING ONLY — never evaluation (FOOP-65 §5.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConcatProvenance {
    /// Ordinary brane concatenation (juxtaposition): `{a}{b}{c}`.
    Juxtaposition,
    /// Tail concatenation (backtick chain): `` c`b`a `` — the elements are
    /// already stored REVERSED relative to source (FOOP-65 §5.2).
    TailConcatenation,
}

/// Which concatenation elements wrote their SF/SFF marker in source.
///
/// `build_concat_element` synthesizes a `StayFoolish` around a BARE element,
/// so `f2` and `<f2>` become identical FIR and the renderer cannot tell them
/// apart. This records the indices that were marked, so it can put back
/// exactly those. Sparse — bare is the common case, so this is usually empty.
/// Sequencing only, never evaluation. (FOOP-36 N1, concatenation-only.)
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ConcatRenderingAid {
    /// `(index, marker)` for source-marked elements, ascending by index.
    marked: Vec<(usize, StayMarker)>,
}

/// The marker an element was written with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StayMarker {
    /// `<x>`
    Sf,
    /// `<<x>>`
    Sff,
}

impl ConcatRenderingAid {
    fn record(&mut self, index: usize, marker: StayMarker) {
        self.marked.push((index, marker));
    }

    fn is_empty(&self) -> bool {
        self.marked.is_empty()
    }

    /// The marker element `index` was written with, if any.
    #[must_use]
    pub fn marker_at(&self, index: usize) -> Option<StayMarker> {
        self.marked.iter().find_map(|&(i, m)| (i == index).then_some(m))
    }
}

/// The name used for an anonymous statement (a bare expression with no LHS identifier). The sequencer
/// renders a statement named `???` WITHOUT a `name=` prefix (FOOP-62 #19).
pub(crate) const ANON_STMT_NAME: &str = "???";

/// One variant per FIR kind.
///
/// Each variant's fields are that kind's own non-tree-structural data —
/// parent/children are handled generically by [`FVMStorage::make_my_child`],
/// so no variant carries a parent or child list.
///
/// This enum exists so construction dispatches on data rather than
/// fragmenting into a `create_x_child` method per kind.
#[derive(Debug, Clone, PartialEq)]
pub enum FirSpec {
    IndepInt {
        value: i64,
    },
    Nk {
        reason: String,
    },
    Operator {
        op: String,
    },
    /// `nf_reason` is not part of the spec: it starts `None` always,
    /// discovered later during `fir_op_step`, never known at construction.
    Statement {
        identifier: Identifier,
        line_number: usize,
    },
    Brane {
        characterizations: Characterizations,
    },
    /// `sf_inner_pattern` is not part of the spec: it starts `None` always.
    Search {
        pattern: String,
        anchored: bool,
        forward: bool,
        is_value_search: bool,
        contexted: bool,
    },
    Index {
        offset: i32,
        anchored: bool,
        contexted: bool,
    },
    /// `referent` names the original found statement this reference wraps.
    FoolRef {
        referent: FirPointer,
    },
    StayFoolish,
    StayFullyFoolish,
    ConcatHelper,
    /// `helpers_populated` is not part of the spec: it starts `false` and is
    /// set at most once, after construction (see [`ProtoBrane::helpers_populated`]).
    Concatenation {
        provenance: ConcatProvenance,
        /// Which elements wrote their SF/SFF marker in source. Sequencing only; see [`ConcatRenderingAid`].
        rendering_aid: ConcatRenderingAid,
    },
    Creation,
    Comparison {
        op: crate::system_foo::ComparisonOp,
    },
}

impl FirSpec {
    /// The `Nyes` a freshly-constructed node of this spec starts at.
    ///
    /// `Creation` and `IndepInt` are fully determined the moment they're
    /// written — a literal integer or a creation mark needs no children and
    /// no computation to know its value — so they start `Independent`
    /// (already conclusive). Every other kind depends on stepping (its own
    /// computation, or its children's) to reach a conclusive value, so it
    /// starts `Prembrionic`.
    fn initial_nyes(&self) -> Nyes {
        match self {
            FirSpec::Creation | FirSpec::IndepInt { .. } => Nyes::Independent,
            _ => Nyes::Prembrionic,
        }
    }
}

impl FVMStorage {
    /// Creates a fresh, empty arena with its own randomized [`ArenaId`] stamp.
    pub fn new() -> Self {
        Self {
            arena_id: ArenaId::mint(),
            slots: Vec::new(),
        }
    }

    /// Validates `ptr` against this arena and its slot's current generation,
    /// panicking on mismatch.
    ///
    /// A validation failure here means a caller is holding a `FirPointer`
    /// minted by a DIFFERENT `FVMStorage` instance — a programming error, not
    /// a recoverable Foolish-program condition (the arena has no equivalent of
    /// an out-of-bounds Foolish index; `FirPointer`'s whole design goal is to
    /// make this unrepresentable at the type level for everything except a
    /// cross-arena mix-up, which can only happen by a caller holding onto a
    /// pointer past its arena's lifetime or mixing two arenas together).
    fn validate(&self, ptr: FirPointer) -> usize {
        assert_eq!(
            ptr.arena, self.arena_id,
            "FirPointer minted by a different FVMStorage instance"
        );
        let index = ptr.index as usize;
        let slot = self
            .slots
            .get(index)
            .expect("FirPointer index out of bounds for its own arena");
        assert_eq!(
            slot.generation, ptr.generation,
            "FirPointer generation mismatch (stale pointer)"
        );
        index
    }

    /// Retrieve this pointer's [`FirSpec`] for reading.
    pub fn get(&self, ptr: FirPointer) -> &FirSpec {
        let index = self.validate(ptr);
        &self.slots[index].payload.spec
    }

    /// Retrieve this pointer's current [`Nyes`].
    pub fn get_nyes(&self, ptr: FirPointer) -> Nyes {
        let index = self.validate(ptr);
        self.slots[index].payload.nyes
    }

    pub fn alarm_reason(&self, ptr: FirPointer) -> Option<&str> {
        let index = self.validate(ptr);
        self.slots[index].payload.alarm_reason()
    }

    /// This BRANE's unsteppable cause, if it was halted (FOOP-86 §6.4): the statement that gave a
    /// null-characterized name a meaning it could not have here. `None` for every brane that stepped to
    /// completion, and for every non-brane kind.
    pub fn unsteppable_cause(&self, ptr: FirPointer) -> Option<FirPointer> {
        let index = self.validate(ptr);
        self.slots[index].payload.unsteppable_cause()
    }

    /// Records the halt (FOOP-86 §6.3). First-writer-wins — see `ProtoBrane::set_unsteppable_cause`.
    pub(crate) fn set_unsteppable_cause(&mut self, ptr: FirPointer, cause: FirPointer) {
        let index = self.validate(ptr);
        self.slots[index].payload.set_unsteppable_cause(cause);
    }

    /// Retrieve, modify, and return in one call — the "retrieve a payload, be able to modify it before
    /// returning" primitive. Closure-scoped so there is no separate get/set pair to keep in sync, and no
    /// `RefCell`-style runtime borrow tracking is needed: the `&mut self` borrow on `FVMStorage` is the
    /// only exclusivity check required. `pub(crate)`: `ProtoBrane` is this module's own internal payload
    /// type, never exposed outside it.
    pub(crate) fn with_mut<R>(&mut self, ptr: FirPointer, f: impl FnOnce(&mut ProtoBrane) -> R) -> R {
        let index = self.validate(ptr);
        f(&mut self.slots[index].payload)
    }

    /// Retrieve one exclusive, held `&mut ProtoBrane` for a run of several SEQUENTIAL writes with nothing
    /// storage-needing interleaved between them — the same capability as `with_mut`, offered as a plain
    /// borrow rather than a closure; the choice between the two is style, not capability.
    pub(crate) fn get_mut(&mut self, ptr: FirPointer) -> &mut ProtoBrane {
        let index = self.validate(ptr);
        &mut self.slots[index].payload
    }

    /// This pointer's parse-time children, in construction order.
    pub fn foolish_children(&self, ptr: FirPointer) -> &[FirPointer] {
        let index = self.validate(ptr);
        &self.slots[index].foolish_children
    }

    /// This pointer's parent.
    pub fn parent(&self, ptr: FirPointer) -> FirPointer {
        let index = self.validate(ptr);
        self.slots[index].parent
    }

    /// Arena-owning implementation: allocates a slot for `spec`, sets its
    /// parent to `parent`, appends the new pointer to parent's child list,
    /// returns the new `FirPointer`. This is the one place a `FirPointer` is
    /// ever constructed from raw parts.
    ///
    /// Called directly only where no parent `FirPointer` exists yet (the very
    /// first/root node of a tree — see [`FVMStorage::make_root`]); every other
    /// call site uses [`FirPointer::create_child`].
    ///
    /// Bump-allocates: the new pointer's `index` is `slots.len()` before the
    /// push. `generation` is always `0` in FOOP-16's scope (no slot reuse).
    pub fn make_my_child(&mut self, parent: FirPointer, spec: FirSpec) -> FirPointer {
        self.validate(parent);
        let ptr = self.allocate(spec, parent);
        self.slots[parent.index as usize].foolish_children.push(ptr);
        ptr
    }

    /// Allocates a fresh node with `parent` as its `.parent` field, WITHOUT
    /// appending it to `parent`'s `foolish_children` list. For nodes that
    /// are a computed RESULT, not part of the parse-derived topology (e.g.
    /// `combine`'s NK-on-child-NK/division-by-zero branches,
    /// `ConcatenationFir`'s type-error branch, `IndexFir`'s
    /// named-non-brane-anchor diagnostic) — `create_child`/`make_my_child`'s
    /// ALWAYS-append contract is correct for parse topology but wrong here.
    ///
    /// Using `create_child` for a result node instead of this method is a
    /// real, silent bug: the result node ends up corrupting
    /// `foolish_children`, which then feeds into anything iterating it
    /// afterward — the operand-rendering loops in output serialization, and
    /// `combine`'s own `any_nk` re-check on a later step. For example,
    /// `{a = 10 / 0 * 5;}`'s outer `*` operator would have exactly 2
    /// `foolish_children` (`/`-node, `5`) before settling and 3 after
    /// (`/`-node, `5`, a phantom fresh `Nk{reason:"operator nk"}`) if its
    /// result were wrongly self-appended to the very list `combine` itself
    /// reads on next entry.
    pub(crate) fn make_orphan_child(&mut self, parent: FirPointer, spec: FirSpec) -> FirPointer {
        self.validate(parent);
        self.allocate(spec, parent)
    }

    /// Appends an ALREADY-EXISTING pointer to `parent`'s `foolish_children` list WITHOUT allocating a new
    /// slot and WITHOUT reparenting `child` (its own `.parent` field, and therefore its home brane and line
    /// number, are left exactly as they were). This is `revive_constanic`'s "share-not-clone" path's other
    /// half: sharing a node means its own parent link stays untouched, but the NEW parent's
    /// `foolish_children` list must still record the shared pointer as one of its children — without this
    /// append, a caller like `populate_concat_helpers` that walks the new parent's `foolish_children`
    /// afterward would silently see the shared child missing, even though the share reported success.
    pub(crate) fn attach_shared_foolish_child(&mut self, parent: FirPointer, child: FirPointer) {
        self.validate(parent);
        self.validate(child);
        self.slots[parent.index as usize].foolish_children.push(child);
    }

    /// Inserts the very first node of a fresh arena, self-parented (its own
    /// `FirPointer` is its own parent).
    ///
    /// Unlike `make_my_child`, there is no pre-existing parent to validate or
    /// wire into — this method exists specifically for that no-parent case.
    pub fn make_root(&mut self, spec: FirSpec) -> FirPointer {
        let index = self.slots.len() as u32;
        let placeholder = FirPointer {
            arena: self.arena_id,
            index,
            generation: 0,
        };
        // allocate() needs a parent value up front; a root is its own parent,
        // so pre-compute the pointer it will receive and pass it as both.
        self.allocate(spec, placeholder)
    }

    /// Shared slot-push logic for [`Self::make_my_child`] and [`Self::make_root`]: constructs the new
    /// `FirPointer`, pushes its `Slot`, and returns the pointer. Does NOT wire the new pointer into any
    /// parent's child list — callers do that themselves (`make_my_child` does; `make_root` has no parent to
    /// wire into).
    fn allocate(&mut self, spec: FirSpec, parent: FirPointer) -> FirPointer {
        let index = self.slots.len() as u32;
        let ptr = FirPointer {
            arena: self.arena_id,
            index,
            generation: 0,
        };
        let nyes = spec.initial_nyes();
        self.slots.push(Slot {
            payload: ProtoBrane {
                spec,
                nyes,
                ubc_children: Vec::new(),
                tasks: VecDeque::new(),
                alarm_reason: None,
                unsteppable_cause: None,
                helpers_populated: false,
            },
            parent,
            foolish_children: Vec::new(),
            generation: 0,
        });
        ptr
    }
}

impl Default for FVMStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
impl FVMStorage {
    /// Creates a fresh arena containing a single self-rooting leaf. A leaf here is an `IndepInt` — the
    /// simplest kind with no interesting children — at the given `Nyes`.
    pub(crate) fn test_leaf(nyes: Nyes) -> (Self, FirPointer) {
        let mut storage = Self::new();
        let ptr = storage.make_root(FirSpec::IndepInt { value: 0 });
        storage.with_mut(ptr, |fir| fir.set_nyes(nyes));
        (storage, ptr)
    }

    /// Creates a fresh arena containing a root `Brane` with the given children specs.
    pub(crate) fn test_root_brane(children_specs: &[FirSpec]) -> (Self, FirPointer) {
        let mut storage = Self::new();
        let root = storage.make_root(FirSpec::Brane {
            characterizations: Characterizations::default(),
        });
        for spec in children_specs {
            root.create_child(&mut storage, spec.clone());
        }
        (storage, root)
    }
}

impl FirPointer {
    /// The primary construction call site. Delegates to [`FVMStorage::make_my_child`].
    pub fn create_child(self, storage: &mut FVMStorage, spec: FirSpec) -> FirPointer {
        storage.make_my_child(self, spec)
    }

    /// This pointer's parent, per the arena's stored parent link. Always `Some` in practice — even the
    /// structural root's "parent" is itself — since the arena never drops a live node out from under a
    /// valid pointer. `Option` is kept in the signature for callers that need to distinguish the root case
    /// explicitly.
    pub fn get_parent(self, storage: &FVMStorage) -> Option<FirPointer> {
        Some(storage.parent(self))
    }

    /// Whether this pointer is the structural root of its arena (its own parent).
    pub fn is_root(self, storage: &FVMStorage) -> bool {
        storage.parent(self) == self
    }

    /// Climbs the parent chain to the first brane-like kind: climb until `parent()` pointer-equals `self`
    /// (structural root) → `None`; else check brane-likeness → stop, else recurse.
    pub fn home_brane(self, storage: &FVMStorage) -> Option<FirPointer> {
        let parent = storage.parent(self);
        if parent == self {
            return None;
        }
        if parent.is_brane_like(storage) {
            Some(parent)
        } else {
            parent.home_brane(storage)
        }
    }

    /// Whether this pointer is brane-like (has statements to iterate).
    fn is_brane_like(self, storage: &FVMStorage) -> bool {
        FirCursor::new(self, storage).is_brane_like()
    }

    /// Whether this pointer is a `Statement`.
    fn is_statement(self, storage: &FVMStorage) -> bool {
        matches!(storage.get(self), FirSpec::Statement { .. })
    }

    /// The statement this pointer's search would read as its position: climb until a `Statement` kind is
    /// found, or until `parent()` pointer-equals `self` (structural root, returned as-is).
    fn get_my_statement(self, storage: &FVMStorage) -> FirPointer {
        if self.is_statement(storage) {
            return self;
        }
        let parent = storage.parent(self);
        if parent == self {
            self
        } else {
            parent.get_my_statement(storage)
        }
    }

    /// The constanic result this pointer resolves to, if any. Applies the constanic gate itself —
    /// pre-constanic always answers `None`. `pub(crate)`: also called directly by
    /// `search_fir_dispatch::statement_value_for_comparison`, a nested module.
    pub(crate) fn settled_constanic_result(self, storage: &FVMStorage) -> Option<FirPointer> {
        if !storage.get_nyes(self).is_constanic() {
            return None;
        }
        let index = storage.validate(self);
        storage.slots[index].payload.ubc_children().first().copied()
    }

    /// Recursively unwraps through `settled_constanic_result`, returning `self` when there is none.
    pub fn value(self, storage: &FVMStorage) -> FirPointer {
        match self.settled_constanic_result(storage) {
            Some(child) => child.value(storage),
            None => self,
        }
    }

    /// Performs ONE stepping action: if the front task is already constanic, pop it; otherwise recurse into
    /// it. Once there is no front task left, calls this node's own `fir_op_step`.
    pub fn step(self, storage: &mut FVMStorage) -> FirPointer {
        step_inner(self, storage, ArenaScope::default(), 0)
    }

    /// The display name a `Creation` reports when read from `viewed_from`,
    /// if any. `self` must be a `FirSpec::Creation` pointer.
    ///
    /// Two conditions must both hold (FOOP-33): (1) `viewed_from` is
    /// somewhere OTHER than the creation's own defining statement — that
    /// statement is where it was born, and reporting the same name back
    /// there would read as self-referential; (2) the defining statement's
    /// name is null-characterized — only a protected constant like `'True`
    /// qualifies.
    #[must_use]
    pub fn get_display_name(self, storage: &FVMStorage, viewed_from: FirPointer) -> Option<String> {
        let parent = storage.parent(self);
        // A self-parenting node is the root; it has no defining statement.
        if parent == self {
            return None;
        }
        let identifier = FirCursor::new(parent, storage).as_stmt_identifier()?;
        let body = storage.foolish_children(parent).first().copied()?;
        if body != self {
            return None;
        }
        // Condition 2: only a null-characterized (protected-constant) name qualifies at all.
        if !identifier.is_nully_characterizing_coordinate_name() {
            return None;
        }
        let name = identifier.searchable_name().to_owned();
        // Condition 1: never report the name when viewed from the creation's own defining statement -- only
        // from a different statement (a reference reached elsewhere).
        if parent == viewed_from {
            return None;
        }
        Some(name)
    }

    /// The index of `stmt` among `self`'s statements, by identity. `self`
    /// must be brane-like.
    ///
    /// An ordinary Rust-side walk, not a Foolish search — the `sift_*`
    /// naming convention would apply, but `find_stmt_index` is kept to match
    /// this operation's established name elsewhere in the crate.
    pub fn find_stmt_index(self, storage: &FVMStorage, stmt: FirPointer) -> Option<usize> {
        let cursor = FirCursor::new(self, storage);
        let count = cursor.stmt_count()?;
        (0..count).find(|&i| cursor.stmt_at(i) == Some(stmt))
    }

    /// Climbs the parent chain from `self` until a `Statement` kind is found, then returns that statement
    /// together with its home brane. `None` if the climb reaches the structural root without finding one.
    pub fn find_enclosing_stmt_and_brane(self, storage: &FVMStorage) -> Option<(FirPointer, FirPointer)> {
        let mut current = storage.parent(self);
        let mut prev = self;
        loop {
            if current.is_statement(storage) {
                let brane = current.home_brane(storage)?;
                return Some((current, brane));
            }
            if current == prev {
                return None;
            }
            prev = current;
            current = storage.parent(current);
        }
    }
}

/// Whether the backward search `?<name>=<creation>` performed at
/// `viewed_from` finds `creation` -- "is this original name in context HERE,
/// and does it mean THIS creation?".
///
/// This is a RENDERING question (FOOP-36 §N4), deliberately kept OUT of
/// [`FirPointer::get_display_name`]. That method answers a different, FOOP-33
/// question -- "what is this creation's original name?" -- and the no-rename
/// rule (`check_rename_of_named_creation`) uses it as an identity oracle.
/// Narrowing it by context would silently disable that language rule.
///
/// This runs the real search engine ([`search_engine::contextful_search_scan`]
/// with [`SearchPredicate::NameValue`]), not a hand-rolled walk, so the name
/// gate and the identity gate are applied together on each candidate exactly
/// as an atomic `?name=value` search applies them (FOOP-23 §C.3.1). The value
/// gate reduces to arena-pointer identity for creations, via `default_equal`.
///
/// No `Search` FIR is constructed and no node is mutated: the engine's scan
/// takes `&FVMStorage`, which is what lets the sequencer -- whose whole
/// contract is to read already-constanic FIR -- ask a real search question.
///
/// The scan walks the viewing statement's home brane backward from its own
/// position (Foolish cannot look forward), then repeats outward through
/// enclosing branes, unanchored-search style. A nearer statement of the same
/// name SHADOWS a farther one, and a shadowed name simply fails to match --
/// which is precisely the case that makes rendering a bare original name
/// unsound.
pub(crate) fn search_name_finds_this_creation(
    storage: &FVMStorage,
    viewed_from: FirPointer,
    name: &str,
    creation: FirPointer,
) -> bool {
    use search_engine::{BraneNavigator, ScanOutcome, SearchPredicate, contextful_search_scan};

    let predicate = SearchPredicate::NameValue {
        name: name.to_owned(),
        value: creation,
    };
    let mut stmt = viewed_from;
    for _ in 0..MAX_DEPTH {
        let Some(brane) = stmt.home_brane(storage) else {
            return false;
        };
        let cursor = FirCursor::new(brane, storage);
        let Some(count) = cursor.stmt_count() else {
            return false;
        };
        if count > 0 {
            let mut nav = BraneNavigator::new(storage, brane, false);
            // Backward from just before our own position; the whole brane
            // when we entered it from a nested one.
            let upper = brane.find_stmt_index(storage, stmt).unwrap_or(count);
            if upper > 0 {
                nav.set_range(0, upper - 1);
                match contextful_search_scan(storage, &mut nav, &predicate) {
                    ScanOutcome::Found(_) => return true,
                    // An Nk candidate halts the scan, exactly as it would halt a real search: the answer is
                    // not knowable, so the name cannot be justified here.
                    ScanOutcome::NkStop => return false,
                    ScanOutcome::Miss => {}
                }
            }
        }
        match brane.find_enclosing_stmt_and_brane(storage) {
            Some((outer_stmt, _)) => stmt = outer_stmt,
            None => return false,
        }
    }
    false
}

/// Guard against runaway recursion on pathologically deep trees.
const MAX_DEPTH: usize = 100;

/// Carries `step`'s scope down the tree: `current_statement`/`current_brane` (the IB/AB search anchors) and
/// `has_ancestral_sfm` (threaded through to `clone_stmt_result`/`revive_constanic` by `IndexFir`'s
/// contexted/anchored dispatch).
#[derive(Debug, Clone, Copy, Default)]
struct ArenaScope {
    current_statement: Option<FirPointer>,
    current_brane: Option<FirPointer>,
    has_ancestral_sfm: bool,
}

/// Recursion companion for [`FirPointer::step`], carrying the depth counter.
fn step_inner(ptr: FirPointer, storage: &mut FVMStorage, scope: ArenaScope, depth: usize) -> FirPointer {
    if depth > MAX_DEPTH {
        return ptr;
    }
    // FOOP-86 §6.3 — THE HALT, checked BEFORE draining the next task. A statement that became unsteppable
    // recorded itself as this brane's cause while IT was being stepped; the remaining tasks in the queue
    // are the statements after it, and they must never be stepped. Checking here (rather than in the
    // brane's own `fir_op_step` arm, which only runs once the queue is already empty) is what makes "never
    // stepped" true.
    if halt_if_unsteppable(ptr, storage) {
        return ptr;
    }
    let front = storage.with_mut(ptr, |fir| fir.front_task());
    match front {
        Some(front_ptr) => {
            if storage.get_nyes(front_ptr).is_constanic() {
                storage.with_mut(ptr, |fir| fir.pop_front_task());
            } else {
                // Set has_ancestral_sfm when `ptr` is StayFoolish; set current_statement when `ptr` (the
                // node ABOUT TO RECURSE INTO ITS CHILD) is a Statement; set current_brane when `ptr` is
                // brane-like. All three are set on `ptr`'s own scope before recursing into `front_ptr`.
                let mut child_scope = scope;
                if matches!(storage.get(ptr), FirSpec::StayFoolish) {
                    child_scope.has_ancestral_sfm = true;
                }
                if matches!(storage.get(ptr), FirSpec::Statement { .. }) {
                    child_scope.current_statement = Some(ptr);
                }
                if FirCursor::new(ptr, storage).is_brane_like() {
                    child_scope.current_brane = Some(ptr);
                }
                step_inner(front_ptr, storage, child_scope, depth + 1);
            }
            ptr
        }
        None => {
            fir_op_step(ptr, storage, scope);
            ptr
        }
    }
}

/// Enum dispatch for `fir_op_step`, one arm per [`FirSpec`] variant (rust_instructions.md §7: preferred
/// over `dyn` when the variant set is closed and known). The match is exhaustive with no fallback arm, so a
/// future 15th kind added without its own arm is a compile error naming the missing variant, not a silent
/// or panicking catch-all.
fn fir_op_step(ptr: FirPointer, storage: &mut FVMStorage, scope: ArenaScope) {
    let spec = storage.get(ptr).clone();
    match spec {
        // An IndepInt has no children or tasks, so there is no Braning phase — one step settles it.
        FirSpec::IndepInt { .. } => {
            if !storage.get_nyes(ptr).is_constanic() {
                storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Constant));
            }
        }
        FirSpec::Nk { .. } => {
            if !storage.get_nyes(ptr).is_constanic() {
                storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Nk));
            }
        }
        // `Operator` is not brane-like — it has no `stmt_count`/ `is_brane_like` override.
        FirSpec::Operator { .. } => match storage.get_nyes(ptr) {
            Nyes::Prembrionic | Nyes::Embryonic => {
                storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Braning));
                let children: Vec<FirPointer> = storage.foolish_children(ptr).to_vec();
                let all_foolish_children_conclusive =
                    children.iter().all(|&c| storage.get_nyes(c).is_conclusive());
                if !all_foolish_children_conclusive {
                    for child in children {
                        storage.with_mut(ptr, |fir| fir.push_task(child));
                    }
                }
            }
            Nyes::Braning => combine(ptr, storage),
            _ => {}
        },
        // Push the body as a task; once it's constanic, adopt its NYES. If this statement's name is
        // null-characterized, run the FOOP-33 §4 refusal checks (`check_null_const_conflict`/
        // `check_rename_of_named_creation`) first — a name-constant redefinition or a named-creation rename
        // is caught and recorded via `nf_reason` before the body's value is adopted.
        FirSpec::Statement { .. } => match storage.get_nyes(ptr) {
            Nyes::Prembrionic | Nyes::Embryonic => {
                storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Braning));
                let children: Vec<FirPointer> = storage.foolish_children(ptr).to_vec();
                for child in children {
                    storage.with_mut(ptr, |fir| fir.push_task(child));
                }
            }
            Nyes::Braning => {
                if let Some(&body) = storage.foolish_children(ptr).first() {
                    let body_nyes = storage.get_nyes(body);
                    if body_nyes.is_constanic() {
                        let is_nully = match storage.get(ptr) {
                            FirSpec::Statement { identifier, .. } => {
                                identifier.is_nully_characterizing_coordinate_name()
                            }
                            _ => false,
                        };
                        if is_nully {
                            // `check_null_const_conflict`'s `ib_search_by_pattern` call wants the SEARCHING
                            // STATEMENT (it derives the home brane and index from `ptr` itself), but its
                            // `ab_search_by_pattern` call wants the STARTING BRANE that `_ab_search` climbs
                            // from. Passing the statement itself there would make
                            // `current_brane.get_my_statement() == current_brane` trivially true (a
                            // statement's own home statement is itself) and short-circuit to `None`
                            // immediately — so `ptr`'s home brane, not `ptr`, goes to the
                            // `ab_search_by_pattern` call.
                            let home_brane = ptr.home_brane(storage);
                            search_fir_dispatch::check_null_const_conflict(
                                storage,
                                ptr,
                                body,
                                Some(ptr),
                                home_brane,
                            );
                            search_fir_dispatch::check_rename_of_named_creation(storage, ptr, body);
                        }
                        // Route 3 (FOOP-86 §6.2) — recoordination. A statement whose settled VALUE is a
                        // brane brings that brane's members into this context. If one of them is a
                        // null-characterized name already defined here, the brane cannot be coordinated in:
                        // THIS STATEMENT is the unsteppable one, so the check runs on the statement holding
                        // the value, not on the members it would have introduced.
                        search_fir_dispatch::check_recoordinated_null_const_conflict(storage, ptr, body);
                        // Do NOT clobber a terminal state the route checks above already reached.
                        // `check_recoordinated_null_const_conflict` settles THIS statement NK (FOOP-86 §6.2
                        // route 3) and the route 1/4 checks may have halted the brane; writing `body_nyes`
                        // unconditionally would erase either. Same clobbering class as the
                        // `name_search_step` Embryonic bug.
                        if !storage.get_nyes(ptr).is_constanic() {
                            storage.with_mut(ptr, |fir| fir.set_nyes(body_nyes));
                        }
                    }
                }
            }
            _ => {}
        },
        FirSpec::Brane { .. } => match storage.get_nyes(ptr) {
            Nyes::Prembrionic | Nyes::Embryonic => {
                let children: Vec<FirPointer> = storage.foolish_children(ptr).to_vec();
                if children.is_empty() {
                    storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Constant));
                } else {
                    storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Braning));
                    for child in children {
                        storage.with_mut(ptr, |fir| fir.push_task(child));
                    }
                }
            }
            Nyes::Braning => {
                // FOOP-86 §6.3's halt is checked in `step_inner`, before each task is drained — see
                // there. `decide_brane_nyes` re-asserts it anyway: the classifier never returns NK, so
                // a halted brane reaching here must not be reclassified.
                let children: Vec<FirPointer> = storage.foolish_children(ptr).to_vec();
                if let Some(nyes) = decide_brane_nyes(storage, ptr, &children) {
                    storage.with_mut(ptr, |fir| fir.set_nyes(nyes));
                }
            }
            _ => {}
        },
        // A no-op — a `FoolRef` is born `Constant` at construction (see
        // `push_search_result_pair`) and never needs stepping.
        FirSpec::FoolRef { .. } => {}
        // Once its wrapped `expr` is constanic, expose EXPR'S OWN resolved value (its `ubc_children[0]`, or
        // `expr` itself if it has none) as this node's own `ubc_children[0]`, adopting that value's `Nyes`.
        // SF unwraps to a shared value; it never produces a genuinely new node of its own.
        FirSpec::StayFoolish => match storage.get_nyes(ptr) {
            Nyes::Prembrionic | Nyes::Embryonic => {
                let children: Vec<FirPointer> = storage.foolish_children(ptr).to_vec();
                if children.is_empty() {
                    storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Constant));
                } else {
                    storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Braning));
                    for child in children {
                        storage.with_mut(ptr, |fir| fir.push_task(child));
                    }
                }
            }
            Nyes::Braning => {
                if let Some(&expr) = storage.foolish_children(ptr).first() {
                    let expr_nyes = storage.get_nyes(expr);
                    if expr_nyes.is_constanic() {
                        let (result, result_nyes) =
                            match FirCursor::new(expr, storage).ubc_children().first().copied() {
                                Some(r) => (r, storage.get_nyes(r)),
                                None => (expr, expr_nyes),
                            };
                        let me = storage.get_mut(ptr);
                        me.push_ubc_child(result, result_nyes);
                        me.set_nyes(result_nyes);
                    }
                }
            }
            _ => {}
        },
        // Same value-unwrap shape as `StayFoolish`, with two differences: (1) SFF always moves to `Braning`
        // and pushes tasks unconditionally — there is no empty-children short-circuit; (2) the constanic
        // `Nyes` goes through `nyes_from_found`: an SFF wrapper can never itself be Econstanic — an
        // Econstanic result means SFF is still WAITING on it (Woconstanic), while the pushed result keeps
        // its own Econstanic unchanged.
        FirSpec::StayFullyFoolish => match storage.get_nyes(ptr) {
            Nyes::Prembrionic | Nyes::Embryonic => {
                let children: Vec<FirPointer> = storage.foolish_children(ptr).to_vec();
                storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Braning));
                for child in children {
                    storage.with_mut(ptr, |fir| fir.push_task(child));
                }
            }
            Nyes::Braning => {
                let children: Vec<FirPointer> = storage.foolish_children(ptr).to_vec();
                if decide_nyes_due_to_children(storage, &children).is_some()
                    && let Some(&expr) = children.first()
                {
                    let expr_nyes = storage.get_nyes(expr);
                    let (result, result_nyes) =
                        match FirCursor::new(expr, storage).ubc_children().first().copied() {
                            Some(r) => (r, storage.get_nyes(r)),
                            None => (expr, expr_nyes),
                        };
                    let constanic_nyes = nyes_from_found(result_nyes);
                    let me = storage.get_mut(ptr);
                    me.push_ubc_child(result, result_nyes);
                    me.set_nyes(constanic_nyes);
                }
            }
            _ => {}
        },
        // Same shape as `Brane`'s arm — a `ConcatHelper` is transparent, inheriting brane-shaped stepping.
        FirSpec::ConcatHelper => match storage.get_nyes(ptr) {
            Nyes::Prembrionic | Nyes::Embryonic => {
                let children: Vec<FirPointer> = storage.foolish_children(ptr).to_vec();
                if children.is_empty() {
                    storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Constant));
                } else {
                    storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Braning));
                    for child in children {
                        storage.with_mut(ptr, |fir| fir.push_task(child));
                    }
                }
            }
            Nyes::Braning => {
                let children: Vec<FirPointer> = storage.foolish_children(ptr).to_vec();
                if let Some(nyes) = decide_brane_nyes(storage, ptr, &children) {
                    storage.with_mut(ptr, |fir| fir.set_nyes(nyes));
                }
            }
            _ => {}
        },
        // Every element must resolve to a brane, or the whole concatenation is NK (with the offending
        // indexes named); if some elements are still unresolved (not yet brane-like but not a type error
        // either) the concatenation waits (`Woconstanic`). Once every element is brane-like,
        // `populate_concat_helpers` builds and joins the merged lines exactly once (`helpers_populated`
        // below is that gate).
        FirSpec::Concatenation { .. } => match storage.get_nyes(ptr) {
            Nyes::Prembrionic | Nyes::Embryonic => {
                let children: Vec<FirPointer> = storage.foolish_children(ptr).to_vec();
                if children.is_empty() {
                    storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Constant));
                } else {
                    storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Braning));
                    for child in children {
                        storage.with_mut(ptr, |fir| fir.push_task(child));
                    }
                }
            }
            Nyes::Braning => {
                let children: Vec<FirPointer> = storage.foolish_children(ptr).to_vec();
                let mut all_brane_like = true;
                let mut type_errors: Vec<usize> = Vec::new();
                for (idx, &elem) in children.iter().enumerate() {
                    let resolved = elem.value(storage);
                    let brane_like = FirCursor::new(resolved, storage).is_brane_like();
                    all_brane_like &= brane_like;
                    if !brane_like && storage.get_nyes(elem).is_constantew() {
                        type_errors.push(idx);
                    }
                }

                if !type_errors.is_empty() {
                    let list = type_errors
                        .iter()
                        .map(usize::to_string)
                        .collect::<Vec<_>>()
                        .join(",");
                    let nk_ptr = storage.make_orphan_child(
                        ptr,
                        FirSpec::Nk {
                            reason: format!(
                                "concatenation constituent indexes where it's not a brane: {list}"
                            ),
                        },
                    );
                    let me = storage.get_mut(ptr);
                    me.push_ubc_child(nk_ptr, Nyes::Nk);
                    me.set_nyes(Nyes::Nk);
                    return;
                }
                if !all_brane_like {
                    storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Woconstanic));
                    return;
                }

                // Two-pass: the first pass builds the helper(s) and pushes them as tasks, staying
                // pre-constanic so the driver drains the helpers' own stepping before re-entry; the second
                // pass settles from the DRAINED helper results (the joined/recoordinated copies), not the
                // raw elements.
                let already_populated = storage.get_mut(ptr).helpers_populated();
                if !already_populated {
                    storage.get_mut(ptr).set_helpers_populated();
                    search_fir_dispatch::populate_concat_helpers(storage, ptr);
                    let helpers: Vec<FirPointer> = FirCursor::new(ptr, storage).ubc_children().to_vec();
                    for helper in helpers {
                        storage.with_mut(ptr, |fir| fir.push_task(helper));
                    }
                } else {
                    let helpers: Vec<FirPointer> = FirCursor::new(ptr, storage).ubc_children().to_vec();
                    // A MERGED brane halted by a conflicting null-const (FOOP-86 §6.2 route 2)
                    // carries the cause on the HELPER, not on this node, so the halt must be read
                    // from the helpers. Without this a halted merge would be reclassified CONSTANT
                    // and lose its `!! NK:` annotation — the classifier never returns NK.
                    let decided_nyes = if helpers.is_empty() {
                        Nyes::Constant
                    } else if helpers.iter().any(|&h| storage.unsteppable_cause(h).is_some()) {
                        Nyes::Nk
                    } else {
                        decide_nyes_due_to_children(storage, &helpers).unwrap_or(Nyes::Constant)
                    };
                    storage.with_mut(ptr, |fir| fir.set_nyes(decided_nyes));
                }
            }
            _ => {}
        },
        // A no-op — a creation is born `Independent` at construction and never needs stepping.
        FirSpec::Creation => {}
        // Same two-phase shape as `Operator`: push operands, then combine once Braning.
        FirSpec::Comparison { op } => match storage.get_nyes(ptr) {
            Nyes::Prembrionic | Nyes::Embryonic => {
                storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Braning));
                let children: Vec<FirPointer> = storage.foolish_children(ptr).to_vec();
                for child in children {
                    storage.with_mut(ptr, |fir| fir.push_task(child));
                }
            }
            Nyes::Braning => {
                let operands: Vec<FirPointer> = storage.foolish_children(ptr).to_vec();
                let any_unevaluated_here = operands.iter().any(|&o| {
                    storage
                        .foolish_children(o)
                        .first()
                        .is_some_and(|&inner| storage.get_nyes(inner) == Nyes::Econstanic)
                });
                if any_unevaluated_here {
                    storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Econstanic));
                    return;
                }

                // Read each operand THROUGH its SFF wrapper: `.value()` follows the constanic chain to
                // whatever the recoordinated index landed on.
                let values: Vec<Option<i64>> = operands
                    .iter()
                    .map(|&o| {
                        let resolved = o.value(storage);
                        FirCursor::new(resolved, storage).as_i64()
                    })
                    .collect();

                let (Some(&Some(left)), Some(&Some(right))) = (values.first(), values.get(1)) else {
                    // The operands DID evaluate here, and at least one is not an integer. Only integers are
                    // comparable (FOOP-33 §5, same principle `default_equal` follows).
                    let nk_ptr = storage.make_orphan_child(
                        ptr,
                        FirSpec::Nk {
                            reason: "comparison: non-integer operand".to_string(),
                        },
                    );
                    storage.with_mut(nk_ptr, |fir| fir.set_nyes(Nyes::Nk));
                    let me = storage.get_mut(ptr);
                    me.push_ubc_child(nk_ptr, Nyes::Nk);
                    me.set_alarm_reason("comparison: non-integer operand".to_string());
                    me.set_nyes(Nyes::Nk);
                    return;
                };

                let verdict = op.compare(left, right);
                // Resolve `'True`/`'False` by ordinary ancestral search from THIS comparison's own position
                // — it lives inside system.foo, so the search finds the very creations system.foo declares
                // (FOOP-33 §5: referentially identical to a user's own `'True` reference).
                let name = if verdict { "'True" } else { "'False" };
                let home_brane = ptr.home_brane(storage);
                let boolean = search_fir_dispatch::ab_search_by_pattern(storage, name, home_brane)
                    .and_then(|(found, _)| {
                        search_fir_dispatch::statement_value_for_comparison(storage, found)
                    })
                    .map(|body| body.value(storage));
                let Some(boolean) = boolean else {
                    // system.foo always defines 'True/'False; failing to find one means the prelude itself
                    // is malformed — an interpreter defect, not an unevaluable program. No `Result` to
                    // propagate through `fir_op_step`'s arena signature yet (same convention as
                    // `IndexFir`'s unanchored-offset invariant panic above), so this states the same
                    // invariant via `panic!`.
                    panic!(
                        "system.foo must define 'True and 'False, but {} could not resolve one",
                        op.searchable_name()
                    );
                };
                let clone = storage.revive_constanic(boolean, ptr, 0, scope.has_ancestral_sfm, false);
                let mut cursor = FirCursorMut::new(ptr, storage);
                cursor.push_ubc_child(clone);
                cursor.set_nyes(Nyes::Constant);
            }
            _ => {}
        },
        FirSpec::Search { is_value_search, .. } => {
            if is_value_search {
                search_fir_dispatch::value_search_step(storage, ptr, scope.has_ancestral_sfm);
            } else {
                search_fir_dispatch::name_search_step(
                    storage,
                    ptr,
                    scope.current_statement,
                    scope.current_brane,
                    scope.has_ancestral_sfm,
                );
            }
        }
        FirSpec::Index {
            offset,
            anchored,
            contexted,
        } => {
            match storage.get_nyes(ptr) {
                Nyes::Prembrionic | Nyes::Embryonic => {
                    if anchored {
                        let anchor = storage.foolish_children(ptr)[0];
                        storage.with_mut(ptr, |fir| {
                            fir.push_task(anchor);
                            fir.set_nyes(Nyes::Braning);
                        });
                    } else {
                        // An unanchored non-negative offset is a construction-time invariant violation the
                        // compiler itself should never produce — only
                        // `Astn::HeadTail`/`Astn::UnanchoredSeek` build unanchored `Index` nodes, and
                        // always with negative offsets — so this is a bug to panic on, not a reachable
                        // runtime program state.
                        assert!(offset < 0, "unanchored index requires negative offset");
                        match ptr.find_enclosing_stmt_and_brane(storage) {
                            Some((stmt_ref, brane_ref)) => {
                                match brane_ref.find_stmt_index(storage, stmt_ref) {
                                    Some(idx) => {
                                        let target = idx as i32 + offset;
                                        let len =
                                            FirCursor::new(brane_ref, storage).stmt_count().unwrap_or(0)
                                                as i32;
                                        if target < 0 || target >= len {
                                            storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Nk));
                                        } else {
                                            let mut nav = search_engine::BraneNavigator::new(
                                                storage, brane_ref, true,
                                            );
                                            let predicate = search_engine::SearchPredicate::Index(target);
                                            match search_engine::contextful_search_scan_no_body_check(
                                                storage, &mut nav, &predicate,
                                            ) {
                                                search_engine::ScanOutcome::Found(stmt) => {
                                                    let body =
                                                        storage.foolish_children(stmt).first().copied();
                                                    match body {
                                                        Some(_) => {
                                                            let clone =
                                                                search_fir_dispatch::clone_stmt_result(
                                                                    storage,
                                                                    stmt,
                                                                    ptr,
                                                                    scope.has_ancestral_sfm,
                                                                );
                                                            let mut cursor =
                                                                FirCursorMut::new(ptr, storage);
                                                            cursor.push_search_result_pair(clone, stmt);
                                                            cursor.set_nyes(Nyes::Braning);
                                                        }
                                                        None => storage
                                                            .with_mut(ptr, |fir| fir.set_nyes(Nyes::Nk)),
                                                    }
                                                }
                                                _ => storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Nk)),
                                            }
                                        }
                                    }
                                    None => storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Nk)),
                                }
                            }
                            None => storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Nk)),
                        }
                    }
                }
                Nyes::Braning => {
                    if !FirCursor::new(ptr, storage).ubc_children().is_empty() {
                        search_fir_dispatch::settle_from_ubc_result(storage, ptr);
                    } else if contexted && anchored {
                        let anchor = storage.foolish_children(ptr)[0];
                        let fool_ref_fir = FirCursor::new(anchor, storage).ubc_children().get(1).copied();
                        let contexted_result = fool_ref_fir.and_then(|frf| {
                            let referent = FirCursor::new(frf, storage).as_fool_ref_referent()?;
                            let h_brane = referent.home_brane(storage)?;
                            let p = h_brane.find_stmt_index(storage, referent)?;
                            let target = p as i32 + offset;
                            let len = FirCursor::new(h_brane, storage).stmt_count().unwrap_or(0) as i32;
                            if target < 0 || target >= len {
                                return None;
                            }
                            let mut nav = search_engine::BraneNavigator::new(storage, h_brane, true);
                            let predicate = search_engine::SearchPredicate::Index(target);
                            match search_engine::contextful_search_scan_no_body_check(
                                storage, &mut nav, &predicate,
                            ) {
                                search_engine::ScanOutcome::Found(stmt) => Some(stmt),
                                _ => None,
                            }
                        });
                        match contexted_result {
                            Some(stmt) => {
                                let clone = search_fir_dispatch::clone_stmt_result(
                                    storage,
                                    stmt,
                                    ptr,
                                    scope.has_ancestral_sfm,
                                );
                                let mut cursor = FirCursorMut::new(ptr, storage);
                                cursor.push_search_result_pair(clone, stmt);
                            }
                            None => {
                                if !storage.get_nyes(anchor).is_constanic() {
                                    // Anchor still stepping — no progress this call; leave NYES as-is.
                                } else {
                                    storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Nk));
                                }
                            }
                        }
                    } else if anchored {
                        let anchor = storage.foolish_children(ptr)[0];
                        let resolved = anchor.value(storage);
                        // FOOP-86 §6.4b: an anchor that resolves to a HALTED brane cannot be indexed into —
                        // the brane has no meaning, so no position in it can be meaningfully addressed.
                        // Checked before `is_brane_like`, which a halted brane still satisfies.
                        if storage.unsteppable_cause(resolved).is_some() {
                            storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Nk));
                            return;
                        }
                        if !FirCursor::new(resolved, storage).is_brane_like() {
                            // FOOP-75 §7: settling NK is only half the answer — name the offending anchor
                            // when it is itself nameable (an integer literal), so the rendered output reads
                            // `d =$ ??? (4 is not a brane)` instead of a bare miss. When the anchor is some
                            // other FIR (commonly a search that itself settled NK), there is no value to
                            // name — leave the result unset so the existing failed-anchor rendering shows
                            // through unchanged.
                            let named = FirCursor::new(resolved, storage).as_i64().map(|v| v.to_string());
                            if let Some(shown) = named {
                                let reason = format!("{shown} is not a brane");
                                let nk_ptr = storage.make_orphan_child(
                                    ptr,
                                    FirSpec::Nk {
                                        reason: reason.clone(),
                                    },
                                );
                                storage.with_mut(nk_ptr, |fir| fir.set_nyes(Nyes::Nk));
                                let me = storage.get_mut(ptr);
                                me.push_ubc_child(nk_ptr, Nyes::Nk);
                                me.set_alarm_reason(reason);
                            }
                            storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Nk));
                            return;
                        }
                        let mut nav = search_engine::BraneNavigator::new(storage, resolved, true);
                        let predicate = search_engine::SearchPredicate::Index(offset);
                        match search_engine::contextful_search_scan_no_body_check(
                            storage, &mut nav, &predicate,
                        ) {
                            search_engine::ScanOutcome::Found(stmt) => {
                                let body = storage.foolish_children(stmt).first().copied();
                                match body {
                                    Some(_) => {
                                        let clone = search_fir_dispatch::clone_stmt_result(
                                            storage,
                                            stmt,
                                            ptr,
                                            scope.has_ancestral_sfm,
                                        );
                                        let mut cursor = FirCursorMut::new(ptr, storage);
                                        cursor.push_search_result_pair(clone, stmt);
                                    }
                                    None => storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Nk)),
                                }
                            }
                            _ => storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Nk)),
                        }
                    } else {
                        storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Nk));
                    }
                }
                _ => {}
            }
        }
    }
}

/// Remaps a found node's `Nyes` for the caller's own settlement: Econstanic/Woconstanic → Woconstanic;
/// Constant/Independent → Constant; Nk → Nk; anything else (pre-constanic) passes through unchanged.
fn nyes_from_found(found: Nyes) -> Nyes {
    match found {
        Nyes::Econstanic | Nyes::Woconstanic => Nyes::Woconstanic,
        Nyes::Constant | Nyes::Independent => Nyes::Constant,
        Nyes::Nk => Nyes::Nk,
        other => other,
    }
}

/// FOOP-86 §6.3's halt. Returns `true` when `brane` has an unsteppable cause
/// and the halt was performed, `false` when there is nothing to halt.
///
/// What the halt does, in the order §6.3 specifies:
///
/// 1. **Stepping stops** — the remaining task queue is discarded, so no
///    statement after the cause is ever stepped.
/// 2. **The remainder settles `Nk`** — every statement strictly after the
///    cause that has not reached a constanic state. This is BOOKKEEPING, not
///    evaluation: no stepping is performed for them, no per-statement finding
///    is computed. It is required because a literally-untouched statement sits
///    at `Prembrionic`, which is pre-constanic — `decide_nyes_due_to_children`
///    would then hold the brane at `Braning` forever (spinning to the
///    iteration cap) and the renderer would annotate `!! PREMBRIONIC` instead
///    of the NK §6.5a specifies. See §6.4's "⚠ Implementation tension".
/// 3. **The brane takes `Nk` DIRECTLY** — not through
///    `decide_nyes_due_to_children`. The brane is NK because it failed to
///    finish its own work, not because a member is NK (§6.4c); a brane
///    containing an NK value is perfectly valid. Setting it here rather than
///    letting the rollup infer it also keeps this rule correct if the rollup's
///    own NK-propagation is ever changed (§6.6b).
///
/// The statement named by `unsteppable_cause` is NOT touched — it stepped
/// normally and reverts to Foolish with its honest value (§6.3).
fn halt_if_unsteppable(brane: FirPointer, storage: &mut FVMStorage) -> bool {
    let Some(cause) = storage.unsteppable_cause(brane) else {
        return false;
    };
    if storage.get_nyes(brane) == Nyes::Nk {
        return true; // already halted; nothing left to do, but still halted.
    }
    while storage.with_mut(brane, |fir| fir.front_task()).is_some() {
        storage.with_mut(brane, |fir| fir.pop_front_task());
    }
    let statements: Vec<FirPointer> = storage.foolish_children(brane).to_vec();
    let after_cause = statements
        .iter()
        .position(|&s| s == cause)
        .map_or(statements.len(), |i| i + 1);
    for &stmt in &statements[after_cause..] {
        // The statement AND its body: the renderer annotates from the body (§6.5a), and
        // `decide_nyes_due_to_children` walks statements, so both must leave the pre-constanic states or
        // the brane spins and the output reads `!! PREMBRIONIC` instead of the specified NK.
        let bodies: Vec<FirPointer> = storage.foolish_children(stmt).to_vec();
        for body in bodies {
            if !storage.get_nyes(body).is_constanic() {
                storage.with_mut(body, |fir| fir.set_nyes(Nyes::Nk));
            }
        }
        if !storage.get_nyes(stmt).is_constanic() {
            storage.with_mut(stmt, |fir| fir.set_nyes(Nyes::Nk));
        }
    }
    storage.with_mut(brane, |fir| fir.set_nyes(Nyes::Nk));
    true
}

/// Classifies a Braning node's decided `Nyes` from its children's states, in priority order:
/// all-Independent → Independent; all-Constant (nothing pending) → Constant; any pre-constanic child →
/// Braning (keep waiting — this is the one outcome that is NOT constanic); else any Econstanic/ Woconstanic
/// → Woconstanic; else any Nk → Nk.
/// Classifies a brane-like node's settled `Nyes` from its children's states.
///
/// **This function never returns NK.** A brane is NK if and only if it contains an unsteppable
/// statement (FOOP-94, as amended by the human 2026-09-23), and that NK is written directly by
/// `halt_if_unsteppable` rather than derived here. Brane NK is therefore a record of a run-time
/// error (FOOP-86 §6.2a), not a rollup of member states.
///
/// So an NK member never contaminates its brane, however many there are: a brane holding one
/// failed division — or holding nothing but failed divisions — still classifies conclusively,
/// because it did its own part correctly (§6.4c). The members stay NK and stay searchable; only
/// the container's classification changes.
///
/// NK does **not** count as Independent-like. A brane of Independents plus one NK classifies
/// CONSTANT, not INDEPENDENT — deliberately conservative: INDEPENDENT asserts "no context
/// dependencies at all", which an NK member's unresolved history does not support.
///
/// Cascade, first match wins:
///
/// | # | Condition | Result |
/// |---|-----------|--------|
/// | 1 | all INDEPENDENT | INDEPENDENT |
/// | 2 | all ∈ {CONSTANT, INDEPENDENT, NK} | CONSTANT |
/// | 3 | any pre-constanic | BRANING (keep stepping) |
/// | 4 | any ECONSTANIC/WOCONSTANIC | WOCONSTANIC |
fn decide_nyes_due_to_children(storage: &FVMStorage, children: &[FirPointer]) -> Option<Nyes> {
    let mut all_independent = true;
    let mut conclusive_or_nk = true;
    let mut preconstanic_count = 0usize;
    let mut econstanic_woconstanic_count = 0usize;

    for &c in children {
        let nyes = storage.get_nyes(c);
        if nyes != Nyes::Independent {
            all_independent = false;
        }
        match nyes {
            Nyes::Prembrionic | Nyes::Embryonic | Nyes::Braning => {
                preconstanic_count += 1;
                conclusive_or_nk = false;
            }
            Nyes::Econstanic | Nyes::Woconstanic => {
                econstanic_woconstanic_count += 1;
                conclusive_or_nk = false;
            }
            Nyes::Nk | Nyes::Constant | Nyes::Independent => {}
        }
    }

    if all_independent {
        Some(Nyes::Independent)
    } else if conclusive_or_nk {
        Some(Nyes::Constant)
    } else if preconstanic_count > 0 {
        Some(Nyes::Braning)
    } else if econstanic_woconstanic_count > 0 {
        Some(Nyes::Woconstanic)
    } else {
        unreachable!("ALARM: decide_nyes_due_to_children: no decision made.")
    }
}

/// `decide_nyes_due_to_children` for a BRANE, with the halt honoured.
///
/// A brane halted by an unsteppable statement is NK, and that NK is the record of a run-time
/// error rather than a classification of its members (FOOP-86 §6.2a, FOOP-94 as amended
/// 2026-09-23). Reclassifying it would erase the halt: since the classifier never returns NK,
/// a halted brane would be relabelled CONSTANT and its `!! NK:` annotation would vanish.
///
/// This was a live regression, caught by `foop/86/unsteppable_concat_merge`. It was previously
/// MASKED: the old any-NK-child rule happened to re-derive the same NK the halt had written, so
/// nothing appeared to depend on the ordering. Removing that rule exposed the overwrite.
fn decide_brane_nyes(storage: &FVMStorage, brane: FirPointer, children: &[FirPointer]) -> Option<Nyes> {
    if storage.unsteppable_cause(brane).is_some() {
        return Some(Nyes::Nk);
    }
    decide_nyes_due_to_children(storage, children)
}

/// Resolves an `Operator` node once all its operands are known: an NK
/// operand short-circuits to NK, a non-integer operand yields `Woconstanic`
/// (not yet resolvable), division/modulo by zero produce NK, and otherwise
/// the arithmetic result becomes a fresh `Constant` `IndepInt` child.
///
/// Building the result node directly under `ptr` needs no separate
/// build-standalone-then-reparent step: `create_child`/`make_orphan_child`
/// already produce an already-parented node, so `revive_constanic` (which
/// exists for relocating an *existing* subtree into a new context) has
/// nothing to do here.
fn combine(ptr: FirPointer, storage: &mut FVMStorage) {
    let op = match storage.get(ptr) {
        FirSpec::Operator { op } => op.clone(),
        other => unreachable!("combine called on non-Operator spec: {other:?}"),
    };
    let children: Vec<FirPointer> = storage.foolish_children(ptr).to_vec();

    let any_nk = children.iter().any(|&c| storage.get_nyes(c) == Nyes::Nk);
    if any_nk {
        let reason = children
            .iter()
            .find_map(|&c| {
                if storage.get_nyes(c) == Nyes::Nk {
                    let cursor = FirCursor::new(c, storage);
                    cursor.as_nk_reason().map(str::to_string)
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "operator nk".to_string());
        let nk_ptr = storage.make_orphan_child(ptr, FirSpec::Nk { reason });
        let me = storage.get_mut(ptr);
        me.push_ubc_child(nk_ptr, Nyes::Nk);
        me.set_nyes(Nyes::Nk);
        return;
    }

    let values: Vec<i64> = children
        .iter()
        .filter_map(|&c| FirCursor::new(c, storage).as_i64())
        .collect();

    if values.len() != children.len() {
        storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Woconstanic));
        return;
    }

    let result = match op.as_str() {
        "+" if values.len() == 2 => values[0] + values[1],
        "-" if values.len() == 2 => values[0] - values[1],
        "*" if values.len() == 2 => values[0] * values[1],
        "/" if values.len() == 2 => {
            if values[1] == 0 {
                let nk_ptr = storage.make_orphan_child(
                    ptr,
                    FirSpec::Nk {
                        reason: "division by zero".to_string(),
                    },
                );
                let me = storage.get_mut(ptr);
                me.push_ubc_child(nk_ptr, Nyes::Nk);
                me.set_nyes(Nyes::Nk);
                return;
            }
            values[0] / values[1]
        }
        "%" if values.len() == 2 => {
            if values[1] == 0 {
                let nk_ptr = storage.make_orphan_child(
                    ptr,
                    FirSpec::Nk {
                        reason: "division by zero".to_string(),
                    },
                );
                let me = storage.get_mut(ptr);
                me.push_ubc_child(nk_ptr, Nyes::Nk);
                me.set_nyes(Nyes::Nk);
                return;
            }
            values[0] % values[1]
        }
        "-" if values.len() == 1 => -values[0],
        _ => {
            // The compiler is the only producer of `Operator` specs and only ever uses known operators, so
            // an unknown one here means the compiler and this dispatch have fallen out of sync — an
            // internal-consistency bug, not a runtime error to propagate.
            unreachable!("combine: unknown operator {op:?} ({} operands)", values.len())
        }
    };

    let result_ptr = storage.make_orphan_child(ptr, FirSpec::IndepInt { value: result });
    storage.with_mut(result_ptr, |fir| fir.set_nyes(Nyes::Constant));
    let me = storage.get_mut(ptr);
    me.push_ubc_child(result_ptr, Nyes::Constant);
    me.set_nyes(Nyes::Constant);
}

/// A [`FirPointer`] paired with a borrow of the [`FVMStorage`] to read it through. Captures storage once so
/// a run of navigation calls on one node doesn't repeat `&storage` at every call. Read-only: cheap to
/// construct and multiple calls through one (or several at once) compose freely, the same as any shared
/// borrow.
#[derive(Clone, Copy)]
pub struct FirCursor<'s> {
    ptr: FirPointer,
    storage: &'s FVMStorage,
}

impl<'s> FirCursor<'s> {
    /// Wraps `ptr` for reading through `storage`.
    pub fn new(ptr: FirPointer, storage: &'s FVMStorage) -> Self {
        Self { ptr, storage }
    }

    /// The pointer this cursor wraps — for callers that need pointer IDENTITY (e.g. cycle detection while
    /// walking a pre-constanic, possibly self-referential tree), not just the node it addresses.
    pub fn ptr(&self) -> FirPointer {
        self.ptr
    }

    /// This node's [`FirSpec`].
    pub fn node(&self) -> &'s FirSpec {
        self.storage.get(self.ptr)
    }

    pub fn foolish_children(&self) -> &'s [FirPointer] {
        self.storage.foolish_children(self.ptr)
    }

    pub fn ubc_children(&self) -> &'s [FirPointer] {
        let index = self.storage.validate(self.ptr);
        self.storage.slots[index].payload.ubc_children()
    }

    /// `ubc_children` first (the evaluator renders these as `result=`), then
    /// `foolish_children` — this render order is load-bearing for output.
    pub fn all_children(&self) -> impl Iterator<Item = FirPointer> + 's {
        self.ubc_children().iter().chain(self.foolish_children()).copied()
    }

    /// `None` only for the true structural root — see [`FirPointer::get_parent`].
    pub fn parent(&self) -> Option<FirPointer> {
        self.ptr.get_parent(self.storage)
    }

    pub fn is_root(&self) -> bool {
        self.ptr.is_root(self.storage)
    }

    pub fn get_nyes(&self) -> Nyes {
        self.storage.get_nyes(self.ptr)
    }

    pub fn front_task(&self) -> Option<FirPointer> {
        let index = self.storage.validate(self.ptr);
        self.storage.slots[index].payload.front_task()
    }

    pub fn home_brane(&self) -> Option<FirCursor<'s>> {
        self.ptr
            .home_brane(self.storage)
            .map(|p| FirCursor::new(p, self.storage))
    }

    pub fn statement(&self) -> FirCursor<'s> {
        FirCursor::new(self.ptr.get_my_statement(self.storage), self.storage)
    }

    /// Applies the constanic gate itself: `None` unless this node is constanic.
    pub fn settled_constanic_result(&self) -> Option<FirCursor<'s>> {
        self.ptr
            .settled_constanic_result(self.storage)
            .map(|p| FirCursor::new(p, self.storage))
    }

    /// `IndepInt` reports its own value directly; every other kind falls through to its constanic result. A
    /// kind with no constanic result answers `None`.
    pub fn as_i64(&self) -> Option<i64> {
        match self.node() {
            FirSpec::IndepInt { value } => Some(*value),
            _ => self.settled_constanic_result().and_then(|c| c.as_i64()),
        }
    }

    pub fn as_nk_reason(&self) -> Option<&'s str> {
        match self.node() {
            FirSpec::Nk { reason } => Some(reason),
            _ => None,
        }
    }

    pub fn as_op_name(&self) -> Option<&'s str> {
        match self.node() {
            FirSpec::Operator { op } => Some(op),
            FirSpec::Comparison { op } => Some(op.searchable_name()),
            _ => None,
        }
    }

    pub fn as_stmt_identifier(&self) -> Option<&'s Identifier> {
        match self.node() {
            FirSpec::Statement { identifier, .. } => Some(identifier),
            _ => None,
        }
    }

    pub fn as_stmt_line_number(&self) -> Option<usize> {
        match self.node() {
            FirSpec::Statement { line_number, .. } => Some(*line_number),
            _ => None,
        }
    }

    /// `Brane` and `ConcatHelper` report their foolish-children count. `Concatenation` is `Some(0)` only
    /// when genuinely empty (no helper populated and no elements); otherwise it's the sum of every
    /// `ubc_children` helper's own `stmt_count` (summed generally, though in practice there is at most one
    /// helper). Every other kind is `None` (not brane-like).
    pub fn stmt_count(&self) -> Option<usize> {
        match self.node() {
            FirSpec::Brane { .. } | FirSpec::ConcatHelper => Some(self.foolish_children().len()),
            FirSpec::Concatenation { .. } => {
                if self.ubc_children().is_empty() && self.foolish_children().is_empty() {
                    return Some(0);
                }
                Some(
                    self.ubc_children()
                        .iter()
                        .map(|&h| FirCursor::new(h, self.storage).stmt_count().unwrap_or(0))
                        .sum(),
                )
            }
            _ => None,
        }
    }

    /// `Brane` and `ConcatHelper` index their foolish children directly. `Concatenation` walks its
    /// `ubc_children` helpers in order, subtracting each helper's own `stmt_count` from `idx` until it
    /// lands inside one, then delegates to that helper's `stmt_at`.
    pub fn stmt_at(&self, idx: usize) -> Option<FirPointer> {
        match self.node() {
            FirSpec::Brane { .. } | FirSpec::ConcatHelper => self.foolish_children().get(idx).copied(),
            FirSpec::Concatenation { .. } => {
                let mut remaining = idx;
                for &helper in self.ubc_children() {
                    let helper_cursor = FirCursor::new(helper, self.storage);
                    let count = helper_cursor.stmt_count().unwrap_or(0);
                    if remaining < count {
                        return helper_cursor.stmt_at(remaining);
                    }
                    remaining -= count;
                }
                None
            }
            _ => None,
        }
    }

    /// Mirrors `crate::fir_trait::Fir::as_brane_characterizations`: `Brane`'s own override returns its
    /// characterizations' components.
    pub fn as_brane_characterizations(&self) -> &'s [String] {
        match self.node() {
            FirSpec::Brane { characterizations } => characterizations.components(),
            _ => &[],
        }
    }

    pub fn is_brane_like(&self) -> bool {
        self.stmt_count().is_some()
    }

    pub fn as_search_pattern(&self) -> Option<&'s str> {
        match self.node() {
            FirSpec::Search { pattern, .. } => Some(pattern),
            _ => None,
        }
    }

    pub fn as_search_anchored(&self) -> bool {
        matches!(self.node(), FirSpec::Search { anchored: true, .. })
    }

    pub fn as_search_is_value(&self) -> bool {
        matches!(
            self.node(),
            FirSpec::Search {
                is_value_search: true,
                ..
            }
        )
    }

    pub fn as_search_contexted(&self) -> bool {
        match self.node() {
            FirSpec::Search { contexted, .. } | FirSpec::Index { contexted, .. } => *contexted,
            _ => false,
        }
    }

    pub fn as_index_offset(&self) -> i32 {
        match self.node() {
            FirSpec::Index { offset, .. } => *offset,
            _ => 0,
        }
    }

    pub fn as_index_anchored(&self) -> bool {
        matches!(self.node(), FirSpec::Index { anchored: true, .. })
    }

    pub fn as_fool_ref_referent(&self) -> Option<FirPointer> {
        match self.node() {
            FirSpec::FoolRef { referent } => Some(*referent),
            _ => None,
        }
    }

    pub fn as_concat_provenance(&self) -> ConcatProvenance {
        match self.node() {
            FirSpec::Concatenation { provenance, .. } => *provenance,
            _ => ConcatProvenance::Juxtaposition,
        }
    }

    pub fn as_creation_display_name(&self, viewed_from: Option<FirPointer>) -> Option<String> {
        if !matches!(self.node(), FirSpec::Creation) {
            return None;
        }
        self.ptr.get_display_name(self.storage, viewed_from?)
    }
}

/// The mutating counterpart of [`FirCursor`]. Rust allows only one `&mut` at
/// a time, so unlike `FirCursor` this does NOT support "wrap once, call five
/// mutating methods" — each mutating call still needs its own `&mut`
/// reborrow under the hood. Its value is bundling `ptr`+`storage` for ONE
/// logical mutating operation, not batching several ([`FVMStorage::get_mut`]
/// is for "several writes with nothing storage-needing in between").
///
/// **Must never be held live across a call into [`FirPointer::step`].**
/// Rust's borrow checker enforces this at compile time: holding a live
/// `FirCursorMut` (or any `&mut FVMStorage` borrow) across a recursive
/// `step` call simply fails to compile.
pub struct FirCursorMut<'s> {
    ptr: FirPointer,
    storage: &'s mut FVMStorage,
}

impl<'s> FirCursorMut<'s> {
    /// Wraps `ptr` for mutating through `storage`.
    pub fn new(ptr: FirPointer, storage: &'s mut FVMStorage) -> Self {
        Self { ptr, storage }
    }

    /// A FIR owns its own `nyes`; it must never be changed from outside the FIR. The ONLY sanctioned
    /// writers are (1) a FIR on ITSELF, inside its own `fir_op_step`, and (2) construction. `pub(crate)`,
    /// not `pub`, is the enforcement.
    pub(crate) fn set_nyes(&mut self, n: Nyes) {
        self.storage.get_mut(self.ptr).set_nyes(n);
    }

    pub fn create_child(&mut self, spec: FirSpec) -> FirPointer {
        self.ptr.create_child(self.storage, spec)
    }

    /// Pushes a parse-time child under an SF/SFF marker, panicking (unconditionally, not a `debug_assert!`)
    /// if any search-kind descendant of `child` is not exactly `ECONSTANIC` — an SFF body must be built
    /// entirely from unevaluated material, so a descendant search that already ran is an
    /// internal-consistency violation, not a recoverable condition. `child` must already be a child of
    /// `self.ptr`.
    pub fn check_sff_marked_child(&self, child: FirPointer) {
        if let Some(offender) = sift_for_first_non_econstanic_descendent_search(self.storage, child) {
            let spec = self.storage.get(offender);
            let nyes = self.storage.get_nyes(offender);
            panic!(
                "ubca INTERNAL CONSISTENCY error: SFF-marked child has a \
                 descendant {spec:?} search at {nyes:?}, expected ECONSTANIC. \
                 The `under_sff` construction rule (compiler::build_fir) did \
                 not reach it. An SFF body must be constanic-unevaluated — \
                 every descendant search kind must be built ECONSTANIC so it \
                 never runs. Refusing to continue: stepping this body would \
                 evaluate a search that must not run."
            );
        }
    }

    pub fn push_ubc_child(&mut self, child: FirPointer) {
        let child_nyes = self.storage.get_nyes(child);
        self.storage.get_mut(self.ptr).push_ubc_child(child, child_nyes);
    }

    pub fn push_search_result(&mut self, result: FirPointer) {
        let result_nyes = self.storage.get_nyes(result);
        self.storage
            .get_mut(self.ptr)
            .push_search_result(result, result_nyes);
    }

    /// Pushes a search RESULT and its `FoolRef` bookkeeping entry to `ubc_children`, in that order — the
    /// FoolRef two-child invariant: `[0]` is the value every reader accesses via `.first()`, `[1]` is
    /// invisible to them. `referent` is the ORIGINAL found statement, not the cloned result — a genuinely
    /// shared `FirPointer` (see `revive_constanic`'s `FoolRef`-always-shares rule, which this invariant
    /// depends on).
    pub fn push_search_result_pair(&mut self, result: FirPointer, referent: FirPointer) {
        let fool_ref = self.create_child(FirSpec::FoolRef { referent });
        self.storage
            .with_mut(fool_ref, |fir| fir.set_nyes(Nyes::Constant));
        self.push_search_result(result);
        self.push_ubc_child(fool_ref);
    }

    pub fn clear_ubc_children(&mut self) {
        self.storage.get_mut(self.ptr).clear_ubc_children();
    }

    pub fn pop_front_task(&mut self) {
        self.storage.get_mut(self.ptr).pop_front_task();
    }

    pub fn push_task(&mut self, t: FirPointer) {
        self.storage.get_mut(self.ptr).push_task(t);
    }
}

/// The first descendant search kind that is NOT exactly `Nyes::Econstanic`,
/// or `None` if every one of them is. The check is `== Econstanic`
/// specifically, not `is_constanic()`: an SFF-marked search sitting at
/// `Constant` or `Nk` means it DID run, which is exactly what this guard
/// exists to catch.
///
/// Naming: `sift_*`, not `search_*` — an ordinary Rust-side tree walk with no
/// Foolish search semantics.
fn sift_for_first_non_econstanic_descendent_search(
    storage: &FVMStorage,
    node: FirPointer,
) -> Option<FirPointer> {
    let is_search_kind = matches!(storage.get(node), FirSpec::Search { .. } | FirSpec::Index { .. });
    if is_search_kind && storage.get_nyes(node) != Nyes::Econstanic {
        return Some(node);
    }
    storage
        .foolish_children(node)
        .iter()
        .find_map(|&child| sift_for_first_non_econstanic_descendent_search(storage, child))
}

/// Ends `$handle`'s borrow, evaluates `$reacquire` (which may itself need
/// `&mut FVMStorage` — e.g. a nested `create_child` call), then re-acquires a
/// fresh handle via the same accessor and binds it back to `$handle`. No
/// `unsafe`, no magic: pure sugar over "drop the borrow, do the
/// storage-needing thing, get the borrow back," which is otherwise legal
/// Rust but visually noisy to write out by hand at every site that needs it.
///
/// `$handle` is a `&mut`-typed borrow (e.g. `&mut ProtoBrane` from
/// `FVMStorage::get_mut`), so ending its borrow is `let _ = $handle;`, not
/// `drop($handle)` — `drop` on a `&mut T` reference is a no-op (it drops the
/// reference value itself, a `Copy`-free but trivially-droppable pointer, not
/// the pointee), which `clippy::drop_ref`/rustc's own `dropping_references`
/// lint catches. `let _ = ...` genuinely ends the borrow's lifetime at that
/// point under NLL, which is the actual effect this macro needs.
#[macro_export]
macro_rules! temporary_release {
    ($handle:ident, $reacquire:expr, $body:expr) => {{
        let _ = $handle;
        let __result = $body;
        let $handle = $reacquire;
        (__result, $handle)
    }};
}

impl FVMStorage {
    /// Also known as **"constanic clone."** "Revival" is the right word for
    /// what this does: it makes a
    /// copy of an already-constanic FIR for use in a new context
    /// (AB/IB recoordination — a named brane referenced elsewhere and
    /// detached/recloned into that new site), and the copy is given an
    /// EARLIER `Nyes` than the original whenever the original's constanic
    /// state was *context-dependent* rather than self-contained: an `Econstanic`/
    /// `Woconstanic` node (typically a constanic search result) is regressed
    /// back to `Embryonic` in the copy — brought back to an earlier point in
    /// its own lifecycle — so it re-settles fresh against its new home
    /// rather than carrying over an answer that was only ever valid in the
    /// old one (see `Nyes::transform_for_clone`). A context-independent
    /// constanic value (`Constant`/`Independent`/`Nk`) needs no such
    /// revival — its answer doesn't depend on where it lives — and is
    /// simply shared as-is (case 1 below), never rebuilt with a new `Nyes`.
    ///
    /// Recursive, per-node, matching on the source's [`FirSpec`] — NOT a
    /// bulk subtree copy. Preserves:
    ///
    /// 1. **Share-not-clone.** `Constant`/`Independent` non-`Brane` nodes
    ///    return the SAME `FirPointer`, not a new slot. `FoolRef` and
    ///    `Creation` kinds ALWAYS share, unconditionally, regardless of NYES
    ///    state — this is what keeps the `FoolRefFir` two-child invariant's
    ///    original-statement reference genuinely shared, and a named
    ///    creation's identity intact.
    /// 2. **`StayFoolish`/`StayFullyFoolish` unwrapping** — checked FIRST,
    ///    before the share-not-clone check. `StayFoolish` tries its constanic
    ///    `ubc_children[0]` first; either kind falls through to its first
    ///    `foolish_children` entry; if both are empty, an `eprintln!` ALARM
    ///    fires and the wrapper clones as-is.
    /// 3. **Recursive per-node rebuild** for every other kind (this is where
    ///    the `Nyes`-regressing revival above actually happens, via
    ///    `Nyes::transform_for_clone`): children come from cloning each
    ///    `foolish_children`/`ubc_children` entry in turn, so the whole
    ///    subtree is rebuilt top-down, one recursive call per surviving node.
    ///
    /// `index` becomes a cloned `Statement`'s new `line_number` — used
    /// directly as the position, not carried over from the original.
    ///
    /// A pointer into the original subtree remains valid after a clone: this
    /// method only ADDS new slots for the freshly-rebuilt nodes; the
    /// original subtree's slots are untouched.
    pub fn revive_constanic(
        &mut self,
        root: FirPointer,
        new_parent: FirPointer,
        index: usize,
        sfm: bool,
        skip_foolish_children: bool,
    ) -> FirPointer {
        // StayFoolish/StayFullyFoolish unwrapping — checked FIRST, before the share-not-clone check below.
        // Only `StayFoolish` (not `StayFullyFoolish`) tries its constanic `ubc_children[0]` first; either
        // kind falls through to its first `foolish_children` entry; if BOTH are empty, this logs an ALARM
        // and falls through to clone the wrapper as-is via the normal share/rebuild logic below.
        let spec = self.get(root).clone();
        if matches!(spec, FirSpec::StayFoolish | FirSpec::StayFullyFoolish) {
            if matches!(spec, FirSpec::StayFoolish)
                && let Some(result) = FirCursor::new(root, self).ubc_children().first().copied()
            {
                return self.revive_constanic(result, new_parent, index, sfm, skip_foolish_children);
            }
            if let Some(inner) = self.foolish_children(root).first().copied() {
                return self.revive_constanic(inner, new_parent, index, sfm, skip_foolish_children);
            }
            eprintln!("ALARM: SF/SFF node has no children — cloning wrapper as-is");
        }

        let nyes = self.get_nyes(root);
        let spec = self.get(root).clone();

        // Share-not-clone: Constant/Independent non-Brane always shares; FoolRef/Creation always share
        // regardless of NYES. The shared pointer is NOT reparented (its own `.parent` stays exactly as it
        // was) but IS appended to `new_parent`'s `foolish_children` list (`attach_shared_foolish_child`) —
        // without this append, a caller like `populate_concat_helpers` that walks `new_parent`'s
        // `foolish_children` afterward would silently see the shared child missing, even though
        // `revive_constanic` reported success.
        let is_share_kind = matches!(spec, FirSpec::FoolRef { .. } | FirSpec::Creation);
        let is_conclusive_non_brane = nyes.is_conclusive() && !matches!(spec, FirSpec::Brane { .. });
        if is_share_kind || is_conclusive_non_brane {
            self.attach_shared_foolish_child(new_parent, root);
            return root;
        }

        // Recursive per-node rebuild. The new node's own spec is the source's spec, with a Statement's
        // line_number renumbered to `index`.
        let new_spec = match spec {
            FirSpec::Statement { identifier, .. } => FirSpec::Statement {
                identifier,
                line_number: index,
            },
            other => other,
        };
        let clone_nyes = nyes.transform_for_clone(sfm);
        let new_ptr = new_parent.create_child(self, new_spec);
        self.with_mut(new_ptr, |fir| fir.set_nyes(clone_nyes));
        // FOOP-86 §6.4b: a clone of a HALTED brane is still a halted brane. Without carrying the
        // cause, an access whose anchor resolves through a clone (`A^`, `A#0`, a plain `r = A`) would
        // find an ordinary brane and read its contents, when §6.4b requires every access into an NK
        // brane to settle NK. The cause travels with the clone so it answers "I am unsteppable"
        // exactly as the original does.
        if let Some(cause) = self.unsteppable_cause(root) {
            self.set_unsteppable_cause(new_ptr, cause);
        }

        if !skip_foolish_children {
            let children: Vec<FirPointer> = self.foolish_children(root).to_vec();
            for (i, child) in children.into_iter().enumerate() {
                self.revive_constanic(child, new_ptr, i, sfm, false);
            }
        }
        let ubc_children: Vec<FirPointer> = {
            let index_in_slots = self.validate(root);
            self.slots[index_in_slots].payload.ubc_children().to_vec()
        };
        for ubc in ubc_children {
            // `create_child`'s parent construction path always appends the new pointer to `new_parent`'s
            // `foolish_children` — correct for rebuilding parse-time topology, but a UBC-CHILD clone
            // belongs ONLY in `ubc_children`, never `foolish_children`. Pop the wrongly-appended entry back
            // off before recording it correctly below.
            let cloned = self.revive_constanic(ubc, new_ptr, 0, sfm, false);
            let index_in_slots = self.validate(new_ptr);
            let fc = &mut self.slots[index_in_slots].foolish_children;
            if fc.last() == Some(&cloned) {
                fc.pop();
            }
            let cloned_nyes = self.get_nyes(cloned);
            self.with_mut(new_ptr, |fir| fir.push_ubc_child(cloned, cloned_nyes));
        }
        new_ptr
    }
}

/// Branch order: NK-on-either-side → Unknowable; both-integers → compare;
/// else resolve `.value()` and compare kind (`Creation`-vs-`Creation` →
/// pointer identity; `Brane`-vs-`Brane` → Unknowable; anything else →
/// NotEqual).
///
/// Kind discrimination is done directly on [`FirSpec`], which already
/// carries the same information a separate `kind()`-style accessor would.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Equality {
    Equal,
    NotEqual,
    Unknowable,
}

pub(crate) fn default_equal(storage: &FVMStorage, a: FirPointer, b: FirPointer) -> Equality {
    if storage.get_nyes(a) == Nyes::Nk || storage.get_nyes(b) == Nyes::Nk {
        return Equality::Unknowable;
    }
    if let (Some(av), Some(bv)) = (
        FirCursor::new(a, storage).as_i64(),
        FirCursor::new(b, storage).as_i64(),
    ) {
        return if av == bv {
            Equality::Equal
        } else {
            Equality::NotEqual
        };
    }
    // Resolve through to the constanic value (e.g. a search reference to a
    // creation resolves to the CreationFir it found) before comparing kinds.
    // `.value()` is a no-op for FIRs that are already their own value.
    let a_resolved = a.value(storage);
    let b_resolved = b.value(storage);
    let a_spec = storage.get(a_resolved);
    let b_spec = storage.get(b_resolved);
    if matches!(a_spec, FirSpec::Creation) && matches!(b_spec, FirSpec::Creation) {
        return if a_resolved == b_resolved {
            Equality::Equal
        } else {
            Equality::NotEqual
        };
    }
    // Two branes: brane-vs-brane equivalence is unspecified (FOOP-23) → genuinely unknowable.
    if matches!(a_spec, FirSpec::Brane { .. }) && matches!(b_spec, FirSpec::Brane { .. }) {
        return Equality::Unknowable;
    }
    // Different non-NK constanic kinds (brane-vs-integer, integer-vs-creation, etc.)
    // are provably not equal — a brane is never an integer (different FIR kinds, decidable).
    // The matcher should Reject (skip) and continue scanning, not NkStop (abort).
    Equality::NotEqual
}

pub(crate) mod search_engine;

mod search_fir_dispatch;

pub mod stepping;

mod arena_compiler;

/// Minimal re-export surface for `UbcaEvaluator::evaluate` — `arena_compiler` stays a private
/// module, while `stepping` is `pub` as the project's debugging API; only the exact functions
/// `evaluate`'s body needs are re-exported here, not the modules' full surface.
pub(crate) use arena_compiler::{compose_program_with_system, program_result};
pub(crate) use stepping::step_to_constanic;

#[cfg(test)]
mod tests;
