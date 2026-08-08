---
foop: D65
title: The tail concatenator — backtick application that brings the method name to the front
author: Sisyphus / qwen3.8-max (directed by Atlas)
status: Draft
type: Standards
created: 2026-08-07
phase: phase-2
supersedes: []
depends_on: [FOOP-95]
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
then tail concatenation. **The tail concatenator has the same combining
effect as ordinary brane concatenation** — it *is* a concatenation, and it
reuses the existing `ConcatenationFir` rather than introducing a new FIR
kind. Provenance is carried by a **flag** on `ConcatenationFir` recording
whether the node came from a tail concatenation or from ordinary
juxtaposition; the flag affects **sequencing only** — never evaluation.
**Precedence is resolved entirely in the FVM's Foolish→FIR translation**
(`build_fir`), so the FIR layer sees only ordinary concatenations and the
whole change collapses to a parser/lexer/sequencer update. The special
backtick rendering applies **only while all of a node's constituents are
still embryonic** (§5.3.1); consequently the reversal is observable only
before stepping, and the facility that makes it observable — the AS-PARSED
pre-step perspective, together with the `stmt_count` purity repair it
requires — is **[FOOP-95](FOOP-95.md)**, on which this FOOP depends for
that one rendering (§6). This FOOP adds no evaluation behavior beyond the
equivalence. The tail concatenator does **not** obviate the `'`
null-characterization in the name of the method.

## Motivation

Foolish application today is postfix juxtaposition — `{lv,3,0}'cmod`,
`{cond1, sum+lv, sum, 'ite}`, `{loop} loop` — the argument brane first, the
method name last. It works, but it reads inside-out compared to the
function-call notation every Foolisher knows from every other language.

The backtick brings the method name to the front — `'cmod`{lv,3,0}`,
`'ite`{cond1, sum+lv, sum}` — while meaning *exactly* the same
concatenation. It is not a *silent* parse-time rewrite: the FIR keeps a
provenance flag saying "this concatenation came from a tail concatenator",
so the sequencer can render the backtick form back out, and so later
features that need to distinguish the two spellings have a place to hang
without re-litigating the syntax. But it is also not a whole new FIR kind:
the combining behaviour is *identical* to ordinary brane concatenation, so
a flag on `ConcatenationFir` buys everything a separate type would, at a
fraction of the code (§5, and Rejected Alternative C).

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

#### §3.1 The worked example — `` a`b`c`d e f `` (authoritative)

This example is the clearest statement of the whole semantics; every other
rule in §2–§5 can be read off it.

```foolish
a`b`c`d e f
```

**Step 1 — operand split (§2, backtick weakest).** The backtick is weaker
than juxtaposition, so juxtaposition groups first and each backtick operand
is a fully-grouped expression. The four operands are:

```
a  |  b  |  c  |  d e f
```

— the trailing `d e f` is ONE operand (an ordinary brane concatenation),
not three.

**Step 2 — flat chain (§2 associativity, §4 parser).** The run parses to a
single flat node in **source** order:

```
TailConcatenation [ a, b, c, Concatenation[d, e, f] ]
```

**Step 3 — reversal (§3).** The chain's meaning is the operands
concatenated in **reverse source order**:

```
a`b`c`d e f     ≡     (d e f) c b a
```

**Step 4 — the FIR the FVM builds.** The result is **exactly two
`ConcatenationFir`s**:

```
Concat[tail-flagged] ( Concat[ordinary]( d, e, f ), c, b, a )
```

- The **inner** `Concat(d, e, f)` is the juxtaposition operand — grouped
  first because the backtick is weakest.
- The **outer** concatenation carries the tail-concatenation flag and holds
  the reversed chain: the inner concatenation, then `c`, `b`, `a`.
- The reversal is performed **once**, in `build_fir`, when the flat
  `Astn::TailConcatenation` is lowered (§5). Nothing downstream of the
  compiler ever re-reverses anything.

Two operands are *not* wrapped in an extra node each: `a`, `b`, `c` are
plain element FIRs of the outer concatenation, exactly as they would be in
the ordinary spelling `d e f c b a`. Evaluated, the two spellings are
statement-for-statement identical — the only difference the flag makes is
that the sequencer may render the outer node back in backtick form.

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

### §5. `ConcatenationFir` plus a provenance flag — no new FIR kind

**Revised 2026-08-08 (Atlas).** An earlier revision of this FOOP specified a
separate `TailConcatenationFir` that has-a `ConcatenationFir` and delegates
to it. That design is **withdrawn** (it survives as Rejected Alternative C).
Atlas's direction:

> "The tail concatenation needs to have similar combining effect as normal
> brane concatenation. […] Since TailConcatFir is just a ConcatenationFir,
> we might even just use ConcatenationFir and use a flag that says 'this
> came in from a tail concatenation, or normal concatenation'. Seems easiest
> implementation. That flag only causes sequencing differences. The
> precedence is implemented as part of FVM translation from Foolish to FIR.
> This reduces the changes to this project to a much smaller
> parser/lexer/sequencer update."

The reasoning that survives from the withdrawn design: a tail concatenation
**is** a concatenation, and it must remain recognizable as having come from
a backtick. What changes is the *mechanism* for that recognizability — a
one-bit field instead of a wrapper type.

#### §5.1 The flag

```rust
// foolish-ubca/src/fir_kinds.rs — ConcatenationFir gains ONE field
pub struct ConcatenationFir {
    pub(crate) core: ProtoBrane,
    pub(crate) _helpers_populated: std::cell::Cell<bool>,
    /// Provenance: how this concatenation was spelled in source.
    /// Affects SEQUENCING ONLY — never evaluation (§5.3).
    pub(crate) provenance: ConcatProvenance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConcatProvenance {
    /// Ordinary brane concatenation (juxtaposition): `{a}{b}{c}`.
    Juxtaposition,
    /// Tail concatenation (backtick chain): `` c`b`a `` — the elements are
    /// already stored REVERSED relative to source (§5.2).
    TailConcatenation,
}
```

`Juxtaposition` is the default; every existing construction site keeps its
current behaviour by taking it.

#### §5.2 Precedence and reversal live in the FVM translation

**All of the tail concatenator's distinctive work happens in `build_fir`**
(`foolish-ubca/src/compiler.rs`), when the flat `Astn::TailConcatenation` is
lowered:

1. Build the operand FIRs in source order, exactly as the existing
   `Astn::Concatenation` arm does (reusing `build_concat_element`).
2. **Reverse** them (§3).
3. Construct a `ConcatenationFir` over the reversed elements with
   `provenance: ConcatProvenance::TailConcatenation`.

That is the entire compiler change: one new `match` arm that differs from
the `Astn::Concatenation` arm by a `.rev()` and a flag value. Precedence —
"juxtaposition groups first, then the backtick" — is already fully resolved
by the time `build_fir` runs, because the parser produced the operand
grouping (§4); the FIR layer never reasons about precedence at all.

#### §5.3 What the flag does and does not do

- **Evaluation: nothing.** `fir_op_step`, `stmt_count`, `stmt_at`,
  `populate_concat_helpers`, the join-readiness gate, the type-error path,
  and NYES settling are **untouched** and read the flag never. A
  tail-flagged concatenation steps through the identical
  Prembrionic → Embryonic → Braning → settled progression and produces the
  identical joined brane as the juxtaposition spelling of the same reversed
  elements. This is the testable invariant: **`fn`X` and `X fn` settle to
  branes that are statement-for-statement identical, and differ only in how
  an un-settled form is rendered.**
- **Sequencing: the one difference, and only while wholly un-stepped.**
  A tail-flagged concatenation renders in backtick form — the elements in
  *reverse storage order*, joined by `` ` `` — **only when every one of its
  constituents is still EMBRYONIC** (Atlas, 2026-08-08; "embryonic" here
  covers the pre-stepped states, i.e. every constituent is PREMBRYONIC or
  EMBRYONIC — none has begun BRANING or settled). The moment any
  constituent has progressed further, the node renders in the ordinary
  concatenation form (§5.3.1). A **settled** concatenation renders as its
  joined brane and the flag is invisible there, exactly as it is for
  juxtaposition.
- **Constanic clone / recoordination: propagate the flag.** The existing
  `FirKind::Concatenation` arm of `constanic_clone_at`
  (`fir_kinds.rs:339-356`) already copies `_helpers_populated`; it copies
  `provenance` the same way. **No new `constanic_clone_at` arm.**
- **NYES: no new states, no new transitions.** ConcatenationFir's existing
  progression is the progression. (The AGENTS.md `*_nyes_transitions`
  mandate therefore applies as an *extension* of the existing concatenation
  transition test — a tail-flagged case added to it — not as a new test for
  a new kind.)

#### §5.3.1 The all-embryonic gate on backtick rendering

**Atlas, 2026-08-08:** *"The sequencer shall only perform special rendering
on tail concatenations when all the constituents are embryonic."*

This settles the question this FOOP previously left open (whether backtick
rendering is round-trip or canonical) with a third, sharper answer: it is
**round-trip, but only for the un-stepped program.**

```
render(concatenation C):
    if C.provenance == TailConcatenation
       and every element of C is still embryonic   → backtick form
                                                        (elements reversed)
    else                                            → existing rendering,
                                                        unchanged
```

Rationale — why the gate is the right rule, not a restriction bolted on:

- **The backtick is a fact about source, not about values.** Once stepping
  begins, the elements are being replaced by their resolved values and the
  reversed textual chain no longer describes what is there. Rendering
  `` c`b`a `` over half-resolved elements would be actively misleading.
- **It makes the flag's blast radius provably tiny.** Under this gate the
  flag can only alter output for a FIR tree on which *nothing has been
  stepped*. Every existing einmo baseline is produced by stepping to
  settlement, so **no existing OUTPUT can move** — the non-regression
  argument becomes structural rather than empirical.
- **It keeps §5.3's evaluation invariant intact.** The gate reads NYES; it
  never writes it.

**Consequence — the reversal is otherwise unobservable.** With this gate,
the only way to ever *see* a tail concatenation rendered in reverse-order
backtick form is to sequence the program **before any step has been
taken**. That is exactly what §6 adds.

#### §5.4 The plumbing the flag has to cross

The flag is born in `foolish-ubca` but read in `foolish-core`'s sequencer,
so it must cross the `FirQueryable` bridge:

- `foolish-core/src/fir.rs`: `ConcatenationQuery` (line 563) is today
  `(Vec<Box<dyn FirQueryable>>, QueryChild)`. It gains the provenance as a
  third component, and `ConcatenationFir` / `ConcatenationFirBuilder` (lines
  356-360, 2096-2136) gain the matching field with a `Juxtaposition`
  default.
- `foolish-ubca/src/evaluator.rs`: the `FirKind::Concatenation` arm of
  `proto_to_core_fir_inner` (line 706) passes the flag through to
  `ConcatenationFirBuilder`. The settled path (which builds a
  `NormalBraneFir` from the joined statements) is untouched — settled tail
  concatenations render as branes, flag or no flag.

This is mechanical, additive, and touches no evaluation logic. **No new
`FirKind` variant, no new `constanic_clone_at` arm, no new `fir_op_step`,
no new NYES state.**

#### §5.5 The tradeoff, stated plainly

The withdrawn design bought a **distinct type** — a place for future
features to add fields and to change `value()` — at the cost of a new
`FirKind` variant, a new `constanic_clone_at` arm, a delegating
`fir_op_step`, a wrapper the recoordination path must be proven to survive,
and a `value()` indirection every consumer must get right.

The flag design gives up the distinct type. Future features that want to
change what a tail concatenation *evaluates to* (rather than how it renders)
would have to either branch on the flag inside `ConcatenationFir` — which
would violate §5.3's "sequencing only" invariant and should be treated as a
signal to revisit this decision — or promote the flag back into a separate
FIR at that time. **That promotion is a strictly smaller change than the one
this FOOP avoids**, because by then the syntax, the precedence, the parser,
and the reversal will all be settled and tested; only the FIR representation
would move. Atlas judged the near-certain code savings worth the deferred,
recoverable option. See Rejected Alternative C.

### §6. Dependency — the AS-PARSED perspective is [FOOP-95](FOOP-95.md)

**Scope split, Atlas 2026-08-08:** *"So this now seems like a rather large
project. Perhaps separate 'activate pre-step perspective of Foolish' into
its own FOOP. I'd imagine there's much work to be done there to straighten
up the sequencer."* — and: *"Then render FOOP-65 dependent on that FOOP."*

The pre-step ("AS-PARSED") perspective, and the `stmt_count` purity repair
it requires, are **specified in [FOOP-95](FOOP-95.md)**, not here. They
outgrew this FOOP: between splitting a mutating accessor across ~20 call
sites, straightening up the sequencer to render an un-stepped tree
faithfully, and re-promoting *every* baseline in the einmo corpus, that
work is larger than the tail concatenator itself.

**FOOP-65 depends on FOOP-95.** The dependency is real but narrow, and it
is about *observability*, not about evaluation:

- Everything in §1–§5 — the backtick token, the parse level, the flat
  chain, the reversal in `build_fir`, the provenance flag, and the
  evaluation equivalence `fn`X` ≡ `X fn` — is **fully testable without
  FOOP-95**, through parser unit tests, compiler-shape tests, FVM
  equivalence tests, and ordinary settled einmo baselines.
- What FOOP-95 unlocks is the **§5.3.1 rendering only**. Because backtick
  form is emitted solely while every constituent is still embryonic, and
  because every existing einmo baseline is rendered *after* stepping to
  settlement, there is **no existing test vantage from which the reversed
  backtick form is visible**. FOOP-95's pre-step perspective is that
  vantage.

**Consequences for sequencing the work:**

- FOOP-95 **should land first**. FOOP-65 can then verify §5.3.1 the moment
  it lands.
- If FOOP-65 lands first, the §5.3.1 sequencer branch must still be
  implemented and unit-tested at the `foolish-core` level (a directly
  constructed all-embryonic tail-flagged node, formatted and asserted) —
  that path does not need einmo. The einmo-level confirmation is then
  deferred to FOOP-95, and FOOP-65's plan carries a checkbox to add it.
- Nothing in FOOP-95 depends on FOOP-65. FOOP-95's value is general (it
  shows what the parser and `build_fir` produced for *any* program); the
  tail concatenator is merely its first sharp-edged consumer.

The einmo baselines this FOOP promotes are therefore **settled-only**, and
this FOOP does **not** touch any foreign baseline (see Test Plan).

#### §6.1 Significant step — inspection of embryonic Foolish

**Atlas, 2026-08-08:** *"In both FOOPs please indicate that a significant
step would be 'inspection of embryonic Foolish for reasonably informative
rendering of the Foolish for purpose of development', that'd be both by
agent and by human"* — *"…for purposes of future development writing and
maintaining Foolish programs."*

This step is shared with [FOOP-95](FOOP-95.md) §4, where it is specified in
full. It applies here in its own right, scoped to this FOOP's construct:
**the tail-flagged concatenation's embryonic rendering must be inspected —
by agent and by human — and judged reasonably informative for the purposes
of future development, writing and maintaining Foolish programs**, before
its form is frozen into any baseline.

Concretely, for FOOP-65 that means asking of `` a`b`c`d e f `` rendered
all-embryonic (§5.3.1):

- Does the rendering read as **Foolish** a program author recognises —
  visibly related to what they typed?
- Can they read the **operand grouping** off it: that `d e f` is one
  operand, and that `c`, `b`, `a` are the reversed chain (§3.1)?
- Is the backtick form actually *more* informative here than the
  juxtaposition form would be? If a reader would be better served by
  seeing `d e f c b a`, that is a finding against §5.3.1, and it should be
  reported rather than quietly rendered.

The human's judgement governs; the agent may not settle this on its own
assessment. If FOOP-65 lands before FOOP-95, this inspection is performed
against directly-constructed `foolish-core` unit-test renderings (the same
vantage §6 describes) rather than einmo output.

## FIR Impact

- **NO new FIR kind. NO new `FirKind` variant. NO new `constanic_clone_at`
  arm.** (This is the §5 revision; the earlier separate-FIR plan required
  all three.)
- **`ConcatenationFir` gains one field** — `provenance:
  ConcatProvenance` (`Juxtaposition` | `TailConcatenation`), defaulting to
  `Juxtaposition` at every existing construction site (§5.1).
- **The existing `FirKind::Concatenation` arm of `constanic_clone_at`
  copies the new field**, alongside the `_helpers_populated` copy it
  already does (`fir_kinds.rs:339-356`). No new arm.
- **New token `Backtick`** (foolish-parser) + `Astn::TailConcatenation`
  (flat, n-ary, source order) — the AST node exists only between the
  parser and `build_fir`; it does not survive into the FIR.
- **`foolish-core` bridge widens by one value**: `ConcatenationQuery`
  (`fir.rs:563`), `ConcatenationFir` (`fir.rs:356-360`), and
  `ConcatenationFirBuilder` (`fir.rs:2096-2136`) each carry the provenance
  so the sequencer can read it (§5.4).
- **No new NYES states.** Serialization/rendering of a **settled** tail
  concatenation is byte-identical to the equivalent juxtaposition (both
  render as the joined brane); only the **un-settled** rendering may differ.

## UBC Step Impact

- **NO new `fir_op_step`.** `ConcatenationFir::fir_op_step` is unchanged
  and never reads the flag — a tail-flagged concatenation steps exactly as
  a juxtaposition one (§5.3).
- **Precedence is resolved in the Foolish→FIR translation** (`build_fir`),
  not in the FIR: one new `match` arm for `Astn::TailConcatenation` that
  builds the operands with the existing `build_concat_element` machinery,
  **reverses** them, and sets the flag (§5.2). The reversal happens once,
  at compile time.
- **Parser gains one precedence level** above the current expression
  grammar (§4); the juxtaposition loop, `$`, and search-suffix paths are
  untouched.
- **Sequencer gains one conditional** in the un-settled concatenation
  branch (`foolish-core/src/sequencer.rs:496-531`) to render backtick form,
  gated on all constituents being embryonic (§5.3, §5.3.1).
- **NOT in this FOOP:** the `stmt_count` purity split and the AS-PARSED
  pre-step perspective moved to [FOOP-95](FOOP-95.md) (§6). This FOOP
  touches neither `stmt_count` nor the einmo output format.
- **Concatenation semantics untouched** — the tail concatenator is
  concatenation, spelled differently, with the same combining effect.
- **Net scope:** a parser/lexer change, one compiler arm, one struct field
  threaded through the query bridge, and one sequencer branch. No stepping,
  cloning, or NYES change; no change to any existing baseline.

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

**Unit — compiler shape (foolish-ubca):** the §3.1 worked example is a test.
- `` a`b`c`d e f `` compiles to **exactly two** `ConcatenationFir`s: an
  outer one flagged `TailConcatenation` with four elements — element `[0]`
  an inner `ConcatenationFir` flagged `Juxtaposition` holding `d, e, f` in
  source order, then `c`, `b`, `a` — per §3.1 Step 4.
- The reversal happens once, at build time: the outer node's stored element
  order is already `[Concat(d,e,f), c, b, a]`.

**Unit — FVM (foolish-ubca):**
- **Extend** the existing concatenation NYES-transition test with a
  tail-flagged case (AGENTS.md's `*_nyes_transitions` mandate applies as an
  extension here, not a new test — there is no new FIR kind). It asserts
  the flag changes *nothing* about the progression.
- **Equivalence (the §5.3 invariant):** for several X (brane literal,
  search, concatenation), `fn`X` and `X fn` settle to the same brane,
  statement-for-statement.
- **Chain reversal:** `f`g`h`X` settles as `X h g f`.
- **Flag survives recoordination:** constanic-clone a tail-flagged
  concatenation and assert the clone's `provenance` is still
  `TailConcatenation` (§5.3 — the existing `FirKind::Concatenation`
  clone arm copies it).
- **Flag is evaluation-inert:** construct two `ConcatenationFir`s over the
  *same* element FIRs differing only in `provenance`; assert identical
  settled NYES and identical joined statements.
- **System-operator application:** `('lt`{1, 2})$` → the `system.foo`
  `'True` creation (identity), proving recoordination works through the
  tail-flagged concatenation — the FOOP-55 usage shape.

**Unit — sequencer (foolish-core):**
- An **all-embryonic** tail-flagged concatenation renders in backtick form,
  elements reversed; the juxtaposition-flagged equivalent renders in
  juxtaposition form (the only observable difference the flag makes).
- **The §5.3.1 gate:** once ANY constituent has progressed past embryonic,
  the tail-flagged node renders in the ordinary form — pin this with a test
  that steps exactly one constituent and asserts the rendering flips.
- A **settled** tail-flagged concatenation renders as its joined brane —
  byte-identical to the settled juxtaposition equivalent.

**Deferred to [FOOP-95](FOOP-95.md):** the `stmt_count` purity tests and
the AS-PARSED perspective tests. The einmo-level confirmation that
`` a`b`c`d e f `` *renders* in reversed backtick form also lands there —
it needs FOOP-95's pre-step vantage (§6). The `foolish-core` unit test of
the same rendering (directly-constructed all-embryonic node) stays here.

**Einmo approval tests** under `foolish-ubca/einmo_suite/input/foop/65/`:
- `tail_concat_basic.foo` (equivalence pairs, side by side),
  `tail_concat_chain.foo` (the flat chain — **including the §3.1 worked
  example `` a`b`c`d e f `` beside its juxtaposition twin `d e f c b a`,
  which must settle identically**), `tail_concat_system_ops.foo`
  (`'lt`/`'eq` via backtick), `comprehensive.foo` (reserved name — mix the
  backtick with searches, `$`, markers, nested branes).
- **Non-regression gate:** the full einmo suite stays green — **no foreign
  baseline may diverge at all** (backtick-in-comments verified above;
  `rust_instructions.md` §"Phase-by-phase testing discipline"). With §6
  moved to FOOP-95, this FOOP promotes only its own new `foop/65/`
  baselines and touches no existing one — the ordinary, strict rule
  applies with no exception needed.

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

### C. A separate `TailConcatenationFir` that has-a `ConcatenationFir`

**This was the design of this FOOP's first revision; withdrawn 2026-08-08
by Atlas in favour of the §5 flag.** It specified a new FIR kind whose
single foolish child is an inner `ConcatenationFir`, with a delegating
`fir_op_step`, `value()` returning the inner's value, and a dedicated
`FirKind::TailConcatenation` arm in `constanic_clone_at` that clones the
inner and re-wraps it.

Rejected because the tail concatenator's **combining effect is identical to
ordinary brane concatenation** — the wrapper's every method was either
delegation or a no-op. The cost was real: a new `FirKind` variant, a new
`constanic_clone_at` arm, a new `fir_op_step`, a `value()` indirection every
consumer must honour, and a wrapper whose survival through recoordination
must be separately proven. The flag design achieves the same recognizability
("this concatenation came from a backtick") with one struct field and one
sequencer branch (§5.1, §5.3).

**What is given up, honestly:** a distinct type. The withdrawn design was
justified as "the hook on which future features — which may change the
`value()` output — will hang". A flag is a weaker hook: it can carry
provenance, but a feature that changed *evaluation* based on it would break
the §5.3 "sequencing only" invariant that makes this design safe. Should
such a feature actually arrive, the right move is to promote the flag back
into a separate FIR then — a **strictly smaller** change than doing it now,
because the syntax, precedence, parser, and reversal will already be
settled and tested; only the FIR representation would move. Atlas judged
the certain code savings today worth the deferred, recoverable option
(§5.5).

### C2. Pure parse-time desugar to `Astn::Concatenation` — no provenance at all

Cheapest of all: lower `Astn::TailConcatenation` straight to an ordinary
`ConcatenationFir` with the elements reversed, recording nothing. Rejected
because it is genuinely lossy — the sequencer could never render the
backtick form back out (`fn`X` would always echo as `X fn`), and no future
feature could recognize the spelling. The flag (§5.1) costs one field and
removes exactly this objection, which is why it, and not this, is the
design.

### D. Do nothing

Application stays postfix-only; the method-name-first idiom — and the
ergonomic ground FOOP-55's exercise wants to showcase — does not exist.

## Open Questions

- **`$`-after-backtick ergonomics.** Today the application result needs
  parentheses: `(fn`X)$` (a bare `fn`X$` keeps the `$` inside the right
  operand, per §2 — Atlas's example). Whether a future convenience exists
  is deferred — nothing in this FOOP depends on it.
  **ANSWERED by [FOOP-75](FOOP-75.md) §9.3** (Assignment Attached
  Searches): an attached search applies to the *whole* RHS, and since the
  backtick is the weakest operator (§2), the chain **is** the whole RHS.
  So `A =$ fn`X` is the parenthesis-free spelling of `A = (fn`X)$` — the
  same tree. This remains a non-dependency in both directions: FOOP-75
  §9.4 confirms neither FOOP blocks the other and either may land first.
  Two coordination items: (a) FOOP-75 §5.3 adds a `preceded_by_space` field
  to `TokenAndLocation`, which this FOOP's new `Token::Backtick` arm must
  populate if FOOP-75 lands first; (b) **FOOP-75 §9's comparison table is
  now stale** — it records this FOOP as "New FIR? yes —
  `TailConcatenationFir` (a deliberate hook)" and its sequencing obligation
  as "render through the inner concatenation". Under the §5 revision both
  entries change to "no new FIR — a provenance flag on `ConcatenationFir`"
  and "render the flagged concatenation in backtick form". This *narrows*
  the difference between the two FOOPs (both now reuse existing FIRs) and
  changes nothing about FOOP-75's own design; FOOP-75 §9 should be
  corrected when it is next edited.
- **The flag's future features.** Deliberately unspecified here; this FOOP
  only guarantees the provenance is recorded, survives recoordination, and
  affects sequencing alone (§5.3). If a future feature needs the tail
  concatenation to *evaluate* differently, that is the trigger to revisit
  §5 and promote the flag to a distinct FIR (§5.5, Rejected Alternative C)
  — it must not be smuggled in as an evaluation branch on the flag.
- **Un-settled backtick rendering — round-trip or canonical?**
  **ANSWERED by Atlas (2026-08-08), now §5.3.1:** round-trip backtick form,
  but **only while every constituent is still embryonic**. Once any
  constituent has progressed, the ordinary rendering resumes. This is
  strictly narrower than either option originally posed, and it is what
  makes the flag's blast radius provably nil for existing baselines.
- **Ordering against FOOP-95.** FOOP-65 depends on
  [FOOP-95](FOOP-95.md) only for the einmo-level *visibility* of §5.3.1
  (§6). FOOP-95 landing first is preferable; if FOOP-65 lands first the
  rendering is still unit-tested at the `foolish-core` level and the einmo
  confirmation is deferred. Atlas to confirm the intended order.
- **Style guidance.** Whether `docs/howto` should prefer the backtick idiom
  for application — a documentation decision after this lands.

## References

- **Dependency: [FOOP-95](FOOP-95.md)** (the AS-PARSED pre-step
  perspective + the `stmt_count` purity split) — provides the only vantage
  from which §5.3.1's backtick rendering is observable in einmo (§6).
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
  `foolish-ubca/src/compiler.rs` (`build_fir` Concatenation arm 371-383,
  `build_concat_element`, `validate_astn` Concatenation arm 167-172,
  `BodyOverride` 468-505); `foolish-ubca/src/fir_kinds.rs`
  (`ConcatenationFir` struct 2610-2614, `fir_op_step` 2749-2835,
  `constanic_clone_at` `FirKind::Concatenation` arm 339-356);
  `foolish-ubca/src/fir_trait.rs` (`enum FirKind` 31 — **unchanged** under
  the §5 revision); `foolish-ubca/src/evaluator.rs`
  (`FirKind::Concatenation` → `ConcatenationFirBuilder`, 706-764);
  `foolish-core/src/fir.rs` (`ConcatenationFir` 356-360,
  `ConcatenationQuery` 563, `hs_concatenation` 581/799,
  `ConcatenationFirBuilder` 2096-2136); `foolish-core/src/sequencer.rs`
  (§9 concatenation rendering, 496-545 — the un-settled branch is the one
  the flag touches).
- Reproductions (2026-08-07, `jia` @ `62706518`): juxtaposition application
  baseline `{1,2} t` → flat splice `[1, 2, p=9]`; backticks in the einmo
  corpus only inside `!!` comments (`foop/13/comprehensive.foo:17-18,65`,
  `exercises/project_euler/1.foolish:15`).

## Last Updated

**Date**: 2026-08-08
**Updated By**: Claude Code / claude-opus-5
**Changes**: **Architecture revision (Atlas, 2026-08-08): the separate
`TailConcatenationFir` is withdrawn in favour of a provenance FLAG on the
existing `ConcatenationFir`.** Tail concatenation has the same combining
effect as ordinary brane concatenation, so §5 was rewritten as
`ConcatProvenance { Juxtaposition | TailConcatenation }` — one struct
field, propagated by the *existing* `FirKind::Concatenation`
`constanic_clone_at` arm, threaded to the sequencer through
`ConcatenationQuery`. The flag affects **sequencing only, never
evaluation** (§5.3); precedence and the reversal are resolved entirely in
the FVM's Foolish→FIR translation (`build_fir`, §5.2), reducing this FOOP
to a parser/lexer/sequencer change. Added §3.1, the authoritative worked
example `` a`b`c`d e f `` → exactly TWO ConcatenationFirs,
`Concat[tail](Concat[juxt](d,e,f), c, b, a)`, verified consistent with §2
(backtick weakest ⇒ `d e f` is one operand) and §3 (reverse source order).
Rewrote FIR Impact and UBC Step Impact (no new FIR kind, no new `FirKind`
variant, no new `constanic_clone_at` arm, no new `fir_op_step`); extended
the Test Plan (compiler-shape test for §3.1, flag-survives-recoordination,
flag-is-evaluation-inert, sequencer round-trip); the separate-FIR design
became Rejected Alternative C with its tradeoff stated (a distinct type
given up; re-promotion later is strictly smaller work), and the old
alternative C became C2. Added **§5.3.1**: backtick rendering happens
**only while all constituents are embryonic** (Atlas) — which answers the
old open question and makes the flag unable to move any existing baseline.
Added **§6 as a DEPENDENCY section**: the AS-PARSED pre-step perspective
and the `stmt_count` purity split (a real defect — `stmt_count` forces the
concatenation join, disagreeing with its own `stmt_at`/`_search_brane`
siblings) were specified here briefly, then **split out to
[FOOP-95](FOOP-95.md)** at Atlas's direction as too large to ride along;
FOOP-65 now `depends_on: [FOOP-95]` for that one rendering's einmo
visibility only. Unchanged by this revision: backtick is the WEAKEST
operator, the flat n-ary chain, associativity within a run, and `'`
null-characterization. Non-regression holds strictly — this FOOP promotes
only its own `foop/65/` baselines and changes no existing one.
