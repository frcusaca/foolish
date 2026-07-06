---
foop: 32
title: Value search and contexted (&-prefixed) search — value equality, expression patterns, and searching from a statement's position
author: Atlas <hc.busy@gmail.com>
credits: drafted by Claude Code from Atlas's dictated design (sessions 2026-07-04 / 07-05)
status: Draft
type: Standards
created: 2026-07-04
phase: phase-2
supersedes: []
begun: [x]
      (2026-07-05 17:06)
---

# FOOP-23: Value search and contexted (`&`-prefixed) search

## Abstract

Foolish gains two related capabilities, in three strictly ordered parts:

- **Part A — value search, integer-literal equality.** Search a brane for a statement whose
  *value* equals a sought value, complementing name/pattern search. Six forms: anchored forward
  `a~=value`, anchored backward `a?=value`, unanchored backward `?=value`, and the combined
  name-and-value forms `a~id=value`, `a?id=value`, `?id=value`. Equality is implemented for
  independent integer literals only; any other kind of value in the search *pattern* (in
  particular a brane) is an error.
- **Part B — expression patterns.** The pattern may be an expression: `a~=1+2` searches for `3`;
  `a~=c-d` resolves `c` and `d` by ordinary name search, computes the difference, then performs
  the value search.
- **Part C — contexted search (the `&` prefix).** The existing search operators
  (`.` `?` `~` `#` `^` `$`) and the Part A value operators are all **contextless**: they demand
  their anchor resolve through to a whole brane and search that brane, not *reading* incoming
  context. (They still *provide* context — every result carries its found statement's position;
  contextless means the operator does not read, not that it produces nothing to read.) Part C
  adds a **contexted** twin, formed by the `&` prefix — `&?` `&~` `&#` `&^` `&$` `&~=` `&?=`
  (there is **no `&.`**; see Rejected Alternatives). A contexted search anchors on a *statement's
  position* — the original statement a preceding search found — and searches forward/backward
  from there within that statement's home brane. `&` reads *"…and then, from where that landed,
  search this,"* and `&`-searches stack: `a~step_1 &#1` addresses the statement one past
  `step_1` in its home brane; `a~step_1 &?prep` scans back from `step_1`. This split makes every
  operator's behavior depend only on its spelling, never on a heuristic about the anchor's kind,
  and it resolves the `a.brane_field.x` ambiguity: `.` always deepens (find `x` *inside*
  `brane_field`); `a.brane_field &?x` is how you would instead ask for the `x` *near*
  `brane_field` in `a`. Contexted search requires each resolved search to remember the original
  statement it found; this FOOP adds a second `ubc_child`, a new immutable strong reference
  **`FoolRefFir`**, to carry it.

## Motivation

### Why search for a value you already have?

This question stalled value search for a long time, and answering it is the heart of this FOOP.
If you know the value, the search's *value* result is uninteresting — you had it already. What a
value search actually provides is **context**:

1. **a name** — the found statement gives the sought value a name (an anonymous `4` becomes
   `tmp_a`'s `4`);
2. **a position** — the found statement is a location in a brane, from which neighboring
   statements can be reached.

The position payoff is delivered in this same FOOP by Part C: because a resolved search
remembers its *original* statement (via `FoolRefFir`), a value search composes with contexted
anchoring — `doc~=4&#1` means "the statement *after* the first statement whose value is 4" (the
`&` is required: a plain `#1` would demand a brane and fail on the integer result).
Value search finds the place; contexted anchoring exploits it. The name payoff (extracting the
found statement's name as a first-class value) is deliberately left to a future FOOP — see
Future Work.

### Why integer literals only?

Foolish currently has no defined equality for branes or for searches (the vintage
`docs/vintage_legacy/EQUIVALENCE.md` sketches an operator family — `=s=`, `==`, `===`, `=n=`,
`=v=`, … — but none of it is specified or implemented). Rather than block value search on a
brane-equivalence theory, the MVP implements equality between **independent integer values** and
makes every other pattern kind an explicit error. When brane equivalence is later specified,
value search inherits it without changing surface syntax.

There is of course the strict match. If the equality check is requested for non-integral parameters
'b?=a', there is the stipulation that ubca fvm will allow for referential equality. That is if the
rhs of '=' get_value() is a fir that is the same fir in rust ubca fvm, as a candidate, then it is
equal. This to be documented as specification but to be left unimplemented by this foop. The reason
why it is not i mplemented is when they're not referentially the same object, it is not correct to
say they're unequal.

NK's are never equal to each other EVEN if they're referentially same memory space in rust.

### Prior art being superseded

Two incompatible vintage notations exist and are superseded by this FOOP:

- `docs/vintage_legacy/NAMES_SEARCHES_N_BOUNDS.md` §Value Search: forward `?=`, bulk `?=*`, and
  a stray unexplained colon form (`doc:4 = 10`).
- `docs/vintage_legacy/ADVANCED_FEATURES.md` (and the README operator list): `:` / `::`.

This FOOP's family keeps `?=` but assigns it *backward* semantics (mirroring `?` for names),
introduces `~=` for forward (mirroring `~`), and drops the colon notation entirely. Bulk value
search (`?=*`-style find-all) is out of scope, as is all find-all search (`??`, `//`). The context
retrieving ":" has been replaced by '&' to trigger contexted search.

## Terminology

This FOOP fixes the vocabulary for Foolish's search operators. These terms are **authoritative**
and are to be used consistently in this FOOP, in the code, in AGENTS.md, and in the official
Foolish documentation.

- **Home brane of a FIR** (synonym: **brane of a FIR**) — the first brane reached by walking the
  FIR's `.parent` chain; equivalently, the brane in which the FIR's statement has a correct line
  number. The two phrasings mean exactly the same thing; use "**home brane**" when a second
  brane is also under discussion and the specific one must be named, and "brane of" otherwise.
  (The UBCa accessor is `get_my_brane`. See the AGENTS.md documentation todo.)

- **Contextless Anchored Searches** — the existing search operators `.` `?` `~` `#` `^` `$` and
  the Part A value operators `~=` `?=`. Each demands its anchor resolve *through* to a whole
  brane and searches that brane; it **does not read context** (it does not start from a statement
  position) In prose, shorten to **contextless searches**, or plainly **searches** when no
  contrast with contexted search is needed. *(Full form — "Contextless Anchored Searches" — is
  required at least once where the families are introduced in any document.)*

- **Contexted Anchored Searches** — the new `&`-prefixed operators `&?` `&~` `&#` `&^` `&$`
  (and the contexted value forms `&~=` `&?=`). Each anchors on a **statement's position** and
  searches forward/backward from that statement within its home brane, reading the context its
  anchor provides and itself providing a position to any search that chains off it. Shorthand:
  **`&`-searches**; in prose, **contexted searches**. *(Full form required at least once at
  introduction.)*

- **Value searches** — searches triggered by `=` that match on a statement's *value* rather than
  its name (Parts A/B). A contexted value search may be written **`&=`-search** in shorthand when
  that specific combination must be named.

## Specification

Terminology follows FOOP-62 (UBCa): *anchor* is the expression a search is performed on;
*result* is what the search found. NYES states are the UBCa set (PREMBRIONIC … NK).

### Part A — the operator family (integer-literal equality)

#### A.1 Surface syntax

Six forms, mirroring the name-search family (`~` forward / `?` backward / bare unanchored):

| # | Syntax       | Anchoring  | Direction | Matches a statement where …                        |
|---|--------------|------------|-----------|----------------------------------------------------|
| 1 | `a~=value`   | anchored   | forward   | value equals the pattern (first match in `a`)      |
| 2 | `b?=value`   | anchored   | backward  | value equals the pattern (last match in `b`)       |
| 3 | `?=value`    | unanchored | backward  | value equals the pattern (retrospective, IB/AB)    |
| 4 | `a~PATTERN=value` | anchored   | forward   | name matches `id` AND value equals the pattern     |
| 5 | `a?PATTERN=value` | anchored   | backward  | name matches `id` AND value equals the pattern     |
| 6 | `?PATTERN=value`  | unanchored | backward  | name matches `id` AND value equals the pattern     |

There is **no `.=` alias**. `.` aliases `?` for name search, but `a.=10` reads as a compound
assignment to too many eyes; the alias is explicitly rejected (see Rejected Alternatives). There
is **no unanchored forward form**, for the same reason there is no unanchored forward name
search: Foolish cannot look forward in its own brane.

In the combined forms (4–6), `id` is a name pattern with exactly the semantics of the
corresponding name search (`matches_pattern`: literal match, else anchored regex `^id$`). Both
conditions must hold on the same statement. Note also that value searches demands to be written
without spaces; "?id=value". There's no space allowed within name search pattern, nor '?' nor '='
as those are not valid characters for names. the only way to terminate that search is a non-pattern
character, such as a space ' ', or in this case the '=' which triggers a value search interpretation.

#### A.2 Grammar

New tokens: `TildeEquals` (`~=`) and `QuestionEquals` (`?=`), lexed greedily (a `~` or `?`
immediately followed by `=` and not part of an existing token). The combined forms reuse the
existing `Tilde`/`Question` tokens followed by a name pattern, then `Equals`:

```text
value_search_suffix :=
      '~=' value_pattern            !! form 1  (anchored: as suffix; form 3 has no unanchored '~')
    | '?=' value_pattern            !! forms 2 (suffix) and 3 (prefix/unanchored)
    | '~' name_pattern '=' value_pattern    !! form 4
    | '?' name_pattern '=' value_pattern    !! forms 5 (suffix) and 6 (prefix/unanchored)

value_pattern := arith_expr         !! Part A: must be an integer literal; Part B: any expression
```

`value_pattern` is parsed at arithmetic-expression precedence and does **not** include trailing
search suffixes: in `a~=1+2&#5`, the pattern is `1+2` and `&#5` is a contexted anchor on the
whole value search (Part C). (A bare `a~=1+2#5` would parse `1+2` as the pattern and then apply a
*contextless* `#5`, which demands a brane and fails on the integer result — write `&#5` to walk
from the found statement's position.) To use a search result inside a pattern, parenthesize:
`a~=(b.k)` (Part B).

Disambiguation note: the statement-naming `=` is unaffected. In `r = a~id=4;`, the first `=`
names `r`; the parser is already inside an expression when it reaches `~`, so `id=4` can only be
the name-and-value form. Right, the first '=' on a line says we have now entered RHS text of a
statement. Inside RHS text of a statement there are no assignments, only seraches.

#### A.3 Evaluation semantics

A value search scans candidate statements in its direction (forward from the front, backward
from the rear for anchored; the ordinary retrospective IB-then-AB walk for unanchored), testing
each candidate:

- **Name gate** (forms 4–6 only): the statement name must match the name pattern; otherwise the
  candidate is skipped without inspecting its value. The name gate and value gate are tested
  **together on each candidate in the single scan** — this is what makes `~name=value` correct
  and irreducible to a name-then-value chain; see §C.3.1.
- **Value gate**: the candidate's body must be constanic with an integer value (CONSTANT or
  INDEPENDENT integer). If it is and the integer equals the pattern, the candidate **matches**.
  A settled candidate whose value is not an integer (e.g. a brane) simply **does not match** and
  is skipped — non-integer *candidates* are not an error; only a non-integer *pattern* is.
- **Nye candidate — wait**: if the scan reaches a candidate that passes the name gate but whose
  body is still pre-constanic (nigh), the search cannot yet conclude anything about it: it
  suspends (remains BRANING) and re-scans on a later step. Order is sacred: a forward search
  must report the *first* match, so it may not skip past an unsettled candidate.
- **NK candidate — stop**: if the scan reaches a name-gate-passing candidate whose body settled
  NK, the search itself becomes NK (the NK-stop rule; if the successor to deprecated FOOP-11
  changes that rule globally, value search follows).
- **Miss**: scan completes with no match and no suspensions — anchored: **NK**; unanchored:
  **ECONSTANIC** (recoverable by recoordination/concatenation, exactly like unanchored name
  search).
- **Found**: identical to name search — the found statement's body is constanic-cloned into the
  search's `ubc_children` and the search settles via `nyes_from_found` (found ECONSTANIC/
  WOCONSTANIC → WOCONSTANIC; found CONSTANT/INDEPENDENT → CONSTANT).

The search's own *value* is thus the found statement's value — matching the vintage example
(`r = doc?=4` where `tmp_a = 2*2` gives `r` the value 4). The interesting part, per Motivation,
is using the search as an anchor (Part C).

#### A.4 Pattern restriction and the error case

In Part A the `value_pattern` must be an **independent integer literal**. If the pattern is
anything else — in particular a brane — the value search emits an alarm
(`VALUE-SEARCH-UNSUPPORTED-PATTERN`, severity matching the division-by-zero precedent) and
settles **NK**, with the alarm text naming the offending pattern kind. This is an evaluation-time
alarm, not a parse error: the grammar accepts a full expression from day one so that Part B is a
semantics-only extension.

#### A.5 Part A approval-test inputs (proposed `.foo` snapshot inputs)

Result lines carry a trailing `!!` comment stating the expected value and a short reason
(`a = c+d ; !! 5, c+d adds to 5 if this ran right`). `!!` is a Foolish line comment, so the
lexer strips it — it documents the intent for a human reading the input and does not appear in
or affect the humanized output. These annotations are advisory; the authoritative expected result
is the reviewed `.snap`.

`value_search_forward_and_backward.foo` — both directions; both display `10`, direction is
pinned by unit tests here and made observable in Part C:

```foolish
{
	a = {
		id = 4;
		size = 10;
		depth = 10;
	};
	fwd = a~=10;    !! 10, forward value search finds the FIRST 10 (size)
	bwd = a?=10;    !! 10, backward value search finds the LAST 10 (depth)
}
```

`value_search_name_and_value.foo` — combined form; `tmp_a` and `tmp_b` both match the name
pattern but only one passes the value gate at each end:

```foolish
{
	a = {
		tmp_a = 4;
		size = 10;
		tmp_b = 7;
	};
	four  = a~tmp_.*=4;     !! 4, name matches tmp_.* AND value is 4 (tmp_a)
	seven = a?tmp_.*=7;     !! 7, name matches tmp_.* AND value is 7 (tmp_b)
	none  = a~tmp_.*=10;    !! ???, name gate excludes size even though its value is 10
}
```

(`four` → 4, `seven` → 7, `none` → `???` — the name gate excludes `size` although its value is
10.)

`value_search_unanchored.foo` — unanchored backward, hit and miss:

```foolish
{
	pi = 3;
	e2 = 2;
	found = ?=3;      !! 3, unanchored backward value search finds pi
	nope = ?=9;       !! ⎵⎵, no statement valued 9 → ECONSTANIC (recoverable)
	named = ?e.*=2;   !! 2, name matches e.* AND value is 2 (e2)
}
```

(`found` → 3, `nope` → ECONSTANIC `⎵⎵`, `named` → 2.)

`value_search_pattern_error.foo` — non-integer pattern → alarm + NK; non-integer candidate →
skipped, no alarm:

```foolish
{
	a = {
		inner = {x = 1;};
		n = 5;
	};
	bad = a~={q = 1;};    !! ???, brane PATTERN is unsupported → alarm + NK
	ok = a~=5;            !! 5, brane-valued inner is skipped, n=5 matches
}
```

(`bad` → `???` with `VALUE-SEARCH-UNSUPPORTED-PATTERN` alarm; `ok` → 5, the brane-valued
`inner` was skipped silently.)

### Part B — expression patterns

The `value_pattern` may be any expression. The pattern is a foolish child of the value-search
FIR and is stepped to constanicity **before** any candidate scanning begins:

- Pattern settles to an integer (CONSTANT or INDEPENDENT) → scan proceeds, seeking that integer.
  `a~=1+2` seeks 3. `a~=c-d` resolves `c` and `d` by ordinary (unanchored, retrospective) name
  search *in the value search's own context* — not in the anchor brane `a` — computes the
  difference, then scans `a`.
- Pattern settles NK → the value search is NK.
- Pattern settles to a non-integer (a brane) → alarm + NK, exactly as A.4.
- Pattern settles ECONSTANIC (e.g. `c` not found yet) → the value search waits, like any FIR
  whose child is nigh; recoordination may later supply `c`.

Part A's "independent integer literal" restriction is thus revealed as the degenerate case: a
literal is an expression that is born settled. Part B changes no grammar (A.2 already parses an
expression) and no scanning rules — only the pattern-settling stage is new.

#### B.1 Part B approval-test inputs

`value_search_expr_pattern.foo`:

```foolish
{
	c = 12;
	d = 9;
	a = {
		u = 3;
		v = 5;
	};
	r1 = a~=1+2;      !! 3, pattern 1+2 evaluates to 3, finds u
	r2 = a~=c-d;      !! 3, c-d = 12-9 = 3 (c,d resolve here), finds u
	r3 = a~=c-d+v;    !! ⎵⎵, v does NOT resolve here (only inside a) → pattern ECONSTANIC
}
```

(`r1` → 3 via `u`; `r2` → 3 (c−d=3) via `u`; `r3` → NK or alarm? No — `v` resolves
retrospectively from the pattern's own context, where no `v` precedes: the pattern is
ECONSTANIC, so `r3` waits and, no recoordination arriving, the program settles with `r3`
displaying `⎵⎵`. This pins that pattern names resolve in the search's context, *not* inside the
anchor `a`.)

### Part C — contexted search (the `&` prefix)

#### C.1 Contextless deepening (the existing operators, made explicit)

The existing operators (`.` `?` `~` `#` `^` `$`) and the Part A value operators are
**Contextless Anchored Searches**. Chained, they *deepen*: each demands its anchor resolve
through to a brane and searches that brane. Nothing here is new behavior — this subsection just
states the contract that disambiguates chaining:

> A contextless search's anchor must resolve through to a **whole brane**. If the anchor is
> itself a search, the search's *result value* (`ubc_children[0]`) is that brane, and the
> contextless search operates *inside* it. If the anchor's result value is not a brane, the
> contextless search fails (NK on a settled non-brane, as today).

So in `final = some_brane.some_aspect.some_thing_else.some_value`, each `.` deepens into the
brane the previous `.` found — `some_value` is looked up *inside* `some_thing_else`'s brane, not
among `some_thing_else`'s neighbors. This is the coordinate-access reading, and it is the only
reading for `.`. Sequencing is inherited from ordinary anchor evaluation: a contextless search
does not scan until its anchor has settled constanic-and-found (anchor NK → NK; anchor
ECONSTANIC → the search is ECONSTANIC, rescuable by later recoordination); today's UBCa already
enforces this through task ordering and `resolve_anchor`, and this FOOP only makes it normative.

The `a.brane_field.x` question is thereby settled: it deepens (find `x` *inside* `brane_field`).
To instead reach a neighbor of `brane_field` in `a`, use a contexted search — `a.brane_field
&?x` (C.3).

#### C.2 Two-child `ubc_children` and `FoolRefFir`

Contexted search (C.3) needs to know *where* a preceding search's result came from — a statement
position, not just a value. Today a resolved search holds exactly one `ubc_child`: the constanic
clone of the found statement's body (`ProtoBrane::push_search_result` asserts
`ubc_children.len() <= 1`). The clone is detached — new parent, no memory of where it was found.
This FOOP adds that memory:

> A resolved search has exactly **two** `ubc_children`:
> - `ubc_children[0]` — the constanic clone of the found statement's body. Unchanged from
>   today: it is the search's value, the link followed by result chains, and what the
>   sequencer displays.
> - `ubc_children[1]` — a `FoolRefFir` wrapping a reference **to the original found statement**,
>   with its original parent chain, line number, and home brane intact.

`FoolRefFir` is a new FIR kind:

```rust
/// An immutable reference to another FIR — the "fool's reference".
///
/// Wraps a STRONG (non-weak) FirRef to the original statement a search
/// found. Read-only: no method on FoolRefFir mutates the referent, and it
/// exposes no &mut path to it. It takes no steps and holds no children.
pub struct FoolRefFir {
    pub(crate) core: ProtoBrane,   // no foolish_children, no ubc_children
    referent: FirRef,              // strong Rc — the ORIGINAL statement
}
```

- **Strong, not weak.** The reference must be non-weak: the original statement must remain
  reachable through the search even if its home brane is later restructured (concatenation
  rewrites, FOOP-13 ConcatBrane segmenting, temporary branes going out of scope). A weak
  reference that dies would silently break contexted anchoring. The cost is accepted: strong
  cross-links can form `Rc` cycles in pathological mutual-search programs; those leak until FVM
  teardown, which is acceptable for the MVP and noted in Open Questions.
- **Immutable.** `FoolRefFir` is a window, not a handle: the referent cannot be mutated through
  it. Its own NYES is set to CONSTANT at creation (the reference itself is a settled value even
  while the referent may still be evolving), `fir_op_step` is a no-op, and it never enqueues
  tasks.
- **Invisible to values.** `FirRefExt::value`, result-chain walking
  (`deepest_econstanic_in_chain` follows `ubc_children[0]`), the evaluator's result extraction,
  and the Humanizing Sequencer all continue to read `ubc_children[0]` only. `FoolRefFir` never
  appears in HFS output. Existing snapshots must not change from C.2's bookkeeping alone.

Consequential changes: the `push_search_result` single-entry assertion is replaced by the
two-entry invariant (`[0]` clone then `[1]` FoolRefFir, pushed together); `clear_ubc_children`
clears both; every reader that assumed "at most one" is audited (the plan lists them).

#### C.3 The `&` prefix — Contexted Anchored Searches

Prefixing a search operator with `&` promotes it to its **contexted** twin: `&?` `&~`
`&#` `&^` `&$`, and the contexted value forms `&~=` `&?=`. There is **no `&.`** — `.` is itself
the deepen-into-a-brane operator, and a "contexted deepen from a statement position" has no
distinct meaning; UBCa does not provide it (see Rejected Alternatives). A contexted search does
**not** demand a brane; it demands a **statement position**, and searches from that statement
within the statement's home brane. It reads `&` as *"…and then, from where that landed, search
this."*

**What "that" (the anchor position) is:**

- If the `&`-search's left operand is a preceding search (`a~step_1 &#1`), the anchor position
  is that search's *original statement* — `ubc_children[1]`'s `FoolRefFir` referent — located in
  its home brane.
- If the `&`-search stands alone with no left operand (`&~=10`), the anchor position is **the
  current statement's own position** — the statement being evaluated, in its home brane. ("From
  right here, search forward for value 10.")

**Behavior of each contexted operator**, starting at the anchor statement's position `p` in its
home brane `H` (where `#0` addresses the anchor statement itself):

- **`&#N`** — address the statement at `p + N` in `H` (negative toward the front). Out of range
  → the ordinary anchored-index miss (NK on a settled brane).
- **`&?name`** — scan **backward** through `H` from `p` for `name`.
- **`&~name`** — scan **forward** through `H` from `p` for `name`.
- **`&^` / `&$`** — head/tail of `H` (positional, but `H`-relative; equivalent to `&#`-to-edge).
- **`&~=v` / `&?=v`** — contexted **value** search: forward/backward through `H` from `p` for a
  statement whose value equals `v` (the `&=`-search).

**Stacking.** A contexted search is itself a search, so its result carries a `FoolRefFir` too
(C.2). `a~step_1 &#1 &?x` addresses the statement after `step_1`, then scans back from *there*
for `x`. Chains of `&` walk from position to position.

**Bounds.** A contexted search never leaves the home brane `H` of its anchor statement: scans
and indices are clipped to `H`. Escaping `H` into its IB/AB is **not** part of this MVP; a
distinct future spelling is reserved for it (see Future Work). This is the one substantive
difference from the *contextless* unanchored searches, which do climb IB/AB — contexted search
is deliberately position-local.

**Inclusivity.** `&?` and `&~` scans **include** the anchor statement itself (consistent with
`&#0` addressing it). Confirm against IB search's `line_number − 1` convention — see Open
Questions.

Implementation note: `index_into_brane_relative(brane, stmt_idx, offset)` in
`foolish-ubca/src/fir_kinds.rs` already implements statement-relative addressing for unanchored
seeks; `&#` reuses it with `stmt_idx` taken from the anchor statement's position
(`find_stmt_index` by identity against `H`).

#### C.3.1 Why name+value is an atomic operator, not a chain

It is tempting to read the combined form `b~setting=10` (Part A forms 4–6) as sugar for chaining
a name search into a value search — `b&~setting &~=10` — but **that chain is wrong**, and this is
the reason forms 4–6 exist as a single atomic operator rather than a composition.

Consider `b = {setting = 11; setting = 10;}` and the query "the `setting` whose value is 10":

- **The chain `b&~setting &~=10` fails.** A search returns *one* result and stops. `b&~setting`
  forward-finds the **first** `setting` (value 11) and settles there. The following `&~=10`
  then starts contexted from *that* statement's position and scans forward for value 10 — it
  would find the second `setting` by luck here, but only because a value-10 statement happens to
  lie ahead; it is **not** matching "a statement named `setting` with value 10." Reorder to
  `{setting = 10; setting = 11;}` and the chain finds the first `setting` (10) then scans forward
  and finds nothing (no later 10) → miss, even though a `setting`-named 10 exists. The chain
  cannot express "both conditions on the *same* statement" because the first search has already
  collapsed to a single position before the second predicate is applied.
- **The atomic operator `b~setting=10` succeeds.** Per A.3, it scans candidates once, testing
  the name gate **and** the value gate **on each candidate together**, so it correctly reports
  the `setting` statement whose value is 10 regardless of ordering, and correctly finds the
  *second* `setting` in `{setting = 11; setting = 10;}`. Both predicates travel with the scan;
  neither one gets to "win and stop" ahead of the other.

So `~name=value` (and `?name=value`, `?name=value` unanchored) is a first-class conjunctive
search, not chainable sugar. Positional anchoring still composes *after* it: `b~setting=10&#-1`
is "the statement before the (name `setting`, value 10) statement."

A note on a *possible* future generalization (nice to have, **not** MVP priority): the atomic
scan is really "stream the candidates that pass predicate 1, filtered by predicate 2." One could
imagine implementing `?setting=10` by taking the sequence of `?setting` result **contexts**
(every statement the name search would visit, not just the first) and filtering that stream for
`=10`. That framing — a search that yields a *stream of contexts* which a following predicate
filters — would let name+value fall out of a more general streaming-search mechanism instead of
being a bespoke operator. It is attractive but out of scope here; see Future Work. For the MVP,
forms 4–6 are implemented directly as the single-pass two-gate scan of A.3.

#### C.3.2 One engine: cursor-source × predicate (why the classes share code)

The contextless and contexted families are not separate algorithms. Every search in this FOOP is
the same three-step engine:

1. **Establish a starting cursor** — a `(home_brane, position)` pair to scan from. *This is the
   only step that differs between contextless and contexted.*
2. **Iterate statements** from the cursor in the operator's direction, applying a **match
   predicate** (name-match, value-match, name+value, or index-offset). *Shared.*
3. **On match, record `[constanic_clone, FoolRefFir]`** — the value constanically cloned for
   coordination in the new context and the found statement's position(the context) is recorded
   for possible next-step searches. *Shared.*

Two independent properties fall out, and the earlier draft wrongly fused them:

- **Providing context is universal.** *Every* search result carries its found statement's
  position (the `FoolRefFir`, C.2) — contextless results included. So a contextless search like
  `{a=1;b=2;c=3}~=3` does "provide context": its result is the statement `c=3` *with* a position,
  which a following contexted operator can read. Contextless does not mean context-free output;
  it means the operator does not *read* incoming context.
- **Reading context is the operator's choice, marked by `&`.** A contextless operator
  (`.` `?` `~` `#` `^` `$` `~=` `?=`) sets its cursor from its anchor's **value resolved to a
  brane**, positioned at one end (front for forward, rear for backward). A contexted operator
  (`&`-prefixed) sets its cursor from the **incoming result's statement position**. The
  `FoolRefFir` is always present; contextless operators simply ignore it.

Two degeneracies make the classes blend at their edges (both are consequences of the single
engine, and both are things the snaps below pin):

- **Contexted on a bare brane ≡ contextless.** `{a=1;b=2;c=3}&?c` has no incoming statement
  position to read — its anchor is a literal brane. The cursor then degenerates to the
  contextless starting end (the brane's rear, for backward), so `{brane}&?c` behaves exactly as
  `{brane}?c`. `?` is precisely the special case of `&?` whose context is "the whole brane."
- **Contextless on a contexted result reads the value, not the carried position.** In `X.y`
  where `X` is a search that carried a position, `.y` is contextless: it ignores the position,
  takes `X`'s **value** (which must be a brane), and deepens. The carried context is *available
  but unused* — the operator's spelling decides.

This is why the implementation should be one parameterized engine: "contextless vs contexted" is
two **cursor-sources**; "name vs value vs name+value vs index/head/tail" is different
**predicates**. See FIR Impact for how this collapses the FIR kinds.

##### The two collaborators: Candidate Navigator and Statement Matcher

The engine's core is a single loop over a **stream of candidate statements**, driven by two
collaborators with a clean separation that mirrors the interpretable *meaning* of search:

- **Candidate Navigator** — an object that traverses the FIR tree and *yields the candidate
  statements in the order they must be examined*. It embodies "where search looks and in what
  order": it starts at the cursor the cursor-source established, moves in the search's direction,
  crosses (or refuses to cross) brane boundaries per the operator's anchoring, and clips to the
  home brane for contexted searches. It knows nothing about what is being matched.
- **Statement Matcher** — a narrow interface that, given one candidate statement, **approves or
  rejects** it. It embodies "what counts as a hit": name-match, value-match, name+value,
  index-position, head/tail. It knows nothing about traversal order or tree shape.

  **The candidate is the full statement FIR, not a projection of it.** Each Matcher receives the
  whole statement in its context — the statement FIR itself, its **name**, its **body/value**
  FIR, its **line number**, its **parent** FIR reference (hence its home brane), and its NYES —
  everything reachable from the statement `FirRef`. This completeness is deliberate and load-
  bearing: the predicates need different facets (`Name` reads the name; `Value`/`NameValue` read
  the body; `Index`/`Head`/`Tail` read the position/line number), and a Matcher restricted to
  only the value could implement none of the positional or name predicates. Handing the Matcher
  the full contexted candidate is also exactly what lets matchers *compose* (Future Work): a
  fused predicate can test name, value, and position of the *same* candidate because all three
  are in front of it. A Matcher, being narrow and read-only, does not mutate the candidate — it
  inspects and returns approve/reject.

The core loop is then just: *ask the Navigator for the next candidate; ask the Matcher to
approve or reject; on approval produce `[clone, FoolRefFir]`; otherwise continue* — with the
shared wait-on-nye rule (a nye candidate the Matcher cannot yet judge suspends the search) and
the NK-stop rule living in the loop, not in either collaborator.

**The Navigator carries the correctness contract.** Foolish searches have, by definition, a
**single deterministic order** of examining candidates. The Navigator's obligation is exactly
that the stream it yields is:

1. **Correctly ordered** — the one order the configured search semantics mandate (e.g. backward
   from the cursor for `?`/`&?`, forward for `~`/`&~`, retrospective IB-then-AB for the
   unanchored forms), and
2. **Complete** — it yields *every* candidate that search could legitimately reach, exactly
   once, in that order, and then stops. No reachable statement is skipped; none is repeated; none
   beyond the search's bound is offered.

Given a correct-and-complete Navigator, the Matcher is a pure predicate and the loop is trivially
correct: the search finds the first approved candidate in the mandated order, which *is* the
definition of the search. This factoring is chosen deliberately for the reference implementation
because it is the closest expression of what a Foolish search *means* — the Navigator is the
"where and in what order," the Matcher is the "what qualifies," and the two never leak into each
other. Different FIR kinds supply different Navigators (a brane iterates its statements; a
ConcatBrane, per FOOP-13, iterates its segment series through offset arithmetic; a contexted
search starts its Navigator at the anchor statement's position) while sharing the one core loop
and the same Matcher implementations.

Because the Matcher is standalone and traversal-independent, it also *composes* — conjunctions of
matchers (and a find-all Matcher that collects instead of stopping) run over the same Navigator
and loop unchanged. That is out of scope here but is a direct consequence of this factoring; see
Future Work (Composable matchers, Bulk value search).

#### C.4 Part C approval-test inputs

`contextless_deepening_chain.foo` — `.` deepens; the chain reads *inside* each found brane:

```foolish
{
	some_brane = {
		some_aspect = {
			some_thing_else = {
				some_value = 42;
			};
		};
	};
	final = some_brane.some_aspect.some_thing_else.some_value;    !! 42, each . deepens INTO the found brane
}
```

(`final` → 42. This is contextless throughout — no `&` — and pins that `.` deepens.)

`contexted_index.foo` — `&#` anchored on a search, self and forward:

```foolish
{
	recipe = {
		steps = {
			prep_var = 7;
			step_1 = 40;
			bake = 9;
		};
	};
	stage = recipe.steps~step_1;      !! 40, contextless forward name search finds step_1
	self = recipe.steps~step_1&#0;    !! 40, &#0 is the found statement itself
	after = recipe.steps~step_1&#1;   !! 9, &#1 steps one forward in steps → bake
	oob = recipe.steps~step_1&#5;     !! ???, &#5 runs off the end of steps
}
```

(`stage` → 40, `self` → 40, `after` → 9, `oob` → `???`. Note `recipe.steps~step_1` is
contextless — it searches inside `steps` for `step_1`; the trailing `&#N` is contexted, working
from `step_1`'s position in `steps`.)

`contexted_search.foo` — `&?`/`&~` anchored on a search, bounded by the home brane:

```foolish
{
	outside = 99;
	recipe = {
		steps = {
			prep_var = 7;
			step_1 = 40;
			bake = 9;
		};
	};
	before = recipe.steps~step_1&?prep_var;   !! 7, &? scans back from step_1 → prep_var
	afterv = recipe.steps~step_1&~bake;       !! 9, &~ scans forward from step_1 → bake
	escape = recipe.steps~step_1&?outside;    !! ???, outside is beyond home brane steps, unreachable
}
```

(`before` → 7, `afterv` → 9, `escape` → `???` — `outside` lives beyond the home brane `steps`
and a contexted search may not leave it.)

`contexted_value_payoff.foo` — the Motivation's payoff, direction made observable via `&=`:

```foolish
{
	doc = {
		tmp_a = 4;
		c = 30;
		tmp_b = 4;
	};
	after_first_4 = doc~=4&#1;     !! 30, ~=4 finds tmp_a (first 4), &#1 → next stmt c
	before_last_4 = doc?=4&#-1;    !! 30, ?=4 finds tmp_b (last 4), &#-1 → prev stmt c
}
```

(Both → 30, but via different found statements: `doc~=4` found `tmp_a` (forward), then `&#1`
stepped one past it; `doc?=4` found `tmp_b` (backward), then `&#-1` stepped one before it. Unit
tests also pin the found indices. Here `doc~=4` / `doc?=4` are contextless value searches over
`doc`; the trailing `&#N` is contexted.)

`name_value_atomic.foo` — the atomic name+value operator finds the right statement where a chain
would not (spec §C.3.1); duplicate names with different values:

```foolish
{
	b = {
		setting = 11;
		mid = 0;
		setting = 10;
	};
	atomic = b~setting=10;       !! 10, name=setting AND value=10 in one scan (the SECOND setting)
	before = b~setting=10&#-1;   !! 0, &#-1 steps back from the matched setting → mid
}
```

(`atomic` → 10 — the *second* `setting`, matched by name-and-value in one scan; a
`b&~setting &~=10` chain would instead settle on the first `setting` (11) and then value-scan
from there, which does not mean "named `setting` and valued 10". `before` → 0, the statement
before the matched `setting`, showing positional anchoring composes after the atomic operator.)

The next four snaps are the cursor-source/predicate cases from §C.3.2 — the subtle blends the
engine must answer correctly.

`contextless_result_provides_context.foo` — a **contextless** value search provides a position
that a following **contexted** index reads:

```foolish
{
	r = {a = 1; b = 2; c = 3;}~=3&#-1;    !! 2, ~=3 (contextless) finds c, &#-1 reads its position → b
}
```

(`r` → 2. `~=3` is contextless — it demanded the inline brane and forward-found `c=3`; but its
*result* carries `c`'s position, and `&#-1` reads it, stepping back one to `b=2`. This pins that
contextless output still provides context. Note `&#-1`, not `#-1`: a plain `#-1` here would be a
contextless index demanding a brane and would fail on the integer `3`.)

`contexted_on_bare_brane_degenerates.foo` — a **contexted** search whose anchor is a literal
brane behaves as its contextless twin:

```foolish
{
	viaContexted = {a = 1; b = 2; c = 3;}&?c&#-1;   !! 2, &?c on a bare brane degenerates to ?c → c, &#-1 → b
	viaPlain     = {a = 1; b = 2; c = 3;}?c&#-1;    !! 2, same result via plain ?c → c, &#-1 → b
}
```

(Both → 2. `{…}&?c` has no incoming statement position, so its cursor degenerates to the brane's
rear — identical to `{…}?c`; both find `c=3`, then `&#-1` steps back to `b=2`. `&?` ≡ `?` when
the anchor is a bare brane.)

`contextless_on_contexted_reads_value.foo` — a **contextless** `.y` after a contexted `&~b`
reads the found statement's *value* (a brane) and deepens, ignoring the carried position:

```foolish
{
	src = {
		a = 1;
		b = {z = -1; y = -2;};
		c = 3;
	};
	r = src&~b.y;    !! -2, &~b finds b's brane, .y (contextless) deepens into it → y
}
```

(`r` → -2. `src&~b` is contexted — anchored on `src` (a bare brane, so it degenerates to forward
name search) — and finds the statement `b`, whose value is the brane `{z=-1;y=-2;}`, carrying
`b`'s position. Then `.y` is contextless: it *ignores* the carried position, takes `b`'s value
(the brane), and deepens to find `y` → -2. If instead one wrote `src&~b&?a`, the `&?a` would read
`b`'s position and scan back through `src` for `a` → 1. Same anchor, opposite consumption of
context, decided purely by `.` vs `&?`.)

`mixed_chain_walk.foo` — a longer chain mixing all three consumption modes, to pin ordering:

```foolish
{
	doc = {
		intro = 100;
		body = {
			p1 = 10;
			p2 = 20;
			p3 = 10;
		};
		outro = 300;
	};
	firstTen   = doc.body~=10;         !! 10, deepen into body, forward value search → p1
	afterFirst = doc.body~=10&#1;      !! 20, &#1 from p1 → p2
	lastTen    = doc.body?=10;         !! 10, deepen into body, backward value search → p3
	neighborUp = doc.body?=10&#-1;     !! 20, &#-1 from p3 → p2
	deepThenBack = doc.body&~p2&?p1;   !! 10, deepen to body, &~p2 finds p2, &?p1 scans back → p1
}
```

(`firstTen` → 10 (`p1`, contextless forward value search inside `body`); `afterFirst` → 20
(`&#1` from `p1`'s position → `p2`); `lastTen` → 10 (`p3`, backward); `neighborUp` → 20 (`&#-1`
from `p3` → `p2`); `deepThenBack` → 10 — `doc.body` deepens to the `body` brane, `&~p2` on a bare
brane degenerates to forward-find `p2`, then `&?p1` reads `p2`'s position and scans back for `p1`
→ 10. Every step is one of the three cursor-source/consumption cases from §C.3.2.)

## FIR Impact

Per §C.3.2, the search operators are **one parameterized engine** — the `ContextfulSearch`
matching engine — not a kind per operator. Its core is a single loop over a **stream of candidate
statements** produced by a **Candidate Navigator** (traverses the FIR tree, yielding candidates
in the mandated deterministic order — the correct-and-complete stream contract of §C.3.2) and
judged by a **Statement Matcher** (a narrow approve/reject predicate). The engine is parameterized
by a **cursor-source** (which fixes where the Navigator starts — contextless: anchor's
value-as-brane at one end; contexted: incoming result's statement position) and a **match
predicate** (which Matcher — name / value / name+value / index / head / tail). Implementation
strategy (see the plan): build and prove the engine with the *new* features first (value, then
contexted search — no legacy snapshots at risk), **then backfit** the existing contextless
`SearchFir`/`IndexFir`/`HeadTailFir` onto it piecewise, each step gated on zero snapshot diffs.
The end state is a single scan implementation under every search operator.

- **`SearchFir` is generalized (not replaced)** (`foolish-ubca/src/fir_kinds.rs`): it gains a
  `cursor: CursorSource` field (`Contextless` | `Contexted`) and its `pattern`/`anchored`/
  `forward` grow into a `predicate: SearchPredicate`. The Navigator and Matcher are the two
  collaborators the engine loop drives:

  ```rust
  enum CursorSource { Contextless, Contexted }   // fixes where the Navigator starts

  // Candidate Navigator: yields candidate statements (each as the full statement FirRef, in the
  // mandated order, complete, then stops). Implemented per source FIR kind (brane; ConcatBrane
  // via segment offsets; contexted-from-a-position). Knows nothing about what is being matched.
  trait CandidateNavigator {
      // The full statement FIR — name, body/value, line number, parent, NYES all reachable.
      fn next_candidate(&mut self) -> Option<FirRef>;
  }

  // Statement Matcher: pure approve/reject given the FULL candidate statement (not a projection).
  // `matches` receives the whole `candidate: &FirRef`; predicates read whatever facet they need
  // (name / body / line-number / parent). Knows nothing about traversal order.
  enum SearchPredicate {
      Name { pattern: String },                  //  ?name  ~name   (and .name) — reads name
      Value { pattern: FirRef },                 //  ?=v    ~=v      — reads body/value
      NameValue { name: String, value: FirRef }, //  ?name=v ~name=v (atomic, §C.3.1) — name+body
      Index(i32),                                //  #N              — reads line number/position
      Head, Tail,                                //  ^  $            — reads position
  }
  // fn matches(&self, candidate: &FirRef, ctx: &ScanCtx) -> MatchOutcome  // Approve/Reject/Wait
  ```

  The step is: (1) establish the cursor from `cursor`+anchor; (2) scan in `forward`/backward
  applying `predicate.matches(candidate)`; (3) on match push `[clone, FoolRefFir]`. Value/
  NameValue predicates first settle their `value` child (Part B); the wait-on-nye and NK-stop
  rules live in the shared scan, not per predicate. `IndexFir`/`HeadTailFir` fold into the
  `Index`/`Head`/`Tail` predicates (or delegate to them) so there is one scan loop.
- **New FIR kind `FoolRefFir`** (C.2): no children, no steps, CONSTANT at creation, strong
  `referent: FirRef`, no mutation path to the referent.
- **Resolved shape**: every search settles `ubc_children == [clone, FoolRefFir]` (was `[clone]`)
  regardless of cursor-source or predicate — this is what makes providing-context universal
  (§C.3.2) and lets `&` stack on any search. `ProtoBrane::push_search_result`'s single-entry
  assertion becomes the paired two-entry invariant.
- **NYES tables**: the generalized `SearchFir` keeps `SearchFir`'s progression (PREMBRIONIC →
  EMBRYONIC/BRANING → {WOCONSTANIC, CONSTANT, ECONSTANIC, NK}) with BRANING suspension while a
  value-predicate pattern or a nye candidate is pending. `FoolRefFir` is born CONSTANT
  (terminal). Both get `*_nyes_transitions` tests (mandatory per AGENTS.md); predicate/cursor
  variants are exercised by scan unit tests, not separate kinds.
- **Serialization/HFS**: `FoolRefFir` is never sequenced; HFS output keyed to `ubc_children[0]`
  is unchanged. Existing approved snapshots must be byte-identical after the C.2 bookkeeping.
- **Alarms**: new alarm code `VALUE-SEARCH-UNSUPPORTED-PATTERN` (A.4/B), emitted through the
  existing alarm sink alongside the NK settlement, mirroring `DIV-BY-ZERO`.

## UBC Step Impact

- **The generalized `SearchFir::fir_op_step`** runs the three-step engine of §C.3.2:
  - PREMBRIONIC — push the anchor task (if any) and, for Value/NameValue predicates, the pattern
    task.
  - Establish cursor: **Contextless** → `resolve_anchor` to a brane, cursor at front (`forward`)
    or rear; if the anchor's value is not a brane → NK (unchanged contextless behavior).
    **Contexted** → the incoming result's `ubc_children[1]` `FoolRefFir` referent, its home brane
    `H`, and its index `p` via `find_stmt_index`; if the anchor is a **bare brane with no
    incoming position**, degenerate to the contextless cursor (§C.3.2 — `&?` ≡ `?` there).
  - Scan from the cursor applying the predicate, with the shared wait-on-nye and NK-stop rules;
    `Index`/`Head`/`Tail` via `index_into_brane_relative` (`#0`/`&#0` = the cursor statement).
    Contexted scans are clipped to `H`.
  - On match, `handle_found` pushes `[clone, FoolRefFir]` and settles via `nyes_from_found`;
    miss → anchored/contextless NK, unanchored ECONSTANIC.
- **Contextless chaining/sequencing (C.1)**: normative wait states, unchanged mechanism — anchor
  NK → NK; anchor ECONSTANIC → ECONSTANIC; anchor nigh → remain BRANING; anchor
  constanic-and-found → the contextless operator deepens into the found brane's value.

## Test Plan

Tests are written **first** in each part (project rule):

- **Engine (A0)**: pure `ContextfulSearch` unit tests with a hand-built candidate stream — no
  parser/FIR — for the `Name`/`Index` predicates first, then `Value`/`NameValue`: match/miss/wait
  under an explicit `(cursor, direction, predicate)`.
- **Part A**: unit tests for the `Value`/`NameValue` predicates via the engine — direction (which
  statement index matched, forward vs backward), name-gate interaction, nye-candidate suspension,
  NK-candidate contagion, non-integer candidate skipping, non-integer pattern alarm+NK,
  unanchored ECONSTANIC miss vs anchored NK miss; `search_fir_nyes_transitions` (value predicate)
  via `assert_progression`. Approval inputs from A.5.
- **Part B**: unit tests for pattern-settling (expression pattern, NK pattern, ECONSTANIC
  pattern waits, pattern names resolve in search context not anchor brane). Approval inputs
  from B.1.
- **Part C**: unit tests for the two-child invariant, `FoolRefFir` immutability and strong
  liveness (original brane dropped → referent still reachable), `fool_ref_fir_nyes_transitions`;
  the `Contexted` cursor-source — `&#`/`&?`/`&~` including home-brane bounds clipping, `&#0`
  self-identity, `&`-standalone anchoring on the current statement, and `&` stacking; the §C.3.2
  degeneracies (`&?` on a bare brane ≡ `?`; contextless-on-contexted reads value/deepens); that
  plain contextless `#`/`?`/`~`/value-search on a non-brane result still fails NK (the split is
  real); a full-suite snapshot run proving **zero diffs** from the C.2 bookkeeping before the
  contexted operators land. Approval inputs from C.4.
- **C-backfit**: each legacy scan (`IndexFir`/`HeadTailFir`/`SearchFir`/unanchored seek)
  migrated onto the engine with a **zero-diff** full snapshot run + green unit tests as the gate.
- All snapshot changes go through the human review workflow (`.snap.new` → `foolish_review.sh`);
  no auto-acceptance.

## Rejected Alternatives

### A. `.=` as an alias for `?=`

`.` aliases `?` in name search, so symmetry suggests `a.=10`. Rejected: `.=` reads as a
compound-assignment operator to anyone arriving from mainstream languages, and the alias adds no
capability. One spelling per direction: `~=` forward, `?=` backward.

### B. Colon notation (`doc:4`, `::`) from the vintage docs

Rejected: the colon carries no directional information, collides with future type/coordinate
uses of `:`, and the two vintage docs already disagree with each other (`?=` vs `:`). This FOOP
supersedes both.

### C. A separate FIR kind per operator (value / contexted) instead of one engine

An earlier draft proposed separate `ValueSearchFir` / `ContextedSearchFir` kinds. Rejected in
favor of the single `ContextfulSearch` engine (§C.3.2): the operators differ only in cursor-source
and predicate, both of which are data, so distinct kinds would duplicate ~80% of the scan. The one
engine is also the closest expression of what search *means* (Navigator + Matcher).

### D. Weak reference in `FoolRefFir`

Rejected by design requirement: a weak referent that dies under brane restructuring would make
contexted anchoring silently fail; the strong reference's cycle risk is bounded and accepted
for the MVP.

### E. Infer contextless-vs-contexted from the anchor's kind (no `&`)

The earlier draft let a plain `#`/`?`/`~` silently go positional whenever its anchor happened to
be a search result, and deepen when it was a brane. Rejected: the same spelling then meant two
different things depending on a runtime property of the anchor, which is exactly what made
`a.brane_field.x` ambiguous. The explicit `&` prefix makes behavior a pure function of spelling —
`.`/`?`/`~`/`#` always deepen (demand a brane); `&`-forms always navigate from a statement. One
extra character buys unambiguous, locally-readable code.

### F. A `&.` contexted-deepen operator

The `&` prefix has a twin for every other operator, so symmetry suggests `&.`. Rejected: `.` is
*already* the deepen-into-a-brane operator, and "deepen from a statement position" collapses back
to plain `.` on the found statement's value — there is no distinct behavior for `&.` to name.
UBCa does not provide `&.`; use `.` (deepen) or `&?`/`&~` (navigate neighbors) as appropriate.

### G. New search-result naming sugar (`=$`, `=^`, `=#N`, `=.x`, …)

Vintage designs (`NAMES_SEARCHES_N_BOUNDS.md` "Naming Search Results", and the current
implementation's existing `a$=b` / `a^=b` meaning `a = b$` / `a = b^`) fold a search suffix into
the naming operator `=`. This FOOP **adds no new naming sugar** and introduces none of the extra
forms those designs sketch. Each such form would require additional Humanizing-Sequencer work to
render correctly, and value/contexted search deliver their capability through ordinary expression
syntax without it. We keep the *existing* `a$=b` / `a^=b` (already implemented) and choose not to
extend the family for UBCa right now; a later FOOP may revisit it once the sequencer cost is
worth paying.

### H. Do nothing

Value search remains a two-notation vintage sketch with no rationale, and contexted navigation —
needed independently for search-context work — has no substrate (the clone forgets where it came
from). The `FoolRefFir` memory plus the `&` prefix is the smallest change that makes a
statement's *position* a first-class, unambiguously-spelled product of search.

## Open Questions

- **Inclusivity of contexted scans**: `&?prep_var` starts *at* the anchor statement (inclusive,
  consistent with `&#0` = the anchor statement itself) — confirm BDFL intent, or should backward
  scan start at position −1 (strictly before), matching IB search's `line_number − 1` convention?
- **Standalone `&`**: a bare `&~=10` (no left operand) anchors on the *current statement's*
  position. Confirm this is wanted vs. making a left-operand-less `&` a parse error.
- **`FoolRefFir` NYES**: CONSTANT (chosen — the reference is a settled value) vs INDEPENDENT
  (it has no context dependencies of its own). Cosmetic unless something dispatches on it.
- **Cycle mitigation**: is MVP leak-until-teardown acceptance enough, or should the FVM own a
  registry of FoolRefFirs for explicit teardown?
- **Equality maturation**: when brane equivalence (EQUIVALENCE.md's descendants) is specified,
  which equivalence does `~=` adopt by default — `=v=` (value), `==` (post-evaluation), or
  parameterized?
- **Escape spelling**: the beyond-home-brane contexted search (climb `H`'s IB/AB) needs a
  spelling distinct from `&`; reserved but unnamed — Future Work, separate FOOP.

## Future Work (non-MVP, explicitly out of scope)

- **Name extraction**: a way to obtain the found statement's *name* as a value (the second half
  of the context payoff; the vintage `|` "name at cursor" modifier is prior art).
- **Escaping contexted search**: a contexted search that leaves the anchor's home brane into its
  IB/AB (needs a spelling distinct from the home-brane-bounded `&`).
- **Streaming-context search** (nice to have, low priority): generalize the atomic `~name=value`
  operator (§C.3.1) into a composable filter — a search that yields the *stream of contexts* a
  name search would visit (not just the first), which a following predicate filters. `?setting=10`
  would then be "stream `?setting`'s result contexts, keep those where `=10`", and name+value
  would fall out of the general mechanism rather than being a bespoke operator. Attractive but not
  required for the MVP, where forms 4–6 are the direct single-pass two-gate scan.
- **Composable matchers** (a consequence of the Navigator/Matcher factoring, §C.3.2 — recorded as
  a possibility, **not** an MVP feature): because the Statement Matcher is a standalone
  approve/reject predicate independent of traversal, matchers *compose* with no change to the
  Candidate Navigator or the core loop. A conjunction operator — say `&&` — could attach the next
  search's matcher to the previous one, so `b?=2 && ~a.* && ~.*v` would run *one* backward
  traversal of `b` whose Matcher approves a candidate only if it is valued 2 **and** its name
  matches `a.*` **and** its name matches `.*v` — i.e. a single fused predicate `(=2 ∧ name~a.* ∧
  name~.*v)`, not a chain of separate searches. The atomic name+value operator (§C.3.1) is then
  seen as the hard-wired special case `name=value` of a general matcher conjunction; disjunction
  and negation are the obvious further extensions. This is purely a downstream opportunity the
  factoring opens up (the "simultaneous matches" idea); it needs its own FOOP for syntax,
  precedence, and semantics before it is anything more than a possibility.
- **Bulk value search** (`?=*`-style find-all), together with all find-all search. Under the
  Navigator/Matcher factoring this is just a Matcher that *collects* rather than stopping at the
  first approval — same Navigator, same loop — which is a further argument for the split.
- **Non-integer equality**: branes, searches, characterized values — blocked on an equivalence
  FOOP.

## References

- Prior FOOPs: FOOP-4 (bare identifiers → SearchFir; Final), FOOP-62 (UBCa two-store
  ProtoBrane, anchor/result terminology; Final), FOOP-13 (ConcatBrane — restructuring that
  motivates the strong referent), deprecated FOOP-01/FOOP-11/FOOP-51 (anchored-search
  dereferencing, NK-stop, AB resolution — their successors should stay consistent with this
  FOOP's C.1 sequencing rule).
- Vintage docs: `docs/vintage_legacy/NAMES_SEARCHES_N_BOUNDS.md` §Value Search,
  `docs/vintage_legacy/ADVANCED_FEATURES.md` §Search System, `docs/vintage_legacy/EQUIVALENCE.md`.
- Code: `foolish-ubca/src/fir_kinds.rs` (`SearchFir`, `IndexFir`, `matches_pattern`,
  `nyes_from_found`, `index_into_brane_relative`, `FirRefNavExt`, `get_my_brane`),
  `foolish-ubca/src/proto_brane.rs` (`push_search_result` single-entry assertion — replaced),
  `foolish-parser/src/token.rs` / `parser.rs` (Tilde/Question tokens, new `~=`/`?=` and `&`
  tokens, search suffix parsing).

## Last Updated

**Date**: 2026-07-05
**Updated By**: Claude Code 2.1.119 (Claude Code); Fable 5 (claude-fable-5)
**Changes**: Per Atlas: removed `&.` from the contexted operator family (Abstract + §C.3) and
recorded it as Rejected Alternative F (`.` already deepens; a contexted deepen has no distinct
meaning; UBCa does not provide `&.`). Added Rejected Alternative G: no new search-result naming
sugar — keep the existing `a$=b`/`a^=b` (already implemented), add none of the vintage
"Naming Search Results" forms, because each needs extra Humanizing-Sequencer work; may revisit
later. Corrected the Abstract to say contextless searches DO provide context (every result
carries a position) but do not *read* it. Updated stale Rejected Alternative C (was "separate
kind"; now argues for the one `ContextfulSearch` engine). Added trailing `!! value, reason`
result-line comments to all 12 approval-test `.foo` inputs, with a note at A.5 that `!!` is a
stripped line comment documenting intent, not affecting output.

**Date**: 2026-07-05
**Updated By**: Claude Code 2.1.119 (Claude Code); Fable 5 (claude-fable-5)
**Changes**: Made explicit (§C.3.2 + FIR Impact sketch) that the Statement Matcher receives the
**full candidate statement FIR** — name, body/value, line number, parent/home brane, NYES, all
reachable from the statement `FirRef` — not a value-only projection. Noted this is load-bearing:
positional and name predicates need those facets, and it is what lets matchers compose over the
same candidate. Added an A0 plan test asserting each predicate reads its own facet and that a
value-only projection is not what is passed.

**Date**: 2026-07-05
**Updated By**: Claude Code 2.1.119 (Claude Code); Fable 5 (claude-fable-5)
**Changes**: Recorded **composable matchers** as a downstream possibility of the Navigator/Matcher
factoring (Future Work + a pointer at §C.3.2): because the Matcher is a standalone
traversal-independent predicate, matchers compose (a hypothetical `&&` conjunction like
`b?=2 && ~a.* && ~.*v` = one traversal, fused predicate; name+value is its hard-wired special
case; disjunction/negation follow) and a find-all is just a collecting Matcher — same Navigator,
same loop. Explicitly NOT an MVP feature; needs its own FOOP.

**Date**: 2026-07-05
**Updated By**: Claude Code 2.1.119 (Claude Code); Fable 5 (claude-fable-5)
**Changes**: Refined §C.3.2 and FIR Impact to name the engine's two collaborators — the
**Candidate Navigator** (traverses the FIR tree, yields candidates in the single deterministic
mandated order, **complete**: every reachable candidate exactly once then stops — this ordering +
completeness is the load-bearing correctness contract) and the **Statement Matcher** (narrow
approve/reject predicate). Stated that the factoring is chosen for the reference implementation
because it is the closest expression of what a Foolish search *means*, and that different FIR
kinds supply different Navigators (brane iteration; ConcatBrane segment offsets per FOOP-13;
contexted-from-a-position) over one core loop. Added the `CandidateNavigator` trait to the FIR
Impact code sketch.

**Date**: 2026-07-05
**Updated By**: Claude Code 2.1.119 (Claude Code); Fable 5 (claude-fable-5)
**Changes**: Added §C.3.2 (one engine: cursor-source × predicate) — providing context is
universal (every result carries a `FoolRefFir` position), reading context is the operator's
choice marked by `&`; documented the two degeneracies (`&?` on a bare brane ≡ `?`; contextless
`.y` on a contexted result reads the value and deepens). Added four worked-example snaps
(`contextless_result_provides_context`, `contexted_on_bare_brane_degenerates`,
`contextless_on_contexted_reads_value`, `mixed_chain_walk`). Rewrote FIR Impact / Step Impact
around the single `ContextfulSearch` engine (one generalized `SearchFir` with `CursorSource` +
`SearchPredicate`) instead of three separate kinds, with a new-feature-first / backfit-last
implementation strategy. Fixed a Motivation example that used a bare `#1` where the
context-reading `&#1` is required (plain `#`/`^`/`$` always demand a brane, like `$`).

**Date**: 2026-07-05
**Updated By**: Claude Code 2.1.119 (Claude Code); Fable 5 (claude-fable-5)
**Changes**: Added §C.3.1 explaining why name+value (`~name=value`, forms 4–6) is an atomic
conjunctive operator and cannot be synthesized by chaining `&~name` then `&~=value` (the first
search collapses to one result before the second predicate applies; worked `{setting=11;
setting=10}` counter-example, and `b~setting=10&#-1` positional composition). Added the
`name_value_atomic.foo` approval test, cross-linked A.3's name gate to §C.3.1, and added a
low-priority "streaming-context search" idea to Future Work (implement `?setting=10` by filtering
the stream of `?setting` result contexts for `=10`).

**Date**: 2026-07-05
**Updated By**: Claude Code 2.1.119 (Claude Code); Fable 5 (claude-fable-5)
**Changes**: Reworked Part C around Atlas's `&`-prefix model. Added a Terminology section
defining "home brane / brane of a FIR", "Contextless Anchored Searches" (shorthand: contextless
searches / searches), "Contexted Anchored Searches" (`&`-searches / contexted searches), and
"value searches" (`&=`-search). Split search into a contextless family (existing `.` `?` `~` `#`
`^` `$` and value `~=`/`?=` — deepen, demand a brane) and a contexted family (`&`-prefixed —
navigate from a statement's position within its home brane, bounded); this resolves the
`a.brane_field.x` ambiguity (`.` always deepens). Introduced `ContextedSearchFir`; removed the
old "plain `#` silently goes positional" behavior (now Rejected Alternative E). Retitled the
FOOP. Updated grammar note, FIR/Step Impact, Test Plan, Open Questions, Future Work, References,
and all Part C approval-test inputs to `&`-forms.

**Date**: 2026-07-04
**Updated By**: Claude Code 2.1.119 (Claude Code); Fable 5 (claude-fable-5)
**Changes**: Initial draft from Atlas's dictated design: the six-form `~=`/`?=` value-search
family (integer-literal MVP, no `.=` alias), expression patterns, chained-search sequencing,
two-child `ubc_children` with `FoolRefFir` (strong immutable original-statement reference), and
position-anchored `#`/`?`/`~` on searches with home-brane bounds. Proposed approval-test inputs
embedded per part.
