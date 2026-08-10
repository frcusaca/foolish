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
3. **Readiness-gated searches (§5)** — an anchored search currently waits for
   its *entire* anchor to become constanic before it may look at it, which is
   far stronger than any search's meaning requires (`$` asks a question about
   **shape**, not values). Brane-like FIRs gain `is_indexable()` /
   `is_name_searchable()` / `is_value_searchable()`, and a ready search
   **retargets its work queue to the one statement it selected**, dropping the
   siblings it will never read. This is what makes `'ite` short-circuit — and
   therefore what makes the exercise's recursion terminate — **without adding
   any laziness rule to the FVM**. Measured 2026-08-09: the exercise currently
   computes *nothing* (all PREMBRIONIC) at a 40000-step budget, so this is a
   correctness repair, not an optimization. `is_indexable()` is implemented
   here, staged 5a–5d; the other two predicates are specified but deferred.

— and it **documents, with reproductions**, six platform defects (D1–D6)
and five exercise-file defects (E1–E5) found while bisecting the failure.
Per Atlas's direction (2026-08-07) the leading-underscore lexer defect (D1)
is **worked around, not fixed**: the exercise uses an `INTERN_` prefix
instead of a leading `_`, and this FOOP records the full details needed to
fix the parser later if that becomes desirable.

**Dependency:** the exercise rewrite uses FOOP-65's tail-concatenator
(backtick) application form — FOOP-65 merges first; see E4.

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

### §5. Readiness-gated searches — the actual cause of risks 1 and 2

**Measured 2026-08-09.** Risks 1 and 2 above are not independent hazards to be
budgeted around; they are symptoms of one defect in how anchored searches wait.
The exercise does not merely run long — it computes **nothing**. Every statement
in `output/exercises/project_euler/1.foo.einmo` settles `PREMBRIONIC`: `answer`
unresolved, the searches for `loop`, `cond2`, `sum35`, `'ite` unresolved. The
`@einmo set iteration depth to 40000` directive raises the ceiling without
changing the shape of the problem, and must be removed once §5 lands.

#### The defect

`step_inner` (`fir_trait.rs:466-499`) has exactly two mutually exclusive
branches: if the task queue is non-empty, drain it; only when it is empty does
the FIR run its own `fir_op_step`. An anchored search enqueues its anchor at
PREMBRIONIC (`fir_kinds.rs:1634-1641`) and is therefore **structurally blocked
from searching until the entire anchor is constanic** — every statement of it,
including ones the search will never look at.

For `'ite` this is fatal. `IteFir` enqueues all three operands
(`ITE_OPERAND_SRC` — cond, then, else) and guards on
`operands.iter().any(operand_is_unevaluated_here)`, so the driver drains *all
three* before `cond` can be consulted. In the exercise's

```foolish
answer = {cond2, sum35, <loop>, 'ite}$
```

the discarded `<loop>` branch is in that queue and is drained unconditionally.
The base case is correct and simply never gets to stop anything. `'ite` passes
`input/foop/55/ite.foo` (`r1=42`, `r2=99`) only because both branches there are
literals — the greediness is invisible until a branch is recursive.

#### The change: ask a per-predicate readiness question

Waiting for constanic is stronger than any search's meaning requires. `$` asks
"which statement is last?" — a question about **shape**, not about values.
Brane-like FIRs (brane, concatenation, and the other `is_brane_like` kinds) gain
three predicates, and each search waits on the one its predicate needs:

| Predicate | Searches | Ready when |
|-----------|----------|------------|
| `is_indexable()` | `$` `^` `#N` (and `&#`, `&^`, `&$`) | the brane's **shape is settled**: statement count and positions final, and every constituent spliced into place |
| `is_name_searchable()` | `?` `~` `.` (and `&?`, `&~`) | statement **names** are settled; bodies need not be |
| `is_value_searchable()` | `?=` `~=` (and `&?=`, `&~=`) | the **values being matched** are settled |

Once ready, the search **retargets its own work queue**: it resolves the target,
pushes the selected statement to `ubc_children` as it does today, and **replaces
the task queue with just that one item**. Unselected siblings are dropped from
the queue and never stepped.

This is the load-bearing move, and it is not merely "start earlier": it narrows
the dependency set as soon as the search knows which member it actually needs.
The queue stops meaning "everything I was born depending on" and starts meaning
"what I still need". `step_inner`'s two-branch shape is unchanged — `fir_op_step`
gains the ability to *rewrite* the queue, not only to be gated by it.

`'ite` short-circuiting then falls out at the right layer: `{cond, then, else,
'ite}` becomes ready once the brane is indexable, selects on `cond`, and
retargets to the winning branch alone. **No laziness rule is added to the FVM
and no per-operator evaluation order is introduced** — the same mechanism that
makes `$` cheap makes `'ite` terminate.

#### Indexability requires a *complete* brane, not a partial one

A tempting weakening — "as soon as constituents at the front or back are known,
attempt to select" — is **rejected**. A selected statement carries its
dependencies: its backward searches resolve against its home brane, and its line
number is meaningful only relative to that brane. Selecting statement #3 out of
`({a}{b}{c})` before the concatenation has spliced all three operands into place
yields a statement whose own searches would scan an incomplete brane and could
settle NK against members that do not exist yet.

Therefore `is_indexable()` means **the shape can no longer change** — frozen,
not merely currently-readable. This also makes retargeting **monotone**: a
choice made under a frozen shape can never be invalidated by later growth. The
weaker reading would permit a search to select #3 and then have the brane grow,
which the einmo suite already demonstrates going wrong (§Open Questions).

`is_indexable()` is therefore recursive over constituents and admits a **third
answer** — *not yet decidable* — distinct from a definite `false`; an operand
that is itself an unresolved search returning a brane leaves indexability
undecided rather than denied. Conflating "undecided" with "false" would stall
searches that could proceed; conflating it with "true" reintroduces the
incomplete-brane bug above.

#### Scope within this FOOP — indexability is staged

**`is_indexable()` is implemented first and alone**, and is itself split into
stages so the retargeting machinery is proven on the easy case before the hard
one. It is what Euler 1 needs: `$` on the concatenations, and `'ite`
short-circuiting via indexed selection.

| Stage | Subject | Why this order |
|-------|---------|----------------|
| **5a** | **Plain brane** | A brane's shape is settled at parse time — its statement count and positions never change — so `is_indexable()` is trivially `Ready` and the freezing question does not arise. This stage builds and proves the whole retargeting mechanism (readiness question, queue rewrite, dropping unselected siblings) against a case with no ambiguity. |
| **5b** | **Concatenation** | The hard case: shape is settled only once every operand is spliced in, and an operand may itself be an unresolved search. This is where the three-valued answer and the frozen-shape rule earn their keep. Built on 5a's proven mechanism. |
| **5c** | **Remaining brane-like kinds** | Whatever else answers `is_brane_like` — swept once the rule is settled, each with its own readiness answer. |
| **5d** | **`'ite` short-circuit** | Retarget on `cond` instead of enqueueing all three operands. Depends only on 5a for a literal-operand `'ite`; the exercise's recursive branch needs 5b. |

Splitting this way means a regression in 5b cannot be confused with a defect in
the retargeting mechanism itself — 5a's tests pin that independently.

`is_name_searchable()` and `is_value_searchable()` are specified here as the
same shape, so the design is coherent, but are **deferred to their own phases**
and are not required for this exercise to pass.

Value search is expected to need a different shape and is explicitly not solved
here: `?=` must compare candidate *values*, so its readiness is close to
"constanic" for the statements it scans — but only for those it actually
reaches, and a backward scan may settle on the first hit without ever touching
earlier candidates. That suggests readiness for value search may not be a
whole-brane predicate at all, but a per-candidate demand made by the navigator
as it advances. §Open Questions records this.

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
- **§5 readiness predicates**: `is_indexable()` on the `Fir` trait (default
  implementation returning "not yet decidable" for non-brane-like kinds),
  overridden by brane and concatenation. Returns a three-valued answer
  (`Ready` / `NotYet` / `Never`), not a `bool` — see §5. No new FIR kinds and
  no new NYES states; readiness is a question *about* a FIR's shape, not a
  state it occupies.

## UBC Step Impact

- **New `fir_op_step`** for both `'mod` and `'or` — the two-phase
  enqueue-then-combine shape of `ComparisonFir` (`system_foo.rs:326-340`),
  with §1/§2 `combine` rules.
- **`BodyOverride` hook generalized** from comparisons-only to a system
  name table covering `ComparisonOp::ALL` + `ArithOp::ALL` + `'or`
  (`system_foo.rs`); still scoped to `system.foo`'s own top-level
  statements only.
- **§5 changes the wait condition of every anchored search.** `SearchFir`'s
  BRANING arm (`fir_kinds.rs:1673-1694`) stops requiring a constanic anchor and
  instead consults the predicate's readiness question; on ready, it retargets
  its task queue to the single selected statement. `step_inner`
  (`fir_trait.rs:466-499`) is **unchanged** — its two-branch shape already
  permits this; what changes is that `fir_op_step` may now rewrite the queue
  rather than only being gated by it.
- **`IteFir` stops enqueueing all three operands.** It enqueues `cond`, and on
  `cond` settling, retargets to the selected branch alone. This is the
  short-circuit; it is expressed in the same retargeting mechanism as `$`, not
  as a special evaluation rule.
- **Expected step-count reduction across the suite.** Searches that previously
  waited for a whole anchor now settle earlier, so `steps=` in einmo OUTPUT
  will drop for many pre-existing cases. Per the non-regression invariant this
  is the one category of foreign-baseline change this FOOP may legitimately
  produce — it must be **reviewed case by case and reported to the human**, not
  promoted silently (see the plan's Promotion Review Gate). A step count that
  *rises*, or any change to a settled value, is a bug.

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

### E. (§5) Raise the step/iteration budget and ship the `@einmo` directive

The current state: `@einmo set iteration depth to 40000` in the exercise
header. Rejected as a solution — it does not make the program terminate, it
enlarges the room in which it fails to. The measured output is entirely
PREMBRIONIC at 40000 steps; no budget makes a non-terminating demand finish.
The directive is removed when §5 lands.

### F. (§5) Make `IteFir` lazy directly — evaluate `cond`, then one branch

The narrow fix: special-case `'ite` to step its condition first and only then
the selected branch. Rejected because it introduces per-operator evaluation
order into the FVM — a genuine semantic addition — to solve one operator's
instance of a general problem. Every anchored search waits on more than it
needs; `'ite` is merely where it becomes fatal rather than merely slow. The
readiness-gated design fixes the class at the layer where the waiting is
decided, and yields `'ite` short-circuiting as a consequence rather than as a
rule.

### G. (§5) Rewrite `'ite` as the spec's search-based selector only

FOOP-55 §Specification's original form —
`'ite = ({<<-2>>, <<-1>>} _ite)~cond=(<<-3>>)&#1` — gets non-evaluation of the
losing branch from value-search semantics, with no FVM change at all. It
remains the more elegant surface form and is not withdrawn. Rejected **as the
sole fix** because it depends on `is_value_searchable()` (§5, deferred) to know
when the `~cond=` search may proceed, and because it leaves every *other*
anchored search over-waiting. Worth revisiting as surface syntax once §5's
value-search phase lands.

## Open Questions

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
- **(§5) Is the three-valued readiness answer the right shape?**
  `Ready` / `NotYet` / `Never` is proposed so that "undecided" is never
  conflated with "false" (which would stall) nor with "true" (which would
  select out of an incomplete brane). Whether `Never` is reachable at all for
  indexability — as opposed to only via an NK anchor, which is already handled
  — is open.
- **(§5) Retargeting and monotonicity.** The claim is that a frozen shape makes
  retargeting safe permanently. The einmo suite already demonstrates the
  failure mode when a search settles against a brane that later changes; plan
  Phase 4 must name those cases and confirm the frozen-shape rule excludes
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

## Last Updated

**Date**: 2026-08-09
**Updated By**: Claude Code / claude-opus-5
**Changes**: Added **§5 "Readiness-gated searches"** — the actual cause of §4
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
