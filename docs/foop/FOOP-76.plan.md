# FOOP-76.plan — revival-of-equality

**Read `docs/foop/FOOP-76.md` before executing this plan.** The plan is derived from the
specification and assumes its context; the `§`-pointers below refer to that file.

**Worktree variables, expanded:**

```
WORKTREE_ORIGIN_BRANCH  = jia
WORKTREE_ORIGIN_PATH    = /yolo/foolish
WORKTREE_BRANCH_NAME    = foop-76-revival-of-equality
WORKTREE_FULL_FS_PATH   = /yolo/foolish_worktrees/foop-76-revival-of-equality
```

---

## Scope guard, standing for the whole plan

**This FOOP implements one relation and refreshes one document.** Per §FIR Impact and §UBC Step
Impact of the specification, it does **not**:

- change `FirSpec` — not one variant, not one field. It already derives `PartialEq`, and the
  relation is a walk over existing accessors;
- add or change any **step rule**, or touch NYES. The relation is read-only over constanic FIR and
  takes `&FVMStorage`;
- modify `foolish-core/**`, `foolish-parser/**`, `foolish-ubca/**`, `foolish-cli/**`, `einmo/**`
  or `zweimomo/**`;
- add Foolish **surface syntax** — unless §Open Questions **Q1** is answered as reading (b), which
  is the human's call and is Phase 0's blocking question.

**If a task appears to require any of those, STOP and report.** Each is a scope change and a human
decision, never an implementation convenience (AGENTS.md §"The agent is responsible for
correctness").

The files this FOOP is permitted to create or change under reading (a):

| Path | What |
|------|------|
| `foolish-ubca2/src/**` | The relation and its unit tests (§1–§4). Exact module decided in Phase 1 (Q3). |
| `docs/` — the refreshed `EQUIVALENCE.md`, relocated | **The documentation deliverable** (§5). |
| `docs/vintage_legacy/EQUIVALENCE.md` | Removed from here as part of the move (§5). |
| `docs/README.md` | Index updated with the relocated document (§5). |
| `docs/foop/FOOP-23.plan.md` | §D.4's inherited checkbox, closed (§5). |
| `docs/foop/FOOP-36.md` | T2c's note only, if T2c lands here rather than there. |
| `docs/foop/FOOP-76.md` | This FOOP's spec — Q1–Q5 recorded as they are answered. |
| `docs/foop/FOOP-76.plan.md` | This plan — checkboxes and timestamps. |
| `docs/foop/INDEX.md` | Status updates only. |

Anything else is out of scope.

---

## How to work this plan

1. **Read `docs/foop/FOOP-76.md` first.** §1 (shape), §2 (value), §3 (two FVMs), §4 (conditions)
   and §5 (the documentation deliverable) are load-bearing for every phase.
2. **Work top to bottom.** Phases are ordered by dependency: the shape half is the precondition for
   the value half (§2.1), and the residual type must exist before the tests that assert on it.
3. **Check each box as you finish it, with a timestamp on the next indented line** —
   `(YYYY-MM-DD HH:MM)`.
4. **When a checkbox says STOP, stop.** Those mark decisions that are the human's.
5. **Accumulate doubts; report them once, at the end of a phase** (AGENTS.md). Do not interrupt per
   item.
6. **Run the sub-section test subset frequently** — after each increment and each new test — and all
   tests when the sub-section completes (`foop.md` §"Sub-Section Test Subsets").

---

## Orientation — the facts you need, so you need not go find them

*Established while `FOOP-76.md` was written, verified against the tree at `b4b93b45`, 2026-09-15.*
***Verify before relying on any of them; do not re-derive them from scratch.***

| Fact | Location |
|---|---|
| `FirSpec` enum | `foolish-ubca2/src/fvm_storage.rs:304` |
| `FirSpec` derives `PartialEq` — **this is why §1.4 is a paired walk, not a new theory** | `fvm_storage.rs:303` — `#[derive(Debug, Clone, PartialEq)]` |
| `FVMStorage::foolish_children(ptr) -> &[FirPointer]` | `fvm_storage.rs:472` |
| Borrowed-view `foolish_children()` / `ubc_children()` | `fvm_storage.rs:1676` / `:1680` |
| `SYSTEM_FOO_SRC` — the shared ancestor §3.1 seeds from | `foolish-ubca2/src/system_foo.rs:106` |
| `compose_program_with_system` — the deterministic composer | `fvm_storage.rs:4944` |
| `ib_search_with_engine` / `ab_search_with_engine` | `fvm_storage.rs:2908` / `:2948` |
| Search scan takes **`&FVMStorage`** (immutable) — the §UBC-Step-Impact precedent | FOOP-36 §N4.b, `search_engine::contextful_search_scan` |
| Baseline: **791 workspace tests passing** on `jia` at `b4b93b45` | `cargo test --workspace` |

**The three-outcome signature** (§4), verbatim from the human's worked example — the return type
every phase after Phase 4 is written against:

```
NO
YES
YES, provided [(fvm1_creation1, fvm2_creation10), (fvm1_creation3, fvm2_creation2), ...]
```

**The bijectivity counterexample** (§3.2) — the one the human asked be written down, and the plan's
single most important test:

```
[(vm1_a1, vm2_a1), (vm1_a2, vm2_a1)]        ← must answer NO, in BOTH directions
```

---

## Phase 0 — Begin, and settle the two blocking scope questions

**Larger model.** Q1 is not this agent's to decide; it must be asked and answered.

- [ ] Begin work: commit `FOOP-76.md` and `FOOP-76.plan.md` to origin, check `begun: [x]` in the
      frontmatter of `FOOP-76.md`
- [ ] Record the baseline: run `cargo test --workspace` on `jia` and confirm **791 passing**. If the
      number differs, note the actual figure here before proceeding — the merge gate compares
      against whatever is recorded, and an *increase* is as suspect as a decrease.
- [ ] STOP! ASK THE HUMAN: **§Open Questions Q1 — the relation only (a), or the full operator
      taxonomy (b)?** Present the tradeoff as §Q1 states it: (a) is contained — the Rust-side
      relation, `EQUIVALENCE.md` refreshed honestly, FOOP-36's T2c delivered, `FOOP-23.plan.md`
      §D.4 closed, no surface syntax and no einmo cases; (b) additionally specifies and implements
      `=s=`, `==`, `===`, `=n=`/`=N=`, `=c=`/`=C=`, `=v=`/`=V=` as Foolish operators, which needs a
      defined semantics for each (`===` may have no decision procedure at all), lexer/parser work,
      FIR representation, step rules and an einmo suite — plausibly several FOOPs.
      **Record the answer in `FOOP-76.md` §Open Questions Q1.** This plan is written for (a); if
      the answer is (b), the plan gains phases and this checkbox says so rather than improvising.
- [ ] STOP! ASK THE HUMAN: **§Open Questions Q2 — wait on FOOP-66's Q5, or proceed in parallel?**
      FOOP-66 §3 **Q5** studies whether tree calculus's equality machinery transfers here;
      **Q5c** (equality-with-conditions as unification) and **Q5d** (no normal form ⇒ a
      fundamentally different question) are the two that could change §4's framing, which is the
      expensive thing to change late. FOOP-66's INDEX entry calls the dependency **soft**.
      **Record the answer in `FOOP-76.md` §Open Questions Q2.**
- [ ] Create worktree at `/yolo/foolish_worktrees/foop-76-revival-of-equality` with branch
      `foop-76-revival-of-equality`
  - [ ] Run:
        ```bash
        git worktree add -b foop-76-revival-of-equality \
            /yolo/foolish_worktrees/foop-76-revival-of-equality
        ```
  - [ ] `cd /yolo/foolish_worktrees/foop-76-revival-of-equality` — **all further work happens here**,
        including every edit under `docs/foop/`, until merge time

---

## Phase 1 — Read, and place the relation in the vintage taxonomy

**Larger model.** The deliverable is a judgment that shapes §5's rewrite and Q3's answer.

- [ ] Establish relevant tests for this phase. Use [these
      instructions](../../README.md#running-specific-tests) to run unit tests:
      `foolish-ubca2::compose_program_with_system_settles_a_trivial_program`, `foolish-ubca2::system_foo`.
      Nothing is implemented yet; this subset is the "did I break the crate" canary. Add new tests to this
      list as they are written.
- [ ] Read `FOOP-76.md` §1 through §4 in full
- [ ] Read `docs/vintage_legacy/EQUIVALENCE.md` in full (84 lines)
- [ ] Read `docs/why/creation_postulate.md` — §3.2's bijection argument rests on it
- [ ] Read FOOP-36 §2.2 and §2.2.1 — WHICH pair to compare, which this FOOP does not re-decide
- [ ] Read FOOP-23 §Open Questions' "equality maturation" note and `FOOP-23.plan.md` §D.4's
      unchecked `EQUIVALENCE.md` task, which this FOOP inherits (§5)
- [ ] **Write down where this relation sits in the vintage taxonomy** — §5 says it is a `==`-like
      relation (structural, over already-stepped FIR). Confirm or correct that against the document's
      own wording, and record the finding in `FOOP-76.md` §5. Use the document's vocabulary, not a
      parallel one.
- [ ] **Answer §Open Questions Q3 — where the relation lives.** `foolish-ubca2` (it reads
      `FVMStorage`) or lower. Record the decision and its reason in `FOOP-76.md` §Open Questions Q3.
      Decide deliberately; do not let it fall out by default.
- [ ] Run all tests — old and new — and make sure they all pass correctly.

---

## Phase 2 — The lockstep walk and the shape half (§1, §2.1)

**Larger model.** The failure-ordering invariant is subtle, and getting it wrong silently degrades
every diagnosis the relation will ever produce.

- [ ] Establish relevant tests for this sub-section. Use [these
      instructions](../../README.md#running-specific-tests) to run unit tests: `foolish-ubca2::fir_equality`
      (the new module's tests, as they are written), plus the Phase 1 canary subset. Add each new test to
      this list as it lands.
- [ ] (read §1 and §2.1 of `FOOP-76.md`)
- [ ] Implement the **lockstep walk** over `foolish_children` — the same order, position by
      position, index by index (§2.1). The walk is what makes a creation pair meaningful; if it ever
      has to *guess* which right node matches a left node, the design has been violated.
- [ ] Implement the **shape comparison** (§1): kind, then arity, then children pairwise in order,
      then the shape-bearing `FirSpec` fields — `Search { pattern, anchored, forward,
      is_value_search, contexted }`, `Index { offset, anchored, contexted }`, `Operator { op }`,
      `Comparison { op }`, `Statement { identifier }`.
- [ ] Confirm the **ignored** fields are ignored (§1): `Statement { line_number }`,
      `FoolRef { referent }`, `Nk { reason }`, `Concatenation { rendering_aid }`, `ubc_children`,
      NYES.
- [ ] Implement the **strict order of failure** (§2.1): kind, then arity, then — only if both agree
      — descend or compare values. A shape mismatch **fails immediately and the walk stops**; it
      never reaches the creation table at that position or below it. **This is an invariant, not an
      optimization**: a shape failure must be distinguishable from a table failure, since the first
      says *the structure differs* and the second says *the structure agrees but the sharing does
      not*.
- [ ] STOP-CONDITION CHECK: if implementing this appears to require a change to `FirSpec`, or a
      change to any crate outside `foolish-ubca2`, **STOP and report to the human** (see the scope
      guard). Do not proceed.
- [ ] Run all tests — old and new — and make sure they all pass correctly.

---

## Phase 3 — The creation table and the bijection check (§2.2, §2.3, §3.2)

**Larger model.** §3.2 is a language-semantics argument from the Creation Postulate, not a coding
task, and it decides what is a hard `NO` versus a reportable condition.

- [ ] Establish relevant tests for this sub-section. Use [these
      instructions](../../README.md#running-specific-tests) to run unit tests: `foolish-ubca2::fir_equality`
      (including the new `T-SHARE` case), plus the Phase 1 canary subset. Add each new test to this list as
      it lands.
- [ ] (read §2.2, §2.3, §2.4 and §3.2 of `FOOP-76.md`)
- [ ] Implement the **three-case rule** at every creation pair (§2.2): neither in the table → equal
      and record; the pair in the table → equal; either paired to something else → **fail**.
- [ ] Implement the **bijection check in BOTH directions** (§3.2). A left creation may correspond to
      exactly one right creation *and vice versa*, so the table is consulted both ways before
      recording. This is why a contradicted pairing is a hard `NO` and not something to report and
      leave to the caller — the Creation Postulate makes it a contradiction, not an unattractive
      condition.
- [ ] Implement the **reflexive shortcut** (§2.3): same arena, same pointer → equal, **record
      nothing**. Two creations from different arenas are always distinct `FirPointer`s, so `A ≡ A`
      does not arise there at all. **This is an optimization inside one comparison, never a property
      the table's design relies on** — write it so a reader cannot mistake it for the latter.
- [ ] Make **check-and-record one atomic step** (§2.4). Under out-of-order or parallel comparison the
      table is shared state, and two children racing to record `(L→R₁)` and `(L→R₂)` is a real
      violation that can be **LOST** if the check and the record are separable.
- [ ] Carry §2.4's note **into the code as a comment**: the table looks redundant in FOOP-36's T2c
      inputs (the shape half tends to catch the defect first — node counts 7 against 5), and it
      ships anyway because out-of-order execution, out-of-order comparison and the two-FVM setting
      each break the assumption that position implies identity. **The one thing that must not happen
      is a future reader seeing an assertion that rarely fires, concluding it is dead, and deleting
      it.**
- [ ] Run all tests — old and new — and make sure they all pass correctly.

---

## Phase 4 — The return type: `NO` / `YES` / `YES-provided` (§4)

**Smaller model.** The shape is given verbatim in §4 and in this plan's Orientation block; this is
typing it, not designing it.

- [ ] Establish relevant tests for this sub-section. Use [these
      instructions](../../README.md#running-specific-tests) to run unit tests:
      `foolish-ubca2::fir_equality`, plus the Phase 1 canary subset. Add each new test to this list as it
      lands.
- [ ] (read §4 and §4.1 of `FOOP-76.md`)
- [ ] Implement the **single function returning the residual**, with the three outcomes of §4:
      unconditionally equal (empty residual), conditionally equal (the pairs it had to assume), not
      equal (shape mismatch, integer mismatch, or a contradicted pairing).
- [ ] Implement the **boolean as a thin wrapper** asking "is the residual empty?" — **not as
      separate code** (§4). Retrofitting a residual onto a boolean changes every return path, which
      is why the residual is designed in from the start.
- [ ] Implement **integer comparison** (§2): `IndepInt { value }` by value.
- [ ] Do **not** implement §4.1's follow-ons (composing residuals, caller-supplied assumptions,
      minimality, cross-arena composition). They are recorded there so the **return type does not
      have to change later**; leave the type able to accommodate them and go no further.
- [ ] Run all tests — old and new — and make sure they all pass correctly.

---

## Phase 5 — System seeding (§3.1) and the constanic precondition (§3.3)

**Smaller model.** Both procedures are stated and both have fixed targets.

- [ ] Establish relevant tests for this sub-section. Use [these
      instructions](../../README.md#running-specific-tests) to run unit tests: `foolish-ubca2::fir_equality`
      (including `T-SEED` and `T-PRE` as they land),
      `foolish-ubca2::compose_program_with_system_settles_a_trivial_program`, `foolish-ubca2::system_foo`.
- [ ] (read §3.1 and §3.3 of `FOOP-76.md`)
- [ ] Implement **seeding**: before comparing two subtrees from different FVMs, pre-populate the pair
      list with every system creation paired to its counterpart, by walking the two system branes in
      lockstep and pairing creation to creation. Both FVMs compose the same `SYSTEM_FOO_SRC`
      (`system_foo.rs:106`) through the same deterministic composer (`fvm_storage.rs:4944`), so this
      is mechanical. Names are not needed, though they make it checkable.
- [ ] **Seeded pairs never appear in the residual** (§3.1). A residual names only what the users'
      programs introduced; "provided `'True` equals `'True`" is noise that buries what matters.
- [ ] **A system creation paired against a non-system one is a hard `NO`** (§3.1), not a condition.
      No caller judgment can rescue it.
- [ ] Implement **name lookup within a tree** as ordinary nested scoping — or, preferably, **use the
      FVM's own `ib_search` / `ab_search`** (`fvm_storage.rs:2908` / `:2948`). §3 is explicit: the
      language already answers this question correctly and a parallel implementation can only drift
      from it. Do **not** invent per-brane tables with entry/exit lifetimes — that mechanism is what
      produced FOOP-36's now-dissolved Q9.
- [ ] Implement the **constanic precondition** (§3.3). Choose the enforcement — type system,
      assertion, or a checked error return — and **state the choice in `FOOP-76.md` §3.3**. What
      matters is that it is a stated requirement, not an unexamined assumption.
- [ ] Run all tests — old and new — and make sure they all pass correctly.

---

## Phase 6 — Test scaffolding: T-SIG, T-SHAPE, T-PRE

**Smaller model.** Every expected outcome is named in §Test Plan; the target is fixed.

- [ ] Establish relevant tests for this sub-section. Use [these
      instructions](../../README.md#running-specific-tests) to run unit tests: `foolish-ubca2::fir_equality`
      — specifically the new `T-SIG`, `T-SHAPE` and `T-PRE` cases — plus the Phase 1 canary subset.
- [ ] (read §Test Plan of `FOOP-76.md`)
- [ ] **T-SIG** — one test per outcome: `YES` (empty residual); `YES, provided […]` (non-empty, and
      containing *exactly* the assumed pairs — no seeded pairs, no reflexive pairs); `NO` in all
      three sub-cases (shape mismatch, integer mismatch, contradicted pairing). Plus: the boolean
      wrapper agrees with "is the residual empty?" on every one of them.
- [ ] **T-SHAPE** — arity catches FOOP-36's fused `f1f2` concatenation defect (3 constituents against
      2); shape-bearing fields are compared (`?x` against `~x` is a `NO` though kind, arity and
      children agree); ignored fields are ignored (differing `line_number`, `Nk { reason }`,
      `rendering_aid` must not alone produce a `NO`); and **order of failure** — an input that would
      produce both a shape failure and a table failure reports the **shape** one.
- [ ] **T-PRE** — a pre-constanic input is **rejected**, not silently compared, by whichever
      enforcement Phase 5 chose.
- [ ] Run all tests — old and new — and make sure they all pass correctly.

---

## Phase 7 — T-BIJ and T-SHARE: the hand-written expectations

**Larger model. NOT DELEGABLE.** These are the expectations the design depends on. Generating them
from the implementation and calling them hand-written would void the test entirely — the human asked
for T-BIJ's counterexample *specifically*, to make certain it is handled.

- [ ] Establish relevant tests for this sub-section. Use [these
      instructions](../../README.md#running-specific-tests) to run unit tests: `foolish-ubca2::fir_equality`
      — specifically the new `T-BIJ` and `T-SHARE` cases — plus every test established in Phases 1–6.
- [ ] (read §3.2 and §2.2 of `FOOP-76.md`)
- [ ] **T-BIJ — the bijectivity counterexample, BOTH directions.** Two distinct VM1 creations against
      one VM2 creation reached twice must answer **`NO`**:
      `[(vm1_a1, vm2_a1), (vm1_a2, vm2_a1)]`. Derive the expected answer **from the Creation
      Postulate**, not from running the code: `vm1_a1 ≠ vm1_a2` by the postulate while
      `vm2_a1 = vm2_a1` trivially, so the pairing asserts that two things known to differ are both
      equal to one thing — a contradiction no caller judgment can discharge.
  - [ ] Direction 1: two left creations against one right creation → `NO`
  - [ ] Direction 2: one left creation against two right creations → `NO`
  - [ ] Confirm the answer is `NO` and **not** `YES, provided …` — answering with a condition that
        cannot be discharged is worse than answering `NO`, because it looks like an answer
- [ ] **T-SHARE** — `{orig = ⬤; ref = orig;}`, one creation reached twice, against a tree with TWO
      distinct creations in those positions. Sharing preserved passes; sharing split fails.
      **Construct the FIR so the table is genuinely the thing under test** — per §2.4, in FOOP-36's
      own T2c inputs the shape half tends to catch this first (node counts 7 against 5), so a
      carelessly-built case would pass for the wrong reason and prove nothing.
- [ ] **T-SEED** — the §3.1 assertions: system creations correspond by construction; seeded pairs
      never appear in a residual; a system creation against a user creation is a hard `NO`.
- [ ] Write, in the plan or the commit message, **why each expected answer is what the specification
      requires** — in your own words, per case. If you cannot write it, you have not derived it.
- [ ] Run all tests — old and new — and make sure they all pass correctly.

---

## Phase 8 — T2c, FOOP-36's deferred test

**Larger model** — it is a judgment about which side owns the test.

- [ ] Establish relevant tests for this sub-section. Use [these
      instructions](../../README.md#running-specific-tests) to run unit tests:
      `foolish-ubca2::fir_equality`, plus FOOP-36's `T2` round-trip tests if FOOP-36 has merged.
- [ ] (read §Test Plan "T2c" of `FOOP-76.md`, and FOOP-36 §Test Plan's own T2c note)
- [ ] **Check whether FOOP-36 has merged.** T2c's home depends on it; the plan says check rather than
      assume. If FOOP-36 is unmerged, wire T2c here and leave a note in FOOP-36 §Test Plan pointing
      at it.
- [ ] Wire T2c: `spp` vs `spr`, the two stepped FIRs. FOOP-36's T2 procedure **already builds both
      arenas and discards them**, so the hook is cheap.
- [ ] Use the **strict reading** (§4): for FOOP-36's Property 3 a **non-empty residual is a
      FAILURE**, because a rendering that requires two creations to be identified is a rendering that
      lost the distinction. Do not let the conditional form soften this particular caller.
- [ ] Run all tests — old and new — and make sure they all pass correctly.

---

## Phase 9 — The documentation deliverable: `EQUIVALENCE.md` refreshed and relocated (§5)

**Larger model.** This is writing language documentation, and the judgment is which claims are
specified, which are sketch, and where the document belongs.

- [ ] Establish relevant tests for this sub-section. Use [these
      instructions](../../README.md#running-specific-tests) to run unit tests: the full
      `foolish-ubca2::fir_equality` set — the document must describe what was actually built, so the tests
      are the check on the prose.
- [ ] (read §5 of `FOOP-76.md` in full)
- [ ] **Rewrite `EQUIVALENCE.md` against the definitions this FOOP actually established** — §1's
      structural relation, §2's creation correspondence, §3's two-FVM framing, §4's
      conditional/residual form — where the vintage document had only sketches.
- [ ] **Say that `YES, provided […]` is the CONSTRUCTIVE form of the vintage `===`** (§5, point 2).
      `===` quantifies over *all possible coordinations*, which is correspondingly hard to check;
      the residual hands back the exact conditions under which equality holds and lets the caller
      judge them. Carry §3.1's system-equality rule too — it is what makes a cross-FVM comparison
      start seeded rather than empty.
- [ ] **Mark clearly which operators are specified-and-implemented, which are specified-only, and
      which remain sketch** (§5), so a reader is never left believing `===` exists when it does not.
- [ ] **Move the document OUT of `docs/vintage_legacy/`** into the live documentation tree.
      Destination is this FOOP's call: `docs/ubc1/how/` if it reads as engineering reference,
      `docs/why/` if the emphasis is the design rationale for what equality MEANS in Foolish.
      Record the choice and its reason.
- [ ] **Update `docs/README.md`'s index** with the relocated document.
- [ ] **Close `FOOP-23.plan.md` §D.4's unchecked `EQUIVALENCE.md` checkbox** — value-search equality
      currently means integer equality only, pending an equivalence FOOP, and **this is that FOOP**
      (§5, point 3). Revisit FOOP-23 §Open Questions' "equality maturation" note in the same pass.
- [ ] **Answer §Open Questions Q5** — whether the `docs/howto/` equivalence chapter is in scope or a
      follow-on. `docs/howto/03_howto_foolish_todo.foo` lists equivalence among its unwritten
      chapters, and the refreshed document makes it writable. Record the answer in `FOOP-76.md`.
- [ ] **Answer §Open Questions Q4** — whether `pp` vs `pr` (FOOP-36 §2.2's third row) is worth
      implementing alongside as a debugging aid. Cheap now that the relation exists. Record the
      answer; implement only if the answer is yes and the human agrees.
- [ ] Run all tests — old and new — and make sure they all pass correctly.

---

## Phase 10 — einmo cases and the comprehensive test — CONTINGENT ON Q1

**Read this phase before executing it; under reading (a) most of it is discharged, with reasons.**

**Under Q1 reading (a) — the relation only — this FOOP ships no Foolish surface syntax**, so:

- **There are no einmo cases and no `foolish-ubca/einmo_suite/input/foop/76/`.** There is no `.foo`
  input to write, because the relation has no Foolish syntax to exercise. `foop.md`
  §"Comprehensive FOOP Tests" makes a comprehensive test obligatory for a FOOP that ships a
  *feature* with surface behavior; a comprehensive case here would exercise the status quo and sign
  it as this FOOP's contribution. **This omission is a recorded decision, discharged by §Test Plan
  ("Einmo coverage is contingent on Q1"), not a lapse.**
- **There is no Promotion Review Gate and no `einmo promote` invocation**, because nothing is
  generated. A plan that promotes nothing needs no gate; installing an empty one would falsify the
  record.
- **The suite must still be green.** The relation is additive and read-only, so
  `cargo test -p foolish-ubca --lib -- einmo_gate_checked` must pass **unchanged**. If any
  pre-existing baseline diverges, that is a **regression this FOOP introduced** — fix the code;
  **never** promote over it (`rust_instructions.md` §"Phase-by-phase testing discipline").

**Under Q1 reading (b) — operators in scope — this phase is REAL and this plan is incomplete.**

- [ ] Confirm which reading Q1 settled on (Phase 0 recorded it in `FOOP-76.md` §Open Questions Q1)
- [ ] **If (a):** check this box to record that the einmo work is discharged with the reasons above,
      and run `cargo test -p foolish-ubca --lib -- einmo_gate_checked` to confirm the suite is
      unchanged
- [ ] **If (b): STOP.** This plan was written for (a). Return to the human with a revised plan adding
      the operator phases — lexer, parser, FIR representation, step rules,
      `input/foop/76/comprehensive.foo`, and a full **Promotion Review Gate** with one named
      sub-task per einmo case (`foop.md` §"Promotion Review Gate"). Do not improvise those phases
      inside this document.
- [ ] Run all tests — old and new — and make sure they all pass correctly.

---

## Phase 11 — Verify, merge, cleanup

**Smaller model**, with a human STOP.

- [ ] Establish relevant tests for this phase. Use [these
      instructions](../../README.md#running-specific-tests) to run the full workspace suite and the einmo
      gate — this phase's subset IS everything.
- [ ] Verify all work is complete in `/yolo/foolish_worktrees/foop-76-revival-of-equality` and
      committed to `foop-76-revival-of-equality`
- [ ] Confirm the scope guard held: `FirSpec` unchanged; no step rule added or changed; no crate
      outside `foolish-ubca2` modified except the documentation files the guard lists
- [ ] Run all tests — old and new — and make sure they all pass correctly.
  - [ ] `cargo test --workspace` — **at least the 791 recorded in Phase 0**, plus this FOOP's new
        tests. A *decrease* means something broke; an *increase* beyond the new tests means
        something unexpected was added, and both need explaining rather than papering over.
  - [ ] `cargo test -p foolish-ubca --lib -- einmo_gate_checked` — must exit 0, **unchanged**
  - [ ] `cargo fmt --check` and `cargo clippy -D warnings` per `rust_instructions.md`
- [ ] Report ALL accumulated doubts to the human in ONE statement — or record "no doubts". Blocking
      doubts stop here; non-blocking ones are reported alongside (AGENTS.md).
- [ ] STOP! STOP!! STOP!!! ASK HUMAN to check this box before continuing. UNDER NO CIRCUMSTANCES will
      Agent continue past this point automatically!!
  - [ ] Present the human with `cd /yolo/foolish_worktrees/foop-76-revival-of-equality` and ask them
        to review **the relocated `EQUIVALENCE.md`** and **T-BIJ's hand-written expectation** BEFORE
        checking the parent checkbox. Those two are the FOOP's substance: one is language
        documentation that will be read for years, the other is the test the human asked for by name.
- [ ] Merge `foop-76-revival-of-equality` to `jia`
  - [ ] Repair ALL tests in `jia` at `/yolo/foolish` if the merge breaks any
- [ ] Update `docs/foop/INDEX.md` — status, and the by-status list
- [ ] Cleanup `/yolo/foolish_worktrees/foop-76-revival-of-equality`
  - [ ] Check that this `.plan.md` has all but Cleanup checkboxes completed
  - [ ] Remove `/yolo/foolish_worktrees/foop-76-revival-of-equality`
  - [ ] This is the last sub-task checkbox to be checked in this block

---

## Last Updated

**Date**: 2026-09-15
**Updated By**: Claude Code / claude-opus-5
**Changes**: Created the plan for FOOP-76 (Revival of Equality), alongside the specification broken
out of FOOP-36 §N6. Twelve phases ordered by dependency, since the shape half is the **precondition**
for the value half (§2.1) and the residual type must exist before the tests that assert on it: Phase
0 begin, baseline, and **two blocking human questions** (Q1 — relation only or the full operator
taxonomy; Q2 — wait on FOOP-66's Q5 or proceed in parallel); Phase 1 read and place the relation in
the vintage taxonomy (larger model); Phases 2–3 the lockstep walk, shape half, creation table and
bijection check (larger model — §2.1's failure ordering and §3.2's Creation-Postulate argument are
judgment, not typing); Phases 4–6 the return type, system seeding, precondition and test scaffolding
(smaller model — each has a fixed target named in §Test Plan); **Phase 7 the hand-written
expectations T-BIJ and T-SHARE, marked NOT DELEGABLE**, since generating them from the implementation
would void the test the human asked for by name; Phase 8 FOOP-36's deferred T2c; Phase 9 the
`EQUIVALENCE.md` refresh and relocation (larger model); **Phase 10 contingent on Q1** — under reading
(a) it records *why* there is no `input/foop/76/`, no comprehensive case and no Promotion Review Gate
(nothing is generated, so an empty gate would falsify the record), and under reading (b) it STOPS and
returns for a revised plan rather than improvising operator phases; Phase 11 verify, human STOP,
merge, cleanup. A standing scope guard lists the permitted files and directs a STOP on any change to
`FirSpec`, any step rule, or any crate outside `foolish-ubca2`. The Orientation block carries the
verified code facts (`FirSpec` derives `PartialEq` at `fvm_storage.rs:303`, `foolish_children` at
`:472`, `SYSTEM_FOO_SRC` at `system_foo.rs:106`, `compose_program_with_system` at `:4944`) marked
*verify, don't re-derive*, and the **791-test `b4b93b45` baseline** the merge gate compares against.
