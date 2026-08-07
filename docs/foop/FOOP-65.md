---
foop: D65
title: The tail concatenator — backtick application that brings the method name to the front
author: Sisyphus / qwen3.8-max (directed by Atlas)
status: Draft
type: Standards
created: 2026-08-07
phase: phase-2
supersedes: []
begun: [ ]
---

# FOOP-65: The tail concatenator — backtick application that brings the method name to the front

FOOP numbering is little-endian; the full rules live in `foop.md` at the
repository root — **read it before creating or editing a FOOP.** The one
template-specific note: the `foop:` front-matter field may either match the
filename digits directly, or give the big-endian decimal value preceded by
`D` (this file: `foop: D65` — digits `65` reversed = sort key 56). In all
cases, the `FOOP-65.md` file name is ultimately the right numbering.

## Abstract

This FOOP adds the **tail concatenator** — the backtick `` ` `` — to
Foolish's surface syntax. `A = fn`{param1, param2...}` is syntactically
equivalent to `A = {param1, param2...} fn`: the backtick concatenates its
LEFT operand onto the **tail** of its right operand, so a method/function
name can be written **first**, resembling ordinary function calls. The tail
concatenator is the **weakest** operator — weaker than brane concatenation
(juxtaposition) — and within a consecutive run of tail concatenators (or of
ordinary brane concatenation) the concatenation itself is **associative**,
so only the precedence is load-bearing: brane concatenation happens first,
then tail concatenation. The implementation gets a **dedicated FIR kind**
(`TailConcatenationFir`) that executes *as* a concatenation — it has-a
ConcatenationFir inside, and its `value()` returns that inner
concatenation's value. The separate FIR exists as the hook on which future
features (which may change the `value()` output) will hang; this FOOP adds
no behavior beyond the equivalence. The tail concatenator does **not**
obviate the `'` null-characterization in the name of the method.

## Motivation

Foolish application today is postfix juxtaposition — `{lv,3,0}'cmod`,
`{cond1, sum+lv, sum, 'ite}`, `{loop} loop` — the argument brane first, the
method name last. It works, but it reads inside-out compared to the
function-call notation every Foolisher knows from every other language.

The backtick brings the method name to the front — `'cmod`{lv,3,0}`,
`'ite`{cond1, sum+lv, sum}` — while meaning *exactly* the same
concatenation. It is sugar with a dedicated FIR (rather than a silent
parse-time rewrite) deliberately, so that later features that need to know
"this concatenation came from a tail concatenator" — features that may
change what `value()` outputs — have a place to live without re-litigating
the syntax.

The first consumer is FOOP-55 (Project Euler exercise 1), whose application
sites are rewritten in backtick form once this FOOP lands.

## Specification

### §1. Surface form and the core equivalence

```foolish
A = fn`{param1, param2...}      !! the tail concatenator
A = {param1, param2...} fn      !! syntactically equivalent — same concatenation
```

The backtick takes two operands: the **method** (left) and the **argument
expression** (right). It produces the concatenation of the right operand
with the left operand appended at its **tail** — the right operand's
statements first, the left operand last.

```foolish
!! The exercise's congruent-modulo, in backtick form (FOOP-55):
!!   {lv,3,0}'cmod   is written   'cmod`{lv,3,0}
```

**The `'` null-characterization is unaffected.** The method name keeps its
null characterization exactly as before — `'cmod`, `'ite`, `'or`, `'lt` —
the backtick changes word order, not names.

### §2. Precedence and associativity (authoritative)

**The tail concatenator is the weakest operator** — weaker than brane
concatenation (juxtaposition), and therefore weaker than everything
juxtaposition is weaker than. Each operand of a backtick is a full ordinary
expression: juxtaposition, `$`, and search suffixes all bind **inside** the
operands, before the backtick applies.

```foolish
A = fn`{a}{b}{c}$        !! interpreted as   A = ({a}{b}{c}$) fn
```

— the `$` belongs to the right operand `{a}{b}{c}$`; it does NOT extract
the result of the whole application. (Extracting the application result
needs parentheses today: `(fn`{a})$` — see Open Questions.)

**Within a consecutive run of tail concatenators — or of ordinary brane
concatenation — the concatenation itself is associative.** This is worth
restating even though brane concatenation's associativity is already
established (FOOP-3 lineage): it is the reason a backtick **chain** needs
no nesting semantics at all. The only precedence required is that **brane
concatenation happens first, then tail concatenation.**

Why the backtick must stay weakest — Atlas's breakdown (2026-08-07): in
`f`g`h`a b c`, if the tail concatenator went FIRST (bound tighter), it
would grab its operands out of the surrounding juxtaposition: `f`g` → `g f`,
then `(g f)`h` → `h g f`, and the leftover juxtaposition would interleave
wrongly — the result would read `a (h g f) b c` instead of the intended
`a b c (h g f)`. Keeping the tail concatenator weaker than brane
concatenation makes each backtick operand a fully-grouped juxtaposition and
the breakdown cannot occur.

*(Decision history, for the record: the chain associativity was first
stated left, then corrected to right (`f`g`h`a b c` = `(((a b c)h)g)f`),
then settled here: within a run it does not matter — concatenation is
associative — so the chain is parsed FLAT and only the precedence is
pinned.)*

### §3. Semantics — a flat chain reverses source order

A run `e1`e2`...`en` (n ≥ 2) is one flat tail concatenation whose value is
the juxtaposition concatenation of the same operands in **reverse source
order**:

```
e1 ` e2 ` ... ` en   ≡   en ... e2 e1
```

```foolish
f`g`h`a b c     ≡     a b c h g f      !! Atlas's form: (((a b c)h)g)f
```

Binary case: `L`R` ≡ `R L`. Because concatenation is associative (§2), the
flat reading and any nested reading denote the same statement sequence; the
implementation is flat (§4–§5).

### §4. Lexer, AST, parser

**Lexer.** One new single-character token:

```rust
// foolish-parser/src/token.rs
Backtick,           // `
// Display: "`"
```

with the ordinary single-char arm in `Lexer::next_token`
(`lexer.rs:173-285` region). Today a backtick in code position falls into
the unknown-character fallback (`lexer.rs:297-299`, the D1 defect of
FOOP-55); after this FOOP it is a real token. **Non-regression verified:**
the only backticks in the entire einmo input corpus sit inside `!!` line
comments (`foop/13/comprehensive.foo:17-18,65`,
`exercises/project_euler/1.foolish:15`), which the lexer consumes before
the fallback — no existing program's meaning changes.

**AST.** One new node, n-ary and flat, in source order:

```rust
// foolish-parser/src/ast.rs
Astn::TailConcatenation { elements: Vec<Astn> }   // len >= 2, source order
```

**Parser.** A new weakest level ABOVE the current expression grammar. The
current `parse_expr` (additive + juxtaposition loop, `parser.rs:371-388`)
becomes the operand parser (call it the concat level); the new top level
collects the backtick chain flat:

```rust
fn parse_expr(&mut self) -> Result<Astn> {
    if self.peek_token() == Some(&Token::If) { return self.parse_if_expr(); }
    let mut elements = vec![self.parse_concat_level()?];
    while self.peek_token() == Some(&Token::Backtick) {
        self.advance();
        elements.push(self.parse_concat_level()?);   // operand = full juxtaposition expr
    }
    match elements.len() {
        1 => Ok(elements.pop().unwrap()),
        _ => Ok(Astn::TailConcatenation { elements }),
    }
}
```

Properties that fall out of this shape:

- Juxtaposition binds first — the operand loop (`parse_concat_level`)
  greedily consumes `{a}{b}{c}$` before the backtick chain resumes (§2).
- `Backtick` is NOT added to `is_concatenation_continuation`
  (`parser.rs:390-411`) — juxtaposition never crosses a backtick.
- Because every expression-consuming site calls `parse_expr` (assignment
  RHS, parentheses, `<...>`/`<<...>>` marker contents, if-expr branches),
  the backtick works uniformly everywhere an expression parses.
- A backtick with no operand after it errors at the operand parse
  ("expected primary expression" or a dedicated message — implementer's
  choice).

### §5. The `TailConcatenationFir` — a separate FIR that executes as a concatenation

Per Atlas's direction: the implementation makes a **separate FIR** for the
tail concatenator, but it **executes as a ConcatenationFir** — it has-a
ConcatenationFir inside, and that inner concatenation is returned as the
tail concatenator's `value()`. The separate FIR will be given more features
later that may change the `value()` output; for now it is the easier way
to bring the method name to the front while keeping a named hook for what
comes next. **This FOOP adds no behavior beyond the §3 equivalence.**

```rust
// foolish-ubca — new FIR kind
pub struct TailConcatenationFir {
    pub(crate) core: ProtoBrane,   // foolish_children[0] = the inner ConcatenationFir
    // future features land here
}
```

- **Construction (compiler, `build_fir` arm for `Astn::TailConcatenation`):**
  build the operand FIRs, **reverse** them, build the ConcatenationFir over
  the reversed elements — reusing exactly the machinery the existing
  `Astn::Concatenation` arm uses — and wrap it as the
  `TailConcatenationFir`'s single foolish child.
- **Stepping (`fir_op_step`):** delegates — the inner concatenation is
  pushed as the task; when it settles, the wrapper mirrors its NYES. The
  wrapper itself computes nothing.
- **`value()`:** the inner concatenation's value (the concatenation result
  brane). This is the output future features may change.
- **Constanic clone / recoordination:** a new `FirKind::TailConcatenation`
  arm in `constanic_clone_at` — constanic-clone the INNER concatenation
  through the existing ConcatenationFir recoordination path, and re-wrap
  the clone in a fresh `TailConcatenationFir`. The wrapper survives
  cloning (its identity as a tail concatenation is what future features
  hang off); the semantics are the inner's.
- **NYES:** no new states — the wrapper walks the inner's progression
  (Prembrionic → Embryonic → Braning → whatever the concatenation settles
  to).

**Rendering.** The sequencer renders through the inner concatenation — the
tail concatenation has no distinct visual of its own (an un-settled form
renders as its constituent parts; the implementer verifies the sequencer
path and adds a foolish-core arm only if the sequencer structurally
requires one, cf. FOOP-33's CreationFir note).

## FIR Impact

- **New FIR kind `TailConcatenationFir`** (foolish-ubca), new `FirKind`
  arm, new `constanic_clone_at` arm (§5). YAML/JSON shape:
  `{ kind: TailConcatenation }` wrapping its inner concatenation.
- **New token `Backtick`** (foolish-parser) + `Astn::TailConcatenation`.
- **No new NYES states.** No serialization impact beyond rendering through
  the inner concatenation.

## UBC Step Impact

- **New `fir_op_step`** — pure delegation to the inner ConcatenationFir
  (§5). No existing step rule changes.
- **Parser gains one precedence level** above the current expression
  grammar (§4); the juxtaposition loop, `$`, and search-suffix paths are
  untouched.
- **Concatenation semantics untouched** — the tail concatenator is
  concatenation, spelled differently.

## Test Plan

Tests first, per `rust_instructions.md`.

**Unit — lexer/parser (foolish-parser):**
- Backtick lexes to `Token::Backtick` (and is no longer an unknown char).
- `f`X` parses to `TailConcatenation [f, X]`; a chain `f`g`h`X` parses to
  ONE flat `TailConcatenation [f, g, h, X]` (no nesting).
- Precedence pins: `fn`{a}{b}` → `[fn, Concatenation[{a},{b}]]`
  (juxtaposition grouped first); `fn`{a}$` → `[fn, $(Concatenation[{a}])]`
  (`$` inside the operand); `(fn`X)~name` keeps the search suffix on the
  parenthesized whole.
- Backtick works inside brane statements, parentheses, and `<...>`/`<<...>>`
  markers; trailing backtick → parse error.

**Unit — FVM (foolish-ubca):**
- `tail_concatenation_nyes_transitions` — REQUIRED by AGENTS.md for the
  new FIR kind (wrapper mirrors the inner concatenation's progression).
- **Equivalence:** for several X (brane literal, search, concatenation),
  `fn`X` and `X fn` settle to the same brane (statement-for-statement).
- **Chain reversal:** `f`g`h`X` settles as `X h g f`.
- **System-operator application through the wrapper:**
  `('lt`{1, 2})$` → the `system.foo` `'True` creation (identity), proving
  recoordination works through `TailConcatenationFir` — the FOOP-55 usage
  shape.

**Einmo approval tests** under `foolish-ubca/einmo_suite/input/foop/65/`:
- `tail_concat_basic.foo` (equivalence pairs, side by side),
  `tail_concat_chain.foo` (the flat chain), `tail_concat_system_ops.foo`
  (`'lt`/`'eq` via backtick), `comprehensive.foo` (reserved name — mix the
  backtick with searches, `$`, markers, nested branes).
- **Non-regression gate:** the full einmo suite stays green — no foreign
  baseline may diverge (backtick-in-comments verified above;
  `rust_instructions.md` §"Phase-by-phase testing discipline").

## Rejected Alternatives

### A. Tighter precedence — the tail concatenator binds before juxtaposition

Atlas's own breakdown rejects it (§2): in `f`g`h`a b c` a tighter backtick
grabs operands out of the surrounding juxtaposition and the leftover
interleaves wrongly (`a (h g f) b c` instead of `a b c (h g f)`). Weakest
is the only precedence at which every operand is a fully-grouped
juxtaposition.

### B. Nested binary parse/AST (left- or right-associative folding)

Within a run, concatenation is associative (§2, restated deliberately), so
a nested tree denotes the same statement sequence as the flat chain while
carrying a nesting choice that means nothing. Flat n-ary is simpler and
matches `Astn::Concatenation`'s existing shape. The left-then-right
associativity history is recorded in §2 for provenance.

### C. Pure parse-time desugar to `Astn::Concatenation` — no dedicated FIR

Cheapest today, but Atlas explicitly wants the separate FIR: upcoming
features need to recognize "this concatenation is a tail concatenator" and
may change the `value()` output. Desugaring would delete exactly the hook
those features need, forcing the syntax to be re-litigated.

### D. Do nothing

Application stays postfix-only; the method-name-first idiom — and the
ergonomic ground FOOP-55's exercise wants to showcase — does not exist.

## Open Questions

- **`$`-after-backtick ergonomics.** Today the application result needs
  parentheses: `(fn`X)$` (a bare `fn`X$` keeps the `$` inside the right
  operand, per §2 — Atlas's example). Whether a future convenience exists
  (possibly riding the wrapper's future `value()` features) is deferred —
  nothing in this FOOP depends on it.
- **The wrapper's future features.** Deliberately unspecified here; this
  FOOP only guarantees the hook exists and current `value()` is the inner
  concatenation.
- **Style guidance.** Whether `docs/howto` should prefer the backtick idiom
  for application — a documentation decision after this lands.

## References

- Prior FOOPs: FOOP-3 lineage and FOOP-13 (concatenation semantics /
  ConcatBrane — the associativity this FOOP restates and rides); FOOP-55
  (first consumer — the Euler exercise rewritten in backtick form);
  FOOP-33 §5.0/§5.1 (the system-operator application idiom the backtick
  targets: `{1, 2, 'lt}$` → `('lt`{1, 2})$`).
- Process: `foop.md`; `rust_instructions.md` §"Phase-by-phase testing
  discipline"; AGENTS.md (NYES-transition test mandate, einmo workflow).
- Code anchors: `foolish-parser/src/lexer.rs` (single-char arms 173-285,
  unknown-char fallback 297-299); `foolish-parser/src/parser.rs`
  (`parse_expr` 371-388, `is_concatenation_continuation` 390-411,
  parse_primary 947-1067); `foolish-parser/src/token.rs`;
  `foolish-parser/src/ast.rs` (`Astn::Concatenation` shape);
  `foolish-ubca/src/compiler.rs` (`build_fir` Concatenation arm,
  `BodyOverride` 468-505); `foolish-ubca/src/fir_kinds.rs`
  (ConcatenationFir, `constanic_clone_at`).
- Reproductions (2026-08-07, `jia` @ `62706518`): juxtaposition application
  baseline `{1,2} t` → flat splice `[1, 2, p=9]`; backticks in the einmo
  corpus only inside `!!` comments (`foop/13/comprehensive.foo:17-18,65`,
  `exercises/project_euler/1.foolish:15`).

## Last Updated

**Date**: 2026-08-07
**Updated By**: Sisyphus / qwen3.8-max
**Changes**: Created (Draft). Specified the backtick tail concatenator per
Atlas's three clarifications: `fn`{p}` ≡ `{p} fn`; weakest precedence with
brane concatenation first (tighter binding breaks — `a (h g f) b c` vs
`a b c (h g f)`); within a run concatenation is associative, so the chain
is flat n-ary reversing source order (`f`g`h`a b c` ≡ `a b c h g f`).
Dedicated `TailConcatenationFir` has-a ConcatenationFir, `value()` returns
the inner, as the hook for future features that may change `value()`.
Non-regression verified: corpus backticks only in comments.
