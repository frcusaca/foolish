---
foop: D55
title: Project Euler 1 — the 'mod modulo operator, the 'or boolean operator, and the repairs that run the first exercise
author: Sisyphus / qwen3.8-max (directed by Atlas)
status: Draft
type: Standards
created: 2026-08-07
phase: phase-4
supersedes: []
begun: [ ]
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
exactly two language features the exercise needs and no more —

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

### §2. `'or` — boolean OR as a pure-Foolish truth-table search (FOOP-73's preferred design, for `'or` only)

**No new FIR kind. No privileged FVM layer.** `'or` is defined in
`system.foo` as an ordinary Foolish brane holding a truth table, applied by
ordinary search — FOOP-73 §"Preferred design". FIR impact: NONE (that is
the point).

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

## FIR Impact

- **One new FIR kind** in `foolish-ubca/src/system_foo.rs` — `ModuloFir` or
  `SystemArithFir { core, op: ArithOp, self_weak }` (§1; enum shape
  recommended). New `FirKind` arm; new `constanic_clone_at` arm performing
  recoordination; display name `'mod`. YAML/JSON shape:
  `{ kind: Modulo }` (or `{ kind: SystemArith, op: Mod }` to match).
- **`system.foo` grows**: `'mod = ⬤` (placeholder, body overridden) and
  the `'or` truth-table brane (pure Foolish, no override).
- **No new NYES states.** `'mod` uses the three existing terminals
  (ECONSTANIC / CONSTANT / NK); `'or` adds none (pure Foolish).
- **No serialization impact** beyond sequencer rendering of the new kind.

## UBC Step Impact

- **New `fir_op_step`** for the modulo kind — the two-phase
  enqueue-then-combine shape of `ComparisonFir` (`system_foo.rs:326-340`),
  with the §1 `combine` rules.
- **`BodyOverride` hook generalized** from comparisons-only to a system
  name table covering `ComparisonOp::ALL` + `'mod`
  (`system_foo.rs:384-391`); still scoped to `system.foo`'s own top-level
  statements only.
- **No other step-rule changes.** `'or` evaluates entirely through
  existing machinery (concatenation, SFF index search, value search,
  contexted search, contexted index).

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

## Open Questions

- **Modulo with negative operands** — Rust truncating semantics are pinned
  for now; Euclidean modulo, if ever wanted, is a future FOOP.
- **The exercise-file fixes** — Atlas is applying E1/E2/E3; plan Phase 0
  verifies they landed before any implementation and does not edit the
  exercise itself.
- **MAX_DEPTH / iteration budget** for ~1000-deep recursion — measured in
  Phase 3 (§4 risk 1); the remedy, if any is needed, is decided there.
- **The exact final shape of the `'or` lookup** — `T~A=A &~B=B &#1` is
  traced on paper (§2 table); Phase 2 confirms it on the live FVM and
  adjusts only if the trace was wrong.
- **BodyOverride hook naming** — generalizing `comparison_body` to a system
  name table is an implementation detail left to the implementer.
- **The `=$`/`$=` sugars** — `$=` does not parse and `=$` settles
  non-obviously (D6); their intended semantics need their own
  investigation FOOP. FOOP-55 avoids both.

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

**Date**: 2026-08-07 (2)
**Updated By**: Sisyphus / qwen3.8-max
**Changes**: Post-FOOP-65-creation revision. Added the **FOOP-65
dependency** — the exercise rewrite uses backtick (tail-concatenator)
application: new E4 carries the full line-by-line rewrite mapping; §4
risk 0 and the plan's Phase 0 gate on FOOP-65 merged. Added **D6** (`$=`
does not parse; `=$` desugars to `BinaryOp{$, #-1, B}` and settles
non-obviously — repros) and **E5** (the six `$=` lines rewritten as
explicit `(X)$`); corrected the §3 sugar row accordingly. Earlier same
day: created (Draft) — bisected the exercise failure, specified `'mod`
(FOOP-33 §5.1 mechanism) and `'or` (FOOP-73 preferred design, `'or`
only), documented D1–D5 and E1–E3 incl. the INTERN_ decision.
