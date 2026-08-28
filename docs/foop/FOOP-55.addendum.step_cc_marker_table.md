# SFM cross-tabulation: step-chain vs clone-chain

**Status:** confirmed correct by the human, 2026-08-27. This is a *reference*
document — it records how the machinery behaves today and why that behavior is
right. It is not a plan; nothing here is a checkbox.

Companion to FOOP-55 Phase 3J. Empirical probes live in this document's tables;
every row was measured against the tree at commit `e18c31dd`.

---

## The two axes

There is **one** kind of encounter: a `constanic_clone` walk arriving at an
SF (`<…>`) or SFF (`<<…>>`) node. "Step-chain" and "clone-chain" are
**adjectives describing the path that led to that moment** — they are not two
kinds of marks. Both axes ask the same question about different paths:

- **step-chain** — `step → step(crossed an SF/SFF) → … → step`, and then a
  clone begins.
- **clone-chain** — `clone → clone(crossed an SF/SFF) → … → clone`, i.e.
  already inside a recursive clone descent, with more cloning still below.

Conflating them is easy because the code joins them through a single scalar
(the strip budget), and because both were called "SFM".

## What each axis carries

| axis | carrier | distinguishes |
|---|---|---|
| **step-chain** | `Scope.has_ancestral_sfm` (`fir_trait.rs:679`) | **none vs SF.** `if this_kind == FirKind::StayFoolish` — SFF never sets it, deliberately. |
| **clone-chain** | `stay_budget: StripBudget` (`fir_kinds.rs:396`) | **how many strips remain on this path.** Decremented by each encounter. |

SFF is *not* missing from the step axis — it is recorded at **compile** time
instead, by `under_sff` (`compiler.rs:569`, `build_fir(*expr, …, true)`), which
makes every descendant search born `Econstanic`. That is the governing
principle in the code:

> **SF and UFM marks affect how STEPPING works; SFF achieves detachment from
> the environment during COMPILATION.**

## The table

Both axes: *did the path leading to this encounter cross an SF or an SFF?*

| # | step-chain crossed | clone-chain crossed | Behavior now (budget / NYES) | Verdict |
|---|---|---|---|---|
| 1 | none | none | Budget `Some(1)` from `OpInstructions::Normal` (`fir_kinds.rs:234`). At the encounter `spend()` → `(true, Some(0))`: mark **stripped**, descend into content. NYES via `transform_for_clone(false)`: Econstanic/Woconstanic → **Embryonic**, content re-steps in its new context. | **Correct.** |
| 2 | none | **SF** | The earlier encounter already decremented the budget to `Some(0)`, so this one gets `(false, …)` → `Rc::clone(fir_ref)`: mark **kept**, node shared by reference, NYES preserved verbatim. **The fact of the encounter decreasing the budget is itself the model** — no further tracking is needed. Measured: `{a=1; t=< <a> >; r=t; a=10; r;}` → `r=1`. | **Correct.** |
| 3 | none | **SFF** | Same mechanism as row 2. Measured: `{a=1; t=< <<a>> >; r=t; a=10; r;}` → `r=?(result=1, '^a$')` — distinct from row 2's `r=1`, because the mark arm descends into `ubc_children[0]` for SF (the resolved result) but `foolish_children[0]` for SFF (the written body), `fir_kinds.rs:414-434`. | **Correct.** |
| 4 | **SF** | none | Budget `Some(0)` from `OpInstructions::InsideSfm` (`fir_kinds.rs:235`, `fresh().spend().1`). The first encounter therefore refuses to strip: mark **kept**, subtree shared, no NYES transform. | **Correct — budget 0 is what we want here.** |
| 5 | **SF** | **SF** | Budget was already `Some(0)`; unchanged. | **Correct.** |
| 6 | **SF** | **SFF** | Budget was already `Some(0)`; unchanged. | **Correct.** |
| 7 | **SFF** | none | **No answer — `constanic_clone` is never called.** `has_ancestral_sfm` stays false (SFF does not set it), but the search never runs: `under_sff` made it born `Econstanic` at compile time (`compiler.rs:292-296`), so `handle_found` never fires and no clone is initiated. | **Not applicable.** |
| 8 | **SFF** | **SF** | Same as row 7 — no clone occurs, so the clone axis is never consulted. | **Not applicable.** |
| 9 | **SFF** | **SFF** | Same as row 7. | **Not applicable.** |

## Reading of the table

- **Rows 1–3** are the `Normal` family: one strip available, spent by the first
  encounter, every later encounter on the same path preserved. The budget
  decrement *is* the record of "this path already passed a mark"; nothing
  further needs tracking (human, 2026-08-27).
- **Rows 4–6** are the `InsideSfm` family: the budget starts already spent, so
  nothing on the path strips. This is correct — the enclosing SF has not yet
  decided the content's final position, so the copy must stay frozen.
- **Rows 7–9 have no answer.** SFF's compile-time detachment means no search
  runs, so no clone is initiated and neither axis is consulted. This is why
  `fir_trait.rs:679` deliberately excludes `StayFullyFoolish` from the step
  flag — adding it would double-count a deferral SFF already achieved.

Distinct behaviors therefore reduce to **three families**, not nine: strip-once
(1–3), never-strip (4–6), never-clone (7–9).

## Verified probes

Measured against `e18c31dd`, `foolish-cli run`:

| program | result | shows |
|---|---|---|
| `{a=1; t=a; r=t; a=10; r;}` | `r=?(result=1)` | row 1 |
| `{a=1; t=< <a> >; r=t; a=10; r;}` | `r=1` | row 2 |
| `{a=1; t=< <<a>> >; r=t; a=10; r;}` | `r=?(result=1,'^a$')` | row 3 |
| `{a=1; t=<< <<a>> >>; r=t; a=10; r;}` | outer stripped, inner survives unresolved, final → **10** | rows 1→2 in sequence; recoordination works |
| `{a=1; t=<<a>>; r=<t>; a=10; r;}` | `r=?(result=<<WOCONSTANIC ?a>>, '^t$')` | row 4 |
| `{a=1; t=<<a>>; r=<<t>>; a=10; r;}` | search never ran; final → **10** | rows 7–9 |
| `{a=1; t=<<a>>; r=<<<t>>>; a=10; r;}` | `r=?(result=1)` | UFM: unlimited budget, strips all, resolves early |
| `{a=1; t=<<a>>; r=< <<<t>>> >; a=10; r;}` | `r=?(result=1)` | UFM wins over an enclosing SF (`UfmFir` hardcodes `InsideUfm`, never reads the scope flag) |

## Known documentation defects (not behavior defects)

Recorded here because they are a plausible root of the conflation this document
resolves. The **behavior is correct**; only the comments mislead.

1. **`fir_kinds.rs:400-407`** claims `disable_nyes_reset` "becomes `true`" in
   the exhausted-budget mark arm. It does not — the branch does
   `return Rc::clone(fir_ref)` before any flag could be set. NYES preservation
   there is a consequence of **sharing the original node by reference**.
2. **`disable_nyes_reset` is dead in production.** It is hardcoded `false` at
   the public entry (`fir_kinds.rs:372`) and only ever forwarded; nothing
   assigns `true` outside unit tests. Verified by instrumenting both read
   sites (`:330`, `:540`) and running all failing einmo cases: **zero hits**.
   `Nyes::transform_for_clone(true)` is therefore unreached outside tests.
3. **`clone_stmt_result` (`fir_kinds.rs:1777-1794`) carries two contradictory
   doc paragraphs.** The first says the budget is decided "not by any ambient
   `Scope` flag"; the second says it carries `scope.has_ancestral_sfm`. The
   second and the code agree; the first is superseded and should be deleted.

Since rows 1–6 are confirmed correct as they stand, the honest fix for all
three is to **correct the comments to describe what the code does**, not to
make `disable_nyes_reset` live. A separate decision would be needed before
adding any new carrier to the clone axis.

## Also unresolved (unrelated to the table)

`fir_kinds.rs:573-581` — the Search arm's `chain_econstanic` re-enters the
**public** `constanic_clone` with a fresh `OpInstructions::Normal` budget,
while its sibling branch (`:583-590`) forwards the descending `stay_budget`.
One branch restarts the budget, the other continues it. Whether that asymmetry
is deliberate is not stated anywhere; unmeasured.

## Last Updated

**Date**: 2026-08-27
**Updated By**: Claude Code / claude-opus-5
**Changes**: Created. Records the step-chain vs clone-chain cross-tabulation
confirmed correct by the human on 2026-08-27: rows 1-3 (budget decrement is
itself the model, no further tracking needed), rows 4-6 (budget 0 correct),
rows 7-9 (no answer — `constanic_clone` is never called because SFF detaches at
compile time). Includes the verified probe table and the three documentation
defects found while establishing it.
