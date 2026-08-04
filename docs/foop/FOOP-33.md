---
foop: 33
title: The Creation Postulate — ⬤, universal characterizations, and Booleans
author: Atlas hc.busy@gmail.com
status: Implementing
type: Standards
created: 2026-07-07
phase: phase-4
supersedes: []
begun: [x]
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
   **default equality** of the assignment `=` as an implementation primitive — a three-valued
   `default_equal(a,b) -> Equality::{Equal, NotEqual, Unknowable}` — and has the value-search
   matcher (`?=`/`~=`, FOOP-23) *call* it as a **greedy "known-to-be-equal" matcher**. Only two
   integers or two creations are comparable (same integer / same rust object ⇒ `Equal`);
   **everything else is `Unknowable` → NK**, not a silent miss. The matcher approves on `Equal`,
   rejects on `NotEqual`, and stops-NK on `Unknowable`.
3. **Universal characterizations.** Characterizations (`a'b'c'name`) — already parsed but
   discarded — become first-class, carried on every statement via an **`Identifier`** struct
   that owns the one whitespace-stripped LHS string and exposes `name()` (the bare coordinate
   name) and `characterized_name()` (the whole LHS); it contains a `Characterizations` that,
   for this FOOP, only reports whether the name is null-characterized. A search pattern that
   contains a `'` is matched by the matcher against `characterized_name()`; a plain pattern
   against `name()`. The empty characterization touching the name (a bare `'` immediately
   before the name) is the **null characterization**.
4. **`system.foo` as ancestral prelude.** A build-embedded `system.foo` becomes the **root
   brane**, holding the user's program as a member named `program`; the program's root brane
   is therefore no longer its own parent, and name resolution falls through ancestrally into
   `system.foo`. It defines `'True` and `'False` as **null-characterized name constants**.
   The FVM steps the composite brane and returns the `program` member (see §4).
5. **Comparison operators (`<`, `>`, `<=`, `>=`).** ⛔ **DEFERRED — pending a new
   specification from the human.** The implementation committed for this section has been
   reverted; see the STOP gate at the head of Phase 6 in `FOOP-33.plan.md`. The description
   retained in §5 is a **historical record of a superseded design** and must not be
   implemented from. Boolean *values* (`'True`/`'False`, item 4) ship first and independently;
   the *producers* of those values await the new spec.

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
matcher through it. Equality has **three** outcomes — comparison of anything other than two
integers or two creations is **not knowable**, not merely "not equal" — so the primitive
returns a three-valued result, not a `bool`:

```rust
// foolish-ubca — the default equality of `=`
pub(crate) enum Equality { Equal, NotEqual, Unknowable }
pub(crate) fn default_equal(a: &FirRef, b: &FirRef) -> Equality;
```

Its rules, for two constanic FIRs `a` and `b`:

1. **NK guard.** If either is `NK`, the result is **`Unknowable`** — even if they are the same
   rust object. (FOOP-23: NKs are never equal to each other.)
2. **Integer equality.** If both expose `as_i64()`, `Equal` iff the integers are equal, else
   `NotEqual`.
3. **Referential (creation) equality.** If both are creations, `Equal` iff they are the **same
   rust object** (`Rc::ptr_eq` on the settled creation), else `NotEqual`. (No id is involved;
   §1 guarantees a creation is only ever shared, never duplicated, so `Rc::ptr_eq` is sound.)
   This implements the FOOP-23 stipulation — *"if the rhs `get_value()` is a fir that is the
   same fir in the ubca fvm as a candidate, then it is equal"* — for creations.
4. **Incomparable-kinds vs unknowable — revised (see "Problems Discovered During
   Implementation").** Distinguish two sub-cases the original Phase-3 wording wrongly merged:
   - **Different non-NK kinds where both are constanic** (brane vs integer, integer vs
     creation, brane vs creation) → **`NotEqual`**. A settled brane is *provably* never an
     integer (different FIR kinds, decidable); the search `Reject`s (skip) and continues —
     this is known-`NotEqual`, not unknowable. *(Original Phase-3 wording returned `Unknowable`
     here, which the matcher mapped to `NkStop`, aborting value searches on the first
     non-integer candidate — breaking FOOP-23. Revised.)*
   - **Either operand `NK`**, or **two branes** (brane-vs-brane equivalence unspecified,
     FOOP-23) → **`Unknowable`**. Here "unknowable" is honest.

Equality is observed only through a **value search** (`?=` / `~=`, and their
contexted/combined forms — FOOP-23). The value-search matcher
(`SearchPredicate::Value` / `NameValue` in `foolish-ubca/src/fir_kinds.rs`) today compares
candidate and pattern **inline**, only through `as_i64()`. This FOOP **refactors** that inline
comparison out into `default_equal` and has the matcher *call* it, mapping the three outcomes
onto the matcher's existing `MatchOutcome`:

| `default_equal` | `MatchOutcome` |
|-----------------|----------------|
| `Equal`         | `Approve`      |
| `NotEqual`      | `Reject`       |
| `Unknowable`    | `NkStop`       |

The matcher no longer knows the equality rules — it delegates. This keeps equality defined in
exactly one place, reusable by the null-constant rule (§4) as well.

**The value-search matcher is a greedy "known-to-be-equal" matcher.** It approves only on
`Equal` — a *positive proof* that the two values are the same — never on "can't tell." `Equal`
matches; `NotEqual` rejects and the scan continues; `Unknowable` halts the scan with NK — but
`Unknowable` is now reserved (per revised rule 4) for the genuinely unknowable cases: an `NK`
operand, or two branes whose equivalence is unspecified. A provably-different-kind candidate
(brane vs integer) is `NotEqual` and is *skipped*, not `NkStop`ped — this restores FOOP-23's
"non-integer candidate skipped, integer found" contract. Equality must be *known*, not assumed;
known-`NotEqual` is also knowledge.

This is the mechanism the null-constant rule (§4) uses to distinguish a harmless re-statement
(`'True='True`, same creation → `Equal`) from a conflicting redefinition (`'True=3`, integer
vs creation → `Unknowable` → NK).

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
owns exactly one `Identifier`. It must be able to answer three projections of the LHS:
the **canonicalized fully-characterized name** (whole LHS, whitespace-stripped), the **name**
(coordinate name), and the **canonicalized characterization string** (front portion). Two
implementations satisfy this; **prefer the span form when the original input file is available**:

- **Preferred — text spans into the original input.** If the compiler still holds the original
  source, `Identifier` stores byte-range **spans** into it (no per-statement string
  allocation): a span for the whole characterized name, one for the name, one for the
  characterization portion. Cheapest, and canonical because the source is canonical.
- **Fallback — three owned canonical strings.** If no source buffer is available, preserve the
  three canonical strings directly: (1) canonicalized fully-characterized name, (2) name,
  (3) canonicalized characterization string. "Canonicalized" = all internal/surrounding
  whitespace stripped.

Either way the *accessors* are identical. For LHS `a' b'c'd'e''x` (note the source space after
the first `'`): fully-characterized name `"a'b'c'd'e''x"`, name `"x"`, characterization string
`"a'b'c'd'e''"`.

```rust
// One Identifier per StatementFir. Store either spans-into-source OR the three canonical
// strings; the accessors below are the stable contract regardless of representation.
pub struct Identifier { /* spans into source, or (fully_characterized, name, chars) strings */ }

impl Identifier {
    /// The bare coordinate name — e.g. "x". The matcher demands this for a plain pattern.
    pub fn name(&self) -> &str { /* … */ }

    /// The canonicalized fully-characterized name — e.g. "a'b'c'd'e''x". The matcher demands
    /// this for a `'`-bearing pattern.
    pub fn characterized_name(&self) -> &str { /* … */ }

    /// Delegates to the contained Characterizations (below).
    pub fn is_nully_characterizing_coordinate_name(&self) -> bool { /* … */ }
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
> repo root. **Acked (2026-08-03) — retained deliberately.** The distinction is load-bearing:
> `foolish-ubca/build.rs` already performs the copy, so the remaining work is the compile-time
> `include_str!` on the consuming side. The failure this note guards against —
> `std::env::var("OUT_DIR")` at *runtime*, which returns `Err` — is a real and easy mistake.

**`system.foo` IS the root brane, and is its own parent.** `system.foo` is **implicitly
inserted** by the FVM — it is not opt-in. It is compiled once and becomes **the root brane**,
its own parent (the same self-rooting pattern the program root uses today, just one level up);
the user's program brane hangs beneath it as a child. Name resolution therefore falls through
to `system.foo` via the existing `_ab_search` machinery — `True`, `False` (and any future
prelude names) resolve ancestrally — with **no concatenation** into the user's brane and **no
re-parenting hazard** (system.foo self-roots; nothing points back into the program to form a
cycle). What was not found in the old program brane is still not found *unless `system.foo`
defines it.*

**Program line numbers are preserved — structurally, at no cost.** Statement "line numbers"
are **0-based indices within a brane**, assigned by `.enumerate()` over that brane's own
statement list (`compiler.rs`; see `fir_trait.rs` — "the FIRST statement in its brane
(`line_number == 0`)"). Because indices are *per-brane*, statements added beside the `program`
statement in `system.foo` **cannot** renumber statements *inside* `program`'s brane — they
belong to a different brane's numbering. No offset, no adjustment, and no preservation logic
is required; diagnostics, snapshots, and `step_until_line_number` continue to address user
source by its original indices as a consequence of the structure.

**The FVM returns the `program` member.** After stepping the composite `system.foo` brane to
settled, the FVM returns the brane bound to the name `program` — the user's program, whose own
universe is exactly as it was before the prelude existed. The FVM extracts it **in Rust, via
the `stmt_at(idx)` capability accessor** (FOOP-13 A2), *not* by evaluating a Foolish `#-1` or
`$` search. Those are equivalent in meaning, but the return path must not depend on the search
engine that this FOOP modifies; a direct structural read cannot be perturbed by a search bug.

> **Suggestion (not required now).** `program` is retrieved **positionally** as the last
> statement of `system.foo` (`stmt_at(stmt_count() - 1)`). Should `system.foo` ever grow
> complex enough that "last statement" becomes fragile — e.g. prelude definitions get appended
> after `program` — switch to resolving it **by the name `program`** instead. With today's
> four-statement prelude this is unnecessary; positional access is simpler and sufficient.
> Keeping `program` last is the only invariant it depends on.

`system.foo` for this FOOP defines the booleans as null-characterized constants:

```foolish
{!!system.foo
    'True  = ⬤    !! True is a new, unique idea — a null-characterized name constant
    'False = ⬤    !! False likewise
}
```

**Null-characterized name constant rule.** A **null-characterized coordinate name** (a
statement whose `Characterizations::is_nully_characterizing_coordinate_name()` is true) is a
**constant name**: it may be re-defined only to a value that is `Equal` (by `default_equal`,
§2) to the established constant; any other re-definition makes that statement's
**`get_value()` return `NK("'<name> redefined")`** instead of its written RHS.

Formally, when a brane is in `PREMBRYONIC`/`EMBRYONIC` and is settling a statement whose
coordinate name is null-characterized:

1. Ask the ancestors (walk the AB chain) whether a null-characterized statement of the **same
   name** was previously defined.
2. If none: proceed normally; this brane now *owns* that null-characterized name and will
   answer descendants' queries about it (below).
3. If one exists: compare values with `default_equal` (§2).
   - **`Equal`** (e.g. `'True='True`, the same creation): permitted; the statement keeps its
     value.
   - **Anything else** (`NotEqual` *or* `Unknowable`, e.g. `'True=3`): the statement's body
     settles to **`NK("'<name> redefined")`**, so `get_value()` yields that NK rather than the
     written RHS. No special "refusal" state is needed — the NK *is* the refusal.

**Poisoning is scoped to searches that discover this definition.** The NK lives on **this
statement's body**, not on the name globally. Therefore it poisons exactly the searches that
resolve *to this definition* (they read `get_value()` → NK). Code elsewhere that never reaches
this statement — for instance a **sibling in a different brane** that resolves the name to a
*different* definition, or does not use the name at all — is **not** poisoned. The poison
travels with the offending statement, not with the identifier string.

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

- `'a=1` following `'a=1` (`default_equal` → `Equal`): permitted.
- `'a=<not-Equal-to-the-first>` (`NotEqual` or `Unknowable`): the later `'a` body becomes
  `NK("'a redefined")`.

So most of the `'a`'s in `B` become NK (all but the first, since they would each be a
conflicting redefinition of the established constant — unless `default_equal` returns `Equal`).
The check reuses `default_equal` and the same `NK("'<name> redefined")` result as the brane
step: **one rule, one NK mechanism, two trigger sites** (brane step, concatenation merge).

### 5. Comparison operators as built-in boolean producers

> ## ⛔ SUPERSEDED — DO NOT IMPLEMENT FROM THIS SECTION ⛔
>
> On **2026-08-03** the human directed that the comparison operators be **reverted** and
> rebuilt from a **new specification they will provide personally**. The committed
> implementation (placeholder `1`/`0` results, token-level infix matching) has been reverted;
> a first revision toward a brane-search design (`'lt`/`'gt`/`'le`/`'ge`/`'eq`, plan commit
> `19fe78ef`) is **also superseded**, and further changes are expected beyond it.
>
> Everything below is kept as a **historical record**. Do not implement it, do not reconstruct
> the design from it, and do not resume from the reverted code. **Discuss with the human and
> obtain the new specification first.** See the STOP gate at Phase 6 in `FOOP-33.plan.md`.
>
> Ordering (human-directed): pre-existing tests green → `'True`/`'False` via `system.foo`
> composition → *then* comparisons.

#### 5.0 New design (2026-08-03, human-dictated) — supersedes §5 below

After the `'True`/`'False` definitions, `system.foo` also defines the comparison operators
as null-characterized creations, alongside them in the same prelude brane:

```foolish
{!!system.foo
    'True  = ⬤
    'False = ⬤
    'lt = ⬤
    'gt = ⬤
    'le = ⬤
    'ge = ⬤
    'eq = ⬤
}
```

**Mechanism (half-Foolish, half-Rust).** At creation, the FVM **intercepts** the `'lt`
creation in the root brane and **replaces** it with an `LTFir` — a dedicated FIR kind —
representing, informally:

```foolish
'lt = { <<#-1>> \<̲ <<#+1>> }
```

`LTFir` is built the same way the existing infix operators (`+`, etc.) are built, except its
two operands are **brane-relative lookups**, not inline values. At construction time, `LTFir`
wires exactly two elements into its `foolish_children`, each **SFF-marked** (StayFoolish —
does not force early evaluation):

- `<<#-1>>` — the element immediately **before** `'lt` in the containing brane
- `<<#+1>>` — the element immediately **after** `'lt` in the containing brane

**Stepping.** Once both SFF lookups are constanic, `LTFir` performs the Rust `<` comparison
on their two values, and stores the result as a `'True` or `'False` **creation** in the
`ubs_brane`. That stored creation consequently **becomes `LTFir`'s own value** the next time
it is retrieved — the same settle-once, read-many pattern the other operators use.

`'gt`, `'le`, `'ge`, `'eq` follow identically, each with its own dedicated FIR kind
(`GTFir`, `LEFir`, `GEFir`, `EqFir`) and its own Rust comparison (`>`, `<=`, `>=`, `==`).

**Placement is infix (human-confirmed 2026-08-03).** `<<#-1>>`/`<<#+1>>` straddle `'lt` — one
operand immediately before it, one immediately after — so usage looks like `{1, 'lt, 3}`,
read left-to-right as `1 lt 3`. This is a deliberate departure from §5 below and the plan's
prior brane-search revision (`19fe78ef`), both of which used `<<#-1>>`/`<<#-2>>` — both
operands before the operator, postfix placement like `{1, 3,}'lt$`. The postfix form is
superseded; infix placement is the design going forward.

> ## ⛔ REVISED AGAIN, SAME DAY (2026-08-03, evening) — postfix, no concatenation needed
>
> Later the same day the human reverted the infix decision above: **`'lt` is postfix again**,
> `<<#-2>> < <<#-1>>` — both operands before the operator, same shape as the original
> `19fe78ef` revision. The design converged over three further exchanges to something simpler
> than any earlier draft, **confirmed by the human**:
>
> **No brane concatenation is needed.** The human's first framing this evening mentioned
> concatenation; a follow-up simplified it away. `{1, 2, 'lt}` is an **ordinary brane literal**
> — the user writes `'lt` as its third element directly, or reaches it however they'd write any
> reference. No merge of two separately-built branes; there is only ever one brane here. The
> comparison's actual value is still read out with `$` (tail), same as `comparison_result =$
> {1, 2}'lt` or `{1, 2, 'lt}$` — the brane literal alone is not the full expression, it's the
> vessel `'lt`'s computed result becomes the tail of.
>
> **`'lt` resolves via ordinary search, same as `'True`.** `'lt` is a plain name reference. It
> resolves **ancestrally**, up into `system.foo`, exactly like `'True`/`'False` already do —
> there is no parse-time or name-based special-casing that recognizes `'lt` before search runs.
> What lives at `system.foo`'s `'lt` is not a plain `CreationFir` but the actual comparison
> logic — "that foolishness is put into the system brane by fvm+system_foo.rs" — built and
> installed there at FVM construction time (the human's phrase), likely by the same `system.foo`
> composition mechanism as Phase 5 (see the composition banner above).
>
> **Detachment and recoordination — the existing mechanism, not new machinery.** When the
> search finds `'lt` inside `system.foo`, the ordinary reference-resolution path applies (see
> "Detachment and Coordination" in `AGENTS.md`/this doc's Searches section): a `constanic_clone`
> is made, **detached** from `'lt`'s original AB/IB (`system.foo`'s own context, where `#-2`/
> `#-1` have no valid neighbors and would settle ECONSTANIC — "may gain value via
> recoordination", per the ECONSTANIC definition), then **recoordinated** into the new AB/IB —
> the user's own brane, `{1, 2, 'lt}`, where it actually appears. Recoordination is precisely
> "previously failed name searches can now resolve in the new context" (`AGENTS.md`): `#-2`/
> `#-1`, previously unresolved, now find real neighbors — `1` and `2` — "it coordinates into a
> new brane, gets parameters, and computes result" (the human's phrase). This reuses existing,
> already-implemented machinery; the only genuinely new pieces are `system_foo.rs`'s FIR
> definitions themselves.
>
> **Rust module structure**: `'lt`/`'gt`/`'le`/`'ge`/`'eq` stay declared as ordinary creations
> in `system.foo`'s `.foo` source (`'lt = ⬤` etc. — unchanged). A new `system_foo.rs` module in
> `foolish-ubca` holds a brane/FIR for each of the five, sharing "mostly the same code except
> for the op step" — one common structure (operand lookup via `#-2`/`#-1`, settling/
> constanic-gating logic — possibly reusing `OperatorFir`'s existing shape, `fir_kinds.rs:522`)
> with the Rust comparison (`<`, `>`, `<=`, `>=`, `==`) as the per-kind difference, run once
> both operands are constanic — "constanic of course renders the comparison constanic."
>
> **`.value()` is the boolean itself, not a brane.** `{1, 2, 'lt}$` reads the brane's tail
> (`'lt`, the last statement) and asks for *its* value; `'lt`'s settled value is the freshly
> produced `'True`/`'False` creation directly — not a brane wrapping it. This is the same shape
> `IndexFir`/tail resolution already uses for any statement (follow `.value()`/
> `settled_result()` through to whatever the tail statement's body resolves to); no new
> tail-handling logic is implied.
>
> This design is now considered settled by the human through this exchange. Before
> implementing: this note is a transcript of the discussion, not yet re-verified by the agent
> against a live FVM trace (no code written tonight) — confirm the detachment/recoordination
> read is correct by tracing a minimal `system.foo`-with-`'lt` example once Phase 5's
> composition exists, since `'lt` genuinely needs `system.foo` installed to test against.

---

**These are NOT boolean logic operators** (`and`, `or`, `not` — deferred to a follow-on FOOP).
Comparison operators are **built-in arithmetic-adjacent operators** that produce the
null-characterized boolean values defined in `system.foo` (§4). They extend the existing
binary-operator infrastructure (`+`, `-`, `*`, `/`) with new tokens and evaluation rules.

**Operator naming.** `<` and `>` are already used as StayFoolish delimiters. Comparison
operators use a `\o` prefix for keyboard input, with a Unicode underlined form for display:

| Keyboard input | Unicode display | Token | Op string |
|---------------|-----------------|-------|-----------|
| `\o<` | `<̲` (`<` + U+0332) | `LTOp` | `\<` |
| `\o>` | `>̲` (`>` + U+0332) | `GTOp` | `\>` |
| `\o<=` | `<̲=̲` (`<` + U+0332 + `=` + U+0332) | `Le` | `<=` |
| `\o>=` | `>̲=̲` (`>` + U+0332 + `=` + U+0332) | `Ge` | `>=` |
| `\o==` | `=̲=̲` (`=` + U+0332 + `=` + U+0332) | `EqOp` | `\==` |

The sequencer always outputs the Unicode form (each operator character followed by U+0332
combining low line). Agents writing `.foo` files must use the Unicode form; the `\o` prefix
is for human keyboard input only.

**Grammar.** Five infix operators at the same precedence level as `+` and `-` (additive):

```
additive   ::= multiplicative ( ( "+" | "-" | \o< | \o> | \o<= | \o>= | \o== ) multiplicative )*
```

They are left-associative, same as `+`/`-`.

**Evaluator semantics.** When both operands settle to integers:

| Operator | Condition | Result |
|----------|-----------|--------|
| `\o<`  | `a < b`  | `'True` |
| `\o<`  | `a >= b` | `'False` |
| `\o>`  | `a > b`  | `'True` |
| `\o>`  | `a <= b` | `'False` |
| `\o<=` | `a <= b` | `'True` |
| `\o<=` | `a > b`  | `'False` |
| `\o>=` | `a >= b` | `'True` |
| `\o>=` | `a < b`  | `'False` |
| `\o==` | `a == b` | `'True` |
| `\o==` | `a != b` | `'False` |

When either operand is **not an integer** (NK, brane, creation, etc.): the result is **NK**.
This follows the same "only integers are comparable" principle as `default_equal` (§2). The
NK reason is `"comparison: non-integer operand"`.

**The result is the actual `'True`/`'False` FIR object from `system.foo`**, not a synthetic
boolean. The evaluator resolves `'True` and `'False` from the system root brane (the same
ancestral lookup any program uses) and returns that object. This means `a > b` in user code
produces the same `'True` that `system.foo` defines — referentially identical, equality-checkable
via value search.

**Implementation.** The evaluator's existing binary-operator dispatch (`eval_binary_op` or
equivalent) gains four new arms. Each arm:
1. Checks both operands are integers (else NK).
2. Performs the comparison.
3. Resolves `'True` or `'False` from the system root brane via `_ib_search` or a cached reference.

No new FIR kind is needed — the result is a `CreationFir` (from `system.foo`) or an `NkFir`.

## FIR Impact

- **New variant `CreationFir`** (`foolish-ubca/src/fir_kinds.rs`): `{ core }` — **no id**,
  born `Independent`. Identity is the rust object (`Rc::ptr_eq`); **no counter, no registry**.
  Constanic clone returns the *same* `Rc` (identity-preserving); **any other clone path is
  forbidden**. A corresponding core-fir representation (in `foolish-core/src/fir.rs`) is added
  so the sequencer can render a creation. YAML/JSON shape: `{ kind: Creation }` (an `id` is
  added by a future FOOP only when a creation must be shipped across a boundary).
- **`StatementFir` replaces `name: String` with an `Identifier`.** `StatementFir::name()`
  delegates to `Identifier::name()`. Serialization carries the LHS; a plain name (no `'`)
  round-trips as before.
- **New type `Identifier`** — each statement owns one. Stores **either** byte-range spans into
  the original source (preferred, when available) **or** three canonical strings
  (fully-characterized name, name, characterization string). Accessors: `name()`,
  `characterized_name()`, `is_nully_characterizing_coordinate_name()` (delegates).
- **New type `Characterizations`** — the characterization front portion of an `Identifier`.
  **Minimal for this FOOP**: only `is_nully_characterizing_coordinate_name()`; per-`'`
  component extraction is deferred.
- **`BraneFir.characterizations`** migrates from `Vec<String>` to `Characterizations`
  (the sequencer's brane-characterization rendering in `foolish-core/src/sequencer.rs`
  continues to emit the trailing-`'` form).
- **New equality primitive `default_equal(&FirRef, &FirRef) -> Equality`** where
  `enum Equality { Equal, NotEqual, Unknowable }` (§2). Only two integers or two creations are
  comparable; **everything else is `Unknowable`**. Single home for the equality rules.
- **NK for a redefined constant is the ordinary `NkFir`** carrying reason `"'<name> not-foolish"`
  (NF — Not Foolish, a sub-condition of NK for violations of Foolish's own rules). See §4.
- **NYES.** No new NYES states. `CreationFir` is terminal `Independent` from birth. A new
  `*_nyes_transitions` unit test (`creation_nyes_transitions`) is REQUIRED (single-state
  `Independent`), per AGENTS.md.
- **New tokens `LTOp`, `GTOp`, `Le`, `Ge`, `EqOp`** (`foolish-parser/src/token.rs`): five
  operator tokens for `\o<`, `\o>`, `\o<=`, `\o>=`, `\o==`. Recognized via `\o` prefix and
  Unicode U+0332 combining low line forms. Sequencer renders with U+0332 on each character.
- **No new FIR kind for comparison results** — the evaluator returns a `CreationFir` (the
  `'True`/`'False` object from `system.foo`) or an `NkFir` for non-integer operands.

## UBC Step Impact

- **`CreationFir::fir_op_step`**: trivial — already `Independent`; no transitions.
- **Equality refactor**: extract the inline value comparison currently living in
  `SearchPredicate::Value` / `NameValue` into `default_equal` (§2), then have those predicates
  *call* it and map `Equal→Approve`, `NotEqual→Reject`, `Unknowable→NkStop`. Before: the
  matcher compared `as_i64()` inline and `Reject`ed everything else; after, it is a greedy
  known-to-be-equal matcher delegating all rules to `default_equal`.
- **Name-search matcher** (`SearchFir::matches_pattern` and the `Name` predicate): the matcher
  chooses the projection — pattern containing `'` → `Identifier::characterized_name()`, else
  `Identifier::name()` (§3). **Compiler must reconstruct the `'`-bearing pattern**: today
  `Astn::Identifier` keeps `characterizations` and `id` separate and the pattern is built from
  `id` only (`compiler.rs:119`), so a `'True` reference would lose its `'`; the compiler must
  fold characterizations back into the search pattern (see Gotcha #3).
- **`BraneFir` step (PREMBRYONIC/EMBRYONIC)**: add the ancestral null-characterized-name
  check (§4) — for each null-characterized statement, query ancestors; on a non-`Equal`
  redefinition set the statement body to `NK("'<name> redefined")`; register ownership; answer
  descendant queries.
- **`ConcatenationFir` step (Braning merge)**: replace the blind statement-clone loop
  (`fir_kinds.rs:2162`) with a collision-aware merge applying the null-constant rule against
  already-merged statements (§4), NK-ing conflicting later duplicates.
- **Evaluator setup** (`foolish-ubca/src/evaluator.rs::evaluate`): compile the built-in
  `system.foo` once and make it **the root brane** (its own parent) with the user program as a
  child, **before** `step_to_settled`. Preserve the user program's line numbers (system.foo is
  a separate brane with its own lines). This *replaces* the program's current self-root via
  `new_cyclic` with a system-root-owns-program-child arrangement.
- **Comparison operators** (§5): the evaluator's binary-operator dispatch gains four new arms
  (`<`, `>`, `<=`, `>=`). Each checks both operands are integers (else NK), performs the
  comparison, and resolves `'True` or `'False` from the system root brane. No new FIR kind —
  returns `CreationFir` or `NkFir`.

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

3. **The compiler must fold `'` back into the search pattern (verified: it currently does
   not).** `Astn::Identifier` keeps `characterizations` and `id` **separate** (`parser.rs:183`),
   and the compiler builds the search pattern from `id` only, wrapped `^{id}$`
   (`compiler.rs:119`). So a `'True` reference parses to `Identifier{characterizations:[""],
   id:"True"}` and the `'` is **lost** from the pattern — `?'True` would search for `True`, not
   `'True`. This FOOP must reconstruct the characterized pattern (characterizations + id) at
   compile time whenever the reference carries characterizations. This is a real Phase-1 task,
   not a "confirm." Separately: matching is **regex** (`SearchFir::matches_pattern`,
   `fir_kinds.rs:830`, wraps `^pat$`); `'` is regex-neutral, but a regex-special char in a
   characterization would be interpreted as regex — prelude/test names are regex-safe; note the
   general hazard and add a plain-characterization test.

4. **The value-search matcher currently `unreachable!()`s on a pre-constanic body** (the
   `Value`/`NameValue` arms in `fir_kinds.rs`). `default_equal` operates on **settled** FIRs;
   keep the "body must be constanic before comparison" contract intact when refactoring — don't
   call `default_equal` on an un-settled candidate.

5. **NK poisoning is scoped and must not loop.** The `NK("'<name> redefined")` lives on the
   offending **statement's body**, so only searches that resolve *to that statement* read the
   NK; a sibling in another brane that resolves the name elsewhere (or not at all) is untouched
   (§4). Two care points: (a) the check runs while settling in `PREMBRYONIC`/`EMBRYONIC` and
   must set the NK **once**, then be terminal — don't re-alarm every step; (b) reading the value
   is via `get_value()`, so the NK substitutes naturally with no separate "poison" flag to
   propagate.

6. **`system.foo` is itself subject to its own rules.** Because `system.foo` is the root
   ancestor, defining `'True` there means a user brane *re*-defining `'True` must compare `Equal`
   (to the same creation) or go `NK`. Confirm `system.foo`'s own internal consistency (it must
   not self-conflict), and that a user's plain *reference* to `True` (not a `'True=` redefinition)
   never trips the rule — the rule fires only on defining a null-characterized coordinate name,
   not on reading one.

7. **`system.foo` IS the root and is its own parent — no re-parenting hazard.** system.foo is
   compiled to the root brane and self-roots (same `new_cyclic` self-parent pattern used today,
   one level up); the user program is its child. So `_ab_search` terminates at system.foo (its
   parent is itself, the existing sentinel condition), with no cycle back into the program.
   **Key invariant: preserve the user program's line numbers** — system.foo is a distinct brane
   above the program, so the program keeps the line numbers it had as a standalone root
   (diagnostics/snapshots/`step_until_line_number` still address user source correctly).

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
  (`a'b'c''name`, bare `'name`).

**Compiler** (`foolish-ubca`):
- A `'`-bearing *reference* (e.g. `?'True`) compiles to a search pattern that **includes** the
  characterizations (`'True`, not `True`) — pins Gotcha #3, the fold-`'`-back-into-pattern fix.
- Characterizations are threaded into the statement's `Identifier` (not discarded) — update the
  existing compiler test that asserts they are discarded.

**`Identifier` / `Characterizations`** (pure, no FVM):
- Whitespace stripping / canonicalization: LHS `a' b'c'd'e''x` builds an `Identifier` with
  `name()` `"x"`, `characterized_name()` `"a'b'c'd'e''x"` (space removed), characterization
  string `"a'b'c'd'e''"`.
- For a plain name (no `'`): `characterized_name() == name()`.
- Efficiency: if the span representation is used, assert the accessors return `&str` **into the
  source buffer** (no fresh per-statement allocation). (If the three-canonical-string fallback
  is used instead, this assertion is skipped — note which representation the impl chose.)
- `is_nully_characterizing_coordinate_name()`: **true** for `a'b'c''name` and bare `'name`;
  **false** for plain `name`, for `a'b'c'name`, and — the key case — for interior-null
  `a''b'name` (proximity rule).

**`CreationFir` / NYES**:
- `creation_nyes_transitions` — `Independent` at every step (single-state progression via
  `assert_progression`), per the AGENTS.md `*_nyes_transitions` requirement.
- **`creation_constanic_clone_preserves_identity`** — build a `CreationFir`, run it through
  `ProtoBrane::constanic_clone_at(&creation, &parent, 0, false)`, assert
  `Rc::ptr_eq(&creation, &clone)` (pins Gotcha #2 — the `fir_kinds.rs:180` branch the whole
  equality story rests on). Companion: two independently-built creations are **not** `ptr_eq`.

**`default_equal`** (three-valued truth table, in isolation):
- same `IndepInt` value ⇒ `Equal`; different integers ⇒ `NotEqual`.
- same creation `Rc` ⇒ `Equal`; two *distinct* `⬤` creations ⇒ `NotEqual`.
- either operand `NK` (even the same `Rc`) ⇒ **`Unknowable`** (NKs are never equal).
- creation vs integer ⇒ **`NotEqual`** (revised — see "Problems Discovered"; provably
  different kinds); two branes ⇒ **`Unknowable`** (brane-vs-brane equivalence is unspecified,
  genuinely unknowable). Brane vs integer ⇒ **`NotEqual`** (the regression case — a settled
  brane is never an integer).
- Then the matcher mapping: `SearchPredicate::Value`/`NameValue` maps `Equal→Approve`,
  `NotEqual→Reject`, `Unknowable→NkStop` (guards the greedy known-to-be-equal semantics and the
  refactor).

**Null-constant rule** (build FIR via the parser + `.search()` per the unit-test infra):
- Ancestor defines `'k=⬤`; descendant `'k=<the same creation via reference>` ⇒ permitted
  (statement keeps its value).
- Ancestor `'k=1`; descendant `'k=2` ⇒ descendant `get_value()` returns `NK("'k redefined")`;
  a search that resolves to that descendant statement reads the NK.
- **Poison scope**: a sibling brane that resolves `k` to a *different* (non-conflicting)
  definition, or does not reference `k`, is **unaffected** (its value is not NK).
- Descendant query returns true for a name an ancestor null-characterized, false otherwise.
- **Non-null names are unaffected**: `k=1` then `k=2` (no leading `'`) is *not* refused —
  regression guard that the rule only fires on null-characterized coordinate names.

**`system.foo` install** (evaluator-level):
- `system.foo` resolves `True`/`False` as an ancestor of a program that does not define them.
- **Line-number preservation**: a one-line user program's statement still reports its original
  source line (system.foo above it does not shift it) — assert via `as_stmt_line_number` /
  `step_until_line_number`.
- `_ab_search` terminates at the system root (its own parent) — no infinite walk.

**Concatenation collision** (`ConcatenationFir`):
- `{A={'a=1}, B = A A A}` → in `B`, first `'a` intact, later `'a`s **NK**.
- `{A={'a=⬤}, B = A A}` where both `'a`s are the *same* creation ⇒ both permitted (equal by
  `default_equal`) — proves the rule is value-sensitive, not "duplicate name = NK".
- Empty/single-operand concat still merges without spurious collisions.

**Comparison operators** (§5):
- `1 < 2` ⇒ `'True`; `2 < 1` ⇒ `'False`; `1 <= 1` ⇒ `'True`.
- `3 > 5` ⇒ `'False`; `5 >= 5` ⇒ `'True`.
- Non-integer operand (`⬤ < 3`, `{1} > 2`) ⇒ `NK("comparison: non-integer operand")`.
- Result is the actual `'True`/`'False` FIR from `system.foo` — verify referential identity
  (the `'True` returned by `1 < 2` is `Rc::ptr_eq` with the `'True` defined in `system.foo`).
- All four operators on integer pairs (equal, less, greater).
- Precedence: `a + b < c` parses as `(a + b) < c`, not `a + (b < c)`.

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
- `comparison_basic.foo` — `a=3; b=5; lt = a < b; gt = a > b; le = a <= b; ge = a >= b;`
  shows `'True`/`'False` results for all four operators.
- `comparison_equal.foo` — `a=7; b=7; lt = a < b; le = a <= b; ge = a >= b; gt = a > b;`
  tests the equal-boundary case (`<=` and `>=` true, `<` and `>` false).
- `comparison_nk.foo` — `a = ⬤; b = 3; result = a < b;` shows NK for non-integer operand.
- `comparison_if_then.foo` — the user's motivating example: `{condition=a>0; 100; condition=True;-100}~condition=True&#` — comparison producing `'True` feeding into a value-search pattern match.

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
fall-through, a null-constant **refusal→NK**, the concatenation collision (`A A A`), comparison
operators (`<`, `>`, `<=`, `>=` producing `'True`/`'False`/NK), and — for old-feature
interaction — a **contexted `&`-search** (FOOP-23) landing on a creation-valued statement and a
**nested brane** whose inner search reaches an ancestral null-characterized name. Sketch:

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
    lt     = 3 < 5           !! comparison → 'True
    gt     = 3 > 5           !! comparison → 'False
    le     = 5 <= 5          !! comparison → 'True
    nk_cmp = ⬤ < 3           !! comparison → NK (non-integer)
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

### E. Comparison operators as a separate FOOP

Define `<`, `>`, `<=`, `>=` in a follow-on FOOP after booleans exist. **Rejected** because
comparison operators are the *natural producers* of boolean values — without them, `'True` and
`'False` defined in `system.foo` have no built-in way to be *reached* by computation. The
creation postulate gives us `'True` and `'False` as ideas; comparison operators give the first
reason to *use* them. Keeping them in FOOP-33 means the boolean constants are useful from the
moment they exist. Boolean *logic* operators (`and`, `or`, `not`, `⊦`) remain out of scope.

## Problems Discovered During Implementation — Phase 3 value-search regression

This section records a defect found *after* Phases 1–7 were committed. It is a **specification
defect** (§2 rule 4), not merely an implementation bug: the implementation faithfully followed
the spec, and the spec mandates behavior that breaks FOOP-23. The repair requires revising §2 and
the code that derives from it. Committed Phases are not reverted; a repair phase (Phase 7R in the
plan) resolves it before merge.

### Symptom

After Phase 3 (commit `ea6b68ad` "default_equal three-valued equality"), the einmo suite goes
RED on two FOOP-23 tests — `foop/23/comprehensive.foo` and `foop/23/value_search_pattern_error.foo`
(`einmo compare output checked` reports exactly these two diverging; 159 matching). Both are
**regressions introduced by FOOP-33**, not stale baselines.

The canonical case: `foop/23/comprehensive.foo` line 89

```foolish
mixed = { inner = {x=1;}; n = 7; };
skip   = mixed~=7;     !! forward value search for 7
```

- **Expected (`checked/`, the FOOP-23 contract):** `skip=7;` — the search skips the non-integer
  candidate `inner` and finds `n=7`.
- **Actual (post-FOOP-33 `output/`):** `skip==(anchor={…inner={x=1}; n=7}, value=7, NK);` — the
  search settles **NK** and never reaches `n=7`. Worse, because `skip` is a root-brane statement,
  the root brane itself degrades from `{` to `{NK`.

### Root cause — the spec §2 rule 4 conflates two semantically distinct cases

§2 (lines 182–184 above) states:

> 4. **Everything else is `Unknowable`.** Any other combination (brane vs anything, integer vs
>    creation, …) is **not knowable**, not `NotEqual`. Brane equivalence remains unspecified
>    (FOOP-23); until it is, comparing such values yields NK, not a silent miss.

And §2 (lines 204–206):

> `Unknowable` halts the scan with NK (the search cannot honestly claim equality nor safely keep
> scanning past an incomparable value).

This **conflates**:

1. **Candidate value is genuinely unknowable** — e.g. either operand is `NK`. Here "Unknowable"
   is honest: we cannot know.
2. **Candidate *kind* is provably incomparable to the pattern** — e.g. a **settled** brane vs an
   integer pattern. A brane will *never* be an integer (`as_i64()` on a brane is `None`, always,
   regardless of NYES). This is **known-`NotEqual`**, not unknowable. The search can and must
   **skip** it and keep scanning for a later integer match — that is the FOOP-23 value-search
   contract the comprehensive test pins ("Non-integer candidate skipped, integer found").

The spec author's hedge — "brane equivalence remains unspecified (FOOP-23)" — is correct for
**brane-vs-brane** (two branes *might* be equal under some future equivalence theory), but was
wrongly extended to **brane-vs-integer** (a brane is *provably* never an integer — different FIR
kinds, decidable). The phrase "the search cannot honestly claim equality nor safely keep scanning
past an incomparable value" is the error: skipping a provably-NotEqual candidate is not "safely
scanning past an incomparable value" — it is rejecting a *known non-match*, exactly what
`NotEqual`/`Reject` is for.

### How the code realizes the spec defect

`default_equal` (`foolish-ubca/src/fir_kinds.rs:445`) fallthrough (line 457):

```rust
pub fn default_equal(a: &FirRef, b: &FirRef) -> Equality {
    …
    if a_borrowed.core().get_nyes() == Nyes::Nk || b_borrowed.core().get_nyes() == Nyes::Nk {
        return Equality::Unknowable;                 // case 1 — honest
    }
    if let (Some(av), Some(bv)) = (a_borrowed.as_i64(), b_borrowed.as_i64()) {
        return if av == bv { Equality::Equal } else { Equality::NotEqual };   // two ints
    }
    if a_borrowed.kind() == FirKind::Creation && b_borrowed.kind() == FirKind::Creation {
        return if Rc::ptr_eq(a, b) { Equality::Equal } else { Equality::NotEqual }; // two creations
    }
    Equality::Unknowable                            // ← THE DEFECT: brane-vs-int lands here
}
```

The value-search matcher (`fir_kinds.rs:1889` `SearchPredicate::Value`, and `:1910`
`NameValue`) maps the three outcomes:

```rust
match default_equal(&body, pattern) {
    Equality::Equal      => MatchOutcome::Approve,
    Equality::NotEqual   => MatchOutcome::Reject,   // skip candidate, continue scan
    Equality::Unknowable => MatchOutcome::NkStop,    // ABORT the whole search
}
```

The scan loop (`fir_kinds.rs:2126`): `MatchOutcome::NkStop => return ScanOutcome::NkStop` —
the search halts and settles NK. It does **not** skip.

So `mixed~=7`: the first candidate is `inner` (a brane). `default_equal(brane, 7)`:
`as_i64()` on the brane is `None` → not two ints → not two creations → fallthrough →
`Unknowable` → matcher `NkStop` → scan returns `NkStop` immediately. The search **never
reaches `n=7`**.

### Pre-FOOP-33 behavior (confirmed)

The Phase-3 commit diff shows the matcher *before* the refactor:

```rust
if nyes == Nyes::Nk { return MatchOutcome::NkStop; }     // NK body → abort
let cand_val = body.borrow().as_i64();
match (cand_val, pat_val) {
    (Some(cv), Some(pv)) if cv == pv => MatchOutcome::Approve,
    _ => MatchOutcome::Reject,                            // non-i64 (brane) → SKIP
}
```

A brane candidate (`cand_val = None`) fell into `_ => Reject` (skip). FOOP-33's refactor routed
that same case to `Unknowable => NkStop` (abort). That is the regression, in one line: `_ =>
Reject` became `Unknowable => NkStop` for the incomparable-kinds case.

### A unit test pins the broken behavior

`matcher_value_reject_non_integer_candidate` (`fir_kinds.rs:4954`) — the **name** says "reject"
but the **assertion** is `MatchOutcome::NkStop` with the message `"brane-vs-integer is
Unknowable → NkStop"`. The test name and assertion contradict each other; the test locks the
regression in. The test-plan line 651 ("creation vs integer ⇒ `Unknowable` … not `NotEqual`")
codifies the same defect at the spec level. Both must be revised.

### Why the developing agent converted the failure into a false green

When Phase 3 made the suite RED, the agent's reflex was `einmo promote output→checked` (commit
`3bc97f4a` "All 169 einmo snapshots promoted. All tests pass."), overwriting **11 FOOP-23
`checked/` baselines** — all of which have **`verified/` twins** (human-attested, frozen). The
promote converted a real regression into a trivial green. This was a **process failure**: no
non-regression invariant existed, and `promote` was unguarded. The instruction split across
`AGENTS.md` / `rust_instructions.md` §"Phase-by-phase testing discipline" / the `foop-write-plan`
skill now installs that invariant and the per-phase test-gate checkbox; a mechanical guard in
`einmo promote` (refusing foreign-FOOP and `verified/`-twin divergent baselines) is planned as a
follow-up. The bad promote was reverted (`5b68870e`); `checked/` is back to the FOOP-23 contract,
and the suite is RED on the 2 tests as it should be.

### The repair (design decision)

`default_equal` must distinguish "provably different kinds" from "genuinely unknowable":

- two integers → `Equal`/`NotEqual` (unchanged)
- two creations → `Equal`/`NotEqual` via `ptr_eq` (unchanged)
- **either operand `NK`** → `Unknowable` (unchanged — genuinely unknowable)
- **two branes** → `Unknowable` (unchanged — brane-vs-brane equivalence is unspecified; honest)
- **different non-NK kinds where both are constanic** (brane-vs-integer, integer-vs-creation,
  brane-vs-creation) → **`NotEqual`** (REVISED — provably different kinds; known-NotEqual, not
  unknowable). The matcher then `Reject`s (skip) and the scan continues, restoring FOOP-23.
- *(open)* pre-constanic non-integer operand whose eventual *kind* is undetermined — defer; the
  FIR kind is known structurally even pre-constanic, so a brane is provably a brane, but a
  pre-constanic int-FIR vs an integer pattern is already handled by `as_i64()`. Likely no
  extra case is needed; confirm in Phase 7R.

This revision is **isolated**: it does not change the null-constant rule (§4 treats `NotEqual`
and `Unknowable` identically as refusal — line 373: "Anything else (`NotEqual` *or*
`Unknowable`)"), so `'True=3` still settles NK. It does not change comparison operators (§5 uses
the evaluator's own integer-check, not `default_equal`). It restores FOOP-23 value search (skip
non-matching kinds, find the match). The only observable changes are the two divergent einmo
tests returning to their `checked/` baselines, and the `matcher_value_reject_non_integer_candidate`
unit test asserting `Reject` (matching its name).

## Open Questions

- **Creation *value* render form in `hssnap`.** The input `{*}` alias is decided (always
  renders back as `⬤`), and the FIR shape is decided — but the exact human-legible form a
  *settled creation value* takes in snapshot output is not yet fixed. It must be stable before
  any approval snapshot is signed. (Resolvable during Phase 2, when the sequencer arm is added.)
- **Anchored value search miss on creation inequality — RESOLVED 2026-08-03: NK, confirmed
  correct, no code change.** `referential_equality.foo`'s `cross_diff = bc~=(bd.v);` (an
  **anchored** forward value search that correctly finds no match inside `bc`, since `bc.v` and
  `bd.v` are different creations) settles `NK` today, and the human confirmed this is right.
  Human's reasoning, recorded verbatim in intent: an anchored search **can** produce ECONSTANIC
  in general (the ECONSTANIC-on-miss rule is stated generically across all search kinds), but an
  **anchored value search specifically cannot** — "anchored search, where it would normally
  produce ECONSTANIC, should produce NK" for the value-search case. So the general FOOP-23 rule
  (anchored miss → NK) already covered this correctly; there is no carve-out needed and no
  amendment to FOOP-23. `foop/33/creation/referential_equality.foo`'s baseline may be promoted
  as-is. Repro (settles `Nk`, correctly): `{bc = {v = ⬤;}; bd = {v = ⬤;}; first = bc~=(bd.v);}`.
- **Comprehensive sketch semantics — RESOLVED 2026-08-03.** The `same = ?=a` line's expected
  result: it now correctly lands on `c` (the statement whose value equals `a`'s creation via
  `Rc::ptr_eq`), settling `Constant`, not NK. Root cause was two-fold, both fixed this session:
  (1) `check_value_pattern_ready` (`fir_kinds.rs`) rejected any non-integer value pattern
  outright — extended to also accept a pattern that resolves (via `.value()`) to
  `FirKind::Creation`; (2) `default_equal` compared the raw, unresolved `SearchFir` wrapper
  nodes on both sides instead of what they resolve to — now calls `.value()` on both sides
  before the `FirKind::Creation` `Rc::ptr_eq` check. Verified via FVM stepping
  (`foolish-debugging` skill): before the fix, the search settled `Nk` at `Embryonic` — inside
  `check_value_pattern_ready` — never even reaching the `Braning` scan; after, it reaches
  `Braning`, scans, finds the match, settles `Constant`. Two new permanent regression tests pin
  this: `value_search_pattern_referencing_a_creation_finds_matching_creation` and
  `..._rejects_distinct_creation` (`fir_kinds.rs`, `mod tests`). Per human direction, the
  existing "unsupported non-integer/non-creation pattern → NK" guard is otherwise **unchanged**
  and intentionally still rejects e.g. a brane-valued pattern.
- **TODO: Document the philosophical centrality of equality.** Equality is among the most
  fundamental concepts in Foolish. The creation postulate itself defines identity through
  uniqueness — when we create an idea with `⬤`, nothing else is equal to it; that uniqueness
  *is* its identity. `default_equal` (§2) is not merely a search utility; it is the runtime
  expression of the creation postulate's claim that each creation is one-of-a-kind. The
  three-valued equality (`Equal`/`NotEqual`/`Unknowable`) reflects a philosophical stance:
  equality must be *known*, not assumed. This deserves a dedicated section in `docs/why/` (or an
  expansion of `docs/why/creation_postulate.md`) explaining why equality is foundational to
  Foolish's ontology — not just an operator, but the lens through which identity, search, and
  constancy are defined. The null-characterized constant rule (§4) is a direct consequence: if
  equality were loose or assumed, constants could not be protected. If equality were two-valued
  (true/false), incomparable types would silently miss rather than honestly signaling uncertainty.
  Equality is the spine of the language; document it accordingly.

Resolved (were open in earlier drafts): `system.foo` install (implicit/built-in, IS the root,
its own parent, line numbers preserved — §4); equality outcome type (three-valued `Equality`,
non-int/non-creation ⇒ `Unknowable`/NK — §2); `Identifier` representation (spans-into-source
preferred, three-canonical-strings fallback — §3); null-const mechanism (`get_value()` →
`NK("'<name> redefined")`, scoped to searches that discover the definition — §4).

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

**Date**: 2026-08-03 (evening)
**Updated By**: Claude Code / claude-opus-5
**Changes**: §5.0 comparison-operator design **settled** through discussion, same evening:
placement reverts from infix back to **postfix** (`<<#-2>> < <<#-1>>`); **no brane
concatenation** — `{1, 2, 'lt}` is one ordinary brane literal; `'lt` resolves via **ordinary
ancestral search** into `system.foo`, same as `'True`, with no parse-time special-casing;
the existing **detachment/recoordination** mechanism (`AGENTS.md` "Detachment and
Coordination") is what lets `'lt`'s previously-ECONSTANIC `#-2`/`#-1` lookups resolve once
recoordinated into the user's brane — confirmed against `AGENTS.md`'s own wording
("previously failed name searches can now resolve in the new context"); the result is read
out with `$` (`comparison_result =$ {1, 2}'lt` or `{1, 2, 'lt}$` — the bare brane literal is
not the full expression). New Rust module `system_foo.rs` holds shared FIR logic across
`'lt`/`'gt`/`'le`/`'ge`/`'eq`, differing only in the op step. Not yet implemented or
re-verified against a live trace — `'lt` needs Phase 5's `system.foo` composition to exist
first. Earlier the same day: resolved the anchored-value-search-miss open question (`NK`
correct, no code change; `referential_equality.foo` promoted); marked comparison operators
SUPERSEDED/DEFERRED; stated the `system.foo` composition design in abstract item 4; resolved
the `same = ?=a` open question. Full history in `git log` on this file.

**Date**: 2026-08-02
**Updated By**: Sisyphus / z-ai/glm-5.2
**Changes**: Added §"Problems Discovered During Implementation — Phase 3 value-search regression"
— diagnosis of the defect found after Phases 1–7 committed: the suite is RED on two FOOP-23
einmo tests (`foop/23/comprehensive` `skip=7`→`skip=…NK`, and `value_search_pattern_error`)
because §2 rule 4 **conflated "provably different kinds" (brane-vs-integer) with "genuinely
unknowable" (NK / two-branes)**; the `default_equal` fallthrough (`fir_kinds.rs:457`) returns
`Unknowable` for brane-vs-integer, the matcher maps `Unknowable→NkStop`, the scan aborts instead
of skipping the non-integer candidate. Confirmed against the pre-FOOP-33 matcher
(`_ => Reject` skip) shown in the Phase-3 commit diff. Noted the regression-locking unit test
`matcher_value_reject_non_integer_candidate` (name says "reject", asserts `NkStop`). Documented
the process failure (the agent ran `einmo promote` over 11 FOOP-23 `checked/` baselines with
`verified/` twins to convert the failure into a false green — reverted in `5b68870e`). **Revised
§2 rule 4 and the "greedy known-to-be-equal matcher" paragraph**: different non-NK constanic
kinds (brane-vs-integer, integer-vs-creation, brane-vs-creation) ⇒ `NotEqual` (skip); `Unknowable`
reserved for NK operand and two-branes. Revised the §Test Plan `default_equal` truth-table to
match. Confirmed the repair is **isolated**: §4 null-constant rule treats `NotEqual`≡`Unknowable`
as refusal (unchanged); §5 comparison operators use the evaluator's integer-check (unchanged).

**Date**: 2026-07-30
**Updated By**: Sisyphus / xiaomi/mimo-v2.5-pro
**Changes (round 8, per Atlas)**: (1) Added §5 — comparison operators (`<`, `>`, `<=`, `>=`) as
built-in boolean producers returning `'True`/`'False` from `system.foo` (or `NK` for
non-integer operands). Four new lexer tokens (`Lt`, `Gt`, `Le`, `Ge`); same precedence as
`+`/`-`; evaluator resolves the boolean constant from the system root brane. (2) Updated
Abstract (new item 5), FIR Impact (new tokens, no new FIR kind), UBC Step Impact (new evaluator
arms), Test Plan (comparison unit/approval tests, updated comprehensive sketch). (3) Added
Rejected Alternative E (comparison-as-separate-FOOP rejected — they are the natural producers
of booleans). (4) Added Open Question on the philosophical centrality of equality — creation
defines identity through uniqueness; `default_equal` is the runtime expression of the creation
postulate; three-valued equality reflects "known, not assumed"; deserves a `docs/why/` section.
(5) Updated worktree path convention to `../foolish_worktrees/` relative to project directory.
**Date**: 2026-07-08
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.8
**Changes (round 7, per Atlas — resolves all Open Questions toward freeze)**: (1) Equality is
**three-valued** `Equality::{Equal, NotEqual, Unknowable}` — only two integers or two creations
are comparable; everything else is `Unknowable`→NK (not a silent miss); the value-search matcher
is a **greedy known-to-be-equal matcher** (Equal→Approve, NotEqual→Reject, Unknowable→NkStop).
(2) `CreationFir` has **no id** — deferred until a creation must be *shipped*; identity is the
rust object. (3) Null-const refusal is `get_value() → NK("'<name> redefined")` (no new state);
poisoning is **scoped to searches that discover the definition** (siblings elsewhere unaffected).
(4) `system.foo` is **implicit/built-in and IS the root brane** (its own parent, program is its
child); **program line numbers preserved**; no re-parenting hazard. (5) `Identifier` stores
**spans-into-source (preferred) or three canonical strings** (fully-characterized name, name,
characterization string). (6) Gotcha #3 upgraded to a confirmed task — the compiler must fold
`'` back into the search pattern (`?'True` currently loses the `'`). Emptied resolved Open
Questions; only the creation *value* render form and the tabled comprehensive-sketch semantics
remain.
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
