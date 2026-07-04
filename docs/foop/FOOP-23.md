---
foop: 32
title: Value search — the ~=/?= operator family, expression patterns, and anchoring searches on searches
author: Atlas <hc.busy@gmail.com>
credits: drafted by Claude Code from Atlas's dictated design (session 2026-07-04)
status: Draft
type: Standards
created: 2026-07-04
phase: phase-2
supersedes: []
begun: [ ]
---

# FOOP-23: Value search — the `~=`/`?=` operator family, expression patterns, and anchoring searches on searches

## Abstract

Foolish gains **value search**: searching a brane for a statement whose *value* equals a sought
value, complementing the existing name/pattern search family. The MVP is three strictly ordered
parts:

- **Part A — the operator family, integer-literal equality.** Six forms: anchored forward
  `a~=value`, anchored backward `b?=value`, unanchored backward `?=value`, and the combined
  name-and-value forms `a~id=value`, `a?id=value`, `?id=value`. Equality is implemented for
  independent integer literals only; any other kind of value in the search pattern (in
  particular a brane) is an error.
- **Part B — expression patterns.** The pattern may be an expression: `a~=1+2` searches for `3`;
  `a~=c-d` resolves `c` and `d` by ordinary name search, computes the difference, then performs
  the value search.
- **Part C — anchoring searches on searches.** Chained searches
  (`final = some_brane.some_aspect.some_thing_else.some_value`) are formalized: each stage waits
  for the previous stage to settle constanic-and-found before scanning. To support this, every
  resolved search keeps **two** `ubc_children`: `[0]` the constanic clone of the found
  statement's body (unchanged from today) and `[1]` a new **`FoolRefFir`** — an immutable,
  strong (non-weak) reference to the *original* found statement, with its original parent chain
  intact. Anchoring an index or a search on a search then becomes positional: `a~step_1#5`
  indexes `+5` from the original statement's position in its home brane (`#0` is the statement
  itself); `a~step_1?prep_var` searches backward from that position. Neither exceeds the bounds
  of the brane the original statement belongs to.

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
remembers its *original* statement (via `FoolRefFir`), a value search composes with positional
anchoring — `doc~=4#1` means "the statement *after* the first statement whose value is 4."
Value search finds the place; chained anchoring exploits it. The name payoff (extracting the
found statement's name as a first-class value) is deliberately left to a future FOOP — see
Future Work.

### Why integer literals only?

Foolish currently has no defined equality for branes or for searches (the vintage
`docs/vintage_legacy/EQUIVALENCE.md` sketches an operator family — `=s=`, `==`, `===`, `=n=`,
`=v=`, … — but none of it is specified or implemented). Rather than block value search on a
brane-equivalence theory, the MVP implements equality for **independent integer values** and
makes every other pattern kind an explicit error. When brane equivalence is later specified,
value search inherits it without changing surface syntax.

### Prior art being superseded

Two incompatible vintage notations exist and are superseded by this FOOP:

- `docs/vintage_legacy/NAMES_SEARCHES_N_BOUNDS.md` §Value Search: forward `?=`, bulk `?=*`, and
  a stray unexplained colon form (`doc:4 = 10`).
- `docs/vintage_legacy/ADVANCED_FEATURES.md` (and the README operator list): `:` / `::`.

This FOOP's family keeps `?=` but assigns it *backward* semantics (mirroring `?` for names),
introduces `~=` for forward (mirroring `~`), and drops the colon notation entirely. Bulk value
search (`?=*`-style find-all) is out of scope, as is all find-all search (`??`, `//`).

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
| 4 | `a~id=value` | anchored   | forward   | name matches `id` AND value equals the pattern     |
| 5 | `a?id=value` | anchored   | backward  | name matches `id` AND value equals the pattern     |
| 6 | `?id=value`  | unanchored | backward  | name matches `id` AND value equals the pattern     |

There is **no `.=` alias**. `.` aliases `?` for name search, but `a.=10` reads as a compound
assignment to too many eyes; the alias is explicitly rejected (see Rejected Alternatives). There
is **no unanchored forward form**, for the same reason there is no unanchored forward name
search: Foolish cannot look forward in its own brane.

In the combined forms (4–6), `id` is a name pattern with exactly the semantics of the
corresponding name search (`matches_pattern`: literal match, else anchored regex `^id$`). Both
conditions must hold on the same statement.

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
search suffixes: in `a~=1+2#5`, the pattern is `1+2` and `#5` is a chained anchor on the whole
value search (Part C). To use a search result inside a pattern, parenthesize: `a~=(b#2)`
(Part B).

Disambiguation note: the statement-naming `=` is unaffected. In `r = a~id=4;`, the first `=`
names `r`; the parser is already inside an expression when it reaches `~`, so `id=4` can only be
the name-and-value form.

#### A.3 Evaluation semantics

A value search scans candidate statements in its direction (forward from the front, backward
from the rear for anchored; the ordinary retrospective IB-then-AB walk for unanchored), testing
each candidate:

- **Name gate** (forms 4–6 only): the statement name must match the name pattern; otherwise the
  candidate is skipped without inspecting its value.
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

`value_search_forward_and_backward.foo` — both directions; both display `10`, direction is
pinned by unit tests here and made observable in Part C:

```foolish
{
	a = {
		id = 4;
		size = 10;
		depth = 10;
	};
	fwd = a~=10;
	bwd = a?=10;
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
	four  = a~tmp_.*=4;
	seven = a?tmp_.*=7;
	none  = a~tmp_.*=10;
}
```

(`four` → 4, `seven` → 7, `none` → `???` — the name gate excludes `size` although its value is
10.)

`value_search_unanchored.foo` — unanchored backward, hit and miss:

```foolish
{
	pi = 3;
	e2 = 2;
	found = ?=3;
	nope = ?=9;
	named = ?e.*=2;
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
	bad = a~={q = 1;};
	ok = a~=5;
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
	r1 = a~=1+2;
	r2 = a~=c-d;
	r3 = a~=c-d+v;
}
```

(`r1` → 3 via `u`; `r2` → 3 (c−d=3) via `u`; `r3` → NK or alarm? No — `v` resolves
retrospectively from the pattern's own context, where no `v` precedes: the pattern is
ECONSTANIC, so `r3` waits and, no recoordination arriving, the program settles with `r3`
displaying `⎵⎵`. This pins that pattern names resolve in the search's context, *not* inside the
anchor `a`.)

### Part C — anchoring searches on searches (chained search and `FoolRefFir`)

#### C.1 Chained-search sequencing

In `final = some_brane.some_aspect.some_thing_else.some_value`, each search is the anchor of the
next. The rule, stated explicitly:

> A search whose anchor is itself a search does not scan until the anchor search is **constanic
> and found**. Anchor NK → the dependent search is NK. Anchor ECONSTANIC (searched, nothing
> found) → the dependent search is ECONSTANIC (it may be rescued if recoordination later
> resolves the anchor). Anchor WOCONSTANIC or CONSTANT with a result → the dependent search
> scans.

So `some_value` does not search until `some_thing_else` is constanic-and-found;
`some_thing_else` does not search until `some_aspect` has settled to constanicity — the chain
resolves strictly left to right. (Today's UBCa already achieves much of this ordering through
task sequencing and `resolve_anchor`; this FOOP makes the contract normative and extends it to
positional anchoring below.)

#### C.2 Two-child `ubc_children` and `FoolRefFir`

Today a resolved search holds exactly one `ubc_child`: the constanic clone of the found
statement's body (`ProtoBrane::push_search_result` asserts `ubc_children.len() <= 1`). The clone
is detached — it has a new parent and no memory of where it was found. That memory is exactly
what positional anchoring needs. This FOOP specifies:

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
  reference that dies would silently break positional anchoring. The cost is accepted: strong
  cross-links can form `Rc` cycles in pathological mutual-search programs; those leak until FVM
  teardown, which is acceptable for the MVP and noted in Open Questions.
- **Immutable.** `FoolRefFir` is a window, not a handle: the referent cannot be mutated through
  it. Its own NYES is set to CONSTANT at creation (the reference itself is a settled value even
  while the referent may still be evolving), `fir_op_step` is a no-op, and it never enqueues
  tasks.
- **Invisible to values.** `FirRefExt::value`, result-chain walking
  (`deepest_econstanic_in_chain` follows `ubc_children[0]`), the evaluator's result extraction,
  and the Humanizing Sequencer all continue to read `ubc_children[0]` only. `FoolRefFir` never
  appears in HFS output. Existing snapshots must not change from Part C's bookkeeping alone.

Consequential changes: the `push_search_result` single-entry assertion is replaced by the
two-entry invariant (`[0]` clone then `[1]` FoolRefFir, pushed together); `clear_ubc_children`
clears both; every reader that assumed "at most one" is audited (the plan lists them).

#### C.3 Positional anchoring: `#`, `?`, `~` anchored on a search

When an index or a search finds that its anchor is a **search** (rather than resolving through
to a brane), it anchors on the search's *original statement* — `anchor.ubc_children[1]`'s
referent — and works positionally within that statement's **home brane** (the brane its original
parent chain places it in):

- **`a~step_1#5`** — the index sees its anchor is a search; it locates the original statement of
  `a~step_1`'s result in its home brane and addresses the statement at *that position + 5*,
  where **`#0` is the found statement itself**, positive offsets move toward the end of the home
  brane, negative offsets toward the front. Out of range → the ordinary anchored-index miss
  (NK on a settled brane).
- **`a~step_1?prep_var`** — the backward search sees its anchor is a search; it starts at the
  original statement's position and scans **backward** through the home brane for `prep_var`
  (the found statement itself is included in the scan, consistent with `#0` addressing it; see
  Open Questions).
- **`a~step_1~post_var`** — symmetric: forward from the original statement's position toward the
  end of the home brane.
- **Bounds.** Exactly like every anchored search, position-anchored `~`/`?`/`#` never leave the
  brane they are anchored to: the scan is clipped to the home brane of the original statement.
  Escaping the home brane (continuing into its IB/AB) is *not* part of this MVP; a distinct
  operator spelling (e.g. `?&`) is reserved for it — see Future Work.
- Value-search forms compose identically: `doc~=4#1` is "the statement after the first
  statement whose value is 4" — the promised context payoff.

Implementation note: `index_into_brane_relative(brane, stmt_idx, offset)` in
`foolish-ubca/src/fir_kinds.rs` already implements statement-relative addressing for unanchored
seeks; position-anchored `#` reuses it with `stmt_idx` taken from the original statement's
position (`find_stmt_index` by identity against the home brane).

#### C.4 Part C approval-test inputs

`chained_search_sequencing.foo` — the normative left-to-right chain:

```foolish
{
	some_brane = {
		some_aspect = {
			some_thing_else = {
				some_value = 42;
			};
		};
	};
	final = some_brane.some_aspect.some_thing_else.some_value;
}
```

(`final` → 42.)

`search_anchored_index.foo` — `#` anchored on a search, forward and self:

```foolish
{
	recipe = {
		steps = {
			prep_var = 7;
			step_1 = 40;
			bake = 9;
		};
	};
	stage = recipe.steps~step_1;
	self = recipe.steps~step_1#0;
	after = recipe.steps~step_1#1;
	oob = recipe.steps~step_1#5;
}
```

(`stage` → 40, `self` → 40, `after` → 9, `oob` → `???`.)

`search_anchored_search.foo` — `?`/`~` anchored on a search, bounded by the home brane:

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
	before = recipe.steps~step_1?prep_var;
	afterv = recipe.steps~step_1~bake;
	escape = recipe.steps~step_1?outside;
}
```

(`before` → 7, `afterv` → 9, `escape` → `???` — `outside` lives beyond the home brane and
position-anchored search may not leave it.)

`value_search_positional_payoff.foo` — the Motivation's payoff, direction made observable:

```foolish
{
	doc = {
		tmp_a = 4;
		c = 30;
		tmp_b = 4;
	};
	after_first_4 = doc~=4#1;
	before_last_4 = doc?=4#-1;
}
```

(Both → 30, but via different found statements: `~=` found `tmp_a` and stepped forward; `?=`
found `tmp_b` and stepped backward. A differing middle statement would distinguish them; unit
tests also pin the found indices.)

## FIR Impact

- **New FIR kind `ValueSearchFir`** (`foolish-ubca/src/fir_kinds.rs`): `core: ProtoBrane`,
  `name_pattern: Option<String>` (forms 4–6), `anchored: bool`, `forward: bool`.
  `foolish_children`: `[anchor]` (anchored forms) + `[value_pattern]` (always). A separate kind
  rather than more flags on `SearchFir` — the step table differs (pattern-settling stage,
  wait-on-nye-candidate rule) and per FOOP-62/AGENTS.md every kind carries its own NYES
  transition test.
- **New FIR kind `FoolRefFir`** (C.2): no children, no steps, CONSTANT at creation, strong
  `referent: FirRef`, no mutation path to the referent.
- **`SearchFir` and `ValueSearchFir` resolved shape**: `ubc_children == [clone, FoolRefFir]`
  (was `[clone]`). `ProtoBrane::push_search_result`'s single-entry assertion becomes the paired
  two-entry invariant.
- **NYES tables**: `ValueSearchFir` follows `SearchFir`'s progression (PREMBRIONIC →
  EMBRYONIC/BRANING → {WOCONSTANIC, CONSTANT, ECONSTANIC, NK}) with the added BRANING
  suspension while the pattern or a name-gated candidate is nigh. `FoolRefFir` is born
  CONSTANT (terminal). Both get `*_nyes_transitions` unit tests (mandatory per AGENTS.md).
- **Serialization/HFS**: `FoolRefFir` is never sequenced; HFS output keyed to `ubc_children[0]`
  is unchanged. Existing approved snapshots must be byte-identical after Part C bookkeeping.
- **Alarms**: new alarm code `VALUE-SEARCH-UNSUPPORTED-PATTERN` (A.4/B), emitted through the
  existing alarm sink alongside the NK settlement, mirroring `DIV-BY-ZERO`.

## UBC Step Impact

- **`ValueSearchFir::fir_op_step`**: PREMBRIONIC — push anchor task (anchored) and pattern task;
  EMBRYONIC/BRANING — settle pattern first (B; in Part A verify literal), then scan per A.3 with
  the wait-on-nye and NK-stop rules; settle per `nyes_from_found`, pushing the result pair.
- **`SearchFir::handle_found`** (and the value-search equivalent): additionally constructs and
  pushes the `FoolRefFir` for the original statement.
- **`IndexFir` step**: when the resolved anchor is a Search (today `_ => None` at the
  kind-dispatch), take the anchor's `ubc_children[1]` referent, find its index in its home
  brane, and address `stmt_idx + offset` via `index_into_brane_relative` (`#0` = the statement).
- **`SearchFir` step (anchored arm)**: same dispatch gains a Search case — scan the home brane
  from the original statement's position, backward (`?`) or forward (`~`), clipped to the home
  brane.
- **Chained sequencing (C.1)**: normative wait states — anchor NK → NK; anchor ECONSTANIC →
  ECONSTANIC; anchor nigh → remain BRANING; anchor constanic-and-found → scan.

## Test Plan

Tests are written **first** in each part (project rule):

- **Part A**: unit tests in `fir_kinds.rs` for `ValueSearchFir` — direction (which statement
  index matched, forward vs backward), name-gate interaction, nye-candidate suspension,
  NK-candidate contagion, non-integer candidate skipping, non-integer pattern alarm+NK,
  unanchored ECONSTANIC miss vs anchored NK miss; `value_search_fir_nyes_transitions` via
  `assert_progression`. Approval inputs from A.5.
- **Part B**: unit tests for pattern-settling (expression pattern, NK pattern, ECONSTANIC
  pattern waits, pattern names resolve in search context not anchor brane). Approval inputs
  from B.1.
- **Part C**: unit tests for the two-child invariant, `FoolRefFir` immutability and strong
  liveness (original brane dropped → referent still reachable), `fool_ref_fir_nyes_transitions`,
  position-anchored `#`/`?`/`~` including bounds clipping and `#0` identity; a full-suite
  snapshot run proving **zero diffs** from C.2 bookkeeping before the C.3 features land.
  Approval inputs from C.4.
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

### C. Overloading `SearchFir` with a value-pattern flag instead of a new kind

Rejected: the step table genuinely differs (pattern-settling stage, wait-on-nye-candidate), the
mandatory per-kind NYES-transition tests want a kind boundary, and enum-dispatch style (per
`rust_instructions.md`) favors concrete kinds over flag soup.

### D. Weak reference in `FoolRefFir`

Rejected by design requirement: a weak referent that dies under brane restructuring would make
positional anchoring silently fail; the strong reference's cycle risk is bounded and accepted
for the MVP.

### E. Do nothing

Value search remains a two-notation vintage sketch with no rationale, and chained positional
anchoring — needed independently for search-context navigation — has no substrate (the clone
forgets where it came from). The FoolRefFir memory is the smallest change that makes position a
first-class product of search.

## Open Questions

- **Inclusivity of position-anchored scans**: `a~step_1?prep_var` starts *at* the original
  statement (inclusive, consistent with `#0` = the statement itself) — confirm BDFL intent, or
  should backward scan start at position −1 (strictly before), matching IB search's
  `line_number − 1` convention?
- **`FoolRefFir` NYES**: CONSTANT (chosen — the reference is a settled value) vs INDEPENDENT
  (it has no context dependencies of its own). Cosmetic unless something dispatches on it.
- **Cycle mitigation**: is MVP leak-until-teardown acceptance enough, or should the FVM own a
  registry of FoolRefFirs for explicit teardown?
- **Equality maturation**: when brane equivalence (EQUIVALENCE.md's descendants) is specified,
  which equivalence does `~=` adopt by default — `=v=` (value), `==` (post-evaluation), or
  parameterized?
- **`?&` spelling**: the beyond-home-brane escape operator is reserved but unspecified — Future
  Work, separate FOOP.

## Future Work (non-MVP, explicitly out of scope)

- **Name extraction**: a way to obtain the found statement's *name* as a value (the second half
  of the context payoff; the vintage `|` "name at cursor" modifier is prior art).
- **`?&` (or similar)**: position-anchored search that escapes the home brane into its IB/AB.
- **Bulk value search** (`?=*`-style find-all), together with all find-all search.
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
  `nyes_from_found`, `index_into_brane_relative`, `FirRefNavExt`),
  `foolish-ubca/src/proto_brane.rs` (`push_search_result` single-entry assertion — replaced),
  `foolish-parser/src/token.rs` / `parser.rs` (Tilde/Question tokens, search suffix parsing).

## Last Updated

**Date**: 2026-07-04
**Updated By**: Claude Code 2.1.119 (Claude Code); Fable 5 (claude-fable-5)
**Changes**: Initial draft from Atlas's dictated design: the six-form `~=`/`?=` value-search
family (integer-literal MVP, no `.=` alias), expression patterns, chained-search sequencing,
two-child `ubc_children` with `FoolRefFir` (strong immutable original-statement reference), and
position-anchored `#`/`?`/`~` on searches with home-brane bounds. Proposed approval-test inputs
embedded per part.
