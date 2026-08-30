---
foop: D61
title: foolish-ubca2 — arena-backed FIR storage via copy-migration
author: Claude Code <noreply@anthropic.com>
status: Implementing
type: Standards
created: 2026-08-30
phase: phase-2
supersedes: []
begun: [x]
---

# FOOP-16: foolish-ubca2 — arena-backed FIR storage via copy-migration
FOOP numbering is little-endian; the full rules live in `foop.md` at the repository root —
**read it before creating or editing a FOOP.** The one template-specific note: the `foop:`
front-matter field may either match the filename digits directly (`foop: 16`) or give the
big-endian decimal value, preceded by `D` (`foop: D61`, used here) — both mean the same thing,
sort key 61, file `FOOP-16.md`.

## Abstract

`foolish-ubca` builds its FIR tree with `Rc<RefCell<dyn Fir>>` children and `Weak<RefCell<dyn
Fir>>` parent back-pointers, constructed through `Rc::new_cyclic` at 68 call sites across the
crate. This FOOP proposes a new crate, `foolish-ubca2`, built as a byte-for-byte copy of
`foolish-ubca` and then migrated, piece by piece, onto a `u32`-indexed arena (`FVMStorage`)
addressed through a validated handle type (`FirPointer`), with construction collapsing to a
single `create_child` call per node instead of the current multi-step `Rc::new_cyclic` ceremony.
`foolish-ubca` itself is never modified — it stays the frozen correctness oracle for the entire
migration, and `foolish-ubca2`'s own einmo suite is validated case-by-case against
`foolish-ubca`'s existing `checked/` baselines using the crate-agnostic `einmo::Evaluator`
trait, the same mechanism `zweimomo` already uses for cross-language validation today.

## Motivation

Building or cloning any FIR subtree today pays the same fixed tax at every one of 68 sites (45
in `fir_kinds.rs`, 18 in `compiler.rs`, 1 in `proto_brane.rs`): obtain a self-`Weak` via
`Rc::new_cyclic` before the node exists, widen it to `dyn Fir`, thread it by hand into every
child builder, then separately push the new child into the parent's `Vec<FirRef>`. Nothing in
the type system ties the "set the child's parent pointer" step to the "push into the parent's
children" step — they are two manual operations that must be kept in sync by the author, every
time. `constanic_clone_at` (the AB/IB recoordination path used whenever a brane is referenced by
name and detached/recoordinated into a new context) repeats this ceremony recursively across an
entire cloned subtree, making it one of the largest and most delicate blocks of pointer-wiring
code in the crate.

An arena replaces all of this with integer-index copies. Parent and child links become
`FirPointer` values — `Copy`, comparable, and safe to pass around without borrow-checker
interaction — and a single `create_child` call both allocates a node and wires it into its
parent, atomically, with no separate back-pointer step to omit or get out of sync. This is
fundamentally a **construction-boilerplate-removal** change: the goal is that the ~68 call sites
doing 4–6 lines of `Rc::new_cyclic`/`Weak` ceremony each collapse to one `create_child(...)` call
each, and that `constanic_clone_at`'s recursive re-wiring collapses to `clone_subtree` copying
indices.

Today: building a brane's statement chain in `compiler.rs` requires nesting
`Rc::new_cyclic(|me: &Weak<RefCell<StatementFir>>| { ... })` closures, each one manually setting
`parent: parent.clone()` and separately appending to the parent's
`foolish_children`/`ubc_children` vectors. After this FOOP, the same chain is built as a flat
sequence of `stmt_id.create_child(&mut storage, spec)` calls, each one self-wiring.

## Specification

### `FirPointer` — a validated arena handle

```rust
/// A validated handle into one specific FVMStorage arena. Cannot be
/// constructed, incremented, or otherwise fabricated outside the module
/// FVMStorage lives in — only ever handed out by an FVMStorage method that
/// just finished allocating or validating it. Bounds-checking and
/// validity-checking happen once, centrally, inside FVMStorage; callers
/// never re-derive or assume validity.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct FirPointer {
    arena: ArenaId,   // randomized stamp identifying which FVMStorage
                       // minted this pointer — NOT a live &FVMStorage
                       // borrow, so FirPointer stays Copy/storable and
                       // does not tie up the arena's mutable access.
    index: u32,        // dense arena slot — sequential, O(1) indexing.
    generation: u32,    // per-slot reuse counter — sequential per slot.
}
```

All three fields are private. There is no public constructor and no arithmetic `impl` on
`FirPointer` — the only way to obtain one is as the return value of an `FVMStorage` method.
`index` and `generation` are sequential (dense slots, cheap indexing, cache-friendly node
placement); `arena` is a randomized stamp minted once per `FVMStorage` instance, so a pointer
minted from one arena can never silently validate against a different arena instance. This
directly implements `rust_instructions.md` §1b.4 ("make illegal states unrepresentable over
validating at call sites — encode invariants in the type system"): a bare `u32` index would let
a caller fabricate an out-of-bounds or wrong-arena value that compiles cleanly; `FirPointer`
makes that unrepresentable.

`FirPointer` and `FVMStorage` are defined in the same module. Rust's privacy is module-scoped,
not type-scoped, so `FVMStorage`'s own methods read `FirPointer`'s private fields directly — no
accessor methods are needed for that internal access — while every caller outside the module is
unable to construct, inspect, or do arithmetic on a `FirPointer`.

### `FVMStorage` — the arena

```rust
pub struct FVMStorage {
    arena_id: ArenaId,
    slots: Vec<Slot>,
}

struct Slot {
    fir: Fir,
    generation: u32,
}

impl FVMStorage {
    /// Retrieve for reading. Validates ptr.arena == self.arena_id and
    /// ptr.generation == slot.generation before indexing.
    pub fn get(&self, ptr: FirPointer) -> &Fir { .. }

    /// Retrieve, modify, and return in one call — the "retrieve a Fir, be
    /// able to modify it before returning" primitive. Closure-scoped so
    /// there is no separate get/set pair to keep in sync, and no
    /// RefCell-style runtime borrow tracking is needed: the &mut self
    /// borrow on FVMStorage is the only exclusivity check required.
    pub fn with_mut<R>(&mut self, ptr: FirPointer, f: impl FnOnce(&mut Fir) -> R) -> R { .. }

    /// Retrieve one exclusive, held `&mut Fir` for a run of several
    /// SEQUENTIAL writes with nothing storage-needing interleaved between
    /// them — e.g. "push a ubc_child, then set_nyes" as one logical
    /// finishing step. Kept alongside `with_mut`, not instead of it: this
    /// is the same capability, offered as a plain borrow rather than a
    /// closure, for callers who want several `&mut Fir` method calls in a
    /// row without re-invoking `with_mut` for each one. Neither form is
    /// more powerful than the other — the choice between them is style,
    /// not capability. See "The `FirCursor`/`FirCursorMut` wrapper" below
    /// for why this exists and the `OperatorFir::combine` walkthrough that
    /// motivated it.
    pub fn get_mut(&mut self, ptr: FirPointer) -> &mut Fir { .. }

    /// Arena-owning implementation: allocates a slot for `spec`, sets its
    /// parent to `parent`, appends the new pointer to parent's child list,
    /// returns the new FirPointer. This is the one place a FirPointer is
    /// ever constructed from raw parts. Called directly only where no
    /// parent FirPointer exists yet (the very first/root node of a tree);
    /// every other call site uses FirPointer::create_child below.
    pub fn make_my_child(&mut self, parent: FirPointer, spec: FirSpec) -> FirPointer { .. }

    /// AB/IB recoordination: clones `root`'s subtree, reparenting the
    /// clone under `new_parent`. Replaces constanic_clone_at's recursive
    /// Rc::new_cyclic re-wiring with index copying.
    pub fn clone_subtree(&mut self, root: FirPointer, new_parent: FirPointer) -> FirPointer { .. }
}
```

### Arena allocation and expansion

`FVMStorage`'s `slots: Vec<Slot>` is a plain growable `Vec`, not a fixed-capacity buffer — no
pre-sizing is required, though a `with_capacity` hint sized to the compiler's known
statement/expression count for a given source file is a reasonable performance optimization to
consider at implementation time, not a correctness requirement.

Allocation is a bump allocator: `make_my_child`/`create_child` pushes a new `Slot` and the new
`FirPointer`'s `index` is `slots.len() - 1` at push time. There is **no free-list reuse** in this
FOOP's scope — a slot, once allocated, is never reclaimed and reassigned to a different node.
This matches `constanic_clone_at`'s existing behavior of always allocating a fresh clone rather
than mutating in place: subtrees the pre-arena code discards via dropped `Rc` refcounts simply
become permanently-unused arena slots under `FVMStorage`, wasting memory but never causing a
correctness bug. Introducing slot reuse is a plausible future optimization, but it is explicitly
**out of scope for FOOP-16** — reuse is exactly what would make the `generation` field
load-bearing for safety (distinguishing "a stale pointer to a since-recycled slot" from "a valid
pointer") rather than merely defensive as it is today. That is a natural follow-up FOOP, not
something this one needs to implement; "make it correct first" is the scope boundary here.

Expansion uses standard `Vec` growth (geometric reallocation) — no custom growth strategy is
needed. Because `FirPointer::index` is a plain `u32` slot number rather than a raw pointer or
reference into the `Vec`'s backing buffer, a `Vec` reallocation on growth is completely
transparent to every existing `FirPointer`: they remain valid across reallocation, unlike raw
pointers/references into a growing `Vec` would be. This is precisely the advantage an
index-based arena has over an intrusive linked structure here.

**`FirPointer`'s identity properties, stated explicitly:**

- **Sequential or random?** Both `index` and `generation` are sequential, not random. `index` is
  assigned by the bump allocator in creation order — dense, cache-friendly, and enabling O(1)
  direct indexing. `generation` is a per-slot monotonic counter starting at 0, incremented only
  on slot reuse; since slot reuse does not happen in FOOP-16's scope (above), every `generation`
  value in a FOOP-16-era arena is 0 in practice, becoming meaningful only once a future FOOP
  introduces reuse. The third field, `arena` (the `ArenaId` stamp), is the one exception: it
  **is** randomized, minted once per `FVMStorage::new()` call, specifically so a `FirPointer`
  from one arena instance can never silently validate against a different arena instance — a
  sequential/incrementing arena-instance counter would risk exactly that collision across
  independent runs, processes, or threads. The split is deliberate: `index`/`generation` are
  sequential for performance; `arena` is randomized for safety.
- **Externalizable or internal-only?** `FirPointer` is internal-only by design. It must never be
  serialized, persisted, or exposed outside the lifetime of the single `FVMStorage` that minted
  it: no `Serialize`/`Deserialize` impl, no `Display`/`FromStr`, no public constructor, private
  fields readable only by `FVMStorage` via same-module access (above). This matters directly for
  einmo interop: `.einmo` files persist OUTPUT as core-FIR values (`foolish_core::fir::FirRef`,
  the `CoreFirRef` type already used in `evaluator.rs`'s `Evaluator::evaluate` signature) — never
  `foolish-ubca`'s own internal `FirRef`/`FirPointer` — so `FVMStorage`'s internal pointers never
  cross that serialization boundary. The existing proto-FIR-to-core-FIR conversion step (the
  `proto_to_core_fir*` family, Phase 3 of the plan) is exactly where internal `FirPointer`s are
  left behind and only their *values* survive into the signed output. `FirPointer` is not, and
  must never become, a stable cross-process or cross-run identifier — it is scoped exclusively to
  one in-memory `FVMStorage` instance's lifetime.
- **Copy and hashable.** `FirPointer` is `Copy` (12 bytes: three `u32`s) and `Eq`/`Hash`, so it
  is usable directly as a `HashMap`/`HashSet` key wherever a caller needs a visited-set during
  traversal — e.g. the search engine's `CandidateNavigator`. This is a direct, concrete
  consequence of the type's small, plain-data shape, and part of the answer to "why not just
  keep using `Rc`."

### `FirPointer`'s handle-side methods — the primary calling convention

The crate's existing calling convention, verified against actual call sites in `evaluator.rs`
and `fir_kinds.rs`, is that every operation is a method on the FIR handle, never a free function
taking a FIR as an argument: `root.step(scope)`, `anchor.resolve_anchor()`,
`brane_ref.find_stmt_index(&stmt_ref)`, `fir_ref.deepest_econstanic_in_chain()`. Three extension
traits — `FirRefExt`, `FirRefNavExt`, `NyesExt` — exist purely to preserve that method-call
syntax on the foreign `Rc<RefCell<dyn Fir>>` type. `FirPointer` continues this exact convention
with the new handle type, adding `&FVMStorage`/`&mut FVMStorage` as an explicit parameter
wherever arena access is needed — never storing a handle back to the arena on the type itself,
which would reintroduce a shared-ownership problem through the side door:

```rust
impl FirPointer {
    /// Primary construction call site, used everywhere in this FOOP's
    /// plan. Delegates to FVMStorage::make_my_child.
    pub fn create_child(self, storage: &mut FVMStorage, spec: FirSpec) -> FirPointer {
        storage.make_my_child(self, spec)
    }

    pub fn get_parent(self, storage: &FVMStorage) -> Option<FirPointer> { .. }
    pub fn home_brane(self, storage: &FVMStorage) -> FirPointer { .. }
}
```

`FirSpec` is one enum mirroring the FIR-kind split already present in the crate (one variant per
`impl Fir for XFir` kind) — construction dispatches on it rather than fragmenting into a
`create_x_child` method per kind.

Before/after for chain construction (the case this FOOP is explicitly aimed at flattening):

```rust
// Before — compiler.rs, building one statement (illustrative shape of the
// existing pattern; see compiler.rs build_fir/build_stmts for the real
// nested closures):
let stmt_ref = Rc::new_cyclic(|me: &Weak<RefCell<StatementFir>>| {
    RefCell::new(StatementFir {
        proto_brane: ProtoBrane {
            parent: parent.clone(),
            foolish_children: vec![],
            ubc_children: RefCell::new(vec![]),
            ..Default::default()
        },
        identifier,
        line_number,
        ..Default::default()
    })
});
parent_children_vec.push(stmt_ref.clone());  // separate, easy-to-forget step

// After:
let stmt_id = brane_id.create_child(&mut storage, FirSpec::Statement {
    identifier, line_number,
});
```

### The `FirCursor`/`FirCursorMut` wrapper

`ProtoBrane` — not `trait Fir` — is where nearly all tree-structural state actually lives today:
every FIR kind's struct holds one `core: ProtoBrane` field carrying `foolish_children`,
`ubc_children`, `nyes`, `tasks`, `parent`, `alarm_reason`, and every one of `ProtoBrane`'s
inherent methods (`foolish_children()`, `ubc_children()`, `push_ubc_child()`, `parent()`,
`is_root()`, `get_nyes()`/`set_nyes()`, `front_task()`/`pop_front_task()`/`push_task()`, and the
SFF-invariant sift) is the real, method-by-method specification for what a storage-aware wrapper
needs to provide. `trait Fir`'s ~40 methods are mostly per-kind *data* accessors with default
bodies (`as_i64`, `as_op_name`, `as_stmt_identifier`, `stmt_count`, `stmt_at`, …) that answer
"what does this node hold," not "navigate the tree" — those stay plain methods reached through
one `storage.get(ptr)` call and need no special wrapper. The methods that genuinely recurse
across nodes (`_get_my_statement`, `_get_my_brane`, `_ib_search`/`_ab_search`, `settled_result`)
are the ones a wrapper earns its keep on.

Two lifetime-parameterized cursor types, not one type generic over mutability — stable Rust
cannot cleanly unify `&`/`&mut` without `unsafe` or `dyn` indirection, and neither fits this
project's construct-preference order:

```rust
/// A FirPointer paired with a borrow of the FVMStorage to read it through.
/// Captures storage once so a run of navigation calls on one node doesn't
/// repeat `&storage` at every call. Read-only: FirCursor is Copy-cheap to
/// construct and multiple calls through one (or several at once) compose
/// freely, the same as any shared borrow.
pub struct FirCursor<'s> {
    ptr: FirPointer,
    storage: &'s FVMStorage,
}

/// The mutating counterpart. Rust allows only one &mut at a time, so unlike
/// FirCursor this does NOT support "wrap once, call five mutating methods" —
/// each mutating call still needs its own &mut reborrow under the hood. Its
/// value is bundling ptr+storage for ONE logical mutating operation, not
/// batching several.
pub struct FirCursorMut<'s> {
    ptr: FirPointer,
    storage: &'s mut FVMStorage,
}
```

**Read side (`FirCursor`)** — mirrors `ProtoBrane`'s read methods directly, since shared borrows
compose:

| Method | Maps to today's | What changes |
|---|---|---|
| `node(&self) -> &Fir` | `self.borrow()` on a `FirRef` | direct arena lookup, no `RefCell` |
| `foolish_children(&self) -> &[FirPointer]` | `ProtoBrane::foolish_children()` | unchanged shape, `FirPointer` instead of `FirRef` |
| `ubc_children(&self) -> &[FirPointer]` | `ProtoBrane::ubc_children()` | **removes** the `self.ubc_children.borrow().clone()` dance — today's clone-out-of-`Vec` exists purely to defend against reentrant `RefCell` borrow panics; a plain slice read is safe once there is no `RefCell` to reenter |
| `all_children(&self) -> impl Iterator<Item = FirPointer> + '_` | `ProtoBrane::all_children()` | same render-order contract preserved exactly: ubc first, then foolish |
| `parent(&self) -> Option<FirPointer>` | `ProtoBrane::parent()` (`Weak::upgrade()`) | **simplifies**: today's doc comment states `upgrade()` returns `None` "only during teardown" — the arena never drops a live node out from under a valid pointer, so that liveness case disappears; `Option` remains only for the true structural root, whose "parent" is itself (mirroring today's self-referential root `Weak`, confirmed by `proto_brane_parent_link`'s test: `root.borrow().core().parent().unwrap()` pointer-equals `root`) |
| `is_root(&self) -> bool` | `ProtoBrane::is_root(&self, self_rc: &FirRef)` | **simplifies**: today's version needs the caller to pass back its own `Rc` handle because a bare `ProtoBrane` has no self-identity; `FirCursor` already carries `self.ptr` as its own identity, so no parameter is needed — `self.ptr == self.parent().unwrap()` |
| `get_nyes(&self) -> Nyes` | `ProtoBrane::get_nyes()` | unchanged |
| `front_task(&self) -> Option<FirPointer>` | `ProtoBrane::front_task()` | unchanged shape |
| `home_brane(&self) -> Option<FirCursor<'s>>` | `Fir::_get_my_brane` | arena-threaded parent-chain climb to the first brane-like kind; preserves the exact termination logic (climb until `core().parent()` pointer-equals `self` → `None`; else check `is_brane_like()` → stop, else recurse), with `Rc::ptr_eq` becoming a `FirPointer` equality check |
| `statement(&self) -> FirCursor<'s>` | `Fir::_get_my_statement` | same climb-to-`FirKind::Statement`-or-self-if-root shape |
| `settled_result(&self) -> Option<FirCursor<'s>>` | `Fir::settled_result` | preserves the exact constanic-gate contract verbatim from today's doc comment: "applies the constanic gate ITSELF — pre-constanic always answers `None`" |

**Mutating side (`FirCursorMut`)** — one mutation per call, matching the `with_mut` shape already
specified above:

| Method | Maps to today's | What's preserved |
|---|---|---|
| `set_nyes(&mut self, n: Nyes)` | `ProtoBrane::set_nyes` (`pub(crate)`) | the exact OWNERSHIP CONTRACT from today's doc comment, verbatim: "a FIR owns its own nyes — nyes must NOT be changed from outside the FIR. The ONLY sanctioned writers are: (1) a FIR on ITSELF, inside its own `fir_op_step`, and (2) construction." Enforced the same way today's `pub(crate)` visibility enforces it: `FirCursorMut::set_nyes` (or the `FVMStorage` setter it forwards to) stays `pub(crate)`, not `pub` — no code outside the crate, and no code inside it other than a node's own `fir_op_step` or a construction path, may call it. |
| — (no equivalent) | `ProtoBrane::push_foolish_child` (`&mut self`, construction-time only) | **has no `FirCursorMut` equivalent — it is superseded entirely by `create_child`.** Today's doc comment states this method is reachable "ONLY while the ProtoBrane is still owned… before the FIR is wrapped in `Rc<RefCell<…>>` and goes live." `FirCursorMut` implies a node already *live* in the arena; the construction-time-only case is exactly what `create_child`/`make_my_child` cover, so there is deliberately no second path. |
| `create_child(&mut self, spec: FirSpec) -> FirPointer` | `ProtoBrane::push_foolish_child` + the enclosing `Rc::new_cyclic` | delegates to `self.ptr.create_child(self.storage, spec)` — the live, in-arena equivalent for a node that needs to grow a new child post-construction |
| `push_foolish_child_sff_marked(&mut self, child: FirPointer)` | `ProtoBrane::push_foolish_child_sff_marked` | preserves the exact SFF invariant check: walks `child`'s foolish-store descendants, panics unconditionally (not `debug_assert!`) if any search-kind descendant is not exactly `ECONSTANIC`. The walk itself becomes an arena-threaded `sift_*` function (per the codebase's own naming rule — a Rust-side tree walk, not a Foolish search): `fn sift_for_first_non_econstanic_descendent_search(storage: &FVMStorage, node: FirPointer) -> Option<FirPointer>`, preserving the exact `== Econstanic` check (not `is_constanic()`) and the exact panic-message intent. |
| `push_ubc_child(&mut self, child: FirPointer)` | `ProtoBrane::push_ubc_child` | preserves exact behavior: pushes to `ubc_children` AND enqueues as a task if the child is not already constanic |
| `push_search_result(&mut self, result: FirPointer)` | `ProtoBrane::push_search_result` | preserves the exact SINGULAR-RESULT INVARIANT (FOOP-62) `debug_assert!` |
| `clear_ubc_children(&mut self)` | `ProtoBrane::clear_ubc_children` | unchanged |
| `pop_front_task(&mut self)` / `push_task(&mut self, t: FirPointer)` | same-named `ProtoBrane` methods | unchanged shape |

**Test helpers.** `proto_brane.rs`'s own test module hand-builds trees via `make_leaf`/
`make_root_brane` (`fir_trait.rs`'s test module) rather than compiling real Foolish source for
every unit test — the arena world needs the same shortcut, or every one of the thirteen per-kind
migration tasks below will invent its own ad hoc scaffolding inconsistently. `FVMStorage` should
carry a small, `#[cfg(test)]`-gated pair of equivalents — e.g. `FVMStorage::test_leaf(nyes: Nyes)
-> (FVMStorage, FirPointer)` and `FVMStorage::test_root_brane(children_specs: &[FirSpec]) ->
(FVMStorage, FirPointer)` — mirroring the existing helpers' signatures closely enough that a
reader who knows one recognizes the other. This is a small addition to the foundational task in
Phase 1 (see `FOOP-16.plan.md`), not a separate phase.

**Stepping does not go through `FirCursorMut` as its receiver.** `step_inner`'s real recursion
(read in full from `fir_trait.rs`) recurses into a *child* node, reborrowing storage fresh at
each depth level — a `FirCursorMut` holding one `&mut` borrowed for an entire recursive call tree
cannot express that per-level reborrowing cleanly. So `step` stays a direct `FirPointer` method
taking `&mut FVMStorage` explicitly, and the arena-threaded translation preserves `step_inner`'s
logic exactly:

```rust
impl FirPointer {
    pub fn step(self, storage: &mut FVMStorage, scope: &Scope) -> Result<StepReport, UbcError> {
        step_inner(self, storage, scope, 0)
    }

    /// Arena-threaded FirRefExt::value: recursively unwraps through
    /// settled_result, returning a clone of self when there is none.
    pub fn value(self, storage: &FVMStorage) -> FirPointer {
        match FirCursor { ptr: self, storage }.settled_result() {
            Some(child) => child.ptr.value(storage),
            None => self,
        }
    }
}

/// Direct translation of fir_trait.rs's step_inner — same MAX_DEPTH guard,
/// same front-task constanic-gate (pop vs. recurse), same Scope mutation for
/// StayFoolish/Statement/brane-like kinds before recursing.
fn step_inner(
    ptr: FirPointer,
    storage: &mut FVMStorage,
    scope: &Scope,
    depth: usize,
) -> Result<StepReport, UbcError> {
    if depth > MAX_DEPTH {
        return Ok(StepReport::NoProgress);
    }
    let front = storage.get(ptr).core().front_task(); // Option<FirPointer>

    match front {
        Some(front_ptr) => {
            if storage.get(front_ptr).core().get_nyes().is_constanic() {
                storage.with_mut(ptr, |fir| fir.core().pop_front_task());
            } else {
                let this_kind = storage.get(ptr).kind();
                let mut child_scope = if this_kind == FirKind::StayFoolish {
                    scope.with_ancestral_sfm(true)
                } else {
                    scope.clone()
                };
                if this_kind == FirKind::Statement {
                    child_scope.current_statement = Some(ptr);
                }
                if storage.get(ptr).is_brane_like() {
                    child_scope.current_brane = Some(ptr);
                }
                step_inner(front_ptr, storage, &child_scope, depth + 1)?;
            }
            Ok(StepReport::Progress(storage.get(ptr).core().get_nyes()))
        }
        None => {
            // fir_op_step needs &mut access to mutate its own nyes/tasks —
            // translated via with_mut, not a bare &Fir call as this sketch's
            // types suggest; the exact signature is an implementation-time
            // detail settled when Phase 3 migrates this function for real.
            storage.with_mut(ptr, |fir| fir.fir_op_step(scope))?;
            Ok(StepReport::Progress(storage.get(ptr).core().get_nyes()))
        }
    }
}
```

### Resolved: two cursor types, not one — settled against a real call site

The choice between two lifetime-bound cursor types and a single `Cell`/`RefCell`-backed cursor
was an open question in an earlier draft of this FOOP. It is settled now, worked through against
`OperatorFir::combine` (`foolish-ubca/src/fir_kinds.rs:618–756`) — a real function, not a
hypothetical one, and one that both mutates its own node and constructs new ones, so it exercises
exactly the shape either cursor design has to handle well.

**What `combine` actually does today.** Its NK branch does not build a new `NkFir` and drop it
straight into its own `ubc_children`. It builds a *standalone* node first (`Rc::new_cyclic`,
self-parented via its own `Weak`, exactly like a root), then calls
`ProtoBrane::constanic_clone_at(&nk_ref, &self_weak, 0, scope.has_ancestral_sfm, false)` to clone
that standalone node *under* `self`, and pushes the **clone** — not the original — as the
`ubc_child`. This same two-step "build standalone, then clone-to-reparent" pattern repeats three
more times in the same function (division-by-zero, modulo-by-zero, the successful-arithmetic
`IndepIntFir` result), each with its own `Rc::new_cyclic`/`Weak`/`constanic_clone_at` triplet.

**This changes the boilerplate-removal case for `create_child`, not just the cursor question.**
Under `FVMStorage`, `create_child` builds a node **already parented** where it's meant to live —
there is no "build standalone, then clone to reparent" two-step at all, because there is no
self-`Weak` to establish before the node exists. `combine`'s four repeated
construct-then-clone-to-reparent triplets each collapse to one `create_child` call. This is a
sharper, more concrete instance of this FOOP's Motivation than the compiler.rs example already
given — a real function this FOOP will touch, not an illustrative sketch.

**Why the cursor question resolves in favor of two types.** Walking `combine`'s NK branch through
both candidate designs side by side:

```rust
// Arena translation of combine's NK branch, using the resolved design
// (FirPointer::create_child + FVMStorage::get_mut):
fn combine(&self, storage: &mut FVMStorage, self_ptr: FirPointer, scope: &Scope) -> Result<(), UbcError> {
    let children: Vec<FirPointer> = storage.get(self_ptr).foolish_children().to_vec();
    let any_nk = children.iter().any(|&c| storage.get(c).get_nyes() == Nyes::Nk);
    if any_nk {
        let reason = children
            .iter()
            .find_map(|&c| {
                let fir = storage.get(c);
                (fir.get_nyes() == Nyes::Nk)
                    .then(|| fir.as_nk_reason().map(str::to_string))
                    .flatten()
            })
            .unwrap_or_else(|| "operator nk".to_string());
        // One call — no standalone-then-clone-to-reparent needed, since
        // create_child builds the node already parented under self_ptr.
        let nk_ptr = self_ptr.create_child(storage, FirSpec::Nk { reason });
        // One held &mut Fir, two sequential writes through it:
        let me = storage.get_mut(self_ptr);
        me.push_ubc_child(nk_ptr);
        me.set_nyes(Nyes::Nk);
        return Ok(());
    }
    // ... arithmetic branch follows the identical shape
}
```

Two observations settle the question:

1. **`create_child` already requires `&mut FVMStorage`**, because it allocates a new arena slot.
   Any `fir_op_step`/`combine`-style body that constructs a node — which `combine` does, four
   times — needs `&mut storage` as a parameter for that reason alone, regardless of which cursor
   design wins. The `Cell`/`RefCell`-backed alternative's promised saving — "hold a cheap
   read-only cursor most of the time, pay for `&mut` only occasionally" — does not materialize
   for this shape of code, because the mutation-needing call is not occasional here; it is the
   point of the branch.
2. **The real ergonomic complaint was never "I need `&mut` too often."** It was "three separate
   `with_mut` closures for what is one logical finishing step." `FVMStorage::get_mut` (added
   above) answers that complaint directly: one held `&mut Fir`, `push_ubc_child` then `set_nyes`
   through it, no closure boilerplate — with no need to weaken `FVMStorage`'s all-writes-go-
   through-`&mut`-borrows guarantee to get it.

With `get_mut` in hand, the `Cell`/`RefCell`-backed alternative's only remaining advantage —
letting one cursor mix reads and writes without a second type — is not needed for the code this
FOOP actually has to migrate. Keeping `FirCursor`/`FirCursorMut` as two lifetime-bound types
means a misuse (holding a mutable handle live across a recursive `step_inner` call — see "Borrow
discipline under the arena" below) stays a compile error, not a reintroduced runtime-panic risk on
exactly the fields (`nyes`, `tasks`, `ubc_children`) this FOOP's Motivation names as the reason to
move to an arena in the first place. **Decision: keep the two-cursor-type design as specified
above, with `FVMStorage::get_mut` added to remove the specific "three separate closures"
complaint that motivated reconsidering it.**

**A narrower, real need remains: interleaved reacquisition.** `get_mut` covers "several writes,
nothing storage-needing in between." It does not cover a rarer but genuine shape: code that holds
a mutation handle, needs to make a storage-needing call *in the middle* (e.g. `create_child`-ing a
*second* node partway through, after already starting to mutate the first), then needs to resume
writing to the *original* handle. This is genuine interleaving, not achievable with one `get_mut`
call, and today's `Rc<RefCell<dyn Fir>>` code does not face it (a `RefCell` borrow can simply be
re-opened after being dropped, with no accompanying type to reacquire). A small macro closes the
gap without inventing anything unsafe:

```rust
/// Drops `$handle`, evaluates `$reacquire` (which may itself need &mut
/// FVMStorage — e.g. a nested create_child call), then re-acquires a fresh
/// handle via the same accessor and binds it back to `$handle`. No unsafe,
/// no magic: pure sugar over "drop the borrow, do the storage-needing
/// thing, get the borrow back," which is otherwise legal Rust but visually
/// noisy to write out by hand at every site that needs it.
macro_rules! temporary_release {
    ($handle:ident, $reacquire:expr, $body:expr) => {{
        drop($handle);
        let __result = $body;
        let $handle = $reacquire;
        (__result, $handle)
    }};
}

// Illustrative use (not a specific real function — a plausible shape:
// finish writing to `first`, build a second node mid-sequence, then
// resume writing to `first`):
let first = storage.get_mut(first_ptr);
first.push_ubc_child(partial_result_ptr);
let (second_ptr, first) = temporary_release!(
    first,
    storage.get_mut(first_ptr),
    first_ptr.create_child(storage, FirSpec::IndepInt { value: 0 })
);
first.set_nyes(Nyes::Woconstanic);
```

`temporary_release!` is documented here as an available primitive for whichever migrated
function(s) turn out to need this interleaved shape (Phase 1's foundational task should add it
alongside `get_mut`); it is not required by `combine` itself, which the walkthrough above shows
does not need it.

### `clone_subtree` — grounded in `constanic_clone_at`'s real logic

`constanic_clone_at` (`ProtoBrane::constanic_clone_at`, `fir_kinds.rs`, ~30 call sites) is
**recursive per-node**, not "clone the whole subtree, then relink." Each call matches on the
source node's `FirKind` and either:

1. **Shares, not clones.** A node that is already `Constant` or `Independent` and is *not* a
   `Brane` returns `Rc::clone(fir_ref)` — the same `Rc`, not a new allocation. `FoolRefFir` and
   `CreationFir` nodes *always* short-circuit to `Rc::clone(fir_ref)`, unconditionally, regardless
   of NYES state — this is what makes the FoolRefFir two-child invariant's `[1]` (the original
   found statement) a genuinely shared reference rather than a copy, and what keeps a named
   creation's identity intact across detachment.
2. **Unwraps.** `StayFoolish`/`StayFullyFoolish` nodes are stripped entirely: the clone recurses
   into the settled result (if the SF node is already constanic) or the first foolish child
   (otherwise), never producing a cloned SF/SFF wrapper node itself.
3. **Recursively rebuilds**, for every other kind: a fresh `Rc::new_cyclic` node whose children
   come from `clone_children_for_constanic_clone`, which maps `constanic_clone_at` over the
   source's `foolish_children` (each with its own positional `index`) and `ubc_children` in turn
   — so the whole subtree is rebuilt top-down, one recursive call per surviving node, not
   allocated in a single batch.

Three parameters beyond `(source, new_parent)` are load-bearing, not incidental: `index` (used
directly as a cloned `StatementFir`'s new `line_number` — clones are renumbered by their new
position, not carried over from the original), `descendent_of_sfm_and_foolishly_ignorant` (the
SFM flag threaded into `Nyes::transform_for_clone`, governing how NYES is remapped for a clone
living under a Stay-Foolish-Marked ancestor), and `skip_foolish_children` (used at the top level
of a clone to omit re-cloning parse-time children when only the ubc/result side is being
recoordinated).

`FVMStorage::clone_subtree`'s design mirrors this exactly, translated to arena terms:

```rust
impl FVMStorage {
    /// Direct arena-threaded translation of constanic_clone_at. Recursive,
    /// per-node, matching on FirKind — NOT a bulk subtree copy. Preserves:
    /// (1) the share-not-clone short-circuit for Constant/Independent
    /// non-Brane nodes, and the unconditional share for FoolRef/Creation;
    /// (2) SF/SFF unwrapping; (3) the index/sfm/skip_foolish_children
    /// threading exactly as today.
    pub fn clone_subtree(
        &mut self,
        root: FirPointer,
        new_parent: FirPointer,
        index: usize,
        sfm: bool,
        skip_foolish_children: bool,
    ) -> FirPointer { .. }
}
```

**A pointer into the original subtree remains exactly as valid after a clone as it was before.**
Arena slots are never mutated or reclaimed to redirect old pointers — `clone_subtree` only
*adds* new slots for the freshly-rebuilt nodes; the original subtree's slots are untouched. This
is the arena-era restatement of the correctness property `Rc` reference counting gives "for
free" today: the original subtree stays reachable (and meaningful) for as long as anything still
holds a `FirPointer` into it, exactly as an `Rc` clone keeps the original alive today via shared
ownership, except the arena's version costs nothing at clone time (no refcount bump) because
nothing about the original is touched at all.

### Borrow discipline under the arena

The `FirCursor`/`FirCursorMut` design above answers *what* a storage-aware navigation call
looks like; it does not yet answer a sharper question that stepping raises directly: while an
ancestor node's `step_inner` frame is "in the middle of" recursing into a child, that child's own
`fir_op_step` routinely needs to read back UP the tree — an ancestral (`&`-prefixed) search walks
`.parent()`/`home_brane()` from the child's position outward. If the ancestor's own state were
still "borrowed" for the duration of that recursive call, the child's read would either alias
against it or have to wait — either way, stepping would be broken by construction. It works
today, and needs to keep working here, but the *mechanism* that makes it work changes, and the
difference is worth stating precisely rather than assumed.

**How today's code manages this.** `fir_trait.rs`'s trait-level doc comment (lines 413–422, on
`FirRefExt`) states the rule directly: `step_inner` recurses on the **`Rc` handle**
(`this: &FirRef`), never on a `RefCell` borrow of the pointee, and "each `RefCell` borrow of the
pointee is opened and dropped within a single statement before any recursion." Concretely, in
the real function (`fir_trait.rs:458–491`):

```rust
let front = this.borrow().core().front_task();   // borrow opened, value cloned out, dropped
// … `this` is now completely unborrowed …
step_inner(&front_rc, &child_scope, depth + 1)?;   // recursing — nothing of `this` is held open
```

While the child recurses, the ancestor's `RefCell` is entirely free. When the child's
`fir_op_step` walks `.parent()` (a `Weak::upgrade()` followed by a fresh `.borrow()`), that
borrow succeeds because nothing else is holding the ancestor's cell open at that moment. Two
mechanisms do the work today, and they're doing two different jobs: the `Rc` reference count
keeps the ancestor *alive*; the open-then-drop `RefCell` discipline keeps it *unborrowed* during
the descent.

**The arena-era restatement of the same rule.** There is now one `FVMStorage`, not one `RefCell`
per node, so `&mut FVMStorage` is a single all-or-nothing borrow rather than many independent
ones. The equivalent discipline is the same shape, just relocated: `storage.get(ptr)` and
`storage.with_mut(ptr, ...)` must each return or consume their borrow within one statement — and
because Rust's non-lexical lifetimes drop a borrow as soon as its last use ends, `storage` is
fresh and unborrowed again by the time a nested `step_inner(front_ptr, storage, …)` call happens.
That's exactly what the transcribed `step_inner` above already does: `storage.get(ptr)…` ends
its borrow at the end of its own statement, so the recursive call receives an unencumbered `&mut
FVMStorage`, and the child's own `child_ptr.get_parent(storage)` — or any ancestral search
reaching back through `storage` — is a brand-new, entirely legal borrow of the same value.

**Where this is stricter than today, not merely equivalent to it.** It would be reassuring to
say the arena carries the same safety property as `RefCell` did, and stop there — but that
undersells a real difference worth naming plainly. Today's discipline is enforced *per node*, at
*runtime*, by `RefCell`'s panic: a violation panics inside the one offending node's `borrow()`
call, which is easy to localize to a specific test. Under the arena, `&mut FVMStorage` is *one*
shared resource for the entire tree: if *any* frame anywhere in a recursive call stack holds a
live `&Fir`/`&mut Fir` (via `storage.get`/`with_mut`, or via a live `FirCursor`/`FirCursorMut`)
across a nested `step_inner` call, the code simply **does not compile** — a strictly stronger
guarantee than a runtime panic risk, caught at build time for every call path rather than only
the ones a test happens to exercise. The trade is in *diagnosability*, not safety: a compile-time
borrow-checker error at a deeply nested recursive call site can be less immediately obvious about
which *logical* operation was the culprit than "this specific test panicked" is today.

**The concrete implication for `FirCursorMut`.** It must never be held live across a call into
`step_inner`. Leaf mutating operations that do not themselves trigger a recursive step —
`push_ubc_child`, `set_nyes`, `push_search_result` — are exactly what it is for, and using it
there is safe by construction. But a `fir_op_step` implementation must never hold a
`FirCursorMut` open *while also* recursing into a child's `step`: that would be the identical
hazard `RefCell` catches today, just caught by the compiler instead. This is a real, if modest,
discipline the migrated code has to follow when each `fir_op_step` body is *written* — read what
is needed, let the cursor drop, then mutate or recurse, never interleaved — not something that
falls out automatically merely because `FirCursorMut` exists. Getting it right is a net
improvement over today (a bug becomes a compile error everyone hits, not a panic that only
surfaces when the unlucky code path executes in some test), but it is not free: it requires the
discipline to actually be followed, the same way today's `RefCell` discipline requires the
open-then-drop pattern to actually be followed, just enforced one build earlier and one order of
magnitude more reliably.

### `dyn Fir` is kept

This FOOP changes the *pointer and ownership scheme*, not the *dispatch mechanism*. The crate's
existing uniform, kind-agnostic stepping over `dyn Fir` is a deliberate, already-justified use of
dynamic dispatch per `rust_instructions.md`'s construct-preference guidance ("types over
generics, generics over `dyn` — reach for dynamic dispatch only when you need it"); the
evaluator's stepping loop needs exactly this kind of uniform dispatch across heterogeneous FIR
kinds, so `dyn Fir` stays. `FVMStorage`'s slots hold `Fir` values (or `Box<dyn Fir>`/an enum
wrapping each kind, whichever the executing agent finds is the more natural fit once
`fir_kinds.rs` is actually being migrated — this is intentionally left as an implementation-time
decision, not fixed here, since it does not change any of `FVMStorage`'s external interface).

### Copy-migration strategy: `foolish-ubca2` as a new crate

This FOOP does **not** modify `foolish-ubca` in place. A new crate, `foolish-ubca2`, is created
as a literal copy and migrated internally, while `foolish-ubca` stays completely untouched and
frozen as the reference oracle for the FOOP's entire duration. `foolish-ubca`'s existing
`checked/`/`verified/` einmo baselines are never modified by this FOOP.

**Validation mechanism** (verified by reading the actual code, not assumed): `einmo::Evaluator`
is a two-line trait — `fn evaluate(&self, source: &str) -> Result<Vec<String>, String>`,
`Sync`-only — with zero dependency on any Foolish workspace crate (`foolish-ubca/Cargo.toml`
states this explicitly in its own comments). `einmo::TestConfig::new(work_dir, level)` binds a
suite to an arbitrary directory tree (`input/`, `output/`, `checked/`, `verified/`,
path-mirrored via `mirror_input_path`) with no awareness of which crate produced the `Evaluator`
passed to it. This pattern is already proven in the codebase: `zweimomo/src/evaluators.rs` wraps
`foolish_ubca::UbcaEvaluator` in a `UbcaEvaluatorAdapter: einmo::Evaluator` and cross-validates
it against `RustPythonEvaluator` and a Boa-based JS adapter, all run over the same einmo
fixtures, for cross-*language* validation. This FOOP applies the identical mechanism turned
toward cross-*implementation* validation: `foolish-ubca2` becomes a second Rust `Evaluator`, run
against the same einmo_suite input fixtures, with its `output/` compared against
`foolish-ubca`'s existing, already-`checked/` baselines as the correctness oracle. Equivalence is
proven case-by-case, incrementally, as the migration proceeds — not in one big-bang comparison at
the end.

**Non-regression invariant across two crates**: `foolish-ubca`'s `checked/`/`verified/` trees
are read-only for this FOOP's entire duration. `foolish-ubca2` gets its own
`output/`/`checked/`/`verified/` directories from Phase 0 onward, seeded by *copying* — never
regenerating — `foolish-ubca`'s existing `checked/`. "Done" for any given einmo case, at any
phase, means `foolish-ubca2`'s `output/` for that case is byte-identical to `foolish-ubca`'s
`checked/` (or `verified/`) for the same case, reviewed and promoted the normal einmo way but
exclusively into `foolish-ubca2`'s own `checked/` tree. This is a direct cross-crate extension of
the existing rule that a FOOP under development must not change the OUTPUT of any einmo test
belonging to a different, already-shipped FOOP: here, the "different, already-shipped" body of
work is the entirety of `foolish-ubca` itself.

**The `zweimomo` workspace gap.** `zweimomo` exists on disk (`zweimomo/Cargo.toml`,
`zweimomo/src/`) but is **not currently a workspace member** — the root `Cargo.toml`'s
`[workspace] members` lists only `foolish-parser`, `foolish-core`, `foolish-cli`, `foolish-ubca`,
`einmo`. `AGENTS.md` lists `zweimomo` as one of the "Crates of Foolish," so this is a real,
pre-existing discrepancy between documentation and workspace configuration, not something this
FOOP introduces. Phase 0 (see plan) defaults to keeping `foolish-ubca2`'s einmo wiring
self-contained — its own copy of `ubca_snapshot_tester.rs`-style adapter code, exactly mirroring
how `foolish-ubca` wires its own gate today — rather than fixing the `zweimomo` workspace gap as
a prerequisite. See Rejected Alternatives and Open Questions.

## FIR Impact

This FOOP changes how every existing FIR kind is stored and addressed; it adds no new FIR
variant and changes no FIR semantics. Concretely, in `foolish-ubca2` only:

- Every FIR kind's `ProtoBrane`-embedded `foolish_children: Vec<FirRef>`, `ubc_children:
  RefCell<Vec<FirRef>>`, and `parent: Weak<RefCell<dyn Fir>>` fields are replaced with
  `FirPointer`-based equivalents backed by `FVMStorage` (parent/children become `FirPointer`
  values read from the arena, not fields duplicated onto each `Fir` value beyond its own
  identity).
- Every `Rc::new_cyclic` construction site (45 in `fir_kinds.rs`, 18 in `compiler.rs`, 1 in
  `proto_brane.rs`) is replaced with a `FirPointer::create_child` call.
- `constanic_clone_at`'s recursive subtree-recoordination logic is replaced by
  `FVMStorage::clone_subtree`.
- `FirRef`, `FirRefExt`, `FirRefNavExt` are removed once nothing references them (Phase 5 of the
  plan).

No new NYES states, no new terminal states, no change to which FIR kinds exist or what values
they carry.

## UBC Step Impact

Step *rules* are unchanged — this FOOP is purely a storage-mechanism change. What changes is how
a step reads and writes FIR state during evaluation: `evaluator.rs`'s stepping loop's
`.borrow()`/`.borrow_mut()`/`Weak::upgrade()` call sites are replaced with `FVMStorage::get`/
`FVMStorage::with_mut` calls. No interaction with constanic coordination semantics is
introduced; the coordination *logic* itself is unchanged, only the pointer operations it's
expressed in terms of.

## Test Plan

- `foolish-ubca2`'s own einmo suite (`foolish-ubca2/einmo_suite/`), seeded in Phase 0 by copying
  `foolish-ubca`'s `input/` and `checked/`, is the primary correctness oracle throughout. Every
  phase ends with a full `einmo_gate_checked` run against this suite; higher-risk sub-tasks
  (Phase 2, the search engine) additionally run a targeted subset after each individual task.
- Existing `foolish-ubca` unit tests (the `*_nyes_transitions` tests, the `ContextfulSearch`
  engine tests in `fir_kinds.rs`) get copied into `foolish-ubca2` in Phase 0 and must continue to
  pass against the migrated code at every phase gate — they exercise internal FVM state (NYES
  transitions, search predicate matching) that einmo's black-box snapshot comparison does not
  directly pin down.
- A comprehensive snapshot test, `foolish-ubca2/einmo_suite/input/foop/16/comprehensive.foo`, is
  written after all implementation phases (Phase 6). Because this FOOP is an internal
  representation change rather than new language surface, this test's job is to exercise a wide,
  representative cross-section of *existing* feature combinations (nested branes, contexted
  operators, value search, combined name+value forms, head/tail, AB/IB recoordination via a
  named-brane reference) through `foolish-ubca2`, and its promotion gate compares against
  `foolish-ubca`'s output for the same input rather than against a from-scratch expectation.
- `foolish-ubca`'s own suite is re-run, unmodified, at the start and end of the FOOP purely as a
  sanity check that it was never touched — not as a test *of* this FOOP's changes.

## Rejected Alternatives

### A. In-place phased migration of `foolish-ubca` itself

Migrate `foolish-ubca` directly, phase by phase, rather than copying to a new crate. Rejected
because it would require `foolish-ubca`'s own `checked/` baselines to be mutated
mid-migration, violating the non-regression invariant that governs all FOOP work, and — more
importantly — it would leave no working reference oracle to diff against if a later phase
silently regressed something an earlier phase had already "migrated." The copy-migration
strategy keeps `foolish-ubca` frozen and always available as ground truth for exactly this
reason.

### B. Do nothing

Leave `foolish-ubca` on `Rc<RefCell<dyn Fir>>`/`Weak` indefinitely. Rejected because the
construction-boilerplate cost is real and quantified: 68 manual `Rc::new_cyclic` sites, each
requiring the author to separately and correctly wire a self-`Weak` reference and a parent-side
child-list push, with no compiler-enforced link between the two steps. `constanic_clone_at`'s
recursive re-wiring is one of the largest and most delicate blocks of pointer-manipulation code
in the crate as a direct consequence.

### C. Host `foolish-ubca2`'s einmo adapter in `zweimomo`

Fix the `zweimomo` workspace-membership gap as a Phase 0 prerequisite and place
`Ubca2EvaluatorAdapter` there, consistent with `zweimomo`'s stated purpose as the home for
cross-implementation `Evaluator` adapters. Rejected as the *default* for this FOOP — not because
it is architecturally wrong, but because it bundles an unrelated pre-existing workspace-hygiene
fix into this FOOP's Phase 0 as a hard prerequisite, raising this FOOP's blast radius for no
correctness benefit. The self-contained default (a `foolish-ubca2`-local adapter, mirroring
`foolish-ubca`'s own `ubca_snapshot_tester.rs`) achieves the same validation mechanism without
that dependency. Fixing the `zweimomo` gap and relocating the adapter there remains available as
later, separate cleanup — see Open Questions.

## Open Questions

- **`foolish-ubca`'s eventual fate.** Once `foolish-ubca2` is complete and its own `checked/` is
  fully promoted (Phase 6), does `foolish-ubca` stay as a permanent frozen reference
  implementation, get formally deprecated with a pointer to `foolish-ubca2`, or get removed
  entirely? Each has real tradeoffs (keeping both means double-maintaining two evaluators if new
  language features land during or after the migration; removing `foolish-ubca` early loses the
  oracle before it can be leaned on for a regression bisect). Not decided by this FOOP.
- **Is "`foolish-ubca2`" the permanent crate name, or a placeholder?** Renaming after Phase 0
  means a second Cargo.toml/import churn pass across the new crate — a naming decision before
  Phase 0 starts is cheaper than one after.
- **The `zweimomo` workspace-membership gap** — fixed as part of this FOOP (Rejected Alternative
  C) or tracked as separate, unrelated cleanup? Defaults to "separate" (see Specification) but is
  not finally decided here.

## References

- Prior design discussion: this FOOP's specification and plan were developed in an interactive
  session covering the `FirPointer`/`FVMStorage` design, the `create_child` calling convention,
  and the einmo/`zweimomo` validation mechanism, prior to being written down here.
- `rust_instructions.md` §1b.4 ("make illegal states unrepresentable") — grounds `FirPointer`'s
  validated-handle design.
- `foolish-ubca/Cargo.toml` — confirms `einmo`'s zero-dependency-on-Foolish design.
- `zweimomo/src/evaluators.rs` — the existing, proven `Evaluator`-adapter-over-shared-fixtures
  pattern this FOOP reuses.
- Code locations: `foolish-ubca/src/fir_kinds.rs`, `foolish-ubca/src/compiler.rs`,
  `foolish-ubca/src/proto_brane.rs`, `foolish-ubca/src/evaluator.rs`,
  `foolish-ubca/src/ubca_snapshot_tester.rs`.
