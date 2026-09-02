---
foop: D63
title: A Foolish-rendering sequencer for foolish-ubca2 — output that parses back in
author: Claude Code / claude-opus-5 (directed by the human)
status: Draft
type: Standards
created: 2026-09-01
phase: phase-4
supersedes: []
begun: [ ] 
---

# FOOP-36: A Foolish-rendering sequencer for `foolish-ubca2`

FOOP numbering is little-endian; the full rules live in `foop.md` at the repository root —
**read it before creating or editing a FOOP.** The `foop:` front-matter field here is the
big-endian sort key preceded by `D` (`foop: D63`, file `FOOP-36.md`, following FOOP-26's 62).

## Abstract

Today a `foolish-ubca2` einmo OUTPUT is a *debug dump*: `{WOCONSTANIC`, `?(pattern='^x$',
UNANCHORED, ECONSTANIC)`, `Op+(…, …, WOCONSTANIC)`. It faithfully describes FIR internals, and
it is **not Foolish** — it cannot be lexed, let alone parsed. That single fact is what makes
writing an einmo case expensive: a Foolisher can write the INPUT from knowledge of the
language, but can only obtain the OUTPUT by running the evaluator and reading whatever it
emitted, which is precisely the thing the test is supposed to be checking.

This FOOP gives `foolish-ubca2` **its own sequencer**, owned by the crate, whose default mode
renders a FIR — settled **or mid-evaluation** — as **valid Foolish source that parses back
in**. Constants render as their values. A **search renders as the search that was written**
unless its result is a genuine value (CONSTANT or INDEPENDENT) — so `r = b?a.*` stays
`r = b?a.*` when the result is ECONSTANIC, WOCONSTANIC or NK, and collapses to `r = 3` only
when a real value was produced. That is not an answer withheld: handed the search, the next
compiler re-coordinates it itself. What disappears is the FIR machinery — `?(pattern='^y$', UNANCHORED)`,
`Op+(…, WOCONSTANIC)`, the bare NYES tokens.
`{a=1+2}` renders `a = 3` — the operator is spent and its value is the Foolish. No NYES
tokens, no `Op…(`, no `?(pattern=…)`, no `ANCHORED`. `NK` renders as `???` with a brief
parenthetical reason. ECONSTANIC and WOCONSTANIC — states that have no Foolish surface syntax
at all — are rendered as the **value the reader would write**, with the state demoted to a
`!!` comment, which the parser already discards.

Rendering pre-constanic FIR is **kept, not dropped**: einmo debugs as well as approves, and a
half-stepped program must still render as something a Foolisher can read. A pre-constanic node
renders as the program it still is, with `EMBRYONIC`/`BRANING` in a comment (§2.1). And the
existing detailed rendering is **not** removed and **not** changed — it becomes a second,
explicitly-selected mode, for when the FIR's internal shape, rather than its source shape, is
what is under investigation.

## Motivation

### The concrete cost, in the artifacts we have

These are current, committed `foolish-ubca2/einmo_suite/checked/` OUTPUT sections. Each is
the *expected answer* a human is asked to write, review, and sign:

```text
misc/undeclared_identifier.foo    INPUT: {x = non_existent;}
{WOCONSTANIC
  x=?(pattern='^nonˍexistent$', UNANCHORED, ECONSTANIC)
}
```

```text
misc/sff_resolves_on_each_use.foo INPUT: {a=1; b=2; s=<<a+b>>; a=10; s;}
{WOCONSTANIC
  a=1;
  b=2;
  s=<<WOCONSTANIC
      Op+(?(pattern='^a$', UNANCHORED, ECONSTANIC), ?(pattern='^b$', UNANCHORED, ECONSTANIC), WOCONSTANIC)
  >>;
  a=10;
  12
}
```

To write that second OUTPUT by hand a Foolisher must know: that `<<a+b>>` keeps its body
unevaluated and so renders its *interior*, not its value; that the interior's `a` renders as a
`?(…)` search node rather than `1`; that the search's regex is anchored-in-the-string
(`'^a$'`) yet reported `UNANCHORED` (a different sense of "anchored"); that the operator node
prints its own NYES *after* its operands; and that the enclosing brane's opener carries a bare
`WOCONSTANIC` token. **None of that is Foolish.** All of it is FIR-internal vocabulary, and
every bit of it is a chance to write the expected answer wrong in a way that looks plausible.

That is the failure mode this FOOP targets, and it is worse than tedium. AGENTS.md and
`foop.md` require that every promoted OUTPUT line be *justified against the specification* —
but the current rendering makes the reviewer's job "does this match what the evaluator
printed?", which is exactly the question the Promotion Review Gate forbids as a justification.
A rendering the reviewer can independently derive from the language spec changes the gate from
a matching exercise into a reading one.

### What becomes possible

With Foolish-rendering output, the same two cases read:

```text
misc/undeclared_identifier.foo    INPUT: {x = non_existent;}
{
  x = nonˍexistent  !! ECONSTANIC (unfound)
}
```

The search is rendered, not its outcome — §3's rule for a constanic search, and here the
search is unanchored so it reverts to the search alone. An unanchored miss is ECONSTANIC, not
NK: it may still gain a value by recoordination (FOOP-23), so `x` is emphatically not `???`,
and rendering the search is exactly what lets the next compiler discover that for itself.

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

The second is *the program a Foolisher would write to mean the same thing*. `s = <<a+b>>`
still holds a deferred sum, and `a` is re-stated as 10 — both plainly right, and both
predictable from the INPUT by a reader who knows the SFF rules, which is the whole point.

**The trailing line is the one to settle empirically, not by assertion.** It is shown as `12`
above, but §3's rule says a *constanic search* reverts to the search — so if that use site
reaches the sequencer as a search for `s`, it renders `s`, not `12`. Which of the two is
correct depends on what the FIR at that position actually is, and §Open Questions Q7 makes it
a Phase 1 determination. It is exactly the kind of line this FOOP exists to make writable, so
getting it right matters more than getting it settled early.

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

### §1 Two modes, one entry point

`foolish-ubca2` gains a module `src/sequencer.rs` exposing:

```rust
/// How a settled FIR is rendered.
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
    pub fn format(fir: &dyn FirQueryable, mode: SequenceMode) -> String;
}
```

`Detailed` **delegates to `foolish_core::FirSequencer::format`** — it is not a reimplementation
and it is not permitted to drift. This is what keeps the FOOP's blast radius at zero for
existing debugging workflows and for `foolish-ubca`.

`Foolish` is the new renderer, specified below. It is the default because it is what the einmo
adapter and the REPL should show; a caller that wants internals asks for them by name.

### §2 The round-trip property (the definition of correct)

For any input program `P` that `UbcaEvaluator` settles, let `R = format(eval(P), Foolish)`.
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

**`Foolish` mode must render a FIR in ANY state, settled or not.** Einmo is a debugging tool
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

The rule in one sentence: **a constant renders as its value; a constanic search reverts to the
search statement that was written.**

**Constants always render in Foolish.** An integer renders `7`, a merged brane renders its
statements. Wherever a genuine value exists, the value is the Foolish.

**The same predicate governs operators and searches alike.** Both are *processes* with a
result, and both collapse to that result only when it is a genuine value:

> Render the **original expression** — the operator with its operands, or the search with its
> anchor — when its result is **not found**, or is **constanic but neither CONSTANT nor
> INDEPENDENT** (ECONSTANIC, WOCONSTANIC, NK). Otherwise render **the result's value**.

So `3 + 4` renders `7`, but `a + b` whose operands are ECONSTANIC renders `a + b` — operator
and parts — not a value it never reached. This is one rule with two instances, not two rules;
the sections below spell out each.

**A search renders as the original search statement unless its result is a genuine value.** The
test is on the search's `result()`, not on the search's own NYES:

> Render the **original search** when either:
> - `result()` is **not found** (no result at all), **or**
> - `result()` is **constanic but neither CONSTANT nor INDEPENDENT** — i.e. ECONSTANIC,
>   WOCONSTANIC, or NK.
>
> Otherwise — `result()` is CONSTANT or INDEPENDENT — render **the result's value**.

The principle: a search collapses to a value only when it actually produced one. CONSTANT and
INDEPENDENT are values; ECONSTANIC and WOCONSTANIC are "not settled into a value yet, and
context may still change them"; NK is "no value, and none is coming". In none of those three
is there a value to print, so the question that was asked is what gets printed.

Anchoring decides the *shape* of the rendered search, not whether it renders:

- **Unanchored** → the search alone: `?x`, `nonexistent`.
- **Anchored** → the anchor, rendered by these same rules, then the search: `b?a.*`, `a.field`.

Today `r = b?a.*` renders `r=3`, discarding both the anchor and the question. Under this rule
it renders `r = b?a.*` when the result is not a plain value, and `r = 3` when it is.

**Why this is right.** A search reverting to its written form is not information withheld from
the reader — it is the program restated. Handed `b?a.*`, the next compiler performs the search
itself; handed `nonexistent`, it may resolve it in a *new* context, which is exactly what
ECONSTANIC promises (§5.1: ECONSTANIC and WOCONSTANIC are **non-constanew** — their values may
change under recoordination). Collapsing those to a value would freeze an answer the language
says is still open. A rendered search is never a search that was lost; it is one that will be
re-coordinated wherever the output is next read.

What §3 removes is the **machinery**, never the value or the question: the `?(…)` wrapper, the
`pattern='^…$'` regex spelling, `ANCHORED`/`UNANCHORED`, the bare NYES tokens, `Op…(`,
`result=`. Those are FIR-internal vocabulary that no Foolisher would write.

```text
{b = {a1=1; a2=2; a3=3}; r = b?a.*;}

  today    r=3
  §3       r = b?a.*
```

| FIR kind | `Foolish` rendering | Note |
|---|---|---|
| ConstantInt | `7` | unchanged |
| Creation | `⬤`, or its original name (`'True`) | FOOP-33 rule, unchanged |
| Brane | `{ … }` with `;`-separated statements | opener carries **no** state token |
| Statement | `name = body` | `ˍ` (U+02CD) is a valid ident char (lexer `is_id_sep`), so mangled names round-trip |
| Characterized brane | `a'b'{ … }` | unchanged from current `chars` handling |
| Operator, result CONSTANT/INDEPENDENT | its **value** (`3 + 4` → `7`) | it produced a genuine value; the operator is spent |
| Operator, result missing / ECONSTANIC / WOCONSTANIC / NK | the **operator and its parts** (`a + b`, `1/0`) | same predicate as Search |
| Search, result CONSTANT/INDEPENDENT | the **result's value** | the search produced a genuine value |
| Search, result missing / ECONSTANIC / WOCONSTANIC / NK | the **original search** — unanchored `?x`; anchored `b?a.*` (anchor, then search) | no value was produced; §3.1 |
| Index (`#N` / `^` / `$`) | same predicate on its result: value, else the index with its anchor (`#-1`, `b#0`, `^`, `$`) | as Search |
| **NK, any kind** | the written expression + `!! NK: …` comment | §5 — NK reverts like every other constanic |
| `???` written in source | `???` | the no-no literal is Foolish; source renders as itself |
| SF `<X>` / SFF `<<X>>` | `<` / `<<` + interior rendered in this same mode + `>` / `>>` | interior is written form, never a NYES dump |
| Concatenation, **merged** | the merged brane `{…}` | the concatenation succeeded; §3.2 |
| Concatenation, **unmerged** | `A B` — each constituent rendered recursively | §3.2; `⨃` is never emitted (not input syntax) |
| **Any kind, pre-constanic** | its written form + `!!` state comment | §2.1, §4 |

The `=$` attached-search spelling (FOOP-75 §4) is retained where it is the canonical *input*
form, since it is Foolish. `=^`/`=$` render as `name =$ value`.

#### §3.1 A rendered search is never a lost search

The rule that a search reverts to its written form — whenever its result is not a genuine
value — can look like it discards an answer the evaluator worked for. It does not, and the
reason is worth stating plainly because it is what makes §2's round-trip meaningful.

A search rendered as a search is one that will be **re-coordinated wherever the output is next
read**. Handed `b?a.*`, the next compiler runs the search against `b` and settles it constanic
exactly as this one did. Handed `nonexistent` — an unanchored miss, ECONSTANIC — it may resolve
it in a *new* context, which is precisely the promise ECONSTANIC makes (FOOP-23: an unanchored
miss may gain a value by recoordination). So the printed program is not a weaker statement than
the FIR; it is the same statement, in the language.

This is also why the rule keys on the **result's** state rather than on the search's own. A
search whose result is ECONSTANIC has *found* something — a statement — but that statement has
not settled into a value, so there is nothing to print but the question. The result chain
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

**Why this is not a special case.** It falls straight out of §3: a merged concatenation has a
value, so render the value; an unmerged one does not, so render it as written, and "as written"
means each constituent gets the same treatment. The only thing §3.2 adds is that the
`merged`-slot is the signal — `hs_concatenation()` returns `(elements, merged)`, and `merged`
being `Some` is exactly the question "did it succeed?".

**`⨃` is never emitted in either case.** The current sequencer prefixes concatenations with
`⨃`, which is not input syntax and would not re-parse.

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
u = {p = 1; q = p + 1;} !! EMBRYONIC
```

The annotation is what makes `Foolish` mode usable for **debugging** (§2.1): a half-stepped
program renders as the program it still is, with how far each part has got in the margin. The
source text is unchanged by stepping; only the comments move. That is a far more legible
debugging artifact than the current dump — and `Detailed` mode remains for the cases where the
FIR's *internal* shape, rather than its source shape, is the thing under investigation.

Because §3 renders written form in every state, the annotation carries information that is
**otherwise unrecoverable from the output** — which is precisely why it is worth emitting.

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
informative annotations in noise. Comments are emitted only for the five states above.

### §4.1 Line width — 108 characters

Rendered output targets a maximum line width that is **configurable, defaulting to 108
characters** — the project's document width (AGENTS.md §Code Style). The current sequencer
hard-codes `LINE_BUDGET = 128` (`foolish-core/src/sequencer.rs` line 14); `Foolish` mode makes
it a parameter with 108 as the default.

**How it is configured.** The width rides on `SequenceMode`'s companion, not on a global
constant, so a caller that wants a different width says so at the call site and every nested
render inherits it:

```rust
pub struct SequenceOptions {
    pub mode: SequenceMode,
    /// Target max line width. Default 108 (AGENTS.md §Code Style).
    pub width: usize,
    /// Annotate NK expressions with `!! NK: <reason>` (§5). Default true.
    /// Turned off, an NK expression renders as bare Foolish with no trace
    /// of its NK-ness — which is what a caller wanting pure source wants.
    pub comment_nk: bool,
}

impl Default for SequenceOptions {
    fn default() -> Self {
        Self { mode: SequenceMode::Foolish, width: 108, comment_nk: true }
    }
}
```

`Ubca2Sequencer::format(fir, mode)` stays as the common-case entry point (it builds
`SequenceOptions::default()` with the given mode); `format_with(fir, &SequenceOptions)` is the
form that takes an explicit width. The einmo adapter uses the default — **baselines are
rendered at 108 and nothing else**, or the corpus would not be reproducible.

The budget behaves exactly as the existing one does — it is the **single-line vs multi-line
decision threshold**, threaded down through nested renders as `line_hint` and reduced by the
indent at each level, so a construct that does not fit on one line at its depth breaks across
lines with its body indented. That machinery is already correct; only the constant differs.

**It is a target, not a guarantee.** Three things can legitimately exceed it, and the renderer
must not mangle output to prevent them:

1. **An atom longer than the budget.** A single long identifier, a `???` reason at its 60-char
   cap on a deeply indented line, or a brane name near the limit cannot be split — Foolish has
   no line-continuation syntax, and inventing one would break Property 1 (§2).
2. **A `!!` annotation pushing a line over.** The comment is appended after the statement
   (§4). Wrapping the comment onto its own line would perturb the line-per-statement
   correspondence that makes the output readable, so an annotated line may exceed 108.
3. **Source that was itself over-width.** Concatenation chains and long juxtapositions render
   as written (§3); if the Foolisher wrote a 195-character statement, rendering it faithfully
   reproduces it. Reformatting the user's program is not this FOOP's job.

**Measured against the current corpus** (5,435 output lines across all 179 cases): 20 lines
exceed 108 today and 3 exceed even 128 — confirming the existing budget is already soft. Of
those 20, all but two are the old rendering's own verbosity
(`?(result=…, pattern='^oneˍzero$', UNANCHORED, NK)`), which §3 removes entirely. The two that
remain are echoed input: a 195-character concatenation chain and a 128-character source
comment. So 108 is comfortably reachable for output this FOOP generates, and the exceptions
above cover what is left.

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

NK needs no rule of its own — it falls out of §3's predicate. An NK result is "constanic but
neither CONSTANT nor INDEPENDENT", so **the original expression renders** and the NK-ness goes
in a `!!` comment. `1/0` is the operator instance of that predicate; an NK search is the search
instance. This section exists only because the current renderer treats NK as the one conclusion
worth printing, and it is not.

```foolish
a = 1/0;                !! NK: DIV-BY-ZERO: division by zero
```

Not `a = ???`. The division is the program; that it is unknowable is the evaluator's finding
about the program, and findings go in comments. Rendering `???` would additionally substitute
the *no-no literal* for something the Foolisher never wrote.

**An NK search result renders as the search, not as NK.** This is not a special case — it is
§3's predicate: an NK result is "constanic but neither CONSTANT nor INDEPENDENT", so the
original search renders:

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

**Whether NK is commented at all is a flag**, `SequenceOptions::comment_nk` (§4.1), default
**on**. The two settings serve different readers:

- **On** (einmo's setting): `a = 1/0;  !! NK: DIV-BY-ZERO: division by zero`. A reviewer sees
  both the program and the finding, which is what makes a baseline reviewable.
- **Off**: `a = 1/0;` — pure Foolish, no trace of the evaluator's finding. For a caller that
  wants source rather than a report.

Either way the *expression* is identical; only the annotation moves. The einmo corpus is
rendered with the default, so it is reproducible.

**NK is constanic AND constanew** — see §5.1 for why that matters and what it does not change.

When on, the reason is drawn from `hs_nk()` and follows §4's placement rule:

- Prefixed `NK:` so it is distinguishable from a §4 state annotation at a glance.
- The `Alarm` code included when present: `!! NK: DIV-BY-ZERO: division by zero`.
- **One line**: newlines and the einmo separator `①` replaced by a space (§4.2).
- Truncated to a stated maximum of **60 characters**, with a trailing `…` when longer. Brief is
  the point; the full alarm text stays available in `Detailed` mode.

#### §5.1 NK is constanew; ECONSTANIC is not

FOOP-62 §Terminology (the in-force authority) divides the constanic states:

| Term | States | Meaning |
|---|---|---|
| **constanew** | CONSTANT, INDEPENDENT, **NK** | constant *everywhere* — won't change no matter what |
| **non-constanew constanic** | ECONSTANIC, WOCONSTANIC | value may change when context is recoordinated |

FOOP-62 lists `is_constanew()` as a predicate, but **it does not exist in the code** (verified
2026-09-02): `foolish-core/src/fir.rs` has `is_constanic()` and `is_nnk_constanic()` only. This
FOOP needs no such predicate — §3 keys on constanic, and §5's NK handling keys on `hs_nk()` —
so it does not add one. Noted because the gap is easy to trip over when reading FOOP-62.

This is the precise version of what §3.1 says loosely. When a rendered **ECONSTANIC** search is
re-read in a new context, it may genuinely resolve there — that is non-constanew, and it is why
rendering the search rather than a value is not merely tidier but *necessary*: collapsing it to
a value would freeze an answer the language says is still open.

**NK is the opposite: constanew, so re-coordination changes nothing.** `1/0` is NK here and NK
anywhere. That makes NK the *easy* case for §2's round trip rather than a hard one — `1/0`
renders `1/0`, re-parses, re-settles NK, and `R2 == R1`. The reason it still renders as written
rather than as `???` is not fear of losing an answer; it is simply that `1/0` is the program and
`???` is a different expression the Foolisher did not write.

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

**None.** No new FIR variant, no state-machine change, no serialization change. This FOOP
reads FIR through the existing `FirQueryable` accessors and writes text. (An earlier draft
feared §3 would require marking FIRs by provenance; §Open Questions Q5 records why it does
not — the resolved/unresolved distinction is already structural.)

One **read-only** addition may prove necessary: rendering an unsettled search or operator in
its *written form* (§4) requires the surface spelling. `hs_search()` supplies pattern,
direction and anchoring; `hs_operator()` supplies the glyph and operands. If any kind turns
out not to expose enough to reconstruct its written form, the remedy is a **new
`FirQueryable` default method** returning `Option<String>` — additive, defaulting to `None`,
breaking no implementor. §Open Questions Q2 tracks this; it is now the ONLY way this FOOP could
grow past its stated blast radius, and **it must be reported to the human before it is made,
never taken as an implementation convenience.**

## UBC Step Impact

**None.** The sequencer runs on already-settled FIR. No step rule changes; no evaluation
order changes; no NYES transition changes. Step counts in einmo output are unaffected.

## Test Plan

**T0 — `einmo_suite2`: the hand-written rendering contract.** A NEW suite,
`foolish-ubca2/einmo_suite2/`, holding ONE case (`input/foop/36/rendering_contract.foo`) — a
single brane whose members are sub-branes, one per rendering concern (§3's rows, §3.1's
substitution rule, §4's five annotated states, §5's NK forms). **Its expected OUTPUT is typed
by hand from this specification before the renderer is written**, and the renderer is then
developed until it reproduces what was typed.

This is listed first because it runs first, and because it is **this FOOP's own acceptance
test**. The FOOP claims einmo expectations become writable by a person reading the spec; T0 is
that claim, executed. If it proves impractical, the design is wrong and the right time to
discover that is on one case — not after 179 have been promoted. It also means the renderer is
developed against a *human-authored* target rather than its own output, which is the only way
the later Promotion Review Gate can be more than a matching exercise.

**T1 — Unit tests, `foolish-ubca2/src/sequencer.rs` (tests module).** Per §3 row: one test
per FIR kind asserting the exact rendered string, in both modes. `Detailed` tests assert
byte-equality with `foolish_core::FirSequencer::format` on the same FIR — the delegation
contract of §1, pinned so it cannot silently drift.

**T2 — Round-trip property tests (§2).** The FOOP's load-bearing tests. The procedure, stated
as the six steps a test performs:

1. **Compile** the program `P`.
2. **Step it to finish** (settled).
3. **Output it** in `Foolish` mode → call this `R1`.
4. **Compile `R1`.** That it compiles at all is Property 1; a parse failure fails here.
5. **Step that to finish.**
6. **Output again** → `R2`. **Assert `R2 == R1`.**

That equality is Property 2. It is what "the rendering has reached a fixed point" means
operationally, and it is a far stronger check than eyeballing one rendering: any construct that
renders to something meaning even slightly different will drift on the second pass and the test
catches it, without anyone having to predict the right answer in advance.

Run it over a curated set covering every §3 row. Property 2 is asserted **only** where the FIR
is constanic (§2.1's table) — a pre-constanic FIR legitimately steps further on the second
pass, so only steps 1–4 apply to it.

This procedure is also how §Open Questions **Q7** gets settled empirically: whichever way a
trailing use site renders, `R2 == R1` says whether that rendering is stable.

**T2b — Pre-constanic rendering (§2.1).** Take FIRs stepped a bounded number of steps rather
than to settlement — the crate's stepping entry points make this directly constructible — and
assert of each: it renders in `Foolish` mode, the rendering **parses** (Property 1), the
rendering contains **no NYES token as syntax**, and the state appears **only** inside a `!!`
comment. Idempotence is deliberately NOT asserted here (§2.1). Cover at least one
PREMBRYONIC, one EMBRYONIC and one BRANING node, and one case halted by an `ALARM:` mid-step,
since that is the shape einmo actually captures when debugging.

**T3 — Corpus-wide round-trip.** A single unit test that walks every
`einmo_suite/input/**/*.foo`, evaluates, renders in `Foolish` mode, and asserts the result
parses. This is §2's Property 1 applied to the whole corpus at once and is the cheapest broad
guard against a rendering that is locally fine and globally unparseable. It asserts
parse-ability only — not idempotence — so it stays fast, does not double the suite's
evaluation cost, and remains correct for whatever pre-constanic nodes the corpus's
non-settling cases leave behind (§2.1).

**T4 — Einmo baselines (migration).** Only after T0–T3 pass: all 179 existing `foolish-ubca2`
cases re-render. Every one must go through the Promotion Review Gate. **This is the bulk of
the work and the plan phases it accordingly** (one review sub-phase per suite subdirectory, not
one 179-item list), because a gate whose boxes are checked faster than the cases could be read
is a false record — `foop.md` §"Promotion Review Gate".

**T7 — Line width (§4.1).** Unit tests that a construct exceeding 108 characters at its
indent level breaks across lines with its body indented, and that nesting reduces the budget by
the indent. Plus the three stated exceptions: an unsplittable long atom, an annotated line, and
echoed over-width source each render intact rather than mangled. **Not** a corpus-wide width
assertion — see §4.1.

**T8 — Comment style and separator safety (§4.2).** A unit test asserting the renderer never
emits the configured separator `①` (U+2460) anywhere in its output — chiefly via an NK reason
containing one, which §5 collapses to a space. Plus a check over every `.foo` input this FOOP
authors that the §4.2 layout rules hold (blank line before a full-line comment and none after;
blank lines both sides of a `!!!` fence).

**T5 — `foolish-ubca` untouched.** `cargo test -p foolish-ubca --lib -- einmo_gate_checked`
must pass unchanged, before and after. This is the non-regression invariant, and here it
should hold *trivially* — if it does not, this FOOP has modified shared code it promised not
to, and that is a stop-and-report condition.

**T6 — Comprehensive case.** `foolish-ubca2/einmo_suite/input/foop/36/comprehensive.foo`,
exercising at least one path through every §3 row plus the §4/§5 annotation rules together.

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
- **Q2.** Does every §3 written form reconstruct from existing `FirQueryable` accessors, or
  is one additive default method needed (see §FIR Impact)? Resolve by inspection in Phase 1,
  **before** implementation; if a method is needed, say so in the phase report rather than
  adding it silently.
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
- **Q5 — RESOLVED (human, 2026-09-02): dissolved, by a sharper statement of §3.** The
  question asked whether the renderer must distinguish source-written FIRs from substituted
  ones, and feared that a *consumed* search (`result = {y = 1;}?y` arriving as a bare `1`)
  made §3 unimplementable. Both concerns fall away under the rule the human gave:

  > **A constanic search reverts to the original search statement** — unanchored ones to the
  > search alone, anchored ones to the anchor followed by the search. *Constanic* is the
  > operative word: the state class, not "found" or "unresolved".

  Because a search renders as a search whatever its result chain, there is no case where the
  renderer must recover a written form that evaluation destroyed, and no need for FIR
  provenance marking or a new accessor. And because every rendered search is one the next
  compiler will **re-coordinate**, nothing is lost by printing it — §3.1 states that
  explicitly. **No FIR change; nothing left for Phase 1 to decide beyond Q2.**
- **Q6. `verified/` is populated — how should the re-signing be handled?** (Verified
  2026-09-02: 179 signed artifacts present, `einmo_gate_verified` green.) This FOOP re-renders
  every one of them, which will break that gate, and only a human key can restore it. The
  human must choose, before Movement III:
  - **(a)** Re-attest after review — the human runs
    `einmo promote checked to verified foolish-ubca2/einmo_suite --interactive` once the new
    baselines are reviewed and promoted. Most faithful; costs a human review pass over 179
    cases.
  - **(b)** Land the FOOP with `einmo_gate_verified` red, re-attesting later as a separate,
    scheduled act. Keeps the failure visible and honest, but leaves the tree's highest-trust
    gate failing for a period — which AGENTS.md's "never start Phase+ work when tests are
    broken" rule then makes awkward for whatever comes next.
  - **(c)** Split: re-attest a representative subset immediately, the rest on a schedule.

  **RESOLVED (human, 2026-09-02): option (a) — the human mass-verifies after the agent has
  reviewed.** The agent works the Promotion Review Gate case by case and promotes
  `output` → `checked` as normal; the human then re-attests `checked` → `verified` for the
  whole suite in one pass. So `einmo_gate_verified` is red between Movement III and that
  re-attestation, and that is expected and accepted.

  Two things this does NOT license. The agent still may not `#[ignore]` the gate (AGENTS.md —
  and this FOOP is exactly the case that rule anticipates), and the human's mass verification
  is **downstream of a real per-case review, not a substitute for one**: it presumes the agent
  has already justified each case, which is the gate's whole point.
- **Q7. Does a trailing use site render its value or revert to a search?** §3 says a constanic
  search reverts to the written search; a constant renders as its value. A bare trailing `s;`
  (as in `misc/sff_resolves_on_each_use`, `{a=1; b=2; s=<<a+b>>; a=10; s;}`) is one or the
  other depending on what the FVM leaves at that position — a `SearchFir` for `s`, or the
  constant `12` it resolved to. The committed baseline shows `12` under the OLD rendering,
  which does not settle it, because the old renderer collapsed searches to values anyway.
  **Determine in Phase 1 by inspecting the FIR** — the same inspection that confirms §3's
  dispatch — and record the answer, since it fixes how a large family of trailing-use-site
  lines renders across the corpus. Neither answer is a problem; guessing is.
- **Q8 — RESOLVED (human pointed to the term; FOOP-62 §Terminology confirms it): NK is
  constanew, so the round trip is straightforward.** The vocabulary recovered: **constanew** =
  CONSTANT, INDEPENDENT, NK — constant everywhere, won't change no matter what;
  **non-constanew constanic** = ECONSTANIC, WOCONSTANIC — may change under recoordination.
  Since NK is constanew, re-parsing and re-stepping `1/0` settles NK again, so `R2 == R1` holds
  and NK sits comfortably in §2.1's "Property 2 required" column. §5.1 records this. An earlier
  draft of §5 asserted the round trip closed without having established why — the claim happened
  to be right, but the reasoning was absent, and the human was right to strike it.

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

**Date**: 2026-09-02
**Updated By**: Claude Code / claude-opus-5
**Changes**: Created FOOP-36 — a `foolish-ubca2`-owned sequencer whose default `Foolish` mode
renders FIR (settled **or pre-constanic**) as parseable Foolish source. **§3's rule is now
render-the-expression-as-written, in every NYES state**: a resolved search renders as the
search (`r = b?a.*`, not `r = 3`), because the next compiler re-resolves it and reaches
constanic itself — rendering the value would keep the text parseable while losing the FIR's
shape. §3.1 adds the one exception, which turns on **position, not state**: a FIR substituted
into a use site renders the substituted value, since it has no source text there
(`misc/sff_vs_sf_timing_difference` is the case that proves it). NK is the one conclusion
still stated, with a brief reason; the five states with no Foolish syntax become `!!` comments.
Round-trip splits into Property 1 (parses — universal) and Property 2 (idempotent — constanic
only, §2.1), so einmo keeps its debugging role. `Detailed` mode delegates unchanged to
`foolish_core::FirSequencer`, leaving `foolish-ubca` untouched. Recommends landing before
FOOP-26. Adds §"Plan of Execution for Plan" (per-phase model selection — judgment phases to a
larger model, execution-against-fixed-target phases to a smaller one — and the four
responsibilities that may never be delegated) and **T0**, the hand-written `einmo_suite2`
rendering contract that is written before the renderer exists and is the FOOP's own acceptance
test. Adds **§4.1**: output targets **108** characters (AGENTS.md §Code Style), replacing the
current sequencer's 128 — a soft single-vs-multi-line threshold with three stated exceptions
(unsplittable atoms, `!!`-annotated lines, echoed over-width source) and no corpus-wide width
gate; **T7** tests it. Adds **§4.2** (comment style in einmo inputs; the separator is `①` for
ubca2 suites and `!!` for `foolish-ubca`'s — verified from artifacts, as FOOP-92's text and the
toml comment were stale) with **T8**. Adds **Q6**, a blocking finding: `verified/` is
POPULATED (179 signed artifacts, gate green), not empty as FOOP-16 states, so re-rendering
breaks a frozen tier that only a human key can restore. Adds **§3.2**: concatenation splits on
whether the merge SUCCEEDED — merged renders the merged brane
(`{{a=1}{b=2}{c=3}}` → `{{a=1, b=2, c=3}}`), unmerged renders the juxtaposition with each
constituent recursively simplified (the simplest rendering of `foolish_children`), and `⨃` is
never emitted. Q4 and Q6 are RESOLVED by the human: FOOP-36 goes first; the human mass-verifies
`checked`→`verified` after the agent's per-case review, so `einmo_gate_verified` is expected
red in between.
