---
foop: D63
title: A Foolish-rendering sequencer for foolish-ubca2 — output that parses back in
author: Claude Code / claude-opus-5 (directed by the human)
status: Draft
type: Standards
created: 2026-09-01
phase: phase-4
supersedes: []
begun: [x]
---

# FOOP-36: A Foolish-rendering sequencer for `foolish-ubca2`

FOOP numbering is little-endian; the full rules live in `foop.md` at the repository root —
**read it before creating or editing a FOOP.** The `foop:` front-matter field here is the
big-endian sort key preceded by `D` (`foop: D63`, file `FOOP-36.md`, following FOOP-26's 62).

## Abstract

This FOOP gives `foolish-ubca2` **its own sequencer**, owned by the crate, whose default
`Foolish` mode renders a FIR — constanic or mid-evaluation — as **valid Foolish source that
parses back in**. The goal is that a Foolisher can write an einmo case's expected OUTPUT from
the specification, without running the evaluator.

One rule does most of the work:

> **When a result is an *inconclusive constanic* — WOCONSTANIC, ECONSTANIC or NK, i.e. settled
> without reaching a value (§0) — render the original expression.** A search renders as the
> search; an operator renders as the op on its parameters. Only a **conclusive** result
> (CONSTANT or INDEPENDENT) collapses to its value.

So `{a=1+2}` renders `a = 3`, while `r = b?a.*` stays `r = b?a.*` unless a real value was
produced. Nothing is withheld by that: handed the search, the next compiler re-coordinates it
itself, which is exactly what ECONSTANIC promises.

The rest follows from the same principle:

- **NK is inconclusive**, so it reverts too — `1/0` renders `1/0`, with `!! NK: …` beside it
  (§5). The only `???` in output is where the Foolisher wrote one.
- **States with no Foolish syntax** — PREMBRYONIC, EMBRYONIC, BRANING, ECONSTANIC,
  WOCONSTANIC — are named in `!!` comments, which the parser discards (§4).
- **Pre-constanic FIR still renders**, because einmo debugs as well as approves: a half-stepped
  program renders as the program it still is, annotated with how far each part has got (§2.1).
- **Line width is configurable, defaulting to 108** (§4.1); whether NK carries its reason is a
  flag (§5).

`Foolish` mode emits no FIR machinery: no `?(…)`, no `Op…(`, no `pattern='^…$'`, no
`ANCHORED`/`UNANCHORED`, no bare NYES tokens. The existing detailed rendering is neither
removed nor changed — it becomes `Detailed`, a second explicitly-selected mode that delegates
unchanged to `foolish_core::FirSequencer`, so `foolish-ubca` cannot regress by construction
(§1, §6). §7 sets old and new side by side.

The approval suite is **replaced rather than edited**: a new `einmo_suite2` receives the 179
inputs, renders them under the new sequencer, and becomes what `cargo test` exercises.
`einmo_suite` is left frozen and still passing as the reference to diff against, and this FOOP
does everything except remove it.

## Motivation

### What is wanted

**A Foolisher should be able to write an einmo case's expected OUTPUT from the specification,
without running the evaluator.** That is the whole goal, and everything in this FOOP follows
from it.

An einmo case is two halves. The INPUT is Foolish, so anyone who knows the language can write
it. The OUTPUT should be Foolish too — the same program, evaluated: values where values were
reached, the original expressions where they were not, and the evaluator's findings in
comments. Given that, the expected OUTPUT is *derivable*, and a reviewer can check a baseline
by reading it against the language spec.

This matters beyond convenience. AGENTS.md and `foop.md` require every promoted OUTPUT line to
be **justified against the specification**, and explicitly forbid "it matches what the
evaluator printed" as a justification. An OUTPUT written in FIR-internal vocabulary can only
be checked the forbidden way. An OUTPUT written in Foolish can be checked the required way.

So the target is:

```text
misc/undeclared_identifier.foo    INPUT: {x = non_existent;}
{
  x = nonˍexistent  !! ECONSTANIC (unfound)
}
```

```text
misc/sff_resolves_on_each_use.foo INPUT: {a=1; b=2; s=<<a+b>>; a=10; s;}
{
  a = 1;
  b = 2;
  s = <<a + b>>;
  a = 10;
  12
}
```

Both are *the program a Foolisher would write to mean the same thing*. In the first, the
search is rendered rather than its outcome: an unanchored miss is ECONSTANIC, not NK — it may
still gain a value by recoordination (FOOP-23) — so `x` is emphatically not `???`, and
rendering the search is what lets the next compiler discover that for itself. In the second,
`s = <<a+b>>` still holds a deferred sum and `a` is re-stated as 10; a reader who knows the SFF
rules predicts every line without running anything.

§7 sets these against what the same cases render as today, in one place.

### The suite is replaced, not edited

**`einmo_suite2` is built to replace `einmo_suite`.** It is not a scratch pad for developing
the renderer and it is not a parallel experiment — it is the readable, editable suite that
`foolish-ubca2`'s approval testing moves to, and by the end of this FOOP it is what
`cargo test` exercises.

The replacement is done by **copying inputs across and rendering them anew**, rather than by
re-rendering `einmo_suite` in place. Three things follow, and each is a reason to prefer it:

1. **Both renderings stay on disk while the 179 cases are reviewed.** Judging a changed
   baseline means asking "is this the same program, said in Foolish?" — which needs the old
   output to hand. Migrating in place would delete the very thing the reviewer needs.
2. **`einmo_suite`'s `verified/` tier is never disturbed.** It holds 179 human-signed
   artifacts and its gate passes today; leaving the suite frozen means that tier stays valid
   and no re-attestation is forced by this FOOP. The tier that needs human attention instead is
   `einmo_suite2`'s own `verified/`, which starts empty — a new-suite question rather than a
   broken-tier one.
3. **The tree is green throughout.** `einmo_suite`'s gates keep passing on the old rendering
   from first commit to merge; `einmo_suite2`'s gates go green as its baselines are reviewed
   and promoted. There is no window in which `foolish-ubca2` has no working approval suite.

**The cut-over happens at the end of the project.** `cargo test` points at `einmo_suite` for
the whole of Movements I and II; only in Movement III, once `einmo_suite2` holds every input
and every baseline has been reviewed, does the crate's approval testing switch over. Up to that
moment the development procedure is exactly what it is today — same gates, same commands, same
green tree.

**This FOOP does everything except remove `einmo_suite`.** Retiring it is a separate act for
the human to authorize, once `einmo_suite2` has been trusted for a while — and until then the
old suite is useful precisely as the frozen reference that makes the new one auditable.

### A second benefit: the corpus checks itself

Making OUTPUT valid Foolish gives the suite a free invariant: **every case's OUTPUT is a valid
INPUT.** The whole corpus can be re-fed to the evaluator as a self-check (§2, §Test Plan T3),
and a rendering that is locally plausible but globally unparseable is caught mechanically
rather than by eye.

### Why `foolish-ubca2` should own its sequencer

`FirSequencer` lives in `foolish-core` and is shared by `foolish-ubca` and `foolish-ubca2`,
which lib.rs describes as "two **independent implementations of the same Foolish evaluator**."
The sequencer is the one place that independence leaks: a rendering change wanted by one crate
is forced through a module the other depends on, and every ubca baseline moves with it.

`foolish-ubca2` is the implementation intended to replace `foolish-ubca`. Giving it its own
sequencer, in its own crate, is therefore not duplication for its own sake — it is the
rendering half of the same replacement, and it lets ubca2's output evolve (this FOOP, and
whatever FOOP-26's three-beat step needs) **without touching a single `foolish-ubca` baseline**.
That property is worth a great deal on its own: it means this FOOP cannot cause a
non-regression violation in the sibling crate, because it does not modify code the sibling
compiles.

### Why this should land before FOOP-26

FOOP-26 changes what programs *mean* on `foolish-ubca2` — mark semantics, concatenation as an
operator, the three-beat step. Every one of those changes will move einmo baselines, and each
moved baseline has to be justified line by line at the Promotion Review Gate.

Doing FOOP-36 first makes that work materially easier, in three ways:

1. **The diffs become readable.** A FOOP-26 baseline change would currently show up as one
   `?(pattern='^a$', UNANCHORED, ECONSTANIC)` becoming another — a diff in FIR-internal
   vocabulary, where the reviewer must decode both sides before judging either. After FOOP-36
   the same change shows up as Foolish source changing, which is the language the FOOP-26 spec
   is written in.
2. **The reviewer can predict the answer.** FOOP-26's §2 states mark rules as language
   semantics. With Foolish-rendered output, a reviewer reads those rules and writes down the
   expected OUTPUT *before* running anything — which is what the Promotion Review Gate has
   always asked for and what the current rendering makes impractical.
3. **The two FOOPs stop competing for the same lines.** FOOP-36 rewrites every ubca2 baseline
   once, for rendering reasons only, with semantics held fixed. FOOP-26 then rewrites only the
   ones whose *meaning* actually changed. Interleaved, the two effects land in the same diff
   and neither can be reviewed cleanly.

The ordering costs FOOP-26 nothing: FOOP-36 changes no FIR, no step rule, and no step count
(§FIR Impact, §UBC Step Impact), so nothing FOOP-26 builds on moves under it. §Open Questions
Q4 remains — the human confirms the sequencing.

## Specification

### §0 Terminology: conclusive and inconclusive

These definitions are **AGENTS.md §Foolish Terminology**, restated here because §3's rendering
rule is written in them. AGENTS.md is authoritative; if this section and it ever disagree,
AGENTS.md wins.

- **Constanic** (say "cons-TAN-nic") — Constant in Context. Any terminal NYES state:
  ECONSTANIC, WOCONSTANIC, CONSTANT, INDEPENDENT, NK.
  *Pre-constanic* (nigh) = PREMBRYONIC, EMBRYONIC, BRANING — needs more stepping.
- **Constantew** — CONSTANT EveryWhere. A FIR that won't change no matter what: CONSTANT,
  INDEPENDENT, NK. Constantew ⊂ constanic. A **non-constantew constanic** (ECONSTANIC,
  WOCONSTANIC) may gain a value when context is recoordinated.
- **Conclusive** (shorthand **Conc**) — a FIR whose NYES is CONSTANT or INDEPENDENT: it reached
  a value. (Predicate: `NyesExt::is_conclusive()`, added by this FOOP — §0.1.2. The
  pre-constanic group's predicate already exists under the older name **`is_nye()`**.) **Inconclusive** is everything else — **all other pre-constanic and constanic
  states**. The often-used phrase **"inconclusive constanic"** narrows that to the terminal
  ones: **WOCONSTANIC, ECONSTANIC, NK**.

Note the two cuts are different, and differ exactly on **NK**: NK is constantew (nothing will
change it) yet inconclusive (it never produced a value). **Rendering keys on conclusive;
recoordination keys on constantew.** That is why NK renders as the original expression rather
than as a value — see §5.

Note also that *inconclusive* alone spans pre-constanic states, while *inconclusive constanic*
excludes them. §3's rule is stated over the narrower phrase, because a FIR's **result** is what
it tests and a result under inspection is constanic; §2.1 covers pre-constanic FIR separately,
and it renders the same way for the same reason — no value was reached.

#### §0.1 Survey: what "settled" meant in `foolish-ubca2` before FOOP-56

Before FOOP-56, `foolish-ubca2` used "settled" heavily — **134 lines, 131 of them in
`fvm_storage.rs`** — as informal prose, never as a callable predicate. **`is_settled()` does
not exist**: FOOP-62 §Terminology specifies it, but the implemented `NyesExt` predicates are
`is_preconstanic()` (and its `is_nye()` alias), `is_constanic()`, `is_constantew()`, and
`is_conclusive()`. Anything calling `is_settled()` would not compile.

The prior uses were **not consistent**. Each meant one of the groups from §0, and they differ.
They are classified below by what the code required:

**Group 1 — "settled" ≡ constanic.** The gate is literally `is_constanic()`.

| Site | Line | What it requires |
|---|---|---|
| `FirPointer::settled_constanic_result` | 639 | `is_constanic()` on the OWNER — but see §0.1.1: what the slot can hold is narrower |
| `FirCursor::settled_constanic_result` | 1602 | delegates to the above |
| `step_to_constanic` | 3272 | loops until `is_constanic()`; the error path re-tests the same |
| duplicate-definition compare | 2589 | "prior definition not yet settled" = `!is_constanic()` |
| conflicting-redefinition compare | 2702 | "one side not yet settled" = either `!is_constanic()` |
| `anchor_constanic` | 3178 | `is_constanic()` on the anchor |

**Group 2 — "settled" ≡ conclusive.** The gate is `Constant | Independent`, i.e. §0's
*conclusive*. This is the site that would be wrong if read as "constanic".

| Site | Line | What it requires |
|---|---|---|
| `all_foolish_children_conclusive` (Operator) | 816–819 | `is_conclusive()` — an operator queues its operands as tasks unless every one is **conclusive**. An ECONSTANIC operand is constanic but NOT enough. |
| `operator_pushes_tasks_for_econstanic_operand` (test) | 5333 | proves the distinction: an ECONSTANIC operand is constanic yet still queued because it is not conclusive. |

##### §0.1.1 `settled_constanic_result` means **constanic**

The gate at line 639 tests `is_constanic()` on the node owning the slot, and that is also the
right description of the slot's contents. Two mechanisms push toward something narrower, but
neither closes the door:

- **`Nyes::transform_for_clone`** (`foolish-core/src/fir.rs`) preserves only CONSTANT,
  INDEPENDENT and NK — exactly **constantew** — turning ECONSTANIC, WOCONSTANIC and every
  pre-constanic state into EMBRYONIC. A result arriving by `clone_stmt_result` →
  `revive_constanic` is therefore constantew or embryonic.
- **`push_ubc_child`** (line 151) queues a non-constanic child as a task, so an embryonic entry
  gets stepped onward rather than lingering.

**But ECONSTANIC and WOCONSTANIC do reach the slot**, by a route that bypasses cloning:
`StayFoolish` "expose[s] EXPR'S OWN resolved value … adopting that value's `Nyes`" (line
902–904), and the write sites at 932 and 970 pass a found value's NYES straight through. Of the
~20 slot writes, several are `Nyes::Nk`, one is `Nyes::Constant` (1527), and the remainder
carry whatever the found value had.

**So the accurate qualifier is `constanic`, not `constantew` or `conclusive`.** FOOP-56 names
the method **`settled_constanic_result`**, stating the gate it applies.
`settled_constantew_result` would be wrong — it would promise something the StayFoolish path
does not deliver.

**Consequence for §3.** All three arms of the predicate are genuinely reachable: conclusive
results (shared or preserved CONSTANT/INDEPENDENT), NK results (many write sites), and
ECONSTANIC/WOCONSTANIC results (via SF and the found-value paths). No arm is dead code, and the
`einmo_suite2` cases must cover each. Phase 1 confirms the distribution empirically.

##### §0.1.2 The predicates — see FOOP-56

§0's four groups each have a predicate on `foolish-ubca2`'s `NyesExt`:

| Group | Predicate |
|---|---|
| Pre-constanic | `is_preconstanic()`, with **`is_nye()`** as an alias |
| Constanic | `is_constanic()` |
| Constantew | `is_constantew()` |
| Conclusive | `is_conclusive()` |

**[FOOP-56](FOOP-56.md) added `is_preconstanic()` and `is_conclusive()` before this FOOP.** It
also replaced the five hand-rolled
`matches!(nyes, Nyes::Constant | Nyes::Independent)` conclusive tests in `fvm_storage.rs`
(lines 818, 2007, 3739, 3810, 3950) with named calls, and qualifies every bare "settled" with
its group — the survey in §0.1 above is what it works from.

This FOOP's renderer should **use those predicates** rather than hand-rolling state lists.

**Group 3 — "settled" = the outcome of a classification, spanning several groups.** Here
"settled" names *the state being computed*, not a test.

| Site | Line | What it computes |
|---|---|---|
| `constanic_nyes = nyes_from_found(...)` | 968 | maps a found result's NYES: ECONSTANIC/WOCONSTANIC → WOCONSTANIC, CONSTANT/INDEPENDENT → CONSTANT, NK → NK. Output is constanic; the input need not be. |
| `let decided_nyes = ...decide_nyes_due_to_children(...)` | 1070 | classifies a Braning node from its children — may yield **Braning** (still pre-constanic!) when a child is pre-constanic. |

**Group 4 — prose in doc comments and test names.** The remaining ~120 occurrences. Mostly
accurate but imprecise; several would read better as "constanic" or "conclusive". Notable:
`indep_int_stepping_already_conclusive_is_noop` (5209) means CONSTANT/INDEPENDENT — conclusive;
`revive_constanic_unwraps_stay_foolish_to_its_settled_constanic_result` (5623) means constanic, via
`settled_constanic_result`.

**What this FOOP does about it.** "Settled" is the word every agent reached for and it is
staying. The problem is not the word but that it is used bare, leaving the reader to work out
which group is meant. The remedy is a **descriptor**, not a replacement:

| Before FOOP-56 | Implemented name | Why |
|---|---|---|
| `FirPointer::settled_result` (639) | **`settled_constanic_result`** | it gates on `is_constanic()`, and per §0.1.1 that is genuinely what the slot holds — `constantew` would over-promise |
| `FirCursor::settled_result` (1602) | **`settled_constanic_result`** | delegates to the above |
| `all_settled` (816) | **`all_foolish_children_conclusive`** | a local `bool` (not a function — see below). It iterates `storage.foolish_children(ptr)` and gates on `is_conclusive()`. |
| `step_to_settled` (3272) | **`step_to_constanic`** | it loops until `is_constanic()` |

**`all_foolish_children_conclusive` is a local `bool`, not a predicate function.** It is
`let all_foolish_children_conclusive = children.iter().all(…)`, consumed by its following `if` —
there is nothing to call, so a verb form like `are_all_settled()` does not apply. As a boolean
binding the idiomatic Rust name is a noun phrase, hence `all_foolish_children_conclusive`.

**[FOOP-56](FOOP-56.md) made these renames**, added the two missing predicates (§0.1.2), and
corrected `lib.rs`'s stale `is_settled()` claim. It landed before this FOOP so the vocabulary
exists in the code this FOOP describes.

§3's rule is stated over *conclusive* and *inconclusive constanic* — each naming exactly one
group.

### §1 Two modes, one entry point

`foolish-ubca2` gains a module `src/sequencer.rs` exposing:

```rust
/// How a FIR is rendered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SequenceMode {
    /// FOOP-36: valid Foolish source that parses back to an equivalent FIR.
    /// Internal state appears only inside `!!` comments.
    #[default]
    Foolish,
    /// The FIR-internal rendering: NYES tokens, `Op…(`, `?(pattern=…)`.
    /// Unchanged from `foolish_core::FirSequencer` — for debugging.
    Detailed,
}

pub struct Ubca2Sequencer;

impl Ubca2Sequencer {
    pub fn format(storage: &FVMStorage, fir: FirPointer, mode: SequenceMode) -> String;
}
```

`Foolish` reads `foolish-ubca2`'s arena FIR directly, through `FVMStorage`, `FirPointer`, and
`FirCursor`. That is the evaluator's complete representation: a search still carries its
`pattern`, `anchored`, `forward`, `is_value_search`, and `contexted` fields, its written anchor
and value operand remain in `foolish_children`, and any produced value is in `ubc_children`.
Rendering before `proto_to_core_fir` is load-bearing because that compatibility conversion
does not preserve every surface-relevant arena field.

`Detailed` converts the selected arena node with `proto_to_core_fir`, then **delegates to
`foolish_core::FirSequencer::format`**. It is not a reimplementation and is not permitted to
drift. This keeps existing debugging output byte-compatible without making the new Foolish
renderer depend on the lossy compatibility representation.

`Foolish` is the new renderer, specified below. It is the default because it is what the einmo
adapter and the REPL should show; a caller that wants internals asks for them by name.

### §2 The round-trip property (the definition of correct)

For any input program `P` that `UbcaEvaluator` settles in its arena, let
`R = format(storage, program_fir, Foolish)`.
Then:

1. **`R` lexes and parses.** `foolish_parser` accepts `R` with no error.
2. **`R` is idempotent under re-evaluation:** `format(eval(R), Foolish) == R`. Operationally:
   compile `P`, step to finish, output `R1`; then compile `R1`, step to finish, output `R2` —
   and `R2 == R1`. §Test Plan T2 states the six steps as a test writes them.

Property 2, not "R evaluates to the same FIR", is the testable contract. It is strictly
weaker than semantic identity and strictly stronger than "it parses" — and it is the property
that actually matters for a snapshot baseline, because it says the rendering has reached a
fixed point with nothing left to lose. §Rejected Alternatives F records why the stronger
property (semantic identity of the re-parsed FIR) is not demanded.

This gives the suite a **new, free, general invariant**: every einmo case's own OUTPUT is a
valid INPUT, so the whole corpus can be re-fed to the evaluator as a self-check. §Test Plan
§T3 makes that a test.

#### §2.1 Property 1 holds for pre-constanic FIR; Property 2 does not

**`Foolish` mode must render a FIR in ANY state, constanic or pre-constanic.** Einmo is a debugging tool
as well as an approval tool — a case may be captured mid-evaluation (a step budget, an
`ALARM:` that halts stepping, a deliberately-stepped snapshot), and when it is, the OUTPUT
still has to be something a Foolisher can read and, per this FOOP's whole purpose, could have
typed. A renderer that only handles CONSTANT would fall back to the debug dump exactly when
the reader most needs help.

So the contract splits:

| | PREMBRYONIC / EMBRYONIC / BRANING | ECONSTANIC / WOCONSTANIC | CONSTANT / INDEPENDENT / NK |
|---|---|---|---|
| **Property 1** (parses) | **required** | **required** | **required** |
| **Property 2** (idempotent) | not required | required | required |

Property 1 is universal: whatever state a FIR is in, its rendering is valid Foolish. That is
the invariant §T3 checks over the whole corpus and the one this FOOP is really about.

Property 2 is required only of **constanic** FIR (all five terminal states — `Nyes::is_constanic`).
A pre-constanic FIR is *by definition* mid-flight: re-evaluating its rendering resumes the
evaluation and legitimately produces something further along. Demanding a fixed point there
would be demanding that stepping do nothing. The written form of a pre-constanic node is
still its source form, so re-parsing it is meaningful — it just is not stationary.

A pre-constanic node therefore renders exactly as every other node does under §3 — its
written form — with the state in a `!!` comment per §4. Stepping changes the comments, not
the source text. The comment is what carries `BRANING` vs `EMBRYONIC`, information that has
no Foolish syntax and must not invent any.

### §3 What each FIR kind renders as

The rule in one sentence: **a conclusive result renders as its value; an inconclusive constanic
result renders as the original expression.**

**Constants always render in Foolish.** An integer renders `7`, a merged brane renders its
statements. Wherever a genuine value exists, the value is the Foolish.

**The same predicate governs operators and searches alike.** Both are *processes* with a
result, and both collapse to that result only when it is a genuine value:

> **When a result is an inconclusive constanic, render the original expression.** For a search
> that is the original search; for an operator it is the op operating on its parameters. A
> missing result reads the same way. Otherwise — the result is **conclusive** — render the
> result's value.

So `3 + 4` renders `7` (conclusive result), but `a + b` over ECONSTANIC operands renders
`a + b` — the op on its parameters — because it never reached a value. This is one rule with
two instances, not two rules; the sections below spell out each.

**When a search result is an inconclusive constanic, render the original search.** The test is
on the search's `result()`, not on the search's own NYES:

> Render the **original search** when `result()` is **not found**, or is an **inconclusive
> constanic** (ECONSTANIC, WOCONSTANIC, NK). Otherwise — the result is **conclusive** —
> render the result's value.

The principle: a search collapses to a value only when it actually produced one. ECONSTANIC and
WOCONSTANIC are "constanic, but not into a value, and context may still change them"; NK is "no
value, and none is coming". In none of those is there a value to print, so the question that
was asked is what gets printed.

Anchoring decides the *shape* of the rendered search, not whether it renders:

- **Unanchored** → the search alone: `?x`, `nonexistent`.
- **Anchored** → the anchor, rendered by these same rules, then the search: `b?a.*`, `a.field`.

So `r = b?a.*` renders `r = b?a.*` when its result is an inconclusive constanic, and `r = 3`
when the result is conclusive.

**Why this is right.** A search reverting to its written form is not information withheld from
the reader — it is the program restated. Handed `b?a.*`, the next compiler performs the search
itself; handed `nonexistent`, it may resolve it in a *new* context, which is exactly what
ECONSTANIC promises (§5.1: ECONSTANIC and WOCONSTANIC are **non-constantew** — their values may
change under recoordination). Collapsing those to a value would freeze an answer the language
says is still open. A rendered search is never a search that was lost; it is one that will be
re-coordinated wherever the output is next read.

`Foolish` mode emits **no FIR machinery at all**: no `?(…)` wrapper, no `pattern='^…$'` regex
spelling, no `ANCHORED`/`UNANCHORED`, no bare NYES token, no `Op…(`, no `result=`. Those are
FIR-internal vocabulary that no Foolisher would write. §7 shows them side by side with what
replaces them.

| FIR kind | `Foolish` rendering | Note |
|---|---|---|
| ConstantInt | `7` | unchanged |
| Creation | `⬤`, or its original name (`'True`) | FOOP-33 rule, unchanged |
| Brane | `{ … }` with `;`-separated statements | opener carries **no** state token |
| Statement | `name = body` | `ˍ` (U+02CD) is a valid ident char (lexer `is_id_sep`), so mangled names round-trip |
| Characterized brane | `a'b'{ … }` | unchanged from current `chars` handling |
| Operator, **conclusive** result | its **value** (`3 + 4` → `7`) | the operator is spent |
| Operator, **inconclusive constanic** result (or none) | the **op on its parameters** (`a + b`, `1/0`) | same predicate as Search; §0 |
| Search, **conclusive** result | the **result's value** | the search produced a value |
| Search, **inconclusive constanic** result (or none) | the **original search** — unanchored `?x`; anchored `b?a.*` (anchor, then search) | §0, §3.1 |
| Index (`#N` / `^` / `$`) | conclusive result → its value; else the index with its anchor (`#-1`, `b#0`, `^`, `$`) | as Search |
| **NK, any kind** | the written expression + `!! NK: …` comment | §5 — NK is inconclusive, so it reverts |
| `???` written in source | `???` | the no-no literal is Foolish; source renders as itself |
| SF `<X>` / SFF `<<X>>` | `<` / `<<` + interior rendered in this same mode + `>` / `>>` | interior is written form, never a NYES dump |
| Concatenation, **merged** | the merged brane `{…}` | the concatenation succeeded; §3.2 |
| Concatenation, **unmerged** | `A B` — each constituent rendered recursively | §3.2; `⨃` is never emitted (not input syntax) |
| **Any kind, pre-constanic** | its written form + `!!` state comment | §2.1, §4 |

The `=$` attached-search spelling (FOOP-75 §4) is retained where it is the canonical *input*
form, since it is Foolish. `=^`/`=$` render as `name =$ value`.

#### §3.1 A rendered search is never a lost search

The rule that a search reverts to its written form — whenever its result is an inconclusive
constanic — can look like it discards an answer the evaluator worked for. It does not, and the
reason is worth stating plainly because it is what makes §2's round-trip meaningful.

A search rendered as a search is one that will be **re-coordinated wherever the output is next
read**. Handed `b?a.*`, the next compiler runs the search against `b` and settles it constanic
exactly as this one did. Handed `nonexistent` — an unanchored miss, ECONSTANIC — it may resolve
it in a *new* context, which is precisely the promise ECONSTANIC makes (FOOP-23: an unanchored
miss may gain a value by recoordination). So the printed program is not a weaker statement than
the FIR; it is the same statement, in the language.

This is also why the rule keys on the **result's** state rather than on the search's own. A
search whose result is ECONSTANIC has *found* something — a statement — but that statement is
inconclusive, so there is nothing to print but the question. The result chain
behind a search, however deep, is the evaluator's working and none of it is printed.

What this rule does NOT do is convert values into searches. A constant that no search produced
renders as its value: `3 + 4` is `7`, a merged brane is its statements, an integer is itself.
Only a search renders as a search.

#### §3.2 Concatenation renders its constituents, each already simplified

Concatenation is the one kind whose rendering depends on whether the operation *succeeded*,
because success and failure produce genuinely different Foolish:

**Merged — render the merged brane.** When the elements combined, the concatenation's value is
one brane and that is what it renders as. The constituent branes are gone; showing them would
be showing the machinery:

```foolish
{{a=1}{b=2}{c=3}}    renders    {{a=1, b=2, c=3}}
```

**Unmerged — render the juxtaposition, each constituent rendered recursively.** When the
elements did not combine, there is no merged brane to show, so the written form stands. But
each constituent is itself rendered by this same sequencer, in this same mode — so a
constituent that reached a value shows that value, while one that did not shows its own written
form. It is **the simplest rendering of `foolish_children`** under the rest of §3, applied
element by element:

```foolish
{f=3; a={a=1,aa=f}{b=notfound}not_found_brane{d=f}}
```

renders

```foolish
{
  f = 3;
  a = {a=1, aa=3}{b=notfound}not_found_brane{d=3}
}
```

Read left to right: `aa=f` resolved against the enclosing brane and renders `3`; `b=notfound`
did not resolve, so it keeps its written form `notfound` (an unanchored miss is ECONSTANIC,
not NK — §4/§5); `not_found_brane` is likewise an unresolved name and stays as written; and
`d=f` resolved, like `aa`, to `3`. The concatenation itself renders as the juxtaposition of
those four, because it never merged.

Note what the first and last constituents demonstrate together: **a constituent search resolves
against the context the concatenation sits in**, so `f` is reachable from inside both `{a=1,
aa=f}` and `{d=f}` even though neither declares it. Both render `3`. That the concatenation
failed to *merge* does not stop its constituents from evaluating — which is precisely why
rendering them recursively, rather than echoing the source text, is the right rule.

**Deferred implementation case (human decision, 2026-09-03).** The current concatenation
implementation instead constructs the searches inside those bare brane constituents under SFF;
they settle ECONSTANIC with no result, so a read-only renderer cannot truthfully print `3`.
FOOP-46's BraneConcatOp work owns restoring the specified search behavior and then updating this
sequencer against it. Until that FOOP lands, FOOP-36's contract tests the ordinary nested-brane
case `{f=3; a=2; b={f1=f; f2=a; f3=not_found;};}`, where the two conclusive searches render `3`
and `2`, while the unresolved one remains written.

#### §3.2.1 §3.2 and FOOP-46's phased design converge

§3.2's two renderings and **FOOP-46's two phases are the same distinction**, arrived at from
opposite directions. FOOP-46 §1–§2 describes a `BraneConcatOp` that is either **Gathering** —
its `foolish_children` are the unmerged constituents, and it "is not yet a brane in any
meaningful sense" — or **Joined**, holding one brane in `ubc_children` with the constituents'
statements revived into it.

That maps onto §3.2 exactly:

| FOOP-46 phase | What the FIR holds | §3.2 renders |
|---|---|---|
| **Gathering** | unmerged constituents | the juxtaposition, `A B`, each constituent recursively simplified |
| **Joined** | one brane in `ubc_children` | that brane, `{…}` |

So the rendering rule does not merely *survive* FOOP-46 — it is the natural display of what
FOOP-46 builds, and each constituent's own render (a value where it reached one, its written
form where it did not) is exactly the per-constituent readiness the phased design reasons about.

**One detail to settle, and it should fall out cleanly.** §3.2 currently asks
`hs_concatenation()` whether `merged.is_some()`. FOOP-46 §4 may satisfy that question
differently — its option 1 populates `ubc_children` with a `FirSpec::Brane` and deletes
`FirSpec::ConcatHelper`, making `.value()` the interface rather than a `merged` slot. Either
way the *question* §3.2 asks is the same one FOOP-46's phases already answer; only the accessor
changes. §4's own text notes that `settled_constanic_result`/`value()` already return the brane once
constanic, so the Joined case needs no new mechanism.

The care needed is that a **reader of the output** can still tell a merged concatenation from a
plain brane, since §3.2 renders them differently (`A B` versus `{…}`) and that difference is
information about the program. If FOOP-46's chosen shape makes them indistinguishable in the
FIR, §3.2 should be amended deliberately to render them alike, rather than the distinction
lapsing unnoticed.

**What this FOOP does about it.** It writes the requirement into both FOOPs as a section each
owns (plan Phase 6.5), so the convergence is on the record in all three places rather than
depending on whoever lands last noticing it. Since the designs already agree, those sections
should be short — a statement of what rendering needs, and a pointer here.

**Why this is not a special case.** It falls straight out of §3: a merged concatenation has a
value, so render the value; an unmerged one does not, so render it as written, and "as written"
means each constituent gets the same treatment. The only thing §3.2 adds is that the
`merged`-slot is the signal — `hs_concatenation()` returns `(elements, merged)`, and `merged`
being `Some` is exactly the question "did it succeed?".

**`⨃` is never emitted in either case.** It is not input syntax and would not re-parse.

### §4 States with no syntax are annotated, never rendered

`PREMBRYONIC`, `EMBRYONIC`, `BRANING`, `ECONSTANIC` and `WOCONSTANIC` are **evaluator states,
not language constructs**. There is no Foolish token for any of them, so none may appear in
the rendered source as syntax.

Under §3 this is nearly self-enforcing: every kind already renders as its written form
regardless of state, so there is no place a state token could go. What remains is the
question of whether to *say anything at all* about the state — and the answer is a `!!`
comment, which the parser discards:

```foolish
s = <<a + b>>;          !! WOCONSTANIC
x = nonexistent;        !! ECONSTANIC (unfound)
t = a + b;              !! BRANING
u = {                    !! EMBRYONIC
  p = 1;
  q = p + 1;
}
```

The annotation is what makes `Foolish` mode usable for **debugging** (§2.1): a half-stepped
program renders as the program it still is, with how far each part has got in the margin. The
source text is unchanged by stepping; only the comments move. That is a far more legible
debugging artifact than the current dump — and `Detailed` mode remains for the cases where the
FIR's *internal* shape, rather than its source shape, is the thing under investigation.

Because §3 renders written form in every state, the annotation carries information that is
**otherwise unrecoverable from the output** — which is precisely why it is worth emitting. This
is exactly the test that excludes a **brane's own** state, below: a brane's annotation almost
always duplicates information a member line already carries.

**Comments are pure annotation.** `foolish_parser` discards `!!` to end-of-line (verified:
`foolish-parser/src/lexer.rs` handles `!!` line comments and `!!!` block comments), so every
comment this renderer emits leaves the re-parsed FIR unchanged. It must nonetheless be emitted
**deterministically**, because Property 2 compares strings and einmo compares bytes.

**Comment placement rule:** one comment per rendered line, appended after the statement's `;`
(or after the last token of a trailing statement), separated by exactly two spaces. Multi-line
constructs annotate their **opening** line. This keeps annotations off their own lines, where
they would perturb line-per-statement correspondence with the input.

**Annotate only what is not obvious.** A CONSTANT/INDEPENDENT node gets no comment — its
written form and its settledness agree, and a `!! CONSTANT` on every line would bury the
informative annotations in noise. Comments are emitted only for the five states above — and, per
§4.0 below, never as a rollup on a brane's own opening line.

#### §4.0 A brane's own state is never annotated — except a direct alarm

**A `Brane` (or `ConcatHelper`) node's own NYES is never rendered as a `!!` comment on its
opening line**, regardless of what that state is. This is a stronger rule than "annotate only
what is not obvious" above — it is a flat exclusion for one FIR kind, stated separately because
it is easy to get backwards.

**Why a brane is different from every other kind.** Every OTHER kind's state comment describes
something about *that specific node* that is otherwise unrecoverable — an operator's own
ECONSTANIC operand, a search's own NK reason. A brane's state, by contrast, is always a
**rollup** of its members (`decide_nyes_due_to_children`, `fvm_storage.rs` ~1070): the brane is
WOCONSTANIC because some member is WOCONSTANIC, or NK because some member's reason bubbled up.
That specific member **already renders its own accurate annotation on its own line** — so
repeating the rollup on the bracket is not new information, it is an echo, and per real
`einmo_suite2` fixtures it was frequently the exact same reason text duplicated onto two lines
(the enclosing brane's `{` and the responsible member's own line). §2.1's round-trip argument
covers this precisely: re-stepping the written brane reaches the same distribution of member
states, so nothing is lost by leaving the bracket bare.

**The one exception: a DIRECT alarm on the brane itself.** When a brane's NYES slot carries an
alarm reason that was set on *that pointer specifically* (`FVMStorage::alarm_reason`, a
per-pointer lookup with no recursion into children — contrast the general `nk_reason` used
elsewhere, which deliberately walks into children and is exactly the borrowing this rule
otherwise forbids), the brane's opening line DOES get `!! NK: <reason>`. This matters because
the evaluator's step-cap (`ITERATION-EXCEEDED`) alarm is set only on the **composed root**,
which is a brane, and no member of that brane carries the reason — suppressing it
unconditionally would silently discard the one piece of information nowhere else in the output,
defeating §2.1's whole debugging purpose. `warn_iteration_excess` (§4.1) still governs whether this
exception fires; `suppress_sequencing_comments` (§4.1) overrides it like everything else.

```foolish
{  !! NK: Iteration exceeded 9999
  f1 = {
    f1  !! ECONSTANIC
  };
  stuck = f1  !! BRANING
}
```

Here the outer brane carries a DIRECT alarm (the step cap fired on the composed root itself), so
it alone gets the exception. `f1`'s own sub-brane has no direct alarm — only a derived
ECONSTANIC rollup from its `f1` member — so it stays bare, and `f1`'s own line already carries
`!! ECONSTANIC`. `stuck = f1` never got the chance to settle before the cap fired (still
BRANING, pre-constanic — §2.1), so it too stays written with no borrowed reason of its own.

### §4.1 Line width — 108 characters

Rendered output targets a maximum line width that is **configurable, defaulting to 108
characters** — the project's document width (AGENTS.md §Code Style). The width is a parameter
of the render call, not a global constant.

**How it is configured.** The width rides on `SequenceMode`'s companion, not on a global
constant, so a caller that wants a different width says so at the call site and every nested
render inherits it:

```rust
/// Which evaluator findings the sequencer announces in `!!` comments.
/// One switch per warning kind; the default warns about what is ABNORMAL
/// or UNKNOWABLE and stays quiet about what is ORDINARY.
pub struct SequenceWarnings {
    /// An NK whose result is NOT a brane (`1/0`, an anchored miss). Default ON.
    pub warn_nk: bool,
    /// A brane that is itself NK. Default OFF — a rollup its member lines
    /// already explain (§4.0, §5.2).
    pub warn_brane_nk: bool,
    /// Default OFF — ordinary: dependencies settled without a value.
    pub warn_woconstanic: bool,
    /// Default OFF — ordinary: an unanchored miss, which may yet recoordinate.
    pub warn_econstanic: bool,
    /// Pre-constanic (BRANING, PREMBRYONIC, EMBRYONIC). Default ON — abnormal;
    /// sequencing normally runs on settled FIR (§5.3).
    pub warn_braning: bool,
    /// The step cap's banner (§4.4). Default ON — nothing else says so.
    pub warn_iteration_excess: bool,
}

pub struct SequenceOptions {
    pub mode: SequenceMode,
    /// Advisory target only; nothing enforces it (§4.1.1).
    pub width: usize,
    pub warnings: SequenceWarnings,
    /// Overrides EVERY warning above: no `!!` comment of any kind.
    pub suppress_sequencing_comments: bool,
}
```

`SequenceWarnings::silent()` turns everything off; `::verbose()` turns everything on, which is
how a debugger sees the ordinary states the default hides.

**Why these defaults.** ECONSTANIC and WOCONSTANIC are the *routine* outcomes of ordinary
programs — an unanchored name that did not resolve here is not news, and FOOP-23 says it may
still resolve elsewhere. Annotating every one buries the findings that do matter. A non-brane
NK is the opposite: `1/0` is unknowable and the reader must be told. A brane's own NK is a
rollup its members already explain (§4.0, §5.2), so it is off; BRANING and the step cap are
abnormal, so they are on.

**One object, not a pile of booleans on `SequenceOptions`** (human, 2026-09-03). The knobs
earned their own type once there were more than two; `suppress_sequencing_comments` stays on
`SequenceOptions` because it is an override of the whole set, not a member of it.

### §4.4 The step-cap banner sits above the program

When the evaluator's step cap fires, the finding is about the **whole Foolish program**, not
about the brace it would otherwise trail. It therefore renders on **its own line, above the
program**:

```foolish
!! This Foolish program did not complete stepping within the limit of 9999 steps !!
{
  f1 = {
    f1
  };
  stuck = f1  !! BRANING
}
```

This follows §4.2's own rule that a full-line `!!` comment marks the code BELOW it, and it is
phrased for a human reader rather than as the raw `Iteration exceeded 9999` alarm text. The
root brane is the **Foolish program**, which is what the banner names.

**Only the step-cap alarm gets this treatment, and only on a brane.** Other alarms — a value
search's `VALUE-SEARCH-UNSUPPORTED-PATTERN`, say — sit on ordinary expression nodes
mid-statement, where a full-line comment above the node would split the statement in two and
fail to re-parse (caught by §T3 on `foop/23/value_search_pattern_error`). Those keep the
ordinary trailing-comment form:

```foolish
  bad = a~={q = 1};  !! NK: VALUE-SEARCH-UNSUPPORTED-PATTERN: pattern is neither intege…
```

**`suppress_sequencing_comments` is an override, not a peer flag.** It sits one level above the
whole `SequenceWarnings` set: each `warn_*` switch decides whether its own finding is
announced, while `suppress_sequencing_comments` silences **every** `!!` annotation this
renderer emits regardless of what any of them say. A caller wanting the closest thing to a
plain pretty-printer sets this rather than enumerating every warning kind — though
`SequenceWarnings::silent()` reaches the same output through the set itself.

`Ubca2Sequencer::format(storage, fir, mode)` stays as the common-case entry point (it builds
`SequenceOptions::default()` with the given mode);
`format_with(storage, fir, &SequenceOptions)` is the form that takes an explicit width. The
einmo adapter uses the default, so the corpus is reproducible.

#### §4.1.1 One statement per line — always, and no wrapping yet

**Foolish Standard Formatting puts every statement on its own line.** A brane renders its
opener, then one line per member statement, then its closer — unconditionally, regardless of
how short the brane is. There is no single-line collapsing of `{a = 1; b = 2}` into one line
and no width threshold deciding between the two forms.

```foolish
{
  a = 1;
  b = 2
}
```

**Within a line, there is no wrapping at all.** A statement that runs long simply runs long.
The 108-column width is therefore, for now, an **advisory target that nothing enforces** — it
is documented because it is the project's document width (AGENTS.md §Code Style) and because a
later FOOP will define wrapping against it, but this FOOP's renderer never breaks a line to
satisfy it.

**Why no wrapping yet, explicitly.** Foolish has **no end-of-line continuation syntax** — `\`
at end of line is not defined in the lexer, and inventing one here would be a language change
smuggled in through a rendering FOOP. Wrapping without a continuation marker means either
breaking a statement across lines in a way that changes how it parses (breaking Property 1,
§2), or inventing an indentation-continuation convention the parser does not know about.
Neither is this FOOP's to decide. **A later FOOP defines EOL continuation semantics and the
wrapping rules that build on them; until then, long lines are correct output.**

**What this replaces.** Earlier revisions of this section made 108 a live single-vs-multi-line
threshold threaded down as `line_hint`. That machinery is gone for statements: the decision it
made no longer exists, because the multi-line form is now the only form. `SequenceOptions::width`
is retained in the struct (callers may still set it, and a future wrapping FOOP will want it)
but the `Foolish` renderer does not currently consult it for statement layout.

**Why always multi-line is the right default, not merely simpler.** One statement per line is
what makes the `!!` annotation scheme (§4) coherent: an annotation belongs to exactly one
statement and sits at the end of exactly one line. The moment several statements share a line,
a single `!!` — which the lexer reads to end-of-line — swallows every statement after it on
that line. That is not a hypothetical: it was a real, corpus-caught bug in this FOOP's own
renderer, where an inline-collapsed multi-line operand put a `!!` mid-line and silently ate the
anchor's closing brace and the statement's `;`. Making the multi-line form universal removes
that failure mode by construction rather than by patching each site that collapses lines.

### §4.3 Foolish Standard Formatting — canonical spellings

Beyond layout, `Foolish` mode normalizes three spellings. Each has the same shape of
justification: several source spellings mean the same thing, so the renderer picks one, and the
one it picks is the one that re-parses unambiguously.

#### §4.3.1 The attached search form: `A = B SEARCH` renders `A =SEARCH B`

FOOP-75 §4 defines the attached spelling and §2 establishes that both spellings build the same
tree. The search operator attaches to the **right** of the `=` and the anchor follows it:

```foolish
h = b.x^        renders    h =^ b?x
e = {}^         renders    e =^ {}      !! NK: anchored index found no match
```

**Only `^` and `$` ever attach.** Every other search operator renders **postfix**, even when
the source wrote it attached — `A =#1 B` renders `A = B#1`. Head and tail are the only two
whose attached form is unambiguous on replay; the rest re-scan ambiguously (FOOP-75 §6), and
one canonical spelling per operation is worth more than preserving which spelling the Foolisher
happened to use.

**`#0` and `#-1` canonicalize first.** An anchored index at offset 0 IS head and at offset -1
IS tail, however it was spelled, so `result = empty#0` renders `result =^ empty` and
`d = b#-1` renders `d =$ b`. The canonicalization happens before the attach decision, so a
source-level `#0` reaches the attached form and a source-level `#2` does not.

**§4.1's transparency rule comes first.** A search that settled conclusively renders as its
value, not as a search at all (FOOP-75 §4.1), so the attached form appears **only** for
statements still showing their search structure — unsettled, or NK. `d = b#-1` on a brane whose
tail is `30` renders `d = 30`, not `d =$ b`.

#### §4.3.2 Marker runs: unambiguous opens, greedy closes

SF/SFF delimiters (`<`, `<<`, and the longer forms a later FOOP adds) are lexed by greedily
pairing adjacent characters, so a run of them at a boundary can group in a way neither wrapper
intended. The language rule, stated here so later FOOPs can find it:

> **An opening marker run must be unambiguous.** `<<<<` is NOT acceptable — it does not say
> whether it opens two `<<`s, or a `<` and a `<<<`, or some other split. `<< <<` and `<<< <`
> are acceptable, because the space says where each marker ends.
>
> **A closing marker run is decoded greedily against the open-marker nesting.** The decoder
> knows which markers are open and in what order, so it consumes each close against the
> innermost still-open marker. A long run like `>>>>>>>>>>>>>` therefore resolves without
> needing internal spaces, and `< << <<< < <<< << >>>>>>>>>>>>>` decodes successfully.

**The sequencer's whole obligation under this rule is narrow: emit a space after an opening
marker when the token that follows is itself a marker.** Closing runs need nothing — the greedy
decoder handles them. (This FOOP's renderer currently also spaces some closing boundaries; that
is conservative rather than required, and a later FOOP implementing the greedy decoder above can
remove it. See §4.3.2.1.)

##### §4.3.2.1 Why the closing-side space is currently unconditional

An attempt was made to insert the closing-side space only where the boundary run is genuinely
ambiguous (odd length). It fails, and the reason is worth recording: whether a given close is
safe depends on what an **enclosing** wrapper appends immediately afterward, which the render
call producing the inner text cannot see. `b = <1 + <<b>> + <c>>` looked safe considered one
level at a time and still fused. Until the greedy decoder of §4.3.2 exists, the renderer takes
the conservative route — a space whenever the interior touches this wrapper's own delimiter
character — accepting an occasionally-unnecessary space in exchange for never emitting an
unparseable run.

#### §4.3.3 No needless parentheses

**`((a))` renders `(a)`.** Redundant grouping is removed; parentheses appear only where they
delimit or disambiguate something. In particular a search pattern that is already stored
parenthesized (FOOP-75 §6.1: the current parser absorbs the parens INTO `pattern`, so `?(ho)`
stores `"(ho)"`) is written back exactly as stored rather than wrapped again — `?((ho))` is not
disambiguation, it is a stray unmatched paren the parser cannot place.

**Do not add a width assertion to the einmo gates.** Width is a formatting target; a case
whose *input* is legitimately wide would fail a hard gate for a reason unrelated to what it
tests. §Test Plan T7 checks the budget where it is meaningful — that the renderer breaks lines
it can break — not as a corpus-wide invariant.

### §4.2 Comment style in einmo inputs, and the separator constraint

The `.foo` inputs of an einmo suite are read by humans far more often than ordinary source —
they are the *statement of what is being tested*. This FOOP therefore fixes their comment
style, and the rules apply to every input this FOOP writes (`einmo_suite2`'s
`rendering_contract.foo`, `foop/36/comprehensive.foo`) and to any input edited afterwards.

**1. Short inline comments are permitted**, trailing the code they annotate:

```foolish
brane_operand = {1, {x = 5;}, 'lt}$;    !! a brane is not comparable -> NK
```

**2. Block comments (`!!!` fences) must be surrounded by blank lines** — one before AND one
after. A fenced block is a section heading, not a remark about the next line, and the space on
both sides is what makes it read that way:

```foolish
	a = 1;

	!!!
	FOOP-33 §5: only integers are comparable. When an operand DID evaluate
	but is not an integer, the comparison settles NK rather than inventing
	an ordering.
	!!!

	b = {1, ⬤, 'lt}$;
```

**3. A full-line comment (a line starting with `!!`) marks the code BELOW it.** It takes a
blank line **before** it, separating it from the preceding content — and **no** blank line
after, so it sits tight against the lines it describes:

```foolish
	a = 1;

	!! Anchored miss settles NK: the name is provably not in that brane.
	miss = b?nonexistent;
	also_miss = b?absent;
```

Writing a blank line after the comment orphans it — the reader can no longer tell whether it
belongs to what follows or to what came before. Proximity is the whole signal.

#### The separator collision — `①`, not `!!`

Einmo splits sections on a configured **separator string**, and its collision rule is a plain
substring check on each section body (`einmo/src/format.rs::serialize`): a match is a **hard
error at write time** (`EinmoError::SeparatorCollision`) — the file does not serialize. So the
constraint on comment style is: *whatever the separator is, content must never contain it.*

**The two suites differ, and this is verified from the artifacts rather than from FOOP-92's
spec text, which lags:**

| Suite | Separator | Header line |
|---|---|---|
| `foolish-ubca/einmo_suite` | `"!!" + LF` | `#einmo 1 encoding=utf-8 separator=!!\n` |
| `foolish-ubca2/einmo_suite` | **`"①" + LF`** (U+2460) | `#einmo 1 encoding=utf-8 separator=①\n` |
| `foolish-ubca2/einmo_suite2` (new) | **`"①" + LF`** — matches its sibling | — |

`einmo_suite2` uses **`①\n`**, the same two-character sequence as `einmo_suite`, so both ubca2
suites are configured alike and a case can move between them unchanged.

Consequently, for ubca2 suites:

> **No line may consist of, or end with, exactly `①`.**

`!!` carries **no** such restriction in a ubca2 suite — a bare `!!` line, or a statement with an
empty trailing `!!`, is merely a Foolish comment and serializes fine. (In `foolish-ubca`'s
suite, which still separates on `!!`, the opposite holds. This FOOP writes no inputs there, but
anyone copying a case *into* that suite must re-check it.)

Since `①` appears in no Foolish operator or identifier, the rule is nearly free — but the
renderer must still not emit it: §5 already collapses `①` in an NK reason to a space, which is
exactly this constraint, and §Test Plan T8 checks it.

**Note a stale comment to fix in passing.** `foolish-ubca2/einmo_suite/einmo.toml` says the
suite "uses the Foolish line-comment separator … set in code via
`TestConfig::foolish_separator()`", but `ubca_snapshot_tester.rs` calls plain
`TestConfig::new(...)` and the artifacts carry `separator=①\n`. The comment describes what the
suite does *not* do. The plan corrects it.

### §5 NK renders as the original Foolish, with the reason in a comment

NK needs no rule of its own — it falls out of §3's predicate. An NK result is an **inconclusive
constanic** (§0), so **the original expression renders** and the NK-ness goes in a `!!` comment. `1/0` is the operator instance of that predicate; an NK search is the search
instance. It is stated separately here because NK is the case a reader is most likely to expect
an exception for.

```foolish
a = 1/0;                !! NK: DIV-BY-ZERO: division by zero
```

Not `a = ???`. The division is the program; that it is unknowable is the evaluator's finding
about the program, and findings go in comments. Rendering `???` would additionally substitute
the *no-no literal* for something the Foolisher never wrote.

**An NK search result renders as the search, not as NK.** Not a special case — just §3's
predicate: an NK result is an inconclusive constanic, so the original search renders:

```foolish
miss = b?nonexistent;   !! NK: anchored miss
```

Not `miss = ???`. This matters because NK's reason for an anchored miss is "the name is
provably not in *that* brane" — a statement about a specific brane, which the rendered search
names and a bare `???` does not.

**The one place `???` IS the rendering: where the Foolisher wrote `???`.** The no-no literal
is Foolish source, and source renders as itself. So `x = ???` renders `x = ???` — not because
the value is unknowable, but because that is what was written.

#### The reason comment — flagged on or off

**Whether NK is commented at all is a flag**, `SequenceWarnings::warn_nk` (§4.1), default
**on**. The two settings serve different readers:

- **On** (einmo's setting): `a = 1/0;  !! NK: DIV-BY-ZERO: division by zero`. A reviewer sees
  both the program and the finding, which is what makes a baseline reviewable.
- **Off**: `a = 1/0;` — pure Foolish, no trace of the evaluator's finding. For a caller that
  wants source rather than a report.

Either way the *expression* is identical; only the annotation moves. The einmo corpus is
rendered with the default, so it is reproducible.

**`SequenceOptions::suppress_sequencing_comments` (§4.1) overrides every warning.** Setting it
silences NK's reason comment regardless of `warn_nk`, along with every other annotation — a
caller wanting zero evaluator commentary sets this one flag rather than `warn_nk: false` plus
some hypothetical state-comment equivalent that does not otherwise exist.

**NK is constanic AND constantew** — see §5.1 for why that matters and what it does not change.

When on, the reason is drawn from `hs_nk()` and follows §4's placement rule:

- Prefixed `NK:` so it is distinguishable from a §4 state annotation at a glance.
- The `Alarm` code included when present: `!! NK: DIV-BY-ZERO: division by zero`.
- **One line**: newlines and the einmo separator `①` replaced by a space (§4.2).
- Truncated to a stated maximum of **60 characters**, with a trailing `…` when longer. Brief is
  the point; the full alarm text stays available in `Detailed` mode.

#### §5.1 NK is constantew; ECONSTANIC is not

FOOP-62 §Terminology (the in-force authority) divides the constanic states:

| Term | States | Meaning |
|---|---|---|
| **constantew** | CONSTANT, INDEPENDENT, **NK** | constant *everywhere* — won't change no matter what |
| **non-constantew constanic** | ECONSTANIC, WOCONSTANIC | value may change when context is recoordinated |

FOOP-62 lists `is_constantew()` as a predicate, but **it does not exist in the code** (verified
2026-09-02): `foolish-core/src/fir.rs` has `is_constanic()` and `is_nnk_constanic()` only. This
FOOP needs no such predicate — §3 keys on constanic, and §5's NK handling keys on `hs_nk()` —
so it does not add one. Noted because the gap is easy to trip over when reading FOOP-62.

Constantew and conclusive (§0) are different cuts, and NK is what separates them: NK is
constantew (nothing will change it) yet inconclusive (it never produced a value). Rendering
keys on **conclusive**; recoordination keys on **constantew**.

When a rendered **ECONSTANIC** search is re-read in a new context, it may genuinely resolve
there — that is non-constantew, and it is why rendering the search rather than a value is not
merely tidier but *necessary*: collapsing it to a value would freeze an answer the language
says is still open.

**NK is the opposite: constantew, so re-coordination changes nothing.** `1/0` is NK here and NK
anywhere. That makes NK the *easy* case for §2's round trip rather than a hard one — `1/0`
renders `1/0`, re-parses, re-settles NK, and `R2 == R1`. The reason it still renders as written
rather than as `???` is not fear of losing an answer; it is simply that `1/0` is the program and
`???` is a different expression the Foolisher did not write.

#### §5.2 The brane exception: an NK **brane** renders its contents

**NK reverts to the written expression — EXCEPT when the NK result is a brane, in which case
the brane is rendered** (human, 2026-09-03).

```foolish
{a = 1; b = {c = #-1; d = 2; e = #-1}; f = #-1;}

  f = {            !! not `f = #-1`
    c = #-1;  !! NK: unknown
    d = 2;
    e = 2
  }
```

**Why a brane is different from a scalar.** For `1/0`, the written form **is** the information —
`1/0` says everything there is to say. A brane's NK, by contrast, is a **rollup**: the brane
above is NK only on account of `c`, while `d` and `e` resolved perfectly well. Reverting the
whole statement to `#-1` would hide both the structure the search genuinely found and the
members that did resolve.

Rendering the brane instead leaves **every member in its own correct state** — the unresolved
ones as their original searches, the resolved ones as their values. That is also what makes the
output right to *re-read elsewhere*: the unresolved members are still searches, free to resolve
in the new context, which is exactly what recoordination promises. The reader is not told which
members are constanic, but can infer it from what each one renders as.

**The brane's own opening line stays bare**, per §4.0 — the member lines carry the accurate
annotations, and a rollup on the brace would be an echo (in the case above, literally a bare
`!! NK: unknown`, which says strictly less than the lines beneath it).

#### §5.3 BRANING reverts, and says so

**A BRANING result always reverts to its written form** — the §5.2 brane exception does **not**
extend to it — **but it carries a `!! BRANING` annotation.**

**Why it reverts.** Tried the other way and reverted it (2026-09-03): a BRANING brane is still
*mid-evaluation* and may be **self-referential**, so rendering its contents unrolls the
recursion. `{f1 = { f1 }; stuck = f1;}` expanded roughly 32 levels of nested braces before the
step cap stopped it. An NK brane is safe precisely **because** it has settled — its contents are
fixed and finite. A BRANING brane has no such guarantee.

**Why it is annotated even though §4.0 suppresses brane rollups.** Reaching the sequencer still
BRANING is **abnormal** — sequencing normally runs on settled FIR. Unlike an NK rollup, which
the member lines already explain, this one tells the reader something they cannot otherwise
see and would want to know: that evaluation did not finish. It is worth the line precisely
because it is not normal.

Like every annotation this renderer emits, it answers to `suppress_sequencing_comments` (§4.1).

### §6 Nothing is removed

- `foolish_core::sequencer` is **not modified**. Not one line.
- `foolish-ubca`'s einmo baselines are **not touched**. It does not compile this code.
- `Detailed` mode reaches the identical bytes it does today, via delegation.
- `foolish-ubca2`'s `checked/` baselines **do** all change — that is this FOOP's whole visible
  effect, and it is the one thing here that needs the Promotion Review Gate, case by case,
  across all 179 inputs (§Test Plan §T4 addresses the scale).

**⚠ `foolish-ubca2/einmo_suite/verified/` is POPULATED — all 179 cases, human-signed, and
`einmo_gate_verified` passes today** (verified 2026-09-02; the crate's full suite is 134/134).
Earlier notes — including FOOP-16 and `ubca_snapshot_tester.rs`'s own doc comment — say
`verified/` is empty and the gate is expected to fail. **That is stale.** The consequence is
serious and is the single biggest risk in this FOOP: every case this FOOP re-renders has a
`verified/` twin, and a `verified/` artifact is **frozen — it may not be touched without a
human reviewer's key** (AGENTS.md; `foop.md` §Promotion Review Gate).

So re-rendering all 179 baselines will break `einmo_gate_verified`, and **an agent cannot fix
that**: promoting `checked` → `verified` requires the human's passphrase, interactively. This
is a **blocking human decision**, raised in §Open Questions Q6, and it must be settled before
Movement III (the adapter switch) — not discovered at Phase 6.

## FIR Impact

**None.** No new FIR variant, no state-machine change, no serialization change. This FOOP reads
`foolish-ubca2`'s existing arena FIR through `FVMStorage`/`FirCursor` and writes text. The arena
already retains every field needed by §3, including search direction and contexting. The
legacy `proto_to_core_fir` representation is used only by `Detailed` mode and the existing
`foolish_core::Evaluator` compatibility API; Foolish rendering occurs before that conversion.

## UBC Step Impact

**None.** The sequencer runs on already-constanic FIR. No step rule changes; no evaluation
order changes; no NYES transition changes. Step counts in einmo output are unaffected.

## Test Plan

`einmo_suite2` is not a side experiment — it **becomes the approval suite for `foolish-ubca2`**
(§Motivation "The suite is replaced, not edited"). Everything the old suite guaranteed, the new
one must guarantee, and the rendering it is written in is itself new and unproven. The tests
below are therefore in two groups: those that prove the **sequencer** is right, and those that
prove the **suite** is a fit replacement.

### Group A — the sequencer

**T0 — the hand-written rendering contract.** `einmo_suite2/input/foop/36/rendering_contract.foo`
— one brane whose members are sub-branes, one per rendering concern (§3's rows, §3.2's
concatenation split, §4's annotated states, §4.1's width behaviour, §5's NK forms). **Its
expected OUTPUT is typed by hand from this specification before the renderer is written**, and
the renderer is developed until it reproduces what was typed.

First because it runs first, and because it is **this FOOP's own acceptance test**: the FOOP
claims einmo expectations become writable by a person reading the spec, and T0 is that claim
executed. If it proves impractical, the design is wrong, and one case is a far cheaper place to
learn that than 179. It also means the renderer is developed against a *human-authored* target
rather than against its own output — the only way the later Promotion Review Gate is more than
a matching exercise.

**T1 — Unit tests** (`foolish-ubca2/src/sequencer.rs`, tests module). One test per §3 row
asserting the exact rendered string. Both sides of §3's predicate for searches, operators and
indexes: a **conclusive** result collapsing to its value, and each flavour of **inconclusive
constanic** result (ECONSTANIC, WOCONSTANIC, NK, and result-absent) rendering the original
expression. `Detailed`-mode tests assert byte-equality with `foolish_core::FirSequencer::format`
on the same FIR — §1's delegation contract, pinned so it cannot drift.

**T2 — Round-trip properties (§2).** The load-bearing tests, as six literal steps:

1. **Compile** the program `P`.
2. **Step to finish** (settled).
3. **Output** in `Foolish` mode → `R1`.
4. **Compile `R1`** — that it compiles at all is Property 1.
5. **Step to finish.**
6. **Output** → `R2`. **Assert `R2 == R1`** (Property 2).

Property 2 is the fixed point stated operationally, and it is far stronger than reading one
rendering: a construct that renders to something even slightly different drifts on the second
pass and the test catches it, with nobody predicting the right answer in advance.

The input must **instrument a variety of constanic states**, not only constants: CONSTANT and
INDEPENDENT values; ECONSTANIC searches (unanchored misses); WOCONSTANIC statements; NK
expressions (`1/0`, an anchored miss); merged and unmerged concatenations; SF and SFF. NK is
constantew (§5.1) so it re-settles NK and the equality holds; ECONSTANIC is non-constantew and is
the interesting one to watch. Property 2 is asserted **only** where the FIR is constanic
(§2.1's table) — for pre-constanic FIR only steps 1–4 apply.

This procedure also settles §Open Questions **Q7** empirically.

**T2b — Pre-constanic rendering (§2.1).** FIRs stepped a bounded number of steps rather than to
settlement: each renders, **parses**, contains **no NYES token as syntax**, and names its state
**only** inside a `!!` comment. Idempotence deliberately not asserted (§2.1). Cover at least one
PREMBRYONIC, one EMBRYONIC and one BRANING node, plus one case halted by an `ALARM:` mid-step —
the shape einmo actually captures when debugging.

**T7 — Line width (§4.1).** A construct exceeding the configured width at its indent breaks
across lines with its body indented; nesting reduces the budget by the indent; a non-default
width changes where breaks fall. Plus §4.1's three exceptions rendering intact rather than
mangled: an unsplittable long atom, an annotated line pushed over by its `!!`, and echoed
over-width source. **Not** a corpus-wide width assertion — §4.1 says why.

**T8 — Separator safety and comment style (§4.2).** The renderer never emits `①` (U+2460)
anywhere — chiefly via an NK reason containing one, which §5 collapses to a space. Plus a check
that every `.foo` input this FOOP authors follows §4.2's layout rules (blank line before a
full-line comment and none after; blank lines both sides of a `!!!` fence).

**T9 — Flags (§4.1, §5).** `warn_nk` off renders `a = 1/0;` with no annotation and on renders
`a = 1/0;  !! NK: …`, the *expression* identical under both. A non-default `width` changes line
breaking. `suppress_sequencing_comments` on silences both NK's reason comment and every §4 state
comment, overriding every `warn_*` switch even when they explicitly ask for annotations. The einmo
adapter uses the defaults, so the corpus is reproducible.

### Group B — the suite as a fit replacement

*These exist because `einmo_suite2` inherits the old suite's job. A rendering bug caught by
Group A is a bug; a coverage or integrity gap caught here would be a silently weaker test
suite, which is worse.*

**T3 — Corpus-wide parse.** One test walking every `einmo_suite2/input/**/*.foo`: evaluate,
render in `Foolish` mode, assert the result **parses**. §2's Property 1 across the whole corpus
— the cheapest broad guard against a rendering that is locally fine and globally unparseable.
Parse-ability only, not idempotence, so it stays fast and stays correct for non-settling cases.

**T4 — Baseline review and promotion.** The 179 inputs are copied into `einmo_suite2` and
rendered under the new sequencer; every one goes through the Promotion Review Gate before
becoming `einmo_suite2/checked/`. **This is the bulk of the work** and the plan phases it
accordingly — one review sub-phase per suite subdirectory, never one 179-item list, because a
gate whose boxes are checked faster than the cases could be read is a false record
(`foop.md` §"Promotion Review Gate").

**T5 — `foolish-ubca` untouched.** `cargo test -p foolish-ubca --lib -- einmo_gate_checked`
passes unchanged, before and after. It should hold *trivially*; if it does not, this FOOP has
modified shared code it promised not to — stop and report.

**T5b — `einmo_suite` untouched.** The old suite's three gates — including
`einmo_gate_verified` against its 179 human-signed artifacts — pass unchanged throughout. A
moved baseline there means something re-rendered the frozen reference.

**T6 — Comprehensive case.** `einmo_suite2/input/foop/36/comprehensive.foo`, exercising at
least one path through every §3 row plus §3.2, §4, §4.1 and §5 together, with its expected
output hand-written before running (as T0).

**T10 — Coverage parity.** `einmo_suite2` contains an input for **every** input in
`einmo_suite` — same relative paths, same count (179), plus this FOOP's own two cases. A
mechanical check, because "the new suite quietly tests less than the old one" is the failure
mode that would matter most and is invisible from a green run.

**T11 — Suite integrity.** `einmo_suite2` satisfies einmo's own soundness checks at each
validation level (`results.integrity.is_clean()`), its gates serialize against `einmo_suite`'s
under the shared `GATE_LOCK`, and its `einmo.toml` is configured as §4.2 requires — separator
`①`+LF, distinct `checked` passphrase, `verified` left unconfigured so a human must type one.

**T12 — Value non-regression across the cut-over.** For every one of the 179 cases, the
*values* in `einmo_suite2`'s output match those in `einmo_suite`'s frozen `checked/`. The
rendering changes; the program's meaning must not. A `12` that became a `13`, or a settled case
that became NK, is a bug in this FOOP and not a new baseline. Mechanical where the shapes allow
and by reading where they do not — this is what the Phase 6 review is *for*, and T12 is its
statement as a requirement.

## Plan of Execution for Plan

This FOOP is deliberately shaped to be executed by a **smaller model than the one that wrote
it**, phase by phase, with the specification carrying the reasoning and the plan carrying the
facts. This section records how, so the choice is made once here rather than improvised per
phase.

### Model selection is per-phase, not per-FOOP

The phases differ sharply in what they demand. Sizing them all to the hardest one wastes
capability on mechanical work; sizing them all to the easiest one puts judgment calls in the
wrong hands. Concretely, on the three harnesses in use here:

| Harness | Larger model — judgment phases | Smaller model — execution phases |
|---|---|---|
| Claude | Opus / Sonnet | Sonnata |
| Codex | — | GPT-terra |
| Local | — | Qwen3.8-27B |

### Which phase needs which

| Phase | Character | Needs |
|---|---|---|
| **0** — Begin | Record baselines; one **blocking** question (FOOP-26 ordering) | Small model, but it must **stop and ask**, not decide |
| **1** — Q2 / Q5 | Read code, answer two design questions, possibly **STOP** | **Larger model.** Q5's answer changes whether §3.1 is implementable at all |
| **2** — Skeleton | Type the given code, pin delegation | Small model; the code is in the plan verbatim |
| **3b–3c** — Hand-write expectations | **Read spec, predict output.** The FOOP's acceptance test | **Larger model.** This is the one phase where being able to derive output from spec IS the deliverable |
| **3d** — Implement to green | Iterate renderer against a fixed target | Small model; the target is fixed and failures are concrete |
| **4** — Feature completion | Write property tests | Small model, with the §2.1 table in front of it |
| **5** — Adapter switch | One-line change; verify four expected outcomes | Small model |
| **6** — 179-case review | **Judgment, 179 times.** Promotion is a correctness claim | **Larger model**, or split across several with per-subdirectory reports |
| **7** — Comprehensive | Predict output, then reconcile | **Larger model** (same reason as 3b) |
| **8** — Merge | Mechanical, with a human STOP | Small model |

### What makes the small-model phases safe

Three properties of the plan, all deliberate:

1. **Facts are inline, not referenced.** The plan's Orientation block carries the trait shape,
   the verified lexer behavior, file sizes, tuple arities and exact command forms, so an
   executing agent spends its context on the work rather than on rediscovery. Every fact is
   marked *verify, don't re-derive*.
2. **Each phase has a fixed target.** After Phase 3c there is a hand-written expected output;
   after Phase 5 there is a mechanical diff. An agent that cannot judge "is this right?" can
   still answer "does this match the thing a human wrote?" — which is a different and much
   easier question.
3. **The stop conditions are named.** Phase 1's Q5, Phase 5's four expected outcomes, and the
   standing scope guard each say explicitly what a wrong result looks like and that the answer
   is to STOP and report. A small model does not have to recognize trouble unaided; it has to
   match a stated condition.

### What must not be delegated

Per AGENTS.md §"The agent is responsible for correctness", these remain the responsibility of
whichever agent performs them, regardless of size, and none may be discharged by "the tests
passed":

- **Every `output` → `checked` promotion** (Phases 6 and 7). Promotion is an assertion about
  the program, justified by reading, not by matching.
- **The hand-written expectation** (Phase 3c). Generating it and calling it hand-written would
  void the FOOP's entire purpose.
- **Any decision to touch `foolish-core`** (§FIR Impact) — reported to the human first, never
  taken as an implementation convenience.
- **Marking any Verified-tier test `#[ignore]`** — never an agent's call (AGENTS.md).

## §7 What changes, in one place

Every before/after comparison in this FOOP lives here; the specification sections above state
the design forward, without reference to what preceded it.

**A settled search.** `{b = {a1=1; a2=2; a3=3}; r = b?a.*;}`

| | rendering |
|---|---|
| today | `r=3` |
| §3 | `r = b?a.*` when the result is an inconclusive constanic; `r = 3` when conclusive |

**An unfound name.** `{x = non_existent;}`

| | rendering |
|---|---|
| today | `{WOCONSTANIC` … `x=?(pattern='^nonˍexistent$', UNANCHORED, ECONSTANIC)` |
| §3, §4 | `x = nonˍexistent  !! ECONSTANIC (unfound)` |

**A deferred sum.** `{a=1; b=2; s=<<a+b>>; a=10; s;}`

| | rendering |
|---|---|
| today | `s=<<WOCONSTANIC` / `Op+(?(pattern='^a$', UNANCHORED, ECONSTANIC), ?(pattern='^b$', …), WOCONSTANIC)` / `>>` |
| §3, §4 | `s = <<a + b>>;` |

**Division by zero.** `{a = 10/0;}`

| | rendering |
|---|---|
| today | `a=Op*(??? (division by zero), DIV-BY-ZERO: …, NK)` |
| §3, §5 | `a = 10/0;  !! NK: DIV-BY-ZERO: division by zero` |

**Summary of what disappears.** The `?(…)` and `Op…(` wrappers; `pattern='^…$'` regex
spellings; `ANCHORED`/`UNANCHORED` tokens; bare NYES tokens on brane openers and in operand
lists; `result=` slots; the `⨃` concatenation prefix. What replaces them is the program, plus
`!!` comments where the evaluator has a finding worth recording.

**What does not change.** `Detailed` mode (§1, §6) reaches byte-identical output to today, and
`foolish-ubca` is untouched.

## Rejected Alternatives

### A. Do nothing — keep the detailed rendering as the only mode
Leaves einmo OUTPUT unwritable by hand and unjustifiable except by comparison with the
evaluator. That is the precise thing AGENTS.md's Promotion Review Gate forbids as a
justification, so "do nothing" preserves a standing conflict between what the process demands
of reviewers and what the artifacts let them do. Rejected.

### B. Add a `Foolish` mode to `foolish_core::FirSequencer` (shared)
The smallest diff, and tempting. Rejected because it puts ubca2's rendering evolution inside
the module `foolish-ubca` depends on: every later change risks the sibling's baselines, and
the two crates' deliberate independence (lib.rs) is undermined at exactly the seam where they
are supposed to be merely *compared*. It also makes this FOOP's non-regression guarantee a
matter of care rather than of construction.

### C. Post-process the detailed output with a text transform
Strip `NYES` tokens and unwrap `?(…)` with regexes over the rendered string. Rejected: the
information needed to render a *written form* (§4) is destroyed by the time text exists, the
transform would be un-reviewable, and it would silently rot as the detailed renderer changed.

### D. Render only fully-CONSTANT programs, and refuse the rest
Would let §4 be skipped entirely. Rejected because the ECONSTANIC/WOCONSTANIC cases are
exactly the interesting ones — SF/SFF, unfound searches, deferred macros are most of what the
suite tests, and FOOP-26 adds more.

### E. Emit the state annotations as structured einmo metadata instead of `!!` comments
Cleaner in principle. Rejected for this FOOP: it changes the einmo envelope format (a
cross-crate concern owned by FOOP-54/64), and `!!` comments are already parser-discarded, so
they cost nothing and keep the OUTPUT a single self-contained Foolish program. Worth
revisiting if annotation volume grows.

### F. Demand semantic identity instead of idempotence (§2)
Require that re-parsing `R` yield a FIR *equivalent* to `eval(P)`, rather than merely that
rendering reach a fixed point. Rejected on two grounds. First, it is not well-defined without
a FIR equivalence relation, which does not exist in the codebase and would be a substantial
FOOP of its own. Second, it is false in general by design: a search that settled ECONSTANIC
renders as its written form, and re-parsing that form in a *different* context may legitimately
resolve it — recoordination is the language working correctly, not the renderer losing
information. Idempotence captures what a baseline actually needs (nothing further to lose)
without asserting something the language does not promise.

## Open Questions

Ordered by number. **Q4 and Q6 are RESOLVED** (human decisions, recorded inline below and
reflected in the plan); **Q2 and Q5 are for Phase 1 to answer before any rendering code is
written** (Q7 alongside them); Q1 and Q3 are cosmetic and were settled by the human.

- **Q1 — RESOLVED (human, 2026-09-02): out of scope. This FOOP targets einmo only.** The
  einmo adapter switches to `Foolish`; the REPL and every other caller are left exactly as
  they are. `Ubca2Sequencer` is additive, so nothing outside the adapter changes behavior
  unless a later FOOP chooses to move it.
- **Q2 — RESOLVED (human, 2026-09-03): render directly from ubca2's arena and standardize
  equivalent source spellings.** `FirSpec::Search` retains the pattern, anchoring, direction,
  value-search flag, and contexted flag; its `foolish_children` retain the anchor and value
  operand even when a search misses. A hit renders its conclusive value; a miss therefore has
  enough information to render the canonical search. Surface-equivalent spellings normalize:
  for example, postfix `A = B$` renders as attached `A =$ B`. The lossy
  `proto_to_core_fir` compatibility conversion is downstream of Foolish rendering and is used
  only for `Detailed` delegation and the existing shared evaluator API. No `foolish-core`
  accessor or FIR change is needed.
- **Q3 — RESOLVED (human, 2026-09-02): the output width is CONFIGURABLE, defaulting to 108.**
  See §4.1. The 60-character NK-reason cap and the two-space comment gutter stay fixed for now
  — they are not width, and no need to vary them has appeared; raise them again if the first
  rendered baselines suggest otherwise.
- **Q4 — RESOLVED (human, 2026-09-02): FOOP-36 goes first.** FOOP-26 changes SF/SFF mark
  semantics and makes concatenation an operator, both of which §3 renders; the two FOOPs touch
  the same cases from opposite sides (meaning vs. rendering). Rendering lands first, so
  FOOP-26's diffs appear in Foolish rather than FIR-dump vocabulary and its reviewers can
  predict expected output from its own spec. The argument is §Motivation "Why this should land
  before FOOP-26"; the cost to FOOP-26 is nil because this FOOP moves no FIR, no step rule and
  no step count.
- **Q5 — RESOLVED (human, 2026-09-02, clarified 2026-09-03): conclusive searches render their
  value; inconclusive searches render the canonical search.** Thus `{A=B?=5}` renders `{A=5}`
  when B contains a matching value. If B is constanic but has no match, the arena search is NK,
  has no `ubc_children` result, and still retains B and 5 in `foolish_children`, so it renders
  `{A=B?=5}` with the NK annotation. No provenance marking is needed: the arena preserves the
  search node and its result slot distinguishes success from miss.
- **Q6 — RESOLVED (human, 2026-09-02), and largely defused by the replacement approach.** The
  original concern: `foolish-ubca2/einmo_suite/verified/` holds 179 human-signed artifacts and
  its gate passes today (measured 2026-09-02), so re-rendering those baselines in place would
  break a frozen tier that only a human key could restore.

  **Building `einmo_suite2` alongside removes that problem entirely** — `einmo_suite`'s
  `verified/` tier is never touched and its gate keeps passing (§Motivation "The suite is
  replaced, not edited"; §Test Plan T5b).

  What remains is the *new* suite's attestation, which is an ordinary new-suite question:
  `einmo_suite2/verified/` starts empty. The human's decision stands — **the agent reviews and
  promotes `output` → `checked` case by case, and the human then mass-verifies
  `checked` → `verified` in one pass.** Until that pass, `einmo_suite2` has no verified tier.
  **The agent must not `#[ignore]` a Verified-tier gate** (AGENTS.md), and the human's
  mass-verify is downstream of a real per-case review, never a substitute for one.
- **Q7 — RESOLVED (inspection, 2026-09-03): a conclusive trailing use renders its value.** In
  `misc/sff_resolves_on_each_use`, the trailing anonymous body remains a `Search("^s$")` in
  the arena, but it is CONSTANT and its first UBC child is the CONSTANT `+` result with value
  12. Under §3 it therefore renders `12`. An inconclusive trailing search would retain its
  canonical written search instead.
- **Q8 — RESOLVED (human pointed to the term; FOOP-62 §Terminology confirms it): NK is
  constantew, so the round trip is straightforward.** The vocabulary recovered: **constantew** =
  CONSTANT, INDEPENDENT, NK — constant everywhere, won't change no matter what;
  **non-constantew constanic** = ECONSTANIC, WOCONSTANIC — may change under recoordination.
  Since NK is constantew, re-parsing and re-stepping `1/0` settles NK again, so `R2 == R1` holds
  and NK sits comfortably in §2.1's "Property 2 required" column. §5.1 records this. An earlier
  draft of §5 asserted the round trip closed without having established why — the claim happened
  to be right, but the reasoning was absent, and the human was right to strike it.

## Proposed Next Steps

*Work this FOOP deliberately did not do, recorded so it is not re-derived. Neither is committed
to; both are proposals for a future FOOP to accept or reject.*

### N1. Instrument an additional element on the FIR to aid rendering

**The problem this solves.** This FOOP's renderer reads the FIR and nothing else, and the FIR
is a *semantic* record — it deliberately does not keep how a thing was **written**. Several
distinct source spellings therefore reach the sequencer indistinguishable, and it must pick one
output spelling for all of them, with no way to prefer the one the Foolisher actually typed.

The worked example, verified during this FOOP's development: `b.x` and `b?x` are the same
operation. `Astn::DotSearch` lowers (`fvm_storage.rs`'s `build_fir`) to exactly

```rust
FirSpec::Search { pattern: format!("^{coordinate}$"), anchored: true,
                  forward: false, is_value_search: false, contexted: false }
```

— byte-for-byte what a plain-name `?` produces, with **no record of which spelling was
written**. (FOOP-23 agrees from the language side: "`.` aliases `?` for name search".) So the
renderer cannot render `b.x` for what was written `b.x` and `b?x` for what was written `b?x`;
it can only choose one for both. The same shape of problem recurs for every canonicalization
§4.3 makes — attached vs postfix search forms, `#0` vs `^`, parenthesized vs bare patterns.

**The proposal.** Instrument an additional element on the FIR as a **sequencing aid**, which
the renderer consults when choosing among equivalent spellings. It can be populated from either
of two sources, and both are worth having:

- **From the original Foolish compiler** — what the Foolisher actually wrote, captured at parse
  time and carried through. This is what lets output preserve the author's spelling rather than
  imposing a canonical one.
- **From stepping** — information the evaluator learns that the source never stated, accumulated
  as the FIR is stepped. This is what lets output reflect what *happened*, not merely what was
  typed.

**Why it is its own FOOP.** It adds to the FIR, so it touches `foolish-core` and every
implementor — squarely outside this FOOP's stated blast radius (§FIR Impact: "**None.** No new
FIR variant"). It also changes what "correct output" means for every canonicalization in §4.3,
so it wants its own specification and its own baseline review rather than riding along inside a
rendering FOOP.

### N2. Render a regexp-free anchored backward search in the dot form

The concrete instance of N1, and the reason N1 was noticed. An **anchored** backward name search
whose pattern is a plain identifier would render `b.x` rather than `b?x`, chaining as
`a = b.c.d.e.f.g`.

It was implemented and green during this FOOP's development, then deliberately backed out
(human, 2026-09-03). Without N1's instrument it is a *guess* at which spelling to prefer,
applied uniformly, and it moves a large fraction of baselines (every anchored plain-name
backward search renders `b?x` today) for no gain a reviewer could check. Two details
established while it was in, worth keeping:

- **`canonical_name_pattern`** (`sequencer.rs`) is the right "regexp-free identifier"
  predicate. Note the **unanchored half already ships in this FOOP** — an unanchored plain-name
  search renders as the bare identifier today, via that same predicate. Only the anchored half
  is deferred.
- **The dot must not attach to `=`.** `a =.g b.c.d.e.f` was verified to parse, so attaching is
  *possible*, but it was declined: `howˍis = hw.how` reads better than `howˍis =.how hw`.
  §4.3.1 stands as written — only `^` and `$` ever attach — and a future FOOP implementing the
  dot form should keep it that way.

### N3. Line wrapping, once EOL continuation exists

§4.1.1 puts one statement per line and does **no** wrapping within a line: a long statement
simply runs long. That is not a preference but a consequence — Foolish has no end-of-line
continuation syntax, so there is no way to break a line without either changing how it parses
or inventing a convention the parser does not know. A future FOOP defines EOL continuation
semantics; the wrapping rules that build on them, and the 108-column budget they would honour,
belong with it.

## References

- **FOOP-26** — `foolish-ubca2` marks / concatenation-as-operator / three-beat step. Draft,
  same crate, overlapping cases. §Open Questions Q4 (resolved: FOOP-36 first).
- **FOOP-46** — `BraneConcatOp`, a rewritten concatenation operator with phased search
  resolution. Draft, created the same day, same crate. It changes what concatenation *means*;
  §3.2 here specifies how concatenation *renders*. The two should agree on one point in
  particular — what counts as a **successful merge**, since §3.2 keys its entire two-way split
  on `hs_concatenation()`'s `merged` slot being `Some`. If FOOP-46 changes when that slot is
  populated, §3.2's rendering follows it automatically; if it changes the slot's *shape*, §3.2
  needs revisiting. Worth a read before implementing either.
- **FOOP-62** §9 — the current HFS NYES display rules this FOOP's `Detailed` mode preserves.
- **FOOP-33** — creation original-name rendering (`'True`), preserved verbatim in §3.
- **FOOP-75** §4 — attached-search `=$` canonical spelling, retained in §3.
- **FOOP-23** — search semantics; NK vs ECONSTANIC miss outcomes, which §4/§5 render.
- **FOOP-64** — the einmo three-stage pipeline and gates this FOOP's baselines flow through.
- `foolish-core/src/sequencer.rs` — the current renderer; `Detailed` delegates to it.
- `foolish-ubca2/src/ubca_snapshot_tester.rs` — the einmo adapter that calls the sequencer.
- `foolish-parser/src/lexer.rs` `is_id_sep` — why `ˍ`-mangled names round-trip.
- `AGENTS.md` §"The agent is responsible for correctness"; `foop.md` §"Promotion Review Gate".

## Last Updated

**Date**: 2026-09-03
**Updated By**: Claude Code / claude-sonnet-5
**Changes**: **Warning configuration and the NK/BRANING brane rules.** New
**`SequenceWarnings`** object (§4.1) replaces the single `comment_nk` boolean with one switch
per warning kind — `warn_nk`, `warn_brane_nk`, `warn_woconstanic`, `warn_econstanic`,
`warn_braning`, `warn_iteration_excess` — plus `silent()`/`verbose()` presets. The
**conventional default warns about what is abnormal or unknowable and stays quiet about what is
ordinary**: non-brane NK, BRANING and the step cap are ON; WOCONSTANIC, ECONSTANIC and a
brane's own rollup NK are OFF. New **§4.4**: the step-cap alarm renders as a banner on its own
line ABOVE the program (`!! This Foolish program did not complete stepping within the limit of
9999 steps !!`) rather than trailing the root brace — and ONLY the step-cap alarm, only on a
brane, since other alarms sit mid-statement where a full-line comment would not re-parse. New
**§5.2**: NK reverts to the written expression EXCEPT when the result is a brane, which renders
its contents so every member keeps its own state. New **§5.3**: BRANING always reverts (a
BRANING brane may be self-referential and unroll) but carries `!! BRANING`. Earlier the same
day: **§4.0** brane rollups are not annotated; **§4.1.1** one statement per line with no
wrapping; **§4.3** canonical spellings (attached `^`/`$` only, marker runs, no needless
parens); **§Proposed Next Steps** N1–N3.

The design as it now stands: `foolish-ubca2` gets its own sequencer whose default `Foolish`
mode renders FIR — settled or mid-evaluation — as parseable Foolish. **§0** introduces
the terminology from **AGENTS.md §Foolish Terminology** (the authority): *constanic*,
*constantew* (CONSTANT EveryWhere), and *conclusive* (**Conc**) = CONSTANT/INDEPENDENT.
*Inconclusive* is everything else INCLUDING pre-constanic states; the phrase *inconclusive
constanic* narrows to WOCONSTANIC/ECONSTANIC/NK, which is what §3's rule is stated over. **§3**: when a result is an inconclusive
constanic, render the original expression — the original search, or the op on its parameters;
a conclusive result collapses to its value. **§3.1** explains why that loses nothing; **§3.2**
splits concatenation on whether the merge succeeded. **§4** puts states with no Foolish syntax
in `!!` comments; **§4.1** makes width configurable, default 108; **§4.2** covers einmo input
comment style and the per-suite separator. **§5** derives NK's rendering from §3's predicate
(`1/0`, never `???`), with `warn_nk` as a flag; **§5.1** distinguishes constantew from
conclusive. **§6** keeps `Detailed` delegating unchanged to `foolish_core::FirSequencer`, so
`foolish-ubca` cannot regress.

**The suite is REPLACED, not edited.** `einmo_suite2` is built to become `foolish-ubca2`'s
approval suite: the 179 inputs are copied across and rendered anew, and **the cut-over happens
at the end of the project** — `cargo test` points at the old suite throughout Movements I and
II, so the development procedure does not change until Movement III. `einmo_suite` is then left
frozen and still green as the reference to diff against, and this FOOP does everything except
remove it. That also defuses Q6: the old `verified/` tier is never disturbed.

**The Test Plan is split in two**, because the new suite inherits the old one's job. **Group A**
proves the sequencer (T0 hand-written contract, T1 per-row units covering both sides of §3's
predicate, T2 six-step round trip over a variety of constanic states, T2b pre-constanic, T7
width, T8 separator safety, T9 flags). **Group B** proves the suite is a fit replacement (T3
corpus-wide parse, T4 baseline review, T5/T5b non-regression for `foolish-ubca` and the frozen
old suite, T6 comprehensive, **T10 coverage parity**, **T11 suite integrity**, **T12 value
non-regression across the cut-over**). T10 and T12 exist because "the new suite quietly tests
less" and "a value changed, not just its rendering" are the failure modes invisible from a
green run.

§0.1 surveys all 134 uses of "settled" in `foolish-ubca2` and classifies each by NYES group;
§0.1.1 establishes that `settled_constanic_result` means **constanic** (not constantew — the StayFoolish
path at 902–904 admits ECONSTANIC/WOCONSTANIC), so all three arms of §3's predicate are
reachable. Adds **§3.2.1**: §3.2's two renderings and **FOOP-46's two phases are the same distinction** —
Gathering ↔ the juxtaposition, Joined ↔ the brane — so the rendering is the natural display of
what FOOP-46 builds rather than a constraint on it. One detail is left to settle deliberately:
§4 may delete `FirSpec::ConcatHelper`, and §3.2 renders a merged concatenation and a plain
brane differently, so either the FIR keeps something that distinguishes them or §3.2 is amended
to render them alike. This FOOP **propagates that into both FOOPs** as sections they own (plan
Phase 6.5), so the agreement is on the record in all three places. §0.1.2 lists the predicate per group; the two missing ones and all the renames were split into
**FOOP-56**, scheduled to land before this FOOP.

Resolved: Q1 (out of scope — einmo only), Q3 (configurable width), Q4 (FOOP-36 lands first),
Q5 (dissolved), Q6 (human mass-verifies after per-case review), Q8 (NK is constantew).
Phase 1 resolved Q2 (render directly from the arena) and Q7 (a conclusive trailing use renders
its value) on 2026-09-03.
