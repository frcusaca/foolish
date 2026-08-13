---
foop: D55
title: Project Euler 1 — the 'mod modulo operator, the 'or boolean operator, and the repairs that run the first exercise
author: Sisyphus / qwen3.8-max (directed by Atlas)
status: Implementing
type: Standards
created: 2026-08-07
phase: phase-4
supersedes: []
begun: [x]
---

# FOOP-55: Project Euler 1 — 'mod, 'or, and the repairs that run the first exercise

FOOP numbering is little-endian; the full rules live in `foop.md` at the
repository root — **read it before creating or editing a FOOP.** The one
template-specific note: the `foop:` front-matter field may either match the
filename digits directly, or give the big-endian decimal value preceded by
`D` (this file: `foop: D55`, which for the palindrome 55 is also plain `55`).
In all cases, the `FOOP-55.md` file name is ultimately the right numbering.

## Abstract

This FOOP makes the first real program written for the UBCa FVM run: the
Project Euler exercise at
`foolish-ubca/einmo_suite/input/exercises/project_euler/1.foolish` ("sum of
all multiples of 3 or 5 below 1000", expected answer **233168**). It adds
exactly two language features the exercise needs, plus one evaluation-strategy
repair without which the exercise cannot terminate at any step budget —

1. **`'mod`** — integer modulo as a null-characterized system operator,
   postfix: `{a, b, 'mod}$`. Implemented with the exact mechanism FOOP-33
   §5.0/§5.1 built for the comparison operators: declared `'mod = ⬤` in
   `system.foo`, body replaced at compile time by the `BodyOverride` hook,
   operands are the two SFF-marked brane-relative lookups `<<#-2>>` /
   `<<#-1>>`, detachment/recoordination supplies the caller's neighbours.
   Unlike the comparisons, `'mod` returns an **integer**, not a boolean.
2. **`'or`** — boolean OR on the `'True`/`'False` creations, defined **in
   pure Foolish inside `system.foo`** as a truth-table brane applied by
   search — the preferred design of FOOP-73, realized here for the single
   operator `'or`. No new FIR kind, no privileged FVM layer.
3. **The upgraded SFF mark (§5)** — a constanic clone currently strips **every**
   SF/SFF mark it meets in one recursive pass, so a mark can protect a term
   across exactly one coordination and no more. That is enough for a one-level
   macro and insufficient for anything that builds a lookup table and then
   selects from it — which is what `'ite` does. §5 makes the mark a **counter**:
   a clone strips **at most one** mark, the budget belonging to the whole clone
   operation rather than to any node. `<<X>>` is unchanged; `<< <<X>> >>` sits
   out one coordination and resolves at the next.

   This is a **correctness** fix, not an optimization: premature stripping does
   not merely resolve early, it resolves against the **wrong neighbours**, and
   an early miss settles NK — a terminal state no later recoordination can
   repair. It is also what makes the exercise terminate. The branch a value
   search does not select is never coordinated a second time, so its inner mark
   never comes off, so it never searches, so `<loop>` never recurses. **No
   laziness rule is added to the FVM, no evaluation order changes, and no FIR
   gains a new state** — the deferral is carried by the term. Measured
   2026-08-09: the exercise currently computes *nothing* (all PREMBRIONIC) at a
   40000-step budget.

   Three other designs were considered — an `@` operator projecting a search
   result's index, true breadth-first execution, and a message-passing
   evaluator. **Appendix A** sets out all four in prose and records why this one
   was chosen; it also **authoritatively defines the UBCa/UBCb/UBCc/UBCd code
   names** used from here on.

— and it **documents, with reproductions**, six platform defects (D1–D6)
and five exercise-file defects (E1–E5) found while bisecting the failure.
Per Atlas's direction (2026-08-07) the leading-underscore lexer defect (D1)
is **worked around, not fixed**: the exercise uses an `INTERN_` prefix
instead of a leading `_`, and this FOOP records the full details needed to
fix the parser later if that becomes desirable.

**Dependencies: none.** The FOOP-65 (tail concatenator) dependency was
**dropped 2026-08-09** — verified live that the exercise's juxtaposition
application form evaluates correctly as written, so E4's backtick rewrite is
optional polish rather than a precondition.

## Motivation

On 2026-08-07 the first exercise program was committed
(`62706518 "An exercise."`). It does not run. A bisect (reproductions
below) shows the failure is not one bug but a stack of them, spanning the
lexer, the exercise file itself, and two operators that do not exist yet.

The world after this FOOP: the exercise evaluates to constanic and its
einmo baseline reads `233168`; Foolish has integer modulo and boolean OR;
the defects found along the way are documented with reproductions and
dispositions; and the next exercise (Euler 2+) starts from a platform
whose sharp edges are known and pinned by tests.

### The program (as committed, before Atlas's fixes)

```foolish
!!!
# Multiples of 3 or 5
If we list all the natural numbers below 10 that are multiples of 3 or 5, we get 3,5,6 and 9.  The sum of these multiples is 23.
Find the sum of all the multiples of 3 or 5 below 1000.
!!!

{
  !! If-Then-Else
  _ite = {
    cond='True, <<#-3>>;
    cond='False, <<#-2>>;
  }
  'ite = ({<<-2>>, <<-1>>} _ite)~cond=(<<-3>>)&#1

  !! Congrulent modulo ```{a,b,c}'cmod``` computes boolean a%b==c
  _cmod = {_eq, {_numerator,_divisor, 'mod}$, 'eq};
  'cmod = {_numerator=#-3,_divisor=#-3,_eq=#-3, _cmod$}
  loop = {
    !! recursion without self-reference
    self=<<#-1>>
    !! lv is the loop variable, it comes from parent.
    !! sum35 is the accumulant.

    lv=lv+1;
    divides_3 $= {lv,3,0}'cmod;
    divides_5 $= {lv,5,0}'cmod;
    cond1 $=  {divides_3, divides_5, 'or};
    sum $= {cond1, sum+lv, sum, 'ite}

    cond2 = {lv, 1000, 'lt}$;
    continue = <self loop>;
    exit=sum35
    answer $= {cond2, <continue>, sum35, 'ite}
  }
  lv  = 1; sum = 0;
  answer $= {loop} loop
}
```

## Findings — platform defects (discovered by bisect, 2026-08-07)

Each defect carries a reproduction. All were run with
`target/debug/foolish-cli run <file>` on `jia` @ `62706518`.

### D1. A leading `_` in a name is not lexed — silent rename, then an inscrutable parse error. **WORKED AROUND, not fixed, per Atlas.**

**Mechanism.** The lexer starts an identifier only on `is_letter`
(`foolish-parser/src/lexer.rs:293`); `_` is accepted only *inside* an
identifier via `is_id_sep` (`lexer.rs:80-86`, `identifier()` at
`lexer.rs:413-439`). A leading `_` therefore falls into the
unknown-character **fallback** (`lexer.rs:297-299`):

```rust
// Fallback: skip unknown character
self.advance();
(self.make_token(Token::LineComment), false) // effectively skip
```

The fallback advances one character and emits a **fake `Token::LineComment`**.
Two consequences:

- **Silent rename.** `{_a = 1;}` evaluates to `a=1` — the `_` is swallowed
  and the statement is silently named `a`. Any code referring to `_a`
  refers to `a` too, so nothing visibly breaks until two different `_x`/`x`
  names collide.
- **Inscrutable parse error.** `Parser::expect` (`parser.rs:58-73`) does not
  call `skip_comments` (`parser.rs:88-95`), so when a fake comment lands
  where `expect` looks, the error is `expected <token>, found LineComment`
  — `expected` is the literal string `"<token>"` (`parser.rs:66`) and
  `found` names the *fake* token, so the message points at nothing.

**The exercise's original failure is exactly this.** Its line 13
(`'ite = ({<<-2>>, <<-1>>} _ite)~cond=(<<-3>>)&#1`) fails with
`expected <token>, found LineComment` because `_ite`'s leading `_` emits a
fake comment inside the parenthesized expression. (The reported position —
"line 9, column 23" — is itself wrong; see D2.)

Reproductions:

```foolish
!! q5.foo — silent rename: output is  a=1
{_a = 1;}

!! f1.foo — parse error: expected <token>, found LineComment
{_ite={}
x = ({1, 2} _ite);
}
```

**Decision (Atlas, 2026-08-07): do not change the lexer/parser in this
FOOP.** Foolish is to be treated, for now, as **disallowing a leading `_`
in names**. The exercise renames its underscore-prefixed names with an
`INTERN_` prefix (E3). This keeps FOOP-55 minimal: the exercise runs on the
parser as it is.

**Details for a future fix, if needed** (none of this is in scope here;
recorded so the fix is a copy-paste job later):

1. `lexer.rs:293` — start an identifier on `is_id_sep` as well as
   `is_letter`. Design decision to make first: does a leading `_` map to
   SEP (U+02CD) exactly like an interior `_` (so `_ite` and `INTERN˭ite`
   would be the *same* name — probably undesirable while the INTERN_
   convention lives), or is it kept distinct. Pin whichever choice with
   lexer tests (`lex_identifiers` style, `lexer.rs:461-464`).
2. `lexer.rs:297-299` — replace the fake-`LineComment` fallback with a real
   diagnostic (e.g. a `Token::UnknownChar(c)` the parser rejects with the
   character and position). Silently swallowing unknown characters is what
   made this defect opaque for as long as it was.
3. `parser.rs:58-73` — either make `expect` skip comments like the rest of
   the parse paths, or filter `LineComment`/`BlockComment` out of the token
   stream once in `parse()` (`parser.rs:31-35`) so no code path can ever
   see one. Each of the three fixes is independent and separately
   testable.

### D2. Block comments do not count newlines — error positions after a `!!!` block are wrong. **Documented; deferred with D1.**

`block_comment` (`lexer.rs:302-329`) consumes the comment body through
`advance()` (`lexer.rs:69-74`), which increments `column` only; a `\n`
inside the comment never increments `line` nor resets `column`. Repro: the
exercise's true failure is file line 13, reported as **"line 9, column
23"** — the five-line `!!!` header undercounts the line by four. Fix is the
same family as D1 (count `\n` inside `block_comment`); deferred with it.
Until fixed, **subtract the block-comment line span when reading any error
position that follows a `!!!` block.**

### D3. Unary minus loses its sign inside the SFF marker `<<...>>` (but NOT inside the SF marker `<...>`). **Documented; not a blocker.**

```foolish
{x = <<-5>>;}   !! → x=5   (sign LOST)
{x = <-5>;}     !! → x=-5  (sign kept)
{x = -5;}       !! → x=-5  (sign kept)
```

Both marker arms of the parser call the same `parse_expr`
(`parser.rs:962-977`), so the divergence is downstream — presumed to be the
compiler's `Astn::StayFullyFoolish` arm (the `build_fir` SFF path; cf.
`proto_brane.rs:376` "build_fir's SFF arm"). The implementation phase
should confirm by dumping the AST/FIR before any fix is attempted.

Not a blocker for this FOOP: the corrected exercise uses `<<#-N>>` index
searches, whose offsets parse through `parse_seek_index`
(`parser.rs:990-1000`) and work (repro: `{x = <<#-3>>;}` →
`#(offset=-3, UNANCHORED, ECONSTANIC)`). But any future program that writes
`<<-N>>` meaning a negative literal will silently get the wrong sign, so
this is recorded here for the FOOP that fixes it.

### D4. Juxtaposition inside `<...>`/`<<...>` accepts only brane/identifier starts. **Documented only.**

`{x = <1 2>;}` fails to parse (`expected <token>, found Integer(2)`):
`is_concatenation_continuation` (`parser.rs:390-411`) continues a
concatenation only on `LBrace|LParen|Ident|Up|LtLt|Lt`. The exercise's
`<self loop>` and `<continue>` start with identifiers and parse fine, so
this FOOP does not touch it.

### D5. (Observation, not a defect.) A contexted index search that lands on its own statement does not terminate.

`{y={cond=3;}\nx = y&#1;}` alarms `ALARM: ubca evaluation error: Iteration
exceeded 9999` (`evaluator.rs:168`) and stops BRANING: `&#1` from `y`
lands on `x`, whose body is the search itself. This is expected
self-reference behavior, noted here so it is not misdiagnosed as a bug
during Phase 3 integration (the exercise's own `&#1` uses land on *other*
statements and are cycle-free).

### D6. The `$=` assignment form does not parse; `=$` parses but settles non-obviously. **Documented; the exercise avoids both (E5).**

- `{x $= {1,2};}` → parse error `expected primary expression, found
  Assign` — the `$=` form the exercise uses six times is not accepted by
  the parser; only `=$` is (`parser.rs:324-340`, Assign then Dollar).
- `=$` desugars to `BinaryOp{ op:"$", left: UnanchoredSeek{offset:-1},
  right: B }` (`parser.rs:330-339`) — a `$` operator whose left operand is
  a `#-1` seek. With no preceding statement, `{x =$ {1,2};}` settles
  `x =$ ??? ({ is not a brane)`; with one, `{a=1;\nx =$ {a,2};}` leaves
  `x` WOCONSTANIC rendering `x =$ { (WOCONSTANIC) ... }` — not the clean
  `x = 2` the README's "A =$ B means A = B$" promises.
- Disposition: the `=$` sugar's true semantics need their own
  investigation (a future FOOP, out of scope here). FOOP-55's exercise
  rewrite uses explicit parentheses + `$` — `X = (application)$` — and
  depends on neither sugar form.

### D7. A system operator nested inside a juxtaposed definition never settles — unbounded `Index` recursion. **BLOCKS the pure-Foolish route.**

Found 2026-08-11 while implementing §5's Phase 2C (`congruent_modulo` in pure
Foolish). Not caused by §5 — reproduced at `2e3c18ab`, before the strip-budget
implementation.

**Reproduction.** Composed with `system.foo`:

```foolish
{
	f = {out = ({<<#-2>>, <<#-1>>, 'mod})$;};
	t = ({7, 3} f)$;
}
```

`t` never settles. The arithmetic control is fine — replacing the operator with
`+` (`f = {out = <<#-2>> + <<#-1>>;}`) gives `t=10` — and so is the operator
without the wrapping definition (`({7, 3, 'mod})$` gives `1`). Only the
*combination* fails.

**Mechanism.** The FIR tree under the stuck definition shows `Modulo`'s operand
growing without bound:

```
f[2] Statement Braning
  f[0] Search Braning ^'mod$
    u[0] Modulo Braning
      f[0] Index Braning
        u[0] Index Braning
          u[0] Index Braning        <- another layer every step
            u[1] FoolRef Constant
```

Each index resolution produces *another* `Index` in `ubc_children` rather than
settling to a value. The two operands that belong to the enclosing brane
(`StayFullyFoolish → Index Econstanic`) resolve and stop correctly; it is
`Modulo`'s own recoordinated operand that recurses.

**Consequence for this FOOP.** This is why plan Phase 2 abandoned the
pure-Foolish truth table and fell back to `OrFir` — the note there ("the value
search `T~A=A` inside system.foo can't resolve when `A` is ECONSTANIC") records
the symptom; this is the mechanism. It blocks **Phase 2C** (`congruent_modulo`)
and the pure-Foolish **`'ite`** of Phase 3D by the same route.

§5 is necessary but **not sufficient** for the pure-Foolish definitions. D7 must
be fixed as well, or `'ite`/`congruent_modulo` must be built some other way.
Disposition is an Open Question below.

### D8. **RETRACTED — not a defect.** SFF-marked self-reference correctly spins BRANING.

Filed 2026-08-11 and retracted the same day. Recorded because the mistake is
easy to repeat and the correction is the useful part.

**The claim was** that a search landing on an SFF-marked statement had no defined
outcome and spun BRANING, and that this was the root cause of D7 and of every
function-shaped program failing. The reproduction offered:

```foolish
{ n = <<#-1>>; r = n; }        !! BRANING forever
```

**The reproduction was malformed.** `n` sits at index 0, so its `#-1` reaches
*before the brane* — the definition is **self-referential**, and BRANING forever
is the honest answer to a self-referential definition, not a defect.

**The real distinction is SF versus SFF**, and it is a difference of *when*:

| Mark | Resolves | Right for |
|------|----------|-----------|
| `<X>` (SF) | **here**, at its own position | "look at my neighbour" |
| `<<X>>` (SFF) | **there**, wherever it is recoordinated to | a macro body that must not bind until applied |

Given a real preceding member, both behave exactly as specified:

```foolish
{100, n = <#-1>;  r = n+1;}    !! n=100, r=101   -- SF resolves here
{100, n = <<#-1>>; r = n+1;}   !! n stays marked -- SFF defers, so the
                               !! local `r = n+1` has nothing to read, forever
```

**The rule is not "local use wants SF".** It is: *where does the value come
from?*

| The value comes from | Mark |
|----------------------|------|
| my own brane's neighbours | `<X>` — SF, resolve here |
| the caller, at recoordination | `<<X>>` — SFF, resolve there |
| the caller, but it must survive an **extra** structural boundary first | `<< <<X>> >>` — one mark per boundary crossed |

The mark count is a **budget spent across a journey — one per CONSTANIC CLONE**.
Not one per syntactic boundary, and **not** "concatenation is free".

> **Measured 2026-08-11, correcting an earlier claim in this section.**
> Concatenation *does* spend marks:
> ```foolish
> t = {v = << <<#-1>> >>;};
> c = {1, 2} t;              !! c = {1; 2; v=2} -- BOTH marks gone
> ```
> Instrumenting the strip budget shows it working correctly (one
> `may_strip=false`, a mark properly retained) — but **two separate clones**
> each spend one. `ConcatenationFir` calls `constanic_clone_at` per statement
> (`fir_kinds.rs:2865`), and evaluation performs a further clone downstream.
>
> **A single source-level step can therefore perform more than one clone**,
> which means the required mark depth **cannot be counted by reading the
> source**. This is the concrete form of the standing SF/SFF concern below, and
> it is why the §5 mark-depth experiments bracketed `'ite` without landing it
> (one mark → the pattern died NK; three → nothing resolved): the depths were
> being guessed because they are not derivable.

With that caveat, `fbfn`'s recursive branch is verified to work **in the shape
tested**:

```foolish
key='False, value=<< <<({fbfn,param-1}fbfn)$>> >>
```

1. **As written** — two marks.
2. **The search for `fibtbl`** constanic-clones the table, spending one — the
   found row now carries `<<call>>`, still unresolved.
3. **`'match` selects it and it lands at the use site** — the second coordination
   spends the last, and only *now* does the call resolve.

A **single** mark would resolve at step 2, during the table lookup, before
`'match` had chosen anything — and the recursion would fire unconditionally. The
`'True` branch (`value=0`) needs no mark at all: a literal has nothing to defer.

Verified — the doubled branch survives the lookup and is never evaluated:

```
value=<<WOCONSTANIC <<WOCONSTANIC $(ECONSTANIC)>> >>   <- the call, un-run
r=0   out=0                                            <- stop branch taken
```

The failing bisection cases below were **not** wrong-mark choices — they were
SFF marks pointing at **nothing** (no preceding member to recoordinate from):

| Body | Result | Why |
|------|--------|-----|
| `{n = <<#-1>>; r = 100;}` | `100` | `n` is never read locally |
| `{k = 1; r = <<#-1>> + 100;}` | `101` | the mark is consumed in place, not via a name |
| `{n = <<#-1>>; r = n + 100;}` | hangs | SFF bound, then read **locally** — wrong mark |

**D7 is probably the same mistake.** Its reproduction —
`f = {out = ({<<#-2>>, <<#-1>>, 'mod})$;}` — uses SFF for operands consumed in
the *same* brane. D7 should be re-tested with SF before being treated as a
platform defect.

#### The standing concern

> **SF/SFF flexibility is dangerous and hard to reason about.** *(Atlas,
> 2026-08-11.)*

Both marks parse. Both look reasonable at a glance. One silently never
terminates, with no diagnostic naming which was wanted. The rule — *use SF when
the value is read in the same brane, SFF when it must survive to a use site* —
is simple to state and easy to get wrong, and getting it wrong costs an
interpreter hang rather than an error message.

Worth its own FOOP: either a diagnostic for the common failure (an SFF-marked
statement read by name within its own brane is almost certainly a usage error),
or a re-examination of whether both marks need to be surface syntax.

### D9. A **recoordinated ECONSTANIC clone is never enqueued**, so it never runs. **BLOCKS `'match`.**

Found 2026-08-13. Two earlier framings of this finding were wrong and are
superseded; the correction is recorded because the wrong readings are easy to
reach.

**The mechanism works exactly as designed, up to one step.** Atlas's trace of
`{a = {1,2}, b=<<#-2>>, c= a b}`:

1. the search for `b` **succeeds** — the statement is found
2. its body `<<#-2>>` is **constanic-cloned** into the search's `ubc_child`
3. the clone **strips the SFF mark**, activating the `#-2` search
4. the clone is **stepped to constanic** — and `#-2` now counts from `b`'s
   position in the **concatenation**, not from its definition site
5. all of the concatenation's `foolish_children` are then constanic

Steps 1–3 happen. **Step 4 does not.** The observed state:

```
?(result=#(offset=-2, UNANCHORED, ECONSTANIC), pattern='^b$', UNANCHORED, WOCONSTANIC)
   ↑ search found b            ↑ mark stripped, bare #-2 — but ECONSTANIC and never stepped
```

**The cause is one line** — `ProtoBrane::push_ubc_child`
(`proto_brane.rs:200-205`):

```rust
pub fn push_ubc_child(&self, child: FirRef) {
    self.ubc_children.borrow_mut().push(Rc::clone(&child));
    if !child.borrow().core().get_nyes().is_constanic() {   // <- ECONSTANIC is constanic
        self.tasks.borrow_mut().push_back(child);           //    so this never runs
    }
}
```

Instrumented and confirmed: `kind=Index nyes=Econstanic enqueued=false`.

**ECONSTANIC is doing double duty, and here the two meanings collide:**

- *"terminal — stop stepping me"*, which `is_constanic()` asserts
- *"no value **in that context**; may gain one via recoordination"*, which is
  what the state means

Both are true at the **original** site: the definition's `#-2` genuinely has
nothing to find and should stop. But this clone has just been **recoordinated
into a new context** — precisely the event ECONSTANIC says can revive it.
Cloning carried the state across unchanged, so a term that *should* now run
arrives pre-declared as finished.

**The fix.** A constanic clone that lands in a new context must be enqueued
rather than assumed settled: either the clone is born pre-constanic so the
existing guard enqueues it, or `push_ubc_child` distinguishes "terminal here"
from "terminal anywhere" and enqueues an ECONSTANIC child. The first is
narrower; the second states the real rule.

**Superseded framings, recorded so they are not re-derived:**

- *"No search resolves on a recoordinated brane operand."* Wrong —
  recoordination delivers correctly; the clone simply never runs.
- *"SF/SFF do not forward `stmt_count`, so `constanic_is_brane_like` is false."*
  True but not the cause: `<<{a=1;b=2}>>^` already works, because a literal
  brane's shape is lexical. Three-valued brane-likeness remains worth having —
  a `None` for "not yet knowable" is honest — but it is **not** what blocks
  `'match`.

### D10. A concatenation with a **deferred element** settles its shape too early. **Unimplemented case, not on `fbfn`'s path.**

Found 2026-08-13 while establishing D9's fix. **This is a case the concatenation
code was never written for**, rather than a defect in what it does handle:
`.unwrap_or(0)` is what one writes when every element is assumed present.

**Not on the critical path.** `fbfn`'s concatenations
(`{fbfn, param-1}fbfn`, `{fibtbl, cond}'match`) have all elements present at the
call site, so this does not block Euler 1 or fibonacci. Recorded so it is not
rediscovered as a mystery.


Asked whether a legitimate "don't know" can arise anywhere other than an
ECONSTANIC search, the answer is: **the root cause is always an ECONSTANIC
search, but concatenation propagates it as a silent wrong answer rather than as
a deferral.**

`ConcatenationFir::stmt_count` (`fir_kinds.rs:3230`) **never returns `None`**.
When its helpers are unpopulated it populates them and sums:

```rust
.map(|h| h.borrow().stmt_count().unwrap_or(0))
```

and `populate_concat_helpers` does the same over its elements. So an element
that cannot yet say how many statements it has is counted as having **zero**.

**Verified — an answer settled too early, not merely a wrong one:**

```foolish
{f = {c = {1,2} <<#-2>>; n = c$;}; tbl={x=8;y=9;}; r =$ {tbl,0}f}
   => r = 2
```

The marked element is `ECONSTANIC` — deferred, not failed. **`2` is not a wrong
count of what is locally known**; it is the honest count of the part that has
arrived. The defect is that the concatenation *commits* to it.

What it should be is **4**. A brane element contributes its **statements**,
flattened — verified with a fully-known element:

```foolish
{tbl={x=8;y=9;}; c = {1,2} tbl; cnt = c$;}
   => c = {1; 2; x=8; y=9}    cnt = 9
```

So once `<<#-2>>` resolves to `tbl`, `{1,2} <<#-2>>` is four statements with
tail `9`.

**And the freeze is the real damage.** In the deferred case `r =$ {tbl,0}f`
**never flattened at all** — `r=2` is `f`'s own `n`, computed at the definition
site. `stmt_count` answered `2` early, `$` took `2`, and `n` settled
**CONSTANT**. Once settled, nothing re-opens it, so the later recoordination had
nothing left to correct.

**That is the difference the three states make.** With `NotReady`, `n` stays
unsettled and resolves to `9` after recoordination. With `0`, `n` settles to `2`
permanently.

**Compare with the `constanic_is_brane_like` boolean.** There a don't-know reads
as "no" and produces a **loud NK** — visible. Here it reads as **zero** and
produces a **quietly frozen answer** — no alarm, no NK, nothing to notice.

**When does a FIR actually need `stmt_count`?** That is the question to settle,
and it is what makes the three-state answer necessary rather than merely tidy.
The two callers differ:

- A **search** asking "are you brane-like?" needs only to distinguish *yes* /
  *no* / *not yet* — it never needs the number.
- **Indexing and the Equivalence Law** need the exact count, and are entitled to
  it only once the shape is frozen.

## Findings — exercise-file defects (Atlas is fixing the file)

The human (Atlas) is repairing the exercise file directly; this FOOP does
not edit it. Recorded so the plan can verify the fixes landed.

### E1. The accumulator is updated as `sum` but read as `sum35`

The loop updates `sum` (`sum $= {cond1, sum+lv, sum, 'ite}`) but returns
`sum35` (`exit=sum35`; `answer $= {cond2, <continue>, sum35, 'ite}`); the
comment says "sum35 is the accumulant". As committed, `sum35` resolves to
nothing and the answer can never be 233168. Direction: rename the
accumulator to `sum35` (per the comment).

### E2. Line 13 is missing its `#` — the three `<<-N>>` must be positional searches `<<#-N>>`

`'ite = ({<<-2>>, <<-1>>} _ite)~cond=(<<-3>>)&#1`. For the if-then-else
selector to work, `<<-2>>`, `<<-1>>`, `<<-3>>` must be `<<#-2>>`,
`<<#-1>>`, `<<#-3>>` — as written they are integer literals (which also
lose their sign per D3). Design reading of the intended mechanism (a
*reading*, verified empirically in Phase 3, not yet traced on a live FVM):
in a use `{C, T, F, 'ite}`, the recoordinated RHS splices into the usage
context; `{<<#-2>>, <<#-1>>}` picks up `T` and `F` relative to `'ite`'s own
position, then merges with the `INTERN_ite` table
`[cond='True, <<#-3>>, cond='False, <<#-2>>]`; `~cond=(<<#-3>>)` matches
the `cond` row whose value equals the caller's condition (the pattern
`<<#-3>>` resolving to `C` relative to the `'ite` statement); and `&#1`
takes the result expression sitting beside the matched row, which
re-resolves to `T` or `F`.

### E3. Leading-underscore names → `INTERN_` prefix (the D1 decision)

`_ite` → `INTERN_ite`, `_cmod` → `INTERN_cmod`, `_eq` → `INTERN_eq`,
`_numerator` → `INTERN_numerator`, `_divisor` → `INTERN_divisor`.
(Interior underscores are fine — `is_id_sep` — so the prefixed forms lex
cleanly; the `_` maps to SEP U+02CD inside the name, identically at every
use site, so references match.)

### E4. Application sites → backtick (tail-concatenator) form, per FOOP-65

Atlas direction (2026-08-07): the exercise's application sites are to be
written with the tail concatenator — method name to the front, FOOP-65 §1:
`{lv,3,0}'cmod` is written `'cmod`{lv,3,0}`. The `'` null-characterization
is unaffected. **FOOP-55 therefore depends on FOOP-65 merging first** (the
plan's Phase 0 gates on it).

Full rewrite mapping (applying E1's `sum35`, E2's `#`, E5's explicit
`(X)$`, and the E3 `INTERN_` renames simultaneously):

| As committed | Rewritten |
|---|---|
| `divides_3 $= {lv,3,0}'cmod;` | `divides_3 = ('cmod`{lv,3,0})$;` |
| `divides_5 $= {lv,5,0}'cmod;` | `divides_5 = ('cmod`{lv,5,0})$;` |
| `cond1 $=  {divides_3, divides_5, 'or};` | `cond1 = ('or`{divides_3, divides_5})$;` |
| `sum $= {cond1, sum+lv, sum, 'ite}` | `sum35 = ('ite`{cond1, sum35+lv, sum35})$;` |
| `cond2 = {lv, 1000, 'lt}$;` | `cond2 = ('lt`{lv, 1000})$;` |
| `continue = <self loop>;` | `continue = <loop`self>;` |
| `answer $= {cond2, <continue>, sum35, 'ite}` | `answer = ('ite`{cond2, <continue>, sum35})$;` |
| `answer $= {loop} loop` | `answer = (loop`{loop})$;` |
| `'ite = ({<<-2>>, <<-1>>} _ite)~cond=(<<-3>>)&#1` | `'ite = (INTERN_ite`{<<#-2>>, <<#-1>>})~cond=(<<#-3>>)&#1` |
| `_cmod = {_eq, {_numerator,_divisor, 'mod}$, 'eq};` | `INTERN_cmod = {INTERN_eq, ('mod`{INTERN_numerator, INTERN_divisor})$, 'eq};` |
| `'cmod = {_numerator=#-3,_divisor=#-3,_eq=#-3, _cmod$}` | `'cmod = {INTERN_numerator=#-3, INTERN_divisor=#-3, INTERN_eq=#-3, INTERN_cmod$}` |

Application-result extraction is written with parentheses + `$` —
`('cmod`{lv,3,0})$` — because the backtick is the WEAKEST operator
(FOOP-65 §2): a bare trailing `$` would bind inside the right operand
(`fn`X$` ≡ `(X$) fn`, Atlas's example), not over the application. The
`$=` form is not used at all (E5).

### E5. The six `$=` lines cannot parse as written — rewrite as explicit `(X)$`

`$=` is not accepted by the parser at all (D6): every `$=` line in the
exercise is a parse error once the leading-underscore issue is out of the
way. The E4 mapping rewrites each as `name = (application)$` — explicit,
sugar-free, and immune to the `=$` semantics question (D6).

## Specification

### §1. `'mod` — integer modulo as a system operator

**Surface form.** No new syntax. `'mod` is a null-characterized name used
postfix, exactly like the FOOP-33 comparisons: `{a, b, 'mod}$` — the modulo
of the two elements immediately before it in the containing brane. It
resolves by ordinary ancestral search into `system.foo`; there is no
parse-time or name-based special-casing at the use site (FOOP-33 §5.0).

**Declaration.** `system.foo` gains one line beside the comparisons:

```foolish
'mod   = ⬤    !! modulo: <<#-2>> % <<#-1>>
```

**Installation.** The compile-time `BodyOverride` hook
(`compiler.rs:468-505`) replaces the placeholder `⬤` body with the modulo
FIR when `system.foo` is composed — the same mechanism as the comparisons
(`system_foo.rs:384-391`). The hook's name table generalizes from
`ComparisonOp::ALL` to include `'mod` (renaming the hook is an
implementation detail). The hook still runs ONLY over `system.foo`'s own
top-level statements; a user's own `'mod` in program source is untouched.

**FIR.** One new FIR kind in `system_foo.rs`, mirroring `ComparisonFir`
(`system_foo.rs:158-341`) point for point with one difference — the result
is an integer, not a boolean:

- Two SFF-marked operand lookups, compiled from the Foolish sources
  `<<#-2>>` and `<<#-1>>` through `build_operand` /
  `push_foolish_child_sff_marked` (the panic guard is retained: an operand
  left runnable inside `system.foo` is an interpreter defect).
- `fir_op_step`: the same two-phase shape as `ComparisonFir` —
  Prembrionic/Embryonic → Braning (enqueue operands); Braning → `combine`.
- `combine` rules, in order:
  1. **Any operand unevaluated-here** (ECONSTANIC inside its SFF wrapper,
     per `operand_is_unevaluated_here`, `system_foo.rs:351-358`) → settle
     **ECONSTANIC**. *Never NK here* — NK is terminal and would poison the
     `'mod` definition inside `system.foo`, so no search could ever hand it
     out to be recoordinated (FOOP-33 §5.1 point 1; this is load-bearing).
  2. Read each operand through its wrapper (`.value()` → `as_i64()`);
     **both integers → `a % b`**, stored as the result; settle CONSTANT.
  3. **Divisor zero → NK**, reason `"division by zero"` — identical to the
     existing `OperatorFir` `"%"` arm (`fir_kinds.rs:691-711`).
  4. **Either operand evaluated-here but not an integer → NK**, reason
     `"modulo: non-integer operand"` (same shape as ComparisonFir's
     `"comparison: non-integer operand"`).
- **Result rendering:** the result is an integer value; the statement
  renders as `'mod` the same way a comparison statement renders (the
  existing `searchable_name`/display-name path, `system_foo.rs:323`).
- **Constanic clone:** a new `FirKind` arm in `constanic_clone_at` performs
  the recoordination, exactly as `FirKind::Comparison` does (FOOP-33 §5.1:
  "a `FirKind::Comparison` arm in `constanic_clone_at` is what actually
  performs the recoordination").
- **Shape choice:** a standalone `ModuloFir` or a `SystemArithFir`
  parameterized by an `ArithOp` enum whose first (and, for this FOOP, only)
  variant is `Mod`. The enum shape is RECOMMENDED — `rust_instructions.md`
  §"finite word-domains → enum", and it leaves the door open for a future
  `'div` etc. without a second copy of the machinery. Either shape
  satisfies this specification.

**Negative operands.** Rust `%` (truncating remainder, sign of the
dividend). The exercise uses only positive operands; the truncating
behavior is pinned by tests, and Euclidean modulo — if ever wanted — is a
future FOOP. (Open question, below.)

**NYES.** No new states. AGENTS.md mandates a `*_nyes_transitions` test for
every new FIR kind: `modulo_nyes_transitions` pins all THREE terminal
outcomes, following `comparison_nyes_transitions`
(`system_foo.rs:627-669`): ECONSTANIC inside `system.foo` (no neighbours),
CONSTANT with two integer neighbours, NK with a non-integer neighbour.

### §2. `'or` — boolean OR (FVM-computed, per FOOP-73 fallback)

**DESIGN CHANGE (2026-08-09):** The pure-Foolish truth-table approach
described below was implemented and **failed** — the value search `T~A=A`
inside system.foo cannot resolve when `A` is ECONSTANIC (no neighbours in
system.foo), so the root brane never settles. The FVM-computed fallback
(FOOP-73 §Fallback) was taken instead: `'or = ⬤` with a dedicated `OrFir`
FIR kind that checks operand identity against system.foo's `'True`/`'False`
creations via `Rc::ptr_eq`. This reintroduces a privileged FVM layer for
`'or`, which is the trade-off FOOP-73 anticipated.

The pure-Foolish design is preserved below for reference.

---

**~~No new FIR kind. No privileged FVM layer.~~** ~~`'or` is defined in
`system.foo` as an ordinary Foolish brane holding a truth table, applied by
ordinary search — FOOP-73 §"Preferred design". FIR impact: NONE (that is
the point).~~

`system.foo` gains:

```foolish
'or = {
    A = <<#-2>>;
    B = <<#-2>>;
    T = {
        A='True,  B='True,  'True;
        A='True,  B='False, 'True;
        A='False, B='True,  'True;
        A='False, B='False, 'False;
    };
    r = T~A=A &~B=B &#1;
}
```

**Argument binding — why both are `#-2`** (FOOP-73 §"Why both are #-2"):
`#-N` seeks backward from the seeking statement's OWN position, and each
binding statement occupies a position, so the constant `-2` walks
successive arguments. In the exercise's usage
`{divides_3, divides_5, 'or}`, `'or`'s body splices after the args:
`[divides_3(0), divides_5(1), A(2), B(3), T(4), r(5)]` — `A` at 2 →
`2-2=0` → `divides_3` ✓; `B` at 3 → `3-2=1` → `divides_5` ✓. (Using `#-1`
for `B` would land on `A` — the bug FOOP-73 warns about.)

**The lookup, traced on all four rows.** `T` is FLAT — twelve statements,
rows grouped by `A` value, in order. `r = T~A=A &~B=B &#1`: `~A=A` searches
`T` forward for the first statement named `A` whose value equals the bound
`A` (creation-vs-creation, referential — FOOP-33 §2 rule 3); `&~B=B`
continues forward *from that position* for a statement named `B` whose
value equals the bound `B`; `&#1` takes the next statement — the row's
result.

| A | B | `~A=A` lands | `&~B=B` lands | `&#1` yields |
|---|---|--------------|---------------|--------------|
| 'True | 'True | stmt 0 (A='True) | stmt 1 (B='True) | stmt 2 = 'True ✓ |
| 'True | 'False | stmt 0 | stmt 4 (B='False; stmt 3 is named A, skipped) | stmt 5 = 'True ✓ |
| 'False | 'True | stmt 6 (first A='False) | stmt 7 (B='True) | stmt 8 = 'True ✓ |
| 'False | 'False | stmt 6 | stmt 10 (stmt 9 named A, skipped) | stmt 11 = 'False ✓ |

The row grouping is load-bearing: all rows sharing an `A` value must sit
contiguously after the first `A=` statement of that value, or the forward
scan can land in the wrong row.

**Non-boolean arguments** → the lookup finds no row → anchored miss →
**NK** (FOOP-23 settlement: an anchored search that finds nothing settles
NK). This matches FOOP-73's test plan ("`{T,3}and`→NK").

**Preconditions the design relies on** (all verified empirically in plan
Phase 2 before `'or` is declared done):

1. Concatenation splices a resolved brane value's statements FLAT into the
   merged brane (FOOP-73 precondition 1; the exercise's own `'cmod`
   requires exactly this — its `#-3` bindings only reach `lv`/`3`/`0`
   through a flat splice).
2. Value search accepts a search (an identifier) as its value pattern and
   compares settled values (`SearchPredicate::Value { pattern: FirRef }`,
   `fir_kinds.rs:2063-2064`; FOOP-23 "expression patterns").
3. Creation equality is referential (FOOP-33 §2 rule 3) — rows match the
   bound `'True`/`'False` by identity.
4. `&#1` from the found `B` statement lands on the row's result statement.

**Fallback (only if the table search proves insufficient in practice):**
FOOP-73 §Fallback — `'or` becomes FVM-computed: a dedicated FIR kind
installed via the `BodyOverride` hook (the `'mod` shape, but comparing
operands by `Rc::ptr_eq` against the `system.foo` booleans and returning
the `'True`/`'False` creation). This reintroduces a privileged layer, so
the plan carries a STOP-and-consult-Atlas checkbox before taking it, and
FOOP-73 must be updated if it is taken.

**Naming note.** FOOP-73's lean draft sketched uppercase `AND`/`OR` names;
this FOOP standardizes on the null-characterized lowercase form `'or`,
consistent with `'True`/`'False`/`'lt`/`'eq` in `system.foo`. FOOP-73
remains the governing Draft for `'and`/`'not`/`'nor`/`'xor` and for the
`b'`-characterization typing that arrives with FOOP-63.

### §3. What already exists (verified working — NO work in this FOOP)

Bisect evidence on `jia` @ `62706518`:

| Construct the exercise uses | Evidence |
|---|---|
| `!!!` block and `!!` line comments | parse+run OK |
| comma as statement separator | `{_ite = {cond=1, 2;}}` renders both statements |
| `<<#-N>>` SFF-marked index search | `{x = <<#-3>>;}` → `#(offset=-3, UNANCHORED, ECONSTANIC)` |
| `<ident …>` SF marker | `{self=1; x = <self>;}` parses and evaluates |
| `$` extraction via explicit parentheses + `$` — the `=$`/`$=` sugars are NOT used (D6/E5) | postfix `$` parsed at `parser.rs:684,747`; `OperatorFir` `"$"` arm `fir_kinds.rs:713`; the exact `(X)$` shape verified live in Phase 3 |
| `+ - * /` incl. unary minus at top level | `{x = -2;}` → `-2`; `{x = 5 - 2;}` → `3` |
| `'True`/`'False`, `'lt`/`'gt`/`'le`/`'ge`/`'eq` | FOOP-33 §5.1 as built; `system/system.foo` |
| value search `~cond=(pattern)` | `{y={cond=3;}\nx = y~cond=(3);}` finds the row |
| concatenation / juxtaposition | `{1, 2} name` parses as `Concatenation` |

### §4. Integration risks (verified/fixed in plan Phase 3)

0. **Dependency: FOOP-65 (tail concatenator) must be merged first.** The
   exercise rewrite (E4) is written in backtick application form; the
   plan's Phase 0 gates on FOOP-65 being on `jia`. If FOOP-65 slips, this
   FOOP waits — do NOT rewrite the exercise in juxtaposition form as a
   stopgap (it would only re-churn the einmo inputs later).
1. **MAX_DEPTH = 100** (`fir_trait.rs:395`) vs ~1000 recursion depth.
   `step_inner` returns `NoProgress` beyond depth 100 (`fir_trait.rs:453-486`).
   The exercise's `continue = <self loop>` recursion nests on the order of
   1000 deep (lv 1→999). Whether the recursion nests *step depth*
   linearly — and what that means for the depth guard — is the single
   largest integration risk of this FOOP. Phase 3 measures it on the live
   FVM first; if the guard bites, the remedy (raising the constant with
   justification, iterative stepping for this pattern, or evidence that
   recoordination keeps depth bounded) is decided there, with Atlas
   consulted if the choice is semantic.
2. **Evaluator iteration cap** — `Iteration exceeded 9999`
   (`evaluator.rs:168`). ~1000 iterations × work-per-iteration may approach
   it; measured alongside risk 1.
3. **`lv = lv+1`** — name reuse with SSA semantics (README §Renaming) where
   the new `lv` reads the parent-context `lv`; verified live in Phase 3.
4. **`{loop} loop`** — the entry-point shape that seeds the recursion.

### §5. The SFF mark defers one coordination per nesting level

**The defect.** The exercise computes *nothing* — every statement settles
PREMBRIONIC at a 40000-step budget. The proximate cause is that
`answer = {cond2, sum35, <loop>, 'ite}$` drives its discarded `<loop>` branch to
constanic before the condition can stop it. The root cause is narrower and
lives in one function: **a constanic clone strips every SF/SFF mark it meets, in
one pass.**

`constanic_clone_at` (`fir_kinds.rs:160-193`) handles an SF/SFF node by taking
its content and cloning *that*, discarding the wrapper (lines 183-191). The call
is recursive, so when the content is itself a marked node the inner mark is
stripped in the same pass. Nesting is therefore a no-op today, which is
demonstrable: `A = {v = 1 + <<#-1>>}` and `A = {v = 1 + << <<#-1>> >>}` both
yield `B=42` under `B = ({X=41} A)$`.

Premature stripping is not merely eager — it searches **in the wrong place**,
and the damage is permanent. Given

```foolish
{
	blah = 7;
	A = <{abcd = 1; deep = << <<#-2>> >>}>;
	B = A;
	C = B;
}
```

`deep` resolves at `B`, where `#-2` finds nothing, and settles **NK** — a
terminal state. By the time the value reaches `C`, where the search would have
succeeded, it is irrecoverably dead. A doubly-marked term asserts "not here, not
yet"; the current code overrules it.

**The change.** One layer of deferral is discharged per coordination:

> A constanic clone may strip **at most one** SF/SFF mark **per root-to-leaf
> path**. Descending into a child inherits the parent's remaining budget;
> spending it in one child does **not** affect that child's siblings. The first
> mark on a path is stripped; any further mark *on that same path* is
> **retained**.

**Corrected 2026-08-11, during implementation.** An earlier draft said the
budget belonged to the whole clone *tree*. That is wrong, and twelve existing
tests proved it within minutes: the comparison and modulo operators take **two**
SFF operands — `<<#-2>>` and `<<#-1>>` — as **siblings**, and a per-tree budget
let only the first of them resolve (`modulo_basic_semantics` went from `Some(1)`
to `None`, taking every comparison operator with it).

The distinction is between marks stacked **vertically** and marks side by side
**horizontally**:

- **Nesting (vertical, same path)** — `<< <<X>> >>`. Depth is a **deferral
  count**: one mark comes off per coordination.
- **Siblings (horizontal, different paths)** — `{<<#-2>>, <<#-1>>}`. Each is an
  independent one-level mark and each resolves on the same coordination,
  exactly as before.

`StripBudget` is therefore `Copy` and passed **by value**, not by reference —
the type system carries the rule.

Consequences, in order of importance:

1. **`<<X>>` is unchanged.** One mark, one coordination, resolves exactly as
   today. The 16 einmo inputs using single marks must produce byte-identical
   OUTPUT; see §Test Plan.
2. **`<< <<X>> >>` sits out one coordination**, then behaves like `<<X>>` at the
   next. Deferral is a count, and it is written in the term rather than inferred
   by the evaluator.
3. **The budget is per-path.** In `A = <{abcd, << <<blah>> >>}>` the path
   root→SF→brane→SFF carries two marks, and only one is spent — so the rule is
   independent of *which kind* of mark is met first. But two marks in *sibling*
   subtrees are on different paths and each gets its own strip, which is what
   keeps `'mod`'s two operands working.
4. **Each use site counts separately.** `B = A` and `C = A` produce independent
   clones, so `C` does not observe `B`'s decrement. Deferral is per-coordination
   -path, which is what makes macro-style definitions compose.

**Retained marks are shared, not copied.** When the budget is spent, the clone
returns `Rc::clone(fir_ref)` — a reference to the original mark — rather than a
deep copy. This is sound because an unstripped SF/SFF node is **immutable and
position-independent**: it has not searched, so it holds no resolved reference
to any brane, and there is no per-site state in it to corrupt. The per-site
state lives in the clone operation's budget flag, not in the node. (Contrast a
resolved search, whose `FoolRefFir` carries a position in a specific home brane
and must never be shared across contexts.) `constanic_clone_at:194-199` already
shares constanic non-brane nodes this way; this extends the same treatment to a
retained mark.

**Nested marks must be written with a separator** — `<< <<A>> >>` or
`<<(<<A>>)>>`, never `<<<<A>>>>`. All three forms already lex identically (the
parenthesized form leaves no grouping node), so this is a **style rule**, not a
grammar change: four adjacent angle brackets have no visual boundary and must
not appear in new code.

#### Two ways marks nest — and why one rule covers both

Marks compose in two structurally different ways. They *feel* like they ought to
need different rules; they do not. Both are covered by "at most one strip per
clone tree", applied at the granularity each case presents.

**Priority: case 1 must work; case 2 must not regress.** Euler 1 depends
entirely on case 1 — `'ite`'s doubly-marked branches surviving table
construction. Case 2's single-marked chains already work and need only stay
byte-identical. Whether case 2's *deferral* behavior (points 2 and 3 below) is
the RIGHT or OPTIMAL semantics is a question to revisit once there is a working
Euler 1 to reason from; it is explicitly **not** a gate on this FOOP. An
implementer who finds case 2 behaving oddly should record it and move on, not
stop to perfect it.

##### Case 1 — syntactic nesting: marks on ONE term

```foolish
{a = << 1 + << 2 + <<x>> + c >> + d >>}
```

This is a single expression, parsed into a single tree, with marks lexically
inside one another. **One coordination clones one tree and spends one strip**,
so depth is a property of the term: the innermost `<<x>>` is three boundaries
deep and sits out three coordinations before it searches.

Today all three come off at once. Verified:

```foolish
{
	M = {v = << 1 + << 2 + <<#-1>> >> >>};
	r = ({x=10} M)$;          !! → r=13   (1 + 2 + 10)
}
```

Under §5 the outer mark is stripped by the coordination that builds `M`'s
result, and the inner two ride through — resolving one boundary at a time.
**This is the case the exercise depends on**: `'ite`'s branches are doubly
marked so they survive table construction and resolve only at the use site.

##### Case 2 — search chaining: marks on SEPARATE terms

```foolish
{A = <<a>>; B = <<A>>; C = <<B>>; r = C}
```

Nothing is lexically nested here. `B`'s mark wraps a *search for `A`* — not
`A`'s mark. These are four statements, each with its own body and its own future
clone.

The trace, and the load-bearing detail:

1. `r = C` coordinates `C`. That clone spends its budget on `C`'s mark; what it
   copies is the search for `B`, now searchable.
2. **That clone operation exits — and the budget dies with it.**
3. The search for `B` runs and finds `B`'s statement, whose body is `<<A>>`.
   Coordinating *that* result is a **new** `constanic_clone` call with a
   **fresh** budget, which strips `B`'s mark.
4. The same again for `A`'s mark, after which the search for `a` runs and misses.

So the chain resolves **one hop per link**, and `r` settles "couldn't find `a`"
— ECONSTANIC, since the searches are unanchored.

**This reading is provisional.** What has actually been verified is narrow:
chains of length 1, 2, and 3 terminate on the current tree, and the 3-link chain
produces the nested result the trace predicts. Those chains all resolve to a
*miss*, so the more interesting path — a chain that finds something and carries
a value back through each hop — is **untested**. Whether "one fresh budget per
resolution hop" is the actual mechanism, or merely consistent with these
shapes, is not established; nor is the behavior when a link is a brane rather
than a bare search, or when marks sit on both sides of a link. Treat the trace
below as a hypothesis with three supporting observations, not as specified
semantics.

```
r = ?(result = ?(result = ?(result = ?(pattern='^a$', ECONSTANIC),
                            pattern='^A$', WOCONSTANIC),
                 pattern='^B$', WOCONSTANIC),
      pattern='^C$', WOCONSTANIC)
```

##### Why the same rule serves both

The budget belongs to a **clone operation**, and the two cases simply present
different numbers of them:

| | Clone operations | Budgets | Depth accumulates |
|---|---|---|---|
| **Syntactic** | one (walking one tree) | one, contested by the marks in it | *within* a term — nesting is a **count** |
| **Search-chained** | one per resolution hop | one each, fresh every time | *across* terms — chaining is a **pipeline** |

Neither needs the other's machinery, and no second mechanism is introduced.
Syntactic nesting lets a macro author say "ride through N boundaries"; search
chaining lets each stage of an indirection defer exactly one hop.

##### What case 2 WILL do after §5 — the specified behavior

Stating this plainly so there is something to test against, and so a divergence
is a detectable defect rather than an open question:

1. **Single-marked chains are unchanged.** `{A=<<a>>; B=<<A>>; C=<<B>>; r=C}`
   behaves after §5 exactly as it does today — one hop per link, settling
   ECONSTANIC. Every statement carries one mark, so no clone tree ever meets a
   second mark and the budget is never contested. **Its einmo baseline must be
   byte-identical, step counts included.** This is the control that proves §5
   changed only what it claims to.

2. **A doubly-marked link defers one extra hop.** In
   `{A=<<a>>; B=<< <<A>> >>; C=<<B>>; r=C}`, coordinating `B`'s result spends
   its budget on the outer mark and hands on a still-marked `<<A>>`; the search
   for `A` therefore does not fire on that hop, and fires on the next
   coordination instead. The chain grows one link longer in effect without
   growing one statement longer in source.

3. **Mixed cases compose by the same rule.** A link whose body is syntactically
   nested (`B = << 1 + <<A>> >>`) spends one strip on the outer mark of *that
   term*, exactly as case 1 describes, because it is one term inside one clone.

**Uncertainty, and what to do about it.** Point 1 is verified. Points 2 and 3
follow from the rule but are **not demonstrated** — the verified chains all
resolve to a miss, so no value has been carried back through a multi-hop chain,
and the interaction with recursion in particular is unexplored (see
§Open Questions). The plan tests all three as einmo cases. **If the
implementation contradicts points 2 or 3, adjust this section to match the
implementation and record why** — these are the FOOP's predictions, not
constraints the code must be bent to satisfy. Point 1 is different: a
divergence there is a regression in the strip budget and must be fixed in the
code.

#### Why this is what the exercise needs

`'ite` is defined as a table lookup whose branches are SFF-marked:

```foolish
INTERNAL_ite = {
	cond='True, << <<#-3>> >>;
	cond='False, << <<#-2>> >>;
}
'ite = ({<<#-2>>, <<#-1>>} INTERNAL_ite)~cond=(<<#-3>>)&#1
```

The extra mark makes each branch survive the coordination that builds the
lookup table. Only the row the value search selects is coordinated again — at
the use site — and only then does its inner mark come off and the branch
resolve. **The unselected branch is never coordinated a second time, so it never
searches, so `<loop>` never recurses.**

No laziness rule is added to the FVM, no evaluation order changes, no FIR gains
a new state, and no search learns a new trick. The deferral is carried by the
term.

#### Euler 1 under the upgraded SFF mark

This is the program this FOOP is expected to run. The only change from the
current input is the doubled marks on the two `INTERNAL_ite` branches:

```foolish
!! Project Euler 1: Multiples of 3 or 5
!! Find the sum of all the multiples of 3 or 5 below 1000.
!! Expected answer: 233168

{
	!! If-Then-Else. The branches are DOUBLY marked: they must survive the
	!! coordination that builds the lookup table, and resolve only at the
	!! use site — so the unselected branch is never evaluated.
	INTERNAL_ite = {
		cond='True, << <<#-3>> >>;
		cond='False, << <<#-2>> >>;
	}
	'ite = ({<<#-2>>, <<#-1>>} INTERNAL_ite)~cond=(<<#-3>>)&#1

	!! Congruent modulo: {a,b,c}'cmod computes the boolean a%b==c
	INTERNAL_cmod = {INTERNAL_eq, {INTERNAL_numerator, INTERNAL_divisor, 'mod}$, 'eq};
	'cmod = {INTERNAL_numerator=#-3, INTERNAL_divisor=#-3, INTERNAL_eq=#-3, INTERNAL_cmod$}

	loop = {
		self=<<#-1>>
		lv = lv+1;
		divides_3 = ({lv,3,0}'cmod)$;
		divides_5 = ({lv,5,0}'cmod)$;
		cond1 = ({divides_3, divides_5, 'or})$;
		sum35 = ({cond1, sum35+lv, sum35, 'ite})$
		cond2 = ({lv, 1000, 'lt})$;
		continue = <self loop>;
		exit = sum35
		answer = ({cond2, <continue>, sum35, 'ite})$
	}
	lv = 1; sum35 = 0;
	answer = ({loop} loop)$
}
```

The `@einmo set iteration depth to 40000` directive is **removed**: if the
exercise still needs it, this section is not finished.

#### Implementation

One site, three parts (`foolish-ubca/src/fir_kinds.rs:160-199`):

1. Thread a **strip-budget** flag through `constanic_clone_at`, alongside the
   existing `descendent_of_sfm_and_foolishly_ignorant` parameter (line 164) —
   the same threading, already present.
2. At an SF/SFF node with the budget **available**: strip as today (lines
   183-191) and mark the budget spent for the remainder of the tree.
3. At an SF/SFF node with the budget **spent**: `return Rc::clone(fir_ref)`.
   Note this needs its own arm — the existing share at lines 194-199 keys on
   `Constant | Independent`, and a marked node is WOCONSTANIC, so it does not
   fall through to that path.

Confirm the new branch cannot reach the
`eprintln!("ALARM: SF/SFF node has no children")` at line 192.

### §6. The brane view, and the unanchored forward search `~name`

`'ite` needs to select a row from a lookup table without carrying a position
across coordination boundaries. The contexted index `&#1` cannot do it: its
carried position must survive the definition-site coordination, and §5's mark
counting brackets the answer without landing it (one mark and the pattern dies
NK; three and nothing ever resolves). §6 removes the need for a carried position
altogether.

#### The gap

Foolish has an **anchored** forward search — `tbl~a` scans `tbl` from the front
and finds the *first* `a`. It has an **unanchored backward** search — `?a`
scans my own brane rear-to-front from just before me, finding the *nearest
preceding* `a`. It has **no unanchored forward** form: bare `~a` is a parse
error today ("expected primary expression, found Tilde").

```foolish
tbl = {a = 1; b = 2; a = 99;}
tbl~a        !! → 1    anchored forward: FIRST match
tbl?a        !! → 99   anchored backward: LAST match
?a           !! → the nearest PRECEDING match in my own brane
~a           !! parse error — the hole this section fills
```

`ast.rs` records the reason: *"There is no unanchored forward (`~pattern`) form:
Foolish cannot look forward in its own brane (FOOP-23 §Specification A.1)."*
That rationale is about looking **ahead**, into statements that have not
settled. §6 does not do that.

#### The brane view

A **brane view** is a brane-like object exposing a **contiguous range** of
another brane's statements — for this feature, `[0, my_index-1]`, everything
before the searching statement — while keeping the **same parent** as the brane
it views.

Three properties make it the right primitive:

1. **Same parent.** A statement reached through the view resolves its own
   searches against the real enclosing context. Being viewed distorts nothing
   about position, ancestry, or line number.
2. **Contiguous, and starting at 0.** Index `i` in the view *is* index `i` in
   the underlying brane, so `#N`, `&#`, and line numbers stay honest with no
   translation layer.
3. **It can be constanic before the whole brane is.** This is the load-bearing
   property. A view over `[0, k]` is constanic as soon as those `k+1`
   statements are — and FIFO draining guarantees exactly that, since statement
   `k+1` cannot step until its predecessors have settled. So the view is
   *already* constanic at the moment a search anchors on it, with no waiting on
   statements after me that I am not permitted to see.

#### A view is never stepped, and its NYES is computed live

Two rules, and they are a matched pair — together they are what *makes*
property 3 true rather than something that must be maintained:

**1. A view is never enqueued.** It has no evaluation of its own. Its
statements belong to the source brane and are stepped there, through the source
brane's own task queue. A view must therefore never be pushed as a task —
implementations must check for this explicitly, because enqueueing one would
step the same statements twice through two different owners.

**2. `get_nyes()` is an active scan of its direct children, not stored state.**
A view holds no NYES of its own to cache, and therefore none that can go stale
as the underlying statements settle. Asked, it computes; the answer is always
current by construction.

The classification uses the **existing** rule — `_decide_nyes_due_to_children`
(`fir_kinds.rs`), the same function an ordinary brane uses to classify itself
from its members. Reusing it matters: a second, view-specific classification
would be a place for the two to drift, and a view's whole purpose is to report
faithfully about statements it does not own. (Note `Nyes` is `PartialEq, Eq`
but **not** `Ord`, so "lowest" is not a `.min()` — the ordering is the rule that
function encodes, not the enum's declaration order.)

A view consequently has **no `set_nyes`**. Writing a NYES to a view would be
meaningless — it owns no evaluation to record. That is an invariant worth
enforcing in the type rather than by convention.

Together these give property 3 for free: a `[0, my_index-1]` view reports
constanic exactly when its window is constanic, and FIFO draining guarantees
that window has settled before the searching statement steps. Nobody has to
arrange it.

Property 3 is why a view beats clipping a navigator's range. A clipped scan
still asks "has the anchor brane settled?" — and it has not. The view asks "has
*my window* settled?" — and it always has. That is the same move §5 makes:
narrow the dependency to what is actually needed.

#### `~name` unanchored ≡ `view~name` anchored

The unanchored forward search is then not a new search semantics at all:

> **`~name`** builds a brane view of `[0, my_index-1]` over the home brane and
> performs the **ordinary anchored forward search** on it.

`~name` finds the **earliest** match in the candidate window; `?name` finds the
**nearest preceding** one. Same window, opposite direction — restoring the
symmetry the anchored forms already have.

**The candidate window stops before the searching statement.** The searching
statement itself and everything after it are not candidates. `{a=1; r = ~a;
a=99;}` gives `r=1`: the later `a=99` is invisible, and there is no self-match.

Because the window contains only settled statements, FOOP-23 §A.1's concern does
not arise — nothing looks forward into unsettled territory. A miss remains
**ECONSTANIC**, not NK, exactly as the unanchored backward form specifies: the
search may still gain a value when the brane is recoordinated elsewhere.

#### Why `'ite` wants this

With `~`, the branch table is selected by an **anchored** search over a view —
no carried position, therefore nothing that must survive a coordination, and
therefore no `&#1` and no mark-depth puzzle. The table's rows are matched
directly where they sit.

#### Implementation

- **Parser** (`foolish-parser/src/parser.rs`): accept bare `Token::Tilde` in
  primary position, mirroring the bare `Token::Question` arm, emitting
  `RegexpSearch { anchor: None, operator: RegexpForward, .. }`. Update the
  `RegexpSearch` doc comment in `ast.rs`, which currently states the unanchored
  forward form does not exist.
- **Brane view** (`foolish-ubca`): a brane-like exposing `[start, end]` of a
  source brane with the source's parent. Read-only — a lens for searching, never
  a mutation path.
- **Search** (`fir_kinds.rs`): `ib_search_with_engine` and
  `ab_search_with_engine` both hardcode `BraneNavigator::new(&brane, false)`
  with an inline `set_range(0, idx-1)`. Both become a view anchored search
  honouring `self.forward`, so the range logic lives in one place instead of
  being re-derived per call site.

### §7. `ExtremumFir` — order-statistic selection over a concatenation

Euler 1 needs a maximum. Rather than a max operator and a min operator, §7
specifies **one** FIR parameterized by an **index into the ascending sort** of
the integers it is given, with `'max_int_val` and `'min_int_val` as the two
named aliases.

#### Sort ascending, then index — the same convention `#` already uses

The candidates are collected and sorted **ascending**. The parameter selects
from that sorted sequence, with negatives counting from the end:

| Index | Selects |
|-------|---------|
| `0` | smallest |
| `1` | second smallest |
| `-1` | **largest** |
| `-2` | second largest |

So `'min_int_val` is index `0` and `'max_int_val` is index `-1`.

This is deliberately the **same indexing rule as `#`** — 0-based from the
front, negatives from the end, as `src#-1` already means "the last statement".
A Foolisher learns one convention and applies it either to source order (`#`) or
to sorted order (§7), rather than two.

**No wraparound.** An index outside the candidate count settles **NK**: with two
candidates, index `2` and index `-3` are both out of range. This matches `#`,
where `src#3` on a three-member brane is already NK. Out-of-range is a fact
about this call, not something recoordination could change.

Only the two aliases are declared for now; the parameterization leaves
"third largest" and friends available without a new FIR kind when something
needs them.

#### The declaration is a BRANE, not a bare FIR

```foolish
'min_int_val = {ExtremumFir(0)}
'max_int_val = {ExtremumFir(-1)}
```

The body is a **brane containing** the FIR. That one detail carries the whole
design:

- **Juxtaposition splices it.** `{1,2,3}'max_int_val` concatenates to the
  flattened brane **`{1, 2, 3, ExtremumFir(-1)}`** — the FIR arrives as the last
  member, with the integers before it. Verified: `{1,2,3} marker` where
  `marker = {sentinel = 42;}` flattens to `{1; 2; 3; sentinel=42}`.
- **A bare FIR body would not concatenate at all.** `'or`'s body is an `OrFir`,
  not a brane, so `{1,2,3} 'or` does not flatten — the juxtaposition degenerates
  to a bare search. The `{…}` wrapper is what makes an operator concatenable.
- **Misuse becomes a type error, not a runtime check.** `1 + 'max_int_val`
  resolves the name to a **brane**, and `+` on a brane already fails. The
  meaningless case is unrepresentable rather than something to detect and
  diagnose — `rust_instructions.md` §1b rule 4 applied at the Foolish level. No
  bespoke error message to write or maintain.

#### The brane wrapper is the general form for computing postfix operators

The wrapper is not special to the extrema. **Every computing postfix operator
should be declared `'name = {NameFir}`** — the extrema are simply the first
written that way, because a variadic fold cannot be expressed any other way.

The two mechanisms differ in what the operator receives, and that difference is
the whole of the design:

| | SFF offsets (`'or`, `'mod`, comparisons today) | Brane wrapper (§7) |
|---|---|---|
| **Arity** | fixed — `<<#-2>>`, `<<#-1>>` | any, including zero |
| **How operands arrive** | recoordination resolves the offsets | concatenation splices the operator in beside them |
| **Self-location** | never needed; the operator does not know where it sits | required — `.parent()` and its own index |
| **`1 + 'name`** | type-checks (body is value-shaped), needs a runtime check | **type error** — the body is a brane |

**But an operator must be written knowing what it is getting into.** The
wrapper does not merely relocate the operands; it changes what the operator is
responsible for:

1. **Any number of preceding members, including none.** Fixed arity is no
   longer supplied by the mechanism. An operator that genuinely wants exactly
   two must *state* that and fail cleanly when it does not get it — the
   requirement moves from the machinery into the operator.
2. **Members of any kind.** Non-integers, branes, creations, other operators.
   Each operator decides whether a non-candidate is skipped (a fold) or fatal
   (a positional operand).
3. **Read through `stmt_count`/`stmt_at`, never `foolish_children`.** The
   container may be a ConcatBrane, whose children are `_ConcatHelper`s rather
   than statements (FOOP-13).
4. **The deferral rule still binds.** A pre-constanic member means
   ECONSTANIC-and-wait, never NK — NK is terminal and would poison the
   definition inside `system.foo`.

**Migrating the existing operators is therefore a real change to each, not a
mechanical rewrap**, and is **out of scope for this FOOP**: `'mod`, `'or` and
the comparisons work today, Euler 1 does not need them converted, and each
conversion is a behavioural change deserving its own tests. Recorded in the plan
as a follow-up.

#### Stepping

When `ExtremumFir` steps it asks its **parent** — the flattened brane — for the
entries preceding it, folds the integers, and produces **one** `ubc_children`
element which is the answer.

It reads the container through **`stmt_count`/`stmt_at`**, never
`foolish_children`. Inside a ConcatBrane (FOOP-13) the children are
`_ConcatHelper`s, not the statements; only the trait accessors perform the
offset arithmetic that makes FOOP-13's Equivalence Law hold. Using them means
the operator works identically whether its container is materialized or not.

| Situation | Outcome |
|-----------|---------|
| index in range | that order statistic, settled **INDEPENDENT** — it depends on nothing outside itself |
| a non-integer member | **skipped**, not fatal |
| any preceding member still pre-constanic | **ECONSTANIC**, and wait |
| no integers at all | **NK**, reason `"<name>: no integer operands"` |
| index outside the candidate count | **NK**, reason `"<name>: index N of M candidates"` — no wraparound |

**Why non-integers are skipped rather than fatal.** `'mod` and `'or` name their
operands positionally (`<<#-2>>`, `<<#-1>>`), so a non-integer *is* the named
operand and there is nothing else the expression could mean — NK is right. A
fold has made no such commitment: it asks "the largest integer here", and a
member that is not an integer simply is not a candidate.

**Why a pre-constanic member defers rather than folding a partial scan.** The
answer could still change. And it must settle ECONSTANIC rather than NK: NK is
terminal and would poison the definition inside `system.foo` — the exact failure
that forced `OrFir` (see plan Phase 2 and §D7).

**Why no integers is NK rather than ECONSTANIC.** The extremum of an empty set
is not a value, and unlike the deferral case there is nothing recoordination
could supply.

### §8. `@` (search position) and `#` over an expression — pattern matching

§5's mark counting and §6's `~` each removed part of `'ite`'s difficulty, but the
construct still needed a *position* to survive coordination. §8 removes the need
by turning the position into an **integer**, which survives coordination
trivially because it is just a number.

#### The two additions

**`@` — project a search result's position.** A search already carries its found
statement as `ubc_children[1]` (a `FoolRefFir` holding the original with its
parent chain and line number intact — FOOP-23's two-child invariant). Nothing in
the *language* reads that today except a following `&`-search, implicitly. `@`
exposes it:

```foolish
tbl = {zzz=0; key=77; other=5;}
tbl~key=(77)@        !! → 1, the found statement's index
```

**`#` accepts an expression, not only a literal.** Today `tbl#(1+1)`, `tbl#n`
and `tbl# (1+1)` are all parse errors ("expected integer, found LParen"), and
`tbl#1+1` parses as `(tbl#1)+1` — indexing by 1 then adding 1 to the *value*.
§8 makes the index an ordinary operand.

#### Dependencies and NYES

This is the part that makes both well-behaved, and it is the **ordinary**
dependency rule, not a special case:

| | Dependencies | NYES once they are constanic |
|---|--------------|------------------------------|
| `@` | the anchor | **WOCONSTANIC** |
| `#` | the anchor **and** the index number | **WOCONSTANIC** |

WOCONSTANIC is exactly "waiting on constanics — dependencies themselves
constanic" (AGENTS.md). A hit and a miss settle the same way, because the NYES
comes from the *dependencies*, not from the search outcome. Accepting an
expression is therefore not a new "evaluation phase" for `#` — it is a **second
dependency**.

#### The value: `-1` on a miss

`@` yields the found statement's index, or **`-1`** when the search found
nothing. The `-1` is not a sentinel smuggled into the value domain — it is what
makes a default branch fall out of arithmetic.

#### `@` is a continuation search, and searches answer for themselves

Two rules replace what would otherwise be `@` re-deriving facts about its anchor.

**1. `@` is a continuation search: its anchor must BE a search.** `x@` where `x`
is a brane, an integer, or anything else is not a meaningful question — there is
no search whose position could be projected. This is checked **while building
the FIR tree from the AST**, so a malformed `@` never reaches evaluation.

**A malformed continuation becomes a true NK** — not a compile error. Checking
at construction time means the FIR is *built as* an NK, not that the build
fails: an unanswerable question yields NK, exactly as the rest of Foolish does,
rather than refusing to run the program. `@` can then assume a well-formed
anchor, because a malformed one never becomes an `@` at all.

**2. Every search answers `candidates_exhausted()` about itself.**

> **`candidates_exhausted()`** — the scan ran to completion and no candidate
> matched.

It reports **one observable fact**, not a compound claim. That is deliberate:
an earlier draft called this `anchor_not_nk_and_still_not_found()`, which
bundled a fact about the anchor with a fact about the outcome, so every caller
inherited a conjunction it might not want.

The NK distinction then **falls out** rather than being encoded:

| Situation | Scan | `candidates_exhausted()` |
|-----------|------|--------------------------|
| anchor was a real brane, nothing matched | ran over every candidate | **true** |
| anchor was NK | never ran — there were no candidates | **false** |

**It does not cascade.** It reflects *this* search's status only. A descendant
wanting an ancestor's answer asks that ancestor — `.parent`, `.parent.parent`
— rather than having a hidden traversal built in. This is the shallow-reference
rule again: each node answers for itself, and anyone wanting a different node's
answer addresses that node.

Deliberately **universal on every search**, not a hook for `@`: the question is
one any consumer might ask, and answering it belongs to the search
(`rust_instructions.md` rule zero — a function reporting on an object belongs to
that object). `@` special-cases nothing.

| Search | `@` |
|--------|-----|
| found | the found statement's index |
| `candidates_exhausted()` | **`-1`** — falls through to the default row |
| constanic, but its anchor was NK | **NK** — "where in nothing?" has no answer |
| pre-constanic | not settled yet; `@` waits |

**Why a predicate rather than a reason string.** An earlier draft had the search
record `SEARCH-MISS` / `SEARCH-ANCHOR-NK` via `set_alarm_reason`, and `@` match
on the text. That is stringly-typed — a typo misclassifies silently — and it
makes the consumer parse what the producer already knows. A predicate is
computed from state the search holds, cannot be mistyped, and puts the
classification in one place.

Reason strings remain worth recording for **debuggability** (an NK whose cause a
Foolisher can read beats a bare one), but they are no longer load-bearing for
correctness. Note the FVM already computes this distinction and discards it: at
`fir_kinds.rs:1672` an anchored search tests `resolved.get_nyes() == Nyes::Nk`
— it knows there was no table to search — then sets a bare `Nyes::Nk`
indistinguishable from a genuine miss.

#### Value chases through; position does not

A continuation search is **shallow syntax**: context does not survive a
reference. This is the rule that decides what `@` reports, and it differs from
what a *value* does.

```foolish
{b = ?hello_world; … {a = b&=10}}
```
The **value** chases through. `b`'s value resolves to whatever `?hello_world`
found, so the value search compares against `10` correctly and matches when
`hello_world` is `10`. Constanic stepping delivers a firm `value()`, and a value
is a thing that can be carried anywhere.

```foolish
{b = ?hello_world; … {a = b@+1}}
```
The **position** does not. `@` reports **`b`'s own position**, not
`?hello_world`'s. `{a = b@+1}` asks "what comes after `b`, *here*?" — answering
with `?hello_world`'s neighbour would answer a question nobody asked, about a
brane the reader is not looking at.

The asymmetry is principled: a value is context-free, a position is meaningful
only relative to one specific brane. Following the chain would silently change
which brane the answer is about.

This is also *why* continuations are checked at construction time (below): the
syntax is shallow by design, so "is my anchor a search?" is a fact about the
source text, knowable without evaluating anything.

#### The requirement on contexted searches

Every contexted (continuation) search carries the same structural requirement,
and §8 makes it explicit and enforced:

> **A continuation's anchor must BE a search.**

This covers `&?`, `&~`, `&#`, `&=`, `&^`, `&$` — and now `@`. The reason is the
shallow-reference rule above: a continuation navigates *from a position*, and
only a search produces one. `{a=1}&#1` asks "one past where that landed", of
something that never landed anywhere; there is no position to continue from and
no sensible answer to give.

**Checked while building the FIR tree from the AST**, not at evaluation. The
requirement is a fact about the *source text* — "is the thing to my left a
search?" — so it is knowable without evaluating anything, and catching it at
construction means:

- a continuation FIR can **assume** its anchor is a search, with no defensive
  branch for a shape that cannot reach it;
- the diagnostic names the real problem (`&#1` applied to a brane) rather than
  surfacing later as a puzzling NK;
- the rule is stated once, in one place, for every continuation operator instead
  of being re-checked by each.

The existing operators are **not currently checked** — this FOOP adds the check
along with `@`, so `@` is not a special case but the newest member of a family
that now has its requirement enforced. Tests must cover a rejected anchor for
**each** continuation operator, not only `@`; a check that exists for one
operator and silently does not for its siblings is worse than none, because it
teaches a rule the language does not actually keep.

#### Non-equable candidates, and `NOT_EQUABLE_IS_NOT_EQUAL`

`candidates_exhausted()` means "the scan **decided** every candidate and none
matched" — not merely "the scan reached the end of the list". That raises the
question of what a *non-decidable* candidate does, and the answer is already
settled by an existing policy, now made explicit.

**An NK candidate never arises as a scan problem.** Verified: a brane with an NK
member goes **NK itself**, so a search over it has an NK *anchor* and the scan
never runs. `candidates_exhausted()` is `false` and `@` propagates NK — the
correct outcome, reached by brane-level NK propagation rather than by
per-candidate logic.

```foolish
tbl = {p=1; q=2; bad={a=1;}?nope;}    !! tbl itself is NK
hit = tbl~=(1)                        !! does NOT resolve — anchor is NK
```

**Non-comparable candidates are skipped, and the scan still completes.** A brane
compared against an integer is *not equable*, and `default_equal` classifies
that as **NotEqual** — so the matcher Rejects it and keeps scanning. Verified:
`{p=1; q={z=9;}; r=3;}~=(3)` finds `3`, scanning past the brane member; an
ECONSTANIC member is skipped the same way.

**Terminology: Not Mutually Identifiable — a clarification to FOOP-33 §2.**
This is vocabulary and an explicit seam, **not a behaviour change**: the default
stays what FOOP-33 already specified. A future FOOP should restate the equality
primitive succinctly in one place; §8 only records the clarification where the
work happened. "Comparison" overstates what this
primitive does — it does not order or measure, it asks whether two entities bear
the **same identity** (which is why two creations are equal exactly when they
are the same object). The formal term for a pair of kinds that can never bear
the same identity — a number and a brane, a vector and a matrix — is **Not
Mutually Identifiable**, defined in FOOP-33 §2. It is *decided*, not undecided:
a number is never a brane. That is exactly why it classifies as `NotEqual` and
not `Unknowable`, and calling it "incomparable" invites the opposite reading —
the reading that produced the FOOP-33 Phase-3 defect. The constant is named `NOT_MUTUALLY_IDENTIFIABLE_IS_NOT_EQUAL`.

Whether "not mutually identifiable" means **not equal** or **unknowable** is a **policy,
not a fact about the values**. §8 makes it an explicit, documented constant —
`NOT_MUTUALLY_IDENTIFIABLE_IS_NOT_EQUAL` in `fir_kinds.rs` — so it can be made configurable
later (per-suite, or per-search) without first having to find where the decision
was buried.

**The default is preserved**: the flag is `true`, which is exactly FOOP-33's
specified behaviour, and the alternative branch
documents what `false` would mean. It is deliberately a `const` rather than a
runtime setting; adding the configuration surface is future work, and this FOOP
only names the seam.

**Flipping it is not a small change.** `rust_instructions.md` records the
FOOP-33 incident where a three-valued `default_equal` returned Unknowable for
brane-vs-integer, which made value searches **abort** on the first
non-comparable candidate instead of skipping it — turning a working `mixed~=7`
into NK and silently changing eleven baselines. The test
`not_mutually_identifiable_is_not_equal` pins both the policy and the resulting
`Equality::NotEqual`.

So a scan that skipped non-comparable candidates and matched nothing **is**
exhausted: every candidate was decided, and "not equable ⇒ not equal" is what
deciding means under the shipped policy. This mirrors §7's rule that a
non-integer member "simply is not a candidate" for a fold.

#### The pattern-matching idiom

```foolish
tbl = {
  else_value=Z,
  key=A, value=B,
  key=AA, value=BB,
}

'match = {key=<<#-2>>, tbl=<<#-2>>, tbl~key=(key)@+1}
result =$ {key, tbl}'match
```

Or inline, with no wrapper at all:

```foolish
tbl {~key=PAT@+1}
```

Why `else_value` is written **first**, and why the `+1`:

| Outcome | `@` | `@+1` | `#(@+1)` selects |
|---------|-----|-------|------------------|
| hit on `key=A` (index 1) | `1` | `2` | `value=B` — the row beside it |
| hit on `key=AA` (index 3) | `3` | `4` | `value=BB` |
| **miss** | `-1` | `0` | **`else_value`** — index 0 |

**One expression handles both cases with no branch.** The `+1` that steps a hit
from its `key=` row to the adjacent `value=` row *also* steps a miss from `-1`
to `0`, which is precisely where the default was placed. That is the whole
reason the default comes first rather than last.

#### Why this succeeds where `&#1` did not

`&#1` carries a **position** — a reference into a specific brane at a specific
place — and that is exactly what could not survive the definition-site
coordination. §5's experiments bracketed the mark depth without landing it (one
mark and the pattern died NK, three and nothing resolved).

`@` carries an **integer**. Integers survive coordination because there is
nothing contextual about them. The construct stops depending on a preserved
position and starts depending on arithmetic.

It also subsumes `'ite` without a dedicated operator: a two-row table keyed on
`'True`/`'False` *is* an if-then-else, and an *n*-row table is an *n*-way
switch, which no fixed-arity operator provides.

#### Implementation note

`@` is currently **silently ignored** rather than a parse error:
`tbl~key=(77)@` and `tbl~key=(77)` both evaluate to `77` today. That is the
dangerous failure mode — a program written to this proposal would *run* and give
a plausible wrong answer. The tests must pin that `@` and no-`@` now differ.

### §9. Concatenation element marking — the five rules

`build_concat_element` / `classify_concat_element` (`compiler.rs:30-115`) decide
what mark, if any, a concatenation element carries. The rules were implicit in
the code and are stated here, forward-facing, as a **cascade tested in order**:

| # | Test | Action |
|---|------|--------|
| 1 | **Already SF- or SFF-marked at the top** | **as written** — add nothing, change nothing |
| 2 | **Constantew** (CONSTANT / INDEPENDENT / NK — "constant everywhere") | **as written** — nothing to defer |
| 3 | **Any search** | wrap **SF** |
| 4 | **Any brane-like** | build **SFF** (`under_sff = true`) |
| 5 | otherwise | `Error` → NK at construction |

**Rule 1 must be tested FIRST, before looking at what the mark contains.** The
current code inverts this for branes — it matches `StayFoolish{Brane}` and
`StayFullyFoolish{Brane}` in their own arms and re-wraps — which produces two
defects:

- `<{…}>` builds without `under_sff` and is then **re-wrapped in SF**: a mark
  added on top of the user's.
- `<<{…}>>` is classified `SfBrane` and **silently downgraded to SF**
  (`compiler.rs:67-68`), so writing a doubled mark in a concatenation gets
  single-mark semantics with no diagnostic.

Testing "is it already marked?" before "what is inside it?" makes both
impossible to write. `SfSearch` already behaves correctly — its comment reads
"idempotent NOOP, build as-is" — so rule 1 generalizes what that arm already
does.

**Rule 2 is the constantew case.** A CONSTANT, INDEPENDENT or NK element has no
context dependency, so a mark could not defer anything. Note the classifier runs
on the **AST**, before evaluation, so it cannot ask a NYES question — it
recognises the forms that are constantew *by construction*: `IntLit`,
`UnknownLit`, `Creation`, and arithmetic over them. (Today this is unreachable
in practice: the parser's `is_concatenation_continuation` does not accept an
integer as a continuation, so `{0} 5` parses as two statements rather than a
concatenation. The rule is stated so the classifier is complete on its own
terms.)

**Rule 3 gained two members in §8**: `Astn::SearchPosition` (`@`) and
`Astn::ComputedSeek` (`#(expr)`). Both are searches and both were missing from
the classifier, so they fell through to `Error`.

**But adding them does not make them usable as elements, and should not.** A
postfix search yields a **single value**, not a brane, and a concatenation
requires brane-like elements (`populate_concat_helpers`: "each element value
must be brane-like"). So `{0} tbl~k=(1)@` and `{0} tbl#(1)` still settle NK —
and so do `{0} tbl^` and `{0} tbl#1`, which were classified `BareSearch` all
along. Verified: all four behave identically, while `{0} tbl` (a search
resolving to a **brane**) concatenates to `{0; a=1; b=2}`.

An earlier draft of this section called the omission "§8's, not a gap in the
rules". That was wrong in an instructive way: the classifier was never the gate.
Classification decides the **mark**; whether an element is *usable* is a typing
question answered later, on the resolved value. Rule 3 is about the former only.

## FIR Impact

- **Two new FIR kinds** in `foolish-ubca/src/system_foo.rs`:
  - `ModuloFir` with `ArithOp` enum (§1). New `FirKind::Modulo` arm;
    new `constanic_clone_at` arm; display name `'mod`.
  - `OrFir` (§2, FVM-computed fallback). New `FirKind::Or` arm;
    new `constanic_clone_at` arm; display name `'or`.
- **`system.foo` grows**: `'mod = ⬤` and `'or = ⬤` (both body-overridden).
- **No new NYES states.** Both use the three existing terminals
  (ECONSTANIC / CONSTANT / NK).
- **No serialization impact** beyond sequencer rendering of the new kinds.
- **§5 changes no FIR kind and adds no NYES state.** The SF/SFF kinds, their
  `fir_op_step`, and every terminal state are untouched. What changes is how
  `constanic_clone_at` *treats* a mark it meets while cloning.
- **`system.foo` is unaffected** by §5.

## UBC Step Impact

- **Stepping is unchanged.** FIFO draining, `step_inner`, the task queue, the
  two-branch driver, and every search's wait condition are exactly as they are.
  §5 touches only `constanic_clone_at` — the recoordination path, not the
  stepping path.
- **`constanic_clone_at` gains a strip-budget parameter** and one new arm
  (retain-and-share a mark when the budget is spent). Single-mark programs take
  the same path they take today.
- **No step-count change is expected for single-mark programs.** A `steps=`
  movement in any of the 16 single-mark einmo baselines is a bug, not a baseline
  update — see §Test Plan.

## Test Plan

Tests first, per `rust_instructions.md` and AGENTS.md.

All einmo inputs this FOOP writes use the backtick application form
(FOOP-65) and explicit parentheses + `$` result extraction (D6/E5); they
are written after FOOP-65 merges.

**Unit tests (Rust, internal state):**
- `modulo_nyes_transitions` — REQUIRED for the new FIR kind; pins all three
  terminals, modeled on `comparison_nyes_transitions`
  (`system_foo.rs:627-669`).
- Modulo semantics through `compose_program_with_system`: `7 % 3 = 1`,
  `0 % 5 = 0`, truncation pinned (`(-7) % 3` and `7 % (-3)` per Rust `%`),
  `x % 0` → NK `"division by zero"`, brane operand → NK
  `"modulo: non-integer operand"`, ECONSTANIC inside `system.foo`.
- `'or` rows through `compose_program_with_system`: all four
  `{A, B, 'or}$` rows → the right `'True`/`'False` creation (by identity);
  non-boolean argument → NK.

**Einmo approval tests** under `foolish-ubca/einmo_suite/input/foop/55/`:
- `mod_basic.foo`, `mod_edge.foo` — modulo surface behavior incl. the NK
  cases.
- `or_table.foo` — all four `'or` rows + a non-boolean miss.
- `cmod.foo` — the exercise's congruent-modulo composition
  (`{a,b,c}'cmod` ≡ `a % b == c`).
- `ite.foo` — the exercise's if-then-else mechanism in isolation.
- `euler_small.foo` — the exercise's algorithm with 1000 → 10 (expected 23).
- `comprehensive.foo` — the reserved comprehensive test
  (`einmo_suite/input/foop/55/comprehensive.foo` per INDEX P5 re-homing):
  `'mod`/`'or` interacting with comparisons, value search, contexted
  search, concatenation, and the SF/SFF markers.
- The exercise itself — `input/exercises/project_euler/1.foolish` gains its
  `checked/` baseline (answer 233168) **after human review**; the
  verified-stage signature requires the human key (einmo.toml leaves
  `verified` unconfigured on purpose).

Every OUTPUT line is justified before promotion (AGENTS.md einmo workflow
step 4); no promotion over any foreign FOOP's baseline
(`rust_instructions.md` §"Phase-by-phase testing discipline").

### §5 — the SFF mark upgrade

**Unit tests (`fir_kinds.rs`), written first:**

- One clone strips exactly one mark: `<<X>>` → stripped; `<< <<X>> >>` → one
  mark retained.
- The budget is **per-tree**: `<{a; << <<b>> >>}>` spends one strip across the
  SF wrapper and the inner SFF combined, not one each.
- The budget is **per-use-site**: `B = A` and `C = A` each get their own.
- A retained mark is **shared** (`Rc::ptr_eq` against the original), not deep
  copied.
- The retained path never reaches the "SF/SFF node has no children" ALARM.

**Existing einmo inputs — 17 use SFF marks; exactly ONE nests:**

- `misc/sff_nested.foo` — `{a=1,b=2; c=<<a+<<b>>>>; c; c;}` — is a **direct
  semantic conflict**. Its inner `<<b>>` means "resolve on each use" today and
  "defer one coordination" after §5. **Deprecate it**: the input is replaced by
  the new `foop/55/SFF/` cases below, which cover the same ground under the new
  rule. Deprecation requires human review — the agent does not silently rewrite
  a baseline whose meaning changed.
- The **other 16** use single marks and **must produce byte-identical OUTPUT**,
  step counts included. Any divergence there is a regression in the strip
  budget, not an expected consequence — fix the code.

**New einmo inputs under `foop/55/SFF/`** (reserved for this FOOP):

| Input | Covers |
|-------|--------|
| `single_mark_unchanged.foo` | the `<<X>>` cases, pinning no-change |
| `double_mark_defers.foo` | `<< <<X>> >>` sits out one coordination, resolves at the next |
| `budget_is_per_tree.foo` | SF wrapper + inner SFF share one strip |
| `budget_is_per_use_site.foo` | `B=A`, `C=A` decrement independently |
| `nested_in_expression.foo` | `1 + << <<X>> >>` inside an operator |
| `deferred_avoids_premature_nk.foo` | the `A=<{...}>; B=A; C=B` case: resolves at `C` instead of dying NK at `B` |
| `separator_forms_agree.foo` | `<< <<A>> >>` and `<<(<<A>>)>>` produce identical OUTPUT |
| `nest_case1_syntactic.foo` | **Case 1** — marks on ONE term: `<< 1 + << 2 + <<x>> >> >>` resolves one boundary per coordination. Pins that depth is a count *within* a term. |
| `nest_case2_search_chain.foo` | **Case 2** — marks on SEPARATE terms: `{A=<<a>>; B=<<A>>; C=<<B>>; r=C}` resolves one hop per link and settles ECONSTANIC ("couldn't find `a`"). **This must be byte-identical before and after §5** — every statement is single-marked, so no clone tree ever meets a second mark. It is the control proving §5 changed only what it claims to. |
| `nest_case2_chain_lengths.foo` | Case 2 at chain lengths 1, 2, 3 — all terminate, each adding one `result=` layer. Pins the per-hop-fresh-budget behavior. |
| `nest_case2_double_link.foo` | §5 point 2 — `{A=<<a>>; B=<< <<A>> >>; C=<<B>>; r=C}` defers one extra hop. **Prediction, not verified**; if the implementation disagrees, amend §5 to match and record why. |
| `nest_case2_mixed.foo` | §5 point 3 — a link whose body is syntactically nested (`B = << 1 + <<A>> >>`) spends one strip on that term's outer mark. Also a prediction. |
| `nest_chain_that_hits.foo` | The untested path: a chain that **finds** something and carries a value back through each hop, rather than missing. |
| `continuation_value_vs_position.foo` | §8's asymmetry, both halves in one case: `{b = ?hello_world; … {a = b&=10}}` MATCHES when `hello_world` is 10 (value chases through), while `{a = b@+1}` reports **`b`'s own** position, not `?hello_world`'s (position does not). |

Each is reviewed statement by statement through the Promotion Review Gate
(`foop.md`) before promotion.

## Rejected Alternatives

### A. Fix the leading-underscore lexer defect now

Would let the exercise keep its `_ite`/`_cmod` names. **Deferred by Atlas
(2026-08-07)** — the `INTERN_` prefix works around it and keeps this FOOP
minimal; the full fix details are preserved in D1 so the parser can still
be changed later.

### B. Infix `%` surface operator

`OperatorFir` already evaluates a `"%"` arm (`fir_kinds.rs:691-711`) but
the lexer/parser have no `%` token, so no surface syntax reaches it. The
exercise uses the postfix `'mod` creation form, which composes with the
language's application idiom; minting an infix token is out of scope here
(it is FOOP-83's neighbourhood — integer math strengthening).

### C. FVM-computed `'or` as the first choice

A `'mod`-shaped FIR kind dispatching on `Rc::ptr_eq` against the
`system.foo` booleans. **Demoted to fallback** (FOOP-73 Rejected
Alternative A): it reintroduces a privileged layer the truth-table design
avoids entirely. Taken only if the table search proves insufficient, and
only after the plan's STOP-and-consult checkbox.

### D. Do nothing

The exercise stays dead: the first real program written for the language
cannot run, and the defects found by bisecting it stay undocumented and
unpinned.

The §5 design space — the four approaches considered and why the SFF-mark
upgrade won — is set out in prose in **Appendix A**.

## Open Questions
- **(§5) Does the strip budget serve every recursive shape we need?** Euler 1's
  recursion is one shape: a self-reference in a branch the value search does not
  select. The rule is expected to generalize — an unselected branch is never
  coordinated again regardless of *why* it is expensive — but this is **not
  established**, and no attempt has been made to enumerate the recursive
  patterns Foolish will need (mutual recursion, a recursive call in the
  *selected* branch, accumulator-passing where the recursive term is also the
  value). §5 is specified against the shape the exercise needs. Later exercises
  will find the others, and this section is expected to grow rather than to have
  anticipated them. **Adjust §5 when a shape breaks it**, and record what broke.
- **(D7) How is the pure-Foolish route unblocked?** D7 (a system operator inside
  a juxtaposed definition never settles — unbounded `Index` recursion) is a
  pre-existing defect independent of §5, and it blocks `congruent_modulo`
  (Phase 2C) and pure-Foolish `'ite` (Phase 3D). Three options, undecided:
  **(a)** fix D7 in this FOOP — correct but widens scope into the index/
  recoordination machinery; **(b)** keep `'ite` as a Rust FIR kind for now,
  accepting the collaborator burden Appendix A.A argues against, and fix D7 in
  its own FOOP; **(c)** find a formulation of `'ite` that avoids the failing
  shape entirely. §5's own work (Phase 3A) is complete and unaffected either
  way.



- **Modulo with negative operands** — Rust truncating semantics are pinned
  for now; Euclidean modulo, if ever wanted, is a future FOOP.
- **The exercise-file fixes** — Atlas is applying E1/E2/E3; plan Phase 0
  verifies they landed before any implementation and does not edit the
  exercise itself.
- **(§5) What exactly freezes a concatenation's shape?** `is_indexable()` must
  mean "the shape can no longer change", not "the shape is currently readable"
  (§5). For a concatenation whose operands are themselves searches returning
  branes, the answer is *not yet decidable* until those searches settle. The
  precise condition — and where it is computed without walking the whole tree
  on every step — is the first thing plan Phase 4 measures on the live FVM.
- **(§5) Early exit and monotonicity.** The claim is that a frozen shape makes
  the early exit safe permanently. The einmo suite already demonstrates the
  failure mode when a search settles against a brane that later changes; plan
  Phase 3B must name those cases and confirm the frozen-shape rule excludes
  them.
- **(§5, deferred) Is `is_value_searchable()` a whole-brane predicate at all?**
  A backward value search may settle on the first hit without touching earlier
  candidates, which suggests a per-candidate demand made by the navigator as it
  advances, rather than a predicate over the brane. Not resolved here; the
  value-search phase is deferred and this question gates it.

- **The exact final shape of the `'or` lookup** — `T~A=A &~B=B &#1` is
  traced on paper (§2 table); Phase 2 confirms it on the live FVM and
  adjusts only if the trace was wrong.
- **BodyOverride hook naming** — generalizing `comparison_body` to a system
  name table is an implementation detail left to the implementer.
- **The `=$`/`$=` sugars** — `$=` does not parse and `=$` settles
  non-obviously (D6); their intended semantics need their own
  investigation FOOP. FOOP-55 avoids both.
- **MAX_DEPTH / iteration budget** (§4 risks 1-2) — **largely subsumed by §5.**
  These were framed as budget questions; the measurement of 2026-08-09 shows
  the exercise computes nothing at any budget, so the cause is the greedy wait,
  not the ceiling. Whether a depth guard is *still* needed once §5 lands — and
  whether `step_inner`'s silent `NoProgress` at `MAX_DEPTH`
  (`fir_trait.rs:467-469`) should instead be a loud error — is measured in
  plan Phase 7, after §5's staged implementation.

## References

- Prior FOOPs: FOOP-33 (creation, `system.foo`, comparisons — §5.0/§5.1 are
  the mechanism this FOOP extends); FOOP-73 (boolean operators — this FOOP
  realizes its preferred design for `'or` only); FOOP-23 (value search,
  contexted search, settlement); FOOP-62 (two-store stepping); FOOP-83
  (integer math — future home of infix concerns); FOOP-65 (tail
  concatenator — prerequisite: the exercise rewrite E4 uses backtick
  application).
- Process: `foop.md`; `rust_instructions.md` §"Phase-by-phase testing
  discipline"; AGENTS.md §"The einmo review workflow".
- Code anchors: `foolish-ubca/src/system_foo.rs` (ComparisonFir 158-341,
  `operand_is_unevaluated_here` 351-358, `comparison_body` 384-391,
  `comparison_nyes_transitions` 627-669); `foolish-ubca/src/compiler.rs`
  (`BodyOverride` 468-505); `foolish-ubca/src/fir_kinds.rs` (`OperatorFir`
  combine 655-713 incl. the `"%"` arm, `SearchPredicate` 2059+);
  `foolish-ubca/src/fir_trait.rs` (`MAX_DEPTH` 395, `step_inner` 453-486);
  `foolish-ubca/src/evaluator.rs` (iteration alarm 168, `"$"` 351);
  `foolish-parser/src/lexer.rs` (identifier gate 293, fallback 297-299,
  `block_comment` 302-329, `is_id_sep` 80-86);
  `foolish-parser/src/parser.rs` (`expect` 58-73, `skip_comments` 88-95,
  assignment operators 315-358, `is_concatenation_continuation` 390-411,
  SF/SFF arms 962-977, `parse_seek_index` 990-1000).
- The exercise: `foolish-ubca/einmo_suite/input/exercises/project_euler/1.foolish`
  (commit `62706518`).

---

# Appendix A — the §5 design space

Four approaches were considered for making the exercise terminate. They are
recorded here in full because three of them remain live proposals for the
platform, and because the reasoning that eliminated each is the reasoning that
justifies the one chosen.

## A. The upgraded SFF mark — **chosen**

This is the design specified in §5, and it began as a question about macro-style
definitions rather than about termination. A Foolish "macro" — `'ite`, `'cmod`,
a truth table — is a brane written once and coordinated at many use sites. Its
body must survive being carried to the use site *without resolving on the way*,
because resolving early means resolving against the wrong neighbours.

Today a constanic clone strips every mark it meets, in a single recursive pass,
so a mark can protect a term across exactly one boundary and no more. That is
enough for a one-level macro and insufficient for anything that builds a table
and *then* selects from it — which is precisely what `'ite` does.

Making the mark a **counter** — one layer discharged per coordination — gives
the macro author explicit control over how many boundaries a term rides
through. Deferral becomes a property of the term, written in the source, rather
than a behavior the evaluator infers.

Termination then falls out as a consequence rather than as a goal: the branch
the value search does not select is never coordinated a second time, so its
inner mark never comes off, so it never searches, so the recursion never fires.
Nothing in the FVM learns to be lazy; the unselected branch simply is not yet a
search.

Its cost is honest and bounded: it changes the meaning of nested marks, which
one existing test relies on, and it demands careful testing that single-mark
programs are untouched.

## B. `@` — retrieving the index from a search's context

The second proposal added an operator, `@`, projecting a search result's
**position** where a bare result yields its value: `c~cond='True@` gives the
0-based index of the matching row in its home brane. Selection could then be
written as arithmetic — `r#(c~cond='True@)` indexes a parallel table of
branches, with `-1` naturally addressing the last row as a default case, since
Foolish already counts negative indices from the end.

The mechanism needed nothing new underneath: FOOP-23's two-child invariant means
every search result already carries `[0]` its value and `[1]` a `FoolRefFir`
holding the original statement with its position intact. `@` merely exposes what
providing-context has been carrying all along. `@` was preferred over a trailing
`#` because `#` is absorbed into `?`-patterns today (`src?b#` searches for the
name `b#`), so overloading it would break the pattern language and require a
delimiter rule; `@` has no such conflict and composes directly with arithmetic.

Two findings stopped it. The first is practical: `#` accepts only a **literal**
integer — `src#n` and `src#(0-1)` both fail to parse — so the design needed a
second, larger change giving `#` a computed operand, which in turn gives the
index search an evaluation phase it does not currently have.

The second finding is the reason this appendix entry matters. Probing what `#`
does on an anonymous brane showed that the short-circuit **already exists**:
`pick = {ok=1, bad=?nonexistent}#0` settles `1` with the unselected member never
stepped, and `{stop=7, go=<lp>}#0` terminates cleanly with the recursive branch
untouched. Indexing an anonymous brane already avoids what it does not select.

So access was never the problem. The problem was that the branches had already
resolved — the SFF mark protecting them was stripped too early, at table-
construction time. Investigating `@` is what located the actual defect, and once
located, `@` was no longer needed to fix it. It remains a reasonable future
addition on its own merits.

## C. True breadth-first execution — **UBCc**

The third proposal made `depth` a real parameter of stepping: `if depth == 0
{ return }`, otherwise do this FIR's work and recurse with `depth - 1`, with the
FVM holding the depth per invocation, starting small and growing when a sweep
does not settle. Evaluation would sweep a bounded frontier across the whole tree
instead of descending one spine, and the condition of an `'ite` would settle in
an early sweep — long before the recursive branch could run away.

This was initially proposed as a change of *strategy* that could not alter
settled values. That was wrong, and the error is worth recording. The task queue
is FIFO and a front task is popped only once constanic, so statements drain
strictly in order: when `b` is stepped, `a` has already settled. FIRs are
**entitled** to that. `ib_search_with_engine` scans the preceding range
`[0, idx-1]` and takes the found statement's NYES *as truth* with no readiness
check (`fir_kinds.rs:1430`); roughly 185 `get_nyes()` reads and 83 `.value()`
calls across the crate rest on the same entitlement; and FOOP-23 *defines* the
immediate brane as "context accumulated so far, lines before the current
expression". Breadth-first stepping does not reorder the meaning of "so far" —
it dissolves it.

Sequential draining is therefore part of Foolish's evaluation **semantics**, not
an implementation strategy. Adopting breadth-first execution means every FIR
must newly cope with unsettled predecessors, and settled values may legitimately
differ — which is exactly the condition under which the signed einmo baselines
stop being a valid oracle, and therefore the condition under which a **separate
implementation** is warranted rather than a mutation of the existing one.

Breadth-first execution is a genuinely attractive design and is **not
withdrawn** — it is renamed. It becomes **UBCc**, to be built beside UBCa rather
than inside it, with its own baselines.

## D. Massively distributed message passing — **UBCd**

The most general proposal: each UBC is a small computer with its own state, and
searches are dispatched as messages to parents, which reply when they can
answer. Waiting stops being a drain loop and becomes a correspondence — a FIR
that has its answer simply stops waiting, and nothing over-drains, because
nothing was ever draining in the first place.

This subsumes the exercise's problem completely and several others besides. It
is also a rewrite of the evaluator's core: it changes what a "step" *is*, and
therefore every `steps=` in every einmo baseline. It cannot be carried by a FOOP
whose purpose is to run one exercise.

It is recorded here as **UBCd**, a design worth its own Major.

## The UBC lineage — authoritative code names

**This section defines the UBC letter names, and these are the names to use in
all ongoing discussion.** They have been used loosely before; from here they are
fixed. A letter names an *implementation of the Foolish evaluator*, not a
version of the language — all of them implement the same Foolish, and any two
that disagree about what a program means have a bug in at least one of them.

| Name | Design | Status |
|------|--------|--------|
| **UBCa** | The reference implementation. FIFO sequential draining, depth-first descent, greedy dependency resolution. | **In use.** §5 of this FOOP modifies it. |
| **UBCb** | **Dependency tracking with priority stepping** — the evaluator records which FIRs are being waited on, and steps first those experiencing the most demand from dependents. | **Attempted for several months; not adopted.** Its dependency-tracking machinery is expected to be reusable. |
| **UBCc** | **True breadth-first execution** (Appendix A.C) — `depth` as a real parameter of stepping, sweeping a bounded frontier that grows until the program settles. | Proposed. |
| **UBCd** | **Massively distributed message passing** (Appendix A.D) — each UBC a small computer with its own state; searches dispatched as messages to parents, answered when answerable. | Proposed. |

The relationships matter as much as the definitions:

- **UBCb feeds UBCd.** Dependency tracking is not wasted work even though UBCb
  was not adopted. Knowing *who is waiting on whom* is the same information a
  message-passing evaluator needs in order to route a reply, so UBCb's machinery
  is expected to be reusable in UBCd rather than discarded with it.
- **UBCc stays close to UBCa.** Breadth-first execution is a change of traversal
  within the same basic architecture — one machine, one tree, a task queue. It
  is therefore the natural *next reference implementation*: near enough to UBCa
  to be compared against it case by case, and far enough to answer the question
  UBCa answers badly.
- **UBCb and UBCc answer the same question from opposite ends** — *which
  pending work should the machine do next?* UBCb measures demand; UBCc bounds
  depth. Which is built next is deliberately left open here.

None of UBCb, UBCc, or UBCd is a prerequisite for this FOOP.

## Last Updated

**Date**: 2026-08-09
**Updated By**: Claude Code / claude-opus-5
**Changes**: Added **§6 — the brane view and the unanchored forward search
`~name`** (2026-08-11), plus **D7**. §5's mark counting was demonstrated to
compose exactly as specified (one mark on `'ite`'s value-search pattern => the
pattern dies NK; three => it survives but nothing resolves; each mark buys one
coordination), but it never lands `'ite`, because `&#1` is a CONTEXTED index
whose carried position must survive the definition-site coordination. §6 removes
the need for one: a brane view is a contiguous `[0, my_index-1]` window sharing
the home brane's parent, and `~name` is the ORDINARY anchored forward search
over it. The view can be constanic before the whole brane is — FIFO guarantees
the window has settled — so it never waits, the same narrowing move §5 makes.
FOOP-23 §A.1's "cannot look forward in its own brane" is preserved literally:
the window holds only settled statements. Earlier: **§5 rewritten around the upgraded SFF mark** — the third and final
design. A constanic clone currently strips EVERY SF/SFF mark in one recursive
pass (`constanic_clone_at`, `fir_kinds.rs:183-191`), so nesting is a no-op today
and a mark protects a term across exactly one coordination. §5 makes it a
counter: at most one strip per clone TREE (not per node), with retained marks
SHARED by `Rc::clone` since an unstripped mark is immutable and
position-independent. Demonstrated to be a correctness fix, not an optimization:
premature stripping resolves against the wrong neighbours and an early miss
settles NK, which is terminal. §5 now includes the **full Euler 1 program in
fenced code** as it is expected to run. Body reorganized for implementation:
design, program, and implementation steps in §5; all decision-making moved to
**Appendix A**, which sets out the four candidate designs in prose (A the SFF
mark — chosen; B the `@` index projection — whose investigation located the real
defect; C breadth-first execution; D message passing) and **authoritatively
defines the UBC code names**: UBCa reference, UBCb dependency-tracking
(attempted for months, machinery expected to feed UBCd), UBCc breadth-first
(closest to UBCa, the natural next reference implementation), UBCd message
passing. §Test Plan gains the SFF corpus analysis: of 17 `<<`-bearing einmo
inputs exactly ONE nests (`misc/sff_nested.foo`) and is a direct semantic
conflict to be deprecated under human review; the other 16 must be
byte-identical. New `foop/55/SFF/` suite specified. Earlier same day:  **§5 redesigned** around two mechanisms per Atlas's
direction, replacing the readiness-gating design of the first draft: (i) `depth`
as a real parameter of `step` — `if depth==0 return`, else work and recurse with
`depth-1` — held by the FVM per invocation, starting ≈5 and growing by a delta
(≥1) when a sweep does not settle, making stepping genuinely **breadth-first**;
and (ii) **`i_have_what_i_need()`**, where a FIR steps `foolish_children` until
they are **ALL constanic OR** the predicate is true, default `false` so no
un-opted kind can regress. The predicate **cannot be wrong** — a premature
`true` settles a FIR on incomplete information, silently. `is_indexable()` is
retained but becomes a plain `bool`: it is asked *of the brane-like*, which
reports its own shape (encapsulation, `rust_instructions.md` rule zero), and the
three-valued answer is withdrawn because "not yet" and "no" now share the same
safe consequence — keep draining. Staged **5.0 depth → 5a plain brane → 5b
concat → 5c other kinds → 5d `'ite`**, with 5.0 required to leave settled values
identical. Added Rejected Alternatives H (full message-passing FVM — subsumes
the problem but is a core rewrite deserving its own Major) and I (explicit
dependency tracking — the same regress the greedy wait falls into). Earlier
same day: added **§5 "Readiness-gated searches"** — the actual cause of §4
risks 1-2. Measured: the exercise computes *nothing* (every statement
PREMBRIONIC) at a 40000-step budget, so the `@einmo set iteration depth to
40000` directive enlarges the room in which it fails rather than fixing it.
Root cause: `step_inner` (`fir_trait.rs:466-499`) never runs a FIR's own
`fir_op_step` while its task queue is non-empty, and an anchored search
enqueues its whole anchor — so `IteFir`, which enqueues all three operands,
drains the discarded `<loop>` branch unconditionally and never terminates.
Specifies `is_indexable()` / `is_name_searchable()` / `is_value_searchable()`
on brane-like FIRs, with a ready search **retargeting its task queue to the one
selected statement**; `'ite` short-circuiting then falls out with no laziness
rule added to the FVM. Records that indexability must mean **shape frozen**,
not merely currently-readable (a selected statement carries its dependencies —
selecting out of an incomplete concatenation would resolve its searches against
a brane missing members), hence a three-valued readiness answer where
"undecided" is distinct from "false". `is_indexable()` staged **5a plain brane
→ 5b concatenation → 5c other brane-like kinds → 5d `'ite`** so the retargeting
mechanism is proven on the parse-time-settled case before the freezing question
arises; the other two predicates are specified but deferred. Added Rejected
Alternatives E (raise the budget), F (make `IteFir` lazy directly), G (the
spec's search-based `'ite` alone), five §5 Open Questions, and FIR/UBC-Step
Impact entries including the expected suite-wide step-count *reduction* that
must be reviewed case by case. Earlier: post-FOOP-65-creation revision, adding
the **FOOP-65**
dependency** — the exercise rewrite uses backtick (tail-concatenator)
application: new E4 carries the full line-by-line rewrite mapping; §4
risk 0 and the plan's Phase 0 gate on FOOP-65 merged. Added **D6** (`$=`
does not parse; `=$` desugars to `BinaryOp{$, #-1, B}` and settles
non-obviously — repros) and **E5** (the six `$=` lines rewritten as
explicit `(X)$`); corrected the §3 sugar row accordingly. Earlier same
day: created (Draft) — bisected the exercise failure, specified `'mod`
(FOOP-33 §5.1 mechanism) and `'or` (FOOP-73 preferred design, `'or`
only), documented D1–D5 and E1–E3 incl. the INTERN_ decision.
