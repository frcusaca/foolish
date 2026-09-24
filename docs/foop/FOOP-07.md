---
foop: 70
title: Direct Access Search vs Search — miss outcomes stop keying on anchoring
author: Claude Code / claude-opus-5 (directed by the human)
status: Draft
type: Standards
created: 2026-09-24
phase: phase-2
supersedes: []
begun: [ ]
---

# FOOP-07: Direct Access Search vs Search — miss outcomes stop keying on anchoring

## Abstract

A search that produces no result today settles **NK if it was anchored** and **ECONSTANIC if it
was not**. This FOOP replaces that rule. The outcome will key on **what kind of search it is**
and **why it produced nothing**:

- **Direct Access Searches** (`.` and `#`, including `^`/`$`) assert the thing is *there*.
  Exhausting their scan is a **provable** absence → **NK**.
- **Searches** (`?`, `~`, and the value searches `?=`/`~=`) look for something that may arrive
  later. Exhausting their scan proves nothing → **ECONSTANIC** ("NOT FOUND").
- A scan that **never ran** — because the anchor is not yet a brane — is **waiting**, not
  absent → **WOCONSTANIC**. It is NK only when the anchor itself is NK.

Both remain **searches**, and both keep the name: the group is called **Direct Access Search**
(human, 2026-09-24 — *"we still call them Direct Access Search, still called search, just
slightly different semantics"*). The distinction is a marker on the search, not a new FIR kind.

**This is what finally separates `.` from `?`.** They are aliases today, in the strongest
possible sense — see §2.

## Motivation

### §1 The rule that is wrong

`fvm_storage.rs` settles an exhausted scan with:

```rust
ScanOutcome::Miss => {
    let settle = if anchored { Nyes::Nk } else { Nyes::Econstanic };
```

AGENTS.md §"NK vs ECONSTANIC miss outcomes" states the same rule, with this justification:

> **Anchored miss → NK.** A contextless anchored search (`a?name`) that finds nothing settles NK
> (the name is provably not in that brane).

The justification does not hold for a name search. `a?name` finding nothing proves the name is
not in that brane **right now**; recoordination can bring one in later, which is precisely what
ECONSTANIC exists to express. The reasoning IS sound for `a#99` — a brane's size is known, so
exhaustion is proof. The rule attached the right conclusion to the wrong predicate.

### §2 `.` and `?` are currently the same operator

`Astn::DotSearch` builds:

```rust
FirSpec::Search { pattern: format!("^{coordinate}$"), anchored: true, forward: false,
                  is_value_search: false, contexted: false }
```

which is byte-identical to what `?` builds. Nothing downstream can tell them apart, and the
sequencer therefore renders `g.nope` back as `g?nope` — observable today. AGENTS.md's Searches
table says as much: "`.` backward anchored name (alias for `?`)".

Under this FOOP they stop being aliases: they differ exactly on miss outcome. The renderer must
then distinguish them too, or it will show a reader a `?` whose NK they would misjudge (§4.3).

### §3 A scan that never ran is not an absence

Measured on `jia` at `b244c167`:

| Program | anchor NYES | body NYES today | correct |
|---|---|---|---|
| `{hit = nothere.x;}` | Econstanic | **Nk** | WOCONSTANIC |
| `{hit = later.x; later = {x=42};}` | Econstanic | **Nk** | WOCONSTANIC |
| `{g = {x=1}; hit = g.nope;}` | Constant | Nk | NK (correct) |
| `{g = {x=1}; hit = g#99;}` | Constant | Nk | NK (correct) |

Rows 1–2 are a bug this FOOP fixes, raised by the human (2026-09-24): *"direct access still can
reach woconstanic if anchor is constanic"*. The anchor never resolved to a brane, so no scan was
exhausted and nothing was proven absent — yet the result is frozen NK. Row 2 is the sharper
case: `later` is defined on the very next line, so the value is plainly reachable under
recoordination, and NK forecloses it.

## Specification

### §4.1 The two groups

| Group | Operators | Miss semantics |
|---|---|---|
| **Direct Access Search** | `.` `#N` `^` `$` (and `&#`, `&^`, `&$`) | provable absence → **NK** |
| **Search** | `?` `~` `?=` `~=` (and `&?`, `&~`, `&?=`, `&~=`) | not found → **ECONSTANIC** |

**Value searches are Searches, not Direct Access** (human, 2026-09-24). A value that is absent
now may be present after recoordination, exactly as a name may.

`^`/`$` already compile to `FirSpec::Index`, so they inherit Direct Access by construction —
consistent, since `{}^` is as provably empty as `{}#99`.

### §4.2 The settle rule

Replaces the `if anchored` rule at every miss site. In order:

| Anchor state | Scan | Outcome |
|---|---|---|
| NK | never ran | **NK**, reason *"due to NK anchor"* |
| not constanic, or constanic but not a brane | never ran | **WOCONSTANIC** — still waiting |
| resolved brane, scan exhausted | ran | Direct Access → **NK**; Search → **ECONSTANIC** |

`ScanOutcome::NkStop` (the scan hit an NK candidate) keeps its current NK outcome.

**Unanchored searches are unaffected** — they already settle ECONSTANIC on a miss, which §4.1
now derives from the group rather than from anchoring.

### §4.3 Rendering

Three distinct reasons replace today's two generic strings:

| Situation | Annotation |
|---|---|
| anchor is NK | `!! NK: due to NK anchor.` |
| Direct Access exhausted | `!! NK: <name> is not in <anchor>` (wording TBD in the plan) |
| Search exhausted | ECONSTANIC — rendered per FOOP-36's existing ECONSTANIC treatment |

Today both of the first two print `!! NK: anchored search found no match`, which is **false** for
the NK-anchor case: the name *was* there, the brane was refused (human, 2026-09-23 — *"should
probably say `!! NK: due to NK anchor.`"*).

**`.` must render as `.`.** Since the two operators now differ in meaning, the reverted form must
show which one was written.

### §4.4 Implementation sketch

1. Add `direct_access: bool` to `FirSpec::Search`; `FirSpec::Index` is direct access by
   definition. Set it in `build_fir`'s `Astn::DotSearch` arm.
2. Replace the `if anchored` settle with §4.2's three-case rule.
3. Teach the sequencer to render `.` and to emit §4.3's reasons.

## FIR Impact

`FirSpec::Search` gains one field. No new FIR kind — Direct Access Searches remain searches.

## UBC Step Impact

Miss settlement changes at every search step site. Some programs that froze NK will now settle
WOCONSTANIC or ECONSTANIC and may **gain a value under recoordination** where they previously
could not — this changes evaluation results, not only annotations.

## Test Plan

Unit tests AND einmo cases for every row of §4.1 and §4.2, per the standing dual-testing
requirement:

- each Direct Access operator exhausted against a resolved brane → NK
- each Search operator exhausted against a resolved brane → ECONSTANIC
- every operator against an NK anchor → NK with the NK-anchor reason
- every operator against an unresolved anchor → WOCONSTANIC
- the §3 row-2 case (`{hit = later.x; later = {x=42};}`) resolving under recoordination
- `.` round-trips as `.`, not as `?`

Expect **broad einmo OUTPUT movement**: every `.`-written case re-renders, and every exhausted
`?`/`~` changes NYES. Each moved case needs its own Promotion Review Gate entry.

## Rejected Alternatives

- **Reinterpret `anchored`.** `g?nope` is also anchored, so the flag cannot carry the meaning.
- **Make `.` a separate FIR kind.** Rejected by the human: they are still searches, and a marker
  expresses "same thing, different miss semantics" more honestly than a parallel kind.
- **Leave rendering alone.** Untenable once the operators differ: printing `?` for a written `.`
  would misrepresent why the result is NK.

## Open Questions

- Exact wording for the Direct-Access-exhausted annotation (§4.3).
- Whether `&`-contexted forms inherit their base operator's group in every case. §4.1 assumes
  yes; the plan should verify each against a test.
- Whether any existing einmo case depends on an anchored name-miss being NK in a way that
  indicates a genuine design conflict rather than a baseline to re-review.

## References

- AGENTS.md §"NK vs ECONSTANIC miss outcomes" — the rule this FOOP replaces.
- AGENTS.md §"Searches (FOOP-23)" — the table calling `.` an alias for `?`.
- FOOP-23 — the one-engine model (`CursorSource`, `SearchPredicate`, `ScanOutcome`).
- FOOP-94 — brane NK became a run-time-error record; this FOOP does the analogous
  narrowing for SEARCH NK.

## Last Updated

**Date**: 2026-09-24
**Updated By**: Claude Code / claude-opus-5
**Changes**: Created. Specifies the split between Direct Access Searches (`.`, `#`, `^`, `$` —
exhausted scan is a provable absence → NK) and Searches (`?`, `~`, `?=`, `~=` — exhausted scan
proves nothing → ECONSTANIC), replacing the current anchored-vs-unanchored rule. Adds the
never-ran case: an anchor that is not yet a brane leaves the search WOCONSTANIC rather than
frozen NK, fixing a measured bug where `{hit = later.x; later = {x=42};}` forecloses a value
that recoordination would supply. Records that `.` and `?` are presently the SAME FIR node, so
this is the change that makes them genuinely distinct operators, and that rendering must
therefore preserve `.`. All rulings in this draft are the human's, 2026-09-23/24.
