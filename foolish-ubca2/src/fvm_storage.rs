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
    children: Vec<FirPointer>,
    generation: u32,
}

/// Placeholder per-node payload for this foundational task.
///
/// Deliberately NOT `trait Fir` — see this module's top-level doc comment for
/// why. Holds exactly the data every kind needs generically (a `FirSpec`
/// classifying which kind this slot represents, plus a mutable `Nyes`) so
/// `FVMStorage`'s own read/write/round-trip behavior can be proven correct in
/// isolation before any real kind depends on it. Each per-kind migration task
/// replaces reads/writes of this placeholder with that kind's own arena-aware
/// `Fir` impl; `ArenaFir` itself is deleted once every kind has migrated
/// (tracked as part of Phase 1's per-kind tasks, not a separate cleanup).
#[derive(Debug, Clone)]
struct ArenaFir {
    spec: FirSpec,
    nyes: Nyes,
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
    /// The closure receives `&mut Nyes` only (not the whole slot) at this
    /// foundational stage — `set_nyes` is the one placeholder mutation this
    /// task needs to prove `with_mut`/`get_mut` round-trip correctly; a real
    /// per-kind `&mut dyn Fir` receiver arrives with the first per-kind
    /// migration task.
    pub fn with_mut<R>(&mut self, ptr: FirPointer, f: impl FnOnce(&mut Nyes) -> R) -> R {
        let index = self.validate(ptr);
        f(&mut self.slots[index].payload.nyes)
    }

    /// Retrieve one exclusive, held `&mut Nyes` for a run of several
    /// SEQUENTIAL writes with nothing storage-needing interleaved between
    /// them. See FOOP-16.md §Specification "`FVMStorage` — the arena" and the
    /// `OperatorFir::combine` walkthrough that motivated this alongside
    /// `with_mut` — the two are equally powerful; the choice is style.
    pub fn get_mut(&mut self, ptr: FirPointer) -> &mut Nyes {
        let index = self.validate(ptr);
        &mut self.slots[index].payload.nyes
    }

    /// This pointer's children, in construction order.
    pub fn children(&self, ptr: FirPointer) -> &[FirPointer] {
        let index = self.validate(ptr);
        &self.slots[index].children
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
        self.slots[parent.index as usize].children.push(ptr);
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
            payload: ArenaFir { spec, nyes },
            parent,
            children: Vec::new(),
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
        let parent_is_brane_like = matches!(
            storage.get(parent),
            FirSpec::Brane { .. } | FirSpec::ConcatHelper
        );
        if parent_is_brane_like {
            Some(parent)
        } else {
            parent.home_brane(storage)
        }
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
        assert_eq!(storage.children(root), &[child]);

        storage.with_mut(child, |nyes| *nyes = Nyes::Constant);
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

        *storage.get_mut(child) = Nyes::Constant;
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
}
