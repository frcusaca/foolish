---
foop: 33
title: The Creation Postulate — ⬤, universal characterizations, and Booleans
author: Atlas hc.busy@gmail.com
status: Draft
type: Standards
created: 2026-07-07
phase: phase-4
supersedes: []
begun: [ ]
---

# FOOP-33: The Creation Postulate — ⬤, universal characterizations, and Booleans

## Abstract

This FOOP realizes the **Creation Postulate** (`docs/why/creation_postulate.md`) in the
UBCa. It adds four dependent features and stops, deliberately, short of boolean *operators*:

1. **`⬤` creation (U+2B24, or the ASCII alias `{*}`).** A new leaf that posits a genuinely
   new, unique value, born `Independent`. Two creations are equal iff they are the *same rust
   object* (`Rc::ptr_eq`) — no id, no registry; a creation is only ever shared (constanic
   clone returns the same object), never duplicated. Creations are reached only by search.
2. **Default equality, used by search.** There is no `==` operator. This FOOP defines the
   **default equality** of the assignment `=` as an implementation primitive — a function
   over two FIRs returning a Rust `bool` — and has the value-search matcher (`?=`/`~=`,
   FOOP-23) *call* that primitive. Same integer ⇒ equal; same creation ⇒ equal; NK ⇒ never
   equal, even to itself.
3. **Universal characterizations.** Characterizations (`a'b'c'name`) — already parsed but
   discarded — become first-class, carried on every statement via an **`Identifier`** struct
   that owns the one whitespace-stripped LHS string and exposes `name()` (the bare coordinate
   name) and `characterized_name()` (the whole LHS); it contains a `Characterizations` that,
   for this FOOP, only reports whether the name is null-characterized. A search pattern that
   contains a `'` is matched by the matcher against `characterized_name()`; a plain pattern
   against `name()`. The empty characterization touching the name (a bare `'` immediately
   before the name) is the **null characterization**.
4. **`system.foo` as ancestral prelude.** A build-embedded `system.foo` becomes the **parent
   (ancestral) brane** of every program's root brane. It defines `'True` and `'False` as
   **null-characterized name constants**.

The crux is the **null-characterized name constant** rule: once a null-characterized name
is defined, it may only be re-defined to an equal value; any other re-definition refuses and
turns **NK**, poisoning further use of that name. This rule is enforced in two places — the
brane step (a brane checks its ancestors) and concatenation (a merge checks the statements
merged before it) — so that `'True` in `system.foo` cannot be silently overwritten by a
Foolisher writing `'True=3`.

Boolean *operators* (`⊦`, `not`, `and`, `or`) and their assertion tests are explicitly out
of scope and are deferred to a follow-on FOOP.

## Motivation

`docs/why/creation_postulate.md` and `docs/vintage_legacy/CREATION.md` describe how Foolish
bootstraps every concept from nothing using `⬤`: the unknown, characterization, brane,
integer, float, string, and — the target of this FOOP — the booleans `True` and `False`.
Today none of this is expressible: the lexer, parser, and FIR know nothing about `⬤`, the
parser already recognizes the `'` characterization syntax but the compiler throws the
characterizations away, and there is no `system.foo` prelude and no notion of a protected
constant.

Booleans are the smallest genuinely useful concept the Creation Postulate can deliver.
`True` and `False` are not derived from `1` and `0`; they are independent ideas posited by
`⬤`, given names, and made resistant to accidental redefinition. To reach them we need
exactly the four features above — and no more. Boolean *behavior* (negation, conjunction,
disjunction, assertion) is a separate, larger design that this FOOP intentionally leaves to
a successor, keeping the present change "rather simple minded" and provably correct.

The world after this FOOP: a Foolisher can write `⬤` to create a new value, refer to it by
name, ask (via value search) whether two names denote the same creation, characterize any
LHS, search on characterization+name, and rely on `True`/`False` being defined and
un-clobberable in every program.

## Specification

### 1. The creation dot `⬤` (and its ASCII alias `{*}`)

**Grammar.** `⬤` (U+2B24) is a new primary expression, valid anywhere an expression
literal (`IntLit`, `UnknownLit`) is valid. Because `⬤` is hard to type, the spaceless
three-token sequence `{*}` is an **exact alias** for it. The alias is recognized at the
**parser**, not the lexer: `{`, `*`, `}` lex to their ordinary tokens (`LBrace`, `Mul`,
`RBrace`), and the parser recognizes the `LBrace Mul RBrace` sequence and emits **the same
`Astn::Creation`** node that the `⬤` token produces. This is unambiguous: `*` is not a valid
identifier or characterization name, so `{` immediately followed by `*` can never open a real
brane statement (see Gotcha #1). After parse, `{*}` and `⬤` are indistinguishable, and the
compiler's single `Astn::Creation → CreationFir` arm handles both.

```
creation   ::= "⬤" | "{" "*" "}"     (the {*} form has no interior whitespace)
primary    ::= int_lit | "???" | creation | identifier | brane | ...
```

Typical uses reproduce the bootstrap forms from `CREATION.md`:

```foolish
c    =  ⬤        !! a bare creation
b    = c'⬤       !! a creation characterized as c
d    = {*}       !! identical to  d = ⬤  (ASCII alias)
True = ⬤         !! (see §4 for the null-characterized 'True form)
```

**Sequencer output.** The humanizing sequencer **always** renders a creation as the Unicode
`⬤`, never as `{*}`. `{*}` is an input convenience only; the canonical output form is the
dot.

**Lexer / AST.** The lexer gains **one** new token — for `⬤` (U+2B24). It gets **no** `{*}`
handling: `{`, `*`, `}` keep lexing to `LBrace`, `Mul`, `RBrace`. The parser then produces a
new AST node from *both* the `⬤` token and the `LBrace Mul RBrace` sequence:

```rust
// foolish-parser/src/ast.rs — new Astn arm
Astn::Creation,
```

`⬤` carries no characterizations of its own; characterizations attach to the LHS (the
statement) or to a brane, exactly like any other RHS (§3).

**FIR.** A new `CreationFir` is the FIR for `⬤`, carrying **no id**:

```rust
// foolish-ubca/src/fir_kinds.rs
pub struct CreationFir {
    pub(crate) core: ProtoBrane, // that's all — no id field yet
}
```

A `CreationFir` is **born `Independent`** — it is a self-contained constant with no context
dependencies. It never searches, never recoordinates its value, and never regresses.

**Identity is the rust object; the only clone is the identity-preserving constanic clone.**
This FOOP does **not** give a creation a numeric id. A creation's identity is simply *which
rust `CreationFir` object it is*: two creations are the same creation iff they are the same
`Rc` (`Rc::ptr_eq` on the settled creation). There is no counter, no registry, no `HashMap`.
An explicit id is deferred until we need to **ship** a creation somewhere (serialize/transmit
across a boundary), at which point a future FOOP mints one; until then "same rust object" is
the whole identity story.

Because a creation is `Independent`, cloning it is trivial and **returns the same object**:

- **Constanic clone of a `CreationFir` → the same `Rc`** (identity-preserving; okay). The
  constanic-clone path, on an `Independent` creation, hands back the identical object rather
  than constructing a new `CreationFir`. This is consistent with an independent constant and
  keeps `Rc::ptr_eq` sound across the one clone path that exists.
- **Any *other* clone is forbidden.** No deep-copy / duplicate-`CreationFir` path may exist.
  If some path other than constanic clone would duplicate a creation, that is a bug to fix (or
  a design escalation — see Open Questions), not a silent second copy.

Consequently `x = ⬤ ; y = x` resolves `y`'s value — through the constanic clone — to the very
same `CreationFir` `Rc` as `x`'s, so `Rc::ptr_eq` holds and they compare equal (§2). Only a
fresh `⬤`/`{*}` literal is a new object and hence a new creation.

### 2. Default equality (`=`), used by search

There is **no equality operator**. Instead this FOOP names the equality that the assignment
`=` already implies as a reusable **default-equality primitive**, and routes the value-search
matcher through it. The primitive is an implementation-level function over two settled FIRs:

```rust
// foolish-ubca — the default equality of `=`, as a Rust boolean
pub(crate) fn default_equal(a: &FirRef, b: &FirRef) -> bool;
```

Its rules, for two constanic FIRs `a` and `b`:

1. **NK guard.** If either is `NK`, `default_equal` is `false` — even if they are the same
   rust object. (FOOP-23 stipulation; restated because it now interacts with identity.)
2. **Integer equality.** If both expose `as_i64()`, `true` iff the integers are equal.
3. **Referential (creation) equality.** Otherwise, if both are creations, `true` iff they are
   the **same rust object** — `Rc::ptr_eq` on the settled creation. (No id is involved; §1
   guarantees a creation is only ever shared, never duplicated, so `Rc::ptr_eq` is sound.)
   This is the FOOP-23 stipulation — *"if the rhs `get_value()` is a fir that is the same fir
   in the ubca fvm as a candidate, then it is equal"* — now implemented for creations.
4. **Otherwise.** `false` (e.g. two distinct branes — brane equivalence remains unspecified
   per FOOP-23). A `false` here means "not known equal," not an assertion of deep inequality.

Equality is observed only through a **value search** (`?=` / `~=`, and their
contexted/combined forms — FOOP-23). The value-search matcher
(`SearchPredicate::Value` / `NameValue` in `foolish-ubca/src/fir_kinds.rs`) today compares
candidate and pattern **inline**, only through `as_i64()`. This FOOP **refactors** that inline
comparison out into `default_equal` and has the matcher *call* it: `Value`/`NameValue` approve
a candidate iff `default_equal(candidate_value, pattern_value)`. The matcher no longer knows
the equality rules — it delegates. This keeps equality defined in exactly one place, reusable
by the null-constant rule (§4) as well.

This is the mechanism the null-constant rule (§4) uses to distinguish a harmless re-statement
(`'True='True`, same creation) from a conflicting redefinition (`'True=3`, different value).

### 3. Universal characterizations

**Parsing (already present).** `foolish-parser` already parses a characterization stack on
assignments, identifiers, and branes: `a'b'c'name` yields the characterizations `a`, `b`, `c`
and name `name`. The **null characterization** is a bare `'` immediately before the name:
`a'b'c''name` is `name` characterized by `a`, `b`, `c`, and the empty (null) characterization
touching the name. RHS characterizations are likewise parsed (`coordinate = special'{...}`).

**Proximity is king.** Only the characterization slot **immediately touching the name** makes
a *null-characterized coordinate name*. An interior empty like `a''b'name` is a null
characterization applied to `b'name` (the concept `b'name`), **not** to `name` — so
`a''b'name` is *not* a null-characterized coordinate name. This positional rule is the whole
contract; it resolves what would otherwise be ambiguous.

**The `Identifier` struct (owns the LHS), containing a `Characterizations`.** Each statement
owns exactly one `Identifier`. The `Identifier` holds the **one authoritative owned string** —
the entire LHS with all internal/surrounding whitespace stripped — and describes its parts as
**spans** into that one string (single allocation per statement). It contains a
`Characterizations` (the front portion) and a `name` span (the coordinate name at the tail).

For LHS `a' b'c'd'e''x` (note the source space after the first `'`), the `Identifier` string
is `"a'b'c'd'e''x"` (whitespace stripped), `name` is the `"x"` span, and `Characterizations`
covers the `"a'b'c'd'e''"` front span.

```rust
// Each StatementFir owns one Identifier. `text` is the sole allocation; everything else spans it.
pub struct Identifier {
    text: String,                 // whole LHS, whitespace-stripped, e.g. "a'b'c'd'e''x"
    name: Range<usize>,           // the coordinate-name span, e.g. the "x" at the tail
    characterizations: Characterizations, // reports on the front span (spans into `text`)
}

impl Identifier {
    /// The bare coordinate name — e.g. "x". The matcher demands this for a plain pattern.
    pub fn name(&self) -> &str { &self.text[self.name.clone()] }

    /// The full characterized name — the whole `text`, e.g. "a'b'c'd'e''x". The matcher
    /// demands this for a `'`-bearing pattern.
    pub fn characterized_name(&self) -> &str { &self.text }

    /// Delegates to the contained Characterizations (below).
    pub fn is_nully_characterizing_coordinate_name(&self) -> bool {
        self.characterizations.is_nully_characterizing_coordinate_name()
    }
}
```

**`Characterizations` — minimal for this FOOP.** `Characterizations` describes the
characterization front span within the `Identifier`'s string. **For this FOOP it does not yet
parse the individual `'`-separated components** — it only needs to answer whether the slot
immediately touching the name is null:

```rust
pub struct Characterizations { /* span(s) into Identifier's `text`; details are impl choice */ }

impl Characterizations {
    /// True iff the characterization slot **immediately touching the name** is null (empty) —
    /// i.e. this is a null-characterized coordinate name (a constant name). Proximity is
    /// king: an interior empty does NOT count. This is the ONE thing the engine reads, used by
    /// the null-constant rule (§4) and the descendant query. Per-`'` component extraction
    /// (`get_characterizations`) is deferred to a future FOOP.
    pub fn is_nully_characterizing_coordinate_name(&self) -> bool { /* … */ }
}
```

**Threading into the FIR.** `StatementFir` replaces its bare `name: String` with the
`Identifier`:

```rust
pub struct StatementFir {
    pub(crate) core: ProtoBrane,
    pub(crate) identifier: Identifier, // owns the LHS string, name span, and Characterizations
    pub(crate) line_number: usize,
}
// StatementFir::name() now delegates to identifier.name()
```

The compiler, which currently reads only `Astn::Assignment.identifier` and discards
`characterizations`, now builds the `Identifier` from the parsed name + characterizations
(stripping whitespace). `BraneFir`'s brane-level characterizations (today a `Vec<String>`)
migrate to a `Characterizations` as well.

**Storage, not use — except search. The matcher picks the domain.** The identifier is *stored*
and otherwise inert. The one place it participates is **search matching**, and the choice of
which projection to match against lives in the matcher:

- A search pattern **without** a `'` → the matcher demands `Identifier::name()` (the bare
  coordinate name). Unchanged from today.
- A search pattern **containing** a `'` → the matcher demands `Identifier::characterized_name()`
  (the whole LHS). For a statement `a'b'x=3`, that is `a'b'x`; so `?a'b'x` finds it and `?x`
  does not (the plain-name pattern is matched against `name()`, which is just `x`).

This makes a null-characterized coordinate name addressable specifically: `'True` (a pattern
with a leading null `'`) is matched against `characterized_name()`, which for the constant is
exactly `'True`, and does **not** match a plainly-named `True` (whose `characterized_name()`
is just `True`).

### 4. `system.foo` as ancestral prelude, and null-characterized name constants

**`system.foo` — repo-root `system/`, packaged into the crate.** The source file lives at a
dedicated **`system/` folder at the repository root** (`system/system.foo`), authored as an
ordinary `.foo` artifact. Because that folder sits *outside* the `foolish-ubca` crate
directory, a plain in-crate `include_str!` cannot both point at the root file *and* have Cargo
package it. So `foolish-ubca` gains a **`build.rs`** that, at build time, copies
`system/system.foo` into the crate's build-output directory; the crate then embeds it with a
**compile-time** `include_str!(concat!(env!("OUT_DIR"), "/system.foo"))`.

> **@human — on `OUT_DIR`.** `OUT_DIR` is the standard Cargo variable (not `RESOURCE_PATH`,
> which is a different ecosystem's convention). Cargo sets `OUT_DIR` *only while running
> `build.rs`*, pointing at a per-crate build scratch dir under `target/.../out/`. The
> `env!("OUT_DIR")` **macro** reads it *at compile time* and `include_str!` bakes the file's
> **contents** into the binary then and there. So `OUT_DIR` is **not** needed — and **not
> available** — at runtime; the program never touches the filesystem for `system.foo`. (Only
> `std::env::var("OUT_DIR")` *at runtime* would fail, and we do not do that.) Net effect:
> `system.foo` ships inside the compiled crate, no runtime file dependency, authored at the
> repo root.

Before the FVM steps a program, `system.foo` is compiled to a brane and installed as the
**parent (ancestral) brane** of the program's root brane. Name resolution therefore falls
through to `system.foo` via the existing `_ab_search` machinery: `True`, `False` (and any
future prelude names) resolve ancestrally without any concatenation into the user's root.

`system.foo` for this FOOP defines the booleans as null-characterized constants:

```foolish
{!!system.foo
    'True  = ⬤    !! True is a new, unique idea — a null-characterized name constant
    'False = ⬤    !! False likewise
}
```

**Null-characterized name constant rule.** A **null-characterized coordinate name** (a
statement whose `Characterizations::is_nully_characterizing_coordinate_name()` is true) is a
**constant name**: it may be re-defined only to an **equal value** (by `default_equal`, §2);
any other re-definition **refuses and becomes NK**, and that NK poisons further use of the
name.

Formally, when a brane is in `PREMBRYONIC`/`EMBRYONIC` and is settling a statement whose
coordinate name is null-characterized:

1. Ask the ancestors (walk the AB chain, into `system.foo`) whether a null-characterized
   statement of the **same name** was previously defined.
2. If none: proceed normally; this brane now *owns* that null-characterized name and will
   answer descendants' queries about it (below).
3. If one exists: compare values with `default_equal` (§2).
   - **Equal** (e.g. `'True='True`, the same creation): permitted, no-op-consistent.
   - **Unequal** (e.g. `'True=3`): **refuse** — the offending statement's body becomes
     **NK**. The name is thereby poisoned; subsequent searches that resolve to it inherit
     the NK.

**Descendant query.** A brane that owns a null-characterized name responds to descendant
branes' question *"is this name a null-characterized coordinate name (a constant) here?"* This
is the query step 1 issues while walking ancestors. It is a read-only structural query
answered from the brane's stored `Characterizations`.

**Concatenation must handle it too.** Concatenation (`ConcatenationFir`) currently merges the
statements of its operands by blind clone, with no collision detection. It must be built to
enforce the same rule against the statements merged **before** a given statement in the
merged sequence. Concretely, for `{A={'a=1}, B = A A A}` the concatenation `A A A` produces
three `'a` statements; the first `'a` establishes the constant, and each later `'a` is
checked against it:

- `'a=1` following `'a=1` (equal value): permitted.
- `'a=<anything else>` (unequal): the later `'a` becomes **NK**.

So most of the `'a`'s in `B` become NK (all but the first, since they would each be a
conflicting redefinition of the established constant — unless their values are equal by
`default_equal`). The check reuses `default_equal` and mirrors the brane-step rule exactly:
**one rule, two trigger sites** (brane step, concatenation merge).

## FIR Impact

- **New variant `CreationFir`** (`foolish-ubca/src/fir_kinds.rs`): `{ core }` — **no id**,
  born `Independent`. Identity is the rust object (`Rc::ptr_eq`); **no counter, no registry**.
  Constanic clone returns the *same* `Rc` (identity-preserving); **any other clone path is
  forbidden**. A corresponding core-fir representation (in `foolish-core/src/fir.rs`) is added
  so the sequencer can render a creation. YAML/JSON shape: `{ kind: Creation }` (an `id` is
  added by a future FOOP only when a creation must be shipped across a boundary).
- **`StatementFir` replaces `name: String` with an `Identifier`.** The `Identifier` owns the
  whitespace-stripped LHS string; `StatementFir::name()` delegates to `Identifier::name()`.
  Serialization carries the LHS text; a plain name (no `'`) round-trips as before.
- **New type `Identifier`** — each statement owns one; the sole per-statement allocation for
  the LHS. Fields: owned `text`, `name` span, a contained `Characterizations`. Methods:
  `name()` (bare coordinate name), `characterized_name()` (whole LHS),
  `is_nully_characterizing_coordinate_name()` (delegates).
- **New type `Characterizations`** — spans the characterization front portion of an
  `Identifier`'s string. **Minimal for this FOOP**: only
  `is_nully_characterizing_coordinate_name()` is implemented; per-`'` component extraction is
  deferred to a future FOOP.
- **`BraneFir.characterizations`** migrates from `Vec<String>` to `Characterizations`
  (the sequencer's brane-characterization rendering in `foolish-core/src/sequencer.rs`
  continues to emit the trailing-`'` form).
- **New equality primitive `default_equal(&FirRef, &FirRef) -> bool`** — the meaning of the
  default `=` (§2), the single home for the NK/integer/creation-identity rules.
- **NYES.** No new NYES states. `CreationFir` is terminal `Independent` from birth. The
  null-constant refusal produces `NK` on the offending statement body via the ordinary NK
  path (no new state). A new `*_nyes_transitions` unit test (`creation_nyes_transitions`) is
  REQUIRED (a single-state progression: `Independent` from the start), per AGENTS.md.

## UBC Step Impact

- **`CreationFir::fir_op_step`**: trivial — already `Independent`; no transitions.
- **Equality refactor**: extract the inline value comparison currently living in
  `SearchPredicate::Value` / `NameValue` into `default_equal` (§2), then have those predicates
  *call* it. Before: the matcher compared `as_i64()` inline and `Reject`ed everything else.
  After: the matcher approves iff `default_equal` returns true, and `default_equal` owns the
  NK/integer/creation-identity rules.
- **Name-search matcher** (`SearchFir::matches_pattern` and the `Name` predicate): the matcher
  chooses the projection — when the pattern contains `'`, match against the candidate's
  `Identifier::characterized_name()`; otherwise against `Identifier::name()` (§3). Patterns
  without `'` are unchanged.
- **`BraneFir` step (PREMBRYONIC/EMBRYONIC)**: add the ancestral null-characterized-name
  check (§4) — for each null-characterized statement, query ancestors; refuse→NK on unequal
  redefinition; register ownership; answer descendant queries.
- **`ConcatenationFir` step (Braning merge)**: replace the blind statement-clone loop with a
  collision-aware merge that applies the null-constant rule against already-merged statements
  (§4), NK-ing conflicting later duplicates.
- **Evaluator setup** (`foolish-ubca/src/evaluator.rs::evaluate`): compile the embedded
  `system.foo` once, and wire it as the AB parent of each compiled program root brane before
  `step_to_settled`. (Today each root brane self-roots via `new_cyclic`; this changes the
  parent wiring.)

## Gotchas & Exceptions (read before implementing)

These are the traps a coding agent (or reviewing human) will hit. Each is verified against the
current code.

1. **`{*}` is recognized at the PARSER, not the lexer — and it does not collide.** `{`/`*`/`}`
   keep their ordinary tokens (`LBrace`/`Mul`/`RBrace`); the parser recognizes the
   `LBrace Mul RBrace` sequence at brane-open and emits `Astn::Creation` (identical to `⬤`).
   There is **no ambiguity to guard against**: inside a brane body the parser expects a
   statement, i.e. an identifier / characterization name (`is_assignment_start` only accepts
   `Token::Ident`, `parser.rs:249`) or an expression. `*` (`Token::Mul`) is **not** a valid
   identifier or characterization name, so `{` immediately followed by `*` can never begin a
   real brane statement — the creation form is unambiguous. (Absent this rule, `{*}` would fall
   to `parse_expr`, hit the unary-`*` prefix path (`parser.rs:560`), and *error* for lack of an
   operand — further confirming `*`-at-name-position has no legitimate brane meaning.) The alias
   is the spaceless three-token run only; `{ * }`, `{}`, `{ *}`, and a brane that legitimately
   *contains* `*` in expression position keep their existing meaning. This is a *parser* test,
   not a lexer test.

2. **`constanic_clone_at` ALREADY returns the same `Rc` for `Independent` non-branes.** At
   `foolish-ubca/src/fir_kinds.rs:180-185`, the constanic-clone path returns `Rc::clone(fir_ref)`
   when `nyes ∈ {Constant, Independent}` and the kind is not `Brane`. A `CreationFir` is born
   `Independent` and is not a brane, so it hits this branch automatically — **no special case
   is needed** to make constanic clone identity-preserving. The clone-discipline work is
   therefore a *verify-and-test* task (confirm the creation reaches this branch and that
   `Rc::ptr_eq` holds after `x = ⬤ ; y = x`), plus **not** deriving/implementing a deep `Clone`
   on `CreationFir` that some other path could invoke. Do not add a `match` arm for
   `FirKind::Creation` in `constanic_clone_at` that constructs a new `CreationFir` — that would
   *break* identity.

3. **Name matching is regex, and `'` flows through it.** `SearchFir::matches_pattern`
   (`fir_kinds.rs:830`) compiles the pattern as a regex (`^pat$` unless it already contains
   `^`/`$`). The apostrophe is regex-neutral, so feeding `characterized_name()` (e.g. `a'b'x`)
   through the regex is fine — but any regex-special character appearing in a characterization
   would be interpreted as regex. For this FOOP the prelude names (`True`/`False`) and test
   names are regex-safe; note the general hazard and add a test with a plain characterization.
   Also: the bare-identifier compile path wraps patterns as `^{id}$` (`compiler.rs:119`), so the
   `'` must survive from parse into the pattern string for `'True` to match — confirm the
   parser carries the apostrophe into the search pattern, not just into the `Identifier`.

4. **The value-search matcher currently `unreachable!()`s on a pre-constanic body** (the
   `Value`/`NameValue` arms in `fir_kinds.rs`). `default_equal` operates on **settled** FIRs;
   keep the "body must be constanic before comparison" contract intact when refactoring — don't
   call `default_equal` on an un-settled candidate.

5. **NK poisoning must not loop.** When a null-constant conflict turns a statement `NK`, later
   references resolving to it inherit `NK` (good). Ensure the ancestral check itself does not
   re-trigger on the now-NK statement in a way that re-alarms every step — the refusal is a
   one-time settle to `NK`, then terminal.

6. **`system.foo` is itself subject to its own rules.** Because `system.foo` becomes an
   ancestor, defining `'True` there means a user brane defining `'True` again must compare
   equal or go `NK`. Confirm `system.foo`'s *own* internal consistency first (it must not
   self-conflict) and that installing it as parent does not accidentally make the user's first
   legitimate reference look like a redefinition.

7. **Parent-wiring change touches a `new_cyclic` invariant.** Root branes self-root today via
   `Rc::new_cyclic` (`evaluator.rs`). Installing `system.foo` as the AB parent means the root's
   parent `Weak` must point at the system brane instead of itself. Verify `_ab_search`
   terminates (the system brane's own parent should be a clean sentinel, not a cycle back into
   the program).

## Test Plan

Tests first, per project rules. Unit tests pin internal FVM state (identity, subspans, NYES);
approval tests pin observable behavior byte-for-byte; the comprehensive weaves it all together.

### Unit tests (Rust — internal state)

**Parser / lexer** (`foolish-parser`):
- `parses_star_brane_as_creation`: `⬤` (lexer token) and the `LBrace Mul RBrace` sequence
  `{*}` both parse to `Astn::Creation` — *and its negatives*: `{ * }`, `{}`, `{ *}`, `{* }`,
  and a brane that legitimately contains `*` in expression position (e.g. `{y = 2 * x}`) do
  **not** become creations (they keep their existing parse). This negative set is the real
  test; the positive case is easy. Recognition is collision-free because `*` is never a valid
  name at the brane-statement position.
- Characterization stack survives to the AST for both LHS (`a'b'c'name`) and the null form
  (`a'b'c''name`, bare `'name`), and the `'` reaches the *search pattern* for `?'True`.

**`Identifier` / `Characterizations`** (pure, no FVM):
- Whitespace stripping: LHS `a' b'c'd'e''x` builds an `Identifier` whose `text` is
  `"a'b'c'd'e''x"`, `name()` is `"x"`, `characterized_name()` is `"a'b'c'd'e''x"`.
- `name()` and `characterized_name()` return **subspans** of the one owned `text` — assert
  `characterized_name()` for a plain name (no `'`) equals `name()` and that both borrow `text`
  (a `&str`-into-buffer / no-fresh-allocation check).
- `is_nully_characterizing_coordinate_name()`: **true** for `a'b'c''name` and bare `'name`;
  **false** for plain `name`, for `a'b'c'name`, and — the key case — for interior-null
  `a''b'name` (proximity rule).
- Plain name (no `'`) → `is_nully_characterizing_coordinate_name()` is false and
  `characterized_name() == name()`.

**`CreationFir` / NYES**:
- `creation_nyes_transitions` — `Independent` at every step (single-state progression via
  `assert_progression`), per the AGENTS.md `*_nyes_transitions` requirement.
- **Identity is preserved through constanic clone**: build a `CreationFir`, run it through
  `ProtoBrane::constanic_clone_at`, assert `Rc::ptr_eq(original, clone)` (this pins Gotcha #2 —
  the existing `fir_kinds.rs:180` branch). A regression here would silently break equality.

**`default_equal`** (the equality truth table, in isolation):
- same `IndepInt` value ⇒ true; different ⇒ false.
- same creation `Rc` ⇒ true; two *distinct* `⬤` creations ⇒ false.
- NK vs NK (even the same `Rc`) ⇒ **false** (explicit — this is the subtle one).
- creation vs integer ⇒ false; two distinct branes ⇒ false.
- Then: `SearchPredicate::Value`/`NameValue` **delegates** — same creation ⇒ `Approve`,
  distinct ⇒ `Reject`, integer paths unchanged (guards against the refactor changing behavior).

**Null-constant rule** (build FIR via the parser + `.search()` per the unit-test infra):
- Ancestor defines `'k=⬤`; descendant `'k=<the same creation via reference>` ⇒ permitted.
- Ancestor `'k=1`; descendant `'k=2` ⇒ descendant body settles **NK**; a following reference to
  `k` inherits NK.
- Descendant query returns true for a name an ancestor null-characterized, false otherwise.
- **Non-null names are unaffected**: `k=1` then `k=2` (no leading `'`) is *not* refused —
  regression guard that the rule only fires on null-characterized names.

**Concatenation collision** (`ConcatenationFir`):
- `{A={'a=1}, B = A A A}` → in `B`, first `'a` intact, later `'a`s **NK**.
- `{A={'a=⬤}, B = A A}` where both `'a`s are the *same* creation ⇒ both permitted (equal by
  `default_equal`) — proves the rule is value-sensitive, not "duplicate name = NK".
- Empty/single-operand concat still merges without spurious collisions.

### Approval tests (`.foo` → insta snapshots; human-signed)

One focused input per behavior (small, legible, full-width-space indentation), plus negatives:

- `creation_basic.foo` — `x=⬤`; `y=x`; `z=⬤`; a value search showing `y` equals `x` and `z`
  does not. Confirms identity + inequality of distinct creations in observable output.
- `creation_ascii_alias.foo` — a `{*}` input; snapshot must render `⬤` (pins Gotcha #1 output
  side and the sequencer rule).
- `characterization_search.foo` — a brane with `a'b'x=3` (and a decoy plain `x=9`): `?a'b'x`
  finds the `3`; `?x` finds the `9` (or the plain one), demonstrating the two match domains.
- `null_char_addressing.foo` — `'True` vs a plain `True` in the same brane; `?'True` hits only
  the null-characterized one.
- `system_prelude.foo` — a program that simply references `True`/`False`, resolving ancestrally
  into `system.foo` with no local definition.
- `null_const_permit.foo` — `'True='True` (or the same creation) is permitted (no NK).
- `null_const_refuse.foo` — `'True=3` settles NK and poisons subsequent `True` use.
- `concat_null_collision.foo` — the `{A={'a=1}, B=A A A}` case, so the NK-of-later-duplicates
  is visible in a signed snapshot.

**Existing snapshots to re-review (expect diffs):** any snapshot whose brane-characterization
rendering shifts under the `Characterizations` migration, and any program that now has
`system.foo` as an ancestor (name resolution that previously missed may now resolve, or step
counts may change). Treat every such diff as a *semantic* review, not a formatting rubber-stamp
(per AGENTS.md ⚠️). The compiler unit test asserting characterizations are discarded must be
updated to assert they are *threaded*.

### Comprehensive — `foop_33_comprehensive.foo`

The reserved single input that exercises every new surface interacting with old features. Build
it to touch, at minimum, one path through each of: creation (`⬤` and `{*}`), referential
equality via value search (equal and unequal), quote-bearing characterization search, a
null-characterized constant defined ancestrally (`True`), the `system.foo` parent-brane
fall-through, a null-constant **refusal→NK**, the concatenation collision (`A A A`), and — for
old-feature interaction — a **contexted `&`-search** (FOOP-23) landing on a creation-valued
statement and a **nested brane** whose inner search reaches an ancestral null-characterized
name. Sketch:

```foolish
{!! foop_33_comprehensive.foo
    yes    = True            !! ancestral null-const resolves via system.foo
    a      = ⬤               !! a fresh creation
    b      = a               !! same creation as a
    c      = {*}             !! a different fresh creation (ASCII alias)
    same   = ?=a             !! value search: finds b (== a), not c
    tag'x  = 7               !! characterized coordinate name
    hit    = ?tag'x          !! quote-bearing search matches char+name
    miss   = ?x              !! plain-name search does NOT match tag'x
    grp    = {'k=a, 'k=a}    !! concat/dup null-const with EQUAL value → permitted
    bad    = {'k=a, 'k=c}    !! dup null-const with UNEQUAL value → second 'k is NK
    'True  = 3               !! REFUSED → NK (conflicts with ancestral 'True)
    {!! nested
        deep = True          !! ancestral resolution from a nested brane
    }
}
```

The snapshot captures the settled result, every alarm, and the step count. Final approval is
human-signed; the agent generates and verifies the `.snap.new` but never accepts it.

If any part cannot be cleanly tested it is called out in Open Questions.

## Rejected Alternatives

### A. A global creation registry (`HashMap<id, FirRef>`)

Keep a literal global map from creation id to the `CreationFir` (and mint an id per `⬤`).
**Rejected** after consideration: creations are reached only via search, so nothing needs to
enumerate them by id; the map would be a global mutable structure with lifetime/threading cost
for no consumer; and no id is needed at all while a creation lives in-process. Identity is
just *which rust object it is* — `Rc::ptr_eq` — which is all `default_equal` needs. An id is
deferred to a future FOOP, added only when a creation must be shipped across a boundary.

### B. An `==` equality operator (equality as a matcher-internal rule)

Add a first-class `==`, or leave equality as inline logic inside the value-search matcher.
**Rejected**: there is no equality operator — equality matters only during search — and
equality should be a *named primitive* (`default_equal`) that the matcher calls, not a rule
buried in the matcher (nor duplicated by an operator). Making `default_equal` the single home
lets the null-constant rule (§4) reuse the exact same equality. A first-class `==` would also
pull in the unspecified brane-equivalence theory. Deferred / inverted.

### C. Boolean operators in scope (`⊦`, `not`, `and`, `or`)

Deliver the full boolean algebra from `CREATION.md`. **Rejected** as too large and
under-specified (how a created symbol like `and` acquires *operational* behavior via
assertion is a substantial design). This FOOP stops at `True`/`False`; operators are a
follow-on FOOP.

### D. `system.foo` concatenated into root (not parent)

The earlier plan concatenated `system.foo` into the program root. **Rejected** in favor of
making `system.foo` the ancestral parent brane, because the null-constant rule already
"verifies with its ancestors": ancestral parenting makes `True`/`False` genuine ancestral
constants a program cannot shadow, and reuses `_ab_search` fall-through rather than a physical
merge. (Concatenation still exists and still enforces the null-constant rule for
Foolisher-written concatenations — a separate, general concern.)

## Open Questions

- Whether the REPL and `step`/`run` CLI paths all install `system.foo` uniformly (the path is
  decided: repo-root `system/system.foo`, embedded via `build.rs` → `OUT_DIR`; see §4).
- Snapshot/sequencer surface for a creation: how `⬤` and a creation value render in
  `hssnap` output (a stable, human-legible form is needed before approval). The input `{*}`
  alias is decided (always renders back as `⬤`); the value form is not.
- Whether `Characterizations` should store `original` as an owned `String` per statement, or
  borrow from a shared parse buffer. This FOOP specifies owned-per-statement for simplicity;
  revisit only if profiling shows the copies matter.

## References

- Prior FOOPs: FOOP-23 (value search, `FoolRefFir`, the referential-equality stipulation now
  implemented here), FOOP-62 (two-store ProtoBrane, NYES).
- Philosophy: `docs/why/creation_postulate.md`, `docs/vintage_legacy/CREATION.md`.
- Terminology: AGENTS.md §Searches, §Foolish Terminology; `docs/vintage_legacy/STYLES.md`.
- Code locations: `foolish-parser/src/{lexer.rs,parser.rs,ast.rs}` (creation token, `⬤` AST,
  characterization parsing); `foolish-ubca/src/fir_kinds.rs` (`CreationFir`, `Identifier`,
  `Characterizations`, `StatementFir`, `BraneFir`, `ConcatenationFir`, `SearchPredicate`,
  `constanic_clone_at:180`); `foolish-ubca/src/compiler.rs` (`Identifier` build);
  `foolish-ubca/src/evaluator.rs` (`system.foo` install); `foolish-core/src/{fir.rs,sequencer.rs}`
  (core-fir + rendering).

## Last Updated

**Date**: 2026-07-07
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.8
**Changes (round 6, per Atlas)**: `{*}` alias now recognized at the **parser**, not the lexer:
`{`/`*`/`}` keep their ordinary tokens, and the parser emits `Astn::Creation` from the
`LBrace Mul RBrace` sequence (same node as `⬤`); the compiler's single `Astn::Creation` arm
handles both. Recognition is **collision-free** — `*` is not a valid identifier/characterization
name (`is_assignment_start` accepts only `Token::Ident`), so `{*}` can never be a real brane
statement; no ordering guard against the unary-`*` path is needed. Rewrote Gotcha #1 to state
this, named the parser test `parses_star_brane_as_creation` (with the negative set incl.
`{y = 2 * x}`). Updated §1 grammar note, Test Plan, plan Phase 2.
**Changes (round 5, per Atlas)**: Introduced an **`Identifier`** struct that each statement
owns (owns the one whitespace-stripped LHS string; has a `name` span and a contained
`Characterizations`; exposes `name()` and `characterized_name()`). The **matcher chooses** the
projection — `'`-bearing pattern → `characterized_name()`, plain → `name()`. `Characterizations`
is now **minimal** for this FOOP: only `is_nully_characterizing_coordinate_name()` (per-`'`
split deferred). `StatementFir` holds an `Identifier` instead of `name: String`. Updated
Abstract §3, FIR/Step Impact, Gotcha #3, Test Plan, References, and plan Phase 1 accordingly.
**Changes (round 4, editorial + test depth)**: Added a **Gotchas & Exceptions** section (7
verified traps: `{*}` lex-lookahead vs brane syntax; `constanic_clone_at:180` already returns
the same `Rc` for `Independent` non-branes so creation identity is automatic — do NOT add a
`FirKind::Creation` clone arm; regex name-matching and `'` flow-through; the value-matcher
`unreachable!` on pre-constanic bodies; NK-poison non-looping; `system.foo` self-consistency;
`new_cyclic` parent-rewiring / `_ab_search` termination). Substantially expanded the Test Plan
(unit truth tables, subspan/no-alloc assertions, the `creation_constanic_clone_preserves_identity`
test that pins the `:180` behavior, value-sensitive concat cases) and wrote a concrete
`foop_33_comprehensive.foo` sketch.
**Changes (round 3, per Atlas)**: (1) `CreationFir` carries **no id** — identity is the rust
object (`Rc::ptr_eq`); an id is deferred to a future FOOP for shipping across a boundary. No
counter, no registry. (2) Clone discipline: constanic clone of a creation returns the *same*
`Rc` (identity-preserving, okay because `Independent`); any other clone is forbidden. (3)
`system.foo` lives in a repo-root **`system/`** folder and is packaged into the crate via a
`build.rs` that copies it into `OUT_DIR` + compile-time `include_str!` (with a precise `@human`
note that `OUT_DIR` is Cargo-standard and compile-time only, not runtime, not `RESOURCE_PATH`).
(4) Method renamed to `is_nully_characterizing_coordinate_name`.
**Round 2**: `{*}` ASCII alias for `⬤` (both → one `Astn::Creation`; sequencer always renders
`⬤`); equality inverted into a named `default_equal(&FirRef,&FirRef)->bool` primitive the
matcher *calls*; `Characterizations` keeps one owned string with `name`/`chars` as subspans;
null method scoped to the name-adjacent slot only (proximity rule).
**Initial draft**: ⬤ creation, referential equality via value search, universal
characterizations, null-characterized name constants, `system.foo` ancestral prelude, refusal
rule at brane step and concatenation.
