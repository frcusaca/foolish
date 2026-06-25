---
foop: 26
title: UBCa — Two-Store ProtoBrane Tree and Uniform Two-Phase Stepping
author: Atlas <hc.busy@gmail.com>
status: Brewing
type: Standards
created: 2026-06-09
phase: phase-2
supersedes: []
---

# FOOP-62: UBCa — Two-Store ProtoBrane Tree and Uniform Two-Phase Stepping

> **WORKTREE.** This FOOP is implemented in its own worktree:
>
> ```
> WORKTREE_ORIGIN_BRANCH=alpha
> WORKTREE_ORIGIN_PATH=$(pwd)
> WORKTREE_BRANCH_NAME=foop-62-ubca-mimo
> WORKTREE_FULL_FS_PATH=${HOME}/tmp/foolish-worktrees/foop-62-ubca-mimo
> ```
>
> All paths below are relative to that directory's `foolish/` workspace unless stated
> otherwise.

(@human: the filename digits `62` are the identifier — "FOOP 62" in prose. The
`foop: 26` frontmatter is the little-endian sort key, per the FOOP numbering
convention. Verified with `foop_check.py gen_next`.)

## Abstract

Today every FIR node is one of ~10 concrete structs behind a ~50-method `Steppable`
god-trait, each hand-reimplementing tree topology (`state`, `parent`, `children_itr`,
`children_mut`) and storing children in idiosyncratic, differently-typed fields
(`operands`, `anchor`/`target`, `elements`/`merged`, `statements`). FOOP-62 replaces
this with a single uniform container — the **ProtoBrane** — that holds children in
**two provenance-separated stores**: `foolish_children` (parse-derived, fixed-shape) and
`ubc_children` (computation-derived, mutable). Topology and the stepping recursion are
written **once** as defaults; each FIR kind supplies only its own per-node work
(`fir_op_step`) and its leaf data. We build this as a **new crate/module `UBCa`** by cloning
the existing UBC interface and tests, gutting the implementation, and rebuilding on the
new structure. UBCa is **its own source of truth**: it is validated **byte-for-byte against
its own approved snapshot corpus** (`foolish-ubca/snapshot_tests/`). UBC may be consulted
informally during development, but UBCa is **NOT** required to match UBC's output — UBC's own
snapshots have drifted from its evaluator and UBC is no longer an authoritative oracle.

## Terminology: NYES states in UBCa

UBCa classifies NYES states into three categories:

**Pre-constanic (nigh)** — more evaluation/stepping needed:
- PREMBRYONIC, EMBRYONIC, BRANING

**Constanic** — context-dependent terminal (WOCONSTANIC/ECONSTANIC):
- **ECONSTANIC**: search performed, nothing found. May gain value via recoordination.
- **WOCONSTANIC**: Waiting On CONSTANICs — all searches found, but dependencies are themselves constanic.

**Constantew** — constant everywhere (CONSTANT/INDEPENDENT/NK):
- **CONSTANT**: Fully evaluated — a genuine value.
- **INDEPENDENT**: Self-contained constant — no context dependencies.
- **NK**: Not Knowable — provably unfindable (`???`). Terminal.

**Classification priority** (worst wins): NK > WOCONSTANIC/ECONSTANIC > CONSTANT > INDEPENDENT

**Predicates:**
- `is_settled()` = all terminal states (constanic + constantew)
- `is_constanic()` = ECONSTANIC, WOCONSTANIC, CONSTANT, INDEPENDENT, or NK (all terminal states; constantew ⊂ constanic)
- `is_constantew()` = CONSTANT, INDEPENDENT, or NK (constant everywhere)
- `is_nnk_constanic()` = constanic but NOT NK — for code that needs "constanic but not NK" (e.g., search results that propagate NK separately)
but it is settled so it does not block the task queue.

> **Terminology note (2026-06-23):** The words "freeze" and "frozen" are **deprecated** in
> this spec and in code. Use instead:
> - **constanic** — a FIR in any terminal NYES state (wouldn't change unless context changes)
> - **constanew** — a FIR that won't change no matter what (included in "constanic")
> - **non-constanew constanic** — a FIR whose value may change when context is recoordinated
> - **"step to constanic"** — when a FIR progresses through stepping
> - **"set NYES to constanic"** — when NYES is directly assigned
> - **"clone with constanic NYES"** — when cloning preserves constanic state

## Terminology: ignorance — normally ignorant, foolishly ignorant, fully foolish

Every clone and every premembryonic construction in UBCa happens with a *degree of
ignorance* — how much of its surroundings the resulting FIR is permitted to see and
re-resolve. There are exactly two **modes of `constanic_clone`** (normal and foolishly
ignorant) plus one **construction** behavior (fully foolish). The whole spec uses these three
words for them.

`constanic_clone` is **one function** carrying a boolean mode flag, named very descriptively:

```text
constanic_clone(source, new_parent, descendent_of_sfm_and_foolishly_ignorant: bool = false)
```

**Two independent recursions, one flag bridged between them.** The `step()` recursion carries
a `Scope` (parent, line number, …) which also holds **`has_ancestral_sfm: bool`** — true when
the current evaluation is inside an SF-mark's RHS. The `constanic_clone` recursion is *separate*
and carries its own `descendent_of_sfm_and_foolishly_ignorant` parameter. They connect thus:
- **When `step()` calls `constanic_clone`**, it passes `descendent_of_sfm_and_foolishly_ignorant
  = scope.has_ancestral_sfm`.
- **When `constanic_clone` calls itself recursively**, the child inherits the **caller's**
  `descendent_of_sfm_and_foolishly_ignorant` (it does NOT re-read any scope — clone recursion
  propagates its own flag).

**Normally ignorant** — `descendent_of_sfm_and_foolishly_ignorant == false` (the default).
A FIR cloned *normally ignorant* is **not blind**: it sees its new surroundings and
re-resolves there. The clone's NYES is set by a single precise rule:

> **NYES-transfer rule (normal mode).** *Constanic* NYES are transferred **unchanged** to the
> clone; *pre-constanic* states (PREMBRYONIC/EMBRYONIC/BRANING) are transferred as
> **PREMBRYONIC** to the clone.

Concretely, applying that rule:
- CONSTANT / INDEPENDENT children are already resolved (constanic everywhere): their NYES
  transfers **unchanged**, so they are effectively **referenced** rather than re-resolved.
- ECONSTANIC / WOCONSTANIC / NK children are constanic-in-context: their NYES **also
  transfers unchanged** — an ECONSTANIC clone stays ECONSTANIC, a WOCONSTANIC clone stays
  WOCONSTANIC. (Recoordination may later let an ECONSTANIC clone find a value in its new
  AB/IB, but that happens through ordinary stepping, NOT by pre-resetting it at clone time.)
- PREMBRYONIC / EMBRYONIC / BRANING children (pre-constanic) are transferred as
  **PREMBRYONIC**, so their searches re-run against the new AB/IB. This is ordinary
  recoordination (FOOP-7).

**Foolishly ignorant** — `descendent_of_sfm_and_foolishly_ignorant == true`. In this mode
the clone is *foolish*: **ALL NYES are copied unchanged** — constanic AND pre-constanic alike,
with **no reset to PREMBRYONIC**. The clone stubbornly keeps every node exactly as it was
found instead of re-resolving anything.

> **When the flag is set.** Processing the **RHS of an assignment whose RHS carries an
> SF-mark (`<…>`)** sets **`scope.has_ancestral_sfm = true`** on the `step()` Scope, which is
> carried down through the step recursion. Each `constanic_clone` invoked from `step()` is then
> passed `descendent_of_sfm_and_foolishly_ignorant = scope.has_ancestral_sfm`, and clone's own
> recursion **propagates that flag downward** to every descendant cloned while building that
> constanic RHS value. (Hence the names: Scope's `has_ancestral_sfm`; clone's
> `descendent_of_sfm_and_foolishly_ignorant`.) This is how an SF clones its result with constanic NYES at
> assignment time.

> **⚠ THE VERY BIG BUT — a later search of an SF-mark.** When a *later search* resolves to an
> **SF-mark node** and calls `constanic_clone` on it, that clone:
> 1. **automatically STRIPS the SF-mark** (the `StayFoolish` wrapper is removed — the clone is
>    the inner expression, not a re-wrapped SF node), AND
> 2. **does NOT set `descendent_of_sfm_and_foolishly_ignorant`** — it proceeds in **normal
>    mode**. So an ECONSTANIC inner value **re-resolves** in the new context via ordinary
>    stepping, exactly as any normal clone would.
> This is the asymmetry that has caused repeated confusion: building an SF's RHS is foolish
> (everything cloned with constanic NYES), but consuming an SF later via search is normal (mark stripped, normal
> NYES-transfer, re-resolution allowed).

**Fully foolish construction** — the *premembryonic construction* behavior used for an
SFF-marker (`<<…>>`). Here we say: **"this is fully foolish construction of the FIR tree
from here onward."** The SFF-marked expression is constructed with **all descendant search
FIRs instantiated directly as ECONSTANIC** (built that way at construction; nothing is
cloned, because no search ever runs). The marker is then immediately settled from its
child's NYES (e.g. `<<1+1>>` → CONSTANT, `<<a+b>>` → WOCONSTANIC). Fully foolish is a
*construction-time* property, never a clone — SFF executes zero searches, so there is
nothing to clone.

| word | flag / when | act | NYES handling |
|---|---|---|---|
| **normally ignorant** | clone flag = false (default); `scope.has_ancestral_sfm = false` | clone | constanic unchanged; pre-constanic → PREMBRYONIC |
| **foolishly ignorant** | clone flag = true, seeded from `scope.has_ancestral_sfm` while building SF-marked RHS; propagates through clone recursion | clone | **ALL NYES copied unchanged** |
| **(BUT) later search of an SF-mark** | clone flag = false | clone, **mark stripped** | normal rule (re-resolves) |
| **fully foolish** | SFF `<<…>>` premembryonic | construction | descendants built ECONSTANIC; no clone |

## Terminology: anchor and result (NOT "target")

A *search* FIR — and the kinds we classify as searches: plain/dot/regexp **search**,
**Index** (`#`/seek), and **HeadTail** (head/tail) — relates to two distinct other FIRs.
Name them precisely; the word **"target" is BANNED** because it ambiguously meant either one.
(This restates definitions that recur across FOOPs, and records this revision's correction
away from "target".)

- **anchor** — the FIR a search searches **within / relative to**. Example: in `data#5`, the
  `data` brane is the anchor (we index *into* it). An unanchored search (e.g. a bare name) has
  no explicit anchor; it searches its surrounding branes (IB/AB — see §10). The anchor is a
  **non-result** item in the sequencer rendering.
- **result** — the FIR the search **produces** (what it *found*): the resolved value/brane for
  a search, the indexed element for Index, the head/tail element for HeadTail. The result is
  rendered as `result=…` and lives in the search FIR's `ubc_children`. Existing searches are
  **singular-result** (at most one `ubc_children` entry — see §8); an out-of-bounds / not-found
  search settles NK with **no** result.

> **Correction (this revision): use `anchor` and `result`, never `target`.** Earlier code
> named the search-result field `target`, which was ambiguous (it could be read as the anchor).
> All occurrences were renamed to `result` (the produced FIR) or `anchor` (the searched-within
> FIR) per actual use; `target` no longer appears in the UBCa FIR vocabulary.

## Motivation

### The problem today

`foolish-core/src/fir.rs` (≈3500 lines) carries:

- A **fat `Steppable` trait** (lines ~578–742): ~50 methods, ~40 of them defaulted
  no-op accessors (`search_target_ref -> None`, `index_offset -> 0`,
  `set_concat_merged -> {}`) that only one variant ever overrides. Violates the
  "small, capability-named traits" rule in `rust_instructions.md`.
- **Duplicated topology**: 11 structs each declare `state: Nyes` + `parent: ParentPtr`
  (22 field decls for 2 concepts) and each hand-implements identical `state()`
  (`self.state`), `set_state()`, `get_parent()` (`self.parent.upgrade()`) — confirmed
  byte-identical across all variants.
- **Irregular child storage** requiring a 6-arm `ChildrenItr` enum to paper over, plus
  an `unsafe` pointer-projection arm (`fir.rs:567`) for `NormalBrane` statement bodies,
  plus three parallel child-access methods (`children_itr`, `children_mut`,
  `fir_children` — the last clones the whole node just to list children).
- A **second fat trait `FirQueryable`** (~12 methods) mirroring the variants again for
  the humanizing sequencer.
- Pervasive `clone_into_fir()` calls used purely to pattern-match a `dyn Steppable`
  (e.g. `has_unresolved_forward_refs` clones 4× in one function, `ubc.rs:191–225`).

### The world after

- **One container, ProtoBrane**, with two child stores. Topology written once.
- The only mutation to tree *shape* is push/clear on `ubc_children`; the parse-derived
  structure (`foolish_children`) is immutable.
- A FIR kind implements only `fir_op_step()` (its own combining step) + leaf-data accessors
  + `kind()`; stepping itself is the shared task-list drain (§3).
- This makes structural what AGENTS.md already states semantically (the "Foolish
  Semantic Immutability vs FIR Evaluation State" section): `foolish_children` **is** the
  fixed Foolish meaning; `ubc_children` **is** the evolving evaluation record. Mutating
  the source meaning becomes unrepresentable in the type, not merely discouraged.

## Specification

### 1. The two child stores

**Composition model (resolves reviewer ambiguity).** One shared field-holder struct and one
dyn-dispatch trait:

- **`struct ProtoBrane`** — the shared field-holder (children stores, parent, nyes, tasks).
  All topology code lives as **inherent methods** on this struct (written once, not
  overridable, dyn-safety irrelevant). Every Fir kind *contains* one of these as a field
  named `core`.
- **`trait Fir`** — the dyn-dispatch surface. `FirRef = Rc<RefCell<dyn Fir>>` is how nodes
  are stored and passed. `trait Fir` provides `core(&self) -> &ProtoBrane` /
  `core_mut(&mut self) -> &mut ProtoBrane` (so callers reach topology through the handle)
  plus `fir_op_step`, `kind`, and narrow leaf-data accessors. This is the **only** thing
  that is dyn-dispatched; different kinds (BraneFir, SearchFir, OperatorFir…) implement
  `trait Fir` with their own `fir_op_step`. The shared stepping logic (`step_fir_ref`)
  is a **free function over `&FirRef`**, not a trait method.

Each kind (`OperatorFir`, `SearchFir`, …) **contains** a `ProtoBrane` and adds its own leaf
data; it implements `trait Fir` by returning `&self.core`/`&mut self.core` and overriding
`fir_op_step` with its kind-specific combining work. Stepping (`step_fir_ref`) is the same
free function for every kind — kinds differ through *construction-time state* and
`fir_op_step`, not by overriding the step function.
In particular **SF/SFF do NOT need a custom step path**: their wrapped child is *constructed*
in the right state (and recoordinated via `constanic_clone` when found as a search result)
such that the normal `step_fir_ref` drain produces the SF/SFF semantics.
The old `Fir` *enum* + `clone_into_fir` match-dispatch is retired — `dyn Fir` virtual
dispatch replaces it. The `bon` builders build the concrete kind structs wrapped into
`Rc<RefCell<dyn Fir>>` by `build()`.

```rust
/// Shared field-holder. Every Fir kind contains one of these as `core`.
pub struct ProtoBrane {
    /// Parse-time children: created by reading Foolish source, before any stepping.
    /// FIXED: the vector never grows or shrinks and no Rc slot is ever re-seated
    /// once built. The referenced Fir structs DO step and compute in place within
    /// this same vector (interior evolution), but the topology here is constanic.
    foolish_children: Vec<FirRef>,                // FirRef = Rc<RefCell<dyn Fir>>

    /// Compute-time children: FIR created unavoidably during evaluation —
    /// search results, operator results (usually constants), concatenation result
    /// branes. This vector may EXPAND and SHRINK during computation
    /// (push on production, clear/truncate on re-step). Currently holds at most one
    /// element, but is a Vec for uniformity and an iterator interface.
    ///
    /// ORDER IS SIGNIFICANT. The sequencer renders these as the labeled `result=`
    /// item(s), positioned distinctly from (and before) the foolish_children, with
    /// exact comma/indent rules. Push order is therefore part of the observable
    /// output: results MUST be pushed in the order the sequencer expects to render
    /// them. This is not an internal-only ordering — it is snapshot-visible.
    ubc_children: Vec<FirRef>,

    /// Evaluation state (NYES) of THIS node. Single source of truth for the node's NYES.
    /// READ-ONLY from outside (public `get_nyes()`; no public setter — no other member or
    /// caller can change it). Written ONLY by the Fir itself, via exactly three sanctioned
    /// writers: (1) initialization (builder), (2) stepping (the node's own `fir_op_step`),
    /// (3) constanic clone (builder-from-value, §6b). See the NYES CONTRACT note below.
    nyes: Nyes,

    /// Task list for NYES-driven stepping (§3). A `VecDeque<FirRef>` of children to drain
    /// to constanic, in order; the node's own `fir_op_step` runs when it empties (and may
    /// push more tasks). Built during `Embryonic`. Transient evaluation state — not parsed,
    /// not serialized.
    tasks: VecDeque<FirRef>,

    /// Weak back-link to the parent Fir node (see §5). Used for ancestral name
    /// resolution. NON-optional: the root node's parent is a Weak pointing at
    /// itself, detected via `is_root()`. Weak (not strong) so parent links never
    /// form Rc cycles — children are owned strongly downward, parents referenced
    /// weakly upward (the graph-not-tree decision from FOOP-52).
    ///
    /// IMMUTABLE after construction. There is no parent setter. The only way to
    /// give a node a different parent is to CLONE it with the new parent
    /// (clone-with-update) — the detach-and-recoordinate semantics already used
    /// when a brane is referenced by name (see AGENTS.md "Detachment and
    /// Coordination"). This makes a node's structural position fixed for its
    /// lifetime; re-positioning produces a new node.
    parent: Weak<RefCell<dyn Fir>>,
}
```

**Invariant (enforced by the type, not by convention):** `foolish_children` has no public
mutator. The shared topology code lives as **inherent methods on `ProtoBrane`** (written
once, not overridable by kinds, dyn-safety not a concern because these methods are never
called through a trait object). The only public topology mutators touch `ubc_children`:

```rust
impl ProtoBrane {
    // --- read-only child access (no RefMut guards returned) ---
    pub fn foolish_children(&self) -> &[FirRef] { &self.foolish_children }
    pub fn ubc_children(&self) -> &[FirRef]     { &self.ubc_children }
    /// All children in render order: ubc first (result=), then foolish.
    pub fn all_children(&self) -> impl Iterator<Item = &FirRef> {
        self.ubc_children.iter().chain(self.foolish_children.iter())
    }

    // --- ubc mutation (the ONLY public topology mutators) ---
    pub fn push_ubc_child(&mut self, child: FirRef)  { self.ubc_children.push(child); }
    pub fn clear_ubc_children(&mut self)              { self.ubc_children.clear(); }

    // --- task queue (stepping internals, pub(crate)) ---
    pub(crate) fn front_task(&self) -> Option<FirRef> { self.tasks.front().cloned() }
    pub(crate) fn pop_front_task(&mut self)            { self.tasks.pop_front(); }
    pub(crate) fn push_task(&mut self, t: FirRef)      { self.tasks.push_back(t); }

    // --- parent / root ---
    /// Upgrade the parent Weak. Returns None only during teardown (root Rc dropped).
    pub fn parent(&self) -> Option<FirRef> { self.parent.upgrade() }
    /// Root iff parent upgrades to a node that is pointer-equal to the node itself.
    /// Caller passes `self_rc` (the node's own Rc) because ProtoBrane has no
    /// back-pointer.
    pub fn is_root(&self, self_rc: &FirRef) -> bool {
        self.parent.upgrade().map_or(false, |p| Rc::ptr_eq(&p, self_rc))
    }

    // --- nyes (READ-ONLY from outside; no public setter) ---
    pub fn get_nyes(&self) -> Nyes { self.nyes }
    /// PRIVATE writer — only fir_op_step and the builder call this.
    fn set_nyes(&mut self, n: Nyes) { self.nyes = n; }
}

// NYES CONTRACT (encapsulation):
//   * From OUTSIDE a Fir, `nyes` is READ-ONLY — `core().get_nyes()` is the only public
//     accessor. There is NO public setter.
//   * The ONLY writers are: (1) initialization (builder), (2) the node's own
//     `fir_op_step` (via `core_mut().set_nyes()`), (3) constanic clone (§6b).
//   * `set_nyes` is private to `ProtoBrane`; no external caller can call it.
```

The `trait Fir` (per-kind behavior) therefore needs only **dyn-safe methods**:
`core()`/`core_mut()`, `fir_op_step()`, `kind()`, and narrow leaf-data accessors.
All shared topology is reached as `node.core().parent()`, `node.core().front_task()`, etc.
— concrete, inherent, written once, not overridable by kinds.

(@human: this matches the `Links`-struct pattern in the reference tree example: shared
topology fields live in a helper struct with inherent methods; the trait only exposes
`fn links(&self) -> &Links` and per-kind behavior. The mutable-children **iterator** is
intentionally NOT provided — returning `RefMut` guards over `Rc<RefCell<_>>` slots risks a
runtime aliasing panic. Children are *read* via slice refs, and a node mutates by stepping
children one at a time via `step_fir_ref` and pushing/clearing its own `ubc_children`.)

### 2. Every node is a ProtoBrane

The ~10 distinct storage shapes collapse to ONE:

| Node kind        | foolish_children            | ubc_children (on compute) | Leaf data on payload        |
|------------------|-----------------------------|---------------------------|-----------------------------|
| Brane            | statement members (w/ name/line) | —                    | characterizations           |
| Operator (`Plus`…) | operands                  | `[reduced value]`         | op name                     |
| Search           | `[anchor]` (if anchored)    | `[found value]`           | pattern, direction, anchored|
| Index            | `[anchor]` (if anchored)    | `[found value]`           | offset, anchored            |
| HeadTail         | `[anchor]` (if anchored)    | `[found value]`           | is_head, anchored           |
| StayFoolish/SFF  | `[wrapped expr]` (len 1)    | —                         | (marker only)               |
| Concatenation    | `k` input branes            | `[result brane]` (the k+1th) | —                        |
| Statement        | `[body]` (len 1)            | —                         | name, line_number           |
| ConstantInt / Nk | (empty)                     | —                         | value / reason+alarm        |

Notes:
- **Single-child nodes are just length-1 ProtoBranes** (SF/SFF wrapper, statement body).
- **Leaves are length-0 ProtoBranes.** No separate "leaf" storage shape.
- **Concatenation** holds its `k` inputs in `foolish_children`; when all `k` are
  constanic their elements are constanically cloned into the result brane, which is the
  `k+1`th child and lives in `ubc_children`.
- **Brane statements are iterable directly from `foolish_children`**; each statement
  member carries its own `name`/`line_number` (the metadata travels with the member —
  there is no parallel metadata vector to keep in sync). `search_ib`/`search_ab`
  iterate `foolish_children`.

### 3. NYES-driven stepping via a task list

Stepping is **NYES-driven**, not "wait-for-all-children-then-run." Each `step()` advances
work by **one NYES transition** and **returns the node's NYES** so the caller sees progress.
The mechanism is a **task list**.

#### 3.1 The task list

A `ProtoBrane` carries a task list — a **`std::collections::VecDeque<FirRef>`** (the standard
library's double-ended queue; the simplest standard structure that fits — `pop_front` on
completion, `push_back` to enqueue results). **A task is just a Fir**: it is stepped until it
is constanic. The language guarantees every Fir reaches a constanic NYES in **finite time**,
so draining the queue always terminates (the outer `max_steps` guard is a belt-and-suspenders
check for implementation bugs, not a semantic necessity).

**Stepping mutates the task *items*, not via a held borrow into the deque.** Two distinct
mutations happen and must not be conflated:
- the **deque** is mutated structurally (`pop_front` when a task completes, `push_back` when
  `fir_op_step` enqueues a result) — this is `core_mut().pop_front_task()` / `push_task()`;
- the **item** a task points to is mutated *through its own `RefCell`*
  (`task_rc.borrow_mut().fir_op_step(…)`), which is exactly why tasks are
  `Rc<RefCell<dyn Fir>>` — a task can be stepped while it still sits in the queue.

**The borrow discipline: read neighbors into locals (borrows dropped), then write.**
This is the same pattern used throughout `Rc<RefCell<_>>` trees. For a node step:

```rust
// Correct: collect what you need under a short borrow, drop it, then act.
fn step_fir_ref(this: &FirRef, scope: &Scope) -> Result<StepReport, UbcError> {
    // 1. Peek and clone the front handle — borrow of `this` dropped here.
    let front = this.borrow().core().front_task();
    match front {
        Some(front_rc) => {
            if front_rc.borrow().core().get_nyes().is_settled() {
                this.borrow_mut().core_mut().pop_front_task();   // pop: one action
            } else {
                step_fir_ref(&front_rc, scope)?;                 // recurse on handle
            }
            Ok(StepReport::Progress(this.borrow().core().get_nyes()))
        }
        None => {
            // No front task → run this node's own combining work.
            this.borrow_mut().fir_op_step(scope)?;               // virtual dispatch
            Ok(StepReport::Progress(this.borrow().core().get_nyes()))
        }
    }
}
```

**Why this is safe:** at no point is `this` borrowed (shared or exclusive) while a
descendant is also borrowed. The `front_rc` handle is cloned (`Rc::clone` = refcount bump,
not a live borrow), the original borrow is dropped, and *then* `step_fir_ref` is called
recursively. When `fir_op_step` runs and does ancestral resolution
(`this.borrow().core().parent().upgrade()?.borrow()…`) there is no live borrow on any
ancestor — each level released its borrow before recursing. The `RefCell` aliasing rule is
respected at every level. (Confirmed experimentally: the equivalent nested-`borrow_mut`
shape panics `RefCell already mutably borrowed`; the transient-borrow shape does not.)

**One action per `step_fir_ref` call (check-then-act).** A finished front is popped on its
**own** call — not bundled with the step that finished it — keeping one clean action per
call. The outer driver loops `step_fir_ref(root, scope)`; the "drain front until settled,
pop, move to next" behavior is emergent across those calls, not an inner loop inside one.

Conceptually the list is "all my children, then my own combining work":

```
tasks  ≈  [ foolish_child_0, …, foolish_child_k ]   then   (my own fir_op_step)
```

The children are explicit `FirRef` entries. The node's **own** work is *not* a self-`FirRef`
entry (that would be an `Rc` self-cycle and a borrow-reentrancy hazard); instead it is the
**drain-completion action** run when the child-task queue empties — `fir_op_step`. That own
work may **push more tasks** (the result Firs it produces — search results, the concat result
brane), which then drain like any other before the node settles.

The queue is **built during `Embryonic`** (seed it with the foolish_children tasks). It may
grow as `fir_op_step` pushes results, but otherwise the node just follows the list until done.

#### 3.2 `StepReport` and `step_fir_ref`

```rust
/// Step report: either no progress (outer loop may stop), or progress with the
/// node's current NYES after the action. Progress is reported even if the NYES
/// value is identical to the previous call (e.g. a child popped without changing
/// the parent's NYES) — the caller must not use NYES-unchanged as a stop condition.
pub enum StepReport {
    NoProgress,
    Progress(Nyes),
}

pub trait Fir {
    fn core(&self) -> &ProtoBrane;
    fn core_mut(&mut self) -> &mut ProtoBrane;

    /// The node's OWN combining work, run once the child tasks are drained. ONE action.
    /// Reads neighbors into locals (borrows dropped) before writing — see §3.1 discipline.
    /// May push result tasks (and ubc_children) and advance the node's own nyes via
    /// `core_mut().set_nyes(…)`. Default: Woconstanic/Constant/Nk classification (§3.3).
    fn fir_op_step(&mut self, scope: &Scope) -> Result<(), UbcError>;

    fn kind(&self) -> FirKind;
    // ... narrow leaf-data accessors per kind (NOT the old 50-method surface) ...
}

/// The shared step function — written ONCE, called as a free function over a &FirRef.
/// This is NOT a trait method: it takes `this: &FirRef` so the borrow is transient
/// and dropped before any recursive call or fir_op_step invocation, preventing the
/// RefCell aliasing panic that a nested `borrow_mut`-across-recursion shape would cause.
///
/// ONE action per call (check-then-act). The outer driver loops `step_fir_ref(root, scope)`.
pub fn step_fir_ref(this: &FirRef, scope: &Scope) -> Result<StepReport, UbcError> {
    // Read what we need; borrow of `this` ends here.
    let front = this.borrow().core().front_task();   // Option<FirRef> — Rc clone, not a borrow
    match front {
        Some(front_rc) => {
            if front_rc.borrow().core().get_nyes().is_settled() {
                this.borrow_mut().core_mut().pop_front_task();   // pop: this call's one action
            } else {
                step_fir_ref(&front_rc, scope)?;                 // recurse on the handle; `this` not borrowed
            }
            Ok(StepReport::Progress(this.borrow().core().get_nyes()))
        }
        None => {
            // Queue empty → own combining work. fir_op_step may read parent/siblings
            // freely because `this` borrow_mut is the ONLY live borrow at this point.
            this.borrow_mut().fir_op_step(scope)?;
            let nyes = this.borrow().core().get_nyes();
            debug_assert!(
                !this.borrow().core().front_task().is_none() || nyes.is_settled(),
                "empty task list but node is not settled"
            );
            Ok(StepReport::Progress(nyes))
        }
    }
}
```

**`is_settled()`** replaces `is_constanic()` as the pop/stop predicate: it returns `true`
for `Econstanic | Woconstanic | Constant | Independent | Nk`. Nk is a terminal that
produces no further transitions; treating it as settled prevents a divide-by-zero result
from sitting at the front of the queue forever. The pop condition in the drain is `is_settled()`;
the acceptance condition in the outer loop remains "constanic" (Constant or Independent).

Two facts that follow (both confirmed during design):

- **All children constanic ≠ node constanic.** Draining the child tasks only *enables*
  `fir_op_step`; the node becomes constanic only after `fir_op_step` finishes its combining
  work AND any tasks it pushed are themselves drained to constanic.
- **`fir_op_step` may enqueue more `ubc_children`** (e.g. a concat result brane that must
  itself step to constanic — FOOP-3's "further steps delegate to the merged brane"). Those
  are tasks; they drain in queue order.

#### 3.3 How `nyes` actually advances — first pass MIRRORS UBC

The exact NYES progression is the **implementer's choice**. This FOOP fixes the *mechanism*
(task-list drain + `fir_op_step`), not the transition table. The **final states** that the
sequencer renders are pinned by **UBCa's own approved snapshots** (the acceptance gate); UBCa
need NOT reproduce UBC's step *counts* (not in snapshots) and is NOT required to match UBC's
output. UBC's `step_one` per kind + `compute_brane_state` (`ubc.rs:639`) may be consulted as a
*reference* for a reasonable progression, but it is not authoritative.

- Early progression `Prembrionic → Embryonic → Braning` (Braning possibly for several steps)
  is **UBC-defined**; UBCa replicates it. The task queue is built at `Embryonic`.
- The **default `ProtoBrane::fir_op_step`** performs the terminal classification — decide
  **Woconstanic vs Constant (vs Nk)** from the children's NYES and set `nyes` accordingly.
  This is exactly today's `ubc::compute_brane_state` (`ubc.rs:639`): all children
  Constant/Independent ⇒ `Constant`; any `Nk` ⇒ `Nk`; any `Econstanic`/`Woconstanic` ⇒
  `Woconstanic`; else `Braning`.
- Kind-specific firs override `fir_op_step` to do their combining work *and* set their own
  terminal NYES — e.g. `Plus` reduces its two (now-constanic) operands, pushes the constant
  result, and lands `Constant`:

  ```rust
  impl Fir for Plus {
      fn fir_op_step(&mut self, _scope: &Scope) -> Result<(), UbcError> {
          // reached only once both operand tasks are drained (constanic)
          let sum = self.operand_int(0)? + self.operand_int(1)?;
          // parent weak is stored in self.core.parent; pass it to the child's builder.
          let parent_weak = self.core.parent.clone();
          self.core_mut().push_ubc_child(constant_int(sum, parent_weak));
          self.set_nyes(Nyes::Constant);   // private self-mutation; stepping is a sanctioned writer
          Ok(())
      }
  }
  ```

This NYES-driven, one-transition-per-`step()`, queue-drain model produces the **final
rendered states** pinned by UBCa's own approved snapshots — the acceptance gate. (It does not
reproduce UBC's step *counts*, and need not: counts are not in snapshots.)

#### 3.3.1 NYES classification for Brane and Operator

**A node whose children are ECONSTANIC does NOT automatically become WOCONSTANIC.**
This is especially true for Brane and Operator where multiple children will step.

The NYES classification happens naturally through the depth-first FIFO work queue:
- **Nothing departs until constanic** — a node stays in the queue until settled
- **Fir state doesn't improve until queue is empty** — the node's NYES only advances when all child tasks drain
- **Even then, distinguish WOCONSTANIC vs CONSTANT** — after queue empties, `fir_op_step` classifies:
  - ALL children constanic AND some are WOCONSTANIC/ECONSTANIC → WOCONSTANIC
  - ALL children constanic AND all are CONSTANT/INDEPENDENT → CONSTANT
  - Otherwise → BRANING (still waiting for children)

The classification is NOT a separate step — it's what `fir_op_step` does when the task queue empties.

**Default classification rule (ProtoBrane).** A ProtoBrane's default NYES transition from
BRANING to a constanic state follows this priority order:

1. **Stay BRANING** until EVERY child is settled (constanic including NK)
2. Then pick the **worst** state:
   - **NK** if ANY descendant is NK
   - **WOCONSTANIC** if ANY descendant is WOCONSTANIC or ECONSTANIC
   - **CONSTANT** if there is a constant child (and no NK/WOCONSTANIC/ECONSTANIC)
   - **INDEPENDENT** only if ALL children are independent

This is implemented as `decide_nyes_due_to_children(children: &[FirRef]) -> Option<Nyes>`
in `fir_kinds.rs`. Returns `None` if not all children are settled yet (stay BRANING).
Used by BraneFir directly. OperatorFir uses this as a base but may override
(e.g., `1/0` → NK even though children are CONSTANT).

#### 3.4 Runtime safety: depth limits and panic resilience in tests

UBCa's recursive stepping (`step_fir_ref` descends into children) can trigger two
runtime failures: stack overflow from deep recursion, and `RefCell already mutably
borrowed` panics from borrow discipline violations. Both must be caught gracefully
in tests rather than crashing the test runner.

**Depth limit.** `step_fir_ref` accepts a `depth` parameter (default 0) and returns
`NoProgress` when depth exceeds `MAX_DEPTH` (100). The outer `step_fir_ref(this, scope)`
entry point calls the inner function with `depth=0`. This prevents stack overflow on
pathologically deep brane trees.

**Test runner configuration.** The crate's `Cargo.toml` must ensure `panic = "unwind"`
for test/dev profiles (the default). Never set `panic = "abort"` for tests — this kills
the entire test runner on any RefCell panic, preventing other tests from running:

```toml
[profile.test]
panic = "unwind"   # catch panics per-test, not process-wide
```

**Graceful RefCell handling in snapshot tests.** The snapshot test harness should use
`std::panic::catch_unwind` around evaluation to capture RefCell panics as test output
rather than crashing. When a panic occurs, the snapshot output should include the panic
message (e.g., `"RefCell already mutably borrowed"`) so the human reviewer can diagnose
the borrow discipline violation from the `.snap.new` file:

```rust
// In the snapshot tester:
for (name, source) in tests {
    let result = std::panic::catch_unwind(|| {
        evaluator.evaluate(&source)
    });
    match result {
        Ok(Ok(output)) => /* normal snapshot comparison */,
        Ok(Err(e)) => /* evaluation error in output */,
        Err(panic) => {
            let msg = panic.downcast_ref::<&str>()
                .unwrap_or(&"unknown panic");
            // Write panic info as the snapshot output so reviewer sees it
            insta::assert_snapshot!(name, format!("PANIC: {}", msg));
        }
    }
}
```

**`#[should_panic]` for borrow discipline tests.** Unit tests that verify RefCell
borrow violations are caught should use `#[should_panic(expected = "already borrowed")]`
to confirm the panic happens without crashing the runner.

**`try_borrow()` for defensive checks.** In `fir_op_step` implementations that walk
the parent chain or access siblings, use `try_borrow()` with graceful fallback instead
of `borrow()` when the node might be the one currently being stepped. This converts
a hard panic into a recoverable `NoProgress` or `Econstanic`.

### 4. Re-stepping (UBCa job queue replaces UBC's `re_step_brane_bodies`)

UBCa **deprecates** UBC's `re_step_brane_bodies` concept entirely. UBCa uses a per-node
job queue instead, and the mechanism makes explicit re-stepping unnecessary for normal
evaluation:

**The job queue mechanism.** Each ProtoBrane's `tasks` VecDeque operates as follows:
ProtoBrane enqueues non-constanic branes from `foolish_children` in the order they are
specified. Each `step()` call works the child at the **front** of the queue. If that child
returns any constanic state (settled — ECONSTANIC, WOCONSTANIC, CONSTANT, INDEPENDENT, or
NK), it is **popped** and the next child becomes the front. If the step returns that the
child is still pre-constanic (nigh), the child **stays at the front** — the next `step()`
call dequeues it immediately and works on it again. This continues until the front child
reaches a settled state.

This behavior is attractive because it guarantees that **later children, when searching,
should always result constanically in finite steps**: the language guarantees every FIR
reaches a constanic NYES in finite time (see §3.1), so the front child always eventually
settles, and the queue always makes progress through its children.

To recompute after a forward reference resolves, an anchor moves, etc.: **clear
`ubc_children`, rebuild `tasks` from `foolish_children`** (re-enqueue all; already-constanic
ones pop immediately on their next visit). Because `foolish_children` is the untouched
source meaning, re-derivation is always well-defined. §9 elaborates on the re-step-rebuild
task and the Econstanic-pop trap.

### 5. Parent link (first-class)

The per-node **`parent` back-link is retained and is load-bearing** — it is how
ancestral name resolution walks upward (`get_parent`, `get_brane` today; AB search in
`search_ancestral_branes`). The two-store split does **not** change parent semantics, but
FOOP-62 simplifies the representation:

- Each `ProtoBrane` holds **`parent: Weak<RefCell<dyn Fir>>`** — a *weak* reference (children
  are owned strongly downward via `Rc`, parents referenced weakly upward, per the
  graph-not-tree decision in [[foop52-scope-architecture]]). This replaces the
  `Option<FirRef>` / `ParentPtr` scheme.
- **The link is non-optional.** Instead of `None` for "no parent," **the root node's
  `parent` is a `Weak` pointing at itself.** A node is the root iff its parent upgrades to
  a node `Rc::ptr_eq` with itself:

  ```rust
  pub fn is_root(&self) -> bool {
      match self.parent.upgrade() {
          Some(p) => Rc::ptr_eq(&p, /* self as Rc */),
          None => false, // root Rc already dropped — only during teardown
      }
  }
  ```

  This saves the `Option` discriminant on every node and removes per-call `None` handling
  from the upward walk: ancestral search climbs via `parent.upgrade()` and **stops when
  `is_root()`**, rather than testing for `None`.
- **`parent` is immutable after construction — there is no `set_parent`.** A node's
  structural position is fixed for its lifetime. Re-parenting is done by **cloning the node
  with a new parent** (`clone_with_parent`), which is exactly the existing
  detach-and-recoordinate move: when a brane is referenced by name it is cloned, detached
  from its original parent, and recoordinated under the new one (AGENTS.md "Detachment and
  Coordination"). FOOP-62 makes that the *only* way a parent ever differs.
- A child placed into `ubc_children` (a search/op/concat result) is **constructed with its
  `parent` already set (weakly) to the producing node** — not mutated afterward — exactly
  as parse-time children are. Otherwise an ancestral search from inside a computed result
  would lose its way up the tree. Because parent is set at construction and never changed,
  there is no window in which a `ubc_children` node exists with a wrong/empty parent.
- `Scope` continues to carry the *dynamic* coordination context (position + ignorance)
  while `parent` is the *structural* link — but Scope is reworked into a capability
  surface (§10): its positional fields become private, its flat `entries` name list is
  removed (the parent chain IS the name table), and FIRs call `scope.search_ib/search_ab/
  index(…)` instead of reading position and walking themselves.

This is called out explicitly because gutting the implementation risks dropping parent
wiring on `ubc_children`; the test plan includes ancestral-search-through-computed-result
coverage to guard it, plus an `is_root()` unit test.

#### 5.1 Construction: nested `Rc::new_cyclic` (parent immutable at construction)

Because parent is set once and never mutated, a child must receive its parent `Weak` **at
construction** — before the parent's `Rc` fully exists. `Rc::new_cyclic` provides exactly
this: it hands the in-construction node's own `Weak` to a closure, so each child is built
*inside* the parent's closure and stored with that `Weak`. Branes nest cyclically — each
brane mints its own `Weak` and passes it to its children's construction:

```rust
// Each brane builds inside its own new_cyclic.
// new_cyclic is called on the CONCRETE type (BraneFir); the closure receives
// &Weak<RefCell<BraneFir>>. An unsized coercion at a typed `let` turns that into
// the Weak<RefCell<dyn Fir>> that children store as their parent pointer.
fn compile_brane(stmts: Vec<Astn>, parent: Weak<RefCell<dyn Fir>>) -> FirRef {
    let brane_rc: Rc<RefCell<BraneFir>> =
        Rc::new_cyclic(|me: &Weak<RefCell<BraneFir>>| {
            let me_dyn: Weak<RefCell<dyn Fir>> = me.clone(); // unsized coercion
            let children: Vec<FirRef> = stmts.into_iter()
                .map(|s| compile_child(s, me_dyn.clone()))   // child stores Weak<dyn Fir>
                .collect();
            RefCell::new(
                BraneFir::builder()
                    .foolish_children(children)
                    .parent(parent)   // REQUIRED — set before build()
                    .build()
            )
        });
    brane_rc   // Rc<RefCell<BraneFir>> coerces to Rc<RefCell<dyn Fir>> at call site
}

// Root brane: parent is a self-Weak (detected by is_root()).
fn compile_root(stmts: Vec<Astn>) -> FirRef {
    let root_rc: Rc<RefCell<BraneFir>> =
        Rc::new_cyclic(|me: &Weak<RefCell<BraneFir>>| {
            let me_dyn: Weak<RefCell<dyn Fir>> = me.clone();
            let children: Vec<FirRef> = stmts.into_iter()
                .map(|s| compile_child(s, me_dyn.clone()))
                .collect();
            RefCell::new(
                BraneFir::builder()
                    .foolish_children(children)
                    .parent(me_dyn)   // self-Weak → is_root() == true
                    .build()
            )
        });
    root_rc
}
```

Rules this construction obeys:
- **The root's `Weak` is self-referential** — same `new_cyclic`, `parent = me.clone()`; that
  is what `is_root()` detects.
- **Inside the closure the parent `Rc` is not yet complete**, so a child may only *store* the
  `Weak`, never *upgrade* it during construction (an upgrade would yield `None`). Stepping —
  which is the only thing that upgrades parent — happens strictly after construction.
- **Compiler impact**: `compile_astn` gains a `parent: Weak<RefCell<dyn Fir>>` parameter
  threaded downward; branes use `new_cyclic` to mint their own `Weak` before compiling
  children. This replaces the current bottom-up `parent: ParentPtr::new()` (12+ sites in
  `compiler.rs`) and is the highest-risk part of the migration (its own plan sub-task).
- The single `.clone()` per child is a **`Weak` handle clone** (refcount bump + pointer copy,
  no node copy) — the intended, visible cost of giving each child its parent link.

### 6. Builders via `bon`

UBCa constructs FIR nodes through **`bon`-generated builders** (crate `bon`, the
compile-time-checked builder generator), replacing the ~10 hand-written `*FirBuilder`
structs in `fir.rs` (each ~30 lines of boilerplate — `new`/setter-per-field/`build`).

- Add `bon` to `[workspace.dependencies]` and depend on it from the UBCa crate/module.
  (Latest is `bon = "3"`; pin a `3.x` MSRV-compatible version. Edition 2024 is fine.)
- Each FIR payload derives a builder with `#[derive(bon::Builder)]` (or `#[bon::builder]`
  on a constructor), so call sites read:

  ```rust
  let op = OperatorFir::builder()
      .op("+")
      .operands(vec![a, b])
      .parent(parent_weak)   // REQUIRED — see below
      .build();
  ```

- **`parent` is a required builder input, not a defaulted field.** The old hand-builders
  set `parent: ParentPtr::new()` (empty) and relied on a later pass to wire parents.
  FOOP-62 makes `parent` immutable-after-construction (§5), so the builder **must** take it
  up front: mark it a required `bon` field (no `#[builder(default)]`). This makes "a node
  without a parent" unconstructable except the root, which is built via `Rc::new_cyclic`
  with a self-`Weak` (a small dedicated `root()` constructor, not the generic builder).
- `bon` enforces required fields **at compile time**: forgetting `.parent(...)` or
  `.op(...)` fails to compile, which is exactly the "make illegal states unrepresentable"
  rule from `rust_instructions.md`. This is the main reason to prefer `bon` over the
  hand-written builders, beyond deleting boilerplate.
- `foolish_children` is supplied to the builder as a complete vector (it is fixed at
  construction); `ubc_children` is **not** a builder input — it starts empty and is only
  ever populated by stepping (`push_ubc_child`).

#### 6a. The builder is the ONLY way to create a Fir — enforced by the language, not just docs

It is a hard invariant that **every Fir comes into existence through a `bon` builder**.
This is enforced by Rust visibility, so a non-builder construction *does not compile*:

- **All FIR payload fields are private** (`foolish_children`, `ubc_children`, `nyes`,
  `parent`, and each kind's leaf data). No `pub`/`pub(crate)` fields. A struct literal from
  outside the defining module is therefore impossible.
- **The per-kind payload structs are not publicly constructible** except via the generated
  builder. Concretely: keep the struct definitions and their `bon` `#[derive(Builder)]` in a
  private module; re-export only the **builder entry points** (`OperatorFir::builder()`, …)
  and the read/step API. There is no public `OperatorFir { .. }` path, and **no `fir_to_ref`
  that wraps an externally-built value** — boxing into `Rc<RefCell<dyn Fir>>` is done
  *inside* `build()` (the builder returns the `FirRef` form directly), so there is no seam to
  bypass.
- **`#[non_exhaustive]`** on each payload struct prevents external code (and even forgetful
  internal code in another module) from writing a literal — a second, belt-and-suspenders
  compile-time guard.
- Net effect: the *only* symbols a caller can reach to obtain a `FirRef` are the builder
  entry points. "Create a Fir without a builder" is unrepresentable in the type system,
  which is the language-level enforcement requested (docs in this section state the intent;
  the privacy + `#[non_exhaustive]` *enforce* it).

#### 6b. Constanic clone = build-from-existing-Fir + field update

Constanic clone (the recoordination move — clone a brane/value, detach, recoordinate under
a new parent; FOOP-7) is expressed as **a builder seeded from an existing Fir, with a few
fields overridden before `build()`**. `bon` supports this with two pieces:

1. the `#[builder(on(_, overwritable))]` option, so a seeded field can be re-set
   (overwritten) before `build()` — plus `on(String, into)` for ergonomic conversions; and
2. a small hand-written `updater(self) -> XFirBuilder` that seeds a builder from `self`'s own
   fields (this is the value→builder bridge — `bon` does not auto-generate it):

```rust
use bon::Builder;

// A kind CONTAINS a ProtoBrane (core topology) + its own leaf data.
#[derive(Builder)]
#[builder(on(String, into), on(_, overwritable))]   // allow ergonomic Into + field overwrite
struct OperatorFir {
    core: ProtoBrane,   // foolish_children + nyes + parent live here (ubc_children/tasks start empty)
    op: String,             // leaf data
}

impl OperatorFir {
    /// Seed a builder populated with this node's own fields (bon has no auto value→builder).
    fn updater(self) -> OperatorFirBuilder { /* ... */ }
}

// Constanic clone: seed from an existing fir, override only what recoordination needs,
// then build a fresh node. The source is consumed into the updater (a value→builder move);
// the ORIGINAL node referenced elsewhere is untouched because the clone is a new value.
fn constanic_clone(source: OperatorFir, new_parent: Weak<RefCell<dyn Fir>>) -> FirRef {
    source.updater()
        .parent(new_parent)     // override: recoordinate under the new parent (§5)
        .nyes(Nyes::Embryonic)  // override: reset NYES for the new context (overwritable)
        // foolish_children are constanic-cloned per FOOP-7; ubc_children/tasks start empty
        .build()
}
```

- This makes constanic clone go **through the same builder** as fresh construction — so
  6a's "only the builder creates Firs" holds for clones too. There is no separate
  `clone()`-then-mutate path that could bypass the builder.
- **Ignorance of the clone (see Terminology).** `constanic_clone` carries one boolean mode
  flag, `descendent_of_sfm_and_foolishly_ignorant` (default `false`):
    - **`false` (normally ignorant):** obeys the NYES-transfer rule — **constanic NYES transfer
      unchanged** (CONSTANT/INDEPENDENT referenced; ECONSTANIC/WOCONSTANIC/NK keep their state
      and re-resolve later via stepping), **pre-constanic states transfer as PREMBRYONIC**.
    - **`true` (foolishly ignorant):** **ALL NYES copied unchanged** (constanic and
      pre-constanic alike, no reset). Set when building the RHS of an SF-marked assignment;
      propagates recursively to descendants.
  **THE BIG BUT:** when a later *search* clones an **SF-mark node**, the clone **strips the
  SF-mark** and runs with the flag **`false`** (normal mode), so the inner value re-resolves
  normally. **Fully foolish** (SFF) is not a clone at all: SFF construction builds its
  descendants ECONSTANIC up front (§10.1).
- `updater(self)` is the value→builder bridge; `#[builder(on(_, overwritable))]` is what lets
  the seeded `parent`/`nyes` be overwritten. (Confirmed against `bon`'s documented updater
  pattern.) `nyes` set here is one of the three sanctioned NYES writers (constanic clone, §1).
- `clone_with_parent` from §5 is exactly this pattern specialized to the parent override.

(@human: `bon` is a new third-party dependency. Per AGENTS.md's dependency rules it is
widely used, actively maintained, proc-macro-based but builder-only (no runtime/security
surface), and replaces existing hand-rolled code rather than adding new capability.
**APPROVED by Atlas 2026-06-09 — add the latest stable `bon`** (latest at writing: `3.9.1`;
add `bon = "3"` to `[workspace.dependencies]`, pinning a current `3.x`). The value→builder
mechanism is settled (Atlas supplied the pattern): `#[builder(on(_, overwritable))]` +
`on(String, into)` plus a hand-written `updater(self) -> XFirBuilder` that re-feeds `self`'s
fields into a fresh builder; overrides then `.build()`. `bon` does NOT auto-generate the
value→builder bridge, so each payload gets a small `updater`.)

### 7. Crate layout: `foolish-ubca`, a new sibling crate

UBCa is a **new workspace crate `foolish-ubca`**, parallel to the original UBC and to
`foolish-ubcb` — *not* an in-crate module. (The original UBC reference implementation lives
inside `foolish-core` (`src/fir.rs`, `src/ubc.rs`); UBCb is its own crate `foolish-ubcb`.
UBCa follows the UBCb precedent: its own crate.)

- The existing UBC implementation in `foolish-core` (`fir.rs`, `ubc.rs`) stays in place for
  reference, but is **no longer an authoritative oracle** (its approved snapshots have drifted
  from its evaluator). UBCa is validated against its own snapshots, not UBC's.
- **`foolish-ubca`** is created by cloning the UBC *public interface* (the `Scope` API,
  `run_to_completion*`, the snapshot suite harness), then **gutting** the internals and
  rebuilding on the ProtoBrane two-store structure. Add `foolish-ubca` (and, if a CLI
  parity is wanted later, `foolish-ubca-cli`) to the workspace `members`, mirroring the
  `foolish-ubcb` / `foolish-ubcb-cli` pair.
- **UBC's `.foo` test inputs are copied to UBCa as starting inputs.** The `input/` directory
  (the `.foo` test programs) is copied as a starting point. UBCa then maintains its **own**
  `approved/` snapshot corpus under `foolish-ubca/snapshot_tests/approved/` — that corpus is
  UBCa's source of truth, reviewed and signed on UBCa's own terms (AGENTS.md no-auto-accept
  rule still applies). UBCa's snapshots are **NOT** required to equal UBC's snapshots.
- **The humanizer sequencer is developed for UBCa's new FIR classes.** Everything else
  beyond the copied inputs is implemented according to this FOOP's design: the ProtoBrane
  two-store structure, the `trait Fir` dyn-dispatch surface, the task-list stepping, and the
  reworked Scope. The sequencer reads UBCa's FIR types (ProtoBrane's `foolish_children` /
  `ubc_children` stores + leaf accessors via `kind()`) and its output must be **byte-identical
  to UBCa's own approved snapshots**. This is the **hard acceptance constraint** (§8).

### 8. The humanizing sequencer is a HARD acceptance constraint

The sequencer is **not** an afterthought to "confirm can be re-expressed" — **the approval
tests are its output**, byte-for-byte. UBCa is accepted only if its sequencer reproduces the
existing approved `.snap` corpus exactly. This pins concrete requirements on the FIR surface:

- **The sequencer distinguishes a `result` from non-result items** and renders it with a
  `result=` label, positioned before the other items, with exact comma/indentation rules
  (`foolish-core/src/sequencer.rs`, `proto_brane_formatter_with_result`, ~line 180). In the
  two-store model, **the `result=` item(s) come from `ubc_children`** and the other items
  from `foolish_children`. This is *why* `ubc_children` order is snapshot-significant (§1).
- **Anonymous statements are named `???` and render WITHOUT a `name=` prefix.** A statement
  with an LHS identifier (`a = …`) carries that identifier as its name and renders `a=…`. A
  bare expression statement (no LHS) is named **`???`** (`compiler::ANON_STMT_NAME`); the
  sequencer renders such a statement as just its value, with **no** `name=`/`???=` prefix.
- **`result` vs `anchor` for searches (search / Index / HeadTail).** The `result=` of a search
  FIR is **the FIR the search PRODUCED** (what it found by searching / indexing / head-tail).
  The **anchor** — the FIR a search searches *within/relative to* (e.g. `data` in `data#5`) —
  is a **non-result** item, never the `result=`. (The word "target" is banned as ambiguous; use
  `anchor` and `result`.) An out-of-bounds / not-found search settles NK with **no** `result=`.
- **SINGULAR-RESULT INVARIANT.** Every search FIR we currently implement produces **at most one
  result**, so its `ubc_children` holds **at most one** entry, and that entry IS the result.
  This is documented and **runtime-verified** (`ProtoBrane::push_search_result` debug-asserts
  `ubc_children` is empty before pushing). (Multi-result searches are a future extension that
  will hold more `ubc_children`; not implemented now.)
- **Today the sequencer reads FIR through the `FirQueryable` fat trait** (`hs_operator`,
  `hs_search`, `hs_concatenation`, `hs_brane`, `hs_variant`, …). FOOP-62 proposes retiring
  that mirror trait and re-expressing the sequencer over **`kind()` + the uniform child
  iterators + leaf accessors**. This re-expression is **mandatory and must be output-exact**,
  not best-effort:
  - `kind()` replaces `hs_variant()` for dispatch.
  - The per-variant tuples (`hs_operator -> (op, operands)`, `hs_concatenation ->
    (elements, merged)`, `hs_brane -> (characterizations, statements)`, etc.) are
    reconstructed from `foolish_children` (operands/elements/statements + leaf data) and
    `ubc_children` (the `merged`/result), preserving the **exact element order** the current
    tuples carry.
  - `hs_search -> (pattern, direction, anchored, anchor, target)` maps to: leaf data
    (pattern/direction/anchored) + `foolish_children[anchor]` + `ubc_children[result]`.
- **Acceptance rule**: if the sequencer produces any byte difference from **UBCa's own
  approved corpus**, it is either a UBCa bug or a deliberate semantic change — present the
  `.snap.new` to a human for review; never auto-accept. (The corpus is UBCa's, not UBC's.)
- **DEFAULT PLAN (both reviewers concur): keep a thin `FirQueryable` adapter over ProtoBrane
  as the first-pass sequencer path.** The adapter reads the two stores + leaf accessors and
  returns the same tuples the sequencer expects (~100 lines of glue vs ~400 lines of
  rewrite). Prove the corpus green through the adapter FIRST. **Retiring `FirQueryable` is a
  later, optional cleanup explicitly OFF the acceptance path** — not the gate. (The draft had
  this as a fallback; the reviews make it the default.)

### 9. NormalBrane & Statement stepping (the hard case)

Both reviewers flagged that "mirror UBC" is too thin for branes. This section makes it
concrete; it is the most behavior-sensitive part of UBCa.

#### 9.0 The Quiescent-Representation Invariant (MANDATE)

**When `step()` is not running, every FIR MUST faithfully represent the correct state for the
nyes it is currently in.** This is an imperative the whole model is built on, not merely a
justification for in-place stepping. Two tiers:

- **Operational correctness (every nyes, every quiescent point).** Between `step()` calls a
  FIR's structure and its nyes agree: it is a coherent representation of "the Foolish,
  evaluated this far." There is no torn/partial state observable between steps. What the
  "correct state" *is* for a given nyes is defined **operationally** (by the step rules /
  UBC's transitions).
- **Denotational correctness (nyes ∈ {Constant, Independent}).** When a FIR is constanic in
  the strong sense, it additionally has a **genuine denoted value** — the actual answer, not
  just correct progress. At those two nyes, "what does this mean?" has a definite denotation,
  and the FIR must equal it.

`step()` is the **only** thing permitted to transiently break this (mid-step things are in
flux); the instant `step()` returns, the invariant holds again. Consequences that the rest of
§9 and the model lean on:
- **In-place shared stepping is safe** — a referrer pointing at a FIR (and minding its nyes)
  always sees a coherent state, never a torn one.
- **`constanic_clone` preserves it** — a clone is born quiescent-coherent for its declared
  nyes.
- **The sequencer relies on it** — it renders quiescent FIRs, so byte-exact output is
  well-defined precisely because every rendered FIR honors this invariant.
- **It bounds `step()`** — a step may only move a FIR from one quiescent-coherent state to
  another; it may never leave a FIR misrepresenting its nyes.

- **A statement is a len-1 ProtoBrane carrying `name` + `line_number` as leaf data.** Whether
  this is a dedicated `StatementFir { core, name, line }` struct or simply a ProtoBrane that
  carries a name is a marginal call (a thin struct is the obvious home; not architecturally
  significant). The metadata travels **through the builder at construction**: while the brane
  builds (inside its `new_cyclic`), it feeds each statement its `parent` (the brane's `Weak`),
  `line_number`, and `body` via the builder, then `.build()`. The `name` is leaf data used for
  match resolution. No parallel metadata vector; no sync problem — this is exactly what the
  builder is for (accumulate parts, then build).
- **In-place stepping is correct — by §9.0, not "validate then maybe clone."** By the
  Quiescent-Representation Invariant, referring to a statement's RHS = pointing at its FIR and
  minding its nyes; stepping that shared FIR in place moves it between quiescent-coherent
  states, exactly the meaning every referrer should see. The aliasing worry (mimo #1c) **does
  not arise**: anything needing an independent copy gets one via **`constanic_clone`** (a
  search *always* constanic-clones — a separate, immediately-recoordinated copy with specific
  states). Independence is provided by `constanic_clone` where the semantics call for it,
  **never** by defensive copying inside the drain. UBC's `step_boxed` clone is UBC's
  *mechanism*, not the semantics; UBCa does not replicate it.
- **Re-stepping rebuilds the task list.** §4's "clear `ubc_children` and re-derive" is
  incomplete: re-stepping must **rebuild `tasks` from `foolish_children`** (re-enqueue all;
  already-constanic ones pop immediately on their next visit). Without this, a forward
  reference that resolves later has nothing to re-step.
- **The Econstanic-pop trap (Deepseek #5).** `is_settled()` is TRUE for `Econstanic`, so an
  unbound-name child is popped off the queue. When the name later binds, the node only
  re-evaluates it because the re-step **rebuild** above re-enqueues it. This trap MUST be
  called out so nobody "optimizes" the rebuild into a no-op for already-drained children.
- **Incremental scope — proposed resolution: the StatementFir boundary augments scope
  (§10.4).** Current UBC builds scope incrementally so `stmt_i` sees evaluated `stmt_<i`
  (`ubc.rs:278`, `current_stmt_idx`). In UBCa the statement itself carries `line_number`
  and its `parent` is the brane, so the scope for a statement's body is constructed at the
  StatementFir boundary — no brane `step` override and no per-position threading in the
  brane's drain. Incremental visibility (stmt_i sees only evaluated stmt_<i) is preserved
  because `scope.search_ib`/`search_ab` are bounded backward from `current_stmt_idx`, and
  tasks drain in statement order. (PROPOSED; validate in Phase 3 — if it fails, the brane
  `step` wrapper remains the fallback.)
- **SF/SFF: difference is in clone-mode + construction, not in stepping (NO `step` override).**
  An SF/SFF wrapper is a len-1 ProtoBrane over its expr. SF ⇒ **foolishly ignorant**: building
  its RHS clones with `descendent_of_sfm_and_foolishly_ignorant = true` (ALL NYES copied
  unchanged, propagated recursively), freezing the result. SFF ⇒ **fully foolish construction**:
  children instantiated as ECONSTANIC via builder, no searches run, no cloning. The *normal*
  drain then produces the correct behavior; no live `Ignorance` field on `Scope` is required.
  So SF/SFF need no special stepping — only the clone-mode flag (SF) and special construction
  (SFF). (This corrects the review's reading of `step_except_brane_searches` as a separate
  algorithm: in the ProtoBrane model that work moves into the clone flag + construction state,
  not a per-call step override.)
- **Foolish ignorance propagates recursively downward — but a later search of an SF strips it.**
  When the RHS of an SF-marked assignment is built, `descendent_of_sfm_and_foolishly_ignorant`
  is `true` and **propagates recursively to all descendant clones** within that SF expression,
  so everything is cloned with constanic NYES (copied verbatim). **THE BIG BUT:** when a later *search* resolves to
  the **SF-mark node** and clones it, the clone **strips the SF-mark** and runs with the flag
  **`false`** (normal mode) — the inner value re-resolves normally. SFF is not a clone behavior
  at all — it is **fully foolish construction**: descendants instantiated as ECONSTANIC.
- **`constanic_clone` mode flag governs the NYES handling** (see Terminology):

  **Foolishly ignorant (`descendent_of_sfm_and_foolishly_ignorant = true`, building an SF RHS):**
  - **ALL NYES are copied unchanged** — constanic and pre-constanic alike, no reset. The whole
    SF subtree is cloned with constanic NYES exactly as found; the flag propagates recursively to descendants.
  - The effect is that the SF keeps the answer(s) it first found: nothing re-resolves.
  - Example: `b = <a>` where `a` is an unresolved search (ECONSTANIC) → the SF makes `a`'s
    subtree with every NYES copied verbatim (the ECONSTANIC stays ECONSTANIC, NOT reset).

  **Later search of an SF-mark (flag = false, mark stripped):**
  - The `StayFoolish` wrapper is **removed**; the clone is the inner expression in **normal
    mode**, so its NYES follow the normal NYES-transfer rule and ECONSTANIC inners re-resolve.
  - Example: a later search finds `b` (an SF) → clone strips the mark and re-resolves the inner
    value in the searching context, normally ignorant.

  **Normally ignorant (normal context)** — applies the NYES-transfer rule (Terminology):
  - **Constanic** NYES are transferred **unchanged** (CONSTANT/INDEPENDENT are effectively
    referenced; ECONSTANIC stays ECONSTANIC, WOCONSTANIC stays WOCONSTANIC, NK stays NK).
  - **Pre-constanic** states (PREMBRYONIC/EMBRYONIC/BRANING) are transferred as **PREMBRYONIC**
    so they re-evaluate within their new context.
  - A re-resolvable clone is an ECONSTANIC one: it re-runs its search through ordinary
    stepping in the new context — it is NOT pre-reset to pre-constanic at clone time.
  - Example: normal `b = a` where `a` is an unresolved search (ECONSTANIC) → normally-ignorant
    clone stays **ECONSTANIC**, then re-resolves via stepping in `b`'s context.
  - Example: normal `b = a` where `a=10` (CONSTANT) → clone's NYES transfers unchanged
    (CONSTANT) — already a value, nothing to re-evaluate.

  **Fully foolish construction (SFF context):**
  - SFF never runs searches — children are instantiated as ECONSTANIC via builder
  - No constanic-clone occurs (nothing to clone)
  - SFF is immediately Independent (self-contained constant)

- **HFS rendering of constant search results.** When a search resolves to a constant
  value (e.g., `x=42`), the Humanizing FIR Sequencer (HFS) may render the constant
  directly (`42`) rather than preserving the full search wrapper
  (`?(result=42, pattern='^x$', UNANCHORED)`). This is acceptable — `get_value()` on
  a settled search returns the resolved constant, and HFS renders that. Non-constanic
  searches still render with the full wrapper. This decision was made during FOOP-62
  implementation when `sf_non_brane_resolves` snapshot showed `42` instead of
  `?(result=42, ...)`.

- **HFS NYES display rules for searches.** The Humanizing FIR Sequencer follows these
  rules for when to show the NYES state on search/Index/HeadTail renders:
  - **Case a)** When there IS a result, and the search is `nnk_constanic` (constanic but
    not NK), do NOT show nyes — the reader can infer from the result object.
  - **Case b)** When there is NO result, and the state is EMBRYONIC, do NOT show nyes.
  - **In all other cases**, show nyes even if it is PREMBRYONIC.
  - **Especially**: show NK with reason (e.g., `??? (division by zero)`).

- **Constanic-clone of SF-markers.** When constanic-clone is called ON an SF-marker, it
  **strips the SF-marker** and returns the constanic-clone of its only (inner) child. This
  means:
  - `constanic_clone(SF<x>)` = `constanic_clone(x)` (SF is transparent to cloning)
  - The SF wrapper is stripped — only the inner value is cloned; no `StayFoolish` re-wrap.
  - The incoming `descendent_of_sfm_and_foolishly_ignorant` flag is **passed on** to the inner
    clone (NOT forced to false): a normal search consuming an SF clones the inner *normally*
    (constanic NYES kept unchanged per the NYES-transfer rule, pre-constanic → PREMBRYONIC),
    while an SF nested inside an outer SF's RHS keeps the foolish flag and copies NYES verbatim.

### 10. Scope: the search-capability surface (no name accumulation)

Today's `Scope` carries `entries: Vec<(String, FirRef)>` — a flat name→FirRef list that
shadows information already present in the tree. **FOOP-62 removes it.** The structural
tree (mandatory `parent` wiring + statements carrying their own `name`/`line_number`) IS
the name table; accumulating a second copy invites drift between the two. Name resolution
reaches back through the parent pointer and asks methods on the parent FIR, recursively.

`Scope` is retained, but re-purposed: it becomes the **active search-capability surface**.
A searching FIR does not know where it is; `Scope` does. So instead of the FIR reading
positional fields (`current_brane()`, `current_stmt_idx()`) and driving the walk itself,
the FIR calls capability methods and `Scope` supplies the position. `scope.index(-1)` is
enough — the scope already knows the line number to track `-1` from.

#### 10.1 The capability surface

The two clone modes + one construction behavior (Terminology). The two clone modes are a
boolean flag on `constanic_clone`, **not** a `Scope` field:

```rust
/// `constanic_clone`'s mode flag (default false). NOT a Scope field.
/// descendent_of_sfm_and_foolishly_ignorant:
///   false (NORMALLY ignorant): constanic NYES copied unchanged; pre-constanic → PREMBRYONIC.
///   true  (FOOLISHLY ignorant): ALL NYES copied unchanged (constanic AND pre-constanic).
///     Set when building the RHS of an SF-marked assignment; propagates recursively to
///     descendant clones.
///
/// THE BIG BUT: when a later SEARCH clones an SF-mark node, the clone STRIPS the SF-mark and
/// runs with the flag = false (normal mode) — the inner value re-resolves normally.
///
/// Fully foolish (SFF `<<…>>`) is a CONSTRUCTION property, not a clone: descendant search
/// FIRs are instantiated as ECONSTANIC at construction; SFF runs zero searches, never clones.
//
// (Historical: this distinction was once sketched as `enum Ignorance { Normally, Foolishly }`,
//  renamed from EvalContext { Normal, Sf, Sff }. The accepted design carries it as the clone
//  flag above, so there is NO live `enum Ignorance` / `Scope.ignorance` field.)
```

**Implementation note (reconciled with the reference UBCa, branch `foop-62-ubca-mimo`).**
The accepted UBCa implementation does **not** carry this as a runtime field on `Scope`. The
two clone modes are the **`descendent_of_sfm_and_foolishly_ignorant` flag on the single
`constanic_clone`** (false = normal NYES-transfer rule; true = ALL NYES copied unchanged,
set while building an SF-marked RHS and propagated to descendants). Fully-foolish (SFF) is
realized by building descendants ECONSTANIC at construction. So the three ignorance words are
the **canonical vocabulary** and the contract every implementation must honor; there is no
`enum Ignorance` / `Scope.ignorance` field — if you went looking for one in the code and could
not find it, this is why.

> **⚠ Implementation gap (unresolved as of 2026-06-19).**
> Ground truth as of SHA `cc3fe590` on branch `foop-62-ubca-mimo` in directory
> `/home/hcbusy/tmp/foolish-worktrees/foop-62-ubca-mimo` (plus this session's uncommitted
> doc-comment edits to `fir_kinds.rs`):
> - **`Scope` is a 2-field STUB** in `foolish-ubca/src/fir_trait.rs` (`current_brane:
>   Option<FirRef>`, `current_stmt_idx: Option<usize>`) with a `Scope::empty()` constructor.
>   `step_fir_ref(this, scope: &Scope)` and `fir_op_step(&self, scope: &Scope)` already thread
>   it, but every kind ignores it (`_scope`) and call sites pass `Scope::empty()`. There is
>   **no `has_ancestral_sfm` field yet**, and **`bon` is not used** in this crate (no builder).
> - **`constanic_clone_normal_at(fir_ref, new_parent, index)` has NO flag parameter** — only
>   the normal mode exists; the foolishly-ignorant (build-SF-RHS) path is unimplemented.
> - **Compound NYES reset wrongly even in normal mode** — Operator/Search/Index/HeadTail/
>   Brane/Statement/Concatenation clones hard-code `Nyes::Prembrionic` regardless of source
>   NYES, so a *constanic* compound (ECONSTANIC/WOCONSTANIC) is wrongly reset.
> - **SF/SFF re-wrapped on clone** — `FirKind::StayFoolish`/`FirKind::StayFullyFoolish` arms
>   rebuild the wrapper; per "THE BIG BUT" a later search cloning an SF-mark must **strip the
>   mark** (clone the inner expression in normal mode), NOT re-wrap.
> Target: add `has_ancestral_sfm: bool` to `Scope`; add
> `descendent_of_sfm_and_foolishly_ignorant: bool` to the clone; seed clone-from-step with
> `scope.has_ancestral_sfm`; clone-recursion inherits the caller's flag. Tracked in
> FOOP-62.plan.md Phase −1.

```rust
impl Scope {
    // --- unanchored searches (Scope supplies the position) ---

    /// 1. Search BACKWARD within the immediate brane for `pattern`,
    ///    from the current statement position. (search_immediate_brane today)
    pub fn search_ib(&self, pattern: &str) -> Option<FirRef>;

    /// 2. Search backward in the immediate brane; on miss, recursively call the
    ///    parent brane's backward search (bounded at the enclosing statement's line),
    ///    until found or is_root(). (search_ancestral_branes today)
    pub fn search_ab(&self, pattern: &str) -> Option<FirRef>;

    /// 3. UNANCHORED relative positional access — the `#-1 + #-2` syntax form.
    ///    For the statement numbered k (0-based), valid offsets are [-k, -1] —
    ///    STRICTLY NEGATIVE, strictly backward. Scope holds the line number the
    ///    offset is relative to. Out-of-range (including 0 and positives) ⇒ NK.
    ///    NOTE: anchored indexing (`b#1`, `b#-1`) is a DIFFERENT operation with
    ///    both signs valid — it lives on BraneFir (§10.2), not here. Two
    ///    syntactically and semantically distinct ways to index.
    pub fn index(&self, offset: i32) -> Option<FirRef>;

    // --- context gates ---

    /// 4. Is the current evaluation inside an SF-mark's RHS? Read `has_ancestral_sfm`
    ///    (true ⇒ foolishly ignorant). `step()` seeds each constanic_clone's
    ///    `descendent_of_sfm_and_foolishly_ignorant` from this field.
    ///    has_ancestral_sfm = true (foolishly ignorant) ⇒ constanic_clone copies ALL NYES
    ///      unchanged (constanic AND pre-constanic), freezing the SF RHS, recursively.
    ///    has_ancestral_sfm = false (normally ignorant) ⇒ NYES-transfer rule: constanic
    ///      unchanged, pre-constanic → PREMBRYONIC; ECONSTANIC re-resolves via stepping.
    ///    THE BIG BUT: a later search cloning an SF-mark STRIPS the mark and runs with
    ///      the flag false (normal), so the inner value re-resolves.
    ///    Fully foolish (SFF) does NOT flow through here — descendants constructed ECONSTANIC.
    pub fn has_ancestral_sfm(&self) -> bool;
    pub fn how_ignorant(&self) -> Ignorance;

    /// 5. Diagnostic sink (UNBOUND-NAME etc.).
    pub fn emit(&self, alarm: Alarm);
}
```

Private internals: `current_brane: FirRef`, `current_stmt_idx: usize`,
**`has_ancestral_sfm: bool`** (true inside an SF-mark's RHS; seeds each `constanic_clone`'s
`descendent_of_sfm_and_foolishly_ignorant` — see Terminology), `alarms:
Option<Rc<dyn AlarmSink>>`. No `entries`. There is **no** `ignorance: Ignorance` field — the
normal/foolish distinction is the `has_ancestral_sfm` flag carried here plus the clone's own
`descendent_of_sfm_and_foolishly_ignorant`; fully-foolish (SFF) is construction-time. The
positional fields are
**private** — no public `current_brane()`/`current_stmt_idx()` getters; FIRs interact only
through the capability methods. (`block_brane_searches` as a separate bool disappears: it is
subsumed by `has_ancestral_sfm`. Verify during implementation that no current code path
sets `block_brane_searches` independently of the SF context — flagged in Open Questions.)

**Upward navigation: every Fir exposes `get_parent()`, `get_parent_statement()`, and
`get_parent_brane()`** (inherent on `ProtoBrane`, all built on the parent `Weak` chain:
`get_parent()` is one hop; `get_parent_statement()` walks up to the nearest enclosing
StatementFir; `get_parent_brane()` walks up to the nearest enclosing brane; all stop at
`is_root()`). `search_ab` is built on these — conceptually:

```rust
// scope.search_ab(pattern) delegates to the current brane, bounded at the
// current statement position:
self.current_brane.search_ab(pattern, self.current_stmt_idx)

// BraneFir::search_ab(pattern, from_line): search own statements BACKWARD from
// from_line; on miss, recurse upward — each level obtaining its bound the same way:
self.get_parent_brane().search_ab(
    pattern,
    self.get_parent_statement().get_line_number(),
)
// ... stopping (Econstanic / unbound) at is_root().
```

**Edge positions (RULED 2026-06-10)**: the trio returns `Option` — `get_parent_statement()`
/ `get_parent_brane()` yield `None` above the first statement and at the root. `None`
matches the not-found behavior of searches: `search_ab` treats it exactly like a miss
(unbound ⇒ Econstanic), the same terminal as stopping at `is_root()`. One stop condition,
two spellings — keep them from drifting apart.

This **replaces today's downward containment scan** (`line_of_child`/`contains_fir`): the
enclosing statement's line number is read off the StatementFir found by walking UP — never
by scanning a brane's children to discover "which statement contains me." That also
retires a borrow-ordering hazard: while a search's `fir_op_step` runs, the searching node
is mutably borrowed (§3.2), and a downward scan would reach it and panic unless it
compared identity before borrowing. (`Rc::ptr_eq` = pointer identity of two `Rc` handles —
"are these literally the same node?" — as opposed to comparing their contents, which
requires borrowing.) The upward walk borrows only ancestors, which are unborrowed under
the §3.2 transient-borrow discipline; `ptr_eq` remains only inside `is_root()`.

#### 10.2 Anchored searches live on BraneFir, NOT on Scope

Anchored search FIRs (`Search`/`Index`/`HeadTail` with an anchor) first drain their anchor
to settled, then call **inherent methods on the resolved anchor brane** — the anchor is the
position context, so Scope plays no positional role:

```rust
impl BraneFir {
    /// Pattern search within [from_idx, to_idx]. from > to ⇒ BACKWARD search.
    pub fn search(&self, pattern: &str, from_idx: usize, to_idx: usize) -> Option<FirRef>;

    /// Absolute positional access: n >= 0 counts from the front,
    /// n < 0 counts from the back. (index_in_brane today — semantics already match.)
    pub fn index(&self, n: i32) -> Option<FirRef>;

    /// Sugar: head() = index(0)  — the '#0'  anchored indexing.
    ///        tail() = index(-1) — the '#-1' anchored indexing.
    pub fn head(&self) -> Option<FirRef>;
    pub fn tail(&self) -> Option<FirRef>;
}
```

These replace the free functions in `search.rs` (`search_in_brane`, `index_in_brane`,
`head_of_brane`, `tail_of_brane`) — same behavior, moved onto the type that owns the data
(the encapsulation rule from `rust_instructions.md`). These are not all the search
capabilities branes can handle; they are the surface the current FIR kinds need. The
unanchored Scope methods (10.1) are themselves thin wrappers that locate the right brane
and line, then call these BraneFir methods.

#### 10.3 Scope construction via `bon`

`Scope` is built with the same `bon` fluent style as FIR nodes — `#[bon::builder]` on a
constructor (or `#[derive(bon::Builder)]`), so optional parts are simply omitted:

```rust
let scope = Scope::builder()
    .current_brane(brane_rc)
    .current_stmt_idx(3)
    .has_ancestral_sfm(true)      // optional; defaults false. true inside an SF-mark's RHS;
                                  // seeds each constanic_clone's
                                  // descendent_of_sfm_and_foolishly_ignorant (§10.1, Terminology).
    // .alarms(sink)               // optional; omitted here
    .build();
```

This replaces today's `Scope::new()` + chained `.with_brane(…)` / `.with_sff_context()` /
ad-hoc `Clone` mutations.

#### 10.4 Who sets the position? — proposed resolution of the §9 scope-threading question

§9 left open how `current_stmt_idx` is threaded as a brane drains. With Scope-as-capability
the clean answer falls out of the structure: **the StatementFir boundary augments the
scope.** A `StatementFir` carries its own `line_number` as leaf data and its `parent` is the
brane — so when the drain reaches a statement, the scope for that statement's body is built
right there: `current_brane = statement.parent`, `current_stmt_idx = statement.line_number`.
No brane `step` override, no per-position state threaded through the brane's drain loop —
the statement already knows where it is. The statement-built scope supplies only the
**position**; `ignorance` and `alarms` are **inherited** from the incoming scope (merged,
never reset — an SF/SFF context set above the brane must survive into statement bodies).
Entry points that are NOT statement bodies (the root expression; anchor sub-evaluations)
construct their scope explicitly: the root with no position, anchor evaluation inheriting
the caller's scope as UBC does today. (PROPOSED, not yet confirmed: this resolves the §9
open item without any step-function exception. To validate in Phase 3.)

## FIR Impact

Large. This is a representation change:

- **New**: `struct ProtoBrane` field-holder with `foolish_children` / `ubc_children` /
  `nyes` / `parent` / `tasks` (§3 task list) as inherent methods. Public `get_nyes()`
  accessor; NYES written only by init/`fir_op_step`/clone. New narrow `trait Fir`
  (dyn-dispatch surface: `core()`/`core_mut()`, `fir_op_step`, `kind`, leaf accessors).
  Free function `step_fir_ref(&FirRef, &Scope)` replaces trait-method `step()`.
  New `Nyes::is_settled()` predicate (`is_constanic() || == Nk`) used as the task-queue
  pop condition; `is_constanic()` retained for the outer acceptance condition. New
  `StepReport` enum (`NoProgress` / `Progress(Nyes)`) as the return type of `step_fir_ref`.
- **Removed (in UBCa)**: the ~50-method `Steppable` trait, the `ChildrenItr` enum and its
  `unsafe` arm, `children_mut`, `fir_children`, `merged`/`anchor`/`target`/`operands`/
  `elements`/`statements` as bespoke fields (folded into the two stores), the
  `FirQueryable` mirror (the sequencer reads via the uniform child iterators + `kind()`),
  and the ~10 hand-written `*FirBuilder` structs (replaced by `bon`-derived builders, §6).
- **New dependency**: `bon` (workspace) for compile-time-checked FIR builders (§6) and the
  `Scope` builder (§10.3).
- **Scope reworked into a capability surface (§10)**: `entries` flat name list REMOVED
  (the parent chain is the name table); positional fields private; public surface =
  `search_ib` / `search_ab` / `index` / `how_ignorant` / `emit`. `EvalContext { Normal,
  Sf, Sff }` renamed to **`Ignorance { Normally, Foolishly }`** with accessor
  `how_ignorant()`; the separate `block_brane_searches` bool is derived from
  `Ignorance::Foolishly` and dropped as a field. Anchored search helpers move from free
  functions in `search.rs` onto `BraneFir` as inherent methods (`search(pattern, from,
  to)`, `index(n)`, `head()`, `tail()` — §10.2).
- **State machine**: `Nyes` unchanged in meaning; it now lives canonically on the
  `ProtoBrane` node as `nyes`. The exact transition table is **not** redefined by this FOOP
  — UBCa's first pass mirrors UBC's transitions (§3.3). Search-specific
  `Econstanic`/`Woconstanic` nuance is preserved (the search's `fir_op_step` drives those
  transitions).
- **New transient field**: `tasks: VecDeque<FirRef>` — evaluation-time only, built at
  `Embryonic`, not parsed and not serialized.
- **`FirRef` changes type**: from `Rc<RefCell<dyn Steppable>>` to **`Rc<RefCell<dyn Fir>>`**
  (the new narrow trait). The old `Fir` *enum* and `clone_into_fir` match-dispatch are
  retired in UBCa; shared topology code is inherent on `struct ProtoBrane`, per-kind
  behavior dyn-dispatched through `trait Fir` (§1 composition model). This also means the
  `Evaluator` trait (`snapshot_suite.rs`, pinned
  to `dyn Steppable`) must be **genericized over the FIR ref type** (or fronted by a thin
  adapter) for UBCa's own snapshot harness — ~50 lines.
- **Woconstanic short-circuit is preserved** (Deepseek #9). Current `wo_short_circuit`
  (`fir.rs:1770-1789`) follows a Woconstanic chain and collapses it; this is
  **snapshot-visible** (the short-circuited end differs from the chain). UBCa must reproduce
  it: a search whose target chain is Woconstanic copies the short-circuited **end** value into
  its own `ubc_children` (collapsing the chain), matching current output exactly.
- **`has_unresolved_forward_refs`** is re-expressed as a walk over `foolish_children` +
  `ubc_children` (replacing the `clone_into_fir()` match-dispatch), preserving its meaning
  (it gates the `Woconstanic && !forward_refs` break).
- **Serialization**: the snapshot/`.snap` format must be **byte-identical** — UBCa is only
  accepted if it reproduces existing approved snapshots. The internal struct layout
  changes, but serialized output must not. The parent `Weak` continues to serialize as
  `none` (existing `ParentPtr` behavior — parents are never serialized; they are rebuilt on
  load), so the self-referential root link has no serialization footprint.

## UBC Step Impact

Significant, but **behavior-preserving by construction** (the snapshots are the oracle):

- **Before**: per-struct `step_one` (state advance) + brane `step_members` (walk
  statements), driven by `run_to_completion_with_scope`'s `Nyes`-fixpoint loop with a
  `max_steps` guard and the `Woconstanic`/forward-ref break.
- **After**: NYES-driven `step()` via a per-node **task list** (§3): work the front task one
  transition; pop when it goes constanic; when the queue empties, run `fir_op_step` (which may
  push result tasks + `ubc_children` and advance `nyes`); empty queue ⇒ node must be
  constanic. The transitions **mirror UBC** (§3.3); the default `fir_op_step` is
  `compute_brane_state`'s Woconstanic/Constant/Nk classification.
- **Termination is redefined over PROGRESS, not root `prev==new`** (Deepseek #1). While a
  node's task queue drains, the *root's own* NYES can sit at `Braning` for many calls — so the
  old `prev_state == new_state` break would stop the loop early (it assumes every call changes
  root NYES). **Fix**: `step()` returns a progress signal (e.g. `(Nyes, Progress)`), and the
  outer loop stops only when the root is constanic OR **no progress** was made (no NYES change
  *and* no task-queue change anywhere). The `Woconstanic && !forward_refs` break and
  `max_steps` guard are retained. The draft's "retained unchanged" claim is **withdrawn** —
  the loop changes this much.
- **Step counts are NOT an acceptance constraint.** Verified: snapshots contain only formatted
  sequencer output (INPUT/RESULT/COMMENTS/signature — `snapshot_suite.rs:135`,
  `FirSequencer::format`), **no step count**. `format_with_header`'s step count is REPL/debug
  only. So the one-transition-per-call granularity (which differs from UBC's batched
  `step_boxed`) does **not** affect snapshot byte-exactness. The acceptance gate is byte-exact
  **sequencer output**, full stop. (This corrects the earlier draft, which wrongly asserted
  step counts must match — see the feedback synthesis.)

## Test Plan

- **UBCa's own snapshot suite is the primary acceptance gate.** A harness runs every `.foo`
  in `foolish-ubca/snapshot_tests/input/` through UBCa and asserts its **humanizing-sequencer
  output is byte-exact against UBCa's own approved `.snap` corpus**. Step counts are not in
  snapshots and are not compared (see UBC Step Impact).
- **UBCa is NOT diffed against UBC.** UBC is no longer an authoritative oracle (its own
  approved snapshots have drifted from its evaluator). The cross-check-against-UBC requirement
  is **removed**. UBC may be consulted informally, but matching UBC is not a gate.
- **Seeding the corpus**: UBC's `.foo` *inputs* may be copied as starting points; UBCa's
  approved `.snap` files are then established and reviewed on UBCa's own terms. No `.snap` is
  accepted without human review (AGENTS.md no-auto-accept rule).
- **New unit tests** (write FIRST, per AGENTS.md dev process) covering:
  - `foolish_children` immutability (no public mutator; length stable across stepping).
  - `ubc_children` expand AND shrink (push a result; clear on re-step; re-derive).
  - Task-list drain (§3): `step()` works the front task and pops it when constanic;
    `fir_op_step` runs only when the child tasks are drained; `fir_op_step` pushing a
    result task gets drained before the node settles; empty task list ⇒ node constanic
    (the terminal `debug_assert`); `step()` returns the node's NYES.
  - `step()` granularity: one NYES transition per call; child progresses
    Prembrionic→Embryonic→Braning→constanic over successive calls (mirrors UBC).
  - **Parent link on computed children**: ancestral search that must resolve *through* a
    value placed in `ubc_children` (guards the parent-wiring risk in §5).
  - Concatenation: `k` inputs in `foolish_children`, `k+1`th result in `ubc_children`,
    appears only when all inputs constanic; re-step clears and rebuilds it.
  - Single-child-as-len-1-ProtoBrane (SF/SFF, statement body) and len-0 leaves.
  - **Quiescent-Representation Invariant (§9.0)** — between `step()` calls, a FIR's structure
    agrees with its nyes (operational); when nyes ∈ {Constant, Independent}, it denotes its
    genuine value (denotational). Assert at every quiescent point in a stepped tree.
  - **In-place stepping is correct (§9)** — stepping shared `foolish_children` in place yields
    the right rendered states (§9.0); independence where needed comes from `constanic_clone`.
    Test that a referrer sees a body's advancing nyes, and that a search result is a
    `constanic_clone` (independent of its source), NOT a shared alias.
  - **Re-step rebuilds the task list (§9)** — a child popped while `Econstanic` is re-enqueued
    when re-stepping; a later forward-binding then advances it (guards the Econstanic-pop trap).
  - **Termination over progress (§7)** — a brane whose root NYES sits at `Braning` for many
    calls does NOT stop early; the loop stops only on constanic-or-no-progress.
  - **Woconstanic short-circuit** preserved — chain collapses into `ubc_children`, output
    byte-exact against UBCa's own approved snapshot.
  - **`Ignorance` carry-over (§10.1)** — `Foolishly` (SF): a found brane is not consumed
    (search goes Econstanic); SFF: children instantiated as ECONSTANIC via builder, no searches run;
    threaded via `Scope` into the child's step.
  - **Scope capability surface (§10)** — `search_ib` finds only names BEFORE
    `current_stmt_idx`; `search_ab` widens via `get_parent_brane()` bounded at each
    level by `get_parent_statement().get_line_number()` and stops at `is_root()`;
    no public positional getters.
  - **Upward navigation trio** — `get_parent()`, `get_parent_statement()`,
    `get_parent_brane()` from arbitrary depths (expression nested in operator nested in
    statement nested in brane); no downward containment scan anywhere.
  - **Range checking, UNIT tests (per the 2026-06-10 ruling)** — unanchored
    `scope.index(offset)`: boundary `-k` accepted, `-(k+1)` ⇒ NK, `0` ⇒ NK, positive ⇒ NK;
    anchored `brane.index(n)`: `n≥0` from front, `n<0` from back, both-signs out-of-bounds
    ⇒ NK; `head()`/`tail()` on the empty brane.
  - **Range checking, SNAPSHOT coverage** — `.foo` inputs demonstrating out-of-range
    unanchored and anchored indexes render as NK in sequencer output.
  - **BraneFir anchored surface (§10.2)** — `search(pattern, from, to)` with `from > to`
    searching backward; `index(n)` from front (n≥0) / back (n<0); `head()`/`tail()` =
    `index(0)`/`index(-1)`; byte-parity with today's `search.rs` free functions.
  - **Deep nested search through the drain** — a statement body several branes deep
    resolves an outer name via `scope.search_ab` while being stepped by `step_fir_ref`
    (guards the transient-borrow discipline; the nested-`borrow_mut` shape panics here).
  - **Nk in the task queue** — an Nk child is popped (`is_settled`), the parent classifies
    Nk, no stall.
  - **Unresolvable forward ref terminates cleanly** — via `NoProgress`, NOT via the
    `max_steps` guard.
- **Existing tests to update**: only UBCa-side; UBC tests are left as-is (UBC is not the
  acceptance oracle — UBCa is validated against its own snapshots).

## Rejected Alternatives

### A. Do nothing (keep `Steppable` + `ChildrenItr`)

Leaves the god-trait, the duplicated topology, the three parallel child-access methods,
and the `unsafe` iterator arm in place. Rejected: the children-access irregularity was the
original complaint, and the duplication actively fights the encapsulation and
small-trait rules in `rust_instructions.md`.

### B. Undistinguished `FirNode` with a single positional children vec

Collapse all variants into one node whose children are a single positional
`Vec<children>`, re-encoding anchor/target/merged as index conventions. Rejected: it
destroys the typed distinction between, e.g., a search's *anchor* and its *result* (turning
them into index lore — exactly the "illegal states representable" anti-pattern), and it
does not capture the parse-time vs compute-time provenance that the two-store model makes
explicit.

### C. Mutable children iterator (`children_itr_mutable` yielding `RefMut`)

The interface the discussion opened with. Rejected: yielding multiple `RefMut` guards over
`Rc<RefCell<_>>` slots can panic at runtime if any alias; even `RefMut::map`/`map_split`
only narrows a *single* parent-vec borrow. The task-list step (work the front task one
transition at a time, mutate only own `tasks`/`ubc_children`) achieves all required mutation
without the hazard.

### D. In-place full rewrite of `foolish-core`

Gut `fir.rs`/`ubc.rs` directly. Rejected: a high-risk representation change is safer in a
fresh, isolated crate (`foolish-ubca`) that can be validated against its own snapshot corpus
while the old code keeps the workspace building. (Note: the original rationale also cited UBC
as a byte-for-byte diff oracle; that no longer holds — UBC's snapshots have drifted from its
evaluator and UBCa is its own source of truth — but the isolation argument still stands.)

## Open Questions

- ~~UBCa placement~~ — **DECIDED**: a new sibling crate **`foolish-ubca`**, like
  `foolish-ubcb` (§7). The original UBC stays in `foolish-core`.
- ~~Task-queue type~~ — **DECIDED**: `std::collections::VecDeque<FirRef>` (§3.1).
- **`Nyes` ownership detail**: the `nyes` field lives on the `ProtoBrane` node, read via
  public `get_nyes()`, written only by init/step/constanic-clone (decided). Confirm the
  search `Econstanic`/`Woconstanic` transitions need nothing payload-resident beyond what
  `fir_op_step` can set.
- ~~`ubc_children` ordering guarantee~~ — **DECIDED: order IS significant** (snapshot-visible
  via the sequencer's `result=` rendering; see §1 and §8). Results are pushed in render order.
- ~~`bon` value→builder API shape~~ — **DECIDED** (§6b): `#[builder(on(_, overwritable))]`
  + a hand-written `updater(self) -> XFirBuilder` per payload (`bon` has no auto value→builder).
- ~~Composition model~~ — **DECIDED** (§1): kinds CONTAIN a `struct ProtoBrane` (field-holder
  with inherent topology methods); `trait Fir` (dyn-dispatch, per-kind `fir_op_step` +
  `core()`/`core_mut()`); `step_fir_ref` is a free function over `&FirRef`.
- ~~`FirRef` type~~ — **DECIDED**: `Rc<RefCell<dyn Fir>>` (trait object; old enum +
  `clone_into_fir` retired).
- ~~Parent construction~~ — **DECIDED** (§5.1): nested `Rc::new_cyclic`, parent immutable at
  construction; compiler threads `Weak` down (Phase 3b).
- ~~Step counts as acceptance~~ — **RESOLVED (review)**: NOT an acceptance constraint; snapshots
  carry no step count. Byte-exact sequencer output is the gate.
- **Finite-time-to-constanic guarantee**: §3 relies on the language guaranteeing every Fir
  reaches a constanic NYES in finite time (so the task-list drain terminates). Locate where
  this guarantee is documented (a FOOP, STYLES, or ECOSYSTEM) and cite it; if it is only
  implicit, this FOOP should state it explicitly.
- **Task-list NYES transitions**: the exact per-kind progression is the implementer's choice,
  pinned by UBCa's own approved snapshots (§3.3). UBC's transition table may be consulted as a
  reference but is not authoritative.
- ~~Unanchored `index` range~~ — **RULED by Atlas (2026-06-10)**: the two index forms are
  distinct operations. UNANCHORED index (syntax `a = #-1 + #-2`) permits ONLY negative
  offsets, valid `[-k, -1]` for statement k; out-of-range (including 0 and positives) ⇒
  NK. ANCHORED index (syntax `b#1`, `b#-1`) permits both signs (n≥0 from front, n<0 from
  back — §10.2). Current UBC's `IndexFir::step_unanchored` (`fir.rs:1874`) accepting
  positive in-range offsets is therefore a **latent UBC bug**, not language behavior.
  Residual Phase 0 task: verify no approved snapshot exercises a positive unanchored
  offset (if one does, byte-exactness and the ruling collide — escalate to Atlas).
  Range checking gets BOTH unit tests (the `[-k,-1]` boundary, rejection of 0/positive,
  anchored both-signs/out-of-bounds) AND snapshot coverage showing out-of-range ⇒ NK.
- ~~`block_brane_searches` ↔ `Ignorance::Foolishly` mapping~~ — **VERIFIED (2026-06-10)**:
  `block_brane_searches` is never set `true` anywhere in the workspace (declared, defaulted
  false, cloned, read — never written). The flag is vestigial; SF's brane-blocking actually
  lives inside the `step_except_brane_searches` algorithm (hardcoded brane check at
  `ubc.rs:385/428`). Dropping the bool is safe. The real obligation transfers to
  `Ignorance::Foolishly`: it must reproduce what `step_except_brane_searches` does — a
  found BRANE is not consumed (search goes Econstanic); non-brane results resolve normally.
- ~~Incremental scope threading (§9)~~ — **PROPOSED RESOLUTION (§10.4)**: the StatementFir
  boundary builds the scope for its body from its own `line_number` + `parent`; no brane
  `step` override. Validate in Phase 3; brane-wrapper remains the fallback.

## Relationship to existing FOOPs

This FOOP does not invent its semantics — it **unifies the representation** behind several
already-accepted FOOPs, which is corroborating evidence the model is right:

- **FOOP-9** (Implementing) — "Operators are brane-like FIRs with positional unnamed
  operands and no search boundary." This is exactly "an operator is a ProtoBrane whose
  `foolish_children` are its operands." FOOP-62 generalizes it to every node.
- **FOOP-3** (Brewing) — "Concatenation produces a new brane of constanic-cloned elements;
  further steps delegate to the merged brane." That merged/result brane is precisely the
  `k+1`th child in `ubc_children`. FOOP-62 gives it a structural home.
- **FOOP-7** (Brewing) — Constanic Clone recoordination contract. The `clone_with_parent`
  re-parenting move in §5 is this contract; FOOP-62 makes clone-with-new-parent the *only*
  way a parent differs.
- **FOOP-5** (Final) — compile-time vs evaluation-time FIR contract. The
  `foolish_children` (parse-time) / `ubc_children` (eval-time) split is the structural
  expression of that contract.
- **FOOP-31** (Draft) — SPA1, the UBC reference implementation. UBCa is built beside it and
  validated against it.

- **FOOP-8** (Brewing) — "FIRs are mutable; parent pointers are post-clone; Circe excludes
  parent." FOOP-62 is **fully consistent** with all three clauses:
  - *FIRs are mutable* — **yes, they still are.** Under FOOP-62 `ubc_children` grows and
    shrinks, every `foolish_children` member steps and internally mutates, and `state`
    advances. FOOP-62 does not make FIRs immutable; it only pins down *which two things*
    are fixed: the **`foolish_children` vector shape** (its slots — the parse-time
    topology) and the **`parent` field**. Everything else mutates as before.
  - *parent pointers are post-clone* — preserved exactly; `clone_with_parent` is the
    post-clone parent assignment.
  - *Circe excludes parent* — preserved; parent is never serialized.

  So FOOP-62 **refines** FOOP-8 (naming the two fixed pieces) rather than tensioning with
  it. No `supersedes` edge is needed; a back-reference note on FOOP-8 is optional.

## References

- Prior FOOPs: FOOP-52 (scope/search rework); FOOP-9, FOOP-3, FOOP-7, FOOP-5, FOOP-8,
  FOOP-31 (see "Relationship to existing FOOPs").
- Memory: [[foop52-scope-architecture]], [[foop52-sf-sff-and-search-classification]],
  [[foop52-encapsulation-rule]], [[foop62-ubca-two-store-protobrane]].
- Code locations: `foolish/foolish-core/src/fir.rs` (Fir/Steppable/ChildrenItr structs),
  `foolish/foolish-core/src/ubc.rs` (Scope, run_to_completion, re_step_brane_bodies),
  `foolish/foolish-core/src/snapshot_suite.rs` + `ubc_snapshot_tester.rs` (test harness),
  `foolish/foolish-core/snapshot_tests/` (inputs + approved snapshots).
- AGENTS.md: "Foolish Semantic Immutability vs FIR Evaluation State"; snapshot
  no-auto-accept rule. `rust_instructions.md`: small-trait, encapsulation, illegal-states
  rules.

## Last Updated

**Date**: 2026-06-22 (revision 16 — anchor/result terminology + ???-LHS + impl docs)
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.8
**Changes**: Added a "Terminology: anchor and result (NOT 'target')" section formally defining
**anchor** (the FIR a search searches within/relative to) and **result** (the FIR a search
produces), and recording the correction away from the banned word "target". §8 already
documents the singular-result invariant, the `result`-vs-`anchor` rendering rule, and the
`???`-LHS rule (anonymous statements named `???` render without a `name=` prefix). These
capture the FOOP-62 #11/#13/#17/#19 implementation. Synced to alpha.

**Date**: 2026-06-19 (revision 15 — UBCa is its own source of truth; drop "match UBC")
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.8
**Changes**: REMOVED the "UBCa must match UBC byte-for-byte" requirement throughout (Atlas
ruling). UBC is no longer an authoritative oracle — its committed approved snapshots have
drifted from its own evaluator (confirmed by stashing all session changes: deep VALUE diffs,
not just state labels). UBCa is now validated **byte-for-byte against its OWN approved snapshot
corpus**. Scrubbed the Abstract, §7 (crate layout / snap copy), §8 (acceptance rule), §9
(transition oracle), Test Plan (oracle cross-check removed), FIR Impact (harness), and Rejected
Alternative D. "byte-exact" as the snapshot mechanism is retained — only the *cross-check
against UBC* is removed. Synced to alpha + the -mimo plan (Phase 1 & 4 reworded).

**Date**: 2026-06-19 (revision 14 — ignorance terminology + foolish-flag model)
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.8
**Changes**: Merged the `alpha` spec with the `foop-62-ubca-mimo` implementation's design and
defined the **ignorance** terminology in its own section, used consistently in §6b/§9.x/§10:
- **normally ignorant** = `constanic_clone` with `descendent_of_sfm_and_foolishly_ignorant =
  false`: NYES-transfer rule — constanic NYES copied unchanged, pre-constanic → PREMBRYONIC.
- **foolishly ignorant** = the same clone with the flag `true`: **ALL NYES copied unchanged**.
  The flag is sourced from a new `Scope` field **`has_ancestral_sfm: bool`** (true inside an
  SF-mark's RHS); `step()` seeds each clone with `scope.has_ancestral_sfm`, and clone's own
  recursion inherits the **caller's** flag (two independent recursions).
- **THE BIG BUT**: a later *search* that clones an SF-mark **strips the mark** and runs with
  the flag `false` (normal), so the inner value re-resolves.
- **fully foolish** = SFF construction (descendants built ECONSTANIC); not a clone.
There is no live `enum Ignorance` / `Scope.ignorance` — this answers "why is there no enum for
ignorance." Verified ground truth against `-mimo`: `Scope` is a 2-field stub in
`fir_trait.rs`, `step` already takes `&Scope`, `bon` is not used, and
`constanic_clone_normal_at` has no flag and re-wraps SF/SFF + hard-codes PREMBRYONIC for
compounds — all flagged as the ⚠ implementation gap and tracked in FOOP-62.plan.md Phase −1.
This spec is placed in both the `alpha` and `foop-62-ubca-mimo` worktrees.

**Date**: 2026-06-14 (revision 13 — HFS constant rendering decision)
**Updated By**: Sisyphus / mimo-v2.5-pro
**Changes**: Added "HFS rendering of constant search results" decision: searches that
resolve to constants can render as the constant directly. Approved sf_non_brane_resolves
snapshot based on this decision.

**Date**: 2026-06-14 (revision 12 — NYES classification clarification)
**Updated By**: Sisyphus / mimo-v2.5-pro
**Changes**: Added §3.3.1 "NYES classification for Brane and Operator" — clarifies that
nodes whose children are ECONSTANIC do NOT automatically become WOCONSTANIC. Brane/Operator
stays BRANING until ALL children are constanic AND some are WOCONSTANIC/ECONSTANIC.

**Date**: 2026-06-14 (revision 11 — remove Fully ignorance, SFF clarification)
**Updated By**: Sisyphus / mimo-v2.5-pro
**Changes**: Removed `Fully` from `Ignorance` enum. SFF marker means children are
instantiated as ECONSTANIC via builder setup (not cloning). SFF never clones because
it executes zero searches. Added comment that "Fully Foolish constanic clone does not exist."
Updated all references to `Fully` throughout the spec.

**Date**: 2026-06-14 (revision 10 — constanic-clone semantics, SF/SFF clarification)
**Updated By**: Sisyphus / mimo-v2.5-pro
**Changes**: Clarified constanic-clone behavior by ignorance flag:
- Foolishly (SF): constants/independents referenced, constanic states copied with state, no new searches triggered
- Normally: constants/independents referenced, constanic states reset to pre-constanic for re-evaluation
- SFF: children instantiated as ECONSTANIC via builder, no searches run, no cloning
Updated Ignorance enum comment to reflect SF runs searches and uses special constanic-clone.
Added SF constanic clone behavior section.

**Date**: 2026-06-13 (revision 9 — runtime safety, depth limit, panic resilience)
**Updated By**: Sisyphus / mimo-v2.5-pro
**Changes**: Added §3.4 "Runtime safety: depth limits and panic resilience in tests".
Covers depth limit parameter for `step_fir_ref`, `panic = "unwind"` test configuration,
`catch_unwind` in snapshot harness for graceful RefCell panic capture, `#[should_panic]`
for borrow discipline tests, and `try_borrow()` for defensive checks.

**Date**: 2026-06-11 (revision 8 — terminology, ORIGIN, snap copy, job queue)
**Updated By**: Sisyphus / mimo-v2.5-pro
**Changes**: (a) Added **§Terminology** section defining "constanic" for UBCa: includes
ECONSTANIC, WOCONSTANIC, CONSTANT, INDEPENDENT, and NK (all settled states); pre-constanic
(nigh) = PREMBRYONIC/EMBRYONIC/BRANING requiring further stepping. (b) Added **ORIGIN
variables** to worktree declaration per updated foop.md template. (c) Standardized worktree
name to `foop-62-ubca-mimo` (reconciled spec header with plan). (d) **§4 rewritten**:
deprecates UBC's `re_step_brane_bodies` entirely; UBCa uses a per-node job queue where the
front child is stepped until settled (constanic including NK), then popped; guarantees later
children always result constanically in finite steps. (e) **§7 clarified**: UBC's snap tests
(input/ and approved/) are copied to UBCa as-is without change; the humanizer sequencer is
developed for UBCa's new FIR classes (ProtoBrane two-store structure).

**Date**: 2026-06-10 (revision 7 — worktree declaration)
**Updated By**: Claude Code 2.1.119 (Claude Code); Sonnet 4.6
**Changes**: Replaced "WORKED IN PLACE" on FOOP-52 worktree with a dedicated worktree
declaration: `foop-62-ubca-mimo` at
`${HOME}/tmp/foolish-worktrees/foop-62-ubca-mimo`. Removed stale
"same worktree/branch as FOOP-52" references from the header and References section.

**Date**: 2026-06-09
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.8
**Changes**: Initial draft, then revised the stepping model. Two-store ProtoBrane
(foolish_children fixed + ubc_children mutable). **§3 stepping reworked to NYES-driven via a
per-node task list** (replaced the earlier "wait-for-all-children-then-run two-phase" model):
`step() -> Nyes`; `tasks: VecDeque<FirRef>` built at Embryonic; work the front task one
transition, pop when constanic; when empty run `fir_op_step` (may push more tasks +
ubc_children + bump nyes); empty list ⇒ node constanic; one NYES transition per `step()`.
NYES transitions are implementer's choice and the **first pass mirrors UBC** (§3.3); the
default `fir_op_step` is UBC's `compute_brane_state` Woconstanic/Constant/Nk classification.
Clone-and-gut UBC→UBCa with UBC as correctness oracle (finalized .snap corpus brought over).
Parent link first-class (§5): Weak, immutable-after-construction, root is self-Weak via
is_root(). §6 builders via `bon` (approved): builder is the ONLY Fir construction path,
enforced by language (private fields + private module + non_exhaustive); constanic clone =
build-from-existing-value + field override. `nyes` is read-only from outside (`get_nyes()`;
no public setter); written only by init/step/clone. FOOP-62 refines (does not contradict)
FOOP-8's "FIRs are mutable" — FIRs remain mutable; only foolish_children shape + parent are
fixed. Resolved four Open Questions: UBCa is a sibling CRATE `foolish-ubca` (like
`foolish-ubcb`), task queue is `std::VecDeque`, **`ubc_children` order IS significant**
(snapshot-visible via sequencer `result=`), and the `bon` clone mechanism is the
`updater(self)` + `#[builder(on(_, overwritable))]` pattern. Added §8: the humanizing
sequencer is a HARD byte-exact acceptance constraint (it produces the approval tests);
FirQueryable retirement is a goal but byte-exact output is the gate.

**Date**: 2026-06-09 (revision 2 — post Deepseek + mimo review)
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.8
**Changes**: Folded the consolidated review feedback (`FOOP-62.feedback-synthesis.md`).
(1) **Composition model** (§1): kinds CONTAIN a `struct ProtoBrane` (field-holder with
inherent topology methods); `trait Fir` (dyn-dispatch surface, per-kind `fir_op_step` +
`core()`/`core_mut()`); `step_fir_ref` free function over `&FirRef`; `FirRef = Rc<RefCell<dyn Fir>>`
(enum + clone_into_fir retired). (2) **Construction** (§5.1): nested `Rc::new_cyclic`, parent
immutable at construction, compiler threads `Weak` down. (3) **Termination** (§7): redefined over
PROGRESS (`step()` returns `Step{nyes,progress}`); withdrew the "retained unchanged" claim.
(4) **Step counts are NOT an acceptance constraint** — verified snapshots carry no step count;
corrected all "must match step counts" wording to "byte-exact sequencer output." (5) New **§9
NormalBrane & Statement stepping**: StatementFir-as-Fir, incremental scope owned by brane drain,
re-step rebuilds tasks, Econstanic-pop trap, in-place-vs-clone equivalence. (6) §8: thin
`FirQueryable` adapter is now the DEFAULT, not fallback. (7) Woconstanic short-circuit, Evaluator
genericization, has_unresolved_forward_refs, EvalContext carry-over all addressed in FIR Impact /
plan. Resolved Open Questions: composition, FirRef type, parent construction.

**Date**: 2026-06-09 (revision 3 — Atlas clarifications on the survivors)
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.8
**Changes**: (a) Added **§9.0 Quiescent-Representation Invariant** as a MANDATE: when `step()`
is not running, every FIR must faithfully represent its nyes — operationally at all quiescent
points, and denotationally (a genuine value) when nyes ∈ {Constant, Independent}. This is the
property the whole model rests on; in-place stepping, `constanic_clone`, and the sequencer's
byte-exactness all derive from it. (b) **In-place stepping is correct by §9.0** — removed the
"validate then maybe clone" hedge; independence comes from `constanic_clone`, never defensive
copying in the drain. (c) **SF/SFF do NOT override `step`** — corrected the review's reading:
their difference is construction-time state (+ `constanic_clone` rebuild), not a separate
stepping algorithm. `step` is the shared default for ~every kind. (d) **Statement metadata via
the builder** — a statement is a len-1 ProtoBrane carrying name/line as leaf data, fed
parent/line/body through its builder at construction (no parallel vector). Only remaining
`step`-override candidate is the brane's incremental `current_stmt_idx` (flagged, try
construction-state first).

**Date**: 2026-06-10 (revision 4 — borrow-discipline fixes + rename)
**Updated By**: Claude Code 2.1.119 (Claude Code); Sonnet 4.6
**Changes**: (a) **Composition renamed**: `ProtoBraneImpl` → `struct ProtoBrane` (field-holder
with inherent topology methods, matching the `Links`-struct pattern from the reference tree
example); `trait ProtoBrane` eliminated — the dyn-dispatch surface is solely `trait Fir`
(`core()`/`core_mut()`, `fir_op_step`, `kind`, leaf accessors). (b) **`step_fir_ref` free
function** (`&FirRef, &Scope`) replaces the trait-method `step()` — this is the borrow-
discipline fix: each level reads into locals under a transient borrow (dropped before the
recursive call or `fir_op_step`), preventing the `RefCell already mutably borrowed` panic that
arises when a descendant's `fir_op_step` does ancestral resolution while ancestor `borrow_mut`
guards are live on the stack. Confirmed experimentally in `/tmp/foop62-lang-experiment`.
(c) **`StepReport` enum** (`NoProgress` / `Progress(Nyes)`) replaces `Step{nyes,progress}`;
Progress is reported even when NYES is unchanged (e.g. pop-only call). (d) **`is_settled()`**
added to `Nyes` — `is_constanic() || == Nk`; used as the task-queue pop predicate so Nk
terminals don't block the drain. `is_constanic()` retained as the outer acceptance predicate.
(e) §5.1 `Rc::new_cyclic` snippet corrected: closure receives `&Weak<RefCell<BraneFir>>`
(concrete type); unsized coercion to `Weak<RefCell<dyn Fir>>` shown explicitly at a typed
`let`. (f) §7 cross-check clause "and identical step counts" removed — step counts are not
snapshot-visible and the two models differ in granularity by design. (g) §3.3 `Plus` example:
`self.weak_self()` replaced with `self.core.parent.clone()` + `core_mut().push_ubc_child(…)`.
(h) dyn-dispatch role of `trait Fir` and `FirRef = Rc<RefCell<dyn Fir>>` made explicit in §1.

**Date**: 2026-06-10 (revision 5 — Scope as the search-capability surface)
**Updated By**: Claude Code 2.1.119 (Claude Code); Fable 5
**Changes**: New **§10 Scope rework** per Atlas's direction. (a) `Scope.entries` (flat
name→FirRef list) is REMOVED — name resolution reaches back through the parent pointer and
asks methods on the parent FIR recursively; the tree is the name table, nothing accumulates.
(b) Scope becomes the **active search-capability surface**: positional fields go private;
public methods are `search_ib(pattern)` (backward in immediate brane), `search_ab(pattern)`
(IB then recursive parent-chain widening), `index(offset)` (offset relative to the current
statement k, valid `[-k, -1]`), `get_ignorance()`, `emit(alarm)`. `scope.index(-1)` is
enough — Scope holds the line number. (c) **`EvalContext { Normal, Sf, Sff }` renamed
`Ignorance { Normally, Foolishly }`**; `block_brane_searches`
derived from `Foolishly`, dropped as a field. (d) **Anchored searches live on `BraneFir`,
not Scope** (§10.2): `search(pattern, from_idx, to_idx)` (from > to ⇒ backward),
`index(n)` (n≥0 front / n<0 back), `head()`/`tail()` = `#0`/`#-1`; replaces `search.rs`
free functions (encapsulation rule). (e) **Scope built via `bon`** (§10.3), same fluent
style as FIR builders. (f) **§10.4 PROPOSED resolution of §9 scope-threading**: the
StatementFir boundary builds its body's scope from its own `line_number` + `parent` — no
brane `step` override. (g) New Open Questions: unanchored-index positive-offset discrepancy
vs current UBC (snapshot-relevant), `block_brane_searches`↔`Foolishly` coupling check.
Test plan extended: scope capabilities, BraneFir anchored surface, deep nested search
through the drain, Nk-in-queue, clean NoProgress termination.

**Date**: 2026-06-10 (revision 6 — Atlas rulings on rev-5 concerns)
**Updated By**: Claude Code 2.1.119 (Claude Code); Fable 5
**Changes**: (a) **Index range RULED**: unanchored index (`a = #-1 + #-2`) permits ONLY
negative offsets, `[-k, -1]`; out-of-range (incl. 0/positive) ⇒ NK; anchored index
(`b#1`/`b#-1`) is a distinct operation, both signs valid. UBC's positive-offset acceptance
in `step_unanchored` is a latent bug; residual corpus check stays in Phase 0. Range checks
get BOTH unit tests and NK snapshot coverage. (b) **`get_ignorance()` renamed
`how_ignorant()`** — asked and matched as an adverb (Normally/Foolishly). (c) **Upward
navigation trio**: every Fir exposes `get_parent()` / `get_parent_statement()` /
`get_parent_brane()` (inherent on ProtoBrane, walking the parent Weak chain);
`BraneFir::search_ab(pattern, from_line)` recurses upward obtaining each level's bound via
`get_parent_statement().get_line_number()` — the downward containment scan
(`line_of_child`/`contains_fir`) is retired entirely, and with it the ptr_eq-before-borrow
hazard (ptr_eq remains only in `is_root()`). (d) §10.4 sharpened: statement-built scope
supplies position only; ignorance/alarms are INHERITED (merged, never reset); root and
anchor evaluations construct scope explicitly. (e) Dead `block_brane_searches` flag removal
confirmed by Atlas ("keep code clean").
(f) Edge-position ruling: the upward trio returns `Option`; `None` ≡ search miss
(unbound ⇒ Econstanic), same terminal as `is_root()`.
