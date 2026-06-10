# FOOP-62 Feedback Synthesis — Deepseek + mimo, with Claude's resolutions

**Date**: 2026-06-09
**Author**: Claude Code (Claude Code); Opus 4.8
**Inputs**: `FOOP-62.deepseek-feedback.md` (Deepseek V4 Pro), `FOOP-62.mimo-feedback.md` (Sisyphus / mimo-v2.5-pro)
**Method**: combined both reviews, verified the load-bearing factual claims against the
codebase, and formed a single prioritized resolution list. Where the two reviewers disagree,
I checked the code and ruled.

---

## TL;DR — what actually has to change

Both reviewers like the architecture (8/10 design, "the right abstraction"). The concern lists
overlap heavily; once de-duplicated and fact-checked, the **real** blockers are fewer than
either list suggests, because Deepseek's two BLOCKING concerns rest on a false premise.

**The one fact that resolves the most:** *snapshots do not contain step counts.* Verified:
`snapshot_suite.rs:135` formats via `FirSequencer::format` (INPUT → RESULT → COMMENTS →
signature); `format_with_header` (which carries `steps`) is REPL/debug only, and `grep` finds
no `STEPS` token in any approved `.snap`. **mimo is right; Deepseek is wrong on this.** That
demotes Deepseek Concerns 1 & 2 from "BLOCKING" to "not acceptance-relevant."

**True blockers (must decide before code):**
1. **Composition model** — does `Fir` *contain* a `ProtoBrane`, or *is* it one? (mimo #2). The
   spec never says. This touches every line. → **Decide: kinds contain a ProtoBrane.**
2. **`FirRef` type** — stays `Rc<RefCell<dyn Steppable>>` or becomes `Rc<RefCell<dyn Fir>>`/
   enum? (Deepseek #7, mimo #5). → **Decide: `Rc<RefCell<dyn Fir>>`, trait-object, enum-free.**
3. **Compiler parent-wiring** — immutable-parent-at-construction forces a real `compiler.rs`
   refactor (build parent shell via `Rc::new_cyclic`, thread `Weak` down). (mimo #4). Verified:
   12+ `ParentPtr::new()` sites, all bottom-up. → **Plan as a dedicated, highest-risk sub-task.**

**Downgraded / dissolved:** Deepseek #1 (fixpoint), #2 (step counts) — not snapshot-visible, but
see the *real* residue below. The genuine survivor inside them is **state-sharing**, which mimo
caught (#1c): are tasks fresh clones or shared `Rc`s? That *can* change output.

---

## Section A — Where the reviewers agree (high confidence, just do it)

These appear in both reviews (sometimes worded differently); I concur with all:

| Item | Deepseek | mimo | Resolution |
|---|---|---|---|
| Sequencer is risky; keep a thin `FirQueryable` adapter first, retire later | #8 | #3 | **Adopt as the DEFAULT plan, not a fallback.** Spec §8 already allows it; make it primary. |
| `bon` `updater` boilerplate / pin version | #10 | "bon version" | Pin `bon = "=3.9.1"` (or current 3.x); accept per-payload `updater` for now, macro later if it bites. |
| `Rc::new_cyclic` root self-`Weak` is standard but needs care | #7 | "Rc::new_cyclic Low" | Keep; add a worked constructor snippet to the spec. |
| Finite-time-to-constanic guarantee must be located or stated | (implied) | "Open Q" | **Make it explicit in the FOOP** (don't rely on it being documented elsewhere). |
| NYES transition table should have a concrete target | — | "Open Q" | Reference UBC's `step_one` + `compute_brane_state` as the literal target. |

## Section B — The step-count / fixpoint cluster (Deepseek #1, #2; mimo #1)

**My ruling: not an acceptance blocker, but there is a real residue.**

- Deepseek #1 (fixpoint `prev==new` breaks under one-transition-per-step) and #2 (step counts
  diverge) are both **premised on step counts being snapshot-visible**. They are not. So the
  "byte-exact snapshot" gate is unaffected by stepping granularity. Deepseek's own Option-3 and
  mimo's Option-(a) (batch each child to constanic before the next) and Option-(b) (accept
  different counts) converge once you know counts aren't compared.
- **However**, two real things survive and the spec MUST address them:
  1. **Fixpoint termination is still a correctness question even if counts don't matter.** If a
     node's own NYES stays `Braning` while its task queue drains, the *current* `prev==new`
     break could stop the outer loop early (Deepseek's Call-1/Call-2 example). The outer loop's
     stop condition must be redefined in ProtoBrane terms: **stop when the root is constanic OR
     made no progress**, where "progress" includes *task-queue shrinkage / any descendant NYES
     change*, not just root-NYES change. This is Deepseek Option A (stall counter) or
     Option B (progress bool) — I prefer **B: `step() -> (Nyes, Progress)`** so termination is
     explicit, not a heuristic stall count. The "retained unchanged" claim in §3/§7 is **wrong
     as written** and must be softened to "retained, with the stop condition redefined over
     progress."
  2. **State-sharing (mimo #1c) is the genuine output risk.** Current UBC `step_boxed` *clones*
     each statement body before stepping (`ubc.rs:251`), so siblings are independent. If
     ProtoBrane tasks are shared `Rc<RefCell<Fir>>` into `foolish_children`, stepping a task
     mutates the real child in place — which is the *intended* design, but it means the spec
     must confirm that in-place stepping of `foolish_children` reproduces the same final states
     as clone-then-step. **This is the actual thing to validate first in implementation.**

**Action:** rewrite §3.3/§7 termination wording (progress-based, `step()` returns progress);
add an explicit "in-place vs clone-then-step equivalence" note + an early test.

## Section C — NormalBrane re-stepping & statements (Deepseek #3, #4, #5; mimo "foolish_children stepping", "scope threading")

Both reviewers independently flag that the **brane is the hard case** and "mirror UBC" is too
thin. I agree — this is the single most under-specified area. The cluster:

- **Statement metadata home** (Deepseek #4, mimo #2-adjacent): a brane's `foolish_children` are
  statement *bodies*, but `name`/`line_number` live on `StatementFir`. **Resolution: make
  `StatementFir` a first-class `Fir`** (a len-1 ProtoBrane wrapping its body, carrying name/line
  as leaf data). Its `fir_op_step` publishes its name into scope once its body is constanic.
  This is Deepseek #4's second option and matches §2's "Statement: `[body]` (len 1)". The spec
  must say this outright.
- **Incremental scope / sibling visibility** (Deepseek #3, mimo "scope threading"): current UBC
  builds scope incrementally (`current_brane`/`current_stmt_idx`, `ubc.rs:278`) so stmt_i sees
  evaluated stmt_<i. Under interleaved task draining, the scope must be **per-task-position**,
  not per-parent. **Resolution: the brane's drain sets `current_stmt_idx` for the task it is
  stepping** (the brane's `fir_op_step`/drive owns scope threading; it is *not* a generic
  ProtoBrane behavior). This means **the brane overrides the default drain** (or wraps it) — a
  concession that "written once" has one principled exception (see Section D).
- **Re-step task rebuild** (Deepseek #5): when re-stepping, **rebuild the task list from
  `foolish_children`** (re-enqueue all; already-constanic pop immediately) in addition to
  "clear `ubc_children`." Deepseek is right this is currently unspecified. Add to §4.
- **Econstanic-is-constanic trap** (Deepseek #5, sharp catch): `is_constanic()` is true for
  `Econstanic`, so an unbound-name child gets **popped** off the queue — then a later forward
  resolution has nothing to re-step unless the rebuild above re-enqueues it. The §4 rebuild
  rule resolves this; the spec must call out the trap explicitly so nobody "optimizes" the
  rebuild away.

**Action:** add a dedicated **§9 "NormalBrane & Statement stepping"** covering: StatementFir as
Fir, incremental scope threading owned by the brane drain, re-step rebuild, the Econstanic-pop
trap. This is the biggest spec addition.

## Section D — SF/SFF special-casing (Deepseek #6, mimo "EvalContext")

Deepseek is correct and this is important: `step_except_brane_searches` is **not** just a scope
flag — it's a separate evaluation algorithm with its own variant handling (`fir.rs`). mimo adds
that `EvalContext` already lives on `Scope` and must carry over.

**My ruling:** don't pretend "one algorithm fits all." The honest position:
- The **default `protobrane_step` is the shared drain**; SF/SFF (and, per Section C, the brane)
  are the **named exceptions** that override `step()` / wrap the drain with their own
  scope-context modification. This is fine and still a massive reduction from 50 methods — the
  spec just must **stop claiming universality** and instead say "the drain is the default;
  these N kinds override, and here's why."
- `EvalContext` carries over on `Scope` unchanged; SF sets block-brane-searches, SFF sets the
  SFF context, before stepping their child. State the threading explicitly.

**Action:** in §3, demote "written ONCE / uniform" to "shared default with a small, enumerated
set of overriders (Brane, SF, SFF)"; add an `EvalContext`-carries-over note.

## Section E — Infrastructure (mimo #5, "Evaluator"; serialization; has_unresolved_forward_refs)

These are mimo-only and all verified true:

- **`Evaluator` trait type mismatch** (mimo #5): verified — `snapshot_suite.rs` pins
  `FirRef = Rc<RefCell<dyn Steppable>>`. UBCa needs a different ref type. **Resolution:**
  genericize `Evaluator<F>` OR have UBCa's harness wrap its FIR in a thin `Steppable`-shaped
  adapter for the suite. I lean **genericize** (cleaner, no adapter lie), but either is ~50 lines.
- **`has_unresolved_forward_refs`** (mimo): re-express over `foolish_children` + `ubc_children`
  walk instead of `clone_into_fir()` matching. Mechanical; add to plan.
- **Serialization**: confirm the new layout still emits byte-identical `.snap`; parent `Weak`
  serializes as `none` like `ParentPtr` (already in spec §FIR-Impact — just reconfirm).

## Section F — Sequencer (Deepseek #8, mimo #3) — strongest agreement

Both reviewers independently reach the **same** conclusion and I fully agree: **keep a thin
`FirQueryable` adapter over ProtoBrane as the first-pass, prove the corpus green, and only then
attempt to retire the trait.** mimo quantifies it (~100 lines glue vs ~400 lines rewrite) and
tables the per-kind formatting; Deepseek lists the specific risks (`result=` position, search
fields, transparent rendering, concat `⨃`). The spec already permits this; **make it the default
plan, not the fallback.** Retiring `FirQueryable` becomes a *later, optional* cleanup, explicitly
out of the acceptance path.

## Section G — Woconstanic short-circuit (Deepseek #9) — Deepseek-only, real

Deepseek alone caught that `wo_short_circuit` (`fir.rs:1770-1789`) follows a Woconstanic chain
and collapses it, and that this is **sequencer-visible** (the short-circuited target differs from
the chain). Under the two-store model the result sits in `ubc_children`; whether the chain stays
visible or collapses changes output. **Action:** spec must state that Woconstanic short-circuit
behavior is preserved exactly (it's snapshot-visible), and how (the search copies the
short-circuited end value into its `ubc_children`, matching current collapse).

---

## Consolidated, de-duplicated action list (priority order)

**Must decide before any code (true blockers):**
1. **Composition model**: kinds *contain* a `ProtoBrane`; add the struct-embedding + `Fir` trait
   (`protobrane()/protobrane_mut()/fir_op_step()/kind()` + leaf accessors) to §1/§2. *(mimo #2)*
2. **`FirRef = Rc<RefCell<dyn Fir>>`** (trait-object, enum dispatch retired in favor of the new
   trait); state it and the `Weak<RefCell<dyn Fir>>` parent implication. *(Deepseek #7, mimo #5)*
3. **Compiler parent-wiring refactor** as a dedicated highest-risk sub-task (`Rc::new_cyclic`
   shell → thread `Weak` down → 12+ `ParentPtr::new()` sites). *(mimo #4)*

**Must add to the spec (correctness, even though not all snapshot-visible):**
4. Redefine outer-loop termination over **progress**, not root-`prev==new`; `step()` returns a
   progress signal; soften "retained unchanged." *(Deepseek #1)*
5. New **§9 NormalBrane & Statement stepping**: StatementFir-as-Fir (name/line leaf data +
   scope publish), incremental scope threading owned by the brane drain, re-step **rebuilds**
   the task list from `foolish_children`, and the **Econstanic-pop trap**. *(Deepseek #3/#4/#5,
   mimo scope/foolish_children)*
6. Demote "written once/uniform" → "shared default + enumerated overriders (Brane, SF, SFF)";
   document SF/SFF custom algorithm + `EvalContext` carry-over. *(Deepseek #6, mimo EvalContext)*
7. State **Woconstanic short-circuit is preserved** and how (snapshot-visible). *(Deepseek #9)*
8. State the **in-place-step vs clone-then-step equivalence** requirement + make it an early
   validation test. *(mimo #1c)*

**Must add to the plan (infrastructure):**
9. Genericize `Evaluator` (or thin adapter) for the cross-check harness. *(mimo #5)*
10. Re-express `has_unresolved_forward_refs` over the two stores. *(mimo)*
11. Make the **thin `FirQueryable` adapter the default sequencer path**; retiring the trait is a
    later optional cleanup off the acceptance path. *(Deepseek #8, mimo #3)*
12. Pin `bon` version; accept per-payload `updater` (macro later if needed). *(Deepseek #10, mimo)*
13. State the **finite-time-to-constanic guarantee explicitly** in the FOOP; reference UBC's
    `step_one`/`compute_brane_state` as the concrete NYES transition target. *(both, Open Qs)*

---

## Where I disagree with a reviewer

- **Deepseek Concerns 1 & 2 ("BLOCKING")**: downgraded. Both assume step counts are in
  snapshots; they are not (verified). The fixpoint *termination* point inside #1 survives as a
  correctness fix (action 4), but the "STEPS: 4 vs 37" framing is moot.
- **Deepseek #2 Option 1 ("change snapshot format / re-approve 131 snaps")**: reject — it would
  needlessly invalidate the signed corpus to solve a non-problem.
- **mimo "step counts… spec compliance"**: agree the spec *says* counts must match and that
  line should be **dropped/relaxed** (action 4 region) since counts aren't the oracle —
  byte-exact sequencer output is.

## Net assessment

Neither review changes the architecture; both improve its rigor. After folding actions 1–13 the
spec is implementable. The honest cost: the "uniform, written-once" story is slightly less pure
than the draft claimed (Brane/SF/SFF override), and the **compiler refactor + NormalBrane
section** are the real work the first draft under-weighted. The two BLOCKING items that sounded
scariest (Deepseek 1/2) are the least real once you know snapshots carry no step counts.

## Last Updated

**Date**: 2026-06-09
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.8
**Changes**: Initial synthesis of Deepseek + mimo reviews. Fact-checked the step-count-in-
snapshots claim (mimo correct, Deepseek incorrect), the Evaluator/FirRef type mismatch, and the
compiler bottom-up parent wiring. Produced a de-duplicated, priority-ordered 13-item action list
and flagged disagreements.
