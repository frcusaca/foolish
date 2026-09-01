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
    /// Compute-time children (search results, resolved references — as
    /// opposed to `foolish_children`'s fixed parse-time topology). A plain
    /// `Vec`: the arena's `&mut FVMStorage` borrow is the only exclusivity
    /// check a mutator needs, so no interior mutability is required here.
    ubc_children: Vec<FirPointer>,
    /// Task queue driving this node's stepping.
    tasks: VecDeque<FirPointer>,
    alarm_reason: Option<String>,
    /// Set only on `FirSpec::Statement` nodes, by the null-characterized
    /// name-constant rule (`check_null_const_conflict`) or the named-creation
    /// no-rename rule (`check_rename_of_named_creation`) — both discovered
    /// during `fir_op_step`, never known at construction time, which is why
    /// this lives as a runtime-set field rather than a `FirSpec::Statement`
    /// constructor input. `None` in the common case; once set, terminal
    /// (never cleared) — a statement refused once stays refused.
    nf_reason: Option<String>,
    /// Mirrors `ConcatenationFir::_helpers_populated`. Applies ONLY to
    /// `FirSpec::Concatenation` nodes. A monotonic one-way gate distinct
    /// from "`ubc_children` is non-empty": the real field must flip `true`
    /// even on the ZERO-LINES-TO-MERGE path (`populate_concat_helpers`
    /// pushes no helper at all when every element resolves to an empty
    /// brane), so the SECOND `Braning` re-entry can settle from the
    /// (empty) helper set rather than re-attempting the merge forever.
    /// `false` for every other kind, always.
    helpers_populated: bool,
}

impl ProtoBrane {
    /// No caller yet — kept as the symmetric counterpart to [`Self::set_nyes`]
    /// for code that already holds an `&ProtoBrane` (e.g. inside a
    /// `with_mut`/`get_mut` closure) and would otherwise have to route back
    /// through `FVMStorage` just to read what it already has in hand.
    #[expect(
        dead_code,
        reason = "no caller yet — symmetric counterpart to set_nyes"
    )]
    pub(crate) fn get_nyes(&self) -> Nyes {
        self.nyes
    }

    /// A FIR owns its own `nyes`; it must never be changed from outside the
    /// FIR. `pub(crate)`, not `pub`, is the enforcement: only a node's own
    /// `fir_op_step` or its own construction may call this.
    pub(crate) fn set_nyes(&mut self, n: Nyes) {
        self.nyes = n;
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

    /// Takes the child's current `Nyes` as a parameter, rather than looking
    /// it up itself, because `ProtoBrane` cannot reach across arena slots to
    /// read another node's state — the caller
    /// ([`FirCursorMut::push_ubc_child`]) already has `&FVMStorage` access
    /// to read it first.
    pub(crate) fn push_ubc_child(&mut self, child: FirPointer, child_nyes: Nyes) {
        self.ubc_children.push(child);
        if !child_nyes.is_constanic() {
            self.tasks.push_back(child);
        }
    }

    /// A search settles with at most one result ever pushed to
    /// `ubc_children` (the singular-result invariant); a second push
    /// indicates a search re-resolving after already settling, a logic
    /// error rather than a legitimate re-evaluation.
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

    /// `None` unless this is a `FirSpec::Statement` node that a
    /// null-characterized-name rule has refused.
    pub(crate) fn nf_reason(&self) -> Option<&str> {
        self.nf_reason.as_deref()
    }

    /// The caller must check `nf_reason().is_some()` itself first — this
    /// setter does not guard against re-setting, so a refusal already
    /// recorded is never re-alarmed only if the caller checks before calling.
    pub(crate) fn set_nf_reason(&mut self, reason: String) {
        self.nf_reason = Some(reason);
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

/// How a concatenation was spelled in source. Affects SEQUENCING ONLY — never
/// evaluation (FOOP-65 §5.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConcatProvenance {
    /// Ordinary brane concatenation (juxtaposition): `{a}{b}{c}`.
    Juxtaposition,
    /// Tail concatenation (backtick chain): `` c`b`a `` — the elements are
    /// already stored REVERSED relative to source (FOOP-65 §5.2).
    TailConcatenation,
}

/// The name used for an anonymous statement (a bare expression with no LHS
/// identifier). The sequencer renders a statement named `???` WITHOUT a
/// `name=` prefix (FOOP-62 #19).
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
    /// (already settled). Every other kind depends on stepping (its own
    /// computation, or its children's) to reach a settled value, so it
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

    /// Retrieve this pointer's NF (Not Foolish) reason, if any (FOOP-33 §4).
    /// `None` for every kind other than `FirSpec::Statement`, and `None`
    /// there too unless a null-characterized-name rule has refused this
    /// statement. Consulted by [`FirPointer::settled_result`] to substitute
    /// the refusal NK in place of the written body — see that method's doc
    /// comment.
    pub fn nf_reason(&self, ptr: FirPointer) -> Option<&str> {
        let index = self.validate(ptr);
        self.slots[index].payload.nf_reason()
    }

    /// Terminal (FOOP-33 §4) — the caller owns the "already set" guard, see
    /// `ProtoBrane::set_nf_reason`.
    pub(crate) fn set_nf_reason(&mut self, ptr: FirPointer, reason: String) {
        let index = self.validate(ptr);
        self.slots[index].payload.set_nf_reason(reason);
    }

    /// Retrieve, modify, and return in one call — the "retrieve a payload, be
    /// able to modify it before returning" primitive. Closure-scoped so there
    /// is no separate get/set pair to keep in sync, and no `RefCell`-style
    /// runtime borrow tracking is needed: the `&mut self` borrow on
    /// `FVMStorage` is the only exclusivity check required. `pub(crate)`:
    /// `ProtoBrane` is this module's own internal payload type, never exposed
    /// outside it.
    pub(crate) fn with_mut<R>(
        &mut self,
        ptr: FirPointer,
        f: impl FnOnce(&mut ProtoBrane) -> R,
    ) -> R {
        let index = self.validate(ptr);
        f(&mut self.slots[index].payload)
    }

    /// Retrieve one exclusive, held `&mut ProtoBrane` for a run of several
    /// SEQUENTIAL writes with nothing storage-needing interleaved between
    /// them — the same capability as `with_mut`, offered as a plain borrow
    /// rather than a closure; the choice between the two is style, not
    /// capability.
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

    /// Appends an ALREADY-EXISTING pointer to `parent`'s `foolish_children`
    /// list WITHOUT allocating a new slot and WITHOUT reparenting `child`
    /// (its own `.parent` field, and therefore its home brane and line
    /// number, are left exactly as they were). This is `revive_constanic`'s
    /// "share-not-clone" path's other half: sharing a node means its own
    /// parent link stays untouched, but the NEW parent's `foolish_children`
    /// list must still record the shared pointer as one of its children —
    /// without this append, a caller like `populate_concat_helpers` that
    /// walks the new parent's `foolish_children` afterward would silently
    /// see the shared child missing, even though the share reported success.
    pub(crate) fn attach_shared_foolish_child(&mut self, parent: FirPointer, child: FirPointer) {
        self.validate(parent);
        self.validate(child);
        self.slots[parent.index as usize]
            .foolish_children
            .push(child);
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

    /// Shared slot-push logic for [`Self::make_my_child`] and
    /// [`Self::make_root`]: constructs the new `FirPointer`, pushes its
    /// `Slot`, and returns the pointer. Does NOT wire the new pointer into any
    /// parent's child list — callers do that themselves (`make_my_child` does;
    /// `make_root` has no parent to wire into).
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
                nf_reason: None,
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
    /// Creates a fresh arena containing a single self-rooting leaf. A leaf
    /// here is an `IndepInt` — the simplest kind with no interesting
    /// children — at the given `Nyes`.
    pub(crate) fn test_leaf(nyes: Nyes) -> (Self, FirPointer) {
        let mut storage = Self::new();
        let ptr = storage.make_root(FirSpec::IndepInt { value: 0 });
        storage.with_mut(ptr, |fir| fir.set_nyes(nyes));
        (storage, ptr)
    }

    /// Creates a fresh arena containing a root `Brane` with the given
    /// children specs.
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
    /// The primary construction call site. Delegates to
    /// [`FVMStorage::make_my_child`].
    pub fn create_child(self, storage: &mut FVMStorage, spec: FirSpec) -> FirPointer {
        storage.make_my_child(self, spec)
    }

    /// This pointer's parent, per the arena's stored parent link. Always
    /// `Some` in practice — even the structural root's "parent" is itself —
    /// since the arena never drops a live node out from under a valid
    /// pointer. `Option` is kept in the signature for callers that need to
    /// distinguish the root case explicitly.
    pub fn get_parent(self, storage: &FVMStorage) -> Option<FirPointer> {
        Some(storage.parent(self))
    }

    /// Whether this pointer is the structural root of its arena (its own
    /// parent).
    pub fn is_root(self, storage: &FVMStorage) -> bool {
        storage.parent(self) == self
    }

    /// Climbs the parent chain to the first brane-like kind: climb until
    /// `parent()` pointer-equals `self` (structural root) → `None`; else
    /// check brane-likeness → stop, else recurse.
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

    /// The statement this pointer's search would read as its position:
    /// climb until a `Statement` kind is found, or until `parent()`
    /// pointer-equals `self` (structural root, returned as-is).
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

    /// The settled result this pointer resolves to, if any. Applies the
    /// constanic gate itself — pre-constanic always answers `None`.
    /// `pub(crate)`: also called directly by
    /// `search_fir_dispatch::statement_value_for_comparison`, a nested
    /// module.
    pub(crate) fn settled_result(self, storage: &FVMStorage) -> Option<FirPointer> {
        if !storage.get_nyes(self).is_constanic() {
            return None;
        }
        let index = storage.validate(self);
        storage.slots[index].payload.ubc_children().first().copied()
    }

    /// Recursively unwraps through `settled_result`, returning `self` when
    /// there is none.
    pub fn value(self, storage: &FVMStorage) -> FirPointer {
        match self.settled_result(storage) {
            Some(child) => child.value(storage),
            None => self,
        }
    }

    /// Performs ONE stepping action: if the front task is already constanic,
    /// pop it; otherwise recurse into it. Once there is no front task left,
    /// calls this node's own `fir_op_step`.
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
        // Condition 2: only a null-characterized (protected-constant) name
        // qualifies at all.
        if !identifier.is_nully_characterizing_coordinate_name() {
            return None;
        }
        let name = identifier.searchable_name().to_owned();
        // Condition 1: never report the name when viewed from the
        // creation's own defining statement -- only from a different
        // statement (a reference reached elsewhere).
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

    /// Climbs the parent chain from `self` until a `Statement` kind is
    /// found, then returns that statement together with its home brane.
    /// `None` if the climb reaches the structural root without finding one.
    pub fn find_enclosing_stmt_and_brane(
        self,
        storage: &FVMStorage,
    ) -> Option<(FirPointer, FirPointer)> {
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

/// Guard against runaway recursion on pathologically deep trees.
const MAX_DEPTH: usize = 100;

/// Carries `step`'s scope down the tree: `current_statement`/`current_brane`
/// (the IB/AB search anchors) and `has_ancestral_sfm` (threaded through to
/// `clone_stmt_result`/`revive_constanic` by `IndexFir`'s contexted/anchored
/// dispatch).
#[derive(Debug, Clone, Copy, Default)]
struct ArenaScope {
    current_statement: Option<FirPointer>,
    current_brane: Option<FirPointer>,
    has_ancestral_sfm: bool,
}

/// Recursion companion for [`FirPointer::step`], carrying the depth counter.
fn step_inner(
    ptr: FirPointer,
    storage: &mut FVMStorage,
    scope: ArenaScope,
    depth: usize,
) -> FirPointer {
    if depth > MAX_DEPTH {
        return ptr;
    }
    let front = storage.with_mut(ptr, |fir| fir.front_task());
    match front {
        Some(front_ptr) => {
            if storage.get_nyes(front_ptr).is_constanic() {
                storage.with_mut(ptr, |fir| fir.pop_front_task());
            } else {
                // Set has_ancestral_sfm when `ptr` is StayFoolish; set
                // current_statement when `ptr` (the node ABOUT TO RECURSE
                // INTO ITS CHILD) is a Statement; set current_brane when
                // `ptr` is brane-like. All three are set on `ptr`'s own
                // scope before recursing into `front_ptr`.
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

/// Enum dispatch for `fir_op_step`, one arm per [`FirSpec`] variant
/// (rust_instructions.md §7: preferred over `dyn` when the variant set is
/// closed and known). The match is exhaustive with no fallback arm, so a
/// future 15th kind added without its own arm is a compile error naming the
/// missing variant, not a silent or panicking catch-all.
fn fir_op_step(ptr: FirPointer, storage: &mut FVMStorage, scope: ArenaScope) {
    let spec = storage.get(ptr).clone();
    match spec {
        // An IndepInt has no children or tasks, so there is no Braning
        // phase — one step settles it.
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
        // `Operator` is not brane-like — it has no `stmt_count`/
        // `is_brane_like` override.
        FirSpec::Operator { .. } => match storage.get_nyes(ptr) {
            Nyes::Prembrionic | Nyes::Embryonic => {
                storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Braning));
                let children: Vec<FirPointer> = storage.foolish_children(ptr).to_vec();
                let all_settled = children
                    .iter()
                    .all(|&c| matches!(storage.get_nyes(c), Nyes::Constant | Nyes::Independent));
                if !all_settled {
                    for child in children {
                        storage.with_mut(ptr, |fir| fir.push_task(child));
                    }
                }
            }
            Nyes::Braning => combine(ptr, storage),
            _ => {}
        },
        // Push the body as a task; once it's constanic, adopt its NYES. If
        // this statement's name is null-characterized, run the FOOP-33 §4
        // refusal checks (`check_null_const_conflict`/
        // `check_rename_of_named_creation`) first — a name-constant
        // redefinition or a named-creation rename is caught and recorded via
        // `nf_reason` before the body's value is adopted.
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
                            // `check_null_const_conflict`'s `ib_search_by_pattern` call
                            // wants the SEARCHING STATEMENT (it derives the home brane
                            // and index from `ptr` itself), but its
                            // `ab_search_by_pattern` call wants the STARTING BRANE that
                            // `_ab_search` climbs from. Passing the statement itself
                            // there would make `current_brane.get_my_statement() ==
                            // current_brane` trivially true (a statement's own home
                            // statement is itself) and short-circuit to `None`
                            // immediately — so `ptr`'s home brane, not `ptr`, goes to
                            // the `ab_search_by_pattern` call.
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
                        storage.with_mut(ptr, |fir| fir.set_nyes(body_nyes));
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
                let children: Vec<FirPointer> = storage.foolish_children(ptr).to_vec();
                if let Some(nyes) = decide_nyes_due_to_children(storage, &children) {
                    storage.with_mut(ptr, |fir| fir.set_nyes(nyes));
                }
            }
            _ => {}
        },
        // A no-op — a `FoolRef` is born `Constant` at construction (see
        // `push_search_result_pair`) and never needs stepping.
        FirSpec::FoolRef { .. } => {}
        // Once its wrapped `expr` is constanic, expose EXPR'S OWN resolved
        // value (its `ubc_children[0]`, or `expr` itself if it has none) as
        // this node's own `ubc_children[0]`, adopting that value's `Nyes`.
        // SF unwraps to a shared value; it never produces a genuinely new
        // node of its own.
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
                        let (result, result_nyes) = match FirCursor::new(expr, storage)
                            .ubc_children()
                            .first()
                            .copied()
                        {
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
        // Same value-unwrap shape as `StayFoolish`, with two differences:
        // (1) SFF always moves to `Braning` and pushes tasks
        // unconditionally — there is no empty-children short-circuit; (2)
        // the settled `Nyes` goes through `nyes_from_found`: an SFF wrapper
        // can never itself be Econstanic — an Econstanic result means SFF is
        // still WAITING on it (Woconstanic), while the pushed result keeps
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
                    let (result, result_nyes) = match FirCursor::new(expr, storage)
                        .ubc_children()
                        .first()
                        .copied()
                    {
                        Some(r) => (r, storage.get_nyes(r)),
                        None => (expr, expr_nyes),
                    };
                    let settled_nyes = nyes_from_found(result_nyes);
                    let me = storage.get_mut(ptr);
                    me.push_ubc_child(result, result_nyes);
                    me.set_nyes(settled_nyes);
                }
            }
            _ => {}
        },
        // Same shape as `Brane`'s arm — a `ConcatHelper` is transparent,
        // inheriting brane-shaped stepping.
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
                if let Some(nyes) = decide_nyes_due_to_children(storage, &children) {
                    storage.with_mut(ptr, |fir| fir.set_nyes(nyes));
                }
            }
            _ => {}
        },
        // Every element must resolve to a brane, or the whole concatenation
        // is NK (with the offending indexes named); if some elements are
        // still unresolved (not yet brane-like but not a type error either)
        // the concatenation waits (`Woconstanic`). Once every element is
        // brane-like, `populate_concat_helpers` builds and joins the merged
        // lines exactly once (`helpers_populated` below is that gate).
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

                // Two-pass: the first pass builds the helper(s) and pushes
                // them as tasks, staying pre-constanic so the driver drains
                // the helpers' own stepping before re-entry; the second pass
                // settles from the DRAINED helper results (the
                // joined/recoordinated copies), not the raw elements.
                let already_populated = storage.get_mut(ptr).helpers_populated();
                if !already_populated {
                    storage.get_mut(ptr).set_helpers_populated();
                    search_fir_dispatch::populate_concat_helpers(storage, ptr);
                    let helpers: Vec<FirPointer> =
                        FirCursor::new(ptr, storage).ubc_children().to_vec();
                    for helper in helpers {
                        storage.with_mut(ptr, |fir| fir.push_task(helper));
                    }
                } else {
                    let helpers: Vec<FirPointer> =
                        FirCursor::new(ptr, storage).ubc_children().to_vec();
                    let settled = if helpers.is_empty() {
                        Nyes::Constant
                    } else {
                        decide_nyes_due_to_children(storage, &helpers).unwrap_or(Nyes::Constant)
                    };
                    storage.with_mut(ptr, |fir| fir.set_nyes(settled));
                }
            }
            _ => {}
        },
        // A no-op — a creation is born `Independent` at construction and
        // never needs stepping.
        FirSpec::Creation => {}
        // Same two-phase shape as `Operator`: push operands, then combine
        // once Braning.
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

                // Read each operand THROUGH its SFF wrapper: `.value()`
                // follows the settled chain to whatever the recoordinated
                // index landed on.
                let values: Vec<Option<i64>> = operands
                    .iter()
                    .map(|&o| {
                        let resolved = o.value(storage);
                        FirCursor::new(resolved, storage).as_i64()
                    })
                    .collect();

                let (Some(&Some(left)), Some(&Some(right))) = (values.first(), values.get(1))
                else {
                    // The operands DID evaluate here, and at least one is
                    // not an integer. Only integers are comparable (FOOP-33
                    // §5, same principle `default_equal` follows).
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
                // Resolve `'True`/`'False` by ordinary ancestral search from
                // THIS comparison's own position — it lives inside
                // system.foo, so the search finds the very creations
                // system.foo declares (FOOP-33 §5: referentially identical
                // to a user's own `'True` reference).
                let name = if verdict { "'True" } else { "'False" };
                let home_brane = ptr.home_brane(storage);
                let boolean = search_fir_dispatch::ab_search_by_pattern(storage, name, home_brane)
                    .and_then(|(found, _)| {
                        search_fir_dispatch::statement_value_for_comparison(storage, found)
                    })
                    .map(|body| body.value(storage));
                let Some(boolean) = boolean else {
                    // system.foo always defines 'True/'False; failing to
                    // find one means the prelude itself is malformed — an
                    // interpreter defect, not an unevaluable program. No
                    // `Result` to propagate through `fir_op_step`'s arena
                    // signature yet (same convention as `IndexFir`'s
                    // unanchored-offset invariant panic above), so this
                    // states the same invariant via `panic!`.
                    panic!(
                        "system.foo must define 'True and 'False, but {} could not resolve one",
                        op.searchable_name()
                    );
                };
                let clone =
                    storage.revive_constanic(boolean, ptr, 0, scope.has_ancestral_sfm, false);
                let mut cursor = FirCursorMut::new(ptr, storage);
                cursor.push_ubc_child(clone);
                cursor.set_nyes(Nyes::Constant);
            }
            _ => {}
        },
        FirSpec::Search {
            is_value_search, ..
        } => {
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
                        // An unanchored non-negative offset is a
                        // construction-time invariant violation the compiler
                        // itself should never produce — only
                        // `Astn::HeadTail`/`Astn::UnanchoredSeek` build
                        // unanchored `Index` nodes, and always with negative
                        // offsets — so this is a bug to panic on, not a
                        // reachable runtime program state.
                        assert!(offset < 0, "unanchored index requires negative offset");
                        match ptr.find_enclosing_stmt_and_brane(storage) {
                            Some((stmt_ref, brane_ref)) => {
                                match brane_ref.find_stmt_index(storage, stmt_ref) {
                                    Some(idx) => {
                                        let target = idx as i32 + offset;
                                        let len = FirCursor::new(brane_ref, storage)
                                            .stmt_count()
                                            .unwrap_or(0)
                                            as i32;
                                        if target < 0 || target >= len {
                                            storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Nk));
                                        } else {
                                            let mut nav = search_engine::BraneNavigator::new(
                                                storage, brane_ref, true,
                                            );
                                            let predicate =
                                                search_engine::SearchPredicate::Index(target);
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
                        let fool_ref_fir = FirCursor::new(anchor, storage)
                            .ubc_children()
                            .get(1)
                            .copied();
                        let contexted_result = fool_ref_fir.and_then(|frf| {
                            let referent = FirCursor::new(frf, storage).as_fool_ref_referent()?;
                            let h_brane = referent.home_brane(storage)?;
                            let p = h_brane.find_stmt_index(storage, referent)?;
                            let target = p as i32 + offset;
                            let len =
                                FirCursor::new(h_brane, storage).stmt_count().unwrap_or(0) as i32;
                            if target < 0 || target >= len {
                                return None;
                            }
                            let mut nav =
                                search_engine::BraneNavigator::new(storage, h_brane, true);
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
                        if !FirCursor::new(resolved, storage).is_brane_like() {
                            // FOOP-75 §7: settling NK is only half the
                            // answer — name the offending anchor when it is
                            // itself nameable (an integer literal), so the
                            // rendered output reads `d =$ ??? (4 is not a
                            // brane)` instead of a bare miss. When the anchor is
                            // some other FIR (commonly a search that itself
                            // settled NK), there is no value to name — leave the
                            // result unset so the existing failed-anchor
                            // rendering shows through unchanged.
                            let named = FirCursor::new(resolved, storage)
                                .as_i64()
                                .map(|v| v.to_string());
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

/// Remaps a found node's `Nyes` for the caller's own settlement:
/// Econstanic/Woconstanic → Woconstanic; Constant/Independent → Constant;
/// Nk → Nk; anything else (pre-constanic) passes through unchanged.
fn nyes_from_found(found: Nyes) -> Nyes {
    match found {
        Nyes::Econstanic | Nyes::Woconstanic => Nyes::Woconstanic,
        Nyes::Constant | Nyes::Independent => Nyes::Constant,
        Nyes::Nk => Nyes::Nk,
        other => other,
    }
}

/// Classifies a Braning node's settled state from its children's states, in
/// priority order: all-Independent → Independent; all-Constant (nothing
/// pending) → Constant; any pre-constanic child → Braning (keep waiting);
/// else any Econstanic/Woconstanic → Woconstanic; else any Nk → Nk.
fn decide_nyes_due_to_children(storage: &FVMStorage, children: &[FirPointer]) -> Option<Nyes> {
    let mut all_constant = true;
    let mut all_independent = true;
    let mut preconstanic_count = 0usize;
    let mut nk_count = 0usize;
    let mut econstanic_woconstanic_count = 0usize;

    for &c in children {
        match storage.get_nyes(c) {
            Nyes::Prembrionic | Nyes::Embryonic | Nyes::Braning => {
                preconstanic_count += 1;
                all_constant = false;
                all_independent = false;
            }
            Nyes::Nk => {
                nk_count += 1;
                all_constant = false;
                all_independent = false;
            }
            Nyes::Econstanic | Nyes::Woconstanic => {
                econstanic_woconstanic_count += 1;
                all_constant = false;
                all_independent = false;
            }
            Nyes::Constant => {
                all_independent = false;
            }
            _ => {}
        }
    }
    if all_independent {
        Some(Nyes::Independent)
    } else if all_constant {
        Some(Nyes::Constant)
    } else if preconstanic_count > 0 {
        Some(Nyes::Braning)
    } else if econstanic_woconstanic_count > 0 {
        Some(Nyes::Woconstanic)
    } else if nk_count > 0 {
        Some(Nyes::Nk)
    } else {
        unreachable!("ALARM: decide_nyes_due_to_children: no decision made.")
    }
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
            // The compiler is the only producer of `Operator` specs and only
            // ever uses known operators, so an unknown one here means the
            // compiler and this dispatch have fallen out of sync — an
            // internal-consistency bug, not a runtime error to propagate.
            unreachable!(
                "combine: unknown operator {op:?} ({} operands)",
                values.len()
            )
        }
    };

    let result_ptr = storage.make_orphan_child(ptr, FirSpec::IndepInt { value: result });
    storage.with_mut(result_ptr, |fir| fir.set_nyes(Nyes::Constant));
    let me = storage.get_mut(ptr);
    me.push_ubc_child(result_ptr, Nyes::Constant);
    me.set_nyes(Nyes::Constant);
}

/// A [`FirPointer`] paired with a borrow of the [`FVMStorage`] to read it
/// through. Captures storage once so a run of navigation calls on one node
/// doesn't repeat `&storage` at every call. Read-only: cheap to construct and
/// multiple calls through one (or several at once) compose freely, the same
/// as any shared borrow.
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
        self.ubc_children()
            .iter()
            .chain(self.foolish_children())
            .copied()
    }

    /// `None` only for the true structural root — see
    /// [`FirPointer::get_parent`].
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

    /// Applies the constanic gate itself: `None` unless this node is
    /// constanic.
    pub fn settled_result(&self) -> Option<FirCursor<'s>> {
        self.ptr
            .settled_result(self.storage)
            .map(|p| FirCursor::new(p, self.storage))
    }

    /// `IndepInt` reports its own value directly; every other kind falls
    /// through to its settled result. A kind with no settled result answers
    /// `None`.
    pub fn as_i64(&self) -> Option<i64> {
        match self.node() {
            FirSpec::IndepInt { value } => Some(*value),
            _ => self.settled_result().and_then(|c| c.as_i64()),
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

    /// `Brane` and `ConcatHelper` report their foolish-children count.
    /// `Concatenation` is `Some(0)` only when genuinely empty (no helper
    /// populated and no elements); otherwise it's the sum of every
    /// `ubc_children` helper's own `stmt_count` (summed generally, though in
    /// practice there is at most one helper). Every other kind is `None`
    /// (not brane-like).
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

    /// `Brane` and `ConcatHelper` index their foolish children directly.
    /// `Concatenation` walks its `ubc_children` helpers in order,
    /// subtracting each helper's own `stmt_count` from `idx` until it lands
    /// inside one, then delegates to that helper's `stmt_at`.
    pub fn stmt_at(&self, idx: usize) -> Option<FirPointer> {
        match self.node() {
            FirSpec::Brane { .. } | FirSpec::ConcatHelper => {
                self.foolish_children().get(idx).copied()
            }
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

    /// Mirrors `crate::fir_trait::Fir::as_brane_characterizations`:
    /// `Brane`'s own override returns its
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
            FirSpec::Concatenation { provenance } => *provenance,
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

    /// A FIR owns its own `nyes`; it must never be changed from outside the
    /// FIR. The ONLY sanctioned writers are (1) a FIR on ITSELF, inside its
    /// own `fir_op_step`, and (2) construction. `pub(crate)`, not `pub`, is
    /// the enforcement.
    pub(crate) fn set_nyes(&mut self, n: Nyes) {
        self.storage.get_mut(self.ptr).set_nyes(n);
    }

    pub fn create_child(&mut self, spec: FirSpec) -> FirPointer {
        self.ptr.create_child(self.storage, spec)
    }

    /// Pushes a parse-time child under an SF/SFF marker, panicking
    /// (unconditionally, not a `debug_assert!`) if any search-kind
    /// descendant of `child` is not exactly `ECONSTANIC` — an SFF body must
    /// be built entirely from unevaluated material, so a descendant search
    /// that already ran is an internal-consistency violation, not a
    /// recoverable condition. `child` must already be a child of `self.ptr`.
    pub fn check_sff_marked_child(&self, child: FirPointer) {
        if let Some(offender) = sift_for_first_non_econstanic_descendent_search(self.storage, child)
        {
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
        self.storage
            .get_mut(self.ptr)
            .push_ubc_child(child, child_nyes);
    }

    pub fn push_search_result(&mut self, result: FirPointer) {
        let result_nyes = self.storage.get_nyes(result);
        self.storage
            .get_mut(self.ptr)
            .push_search_result(result, result_nyes);
    }

    /// Pushes a search RESULT and its `FoolRef` bookkeeping entry to
    /// `ubc_children`, in that order — the FoolRef two-child invariant:
    /// `[0]` is the value every reader accesses via `.first()`, `[1]` is
    /// invisible to them. `referent` is the ORIGINAL found statement, not
    /// the cloned result — a genuinely shared `FirPointer` (see
    /// `revive_constanic`'s `FoolRef`-always-shares rule, which this
    /// invariant depends on).
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
    let is_search_kind = matches!(
        storage.get(node),
        FirSpec::Search { .. } | FirSpec::Index { .. }
    );
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
    /// copy of an already-constanic (settled) FIR for use in a new context
    /// (AB/IB recoordination — a named brane referenced elsewhere and
    /// detached/recloned into that new site), and the copy is given an
    /// EARLIER `Nyes` than the original whenever the original's settledness
    /// was *context-dependent* rather than self-contained: an `Econstanic`/
    /// `Woconstanic` node (typically a settled search result) is regressed
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
    ///    before the share-not-clone check. `StayFoolish` tries its settled
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
        // StayFoolish/StayFullyFoolish unwrapping — checked FIRST, before
        // the share-not-clone check below. Only `StayFoolish` (not
        // `StayFullyFoolish`) tries its settled `ubc_children[0]` first;
        // either kind falls through to its first `foolish_children` entry;
        // if BOTH are empty, this logs an ALARM and falls through to clone
        // the wrapper as-is via the normal share/rebuild logic below.
        let spec = self.get(root).clone();
        if matches!(spec, FirSpec::StayFoolish | FirSpec::StayFullyFoolish) {
            if matches!(spec, FirSpec::StayFoolish)
                && let Some(result) = FirCursor::new(root, self).ubc_children().first().copied()
            {
                return self.revive_constanic(
                    result,
                    new_parent,
                    index,
                    sfm,
                    skip_foolish_children,
                );
            }
            if let Some(inner) = self.foolish_children(root).first().copied() {
                return self.revive_constanic(inner, new_parent, index, sfm, skip_foolish_children);
            }
            eprintln!("ALARM: SF/SFF node has no children — cloning wrapper as-is");
        }

        let nyes = self.get_nyes(root);
        let spec = self.get(root).clone();

        // Share-not-clone: Constant/Independent non-Brane always shares;
        // FoolRef/Creation always share regardless of NYES. The shared
        // pointer is NOT reparented (its own `.parent` stays exactly as it
        // was) but IS appended to `new_parent`'s `foolish_children` list
        // (`attach_shared_foolish_child`) — without this append, a caller
        // like `populate_concat_helpers` that walks `new_parent`'s
        // `foolish_children` afterward would silently see the shared child
        // missing, even though `revive_constanic` reported success.
        let is_share_kind = matches!(spec, FirSpec::FoolRef { .. } | FirSpec::Creation);
        let is_constanic_non_brane = matches!(nyes, Nyes::Constant | Nyes::Independent)
            && !matches!(spec, FirSpec::Brane { .. });
        if is_share_kind || is_constanic_non_brane {
            self.attach_shared_foolish_child(new_parent, root);
            return root;
        }

        // Recursive per-node rebuild. The new node's own spec is the
        // source's spec, with a Statement's line_number renumbered to
        // `index`.
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
            // `create_child`'s parent construction path always appends the
            // new pointer to `new_parent`'s `foolish_children` — correct for
            // rebuilding parse-time topology, but a UBC-CHILD clone belongs
            // ONLY in `ubc_children`, never `foolish_children`. Pop the
            // wrongly-appended entry back off before recording it correctly
            // below.
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
    // Resolve through to the settled value (e.g. a search reference to a
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

/// The search engine: the candidate-navigation and predicate-matching
/// machinery `SearchFir`'s dispatch (`mod search_fir_dispatch` below) drives
/// during a search step.
pub(crate) mod search_engine {
    use super::{Equality, FVMStorage, FirCursor, FirPointer, default_equal};

    use foolish_core::fir::Nyes;
    use regex::Regex;

    /// Exact match, or a regex match if `pattern` isn't already anchored.
    pub(crate) fn matches_pattern(stmt_name: &str, pattern: &str) -> bool {
        if stmt_name == pattern {
            return true;
        }
        let re = if pattern.contains('^') || pattern.contains('$') {
            Regex::new(pattern)
        } else {
            Regex::new(&format!("^{}$", pattern))
        };
        if let Ok(re) = re {
            return re.is_match(stmt_name);
        }
        false
    }

    /// Where a Navigator starts scanning from.
    #[expect(
        dead_code,
        reason = "never constructed — BraneNavigator::new takes an explicit forward flag \
                  directly instead of going through this type"
    )]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) enum CursorSource {
        Contextless,
        Contexted,
    }

    /// The result of applying a predicate to a single candidate statement.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) enum MatchOutcome {
        Approve,
        Reject,
        NkStop,
    }

    /// Result of the core scan loop.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub(crate) enum ScanOutcome {
        Found(FirPointer),
        NkStop,
        Miss,
    }

    /// Match predicates for the ContextfulSearch engine.
    ///
    /// Each variant reads a different facet of the candidate statement FIR.
    /// The candidate is the *full* statement — name, body/value, line
    /// number, parent, NYES — everything reachable from the statement
    /// `FirPointer` via `&FVMStorage`.
    #[derive(Debug)]
    pub(crate) enum SearchPredicate {
        /// Name-match: `?name` / `~name` / `.name`. Reads the candidate's name.
        Name { pattern: String },
        /// Value-match: `?=v` / `~=v`. Reads the candidate's body integer value.
        Value { pattern: FirPointer },
        /// Atomic name+value: `?name=v` / `~name=v`. Both gates on the same candidate.
        NameValue { name: String, value: FirPointer },
        /// Positional index: `#N`. Reads the candidate's position in the
        /// scan. The only predicate `IndexFir`'s own dispatch constructs —
        /// `^`/`$` head/tail both compile down to an `Index` with the
        /// appropriate offset (`0` for head, a tail-relative negative
        /// offset for tail) rather than to `Head`/`Tail` below.
        Index(i32),
        /// First position: `^`. Matches when position == 0. Never
        /// constructed by production code — see `Index`'s doc comment above.
        #[cfg_attr(
            not(test),
            expect(
                dead_code,
                reason = "no production caller builds this variant — IndexFir compiles ^/$ down to Index instead"
            )
        )]
        Head,
        /// Last position: `$`. Matches when position == total - 1. Never
        /// constructed by production code — see `Index`'s doc comment above.
        #[cfg_attr(
            not(test),
            expect(
                dead_code,
                reason = "no production caller builds this variant — IndexFir compiles ^/$ down to Index instead"
            )
        )]
        Tail,
    }

    /// Context passed to the predicate during a scan.
    #[derive(Debug, Clone)]
    pub(crate) struct ScanCtx {
        /// 0-based position of the current candidate within its home brane.
        pub(crate) position: usize,
        /// Total number of candidates in the home brane.
        pub(crate) total: usize,
    }

    impl SearchPredicate {
        /// Apply this predicate to a candidate statement.
        pub(crate) fn matches(
            &self,
            storage: &FVMStorage,
            candidate: FirPointer,
            ctx: &ScanCtx,
        ) -> MatchOutcome {
            match self {
                Self::Name { pattern } => {
                    // Matches against searchable_name (the full characterized LHS as
                    // one string) — a plain pattern naturally won't match a
                    // characterized name, and a '-bearing pattern matches only the
                    // identically-characterized name. See Identifier::searchable_name.
                    let name = match FirCursor::new(candidate, storage).as_stmt_identifier() {
                        Some(id) => id.searchable_name().to_owned(),
                        None => return MatchOutcome::Reject,
                    };
                    if !matches_pattern(&name, pattern) {
                        return MatchOutcome::Reject;
                    }
                    check_body_nyes(storage, candidate)
                }
                Self::Value { pattern } => {
                    let body = match storage.foolish_children(candidate).first().copied() {
                        Some(b) => b,
                        None => return MatchOutcome::Reject,
                    };
                    match default_equal(storage, body, *pattern) {
                        Equality::Equal => MatchOutcome::Approve,
                        Equality::NotEqual => MatchOutcome::Reject,
                        Equality::Unknowable => MatchOutcome::NkStop,
                    }
                }
                Self::NameValue { name, value } => {
                    let stmt_name = match FirCursor::new(candidate, storage).as_stmt_identifier() {
                        Some(id) => id.searchable_name().to_owned(),
                        None => return MatchOutcome::Reject,
                    };
                    if !matches_pattern(&stmt_name, name) {
                        return MatchOutcome::Reject;
                    }
                    let body = match storage.foolish_children(candidate).first().copied() {
                        Some(b) => b,
                        None => return MatchOutcome::Reject,
                    };
                    match default_equal(storage, body, *value) {
                        Equality::Equal => MatchOutcome::Approve,
                        Equality::NotEqual => MatchOutcome::Reject,
                        Equality::Unknowable => MatchOutcome::NkStop,
                    }
                }
                Self::Index(offset) => {
                    let target = if *offset >= 0 {
                        *offset as usize
                    } else if ctx.total == 0 {
                        return MatchOutcome::Reject;
                    } else {
                        (ctx.total as i32 + offset) as usize
                    };
                    if ctx.position == target {
                        check_body_nyes(storage, candidate)
                    } else {
                        MatchOutcome::Reject
                    }
                }
                Self::Head => {
                    if ctx.position == 0 {
                        check_body_nyes(storage, candidate)
                    } else {
                        MatchOutcome::Reject
                    }
                }
                Self::Tail => {
                    if ctx.total > 0 && ctx.position == ctx.total - 1 {
                        check_body_nyes(storage, candidate)
                    } else {
                        MatchOutcome::Reject
                    }
                }
            }
        }

        /// Like [`Self::matches`] but skips the body-NYES gate.
        ///
        /// For positional/name-only predicates (Index, Head, Tail, Name) the
        /// candidate's body settling state is irrelevant — the caller decides
        /// what to do. Value/NameValue predicates delegate to [`Self::matches`]
        /// because they need the body settled to compare values.
        pub(crate) fn matches_no_body_check(
            &self,
            storage: &FVMStorage,
            candidate: FirPointer,
            ctx: &ScanCtx,
        ) -> MatchOutcome {
            match self {
                Self::Name { pattern } => {
                    let name = match FirCursor::new(candidate, storage).as_stmt_identifier() {
                        Some(id) => id.searchable_name().to_owned(),
                        None => return MatchOutcome::Reject,
                    };
                    if !matches_pattern(&name, pattern) {
                        return MatchOutcome::Reject;
                    }
                    MatchOutcome::Approve
                }
                Self::Index(offset) => {
                    let target = if *offset >= 0 {
                        *offset as usize
                    } else if ctx.total == 0 {
                        return MatchOutcome::Reject;
                    } else {
                        (ctx.total as i32 + offset) as usize
                    };
                    if ctx.position == target {
                        MatchOutcome::Approve
                    } else {
                        MatchOutcome::Reject
                    }
                }
                Self::Head => {
                    if ctx.position == 0 {
                        MatchOutcome::Approve
                    } else {
                        MatchOutcome::Reject
                    }
                }
                Self::Tail => {
                    if ctx.total > 0 && ctx.position == ctx.total - 1 {
                        MatchOutcome::Approve
                    } else {
                        MatchOutcome::Reject
                    }
                }
                // Value/NameValue need body settled for comparison.
                _ => self.matches(storage, candidate, ctx),
            }
        }
    }

    /// Check a candidate's body NYES after it passes positional/name gates.
    /// A pre-constanic body reaching this point is an internal-consistency
    /// violation, not a legitimate outcome — hence `unreachable!` rather
    /// than a handled case. NK → NkStop. Otherwise → Approve.
    fn check_body_nyes(storage: &FVMStorage, candidate: FirPointer) -> MatchOutcome {
        let nyes = storage
            .foolish_children(candidate)
            .first()
            .map(|&b| storage.get_nyes(b));
        match nyes {
            Some(n) if !n.is_constanic() => unreachable!("pre-constanic body in search candidate"),
            Some(Nyes::Nk) => MatchOutcome::NkStop,
            _ => MatchOutcome::Approve,
        }
    }

    /// Navigator contract: yields candidate statements as (`FirPointer`,
    /// brane_position), with two correctness requirements:
    ///
    /// 1. **Correctly ordered** — the one mandated order.
    /// 2. **Complete** — every reachable candidate, exactly once, then stops.
    pub(crate) trait CandidateNavigator {
        /// Yield the next candidate as (statement `FirPointer`, 0-based brane position).
        fn next_candidate(&mut self) -> Option<(FirPointer, usize)>;
        /// Total number of candidates in the source.
        fn total(&self) -> usize;
    }

    /// Iterates a brane's statements in order, forward or backward.
    #[derive(Debug)]
    pub(crate) struct BraneNavigator {
        children: Vec<FirPointer>,
        pos: usize,
        forward: bool,
        done: bool,
    }

    impl BraneNavigator {
        pub(crate) fn new(storage: &FVMStorage, brane: FirPointer, forward: bool) -> Self {
            let cursor = FirCursor::new(brane, storage);
            let len = cursor.stmt_count().unwrap_or(0);
            let children: Vec<FirPointer> = (0..len).filter_map(|i| cursor.stmt_at(i)).collect();
            let start = if forward || len == 0 { 0 } else { len - 1 };
            Self {
                children,
                pos: start,
                forward,
                done: len == 0,
            }
        }

        pub(crate) fn set_range(&mut self, start: usize, end: usize) {
            if start > end || start >= self.children.len() {
                self.done = true;
                return;
            }
            let end = end.min(self.children.len() - 1);
            if self.forward {
                self.pos = start;
                self.done = false;
            } else {
                self.pos = end;
                self.done = false;
            }
        }
    }

    impl CandidateNavigator for BraneNavigator {
        fn next_candidate(&mut self) -> Option<(FirPointer, usize)> {
            if self.done || self.pos >= self.children.len() {
                return None;
            }
            let brane_pos = self.pos;
            let candidate = self.children[brane_pos];
            // Advance cursor.
            if self.forward {
                self.pos += 1;
                if self.pos >= self.children.len() {
                    self.done = true;
                }
            } else if self.pos == 0 {
                self.done = true;
            } else {
                self.pos -= 1;
            }
            Some((candidate, brane_pos))
        }

        fn total(&self) -> usize {
            self.children.len()
        }
    }

    /// The core scan loop of the ContextfulSearch engine: if a candidate's
    /// predicate returns `NkStop`, the scan halts and the search itself
    /// becomes NK. Returns `Miss` when all candidates are exhausted with no
    /// match. The caller decides the settlement: anchored → NK, unanchored
    /// → ECONSTANIC.
    pub(crate) fn contextful_search_scan(
        storage: &FVMStorage,
        nav: &mut dyn CandidateNavigator,
        predicate: &SearchPredicate,
    ) -> ScanOutcome {
        let total = nav.total();
        while let Some((candidate, position)) = nav.next_candidate() {
            let ctx = ScanCtx { position, total };
            match predicate.matches(storage, candidate, &ctx) {
                MatchOutcome::Approve => return ScanOutcome::Found(candidate),
                MatchOutcome::Reject => {}
                MatchOutcome::NkStop => return ScanOutcome::NkStop,
            }
        }
        ScanOutcome::Miss
    }

    /// Like [`contextful_search_scan`] but uses
    /// [`SearchPredicate::matches_no_body_check`] — for contextless searches
    /// (`IndexFir`, `SearchFir` name search) where body settling is the
    /// caller's responsibility.
    pub(crate) fn contextful_search_scan_no_body_check(
        storage: &FVMStorage,
        nav: &mut dyn CandidateNavigator,
        predicate: &SearchPredicate,
    ) -> ScanOutcome {
        let total = nav.total();
        while let Some((candidate, position)) = nav.next_candidate() {
            let ctx = ScanCtx { position, total };
            match predicate.matches_no_body_check(storage, candidate, &ctx) {
                MatchOutcome::Approve => return ScanOutcome::Found(candidate),
                MatchOutcome::Reject => {}
                MatchOutcome::NkStop => return ScanOutcome::NkStop,
            }
        }
        ScanOutcome::Miss
    }
}

/// `SearchFir`'s own predicate-building and dispatch logic. Free functions
/// taking `FirPointer` + `&mut FVMStorage` explicitly, matching this
/// module's `fir_op_step`/`combine` convention, rather than methods on
/// `FirPointer` itself — these are `SearchFir`-specific, not generic arena
/// operations every kind needs.
mod search_fir_dispatch {
    use super::search_engine::{
        BraneNavigator, ScanOutcome, SearchPredicate, contextful_search_scan,
        contextful_search_scan_no_body_check,
    };
    use super::{FVMStorage, FirCursor, FirPointer, FirSpec};

    use foolish_core::fir::Nyes;

    fn nyes_from_found(found: Nyes) -> Nyes {
        super::nyes_from_found(found)
    }

    /// The statement a search found presents its NF-substitution value if
    /// it's been refused (`nf_reason` set: a fresh, already-`Nk` node
    /// instead of cloning its written RHS — `settled_result`'s own contract
    /// requires the presented value already BE constanic, which a
    /// `Prembrionic`-starting `FirSpec::Nk` node would not be), otherwise
    /// its written body, cloned via `revive_constanic`.
    pub(super) fn clone_stmt_result(
        storage: &mut FVMStorage,
        stmt: FirPointer,
        new_parent: FirPointer,
        sfm: bool,
    ) -> FirPointer {
        if let Some(reason) = storage.nf_reason(stmt) {
            let reason = reason.to_owned();
            let nk = storage.make_orphan_child(new_parent, FirSpec::Nk { reason });
            storage.with_mut(nk, |fir| fir.set_nyes(Nyes::Nk));
            return nk;
        }
        let body = storage
            .foolish_children(stmt)
            .first()
            .copied()
            .expect("statement must have a body");
        let index = FirCursor::new(stmt, storage)
            .as_stmt_line_number()
            .unwrap_or(0);
        storage.revive_constanic(body, new_parent, index, sfm, false)
    }

    /// Clones the found statement's value under `self`, pairs it with a
    /// `FoolRef` wrapping the ORIGINAL statement (the two-child invariant),
    /// and moves to `Braning`.
    fn handle_found(storage: &mut FVMStorage, ptr: FirPointer, stmt: FirPointer, sfm: bool) {
        let clone = clone_stmt_result(storage, stmt, ptr, sfm);
        let mut cursor = super::FirCursorMut::new(ptr, storage);
        cursor.push_search_result_pair(clone, stmt);
        cursor.set_nyes(Nyes::Braning);
    }

    /// The value a statement PRESENTS: its `settled_result()` (the
    /// NF-refusal NK, if already refused) if set, else the raw written
    /// body. Used by the two NF-refusal checks below, which must compare
    /// against what a PRIOR statement already presents, not its raw RHS —
    /// poisoning must be transitive (FOOP-33 §4).
    pub(super) fn statement_value_for_comparison(
        storage: &FVMStorage,
        stmt: FirPointer,
    ) -> Option<FirPointer> {
        stmt.settled_result(storage)
            .or_else(|| storage.foolish_children(stmt).first().copied())
    }

    /// A null-characterized statement (FOOP-33 §4) checks ITSELF, once its
    /// body is constanic, against any EARLIER same-name null-characterized
    /// statement (IB, then AB) — refusing (`set_nf_reason`) if the two
    /// values are not `Equal`. Terminal: does nothing if `nf_reason` is
    /// already set (no re-alarm once refused).
    pub(super) fn check_null_const_conflict(
        storage: &mut FVMStorage,
        stmt: FirPointer,
        body: FirPointer,
        current_statement: Option<FirPointer>,
        current_brane: Option<FirPointer>,
    ) {
        if storage.nf_reason(stmt).is_some() {
            return;
        }
        let pattern = match storage.get(stmt) {
            FirSpec::Statement { identifier, .. } => identifier.searchable_name().to_string(),
            _ => return,
        };
        let prior = ib_search_by_pattern(storage, &pattern, current_statement)
            .or_else(|| ab_search_by_pattern(storage, &pattern, current_brane));
        let Some((prior_stmt, _)) = prior else {
            return; // no earlier definition -- this statement establishes the constant.
        };
        let Some(prior_body) = statement_value_for_comparison(storage, prior_stmt) else {
            return;
        };
        if !storage.get_nyes(prior_body).is_constanic() {
            return; // prior definition not yet settled -- nothing to compare yet.
        }
        if super::default_equal(storage, body, prior_body) != super::Equality::Equal {
            let name = match storage.get(stmt) {
                FirSpec::Statement { identifier, .. } => identifier.identifier_name().to_string(),
                _ => return,
            };
            refuse_statement(storage, stmt, format!("'{name} not-foolish"));
        }
    }

    /// Shared write path for both NF-refusal rules below: sets `nf_reason`
    /// AND materializes the refusal as a fresh, already-`Nk` node pushed to
    /// `stmt`'s `ubc_children`. This second step is required: `FirPointer::
    /// settled_result`'s generic `ubc_children().first()` read (used
    /// pervasively, including by read-only output serialization, which
    /// cannot itself allocate a node on demand) must find something there to
    /// answer `Some(nk)` for a refused statement — setting `nf_reason` alone
    /// would leave `ubc_children` empty and `settled_result` would fall
    /// through to the raw, unrefused written body instead.
    fn refuse_statement(storage: &mut FVMStorage, stmt: FirPointer, reason: String) {
        storage.set_nf_reason(stmt, reason.clone());
        let nk = storage.make_orphan_child(stmt, FirSpec::Nk { reason });
        storage.with_mut(nk, |fir| fir.set_nyes(Nyes::Nk));
        let me = storage.get_mut(stmt);
        me.push_ubc_child(nk, Nyes::Nk);
    }

    /// A null-characterized statement whose constanic value resolves to a
    /// creation with a DIFFERENT original name is refused (FOOP-33: named
    /// creations cannot be renamed). Terminal, same guard as
    /// `check_null_const_conflict`.
    pub(super) fn check_rename_of_named_creation(
        storage: &mut FVMStorage,
        stmt: FirPointer,
        body: FirPointer,
    ) {
        if storage.nf_reason(stmt).is_some() {
            return;
        }
        let is_nully = match storage.get(stmt) {
            FirSpec::Statement { identifier, .. } => {
                identifier.is_nully_characterizing_coordinate_name()
            }
            _ => return,
        };
        if !is_nully {
            return;
        }
        let resolved = body.value(storage);
        if !matches!(storage.get(resolved), FirSpec::Creation) {
            return; // not a creation reference at all -- nothing to forbid.
        }
        let Some(original_name) = resolved.get_display_name(storage, stmt) else {
            return; // the creation has no original name at all -- nothing to protect.
        };
        let pattern = match storage.get(stmt) {
            FirSpec::Statement { identifier, .. } => identifier.searchable_name().to_string(),
            _ => return,
        };
        if original_name != pattern {
            let name = match storage.get(stmt) {
                FirSpec::Statement { identifier, .. } => identifier.identifier_name().to_string(),
                _ => return,
            };
            refuse_statement(
                storage,
                stmt,
                format!("'{name} not-foolish (Named creations cannot be renamed)"),
            );
        }
    }

    /// The null-characterized name-constant rule (FOOP-33 §4), applied at
    /// concatenation merge time: `check_null_const_conflict`'s own `fir_op_step`
    /// gate never fires for a merge-cloned statement (`revive_constanic` builds
    /// it already-constanic, skipping `Prembrionic`/`Embryonic`/`Braning`
    /// entirely), so this enforces the same rule directly, against statements
    /// already merged BEFORE `new_stmt`. `already_merged` is searched in
    /// REVERSE (nearest-first) so a same-name chain compares each new one
    /// against the NEAREST prior, transitively carrying any earlier refusal
    /// forward via `statement_value_for_comparison`'s settled-result-first
    /// read.
    pub(super) fn apply_null_const_rule_to_merged_stmt(
        storage: &mut FVMStorage,
        new_stmt: FirPointer,
        already_merged: &[FirPointer],
    ) {
        let (is_nully, pattern) = match storage.get(new_stmt) {
            FirSpec::Statement { identifier, .. } => (
                identifier.is_nully_characterizing_coordinate_name(),
                identifier.searchable_name().to_string(),
            ),
            _ => return,
        };
        if !is_nully {
            return;
        }
        let Some(&prior_stmt) = already_merged.iter().rev().find(|&&s| {
            matches!(storage.get(s), FirSpec::Statement { identifier, .. }
                if identifier.searchable_name() == pattern)
        }) else {
            return; // first occurrence of this null-const name in the merge -- permitted.
        };
        let Some(new_body) = statement_value_for_comparison(storage, new_stmt) else {
            return;
        };
        let Some(prior_body) = statement_value_for_comparison(storage, prior_stmt) else {
            return;
        };
        if !storage.get_nyes(new_body).is_constanic()
            || !storage.get_nyes(prior_body).is_constanic()
        {
            return; // one side not yet settled -- nothing to compare yet.
        }
        if super::default_equal(storage, new_body, prior_body) != super::Equality::Equal {
            let name = match storage.get(new_stmt) {
                FirSpec::Statement { identifier, .. } => identifier.identifier_name().to_string(),
                _ => return,
            };
            refuse_statement(storage, new_stmt, format!("'{name} not-foolish"));
        }
    }

    /// Builds a single `ConcatHelper` holding ALL merged lines (no
    /// `MAX_BRANE_SIZE` limit). Constanic-clones every element's statements
    /// (in order, via each element's OWN resolved `.value()`) into one flat
    /// `ConcatHelper`, applying the null-const merge rule to each clone as
    /// it is added, then pushes the helper as `ptr`'s sole `ubc_children`
    /// entry. Structural only — the CALLER decides `ptr`'s own NYES. A
    /// no-op when there are no lines at all.
    pub(super) fn populate_concat_helpers(storage: &mut FVMStorage, ptr: FirPointer) {
        let elements: Vec<FirPointer> = storage.foolish_children(ptr).to_vec();

        let total_lines: usize = elements
            .iter()
            .map(|&e| {
                let resolved = e.value(storage);
                FirCursor::new(resolved, storage).stmt_count().unwrap_or(0)
            })
            .sum();
        if total_lines == 0 {
            return;
        }

        // Build the (empty) helper first so its pointer becomes the parent
        // of every cloned line — cross-element search resolution walks to
        // it. `make_orphan_child`, not `create_child`: the helper belongs
        // ONLY in `ubc_children`, never `foolish_children`.
        let helper = storage.make_orphan_child(ptr, FirSpec::ConcatHelper);

        let mut cloned_stmts: Vec<FirPointer> = Vec::with_capacity(total_lines);
        for &elem in &elements {
            let resolved = elem.value(storage);
            let count = FirCursor::new(resolved, storage).stmt_count().unwrap_or(0);
            for i in 0..count {
                let Some(stmt) = FirCursor::new(resolved, storage).stmt_at(i) else {
                    continue;
                };
                let global_idx = cloned_stmts.len();
                let clone = storage.revive_constanic(stmt, helper, global_idx, false, false);
                apply_null_const_rule_to_merged_stmt(storage, clone, &cloned_stmts);
                cloned_stmts.push(clone);
            }
        }

        if cloned_stmts.is_empty() {
            return;
        }
        let mut cursor = super::FirCursorMut::new(ptr, storage);
        cursor.push_ubc_child(helper);
    }

    pub(super) fn settle_from_ubc_result(storage: &mut FVMStorage, ptr: FirPointer) {
        let result_nyes = FirCursor::new(ptr, storage)
            .ubc_children()
            .first()
            .map(|&r| storage.get_nyes(r))
            .unwrap_or(Nyes::Nk);
        if result_nyes.is_constanic() {
            storage.with_mut(ptr, |fir| fir.set_nyes(nyes_from_found(result_nyes)));
        }
    }

    /// `self`'s value operand: index `1` if anchored (the anchor occupies
    /// `[0]`), else index `0`.
    fn value_child(storage: &FVMStorage, ptr: FirPointer) -> FirPointer {
        let anchored = matches!(storage.get(ptr), FirSpec::Search { anchored: true, .. });
        let idx = if anchored { 1 } else { 0 };
        storage.foolish_children(ptr)[idx]
    }

    /// An immediate-brane name search, scanning backward from (but
    /// excluding) the current statement's own position. `checked_sub`, not
    /// `saturating_sub`: a statement at position 0 has no preceding range
    /// at all, and the `?` on `None` is exactly the self-hit guard.
    fn ib_search_with_engine(
        storage: &FVMStorage,
        ptr: FirPointer,
        current_statement: Option<FirPointer>,
    ) -> Option<(FirPointer, Nyes)> {
        let pattern = match storage.get(ptr) {
            FirSpec::Search { pattern, .. } => pattern.clone(),
            _ => return None,
        };
        ib_search_by_pattern(storage, &pattern, current_statement)
    }

    /// Generalization of [`ib_search_with_engine`] taking the search pattern
    /// directly rather than reading it off a `FirSpec::Search` node — needed
    /// by `check_null_const_conflict` (FOOP-33 §4), which searches by the
    /// STATEMENT's own `searchable_name()`, not by a `Search` node's
    /// pattern (the statement itself is not a `Search`).
    pub(super) fn ib_search_by_pattern(
        storage: &FVMStorage,
        pattern: &str,
        current_statement: Option<FirPointer>,
    ) -> Option<(FirPointer, Nyes)> {
        let stmt = current_statement?;
        let brane = stmt.home_brane(storage)?;
        let idx = brane.find_stmt_index(storage, stmt)?;
        let search_end = idx.checked_sub(1)?;
        let mut nav = BraneNavigator::new(storage, brane, false);
        nav.set_range(0, search_end);
        let predicate = SearchPredicate::Name {
            pattern: pattern.to_string(),
        };
        match contextful_search_scan_no_body_check(storage, &mut nav, &predicate) {
            ScanOutcome::Found(found) => Some((found, storage.get_nyes(found))),
            _ => None,
        }
    }

    /// An ancestral-brane name search, climbing outward one brane at a
    /// time, scanning each ancestor's statements strictly BEFORE the
    /// position the climb entered it from.
    fn ab_search_with_engine(
        storage: &FVMStorage,
        ptr: FirPointer,
        current_brane: Option<FirPointer>,
    ) -> Option<(FirPointer, Nyes)> {
        let pattern = match storage.get(ptr) {
            FirSpec::Search { pattern, .. } => pattern.clone(),
            _ => return None,
        };
        ab_search_by_pattern(storage, &pattern, current_brane)
    }

    /// Generalization of [`ab_search_with_engine`] taking the search pattern
    /// directly — see [`ib_search_by_pattern`]'s doc comment for why
    /// `StatementFir`'s NF-refusal checks need this shape.
    pub(super) fn ab_search_by_pattern(
        storage: &FVMStorage,
        pattern: &str,
        current_brane: Option<FirPointer>,
    ) -> Option<(FirPointer, Nyes)> {
        let mut current_brane = current_brane?;
        loop {
            let stmt = current_brane.get_my_statement(storage);
            if stmt == current_brane {
                return None;
            }
            let parent_brane = stmt.home_brane(storage)?;
            if let Some(idx) = parent_brane.find_stmt_index(storage, stmt)
                && idx > 0
            {
                let mut nav = BraneNavigator::new(storage, parent_brane, false);
                nav.set_range(0, idx - 1);
                let predicate = SearchPredicate::Name {
                    pattern: pattern.to_string(),
                };
                if let ScanOutcome::Found(found) =
                    contextful_search_scan_no_body_check(storage, &mut nav, &predicate)
                {
                    return Some((found, storage.get_nyes(found)));
                }
            }
            if parent_brane == current_brane {
                return None;
            }
            current_brane = parent_brane;
        }
    }

    /// Reads the anchor's `FoolRef` bookkeeping entry (`ubc_children[1]`,
    /// per the two-child invariant), resolves ITS referent's home brane and
    /// position, then scans a range strictly AFTER (forward) or BEFORE
    /// (backward) that position within the SAME home brane — a contexted
    /// search never leaves the home brane (AGENTS.md §Searches).
    fn contexted_search_from_anchor(
        storage: &FVMStorage,
        ptr: FirPointer,
        forward: bool,
    ) -> Option<(FirPointer, Nyes)> {
        let anchor = storage.foolish_children(ptr)[0];
        let fool_ref_fir = FirCursor::new(anchor, storage)
            .ubc_children()
            .get(1)
            .copied()?;
        let referent = FirCursor::new(fool_ref_fir, storage).as_fool_ref_referent()?;
        let h_brane = referent.home_brane(storage)?;
        let p = h_brane.find_stmt_index(storage, referent)?;
        let brane_len = FirCursor::new(h_brane, storage).stmt_count().unwrap_or(0);
        if brane_len == 0 {
            return None;
        }
        let (scan_start, scan_end) = if forward {
            if p + 1 >= brane_len {
                return None;
            }
            (p + 1, brane_len - 1)
        } else {
            if p == 0 {
                return None;
            }
            (0, p - 1)
        };
        let mut nav = BraneNavigator::new(storage, h_brane, forward);
        nav.set_range(scan_start, scan_end);
        let (is_value_search, pattern) = match storage.get(ptr) {
            FirSpec::Search {
                is_value_search,
                pattern,
                ..
            } => (*is_value_search, pattern.clone()),
            _ => return None,
        };
        let predicate = if is_value_search {
            let value_fir = value_child(storage, ptr);
            SearchPredicate::Value { pattern: value_fir }
        } else if pattern.is_empty() {
            return None;
        } else {
            SearchPredicate::Name { pattern }
        };
        match contextful_search_scan(storage, &mut nav, &predicate) {
            ScanOutcome::Found(stmt) => {
                let nyes = storage
                    .foolish_children(stmt)
                    .first()
                    .map(|&b| storage.get_nyes(b))
                    .unwrap_or(Nyes::Nk);
                Some((stmt, nyes))
            }
            _ => None,
        }
    }

    /// The NAME-SEARCH path — the `is_value_search` branch is
    /// [`value_search_step`] below.
    pub(crate) fn name_search_step(
        storage: &mut FVMStorage,
        ptr: FirPointer,
        current_statement: Option<FirPointer>,
        current_brane: Option<FirPointer>,
        has_ancestral_sfm: bool,
    ) {
        let (anchored, forward, contexted) = match storage.get(ptr) {
            FirSpec::Search {
                anchored,
                forward,
                contexted,
                ..
            } => (*anchored, *forward, *contexted),
            other => unreachable!("name_search_step called on non-Search spec: {other:?}"),
        };
        match storage.get_nyes(ptr) {
            Nyes::Prembrionic => {
                if anchored {
                    let anchor = storage.foolish_children(ptr)[0];
                    storage.with_mut(ptr, |fir| fir.push_task(anchor));
                    storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Braning));
                } else {
                    storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Embryonic));
                }
            }
            Nyes::Embryonic => {
                if anchored {
                    let anchor = storage.foolish_children(ptr)[0];
                    storage.with_mut(ptr, |fir| fir.push_task(anchor));
                    storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Braning));
                } else if !FirCursor::new(ptr, storage).ubc_children().is_empty() {
                    settle_from_ubc_result(storage, ptr);
                } else {
                    match ib_search_with_engine(storage, ptr, current_statement) {
                        Some((stmt, _nyes)) => {
                            handle_found(storage, ptr, stmt, has_ancestral_sfm);
                            storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Braning));
                        }
                        None => storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Braning)),
                    }
                }
            }
            Nyes::Braning => {
                if !FirCursor::new(ptr, storage).ubc_children().is_empty() {
                    settle_from_ubc_result(storage, ptr);
                } else if contexted && anchored {
                    match contexted_search_from_anchor(storage, ptr, forward) {
                        Some((stmt, _nyes)) => handle_found(storage, ptr, stmt, has_ancestral_sfm),
                        // `anchored` is always true in this branch, so a
                        // miss settles Nk (an unanchored miss would settle
                        // Econstanic instead).
                        None => storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Nk)),
                    }
                } else if anchored {
                    let anchor = storage.foolish_children(ptr)[0];
                    let resolved = anchor.value(storage);
                    if storage.get_nyes(resolved) == Nyes::Nk
                        || !FirCursor::new(resolved, storage).is_brane_like()
                    {
                        storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Nk));
                    } else {
                        let mut nav = BraneNavigator::new(storage, resolved, forward);
                        let pattern = match storage.get(ptr) {
                            FirSpec::Search { pattern, .. } => pattern.clone(),
                            _ => unreachable!(),
                        };
                        let predicate = SearchPredicate::Name { pattern };
                        match contextful_search_scan_no_body_check(storage, &mut nav, &predicate) {
                            ScanOutcome::Found(stmt) => {
                                handle_found(storage, ptr, stmt, has_ancestral_sfm);
                            }
                            _ => storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Nk)),
                        }
                    }
                } else {
                    match ab_search_with_engine(storage, ptr, current_brane) {
                        Some((stmt, _nyes)) => handle_found(storage, ptr, stmt, has_ancestral_sfm),
                        None => storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Econstanic)),
                    }
                }
            }
            _ => {}
        }
    }

    /// Builds a `Value` predicate if the pattern is empty (`?=`/`~=`), else
    /// a `NameValue` predicate (`?name=v`/`~name=v`). `None` if the value
    /// operand is not yet constanic — the caller
    /// ([`check_value_pattern_ready`]) is responsible for confirming
    /// readiness first.
    fn build_value_predicate(storage: &FVMStorage, ptr: FirPointer) -> Option<SearchPredicate> {
        let value_fir = value_child(storage, ptr);
        if !storage.get_nyes(value_fir).is_constanic() {
            return None;
        }
        let pattern = match storage.get(ptr) {
            FirSpec::Search { pattern, .. } => pattern.clone(),
            _ => return None,
        };
        if pattern.is_empty() {
            Some(SearchPredicate::Value { pattern: value_fir })
        } else {
            Some(SearchPredicate::NameValue {
                name: pattern,
                value: value_fir,
            })
        }
    }

    /// Gates the value-search dispatch on the value operand's own NYES
    /// (FOOP-23): pre-constanic → push as task, not ready; NK → Nk;
    /// WOCONSTANIC → inherit Woconstanic (waiting on constanics, not a
    /// miss); ECONSTANIC → inherit Econstanic; else confirm the resolved
    /// value is either an integer or a creation (the two comparable value
    /// kinds), else Nk.
    fn check_value_pattern_ready(storage: &mut FVMStorage, ptr: FirPointer) -> bool {
        let value_fir = value_child(storage, ptr);
        let nyes = storage.get_nyes(value_fir);
        if !nyes.is_constanic() {
            storage.with_mut(ptr, |fir| fir.push_task(value_fir));
            return false;
        }
        match nyes {
            Nyes::Nk => {
                storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Nk));
                return false;
            }
            Nyes::Woconstanic => {
                storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Woconstanic));
                return false;
            }
            Nyes::Econstanic => {
                storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Econstanic));
                return false;
            }
            _ => {}
        }
        let resolved = value_fir.value(storage);
        let resolved_is_creation = matches!(storage.get(resolved), FirSpec::Creation);
        if FirCursor::new(value_fir, storage).as_i64().is_none() && !resolved_is_creation {
            storage.with_mut(ptr, |fir| {
                fir.set_alarm_reason(
                    "VALUE-SEARCH-UNSUPPORTED-PATTERN: pattern is neither integer nor creation"
                        .to_string(),
                )
            });
            storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Nk));
            return false;
        }
        true
    }

    /// The value-search dispatch (`?=`/`~=`/`?name=v`/`~name=v`), a distinct
    /// three-phase shape from [`name_search_step`]'s two phases:
    /// `Prembrionic` pushes BOTH the
    /// anchor (if anchored) and the value operand as tasks together (unlike
    /// name-search, which pushes only the anchor); `Embryonic` (unanchored
    /// only — anchored searches skip straight to `Braning`) does the
    /// IB-equivalent backward scan bounded to the enclosing statement's own
    /// position; `Braning` does the contexted/anchored/unanchored (AB-style)
    /// dispatch, mirroring `name_search_step`'s `Braning` arm shape closely
    /// but scanning with the value predicate instead of a name predicate.
    pub(crate) fn value_search_step(
        storage: &mut FVMStorage,
        ptr: FirPointer,
        has_ancestral_sfm: bool,
    ) {
        let (anchored, forward, contexted) = match storage.get(ptr) {
            FirSpec::Search {
                anchored,
                forward,
                contexted,
                ..
            } => (*anchored, *forward, *contexted),
            other => unreachable!("value_search_step called on non-Search spec: {other:?}"),
        };
        match storage.get_nyes(ptr) {
            Nyes::Prembrionic => {
                if anchored {
                    let anchor = storage.foolish_children(ptr)[0];
                    storage.with_mut(ptr, |fir| fir.push_task(anchor));
                }
                let value_fir = value_child(storage, ptr);
                storage.with_mut(ptr, |fir| fir.push_task(value_fir));
                if anchored {
                    storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Braning));
                } else {
                    storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Embryonic));
                }
            }
            Nyes::Embryonic => {
                if !FirCursor::new(ptr, storage).ubc_children().is_empty() {
                    settle_from_ubc_result(storage, ptr);
                    return;
                }
                if !check_value_pattern_ready(storage, ptr) {
                    return;
                }
                let predicate = build_value_predicate(storage, ptr).expect("checked ready");
                if let Some((stmt_ref, brane_ref)) = ptr.find_enclosing_stmt_and_brane(storage)
                    && let Some(idx) = brane_ref.find_stmt_index(storage, stmt_ref)
                    && idx > 0
                {
                    let range_end = idx - 1;
                    let mut nav = BraneNavigator::new(storage, brane_ref, false);
                    nav.set_range(0, range_end);
                    match contextful_search_scan(storage, &mut nav, &predicate) {
                        ScanOutcome::Found(stmt) => {
                            handle_found(storage, ptr, stmt, has_ancestral_sfm);
                        }
                        ScanOutcome::NkStop => {
                            storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Nk));
                            return;
                        }
                        ScanOutcome::Miss => {
                            if !anchored {
                                storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Econstanic));
                                return;
                            }
                        }
                    }
                } else if !anchored {
                    storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Econstanic));
                    return;
                }
                storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Braning));
            }
            Nyes::Braning => {
                if !FirCursor::new(ptr, storage).ubc_children().is_empty() {
                    settle_from_ubc_result(storage, ptr);
                    return;
                }
                if !check_value_pattern_ready(storage, ptr) {
                    return;
                }
                let predicate = build_value_predicate(storage, ptr).expect("checked ready");
                let scan_outcome = if contexted && anchored {
                    let anchor = storage.foolish_children(ptr)[0];
                    let anchor_settled = storage.get_nyes(anchor).is_constanic();
                    match contexted_search_from_anchor(storage, ptr, forward) {
                        Some((stmt, _nyes)) => {
                            handle_found(storage, ptr, stmt, has_ancestral_sfm);
                            return;
                        }
                        None => {
                            if !anchor_settled {
                                return;
                            }
                            ScanOutcome::Miss
                        }
                    }
                } else if anchored {
                    let anchor = storage.foolish_children(ptr)[0];
                    let resolved = anchor.value(storage);
                    if storage.get_nyes(resolved) == Nyes::Nk {
                        storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Nk));
                        return;
                    }
                    if !FirCursor::new(resolved, storage).is_brane_like() {
                        ScanOutcome::Miss
                    } else {
                        let mut nav = BraneNavigator::new(storage, resolved, forward);
                        contextful_search_scan(storage, &mut nav, &predicate)
                    }
                } else {
                    match ptr.find_enclosing_stmt_and_brane(storage) {
                        Some((stmt_ref, brane_ref)) => {
                            if let Some(idx) = brane_ref.find_stmt_index(storage, stmt_ref) {
                                let len = storage.foolish_children(brane_ref).len();
                                if idx + 1 < len {
                                    let mut nav = BraneNavigator::new(storage, brane_ref, true);
                                    nav.set_range(idx + 1, len - 1);
                                    contextful_search_scan(storage, &mut nav, &predicate)
                                } else {
                                    ScanOutcome::Miss
                                }
                            } else {
                                ScanOutcome::Miss
                            }
                        }
                        None => ScanOutcome::Miss,
                    }
                };
                match scan_outcome {
                    ScanOutcome::Found(stmt) => {
                        handle_found(storage, ptr, stmt, has_ancestral_sfm);
                    }
                    ScanOutcome::NkStop => {
                        storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Nk));
                    }
                    ScanOutcome::Miss => {
                        let settle = if anchored { Nyes::Nk } else { Nyes::Econstanic };
                        storage.with_mut(ptr, |fir| fir.set_nyes(settle));
                    }
                }
            }
            _ => {}
        }
    }
}

/// The stepping loop and the FIR→core-FIR output-serialization family that
/// `UbcaEvaluator::evaluate` drives.
///
/// # Free functions, not methods
///
/// These conversion functions dispatch across EVERY `FirSpec` variant
/// (`match kind { FirSpec::Search => ..., FirSpec::Operator => ..., ... }`),
/// so there is no single type to attach them to as methods — the same
/// reason `fir_op_step`, `combine`, and every `search_fir_dispatch`
/// function are also free functions taking `FirPointer` explicitly.
mod core_fir_conversion {
    use super::{
        ANON_STMT_NAME, ConcatProvenance, FVMStorage, FirCursor, FirPointer, FirSpec, MAX_DEPTH,
        search_fir_dispatch,
    };

    use foolish_core::fir as core_fir;
    use foolish_core::fir::{
        Alarm, AlarmLevel, AlarmSource, ConcatenationFirBuilder, ConstantIntFirBuilder,
        CreationFirBuilder, FirQueryable, IndexFirBuilder, NkFirBuilder, NormalBraneFirBuilder,
        Nyes, OperatorFirBuilder, SearchFirBuilder, StayFoolishFirBuilder,
        StayFullyFoolishFirBuilder,
    };

    /// Steps `ptr` up to `MAX_STEPS` times, returning `Ok(())` once
    /// constanic, or an error naming the iteration count if the step
    /// budget is exhausted first. Caps total top-level iterations —
    /// distinct from `step_inner`'s `MAX_DEPTH`, which caps recursion depth
    /// within a single iteration.
    const MAX_STEPS: usize = 10_000;

    pub(crate) fn step_to_settled(storage: &mut FVMStorage, ptr: FirPointer) -> Result<(), String> {
        let mut last_step = 0;
        for step in 0..MAX_STEPS {
            ptr.step(storage);
            last_step = step;
            if storage.get_nyes(ptr).is_constanic() {
                return Ok(());
            }
        }
        if !storage.get_nyes(ptr).is_constanic() {
            return Err(format!("Iteration exceeded {last_step}"));
        }
        Ok(())
    }

    /// The UBCA debugger-breakpoint equivalent — steps until `matcher`
    /// accepts the front task (or `None` when there is no front task),
    /// returning the step count, or an error if the FVM settles first or
    /// the step budget is exhausted.
    ///
    /// The matcher takes `&FVMStorage` explicitly alongside
    /// `Option<FirPointer>` (rather than a bare `Option<FirPointer>`)
    /// because a `FirPointer` carries no data of its own — it must be
    /// read through the arena to be inspected.
    ///
    /// No production caller: this is developer-facing debugger tooling, not
    /// part of `evaluate`'s own path. Exercised by this file's own tests.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "developer-facing debugger tooling, not part of evaluate's own path; \
                      exercised only by this file's own tests"
        )
    )]
    pub(crate) fn step_until(
        storage: &mut FVMStorage,
        ptr: FirPointer,
        mut matcher: impl FnMut(&FVMStorage, Option<FirPointer>) -> bool,
    ) -> Result<usize, String> {
        for step in 0..MAX_STEPS {
            let front = FirCursor::new(ptr, storage).front_task();
            if matcher(storage, front) {
                return Ok(step);
            }
            if storage.get_nyes(ptr).is_constanic() {
                return Err(format!(
                    "FVM settled (nyes={:?}) before condition was met at step {step}",
                    storage.get_nyes(ptr)
                ));
            }
            ptr.step(storage);
        }
        Err(format!(
            "Step limit ({MAX_STEPS}) reached before condition was met"
        ))
    }

    /// No production caller — see `step_until`'s doc comment.
    #[cfg_attr(not(test), expect(dead_code, reason = "see step_until's doc comment"))]
    pub(crate) fn step_until_line_number(
        storage: &mut FVMStorage,
        ptr: FirPointer,
        line: usize,
    ) -> Result<usize, String> {
        step_until(storage, ptr, |storage, front| {
            front
                .and_then(|f| FirCursor::new(f, storage).as_stmt_line_number())
                .is_some_and(|l| l == line)
        })
    }

    /// No production caller — see `step_until`'s doc comment.
    #[cfg_attr(not(test), expect(dead_code, reason = "see step_until's doc comment"))]
    pub(crate) fn step_until_statement_name(
        storage: &mut FVMStorage,
        ptr: FirPointer,
        name: &str,
    ) -> Result<usize, String> {
        step_until(storage, ptr, |storage, front| {
            front
                .and_then(|f| FirCursor::new(f, storage).as_stmt_identifier())
                .is_some_and(|id| id.searchable_name() == name)
        })
    }

    /// An anonymous statement (`ANON_STMT_NAME`, or any empty name) renders
    /// with no `name=` prefix.
    fn display_stmt_name(name: Option<&str>) -> Option<String> {
        match name {
            Some(n) if n.is_empty() || n == ANON_STMT_NAME => None,
            Some(n) => Some(n.to_string()),
            None => None,
        }
    }

    pub(crate) fn proto_to_core_fir(storage: &FVMStorage, ptr: FirPointer) -> core_fir::Fir {
        proto_to_core_fir_inner(storage, ptr, false, None, 0)
    }

    /// Renders an SFF body: top-level searches get EMBRYONIC state;
    /// operator operands get CONSTANT state; operators get
    /// WOCONSTANIC/CONSTANT based on operand states. `current_stmt` is the
    /// statement whose body is currently being converted, threaded so
    /// `as_creation_display_name` can tell whether a creation is being
    /// rendered from its own defining statement or from elsewhere.
    fn proto_to_core_fir_sff_body(
        storage: &FVMStorage,
        ptr: FirPointer,
        current_stmt: Option<FirPointer>,
        depth: usize,
    ) -> core_fir::Fir {
        if depth > MAX_DEPTH {
            return NkFirBuilder::new("max recursion depth exceeded").build();
        }
        let cursor = FirCursor::new(ptr, storage);
        match cursor.node() {
            FirSpec::Search { .. } => {
                SearchFirBuilder::new(cursor.as_search_pattern().unwrap_or(""))
                    .anchored(cursor.as_search_anchored())
                    .state(Nyes::Embryonic)
                    .build()
            }
            FirSpec::Operator { .. } => {
                let op = cursor.as_op_name().unwrap_or("?").to_string();
                let operand_firs: Vec<core_fir::Fir> = cursor
                    .foolish_children()
                    .iter()
                    .map(|&c| proto_to_core_fir_sff_operand(storage, c, current_stmt, depth + 1))
                    .collect();
                let op_state = if operand_firs
                    .iter()
                    .any(|f| matches!(f.hs_state(), Nyes::Econstanic | Nyes::Woconstanic))
                {
                    Nyes::Woconstanic
                } else {
                    Nyes::Constant
                };
                OperatorFirBuilder::new(op)
                    .operands(operand_firs)
                    .state(op_state)
                    .build()
            }
            FirSpec::IndepInt { .. } => ConstantIntFirBuilder::new(cursor.as_i64().unwrap_or(0))
                .state(Nyes::Constant)
                .build(),
            FirSpec::Nk { .. } => NkFirBuilder::new(cursor.as_nk_reason().unwrap_or("unknown"))
                .state(Nyes::Nk)
                .build(),
            _ => proto_to_core_fir_inner(storage, ptr, true, current_stmt, depth + 1),
        }
    }

    fn proto_to_core_fir_sff_operand(
        storage: &FVMStorage,
        ptr: FirPointer,
        current_stmt: Option<FirPointer>,
        depth: usize,
    ) -> core_fir::Fir {
        if depth > MAX_DEPTH {
            return NkFirBuilder::new("max recursion depth exceeded").build();
        }
        let cursor = FirCursor::new(ptr, storage);
        match cursor.node() {
            FirSpec::Search { .. } => {
                SearchFirBuilder::new(cursor.as_search_pattern().unwrap_or(""))
                    .anchored(cursor.as_search_anchored())
                    .state(Nyes::Econstanic)
                    .build()
            }
            FirSpec::IndepInt { .. } => ConstantIntFirBuilder::new(cursor.as_i64().unwrap_or(0))
                .state(Nyes::Constant)
                .build(),
            FirSpec::Nk { .. } => NkFirBuilder::new(cursor.as_nk_reason().unwrap_or("unknown"))
                .state(Nyes::Nk)
                .build(),
            _ => proto_to_core_fir_inner(storage, ptr, true, current_stmt, depth + 1),
        }
    }

    /// Direct translation of `evaluator.rs`'s real `anchor_to_core_fir`.
    fn anchor_to_core_fir(
        storage: &FVMStorage,
        ptr: FirPointer,
        current_stmt: Option<FirPointer>,
        depth: usize,
    ) -> core_fir::Fir {
        let cursor = FirCursor::new(ptr, storage);
        let state = storage.get_nyes(ptr);
        if matches!(cursor.node(), FirSpec::Search { .. }) {
            return SearchFirBuilder::new(cursor.as_search_pattern().unwrap_or(""))
                .anchored(cursor.as_search_anchored())
                .state(state)
                .build();
        }
        proto_to_core_fir_inner(storage, ptr, true, current_stmt, depth + 1)
    }

    /// The producer of every OUTPUT line: converts one FIR node (and its
    /// tree of descendants) into the serializable `core_fir::Fir`
    /// representation.
    fn proto_to_core_fir_inner(
        storage: &FVMStorage,
        ptr: FirPointer,
        preserve_search: bool,
        current_stmt: Option<FirPointer>,
        depth: usize,
    ) -> core_fir::Fir {
        if depth > MAX_DEPTH {
            return NkFirBuilder::new("max recursion depth exceeded").build();
        }
        let cursor = FirCursor::new(ptr, storage);
        let state = storage.get_nyes(ptr);

        match cursor.node().clone() {
            FirSpec::IndepInt { .. } => ConstantIntFirBuilder::new(cursor.as_i64().unwrap_or(0))
                .state(state)
                .build(),
            FirSpec::Nk { .. } => {
                let reason = cursor.as_nk_reason().unwrap_or("unknown").to_string();
                let mut builder = NkFirBuilder::new(reason.as_str()).state(state);
                if reason == "division by zero" {
                    builder = builder.alarm(Alarm {
                        level: AlarmLevel::Mild,
                        code: "DIV-BY-ZERO".to_string(),
                        message: "Division by zero produces NK".to_string(),
                        source: AlarmSource::Evaluator,
                    });
                }
                builder.build()
            }
            // A comparison renders as its RESULT — see the real function's
            // own comment.
            FirSpec::Comparison { .. } => {
                if let Some(&result) = cursor.ubc_children().first() {
                    return proto_to_core_fir_inner(
                        storage,
                        result,
                        preserve_search,
                        current_stmt,
                        depth + 1,
                    );
                }
                NkFirBuilder::new("comparison").state(state).build()
            }
            FirSpec::Operator { .. } => {
                if state == Nyes::Constant
                    && let Some(&result) = cursor.ubc_children().first()
                {
                    return proto_to_core_fir_inner(
                        storage,
                        result,
                        preserve_search,
                        current_stmt,
                        depth + 1,
                    );
                }
                if state == Nyes::Nk {
                    let op_name = cursor.as_op_name().unwrap_or("").to_string();
                    let any_operand_nk = cursor
                        .foolish_children()
                        .iter()
                        .any(|&c| storage.get_nyes(c) == Nyes::Nk);
                    if !any_operand_nk
                        && op_name != "$"
                        && let Some(&result) = cursor.ubc_children().first()
                    {
                        return proto_to_core_fir_inner(
                            storage,
                            result,
                            preserve_search,
                            current_stmt,
                            depth + 1,
                        );
                    }
                }
                let op = cursor.as_op_name().unwrap_or("?").to_string();
                let operand_firs: Vec<core_fir::Fir> = if op == "$" {
                    cursor
                        .foolish_children()
                        .iter()
                        .enumerate()
                        .map(|(i, &c)| {
                            if i == 0 {
                                IndexFirBuilder::new(-1)
                                    .anchored(false)
                                    .state(Nyes::Econstanic)
                                    .build()
                            } else {
                                proto_to_core_fir_inner(
                                    storage,
                                    c,
                                    preserve_search,
                                    current_stmt,
                                    depth + 1,
                                )
                            }
                        })
                        .collect()
                } else {
                    cursor
                        .foolish_children()
                        .iter()
                        .map(|&c| {
                            proto_to_core_fir_inner(
                                storage,
                                c,
                                preserve_search,
                                current_stmt,
                                depth + 1,
                            )
                        })
                        .collect()
                };
                OperatorFirBuilder::new(op)
                    .operands(operand_firs)
                    .state(state)
                    .build()
            }
            FirSpec::Statement { .. } => {
                let name =
                    display_stmt_name(cursor.as_stmt_identifier().map(|id| id.searchable_name()));
                // Prefer settled_result() (the NF-refusal NK, if this
                // statement was refused) over the raw written body (FOOP-33
                // §4) — without this, a refusal is enforced internally but
                // never rendered: `'True = 3` would still SHOW `3` instead
                // of the NF NK.
                let body_fir = search_fir_dispatch::statement_value_for_comparison(storage, ptr)
                    .map(|c| {
                        proto_to_core_fir_inner(storage, c, preserve_search, Some(ptr), depth + 1)
                    })
                    .unwrap_or_else(|| NkFirBuilder::new("empty statement").build());
                NormalBraneFirBuilder::new()
                    .statement(name, body_fir)
                    .state(state)
                    .build()
            }
            FirSpec::Brane { .. } => {
                let stmt_tuples: Vec<(Option<String>, core_fir::Fir)> = cursor
                    .foolish_children()
                    .iter()
                    .map(|&c| {
                        let c_cursor = FirCursor::new(c, storage);
                        let name = display_stmt_name(
                            c_cursor.as_stmt_identifier().map(|id| id.searchable_name()),
                        );
                        // Prefer settled_result() over the raw written body —
                        // see the `Statement` arm above.
                        let body_fir =
                            search_fir_dispatch::statement_value_for_comparison(storage, c)
                                .map(|b| {
                                    proto_to_core_fir_inner(
                                        storage,
                                        b,
                                        preserve_search,
                                        Some(c),
                                        depth + 1,
                                    )
                                })
                                .unwrap_or_else(|| NkFirBuilder::new("empty body").build());
                        (name, body_fir)
                    })
                    .collect();
                let mut effective_state = state;
                if state == Nyes::Constant || state == Nyes::Independent {
                    for (_, body) in &stmt_tuples {
                        let body_state = body.hs_state();
                        if matches!(body_state, Nyes::Econstanic | Nyes::Woconstanic) {
                            effective_state = Nyes::Woconstanic;
                            break;
                        }
                        if body_state == Nyes::Nk {
                            effective_state = Nyes::Nk;
                            break;
                        }
                    }
                }
                let mut builder = NormalBraneFirBuilder::new()
                    .characterizations(cursor.as_brane_characterizations().to_vec())
                    .statements(stmt_tuples)
                    .state(effective_state);
                if let Some(reason) = storage.alarm_reason(ptr) {
                    builder = builder.alarm(Alarm {
                        level: AlarmLevel::Mild,
                        code: "ITERATION-EXCEEDED".to_string(),
                        message: reason.replace("ubca evaluation error: ", ""),
                        source: AlarmSource::Evaluator,
                    });
                }
                builder.build()
            }
            FirSpec::Search {
                is_value_search, ..
            } => {
                if state.is_constanic()
                    && let Some(&result) = cursor.ubc_children().first()
                {
                    // When the ubc_child is a settled search whose own
                    // ubc_child is a complex type (Brane, Operator, SF,
                    // SFF), this search came from unwrapping an SF
                    // value. UBC preserves the search wrapper in this
                    // case rather than resolving to the final value.
                    let result_is_search = matches!(storage.get(result), FirSpec::Search { .. });
                    if result_is_search && storage.get_nyes(result).is_constanic() {
                        let result_cursor = FirCursor::new(result, storage);
                        let inner_ubc = result_cursor.ubc_children();
                        let first_inner = inner_ubc.first().copied();
                        let has_complex = first_inner.is_some_and(|r| {
                            let is_complex_type = matches!(
                                storage.get(r),
                                FirSpec::Brane { .. }
                                    | FirSpec::Operator { .. }
                                    | FirSpec::StayFoolish
                                    | FirSpec::StayFullyFoolish
                            );
                            let has_resolved_value =
                                !FirCursor::new(r, storage).ubc_children().is_empty();
                            is_complex_type && !has_resolved_value
                        });
                        if has_complex {
                            let inner_fir = SearchFirBuilder::new(
                                result_cursor.as_search_pattern().unwrap_or(""),
                            )
                            .anchored(result_cursor.as_search_anchored())
                            .state(Nyes::Econstanic)
                            .build();
                            return SearchFirBuilder::new(cursor.as_search_pattern().unwrap_or(""))
                                .anchored(cursor.as_search_anchored())
                                .result(inner_fir)
                                .state(Nyes::Woconstanic)
                                .build();
                        }
                        // Simple result (IndepInt/NK): build inner
                        // search with resolved value.
                        let has_simple = first_inner.is_some_and(|r| {
                            matches!(
                                storage.get(r),
                                FirSpec::IndepInt { .. } | FirSpec::Nk { .. }
                            )
                        });
                        if has_simple {
                            let inner_result_fir = proto_to_core_fir_inner(
                                storage,
                                first_inner.unwrap(),
                                false,
                                current_stmt,
                                depth + 1,
                            );
                            return SearchFirBuilder::new(
                                result_cursor.as_search_pattern().unwrap_or(""),
                            )
                            .anchored(result_cursor.as_search_anchored())
                            .result(inner_result_fir)
                            .state(storage.get_nyes(result))
                            .build();
                        }
                    }

                    let resolved = proto_to_core_fir_inner(
                        storage,
                        result,
                        preserve_search,
                        current_stmt,
                        depth + 1,
                    );
                    if !preserve_search {
                        let resolved_state = storage.get_nyes(result);
                        if matches!(resolved_state, Nyes::Constant | Nyes::Independent) {
                            // `sf_inner_pattern`'s `Some` branch is NOT
                            // reachable here: `FirSpec::Search` carries no
                            // `sf_inner_pattern` field — it starts `None`
                            // always, and nothing sets it. This only
                            // affects rendering a search that itself
                            // resolved through an SF wrapper's own pattern
                            // substitution, a case no test in this crate's
                            // suite currently exercises.
                            return resolved;
                        }
                    }
                    return SearchFirBuilder::new(cursor.as_search_pattern().unwrap_or(""))
                        .anchored(cursor.as_search_anchored())
                        .result(resolved)
                        .state(state)
                        .build();
                }
                let mut builder = SearchFirBuilder::new(cursor.as_search_pattern().unwrap_or(""))
                    .anchored(cursor.as_search_anchored())
                    .state(state);
                if is_value_search {
                    builder = builder.is_value(true);
                    let children = cursor.foolish_children();
                    let has_anchor = cursor.as_search_anchored();
                    if has_anchor && let Some(&a) = children.first() {
                        builder = builder.anchor(proto_to_core_fir_inner(
                            storage,
                            a,
                            false,
                            current_stmt,
                            depth + 1,
                        ));
                    }
                    let value_idx = usize::from(has_anchor);
                    if let Some(&v) = children.get(value_idx) {
                        builder = builder.value(proto_to_core_fir_inner(
                            storage,
                            v,
                            false,
                            current_stmt,
                            depth + 1,
                        ));
                    }
                }
                if let Some(reason) = storage.alarm_reason(ptr) {
                    builder = builder.alarm(Alarm {
                        level: AlarmLevel::Mild,
                        code: "VALUE-SEARCH-UNSUPPORTED-PATTERN".to_string(),
                        message: reason.to_string(),
                        source: AlarmSource::Evaluator,
                    });
                }
                builder.build()
            }
            FirSpec::Index {
                offset, anchored, ..
            } => {
                if state.is_constanic()
                    && let Some(&result) = cursor.ubc_children().first()
                {
                    let resolved = proto_to_core_fir_inner(
                        storage,
                        result,
                        preserve_search,
                        current_stmt,
                        depth + 1,
                    );
                    let resolved_state = storage.get_nyes(result);
                    let result_is_brane = matches!(storage.get(result), FirSpec::Brane { .. });
                    if !preserve_search
                        && (matches!(resolved_state, Nyes::Constant | Nyes::Independent)
                            || result_is_brane)
                    {
                        return resolved;
                    }
                    let mut builder = IndexFirBuilder::new(offset)
                        .anchored(anchored)
                        .result(resolved)
                        .state(state);
                    if anchored && let Some(&anchor_ref) = cursor.foolish_children().first() {
                        builder = builder.anchor(anchor_to_core_fir(
                            storage,
                            anchor_ref,
                            current_stmt,
                            depth + 1,
                        ));
                    }
                    return builder.build();
                }
                let mut builder = IndexFirBuilder::new(offset).anchored(anchored).state(state);
                if anchored && let Some(&anchor_ref) = cursor.foolish_children().first() {
                    builder = builder.anchor(anchor_to_core_fir(
                        storage,
                        anchor_ref,
                        current_stmt,
                        depth + 1,
                    ));
                }
                builder.build()
            }
            FirSpec::StayFoolish => {
                let inner_ref = cursor.foolish_children().first().copied();
                // When the inner expression is itself a settled Search
                // whose OWN result is a "complex" kind (Brane/Operator/SF/
                // SFF), the search wrapper is preserved and rendered
                // UNWRAPPED — the outer `<...>` SF marker is NOT shown at
                // all: a detached-and-recoordinated SF value is presented
                // as its search, not its wrapper.
                if let Some(inner) = inner_ref {
                    let inner_spec = storage.get(inner).clone();
                    if matches!(inner_spec, FirSpec::Search { .. })
                        && storage.get_nyes(inner).is_constanic()
                    {
                        let inner_cursor = FirCursor::new(inner, storage);
                        if let Some(result) = inner_cursor.ubc_children().first().copied() {
                            let result_is_complex = matches!(
                                storage.get(result),
                                FirSpec::Brane { .. }
                                    | FirSpec::Operator { .. }
                                    | FirSpec::StayFoolish
                                    | FirSpec::StayFullyFoolish
                            );
                            if result_is_complex {
                                let result_cursor = FirCursor::new(result, storage);
                                if !result_cursor.ubc_children().is_empty()
                                    || storage.get_nyes(result).is_constanic()
                                {
                                    let inner_result_fir = proto_to_core_fir_inner(
                                        storage,
                                        result,
                                        false,
                                        current_stmt,
                                        depth + 1,
                                    );
                                    return SearchFirBuilder::new(
                                        inner_cursor.as_search_pattern().unwrap_or(""),
                                    )
                                    .anchored(inner_cursor.as_search_anchored())
                                    .result(inner_result_fir)
                                    .state(storage.get_nyes(inner))
                                    .build();
                                }
                                let search_fir = SearchFirBuilder::new(
                                    inner_cursor.as_search_pattern().unwrap_or(""),
                                )
                                .anchored(inner_cursor.as_search_anchored())
                                .state(Nyes::Econstanic)
                                .build();
                                return StayFoolishFirBuilder::new(search_fir)
                                    .state(Nyes::Woconstanic)
                                    .build();
                            }
                            if matches!(storage.get(result), FirSpec::Search { .. }) {
                                let result_cursor = FirCursor::new(result, storage);
                                let inner_fir = SearchFirBuilder::new(
                                    result_cursor.as_search_pattern().unwrap_or(""),
                                )
                                .anchored(result_cursor.as_search_anchored())
                                .state(Nyes::Econstanic)
                                .build();
                                let outer_search = SearchFirBuilder::new(
                                    inner_cursor.as_search_pattern().unwrap_or(""),
                                )
                                .anchored(inner_cursor.as_search_anchored())
                                .result(inner_fir)
                                .state(Nyes::Woconstanic)
                                .build();
                                return StayFoolishFirBuilder::new(outer_search)
                                    .state(Nyes::Woconstanic)
                                    .build();
                            }
                            if matches!(
                                storage.get(result),
                                FirSpec::IndepInt { .. } | FirSpec::Nk { .. }
                            ) {
                                let inner_result_fir = proto_to_core_fir_inner(
                                    storage,
                                    result,
                                    false,
                                    current_stmt,
                                    depth + 1,
                                );
                                return SearchFirBuilder::new(
                                    inner_cursor.as_search_pattern().unwrap_or(""),
                                )
                                .anchored(inner_cursor.as_search_anchored())
                                .result(inner_result_fir)
                                .state(storage.get_nyes(inner))
                                .build();
                            }
                        }
                    }
                }
                let expr_fir = inner_ref
                    .map(|c| proto_to_core_fir_inner(storage, c, true, current_stmt, depth + 1))
                    .unwrap_or_else(|| NkFirBuilder::new("empty sf").build());
                StayFoolishFirBuilder::new(expr_fir).state(state).build()
            }
            FirSpec::StayFullyFoolish => {
                let expr_fir = cursor
                    .foolish_children()
                    .first()
                    .map(|&c| proto_to_core_fir_sff_body(storage, c, current_stmt, depth + 1))
                    .unwrap_or_else(|| NkFirBuilder::new("empty sff").build());
                StayFullyFoolishFirBuilder::new(expr_fir)
                    .state(state)
                    .build()
            }
            FirSpec::Concatenation { provenance } => {
                let joined = !cursor.ubc_children().is_empty();
                let empty_done = matches!(state, Nyes::Constant | Nyes::Independent);
                if state.is_constanic() && (joined || empty_done) {
                    let count = cursor.stmt_count().unwrap_or(0);
                    let stmt_tuples: Vec<(Option<String>, core_fir::Fir)> = (0..count)
                        .filter_map(|i| {
                            let stmt = cursor.stmt_at(i)?;
                            let s_cursor = FirCursor::new(stmt, storage);
                            let name = display_stmt_name(
                                s_cursor.as_stmt_identifier().map(|id| id.searchable_name()),
                            );
                            let body_fir = s_cursor
                                .foolish_children()
                                .first()
                                .map(|&c| {
                                    proto_to_core_fir_inner(
                                        storage,
                                        c,
                                        preserve_search,
                                        Some(stmt),
                                        depth + 1,
                                    )
                                })
                                .unwrap_or_else(|| NkFirBuilder::new("empty body").build());
                            Some((name, body_fir))
                        })
                        .collect();
                    let mut effective_state = state;
                    if state == Nyes::Constant || state == Nyes::Independent {
                        for (_, body) in &stmt_tuples {
                            let body_state = body.hs_state();
                            if matches!(body_state, Nyes::Econstanic | Nyes::Woconstanic) {
                                effective_state = Nyes::Woconstanic;
                                break;
                            }
                            if body_state == Nyes::Nk {
                                effective_state = Nyes::Nk;
                                break;
                            }
                        }
                    }
                    return NormalBraneFirBuilder::new()
                        .statements(stmt_tuples)
                        .state(effective_state)
                        .build();
                }
                let elem_firs: Vec<core_fir::Fir> = cursor
                    .foolish_children()
                    .iter()
                    .map(|&c| {
                        proto_to_core_fir_inner(
                            storage,
                            c,
                            preserve_search,
                            current_stmt,
                            depth + 1,
                        )
                    })
                    .collect();
                let is_tail = provenance == ConcatProvenance::TailConcatenation;
                ConcatenationFirBuilder::new()
                    .elements(elem_firs)
                    .state(state)
                    .is_tail_concatenation(is_tail)
                    .build()
            }
            FirSpec::ConcatHelper => {
                let stmt_tuples: Vec<(Option<String>, core_fir::Fir)> = cursor
                    .foolish_children()
                    .iter()
                    .map(|&c| {
                        let c_cursor = FirCursor::new(c, storage);
                        let name = display_stmt_name(
                            c_cursor.as_stmt_identifier().map(|id| id.searchable_name()),
                        );
                        let body_fir = c_cursor
                            .foolish_children()
                            .first()
                            .map(|&b| {
                                proto_to_core_fir_inner(
                                    storage,
                                    b,
                                    preserve_search,
                                    Some(c),
                                    depth + 1,
                                )
                            })
                            .unwrap_or_else(|| NkFirBuilder::new("empty body").build());
                        (name, body_fir)
                    })
                    .collect();
                NormalBraneFirBuilder::new()
                    .statements(stmt_tuples)
                    .state(state)
                    .build()
            }
            FirSpec::FoolRef { .. } => NkFirBuilder::new("unknown fir kind").build(),
            FirSpec::Creation => {
                let mut builder = CreationFirBuilder::new();
                if let Some(name) = cursor.as_creation_display_name(current_stmt) {
                    builder = builder.name(name);
                }
                builder.build()
            }
        }
    }
}

/// AST→FIR construction — the compiler `UbcaEvaluator::evaluate` drives
/// (via `compose_program_with_system`/`program_result`, re-exported at this
/// file's top level).
mod arena_compiler {
    use super::{
        ANON_STMT_NAME, ConcatProvenance, FVMStorage, FirCursor, FirCursorMut, FirPointer, FirSpec,
    };

    use foolish_core::fir::Nyes;
    use foolish_parser::{AssignmentOperator, Astn, SearchOperator};

    use crate::identifier::{Characterizations, Identifier};

    /// Element types allowed inside a ConcatBrane.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum ConcatElemKind {
        BareBrane,
        BareConcat,
        BareSearch,
        SfSearch,
        SfBrane,
        Error,
    }

    /// Rejects AST shapes this crate doesn't support (before any FIR
    /// construction begins) — a plain, storage-independent AST walk with no
    /// `FirPointer` involvement.
    fn validate_astn(ast: &Astn) -> anyhow::Result<()> {
        match ast {
            Astn::IfExpr { .. } => anyhow::bail!("if-then-else: not supported (FOOP=2)"),
            Astn::UpwardSearch => anyhow::bail!("Upward search: deferred"),
            Astn::DetachmentBrane { .. } => anyhow::bail!("Detachment brane: deferred"),
            Astn::NotImplemented(r) => anyhow::bail!("Not yet implemented: {}", r),
            Astn::Brane { statements, .. } => {
                for s in statements {
                    validate_astn(s)?;
                }
                Ok(())
            }
            Astn::Assignment { expr, .. } => validate_astn(expr),
            Astn::BinaryOp { left, right, .. } => {
                validate_astn(left)?;
                validate_astn(right)
            }
            Astn::UnaryOp { expr, .. } => validate_astn(expr),
            Astn::DotSearch { anchor, .. } => validate_astn(anchor),
            Astn::RegexpSearch { anchor, .. } => {
                if let Some(a) = anchor {
                    validate_astn(a)?;
                }
                Ok(())
            }
            Astn::ValueSearch {
                anchor,
                value_pattern,
                ..
            } => {
                if let Some(a) = anchor {
                    validate_astn(a)?;
                }
                validate_astn(value_pattern)
            }
            Astn::Seek { anchor, .. } => validate_astn(anchor),
            Astn::HeadTail { anchor, .. } => validate_astn(anchor),
            Astn::ContextedSearch { inner } => validate_astn(inner),
            Astn::Concatenation { elements } => {
                for e in elements {
                    validate_astn(e)?;
                }
                Ok(())
            }
            Astn::TailConcatenation { elements } => {
                for e in elements {
                    validate_astn(e)?;
                }
                Ok(())
            }
            Astn::StayFoolish { expr } => validate_astn(expr),
            Astn::StayFullyFoolish { expr } => validate_astn(expr),
            Astn::IntLit(_)
            | Astn::UnknownLit
            | Astn::Creation
            | Astn::Identifier { .. }
            | Astn::UnanchoredSeek { .. } => Ok(()),
        }
    }

    /// Byte-for-byte copy of `compiler.rs`'s real (private)
    /// `classify_concat_element` — plain AST
    /// classification, no `FirPointer` involvement, duplicated for the same
    /// reason as `ConcatElemKind`/`validate_astn` above.
    fn classify_concat_element(ast: &Astn) -> ConcatElemKind {
        match ast {
            Astn::Brane { .. } => ConcatElemKind::BareBrane,
            Astn::Concatenation { .. } => ConcatElemKind::BareConcat,
            Astn::Identifier { .. }
            | Astn::DotSearch { .. }
            | Astn::RegexpSearch { .. }
            | Astn::Seek { .. }
            | Astn::HeadTail { .. }
            | Astn::UnanchoredSeek { .. }
            | Astn::ValueSearch { .. } => ConcatElemKind::BareSearch,
            Astn::ContextedSearch { inner } => {
                if matches!(
                    inner.as_ref(),
                    Astn::Identifier { .. }
                        | Astn::DotSearch { .. }
                        | Astn::RegexpSearch { .. }
                        | Astn::Seek { .. }
                        | Astn::HeadTail { .. }
                        | Astn::UnanchoredSeek { .. }
                        | Astn::ValueSearch { .. }
                ) {
                    ConcatElemKind::BareSearch
                } else {
                    ConcatElemKind::Error
                }
            }
            Astn::StayFoolish { expr } => match expr.as_ref() {
                Astn::Brane { .. } => ConcatElemKind::SfBrane,
                Astn::Identifier { .. }
                | Astn::DotSearch { .. }
                | Astn::RegexpSearch { .. }
                | Astn::Seek { .. }
                | Astn::HeadTail { .. }
                | Astn::UnanchoredSeek { .. }
                | Astn::ValueSearch { .. }
                | Astn::ContextedSearch { .. } => ConcatElemKind::SfSearch,
                _ => ConcatElemKind::Error,
            },
            Astn::StayFullyFoolish { expr } => match expr.as_ref() {
                Astn::Brane { .. } => ConcatElemKind::SfBrane,
                Astn::Identifier { .. }
                | Astn::DotSearch { .. }
                | Astn::RegexpSearch { .. }
                | Astn::Seek { .. }
                | Astn::HeadTail { .. }
                | Astn::UnanchoredSeek { .. }
                | Astn::ValueSearch { .. }
                | Astn::ContextedSearch { .. } => ConcatElemKind::SfSearch,
                _ => ConcatElemKind::Error,
            },
            _ => ConcatElemKind::Error,
        }
    }

    /// `parent` is the ALREADY-CREATED arena parent (the
    /// `Concatenation`/`ConcatHelper` node), so each wrapper here is one
    /// `create_child` call.
    fn build_concat_element(
        storage: &mut FVMStorage,
        ast: Astn,
        parent: FirPointer,
        under_sff: bool,
    ) -> FirPointer {
        match classify_concat_element(&ast) {
            ConcatElemKind::BareBrane => build_fir(storage, ast, Some(parent), true),
            ConcatElemKind::BareConcat => build_fir(storage, ast, Some(parent), under_sff),
            ConcatElemKind::BareSearch => {
                let sf = parent.create_child(storage, FirSpec::StayFoolish);
                build_fir(storage, ast, Some(sf), under_sff);
                sf
            }
            ConcatElemKind::SfSearch => build_fir(storage, ast, Some(parent), under_sff),
            ConcatElemKind::SfBrane => {
                let sff = parent.create_child(storage, FirSpec::StayFullyFoolish);
                build_fir(storage, ast, Some(sff), false);
                sff
            }
            ConcatElemKind::Error => parent.create_child(
                storage,
                FirSpec::Nk {
                    reason: "invalid concatenation element".to_string(),
                },
            ),
        }
    }

    /// `parent: None` means build a ROOT (self-parented via
    /// `FVMStorage::make_root`); `Some(p)` means a child of `p` (via
    /// `create_child`). Every arm that recurses builds its OWN node FIRST —
    /// `create_child`/`make_root` need no placeholder-then-mutate step, since
    /// they need only the node's final field values (tree structure is
    /// handled generically by the arena itself) — so the order is:
    /// construct this node, getting its `FirPointer` immediately, THEN
    /// build children as its `create_child`s.
    fn build_fir(
        storage: &mut FVMStorage,
        ast: Astn,
        parent: Option<FirPointer>,
        under_sff: bool,
    ) -> FirPointer {
        let search_nyes = if under_sff {
            Nyes::Econstanic
        } else {
            Nyes::Prembrionic
        };
        macro_rules! child_parent {
            () => {
                parent.expect("non-Brane FIR must have a parent — only a Brane can be root")
            };
        }
        match ast {
            Astn::IntLit(n) => {
                child_parent!().create_child(storage, FirSpec::IndepInt { value: n as i64 })
            }
            Astn::UnknownLit => child_parent!().create_child(
                storage,
                FirSpec::Nk {
                    reason: "??? literal".to_string(),
                },
            ),
            Astn::Creation => child_parent!().create_child(storage, FirSpec::Creation),
            Astn::Identifier {
                characterizations,
                id,
            } => {
                // Fold characterizations back into the search pattern (Gotcha #3).
                let full_pattern = if characterizations.is_empty() {
                    id.clone()
                } else {
                    let char_str: String =
                        characterizations.iter().map(|c| format!("{c}'")).collect();
                    format!("{char_str}{id}")
                };
                let node = child_parent!().create_child(
                    storage,
                    FirSpec::Search {
                        pattern: format!("^{full_pattern}$"),
                        anchored: false,
                        forward: false,
                        is_value_search: false,
                        contexted: false,
                    },
                );
                storage.with_mut(node, |fir| fir.set_nyes(search_nyes));
                node
            }
            Astn::Brane {
                characterizations,
                statements,
            } => {
                let brane = match parent {
                    Some(p) => p.create_child(
                        storage,
                        FirSpec::Brane {
                            characterizations: Characterizations::from_brane_parts(
                                characterizations,
                            ),
                        },
                    ),
                    None => storage.make_root(FirSpec::Brane {
                        characterizations: Characterizations::from_brane_parts(characterizations),
                    }),
                };
                build_stmts(storage, statements, brane, under_sff);
                brane
            }
            Astn::BinaryOp { op, left, right } => {
                let node = child_parent!().create_child(storage, FirSpec::Operator { op });
                build_fir(storage, *left, Some(node), under_sff);
                build_fir(storage, *right, Some(node), under_sff);
                node
            }
            Astn::UnaryOp { op, expr } => {
                let node = child_parent!().create_child(storage, FirSpec::Operator { op });
                build_fir(storage, *expr, Some(node), under_sff);
                node
            }
            Astn::DotSearch { anchor, coordinate } => {
                let node = child_parent!().create_child(
                    storage,
                    FirSpec::Search {
                        pattern: format!("^{coordinate}$"),
                        anchored: true,
                        forward: false,
                        is_value_search: false,
                        contexted: false,
                    },
                );
                storage.with_mut(node, |fir| fir.set_nyes(search_nyes));
                build_fir(storage, *anchor, Some(node), under_sff);
                node
            }
            Astn::RegexpSearch {
                anchor,
                pattern,
                operator,
                ..
            } => {
                let has_anchor = anchor.is_some();
                let node = child_parent!().create_child(
                    storage,
                    FirSpec::Search {
                        pattern,
                        anchored: has_anchor,
                        forward: operator == SearchOperator::RegexpForward,
                        is_value_search: false,
                        contexted: false,
                    },
                );
                storage.with_mut(node, |fir| fir.set_nyes(search_nyes));
                if let Some(a) = anchor {
                    build_fir(storage, *a, Some(node), under_sff);
                }
                node
            }
            Astn::ValueSearch {
                anchor,
                forward,
                name_pattern,
                value_pattern,
            } => {
                let has_anchor = anchor.is_some();
                let pattern = name_pattern.unwrap_or_default();
                let node = child_parent!().create_child(
                    storage,
                    FirSpec::Search {
                        pattern,
                        anchored: has_anchor,
                        forward,
                        is_value_search: true,
                        contexted: false,
                    },
                );
                storage.with_mut(node, |fir| fir.set_nyes(search_nyes));
                if let Some(a) = anchor {
                    build_fir(storage, *a, Some(node), under_sff);
                }
                build_fir(storage, *value_pattern, Some(node), under_sff);
                node
            }
            Astn::Seek { anchor, offset } => {
                let node = child_parent!().create_child(
                    storage,
                    FirSpec::Index {
                        offset,
                        anchored: true,
                        contexted: false,
                    },
                );
                storage.with_mut(node, |fir| fir.set_nyes(search_nyes));
                build_fir(storage, *anchor, Some(node), under_sff);
                node
            }
            Astn::HeadTail { is_head, anchor } => {
                let offset = if is_head { 0 } else { -1 };
                let node = child_parent!().create_child(
                    storage,
                    FirSpec::Index {
                        offset,
                        anchored: true,
                        contexted: false,
                    },
                );
                storage.with_mut(node, |fir| fir.set_nyes(search_nyes));
                build_fir(storage, *anchor, Some(node), under_sff);
                node
            }
            Astn::UnanchoredSeek { offset } => {
                let node = child_parent!().create_child(
                    storage,
                    FirSpec::Index {
                        offset,
                        anchored: false,
                        contexted: false,
                    },
                );
                storage.with_mut(node, |fir| fir.set_nyes(search_nyes));
                node
            }
            Astn::Concatenation { elements } => {
                let node = child_parent!().create_child(
                    storage,
                    FirSpec::Concatenation {
                        provenance: ConcatProvenance::Juxtaposition,
                    },
                );
                for e in elements {
                    build_concat_element(storage, e, node, under_sff);
                }
                node
            }
            Astn::TailConcatenation { elements } => {
                let node = child_parent!().create_child(
                    storage,
                    FirSpec::Concatenation {
                        provenance: ConcatProvenance::TailConcatenation,
                    },
                );
                for e in elements.into_iter().rev() {
                    build_concat_element(storage, e, node, under_sff);
                }
                node
            }
            Astn::StayFoolish { expr } => {
                let node = child_parent!().create_child(storage, FirSpec::StayFoolish);
                build_fir(storage, *expr, Some(node), under_sff);
                node
            }
            Astn::StayFullyFoolish { expr } => {
                let node = child_parent!().create_child(storage, FirSpec::StayFullyFoolish);
                // SFF marker: from here down, searches are built ECONSTANIC.
                let e = build_fir(storage, *expr, Some(node), true);
                // Sanity-check that `under_sff` actually reached every
                // descendant search — mirrors the real
                // `push_foolish_child_sff_marked` call exactly (the arena's
                // `create_child` above already did the "push" half; this is
                // purely the invariant CHECK, run after the fact since the
                // arena wires parent/child atomically at construction).
                let cursor = FirCursorMut::new(node, storage);
                cursor.check_sff_marked_child(e);
                node
            }
            Astn::ContextedSearch { inner } => {
                let node = build_fir(storage, *inner, parent, under_sff);
                storage.with_mut(node, |fir| fir.set_contexted(true));
                node
            }
            Astn::Assignment { .. } => {
                unreachable!("standalone Assignment should be wrapped in Brane by parser")
            }
            _ => unreachable!("validate_astn should have rejected this"),
        }
    }

    /// Direct translation of `compiler.rs`'s real `build_stmts`.
    fn build_stmts(storage: &mut FVMStorage, asts: Vec<Astn>, parent: FirPointer, under_sff: bool) {
        for (i, ast) in asts.into_iter().enumerate() {
            build_as_statement(storage, ast, parent, i, under_sff);
        }
    }

    /// Direct translation of `compiler.rs`'s real `AstnCompilerExt::
    /// build_as_statement_inner` (the shared body behind
    /// `build_as_statement`/`build_as_statement_overridden`; the
    /// `override_body` hook itself — `system_foo.rs`'s comparison-operator
    /// injection — is NOT translated here, since `system_foo.rs`'s own
    /// arena migration is out of this task's scope; only the ordinary,
    /// unoverridden path is implemented).
    fn build_as_statement(
        storage: &mut FVMStorage,
        ast: Astn,
        parent: FirPointer,
        line: usize,
        under_sff: bool,
    ) -> FirPointer {
        let (characterizations, name, expr, operator) = match ast {
            Astn::Assignment {
                characterizations,
                identifier,
                operator,
                expr,
            } => (characterizations, identifier, *expr, operator),
            other => (
                vec![],
                ANON_STMT_NAME.to_string(),
                other,
                AssignmentOperator::Assign,
            ),
        };
        let identifier = Identifier::from_parts(characterizations, &name);
        let stmt = parent.create_child(
            storage,
            FirSpec::Statement {
                identifier,
                line_number: line,
            },
        );
        build_expr_with_operator(storage, expr, operator, stmt, under_sff);
        stmt
    }

    /// Direct translation of `compiler.rs`'s real `AstnCompilerExt::
    /// build_expr_with_operator`.
    fn build_expr_with_operator(
        storage: &mut FVMStorage,
        ast: Astn,
        operator: AssignmentOperator,
        parent: FirPointer,
        under_sff: bool,
    ) -> FirPointer {
        let body_under_sff = under_sff || operator == AssignmentOperator::SFF;
        match operator {
            AssignmentOperator::Assign => build_fir(storage, ast, Some(parent), body_under_sff),
            AssignmentOperator::SF => {
                let sf = parent.create_child(storage, FirSpec::StayFoolish);
                build_fir(storage, ast, Some(sf), body_under_sff);
                sf
            }
            AssignmentOperator::SFF => {
                let sff = parent.create_child(storage, FirSpec::StayFullyFoolish);
                build_fir(storage, ast, Some(sff), body_under_sff);
                sff
            }
        }
    }

    /// A plain, no-system.foo compile path.
    ///
    /// No production caller: `UbcaEvaluator::evaluate`'s real body goes
    /// through `compose_program_with_system`/`compose_one`/
    /// `compile_root_with_body_override` instead — system.foo composition
    /// is not opt-in (FOOP-33 §4). Kept and exercised by this file's own
    /// tests.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "no production caller — evaluate goes through compose_program_with_system, \
                      not this plain compile path; exercised only by this file's own tests"
        )
    )]
    pub(crate) fn compile_standalone(
        storage: &mut FVMStorage,
        ast: Astn,
    ) -> anyhow::Result<FirPointer> {
        validate_astn(&ast)?;
        if !matches!(ast, Astn::Brane { .. }) {
            anyhow::bail!("only a Brane can be a top-level (root) node");
        }
        Ok(build_fir(storage, ast, None, false))
    }

    /// No production caller — see `compile_standalone`'s doc comment.
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "see compile_standalone's doc comment")
    )]
    pub(crate) fn compile(
        storage: &mut FVMStorage,
        source: &str,
    ) -> anyhow::Result<Vec<FirPointer>> {
        let asts = foolish_parser::parse(source)?;
        asts.into_iter()
            .map(|ast| compile_standalone(storage, ast))
            .collect()
    }

    /// Parses `source`, requires it to be exactly one top-level brane with
    /// exactly one (assignment) statement, then builds ONLY that
    /// statement's body under `parent` via `build_expr_with_operator` —
    /// never wrapping it in a `Statement`/`Brane` of its own. Used by
    /// `system_foo`'s comparison-operator installer to compile each fixed
    /// `OPERAND_SRC` fragment (`"{o = <<#-2>>;}"`) directly beneath the
    /// `ComparisonFir` node.
    pub(crate) fn compile_stmt_body_under(
        storage: &mut FVMStorage,
        source: &str,
        parent: FirPointer,
    ) -> anyhow::Result<FirPointer> {
        let asts = foolish_parser::parse(source)?;
        let [ast] = <[Astn; 1]>::try_from(asts).map_err(|v| {
            anyhow::anyhow!("expected exactly one top-level brane, found {}", v.len())
        })?;
        validate_astn(&ast)?;
        let Astn::Brane { mut statements, .. } = ast else {
            anyhow::bail!("expected a brane");
        };
        if statements.len() != 1 {
            anyhow::bail!("expected exactly one statement, found {}", statements.len());
        }
        let Astn::Assignment { expr, operator, .. } = statements.remove(0) else {
            anyhow::bail!("expected an assignment");
        };
        Ok(build_expr_with_operator(
            storage, *expr, operator, parent, false,
        ))
    }

    /// A body-override hook: takes `&mut FVMStorage` (needed to construct a
    /// replacement body) and the STATEMENT's own `FirPointer`. Returns
    /// `Some(body)` to supply that body INSTEAD of the ordinary compiled
    /// one, or `None` to fall through to normal construction.
    pub(crate) type ArenaBodyOverride<'a> =
        &'a dyn Fn(&Identifier, &mut FVMStorage, FirPointer) -> Option<FirPointer>;

    /// Builds ONE statement, consulting `override_body` first (by the
    /// statement's OWN identifier) before falling through to the ordinary
    /// `build_expr_with_operator` path `build_as_statement` uses. Kept as
    /// its own function since `override_body` is threaded ONLY at the top
    /// level of `compile_root_with_body_override`'s own statement loop —
    /// system.foo's own top-level statements are the only call site in
    /// this module (nested branes, concatenation elements, etc.) needs this
    /// this crate that ever needs this override parameter at all.
    fn build_as_statement_overridden(
        storage: &mut FVMStorage,
        ast: Astn,
        parent: FirPointer,
        line: usize,
        override_body: ArenaBodyOverride<'_>,
    ) -> FirPointer {
        let (characterizations, name, expr, operator) = match ast {
            Astn::Assignment {
                characterizations,
                identifier,
                operator,
                expr,
            } => (characterizations, identifier, *expr, operator),
            other => (
                vec![],
                ANON_STMT_NAME.to_string(),
                other,
                AssignmentOperator::Assign,
            ),
        };
        let identifier = Identifier::from_parts(characterizations, &name);
        let stmt = parent.create_child(
            storage,
            FirSpec::Statement {
                identifier: identifier.clone(),
                line_number: line,
            },
        );
        match override_body(&identifier, storage, stmt) {
            Some(_body) => {
                // The override already built its replacement body AS a
                // child of `stmt` (matching this arena's strictly-top-down
                // construction discipline — see `compile_stmt_body_under`'s
                // own `parent` parameter). Nothing more to do here.
            }
            None => {
                build_expr_with_operator(storage, expr, operator, stmt, false);
            }
        }
        stmt
    }

    /// Arena counterpart to `compiler.rs`'s real `compile_root_with_body_
    /// override`:
    /// compile a top-level brane AST as a self-rooting root, letting
    /// `override_body` replace individual statements' bodies. Identical to
    /// `compile_standalone` except for the per-statement hook.
    pub(crate) fn compile_root_with_body_override(
        storage: &mut FVMStorage,
        ast: Astn,
        override_body: ArenaBodyOverride<'_>,
    ) -> anyhow::Result<FirPointer> {
        validate_astn(&ast)?;
        let Astn::Brane {
            characterizations,
            statements,
        } = ast
        else {
            anyhow::bail!("only a Brane can be a top-level (root) node");
        };
        let root = storage.make_root(FirSpec::Brane {
            characterizations: Characterizations::from_brane_parts(characterizations),
        });
        for (i, stmt_ast) in statements.into_iter().enumerate() {
            build_as_statement_overridden(storage, stmt_ast, root, i, override_body);
        }
        Ok(root)
    }

    /// Builds a `FirSpec::Comparison` node with its two SFF-marked operand
    /// lookups, compiled from `system_foo::OPERAND_SRC`'s fixed Foolish
    /// source via `compile_stmt_body_under`. The operands are compiled from
    /// source, not hand-built, specifically so `build_fir`'s `under_sff`
    /// rule applies to them exactly like any other SFF-marked expression —
    /// no separate panic-guard is needed beyond the ordinary `under_sff`
    /// propagation through `build_fir`/`build_expr_with_operator`.
    pub(crate) fn build_comparison(
        storage: &mut FVMStorage,
        op: crate::system_foo::ComparisonOp,
        parent: FirPointer,
    ) -> FirPointer {
        let cmp = parent.create_child(storage, FirSpec::Comparison { op });
        for src in crate::system_foo::OPERAND_SRC {
            compile_stmt_body_under(storage, src, cmp)
                .expect("OPERAND_SRC is a fixed, valid Foolish expression");
        }
        cmp
    }

    /// Supplies a `Comparison`-shaped body for each comparison operator's
    /// `system.foo` statement, matched by the statement's OWN
    /// null-characterized searchable name against `ComparisonOp::ALL`.
    /// Returns `None` (fall through to ordinary construction) for every
    /// other statement — this hook runs ONLY over `system.foo`'s own
    /// top-level statements, never over user source.
    pub(crate) fn comparison_body(
        identifier: &Identifier,
        storage: &mut FVMStorage,
        stmt: FirPointer,
    ) -> Option<FirPointer> {
        let name = identifier.searchable_name();
        let op = crate::system_foo::ComparisonOp::from_searchable_name(name)?;
        Some(build_comparison(storage, op, stmt))
    }

    /// Composes `system.foo` with a single user program's AST, appended as
    /// a statement named `program` (last), and compiles the combined AST as
    /// one self-rooting brane via `compile_root_with_body_override` with
    /// `comparison_body` as the hook.
    pub(crate) fn compose_one(
        storage: &mut FVMStorage,
        system_ast: Astn,
        program_ast: Astn,
    ) -> anyhow::Result<FirPointer> {
        let Astn::Brane {
            characterizations,
            mut statements,
        } = system_ast
        else {
            anyhow::bail!("system.foo must parse to exactly one top-level brane, found 0");
        };
        statements.push(Astn::Assignment {
            characterizations: vec![],
            identifier: "program".to_string(),
            operator: AssignmentOperator::Assign,
            expr: Box::new(program_ast),
        });
        let composed = Astn::Brane {
            characterizations,
            statements,
        };
        compile_root_with_body_override(storage, composed, &comparison_body)
    }

    /// Parses `system.foo` and the user's source, composing each of the
    /// user's top-level items with `system.foo` per [`compose_one`].
    pub(crate) fn compose_program_with_system(
        storage: &mut FVMStorage,
        user_source: &str,
    ) -> anyhow::Result<Vec<FirPointer>> {
        let program_asts = foolish_parser::parse(user_source)?;
        program_asts
            .into_iter()
            .map(|program_ast| {
                let system_asts = foolish_parser::parse(crate::system_foo::SYSTEM_FOO_SRC)?;
                let [system_ast] = <[Astn; 1]>::try_from(system_asts).map_err(|v| {
                    anyhow::anyhow!(
                        "system.foo must parse to exactly one top-level brane, found {}",
                        v.len()
                    )
                })?;
                compose_one(storage, system_ast, program_ast)
            })
            .collect()
    }

    /// Extracts the `program` member's VALUE from a composed root — the
    /// LAST statement of the composite brane (FOOP-33 §4). Structural
    /// access (`stmt_count`/`stmt_at`), never a Foolish search. `.value()`
    /// on the STATEMENT itself would just return the statement (a plain
    /// `Statement` has no settled result in the common case), so this
    /// resolves through `foolish_children().first()` (the written body)
    /// first, THEN `.value()`.
    pub(crate) fn program_result(
        storage: &FVMStorage,
        composed_root: FirPointer,
    ) -> Option<FirPointer> {
        let count = FirCursor::new(composed_root, storage).stmt_count()?;
        if count == 0 {
            return None;
        }
        let last_stmt = FirCursor::new(composed_root, storage).stmt_at(count - 1)?;
        let body = storage.foolish_children(last_stmt).first().copied()?;
        Some(body.value(storage))
    }
}

/// Minimal re-export surface for `UbcaEvaluator::evaluate` —
/// `arena_compiler`/`core_fir_conversion` themselves stay private modules;
/// only the exact functions `evaluate`'s body needs are re-exported, not
/// the modules' full surface.
pub(crate) use arena_compiler::{compose_program_with_system, program_result};
pub(crate) use core_fir_conversion::{proto_to_core_fir, step_to_settled};

#[cfg(test)]
mod tests {
    use super::*;

    fn brane_spec() -> FirSpec {
        FirSpec::Brane {
            characterizations: Characterizations::default(),
        }
    }

    /// A small, self-contained arena exercising `get`/`with_mut`/`get_parent`
    /// round-tripping, per this task's "Establish relevant tests" checkbox.
    #[test]
    fn make_root_then_child_round_trips_through_get_and_with_mut() {
        let mut storage = FVMStorage::new();
        let root = storage.make_root(brane_spec());
        let child = root.create_child(&mut storage, FirSpec::IndepInt { value: 42 });

        assert_eq!(storage.get(child), &FirSpec::IndepInt { value: 42 });
        assert_eq!(child.get_parent(&storage), Some(root));
        assert!(root.is_root(&storage));
        assert!(!child.is_root(&storage));
        assert_eq!(storage.foolish_children(root), &[child]);

        storage.with_mut(child, |fir| fir.set_nyes(Nyes::Constant));
        assert_eq!(storage.get_nyes(child), Nyes::Constant);
    }

    /// A freshly-created node starts at its spec's initial `Nyes` — the same
    /// starting state each kind's own constructor established when nodes were
    /// built one-by-one rather than allocated from an arena.
    #[test]
    fn initial_nyes_matches_each_kinds_own_constructor() {
        let mut storage = FVMStorage::new();
        let root = storage.make_root(brane_spec());
        assert_eq!(storage.get_nyes(root), Nyes::Prembrionic);

        let creation = root.create_child(&mut storage, FirSpec::Creation);
        assert_eq!(storage.get_nyes(creation), Nyes::Independent);

        let int_child = root.create_child(&mut storage, FirSpec::IndepInt { value: 1 });
        assert_eq!(storage.get_nyes(int_child), Nyes::Independent);

        let op_child = root.create_child(
            &mut storage,
            FirSpec::Operator {
                op: "+".to_string(),
            },
        );
        assert_eq!(storage.get_nyes(op_child), Nyes::Prembrionic);
    }

    /// A `FirPointer` minted by one `FVMStorage` must never validate against a
    /// different instance — the whole reason `arena: ArenaId` exists (see
    /// FOOP-16.md §Specification "`FirPointer`'s identity properties").
    #[test]
    #[should_panic(expected = "different FVMStorage instance")]
    fn pointer_from_a_different_arena_fails_validation() {
        let mut storage_a = FVMStorage::new();
        let root_a = storage_a.make_root(brane_spec());

        let storage_b = FVMStorage::new();
        // root_a was minted by storage_a; reading it through storage_b must panic.
        let _ = storage_b.get(root_a);
    }

    /// `get_mut` gives the same access as `with_mut`, just without the
    /// closure — confirms both really are equally powerful, per FOOP-16.md's
    /// resolution of the two-cursor-type design question.
    #[test]
    fn get_mut_and_with_mut_reach_the_same_state() {
        let mut storage = FVMStorage::new();
        let root = storage.make_root(brane_spec());
        let child = root.create_child(&mut storage, FirSpec::IndepInt { value: 7 });

        storage.get_mut(child).set_nyes(Nyes::Constant);
        assert_eq!(storage.get_nyes(child), Nyes::Constant);
    }

    /// The structural root has no home brane (it climbs to itself and stops).
    #[test]
    fn home_brane_of_the_structural_root_is_none() {
        let mut storage = FVMStorage::new();
        let root = storage.make_root(FirSpec::IndepInt { value: 1 });
        assert_eq!(root.home_brane(&storage), None);
    }

    /// A child of a `Brane` reports that brane as its home brane; a child of
    /// a non-brane-like node (here, a bare `IndepInt` root standing in for
    /// "some non-brane-like ancestor") climbs past it with no brane to find.
    #[test]
    fn home_brane_finds_the_nearest_brane_like_ancestor() {
        let mut storage = FVMStorage::new();
        let root = storage.make_root(brane_spec());
        let inner_int = root.create_child(&mut storage, FirSpec::IndepInt { value: 1 });
        assert_eq!(inner_int.home_brane(&storage), Some(root));

        let non_brane_root = storage.make_root(FirSpec::IndepInt { value: 1 });
        let grandchild = non_brane_root.create_child(&mut storage, FirSpec::IndepInt { value: 2 });
        assert_eq!(grandchild.home_brane(&storage), None);
    }

    /// `test_leaf`/`test_root_brane` round-trip correctly.
    #[test]
    fn test_helpers_build_expected_shapes() {
        let (storage, leaf) = FVMStorage::test_leaf(Nyes::Constant);
        assert_eq!(storage.get_nyes(leaf), Nyes::Constant);
        assert!(leaf.is_root(&storage));

        let (storage, root) =
            FVMStorage::test_root_brane(&[FirSpec::IndepInt { value: 1 }, FirSpec::Creation]);
        assert_eq!(storage.foolish_children(root).len(), 2);
        assert_eq!(storage.get_nyes(root), Nyes::Prembrionic);
    }

    /// `FirCursor` reads match direct `FVMStorage` reads for the same
    /// pointer — proving the wrapper is a pure convenience, not a divergent
    /// second source of truth.
    #[test]
    fn fir_cursor_reads_match_direct_storage_reads() {
        let (mut storage, root) = FVMStorage::test_root_brane(&[FirSpec::IndepInt { value: 10 }]);
        let child = storage.foolish_children(root)[0];
        storage.with_mut(child, |fir| fir.set_nyes(Nyes::Constant));

        let cursor = FirCursor::new(child, &storage);
        assert_eq!(cursor.node(), storage.get(child));
        assert_eq!(cursor.get_nyes(), storage.get_nyes(child));
        assert_eq!(cursor.parent(), Some(root));
        assert_eq!(cursor.home_brane().map(|c| c.ptr), Some(root));
        // `child` has no `Statement` ancestor, so the climb goes all the
        // way to the structural root and stops there — NOT back to `child`
        // itself. `root` is where the climb terminates, since its own
        // parent is itself.
        assert_eq!(cursor.statement().ptr, root);
        assert!(cursor.settled_result().is_none()); // IndepInt never has a settled_result body
    }

    /// `FirCursorMut::push_ubc_child` keeps the two-part contract exactly:
    /// pushes to `ubc_children` AND enqueues as a task only when the child is
    /// not already constanic.
    #[test]
    fn fir_cursor_mut_push_ubc_child_enqueues_only_non_constanic_children() {
        let (mut storage, root) = FVMStorage::test_root_brane(&[]);
        let settled = root.create_child(&mut storage, FirSpec::IndepInt { value: 1 });
        storage.with_mut(settled, |fir| fir.set_nyes(Nyes::Constant));
        let unsettled = root.create_child(
            &mut storage,
            FirSpec::Operator {
                op: "+".to_string(),
            },
        );

        {
            let mut cursor = FirCursorMut::new(root, &mut storage);
            cursor.push_ubc_child(settled);
            cursor.push_ubc_child(unsettled);
        }

        let cursor = FirCursor::new(root, &storage);
        assert_eq!(cursor.ubc_children(), &[settled, unsettled]);
        // Only the unsettled child should have been enqueued as a task.
        assert_eq!(cursor.front_task(), Some(unsettled));
    }

    /// `FirCursorMut::push_search_result`'s SINGULAR-RESULT INVARIANT trips
    /// its `debug_assert!` on a second push.
    #[test]
    #[should_panic(expected = "singular-result")]
    fn push_search_result_rejects_a_second_result() {
        let (mut storage, root) = FVMStorage::test_root_brane(&[]);
        let a = root.create_child(&mut storage, FirSpec::IndepInt { value: 1 });
        let b = root.create_child(&mut storage, FirSpec::IndepInt { value: 2 });
        let mut cursor = FirCursorMut::new(root, &mut storage);
        cursor.push_search_result(a);
        cursor.push_search_result(b); // must panic: already has a result
    }

    /// `check_sff_marked_child` accepts a child whose descendant searches are
    /// all `ECONSTANIC` and panics on one that is not — mirrors
    /// `proto_brane.rs`'s `push_foolish_child_sff_marked_*` test trio.
    #[test]
    fn check_sff_marked_child_accepts_all_econstanic_descendants() {
        let (mut storage, root) = FVMStorage::test_root_brane(&[]);
        let search = root.create_child(
            &mut storage,
            FirSpec::Search {
                pattern: "x".to_string(),
                anchored: true,
                forward: false,
                is_value_search: false,
                contexted: false,
            },
        );
        storage.with_mut(search, |fir| fir.set_nyes(Nyes::Econstanic));

        let cursor = FirCursorMut::new(root, &mut storage);
        cursor.check_sff_marked_child(search); // must not panic
    }

    #[test]
    #[should_panic(expected = "INTERNAL CONSISTENCY error")]
    fn check_sff_marked_child_rejects_a_non_econstanic_descendant() {
        let (mut storage, root) = FVMStorage::test_root_brane(&[]);
        let search = root.create_child(
            &mut storage,
            FirSpec::Search {
                pattern: "x".to_string(),
                anchored: true,
                forward: false,
                is_value_search: false,
                contexted: false,
            },
        );
        // Left at Prembrionic (the default) — NOT Econstanic — exactly the
        // mis-constructed shape the guard exists to catch.
        let cursor = FirCursorMut::new(root, &mut storage);
        cursor.check_sff_marked_child(search);
    }

    /// `revive_constanic`'s share-not-clone behavior: a `Creation` always shares
    /// the SAME `FirPointer`, regardless of NYES — the FoolRef/Creation
    /// unconditional-share rule from `constanic_clone_at`.
    #[test]
    fn revive_constanic_shares_creation_unconditionally() {
        let (mut storage, root) = FVMStorage::test_root_brane(&[]);
        let creation = root.create_child(&mut storage, FirSpec::Creation);
        let other_root = storage.make_root(FirSpec::IndepInt { value: 0 });

        let cloned = storage.revive_constanic(creation, other_root, 0, false, false);
        assert_eq!(cloned, creation, "Creation must share, never clone");
    }

    /// `revive_constanic`'s share-not-clone behavior for a `Constant`
    /// non-`Brane` node: returns the SAME pointer, not a new slot.
    #[test]
    fn revive_constanic_shares_constant_non_brane() {
        let (mut storage, root) = FVMStorage::test_root_brane(&[]);
        let settled = root.create_child(&mut storage, FirSpec::IndepInt { value: 42 });
        storage.with_mut(settled, |fir| fir.set_nyes(Nyes::Constant));
        let other_root = storage.make_root(FirSpec::IndepInt { value: 0 });

        let cloned = storage.revive_constanic(settled, other_root, 0, false, false);
        assert_eq!(
            cloned, settled,
            "Constant non-Brane must share, never clone"
        );
    }

    /// `revive_constanic`'s full-rebuild behavior: a pre-constanic node is
    /// rebuilt as a genuinely new pointer under the new parent, with its
    /// foolish children recursively cloned too, and a `Statement`'s
    /// `line_number` renumbered to the passed `index` — exactly as
    /// `constanic_clone_at`'s `FirKind::Statement` arm does today
    /// (`let line = index;`).
    #[test]
    fn revive_constanic_rebuilds_pre_constanic_nodes_and_renumbers_statement_lines() {
        let (mut storage, root) = FVMStorage::test_root_brane(&[]);
        let stmt = root.create_child(
            &mut storage,
            FirSpec::Statement {
                identifier: Identifier::from_parts(vec![], "x"),
                line_number: 99, // original position — must be overwritten by `index` below
            },
        );
        let other_root = storage.make_root(FirSpec::IndepInt { value: 0 });

        let cloned = storage.revive_constanic(stmt, other_root, 3, false, false);
        assert_ne!(
            cloned, stmt,
            "a pre-constanic Statement must be rebuilt, not shared"
        );
        assert_eq!(cloned.get_parent(&storage), Some(other_root));
        match storage.get(cloned) {
            FirSpec::Statement { line_number, .. } => {
                assert_eq!(*line_number, 3, "line_number must be renumbered to `index`")
            }
            other => panic!("expected FirSpec::Statement, got {other:?}"),
        }
        // The original subtree's pointer must remain exactly as valid as before.
        assert_eq!(storage.get_nyes(stmt), Nyes::Prembrionic);
    }

    /// `revive_constanic` recursively clones foolish children, preserving count
    /// and (for pre-constanic children) producing fresh pointers for each.
    #[test]
    fn revive_constanic_recursively_clones_foolish_children() {
        let (mut storage, root) = FVMStorage::test_root_brane(&[
            FirSpec::Operator {
                op: "+".to_string(),
            },
            FirSpec::Operator {
                op: "+".to_string(),
            },
        ]);
        let other_root = storage.make_root(FirSpec::IndepInt { value: 0 });

        let cloned = storage.revive_constanic(root, other_root, 0, false, false);
        let cloned_children = storage.foolish_children(cloned);
        assert_eq!(cloned_children.len(), 2);
        let original_children = storage.foolish_children(root).to_vec();
        for (c, orig) in cloned_children.iter().zip(original_children.iter()) {
            assert_ne!(c, orig, "each pre-constanic child must be a fresh clone");
        }
    }

    /// `skip_foolish_children: true` omits re-cloning parse-time children —
    /// used at the top level of a clone when only the ubc/result side is
    /// being recoordinated (per `constanic_clone_at`'s own parameter).
    #[test]
    fn revive_constanic_skip_foolish_children_omits_them() {
        let (mut storage, root) = FVMStorage::test_root_brane(&[FirSpec::IndepInt { value: 1 }]);
        let other_root = storage.make_root(FirSpec::IndepInt { value: 0 });

        let cloned = storage.revive_constanic(root, other_root, 0, false, true);
        assert!(storage.foolish_children(cloned).is_empty());
    }

    /// `temporary_release!` drops a handle, runs a storage-needing operation,
    /// and re-acquires a fresh handle bound back to the same name — proven
    /// against the interleaved shape FOOP-16.md's own illustrative example
    /// describes: finish writing to one node, build a second node mid-
    /// sequence, then resume writing to the first.
    #[test]
    fn temporary_release_reacquires_a_usable_handle() {
        let (mut storage, root) = FVMStorage::test_root_brane(&[]);
        let first_ptr = root.create_child(&mut storage, FirSpec::IndepInt { value: 1 });

        let first = storage.get_mut(first_ptr);
        first.set_nyes(Nyes::Woconstanic);
        let (second_ptr, first) = temporary_release!(
            first,
            storage.get_mut(first_ptr),
            first_ptr.create_child(&mut storage, FirSpec::IndepInt { value: 2 })
        );
        first.set_nyes(Nyes::Constant);

        assert_eq!(storage.get_nyes(first_ptr), Nyes::Constant);
        assert_eq!(storage.get(second_ptr), &FirSpec::IndepInt { value: 2 });
    }

    /// `step_inner`'s pop-vs-recurse shape, exercised without ever reaching
    /// the `fir_op_step` dispatch `todo!()`: a front task that is already
    /// constanic gets popped, not recursed into.
    #[test]
    fn step_pops_a_front_task_that_is_already_constanic() {
        let (mut storage, root) = FVMStorage::test_root_brane(&[]);
        let done = root.create_child(&mut storage, FirSpec::IndepInt { value: 1 });
        storage.with_mut(done, |fir| fir.set_nyes(Nyes::Constant));
        storage.with_mut(root, |fir| fir.push_task(done));
        assert_eq!(
            FirCursor::new(root, &storage).front_task(),
            Some(done),
            "task queued before stepping"
        );

        root.step(&mut storage);

        assert_eq!(
            FirCursor::new(root, &storage).front_task(),
            None,
            "the already-constanic front task must be popped, not recursed into"
        );
    }

    /// A literal integer is fully determined the moment it's written, so it
    /// is born `Independent` and needs no stepping at all — no Prembrionic
    /// phase, no Braning phase (an IndepInt has no children/tasks either
    /// way).
    #[test]
    fn indep_int_starts_independent_and_needs_no_stepping() {
        let mut storage = FVMStorage::new();
        let node = storage.make_root(FirSpec::IndepInt { value: 42 });
        assert_eq!(storage.get_nyes(node), Nyes::Independent);

        node.step(&mut storage);

        assert_eq!(storage.get_nyes(node), Nyes::Independent);
        assert_eq!(FirCursor::new(node, &storage).as_i64(), Some(42));
    }

    /// Stepping an already-settled `IndepInt` repeatedly is a no-op.
    #[test]
    fn indep_int_stepping_already_settled_is_noop() {
        let mut storage = FVMStorage::new();
        let node = storage.make_root(FirSpec::IndepInt { value: 1 });
        node.step(&mut storage);
        assert_eq!(storage.get_nyes(node), Nyes::Independent);

        node.step(&mut storage);
        assert_eq!(storage.get_nyes(node), Nyes::Independent);
    }

    /// `NkFir`'s arena migration: mirrors the existing
    /// `fir_kinds.rs::tests::nk_prembrionic_to_nk_in_one_step` test exactly —
    /// Prembrionic → Nk in ONE step.
    #[test]
    fn nk_prembrionic_to_nk_in_one_step() {
        let mut storage = FVMStorage::new();
        let node = storage.make_root(FirSpec::Nk {
            reason: "unbound name".to_string(),
        });
        assert_eq!(storage.get_nyes(node), Nyes::Prembrionic);

        node.step(&mut storage);

        assert_eq!(storage.get_nyes(node), Nyes::Nk);
        assert_eq!(
            FirCursor::new(node, &storage).as_nk_reason(),
            Some("unbound name")
        );
    }

    /// `2 + 3` settles Constant with value `5`. Both operands start
    /// pre-settled (`Constant`), so `combine` fires without a genuine
    /// Braning-phase child-stepping round-trip — this test's own `step`
    /// loop drains the (already-constanic) operand tasks first, then
    /// settles via `combine`.
    #[test]
    fn operator_addition_settles_constant() {
        let mut storage = FVMStorage::new();
        let op = storage.make_root(FirSpec::Operator {
            op: "+".to_string(),
        });
        let a = op.create_child(&mut storage, FirSpec::IndepInt { value: 2 });
        let b = op.create_child(&mut storage, FirSpec::IndepInt { value: 3 });
        storage.with_mut(a, |fir| fir.set_nyes(Nyes::Constant));
        storage.with_mut(b, |fir| fir.set_nyes(Nyes::Constant));

        for _ in 0..10 {
            if storage.get_nyes(op).is_constanic() {
                break;
            }
            op.step(&mut storage);
        }

        assert_eq!(storage.get_nyes(op), Nyes::Constant);
        assert_eq!(FirCursor::new(op, &storage).as_i64(), Some(5));
        assert_eq!(FirCursor::new(op, &storage).as_op_name(), Some("+"));
    }

    /// Mirrors `fir_kinds.rs::tests::operator_div_by_zero_nyes_transitions`
    /// exactly — `1 / 0` settles NK.
    #[test]
    fn operator_division_by_zero_settles_nk() {
        let mut storage = FVMStorage::new();
        let op = storage.make_root(FirSpec::Operator {
            op: "/".to_string(),
        });
        let a = op.create_child(&mut storage, FirSpec::IndepInt { value: 1 });
        let b = op.create_child(&mut storage, FirSpec::IndepInt { value: 0 });
        storage.with_mut(a, |fir| fir.set_nyes(Nyes::Constant));
        storage.with_mut(b, |fir| fir.set_nyes(Nyes::Constant));

        for _ in 0..10 {
            if storage.get_nyes(op).is_constanic() {
                break;
            }
            op.step(&mut storage);
        }

        assert_eq!(storage.get_nyes(op), Nyes::Nk);
        assert_eq!(
            FirCursor::new(op, &storage)
                .settled_result()
                .and_then(|c| c.as_nk_reason().map(str::to_string)),
            Some("division by zero".to_string())
        );
    }

    /// An `Operator` with a pre-constanic (not-yet-settled) operand pushes
    /// tasks for its unsettled operands and moves to `Braning` — mirrors
    /// `impl Fir for OperatorFir`'s `Prembrionic`/`Embryonic` branch exactly
    /// (`if !self.operands_all_settled() { push tasks }`).
    #[test]
    fn operator_pushes_tasks_for_unsettled_operands() {
        let mut storage = FVMStorage::new();
        let op = storage.make_root(FirSpec::Operator {
            op: "+".to_string(),
        });
        let a = op.create_child(
            &mut storage,
            FirSpec::Operator {
                op: "+".to_string(),
            },
        );
        let _b = op.create_child(
            &mut storage,
            FirSpec::Operator {
                op: "+".to_string(),
            },
        );
        // `a`/`b` both start Prembrionic (unsettled) — the default.

        op.step(&mut storage);

        assert_eq!(storage.get_nyes(op), Nyes::Braning);
        assert_eq!(
            FirCursor::new(op, &storage).front_task(),
            Some(a),
            "unsettled operands must be queued as tasks"
        );
    }

    /// The core settle shape, with no null-characterized name in play so
    /// the NF-refusal checks stay out of scope: `a = 9` settles Independent
    /// (a statement mirrors its body's exact settled state).
    #[test]
    fn statement_settles_to_its_bodys_nyes() {
        use crate::identifier::Identifier;

        let mut storage = FVMStorage::new();
        let stmt = storage.make_root(FirSpec::Statement {
            identifier: Identifier::from_parts(vec![], "a"),
            line_number: 0,
        });
        let body = stmt.create_child(&mut storage, FirSpec::IndepInt { value: 9 });

        for _ in 0..10 {
            if storage.get_nyes(stmt).is_constanic() {
                break;
            }
            stmt.step(&mut storage);
        }

        assert_eq!(storage.get_nyes(body), Nyes::Independent);
        assert_eq!(storage.get_nyes(stmt), Nyes::Independent);
        assert_eq!(
            FirCursor::new(stmt, &storage)
                .as_stmt_identifier()
                .map(|id| id.identifier_name()),
            Some("a")
        );
        assert_eq!(
            FirCursor::new(stmt, &storage).as_stmt_line_number(),
            Some(0)
        );
    }

    /// A brane whose statements are all literal (`Independent`) values
    /// settles `Independent` itself — `decide_nyes_due_to_children` checks
    /// all-`Independent` before all-`Constant`.
    #[test]
    fn brane_of_all_independent_statements_settles_independent() {
        use crate::identifier::Identifier;

        let mut storage = FVMStorage::new();
        let brane = storage.make_root(FirSpec::Brane {
            characterizations: Characterizations::default(),
        });
        let stmt_a = brane.create_child(
            &mut storage,
            FirSpec::Statement {
                identifier: Identifier::from_parts(vec![], "a"),
                line_number: 0,
            },
        );
        stmt_a.create_child(&mut storage, FirSpec::IndepInt { value: 1 });
        let stmt_b = brane.create_child(
            &mut storage,
            FirSpec::Statement {
                identifier: Identifier::from_parts(vec![], "b"),
                line_number: 1,
            },
        );
        stmt_b.create_child(&mut storage, FirSpec::IndepInt { value: 2 });

        for _ in 0..30 {
            if storage.get_nyes(brane).is_constanic() {
                break;
            }
            brane.step(&mut storage);
        }

        assert_eq!(storage.get_nyes(brane), Nyes::Independent);
        assert_eq!(FirCursor::new(brane, &storage).stmt_count(), Some(2));
        assert!(FirCursor::new(brane, &storage).is_brane_like());
    }

    /// Mirrors `fir_kinds.rs::tests::brane_with_nk_child_nyes_transitions`
    /// exactly: a brane with one NK statement settles Nk overall.
    #[test]
    fn brane_with_nk_child_settles_nk() {
        use crate::identifier::Identifier;

        let mut storage = FVMStorage::new();
        let brane = storage.make_root(FirSpec::Brane {
            characterizations: Characterizations::default(),
        });
        let stmt_a = brane.create_child(
            &mut storage,
            FirSpec::Statement {
                identifier: Identifier::from_parts(vec![], "a"),
                line_number: 0,
            },
        );
        stmt_a.create_child(&mut storage, FirSpec::IndepInt { value: 1 });
        let stmt_bad = brane.create_child(
            &mut storage,
            FirSpec::Statement {
                identifier: Identifier::from_parts(vec![], "bad"),
                line_number: 1,
            },
        );
        stmt_bad.create_child(
            &mut storage,
            FirSpec::Nk {
                reason: "boom".to_string(),
            },
        );

        for _ in 0..30 {
            if storage.get_nyes(brane).is_constanic() {
                break;
            }
            brane.step(&mut storage);
        }

        assert_eq!(storage.get_nyes(brane), Nyes::Nk);
    }

    /// An empty `Brane` settles `Constant` in one step, via the
    /// `children.is_empty()` short-circuit.
    #[test]
    fn empty_brane_settles_constant_immediately() {
        let mut storage = FVMStorage::new();
        let brane = storage.make_root(FirSpec::Brane {
            characterizations: Characterizations::default(),
        });
        brane.step(&mut storage);
        assert_eq!(storage.get_nyes(brane), Nyes::Constant);
        assert_eq!(FirCursor::new(brane, &storage).stmt_count(), Some(0));
    }

    /// Construction and the pure data accessors round-trip correctly. Does
    /// NOT exercise search dispatch correctness — see the end-to-end
    /// dispatch tests below for that.
    #[test]
    fn search_fir_structural_construction_and_accessors_round_trip() {
        let (mut storage, root) = FVMStorage::test_root_brane(&[]);
        let search = root.create_child(
            &mut storage,
            FirSpec::Search {
                pattern: "^x$".to_string(),
                anchored: true,
                forward: false,
                is_value_search: false,
                contexted: true,
            },
        );

        let cursor = FirCursor::new(search, &storage);
        assert_eq!(cursor.as_search_pattern(), Some("^x$"));
        assert!(cursor.as_search_anchored());
        assert!(!cursor.as_search_is_value());
        assert!(cursor.as_search_contexted());
        assert_eq!(storage.get_nyes(search), Nyes::Prembrionic);
    }

    /// Construction and the pure data accessors round-trip correctly. Does
    /// NOT exercise `#N`/`^`/`$` resolution itself — see the IndexFir
    /// dispatch tests below for that. Index resolution resolves against the
    /// ANCHOR (`foolish_children()[0]`, for the anchored+contexted case) or
    /// the enclosing STATEMENT/BRANE found by walking the PARENT chain
    /// (`find_enclosing_stmt_and_brane`, for the unanchored case) — never
    /// against a sibling directly.
    #[test]
    fn index_fir_structural_construction_and_accessors_round_trip() {
        let (mut storage, root) = FVMStorage::test_root_brane(&[]);
        let index = root.create_child(
            &mut storage,
            FirSpec::Index {
                offset: -1,
                anchored: true,
                contexted: false,
            },
        );

        let cursor = FirCursor::new(index, &storage);
        assert_eq!(cursor.as_index_offset(), -1);
        assert!(cursor.as_index_anchored());
        assert!(!cursor.as_search_contexted());
        assert_eq!(storage.get_nyes(index), Nyes::Prembrionic);
    }

    /// `FoolRefFir`'s arena migration, and — correctness-critical, per this
    /// plan's own note — the FoolRefFir TWO-CHILD INVARIANT: a resolved
    /// search result has exactly two `ubc_children`, `[0]` the result value,
    /// `[1]` a `FoolRefFir` wrapping the ORIGINAL found statement. Confirms
    /// the two children are distinguishable by position exactly as
    /// `ubc_children[0]`/`[1]` are today, and that `FoolRefFir` reports its
    /// referent and is born `Constant`.
    #[test]
    fn push_search_result_pair_preserves_the_two_child_invariant() {
        let (mut storage, root) = FVMStorage::test_root_brane(&[]);
        let result = root.create_child(&mut storage, FirSpec::IndepInt { value: 42 });
        let referent = root.create_child(
            &mut storage,
            FirSpec::Statement {
                identifier: crate::identifier::Identifier::from_parts(vec![], "x"),
                line_number: 0,
            },
        );

        let mut cursor = FirCursorMut::new(root, &mut storage);
        cursor.push_search_result_pair(result, referent);

        let read = FirCursor::new(root, &storage);
        let children = read.ubc_children();
        assert_eq!(
            children.len(),
            2,
            "exactly two ubc_children, per the invariant"
        );
        assert_eq!(children[0], result, "[0] is the result value");
        assert_eq!(
            FirCursor::new(children[1], &storage).node(),
            &FirSpec::FoolRef { referent }
        );
        assert_eq!(
            storage.get_nyes(children[1]),
            Nyes::Constant,
            "FoolRef is born Constant"
        );
        assert_eq!(
            FirCursor::new(children[1], &storage).as_fool_ref_referent(),
            Some(referent),
            "the FoolRef's referent is the ORIGINAL found statement, genuinely shared"
        );
        // `settled_result` (used by `.value()`) reads [0] only — [1] stays
        // invisible. Its contract applies the constanic gate itself, so
        // `root` must be constanic first (a real search FIR would already
        // be constanic by the time it pushes a result; this test sets it
        // directly rather than stepping a real search).
        storage.with_mut(root, |fir| fir.set_nyes(Nyes::Constant));
        assert_eq!(
            FirCursor::new(root, &storage)
                .settled_result()
                .map(|c| c.ptr),
            Some(result)
        );
    }

    /// `StayFoolishFir`'s arena migration: mirrors
    /// `fir_kinds.rs::tests::stay_foolish_nyes_transitions` exactly — SF
    /// wrapping a constant int settles Constant, unwrapping to the inner
    /// value.
    #[test]
    fn stay_foolish_settles_to_inner_expr_value() {
        let mut storage = FVMStorage::new();
        let sf = storage.make_root(FirSpec::StayFoolish);
        let expr = sf.create_child(&mut storage, FirSpec::IndepInt { value: 42 });
        storage.with_mut(expr, |fir| fir.set_nyes(Nyes::Constant));

        for _ in 0..10 {
            if storage.get_nyes(sf).is_constanic() {
                break;
            }
            sf.step(&mut storage);
        }

        assert_eq!(storage.get_nyes(sf), Nyes::Constant);
        assert_eq!(
            FirCursor::new(sf, &storage).ubc_children().first(),
            Some(&expr),
            "SF unwraps to the inner expr itself, since it has no settled result of its own"
        );
    }

    /// Mirrors `fir_kinds.rs::tests::stay_fully_foolish_nyes_transitions`
    /// exactly — SFF wrapping a constant int settles Constant.
    #[test]
    fn stay_fully_foolish_settles_to_inner_expr_value() {
        let mut storage = FVMStorage::new();
        let sff = storage.make_root(FirSpec::StayFullyFoolish);
        let expr = sff.create_child(&mut storage, FirSpec::IndepInt { value: 42 });
        storage.with_mut(expr, |fir| fir.set_nyes(Nyes::Constant));

        for _ in 0..10 {
            if storage.get_nyes(sff).is_constanic() {
                break;
            }
            sff.step(&mut storage);
        }

        assert_eq!(storage.get_nyes(sff), Nyes::Constant);
        assert_eq!(
            FirCursor::new(sff, &storage).ubc_children().first(),
            Some(&expr)
        );
    }

    /// `revive_constanic`'s SF/SFF unwrap: a `StayFoolish` with a settled
    /// result unwraps to that result (recursing through `revive_constanic`
    /// again on it), never producing a cloned SF wrapper node — mirrors
    /// `constanic_clone_at`'s own first branch exactly.
    #[test]
    fn revive_constanic_unwraps_stay_foolish_to_its_settled_result() {
        let (mut storage, root) = FVMStorage::test_root_brane(&[]);
        let sf = root.create_child(&mut storage, FirSpec::StayFoolish);
        let inner = sf.create_child(&mut storage, FirSpec::IndepInt { value: 7 });
        storage.with_mut(inner, |fir| fir.set_nyes(Nyes::Constant));
        // Simulate SF's own settle: push inner as its ubc_children[0].
        {
            let mut cursor = FirCursorMut::new(sf, &mut storage);
            cursor.push_ubc_child(inner);
        }
        let other_root = storage.make_root(FirSpec::IndepInt { value: 0 });

        let cloned = storage.revive_constanic(sf, other_root, 0, false, false);

        // `inner` is Constant non-Brane, so it's SHARED, not cloned — the
        // unwrap recurses into it and the share-rule then returns it as-is.
        assert_eq!(
            cloned, inner,
            "SF unwraps through to its settled result, which then shares"
        );
        assert!(
            storage.foolish_children(other_root).is_empty()
                || storage.foolish_children(other_root) != [sf],
            "no cloned SF wrapper node should ever be produced"
        );
    }

    /// `revive_constanic`'s SF/SFF unwrap falls through to the first foolish
    /// child when there is no settled result yet (or for `StayFullyFoolish`,
    /// which never tries `ubc_children` first at all).
    #[test]
    fn revive_constanic_unwraps_stay_fully_foolish_to_first_foolish_child() {
        let (mut storage, root) = FVMStorage::test_root_brane(&[]);
        let sff = root.create_child(&mut storage, FirSpec::StayFullyFoolish);
        let inner = sff.create_child(
            &mut storage,
            FirSpec::Operator {
                op: "+".to_string(),
            },
        );
        // inner stays Prembrionic — a full-rebuild case, not a share.
        let other_root = storage.make_root(FirSpec::IndepInt { value: 0 });

        let cloned = storage.revive_constanic(sff, other_root, 0, false, false);

        assert_ne!(
            cloned, sff,
            "no cloned SFF wrapper node should ever be produced"
        );
        assert_ne!(
            cloned, inner,
            "a pre-constanic inner must be rebuilt, not shared"
        );
        assert_eq!(
            storage.get(cloned),
            &FirSpec::Operator {
                op: "+".to_string()
            }
        );
    }

    /// `ConcatHelper` steps identically to a `Brane` — it is transparent,
    /// inheriting brane-shaped stepping.
    #[test]
    fn concat_helper_settles_like_a_brane() {
        use crate::identifier::Identifier;

        let mut storage = FVMStorage::new();
        let helper = storage.make_root(FirSpec::ConcatHelper);
        let stmt = helper.create_child(
            &mut storage,
            FirSpec::Statement {
                identifier: Identifier::from_parts(vec![], "x"),
                line_number: 0,
            },
        );
        stmt.create_child(&mut storage, FirSpec::IndepInt { value: 42 });

        for _ in 0..10 {
            if storage.get_nyes(helper).is_constanic() {
                break;
            }
            helper.step(&mut storage);
        }

        assert_eq!(storage.get_nyes(helper), Nyes::Independent);
        assert_eq!(FirCursor::new(helper, &storage).stmt_count(), Some(1));
    }

    /// `ConcatenationFir`'s arena migration is TYPE-CHECK AND JOIN-READINESS
    /// ONLY (see this kind's `fir_op_step` arm doc comment — helper
    /// population/merging is deferred, same NF-mechanism dependency already
    /// deferred at `StatementFir`). This test proves the type-check path: a
    /// concatenation of two settled, brane-like elements is join-ready and
    /// settles `Woconstanic` (an HONEST incomplete-implementation result,
    /// NOT `Constant` — `concatenation_nyes_transitions` in `fir_kinds.rs`
    /// expects `Constant` from the REAL, fully-merging implementation; this
    /// arena test intentionally does NOT mirror that terminal state, since
    /// doing so would misrepresent what this task actually implemented).
    #[test]
    fn concatenation_of_settled_branes_is_join_ready() {
        let mut storage = FVMStorage::new();
        let cat = storage.make_root(FirSpec::Concatenation {
            provenance: ConcatProvenance::Juxtaposition,
        });
        let brane1 = cat.create_child(
            &mut storage,
            FirSpec::Brane {
                characterizations: Characterizations::default(),
            },
        );
        storage.with_mut(brane1, |fir| fir.set_nyes(Nyes::Constant));
        let brane2 = cat.create_child(
            &mut storage,
            FirSpec::Brane {
                characterizations: Characterizations::default(),
            },
        );
        storage.with_mut(brane2, |fir| fir.set_nyes(Nyes::Constant));

        core_fir_conversion::step_to_settled(&mut storage, cat).unwrap();

        // Both elements are EMPTY branes -- zero lines to merge, so the
        // real `populate_concat_helpers` pushes no helper at all, and the
        // "empty helper set -> Constant" convention applies (updated from
        // this test's earlier Woconstanic expectation, which pinned the
        // deliberately-incomplete pre-merge-logic placeholder — now that
        // populate_concat_helpers is real, join-ready empty branes settle
        // Constant, matching the real ConcatenationFir's own documented
        // "Empty (no lines joined) -> Constant" rule).
        assert_eq!(
            storage.get_nyes(cat),
            Nyes::Constant,
            "join-ready elements with zero total lines settle Constant (empty-brane convention)"
        );
        assert_eq!(
            FirCursor::new(cat, &storage).as_concat_provenance(),
            ConcatProvenance::Juxtaposition
        );
    }

    /// A concatenation with a genuinely non-brane, settled element (an
    /// `IndepInt`) settles `Nk` with the exact reason format the real
    /// `fir_op_step` produces — mirrors the type-error branch exactly.
    #[test]
    fn concatenation_with_a_non_brane_element_settles_nk() {
        let mut storage = FVMStorage::new();
        let cat = storage.make_root(FirSpec::Concatenation {
            provenance: ConcatProvenance::Juxtaposition,
        });
        let brane = cat.create_child(
            &mut storage,
            FirSpec::Brane {
                characterizations: Characterizations::default(),
            },
        );
        storage.with_mut(brane, |fir| fir.set_nyes(Nyes::Constant));
        let not_a_brane = cat.create_child(&mut storage, FirSpec::IndepInt { value: 1 });
        storage.with_mut(not_a_brane, |fir| fir.set_nyes(Nyes::Constant));

        for _ in 0..5 {
            if storage.get_nyes(cat).is_constanic() {
                break;
            }
            cat.step(&mut storage);
        }

        assert_eq!(storage.get_nyes(cat), Nyes::Nk);
        let reason = FirCursor::new(cat, &storage)
            .settled_result()
            .and_then(|c| c.as_nk_reason().map(str::to_string));
        assert_eq!(
            reason,
            Some("concatenation constituent indexes where it's not a brane: 1".to_string())
        );
    }

    /// `CreationFir`'s arena migration: born `Independent` and never steps —
    /// mirrors `fir_kinds.rs::tests::creation_nyes_transitions` exactly.
    #[test]
    fn creation_is_born_independent_and_never_steps() {
        let mut storage = FVMStorage::new();
        let root = storage.make_root(FirSpec::Brane {
            characterizations: Characterizations::default(),
        });
        let creation = root.create_child(&mut storage, FirSpec::Creation);
        assert_eq!(storage.get_nyes(creation), Nyes::Independent);

        creation.step(&mut storage);
        assert_eq!(storage.get_nyes(creation), Nyes::Independent);
    }

    /// `get_display_name`'s two-condition rule (FOOP-33), condition 1: a
    /// creation viewed from its OWN defining statement never reports a
    /// name, even though it is null-characterized and the whole RHS —
    /// mirrors `fir_kinds.rs::tests::
    /// creation_viewed_from_its_own_defining_statement_reports_no_name`
    /// exactly, using `Identifier::from_parts(vec![String::new()], "a")` to
    /// construct a null-characterized identifier directly (per that
    /// constructor's own doc comment: a single empty-string characterization
    /// component means null-characterization) rather than through the
    /// (not-yet-arena-migrated) parser/compiler.
    #[test]
    fn creation_viewed_from_its_own_defining_statement_reports_no_name() {
        use crate::identifier::Identifier;

        let (mut storage, root) = FVMStorage::test_root_brane(&[]);
        let stmt = root.create_child(
            &mut storage,
            FirSpec::Statement {
                identifier: Identifier::from_parts(vec![String::new()], "a"),
                line_number: 0,
            },
        );
        assert!(
            FirCursor::new(stmt, &storage)
                .as_stmt_identifier()
                .unwrap()
                .is_nully_characterizing_coordinate_name(),
            "sanity: the constructed identifier must actually be null-characterized"
        );
        let creation = stmt.create_child(&mut storage, FirSpec::Creation);

        let name = creation.get_display_name(&storage, stmt);
        assert_eq!(
            name, None,
            "a creation viewed from its OWN defining statement never reports a name"
        );
    }

    /// Condition 1's positive case: viewed from a DIFFERENT statement, a
    /// null-characterized creation DOES report its defining statement's
    /// name.
    #[test]
    fn creation_viewed_from_elsewhere_reports_its_name() {
        use crate::identifier::Identifier;

        let (mut storage, root) = FVMStorage::test_root_brane(&[]);
        let stmt = root.create_child(
            &mut storage,
            FirSpec::Statement {
                identifier: Identifier::from_parts(vec![String::new()], "a"),
                line_number: 0,
            },
        );
        let creation = stmt.create_child(&mut storage, FirSpec::Creation);
        let elsewhere = root.create_child(
            &mut storage,
            FirSpec::Statement {
                identifier: Identifier::from_parts(vec![], "b"),
                line_number: 1,
            },
        );

        let name = creation.get_display_name(&storage, elsewhere);
        // `get_display_name` reports `identifier.searchable_name()`
        // (`fully_characterized_name`), not the bare `identifier_name()` —
        // for a null-characterized name the searchable form is `"'a"`
        // (`Identifier::from_parts`'s doc comment: an empty-string
        // characterization component renders as a bare `'` prefix).
        assert_eq!(name, Some("'a".to_string()));
    }

    /// Condition 2: a creation defined under a PLAIN (non-null-characterized)
    /// name never reports a name, even when viewed from elsewhere.
    #[test]
    fn creation_under_a_plain_name_never_reports_a_name() {
        use crate::identifier::Identifier;

        let (mut storage, root) = FVMStorage::test_root_brane(&[]);
        let stmt = root.create_child(
            &mut storage,
            FirSpec::Statement {
                identifier: Identifier::from_parts(vec![], "a"), // plain, not null-characterized
                line_number: 0,
            },
        );
        let creation = stmt.create_child(&mut storage, FirSpec::Creation);
        let elsewhere = root.create_child(
            &mut storage,
            FirSpec::Statement {
                identifier: Identifier::from_parts(vec![], "b"),
                line_number: 1,
            },
        );

        assert_eq!(creation.get_display_name(&storage, elsewhere), None);
    }

    /// An operand whose own first foolish child is itself `Econstanic`
    /// (shaped like `<<#-1>>`, an SFF-wrapped index search inside
    /// `system.foo`) makes the whole comparison settle `Econstanic`.
    #[test]
    fn comparison_settles_econstanic_when_an_operand_is_unevaluated_here() {
        use crate::system_foo::ComparisonOp;

        let mut storage = FVMStorage::new();
        let cmp = storage.make_root(FirSpec::Comparison {
            op: ComparisonOp::Lt,
        });
        // Operand shaped like `<<#-1>>`: an SFF-wrapped index search whose
        // own inner search sits Econstanic (searched nothing in this
        // context yet).
        let operand = cmp.create_child(&mut storage, FirSpec::StayFullyFoolish);
        let inner_search = operand.create_child(
            &mut storage,
            FirSpec::Index {
                offset: -1,
                anchored: true,
                contexted: false,
            },
        );
        storage.with_mut(inner_search, |fir| fir.set_nyes(Nyes::Econstanic));
        storage.with_mut(operand, |fir| fir.set_nyes(Nyes::Constant));

        for _ in 0..5 {
            if storage.get_nyes(cmp).is_constanic() {
                break;
            }
            cmp.step(&mut storage);
        }

        assert_eq!(storage.get_nyes(cmp), Nyes::Econstanic);
        assert_eq!(FirCursor::new(cmp, &storage).as_op_name(), Some("'lt"));
    }

    /// When both operands ARE genuinely evaluated, the comparison resolves
    /// to whichever of `'True`/`'False` its ancestral search finds — which
    /// needs `'True`/`'False` reachable via ancestral search from the
    /// Comparison node's own position, so this test builds a minimal
    /// system.foo-shaped ancestor brane declaring them, with the Comparison
    /// node nested inside it (an isolated root with no ancestor to search
    /// would not exercise this path).
    #[test]
    fn comparison_with_evaluated_operands_resolves_the_real_verdict() {
        use crate::system_foo::ComparisonOp;

        let mut storage = FVMStorage::new();
        let root = storage.make_root(FirSpec::Brane {
            characterizations: Characterizations::default(),
        });
        let true_stmt = root.create_child(
            &mut storage,
            FirSpec::Statement {
                identifier: Identifier::from_parts(vec![String::new()], "True"),
                line_number: 0,
            },
        );
        let true_creation = true_stmt.create_child(&mut storage, FirSpec::Creation);
        let false_stmt = root.create_child(
            &mut storage,
            FirSpec::Statement {
                identifier: Identifier::from_parts(vec![String::new()], "False"),
                line_number: 1,
            },
        );
        false_stmt.create_child(&mut storage, FirSpec::Creation);

        // `'True`/`'False` must be in an ANCESTOR brane of `cmp`'s own home
        // brane, not siblings within the SAME brane `cmp` sits in —
        // `ab_search_by_pattern` searches ANCESTORS, never the current
        // brane's own siblings, and the ROOT brane itself is never its own
        // ancestor. Nest one level deeper: an inner brane holds the
        // statement whose body is the Comparison node.
        let inner_holder_stmt = root.create_child(
            &mut storage,
            FirSpec::Statement {
                identifier: Identifier::from_parts(vec![], "program"),
                line_number: 2,
            },
        );
        let inner = inner_holder_stmt.create_child(
            &mut storage,
            FirSpec::Brane {
                characterizations: Characterizations::default(),
            },
        );
        let holder = inner.create_child(
            &mut storage,
            FirSpec::Statement {
                identifier: Identifier::from_parts(vec![], "eq_check"),
                line_number: 0,
            },
        );
        let cmp = holder.create_child(
            &mut storage,
            FirSpec::Comparison {
                op: ComparisonOp::Eq,
            },
        );
        cmp.create_child(&mut storage, FirSpec::IndepInt { value: 1 });
        cmp.create_child(&mut storage, FirSpec::IndepInt { value: 1 });

        core_fir_conversion::step_to_settled(&mut storage, root).unwrap();

        assert_eq!(
            storage.get_nyes(cmp),
            Nyes::Constant,
            "1 =̲=̲ 1 must resolve the real verdict, not defer to Woconstanic"
        );
        let result = FirCursor::new(cmp, &storage)
            .ubc_children()
            .first()
            .copied();
        assert_eq!(
            result,
            Some(true_creation),
            "eq(1, 1) is true -- the comparison's result must be the SAME 'True creation \
             system.foo declares (referential identity, FOOP-33 SS5), not a synthetic boolean"
        );
    }

    // ── Search engine tests ──────────────────────────────────────────
    //
    // Mirror the spirit (not every single case) of `fir_kinds.rs`'s real
    // `ContextfulSearch engine tests` module: Navigator ordering contract,
    // predicate matching per variant, and the scan loop's Found/Miss/NkStop
    // outcomes. The authoritative correctness check for this phase is the
    // targeted einmo re-run (per FOOP-16.plan.md's own instruction that this
    // phase carries the highest silent-regression risk) — these unit tests
    // pin the internal engine state the black-box einmo comparison does not
    // directly exercise.

    use search_engine::{
        BraneNavigator, CandidateNavigator, MatchOutcome, ScanCtx, ScanOutcome, SearchPredicate,
        contextful_search_scan, contextful_search_scan_no_body_check,
    };

    fn make_named_statement(
        storage: &mut FVMStorage,
        brane: FirPointer,
        name: &str,
        line: usize,
        value: i64,
    ) -> FirPointer {
        use crate::identifier::Identifier;
        let stmt = brane.create_child(
            storage,
            FirSpec::Statement {
                identifier: Identifier::from_parts(vec![], name),
                line_number: line,
            },
        );
        let body = stmt.create_child(storage, FirSpec::IndepInt { value });
        storage.with_mut(body, |fir| fir.set_nyes(Nyes::Constant));
        storage.with_mut(stmt, |fir| fir.set_nyes(Nyes::Constant));
        stmt
    }

    /// `BraneNavigator` forward direction yields every candidate, in
    /// construction order, exactly once, then stops — mirrors
    /// `brane_nav_forward_yields_in_order_exactly_once` exactly.
    #[test]
    fn brane_navigator_forward_yields_in_order_exactly_once() {
        let (mut storage, brane) = FVMStorage::test_root_brane(&[]);
        let s0 = make_named_statement(&mut storage, brane, "a", 0, 1);
        let s1 = make_named_statement(&mut storage, brane, "b", 1, 2);
        let s2 = make_named_statement(&mut storage, brane, "c", 2, 3);

        let mut nav = BraneNavigator::new(&storage, brane, true);
        assert_eq!(nav.total(), 3);

        let yielded: Vec<(FirPointer, usize)> =
            std::iter::from_fn(|| nav.next_candidate()).collect();
        assert_eq!(yielded, vec![(s0, 0), (s1, 1), (s2, 2)]);
        assert!(
            nav.next_candidate().is_none(),
            "must stop after all yielded"
        );
    }

    /// Backward direction yields in reverse order, exactly once — mirrors
    /// `brane_nav_backward_yields_reverse_order_exactly_once` exactly.
    #[test]
    fn brane_navigator_backward_yields_reverse_order_exactly_once() {
        let (mut storage, brane) = FVMStorage::test_root_brane(&[]);
        let s0 = make_named_statement(&mut storage, brane, "a", 0, 10);
        let s1 = make_named_statement(&mut storage, brane, "b", 1, 20);
        let s2 = make_named_statement(&mut storage, brane, "c", 2, 30);

        let mut nav = BraneNavigator::new(&storage, brane, false);
        let yielded: Vec<(FirPointer, usize)> =
            std::iter::from_fn(|| nav.next_candidate()).collect();
        assert_eq!(yielded, vec![(s2, 2), (s1, 1), (s0, 0)]);
        assert!(nav.next_candidate().is_none());
    }

    /// An empty brane's navigator yields nothing — mirrors
    /// `brane_nav_empty_brane_yields_nothing` exactly.
    #[test]
    fn brane_navigator_empty_brane_yields_nothing() {
        let (storage, brane) = FVMStorage::test_root_brane(&[]);
        let mut nav = BraneNavigator::new(&storage, brane, true);
        assert_eq!(nav.total(), 0);
        assert!(nav.next_candidate().is_none());
    }

    /// `SearchPredicate::Name` approves an exact match on a settled
    /// candidate.
    #[test]
    fn search_predicate_name_approves_exact_match() {
        let (mut storage, brane) = FVMStorage::test_root_brane(&[]);
        let stmt = make_named_statement(&mut storage, brane, "x", 0, 5);
        let ctx = ScanCtx {
            position: 0,
            total: 1,
        };
        let pred = SearchPredicate::Name {
            pattern: "x".to_string(),
        };
        assert_eq!(pred.matches(&storage, stmt, &ctx), MatchOutcome::Approve);
    }

    /// `SearchPredicate::Name` rejects a non-matching name.
    #[test]
    fn search_predicate_name_rejects_non_match() {
        let (mut storage, brane) = FVMStorage::test_root_brane(&[]);
        let stmt = make_named_statement(&mut storage, brane, "x", 0, 5);
        let ctx = ScanCtx {
            position: 0,
            total: 1,
        };
        let pred = SearchPredicate::Name {
            pattern: "y".to_string(),
        };
        assert_eq!(pred.matches(&storage, stmt, &ctx), MatchOutcome::Reject);
    }

    /// `SearchPredicate::Name` NkStops when the candidate's body is NK —
    /// `check_body_nyes`'s NK branch.
    #[test]
    fn search_predicate_name_nkstops_on_nk_body() {
        let (mut storage, brane) = FVMStorage::test_root_brane(&[]);
        use crate::identifier::Identifier;
        let stmt = brane.create_child(
            &mut storage,
            FirSpec::Statement {
                identifier: Identifier::from_parts(vec![], "bad"),
                line_number: 0,
            },
        );
        let body = stmt.create_child(
            &mut storage,
            FirSpec::Nk {
                reason: "boom".to_string(),
            },
        );
        storage.with_mut(body, |fir| fir.set_nyes(Nyes::Nk));
        storage.with_mut(stmt, |fir| fir.set_nyes(Nyes::Nk));

        let ctx = ScanCtx {
            position: 0,
            total: 1,
        };
        let pred = SearchPredicate::Name {
            pattern: "bad".to_string(),
        };
        assert_eq!(pred.matches(&storage, stmt, &ctx), MatchOutcome::NkStop);
    }

    /// `SearchPredicate::Value` approves when the candidate's body equals
    /// the pattern's value, via `default_equal`.
    #[test]
    fn search_predicate_value_approves_on_equal_body() {
        let (mut storage, brane) = FVMStorage::test_root_brane(&[]);
        let stmt = make_named_statement(&mut storage, brane, "x", 0, 5);
        let pattern = brane.create_child(&mut storage, FirSpec::IndepInt { value: 5 });
        storage.with_mut(pattern, |fir| fir.set_nyes(Nyes::Constant));

        let ctx = ScanCtx {
            position: 0,
            total: 1,
        };
        let pred = SearchPredicate::Value { pattern };
        assert_eq!(pred.matches(&storage, stmt, &ctx), MatchOutcome::Approve);
    }

    /// `SearchPredicate::NameValue` is atomic: both name and value must
    /// match on the SAME candidate in one scan.
    #[test]
    fn search_predicate_name_value_is_atomic_conjunction() {
        let (mut storage, brane) = FVMStorage::test_root_brane(&[]);
        let stmt = make_named_statement(&mut storage, brane, "x", 0, 5);
        let pattern = brane.create_child(&mut storage, FirSpec::IndepInt { value: 5 });
        storage.with_mut(pattern, |fir| fir.set_nyes(Nyes::Constant));

        let ctx = ScanCtx {
            position: 0,
            total: 1,
        };
        // Name matches, value matches -> Approve.
        let both_match = SearchPredicate::NameValue {
            name: "x".to_string(),
            value: pattern,
        };
        assert_eq!(
            both_match.matches(&storage, stmt, &ctx),
            MatchOutcome::Approve
        );

        // Name matches, value does NOT -> Reject (not NkStop: NotEqual, not Unknowable).
        let other_pattern = brane.create_child(&mut storage, FirSpec::IndepInt { value: 999 });
        storage.with_mut(other_pattern, |fir| fir.set_nyes(Nyes::Constant));
        let name_only = SearchPredicate::NameValue {
            name: "x".to_string(),
            value: other_pattern,
        };
        assert_eq!(
            name_only.matches(&storage, stmt, &ctx),
            MatchOutcome::Reject
        );
    }

    /// `SearchPredicate::Index` with a negative offset resolves relative to
    /// `ctx.total` — `#-1` addresses the last candidate.
    #[test]
    fn search_predicate_index_negative_offset_addresses_from_the_end() {
        let (mut storage, brane) = FVMStorage::test_root_brane(&[]);
        let _s0 = make_named_statement(&mut storage, brane, "a", 0, 1);
        let s1 = make_named_statement(&mut storage, brane, "b", 1, 2);

        let ctx = ScanCtx {
            position: 1,
            total: 2,
        };
        let pred = SearchPredicate::Index(-1);
        assert_eq!(pred.matches(&storage, s1, &ctx), MatchOutcome::Approve);
    }

    /// `SearchPredicate::Head`/`Tail` match only position 0 / the last
    /// position respectively.
    #[test]
    fn search_predicate_head_and_tail_match_the_right_position() {
        let (mut storage, brane) = FVMStorage::test_root_brane(&[]);
        let s0 = make_named_statement(&mut storage, brane, "a", 0, 1);
        let s1 = make_named_statement(&mut storage, brane, "b", 1, 2);

        let ctx0 = ScanCtx {
            position: 0,
            total: 2,
        };
        let ctx1 = ScanCtx {
            position: 1,
            total: 2,
        };
        assert_eq!(
            SearchPredicate::Head.matches(&storage, s0, &ctx0),
            MatchOutcome::Approve
        );
        assert_eq!(
            SearchPredicate::Head.matches(&storage, s1, &ctx1),
            MatchOutcome::Reject
        );
        assert_eq!(
            SearchPredicate::Tail.matches(&storage, s1, &ctx1),
            MatchOutcome::Approve
        );
        assert_eq!(
            SearchPredicate::Tail.matches(&storage, s0, &ctx0),
            MatchOutcome::Reject
        );
    }

    /// `matches_no_body_check` skips the body-NYES gate for
    /// positional/name predicates — approves even with a pre-constanic
    /// body, which `matches` would treat as an internal-consistency
    /// violation (`unreachable!`).
    #[test]
    fn matches_no_body_check_skips_the_body_nyes_gate() {
        use crate::identifier::Identifier;
        let (mut storage, brane) = FVMStorage::test_root_brane(&[]);
        let stmt = brane.create_child(
            &mut storage,
            FirSpec::Statement {
                identifier: Identifier::from_parts(vec![], "x"),
                line_number: 0,
            },
        );
        let _body = stmt.create_child(&mut storage, FirSpec::IndepInt { value: 1 });
        // body stays Prembrionic (pre-constanic) — matches() would hit the
        // check_body_nyes unreachable!(); matches_no_body_check must not.

        let ctx = ScanCtx {
            position: 0,
            total: 1,
        };
        let pred = SearchPredicate::Name {
            pattern: "x".to_string(),
        };
        assert_eq!(
            pred.matches_no_body_check(&storage, stmt, &ctx),
            MatchOutcome::Approve
        );
    }

    /// `contextful_search_scan` finds the first approving candidate and
    /// stops — the scan loop's `Found` outcome.
    #[test]
    fn contextful_search_scan_finds_first_match() {
        let (mut storage, brane) = FVMStorage::test_root_brane(&[]);
        let _s0 = make_named_statement(&mut storage, brane, "a", 0, 1);
        let s1 = make_named_statement(&mut storage, brane, "target", 1, 2);
        let _s2 = make_named_statement(&mut storage, brane, "target", 2, 3);

        let mut nav = BraneNavigator::new(&storage, brane, true);
        let pred = SearchPredicate::Name {
            pattern: "target".to_string(),
        };
        let outcome = contextful_search_scan(&storage, &mut nav, &pred);
        assert_eq!(
            outcome,
            ScanOutcome::Found(s1),
            "forward scan finds the FIRST matching candidate, not a later duplicate"
        );
    }

    /// `contextful_search_scan` exhausts with `Miss` when nothing matches.
    #[test]
    fn contextful_search_scan_misses_when_nothing_matches() {
        let (mut storage, brane) = FVMStorage::test_root_brane(&[]);
        let _s0 = make_named_statement(&mut storage, brane, "a", 0, 1);

        let mut nav = BraneNavigator::new(&storage, brane, true);
        let pred = SearchPredicate::Name {
            pattern: "nonexistent".to_string(),
        };
        assert_eq!(
            contextful_search_scan(&storage, &mut nav, &pred),
            ScanOutcome::Miss
        );
    }

    /// `contextful_search_scan` halts immediately with `NkStop` on an
    /// Unknowable candidate — never masks it by continuing to scan further
    /// candidates that might otherwise match.
    #[test]
    fn contextful_search_scan_halts_on_nkstop() {
        let (mut storage, brane) = FVMStorage::test_root_brane(&[]);
        use crate::identifier::Identifier;
        let bad = brane.create_child(
            &mut storage,
            FirSpec::Statement {
                identifier: Identifier::from_parts(vec![], "bad"),
                line_number: 0,
            },
        );
        let bad_body = bad.create_child(
            &mut storage,
            FirSpec::Nk {
                reason: "x".to_string(),
            },
        );
        storage.with_mut(bad_body, |fir| fir.set_nyes(Nyes::Nk));
        storage.with_mut(bad, |fir| fir.set_nyes(Nyes::Nk));
        let _after = make_named_statement(&mut storage, brane, "bad", 1, 1); // would match if scan continued

        let mut nav = BraneNavigator::new(&storage, brane, true);
        let pred = SearchPredicate::Name {
            pattern: "bad".to_string(),
        };
        assert_eq!(
            contextful_search_scan(&storage, &mut nav, &pred),
            ScanOutcome::NkStop
        );
    }

    /// `contextful_search_scan_no_body_check`'s own re-verification (per
    /// this phase's own task instruction): confirms the scan loop needed NO
    /// further logic change beyond what already flows through from
    /// `CandidateNavigator`'s and `SearchPredicate`'s migrations.
    #[test]
    fn contextful_search_scan_no_body_check_finds_pre_constanic_candidates() {
        use crate::identifier::Identifier;
        let (mut storage, brane) = FVMStorage::test_root_brane(&[]);
        let stmt = brane.create_child(
            &mut storage,
            FirSpec::Statement {
                identifier: Identifier::from_parts(vec![], "x"),
                line_number: 0,
            },
        );
        let _body = stmt.create_child(&mut storage, FirSpec::IndepInt { value: 1 });

        let mut nav = BraneNavigator::new(&storage, brane, true);
        let pred = SearchPredicate::Name {
            pattern: "x".to_string(),
        };
        assert_eq!(
            contextful_search_scan_no_body_check(&storage, &mut nav, &pred),
            ScanOutcome::Found(stmt)
        );
    }

    // ── SearchFir end-to-end dispatch tests ──────────────────────────
    //
    // These exercise the FULL fir_op_step dispatch through FirPointer::step
    // (not the lower-level search_engine primitives directly), proving
    // Scope threading (current_statement/current_brane) and the IB/AB/
    // contexted dispatch paths work together end-to-end — the shape real
    // Foolish source produces, even though this crate's compiler/evaluator
    // aren't migrated yet (Phases 3-4), so these trees are hand-built.

    /// An anchored search finding a statement via `contextful_search_scan`
    /// (the anchored-Braning path in `name_search_step`) — `anchor_brane?x`
    /// shape: an anchored search whose FIRST child (the anchor) IS the
    /// brane to scan directly (`.value()` on an already-`Constant` `Brane`
    /// is a no-op — `Brane` never populates its own `ubc_children`, so
    /// `settled_result`/`.value()` return the brane itself unchanged).
    #[test]
    fn search_fir_anchored_finds_statement_in_resolved_brane() {
        let (mut storage, root) = FVMStorage::test_root_brane(&[]);
        let search = root.create_child(
            &mut storage,
            FirSpec::Search {
                pattern: "x".to_string(),
                anchored: true,
                forward: false,
                is_value_search: false,
                contexted: false,
            },
        );
        // The anchor: search's own foolish_children[0].
        let anchor_brane = search.create_child(
            &mut storage,
            FirSpec::Brane {
                characterizations: Characterizations::default(),
            },
        );
        let _x = make_named_statement(&mut storage, anchor_brane, "x", 0, 42);

        for _ in 0..30 {
            if storage.get_nyes(search).is_constanic() {
                break;
            }
            search.step(&mut storage);
        }

        assert_eq!(
            storage.get_nyes(search),
            Nyes::Constant,
            "anchored search must resolve its anchor to a brane, scan it, and find 'x'"
        );
        assert_eq!(FirCursor::new(search, &storage).as_i64(), Some(42));
    }

    /// An anchored search that finds NOTHING settles `Nk` (anchored miss),
    /// not `Econstanic` — confirming the anchored-vs-unanchored miss
    /// distinction the OTHER direction from
    /// `search_fir_unanchored_miss_settles_econstanic` below.
    #[test]
    fn search_fir_anchored_miss_settles_nk() {
        let (mut storage, root) = FVMStorage::test_root_brane(&[]);
        let search = root.create_child(
            &mut storage,
            FirSpec::Search {
                pattern: "nonexistent".to_string(),
                anchored: true,
                forward: false,
                is_value_search: false,
                contexted: false,
            },
        );
        let anchor_brane = search.create_child(
            &mut storage,
            FirSpec::Brane {
                characterizations: Characterizations::default(),
            },
        );
        let _x = make_named_statement(&mut storage, anchor_brane, "x", 0, 1);

        for _ in 0..30 {
            if storage.get_nyes(search).is_constanic() {
                break;
            }
            search.step(&mut storage);
        }

        assert_eq!(storage.get_nyes(search), Nyes::Nk);
    }

    /// IB search: `{x=1; y=?x;}` shape — `y`'s unanchored search finds `x`
    /// earlier in the SAME brane via `name_search_step`'s Embryonic arm
    /// (`ib_search_with_engine`), reading `Scope::current_statement`
    /// (threaded by `step_inner`).
    #[test]
    fn search_fir_ib_search_finds_earlier_statement_in_same_brane() {
        use crate::identifier::Identifier;

        let (mut storage, root) = FVMStorage::test_root_brane(&[]);
        let x = root.create_child(
            &mut storage,
            FirSpec::Statement {
                identifier: Identifier::from_parts(vec![], "x"),
                line_number: 0,
            },
        );
        let x_body = x.create_child(&mut storage, FirSpec::IndepInt { value: 1 });
        storage.with_mut(x_body, |fir| fir.set_nyes(Nyes::Constant));
        storage.with_mut(x, |fir| fir.set_nyes(Nyes::Constant));

        let y = root.create_child(
            &mut storage,
            FirSpec::Statement {
                identifier: Identifier::from_parts(vec![], "y"),
                line_number: 1,
            },
        );
        let search = y.create_child(
            &mut storage,
            FirSpec::Search {
                pattern: "x".to_string(),
                anchored: false,
                forward: false,
                is_value_search: false,
                contexted: false,
            },
        );

        for _ in 0..30 {
            if storage.get_nyes(root).is_constanic() {
                break;
            }
            root.step(&mut storage);
        }

        assert!(
            storage.get_nyes(search).is_constanic(),
            "search must settle (got {:?})",
            storage.get_nyes(search)
        );
        assert_eq!(
            storage.get_nyes(search),
            Nyes::Constant,
            "IB search for 'x' from 'y' must find it and settle Constant"
        );
        assert_eq!(FirCursor::new(search, &storage).as_i64(), Some(1));
    }

    /// An unanchored search with NOTHING preceding it in its brane settles
    /// `Econstanic` (unanchored miss), not `Nk` — the anchored-vs-unanchored
    /// miss distinction (AGENTS.md §Searches "NK vs ECONSTANIC miss
    /// outcomes").
    #[test]
    fn search_fir_unanchored_miss_settles_econstanic() {
        use crate::identifier::Identifier;

        let (mut storage, root) = FVMStorage::test_root_brane(&[]);
        let y = root.create_child(
            &mut storage,
            FirSpec::Statement {
                identifier: Identifier::from_parts(vec![], "y"),
                line_number: 0,
            },
        );
        let search = y.create_child(
            &mut storage,
            FirSpec::Search {
                pattern: "nonexistent".to_string(),
                anchored: false,
                forward: false,
                is_value_search: false,
                contexted: false,
            },
        );

        for _ in 0..30 {
            if storage.get_nyes(root).is_constanic() {
                break;
            }
            root.step(&mut storage);
        }

        assert_eq!(storage.get_nyes(search), Nyes::Econstanic);
    }

    /// AB search: `{x=1; inner={y=?x;};}` shape — `y`'s unanchored search
    /// finds `x` in the ANCESTOR brane via `name_search_step`'s Braning arm
    /// (`ab_search_with_engine`), reading `Scope::current_brane`.
    #[test]
    fn search_fir_ab_search_finds_in_ancestor_brane() {
        use crate::identifier::Identifier;

        let (mut storage, root) = FVMStorage::test_root_brane(&[]);
        let x = root.create_child(
            &mut storage,
            FirSpec::Statement {
                identifier: Identifier::from_parts(vec![], "x"),
                line_number: 0,
            },
        );
        let x_body = x.create_child(&mut storage, FirSpec::IndepInt { value: 99 });
        storage.with_mut(x_body, |fir| fir.set_nyes(Nyes::Constant));
        storage.with_mut(x, |fir| fir.set_nyes(Nyes::Constant));

        let inner_stmt = root.create_child(
            &mut storage,
            FirSpec::Statement {
                identifier: Identifier::from_parts(vec![], "inner"),
                line_number: 1,
            },
        );
        let inner_brane = inner_stmt.create_child(
            &mut storage,
            FirSpec::Brane {
                characterizations: Characterizations::default(),
            },
        );
        let y = inner_brane.create_child(
            &mut storage,
            FirSpec::Statement {
                identifier: Identifier::from_parts(vec![], "y"),
                line_number: 0,
            },
        );
        let search = y.create_child(
            &mut storage,
            FirSpec::Search {
                pattern: "x".to_string(),
                anchored: false,
                forward: false,
                is_value_search: false,
                contexted: false,
            },
        );

        for _ in 0..50 {
            if storage.get_nyes(root).is_constanic() {
                break;
            }
            root.step(&mut storage);
        }

        assert_eq!(
            storage.get_nyes(search),
            Nyes::Constant,
            "AB search for 'x' from inner brane's 'y' must climb out and find it"
        );
        assert_eq!(FirCursor::new(search, &storage).as_i64(), Some(99));
    }

    // ── Stepping loop / core-FIR conversion tests ───────────────────

    use core_fir_conversion::{proto_to_core_fir, step_to_settled};
    use foolish_core::fir::FirQueryable;

    /// `step_to_settled`'s happy path: an `IndepInt` settles within budget.
    #[test]
    fn step_to_settled_settles_a_simple_fir() {
        let mut storage = FVMStorage::new();
        let ptr = storage.make_root(FirSpec::IndepInt { value: 7 });
        assert!(step_to_settled(&mut storage, ptr).is_ok());
        assert_eq!(storage.get_nyes(ptr), Nyes::Independent);
    }

    /// `proto_to_core_fir` on a settled `IndepInt` produces a
    /// `hs_constant_int` matching the value.
    #[test]
    fn proto_to_core_fir_renders_constant_int() {
        let mut storage = FVMStorage::new();
        let ptr = storage.make_root(FirSpec::IndepInt { value: 42 });
        step_to_settled(&mut storage, ptr).unwrap();

        let rendered = proto_to_core_fir(&storage, ptr);
        assert_eq!(rendered.hs_constant_int(), Some(42));
        assert_eq!(rendered.hs_state(), Nyes::Independent);
    }

    /// `proto_to_core_fir` on a settled `Nk` produces `hs_nk` with the
    /// reason string — mirrors the `FirKind::Nk` arm.
    #[test]
    fn proto_to_core_fir_renders_nk_with_reason() {
        let mut storage = FVMStorage::new();
        let ptr = storage.make_root(FirSpec::Nk {
            reason: "unbound name".to_string(),
        });
        step_to_settled(&mut storage, ptr).unwrap();

        let rendered = proto_to_core_fir(&storage, ptr);
        let (reason, alarm) = rendered.hs_nk().expect("should render as Nk");
        assert_eq!(reason, "unbound name");
        assert!(alarm.is_none(), "only 'division by zero' gets an alarm");
    }

    /// `proto_to_core_fir` on `Nk` with reason "division by zero" attaches
    /// the `DIV-BY-ZERO` alarm — mirrors that specific real-code branch.
    #[test]
    fn proto_to_core_fir_division_by_zero_gets_an_alarm() {
        let mut storage = FVMStorage::new();
        let ptr = storage.make_root(FirSpec::Nk {
            reason: "division by zero".to_string(),
        });
        step_to_settled(&mut storage, ptr).unwrap();

        let rendered = proto_to_core_fir(&storage, ptr);
        let (_, alarm) = rendered.hs_nk().unwrap();
        let alarm = alarm.expect("division by zero must carry an alarm");
        assert_eq!(alarm.code, "DIV-BY-ZERO");
    }

    /// `proto_to_core_fir` on a settled `Brane` of settled statements
    /// produces `hs_brane` with the right statement count and names —
    /// mirrors the `FirKind::Brane` arm, including the display-name
    /// suppression for `compiler::ANON_STMT_NAME`-equivalent anonymous
    /// names (not exercised here since these test statements are named).
    #[test]
    fn proto_to_core_fir_renders_brane_with_named_statements() {
        use crate::identifier::Identifier;

        let mut storage = FVMStorage::new();
        let brane = storage.make_root(FirSpec::Brane {
            characterizations: Characterizations::default(),
        });
        let stmt = brane.create_child(
            &mut storage,
            FirSpec::Statement {
                identifier: Identifier::from_parts(vec![], "x"),
                line_number: 0,
            },
        );
        stmt.create_child(&mut storage, FirSpec::IndepInt { value: 5 });

        for _ in 0..10 {
            if storage.get_nyes(brane).is_constanic() {
                break;
            }
            brane.step(&mut storage);
        }

        let rendered = proto_to_core_fir(&storage, brane);
        let (_characterizations, statements) = rendered.hs_brane().expect("should render as Brane");
        assert_eq!(statements.len(), 1);
    }

    /// `proto_to_core_fir` on a settled `Operator` unwraps to its computed
    /// result (an `IndepInt`), not the operator wrapper — mirrors the
    /// `FirKind::Operator` arm's `state == Nyes::Constant` unwrap branch.
    #[test]
    fn proto_to_core_fir_unwraps_settled_operator_to_its_result() {
        let mut storage = FVMStorage::new();
        let op = storage.make_root(FirSpec::Operator {
            op: "+".to_string(),
        });
        let a = op.create_child(&mut storage, FirSpec::IndepInt { value: 2 });
        let b = op.create_child(&mut storage, FirSpec::IndepInt { value: 3 });
        storage.with_mut(a, |fir| fir.set_nyes(Nyes::Constant));
        storage.with_mut(b, |fir| fir.set_nyes(Nyes::Constant));

        for _ in 0..10 {
            if storage.get_nyes(op).is_constanic() {
                break;
            }
            op.step(&mut storage);
        }

        let rendered = proto_to_core_fir(&storage, op);
        assert_eq!(
            rendered.hs_constant_int(),
            Some(5),
            "settled operator renders as its unwrapped result, not the wrapper"
        );
    }

    use core_fir_conversion::{step_until, step_until_line_number, step_until_statement_name};

    /// `step_until_statement_name` finds the SECOND statement in a two-line
    /// brane — mirrors `evaluator.rs::step_until_tests::
    /// step_until_statement_name_finds_second_statement`'s intent.
    #[test]
    fn step_until_statement_name_finds_second_statement() {
        use crate::identifier::Identifier;

        let (mut storage, root) = FVMStorage::test_root_brane(&[]);
        let a = root.create_child(
            &mut storage,
            FirSpec::Statement {
                identifier: Identifier::from_parts(vec![], "a"),
                line_number: 0,
            },
        );
        a.create_child(&mut storage, FirSpec::IndepInt { value: 1 });
        let b = root.create_child(
            &mut storage,
            FirSpec::Statement {
                identifier: Identifier::from_parts(vec![], "b"),
                line_number: 1,
            },
        );
        b.create_child(&mut storage, FirSpec::IndepInt { value: 2 });

        let steps = step_until_statement_name(&mut storage, root, "b").unwrap();
        eprintln!("stopped after {steps} steps");
        let front = FirCursor::new(root, &storage).front_task();
        assert!(front.is_some());
        assert_eq!(
            FirCursor::new(front.unwrap(), &storage)
                .as_stmt_identifier()
                .map(|id| id.searchable_name()),
            Some("b")
        );
    }

    /// `step_until_line_number` stops when the front task reaches the given
    /// line — mirrors `step_until_line_number_finds_line`'s intent.
    #[test]
    fn step_until_line_number_finds_line() {
        use crate::identifier::Identifier;

        let (mut storage, root) = FVMStorage::test_root_brane(&[]);
        for (name, line, value) in [("a", 0, 1), ("b", 1, 2), ("c", 2, 3)] {
            let stmt = root.create_child(
                &mut storage,
                FirSpec::Statement {
                    identifier: Identifier::from_parts(vec![], name),
                    line_number: line,
                },
            );
            stmt.create_child(&mut storage, FirSpec::IndepInt { value });
        }

        let steps = step_until_line_number(&mut storage, root, 2).unwrap();
        eprintln!("stopped after {steps} steps");
        let front = FirCursor::new(root, &storage).front_task().unwrap();
        assert_eq!(
            FirCursor::new(front, &storage).as_stmt_line_number(),
            Some(2)
        );
    }

    /// The generic `step_until` matcher — stops when the front task's own
    /// `Nyes` is constanic — mirrors `step_until_generic_matcher_by_nyes`'s
    /// intent.
    #[test]
    fn step_until_generic_matcher_by_nyes() {
        let (mut storage, root) = FVMStorage::test_root_brane(&[]);
        let a = root.create_child(&mut storage, FirSpec::IndepInt { value: 1 });
        let _b = root.create_child(&mut storage, FirSpec::IndepInt { value: 2 });

        let steps = step_until(&mut storage, root, |storage, front| {
            front.is_some_and(|f| storage.get_nyes(f).is_constanic())
        })
        .unwrap();
        eprintln!("stopped after {steps} steps");
        let front = FirCursor::new(root, &storage).front_task().unwrap();
        assert_eq!(front, a);
        assert!(storage.get_nyes(front).is_constanic());
    }

    // ── arena_compiler tests ─────────────────────────────────────────

    use arena_compiler::compile;

    /// Compiles `{a = 1; b = 2;}` through the arena compiler and confirms
    /// the resulting tree shape: a self-rooted `Brane` with two `Statement`
    /// children, each with an `IndepInt` body — mirrors
    /// `compiler.rs::tests`' overall intent (that module hand-builds each
    /// piece rather than compiling a full source string, so this test's
    /// end-to-end shape is new coverage, not a direct mirror of one test).
    #[test]
    fn arena_compiler_compiles_a_simple_brane() {
        let mut storage = FVMStorage::new();
        let roots = compile(&mut storage, "{a = 1; b = 2;}").unwrap();
        assert_eq!(roots.len(), 1);
        let root = roots[0];
        assert!(root.is_root(&storage));
        assert!(matches!(storage.get(root), FirSpec::Brane { .. }));

        let stmts = storage.foolish_children(root);
        assert_eq!(stmts.len(), 2);
        let a = stmts[0];
        assert_eq!(
            FirCursor::new(a, &storage)
                .as_stmt_identifier()
                .map(|id| id.identifier_name()),
            Some("a")
        );
        assert_eq!(FirCursor::new(a, &storage).as_stmt_line_number(), Some(0));
        let a_body = storage.foolish_children(a)[0];
        assert_eq!(storage.get(a_body), &FirSpec::IndepInt { value: 1 });

        let b = stmts[1];
        assert_eq!(
            FirCursor::new(b, &storage)
                .as_stmt_identifier()
                .map(|id| id.identifier_name()),
            Some("b")
        );
        assert_eq!(FirCursor::new(b, &storage).as_stmt_line_number(), Some(1));
    }

    /// A bare (unnamed) expression statement gets the anonymous name —
    /// mirrors `compiler.rs::tests::build_as_statement_keeps_assignment_name_and_anonymous_fallback`'s
    /// anonymous-fallback half.
    #[test]
    fn arena_compiler_anonymous_statement_gets_the_anon_name() {
        let mut storage = FVMStorage::new();
        let roots = compile(&mut storage, "{1;}").unwrap();
        let root = roots[0];
        let stmt = storage.foolish_children(root)[0];
        assert_eq!(
            FirCursor::new(stmt, &storage)
                .as_stmt_identifier()
                .map(|id| id.identifier_name()),
            Some(ANON_STMT_NAME)
        );
    }

    /// `compile_standalone` (via `compile`) rejects a non-`Brane` top-level
    /// root — mirrors `compile_standalone_rejects_non_brane_root` exactly.
    #[test]
    fn arena_compiler_rejects_non_brane_root() {
        // The parser itself only ever produces a top-level Brane per source
        // string, so to exercise the non-Brane-root rejection path directly
        // (matching the real test's use of `Astn::IntLit(1)` fed straight to
        // `compile_standalone`), call `compile_standalone` directly with a
        // hand-built non-Brane Astn rather than through `compile`'s
        // parse-then-compile pipeline.
        let mut storage = FVMStorage::new();
        let err = arena_compiler::compile_standalone(&mut storage, foolish_parser::Astn::IntLit(1))
            .expect_err("non-Brane root must be rejected");
        assert_eq!(
            err.to_string(),
            "only a Brane can be a top-level (root) node"
        );
    }

    /// `1 + 2` compiles to an `Operator` node with two `IndepInt` operands —
    /// exercises `build_fir`'s `BinaryOp` arm directly.
    #[test]
    fn arena_compiler_binary_op_has_two_operands() {
        let mut storage = FVMStorage::new();
        let roots = compile(&mut storage, "{x = 1 + 2;}").unwrap();
        let root = roots[0];
        let stmt = storage.foolish_children(root)[0];
        let op = storage.foolish_children(stmt)[0];
        assert!(matches!(storage.get(op), FirSpec::Operator { .. }));
        let operands = storage.foolish_children(op);
        assert_eq!(operands.len(), 2);
        assert_eq!(storage.get(operands[0]), &FirSpec::IndepInt { value: 1 });
        assert_eq!(storage.get(operands[1]), &FirSpec::IndepInt { value: 2 });
    }

    /// A name reference (`?x`-shaped bare identifier) compiles to an
    /// anchored-false `Search` — exercises `build_fir`'s `Identifier` arm
    /// and its characterization-folding (Gotcha #3).
    #[test]
    fn arena_compiler_identifier_compiles_to_an_unanchored_search() {
        let mut storage = FVMStorage::new();
        let roots = compile(&mut storage, "{x = 1; y = x;}").unwrap();
        let root = roots[0];
        let y = storage.foolish_children(root)[1];
        let search = storage.foolish_children(y)[0];
        match storage.get(search) {
            FirSpec::Search {
                pattern, anchored, ..
            } => {
                assert_eq!(pattern, "^x$");
                assert!(!anchored);
            }
            other => panic!("expected FirSpec::Search, got {other:?}"),
        }
    }

    /// A dot-search (`a.x`) compiles to an anchored `Search` whose first
    /// child is the anchor — exercises `build_fir`'s `DotSearch` arm.
    #[test]
    fn arena_compiler_dot_search_is_anchored_with_an_anchor_child() {
        let mut storage = FVMStorage::new();
        let roots = compile(&mut storage, "{a = {x=1;}; y = a.x;}").unwrap();
        let root = roots[0];
        let y = storage.foolish_children(root)[1];
        let search = storage.foolish_children(y)[0];
        match storage.get(search) {
            FirSpec::Search {
                pattern, anchored, ..
            } => {
                assert_eq!(pattern, "^x$");
                assert!(anchored);
            }
            other => panic!("expected FirSpec::Search, got {other:?}"),
        }
        assert_eq!(
            storage.foolish_children(search).len(),
            1,
            "anchored search has one anchor child"
        );
    }

    /// `<<x>>` (StayFullyFoolish) builds its descendant search ECONSTANIC —
    /// exercises `build_fir`'s `StayFullyFoolish` arm and the `under_sff`
    /// rule together, proving this crate's own compiler produces bodies
    /// satisfying the SFF invariant.
    #[test]
    fn arena_compiler_sff_marks_descendant_searches_econstanic() {
        let mut storage = FVMStorage::new();
        let roots = compile(&mut storage, "{a = <<x>>;}").unwrap();
        let root = roots[0];
        let stmt = storage.foolish_children(root)[0];
        let sff = storage.foolish_children(stmt)[0];
        assert!(matches!(storage.get(sff), FirSpec::StayFullyFoolish));
        let search = storage.foolish_children(sff)[0];
        assert!(matches!(storage.get(search), FirSpec::Search { .. }));
        assert_eq!(
            storage.get_nyes(search),
            Nyes::Econstanic,
            "a search built under an SFF marker must start ECONSTANIC, never Prembrionic"
        );
    }

    /// `'a = ⬤` compiles a `Creation` as a named creation's whole RHS —
    /// exercises `build_fir`'s `Creation` arm.
    #[test]
    fn arena_compiler_creation_literal() {
        let mut storage = FVMStorage::new();
        let roots = compile(&mut storage, "{'a = \u{2b24};}").unwrap();
        let root = roots[0];
        let stmt = storage.foolish_children(root)[0];
        let body = storage.foolish_children(stmt)[0];
        assert!(matches!(storage.get(body), FirSpec::Creation));
        assert_eq!(storage.get_nyes(body), Nyes::Independent);
    }

    /// `\o<name` (SF sugar via `=$`-equivalent) — a contexted search built
    /// via `Astn::ContextedSearch` — has its `contexted` flag set true post
    /// construction, exercising `build_fir`'s `ContextedSearch` arm and
    /// `ProtoBrane::set_contexted` together.
    #[test]
    fn arena_compiler_contexted_search_sets_the_contexted_flag() {
        let mut storage = FVMStorage::new();
        let roots = compile(&mut storage, "{a = {x=1;}; y = a~x &?x;}").unwrap();
        let root = roots[0];
        let y = storage.foolish_children(root)[1];
        // y's body is the OUTER search (&?x, contexted); its own anchor
        // chain leads down to the ~x search first, per this operator's
        // real parse shape — walk to find a Search with contexted == true
        // anywhere in y's body subtree.
        fn sift_for_contexted_search(storage: &FVMStorage, ptr: FirPointer) -> bool {
            if let FirSpec::Search {
                contexted: true, ..
            } = storage.get(ptr)
            {
                return true;
            }
            storage
                .foolish_children(ptr)
                .iter()
                .any(|&c| sift_for_contexted_search(storage, c))
        }
        assert!(
            sift_for_contexted_search(&storage, y),
            "a contexted search (&?x) must have contexted == true somewhere in the compiled tree"
        );
    }

    // ── IndexFir dispatch tests ──────────────────────────────────────

    /// Builds an `Index` node whose sole foolish child is a fresh `Brane` of
    /// three statements `a=10; b=20; c=30`, returning `(storage, idx,
    /// [a, b, c] statement pointers)`. The anchor brane is built AS the
    /// index node's own child from the start (the arena's tree is built
    /// strictly top-down — there is no "attach an existing pointer as a
    /// child" primitive), avoiding any re-parenting.
    fn index_with_anchor_brane(
        offset: i32,
        anchored: bool,
    ) -> (FVMStorage, FirPointer, [FirPointer; 3]) {
        let mut storage = FVMStorage::new();
        let idx = storage.make_root(FirSpec::Index {
            offset,
            anchored,
            contexted: false,
        });
        let anchor_brane = idx.create_child(
            &mut storage,
            FirSpec::Brane {
                characterizations: Characterizations::default(),
            },
        );
        let mut stmts = [anchor_brane; 3];
        for (i, (name, value)) in [("a", 10i64), ("b", 20), ("c", 30)].into_iter().enumerate() {
            let stmt = anchor_brane.create_child(
                &mut storage,
                FirSpec::Statement {
                    identifier: Identifier::from_parts(vec![], name),
                    line_number: i,
                },
            );
            stmt.create_child(&mut storage, FirSpec::IndepInt { value });
            stmts[i] = stmt;
        }
        (storage, idx, stmts)
    }

    /// An anchored `#1` index into a brane of three statements settles
    /// Constant with the middle statement's value. Exercises `IndexFir`'s
    /// `Prembrionic`/`Embryonic` push-anchor-task arm, then the `Braning`
    /// anchored-search arm (`BraneNavigator` + `SearchPredicate::Index`),
    /// then `settle_from_ubc_result`.
    #[test]
    fn index_fir_finds_element_at_offset_in_anchor_brane() {
        let (mut storage, idx, _stmts) = index_with_anchor_brane(1, true);
        core_fir_conversion::step_to_settled(&mut storage, idx).unwrap();
        assert_eq!(storage.get_nyes(idx), Nyes::Constant);
        let result = FirCursor::new(idx, &storage)
            .ubc_children()
            .first()
            .copied();
        assert!(
            result.is_some(),
            "settled Index must have a ubc_children result"
        );
        assert_eq!(
            FirCursor::new(result.unwrap().value(&storage), &storage).as_i64(),
            Some(20)
        );
    }

    /// An anchored index whose target falls outside the anchor brane's
    /// statement range settles Nk.
    #[test]
    fn index_fir_out_of_bounds_is_nk() {
        let (mut storage, idx, _stmts) = index_with_anchor_brane(5, true);
        core_fir_conversion::step_to_settled(&mut storage, idx).unwrap();
        assert_eq!(storage.get_nyes(idx), Nyes::Nk);
        assert!(FirCursor::new(idx, &storage).ubc_children().is_empty());
    }

    /// `#-1` anchored into a three-statement brane addresses the LAST
    /// statement.
    #[test]
    fn index_fir_negative_offset_from_back() {
        let (mut storage, idx, _stmts) = index_with_anchor_brane(-1, true);
        core_fir_conversion::step_to_settled(&mut storage, idx).unwrap();
        assert_eq!(storage.get_nyes(idx), Nyes::Constant);
        let result = FirCursor::new(idx, &storage)
            .ubc_children()
            .first()
            .copied();
        assert_eq!(
            result.map(|r| FirCursor::new(r.value(&storage), &storage).as_i64()),
            Some(Some(30)),
            "anchored #-1 must address the LAST statement (c=30)"
        );
    }

    /// Direct arena counterpart to an unanchored `#-1` (the real
    /// `Astn::UnanchoredSeek` shape, e.g. compiler-generated for a bare
    /// trailing reference): from a statement's own enclosing brane,
    /// addresses the statement immediately before it. Exercises the
    /// unanchored branch's `find_enclosing_stmt_and_brane` + `BraneNavigator`
    /// path, distinct from the anchored branch above.
    #[test]
    fn index_fir_unanchored_negative_offset_finds_preceding_statement() {
        let mut storage = FVMStorage::new();
        let brane = storage.make_root(FirSpec::Brane {
            characterizations: Characterizations::default(),
        });
        let a = brane.create_child(
            &mut storage,
            FirSpec::Statement {
                identifier: Identifier::from_parts(vec![], "a"),
                line_number: 0,
            },
        );
        a.create_child(&mut storage, FirSpec::IndepInt { value: 10 });
        let b = brane.create_child(
            &mut storage,
            FirSpec::Statement {
                identifier: Identifier::from_parts(vec![], "b"),
                line_number: 1,
            },
        );
        // b's body is an unanchored Index(-1): "the statement one before me".
        let idx = b.create_child(
            &mut storage,
            FirSpec::Index {
                offset: -1,
                anchored: false,
                contexted: false,
            },
        );

        core_fir_conversion::step_to_settled(&mut storage, brane).unwrap();
        assert_eq!(storage.get_nyes(idx), Nyes::Constant);
        let result = FirCursor::new(idx, &storage)
            .ubc_children()
            .first()
            .copied();
        assert_eq!(
            result.map(|r| FirCursor::new(r.value(&storage), &storage).as_i64()),
            Some(Some(10)),
            "unanchored #-1 from statement b must find statement a's value"
        );
    }

    /// An unanchored `IndexFir` whose enclosing statement is itself the
    /// FIRST statement of its brane (no preceding statement to find) must
    /// settle Nk, not panic or hang — the same index-0 boundary discipline
    /// `_ib_search`'s own regression test enforces for name search.
    #[test]
    fn index_fir_unanchored_negative_offset_at_index_zero_settles_nk() {
        let mut storage = FVMStorage::new();
        let brane = storage.make_root(FirSpec::Brane {
            characterizations: Characterizations::default(),
        });
        let a = brane.create_child(
            &mut storage,
            FirSpec::Statement {
                identifier: Identifier::from_parts(vec![], "a"),
                line_number: 0,
            },
        );
        let idx = a.create_child(
            &mut storage,
            FirSpec::Index {
                offset: -1,
                anchored: false,
                contexted: false,
            },
        );

        core_fir_conversion::step_to_settled(&mut storage, brane).unwrap();
        assert_eq!(
            storage.get_nyes(idx),
            Nyes::Nk,
            "no statement precedes index 0 -- must settle Nk, not hang or find itself"
        );
    }

    /// An anchored `IndexFir` whose anchor resolves to a non-brane,
    /// NAMEABLE value (an integer literal) settles Nk AND records a named
    /// reason (FOOP-75 §7) — both via a fresh ubc_children Nk AND via
    /// `alarm_reason`.
    #[test]
    fn index_fir_anchor_not_a_brane_names_the_value() {
        let mut storage = FVMStorage::new();
        let idx = storage.make_root(FirSpec::Index {
            offset: 0,
            anchored: true,
            contexted: false,
        });
        idx.create_child(&mut storage, FirSpec::IndepInt { value: 4 });

        core_fir_conversion::step_to_settled(&mut storage, idx).unwrap();
        assert_eq!(storage.get_nyes(idx), Nyes::Nk);
        let reason = storage.alarm_reason(idx).map(str::to_owned);
        assert_eq!(
            reason.as_deref(),
            Some("4 is not a brane"),
            "a nameable non-brane anchor (an int literal) must name itself in the alarm reason"
        );
        let ubc_nk = FirCursor::new(idx, &storage)
            .ubc_children()
            .first()
            .copied();
        assert!(
            ubc_nk.is_some_and(|nk| matches!(storage.get(nk), FirSpec::Nk { .. })
                && storage.get_nyes(nk) == Nyes::Nk),
            "the named-reason NK must also be pushed as a ubc_children result"
        );
    }

    /// An anchored `IndexFir` whose anchor resolves to a non-brane,
    /// UNNAMEABLE value (e.g. an already-NK search result) settles Nk but
    /// records NO named reason — the "leave the result unset" half of
    /// FOOP-75 §7's rule, distinct from the nameable-anchor test above.
    #[test]
    fn index_fir_anchor_not_a_brane_and_unnameable_records_no_reason() {
        let mut storage = FVMStorage::new();
        let idx = storage.make_root(FirSpec::Index {
            offset: 0,
            anchored: true,
            contexted: false,
        });
        // An anchor that resolves to Nk (unnameable: as_i64() is None for
        // an Nk node) rather than to a brane or a nameable integer.
        let anchor = idx.create_child(
            &mut storage,
            FirSpec::Nk {
                reason: "unbound".to_string(),
            },
        );
        storage.with_mut(anchor, |fir| fir.set_nyes(Nyes::Nk));

        core_fir_conversion::step_to_settled(&mut storage, idx).unwrap();
        assert_eq!(storage.get_nyes(idx), Nyes::Nk);
        assert_eq!(
            storage.alarm_reason(idx),
            None,
            "an unnameable non-brane anchor must NOT synthesize a reason"
        );
        assert!(
            FirCursor::new(idx, &storage).ubc_children().is_empty(),
            "no named-reason NK should be pushed when the anchor cannot be named"
        );
    }

    /// A contexted, anchored index (`&#1`-shaped) reads its anchor's
    /// `FoolRef` bookkeeping entry to find the REFERENT's home brane and
    /// position, then indexes relative to THAT position — not the position
    /// of the index node itself. Exercises the `contexted && anchored`
    /// branch, distinct from the plain-anchored branch above.
    #[test]
    fn index_fir_contexted_finds_statement_relative_to_anchors_referent() {
        let mut storage = FVMStorage::new();

        // The referent's home brane: {a=10; b=20; c=30;}, built as its own
        // root so it can be shared as the FoolRef referent below.
        let home_brane = storage.make_root(FirSpec::Brane {
            characterizations: Characterizations::default(),
        });
        let mut home_stmts = Vec::new();
        for (name, value, line) in [("a", 10i64, 0usize), ("b", 20, 1), ("c", 30, 2)] {
            let stmt = home_brane.create_child(
                &mut storage,
                FirSpec::Statement {
                    identifier: Identifier::from_parts(vec![], name),
                    line_number: line,
                },
            );
            stmt.create_child(&mut storage, FirSpec::IndepInt { value });
            home_stmts.push(stmt);
        }

        // idx: a contexted, anchored Index(offset=1) whose own foolish
        // child (the anchor) is a Search already manually settled as
        // having found home_stmts[0] ("a") via push_search_result_pair —
        // exactly the two-child invariant a real prior search leaves
        // behind, which the contexted branch reads via
        // `ubc_children().get(1)` (the FoolRef).
        let idx = storage.make_root(FirSpec::Index {
            offset: 1,
            anchored: true,
            contexted: true,
        });
        let anchor = idx.create_child(
            &mut storage,
            FirSpec::Search {
                pattern: "^a$".to_string(),
                anchored: true,
                forward: false,
                is_value_search: false,
                contexted: false,
            },
        );
        let a_value_clone = anchor.create_child(&mut storage, FirSpec::IndepInt { value: 10 });
        {
            let mut cursor = FirCursorMut::new(anchor, &mut storage);
            cursor.push_search_result_pair(a_value_clone, home_stmts[0]);
            cursor.set_nyes(Nyes::Constant);
        }

        core_fir_conversion::step_to_settled(&mut storage, idx).unwrap();
        assert_eq!(storage.get_nyes(idx), Nyes::Constant);
        let result = FirCursor::new(idx, &storage)
            .ubc_children()
            .first()
            .copied();
        assert_eq!(
            result.map(|r| FirCursor::new(r.value(&storage), &storage).as_i64()),
            Some(Some(20)),
            "contexted &#1 from an anchor pointing at 'a' must find 'b' (offset 1 from a's position)"
        );
    }

    /// A contexted index whose target falls outside the referent's home
    /// brane range settles Nk.
    #[test]
    fn index_fir_contexted_out_of_range_is_nk() {
        let mut storage = FVMStorage::new();
        let home_brane = storage.make_root(FirSpec::Brane {
            characterizations: Characterizations::default(),
        });
        let a = home_brane.create_child(
            &mut storage,
            FirSpec::Statement {
                identifier: Identifier::from_parts(vec![], "a"),
                line_number: 0,
            },
        );
        a.create_child(&mut storage, FirSpec::IndepInt { value: 10 });

        let idx = storage.make_root(FirSpec::Index {
            offset: 5,
            anchored: true,
            contexted: true,
        });
        let anchor = idx.create_child(
            &mut storage,
            FirSpec::Search {
                pattern: "^a$".to_string(),
                anchored: true,
                forward: false,
                is_value_search: false,
                contexted: false,
            },
        );
        let a_value_clone = anchor.create_child(&mut storage, FirSpec::IndepInt { value: 10 });
        {
            let mut cursor = FirCursorMut::new(anchor, &mut storage);
            cursor.push_search_result_pair(a_value_clone, a);
            cursor.set_nyes(Nyes::Constant);
        }

        core_fir_conversion::step_to_settled(&mut storage, idx).unwrap();
        assert_eq!(storage.get_nyes(idx), Nyes::Nk);
    }

    // ── StatementFir NF-refusal checks (FOOP-33 §4) ─────────────────

    /// A null-characterized statement redefining an existing same-name
    /// null-characterized constant with a DIFFERENT value is refused: its
    /// presented value (`nf_reason`, read via `settled_result` in the real
    /// code — here checked directly) becomes a fresh NK, not its written
    /// RHS.
    #[test]
    fn statement_null_const_conflict_is_refused() {
        let mut storage = FVMStorage::new();
        let brane = storage.make_root(FirSpec::Brane {
            characterizations: Characterizations::default(),
        });
        let first = brane.create_child(
            &mut storage,
            FirSpec::Statement {
                identifier: Identifier::from_parts(vec![String::new()], "True"),
                line_number: 0,
            },
        );
        first.create_child(&mut storage, FirSpec::IndepInt { value: 1 });
        let second = brane.create_child(
            &mut storage,
            FirSpec::Statement {
                identifier: Identifier::from_parts(vec![String::new()], "True"),
                line_number: 1,
            },
        );
        second.create_child(&mut storage, FirSpec::IndepInt { value: 2 });

        core_fir_conversion::step_to_settled(&mut storage, brane).unwrap();
        assert!(
            storage.nf_reason(second).is_some(),
            "redefining a null-characterized constant with a DIFFERENT value must be refused"
        );
        assert!(
            storage.nf_reason(first).is_none(),
            "the FIRST definition establishes the constant -- it is never itself refused"
        );
    }

    /// Re-stating a null-characterized constant's OWN existing value (the
    /// same value, not a conflicting one) is PERMITTED — not a rename, not
    /// a conflict.
    #[test]
    fn statement_null_const_same_value_restatement_is_permitted() {
        let mut storage = FVMStorage::new();
        let brane = storage.make_root(FirSpec::Brane {
            characterizations: Characterizations::default(),
        });
        let first = brane.create_child(
            &mut storage,
            FirSpec::Statement {
                identifier: Identifier::from_parts(vec![String::new()], "True"),
                line_number: 0,
            },
        );
        first.create_child(&mut storage, FirSpec::IndepInt { value: 1 });
        let second = brane.create_child(
            &mut storage,
            FirSpec::Statement {
                identifier: Identifier::from_parts(vec![String::new()], "True"),
                line_number: 1,
            },
        );
        second.create_child(&mut storage, FirSpec::IndepInt { value: 1 });

        core_fir_conversion::step_to_settled(&mut storage, brane).unwrap();
        assert!(
            storage.nf_reason(second).is_none(),
            "restating the SAME value must be permitted, not refused"
        );
    }

    // ── ConcatenationFir real merge (populate_concat_helpers) ───────

    /// Concatenating two non-empty branes actually JOINS their statements
    /// into one flat, constant `ConcatHelper` (not the old Woconstanic
    /// placeholder) — the real end-to-end behavior `populate_concat_
    /// helpers`'s translation exists to produce.
    #[test]
    fn concatenation_of_two_branes_joins_their_statements() {
        let mut storage = FVMStorage::new();
        let cat = storage.make_root(FirSpec::Concatenation {
            provenance: ConcatProvenance::Juxtaposition,
        });
        let brane1 = cat.create_child(
            &mut storage,
            FirSpec::Brane {
                characterizations: Characterizations::default(),
            },
        );
        let a = brane1.create_child(
            &mut storage,
            FirSpec::Statement {
                identifier: Identifier::from_parts(vec![], "a"),
                line_number: 0,
            },
        );
        a.create_child(&mut storage, FirSpec::IndepInt { value: 1 });

        let brane2 = cat.create_child(
            &mut storage,
            FirSpec::Brane {
                characterizations: Characterizations::default(),
            },
        );
        let b = brane2.create_child(
            &mut storage,
            FirSpec::Statement {
                identifier: Identifier::from_parts(vec![], "b"),
                line_number: 0,
            },
        );
        b.create_child(&mut storage, FirSpec::IndepInt { value: 2 });

        core_fir_conversion::step_to_settled(&mut storage, cat).unwrap();
        assert_eq!(storage.get_nyes(cat), Nyes::Independent);

        let helpers = FirCursor::new(cat, &storage).ubc_children().to_vec();
        assert_eq!(
            helpers.len(),
            1,
            "one flat ConcatHelper for both merged branes"
        );
        let helper = helpers[0];
        assert!(matches!(storage.get(helper), FirSpec::ConcatHelper));
        let joined_count = FirCursor::new(helper, &storage).stmt_count();
        assert_eq!(
            joined_count,
            Some(2),
            "both statements a and b must be joined into the helper"
        );

        let joined_a = FirCursor::new(helper, &storage).stmt_at(0).unwrap();
        let joined_b = FirCursor::new(helper, &storage).stmt_at(1).unwrap();
        assert_eq!(
            FirCursor::new(joined_a, &storage)
                .as_stmt_identifier()
                .map(|id| id.identifier_name()),
            Some("a")
        );
        assert_eq!(
            FirCursor::new(joined_b, &storage)
                .as_stmt_identifier()
                .map(|id| id.identifier_name()),
            Some("b")
        );
    }

    /// The null-const merge rule fires during a concatenation join: merging
    /// two branes that each null-characterize the SAME name with DIFFERENT
    /// values refuses the second occurrence, exactly as `StatementFir`'s
    /// own same-brane check does for an ordinary redefinition.
    #[test]
    fn concatenation_merge_applies_null_const_rule_to_conflicting_names() {
        let mut storage = FVMStorage::new();
        let cat = storage.make_root(FirSpec::Concatenation {
            provenance: ConcatProvenance::Juxtaposition,
        });
        let brane1 = cat.create_child(
            &mut storage,
            FirSpec::Brane {
                characterizations: Characterizations::default(),
            },
        );
        let first = brane1.create_child(
            &mut storage,
            FirSpec::Statement {
                identifier: Identifier::from_parts(vec![String::new()], "True"),
                line_number: 0,
            },
        );
        first.create_child(&mut storage, FirSpec::IndepInt { value: 1 });

        let brane2 = cat.create_child(
            &mut storage,
            FirSpec::Brane {
                characterizations: Characterizations::default(),
            },
        );
        let second = brane2.create_child(
            &mut storage,
            FirSpec::Statement {
                identifier: Identifier::from_parts(vec![String::new()], "True"),
                line_number: 0,
            },
        );
        second.create_child(&mut storage, FirSpec::IndepInt { value: 2 });

        core_fir_conversion::step_to_settled(&mut storage, cat).unwrap();

        let helper = FirCursor::new(cat, &storage)
            .ubc_children()
            .first()
            .copied()
            .unwrap();
        let joined_second = FirCursor::new(helper, &storage).stmt_at(1).unwrap();
        assert!(
            storage.nf_reason(joined_second).is_some(),
            "merging a conflicting null-characterized redefinition must be refused, \
             exactly as StatementFir's own same-brane check refuses one"
        );
    }

    // ── compose_program_with_system / evaluate tests ────────────────

    /// End-to-end: composing a trivial user program `{x = 1;}` with the real
    /// embedded `system.foo` source settles, and `program_result` correctly
    /// extracts the user's own root brane (not the composite wrapper).
    #[test]
    fn compose_program_with_system_settles_a_trivial_program() {
        let mut storage = FVMStorage::new();
        let roots = arena_compiler::compose_program_with_system(&mut storage, "{x = 1;}").unwrap();
        assert_eq!(roots.len(), 1);
        let composed_root = roots[0];

        core_fir_conversion::step_to_settled(&mut storage, composed_root).unwrap();

        let program = arena_compiler::program_result(&storage, composed_root)
            .expect("program_result must find the user's program member");
        assert!(
            FirCursor::new(program, &storage).is_brane_like(),
            "program_result must resolve to the user's own root brane"
        );
        let x_stmt = FirCursor::new(program, &storage).stmt_at(0).unwrap();
        assert_eq!(
            FirCursor::new(x_stmt, &storage)
                .as_stmt_identifier()
                .map(|id| id.identifier_name()),
            Some("x")
        );
    }

    /// End-to-end: a user program that USES a comparison operator (`'lt`)
    /// resolves through the full system.foo composition -- proves
    /// build_comparison/comparison_body/ComparisonFir's real verdict
    /// resolution all work together through the real embedded system.foo
    /// source, not just the hand-built trees this file's other Comparison
    /// tests use.
    #[test]
    fn compose_program_with_system_resolves_a_comparison() {
        let mut storage = FVMStorage::new();
        let roots =
            arena_compiler::compose_program_with_system(&mut storage, "{r = {1, 2, 'lt}$;}")
                .unwrap();
        let composed_root = roots[0];

        core_fir_conversion::step_to_settled(&mut storage, composed_root).unwrap();

        let program = arena_compiler::program_result(&storage, composed_root).unwrap();
        let r_stmt = FirCursor::new(program, &storage).stmt_at(0).unwrap();
        let r_body = storage.foolish_children(r_stmt).first().copied().unwrap();
        let r_value = r_body.value(&storage);
        assert!(
            storage.get_nyes(r_value).is_constanic(),
            "1 <̲ 2 must resolve through the real system.foo composition, got {:?}",
            storage.get_nyes(r_value)
        );
        assert!(
            matches!(storage.get(r_value), FirSpec::Creation),
            "the result of a resolved comparison read via $ must be the 'True creation itself"
        );
        // A Creation is born Independent (self-contained, no context
        // dependency) -- that's the SPECIFIC constanic state expected here,
        // not merely "some constanic state".
        assert_eq!(storage.get_nyes(r_value), Nyes::Independent);
    }

    /// Regression: a result-only node built via `ptr.create_child(storage,
    /// ..)` would be silently appended to `ptr`'s `foolish_children` (the
    /// ALWAYS-append contract every `create_child` call has) even though it
    /// should live ONLY in `ubc_children` — corrupting the very list
    /// `combine`'s own `any_nk` re-check (and every output-serialization
    /// operand loop) reads. `{a = 10 / 0 * 5;}`'s outer `*` operator must
    /// have EXACTLY its 2 parse-derived operands in `foolish_children` even
    /// after settling to Nk (its own division-by-zero-propagated result
    /// must live only in `ubc_children`, via `make_orphan_child`).
    #[test]
    fn combine_nk_result_does_not_pollute_foolish_children() {
        let mut storage = FVMStorage::new();
        let roots = arena_compiler::compile(&mut storage, "{a = 10 / 0 * 5;}").unwrap();
        let root = roots[0];
        let stmt = storage.foolish_children(root)[0];
        let outer = storage.foolish_children(stmt)[0];
        assert_eq!(storage.foolish_children(outer).len(), 2);

        core_fir_conversion::step_to_settled(&mut storage, root).unwrap();

        assert_eq!(storage.get_nyes(outer), Nyes::Nk);
        assert_eq!(
            storage.foolish_children(outer).len(),
            2,
            "settling must not append the result NK to foolish_children -- it belongs only \
             in ubc_children (make_orphan_child, not create_child)"
        );
        assert_eq!(
            FirCursor::new(outer, &storage).ubc_children().len(),
            1,
            "the result NK must be recorded in ubc_children"
        );
    }

    /// Minimal reproduction of `einmo_suite/input/foop/33/boolean/
    /// null_char_constant.foo`'s divergence: a user program that redefines
    /// `'True` (declared in `system.foo`) with a CONFLICTING value must
    /// refuse (NF), matching `program_redefining_true_to_a_conflicting_
    /// value_is_refused` in `system_foo.rs`'s OWN test suite (which passes
    /// today via the hand-built tree in that file, NOT through the real
    /// `compose_program_with_system` composition this test uses instead).
    #[test]
    fn compose_program_with_system_refuses_conflicting_true_redefinition() {
        let mut storage = FVMStorage::new();
        let roots =
            arena_compiler::compose_program_with_system(&mut storage, "{'True = 3;}").unwrap();
        let composed_root = roots[0];

        core_fir_conversion::step_to_settled(&mut storage, composed_root).unwrap();

        let program = arena_compiler::program_result(&storage, composed_root).unwrap();
        let true_stmt = FirCursor::new(program, &storage).stmt_at(0).unwrap();
        assert!(
            storage.nf_reason(true_stmt).is_some(),
            "redefining system.foo's 'True with a conflicting value (3) inside the composed \
             user program must be refused -- nf_reason is None, meaning the NF check never fired"
        );
    }

    /// Exact reproduction of `null_char_constant.foo`'s full statement
    /// sequence (restate, same-value re-assert, a reference, THEN the
    /// conflicting redefinition) -- the simpler 1-statement repro above
    /// passes; this one exercises the same multi-statement IB-search-finds-
    /// nearest-prior scan the real case does.
    #[test]
    fn compose_program_with_system_refuses_conflicting_true_redefinition_full_sequence() {
        let mut storage = FVMStorage::new();
        let roots = arena_compiler::compose_program_with_system(
            &mut storage,
            "{restate = 'True; 'True = 'True; conflict = 'True; 'True = 3;}",
        )
        .unwrap();
        let composed_root = roots[0];

        core_fir_conversion::step_to_settled(&mut storage, composed_root).unwrap();

        let program = arena_compiler::program_result(&storage, composed_root).unwrap();
        let second_true_stmt = FirCursor::new(program, &storage).stmt_at(3).unwrap();
        assert_eq!(
            FirCursor::new(second_true_stmt, &storage)
                .as_stmt_identifier()
                .map(|id| id.searchable_name()),
            Some("'True")
        );
        assert!(
            storage.nf_reason(second_true_stmt).is_some(),
            "the FOURTH statement ('True = 3, conflicting) must be refused"
        );
    }

    /// Regression for a real, load-bearing bug found while tracing
    /// `einmo_gate_checked`'s `foop/33/boolean/null_char_constant.foo`
    /// divergence: `check_null_const_conflict`/`check_rename_of_named_
    /// creation`/`apply_null_const_rule_to_merged_stmt` all correctly SET
    /// `nf_reason` on a refused statement, but nothing ever surfaced that
    /// refusal through `FirPointer::settled_result`'s READ path (used by
    /// `statement_value_for_comparison`, which output serialization and
    /// `default_equal` both depend on) -- `settled_result` is generic
    /// across all kinds and reads `ubc_children().first()`, but the NF
    /// write path never pushed anything there, so a refused statement's
    /// `settled_result()` answered `None` and every reader silently fell
    /// through to the raw (unrefused) written body. `'True = 3` rendered as
    /// plain `3` even though `nf_reason` was genuinely `Some("'True
    /// not-foolish")` on the very same pointer. Fixed by having the NF
    /// write path (`refuse_statement`, the new shared helper both call
    /// sites now use) also push a fresh, already-`Nk` node to
    /// `ubc_children` -- exactly what `settled_result`'s generic read
    /// already expects to find there.
    ///
    /// This test uses `UbcaEvaluator::evaluate` itself (not a hand-called
    /// `compose_program_with_system`), since the bug was invisible through
    /// direct `nf_reason` inspection (which is `Some` correctly) and only
    /// showed up in the RENDERED output -- exactly what `evaluate`
    /// produces and what einmo compares.
    #[test]
    fn evaluate_refuses_and_renders_conflicting_true_redefinition() {
        use foolish_core::Evaluator;
        let source = "{restate = 'True; 'True = 'True; conflict = 'True; 'True = 3;}";
        let evaluator = crate::evaluator::UbcaEvaluator;
        let results = evaluator.evaluate(source).unwrap();
        let rendered = format!("{:?}", results[0]);
        assert!(
            rendered.contains(r#"reason: "'True not-foolish""#),
            "the conflicting redefinition must render as an NF-reason NK, got: {rendered}"
        );
        assert!(
            rendered.contains("state: Nk"),
            "the whole composed brane must settle Nk once the refusal propagates, got: {rendered}"
        );
    }

    /// Regression: `handle_found` (called from
    /// `name_search_step`/`value_search_step`) hardcoded `sfm = false` at
    /// every call site, instead of threading `scope.has_ancestral_sfm`
    /// through. `transform_for_clone`'s contract is "SFM-descendant:
    /// preserve the source NYES verbatim (foolishly ignorant)" — with the
    /// bug, a search found from inside an SF (`<...>`) wrapper always
    /// cloned as though NOT SFM-descendant, so its own ECONSTANIC
    /// descendant searches (built ECONSTANIC by the `under_sff` rule when
    /// the ORIGINAL declaration was inside an SFF marker) transitioned to
    /// EMBRYONIC on clone and genuinely re-searched and resolved in the new
    /// context — instead of staying inertly ECONSTANIC, verbatim.
    ///
    /// Concretely: `{a = 1; b = 2; sff = <<a + b>>; sf = <sff>; a = 10;}`'s
    /// `sf` reference resolves `sff` via a name search inside an SF
    /// wrapper; the clone of `sff`'s `a + b` must stay `Woconstanic` with
    /// both operand searches `Econstanic` (settling `[Econstanic,
    /// Econstanic]` in ONE step and never progressing further). With the
    /// bug, the clone's operand searches would instead progress
    /// `Embryonic -> Braning -> Constant`, finding real values (`a=1`,
    /// `b=2`) and fully resolving to `3` — an over-eager resolution the
    /// SFM-verbatim-preservation rule exists to prevent.
    #[test]
    fn search_found_inside_sf_threads_ancestral_sfm_to_its_clone() {
        let mut storage = FVMStorage::new();
        let roots = arena_compiler::compile(
            &mut storage,
            "{a = 1; b = 2; sff = <<a + b>>; sf = <sff>; a = 10; sf; sff;}",
        )
        .unwrap();
        let root = roots[0];
        core_fir_conversion::step_to_settled(&mut storage, root).unwrap();

        let stmts = storage.foolish_children(root).to_vec();
        let sf_body = storage.foolish_children(stmts[3])[0]; // sf's SF wrapper
        let sf_search = storage.foolish_children(sf_body)[0]; // the search for "sff" inside it
        let clone = FirCursor::new(sf_search, &storage)
            .ubc_children()
            .first()
            .copied()
            .expect("sf's search must have settled with a result");

        assert_eq!(
            storage.get_nyes(clone),
            Nyes::Woconstanic,
            "sf's cloned Op+ must stay Woconstanic (SFM-verbatim), not fully resolve"
        );
        let operand_nyes: Vec<_> = storage
            .foolish_children(clone)
            .iter()
            .map(|&c| storage.get_nyes(c))
            .collect();
        assert_eq!(
            operand_nyes,
            vec![Nyes::Econstanic, Nyes::Econstanic],
            "the cloned Op+'s own operand searches must stay Econstanic verbatim, \
             not re-search and resolve in sf's new context"
        );
    }

    // ── Regression guards ────────────────────────────────────────────

    /// FOOP-13 regression guard: a statement that is the
    /// FIRST statement in its brane (`line_number == 0`) must not find
    /// itself via its own backward IB search. The real bug: computing the
    /// backward-scan end as `line_number.saturating_sub(1)` SATURATES to `0`
    /// at `line_number == 0` instead of representing "no preceding
    /// statements", so the scan range `[0, 0]` wrongly includes the
    /// statement's own slot — left unfixed, `{a = a + 1;}` recurses forever
    /// (`handle_found` clones the found statement's still-unresolved
    /// self-search, the clone re-searches, finds the SAME original
    /// statement again, without bound). `ib_search_by_pattern`'s own
    /// `checked_sub` (not `saturating_sub`) is the arena's fix for this
    /// exact bug class, already in place — this test pins it directly.
    #[test]
    fn ib_search_at_index_zero_does_not_find_self() {
        let mut storage = FVMStorage::new();
        let brane = storage.make_root(FirSpec::Brane {
            characterizations: Characterizations::default(),
        });
        let a_stmt = brane.create_child(
            &mut storage,
            FirSpec::Statement {
                identifier: Identifier::from_parts(vec![], "a"),
                line_number: 0,
            },
        );
        let op = a_stmt.create_child(
            &mut storage,
            FirSpec::Operator {
                op: "+".to_string(),
            },
        );
        op.create_child(
            &mut storage,
            FirSpec::Search {
                pattern: "^a$".to_string(),
                anchored: false,
                forward: false,
                is_value_search: false,
                contexted: false,
            },
        );
        op.create_child(&mut storage, FirSpec::IndepInt { value: 1 });

        let result = search_fir_dispatch::ib_search_by_pattern(&storage, "a", Some(a_stmt));
        assert!(
            result.is_none(),
            "BUG: a statement at index 0 of its brane must not find itself \
             via backward IB search — got a hit instead of None"
        );
    }

    /// FOOP-13 regression guard's end-to-end companion: the actual runtime path
    /// (`UbcaEvaluator::evaluate`, not a direct `_ib_search`/
    /// `ib_search_by_pattern` call) must not hang forever on a bare
    /// self-referential search at brane-index 0. Before the fix this
    /// program never settles (steps forever in BRANING); after the fix
    /// `a`'s self-search is correctly absent from its own brane, falls
    /// through, and the whole program settles within `evaluate`'s own step
    /// budget.
    #[test]
    fn evaluate_settles_self_referential_statement_at_index_zero_without_hanging() {
        use foolish_core::Evaluator;
        let evaluator = crate::evaluator::UbcaEvaluator;
        let result = evaluator.evaluate("{a = a + 1;}");
        assert!(
            result.is_ok(),
            "BUG: {{a = a + 1;}} must settle within evaluate's step budget \
             (a's self-search absent, falls through to unanchored-miss) — \
             it must NOT hang forever due to a's own statement finding \
             itself at index 0, got: {result:?}"
        );
    }

    /// Regression guard: `k=1; k=2` (no leading `'`) must NOT be refused —
    /// the null-const rule only fires on null-characterized coordinate
    /// names, never on plain ones.
    #[test]
    fn null_const_rule_does_not_fire_on_plain_names() {
        let mut storage = FVMStorage::new();
        let roots = arena_compiler::compile(&mut storage, "{k=1; k=2;}").unwrap();
        let root = roots[0];
        core_fir_conversion::step_to_settled(&mut storage, root).unwrap();
        let stmts = storage.foolish_children(root).to_vec();
        assert!(
            storage.nf_reason(stmts[0]).is_none(),
            "plain k=1 must never be refused by the null-const rule"
        );
        assert!(
            storage.nf_reason(stmts[1]).is_none(),
            "plain k=2 must never be refused by the null-const rule"
        );
    }

    /// Regression guard: an empty concatenation operand, or a
    /// single-operand concatenation, must merge without any spurious NF —
    /// the collision check must not misfire when there's nothing (or only
    /// one thing) to collide with.
    #[test]
    fn null_const_concatenation_empty_and_single_operand_merge_without_spurious_nf() {
        let mut storage = FVMStorage::new();
        let roots = arena_compiler::compile(&mut storage, "{A={}; B={'a=1;}; C = A B;}").unwrap();
        let root = roots[0];
        core_fir_conversion::step_to_settled(&mut storage, root).unwrap();
        let stmts = storage.foolish_children(root).to_vec();
        let c_body = storage.foolish_children(stmts[2])[0];
        let c_value = c_body.value(&storage);
        assert_eq!(FirCursor::new(c_value, &storage).stmt_count(), Some(1));
        let merged_a = FirCursor::new(c_value, &storage).stmt_at(0).unwrap();
        assert!(
            storage.nf_reason(merged_a).is_none(),
            "single 'a merged from a concatenation with an empty operand must not be NF"
        );
    }

    /// The whole comparison feature, end to end, for all five operators and
    /// both outcomes. `{a, b, 'op}$`: the brane literal's tail is `'op`,
    /// whose settled value is the boolean it computed from its two
    /// preceding neighbours (FOOP-33 §5.0). Each row is expressed as the
    /// plain Rust comparison of 1 and 2, so each row states WHY it is what
    /// it is, not merely what was observed.
    #[test]
    fn each_comparison_operator_produces_the_right_boolean() {
        for (op, expected) in [
            ("'lt", 1 < 2),
            ("'gt", 1 > 2),
            ("'le", 1 <= 2),
            ("'ge", 1 >= 2),
            ("'eq", 1 == 2),
        ] {
            let mut storage = FVMStorage::new();
            let source = format!("{{r = {{1, 2, {op}}}$;}}");
            let roots = arena_compiler::compose_program_with_system(&mut storage, &source).unwrap();
            let composed_root = roots[0];
            core_fir_conversion::step_to_settled(&mut storage, composed_root).unwrap();
            let program = arena_compiler::program_result(&storage, composed_root).unwrap();
            let stmt = FirCursor::new(program, &storage).stmt_at(0).unwrap();
            let body = storage.foolish_children(stmt)[0];
            let got = body.value(&storage);

            assert!(
                matches!(storage.get(got), FirSpec::Creation),
                "{op} must produce a creation ('True/'False), not {:?}",
                storage.get(got)
            );
            let want_name = if expected { "'True" } else { "'False" };
            let want_stmt = storage
                .foolish_children(composed_root)
                .iter()
                .find(|&&s| {
                    FirCursor::new(s, &storage)
                        .as_stmt_identifier()
                        .map(|id| id.searchable_name())
                        == Some(want_name)
                })
                .copied()
                .expect("system.foo declares 'True and 'False");
            let want_body = storage.foolish_children(want_stmt)[0];
            let want = want_body.value(&storage);
            assert_eq!(
                got, want,
                "{{1, 2, {op}}}$ must be system.foo's own {want_name} creation \
                 (referential identity, FOOP-33 §5), expected={expected}"
            );
        }
    }

    /// Ported from `evaluator.rs`'s `creation_display_name_conversion_tests::
    /// creation_reached_through_search_converts_with_its_own_defining_name`
    ///: `b='a` resolves THROUGH a search to the
    /// SAME creation `'a` defines (FOOP-33 Gotcha #2) — viewed from `b`'s
    /// statement (a DIFFERENT statement than `'a`'s own), the rendered
    /// output must report `'a`, not `b`, proving identity (not the
    /// referencing statement's own name) drives the name, and that viewing
    /// from elsewhere is what unlocks it. Uses the full-brane
    /// `proto_to_core_fir` rendering (rather than calling the private
    /// `proto_to_core_fir_inner` directly, as the real test does) since
    /// `current_stmt` threading is naturally exercised by rendering the
    /// whole root, matching how `evaluate` itself renders.
    #[test]
    fn creation_reached_through_search_renders_with_its_own_defining_name() {
        let mut storage = FVMStorage::new();
        let roots = arena_compiler::compile(&mut storage, "{'a=⬤; b='a;}").unwrap();
        let root = roots[0];
        core_fir_conversion::step_to_settled(&mut storage, root).unwrap();
        let rendered_root = core_fir_conversion::proto_to_core_fir(&storage, root);
        let rendered = format!("{rendered_root:?}");
        assert!(
            rendered.contains(r#"name: Some("'a")"#),
            "a creation reached through a search ('a=⬤; b='a), viewed from the \
             REFERENCING statement, must render with its OWN defining statement's \
             name ('a), got: {rendered}"
        );
    }
}
