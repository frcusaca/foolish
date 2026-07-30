---
foop: D54
title: Deadbrane — useless-element detection and FirID cloning semantics
author: Hephaestus <agent@ohmyencode.com>
status: Draft
type: Standards
created: 2026-07-14
phase: phase-2
supersedes: []
begun: [ ]
---

# FOOP-45: Deadbrane — useless-element detection and FirID cloning semantics

> **Scope reduction (Atlas, 2026-07-14):** the **FirID cloning semantics component is removed
> from this FOOP** — deferred to a later implementation pass. FOOP-74 (FIRID) is scheduled by
> itself on the roadmap; Deadbrane's useless-element detection proceeds **without any FIRID
> dependency**. Sections below that pin FirID assignment rules are not to be executed under
> this FOOP.

FOOP numbering is little-endian; the full rules live in `foop.md` at the
repository root — **read it before creating or editing a FOOP.**

## Abstract

This FOOP introduces three small, complementary features:

1. **Deadbrane** — a static analysis method that identifies "useless" statements.
   Uselessness is **frame-of-reference dependent**: a statement may be useless
   from one brane's perspective but useful from a parent brane's perspective.
   The frame of reference for uselessness is the pair **(brane, FI)** — a brane
   and a FoolishIndex pointing to a position within it. The core query is
   `brane.is_useless(fi, query_fir)`: starting from the statement addressed by
   `fi` in this brane, is `query_fir` referenced by any statement at that
   position onward, or by any descendant brane? Transitive uselessness
   propagates: if every statement that references `query_fir` is itself useless,
   then `query_fir` is transitively useless.

2. **FoolishIndex (FI)** — a sequence of signed integers `(i₀, i₁, …, iₙ)` that
   precisely addresses a descendant FIR within a brane tree, without storing a
   reference to the FIR itself. **FI has frame of reference = brane**: an FI is
   always interpreted relative to a brane (the root from which navigation
   begins). Positive indices count forward from the head; negative indices count
   backward from the tail (matching the `#` operator semantics). FI is purely
   numerical — no searches, no names, no patterns.

3. **FirID cloning semantics** — pins and tests the FirID assignment rules for
   constanic cloning. Building on FOOP-74 (which introduced the atomic
   per-instance FIRID counter for cycle detection), this FOOP specifies:
   - `Constant` and `Independent` FIRs share identity on clone (`Rc::clone`,
     no new FIRID) — they are immutable values, not distinct instances.
   - Non-constanic FIRs that undergo constanic cloning **do** get a new FIRID —
     they are fresh instances distinguishable by identity.
   - Brane FIRs always get a new FIRID on clone (mutable topology).
   - The existing `constanic_clone_at` code already implements this; this FOOP
     pins the behavior as a specification invariant and adds unit tests.

## Motivation

### Deadbrane

Foolish programs can accumulate statements that are defined but never
referenced — dead code. In conventional languages, dead code is a readability
and maintenance burden. In Foolish, where branes are the fundamental unit of
organization and search is the primary access mechanism, a statement whose name
is never found by any search is truly invisible to the rest of the program.

Crucially, **uselessness is frame-of-reference dependent**. The frame of
reference is the pair **(brane, FI)** — a brane and a FoolishIndex pointing to
a position within it. Consider:

```foolish
{
    inner = {
        a = 1;         !! useless from (inner, (0)) — nothing after uses it
        b = a + 1;     !! references a — so (inner, (0)) is NOT useless for a
    };
    c = inner.a;       !! 'a' is useful from (outer, (0)) — c references it
}
```

Statement `a` at FI `(0)` in `inner` is **not** useless from `(inner, (0))`
because `b` at FI `(1)` references it. But if `b` didn't reference `a`, then
`a` would be useless from `inner`'s frame. Meanwhile, `a` is useful from the
outer brane's frame because `c` references `inner.a`.

The core query: `brane.is_useless(fi, query_fir)` — "from this brane, starting
at the statement addressed by `fi`, is `query_fir` ever used by any statement
at that position onward, or by any descendant brane?"

Identifying useless statements helps Foolishers:
- Clean up their branes by removing unused definitions.
- Understand the dependency graph of their program.
- Catch accidental typos in name references — a name that was *intended* to
  be referenced but never actually is.

Transitive uselessness extends this: if `a` is useless from (brane, fi), and
every statement that references `a` is itself useless, then `a` is transitively
useless — its entire dependency chain contributes nothing to the brane's
observable output.

### FoolishIndex

Deadbrane needs to *address* where useless statements live. A FoolishIndex is a
stable, serializable address into a brane tree — a sequence of signed integers
that navigates from a root brane to any descendant FIR without holding a live
`FirRef` pointer. This is useful beyond Deadbrane: any tool that needs to
reference a specific position in a brane tree (debugging, IDE integration,
error reporting) benefits from a lightweight, copyable address.

**FI has frame of reference = brane.** An FI is always interpreted relative to a
brane — the root from which navigation begins. The same FI `(2, 0)` addresses
different FIRs depending on which brane you start from.

FoolishIndex is **purely numerical** — no search operators, no name patterns,
no value matching. Each element is a raw positional index into a brane's
statement list. Positive indices count forward from the head; negative indices
count backward from the tail (matching the `#` operator's indexing semantics).
`(0)` is the first statement, `(-1)` is the last.

### FirID cloning semantics

FOOP-74 introduced FIRID as a cycle-detection tool. The existing implementation
already follows the correct cloning semantics (Constant/Independent →
`Rc::clone`, non-constanic → new instance), but this behavior is not pinned by
any specification or test. This FOOP makes the invariant explicit and adds
targeted unit tests so that future refactors cannot accidentally break the
identity-sharing rule without a test failure.

## Specification

### FoolishIndex

#### Definition

A **FoolishIndex** is a non-empty sequence of signed 32-bit integers
`(i₀, i₁, …, iₙ)` that addresses a descendant FIR within a brane tree.
**FoolishIndex is purely numerical** — no search operators, no name patterns,
no value matching, no regex. Each element is a raw positional index into a
brane's statement list. This makes it trivially serializable, comparable, and
free of any evaluation-time semantics.

**Frame of reference = brane.** An FI is always interpreted relative to a
brane — the root from which navigation begins. The same FI `(2, 0)` addresses
different FIRs depending on which brane you start from.

```rust
/// A stable, serializable address into a brane tree.
/// Purely numerical — each element is a positional index (positive from head,
/// negative from tail). No searches, no names, no patterns.
pub struct FoolishIndex(pub Vec<i32>);
```

#### Foolish syntax

In Foolish source, a FoolishIndex is written as the argument to the `#`
operator. This is a **single `#` expression** with a multi-element index —
not search chaining. `#1,2,-1` is one operator applied to one FI `[1, 2, -1]`.

**Syntax:** `#i₀,i₁,i₂,…` — comma-separated signed integers, **no spaces**
after `#` or around commas.

The existing single-index `#N` is the degenerate one-element case.

**Parsing rules:**

The parser sees `#` and begins an FI expression. **No space** is permitted
between `#` and the first digit (or `-`). After parsing the first integer:

1. **`,` immediately** (no whitespace before the comma): parse the next
   integer. Repeat — `,` continues the FI.
2. **Anything else**: the FI is complete.

The entire FI is stored in a **single `#` indexing FIR** — not multiple
chained FIRs. `a#1,2,-1` produces one `IndexFir` whose index is `[1, 2, -1]`.

**Equivalence: `#1#2#3` ≡ `#1,2,3`.** Chained `#` operations are syntactic
sugar for comma-separated FI. `a#1#2#3` and `a#1,2,3` produce the same
single `IndexFir` with FI `[1, 2, 3]`. The parser desugars chained `#` into
a comma-separated FI internally.

**Contexted multi-element FI not permitted (for now):** `&#1,2,3` is a
syntax error. The contexted `&#` operator accepts only a single integer:
`&#N`. Multi-element contexted FI (navigating from a found statement through
nested branes) is deferred to a future FOOP.

**Literal integers only.** The parser rejects non-integer FI elements.
`#-1,(a+1),(b-1)` is a syntax error — computed indices must be evaluated as
separate searches before composing the path.

**Examples:**

```foolish
!! Comma-separated FI:
x = a#1,2,-1;      !! one IndexFir, FI = [1, 2, -1]

!! Equivalent chained form (desugared to same IndexFir):
x = a#1#2#-1;      !! same as a#1,2,-1

!! Single-element (existing syntax):
y = a#0;           !! one IndexFir, FI = [0]
z = a#-1;          !! one IndexFir, FI = [-1]

!! Contexted single-element (existing syntax):
w = a?b&#1;        !! find b, then #1 from b's position in its home brane
```

**Not search chaining:**

`a#1,2,-1` and `a#1#2#-1` are equivalent — both produce a single `IndexFir`
with FI `[1, 2, -1]`. The chained `#` form is syntactic sugar. This is
distinct from *separate* search operations that happen to use `#` — the
parser desugars consecutive `#`-prefixed integers (with no intervening
non-whitespace tokens) into one FI.

#### Formal indexing semantics

The `#` operator has two forms — contextless and contexted — with formally
specified modular arithmetic semantics.

**Contextless: `brane#number`**

Anchored on the brane. The brane is resolved from the left operand, and the
index selects a statement within it.

```
result = brane.children[ number mod brane.size ]
```

Where `mod` is mathematical modular arithmetic (always non-negative):
- `number mod n` maps any integer into `[0, n)` by wrapping.
- `brane.size` is `brane.foolish_children().len()`.
- 0-based from the front: `#0` is the first statement, `#1` is the second.
- Negative indices wrap from the tail: `#-1` is the last statement,
  `#-2` is second-to-last (equivalent to `#(size-1)` and `#(size-2)`).

**Contexted: `reference&#number`**

Anchored on a reference, which carries (brane, statement_index) from a
prior search result. The index is relative to that statement's position
within its home brane.

```
result = brane.children[ (statement_index + number) mod brane_size ]
```

Where:
- `statement_index` is the position of the referenced statement within
  its home brane (the brane reached by walking the statement's `.parent`
  chain — see "home brane" in AGENTS.md).
- `number` is the FI offset.
- `mod` is mathematical modular arithmetic (always non-negative).
- `brane_size` is the home brane's statement count.

The contexted form reads *"from where that statement landed, move `number`
positions forward (or backward) within its home brane, wrapping around."*

**Modular arithmetic note:**

Rust's `%` operator is *remainder*, not modulus — it preserves the sign of
the dividend. The implementation must use true modular arithmetic:
`(a % n + n) % n` or equivalent, to ensure the result is always in `[0, n)`.
This applies to both contextless and contexted forms.

#### Navigation rules

Starting from a root brane `B₀`:

1. **`i₀`** selects a statement in `B₀`:
   - `i₀ ≥ 0`: the statement at position `i₀` from the head (0-indexed).
   - `i₀ < 0`: the statement at position `len(B₀) + i₀` from the tail
     (so `-1` is the last statement, `-2` is second-to-last).

2. **`i₁`** navigates into the *body* of the statement selected by `i₀`:
   - If the body is a brane, `i₁` selects a statement within that brane
     (same positive/negative rules).
   - If the body is not a brane, the index is invalid (see error handling).

3. **`i₂ … iₙ`** continue descending: each level enters the body of the
   previously selected statement, which must be a brane.

4. **The final element `iₙ`** addresses the FIR at that position in the
   innermost brane. If `n = 0`, the FoolishIndex addresses a statement in
   the root brane itself.

#### Examples

```
(0)          → first statement in the root brane
(-1)         → last statement in the root brane
(2, 0)       → first statement in the body of the 3rd statement
               (the 3rd statement's body must be a brane)
(1, 0, 0, 10, -1) → navigate into: 2nd statement → its body (brane)
               → 1st statement → its body (brane) → 11th statement
               → its body (brane) → last statement
```

#### Error handling

A FoolishIndex is **invalid** if:
- Any index is out of bounds for the brane at that level.
- Any intermediate index selects a statement whose body is not a brane
  (but a deeper index is provided).
- The sequence is empty.

Invalid indices produce `None` on resolution (no panic).

#### Resolution

```rust
impl FoolishIndex {
    /// Resolve this index against a root brane. Returns the addressed
    /// FirRef, or None if the index is invalid.
    pub fn resolve(&self, root: &FirRef) -> Option<FirRef>;
}
```

#### Serialization

FoolishIndex is serializable to/from a compact string form: `(1,0,0,10,-1)`.
This makes it suitable for error messages, debugging output, and tool
integration.

### Deadbrane analysis

#### Frame of reference

Uselessness has frame of reference **(brane, FI)**. Every uselessness
determination is anchored to a specific brane and a FoolishIndex pointing to a
position within it. The same statement may be useless from one (brane, FI) pair
and useful from another.

#### Definitions

Given a brane `B` with statements `[s₀, s₁, …, sₙ]` (each `sᵢ` has a name,
a line number, and a body):

- **`B.is_useless(fi, query_fir)`**: returns `true` if `query_fir` is not
  referenced by any statement `sⱼ` in `B` at position `j ≥ fi[0]` (where
  `fi` is a FoolishIndex whose first element addresses a position in `B`),
  AND not referenced by any descendant brane reachable from those statements.
  "Referenced" means: the statement's body (or any sub-expression within it)
  contains a search or expression that resolves to `query_fir` (by FirRef
  identity / FirID).

- **Directly useless at (B, fi)**: a named statement `s` at position `fi` in
  `B` where `B.is_useless(fi, s.body)`. That is, starting from `s`'s own
  position, nothing in `B` (or descendants) uses `s`'s body.

- **Transitively useless at (B, fi)**: a statement `s` at position `fi` where
  every statement that references `s.body` is itself useless (directly or
  transitively). Computed by iteratively marking statements until a fixed
  point.

- **Reachable**: not useless (neither directly nor transitively) from the
  given (brane, FI).

#### Why FirRef identity, not name matching

The original Deadbrane draft used name-based matching (does any search target
match `s.name`?). The refined version uses **FirRef identity** (does any
expression resolve to the same `FirRef` as `s.body`?) because:

1. A name can be shadowed — `a = 1; a = 2;` has two statements named `a`.
   Name-based matching cannot distinguish which `a` is being referenced.
2. FirRef identity is precise: each FIR instance is unique (especially with
   FirID), so `B.is_useless(fi, s.body)` asks exactly "is *this specific
   value* ever used?"
3. The FoolishIndex provides a stable address when FirRef identity is too
   ephemeral (e.g., across serialization boundaries).

#### Algorithm

```
1. Given brane B, FoolishIndex fi (addressing a position in B), and
   query_fir Q:

2. Let start = fi[0] (the positional index into B's statements).
   Collect candidate statements: all statements sⱼ in B where j ≥ start.

3. For each candidate sⱼ:
   a. Walk sⱼ's body (the FIR subtree). For each search expression,
      check if the resolved FirRef matches Q (by pointer identity
      or FirID). If match found → Q is NOT useless; return false.
   b. If sⱼ's body is a brane, recurse: B_child.is_useless((0,), Q).
      If the recursive call returns false → Q is NOT useless.

4. If no candidate references Q → Q is useless from (B, fi).
   Return true.

5. Transitive uselessness (fixed-point iteration):
   a. For each named statement s at position p in B where p ≥ start:
      check B.is_useless((p,), s.body). If true, mark s as directly
      useless.
   b. For each remaining statement s, check if every statement that
      references s.body is already marked useless. If so, mark s as
      transitively useless.
   c. Repeat (b) until no new statements are marked.
```

#### Brane-scoped API

```rust
impl BraneFir {
    /// Is query_fir unused by any statement from position fi[0] onward in
    /// this brane (and all descendant branes)?
    /// Frame of reference: (this brane, fi).
    pub fn is_useless(&self, self_ref: &FirRef, fi: &FoolishIndex, query_fir: &FirRef) -> bool;

    /// Compute the full uselessness report for this brane, starting from
    /// position 0. Returns partitioned sets of FoolishIndex addresses.
    pub fn deadbrane_report(&self, self_ref: &FirRef) -> DeadbraneReport;
}

/// The output of a Deadbrane analysis.
/// Each entry is addressed by FoolishIndex (frame of reference = the brane
/// being analyzed).
pub struct DeadbraneReport {
    pub reachable: Vec<FoolishIndex>,
    pub directly_useless: Vec<(FoolishIndex, String)>,       // (fi, name)
    pub transitively_useless: Vec<(FoolishIndex, String, Vec<String>)>, // (fi, name, chain)
}
```

#### Output format

Each entry in the Deadbrane report is addressed by FoolishIndex (frame of
reference = the brane being analyzed):

```
DEADBRANE REPORT (from brane at ()):
  reachable:            N statements
  directly_useless:     N statements
    - name1 @ (2)       !! FI address in this brane
    - name2 @ (1, 0, 3) !! nested FI address
    ...
  transitively_useless: N statements
    - name3 @ (0), transitively via: name1 → name2
    ...
```

### FirID cloning semantics

The following rules are **pinned invariants** (not implementation accidents):

1. **Every new FIR instance** (constructed via `ProtoBrane::new` or any
   builder) receives a unique FIRID from the global atomic counter
   (`fetch_add(1, Relaxed)`). This includes parse-time FIRs and compute-time
   FIRs created during evaluation.

2. **Constanic clone of a `Constant` or `Independent` FIR** (non-Brane):
   uses `Rc::clone(fir_ref)` — the clone IS the original, sharing the same
   FIRID. No new FIRID is allocated. Correct because Constant and Independent
   FIRs are immutable values; identity-sharing is safe and desirable.

3. **Constanic clone of a non-constanic FIR** (pre-constanic, NK, ECONSTANIC,
   WOCONSTANIC): creates a new FIR instance via the kind-specific constructor
   in `constanic_clone_at`. The new instance receives a new FIRID. Correct
   because the clone is a distinct instance that may diverge (e.g.,
   recoordinated in a new AB/IB context).

4. **Constanic clone of a `Brane` FIR** (even if Constant/Independent):
   always creates a new FIR instance (the `kind() != FirKind::Brane` guard
   ensures this). The new instance receives a new FIRID. Branes are
   containers with mutable topology; sharing identity would be unsound.

5. **`FoolRefFir`** (born CONSTANT, immutable reference to an original
   statement): receives a new FIRID at construction, as it is a distinct FIR
   instance wrapping a strong reference to the original.

## FIR Impact

**Deadbrane**: None. The analysis is a read-only pass over the FIR tree; it
does not modify any FIR state, add new variants, or change stepping behavior.

**FirID cloning semantics**: None new. FOOP-74 already added `firid: u64` to
`ProtoBrane`. This FOOP only pins the assignment rules and adds tests.

## UBC Step Impact

None. Neither feature changes evaluation semantics or stepping behavior.

## Test Plan

### FoolishIndex tests

- **Unit: `fi_resolve_basic`** — `(0)` resolves to first statement, `(-1)`
  resolves to last statement in a brane.
- **Unit: `fi_resolve_nested`** — `(1, 0)` resolves into the body of the 2nd
  statement (which must be a brane).
- **Unit: `fi_resolve_out_of_bounds`** — returns `None`.
- **Unit: `fi_resolve_non_brane_intermediate`** — returns `None` when an
  intermediate body is not a brane but a deeper index is provided.
- **Unit: `fi_serialization_roundtrip`** — `(1,0,0,10,-1)` → parse →
  to_string → same string.
- **Unit: `fi_parent`** — `(1, 0, 3).parent()` yields `(1, 0)`.

### Indexing modular arithmetic tests

Tests for the formal `#` semantics specified in §Formal indexing semantics.

**Contextless (`brane#number`) — modular wrapping:**

- **Unit: `index_contextless_positive_in_bounds`** — `{a=10; b=20; c=30;}#1`
  resolves to `b` (index 1 of size 3).
- **Unit: `index_contextless_positive_wraps`** — `{a;b;c;}#5` resolves to `c`
  (5 mod 3 = 2). Also test `#3` → `a` (3 mod 3 = 0).
- **Unit: `index_contextless_negative_wraps`** — `{a;b;c;}#-1` resolves to `c`
  (-1 mod 3 = 2). `#-4` resolves to `c` (-4 mod 3 = 2). `#-3` resolves to
  `a` (-3 mod 3 = 0).
- **Unit: `index_contextless_zero`** — `{a;b;c;}#0` resolves to `a`.
- **Unit: `index_contextless_single_element`** — `{a;}#0` and `{a;}#1` and
  `{a;}#-1` all resolve to `a` (size 1, any index mod 1 = 0).
- **Unit: `index_contextless_large_number`** — `{a;b;c;}#100` resolves to `b`
  (100 mod 3 = 1). `{a;b;c;}#-100` resolves to `c` (-100 mod 3 = 2).

**Contexted (`reference&#number`) — relative modular wrapping:**

- **Unit: `index_contexted_forward_in_bounds`** — in `{a;b;c;d;}`, search
  finds `b` at index 1, then `&#1` resolves to `c` ((1+1) mod 4 = 2).
- **Unit: `index_contexted_forward_wraps`** — in `{a;b;c;}`, search finds `c`
  at index 2, then `&#1` resolves to `a` ((2+1) mod 3 = 0).
- **Unit: `index_contexted_backward_in_bounds`** — in `{a;b;c;d;}`, search
  finds `c` at index 2, then `&#-1` resolves to `b` ((2-1) mod 4 = 1).
- **Unit: `index_contexted_backward_wraps`** — in `{a;b;c;}`, search finds `a`
  at index 0, then `&#-1` resolves to `c` ((0-1) mod 3 = 2).
- **Unit: `index_contexted_zero`** — in `{a;b;c;}`, search finds `b` at
  index 1, then `&#0` resolves to `b` ((1+0) mod 3 = 1).
- **Unit: `index_contexted_large_offset`** — in `{a;b;c;}`, search finds `a`
  at index 0, then `&#100` resolves to `b` ((0+100) mod 3 = 1).
- **Unit: `index_contexted_negative_large`** — in `{a;b;c;}`, search finds `b`
  at index 1, then `&#-100` resolves to `c` ((1-100) mod 3 = 2).

### Deadbrane tests

- **Unit: `deadbrane_directly_useless`** — compile `{a=1; b=2;}` where `a` is
  never referenced from (brane, (0)); assert `a` in directly-useless set at
  FI `(0)`.
- **Unit: `deadbrane_frame_of_reference`** — nested brane where `a` is useless
  from (inner, (0)) but useful from (outer, (0)); assert inner report says
  useless, outer report says reachable.
- **Unit: `deadbrane_transitively_useless`** — compile `{a=1; b=a+1;}` where
  `b` is never referenced; assert both `a` and `b` are useless.
- **Unit: `deadbrane_reachable`** — compile `{a=1; b=a+1; c=b+1;}` where `c`
  is used externally; assert all reachable.
- **Unit: `deadbrane_anonymous_excluded`** — anonymous statements (`??? = expr`)
  are not candidates for uselessness analysis.
- **Unit: `deadbrane_empty_brane`** — empty brane produces empty report.
- **Unit: `deadbrane_cycle`** — two statements referencing each other; if
  neither is referenced by anything outside the cycle, both are transitively
  useless.

### FirID cloning tests

- **Unit: `firid_constant_clone_shares_identity`** — constanic-clone a
  `Constant` FIR; assert the clone's FIRID equals the original's.
- **Unit: `firid_independent_clone_shares_identity`** — constanic-clone an
  `Independent` FIR; assert the clone's FIRID equals the original's.
- **Unit: `firid_nonconstanic_clone_gets_new_id`** — constanic-clone a
  pre-constanic FIR (e.g., an Operator in BRANING state); assert the clone's
  FIRID differs from the original's.
- **Unit: `firid_brane_clone_gets_new_id`** — constanic-clone a Brane FIR
  (even if Constant); assert the clone's FIRID differs.
- **Unit: `firid_nk_clone_gets_new_id`** — constanic-clone an NK FIR; assert
  the clone's FIRID differs.
- **Unit: `firid_econstanic_clone_gets_new_id`** — constanic-clone an
  ECONSTANIC FIR; assert the clone's FIRID differs.

### Approval tests

- **Approval: `deadbrane_report.foo`** — a Foolish program with a mix of
  reachable, directly-useless, and transitively-useless statements. The
  approval test output includes the Deadbrane report alongside the normal
  evaluation result.

## Rejected Alternatives

### A. Deadbrane as a runtime alarm (like FOOP-74's cycle alarm)

Run useless-element detection during evaluation and `eprintln!` when a
statement settles without ever being referenced. **Rejected**: uselessness is
a property of the *program text*, not of evaluation dynamics. A statement can
be useless even if the program never evaluates it. Static analysis is correct.

### B. Deadbrane as a compiler warning

Emit compiler warnings for useless statements during compilation. **Rejected**:
warnings during compilation would be noisy for interactive use (REPL, step
mode). A separate analysis pass invoked explicitly by the Foolisher is more
ergonomic. A future `--warn-deadbrane` flag could be added later.

### C. Deadbrane removes useless statements automatically

Instead of reporting, automatically strip useless statements from the FIR tree.
**Rejected**: automatic mutation is risky — a statement might appear useless
due to a typo in a reference. Removing it hides the bug. Reporting is safer;
the Foolisher decides what to do.

### D. FirID — use pointer identity (`Rc::ptr_eq`) instead of a counter

Use raw `Rc` pointer comparison instead of an integer ID. **Rejected**: same
rationale as FOOP-74 Rejected Alternative B — pointer comparison is awkward
to carry on the thread-local stack and is not stable across cloning. A plain
`u64` is cheaper and doubles as a debugging aid.

## Open Questions

- Should Deadbrane be a standalone binary, a subcommand of `foolish-cli`, or
  a library function callable from any consumer? Lean: library function in
  `foolish-ubca` with a thin CLI wrapper in `foolish-cli`.
- Should Deadbrane analyze a single brane or an entire program (all branes
  in a file)? Lean: single brane by default, with a `--recursive` flag for
  nested branes.
- Should the Deadbrane report be integrated into the approval test output
  format (alongside alarms and step count), or kept as a separate output?
  Lean: separate — the approval test format is already established and
  adding a report would churn existing snapshots.
- Should FOOP-74's cycle-alarm tests be moved into this FOOP's FirID test
  suite, or remain separate? Lean: keep FOOP-74's tests separate — they test
  the alarm mechanism, not the cloning semantics.

## References

- Prior FOOP: FOOP-74 (FIRID — atomic per-Fir identity for cycle detection).
  This FOOP's FirID section extends FOOP-74's cloning semantics with explicit
  rules and tests.
- Code: `foolish-ubca/src/fir_kinds.rs` — `constanic_clone_at` (lines 158–198
  for the Constant/Independent `Rc::clone` early return and the kind-dispatched
  clone paths), `ib_search_with_engine`, `ab_search_with_engine` (the search
  implementations that Deadbrane will analyze).
- Code: `foolish-ubca/src/proto_brane.rs` — `ProtoBrane` (field-holder with
  `firid`).
- Code: `foolish-ubca/src/fir_trait.rs` — `FirKind` enum, `ib_search`,
  `ab_search`.
- Related: FOOP-23 (search operator specification — defines what "referenced
  by a search" means).

## Last Updated

**Date**: 2026-07-14
**Updated By**: Hephaestus / xiaomi/mimo-v2.5-pro
**Changes**: Initial draft. Deadbrane (useless-element detection: directly
useless, transitively useless, fixed-point algorithm, structured report) and
FirID cloning semantics refinement (pins Constant/Independent → Rc::clone
identity-sharing, non-constanic → new FIRID, Brane → always new FIRID).
