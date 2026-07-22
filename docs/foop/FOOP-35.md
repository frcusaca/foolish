---
foop: D53
title: The dot search — authoritative definition of the `.` operator
author: Atlas <hc.busy@gmail.com>
status: Draft
type: Standards
created: 2026-07-22
phase: phase-2
supersedes: []
begun: [x]
      (2026-07-22 09:31)
---

# FOOP-35: The dot search

FOOP numbering is little-endian; the full rules live in `foop.md` at the
repository root — **read it before creating or editing a FOOP.**

## Abstract

This FOOP provides the authoritative definition of the `.` (dot) search
operator. The dot search is a contextless anchored backward name search that
resolves its anchor to a brane and scans backward for an exact name match. It
reuses the existing `ContextfulSearch` engine and `SearchPredicate::Name`
predicate. The operator is written `a.name` or `a . name` (whitespace around
`.` is permitted). The coordinate after `.` must be a single identifier matched
strictly — no regex metacharacters, no whitespace within the identifier itself.

## Motivation

The `.` operator is the most common search operator in Foolish. It appears in
every coordinate access (`brane.field`), every chained deepening
(`a.b.c.d`), and every brane navigation pattern. Despite its ubiquity, the
`.` operator's semantics are currently defined only by its implementation — the
compiler lowers `Astn::DotSearch` to a `SearchFir` with a regex-anchored
pattern, and the `ContextfulSearch` engine performs the scan. This FOOP makes
those semantics explicit and normative.

The dot search serves as the foundation for coordinate access in Foolish. When
a Foolisher writes `point.x`, they are performing a backward search through
the brane named `point` for a statement whose name is exactly `x`. The result
is that statement's value, and the search context (the found statement's
position in its home brane) is available to any following contexted (`&`)
operator.

## Specification

### Terminology

The term **dot search** refers exclusively to the `.` operator as defined here.
It is a **contextless anchored backward name search** — each word in that
phrase is load-bearing:

- **Contextless**: the operator does not read incoming context from a preceding
  search. It demands its anchor resolve through to a whole brane and searches
  that brane.
- **Anchored**: the search operates on a specific anchor expression (the
  left-hand side of `.`).
- **Backward**: the scan proceeds from the rear of the brane toward the front,
  returning the *last* matching statement.
- **Name search**: the predicate matches on the candidate statement's name.

### Surface syntax

```
dot_search := anchor '.' coordinate
```

The `.` token separates the anchor from the coordinate. Whitespace is
permitted around `.`:

```foolish
a.name       !! compact form
a . name     !! spaced form — identical semantics
```

Both forms produce the same AST node and the same FIR. The lexer skips
whitespace before tokenizing, so `a . name` lexes identically to `a.name`
except for token positions.

### Coordinate

The coordinate is a single **identifier** — a sequence of identifier characters
with no whitespace or metacharacters. Identifier characters are defined by the
lexer (`foolish-parser/src/lexer.rs`):

- Letters (Unicode `is_alphabetic`)
- ASCII digits
- Separator characters: `_` (underscore), U+02CD (modifier letter low line),
  U+202F (narrow no-break space) — all normalized to U+02CD during lexing

The coordinate is **not** a regex pattern. It is matched exactly: the compiler
wraps it in `^...$` anchors before passing to `matches_pattern`. This means
`a.x` matches only a statement named exactly `x` — not `xy`, `xx`, or any
other name.

A coordinate that contains regex metacharacters (`.`, `*`, `+`, `?`, `[`, `]`,
`(`, `)`, `{`, `}`, `|`, `\`, `^`, `$`) is technically accepted by the lexer
(they are not identifier characters, so the lexer would stop at them), but in
practice the coordinate is always a plain identifier. If a Foolisher needs
regex matching, they use the `?pattern` or `~pattern` operators instead.

### Dot inside regex patterns

The `.` token is consumed by `parse_regexp_pattern()` as part of a regex
pattern when it appears inside a `?` or `~` search. This means `a?x.member`
parses as a single regex search with pattern `x.member`, not as a regex search
for `x` followed by a dot search for `member`. The same applies to
`a~(blah.).member` — the parentheses are consumed as a regex subpattern, then
`.member` continues as part of the same pattern string.

Neither whitespace (`a?x .member`) nor parenthesized subpatterns
(`a~(blah.).member`) isolate the dot from the pattern. The lexer strips
whitespace between tokens, and `parse_regexp_pattern`'s catch-all consumes all
remaining tokens (including `.`) until a structural delimiter (`;`, `,`, `}`,
`)`, `]`, `=`, `&`) is reached.

**The reliable workaround is separate assignment:**

```foolish
found = a?x;        !! regex search for x
result = found.y;   !! dot search for y inside the found brane
```

This is not a dot search limitation — it is a `parse_regexp_pattern` behavior
that affects all regex searches equally. A future parser change to stop pattern
scanning at `.` would make `a?x.y` parse as intended.

### Characterizations in coordinates

The parser's `parse_identifier_or_regexp()` handles optional characterizations
before the identifier: `a'coord` parses as a coordinate with characterization
`a`. The characterization prefix is joined with the identifier via `'` and
passed as the full coordinate string to the compiler. This is existing behavior
and is not changed by this FOOP.

### Evaluation semantics

The dot search evaluates in three phases:

#### Phase 1: Anchor resolution

The anchor expression is stepped to constanic. The search does not proceed
until the anchor has settled:

- **Anchor NK** → the search settles NK (the anchor is provably unfindable).
- **Anchor ECONSTANIC** → the search remains ECONSTANIC (the anchor may gain
  a value via recoordination).
- **Anchor nigh** (PREMBRYONIC, EMBRYONIC, BRANING) → the search waits.
- **Anchor constanic and found** → proceed to Phase 2.

This is the standard anchor-waiting behavior shared by all anchored search
operators.

#### Phase 2: Brane check

The anchor's resolved value (`ubc_children[0]`) is inspected:

- **Value is a brane** → proceed to Phase 3.
- **Value is not a brane** (e.g., an integer, NK) → the search settles NK.
  A non-brane anchor cannot be searched.

This check is performed by `resolve_anchor()` followed by `is_brane_like()`.

#### Phase 3: Backward scan

A `BraneNavigator` is created over the anchor brane, configured for backward
traversal (front-to-rear scan direction = false). The `SearchPredicate::Name`
predicate is applied with the pattern `"^<coordinate>$"`:

1. The navigator yields candidate statements from the rear of the brane toward
   the front.
2. For each candidate, the predicate reads the candidate's name and applies
   `matches_pattern(name, pattern)`.
3. `matches_pattern` performs: literal string equality check first, then
   regex match if the literal check fails. Since the dot search pattern is
   always `^...$`-anchored, the regex match is an exact-match check.
4. On match: the candidate's body is constanic-cloned into the search's
   `ubc_children[0]`, a `FoolRefFir` wrapping the original found statement is
   pushed to `ubc_children[1]`, and the search settles via `nyes_from_found`.
5. On miss (no candidates match): the search settles NK (anchored miss).

### Settlement

The settlement follows the standard `nyes_from_found` rules:

| Found statement's body NYES | Search settles |
|-----------------------------|----------------|
| ECONSTANIC or WOCONSTANIC   | WOCONSTANIC    |
| CONSTANT or INDEPENDENT     | CONSTANT       |
| NK                          | NK             |

On miss (no matching statement in the brane): NK.

### Search context

Every resolved search produces two `ubc_children`:

- `ubc_children[0]` — the constanic clone of the found statement's body (the
  search's value).
- `ubc_children[1]` — a `FoolRefFir` wrapping a strong reference to the
  original found statement, with its parent chain, line number, and home brane
  intact.

The `FoolRefFir` makes the search context available. A following contexted
(`&`) operator reads `ubc_children[1]` to anchor on the found statement's
position. For example:

```foolish
{
	recipe = {
		prep = 7;
		bake = 40;
		clean = 9;
	};
	after_bake = recipe.bake&#1;    !! 9 — &#1 steps one forward from bake's position → clean
}
```

`recipe.bake` finds `bake` at some position in `recipe`; `&#1` reads that
position and steps one statement forward to `clean`.

### Chaining and deepening

The dot search is contextless — it deepens. In a chain like `a.b.c`:

1. `a.b` resolves `a` to a brane, searches backward for `b`, finds it.
2. `.c` takes `a.b`'s value (which must be a brane), searches backward for
   `c` inside that brane.

Each `.` operates on the *value* of the previous search, not on its position.
This is the coordinate-access reading: `a.b.c` navigates *inside* each found
brane.

### Relationship to other search operators

The dot search is one member of the contextless anchored search family:

| Operator | Direction | Predicate     | Pattern source     |
|----------|-----------|---------------|--------------------|
| `.name`  | backward  | Name (exact)  | coordinate literal |
| `?name`  | backward  | Name (regex)  | user-supplied      |
| `~name`  | forward   | Name (regex)  | user-supplied      |
| `#N`     | —         | Index         | numeric offset     |
| `^`      | —         | Head          | —                  |
| `$`      | —         | Tail          | —                  |

The dot search differs from `?name` in one key way: the dot search always
wraps its coordinate in `^...$`, producing an exact match. The `?` operator
accepts a regex pattern that may match multiple names. When the Foolisher
writes `a.x`, they mean exactly `x`; when they write `a?x`, they mean any
name matching the regex `x` (which, for a plain identifier, happens to be
the same — but `a?x.*` matches `xy`, `xz`, etc.).

There is **no `&.`** contexted dot operator. The `.` operator deepens into a
brane; a "contexted deepen from a statement position" has no distinct meaning.
To navigate from a found statement's position, use `&?` or `&~`.

## FIR Impact

The dot search compiles to a `SearchFir` (not a new FIR kind):

```rust
SearchFir {
    core: ProtoBrane::new(vec![anchor_fir], parent, search_nyes),
    pattern: format!("^{}$", coordinate),   // exact-match regex
    anchored: true,
    forward: false,                          // backward scan
    sf_inner_pattern: RefCell::new(None),
    is_value_search: false,
    contexted: false,
}
```

No new FIR variants, no new fields on `SearchFir`, no changes to the
`ContextfulSearch` engine. The dot search is entirely defined by the
`SearchFir` configuration above.

## UBC Step Impact

None. The dot search uses the existing `SearchFir::fir_op_step` implementation
unchanged:

- PREMBRIONIC → push anchor task, set BRANING (anchored search)
- BRANING → resolve anchor, check brane, create `BraneNavigator` (backward),
  run `contextful_search_scan` with `SearchPredicate::Name`
- On found → `handle_found` pushes `[clone, FoolRefFir]`, settles via
  `nyes_from_found`
- On miss → NK (anchored miss)

No new step rules, no interactions with existing step rules.

## Test Plan

- **Existing approval tests** that use `.` access (`contextless_deepening_chain.foo`,
  `mixed_chain_walk.foo`, and others) already pin the dot search behavior.
  No changes to these tests are expected.
- **Unit test**: `parses_dot_search` in `foolish-parser/src/parser.rs` already
  verifies the parser produces `Astn::DotSearch` for `a.x`.
- **Unit test**: `search_finds_name_in_anchored_brane` in
  `foolish-ubca/src/fir_kinds.rs` already verifies the `SearchFir` engine
  finds names in anchored branes.
- **New approval test**: `dot_search_comprehensive.foo` — exercises:
  - Simple coordinate access: `a.x`
  - Chained deepening: `a.b.c.d`
  - Whitespace tolerance: `a . x`
  - Miss (NK): `a.nonexistent`
  - Contexted follow-up: `a.x&#1`
  - Multiple dots with mixed spacing: `a . b.c . d`

## Rejected Alternatives

### A. Make `.` an alias for `?` (regex-capable)

The `.` operator could accept regex patterns like `?` does, so `a.x.*` would
match `xy`, `xz`, etc. Rejected: the dot search's value is its *exactness*.
When a Foolisher writes `a.x`, they mean exactly `x`. Regex matching is
available via `?` and `~` for when it is needed. Conflating the two would
make `.` ambiguous between "exact coordinate access" and "pattern search."

### B. Make `.` forward-search instead of backward

The dot search could scan forward from the front of the brane. Rejected:
backward search returns the *last* matching statement, which is the natural
reading for coordinate access in a brane where later definitions shadow
earlier ones. The `~` operator provides forward search when needed.

### C. New FIR kind `DotSearchFir`

A dedicated FIR kind for the dot search. Rejected: the dot search is entirely
defined by the `SearchFir` configuration (pattern, anchored, forward,
contexted). A separate kind would duplicate ~95% of `SearchFir`'s logic
with no behavioral difference.

### D. `&.` contexted dot operator

A contexted twin of `.` that navigates from a statement's position. Rejected:
`.` deepens into a brane; "deepen from a statement position" collapses back to
plain `.` on the found statement's value. There is no distinct behavior for
`&.` to name. Use `&?` or `&~` for contexted navigation.

## Open Questions

None. The dot search is fully specified and implemented.

## References

- Prior FOOPs: FOOP-4 (bare identifiers compile to SearchFir), FOOP-23
  (ContextfulSearch engine, FoolRefFir two-child invariant, contexted searches),
  FOOP-62 (UBCa two-store ProtoBrane).
- Code: `foolish-parser/src/parser.rs` (`parse_postfix_expr`, line 582),
  `foolish-parser/src/ast.rs` (`Astn::DotSearch`), `foolish-parser/src/lexer.rs`
  (`Token::Dot`, identifier lexing), `foolish-ubca/src/compiler.rs`
  (`DotSearch` → `SearchFir` lowering, line 257),
  `foolish-ubca/src/fir_kinds.rs` (`SearchFir`, `matches_pattern`,
  `contextful_search_scan`, `BraneNavigator`).

## Last Updated

**Date**: 2026-07-22
**Updated By**: Hephaestus / xiaomi/mimo-v2.5-pro
**Changes**: Added "Dot inside regex patterns" section documenting that `.` is consumed by
`parse_regexp_pattern()` inside `?`/`~` searches. Neither whitespace nor parenthesized
subpatterns isolate the dot. Reliable workaround: separate assignment. Added two parser tests
pinning actual behavior.

**Date**: 2026-07-22
**Updated By**: Hephaestus / xiaomi/mimo-v2.5-pro
**Changes**: Initial draft. Authoritative definition of the dot search operator.
