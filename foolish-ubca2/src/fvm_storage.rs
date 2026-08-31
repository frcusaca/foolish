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
    /// Mirrors `StatementFir::nf_reason` (FOOP-33 §4). Applies ONLY to
    /// `FirSpec::Statement` nodes — every other kind leaves this `None`
    /// forever, exactly as the real `nf_reason` field exists only on
    /// `StatementFir`'s own struct, not on `ProtoBrane`. Kept as a plain
    /// `ArenaFir` field (not a `FirSpec::Statement` field) for the same
    /// reason `sf_inner_pattern` is excluded from `FirSpec::Search`'s spec:
    /// it is a `fir_op_step`-time discovery, never a construction input (see
    /// `FirSpec::Statement`'s own doc comment, which already says as much).
    /// `None` in the overwhelmingly common case; `Some(reason)` once set is
    /// terminal (never cleared) — set by the null-characterized name constant
    /// rule (`check_null_const_conflict`) or the named-creation no-rename
    /// rule (`check_rename_of_named_creation`), both ported to this cutover's
    /// `StatementFir` migration task.
    nf_reason: Option<String>,
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

    /// Mirrors `Fir::set_contexted` (default no-op, overridden by
    /// `SearchFir`/`IndexFir` in the real trait): sets the `contexted` flag
    /// on a `FirSpec::Search`/`FirSpec::Index` node in place, matching
    /// `Astn::ContextedSearch`'s real construction-time mutation
    /// (`fir.borrow_mut().set_contexted(true)`, re-read directly from
    /// `compiler.rs`) — a no-op for every other kind, exactly as the
    /// trait's default body is for every kind that does not override it.
    pub(crate) fn set_contexted(&mut self, value: bool) {
        match &mut self.spec {
            FirSpec::Search { contexted, .. } | FirSpec::Index { contexted, .. } => {
                *contexted = value;
            }
            _ => {}
        }
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

    /// Mirrors `ProtoBrane::set_alarm_reason`. Called by
    /// `search_fir_dispatch::check_value_pattern_ready` (Phase 2's final
    /// task) — the `#[expect(dead_code)]` this had at the Phase 1
    /// foundational task is removed now that it has a real caller.
    pub(crate) fn set_alarm_reason(&mut self, reason: String) {
        self.alarm_reason = Some(reason);
    }

    /// Mirrors `ProtoBrane::alarm_reason`. Called through
    /// `FVMStorage::alarm_reason` (Phase 3) — the `#[expect(dead_code)]`
    /// this had is removed now that it has a real caller.
    pub(crate) fn alarm_reason(&self) -> Option<&str> {
        self.alarm_reason.as_deref()
    }

    /// Mirrors `StatementFir::nf_reason`'s reader (FOOP-33 §4). `None` unless
    /// this is a `FirSpec::Statement` node that a null-characterized-name
    /// rule has refused. Terminal once set — mirrors the real field's
    /// "set once, never cleared" contract (see this struct's `nf_reason`
    /// field doc comment).
    pub(crate) fn nf_reason(&self) -> Option<&str> {
        self.nf_reason.as_deref()
    }

    /// Mirrors `StatementFir::check_null_const_conflict`/
    /// `check_rename_of_named_creation`'s ONLY write path: `*self.nf_reason
    /// .borrow_mut() = Some(reason)`. Terminal — the real methods both guard
    /// with `if self.nf_reason.borrow().is_some() { return; }` BEFORE calling
    /// this (Gotcha #5a: no re-alarm once resolved); this setter itself does
    /// not re-check, matching `ProtoBrane::set_alarm_reason`'s equally
    /// unconditional real counterpart — the caller owns the "already set"
    /// guard, exactly as it does today. Called through
    /// `FVMStorage::set_nf_reason`, which has a real caller as of Phase 5's
    /// `StatementFir` NF-check port (`search_fir_dispatch::
    /// check_null_const_conflict`).
    pub(crate) fn set_nf_reason(&mut self, reason: String) {
        self.nf_reason = Some(reason);
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

    /// Retrieve this pointer's alarm reason, if any. Mirrors
    /// `ProtoBrane::alarm_reason`. Called by `core_fir_conversion`'s
    /// `Brane` arm (Phase 3) — the `#[expect(dead_code)]` `ArenaFir::
    /// alarm_reason` had is removed now that it has a real caller reached
    /// through this accessor.
    pub fn alarm_reason(&self, ptr: FirPointer) -> Option<&str> {
        let index = self.validate(ptr);
        self.slots[index].payload.alarm_reason()
    }

    /// Retrieve this pointer's NF (Not Foolish) reason, if any (FOOP-33 §4).
    /// Mirrors `StatementFir::nf_reason`'s reader. `None` for every kind
    /// other than `FirSpec::Statement`, and `None` there too unless a
    /// null-characterized-name rule has refused this statement. Consulted by
    /// [`FirPointer::settled_result`] to substitute the refusal NK in place
    /// of the written body — see that method's doc comment.
    pub fn nf_reason(&self, ptr: FirPointer) -> Option<&str> {
        let index = self.validate(ptr);
        self.slots[index].payload.nf_reason()
    }

    /// Sets this pointer's NF reason (FOOP-33 §4). Mirrors
    /// `StatementFir::check_null_const_conflict`/`check_rename_of_named_
    /// creation`'s write path — terminal, but the caller owns the
    /// "already set" guard (see `ArenaFir::set_nf_reason`'s doc comment).
    pub(crate) fn set_nf_reason(&mut self, ptr: FirPointer, reason: String) {
        let index = self.validate(ptr);
        self.slots[index].payload.set_nf_reason(reason);
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
                nf_reason: None,
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
    /// Delegates to [`FirCursor::is_brane_like`] — now that `Brane`'s AND
    /// `ConcatHelper`'s real `stmt_count` overrides are both migrated, this
    /// is the same real judgment, not a separate placeholder (the earlier
    /// duplication between this method and `FirCursor::is_brane_like` is
    /// resolved by unifying on one implementation).
    fn is_brane_like(self, storage: &FVMStorage) -> bool {
        FirCursor::new(self, storage).is_brane_like()
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
    /// None." `pub(crate)` (not private): also called directly by
    /// `search_fir_dispatch::statement_value_for_comparison`, a nested
    /// module.
    pub(crate) fn settled_result(self, storage: &FVMStorage) -> Option<FirPointer> {
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
        step_inner(self, storage, ArenaScope::default(), 0)
    }

    /// Direct arena-threaded translation of `CreationFir::get_display_name`
    /// (re-read in full immediately before writing this). `self` must be a
    /// `FirSpec::Creation` pointer; `viewed_from` is the statement currently
    /// being rendered. The arena's `FirPointer` equality replaces every
    /// `Rc::ptr_eq` in the original one-for-one — a genuinely cleaner
    /// translation than the original's identity-comparison ceremony, since
    /// `FirPointer` is already `PartialEq`.
    ///
    /// Two conditions, both required (FOOP-33, quoted from the original):
    /// (1) reached somewhere OTHER than its own defining statement — a
    /// creation's parent statement is where it was born, and reporting that
    /// same name back there reads as self-referential; (2) the defining
    /// statement's name is null-characterized — only a protected constant
    /// like `'True` qualifies.
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

    /// Direct arena-threaded translation of `FirRefNavExt::find_stmt_index`
    /// (re-read directly, `fir_kinds.rs`): the index of `stmt` among
    /// `self`'s statements, by identity. `self` must be brane-like.
    ///
    /// Naming note: this is a `sift_*`-shaped operation (an ordinary
    /// Rust-side walk, no Foolish search semantics) but keeps its original
    /// name — `find_stmt_index` — for continuity with the method it
    /// translates; a rename is not part of this task's scope.
    pub fn find_stmt_index(self, storage: &FVMStorage, stmt: FirPointer) -> Option<usize> {
        let cursor = FirCursor::new(self, storage);
        let count = cursor.stmt_count()?;
        (0..count).find(|&i| cursor.stmt_at(i) == Some(stmt))
    }

    /// Direct arena-threaded translation of `fir_kinds.rs`'s free function
    /// `find_enclosing_stmt_and_brane` (re-read in full immediately before
    /// writing this): climbs the parent chain from `self` until a
    /// `Statement` kind is found, then returns that statement together with
    /// its home brane. `None` if the climb reaches the structural root
    /// without finding a `Statement` (mirroring the original's `while let
    /// Some(node) = current` loop, which stops at `None`/self-parenting).
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
/// Arena-threaded stand-in for `fir_trait.rs`'s `Scope`, carrying all three
/// fields the real `Scope` does (`current_statement`/`current_brane` — the
/// IB/AB search anchors — plus `has_ancestral_sfm`). Not named `Scope` to
/// avoid colliding with `crate::fir_trait::Scope`, the still-live real type
/// every unmigrated kind's `Rc`-based `fir_op_step` keeps using.
///
/// `has_ancestral_sfm` was deliberately omitted through Phase 1-4 (no
/// arena-migrated kind read it yet — SFF's own SFM threading was exercised
/// only through `clone_subtree`'s `sfm` parameter, not through this scope)
/// and is added here at Phase 5's `IndexFir` cutover, its first real reader
/// (the real `IndexFir::fir_op_step`'s contexted/anchored branches pass
/// `scope.has_ancestral_sfm` straight through to `constanic_clone_at`, i.e.
/// this arena's `clone_stmt_result`/`clone_subtree`).
#[derive(Debug, Clone, Copy, Default)]
struct ArenaScope {
    current_statement: Option<FirPointer>,
    current_brane: Option<FirPointer>,
    has_ancestral_sfm: bool,
}

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
                // Direct translation of the real step_inner's Scope
                // mutation, re-confirmed against fir_trait.rs: set
                // has_ancestral_sfm when `this` is StayFoolish; set
                // current_statement when `this` (the node ABOUT TO RECURSE
                // INTO ITS CHILD, i.e. `ptr` here) is a Statement; set
                // current_brane when `this` is brane-like. All three are set
                // on `ptr`'s own scope before recursing into `front_ptr`.
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

/// Enum-dispatch `fir_op_step` equivalent (rust_instructions.md §7 "Enum
/// dispatch": matching a known, finite variant set and calling concrete
/// logic is preferred over `dyn` here, since `FirSpec`'s variant set is
/// exactly the crate's FIR-kind set). Each per-kind migration task adds its
/// kind's real combining logic here, translated from that kind's real
/// `impl Fir for XFir`'s `fir_op_step` (re-read directly, not reconstructed).
///
/// All 14 `FirSpec` variants (one per real FIR kind, per that enum's own doc
/// comment) now have a real arm — `IndexFir`'s arm (added at Phase 5's
/// cutover prerequisite) was the last. The `match` below is therefore
/// EXHAUSTIVE with no fallback arm: the `todo!()` catch-all this function
/// used through Phases 1-4 was removed once the compiler confirmed
/// exhaustiveness (an `unreachable_patterns` warning on the old fallback arm
/// was the signal), so a future 15th kind added without its own arm now gets
/// a compile error naming the missing variant, matching
/// rust_instructions.md's "implement fully or don't add it" over a
/// silent/panicking catch-all.
fn fir_op_step(ptr: FirPointer, storage: &mut FVMStorage, scope: ArenaScope) {
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
        // exception for the same reason). The two NF-refusal checks
        // (FOOP-33 §4) are now implemented (Phase 5 cutover prerequisite,
        // `search_fir_dispatch::check_null_const_conflict`/
        // `check_rename_of_named_creation`), now that Phase 2's search
        // engine gives `_ib_search`/`_ab_search`/`.value()` arena
        // equivalents to call and the `nf_reason` slot exists.
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
                            // NOTE: the real `check_null_const_conflict` does NOT use
                            // `Scope` at all — it reaches itself via `self_weak` and
                            // calls `_ib_search(&self_rc, pattern)`/`_ab_search(&self_rc,
                            // pattern)` with `self_ref = ptr` (THIS statement). Its
                            // `ib_search_by_pattern` counterpart wants the SEARCHING
                            // STATEMENT (`Some(ptr)` is correct there — it derives the
                            // home brane and index from it). But `ab_search_by_pattern`
                            // wants the STARTING BRANE (mirroring `_ab_search`'s own
                            // `self._get_my_brane(self_ref)` — computed BEFORE the
                            // climb, not the statement itself): passing `Some(ptr)`
                            // there made `current_brane.get_my_statement() ==
                            // current_brane` trivially true (a statement's own
                            // "get_my_statement" is itself), short-circuiting to `None`
                            // immediately — caught by
                            // `statement_null_const_conflict_is_refused` still failing
                            // after the first fix attempt. Fixed by passing `ptr`'s own
                            // home brane instead.
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
        // Direct translation of `impl Fir for FoolRefFir`'s real
        // `fir_op_step` (re-read from `fir_kinds.rs` immediately before
        // writing this): a no-op — `FoolRefFir` is born `Constant` at
        // construction (`push_search_result_pair`, translated below as
        // `push_search_result_pair`) and never needs stepping.
        FirSpec::FoolRef { .. } => {}
        // Direct translation of `impl Fir for StayFoolishFir`'s real
        // `fir_op_step` (re-read from `fir_kinds.rs` immediately before
        // writing this): once its wrapped `expr` is constanic, expose
        // EXPR'S OWN resolved value (its `ubc_children[0]`, or `expr` itself
        // if it has none) as this node's own `ubc_children[0]`, adopting
        // that resolved value's `Nyes` — SF unwraps to a shared VALUE, never
        // producing its own genuinely-new node.
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
        // Direct translation of `impl Fir for StayFullyFoolishFir`'s real
        // `fir_op_step` (re-read from `fir_kinds.rs` immediately before
        // writing this). Same value-unwrap shape as `StayFoolish` above,
        // with two differences preserved exactly: (1) SFF always moves to
        // `Braning` and pushes tasks unconditionally — no empty-children
        // short-circuit (matches the real code: no `if children.is_empty()`
        // branch exists for SFF); (2) the settled `Nyes` is remapped through
        // `SearchFir::nyes_from_found`-equivalent logic (an SFF wrapper
        // "can't be ECONSTANIC" — an Econstanic result means SFF itself is
        // WAITING on it, i.e. Woconstanic, while the pushed result keeps its
        // own Econstanic unchanged).
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
        // Direct translation of `impl Fir for ConcatHelper`'s real
        // `fir_op_step` (re-read from `fir_kinds.rs` immediately before
        // writing this) — IDENTICAL shape to `BraneFir`'s arm above
        // (ConcatHelper is "transparent: inherits all defaults,
        // BraneFir-shaped stepping," per its own doc comment, confirmed by
        // direct re-read). `_search_brane` override is DEFERRED — Phase 2's
        // job, same carve-out as `BraneFir`'s `_ab_search`/`_search_brane`.
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
        // Direct translation of `impl Fir for ConcatenationFir`'s real
        // `fir_op_step` (re-read from `fir_kinds.rs` immediately before
        // writing this) — the TYPE-CHECK and JOIN-READINESS logic (Braning's
        // "one pass over the elements" block) is implemented in full, since
        // it depends only on generic arena primitives already available
        // (`FirPointer::value`, `FirCursor::is_brane_like`). **Deliberately
        // deferred**: `populate_concat_helpers`'s actual line-merging body —
        // specifically `apply_null_const_rule_to_merged_stmt`, which depends
        // on `default_equal`/`set_nf_reason`/`statement_value_for_comparison`,
        // none of which exist in arena form yet (the same NF-mechanism
        // dependency already deferred at `StatementFir`'s task). Once join
        // readiness is confirmed (`all_brane_like` with no type errors),
        // this arena translation settles `Woconstanic` rather than actually
        // building helpers and joining — a narrower, honestly-incomplete
        // claim, not a silent wrong answer: `_helpers_populated` never
        // becomes `true` under this arena path, so `stmt_count`/`stmt_at`
        // (also not yet migrated here) are never called against a
        // half-built helper state.
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
                    let nk_ptr = ptr.create_child(
                        storage,
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

                // JOIN-READY but not yet migrated (see this arm's doc
                // comment above): settle Woconstanic rather than build
                // helpers, an honestly-incomplete claim rather than a wrong
                // Constant/NK answer.
                storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Woconstanic));
            }
            _ => {}
        },
        // Direct translation of `impl Fir for CreationFir`'s real
        // `fir_op_step` (re-read from `fir_kinds.rs` immediately before
        // writing this): a no-op — a creation is born `Independent` at
        // construction and never needs stepping.
        FirSpec::Creation => {}
        // Direct translation of `impl Fir for ComparisonFir`'s real
        // `fir_op_step`/`combine` (`system_foo.rs`, re-read in full
        // immediately before writing this) — SAME two-phase shape as
        // `OperatorFir`: push operands, then combine once Braning.
        // **Deliberately deferred, not implemented**: `combine`'s actual
        // verdict-resolution tail (`resolve_boolean`, which calls
        // `_ab_search` to find `'True`/`'False` in an ancestor brane) —
        // exactly the search-engine dependency Phase 2 owns exclusively,
        // same carve-out as `BraneFir`'s `_ab_search` and `StatementFir`'s
        // NF checks. This arm implements what IS arena-portable now: the
        // two-phase push/combine shape and the ECONSTANIC-if-unevaluated
        // gate (`operand_is_unevaluated_here`, entirely self-contained — no
        // search dependency, only reads a child's own `foolish_children`/
        // `Nyes`). The integer-vs-non-integer type check and the final
        // boolean settle are NOT implemented — once both operands are
        // genuinely evaluated (not the ECONSTANIC-in-system.foo case), this
        // arena translation settles `Woconstanic` rather than resolving a
        // real verdict, an honestly-incomplete result pending Phase 2.
        FirSpec::Comparison { .. } => match storage.get_nyes(ptr) {
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
                } else {
                    // Real verdict resolution needs `_ab_search` — deferred.
                    storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Woconstanic));
                }
            }
            _ => {}
        },
        // `SearchFir`'s dispatch — RESOLVES the `fir_op_step` `todo!()` for
        // this kind (Phase 2's final task). Direct translation of `impl Fir
        // for SearchFir`'s real `fir_op_step`'s own first line: `if
        // self.is_value_search { return self.value_search_step(scope); }`.
        FirSpec::Search {
            is_value_search, ..
        } => {
            if is_value_search {
                search_fir_dispatch::value_search_step(storage, ptr);
            } else {
                search_fir_dispatch::name_search_step(
                    storage,
                    ptr,
                    scope.current_statement,
                    scope.current_brane,
                );
            }
        }
        // `IndexFir`'s dispatch — RESOLVES the previously-open `fir_op_step`
        // gap for this kind (Phase 5 cutover prerequisite, per FOOP-16.plan.md's
        // own call-out: "before deleting IndexFir's old impl Fir block, either
        // complete IndexFir's arena-side search dispatch... or explicitly
        // re-defer"). Direct, section-by-section translation of `impl Fir for
        // IndexFir`'s real `fir_op_step` (re-read from `fir_kinds.rs`
        // immediately before writing this).
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
                        // Real code returns `Err(UbcError::Eval(...))` for a
                        // non-negative unanchored offset — this arena
                        // translation preserves that as a `panic!`, matching
                        // this crate's established convention (confirmed by
                        // re-reading sibling arms above) that `fir_op_step`'s
                        // arena signature has no `Result` to propagate through
                        // yet; an unanchored non-negative `IndexFir` is a
                        // construction-time invariant violation the compiler
                        // itself should never produce (`arena_compiler`'s
                        // `Astn::HeadTail`/`Astn::UnanchoredSeek` arms only ever
                        // build unanchored `Index` nodes with negative offsets),
                        // not a reachable runtime program state.
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
                                    // Anchor still stepping — no progress this call, matching
                                    // the real code's early `return Ok(())` (leaves NYES as-is).
                                } else {
                                    storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Nk));
                                }
                            }
                        }
                    } else if anchored {
                        let anchor = storage.foolish_children(ptr)[0];
                        let resolved = anchor.value(storage);
                        if !FirCursor::new(resolved, storage).is_brane_like() {
                            // FOOP-75 §7 (verbatim rationale re-read from
                            // `fir_kinds.rs`): settling NK is only half the
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
                                let nk_ptr = ptr.create_child(
                                    storage,
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

/// Direct arena-threaded translation of `SearchFir::nyes_from_found`
/// (re-read from `fir_kinds.rs` immediately before writing this — used by
/// `StayFullyFoolish`'s settle-remapping and, once migrated, by real search
/// dispatch). Preserves the exact mapping: Econstanic/Woconstanic →
/// Woconstanic; Constant/Independent → Constant; Nk → Nk; anything else
/// (pre-constanic) passes through unchanged.
fn nyes_from_found(found: Nyes) -> Nyes {
    match found {
        Nyes::Econstanic | Nyes::Woconstanic => Nyes::Woconstanic,
        Nyes::Constant | Nyes::Independent => Nyes::Constant,
        Nyes::Nk => Nyes::Nk,
        other => other,
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

    /// Mirrors [`crate::fir_trait::Fir::as_op_name`]: `Operator`'s AND
    /// `Comparison`'s own overrides (re-read directly — `Comparison`
    /// returns `self.op.searchable_name()`, a `&'static str`, trivially
    /// compatible with the `'s` lifetime bound here); every other kind's
    /// default is `None`.
    pub fn as_op_name(&self) -> Option<&'s str> {
        match self.node() {
            FirSpec::Operator { op } => Some(op),
            FirSpec::Comparison { op } => Some(op.searchable_name()),
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

    /// Mirrors [`crate::fir_trait::Fir::stmt_count`]: `Brane`'s AND
    /// `ConcatHelper`'s own overrides (re-read directly — `ConcatHelper` is
    /// "transparent: inherits all defaults, BraneFir-shaped stepping," per
    /// its own doc comment, and its `stmt_count`/`stmt_at` overrides are
    /// byte-for-byte identical to `BraneFir`'s) report the foolish-children
    /// count; `Concatenation`'s own real override is more involved (helper
    /// population) and NOT yet migrated (see that kind's `fir_op_step` arm);
    /// every other kind's default is `None` (not brane-like).
    pub fn stmt_count(&self) -> Option<usize> {
        match self.node() {
            FirSpec::Brane { .. } | FirSpec::ConcatHelper => Some(self.foolish_children().len()),
            _ => None,
        }
    }

    /// Mirrors [`crate::fir_trait::Fir::stmt_at`]: `Brane`'s AND
    /// `ConcatHelper`'s own overrides (re-read directly, identical shape)
    /// index their foolish children directly.
    pub fn stmt_at(&self, idx: usize) -> Option<FirPointer> {
        match self.node() {
            FirSpec::Brane { .. } | FirSpec::ConcatHelper => {
                self.foolish_children().get(idx).copied()
            }
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

    /// Mirrors [`crate::fir_trait::Fir::as_fool_ref_referent`]: `FoolRef`'s
    /// own override (re-read directly) returns the original found statement
    /// this reference wraps.
    pub fn as_fool_ref_referent(&self) -> Option<FirPointer> {
        match self.node() {
            FirSpec::FoolRef { referent } => Some(*referent),
            _ => None,
        }
    }

    /// Mirrors [`crate::fir_trait::Fir::as_concat_provenance`]:
    /// `Concatenation`'s own override (re-read directly) returns its
    /// provenance; every other kind's default is `Juxtaposition`.
    pub fn as_concat_provenance(&self) -> crate::fir_kinds::ConcatProvenance {
        match self.node() {
            FirSpec::Concatenation { provenance } => *provenance,
            _ => crate::fir_kinds::ConcatProvenance::Juxtaposition,
        }
    }

    /// Mirrors [`crate::fir_trait::Fir::as_creation_display_name`]:
    /// `Creation`'s own override (re-read directly) delegates to
    /// [`FirPointer::get_display_name`]; every other kind's default is
    /// `None`.
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
    /// Called by `search_fir_dispatch::handle_found` (Phase 2's final
    /// task), the first real `fir_op_step`-adjacent code to hold a
    /// `FirCursorMut` and call this — the `#[expect(dead_code)]` this had
    /// at the foundational task is removed now that it has a real caller.
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

    /// Direct arena-threaded translation of the free function
    /// `push_search_result_pair` (re-read from `fir_kinds.rs` immediately
    /// before writing this): pushes a search RESULT and its `FoolRef`
    /// bookkeeping entry to `ubc_children`, in that order. Preserves the
    /// FoolRefFir TWO-CHILD INVARIANT exactly — after this call,
    /// `ubc_children` holds `[result, fool_ref]`; `[0]` is the value every
    /// existing reader accesses via `.first()`, `[1]` is invisible to them.
    /// `referent` is the ORIGINAL found statement (not the cloned result) —
    /// a genuinely shared `FirPointer`, exactly as `FoolRefFir::referent`
    /// today shares the original `Rc`, not a clone of it (confirmed by
    /// `clone_subtree`'s own `FoolRef`-always-shares rule, which this
    /// invariant depends on staying true).
    pub fn push_search_result_pair(&mut self, result: FirPointer, referent: FirPointer) {
        let fool_ref = self.create_child(FirSpec::FoolRef { referent });
        self.storage
            .with_mut(fool_ref, |fir| fir.set_nyes(Nyes::Constant));
        self.push_search_result(result);
        self.push_ubc_child(fool_ref);
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
    /// 2. **`StayFoolish`/`StayFullyFoolish` unwrapping** — RESOLVED as of
    ///    the `StayFoolishFir`/`StayFullyFoolishFir` per-kind migration task
    ///    (the placeholder this doc comment used to describe, deferred from
    ///    the Phase 1 foundational task, is now closed out): checked FIRST,
    ///    before the share-not-clone check, exactly matching
    ///    `constanic_clone_at`'s own real order. `StayFoolish` tries its
    ///    settled `ubc_children[0]` first; either kind falls through to its
    ///    first `foolish_children` entry; if both are empty, an `eprintln!`
    ///    ALARM fires (matching the original) and the wrapper clones as-is.
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
        // 2. StayFoolish/StayFullyFoolish unwrapping — checked FIRST, before
        // the share-not-clone check below (re-confirmed by direct re-read:
        // `constanic_clone_at`'s SF/SFF branch is the very first thing the
        // real function does). Only `StayFoolish` (not `StayFullyFoolish`)
        // tries its settled `ubc_children[0]` first; either kind falls
        // through to its first `foolish_children` entry; if BOTH are empty,
        // the real function logs an ALARM and falls through to clone the
        // wrapper as-is via the normal share/rebuild logic below — this
        // arena translation does the same (`eprintln!`, matching the
        // original's own diagnostic, not a panic).
        let spec = self.get(root).clone();
        if matches!(spec, FirSpec::StayFoolish | FirSpec::StayFullyFoolish) {
            if matches!(spec, FirSpec::StayFoolish)
                && let Some(result) = FirCursor::new(root, self).ubc_children().first().copied()
            {
                return self.clone_subtree(result, new_parent, index, sfm, skip_foolish_children);
            }
            if let Some(inner) = self.foolish_children(root).first().copied() {
                return self.clone_subtree(inner, new_parent, index, sfm, skip_foolish_children);
            }
            eprintln!("ALARM: SF/SFF node has no children — cloning wrapper as-is");
        }

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

/// Direct arena-threaded translation of `fir_kinds.rs`'s real
/// `default_equal` (re-read in full immediately before writing this).
/// Preserves the exact branch order: NK-on-either-side → Unknowable;
/// both-integers → compare; else resolve `.value()` and compare kind
/// (`Creation`-vs-`Creation` → pointer identity; `Brane`-vs-`Brane` →
/// Unknowable; anything else → NotEqual).
///
/// Kind discrimination is done directly on [`FirSpec`] rather than through a
/// `kind()`-style accessor — `FirSpec` already carries the same information
/// `FirKind` would, so no duplicate enum is needed under the arena.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Equality {
    Equal,
    NotEqual,
    Unknowable,
}

/// Called by `search_engine::SearchPredicate::matches`'s `Value`/`NameValue`
/// arms, which are in turn called by `search_fir_dispatch` (Phase 2's final
/// task) — genuinely reachable from production code now, so the
/// `cfg_attr(not(test), expect(dead_code))` this had is removed.
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

/// Direct arena-threaded translation of `fir_kinds.rs`'s `mod
/// contextful_search` (re-read in full immediately before writing every
/// item below — this is Phase 2 of FOOP-16, the highest silent-regression
/// risk in the entire FOOP; every type/function here is a literal,
/// line-by-line translation of the real module, not a redesign).
///
/// `SearchFir`'s own dispatch task (Phase 2's final task, `mod
/// search_fir_dispatch` below) wires this module into `fir_op_step`'s live
/// dispatch, so every item here is now genuinely reachable from production
/// code — the `cfg_attr(not(test), expect(dead_code))` this module had
/// while unwired is removed.
pub(crate) mod search_engine {
    use super::{Equality, FVMStorage, FirCursor, FirPointer, default_equal};

    use foolish_core::fir::Nyes;

    /// Where the Navigator starts scanning from. Mirrors the real
    /// `CursorSource` verbatim (not yet wired to anything — the FOOP's
    /// `CursorSource::Contextless`/`Contexted` distinction governs how a
    /// `Navigator` is CONSTRUCTED, which is `SearchFir`'s own dispatch
    /// logic's job, migrated in the next Phase 2 task).
    #[expect(
        dead_code,
        reason = "not yet constructed anywhere, including this file's own tests — SearchFir's \
                  dispatch task (the next Phase 2 task) is what decides Contextless vs. \
                  Contexted and constructs a Navigator accordingly"
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

    /// Result of the core scan loop. `Found` carries a genuinely comparable
    /// `FirPointer` (already `Eq`), so unlike the original's hand-written
    /// `Rc::ptr_eq`-based `PartialEq`, this can `#[derive]` directly.
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
    ///
    /// Variant set UNCHANGED from the real `SearchPredicate` (per this
    /// plan's own instruction: "this task is a signature/access-pattern
    /// migration only, not a semantic change"): `Name`, `Value`,
    /// `NameValue`, `Index`, `Head`, `Tail`.
    #[derive(Debug)]
    pub(crate) enum SearchPredicate {
        /// Name-match: `?name` / `~name` / `.name`. Reads the candidate's name.
        Name { pattern: String },
        /// Value-match: `?=v` / `~=v`. Reads the candidate's body integer value.
        Value { pattern: FirPointer },
        /// Atomic name+value: `?name=v` / `~name=v`. Both gates on the same candidate.
        NameValue { name: String, value: FirPointer },
        /// Positional index: `#N`. Reads the candidate's position in the scan.
        ///
        /// `#N` positional index — `IndexFir`'s real search predicate.
        /// Constructed by production code as of Phase 5's cutover
        /// (`IndexFir`'s own `fir_op_step` arm in this file, added to
        /// resolve the previously-open gap this variant's doc comment used
        /// to describe: "migrated STRUCTURAL-FIELDS-ONLY in Phase 1... no
        /// Phase 2+ task anywhere in this plan gives `IndexFir` its real
        /// search dispatch"). No `#[expect(dead_code)]` needed any more —
        /// it has a real, non-test caller now.
        Index(i32),
        /// First position: `^`. Matches when position == 0.
        ///
        /// **Still not constructed by production code** (unlike `Index`
        /// above): the real `IndexFir::fir_op_step` (re-read directly)
        /// dispatches `^`/`$` head/tail through `Astn::HeadTail`'s
        /// `is_head`/offset-based construction, which resolves to an
        /// ordinary `Index` predicate with `offset = 0` (head) or a
        /// tail-relative negative offset — NOT through this `Head`/`Tail`
        /// predicate variant at all (confirmed by re-reading `IndexFir`'s
        /// full real dispatch: it only ever constructs
        /// `SearchPredicate::Index(..)`, never `Head`/`Tail`). This is a
        /// pre-existing, non-blocking observation carried forward from
        /// Phase 2 — `Head`/`Tail` remain exercised only by this file's own
        /// tests, hence `cfg_attr(not(test), ...)` rather than a bare
        /// `#[expect]`.
        #[cfg_attr(
            not(test),
            expect(
                dead_code,
                reason = "IndexFir's own dispatch is unmigrated — no production caller builds this variant yet"
            )
        )]
        Head,
        /// Last position: `$`. Matches when position == total - 1. Same
        /// not-yet-constructed-by-production status as `Index` above.
        #[cfg_attr(
            not(test),
            expect(
                dead_code,
                reason = "IndexFir's own dispatch is unmigrated — no production caller builds this variant yet"
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
        /// Apply this predicate to a candidate statement. Direct
        /// arena-threaded translation of the real `SearchPredicate::matches`
        /// — same match arms, same order, `&FVMStorage` reads standing in
        /// for `.borrow()`.
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
                    if !crate::fir_kinds::SearchFir::matches_pattern(&name, pattern) {
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
                    if !crate::fir_kinds::SearchFir::matches_pattern(&stmt_name, name) {
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

        /// Like [`Self::matches`] but skips the body-NYES gate. Direct
        /// translation of the real `matches_no_body_check`.
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
                    if !crate::fir_kinds::SearchFir::matches_pattern(&name, pattern) {
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
    /// Direct translation of the real `check_body_nyes`: pre-constanic →
    /// unreachable (the real function's own `unreachable!`, preserved
    /// verbatim — a pre-constanic body reaching this point is an internal
    /// consistency violation, not a legitimate outcome). NK → NkStop.
    /// Otherwise → Approve.
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
    /// brane_position). Direct translation of the real `CandidateNavigator`
    /// trait — same two methods, same correctness contract:
    ///
    /// 1. **Correctly ordered** — the one mandated order.
    /// 2. **Complete** — every reachable candidate, exactly once, then stops.
    pub(crate) trait CandidateNavigator {
        /// Yield the next candidate as (statement `FirPointer`, 0-based brane position).
        fn next_candidate(&mut self) -> Option<(FirPointer, usize)>;
        /// Total number of candidates in the source.
        fn total(&self) -> usize;
    }

    /// Iterates a brane's statements in order (forward or backward). Direct
    /// arena-threaded translation of the real `BraneNavigator`: the
    /// **ordering contract is preserved exactly** — the arena's
    /// `Vec`-backed `foolish_children` is walked in the identical order
    /// today's `Vec<FirRef>` iteration produces, forward or backward, since
    /// both read the SAME underlying construction-order `Vec` (arena
    /// `foolish_children` IS the ordered child list, same as
    /// `ProtoBrane::foolish_children` was).
    #[derive(Debug)]
    pub(crate) struct BraneNavigator {
        children: Vec<FirPointer>,
        pos: usize,
        forward: bool,
        done: bool,
    }

    impl BraneNavigator {
        /// Direct translation of the real `BraneNavigator::new`: reads
        /// `stmt_count()`/`stmt_at()` (the brane-like capability accessors,
        /// already migrated onto `FirCursor` for `Brane`/`ConcatHelper` —
        /// `Concatenation`'s own `stmt_count`/`stmt_at` overrides remain
        /// unmigrated per Phase 1's documented deferral, so a
        /// `BraneNavigator` built over an unmerged `Concatenation` sees 0
        /// candidates, matching that kind's current honest-incomplete
        /// state rather than panicking).
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

        /// Direct translation of the real `BraneNavigator::set_range`.
        /// Called by `search_fir_dispatch`'s contexted-search and
        /// value-search bounded scans (Phase 2's final task) — the
        /// `#[expect(dead_code)]` this had is removed now that it has real
        /// callers.
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
        /// Direct translation of the real `next_candidate` — identical
        /// cursor-advance logic (forward: increment, done at end; backward:
        /// decrement, done at zero).
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

    /// The core scan loop of the ContextfulSearch engine. Direct translation
    /// of the real `contextful_search_scan` — same two shared rules, same
    /// order, same `Miss`/`Found`/`NkStop` outcomes:
    ///
    /// - **Wait-on-nye**: not applicable here (arena candidates cannot be
    ///   pre-constanic per `check_body_nyes`'s own `unreachable!`, matching
    ///   the original exactly).
    /// - **NK-stop**: if a candidate's predicate returns `NkStop`, the scan
    ///   halts (the search itself becomes NK).
    ///
    /// Returns `Miss` when all candidates are exhausted with no match. The
    /// caller decides the settlement: anchored → NK, unanchored → ECONSTANIC.
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
    /// [`SearchPredicate::matches_no_body_check`]. Direct translation of the
    /// real `contextful_search_scan_no_body_check` — for contextless
    /// searches (`IndexFir`, `SearchFir` name search) where body settling is
    /// the caller's responsibility.
    ///
    /// This task's own re-verification (per the plan: "confirm after the
    /// previous two tasks that this loop needs no further change beyond
    /// what flows through from `CandidateNavigator`'s and `SearchPredicate`'s
    /// own migrations") confirms exactly that: both scan functions needed
    /// ONLY signature changes (`&FVMStorage` threaded through, `FirPointer`
    /// replacing `FirRef`) — no logic in either loop itself changed at all,
    /// matching the plan's own prediction that this could turn out to be a
    /// re-verification task rather than a code-change task.
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

/// `SearchFir`'s own predicate-building and dispatch logic, arena-threaded.
/// Direct translation of `impl SearchFir`'s and `impl Fir for SearchFir`'s
/// real methods (`fir_kinds.rs`, re-read in full immediately before writing
/// every function below — this is where Phase 1's deferred "search-execution
/// logic" is migrated, wiring `search_engine` into the crate's live
/// evaluation path for the first time). Kept as free functions taking
/// `FirPointer` + `&mut FVMStorage` explicitly, matching this module's
/// established `fir_op_step`/`combine` convention, rather than methods on
/// `FirPointer` itself — these are `SearchFir`-specific, not generic
/// arena operations every kind needs.
mod search_fir_dispatch {
    use super::search_engine::{
        BraneNavigator, ScanOutcome, SearchPredicate, contextful_search_scan,
        contextful_search_scan_no_body_check,
    };
    use super::{FVMStorage, FirCursor, FirPointer, FirSpec};

    use foolish_core::fir::Nyes;

    /// Direct translation of `SearchFir::nyes_from_found` (re-read
    /// directly) — same mapping as the free `nyes_from_found` this module
    /// already defines for `StayFullyFoolish` (both translate the SAME real
    /// function; kept as one shared arena function rather than duplicated,
    /// since the original crate ALSO has both call the same
    /// `SearchFir::nyes_from_found`).
    fn nyes_from_found(found: Nyes) -> Nyes {
        super::nyes_from_found(found)
    }

    /// Direct translation of `SearchFir::clone_stmt_result`, now including
    /// the NF-substitution path (`StatementFir::settled_result`'s override,
    /// FOOP-33 §4): mirrors [`crate::fir_kinds::statement_value_for_comparison`]'s
    /// "`settled_result()`, else the raw written body" contract exactly —
    /// a refused statement (`nf_reason` set) presents a fresh, already-`Nk`
    /// node INSTEAD of cloning its written RHS. Built directly at `Nyes::Nk`
    /// (not via the general `FirSpec::Nk` + step convention, which starts
    /// `Prembrionic`) because `settled_result`'s own contract — "applies the
    /// constanic gate itself" — demands the presented value already BE
    /// constanic; the real override's doc comment makes the same point about
    /// why it does not go through `NkFir::nk`.
    pub(super) fn clone_stmt_result(
        storage: &mut FVMStorage,
        stmt: FirPointer,
        new_parent: FirPointer,
        sfm: bool,
    ) -> FirPointer {
        if let Some(reason) = storage.nf_reason(stmt) {
            let reason = reason.to_owned();
            let nk = new_parent.create_child(storage, FirSpec::Nk { reason });
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
        storage.clone_subtree(body, new_parent, index, sfm, false)
    }

    /// Direct translation of `SearchFir::handle_found` (re-read directly):
    /// clones the found statement's value under `self`, pairs it with a
    /// `FoolRefFir` wrapping the ORIGINAL statement (the two-child
    /// invariant, already verified in Phase 1's `FoolRefFir` task), and
    /// moves to `Braning`.
    fn handle_found(storage: &mut FVMStorage, ptr: FirPointer, stmt: FirPointer, sfm: bool) {
        let clone = clone_stmt_result(storage, stmt, ptr, sfm);
        let mut cursor = super::FirCursorMut::new(ptr, storage);
        cursor.push_search_result_pair(clone, stmt);
        cursor.set_nyes(Nyes::Braning);
    }

    /// Direct translation of `SearchFir::settle_from_ubc_result` (re-read
    /// directly): once a result is pushed, adopt its (remapped) NYES.
    /// Direct arena translation of `fir_kinds.rs`'s free function
    /// `statement_value_for_comparison` (re-read directly): the value a
    /// statement PRESENTS — its `settled_result()` (the NF-refusal NK, if
    /// already refused) if set, else the raw written body. Used by the two
    /// NF-refusal checks below, which must compare against what a PRIOR
    /// statement already presents, not its raw RHS (poisoning must be
    /// transitive per FOOP-33 §4).
    pub(super) fn statement_value_for_comparison(
        storage: &FVMStorage,
        stmt: FirPointer,
    ) -> Option<FirPointer> {
        stmt.settled_result(storage)
            .or_else(|| storage.foolish_children(stmt).first().copied())
    }

    /// Direct arena translation of `StatementFir::check_null_const_conflict`
    /// (FOOP-33 §4, re-read in full immediately before writing this): a
    /// null-characterized statement checks ITSELF, once its body is
    /// constanic, against any EARLIER same-name null-characterized statement
    /// (IB, then AB) — refusing (`set_nf_reason`) if the two values are not
    /// `Equal`. Terminal: does nothing if `nf_reason` is already set
    /// (Gotcha #5a, no re-alarm).
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
            storage.set_nf_reason(stmt, format!("'{name} not-foolish"));
        }
    }

    /// Direct arena translation of `StatementFir::check_rename_of_named_
    /// creation` (FOOP-33, re-read in full immediately before writing this):
    /// a null-characterized statement whose constanic value resolves to a
    /// creation with a DIFFERENT original name is refused — "named creations
    /// cannot be renamed." Terminal, same guard as
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
            storage.set_nf_reason(
                stmt,
                format!("'{name} not-foolish (Named creations cannot be renamed)"),
            );
        }
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

    /// The offset of `self`'s value operand among its foolish children —
    /// direct translation of `SearchFir::value_child`'s indexing rule
    /// (`1` if anchored — the anchor occupies `[0]` — else `0`).
    fn value_child(storage: &FVMStorage, ptr: FirPointer) -> FirPointer {
        let anchored = matches!(storage.get(ptr), FirSpec::Search { anchored: true, .. });
        let idx = if anchored { 1 } else { 0 };
        storage.foolish_children(ptr)[idx]
    }

    /// Direct translation of `SearchFir::ib_search_with_engine` (re-read
    /// directly): an immediate-brane name search, scanning backward from
    /// (but excluding) the current statement's own position. `checked_sub`
    /// (not `saturating_sub`) preserves the index-0 self-hit guard exactly
    /// — a statement at position 0 has no preceding range at all.
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
    /// by `StatementFir`'s NF-refusal checks (FOOP-33 §4,
    /// `check_null_const_conflict`), which search by the STATEMENT's own
    /// `searchable_name()`, not by a `Search` node's pattern (the statement
    /// itself is not a `Search`). `ib_search_with_engine` above is now a
    /// thin wrapper over this, preserving its exact original behavior for
    /// its existing callers.
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

    /// Direct translation of `SearchFir::ab_search_with_engine` (re-read
    /// directly): an ancestral-brane name search, climbing outward one
    /// brane at a time, scanning each ancestor's statements strictly
    /// BEFORE the position the climb entered it from.
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

    /// Direct translation of `SearchFir::contexted_search_from_anchor`
    /// (re-read in full immediately before writing this): reads the
    /// anchor's `FoolRefFir` bookkeeping entry (`ubc_children[1]`, per the
    /// two-child invariant), resolves ITS referent's home brane and
    /// position, then scans a range strictly AFTER (forward) or BEFORE
    /// (backward) that position within the SAME home brane — never
    /// crossing out of it (the contexted-search "never leaves the home
    /// brane" rule, AGENTS.md §Searches).
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

    /// Direct translation of `impl Fir for SearchFir`'s real `fir_op_step`'s
    /// NAME-SEARCH path (the `is_value_search` branch is
    /// [`value_search_step`] below — dispatched the same way the real
    /// `fir_op_step`'s very first line does: `if self.is_value_search {
    /// return self.value_search_step(scope); }`).
    pub(crate) fn name_search_step(
        storage: &mut FVMStorage,
        ptr: FirPointer,
        current_statement: Option<FirPointer>,
        current_brane: Option<FirPointer>,
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
                            handle_found(storage, ptr, stmt, false);
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
                        Some((stmt, _nyes)) => handle_found(storage, ptr, stmt, false),
                        // `anchored` is always true in this branch, so the
                        // real fir_op_step's `if self.anchored { Nk } else
                        // { Econstanic }` always takes the Nk arm here.
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
                                handle_found(storage, ptr, stmt, false);
                            }
                            _ => storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Nk)),
                        }
                    }
                } else {
                    match ab_search_with_engine(storage, ptr, current_brane) {
                        Some((stmt, _nyes)) => handle_found(storage, ptr, stmt, false),
                        None => storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Econstanic)),
                    }
                }
            }
            _ => {}
        }
    }

    /// Direct translation of `SearchFir::build_value_predicate` (re-read
    /// directly): builds a `Value` predicate if the pattern is empty
    /// (`?=`/`~=`), else a `NameValue` predicate (`?name=v`/`~name=v`).
    /// `None` if the value operand is not yet constanic — the caller
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

    /// Direct translation of `SearchFir::check_value_pattern_ready` (re-read
    /// directly): gates the value-search dispatch on the value operand's
    /// own NYES, preserving the exact branch-by-branch NYES propagation
    /// rules (FOOP-23 rendering appendix, quoted in the real source):
    /// pre-constanic → push as task, not ready; NK → Nk; WOCONSTANIC →
    /// inherit Woconstanic (waiting on constanics, not a miss); ECONSTANIC →
    /// inherit Econstanic; else confirm the resolved value is either an
    /// integer or a creation (the two comparable value kinds), else Nk with
    /// the exact alarm-reason string.
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

    /// Direct translation of `SearchFir::value_search_step` (re-read in
    /// full immediately before writing this) — the value-search dispatch
    /// (`?=`/`~=`/`?name=v`/`~name=v`), a distinct three-phase shape from
    /// [`name_search_step`]'s two phases: `Prembrionic` pushes BOTH the
    /// anchor (if anchored) and the value operand as tasks together (unlike
    /// name-search, which pushes only the anchor); `Embryonic` (unanchored
    /// only — anchored searches skip straight to `Braning`) does the
    /// IB-equivalent backward scan bounded to the enclosing statement's own
    /// position; `Braning` does the contexted/anchored/unanchored (AB-style)
    /// dispatch, mirroring `name_search_step`'s `Braning` arm shape closely
    /// but scanning with the value predicate instead of a name predicate.
    pub(crate) fn value_search_step(storage: &mut FVMStorage, ptr: FirPointer) {
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
                            handle_found(storage, ptr, stmt, false);
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
                            handle_found(storage, ptr, stmt, false);
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
                        handle_found(storage, ptr, stmt, false);
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

/// Arena-threaded translation of `evaluator.rs`'s stepping loop and
/// FIR→core-FIR output-serialization family (Phase 3 of FOOP-16). Re-read
/// `evaluator.rs` in full (1246 lines) immediately before writing every
/// function below.
///
/// # Free functions, not methods — resolving `evaluator.rs`'s own `@Agents`
/// embedded comment
///
/// `evaluator.rs`'s real `proto_to_core_fir_sff_body` carries a parenthetical
/// `(@Agents, I suppose this can't be declared as implementation on
/// something associated with SFF marker like SFFMark? ...)` asking whether
/// these conversion functions could be methods rather than free functions.
/// Resolved here rather than left standing: NO — these functions dispatch
/// across EVERY `FirSpec` variant (`match kind { FirKind::Search => ...,
/// FirKind::Operator => ..., ... }`), not just SF/SFF, so there is no single
/// "SFFMark"-shaped type to attach them to; the "whose method is it"
/// question (Rule Zero, rust_instructions.md) has no single answer for a
/// function that legitimately needs to read every kind's own state. This
/// exactly mirrors why `fir_op_step`, `combine`, and every
/// `search_fir_dispatch` function in this same module are ALSO free
/// functions taking `FirPointer` explicitly rather than methods — established
/// convention, not an oversight, and this module's own use of the SAME shape
/// for the SAME reason is the concrete answer to the embedded question.
///
/// No production caller yet — `UbcaEvaluator::evaluate` (the crate's real
/// entry point) is not rewired to call into this module; that requires
/// `system_foo`/`compiler.rs` to be arena-aware first (Phase 4's job).
/// Exercised only by this file's own `#[cfg(test)]` tests, hence
/// `cfg_attr(not(test), expect(dead_code))` rather than a bare `#[expect]` —
/// same pattern as `search_engine`'s own module-level guard at the
/// equivalent point in Phase 2.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "no production caller yet — UbcaEvaluator::evaluate is rewired once Phase 4 \
                  makes system_foo/compiler.rs arena-aware"
    )
)]
mod core_fir_conversion {
    use super::{FVMStorage, FirCursor, FirPointer, FirSpec, MAX_DEPTH};

    use foolish_core::fir as core_fir;
    use foolish_core::fir::{
        Alarm, AlarmLevel, AlarmSource, ConcatenationFirBuilder, ConstantIntFirBuilder,
        CreationFirBuilder, FirQueryable, IndexFirBuilder, NkFirBuilder, NormalBraneFirBuilder,
        Nyes, OperatorFirBuilder, SearchFirBuilder, StayFoolishFirBuilder,
        StayFullyFoolishFirBuilder,
    };

    /// Direct arena-threaded translation of `evaluator.rs`'s real
    /// `step_to_settled` (re-read immediately before writing this): steps
    /// `ptr` up to `MAX_STEPS` times, returning `Ok(())` once constanic, or
    /// an `Eval` error naming the iteration count if the step budget is
    /// exhausted first. `MAX_STEPS` here reuses `search_fir_dispatch`'s
    /// module-level `MAX_DEPTH` guard's SIBLING concept at the top level —
    /// re-read directly: the real `step_to_settled` uses its own
    /// `MAX_STEPS = 10_000` constant, distinct from `step_inner`'s
    /// `MAX_DEPTH = 100` recursion guard (one caps total top-level
    /// iterations, the other caps recursion depth within one iteration) —
    /// so this defines its OWN `MAX_STEPS`, not a reuse of `MAX_DEPTH`.
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

    /// Direct arena-threaded translation of `evaluator.rs`'s real
    /// `step_until` (re-read immediately before writing this): the UBCA
    /// debugger-breakpoint equivalent — steps until `matcher` accepts the
    /// front task (or `None` when there is no front task), returning the
    /// step count, or an error if the FVM settles first or the step budget
    /// is exhausted. Reuses [`MAX_STEPS`] — the real function's own
    /// `MAX_STEPS` constant, re-confirmed to be the SAME `10_000` value as
    /// `step_to_settled`'s (both defined identically in `evaluator.rs`, one
    /// module-level constant there; kept as one shared constant here too,
    /// not duplicated).
    ///
    /// **Deliberate signature deviation**: the real `step_until`'s matcher
    /// is `FnMut(Option<&FirRef>) -> bool` — a `FirRef` can be `.borrow()`'d
    /// directly by the closure with no extra parameter. A bare `FirPointer`
    /// carries no data on its own, so this arena translation's matcher takes
    /// `&FVMStorage` explicitly alongside `Option<FirPointer>`
    /// (`FnMut(&FVMStorage, Option<FirPointer>) -> bool`) — the same
    /// necessary adaptation `SearchPredicate::matches` already made for the
    /// identical reason in Phase 2.
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

    /// Direct translation of `evaluator.rs`'s real `step_until_line_number`.
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

    /// Direct translation of `evaluator.rs`'s real `step_until_statement_name`.
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

    /// Direct translation of `evaluator.rs`'s real `display_stmt_name`
    /// (re-read directly): an anonymous statement (`compiler::ANON_STMT_NAME`,
    /// or any empty name) renders with no `name=` prefix.
    fn display_stmt_name(name: Option<&str>) -> Option<String> {
        match name {
            Some(n) if n.is_empty() || n == crate::compiler::ANON_STMT_NAME => None,
            Some(n) => Some(n.to_string()),
            None => None,
        }
    }

    /// Direct translation of `evaluator.rs`'s real `proto_to_core_fir`.
    pub(crate) fn proto_to_core_fir(storage: &FVMStorage, ptr: FirPointer) -> core_fir::Fir {
        proto_to_core_fir_inner(storage, ptr, false, None, 0)
    }

    /// Direct translation of `evaluator.rs`'s real `proto_to_core_fir_sff_body`
    /// (re-read in full immediately before writing this): top-level searches
    /// get EMBRYONIC state; operator operands get CONSTANT state; operators
    /// get WOCONSTANIC/CONSTANT based on operand states. See `current_stmt`'s
    /// doc comment on the real function, preserved verbatim in spirit: the
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

    /// Direct translation of `evaluator.rs`'s real `proto_to_core_fir_sff_operand`.
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

    /// Direct, line-by-line translation of `evaluator.rs`'s real
    /// `proto_to_core_fir_inner` (re-read in full immediately before writing
    /// this — the direct producer of every einmo OUTPUT line, per this
    /// phase's own framing of why this function family is worth its own
    /// task). Every match arm preserves the original's exact logic,
    /// including the Search/Index/StayFoolish arms' deeply nested
    /// unwrap-vs-preserve-wrapper decisions — none simplified, even where a
    /// shorter form seemed tempting, per this task's own "direct
    /// translation, not a redesign" discipline (matching Phase 2's).
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
            // own comment (re-read directly, preserved verbatim in spirit).
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
                // Deferred NF-substitution: `statement_value_for_comparison`'s
                // settled_result-first preference is not yet arena-portable
                // (StatementFir's Phase 1 task deferred it) — this arena
                // translation reads the raw written body directly, matching
                // that method's own fallback path for the common case.
                let body_fir = cursor
                    .foolish_children()
                    .first()
                    .map(|&c| {
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
                    // case rather than resolving to the final value —
                    // re-read directly, translated verbatim, not
                    // simplified.
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
                            // `sf_inner_pattern`'s `Some` branch (re-read
                            // directly, `as_sf_inner_pattern`) is NOT
                            // reachable here: `FirSpec::Search` carries no
                            // `sf_inner_pattern` field — it starts `None`
                            // always in the arena model (per `FirSpec::
                            // Search`'s own doc comment), and no
                            // arena-migrated code path sets it yet (setting
                            // it is part of the still-unmigrated
                            // SF-unwrap-via-search machinery). An honestly
                            // incomplete gap, not a silent wrong answer:
                            // this only affects rendering a search that
                            // itself resolved through an SF wrapper's own
                            // pattern substitution, which no test in this
                            // crate's suite yet exercises through the arena
                            // path (Phase 3 does not wire this into the
                            // live evaluator — see this module's own doc
                            // comment).
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
                let is_tail = provenance == crate::fir_kinds::ConcatProvenance::TailConcatenation;
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

/// Arena-threaded translation of `compiler.rs`'s AST→FIR construction (Phase
/// 4 of FOOP-16). Re-read `compiler.rs` in full (732 lines) immediately
/// before writing every function below — confirmed exactly 18
/// `Rc::new_cyclic` sites, matching the plan's own count, clustered exactly
/// as the plan describes: 12 in `build_fir`'s per-`Astn`-variant match arms,
/// 1 in `build_concat_element`, 2 in the statement-construction path
/// (`build_as_statement_inner`, `compile_root_with_body_override`), 1 in
/// `build_expr_with_operator`, 1 in `compile_root_with_body_override`'s own
/// root, 1 in the test module's `root_parent` helper (not translated — test
/// scaffolding, not production code).
///
/// No production caller yet: `Compiler::compile` itself is not rewired to
/// call into this module — doing so would require the ENTIRE downstream
/// crate (evaluator, system_foo) to consume `FirPointer` trees instead of
/// `FirRef` trees, which is Phase 5's coordinated cutover, not this task's.
/// This module is a complete, standalone, PARALLEL arena compiler,
/// exercised by its own tests, proven correct in isolation — exactly
/// mirroring how `core_fir_conversion`/`search_fir_dispatch` were each
/// additive and un-wired until their own cutover point.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "no production caller yet — Compiler::compile itself is rewired only at \
                  Phase 5's coordinated cutover, once the whole crate consumes FirPointer trees"
    )
)]
mod arena_compiler {
    use super::{FVMStorage, FirCursorMut, FirPointer, FirSpec};

    use foolish_core::fir::Nyes;
    use foolish_parser::{AssignmentOperator, Astn, SearchOperator};

    use crate::identifier::{Characterizations, Identifier};

    /// Element types allowed inside a ConcatBrane. Byte-for-byte copy of
    /// `compiler.rs`'s real (private) `ConcatElemKind` — duplicated rather
    /// than exposed from `compiler.rs`, since this task does not otherwise
    /// need to touch that file at all, and this is a small, self-contained,
    /// storage-independent AST-classification type.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum ConcatElemKind {
        BareBrane,
        BareConcat,
        BareSearch,
        SfSearch,
        SfBrane,
        Error,
    }

    /// Byte-for-byte copy of `compiler.rs`'s real (private) `validate_astn`
    /// (re-read directly) — a plain, storage-independent AST walk; no
    /// `FirPointer` involvement at all, so this is copied verbatim with no
    /// arena threading needed, for the same reason `ConcatElemKind` above is
    /// duplicated rather than exposed.
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
    /// `classify_concat_element` (re-read directly) — plain AST
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

    /// Direct arena-threaded translation of `compiler.rs`'s real
    /// `build_concat_element` (re-read in full immediately before writing
    /// this): `parent` is the ALREADY-CREATED arena parent (the
    /// `Concatenation`/`ConcatHelper`-equivalent node), so unlike the
    /// original's `Rc::new_cyclic`-per-wrapper-kind construction, each
    /// wrapper here is one `create_child` call.
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

    /// Direct, line-by-line arena-threaded translation of `compiler.rs`'s
    /// real `build_fir` (re-read in full immediately before writing this —
    /// every one of the 12 `Rc::new_cyclic` sites in the real function's
    /// match arms is translated, none skipped).
    ///
    /// `parent: Option<FirPointer>` mirrors the original's `Option<&Weak<...>>`
    /// exactly: `None` means build a ROOT (self-parented via
    /// `FVMStorage::make_root`); `Some(p)` means a child of `p` (via
    /// `create_child`). Every arm that recurses builds its OWN node FIRST
    /// (via `make_root`/`create_child` with a placeholder-then-mutate shape
    /// is NOT needed here — unlike the original's `Rc::new_cyclic`, which
    /// needs the self-`Weak` to exist BEFORE children can be built with it
    /// as their parent, `create_child`/`make_root` need only the FINAL field
    /// values, which for tree-structural fields are nothing at all — so the
    /// arena order is: construct this node first (getting its `FirPointer`
    /// immediately), THEN build children as its `create_child`s. This is
    /// the exact "collapses to one `create_child` call" simplification
    /// FOOP-16.md's Motivation section describes.
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
                        provenance: crate::fir_kinds::ConcatProvenance::Juxtaposition,
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
                        provenance: crate::fir_kinds::ConcatProvenance::TailConcatenation,
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
                crate::compiler::ANON_STMT_NAME.to_string(),
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

    /// Direct translation of `compiler.rs`'s real `Compiler::compile`'s
    /// per-AST-node entry point (`AstnCompilerExt::compile_standalone`),
    /// combined here into one function since the arena has no analogous
    /// `impl AstnCompilerExt for Astn` — free functions taking `Astn` by
    /// value already match this module's own established shape.
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

    /// Direct translation of `compiler.rs`'s real `Compiler::compile`.
    pub(crate) fn compile(
        storage: &mut FVMStorage,
        source: &str,
    ) -> anyhow::Result<Vec<FirPointer>> {
        let asts = foolish_parser::parse(source)?;
        asts.into_iter()
            .map(|ast| compile_standalone(storage, ast))
            .collect()
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
        // directly rather than stepping a real search, which is Phase 2's
        // scope).
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

    /// `clone_subtree`'s SF/SFF unwrap: a `StayFoolish` with a settled
    /// result unwraps to that result (recursing through `clone_subtree`
    /// again on it), never producing a cloned SF wrapper node — mirrors
    /// `constanic_clone_at`'s own first branch exactly.
    #[test]
    fn clone_subtree_unwraps_stay_foolish_to_its_settled_result() {
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

        let cloned = storage.clone_subtree(sf, other_root, 0, false, false);

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

    /// `clone_subtree`'s SF/SFF unwrap falls through to the first foolish
    /// child when there is no settled result yet (or for `StayFullyFoolish`,
    /// which never tries `ubc_children` first at all).
    #[test]
    fn clone_subtree_unwraps_stay_fully_foolish_to_first_foolish_child() {
        let (mut storage, root) = FVMStorage::test_root_brane(&[]);
        let sff = root.create_child(&mut storage, FirSpec::StayFullyFoolish);
        let inner = sff.create_child(&mut storage, FirSpec::IndepInt { value: 9 });
        // inner stays Prembrionic — a full-rebuild case, not a share.
        let other_root = storage.make_root(FirSpec::IndepInt { value: 0 });

        let cloned = storage.clone_subtree(sff, other_root, 0, false, false);

        assert_ne!(
            cloned, sff,
            "no cloned SFF wrapper node should ever be produced"
        );
        assert_ne!(
            cloned, inner,
            "a pre-constanic inner must be rebuilt, not shared"
        );
        assert_eq!(storage.get(cloned), &FirSpec::IndepInt { value: 9 });
    }

    /// `ConcatHelper`'s arena migration: identical `BraneFir`-shaped
    /// stepping ("transparent: inherits all defaults," confirmed by direct
    /// re-read of the real `impl Fir for ConcatHelper`). Mirrors
    /// `fir_kinds.rs::tests::concat_helper_nyes_transitions` exactly.
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

        assert_eq!(storage.get_nyes(helper), Nyes::Constant);
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
            provenance: crate::fir_kinds::ConcatProvenance::Juxtaposition,
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

        // Drain: Prembrionic->Braning (push both elements as tasks), then
        // one step per queued task to pop it (both already Constant), then
        // one more step for the Braning arm's actual type-check pass.
        for _ in 0..5 {
            if storage.get_nyes(cat) == Nyes::Woconstanic {
                break;
            }
            cat.step(&mut storage);
        }

        assert_eq!(
            storage.get_nyes(cat),
            Nyes::Woconstanic,
            "join-ready elements settle Woconstanic under this deferred-merge implementation"
        );
        assert_eq!(
            FirCursor::new(cat, &storage).as_concat_provenance(),
            crate::fir_kinds::ConcatProvenance::Juxtaposition
        );
    }

    /// A concatenation with a genuinely non-brane, settled element (an
    /// `IndepInt`) settles `Nk` with the exact reason format the real
    /// `fir_op_step` produces — mirrors the type-error branch exactly.
    #[test]
    fn concatenation_with_a_non_brane_element_settles_nk() {
        let mut storage = FVMStorage::new();
        let cat = storage.make_root(FirSpec::Concatenation {
            provenance: crate::fir_kinds::ConcatProvenance::Juxtaposition,
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

    /// `ComparisonFir`'s arena migration is the two-phase push/combine shape
    /// PLUS the entirely self-contained ECONSTANIC-if-unevaluated-here gate
    /// (see this kind's `fir_op_step` arm doc comment — the real verdict
    /// resolution needs `_ab_search`, deferred to Phase 2). This test proves
    /// the ECONSTANIC gate: an operand whose own first foolish child is
    /// itself `Econstanic` (mirroring `<<#-1>>`'s SFF-wrapped-search shape
    /// inside `system.foo`, per `operand_is_unevaluated_here`'s real logic,
    /// re-read directly) makes the whole comparison settle `Econstanic`.
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

    /// When both operands ARE genuinely evaluated (not the
    /// unevaluated-in-system.foo case), this arena translation settles
    /// `Woconstanic` — an honestly-incomplete result, since the real verdict
    /// resolution (`resolve_boolean`, via `_ab_search`) is deferred to
    /// Phase 2. Documented explicitly here rather than silently claiming a
    /// `Constant`/`Nk` verdict this task does not actually compute.
    #[test]
    fn comparison_with_evaluated_operands_defers_the_real_verdict() {
        use crate::system_foo::ComparisonOp;

        let mut storage = FVMStorage::new();
        let cmp = storage.make_root(FirSpec::Comparison {
            op: ComparisonOp::Eq,
        });
        let left = cmp.create_child(&mut storage, FirSpec::IndepInt { value: 1 });
        storage.with_mut(left, |fir| fir.set_nyes(Nyes::Constant));
        let right = cmp.create_child(&mut storage, FirSpec::IndepInt { value: 1 });
        storage.with_mut(right, |fir| fir.set_nyes(Nyes::Constant));

        for _ in 0..5 {
            if storage.get_nyes(cmp).is_constanic() {
                break;
            }
            cmp.step(&mut storage);
        }

        assert_eq!(
            storage.get_nyes(cmp),
            Nyes::Woconstanic,
            "real verdict resolution is deferred to Phase 2 (needs _ab_search)"
        );
    }

    // ── Phase 2: search engine arena migration tests ────────────────────
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
    /// candidate, mirroring `matcher_name_approve_on_exact_match`'s intent.
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

    // ── Phase 2, final task: SearchFir end-to-end dispatch tests ────────
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

    // ── Phase 3: evaluator.rs stepping loop / core-FIR conversion tests ──

    use core_fir_conversion::{proto_to_core_fir, step_to_settled};
    use foolish_core::fir::FirQueryable;

    /// `step_to_settled`'s happy path: an `IndepInt` settles within budget.
    #[test]
    fn step_to_settled_settles_a_simple_fir() {
        let mut storage = FVMStorage::new();
        let ptr = storage.make_root(FirSpec::IndepInt { value: 7 });
        assert!(step_to_settled(&mut storage, ptr).is_ok());
        assert_eq!(storage.get_nyes(ptr), Nyes::Constant);
    }

    /// `proto_to_core_fir` on a settled `IndepInt` produces a
    /// `hs_constant_int` matching the value — mirrors the direct
    /// `FirKind::IndepInt` arm of the real `proto_to_core_fir_inner`.
    #[test]
    fn proto_to_core_fir_renders_constant_int() {
        let mut storage = FVMStorage::new();
        let ptr = storage.make_root(FirSpec::IndepInt { value: 42 });
        step_to_settled(&mut storage, ptr).unwrap();

        let rendered = proto_to_core_fir(&storage, ptr);
        assert_eq!(rendered.hs_constant_int(), Some(42));
        assert_eq!(rendered.hs_state(), Nyes::Constant);
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

    // ── Phase 4: arena_compiler tests ────────────────────────────────────

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
            Some(crate::compiler::ANON_STMT_NAME)
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
    /// rule together, mirroring `push_foolish_child_sff_marked_accepts_a_properly_marked_body`'s
    /// intent (that this crate's own compiler produces bodies satisfying
    /// the SFF invariant).
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
    /// `ArenaFir::set_contexted` together.
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

    // ── IndexFir dispatch (Phase 5 cutover prerequisite) ────────────────

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

    /// Direct arena counterpart to the real `index_finds_element_at_offset_
    /// in_anchor_brane`: an anchored `#1` index into a brane of three
    /// statements settles Constant with the middle statement's value.
    /// Exercises `IndexFir`'s `Prembrionic`/`Embryonic` push-anchor-task
    /// arm, then the `Braning` anchored-search arm (`BraneNavigator` +
    /// `SearchPredicate::Index`), then `settle_from_ubc_result`.
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

    /// Direct arena counterpart to the real `index_out_of_bounds_is_nk`: an
    /// anchored index whose target falls outside the anchor brane's
    /// statement range settles Nk.
    #[test]
    fn index_fir_out_of_bounds_is_nk() {
        let (mut storage, idx, _stmts) = index_with_anchor_brane(5, true);
        core_fir_conversion::step_to_settled(&mut storage, idx).unwrap();
        assert_eq!(storage.get_nyes(idx), Nyes::Nk);
        assert!(FirCursor::new(idx, &storage).ubc_children().is_empty());
    }

    /// Direct arena counterpart to the real `index_negative_offset_from_
    /// back`-style indexing: `#-1` anchored into a three-statement brane
    /// addresses the LAST statement.
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

    /// Direct arena counterpart to the real `foop75_non_brane_anchor_names_
    /// the_value`-style diagnostic: an anchored `IndexFir` whose anchor
    /// resolves to a non-brane, NAMEABLE value (an integer literal) settles
    /// Nk AND records a named reason (FOOP-75 §7) — both via a fresh
    /// ubc_children Nk AND via `alarm_reason`.
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

    /// Direct arena counterpart to the real `contexted_index_offset_finds_
    /// next_statement`: a contexted, anchored index (`&#1`-shaped) reads its
    /// anchor's `FoolRefFir` bookkeeping entry to find the REFERENT's home
    /// brane and position, then indexes relative to THAT position — not the
    /// position of the index node itself. Exercises the `contexted &&
    /// anchored` branch, distinct from the plain-anchored branch above.
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

    /// Direct arena counterpart to the real `contexted_index_out_of_range_
    /// is_nk`: a contexted index whose target falls outside the referent's
    /// home brane range settles Nk.
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

    // ── StatementFir NF-refusal checks (FOOP-33 §4, Phase 5 cutover) ────

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
}
