---
foop: 16
title: UBCb State Machine — Per-Variant NYES Table
author: hc <hc.busy@gmail.com>
status: Draft
type: Standards
created: 2026-05-09
phase: phase-2
supersedes: []
---

# FOOP-61: UBCb State Machine — Per-Variant NYES Table

## Abstract

Specifies the NYES state machine for **UBCb** (the foolish-mvp
brane-driven variant of the brane computer). UBCb is **not** the
message-passing system described in the historical UBC2 design
documents. UBCb is **brane-driven**: each brane's `step()` method
advances itself and its members through a fixed sequence of NYES
states. There is no scheduler, no message protocol, no listener
tables.

This FOOP defines:

1. The set of NYES states UBCb uses.
2. A universal contract every "proto-brane" (the abstract supertype of
   all FIR variants) must obey.
3. A per-variant table specifying, for each (variant, state) pair:
   - what activities the variant performs in that state, and
   - what conditions trigger transition to which destination states.

This FOOP works in concert with FOOP-7 (constanic clone) and FOOP-51
(AB list, search_result, short-circuit accumulation). Constanic-clone
permission rules are defined per state in this FOOP and consumed by
FOOP-7.

## Motivation

Three problems motivate this FOOP:

1. **Ambiguity of NYES states across variants.** UBC's existing Nyes
   enum has 8 variants but the lifecycle each FIR variant follows is
   not uniformly documented. Operators have no real BRANING phase;
   Searches have no children; ConstantInts are born terminal. The
   meaning of EMBRYONIC, BRANING, and the constanic states differs
   subtly per variant, and that difference lives only in code.

2. **Constanic clone permission rules.** Per FOOP-7, constanic clone
   is a structural AB extension. But not every NYES state is
   clone-safe. A per-state, per-variant specification of clone
   permission is required.

3. **UBCb's drift toward UBC2 design.** Earlier discussions about UBCb
   incorporated UBC2's message-passing concepts (LUIDs as routing
   addresses, `BLOCKED_ON_MESSAGE`, listener tables, stage-wise
   fairness scheduling). The actual UBCb in foolish-mvp does not use
   these. UBCb is brane-driven; the brane's step method advances the
   work. This FOOP records the brane-driven model unambiguously and
   discards the message-passing artifacts.

## NYES State Set

UBCb defines the following NYES states. All proto-brane FIR variants
use the same state set; behavior at each state is variant-specific
and is specified in the per-variant table below.

| State | Kind | Brief meaning |
|-------|------|---------------|
| `PREMBRYONIC` | NYE | Code present as String and AST. Not yet structured into StatementFirs. Brane: structural skeleton not yet built. |
| `EMBRYONIC` | NYE | Structurally prepared (immutable StatementFir array built; parent set if non-root). For containers: stepping their children to EMBRYONIC, then performing **local-IB-only** resolution (searches that don't cross brane boundaries). For search-bearing FIRs: a pass-through. |
| `BRANING` | NYE | Cross-brane work. For containers: AB walk for upward references(`parent.search()`); digging into referenced child branes; operator computation when operands are complete. For search-bearing FIRs: the active-work state for resolution that crosses brane boundaries. |
| `WOBRANING` | NYE-blocked | Waiting on a referenced child brane to enter EMBRYONIC so the search can dig into it. Distinct from WOCONSTANIC because the wait is on a NYE state, not on a constanic state. (Example: in `{a={something=5}; c=a.something}`, while resolving `c`'s anchored search `a.something`, `a` is still PREMBRYONIC; we cannot search into it; `c` enters WOBRANING until `a` reaches EMBRYONIC.) |
| `WOCONSTANIC` | constanic-pending | A *kind of* constanic state: "waiting on constanics" — constanic, but with descendants still resolving. (Same name and semantics as UBC's `WOCONSTANIC`.) This is a Constanic state that can be easily cloned|
| `NOTFOUNDIC` | constanic-recoverable | "Not Found In Context." Search/lookup has exhausted resolution in the current IB & AB. **Recoverable if cloned into a different context** — under FOOP-7's `constanic_clone` + caller's `.setParent(new_parent)`, the clone is reset to BRANING and re-walks against the **new parent chain** (its old AB stays unchanged but the new parent provides new ancestral context). Distinct from NK because cloning into a new host can rescue a NOTFOUNDIC FIR; cloning cannot rescue an NK. (Renamed from UBC's `ECONSTANIC`.) |
| `CONSTANT` | constanic | Value computed. Still attached (AB and parent populated). |
| `INDEPENDENT` | terminal | Value computed and detached (AB and parent cleared). Per FOOP-51 detach step. |
| `NK` | terminal | Error sentinel. Carries a reason field (e.g., `division-by-zero`, `infinite-recursion`, `invariant-violated`, `index-out-of-range`). Irrecoverable: cloning an NK produces another NK. |

**Notes on the state set:**

- NOTFOUNDIC corresponds to UBC's `ECONSTANIC` (renamed for clarity:
  "Not Found In Context"). Its semantics are sharper under FOOP-51's
  AB model: cloning is what makes NOTFOUNDIC recoverable, and cloning
  is now a structural operation (AB extension) rather than a
  recursive descent.
- The English word "constanic" (lowercase) is the categorical
  adjective meaning "constant in context" — the family of states
  {CONSTANT, WOCONSTANIC, INDEPENDENT, NOTFOUNDIC, NK}. WOCONSTANIC
  is one specific kind of constanic state: "constanic with descendants
  still resolving." A FIR that has reached any constanic state has, in
  some sense, taken its place in the value lattice; how settled it is
  (fully detached, awaiting children, exhausted-but-recoverable, or
  errored) distinguishes the specific state.
- The compiler's "unbound name detected at compile time" produces an
  NK with `reason='undefined-name'`, NOT a NOTFOUNDIC. NOTFOUNDIC is
  produced only at evaluation time by searches that exhausted their
  AB walk; it persists because re-coordination (clone) might still
  resolve it later.
- **WOBRANING tracks "waiting on a child brane to become EMBRYONIC so
  we can dig into it."** Example: while resolving the anchored search
  `c = a.something` in brane `{a={something=5}; c=a.something}`, the
  child brane `a` may still be PREMBRYONIC; we cannot search into a
  PREMBRYONIC brane. The pending search transitions to WOBRANING and
  waits. When `a` reaches EMBRYONIC, the dependent FIR returns to its
  active state (BRANING) and re-attempts the search.

**Terminal vs. non-terminal:**

- **Terminal:** `INDEPENDENT`, `NK`. The protobrane will never advance
  again. (NK is irrecoverable; cloning an NK produces another NK.)
- **Quasi-terminal:** `CONSTANT`. Will advance once more, to
  INDEPENDENT, via the FOOP-51 detach step.
- **Constanic-pending:** `WOCONSTANIC`. Will advance to either CONSTANT
  (then detach) or to NK, depending on descendants.
- **Constanic-recoverable:** `NOTFOUNDIC`. Terminal under stepping
  (will not advance further by self-stepping) but **recoverable
  through cloning** — when constanic-cloned into a new host, the
  clone has a new parent (set by the caller's `.setParent(...)`)
  and re-walks against the new parent's ancestral chain. The original
  NOTFOUNDIC FIR remains NOTFOUNDIC.
- **NYE:** `PREMBRYONIC`, `EMBRYONIC`, `BRANING`. Active work pending.
- **NYE-blocked:** `WOBRANING`. Active but waiting on a specific
  referenced child brane to become EMBRYONIC. Returns to BRANING (or
  whichever NYE phase it was in) once the dependency advances.

## The Proto-Brane Universal Contract

Every FIR variant inherits from `ProtoBrane` and must implement
`step()` per the contract below. The contract specifies, for each
state, *what step() must do* — the per-variant table specifies *how*.

### Universal step() contract

```
trait ProtoBrane:
    fn step(&mut self) -> StepOutcome:
        match self.state:
            PREMBRYONIC  => self.step_prembryonic(),  // structure: build StatementFirs
            EMBRYONIC    => self.step_embryonic(),    // local-IB resolution; step children to EMBRYONIC
            BRANING      => self.step_braning(),      // cross-brane work (AB walk; dig into branes; operator compute)
            WOBRANING    => self.step_wobraning(),    // check if blocking dependency advanced
            WOCONSTANIC  => self.step_woconstanic(),  // re-check children
            CONSTANT     => self.step_finalize(),     // FOOP-51 detach
            NOTFOUNDIC   => StepOutcome::NoOp,        // recoverable only via clone
            INDEPENDENT  => StepOutcome::NoOp,
            NK           => StepOutcome::NoOp,
```

**Two NYE active-work phases, separated by scope.**

- **EMBRYONIC** is for **local-IB work**: build/verify the structural
  representation (StatementFir array, parent pointer), step children
  to EMBRYONIC, and perform searches that resolve **without crossing
  brane boundaries**. A search that can be answered by sibling
  statements is resolved here. A search that requires entering a
  child brane or walking up to AB is *deferred* — it transitions to
  either WOBRANING (waiting on a child brane to become EMBRYONIC) or
  it becomes the brane's BRANING work.
- **BRANING** is for **cross-brane work**: walking AB upward to
  resolve names from ancestral context; digging into referenced child
  branes once they're EMBRYONIC; operator computation when all
  operands are complete.

**Stage-completion invariant.**

> A brane cannot transition to a constanic state from an incomplete
> EMBRYONIC state. EMBRYONIC must complete (all children EMBRYONIC,
> all local-IB searches either resolved or properly deferred to
> WOBRANING/BRANING) before BRANING begins. BRANING must complete
> before any constanic state. The progression is monotonic:
> PREMBRYONIC → EMBRYONIC → BRANING → {WOCONSTANIC | NOTFOUNDIC | CONSTANT | NK} → CONSTANT → INDEPENDENT.

By the end of EMBRYONIC, a brane is **ready to receive search requests
from other branes**. Other branes that referenced this brane (e.g.,
via anchored searches `a.something`) can now dig into its statement
array. This is the architectural reason for separating EMBRYONIC
from BRANING: EMBRYONIC publishes the brane's structural API to the
rest of the FVM.

**Why the separation matters.** It bounds the work each phase does.
EMBRYONIC is linear in the brane's own size (finite statements,
finite local searches). BRANING involves ancestral and cross-brane
work and may need to wait. The phases let the FVM reason about
"I can guarantee EMBRYONIC completion before some larger work begins"
— useful for the compiler-side step-to-EMBRYONIC optimization
(compile-time pre-stepping to either EMBRYONIC or BRANING per the
optimization level).

**Why constanic clone resets to BRANING (not EMBRYONIC).** A
constanic-cloned FIR has new ancestral context (via its new parent set by the caller) but its
**local-IB resolution is unchanged** (its children are
reference-shared in the recursive clone, with their own EMBRYONIC
work already done). The only thing that needs redoing is the BRANING
work: AB walk, cross-brane resolution. So BRANING is the correct
reset destination. Resetting to EMBRYONIC would force redundant
local resolution.

Each `step_<state>` method MAY:
- Mutate `self.state` to advance.
- Mutate `self.search_result` (Search variants only).
- Step a child once (containers only).
- Read AB and parent.

Each `step_<state>` method MUST NOT:
- Recursively step children to completion (UBCb advances one step at
  a time).
- Modify `self.ab` outside the FOOP-51 detach step.
- Replace `self` with a different FIR variant.

### Universal constanic-clone permission

Per FOOP-7, `constanic_clone` is callable **only on FIRs in constanic
states** — that is, WOCONSTANIC, NOTFOUNDIC, CONSTANT, INDEPENDENT, NK.
Calling it on a NYE-state FIR (PREMBRYONIC, EMBRYONIC, BRANING) is a
caller bug; debug builds assert.

In practice this precondition is automatic: callers (search resolution,
brane finalization, concatenation merge) only invoke constanic clone
on values that have already reached a constanic state.

Per-state behavior on clone:

- **WOCONSTANIC, NOTFOUNDIC:** clone with state reset (per FOOP-7, AB is NOT extended); clone's state is
  reset to BRANING; recursive descent into children rewrites parent
  pointers and resets WOCONSTANIC/NOTFOUNDIC descendants to BRANING.
  The clone re-walks its work against the extended ancestral context.
- **CONSTANT, INDEPENDENT, NK:** these states are context-immune. The
  source is shared by reference; no clone is produced. AB extension
  is observably a no-op for these terminal values.

(In earlier drafts of this FOOP, a `WOBRANING` state was provisionally
listed as clone-forbidden. WOBRANING has since been removed from the
NYES state set; embryonic dependencies are tracked via a separate
mechanism — see "Embryonic dependency tracking" below.)

### Universal CONSTANT → INDEPENDENT transition (FOOP-51)

Per FOOP-51, every variant that can reach CONSTANT does so first
(value computed, AB and parent still attached) and then performs a
discrete detach step on the next step() call:

```
fn step_finalize(&mut self):
    self.ab = empty
    self.parent = None
    self.state = INDEPENDENT
```

This is universal across all proto-brane variants and is not repeated
in per-variant tables.

### WOBRANING semantics

WOBRANING captures a specific NYE-stage block: **a search needs to
dig into a referenced brane that is not yet EMBRYONIC**. Without
that brane being EMBRYONIC, we cannot inspect its statements; the
search cannot proceed.

Concrete example. In:

```foolish
{
    a = { something = 5 };
    c = a.something;
}
```

When stepping `c`'s anchored search `a.something`:
1. The Search FIR resolves the anchor `a` against the local IB —
   succeeds, `a` is the brane defined on line 0.
2. The Search FIR needs to dig into `a` to find `something`. But `a`
   is still PREMBRYONIC (it has been parsed but its statement array
   has not been built).
3. The Search FIR transitions to **WOBRANING**, recording that it is
   blocked on `a`.

When `a` reaches EMBRYONIC (its statement array is built), the Search
FIR can be advanced: WOBRANING → BRANING → resolve `something` from
`a`'s statements.

**Per-FIR tracking.** Each FIR in WOBRANING knows what it is blocked
on. The implementation may use a single `blocking_on: Option<FirRef>`
field: when set, the FIR is WOBRANING and cannot make progress until
`blocking_on.state()` advances to EMBRYONIC or beyond.

**WOBRANING is not constanic.** A WOBRANING FIR is not yet known to
have any value or non-value — it's blocked on a structural prerequisite.
It is a NYE state, not a constanic state.

**Cloning a WOBRANING FIR.** Per FOOP-7, `constanic_clone` requires
constanic states only. WOBRANING is NYE, so cloning a WOBRANING FIR
is a precondition violation.

**A brane's collective state is never WOBRANING directly.** A brane
that has WOBRANING children is itself BRANING — its work is to advance
the WOBRANING children by stepping the branes they're blocked on.
Only individual searches (or anchored expressions) can be WOBRANING;
a brane doesn't "wait" — it steps its children, including those at
WOBRANING.

## Per-Variant Table

The table below specifies, for each (variant, state) pair, the
activities and transition conditions. Variants that are born terminal
(see "Born-terminal variants" below) are omitted.

Each row reads: in this state, the variant performs these activities;
on these conditions, transitions to these destination states.

---

### NormalBrane (`{stmt₁; stmt₂; ...; stmtₙ}`)

A NormalBrane contains a list of statements. Each statement is
`(name?, body)` where `name` is optional. The brane's job is to step
each statement's body to a constanic state, and then determine its own
state from the collective state of its statements.

**Note on iteration.** UBCb is brane-driven and steps one operation
per `step()` call. The NormalBrane tracks its current iteration cursor
(`self.cursor: usize`, the index of the statement currently being
stepped) across step() invocations. When `cursor` reaches the end of
the statement list, all statements have had at least one step.

| State | Activities | Transition condition → destination |
|-------|------------|----------------------------------|
| `PREMBRYONIC` | Code is present as String + AST; structural skeleton not yet built. To advance, the FVM: <br>1. Parses the AST node list and **builds an immutable array of StatementFirs**. Each StatementFir has `code` (subspan of parent's source string), `ast` (pointer to AST nodes), optional `CharacterizedName` (initially a string with string equality; later, a Characterization). <br>2. Sets the brane's `parent` pointer (None for root brane). <br>3. Assigns LUID if LUIDs are in use. <br>4. Initializes `self.cursor = 0` for child-stepping iteration. <br>The statement array is **immutable** thereafter — its size never changes and its members are never replaced. This array supports both regex name search and seek (#-N). | Always advances to EMBRYONIC. |
| `EMBRYONIC` | **Local-IB-only work.** Two phases, performed across multiple step() calls: <br>1. **Step children to EMBRYONIC.** Step the body at `self.cursor` once. If the body is now EMBRYONIC or beyond, advance cursor. Stay at EMBRYONIC until all children have reached EMBRYONIC. <br>2. **Resolve local-IB searches.** Once all children are EMBRYONIC, gather all searches in this brane's body (linear scan of the statement array — finite work). Sequentially try to resolve each: <br>&nbsp;&nbsp;&nbsp;- If the search resolves purely from local IB (e.g., `b=a` where `a` is defined earlier in this brane and is already constanic): set `search_result` and the search transitions appropriately. <br>&nbsp;&nbsp;&nbsp;- If the search is anchored on a local sibling brane that is itself still PREMBRYONIC (e.g., `c=a.something` where `a` is line 0): the search transitions to **WOBRANING**, blocked on `a`. <br>&nbsp;&nbsp;&nbsp;- If the search depends on cross-brane resolution (anchored on a brane outside this one, or unanchored requiring AB walk): leave it for BRANING; the search stays NYE (its own state remains EMBRYONIC for now). <br>&nbsp;&nbsp;&nbsp;- If the search is on a constanic local sibling that is fully resolved: complete it now. <br>By the end of EMBRYONIC, this brane is ready to receive search requests from other branes (its statement array is published). | Once (a) all children are EMBRYONIC and (b) all local-IB resolution work is complete (resolvable searches resolved; unresolvable ones in WOBRANING or deferred to BRANING): advance to BRANING. |
| `BRANING` | **Cross-brane work.** For each non-constanic child, step it once. Children typically advance via: <br>- Searches walking AB upward (resolving names from ancestral context). <br>- Searches digging into now-EMBRYONIC referenced branes. <br>- Operators computing values when all operands are constanic-and-CONSTANT. <br>- WOBRANING transitions back to BRANING when their `blocking_on` brane reaches EMBRYONIC. <br>The brane uses `self.cursor` to round-robin through children; if a child cannot advance this turn (e.g., it's WOBRANING and its blocker hasn't advanced), skip and try the next. <br><br>This is also the state a constanic-cloned brane lands in (per FOOP-7) when its source was WOCONSTANIC or NOTFOUNDIC. | When all children are constanic (CONSTANT/WOCONSTANIC/NOTFOUNDIC/NK): compute collective state per "Brane state computation" below and transition. Otherwise: stay at BRANING. |
| `WOCONSTANIC` | At least one child is WOCONSTANIC. On step, re-check children: any WOCONSTANIC child that has advanced lets the brane re-evaluate its collective state. | Re-run "Brane state computation" with current child states. May transition to CONSTANT, NOTFOUNDIC, NK, or stay at WOCONSTANIC. |
| `CONSTANT` | Per FOOP-51 detach step: clear `self.ab`, clear `self.parent`, advance state. | Always advances to INDEPENDENT. |
| `NOTFOUNDIC` | None when stepping in place. At least one child is NOTFOUNDIC and no child is WOCONSTANIC/NK; recoverable only via clone. Cloning the brane extends its AB; per FOOP-7's recursive descent, NOTFOUNDIC and WOCONSTANIC children of the clone get rewritten parent pointers and reset to BRANING. | (no in-place transition) |
| `INDEPENDENT` | None. Terminal. | None. |
| `NK` | None. Terminal. | None. |

#### Brane state computation (used at end of BRANING)

When `self.cursor == self.statements.len()` and all statements have
reached a constanic state, the brane's collective state is determined
by its statements' states. **Priority order** (highest priority first
— the first matching condition wins):

```
fn compute_brane_state(&self) -> Nyes:
    let states: Vec<Nyes> = self.statements.iter()
        .map(|s| s.body.state())
        .collect();

    // Priority 1: any NK child poisons the brane (irrecoverable)
    if states.iter().any(|s| *s == NK):
        return NK

    // Priority 2: any WOCONSTANIC child blocks the brane (might still
    // resolve in place, no clone needed)
    if states.iter().any(|s| *s == WOCONSTANIC):
        return WOCONSTANIC

    // Priority 3: any NOTFOUNDIC child makes the brane recoverable-via-clone
    // (in-place stepping cannot resolve, but cloning into richer context might)
    if states.iter().any(|s| *s == NOTFOUNDIC):
        return NOTFOUNDIC

    // Priority 4: all children CONSTANT/INDEPENDENT — brane is CONSTANT
    if states.iter().all(|s| matches!(s, CONSTANT | INDEPENDENT)):
        return CONSTANT

    panic!("brane in inconsistent state at end of BRANING")
```

**Why NK > WOCONSTANIC > NOTFOUNDIC > CONSTANT.** NK is irrecoverable,
so any NK child means the brane is irrecoverable too. WOCONSTANIC is
recoverable in-place (the child will eventually settle), so it
dominates NOTFOUNDIC (which requires a clone to recover). NOTFOUNDIC
dominates CONSTANT only when at least one child is NOTFOUNDIC; if all
children are constant, the brane is constant.

#### Constanic clone of a NormalBrane

Per FOOP-7, `constanic_clone` requires a constanic source. For
NormalBrane:

- **PREMBRYONIC, EMBRYONIC, BRANING:** caller-precondition violation
  per FOOP-7. (A NYE-state brane is mid-evaluation; cloning it is a
  bug.)
- **WOCONSTANIC:** clone permitted; clone's state reset to BRANING per
  FOOP-7. Statements are recursively cloned: WOCONSTANIC/NOTFOUNDIC
  statement bodies get rewritten parent pointers (pointing at the new
  brane clone) and their states reset to BRANING; CONSTANT/INDEPENDENT/NK
  statement bodies are shared by reference. The clone's `cursor` is
  copied (typically pointing past the end since the brane was already
  collected).
- **NOTFOUNDIC:** same as WOCONSTANIC. The recursive descent ensures
  NOTFOUNDIC descendants get their states reset to BRANING and their
  parent pointers rewritten, so on next step they re-walk their
  searches against the new parent chain visible through the clone.
- **CONSTANT, INDEPENDENT, NK:** shared by reference. The brane's
  value (collective state) is context-immune.

### Operator (`+ - * / %` and unary `-`)

An Operator has an `op` symbol and a list of operands (each an FIR).
Like a brane, operators step their operands one at a time using a
cursor. Once all operands are constanic, the operator computes its
value (or transitions to a constanic-pending state if any operand is
not yet fully CONSTANT).

**Starting condition.** Compiler emits operators at `EMBRYONIC`. The
operand list is fully populated; no statement-name binding is involved.

| State | Activities | Transition condition → destination |
|-------|------------|----------------------------------|
| `PREMBRYONIC` | Pass-through. Operators do not need runtime registration (no LUID, no AB-target use). Initialize `self.cursor = 0` if the compiler did not do so. | Always advances to EMBRYONIC. |
| `EMBRYONIC` | No work. Reserved slot for symmetry with brane lifecycle. | Always advances to BRANING. |
| `BRANING` | Step the operand at `self.cursor` once: <br>```let operand = &mut self.operands[self.cursor];```<br>```operand.step();```<br>If the operand is now in a constanic state, advance the cursor: `self.cursor += 1`. Otherwise leave the cursor for the next call. <br><br>If an operand transitions to WOBRANING (blocked on a PREMBRYONIC brane), the operator stays at BRANING and continues round-robin through other operands; the WOBRANING operand will be retried on subsequent step() calls and will advance once its blocker reaches EMBRYONIC. | If `self.cursor == self.operands.len()` (all operands constanic): determine collective state per "Operator state computation" below. Otherwise: stay at BRANING. |
| `WOCONSTANIC` | At least one operand is WOCONSTANIC; the operator cannot yet compute. On step, re-check operands: if any operand has advanced, recompute collective state. | Re-run "Operator state computation". May transition to CONSTANT, NOTFOUNDIC, NK, or stay at WOCONSTANIC. |
| `CONSTANT` | Per FOOP-51 detach step. The operator's value (an integer) is materialized; AB and parent are cleared. | Always advances to INDEPENDENT. |
| `NOTFOUNDIC` | None when stepping in place. At least one operand is NOTFOUNDIC and no operand is WOCONSTANIC/NK; recoverable only via clone (the clone re-walks NOTFOUNDIC operands per FOOP-7). | (no in-place transition) |
| `INDEPENDENT` | None. Terminal. | None. |
| `NK` | None. Terminal. The reason field carries diagnostics (e.g., `division-by-zero`). | None. |

#### Operator state computation (used at end of BRANING)

Same priority order as Brane state computation: NK > WOCONSTANIC >
NOTFOUNDIC > CONSTANT.

```
fn compute_operator_state(&self) -> Nyes:
    let states: Vec<Nyes> = self.operands.iter().map(|o| o.state()).collect();

    // Priority 1: NK poisons
    if states.iter().any(|s| *s == NK):
        return NK

    // Priority 2: WOCONSTANIC blocks
    if states.iter().any(|s| *s == WOCONSTANIC):
        return WOCONSTANIC

    // Priority 3: NOTFOUNDIC means recoverable-via-clone only
    if states.iter().any(|s| *s == NOTFOUNDIC):
        return NOTFOUNDIC

    // Priority 4: all CONSTANT/INDEPENDENT — compute the value
    if states.iter().all(|s| matches!(s, CONSTANT | INDEPENDENT)):
        // Special cases:
        //   division by zero → return NK with reason='division-by-zero'
        //   integer overflow → return NK with reason='overflow' (or wrap, per spec)
        match self.op:
            "+" | "-" | "*" | "/" | "%" => arithmetic
            "-" (unary)                 => negation
        // On success, materialize the result and return CONSTANT.
        return CONSTANT  // value is stored in self

    panic!("operator in inconsistent state at end of BRANING")
```

#### Constanic clone of an Operator

Same rule as NormalBrane:

- **PREMBRYONIC, EMBRYONIC, BRANING:** clone permitted; cursor is
  copied; operands reference-shared.
- **WOCONSTANIC, NOTFOUNDIC, CONSTANT, INDEPENDENT, NK:** clone
  permitted. CONSTANT operators carry their materialized value
  through the clone. NOTFOUNDIC operators have at least one
  NOTFOUNDIC operand; cloning the operator extends its AB and (per
  FOOP-7) resets the operator's state to EMBRYONIC, but the
  NOTFOUNDIC operands are reference-shared and remain NOTFOUNDIC.
  See "Container clone with NOTFOUNDIC children" open question.

---

### Concatenation (`A B C ...`)

A Concatenation has an ordered list of element FIRs and produces a
new merged brane. The merge happens after every element is constanic.
The merge produces a NormalBrane whose statements are the
constanic-cloned elements of each input brane, with later elements
overriding earlier ones (per FOOP-3).

**Starting condition.** Compiler emits at `EMBRYONIC`. Elements are
fully populated. The `merged` field is `None` initially; it is
populated when the merge fires.

| State | Activities | Transition condition → destination |
|-------|------------|----------------------------------|
| `PREMBRYONIC` | Pass-through. Initialize `self.cursor = 0`, `self.merged = None`. | Always advances to EMBRYONIC. |
| `EMBRYONIC` | No work. Reserved slot. | Always advances to BRANING. |
| `BRANING` | Step the element at `self.cursor` once: ```self.elements[self.cursor].step()```. If constanic, `self.cursor += 1`. | If `self.cursor < self.elements.len()`: stay at BRANING. <br>If `self.cursor == self.elements.len()` and all elements are CONSTANT or INDEPENDENT: build `self.merged` (merge per FOOP-3) and transition to BRANING-on-merged. See "Two-phase BRANING" below. <br>Else priority order on element states: NK → NK with reason `element-nk`; WOCONSTANIC → WOCONSTANIC; NOTFOUNDIC → NOTFOUNDIC. |
| `BRANING` (post-merge) | After `self.merged` is built, this state continues by stepping the merged brane: ```self.merged.step()```. | When `self.merged` reaches a constanic state, mirror it on the Concatenation: CONSTANT → CONSTANT, WOCONSTANIC → WOCONSTANIC, NOTFOUNDIC → NOTFOUNDIC, NK → NK. |
| `WOCONSTANIC` | The Concatenation is constanic-pending because either: (a) an element is WOCONSTANIC and the merge has not yet fired, or (b) the merged brane is WOCONSTANIC. On step, re-check the relevant subtree. | Same priority order: NK → NK; remaining WOCONSTANIC stays; NOTFOUNDIC promotes to NOTFOUNDIC; all-CONSTANT → CONSTANT. |
| `CONSTANT` | Per FOOP-51 detach. The materialized value is `self.merged` (now also CONSTANT). | Always advances to INDEPENDENT. |
| `NOTFOUNDIC` | None when stepping in place. Either an element or the merged brane is NOTFOUNDIC; recoverable only via clone. | (no in-place transition) |
| `INDEPENDENT` | None. Terminal. | None. |
| `NK` | None. Terminal. | None. |

#### Two-phase BRANING note

Concatenation has two BRANING phases: stepping the original elements
to constanic, then building the merge, then stepping the merge. The
table conflates them under "BRANING" with a sub-distinguisher
("post-merge"). Implementation may track a `phase: ConcatPhase` field
to disambiguate at runtime; the NYES state remains BRANING throughout.

#### Constanic clone of a Concatenation

- **PREMBRYONIC, EMBRYONIC, BRANING (pre-merge), BRANING (post-merge):**
  clone permitted; cursor and merged reference-shared.
- **WOCONSTANIC, NOTFOUNDIC, CONSTANT, INDEPENDENT, NK:** permitted.
  NOTFOUNDIC follows the same brane-reset semantics described under
  NormalBrane and Operator.

---

### Search (`name`, anchored or unanchored)

A Search has a `pattern` (regex), a `direction`, an `anchored: bool`,
an optional `anchor: FirRef` (for anchored searches), and a
`search_result: Option<FirRef>` (per FOOP-51, the cached resolution).
Searches do NOT have children to step. Their work is name resolution.

**Starting condition.** Compiler emits at `EMBRYONIC`,
`search_result = None`.

| State | Activities | Transition condition → destination |
|-------|------------|----------------------------------|
| `PREMBRYONIC` | Pass-through. Searches do not register. | Always advances to EMBRYONIC. |
| `EMBRYONIC` | Pass-through. The Search FIR itself has no pre-stepping setup. <br><br>**Note on driving.** The parent brane's EMBRYONIC phase gathers searches from its statement array and attempts **local-IB-only** resolution. If the search resolves locally, the brane sets this Search's `search_result` and state. If the search needs cross-brane resolution (AB walk, or digging into a child brane that's still PREMBRYONIC), the brane leaves the Search at EMBRYONIC for now (deferred to BRANING) or transitions it to WOBRANING (waiting on a child brane). | Always advances to BRANING (but may transition directly to WOBRANING or a constanic state if the parent brane has resolved or deferred this search during EMBRYONIC). |
| `BRANING` | Perform cross-brane resolution per FOOP-51: walk AB → live parent. (Local-IB resolution was already attempted during the parent brane's EMBRYONIC phase.) <br><br>For anchored searches: if the anchor is CONSTANT, perform the anchored walk into the anchor's brane. If the anchor is WOCONSTANIC, transition to WOCONSTANIC and wait. If the anchored target brane is still PREMBRYONIC, transition to WOBRANING. <br><br>This is also the state a constanic-cloned Search lands in when its source was WOCONSTANIC or NOTFOUNDIC (per FOOP-7); the clone re-walks against the new parent chain. | If a match is found, set `self.search_result = Some(constanic_clone(found))` per FOOP-7. The search's state then mirrors the resolved target: <br>- target is CONSTANT/INDEPENDENT → state = CONSTANT <br>- target is WOCONSTANIC → state = WOCONSTANIC <br>- target is itself a Search (chain) → state = WOCONSTANIC; short-circuit logic accumulates AB later <br>- target is NOTFOUNDIC → state = NOTFOUNDIC <br>- target is NK → state = NK with reason `target-nk` <br><br>If no match (cross-brane resolution exhausts AB and parent chain): **state = NOTFOUNDIC**. <br><br>If anchored on a PREMBRYONIC brane: WOBRANING with `blocking_on = that brane`. <br><br>If anchor expression is still NYE: stay at BRANING (re-step anchor next turn). |
| `WOBRANING` | Check `self.blocking_on.state()`. If it has reached EMBRYONIC or beyond, the blocker has advanced; clear `blocking_on` and return to BRANING to retry the search. Otherwise stay at WOBRANING. | If `blocking_on` is now EMBRYONIC or beyond: transition to BRANING. Otherwise: stay. |
| `WOCONSTANIC` | The search has resolved its `search_result` to a constanic-pending target. On step, re-read the target's state. <br><br>**Short-circuit on step:** if `search_result` is itself a SearchFir whose `search_result` is set, perform short-circuit accumulation per FOOP-51 (collapse the chain, accumulate AB, install final target). | If target is now CONSTANT or INDEPENDENT: transition to CONSTANT. <br>If target is now NOTFOUNDIC: transition to NOTFOUNDIC. <br>If target is now NK: transition to NK with reason `target-nk`. <br>Otherwise: stay at WOCONSTANIC. |
| `CONSTANT` | Per FOOP-51 detach. The search's value is read from `search_result`. | Always advances to INDEPENDENT. |
| `NOTFOUNDIC` | None when stepping in place. Terminal-by-stepping; recoverable only by being constanic-cloned and re-coordinated to a new parent. Per FOOP-7, the clone is reset to BRANING; the caller sets the new parent; the clone's BRANING work re-walks the cross-brane resolution against the new parent chain. | (no in-place transition) |
| `INDEPENDENT` | None. Terminal. | None. |
| `NK` | None. Terminal. Reason field distinguishes `target-nk` from other failure modes. | None. |

#### Constanic clone of a Search

Per FOOP-7, `constanic_clone` requires a constanic source. The
applicable cases for Search:

- **WOCONSTANIC:** clone permitted; per FOOP-7 the clone's state is
  reset to BRANING. The clone keeps its `search_result` pointer
  (FOOP-51 determinism invariant); on re-walk at BRANING the cached
  result is checked against the new context. Most often the cached
  result is still valid given the new parent chain.
- **NOTFOUNDIC:** clone permitted; the clone's state is reset to
  BRANING per FOOP-7. On re-walk at BRANING, the search consults the
  new parent chain and may now find what it previously couldn't.
- **CONSTANT, INDEPENDENT, NK:** shared by reference. The Search has
  resolved to a value (or terminal error); AB extension is a no-op.

Cloning a NYE-state Search (PREMBRYONIC, EMBRYONIC, BRANING) is a
caller-precondition violation per FOOP-7.

---

### IndexFir (`#N` positional access)

An IndexFir has an integer `offset`, an `anchored: bool`, and an
optional `anchor: FirRef`. It selects the Nth statement of a
referenced brane (the anchor for anchored, the parent brane for
unanchored). The result is a reference to the selected statement's
body.

**Starting condition.** Compiler emits at `EMBRYONIC`. No
`search_result` field; the result is computed and the IndexFir is
typically replaced or short-circuited to the resolved body.

| State | Activities | Transition condition → destination |
|-------|------------|----------------------------------|
| `PREMBRYONIC` | Pass-through. | Always advances to EMBRYONIC. |
| `EMBRYONIC` | Pass-through. | Always advances to BRANING. |
| `BRANING` | For anchored: step the anchor expression. If anchor is CONSTANT, look up `anchor.statements[offset]` and constanic-clone it. If the anchor's brane is still PREMBRYONIC, transition to WOBRANING blocked on that brane. <br><br>For unanchored: walk the parent chain to find the immediate parent brane, then look up `parent.statements[offset]`. <br><br>This is the state a constanic-cloned IndexFir lands in (per FOOP-7) when its source was WOCONSTANIC or NOTFOUNDIC. | If anchor/parent is constanic and offset in range: mirror resolved state (CONSTANT/WOCONSTANIC/NOTFOUNDIC/NK). <br>If offset is out of range: NK with reason `index-out-of-range` (irrecoverable). <br>If anchor is WOCONSTANIC: WOCONSTANIC, retry next step. <br>If anchor's target brane is PREMBRYONIC: WOBRANING blocked on it. <br>If no parent (root brane, unanchored): **NOTFOUNDIC**. Recoverable: cloning the root into a host gives the clone a parent. |
| `WOBRANING` | Check `self.blocking_on.state()`. If reached EMBRYONIC or beyond: clear `blocking_on`, return to BRANING. Otherwise stay. | If blocker now EMBRYONIC+: BRANING. Otherwise: stay. |
| `WOCONSTANIC` | Re-read anchor state. If anchor advanced, re-resolve. | Target → CONSTANT/INDEPENDENT: CONSTANT. <br>Target → NOTFOUNDIC: NOTFOUNDIC. <br>Target → NK: NK with reason `target-nk`. <br>Otherwise: stay. |
| `CONSTANT` | Per FOOP-51 detach. | Always advances to INDEPENDENT. |
| `NOTFOUNDIC` | None when stepping in place. Recoverable only via clone (per FOOP-7, clone resets to BRANING and re-walks). | (no in-place transition) |
| `INDEPENDENT` | None. Terminal. | None. |
| `NK` | None. Terminal. Reasons: `index-out-of-range` (irrecoverable), `target-nk`. | None. |

#### Constanic clone of an IndexFir

Same as Search: precondition is constanic state; WOCONSTANIC and
NOTFOUNDIC reset to BRANING; CONSTANT/INDEPENDENT/NK shared by
reference.

---

### HeadTail (`^` head, `$` tail)

A HeadTail has `is_head: bool` (true for `^`, false for `$`), an
`anchored: bool`, and optional `anchor: FirRef`. Equivalent to
`IndexFir` with offset 0 (head) or last (tail), but the offset is
computed dynamically based on the resolved brane's statement count.

**Starting condition.** Compiler emits at `EMBRYONIC`.

| State | Activities | Transition condition → destination |
|-------|------------|----------------------------------|
| `PREMBRYONIC` | Pass-through. | Always advances to EMBRYONIC. |
| `EMBRYONIC` | Pass-through. | Always advances to BRANING. |
| `BRANING` | Resolve the brane (anchor for anchored, parent for unanchored). Once the brane is EMBRYONIC, compute `offset = if is_head { 0 } else { brane.statements.len() - 1 }`; constanic-clone the indicated body. If the anchor's brane is still PREMBRYONIC, transition to WOBRANING blocked on that brane. <br><br>This is the state a constanic-cloned HeadTail lands in (per FOOP-7) when its source was WOCONSTANIC or NOTFOUNDIC. | Mirror resolved state (CONSTANT/WOCONSTANIC/NOTFOUNDIC/NK). <br>If brane is empty: NK with reason `head-of-empty` or `tail-of-empty` (irrecoverable). <br>If anchor is WOCONSTANIC: WOCONSTANIC, retry next step. <br>If anchor's brane is PREMBRYONIC: WOBRANING blocked on it. <br>If no parent (root brane, unanchored): NOTFOUNDIC. Recoverable via clone. |
| `WOBRANING` | Check `self.blocking_on.state()`. If EMBRYONIC+: clear, return to BRANING. Otherwise stay. | If blocker now EMBRYONIC+: BRANING. Otherwise: stay. |
| `WOCONSTANIC` | Re-read anchor/parent state. | Same as IndexFir WOCONSTANIC: target → CONSTANT, NOTFOUNDIC, or NK; otherwise stay. |
| `CONSTANT` | Per FOOP-51 detach. | Always advances to INDEPENDENT. |
| `NOTFOUNDIC` | None when stepping in place. Recoverable only via clone (per FOOP-7, clone resets to BRANING and re-walks). | (no in-place transition) |
| `INDEPENDENT` | None. Terminal. | None. |
| `NK` | None. Terminal. Reasons include `head-of-empty`, `tail-of-empty`, `target-nk`. | None. |

#### Constanic clone of a HeadTail

Same as Search.

---

### StayFoolish (`<expr>`)

A StayFoolish wraps an inner expression `expr`. The wrapper steps the
inner expression once per BRANING and propagates the inner state.
StayFoolish is a "soft" stay-foolish: the inner expression *can*
recoordinate when cloned (its AB is extended), in contrast to
StayFullyFoolish which is born INDEPENDENT.

**Starting condition.** Compiler emits at `EMBRYONIC`. The inner
`expr` is at whatever state the compiler placed it (typically
EMBRYONIC).

| State | Activities | Transition condition → destination |
|-------|------------|----------------------------------|
| `PREMBRYONIC` | Pass-through. | Always advances to EMBRYONIC. |
| `EMBRYONIC` | No work. | Always advances to BRANING. |
| `BRANING` | Step the inner expression once: ```self.expr.step()```. | If `self.expr` is now CONSTANT or INDEPENDENT: transition to CONSTANT. <br>If `self.expr` is WOCONSTANIC: transition to WOCONSTANIC. <br>If `self.expr` is NOTFOUNDIC: transition to NOTFOUNDIC. <br>If `self.expr` is NK: transition to NK with reason `inner-nk`. <br>Otherwise (still NYE): stay at BRANING. |
| `WOCONSTANIC` | Re-step the inner expression's state. | If inner advances to CONSTANT/INDEPENDENT: CONSTANT. <br>If inner is NOTFOUNDIC: NOTFOUNDIC. <br>If NK: NK. <br>Otherwise: stay. |
| `CONSTANT` | Per FOOP-51 detach. | Always advances to INDEPENDENT. |
| `NOTFOUNDIC` | None when stepping in place. Inner expr is NOTFOUNDIC; recoverable only via clone. | (no in-place transition) |
| `INDEPENDENT` | None. Terminal. | None. |
| `NK` | None. Terminal. | None. |

#### Constanic clone of a StayFoolish

Same rule as NormalBrane:

- **PREMBRYONIC, EMBRYONIC, BRANING:** permitted; inner expression
  reference-shared.
- **WOCONSTANIC, CONSTANT, INDEPENDENT, NK:** permitted.

---

## Born-Terminal Variants

The following variants are born at a terminal state and have no NYES
transitions. They are omitted from the table.

- **`ConstantInt(n)`**: born at `INDEPENDENT`. A literal integer is
  context-free; it has no AB to drop, no parent to clear. Constanic
  clone returns the same FIR by reference.
- **`StayFullyFoolish(<<expr>>)`**: born at `INDEPENDENT`. The
  double-bracket form is constructed already-detached. Constanic clone
  recurses into the inner expression (per FOOP-7's existing behavior)
  but the outer wrapper does not retain AB.
- **`NkFir(reason)`**: born at `NK`. Constanic clone returns the same
  FIR by reference. Reasons include `search-exhausted`,
  `division-by-zero`, `infinite-recursion`, `invariant-violated`.

## Open Questions

- **LUID assignment timing for non-Brane variants.** This FOOP says
  branes get LUIDs at PREMBRYONIC. Non-brane proto-branes (Operator,
  Concatenation, etc.) likely don't need LUIDs because UBCb is
  brane-driven (no message routing) and only branes appear in AB.
  To confirm during implementation.
- **Cursor model for non-Brane containers.** NormalBrane, Operator,
  and Concatenation all use a cursor. Likely uniform across these
  three; specified consistently in their respective rows.
- **Cross-validation against UBC.** Approval test snapshots will
  differ in the state field between UBC and UBCb (UBC uses
  ECONSTANIC and the original WOCONSTANIC mechanics; UBCb uses
  NOTFOUNDIC and the new constanic-clone semantics). Cross-validation
  must compare *values*, not state fields. To
  specify in test plan during implementation.
- **Embryonic-dependency table semantics.** The "Embryonic dependency
  tracking" mechanism is sketched but not fully specified. Open
  questions: when does the brane scan all children for unblocked
  dependencies? How does it detect that a previously-blocked
  dependency has become unblocked without polling every step? Likely
  implementation: scan once per step, accept O(n) cost per step where
  n is the number of blocked dependencies (typically small).

## Test Plan

- **Per-variant unit tests.** For each variant and each state, a unit
  test constructs a FIR in that state and exercises the activities and
  transitions specified in the table. Asserts the destination state is
  correct.
- **Constanic-clone permission tests.** For each (variant, state)
  pair, a test attempts constanic clone and asserts permission rule
  (permitted or forbidden per the table).
- **Brane state computation tests.** Construct branes with known
  child-state configurations; assert collective-state computation
  matches the table.
- **Approval test parity.** UBCb produces the same VALUE on all
  approval tests as UBC, even though state fields may differ. Cross-
  validation excludes the state field.

## Rejected Alternatives

### A. Inherit UBC's Nyes wholesale

Reuse UBC's enum unchanged. **Rejected:** UBC's NOTFOUNDIC and
WOCONSTANIC have specific meanings tied to UBC's depth-first re-step
mechanics. Reusing them imports semantics that don't fit UBCb's
brane-driven model. UBCb's `WOCONSTANIC` is similarly named but
defined here on its own terms.

### B. Message-passing UBCb (the FOOP-41 model)

Specify states as message-driven: `BLOCKED_ON_MESSAGE`, `READY`,
listener tables, etc. **Rejected:** UBCb in foolish-mvp is brane-
driven. The message-passing model is documented in FOOP-41 as a
future variant, not the current one. Importing those concepts into
UBCb's state machine creates confusion and complexity not justified
by current implementation needs.

### C. Per-variant lifecycle (no universal state machine)

Each variant defines its own state set; no shared NYES enum.
**Rejected:** the OOP model — proto-brane as supertype with shared
state semantics — is more useful for cross-variant reasoning (e.g.,
"is this clone-safe?") and for tooling (debuggers, serialization).

## References

- FOOP-7 (revised): Constanic Clone — consumes the per-state
  permission rules defined here.
- FOOP-51: AB list, search_result, short-circuit accumulation. The
  CONSTANT → INDEPENDENT detach step is universal across UBCb
  variants and is not repeated in per-variant rows.
- FOOP-41: UBCb message-passing variant (separate development track).
  The message-passing concepts are intentionally NOT used here.
- FOOP-6: Phase 2 evaluator is depth-first sequential. UBCb's brane-
  driven model is compatible with FOOP-6's depth-first contract.
- Code: `foolish/foolish-core/src/fir.rs` — current Nyes enum and FIR
  variant definitions.
- Code: `foolish/foolish-core/src/ubc.rs` — current step rules,
  `re_step_brane_bodies`, `compute_brane_state`.

## Last Updated

**Date**: 2026-05-09 (latest revision)
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.7 xHigh effort
**Changes**: Major restructuring of EMBRYONIC/BRANING semantics and
clone semantics:
- Renamed ECONSTANIC → NOTFOUNDIC ("Not Found In Context"). Kept
  WOCONSTANIC's name (preserving "constanic" as the categorical
  English adjective for the family of constanic states).
- Restored WOBRANING as a NYES state with new specific meaning:
  "search waiting on a referenced child brane to become EMBRYONIC."
- Redefined EMBRYONIC vs BRANING semantics: EMBRYONIC = local-IB-only
  work (build StatementFir array, step children to EMBRYONIC, resolve
  searches that don't cross brane boundaries). BRANING = cross-brane
  work (AB walk, dig into child branes, operator computation).
- Added stage-completion invariant: EMBRYONIC must complete before
  BRANING; BRANING before any constanic state.
- Added recursive constanic clone (per FOOP-7): when a container is
  cloned, recurse into WOCONSTANIC/NOTFOUNDIC descendants, rewriting
  parent pointers to the clone and resetting their state to BRANING.
  CONSTANT/INDEPENDENT/NK descendants shared by reference.
- Constanic clone reset destination is BRANING (not EMBRYONIC):
  EMBRYONIC's local-IB work is preserved by the recursive clone;
  only BRANING's cross-brane work needs redoing against new parent chain.
- Constanic clone precondition: source must be in a constanic state.
  NYE-state sources are caller bugs.
- Updated all per-variant rows accordingly.

**Date**: 2026-05-09 (earlier)
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.7 xHigh effort
**Changes**: Initial draft. Sets out the UBCb state set, the universal
proto-brane contract, and the per-variant table for all proto-brane
variants: NormalBrane, Operator, Concatenation, Search, IndexFir,
HeadTail, StayFoolish. Born-terminal variants (ConstantInt,
StayFullyFoolish, NkFir) noted in prose.
