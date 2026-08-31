//! `FVMStorage` — the arena-backed FIR store (FOOP-16).
//!
//! Replaces `Rc<RefCell<dyn Fir>>` children and `Weak<RefCell<dyn Fir>>` parent
//! back-pointers with `u32`-indexed arena slots addressed through the validated
//! handle type [`FirPointer`]. See `docs/foop/FOOP-16.md` §Specification for the
//! full design rationale; this module is a direct implementation of that
//! specification's `FirPointer`/`FVMStorage`/`FirSpec` section.
//!
//! # This module's scope, right now
//!
//! This is a **foundational, additive** module (FOOP-16.plan.md Phase 1's
//! first task): at the point this file is introduced, no existing FIR kind's
//! fields have changed yet, `trait Fir` (`fir_trait.rs`) is untouched, and
//! nothing in the rest of the crate calls into this module. The existing
//! `Fir` trait is built around `&self` + interior mutability (`Cell`/
//! `RefCell` inside `ProtoBrane`) specifically so a `Rc<RefCell<dyn Fir>>`
//! handle can be shared and mutated through a shared reference — that design
//! is exactly what the arena replaces (a `&mut Fir` from `FVMStorage::get_mut`
//! provides the same exclusivity `RefCell`'s runtime check gave, but checked
//! at compile time). Storing today's `Fir` trait object unmodified inside a
//! `Slot` would keep the very interior-mutability machinery this FOOP exists
//! to remove, so this task stores a placeholder [`ArenaFir`] payload instead —
//! proven correct against its own small test suite — and each later per-kind
//! migration task (starting with `IndepIntFir`) is where that kind's `Fir`
//! impl is rewritten to take `&FVMStorage`/`&mut FVMStorage` explicitly and
//! becomes the real `Slot` payload. This sequencing matches the plan's own
//! phrasing for this task: "it only adds the new types alongside the old
//! ones."

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};

use foolish_core::fir::Nyes;

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
/// `generation` is not load-bearing for safety in FOOP-16's scope (no slot
/// reuse — see FOOP-16.md §Specification "Arena allocation and expansion");
/// it is carried now so a future FOOP that introduces slot reuse does not need
/// to change `FirPointer`'s shape.
struct Slot {
    payload: ArenaFir,
    parent: FirPointer,
    /// Parse-time children (immutable topology), mirroring
    /// `ProtoBrane::foolish_children`.
    foolish_children: Vec<FirPointer>,
    generation: u32,
}

/// Placeholder per-node payload for this foundational task.
///
/// Deliberately NOT `trait Fir` — see this module's top-level doc comment for
/// why. Holds exactly the data every kind needs generically, mirroring every
/// field [`crate::proto_brane::ProtoBrane`] carries today EXCEPT
/// `foolish_children`/`parent` (those live directly on [`Slot`], since the
/// arena — not each node — owns tree structure) so [`FirCursor`]/
/// [`FirCursorMut`]'s method table (FOOP-16.md §Specification "The
/// `FirCursor`/`FirCursorMut` wrapper") has something real to read and write.
/// Each per-kind migration task replaces reads/writes of this placeholder
/// with that kind's own arena-aware `Fir` impl; `ArenaFir` itself is deleted
/// once every kind has migrated (tracked as part of Phase 1's per-kind tasks,
/// not a separate cleanup).
#[derive(Debug, Clone)]
pub(crate) struct ArenaFir {
    spec: FirSpec,
    nyes: Nyes,
    /// Compute-time children, mirroring `ProtoBrane::ubc_children`. A plain
    /// `Vec` here, not `RefCell<Vec<_>>` — the arena's `&mut FVMStorage`
    /// borrow is the only exclusivity check needed, so the `RefCell` this
    /// field wraps today becomes unnecessary (see FOOP-16.md §Specification
    /// "`FirCursor`/`FirCursorMut`", `ubc_children` row: "removes the
    /// `self.ubc_children.borrow().clone()` dance").
    ubc_children: Vec<FirPointer>,
    /// Task queue, mirroring `ProtoBrane::tasks`.
    tasks: VecDeque<FirPointer>,
    /// Mirrors `ProtoBrane::alarm_reason`.
    alarm_reason: Option<String>,
}

impl ArenaFir {
    /// Mirrors `ProtoBrane::get_nyes`. No caller yet — every current read
    /// goes through `FVMStorage::get_nyes` instead, which has direct slot
    /// access; kept as the symmetric counterpart to `set_nyes` below for a
    /// future caller that already holds an `&ArenaFir` (e.g. inside a
    /// `with_mut`/`get_mut` closure) and would otherwise need to route back
    /// through `FVMStorage` just to read what it already has in hand.
    #[expect(
        dead_code,
        reason = "no caller yet — symmetric counterpart to set_nyes"
    )]
    pub(crate) fn get_nyes(&self) -> Nyes {
        self.nyes
    }

    /// Mirrors `ProtoBrane::set_nyes`. Not further visibility-restricted here
    /// (unlike `ProtoBrane::set_nyes`'s `pub(crate)`, itself already the
    /// tightest this module needs) because `ArenaFir` itself is `pub(crate)`
    /// — the OWNERSHIP CONTRACT (FOOP-62 #10, quoted in full on
    /// `ProtoBrane::set_nyes`) still applies and is enforced the same way:
    /// only a node's own `fir_op_step` or construction may call this, once
    /// each per-kind migration task wires a real `fir_op_step` through
    /// `FVMStorage::with_mut`/`get_mut`. This module has no caller yet to
    /// misuse it.
    pub(crate) fn set_nyes(&mut self, n: Nyes) {
        self.nyes = n;
    }

    /// Mirrors `ProtoBrane::ubc_children`. Returns a slice directly — no
    /// clone-out-of-`Vec` dance, since there is no `RefCell` to reenter (see
    /// this struct's own doc comment on the `ubc_children` field, and
    /// FOOP-16.md §Specification's `FirCursor` method table).
    pub(crate) fn ubc_children(&self) -> &[FirPointer] {
        &self.ubc_children
    }

    /// Mirrors `ProtoBrane::push_ubc_child`: pushes to `ubc_children` AND
    /// enqueues as a task if the child is not already constanic. Takes the
    /// child's current `Nyes` as a parameter (rather than looking it up
    /// itself) because `ArenaFir` cannot reach across slots to read another
    /// node's state — the caller ([`FirCursorMut::push_ubc_child`]) already
    /// has `&FVMStorage` access to read it first.
    pub(crate) fn push_ubc_child(&mut self, child: FirPointer, child_nyes: Nyes) {
        self.ubc_children.push(child);
        if !child_nyes.is_constanic() {
            self.tasks.push_back(child);
        }
    }

    /// Mirrors `ProtoBrane::push_search_result`'s SINGULAR-RESULT INVARIANT
    /// (FOOP-62) `debug_assert!`, verbatim.
    pub(crate) fn push_search_result(&mut self, result: FirPointer, result_nyes: Nyes) {
        debug_assert!(
            self.ubc_children.is_empty(),
            "search FIR already has a result; existing searches are singular-result \
             (ubc_children must be <= 1)"
        );
        self.push_ubc_child(result, result_nyes);
    }

    /// Mirrors `ProtoBrane::clear_ubc_children`.
    pub(crate) fn clear_ubc_children(&mut self) {
        self.ubc_children.clear();
    }

    /// Mirrors `ProtoBrane::front_task`.
    pub(crate) fn front_task(&self) -> Option<FirPointer> {
        self.tasks.front().copied()
    }

    /// Mirrors `ProtoBrane::pop_front_task`.
    pub(crate) fn pop_front_task(&mut self) {
        self.tasks.pop_front();
    }

    /// Mirrors `ProtoBrane::push_task`.
    pub(crate) fn push_task(&mut self, t: FirPointer) {
        self.tasks.push_back(t);
    }

    /// Mirrors `ProtoBrane::set_alarm_reason`.
    #[expect(
        dead_code,
        reason = "no caller yet — wired in by a later Phase 1 per-kind task"
    )]
    pub(crate) fn set_alarm_reason(&mut self, reason: String) {
        self.alarm_reason = Some(reason);
    }

    /// Mirrors `ProtoBrane::alarm_reason`.
    #[expect(
        dead_code,
        reason = "no caller yet — wired in by a later Phase 1 per-kind task"
    )]
    pub(crate) fn alarm_reason(&self) -> Option<&str> {
        self.alarm_reason.as_deref()
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

/// One variant per FIR kind (per `fir_kinds.rs`'s 13 `impl Fir for` sites plus
/// `system_foo.rs`'s `ComparisonFir` — 14 total, confirmed by direct grep
/// against `foolish-ubca2/src/fir_kinds.rs` and `system_foo.rs` when this type
/// was written; see FOOP-16.plan.md Phase 1's "Re-verify the authoritative
/// FIR-kind list" and the `ComparisonFir` plan-adjustment task).
///
/// Each variant's fields mirror that kind's own non-tree-structural fields —
/// parent/children are handled generically by [`FVMStorage::make_my_child`],
/// so no variant carries a parent or child list.
///
/// This enum exists so construction dispatches on data rather than
/// fragmenting into a `create_x_child` method per kind (FOOP-16.md
/// §Specification "`FirPointer`'s handle-side methods").
#[derive(Debug, Clone, PartialEq)]
pub enum FirSpec {
    /// Mirrors [`crate::fir_kinds::IndepIntFir`].
    IndepInt { value: i64 },
    /// Mirrors [`crate::fir_kinds::NkFir`].
    Nk { reason: String },
    /// Mirrors [`crate::fir_kinds::OperatorFir`].
    Operator { op: String },
    /// Mirrors [`crate::fir_kinds::StatementFir`]. `nf_reason` is not part of
    /// the spec — it starts `None` always, a `fir_op_step`-time discovery,
    /// never a construction input (see that kind's own migration task).
    Statement {
        identifier: Identifier,
        line_number: usize,
    },
    /// Mirrors [`crate::fir_kinds::BraneFir`].
    Brane {
        characterizations: Characterizations,
    },
    /// Mirrors [`crate::fir_kinds::SearchFir`]. `sf_inner_pattern` is not part
    /// of the spec — it starts `None` always, matching today's construction
    /// sites.
    Search {
        pattern: String,
        anchored: bool,
        forward: bool,
        is_value_search: bool,
        contexted: bool,
    },
    /// Mirrors [`crate::fir_kinds::IndexFir`].
    Index {
        offset: i32,
        anchored: bool,
        contexted: bool,
    },
    /// Mirrors [`crate::fir_kinds::FoolRefFir`]. `referent` names the original
    /// found statement this reference wraps.
    FoolRef { referent: FirPointer },
    /// Mirrors [`crate::fir_kinds::StayFoolishFir`]. No fields beyond `core`.
    StayFoolish,
    /// Mirrors [`crate::fir_kinds::StayFullyFoolishFir`]. No fields beyond `core`.
    StayFullyFoolish,
    /// Mirrors [`crate::fir_kinds::ConcatHelper`]. No fields beyond `core`.
    ConcatHelper,
    /// Mirrors [`crate::fir_kinds::ConcatenationFir`]. `_helpers_populated` is
    /// not part of the spec — it is derived post-construction, matching
    /// `constanic_clone_at`'s own `FirKind::Concatenation` arm.
    Concatenation {
        provenance: crate::fir_kinds::ConcatProvenance,
    },
    /// Mirrors [`crate::fir_kinds::CreationFir`]. No fields beyond `core`.
    Creation,
    /// Mirrors `system_foo::ComparisonFir` (not in `fir_kinds.rs` — see the
    /// `ComparisonFir` plan-adjustment task in FOOP-16.plan.md Phase 1).
    Comparison { op: crate::system_foo::ComparisonOp },
}

impl FirSpec {
    /// The `Nyes` a freshly-constructed node of this spec starts at, mirroring
    /// each kind's own constructor call to `ProtoBrane::new(.., Nyes::X)` seen
    /// directly in `fir_kinds.rs`/`system_foo.rs` today: every kind starts
    /// `Prembrionic` except `CreationFir` (`Independent`, since a creation is
    /// self-contained with no context dependency) — confirmed by reading
    /// `CreationFir::creation`, `IndepIntFir::constant_int`, `NkFir::nk`, and
    /// every other kind's constructor directly.
    fn initial_nyes(&self) -> Nyes {
        match self {
            FirSpec::Creation => Nyes::Independent,
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

    /// Retrieve, modify, and return in one call — the "retrieve a payload, be
    /// able to modify it before returning" primitive. Closure-scoped so there
    /// is no separate get/set pair to keep in sync, and no `RefCell`-style
    /// runtime borrow tracking is needed: the `&mut self` borrow on
    /// `FVMStorage` is the only exclusivity check required.
    ///
    /// `pub(crate)` for now, not `pub`: the FOOP-16.md spec's final signature
    /// takes `impl FnOnce(&mut Fir) -> R` once a real arena-aware `Fir` trait
    /// exists; today's receiver is the internal [`ArenaFir`] placeholder,
    /// which must not leak as public API before that trait is real (Rule
    /// zero: private defensively, public by design). Widens to `pub` in the
    /// per-kind migration task that gives it its final signature.
    pub(crate) fn with_mut<R>(&mut self, ptr: FirPointer, f: impl FnOnce(&mut ArenaFir) -> R) -> R {
        let index = self.validate(ptr);
        f(&mut self.slots[index].payload)
    }

    /// Retrieve one exclusive, held `&mut ArenaFir` for a run of several
    /// SEQUENTIAL writes with nothing storage-needing interleaved between
    /// them. See FOOP-16.md §Specification "`FVMStorage` — the arena" and the
    /// `OperatorFir::combine` walkthrough that motivated this alongside
    /// `with_mut` — the two are equally powerful; the choice is style.
    ///
    /// `pub(crate)` for now — same reasoning as [`Self::with_mut`] above.
    pub(crate) fn get_mut(&mut self, ptr: FirPointer) -> &mut ArenaFir {
        let index = self.validate(ptr);
        &mut self.slots[index].payload
    }

    /// This pointer's parse-time children, in construction order, mirroring
    /// `ProtoBrane::foolish_children`.
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

    /// Inserts the very first node of a fresh arena, self-parented (its own
    /// `FirPointer` as its parent — mirroring today's `Rc::new_cyclic`
    /// self-`Weak` root convention, confirmed by `proto_brane_parent_link`'s
    /// test in `proto_brane.rs`: a root's parent, upgraded, pointer-equals the
    /// root itself).
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
            payload: ArenaFir {
                spec,
                nyes,
                ubc_children: Vec::new(),
                tasks: VecDeque::new(),
                alarm_reason: None,
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
    /// Creates a fresh arena containing a single self-rooting leaf, mirroring
    /// `fir_trait.rs::tests::make_leaf`'s signature closely enough that a
    /// reader who knows one recognizes the other (FOOP-16.md §Specification
    /// "Test helpers"). A leaf here is an `IndepInt` — the simplest kind with
    /// no interesting children — at the given `Nyes`.
    pub(crate) fn test_leaf(nyes: Nyes) -> (Self, FirPointer) {
        let mut storage = Self::new();
        let ptr = storage.make_root(FirSpec::IndepInt { value: 0 });
        storage.with_mut(ptr, |fir| fir.set_nyes(nyes));
        (storage, ptr)
    }

    /// Creates a fresh arena containing a root `Brane` with the given
    /// children specs, mirroring `fir_trait.rs::tests::make_root_brane`'s
    /// signature closely enough that a reader who knows one recognizes the
    /// other (FOOP-16.md §Specification "Test helpers").
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
    /// Primary construction call site, used everywhere in this FOOP's plan.
    /// Delegates to [`FVMStorage::make_my_child`].
    pub fn create_child(self, storage: &mut FVMStorage, spec: FirSpec) -> FirPointer {
        storage.make_my_child(self, spec)
    }

    /// This pointer's parent, per the arena's stored parent link.
    ///
    /// Always `Some` in practice — even the structural root's "parent" is
    /// itself (mirroring today's self-referential root `Weak`). `Option`
    /// mirrors [`crate::proto_brane::ProtoBrane::parent`]'s signature so
    /// callers migrating from that method keep the same shape; unlike that
    /// method's doc comment (which reserves `None` for teardown), no
    /// arena-era case actually returns `None` — the arena never drops a live
    /// node out from under a valid pointer.
    pub fn get_parent(self, storage: &FVMStorage) -> Option<FirPointer> {
        Some(storage.parent(self))
    }

    /// Whether this pointer is the structural root of its arena (its own
    /// parent).
    pub fn is_root(self, storage: &FVMStorage) -> bool {
        storage.parent(self) == self
    }

    /// Climbs the parent chain to the first brane-like kind, mirroring
    /// [`crate::fir_trait::Fir::_get_my_brane`]: climb until `parent()`
    /// pointer-equals `self` (structural root) → `None`; else check
    /// brane-likeness → stop, else recurse.
    ///
    /// "Brane-like" at this foundational stage is judged directly on
    /// [`FirSpec`] (`Brane` or `ConcatHelper` — the two kinds whose real `Fir`
    /// impls report `is_brane_like() == true` today, confirmed by reading
    /// `BraneFir`'s and `ConcatHelper`'s `Fir` impls directly) rather than
    /// through a `dyn Fir` call, since no kind has a real arena-aware `Fir`
    /// impl yet. Each per-kind migration task's own `is_brane_like` override
    /// supersedes this once that kind's `Fir` impl exists; this method is
    /// re-pointed at that point rather than duplicated.
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
    /// Placeholder-stage judgment, same caveat as [`Self::home_brane`]'s doc
    /// comment: superseded by each per-kind migration's own `is_brane_like`.
    fn is_brane_like(self, storage: &FVMStorage) -> bool {
        matches!(
            storage.get(self),
            FirSpec::Brane { .. } | FirSpec::ConcatHelper
        )
    }

    /// Whether this pointer is a `Statement`.
    fn is_statement(self, storage: &FVMStorage) -> bool {
        matches!(storage.get(self), FirSpec::Statement { .. })
    }

    /// The statement this pointer's search would read as its position,
    /// mirroring [`crate::fir_trait::Fir::_get_my_statement`] exactly: climb
    /// until a `Statement` kind is found, or until `parent()` pointer-equals
    /// `self` (structural root, returned as-is).
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

    /// The settled result this pointer resolves to, if any, mirroring
    /// [`crate::fir_trait::Fir::settled_result`]'s CONTRACT verbatim:
    /// "applies the constanic gate ITSELF — pre-constanic always answers
    /// None."
    fn settled_result(self, storage: &FVMStorage) -> Option<FirPointer> {
        if !storage.get_nyes(self).is_constanic() {
            return None;
        }
        let index = storage.validate(self);
        storage.slots[index].payload.ubc_children().first().copied()
    }

    /// Arena-threaded [`crate::fir_trait::FirRefExt::value`]: recursively
    /// unwraps through `settled_result`, returning `self` when there is none.
    pub fn value(self, storage: &FVMStorage) -> FirPointer {
        match self.settled_result(storage) {
            Some(child) => child.value(storage),
            None => self,
        }
    }

    /// Performs ONE stepping action (check-then-act) and reports progress.
    ///
    /// Direct arena-threaded translation of [`crate::fir_trait::step_inner`],
    /// re-read verbatim from `fir_trait.rs` before writing this (not from any
    /// earlier reconstructed notes): same `MAX_DEPTH` guard, same front-task
    /// constanic-gate (pop vs. recurse), same `Scope` mutation for
    /// `StayFoolish`/`Statement`/brane-like kinds before recursing.
    ///
    /// `fir_op_step` itself is NOT wired in yet — no kind has an arena-aware
    /// `fir_op_step` at this foundational stage (see this module's top-level
    /// doc comment). This method is complete and tested up to that point: the
    /// front-task-present branches (pop / recurse) are exercised by this
    /// task's own tests; the `None` branch (call `fir_op_step`) is a
    /// `todo!()` until the first per-kind migration task gives it something
    /// real to call.
    pub fn step(self, storage: &mut FVMStorage) -> FirPointer {
        step_inner(self, storage, 0)
    }
}

/// Guard against runaway recursion on pathologically deep trees. Same value
/// as `fir_trait.rs`'s `MAX_DEPTH`, re-read directly from that file (not
/// reconstructed) to confirm the match.
const MAX_DEPTH: usize = 100;

/// Recursion companion for [`FirPointer::step`], carrying the depth counter.
/// Direct translation of `fir_trait.rs`'s real `step_inner`, re-read in full
/// immediately before writing this function:
///
/// ```text
/// fn step_inner(this: &FirRef, scope: &Scope, depth: usize) -> Result<StepReport, UbcError> {
///     if depth > MAX_DEPTH { return Ok(StepReport::NoProgress); }
///     let front = this.borrow().core().front_task();
///     match front {
///         Some(front_rc) => {
///             if front_rc.borrow().core().get_nyes().is_constanic() {
///                 this.borrow().core().pop_front_task();
///             } else {
///                 // Scope mutation for StayFoolish/Statement/brane-like, then recurse.
///                 step_inner(&front_rc, &child_scope, depth + 1)?;
///             }
///             Ok(StepReport::Progress(this.borrow().core().get_nyes()))
///         }
///         None => {
///             this.borrow().fir_op_step(scope)?;
///             Ok(StepReport::Progress(this.borrow().core().get_nyes()))
///         }
///     }
/// }
/// ```
///
/// This translation returns the pointer itself rather than a `StepReport` —
/// callers read `storage.get_nyes(ptr)` for the report; `Scope` threading and
/// `fir_op_step` dispatch are deferred to the per-kind/evaluator migration
/// tasks that give them something real to operate on (Phase 1 per-kind tasks,
/// Phase 3 for the evaluator loop itself) — this function proves the
/// pop-vs-recurse shape is faithfully preserved under the arena now, before
/// any kind depends on it.
fn step_inner(ptr: FirPointer, storage: &mut FVMStorage, depth: usize) -> FirPointer {
    if depth > MAX_DEPTH {
        return ptr;
    }
    let front = storage.with_mut(ptr, |fir| fir.front_task());
    match front {
        Some(front_ptr) => {
            if storage.get_nyes(front_ptr).is_constanic() {
                storage.with_mut(ptr, |fir| fir.pop_front_task());
            } else {
                step_inner(front_ptr, storage, depth + 1);
            }
            ptr
        }
        None => {
            fir_op_step(ptr, storage);
            ptr
        }
    }
}

/// Enum-dispatch `fir_op_step` equivalent (rust_instructions.md §7 "Enum
/// dispatch": matching a known, finite variant set and calling concrete
/// logic is preferred over `dyn` here, since `FirSpec`'s variant set is
/// exactly the crate's FIR-kind set). Each per-kind migration task adds its
/// kind's real combining logic here, translated from that kind's real
/// `impl Fir for XFir`'s `fir_op_step` (re-read directly, not reconstructed).
///
/// Kinds not yet migrated panic with `todo!()` naming the kind — this must
/// never silently no-op (`rust_instructions.md`'s "implement fully or don't
/// add it"), and every kind's real dispatch is closed out by the end of
/// Phase 1's per-kind sweep (tracked per-kind in this plan's checkboxes).
fn fir_op_step(ptr: FirPointer, storage: &mut FVMStorage) {
    let spec = storage.get(ptr).clone();
    match spec {
        // Direct translation of `impl Fir for IndepIntFir`'s real
        // `fir_op_step` (re-read from `fir_kinds.rs` immediately before
        // writing this): "if not already constanic, set Constant." An
        // IndepInt never has children/tasks, so there is no Braning phase —
        // one step settles it, exactly as today's
        // `constant_int_prembrionic_to_constant_in_one_step` test pins.
        FirSpec::IndepInt { .. } => {
            if !storage.get_nyes(ptr).is_constanic() {
                storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Constant));
            }
        }
        // Direct translation of `impl Fir for NkFir`'s real `fir_op_step`
        // (re-read from `fir_kinds.rs` immediately before writing this):
        // identical one-step-settles shape to IndepInt, settling to Nk
        // instead of Constant.
        FirSpec::Nk { .. } => {
            if !storage.get_nyes(ptr).is_constanic() {
                storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Nk));
            }
        }
        // Direct translation of `impl Fir for OperatorFir`'s real
        // `fir_op_step` (re-read from `fir_kinds.rs` immediately before
        // writing this, including its `combine` helper). NOTE: `OperatorFir`
        // is NOT brane-like under the arena either — re-checked directly:
        // it has no `stmt_count`/`is_brane_like` override in the real `impl
        // Fir for OperatorFir`, and no such note exists anywhere in the
        // current `AGENTS.md`/`CLAUDE.md` (grepped, zero matches) — the
        // plan's own "AGENTS.md describes it as brane-like (FOOP-9)" note is
        // stale/incorrect, recorded as a non-blocking doubt in this
        // checkbox's completion note, not acted on (the real source is the
        // authority, not the plan's paraphrase of it).
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
        // Direct translation of `impl Fir for StatementFir`'s real
        // `fir_op_step`'s CORE settle shape only (re-read from `fir_kinds.rs`
        // immediately before writing this): push the body as a task, then
        // once it's constanic, adopt its NYES. The two NF-refusal checks
        // (`check_null_const_conflict`/`check_rename_of_named_creation`,
        // FOOP-33 §4) are DEFERRED here, not implemented — both call
        // `_ib_search`/`_ab_search`/`.value()`, which are themselves
        // search-engine operations Phase 2 owns exclusively (this module's
        // own `SearchFir`/`IndexFir` tasks already carve out the identical
        // exception for the same reason). Implementing a fake NF check
        // against nothing real would be worse than not implementing it;
        // deferred explicitly to a follow-up once Phase 2's search engine
        // migration gives `_ib_search`/`_ab_search`/`.value()` arena
        // equivalents to call. `nf_reason`/`settled_result`'s override are
        // likewise deferred — `ArenaFir` carries no `nf_reason` slot yet.
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
                        storage.with_mut(ptr, |fir| fir.set_nyes(body_nyes));
                    }
                }
            }
            _ => {}
        },
        // Direct translation of `impl Fir for BraneFir`'s real `fir_op_step`
        // (re-read from `fir_kinds.rs` immediately before writing this).
        // `_ab_search`/`_search_brane` overrides are DEFERRED — Phase 2's
        // job, same carve-out as `SearchFir`/`IndexFir`/`Statement` above.
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
        other => todo!("fir_op_step for {other:?}: migrated by that kind's own per-kind task"),
    }
}

/// Direct arena-threaded translation of `fir_kinds.rs`'s free function
/// `_decide_nyes_due_to_children` (re-read immediately before writing this),
/// used by `BraneFir`'s (and `ConcatHelper`'s, once migrated)
/// `fir_op_step`'s `Braning` classification arm. Preserves the exact
/// priority order: all-Independent → Independent; all-terminal-and-not-that
/// → Constant; any pre-constanic → Braning (keep waiting); else any
/// Econstanic/Woconstanic → Woconstanic; else any Nk → Nk.
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

/// Direct arena-threaded translation of `impl OperatorFir { fn combine }`
/// (re-read from `fir_kinds.rs` immediately before writing this — the exact
/// function FOOP-16.md's own specification walks through as the motivating
/// example for `create_child`/`FVMStorage::get_mut`). Each of the four
/// "build standalone, then `constanic_clone_at`-to-reparent" triplets in the
/// original (NK-from-child, division-by-zero, modulo-by-zero, arithmetic
/// result) collapses to ONE `create_child` call here, exactly as the FOOP's
/// Motivation section predicts — `create_child` builds the node already
/// parented under `ptr`, so there is no separate "clone to reparent" step at
/// all under the arena.
///
/// `scope.has_ancestral_sfm` (threaded into `constanic_clone_at` in the
/// original) has no arena equivalent parameter here: `clone_subtree` isn't
/// invoked at all in this translation, because `create_child` already
/// produces an already-parented node — there is nothing to `clone_subtree`.
/// This is the arena-era simplification the FOOP's Motivation section
/// describes directly, not an omission.
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
        let nk_ptr = ptr.create_child(storage, FirSpec::Nk { reason });
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
                let nk_ptr = ptr.create_child(
                    storage,
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
                let nk_ptr = ptr.create_child(
                    storage,
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
        // FOOP-75 §7's deleted "$" arm stays deleted here too — the real
        // `combine` no longer has it (confirmed by direct re-read), so there
        // is nothing to translate.
        _ => {
            // The real `combine` returns `Err(UbcError::Eval(...))` here.
            // This arena translation has no `Result` return (matching
            // `fir_op_step`'s own signature in this module, which is
            // infallible at this stage — error propagation through the
            // arena's `fir_op_step` dispatch is deferred to Phase 3's
            // evaluator migration, which gives `step`/`fir_op_step` their
            // real `Result<_, UbcError>` signatures). An unknown operator is
            // an internal-consistency condition (the compiler is the only
            // producer of `Operator` specs and only ever uses known
            // operators), so `unreachable!` here is a faithful placeholder,
            // not a swallowed error class.
            unreachable!(
                "combine: unknown operator {op:?} ({} operands)",
                values.len()
            )
        }
    };

    let result_ptr = ptr.create_child(storage, FirSpec::IndepInt { value: result });
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

    /// Mirrors `ProtoBrane::foolish_children`.
    pub fn foolish_children(&self) -> &'s [FirPointer] {
        self.storage.foolish_children(self.ptr)
    }

    /// Mirrors `ProtoBrane::ubc_children`.
    pub fn ubc_children(&self) -> &'s [FirPointer] {
        let index = self.storage.validate(self.ptr);
        self.storage.slots[index].payload.ubc_children()
    }

    /// Mirrors `ProtoBrane::all_children`: ubc first (evaluator renders as
    /// `result=`), then foolish — same render-order contract preserved
    /// exactly.
    pub fn all_children(&self) -> impl Iterator<Item = FirPointer> + 's {
        self.ubc_children()
            .iter()
            .chain(self.foolish_children())
            .copied()
    }

    /// Mirrors `ProtoBrane::parent` (`Weak::upgrade()`), simplified: `None`
    /// remains only for the true structural root (see [`FirPointer::get_parent`]'s
    /// doc comment for why the "only during teardown" case disappears under
    /// the arena).
    pub fn parent(&self) -> Option<FirPointer> {
        self.ptr.get_parent(self.storage)
    }

    /// Mirrors `ProtoBrane::is_root`, simplified: no `self_rc` parameter
    /// needed — `self.ptr` already carries self-identity (see
    /// [`FirPointer::is_root`]'s doc comment).
    pub fn is_root(&self) -> bool {
        self.ptr.is_root(self.storage)
    }

    /// Mirrors `ProtoBrane::get_nyes`.
    pub fn get_nyes(&self) -> Nyes {
        self.storage.get_nyes(self.ptr)
    }

    /// Mirrors `ProtoBrane::front_task`.
    pub fn front_task(&self) -> Option<FirPointer> {
        let index = self.storage.validate(self.ptr);
        self.storage.slots[index].payload.front_task()
    }

    /// Mirrors [`crate::fir_trait::Fir::_get_my_brane`].
    pub fn home_brane(&self) -> Option<FirCursor<'s>> {
        self.ptr
            .home_brane(self.storage)
            .map(|p| FirCursor::new(p, self.storage))
    }

    /// Mirrors [`crate::fir_trait::Fir::_get_my_statement`].
    pub fn statement(&self) -> FirCursor<'s> {
        FirCursor::new(self.ptr.get_my_statement(self.storage), self.storage)
    }

    /// Mirrors [`crate::fir_trait::Fir::settled_result`]'s CONTRACT verbatim:
    /// applies the constanic gate itself.
    pub fn settled_result(&self) -> Option<FirCursor<'s>> {
        self.ptr
            .settled_result(self.storage)
            .map(|p| FirCursor::new(p, self.storage))
    }

    /// Mirrors [`crate::fir_trait::Fir::as_i64`]: `IndepInt` reports its own
    /// value directly (its real `Fir` impl's override, re-read directly);
    /// every other migrated kind falls through to `settled_result` first,
    /// matching the trait's default body exactly. Kinds not yet migrated
    /// answer `None` rather than panicking — `as_i64`'s default in
    /// `fir_trait.rs` already tolerates "no settled result" as `None`, so an
    /// unmigrated kind (which never settles under the arena yet) is exactly
    /// that case, not a distinct failure mode needing its own `todo!()`.
    pub fn as_i64(&self) -> Option<i64> {
        match self.node() {
            FirSpec::IndepInt { value } => Some(*value),
            _ => self.settled_result().and_then(|c| c.as_i64()),
        }
    }

    /// Mirrors [`crate::fir_trait::Fir::as_nk_reason`]: `Nk`'s own override
    /// (re-read directly) returns its reason string; every other kind's
    /// default is `None`.
    pub fn as_nk_reason(&self) -> Option<&'s str> {
        match self.node() {
            FirSpec::Nk { reason } => Some(reason),
            _ => None,
        }
    }

    /// Mirrors [`crate::fir_trait::Fir::as_op_name`]: `Operator`'s own
    /// override (re-read directly) returns its op string; every other
    /// kind's default is `None`.
    pub fn as_op_name(&self) -> Option<&'s str> {
        match self.node() {
            FirSpec::Operator { op } => Some(op),
            _ => None,
        }
    }

    /// Mirrors [`crate::fir_trait::Fir::as_stmt_identifier`]: `Statement`'s
    /// own override (re-read directly) returns its identifier.
    pub fn as_stmt_identifier(&self) -> Option<&'s Identifier> {
        match self.node() {
            FirSpec::Statement { identifier, .. } => Some(identifier),
            _ => None,
        }
    }

    /// Mirrors [`crate::fir_trait::Fir::as_stmt_line_number`]: `Statement`'s
    /// own override (re-read directly) returns its line number.
    pub fn as_stmt_line_number(&self) -> Option<usize> {
        match self.node() {
            FirSpec::Statement { line_number, .. } => Some(*line_number),
            _ => None,
        }
    }

    /// Mirrors [`crate::fir_trait::Fir::stmt_count`]: `Brane`'s own override
    /// (re-read directly) reports its foolish-children count; every other
    /// kind's default is `None` (not brane-like).
    pub fn stmt_count(&self) -> Option<usize> {
        match self.node() {
            FirSpec::Brane { .. } => Some(self.foolish_children().len()),
            _ => None,
        }
    }

    /// Mirrors [`crate::fir_trait::Fir::stmt_at`]: `Brane`'s own override
    /// (re-read directly) indexes its foolish children directly.
    pub fn stmt_at(&self, idx: usize) -> Option<FirPointer> {
        match self.node() {
            FirSpec::Brane { .. } => self.foolish_children().get(idx).copied(),
            _ => None,
        }
    }

    /// Mirrors [`crate::fir_trait::Fir::as_brane_characterizations`]:
    /// `Brane`'s own override (re-read directly) returns its
    /// characterizations' components.
    pub fn as_brane_characterizations(&self) -> &'s [String] {
        match self.node() {
            FirSpec::Brane { characterizations } => characterizations.components(),
            _ => &[],
        }
    }

    /// Mirrors [`crate::fir_trait::Fir::is_brane_like`]: `stmt_count().is_some()`.
    pub fn is_brane_like(&self) -> bool {
        self.stmt_count().is_some()
    }

    /// Mirrors [`crate::fir_trait::Fir::as_search_pattern`]: `Search`'s own
    /// override (re-read directly). Pure data accessor — NOT search
    /// execution, so unlike `fir_op_step` this is safe to implement in
    /// Phase 1 without touching Phase 2's search-engine scope.
    pub fn as_search_pattern(&self) -> Option<&'s str> {
        match self.node() {
            FirSpec::Search { pattern, .. } => Some(pattern),
            _ => None,
        }
    }

    /// Mirrors [`crate::fir_trait::Fir::as_search_anchored`].
    pub fn as_search_anchored(&self) -> bool {
        matches!(self.node(), FirSpec::Search { anchored: true, .. })
    }

    /// Mirrors [`crate::fir_trait::Fir::as_search_is_value`].
    pub fn as_search_is_value(&self) -> bool {
        matches!(
            self.node(),
            FirSpec::Search {
                is_value_search: true,
                ..
            }
        )
    }

    /// Mirrors [`crate::fir_trait::Fir::as_search_contexted`]: `Search` and
    /// `Index` both carry a `contexted` flag (re-read directly — both real
    /// `impl Fir` override this method with their own field).
    pub fn as_search_contexted(&self) -> bool {
        match self.node() {
            FirSpec::Search { contexted, .. } | FirSpec::Index { contexted, .. } => *contexted,
            _ => false,
        }
    }

    /// Mirrors [`crate::fir_trait::Fir::as_index_offset`]: `Index`'s own
    /// override (re-read directly).
    pub fn as_index_offset(&self) -> i32 {
        match self.node() {
            FirSpec::Index { offset, .. } => *offset,
            _ => 0,
        }
    }

    /// Mirrors [`crate::fir_trait::Fir::as_index_anchored`].
    pub fn as_index_anchored(&self) -> bool {
        matches!(self.node(), FirSpec::Index { anchored: true, .. })
    }
}

/// The mutating counterpart of [`FirCursor`]. Rust allows only one `&mut` at
/// a time, so unlike `FirCursor` this does NOT support "wrap once, call five
/// mutating methods" — each mutating call still needs its own `&mut`
/// reborrow under the hood. Its value is bundling `ptr`+`storage` for ONE
/// logical mutating operation, not batching several (see FOOP-16.md
/// §Specification's resolution of the two-cursor-type design question: the
/// real complaint `get_mut` already answers is "several writes with nothing
/// storage-needing in between," not "I need `&mut` too often").
///
/// **Must never be held live across a call into [`FirPointer::step`]** — see
/// FOOP-16.md §Specification "Borrow discipline under the arena." This is a
/// discipline enforced by the type system for the SAME reason `RefCell`'s
/// borrow panic enforces it today, just caught one build earlier (a compile
/// error, not a runtime panic risk).
pub struct FirCursorMut<'s> {
    ptr: FirPointer,
    storage: &'s mut FVMStorage,
}

impl<'s> FirCursorMut<'s> {
    /// Wraps `ptr` for mutating through `storage`.
    pub fn new(ptr: FirPointer, storage: &'s mut FVMStorage) -> Self {
        Self { ptr, storage }
    }

    /// Mirrors `ProtoBrane::set_nyes`'s OWNERSHIP CONTRACT verbatim (FOOP-62
    /// #10): a FIR owns its own nyes — nyes must NOT be changed from outside
    /// the FIR. The ONLY sanctioned writers are (1) a FIR on ITSELF, inside
    /// its own `fir_op_step`, and (2) construction. `pub(crate)` — not
    /// `pub` — is the enforcement mechanism, exactly as it is on
    /// `ProtoBrane::set_nyes` today.
    ///
    /// No caller through THIS wrapper yet (this task's own tests and
    /// `clone_subtree` call `ArenaFir::set_nyes` directly via
    /// `with_mut`/`get_mut`, since neither is "a FIR on itself inside its own
    /// `fir_op_step`" — construction is the other sanctioned writer, which is
    /// exactly what those two call sites are). `FirCursorMut::set_nyes`
    /// becomes reachable once the first per-kind migration task gives a real
    /// `fir_op_step` a `FirCursorMut` to call it through.
    #[expect(
        dead_code,
        reason = "reachable once a real fir_op_step exists to call it"
    )]
    pub(crate) fn set_nyes(&mut self, n: Nyes) {
        self.storage.get_mut(self.ptr).set_nyes(n);
    }

    /// Delegates to [`FirPointer::create_child`] — the live, in-arena
    /// equivalent of `ProtoBrane::push_foolish_child` for a node that needs
    /// to grow a new child post-construction. There is deliberately no
    /// `FirCursorMut` equivalent of `push_foolish_child` itself: that method
    /// is construction-time-only (`&mut self` on a not-yet-live
    /// `ProtoBrane`), a case `create_child` already covers completely (see
    /// FOOP-16.md §Specification's `FirCursorMut` method table).
    pub fn create_child(&mut self, spec: FirSpec) -> FirPointer {
        self.ptr.create_child(self.storage, spec)
    }

    /// Mirrors `ProtoBrane::push_foolish_child_sff_marked`: pushes a
    /// parse-time child under an SF/SFF marker, panicking (unconditionally —
    /// not a `debug_assert!`) if any search-kind descendant of `child` is not
    /// exactly `ECONSTANIC`. `child` must already be a child of `self.ptr`
    /// (i.e. already `create_child`-ed) — unlike today's `ProtoBrane` method,
    /// the arena's `create_child` already wires parent/child atomically, so
    /// this method's ONLY remaining job is the invariant CHECK, not the push.
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

    /// Mirrors `ProtoBrane::push_ubc_child`: pushes to `ubc_children` AND
    /// enqueues as a task if the child is not already constanic.
    pub fn push_ubc_child(&mut self, child: FirPointer) {
        let child_nyes = self.storage.get_nyes(child);
        self.storage
            .get_mut(self.ptr)
            .push_ubc_child(child, child_nyes);
    }

    /// Mirrors `ProtoBrane::push_search_result`'s SINGULAR-RESULT INVARIANT
    /// (FOOP-62) `debug_assert!`.
    pub fn push_search_result(&mut self, result: FirPointer) {
        let result_nyes = self.storage.get_nyes(result);
        self.storage
            .get_mut(self.ptr)
            .push_search_result(result, result_nyes);
    }

    /// Mirrors `ProtoBrane::clear_ubc_children`.
    pub fn clear_ubc_children(&mut self) {
        self.storage.get_mut(self.ptr).clear_ubc_children();
    }

    /// Mirrors `ProtoBrane::pop_front_task`.
    pub fn pop_front_task(&mut self) {
        self.storage.get_mut(self.ptr).pop_front_task();
    }

    /// Mirrors `ProtoBrane::push_task`.
    pub fn push_task(&mut self, t: FirPointer) {
        self.storage.get_mut(self.ptr).push_task(t);
    }
}

/// The first descendant search kind (per `fir_kinds.rs`'s
/// `ProtoBrane::sift_for_first_non_econstanic_descendent_search`, re-read
/// directly before writing this) that is NOT exactly `Nyes::Econstanic`, or
/// `None` if every one of them is. Arena-threaded translation, preserving the
/// exact `== Econstanic` check (not `is_constanic()`) — see that method's own
/// doc comment for why the distinction matters (an SFF-marked search sitting
/// at CONSTANT or NK means it DID run, which is exactly what this guard
/// catches).
///
/// Naming: `sift_*`, not `search_*` — an ordinary Rust-side tree walk with no
/// Foolish search semantics (see AGENTS.md/CLAUDE.md's "Sift" terminology).
///
/// "Search kind" is judged directly on [`FirSpec`] (`Search`/`Index`) at this
/// foundational stage, mirroring `Fir::is_search_kind`'s exact variant set
/// (`FirKind::Search | FirKind::Index`, confirmed by reading `fir_trait.rs`
/// directly) rather than through a `dyn Fir` call.
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
/// `$handle` is a `&mut`-typed borrow (e.g. `&mut ArenaFir` from
/// `FVMStorage::get_mut`), so ending its borrow is `let _ = $handle;`, not
/// `drop($handle)` — `drop` on a `&mut T` reference is a no-op (it drops the
/// reference value itself, a `Copy`-free but trivially-droppable pointer, not
/// the pointee), which `clippy::drop_ref`/rustc's own `dropping_references`
/// lint catches. `let _ = ...` genuinely ends the borrow's lifetime at that
/// point under NLL, which is the actual effect this macro needs.
///
/// See FOOP-16.md §Specification: not exercised by `OperatorFir::combine`
/// itself (that walkthrough is what motivated `FVMStorage::get_mut` instead);
/// kept available here for whichever later per-kind or evaluator-migration
/// function turns out to need the rarer interleaved-reacquisition shape.
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
    /// Direct arena-threaded translation of `ProtoBrane::constanic_clone_at`
    /// (re-read in full from `fir_kinds.rs` immediately before writing this —
    /// not from any earlier reconstructed notes). Recursive, per-node,
    /// matching on the source's [`FirSpec`] — NOT a bulk subtree copy.
    /// Preserves:
    ///
    /// 1. **Share-not-clone.** `Constant`/`Independent` non-`Brane` nodes
    ///    return the SAME `FirPointer`, not a new slot. `FoolRef` and
    ///    `Creation` kinds ALWAYS share, unconditionally, regardless of NYES
    ///    state — this is what keeps the `FoolRefFir` two-child invariant's
    ///    original-statement reference genuinely shared, and a named
    ///    creation's identity intact.
    /// 2. **`StayFoolish`/`StayFullyFoolish` unwrapping** — not yet
    ///    representable: no `FirSpec` variant carries a foolish-children
    ///    Vec/settled-result placeholder to unwrap through at this
    ///    foundational stage in a way that would exercise real behavior (SF
    ///    unwrapping's whole point is reading the WRAPPED kind's real state,
    ///    which does not exist yet). Deferred explicitly to the
    ///    `StayFoolishFir`/`StayFullyFoolishFir` per-kind migration tasks,
    ///    which re-implement this arm against those kinds' real arena-aware
    ///    `Fir` impls — noted here rather than faked with a placeholder that
    ///    would look tested without exercising the real unwrap logic.
    /// 3. **Recursive per-node rebuild** for every other kind: children come
    ///    from cloning each `foolish_children`/`ubc_children` entry in turn
    ///    (mirroring `clone_children_for_constanic_clone`), so the whole
    ///    subtree is rebuilt top-down, one recursive call per surviving node.
    ///
    /// `index` becomes a cloned `Statement`'s new `line_number`, exactly as
    /// `constanic_clone_at` does today (used directly as the position, not
    /// carried over from the original). `sfm` and `skip_foolish_children` are
    /// threaded through exactly as their names suggest, matching the source
    /// method's own parameters one-for-one.
    ///
    /// A pointer into the original subtree remains exactly as valid after a
    /// clone as it was before: this method only ADDS new slots for the
    /// freshly-rebuilt nodes; the original subtree's slots are untouched —
    /// the arena-era restatement of the correctness property `Rc` reference
    /// counting gives "for free" today (see FOOP-16.md §Specification).
    pub fn clone_subtree(
        &mut self,
        root: FirPointer,
        new_parent: FirPointer,
        index: usize,
        sfm: bool,
        skip_foolish_children: bool,
    ) -> FirPointer {
        let nyes = self.get_nyes(root);
        let spec = self.get(root).clone();

        // 1. Share-not-clone: Constant/Independent non-Brane always shares;
        // FoolRef/Creation always share regardless of NYES.
        let is_share_kind = matches!(spec, FirSpec::FoolRef { .. } | FirSpec::Creation);
        let is_constanic_non_brane = matches!(nyes, Nyes::Constant | Nyes::Independent)
            && !matches!(spec, FirSpec::Brane { .. });
        if is_share_kind || is_constanic_non_brane {
            return root;
        }

        // 3. Recursive per-node rebuild. The new node's own spec is the
        // source's spec, with a Statement's line_number renumbered to
        // `index` (mirroring constanic_clone_at's FirKind::Statement arm
        // exactly: `let line = index;`).
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
                self.clone_subtree(child, new_ptr, i, sfm, false);
            }
        }
        let ubc_children: Vec<FirPointer> = {
            let index_in_slots = self.validate(root);
            self.slots[index_in_slots].payload.ubc_children().to_vec()
        };
        for ubc in ubc_children {
            let cloned = self.clone_subtree(ubc, new_ptr, 0, sfm, false);
            let cloned_nyes = self.get_nyes(cloned);
            self.with_mut(new_ptr, |fir| fir.push_ubc_child(cloned, cloned_nyes));
        }
        new_ptr
    }
}

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

    /// A freshly-created node starts at its spec's initial `Nyes` — matching
    /// every kind's own constructor call to `ProtoBrane::new(.., Nyes::X)`.
    #[test]
    fn initial_nyes_matches_each_kinds_own_constructor() {
        let mut storage = FVMStorage::new();
        let root = storage.make_root(brane_spec());
        assert_eq!(storage.get_nyes(root), Nyes::Prembrionic);

        let creation = root.create_child(&mut storage, FirSpec::Creation);
        assert_eq!(storage.get_nyes(creation), Nyes::Independent);

        let int_child = root.create_child(&mut storage, FirSpec::IndepInt { value: 1 });
        assert_eq!(storage.get_nyes(int_child), Nyes::Prembrionic);
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

    /// `test_leaf`/`test_root_brane` round-trip correctly, mirroring
    /// `fir_trait.rs`'s `make_leaf`/`make_root_brane` shortcuts.
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
        // `child` has no `Statement` ancestor, so `_get_my_statement`'s real
        // logic (re-verified directly against `fir_trait.rs`) climbs all the
        // way to the structural root and stops there — NOT back to `child`
        // itself. `root` is where the climb terminates (its own parent is
        // itself), matching the shape `get_my_statement_returns_self_if_statement`'s
        // SIBLING test `get_my_statement_climbs_to_parent_statement` documents
        // for the analogous real-Statement case.
        assert_eq!(cursor.statement().ptr, root);
        assert!(cursor.settled_result().is_none()); // IndepInt never has a settled_result body
    }

    /// `FirCursorMut::push_ubc_child` mirrors `ProtoBrane::push_ubc_child`
    /// exactly: pushes to `ubc_children` AND enqueues as a task only when the
    /// child is not already constanic.
    #[test]
    fn fir_cursor_mut_push_ubc_child_enqueues_only_non_constanic_children() {
        let (mut storage, root) = FVMStorage::test_root_brane(&[]);
        let settled = root.create_child(&mut storage, FirSpec::IndepInt { value: 1 });
        storage.with_mut(settled, |fir| fir.set_nyes(Nyes::Constant));
        let unsettled = root.create_child(&mut storage, FirSpec::IndepInt { value: 2 });

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
    /// its `debug_assert!` on a second push — mirrors
    /// `ProtoBrane::push_search_result`'s own test coverage intent.
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

    /// `clone_subtree`'s share-not-clone behavior: a `Creation` always shares
    /// the SAME `FirPointer`, regardless of NYES — the FoolRef/Creation
    /// unconditional-share rule from `constanic_clone_at`.
    #[test]
    fn clone_subtree_shares_creation_unconditionally() {
        let (mut storage, root) = FVMStorage::test_root_brane(&[]);
        let creation = root.create_child(&mut storage, FirSpec::Creation);
        let other_root = storage.make_root(FirSpec::IndepInt { value: 0 });

        let cloned = storage.clone_subtree(creation, other_root, 0, false, false);
        assert_eq!(cloned, creation, "Creation must share, never clone");
    }

    /// `clone_subtree`'s share-not-clone behavior for a `Constant`
    /// non-`Brane` node: returns the SAME pointer, not a new slot.
    #[test]
    fn clone_subtree_shares_constant_non_brane() {
        let (mut storage, root) = FVMStorage::test_root_brane(&[]);
        let settled = root.create_child(&mut storage, FirSpec::IndepInt { value: 42 });
        storage.with_mut(settled, |fir| fir.set_nyes(Nyes::Constant));
        let other_root = storage.make_root(FirSpec::IndepInt { value: 0 });

        let cloned = storage.clone_subtree(settled, other_root, 0, false, false);
        assert_eq!(
            cloned, settled,
            "Constant non-Brane must share, never clone"
        );
    }

    /// `clone_subtree`'s full-rebuild behavior: a pre-constanic node is
    /// rebuilt as a genuinely new pointer under the new parent, with its
    /// foolish children recursively cloned too, and a `Statement`'s
    /// `line_number` renumbered to the passed `index` — exactly as
    /// `constanic_clone_at`'s `FirKind::Statement` arm does today
    /// (`let line = index;`).
    #[test]
    fn clone_subtree_rebuilds_pre_constanic_nodes_and_renumbers_statement_lines() {
        let (mut storage, root) = FVMStorage::test_root_brane(&[]);
        let stmt = root.create_child(
            &mut storage,
            FirSpec::Statement {
                identifier: Identifier::from_parts(vec![], "x"),
                line_number: 99, // original position — must be overwritten by `index` below
            },
        );
        let other_root = storage.make_root(FirSpec::IndepInt { value: 0 });

        let cloned = storage.clone_subtree(stmt, other_root, 3, false, false);
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

    /// `clone_subtree` recursively clones foolish children, preserving count
    /// and (for pre-constanic children) producing fresh pointers for each.
    #[test]
    fn clone_subtree_recursively_clones_foolish_children() {
        let (mut storage, root) = FVMStorage::test_root_brane(&[
            FirSpec::IndepInt { value: 1 },
            FirSpec::IndepInt { value: 2 },
        ]);
        let other_root = storage.make_root(FirSpec::IndepInt { value: 0 });

        let cloned = storage.clone_subtree(root, other_root, 0, false, false);
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
    fn clone_subtree_skip_foolish_children_omits_them() {
        let (mut storage, root) = FVMStorage::test_root_brane(&[FirSpec::IndepInt { value: 1 }]);
        let other_root = storage.make_root(FirSpec::IndepInt { value: 0 });

        let cloned = storage.clone_subtree(root, other_root, 0, false, true);
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

    /// `IndepIntFir`'s arena migration: mirrors the existing
    /// `fir_kinds.rs::tests::constant_int_prembrionic_to_constant_in_one_step`
    /// test exactly — Prembrionic → Constant in ONE step, no Braning phase
    /// (an IndepInt has no children/tasks).
    #[test]
    fn indep_int_prembrionic_to_constant_in_one_step() {
        let mut storage = FVMStorage::new();
        let node = storage.make_root(FirSpec::IndepInt { value: 42 });
        assert_eq!(storage.get_nyes(node), Nyes::Prembrionic);

        node.step(&mut storage);

        assert_eq!(storage.get_nyes(node), Nyes::Constant);
        assert_eq!(FirCursor::new(node, &storage).as_i64(), Some(42));
    }

    /// Stepping an already-constanic `IndepInt` again is a no-op — mirrors
    /// `stepping_already_settled_is_noop`'s intent for this kind.
    #[test]
    fn indep_int_stepping_already_settled_is_noop() {
        let mut storage = FVMStorage::new();
        let node = storage.make_root(FirSpec::IndepInt { value: 1 });
        node.step(&mut storage);
        assert_eq!(storage.get_nyes(node), Nyes::Constant);

        node.step(&mut storage);
        assert_eq!(storage.get_nyes(node), Nyes::Constant);
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

    /// `OperatorFir`'s arena migration: mirrors
    /// `fir_kinds.rs::tests::operator_nyes_transitions` exactly — `2 + 3`
    /// settles Constant with value `5`. Both operands start pre-settled
    /// (`Constant`), so `combine` fires without a genuine Braning-phase
    /// child-stepping round-trip — this test's own `step` loop drains the
    /// (already-constanic) operand tasks first, then settles via `combine`,
    /// exactly mirroring the two-step shape `step_to_settled` exercises in
    /// the original test.
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
        let a = op.create_child(&mut storage, FirSpec::IndepInt { value: 2 });
        let _b = op.create_child(&mut storage, FirSpec::IndepInt { value: 3 });
        // `a`/`b` both start Prembrionic (unsettled) — the default.

        op.step(&mut storage);

        assert_eq!(storage.get_nyes(op), Nyes::Braning);
        assert_eq!(
            FirCursor::new(op, &storage).front_task(),
            Some(a),
            "unsettled operands must be queued as tasks"
        );
    }

    /// `StatementFir`'s arena migration (core settle shape only — the two
    /// NF-refusal checks are deferred to a follow-up after Phase 2's search
    /// engine migration; see this kind's `fir_op_step` arm doc comment).
    /// Mirrors `fir_kinds.rs::tests::statement_nyes_transitions` exactly:
    /// `a = 9` settles Constant.
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

        assert_eq!(storage.get_nyes(body), Nyes::Constant);
        assert_eq!(storage.get_nyes(stmt), Nyes::Constant);
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

    /// `BraneFir`'s arena migration (`_ab_search`/`_search_brane` overrides
    /// deferred to Phase 2). Mirrors `fir_kinds.rs::tests::brane_nyes_transitions`
    /// exactly: a brane of two settled statements settles Constant.
    #[test]
    fn brane_of_settled_statements_settles_constant() {
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

        assert_eq!(storage.get_nyes(brane), Nyes::Constant);
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

    /// An empty `Brane` settles `Constant` in one step — mirrors the
    /// `children.is_empty()` short-circuit in the real `fir_op_step`.
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

    /// `SearchFir`'s arena migration is STRUCTURAL FIELDS AND CONSTRUCTION
    /// ONLY, per this plan's explicit carve-out (search-execution logic —
    /// `SearchPredicate`/`CandidateNavigator`/`contextful_search_scan` — is
    /// Phase 2's job). This test proves construction and the pure data
    /// accessors round-trip correctly; it does NOT attempt to validate
    /// search correctness, which `fir_op_step`'s own `todo!()` for
    /// `FirSpec::Search` still correctly reflects.
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

    /// `IndexFir`'s arena migration is likewise STRUCTURAL FIELDS AND
    /// CONSTRUCTION ONLY — its real `fir_op_step` resolves `#N`/`^`/`$`
    /// through `BraneNavigator`/`SearchPredicate`/
    /// `contextful_search_scan_no_body_check` (re-read directly,
    /// `fir_kinds.rs`), exactly the machinery this plan's `SearchFir` task
    /// already carves out as Phase 2's job — extended here to `IndexFir` for
    /// the identical reason (a genuine plan gap: the original per-kind list
    /// did not give `IndexFir` the same explicit carve-out `SearchFir` got,
    /// even though its real logic is equally search-engine-dependent).
    /// Index resolution (both branches, re-confirmed by direct re-read)
    /// resolves against the ANCHOR (`foolish_children()[0]`, for the
    /// anchored+contexted case) or the enclosing STATEMENT/BRANE found by
    /// walking the PARENT chain (`find_enclosing_stmt_and_brane`, for the
    /// unanchored case) — never against a sibling directly.
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
}
