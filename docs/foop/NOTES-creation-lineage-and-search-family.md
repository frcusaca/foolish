# Design Notes — Creation Lineage & Search Family (FOOP-43/53/63/73/83/93/04)

> **Draft pass. Several rounds expected. Open questions are left open on purpose.**
> Planning/idea document, not itself a FOOP. Gathers notes, code analysis, and a
> dependency-ordered plan for seven FOOPs, built on FOOP-33 (Creation Postulate — written,
> `Final`, **not yet implemented**). When promoting a section to a real FOOP, run
> `foop_check.py gen_next` and move its notes into `FOOP-#.md` + `FOOP-#.plan.md`.

**FOOP number assignments (2026-07-09).** Little-endian next numbers, assigned by priority:

| FOOP    | Feature                                             | §  | Order |
|---------|-----------------------------------------------------|----|-------|
| FOOP-43 | Search-miss → ECONSTANIC, not NK (foundational)     | §7 | **1st — do first** |
| FOOP-53 | Inverse matcher `!`                                 | §4 | 2nd   |
| FOOP-63 | Detachment = parameterized SF/SFF marker            | §2 | 3rd (needs FOOP-43) |
| FOOP-73 | All-results / find-all `~~` `??`                    | §5 | 4th   |
| FOOP-83 | Boolean operators `and/or/not/nor/xor`              | §1 | 5th (needs FOOP-33) |
| FOOP-93 | Recursion Upgrades (write ~1-2 dozen recursive algos first) | §6 | **after the full search suite** |
| FOOP-04 | Macros (research)                                   | §3 | 7th   |
| FOOP-14 | Computed index `#${...}`                            | §8 | any (self-contained) |
| FOOP-24 | Boolean-combinator "beefy" search `&&` `\|\|` `\|`  | §9 | after FOOP-43/53 (composes) |
| FOOP-34 | Strengthen integer math (`**`, `< > <= >=`)        | §10 | `**` any; comparisons after FOOP-83 |
| FOOP-44 | Primitive Characterization (`i'` int, `s'` string, `f'` float + operators) | §11 | after FOOP-33 |

(The `§` column maps to the existing section headings below, which retain their original draft
numbering.)

Author: drafted 2026-07-08 (Claude Code / Opus 4.8) with Atlas; rev 4 2026-07-09. Grounded in a
code read of `foolish-parser/`, `foolish-ubca/`, and `docs/vintage_legacy/`.

---

## The big picture

The seven cluster around **two shared substrates: the one-engine search matcher and the creation
identity.** Detachment (FOOP-63) is a **parameterized SF/SFF marker that prefilters searches** —
it lives in the search family, sharing the *exact* candidate-prefilter locus with the inverse
matcher (FOOP-53).

```
Creation lineage:   FOOP-33 (creation ⬤, ==, system.foo, booleans True/False)
                        └─> FOOP-83 Boolean operators (and/or/not/nor/xor)   [needs FOOP-33]

Search family       FOOP-43 Miss→ECONSTANIC (foundational; revises FOOP-23) ─┐ DO FIRST; prereq
(one-engine model,  FOOP-53 Inverse matcher  !         ─┐ candidate PREFILTERS │ for FOOP-63
independent of      FOOP-73 All-results      ~~/??      ─┤ / scan-mode in the   │
FOOP-33):           FOOP-63 Detachment       [p..]<<E>> ─┘ same engine; compose ┘  a![p..]~~q

Recursion:          FOOP-93 Recursion — AFTER the full search suite. Write Fibonacci + standard
                        algorithms FIRST → they reveal the needed sugar. `↑`. NO cycle detection.

Cross-cutting:      FOOP-04 Macros (research; leans on name/value matching + all-results)
```

Boolean operators (FOOP-83) need FOOP-33 first. The **search family** (FOOP-43/53/63/73) is
independent of creations. **FOOP-43 (miss→ECONSTANIC) is foundational and MUST be done first** —
it fixes a real bug and is a prerequisite for detachment (FOOP-63). FOOP-53/73/63 are candidate
prefilters or scan-mode variants of the *same* matcher and compose. Recursion (FOOP-93) needs
`↑`. Macros (FOOP-04) is research, benefits from all-results (FOOP-73).

### SF vs SFF are the two extremes of the detachment spectrum (key framing)

The undetached defaults of the two markers **differ**, and detachment unifies them:
- **`<E>` (SF) ≡ `[]<E>`** — the **empty** detachment (detach nothing; everything resolves
  normally).
- **`<<E>>` (SFF) ≡ `[*]<<E>>`** — the **full** detachment (detach everything).

So SF and SFF are the two ends of the exclusion-list spectrum, and `[p1,p2,…]` is the general
middle. **This must be documented in three places (FOOP-63 task): README.md, code comments, and
the snapshot tests themselves.**

### What the code already gives us for free (the good news)

The parser/lexer are **ahead of the compiler** for most of these:

- **`~~`, `??`, `..` are already LEXED** (`lexer.rs:140-156`, tokens `TildeTilde`,
  `QuestionQuestion`, `DotDot` in `token.rs:18,22,25`); the parser uses them only for `Display`
  (`parser.rs:1075-1082`). → all-results (FOOP-73) is mostly "wire tokens into the postfix parser
  + a collect-mode scan."
- **`↑` is fully PARSED** into `Astn::UpwardSearch` (`parser.rs:119,920`); only the **compiler**
  rejects it (`compiler.rs:29`). → recursion's `↑` (FOOP-93) is compiler+FIR, not parser.
- **Detachment (FOOP-63) `[p..]<<Expr>>` is NOT yet parsed.** The old `[..]{..}` form parses as
  `Astn::DetachmentBrane` (`parser.rs:144-168`, compiler-rejected `compiler.rs:30`), but the
  reframed syntax attaches patterns to an SF/SFF mark — so FOOP-63 needs *parser* work plus a
  `detachments` field on `StayFoolishFir`/`StayFullyFoolishFir` (`fir_kinds.rs:2021`/`:2082`, no
  fields today) and a matcher prefilter.
- **The one-engine search model** (`mod contextful_search`, `fir_kinds.rs:~1630-2008`) is the
  shared substrate for FOOP-53/63/73: `SearchPredicate::matches -> MatchOutcome{Approve,Reject,
  NkStop}` (line 1709) and `contextful_search_scan` (line 1973). FOOP-23.md:600 anticipated
  find-all: *"a find-all Matcher that collects instead of stopping runs over the same Navigator."*
- **`OperatorFir`** (`fir_kinds.rs:449`) models "settle operands, then compute in Rust"
  (`combine()`, line 483) — the pattern FOOP-83 follows, dispatched by creation identity.

So the bulk is **compiler + FIR + evaluator**, parser mostly done — the key "easy to implement"
finding.

---

## Engineering guidance (shared — every FOOP's FIR Impact should apply this)

Cross-cutting conventions for implementing these FOOPs, so each one doesn't re-derive them.
(Home for deeper contracts: `docs/ubc1/how/d0_fir_subtype_contracts.md`, `ubc_engineering.md`.)

### When to add a NEW FIR kind vs. extend an existing one

Add a **new FIR kind** (a new `FirKind` arm + struct in `fir_kinds.rs` + a `constanic_clone_at`
arm + a `*_nyes_transitions` test) only when the thing has **its own stepping behavior or its own
settled-value identity**. Extend an **existing** FIR (a field / an enum variant on its data) when
the behavior is the same shape with a parameter.

- **New FIR is warranted:** `CreationFir` (FOOP-33 — a new born-`Independent` value with identity);
  `StringFir`/`FloatFir` (FOOP-44 — new primitive value kinds); `CascadingSearchFir` (FOOP-24 —
  new stepping: run branches in order with anchor-propagation).
- **Extend, don't add:** the inverse matcher (FOOP-53 — a `negate` flag on `SearchPredicate`, not
  a new FIR); find-all (FOOP-73 — a scan-*mode*, reusing `SearchFir`/the engine); detachment
  patterns (FOOP-63 — a `detachments` field on the existing `StayFoolishFir`/`StayFullyFoolishFir`);
  boolean operators (FOOP-83 — dispatch inside `OperatorFir::combine`/application, not a new FIR);
  integer ops (FOOP-34 — new arms in `OperatorFir::combine`); computed index (FOOP-14 — a
  computed-offset child on `IndexFir`, or a thin new kind if a Braning-wait phase is cleaner).

### The two reuse patterns almost everything here rides

1. **`OperatorFir::combine` pattern** (`fir_kinds.rs:483-576`) — "settle operands (Braning), then
   compute in Rust." The model for FOOP-83 (booleans, dispatched by creation identity instead of
   op-name string), FOOP-34 (`**`, comparisons), and FOOP-44 float ops. Reuse the div-by-zero→NK
   shape (`fir_kinds.rs:557`) for guarded results.
2. **The one-engine search model** (`mod contextful_search`, `SearchPredicate::matches ->
   MatchOutcome`, `contextful_search_scan`) — extend via a **predicate variant** or a **scan mode**,
   never a parallel engine. FOOP-53 (`negate`), FOOP-73 (collect-mode scan), FOOP-63 (candidate
   prefilter), FOOP-24 (`And`/`Or` predicates), FOOP-44 (`Char` predicate demand) all plug in here.

### Value FIRs are born-Independent, and the clone rule

Primitive value FIRs (`IndepIntFir`, and new `StringFir`/`FloatFir`, `CreationFir`) are born
`Independent`. `constanic_clone_at` (`fir_kinds.rs:180-185`) already returns the **same `Rc`** for
`Independent` non-branes — so a new value kind is identity-preserving through clone for free; do
**not** add a `FirKind::X` arm that deep-copies it (that breaks identity — see FOOP-33 Gotcha #2).
Any new value kind still needs its own `constanic_clone_at` arm *only* if it can be non-Independent
(none here are).

### Mandatory tests when touching FIR/NYES

Per AGENTS.md: **every new FIR kind, and every new/changed NYES state or transition, MUST add or
extend a `<kind>_nyes_transitions` unit test.** These are unit tests (internal state), not approval
cases. FOOP-24 (`CascadingSearchFir`), FOOP-44 (`StringFir`/`FloatFir`), FOOP-14 (if a new kind)
each owe one. Approval `.foo` snapshots pin observable behavior; never auto-accept them.

### The unified "created lookup-table brane, FVM-shortcut" pattern (FOOP-73/63/83)

Booleans (FOOP-73), arithmetic (FOOP-63), and comparisons (FOOP-83) all share **one** design:
each operator is **declared via the Creation Postulate as a Foolish lookup-table brane** — e.g.
`i'lessthan = {A=1,B=2,result=True; A=2,B=3,result=True; …}` (countably infinite, never
enumerated), applied by a **search** (`i'lessthan~A=1~B=2#1`) — but the **FVM detects that specific
table-brane creation (by `Rc::ptr_eq` identity) inside `OperatorFir` and shortcuts to native Rust**
(IEEE float / integer / boolean logic). This keeps "no privileged layer" at the *declaration* level
while the FVM does the real work. When implementing any of the three, reuse this shape — do not
invent a separate operator mechanism per FOOP.

### Characterization-letter convention (FOOP-63)

`B'` (capital) = **brane** characterization; `b'` (lowercase) = **boolean**; `i'` = **integer**;
`f'` = **float**; `s'` = **string**. Case matters (`b'` boolean vs `B'` brane). A declared LHS
characterization also **demands** its RHS carry the same characterization (`B'decision =$ …` requires
a `B'` value).

### Miss-semantics is the shared substrate (FOOP-43)

Several FOOPs depend on FOOP-43's "search miss → ECONSTANIC (may recoordinate), found-`???` → NK
(propagates)" rule: FOOP-24 (detachment reject-all/`[*]`/naked-`<<>>`), FOOP-04
(cascade "fail" signal), FOOP-63 (characterization-demand miss → WOCONSTANIC-wait). Implement
FOOP-43 (the keystone) before them.

> **NB (2026-07-09):** the numbers below refer to the *renumbered* batch (see the number table at
> the top). Older number references elsewhere in this doc may predate the reorg — trust the table.

---

## Coherence review — FIRs, data structures, and stepping (2026-07-09)

A cross-FOOP audit: do the proposed structures hang together? Existing `FirKind` set: Brane,
Statement, Operator, Search, Index, StayFoolish, StayFullyFoolish, Concatenation, IndepInt, Nk,
Unknown, FoolRef.

### New FIR kinds proposed (and the verdict)

| Proposed | FOOP | New kind? | Verdict |
|----------|------|-----------|---------|
| `StringFir`, `FloatFir` | FOOP-63 | **Yes** | Correct — new primitive *values* with their own settled identity; born `Independent`; clone-for-free via `constanic_clone_at:180`. Each owes a `*_nyes_transitions` test. |
| `CascadingSearchFir` | FOOP-04 | **Yes** | Correct — genuinely new *stepping* (run branches in order, thread the fallback `FoolRefFir`). Not expressible as a predicate. |
| `↑` upward search | FOOP-34 | Maybe | Could be a new FIR or resolved to the home brane via the parent chain. Decide in the recursion exercise. |
| Computed index | FOOP-53 | Maybe | Prefer **extending `IndexFir`** with a computed-offset child (+ a Braning-wait phase) over a new `DynamicIndexFir`, unless the wait phase is cleaner as its own kind. |

**No new FIR** (correctly): inverse `!` + `&&`/`||` (FOOP-93 — `SearchPredicate` variants), find-all
(FOOP-14 — scan mode on `SearchFir`), detachment (FOOP-24 — fields on SF/SFF), boolean operators
(FOOP-73 — Foolish table search, *zero* new FVM machinery in the preferred design), integer math
(FOOP-83 — `OperatorFir::combine` arms).

### The `SearchPredicate` enum — the most-extended structure (watch for collision)

`SearchPredicate` (`fir_kinds.rs:1679`, today: Name, Value, NameValue, Index, Head, Tail) is
extended by **three** FOOPs. They must be designed together:
- **FOOP-93:** `negate` flag(s) on variants **and** `And(Box<_>, Box<_>)` / `Or(Box<_>, Box<_>)`.
- **FOOP-63:** a characterization gate — `Char { … }` (or a char-field on existing variants).
- **Consolidation:** these compose (a negated `And` of a `Char` and a `Value`), so the recursive
  `And`/`Or` tree should be the *outer* structure, with `negate` on leaves, and `Char` as another
  leaf. Recommendation: land FOOP-93's `And`/`Or`/`negate` first (it defines the tree shape),
  then FOOP-63 adds `Char` as a leaf — **do not** let FOOP-63 invent a parallel combination
  mechanism. Flag in both FOOPs.

### The Scope struct — extended by two FOOPs

`Scope` (`fir_trait.rs:55`, today `has_ancestral_sfm: bool` + current_statement/current_brane) gains:
- **FOOP-24:** `active_detachments: Vec<String>`.
- **FOOP-33 (Final):** already reshaped Scope work.
No conflict — additive fields. Both push in `step_inner` (`fir_trait.rs:347`), so coordinate the
handoff logic (SFM flag + detachments set at the same site).

### Stepping changes — where each FOOP touches the step loop

- **`OperatorFir::combine`** (`:483`): FOOP-83 (`**`, comparisons), FOOP-73 *fallback only*,
  FOOP-63 (float ops). One function, several new arms — keep the match arms tidy; consider a
  sub-dispatch by operand characterization once FOOP-63 lands.
- **The scan loop / `SearchPredicate::matches`** (`:1709`/`:1978`). Per-candidate pipeline is just
  **two stages, not a 4-gate stack** (Atlas): (1) the **detachment prefilter** (FOOP-24) — a
  *filter*, applied **before** the matcher; a skipped candidate is invisible. As a filter it is
  order-idempotent, so "before" is a free choice. (2) the **matcher** (`SearchPredicate::matches`)
  — which *internally* subsumes negation (FOOP-93 `!` is the predicate's own flag), the
  characterization demand (FOOP-63 `Char` is part of the search pattern), and `And`/`Or` (FOOP-93,
  `NkStop` halts). Find-all (FOOP-14) is a scan-*mode* wrapping this, not a gate. So the sequencing
  is: predicate tree (FOOP-93) is the matcher's shape, `Char` (FOOP-63) slots in as a leaf, `!` is a
  leaf flag; detachment (FOOP-24) is the one pre-matcher filter; find-all (FOOP-14) is the collect
  wrapper. `_TABLE~A=A` (value-pattern-as-search, FOOP-93) settles the value-child before compare.
- **Constanic clone** (`:155-253`): FOOP-43 Component 2 (drop `Search` `[1]`), FOOP-24 (SF/SFF
  strip already exists), FOOP-63 (new value kinds get the Independent-same-`Rc` path free).
  FOOP-43's `[1]`-drop is the only *behavioral* clone change — everything else is additive.
- **ECONSTANIC settle sites** (`:1273` + value/contexted equivalents): FOOP-43 Component 1
  (miss→ECONSTANIC) gates here. (Strict detachment would also have gated here but is backburnered —
  see FOOP-24's backburnered appendix; that hard case is *why* it's deferred.)

### Cross-cutting coherence flags

1. **`SearchPredicate` must be co-designed** across FOOP-93 + FOOP-63 (one tree, `Char`/`negate`
   as leaves) — the biggest "make it fit together" risk.
2. **ECONSTANIC is now semantically loaded** — FOOP-43 (miss), FOOP-24 (detach reject-all),
   FOOP-63 (char-demand) all read/write it. A missed search, a detached-away search, and a
   wrong-characterization search all land on ECONSTANIC but *mean* different things. Consider
   whether the FIR should record *why* it's ECONSTANIC (a reason tag) — parallels the
   FOOP-43 "why NK" helper. Open design question worth deciding before implementing the group.
3. **"Coordination sheds scaffolding"** is one principle with two faces: FOOP-24 (marker stripped
   on clone) and FOOP-43 Component 2 (search position stripped on clone). State them together in
   docs; they're the same idea.
4. **Boolean operators as table search (FOOP-73)** is the strongest coherence win — it needs *no*
   new FVM machinery, only the search features. But it inverts a dependency (booleans now need
   value+contexted search). Resolve the ordering (booleans after searches, or FVM-fallback first).

---

## 0. FOOP-33 — Creation Postulate → Booleans (WRITTEN, Final, UNIMPLEMENTED)

The base. See `FOOP-33.md`/`FOOP-33.plan.md`. Delivers: `⬤` creation (+ `{*}`), `CreationFir`
(Independent, identity = `Rc::ptr_eq`), three-valued `default_equal`, `Identifier`/
`Characterizations`, null-characterized name constants (`get_value()`→`NK("'…redefined")`),
and `system.foo` as the built-in root brane defining `'True`/`'False`.

**Code state (verified):** none of it exists yet — no `CreationFir`, no `Astn::Creation`, no
`system.foo`, no `Identifier`. Its plan is ready to execute top-to-bottom.

**Why it's item 0:** FOOP-83 (boolean operators) is meaningless without `True`/`False` and creation
identity. Everything else is independent of it.

---

## 1. Boolean operators — and / or / not / nor / xor

**Idea.** Declare the five operators as **creations** in `system.foo` (`and = b'⬤`, …), but the
**UBCa FVM performs the logic** — their Foolish body is absent/ignored. Identity is Foolish-
native (a creation); behavior is FVM-native. Deliberate departure from FOOP-33's "no privileged
layer" (state it plainly).

**Application form.** `{T,F} and` is **brane-concatenation-as-function** (RPN — ADVANCED_FEATURES
§Brane Concatenation, `parser.rs` concatenation at line 355). `{T,F}` is the argument brane;
`and` is a name that resolves to the system.foo creation. `not` is unary (`F not`); the other
four are binary over a 2-arg brane.

**Dispatch by identity (the elegant reuse).** The FVM recognizes an operator by asking whether
the resolved value is the **same rust `Rc`** as `system.foo`'s `and`/`or`/… — i.e. FOOP-33's
`Rc::ptr_eq` creation identity. No op-name strings, no privileged keyword.

**Implementation analysis.**
- Trigger site is the concatenation step (`ConcatenationFir::fir_op_step` Braning branch,
  `fir_kinds.rs:2162`) — or a small dedicated application FIR. When the *last* element of a
  concatenation resolves to a known boolean-operator creation, don't merge-as-brane; instead
  compute: pull the args from the preceding element (the arg brane), compare each to
  `system.foo`'s `True` via `default_equal` (FOOP-33), produce `True`/`False`.
- Needs handles to the system.foo creations (`True`, `False`, `and`, …) — natural, since
  system.foo is compiled once at startup (FOOP-33). Keep those `Rc`s where the evaluator can
  reach them for identity checks.
- `not` unary vs binary arg-shape: decide arg extraction (see open Qs).
- Model to copy: `OperatorFir::combine` (settle operands → compute in Rust), `fir_kinds.rs:483`.

**Testing.** Unit: identity dispatch (is-this-`and`?), each truth-table row for all 5 operators,
non-boolean args → NK. Approval: `{T,T}and`→T, `{T,F}and`→F, `T not`→F, full truth tables for
nor/xor; `{T,3}and`→NK. Comprehensive interaction with value search and system.foo resolution.

**Depends on:** FOOP-33 (True/False, creation identity, default_equal, system.foo). **Hard dep.**

**Open questions.** Arg extraction: positional (`#0`/`#1`) vs head/tail (`^`/`$`) vs "the 2 args
of the preceding brane"? Result for non-boolean/insufficient args (NK likely). Are user-written
operator bodies an error, ignored, or a future extension? Short-circuit for `and`/`or`?

---

## 2. Detachment = a PARAMETERIZED stay-foolish marker — `[p1,p2,p3]<<Expr>>`

**REFRAMED (Atlas, 2026-07-08/09).** Detachment is **not** `[..]{..}` brane recoordination. It is
a **parameterized stay-foolish marker** — usable on **both** SF (`[p1,p2,p3]<Expr>`) and SFF
(`[p1,p2,p3]<<Expr>>`). The prefilter behavior of the `[...]` list is identical on either marker;
**what differs is the marker's *default* (empty-`[...]`) behavior** — see the spectrum below.
Detachment is **not bound to any brane — it must decorate a stay-foolish marker.** The patterns
are the **detachments**; store them on the FIR as `detachments`. Standard name: "`[a,b,c]<<Expr>>`
is an SFF marker parameterized by patterns a, b, c."

**Semantics (a search prefilter, not a logical op).** Evaluate `Expr` such that **every search
that `Expr` or any of its children ever performs** — with match pattern `q` — **auto-skips /
rejects any candidate for which at least one of p1/p2/p3 also matches, before even testing `q`.**
Literally a per-candidate prefilter (`pi matches candidate ⟹ skip; don't test q`), not a boolean
combination on `q`. A detachment thus *hides* candidates from all searches inside the marked
expression.

**Naked `<<Expr>>` is ALIASED to `[*]<<Expr>>`** (Atlas, 2026-07-08). Bare SFF *means*
detach-everything: skip every candidate from every search inside → each search exhausts its
stream → settles ECONSTANIC (see FOOP-43) → resolves on coordination. So **SFF is a special case of
parameterized SFF**, not a separate concept. This is clean *because* of FOOP-43:
**exhaustion → ECONSTANIC** (Atlas: reject-all "results in ECONSTANIC of course"). The earlier
"anchored-miss→NK seam" is **dissolved** — with FOOP-43, a search that exhausts its candidates settles
ECONSTANIC regardless of anchoring, so `[*]<<E>>` and `<<E>>` are equal by construction.

**Implementation strategy — keep existing SFF; new code is ONLY for specific detachments.**
(Atlas, 2026-07-08.) Do **not** rebuild naked `<<>>` as `[*]<<>>` under the hood:
- Naked `<<E>>` → keep its **existing implementation** verbatim.
- `[*]<<E>>` → **forwards to** the same existing naked-`<<>>` code (all-detach = current SFF).
- **Only `[p1,p2,…]<<E>>` (a real, non-`*` pattern list) gets the new path**, and that path is an
  **exclusion list**: every search resolves **normally** *except* candidates matching a listed
  pattern. So the new prefilter engages iff `detachments` is non-empty AND ≠ `[*]`. This
  preserves existing SFF behavior exactly and makes the new logic a pure *subtraction* of
  specific candidates from otherwise-normal search — lowest-risk, easiest to test.

**SF and SFF are the two EXTREMES of the detachment spectrum — RESOLVED (Atlas, 2026-07-09).**
The previously-open "what does bare `<E>` equal" question is answered: the two markers' *default*
(undetached) behaviors differ, and detachment unifies them.
- **`<E>` (SF) ≡ `[]<E>`** — the **empty** detachment: detach *nothing*; every search resolves
  normally. (This is why SF is not describable purely as a prefilter — its default filters
  nothing; the SF machinery does its *other* thing, unchanged.)
- **`<<E>>` (SFF) ≡ `[*]<<E>>`** — the **full** detachment: detach *everything*; every search
  exhausts → ECONSTANIC (FOOP-43) → coordination-deferred.

So `[p1,p2,…]` is the general middle; SF and SFF are its endpoints (`[]` and `[*]`). **TASK
(document in THREE places, repeatedly): README.md, code comments, and the snapshot tests
themselves** must each state this SF=`[]` / SFF=`[*]` framing. (Atlas explicitly wants it in all
three, so a reader of any one surface learns it.)

**Constanic-copy mandate.** When constanic-cloning a `[p..]<E>` item, the (parameterized)
SF-marker is **removed just like a normal SF-marker** — "coordination frees everything." Already
true mechanically: `constanic_clone_at` (`fir_kinds.rs:155-179`) strips SF/SFF at the top and
returns the inner content. So detachments do **not** survive coordination on their own — *unless*
re-detached by another parameterized SF-mark wrapping.

**Implementation analysis.**
- **FIR:** `StayFoolishFir` / `StayFullyFoolishFir` currently have no fields
  (`fir_kinds.rs:2021`, `:2082`). Add `detachments: Vec<String>` (patterns; or `Vec<FirRef>`) to
  **both**. A parameterized marker is just SF/SFF with a non-empty `detachments`. Naked SFF is
  the `[*]` (detach-all) case (see aliasing above).
- **Scope handoff:** SF-naivety flows via `Scope.has_ancestral_sfm: bool` (`fir_trait.rs:55`),
  set in `step_inner` (`fir_trait.rs:347-348`) when stepping under a StayFoolish. Extend the
  Scope to also carry the **active detachment patterns** accumulated from all enclosing
  parameterized SF/SFF marks (e.g. `Scope.active_detachments: Vec<String>`).
- **Prefilter locus (shared with FOOP-53):** in the one-engine model — `SearchPredicate::matches` /
  `contextful_search_scan` (`fir_kinds.rs:1709/1973`) — skip a candidate if any active detachment
  pattern matches it, *before* the real predicate. **Same locus** as the inverse-matcher `!`
  (FOOP-53): both are candidate prefilters. They compose cleanly.
- **Parser:** today `[..]` parses only as `[..]{..}` → `Astn::DetachmentBrane` (`parser.rs:144`).
  The new form is `[..]<<Expr>>` (or `[..]<Expr>`): brackets then a stay-foolish mark. Parser
  work: recognize `[patterns]` immediately preceding an SF/SFF mark and attach the patterns to
  the marker node. (Decide the fate of the old `[..]{..}` `DetachmentBrane` form — repurpose or
  deprecate.)
- **DOCUMENTATION TASK (mandated, three surfaces):** the SF≡`[]` / SFF≡`[*]` spectrum framing
  must be written into **(1) README.md, (2) code comments** (on the SF/SFF FIRs and the
  detachment prefilter), **and (3) the snapshot tests themselves** (a comment in the relevant
  `.foo` inputs / an explanatory snapshot). Atlas wants it repeated in all three so it is
  discoverable from any surface. This is a required plan checkbox for FOOP-63.

**Testing.** Unit: a `[a]<<...>>` marker sets active detachments; a search under it skipping a
candidate whose name matches `a`; **naked `<<E>>` ≡ `[*]<<E>>`** and **`<E>` ≡ `[]<E>`** (SF
default = detach nothing); detachment on SF and SFF apply the same skip for a given `[...]`.
Constanic-clone strips the parameterized marker (same as bare SF/SFF). NYES transitions if a
distinct FIR kind is introduced. Approval: `[tmp.*]<<a?x>>` where candidate `tmp_k` is hidden;
`<<E>>` = `[*]<<E>>` and `<E>` = `[]<E>` demonstrations (with the mandated explanatory comments).

**Depends on:** **FOOP-43 (exhaustion→ECONSTANIC)** — needed so reject-all / naked-`<<>>` settle
ECONSTANIC, not NK. Otherwise nothing new (SF/SFF infra + one-engine matcher exist). Composes
with FOOP-53 (shared prefilter) and FOOP-73. **Recursion (FOOP-93) does NOT depend on this** (see
FOOP-93).

**Open questions.** Whether detachment patterns match on name only, or also value/characterization
(mirror the search predicates?). Old `[..]{..}` `DetachmentBrane` disposition. Must `[..]` be
followed by a stay-foolish mark (error otherwise)? (The bare-`<E>` question is RESOLVED — it is
`[]<E>`, see the spectrum above.)

---

## 3. Macros (RESEARCH FOOP)

**Idea.** Given the language's name/value matching (searches, value search, characterizations),
spec **how to write macros** in Foolish. Research first.

**Research directions (from Atlas + reading).**
- Rust's macro systems (declarative `macro_rules!` pattern-matching vs procedural syntax-tree
  macros) as prior art. Syntax-tree search / matching over the AST or FIR.
- Foolish already has a *matching* substrate (SearchPredicate, name/value/characterization
  matching, and — once FOOP-73 lands — all-results). A macro system might be expressible as
  "match a pattern over a brane's statements, rewrite/expand." Explore whether macros are
  *branes that transform branes* (very Foolish) vs a separate expansion phase.
- Where expansion happens: parse-time (AST), compile-time (Astn→FIR), or a FIR that rewrites.

**Implementation analysis.** Too early — this is a spec/research FOOP; deliverable is a design,
not code. Note: leans on FOOP-73 (all-results) for "find every statement matching a pattern," and on
characterizations (FOOP-33) for tagging macro inputs.

**Depends on:** benefits from FOOP-73 (all-results) and FOOP-33 (characterizations). Not a hard dep
for the research, but the *design* will reference them.

**Open questions.** Everything — hygiene, expansion phase, recursion in macros, whether a macro
is a first-class brane, syntax. This is the least-defined; keep it last.

---

## 4. Inverse matcher — `!`

**Idea.** A `!` prefix **negates a search's match predicate** — matches exactly what the
un-negated search rejects. Applies to ALL searches.

**Placement (strict).** After `&` (if present), before the search operator:
`[anchor] [&] [!] <search-op> <pattern>`. **Per-gate** in the combined name+value form:
`b!~a.*!=5` = names NOT matching `a.*` AND value NOT 5 (`!~a.*` negated name gate, `!=5` negated
value gate — FOOP-23 form 4 `~name=value`).

**Implementation analysis.**
- Cleanest impl in the one-engine model: negation flips the predicate outcome. Either add a
  `negate: bool` to the `SearchPredicate` variants (or a `Not(Box<SearchPredicate>)` wrapper),
  and in `matches` (`fir_kinds.rs:1709`) swap `Approve<->Reject` for negated gates. `NkStop`
  stays `NkStop` (an NK candidate is still incomparable, not "matched").
- Parser: add a `Bang` token and consume `!` in postfix search (`parse_postfix_expr`,
  `parser.rs:573`), after the optional `&`, before the op. Thread a `negated` flag through
  `Astn`'s search variants (RegexpSearch/ValueSearch) and into the compiler's SearchFir build.
- **GOTCHA:** `!!` is the comment marker (`lexer.rs:100`; also `!!!`, `#!`). Lex a single `!`
  (followed by a search op char) as `Bang`, but `!!` must stay a comment. Careful lex tests.

**Testing.** Unit: `SearchPredicate` negated outcomes (Approve↔Reject; NkStop unchanged); per-
gate negation in NameValue. Parser: `!~`, `&!~`, `!=`, `!!` still a comment, `!` placement. Approval:
`b!~a.*` finds non-`a`-names; `b!~a.*!=5` combined; interaction with `&`.

**Depends on:** nothing (rides existing engine). Composes with FOOP-73. Independent of FOOP-33.

**Open questions.** Miss outcome of a negated ANCHORED search (NK vs nothing) — mirror FOOP-23
anchored-miss→NK. Does `!` before a positional/head-tail op (`!FOOP-04`, `!^`) mean anything useful?

---

## 5. All-results (find-all) search — `~~`, `??`, …

**Idea.** **Doubling the search operator makes it find-all**; results collected **into a brane**
(not the single-match two-child result). General across the family: `~~` (fwd name), `??` (bwd
name), and by extension value forms and combinations with `&` and `!`.

`a~~tmp.*` = all statements in `a` whose name matches `tmp.*`, returned in a brane.

**Implementation analysis.**
- **Tokens already lexed** (`TildeTilde`, `QuestionQuestion`, `DotDot`; `lexer.rs:140-156`).
  Parser consumes them only for `Display` today — wire them into `parse_postfix_expr`
  (`parser.rs:573`) as find-all variants (new `Astn` search variant or a `find_all: bool` flag).
- **Scan already anticipated:** add a collect-mode scan alongside `contextful_search_scan`
  (`fir_kinds.rs:1973`) — same Navigator, same Predicate, but on `Approve` push into a results
  Vec and CONTINUE instead of `return Found`. Result = a brane of the collected statements.
- Result shape: a brane whose children are the matches. For `&`-chaining (FOOP-23), each entry
  should carry its position — likely each result is (or wraps) a `FoolRefFir` like the single
  search's `[1]` child. Decide whether the find-all brane's entries are values, statements, or
  FoolRefFir-bearing.

**Testing.** Unit: collect-mode scan returns all Approves in order; empty → empty brane.
Approval: `a~~tmp.*` → brane of tmp_* ; `~~`/`??` ordering; find-all + `&`; find-all + `!`
(`a!~~tmp.*`). NYES for the find-all result.

**Depends on:** nothing (rides existing engine + already-lexed tokens). Composes with FOOP-53.
FOOP-23 reserved this (`??`/`//` out of scope) → this delivers it. Independent of FOOP-33.

**Open questions.** `//` (fwd find-all + parents) conflicts with division — drop or respell.
Result-brane entry shape (value vs statement vs FoolRefFir) and how `&` chains off it. Ordering
(scan order). Empty result = empty brane vs NK. Does find-all cross into parent branes (the
vintage `//` "global" idea) or stay local?

---

## 6. FOOP-93 — Recursion Upgrades (AFTER the full search suite)

**Deliberately under-specified — we'll know better when we get there (Atlas, 2026-07-09).** The
concrete content of this FOOP is discovered *during* it, not pinned now.

- **Ordering:** comes **after the full search suite** (FOOP-43/53/63/73/24) — recursion leans on
  a mature search substrate; sequenced last among features.
- **First task (part of the FOOP):** **write many recursive algorithms in Foolish-as-it-exists-
  then — a Fibonacci computer first, then a series of standard algorithms.** Writing them reveals
  the syntactic sugars and recursion upgrades actually needed. (These programs are also the
  approval-test suite.) The rest of the FOOP follows from what that exercise surfaces.

  **Candidate algorithm list (~1–2 dozen; write each as a Python or Rust reference first, then
  port to Foolish).** This list is legitimately plannable now — it's the *what to write*; the
  *how Foolish expresses it* is what's deferred.
  - *Numeric:* Fibonacci (naive + memoized/accumulator), factorial, GCD (Euclid), integer power
    (fast exponentiation), Ackermann, sum-to-n, digit-sum, Collatz-length, is-even/is-odd
    (mutual recursion).
  - *Combinatorial:* binomial coefficient, Towers of Hanoi, permutations/subsets generation,
    Catalan numbers.
  - *Structural (branes / lists / trees):* brane/list length, reverse, map, fold/reduce, member,
    flatten a nested brane, tree depth, tree traversal (pre/in/post), binary search over a sorted
    brane, quicksort/merge-sort.
  - *Classic:* Euclid's GCD (again, as chained-search style), string/brane palindrome check.
  Aim for a **dozen or two DISTINCT recursion shapes** (linear, binary/tree, mutual, accumulator,
  generative), so the friction covers the full space — not 20 variations of one shape.
- **`↑`** (`Astn::UpwardSearch`, parsed but compiler-rejected at `compiler.rs:29`) is the likely
  enabling primitive (self-reference); confirm during the exercise.
- **NO cycle detection.** `infinite_loop.foo` (`{f1={f1};stuck=f1}` → `NK(ITERATION-EXCEEDED)`) is
  **accepted** behavior — a self-referential waiting-cycle legitimately runs to the step budget.

**Depends on:** the full search suite (FOOP-43/53/63/73/24) and likely FOOP-34/44. `↑`.
**Open questions:** deferred to the write-the-algorithms first task.

---

## 7. Search-miss → ECONSTANIC, not NK (FOUNDATIONAL — revises FOOP-23)

**Idea (Atlas, 2026-07-08).** "When a search exhausts its candidate stream, the result is
ECONSTANIC, not NK. It should be that way." The current **anchored-miss → NK** rule is considered
a **bug**.

**The key discriminator — found-but-NK vs NOT-found:**
- `{b=???, a=b.c.d}` → `a` **stays NK**. `b` *is found*; its value is `???` (NK). Deepening `.c`
  into a genuinely-unknowable value is unknowable → **NK propagates**. Terminal. Correct.
- `{a=b.c.d}` (no `b`) → `a` is **WOCONSTANIC**. `b` is *not found* — the search **missed**. A
  miss is **ECONSTANIC** (may recoordinate) → `.c.d` wait → `a` WOCONSTANIC.

So: **search MISS (stream exhausted, no match) → ECONSTANIC**; **search FINDS a value that is NK
→ NK propagates.** The rule turns on the *source* of the NK, not merely "the anchor is NK."

**The bug (verified).** Anchored miss currently settles NK (`fir_kinds.rs:1278`), so a not-found
`b` is **indistinguishable** from a found-`???` `b` at the deepen point (`fir_kinds.rs:1252`
checks only `anchor nyes == Nk`, ignoring provenance). The whole chain then forces NK.

**The fix falls out for free.** Make miss settle **ECONSTANIC** (not NK) at `fir_kinds.rs:1277`.
Then not-found → ECONSTANIC → chain WOCONSTANIC (via the existing `deepest_econstanic_in_chain`,
`fir_kinds.rs:86`), while found-`???` → NK → chain NK. The two cases separate automatically once
miss ≠ NK; the `{b=???}` case already works today.

**Conflicts with documented rule.** FOOP-23.md:211 and AGENTS.md:576-578 state "anchored miss →
NK (provably not in that brane)." This FOOP **revises** that: an absent name is *not*
provably-absent while context is incomplete → ECONSTANIC. **NK survives only for**
genuinely-unknowable (a found `???`) and provable-impossibility (e.g. `#N` out of a *settled
finite* brane — verify that case is preserved). Update FOOP-23 + AGENTS.md accordingly.

**Implementation analysis.** One-line settlement change (`fir_kinds.rs:1277` NK→Econstanic for
anchored miss) plus verifying the chain/`resolve_anchor` NK-check now only fires for found-NK,
not miss. Careful with `value_search_step` miss paths too (FOOP-23 already had a miss-audit note,
FOOP-23.md:1051-1054).

**Testing.** Unit: `{a=b.c.d}` (no b) → a WOCONSTANIC; `{b=???, a=b.c.d}` → a NK; a bare `a?zzz`
(anchored miss) → ECONSTANIC not NK; `#N` out-of-range on a settled brane → still NK (or the
decided outcome). Approval: re-review every snapshot currently NK-by-absence (expect diffs —
treat as semantic review).

**Depends on:** nothing. **Prerequisite for FOOP-63** (detachment reject-all / naked-`<<>>` need
exhaustion→ECONSTANIC). Foundational — do EARLY. Relates to the FOOP-51 residual NYES issues.

**Open questions.** Does ANY search still settle NK by mere absence, or only found-`???` /
provable-impossibility? Exact list of "provable-impossibility" cases that keep NK. Snapshot
churn scope.

---

## 8. FOOP-14 — Computed / dynamic index `#${...}`

**Idea (Atlas, 2026-07-09).** Permit an **expression** in the indexer instead of a literal `#N`.
Syntax `#${...}`: the FIR **evaluates the brane to the right of `$`**, **retrieves its last
element** (its tail), **expects a number**, then **searches `#`** with that number as the offset —
exactly the usual index. Example: `x#${a; b; 3}` → evaluate `{a; b; 3}`, take the tail (`3`), do
`x#3`.

**Implementation analysis.**
- **Today the offset is a fixed literal.** `IndexFir.offset: i32` (`fir_kinds.rs:1315`) is set at
  parse time; `parse_seek_index` (`parser.rs:894`) parses only a bare integer after `#`. The
  dynamic form is a genuinely new capability: the offset becomes a **computed child**.
- **Parser:** in the `#` postfix path (`parser.rs:643`) and `parse_seek_index`, branch on the next
  token — if `$`, parse `${brane}` and emit a new AST variant (e.g. `Astn::DynamicSeek { anchor,
  index_brane }`) instead of `Astn::Seek { offset }`. The `$` token exists (`token.rs:20`).
- **FIR:** either a new `DynamicIndexFir` or extend `IndexFir` with an optional computed-offset
  child. Its step (**new Braning phase**): push the index brane as a task → when it settles, take
  its **tail element** (reuse the HeadTail/`$` retrieval) → read `as_i64` (`fir_kinds.rs:379`) →
  set the offset → run the ordinary `SearchPredicate::Index(offset)` scan (unchanged engine).
- **NYES:** unlike the literal index (which can resolve immediately), the dynamic index must
  **wait for the child brane to settle** first. Add/extend the `IndexFir` (or new-kind)
  `*_nyes_transitions` test — this is a required checkbox per AGENTS.md.

**Testing.** Unit: `${...}` tail extraction + offset wiring; the Braning-wait progression; tail
not-a-number path. Approval: `x#${a;b;3}` == `x#3`; a computed negative offset; tail is `???` →
(NK? — open); the index brane is itself an expression (`x#${1+2}` if the tail can be arithmetic).

**Depends on:** nothing (reuses `$`/tail, `as_i64`, the Index engine). **Self-contained**, can be
done any time, independent of the other FOOPs.

**Open questions.** Tail not a number → NK or alarm? Negative computed offsets. Is the brace
mandatory (`#${...}`) or is `#$expr` also allowed? Does the "last element" mean the brane's tail
*statement's value*, or the raw tail element? (Assume the tail statement's settled value via
`as_i64`.)

---

## 9. FOOP-24 — Boolean-combinator "beefy" search — `&&`, `||`, `|`

**Standardized vocabulary (Atlas, 2026-07-09) — use these exact words in README + code comments:**
- **`&&` and `||` are "matcher boolean operators"** — boolean ops on *matcher results* (a
  combining matcher).
- **`|` is the "cascading connector for search"** (the cascading connector).
- **`&` is the "continuation connector for search"** (the existing contexted `&` — named here for
  the family).

**→ FOOP-24 PLAN TODO: write these three terms into README.md and the relevant code comments**
(on the `SearchPredicate` combinators, the cascade FIR, and the existing `&` contexted path).

**Idea (Atlas, 2026-07-09).** Two kinds: the **matcher boolean operators** `&&`/`||` (easy — pure
boolean logic on matcher Approve/Reject results, no control flow), and the **cascading connector**
`|` (harder — a stateful wrapper FIR with anchor-propagation).

**`&&` / `||` — matcher boolean operators (the EASY part).** They combine matcher *results* into
a composite matcher, tested **per candidate**: `And` = Approve iff both branches Approve, `Or` =
Approve iff either. `(=10 || =4)~a.*` = "name starts with `a` **and** (value == 10 **or** value ==
4)". This is the natural generalization of the atomic `NameValue` predicate (`fir_kinds.rs:1745`,
hardcoded name-gate ∧ value-gate on one candidate) into a recursive predicate tree — no anchor
state, no sequencing, just boolean composition of matcher outcomes.

**`|` — the cascading connector for search (runtime control-flow, NOT a matcher).** `(=10 | =4)`
returns 4 **only if 10 was not found**: run search-1; if it *fails* (misses → ECONSTANIC per
FOOP-43), fall back to search-2. A short-circuit fallback between *whole searches*, not a
per-candidate matcher. Needs a **special wrapper FIR** (`CascadingSearchFir` holding the ordered
branches) — parser-changed but inline; not a new precedence layer. This is the **hard part** of
FOOP-24 (contrast the easy matcher boolean operators `&&`/`||`).

**Cascade anchor-propagation (the subtle part, Atlas 2026-07-09).** `A | B | C` is **not** a flat
or-else: each fallback branch **resumes from the nearest earlier branch that established a
position.** The wrapper threads a running "current fallback anchor/position" down the branches:
- A runs from the original anchor.
- A fails → B runs from **A's** anchor/position.
- If B *establishes a position*, it becomes the current position → on fallback, C resumes from
  **B's**. If B *also fails to establish one*, current stays A's → C falls further back to **A's**.

So: `C` searches A's anchor if A and B both fail; C searches B's anchor if only B fails (i.e. B
got far enough to set a position). This is why it needs a stateful wrapper, not a predicate.
**Reuses FOOP-23 `FoolRefFir`:** "the position a branch established" *is* the `FoolRefFir` (`[1]`
child) — the same position-carrier `&`-searches read — so "resume from B's position" is exactly a
contexted-search resume off B's `FoolRefFir`.

**Also needs a bare leading value predicate `=N`** — today value search is only `~=`/`?=`;
`(=10||=4)` uses a bare `=10`. New little form.

**Implementation analysis.**
- **Matcher boolean operators (`&&`/`||`) — the easy part:** add
  `SearchPredicate::And(Box<_>, Box<_>)` / `Or(Box<_>, Box<_>)` to the enum (`fir_kinds.rs:1679`).
  `matches` recurses on matcher results: `And` = Approve iff both Approve; `Or` = Approve iff
  either. **NkStop composition is the design question** — e.g. does `Or` swallow an NkStop from
  one branch if the other Approves? (Lean: `Or` prefers a concrete match over a branch's NkStop;
  `And` propagates NkStop. Decide in the FOOP.)
- **Cascade FIR (`|`):** a new `CascadingSearchFir` with ordered branch children, **carrying a
  running fallback anchor/position**. Step: run branch[0] from the original anchor; if found →
  result (and its position becomes the current fallback position for later branches); if it
  settles a **miss** (ECONSTANIC, FOOP-43) → run next branch **from the current fallback
  position** (the nearest earlier branch that established one — see anchor-propagation above).
  Reuse the `FoolRefFir` position-carrier and the contexted-search resume path. Depends on
  FOOP-43 making "fail" a clean ECONSTANIC signal.
- **Parser:** `&&`/`||`/`|` are all **net-new tokens** (none exist; no `Pipe` token today). The
  combine-block `(...)` **leads** (anchor/expression position) — `(=10||=4)~a.*`. This is the
  crucial parseability point: a **leading** `(...)` is in expression position, so it does **not**
  hit the regex-group path (`parser.rs:797`, where `(` inside a *pattern* is a regex group). So
  the combine-block is an **anchor-position construct**; `~(=10||=4)` in *pattern* position would
  collide with regex groups and is disallowed/needs care.

**Testing.** Unit: `And`/`Or` predicate recursion truth table incl. NkStop composition; the
cascade FIR runs branch-2 only on branch-1 miss; bare `=N` predicate. Approval: `(=10||=4)~a.*`;
`(=10 && ~a.*)`; `(=10 | =4)` fallback (10 present → 10; 10 absent, 4 present → 4; neither →
miss); compose with `!` (FOOP-53) negating a combined predicate, and `~~` (FOOP-73) find-all over
a combined predicate.

**Depends on:** the one-engine model (`SearchPredicate`, matcher). Cascade `|` depends on
**FOOP-43** (miss→ECONSTANIC as the "fail" signal). Composes with FOOP-53 (`!`) and FOOP-73
(`~~`). Best done after FOOP-43/53 so the predicate-negation and miss-signal are settled.

**Open questions.** NkStop composition for `And`/`Or`. **What exactly counts as a branch having
"established a position"** vs "failed to establish one" — the cascade's anchor-propagation hinges
on this (a partial-anchor / partial-resolve concept the wrapper must define; likely "the branch
resolved its anchor far enough to produce a `FoolRefFir`, even if the final predicate missed").
Does `|` cascade compose with `&&`/`||` inside one `(...)` (mixed precedence: `(=10 && ~a |
=4)`)? Is the combine-block allowed *only* leading, or also mid-chain? Bare `=N` — anchored or
unanchored by default? Does `||` differ observably from cascade `|` when both branches could
match (`||` is per-candidate → first candidate matching either; `|` runs whole search-1 first) —
document the distinction.

---

## 10. FOOP-34 — Strengthen integer math (`**`, `<`, `>`, `<=`, `>=`)

**Idea (Atlas, 2026-07-09).** Add the missing integer operators.

**What's ALREADY DONE (do NOT re-spec).** `OperatorFir::combine` (`fir_kinds.rs:531-576`) already
computes `+ - * / %` and unary `-`, with `/` and `%` guarding div-by-zero → NK. So **multiply
(`*`) and modulus (`%`) already exist** — Atlas's list named them, but they're implemented and
tested.

**The REAL work:**
- **Exponent `**`** — no token, no combine arm. **Self-contained** (pure i64), can ship
  independently of everything. Impl: a `**` arm in `combine` (i64 pow; guard overflow /
  negative exponent → NK or alarm).
- **Comparisons `<` `>` `<=` `>=`** — none in `combine`. **Return BOOLEAN `True`/`False`**
  (Atlas's decision), so they push the system.foo `True`/`False` creation by identity (like
  FOOP-83). → **DEPENDS ON FOOP-33 (creation) + FOOP-83 (booleans)**; order after FOOP-83.
  Composes with the boolean operators: `(3<5) and (x<y)`.

**Parseability snags (comparisons):**
- **`<` collides with the SF-marker `<expr>`** (`parser.rs:938`, `<` … expect `>`), and `>` with
  SFF-close / concat-continuation (`parser.rs:375`). So `a < b` (comparison) vs `<b>` (SF-marker)
  needs disambiguation — same family of collision as `{*}`/brane and `|`/regex.
- **`<=` / `>=` tokens do NOT exist** (`token.rs` has `Lt`/`Gt`/`LtEqGt`=`<=>`/`LtLt`/`GtGt`, not
  `LtEq`/`GtEq`) — new tokens to add.

**Testing.** Unit: `**` (incl. overflow/neg-exp); each comparison → `True`/`False` by identity;
comparison of unequal types (deferred to FOOP-34-with-strings? — see FOOP-44). Approval: `2**8`;
`3<5`→True, `5<3`→False, `5<=5`→True; `(3<5) and (2<1)`. NYES unchanged (operators already have it).

**Depends on:** exponent — nothing. Comparisons — **FOOP-33 + FOOP-83** (booleans). Split
possible: ship `**` now, comparisons after FOOP-83.

**Open questions.** `**` overflow → NK or wrap or alarm? Negative exponent (→ NK, since integer)?
The `<`/`>` SF-marker disambiguation (whitespace-sensitive? require spaces around comparison?).

---

## 11. FOOP-44 — Primitive Characterization (type system: `i'` int, `s'` string, `f'` float)

**Idea (Atlas, 2026-07-09).** The **Primitive Characterization** FOOP: adopt **characterizations
as primitive datatype tags**, turning FOOP-33's (stored-but-inert) `Characterizations` into an
active type system, and deliver the three primitive types + their operators:
- **`i'` — integer** (exists as `IndepIntFir`; gains its tag + primitive dispatch).
- **`s'` — string** (NEW: `StringFir` + string literals).
- **`f'` — floating point** (NEW: `FloatFir` + float literals). `CREATION.md` already sketches
  `f = c'⬤` and `0 = f'⬤`.

**THE CORE TENSION (Atlas, 2026-07-09 — this is the heart of the FOOP).** A search can now
**demand a characterization**, and that demand determines **whether a brane resolves at all.**
When an operator/search demands a properly-characterized parameter (e.g. `+` searching for an
`i'`-characterized operand) and **cannot find one**, the result is **WOCONSTANIC** — *waiting*,
not failed. The brane is waiting for a correctly-characterized parameter to appear via
coordination/recoordination.

This ties the type system directly into the NYES machine and FOOP-43: **a wrong-characterization
candidate is a MISS**, not a match. Not-finding-a-correctly-typed-value → **ECONSTANIC** (may gain
value on recoordination), and dependents → **WOCONSTANIC**. So characterization is **not** a crude
verify-then-NK gate — it is a **`SearchPredicate` dimension** ("find something characterized
`i'`"), and the type system *is* search-with-a-characterization-predicate. This composes with
FOOP-43 (miss→ECONSTANIC), FOOP-24 (`&&`/`||` combine a char-gate with name/value gates), and the
whole one-engine model. **→ FOOP-44 depends on FOOP-43** (the miss→ECONSTANIC/WOCONSTANIC
semantics are what make char-demand "wait" instead of "die").

**The mechanism.** FOOP-33 made characterizations *searchable but semantically inert*; FOOP-44
makes them **type tags the matcher demands.** `get_value_primitive()` replaces the single
hardcoded `as_i64()` (`fir_trait.rs:114`, `fir_kinds.rs:379`): dispatched by characterization →
`i'` integer / `s'` string / `f'` float primitive. Every current `as_i64()` call site
(value-search matcher `fir_kinds.rs:1738/1739/1767/1768`, arithmetic operands, etc.) becomes
characterization-aware. A char-mismatch is a search miss (→ WOCONSTANIC-wait), not necessarily an
immediate NK. (`3 + "x"` where no `i'` operand can ever be found — decide: WOCONSTANIC forever, or
NK when provably no `i'` exists? — relates to FOOP-43's not-found-vs-found-NK discriminator.)

**What it introduces.**
- **String + float literals** — neither exists today (grep: no string/float lexing/AST). Add
  string and float tokens, `Astn::StringLit`/`Astn::FloatLit`, and `StringFir`/`FloatFir` values
  (parallel to `IndepIntFir`).
- **`get_value_primitive()`** — the characterization-dispatched accessor (above).
- **Operators for the new types** — float arithmetic (`+ - * / %` on `f'`, and `**`), string
  operators (concat? length? equality — decide minimal set), and cross-type characterization
  **verification** on every operator.
- **A characterization gate in the one-engine matcher** — likely a `SearchPredicate::Char`
  dimension (or a char-field on existing predicates), so "demand `i'`" is a matcher predicate
  that composes with Name/Value (and with FOOP-24 `&&`/`||`). A wrong-char candidate → Reject
  (miss), which the FOOP-43 semantics turn into ECONSTANIC/WOCONSTANIC-wait.

**The KEY design question (governs all three types).** `get_value_primitive()` dispatches *on the
characterization* — but FOOP-33 put characterizations on the **statement's `Identifier` (the LHS
name)**, NOT on the value FIR. Does the type tag **travel with the value** (a `StringFir`/
`FloatFir`/`IndepIntFir` knows its own primitive kind — the FIR *is* the type), or is it **read
from the defining statement's Identifier**? Natural answer: the value FIR carries its primitive
kind; `i'`/`s'`/`f'` on the LHS is an *assertion* the FVM verifies against it. **Resolve before
speccing** — it shapes the whole FOOP.

**Depends on:** **FOOP-33** (characterizations / `Identifier` / creation `s = c'⬤`, `f = c'⬤`) —
hard. Relates to FOOP-34 (typed comparisons/arithmetic; float `**`). Foundational for all future
datatypes.

**Open questions.** Where the type tag lives (value vs LHS — above). Int/float coercion in mixed
arithmetic (`1.5 + 2`) — coerce, or NK? Minimal string op set (literals + equality first, or
concat/length too?). Does `default_equal` (FOOP-33) extend to string/float equality? Float
equality caveats (exact vs epsilon — likely exact-bits for now). Characterization-*verification*
failure → NK or a new alarm kind? Interaction with null-characterization (FOOP-33 name constants)
— `i'`/`s'`/`f'` are normal chars alongside the null slot.

---

## Next-step increments (iteration 2026-07-08 — push each forward one step)

Each FOOP advanced by one concrete decision, grounded in the code read, viewed as a progressing
set. These are *provisional resolutions* of open questions — still a draft pass.

- **FOOP-33 (creation):** unchanged (already Final). It stays the gate for FOOP-83. No new decision;
  just re-affirm it is implemented first.
- **FOOP-83 Boolean operators — arg extraction resolved (provisional).** Concatenation elements are
  `foolish_children`; the operator is the **last** element, the argument brane the one **before**
  it. So a binary op reads the arg brane's own `foolish_children[0]/[1]` (equivalently head/tail
  `^`/`$`); `not` reads the single preceding value. → Spec the operators as: *settle the arg
  brane, take its head (and tail for binary), compare each to system.foo `True` via
  `default_equal`, emit True/False.* Remaining open: >2 args, and whether args come from a brane
  literal only or any brane-valued expression.
- **FOOP-63 Detachment — matcher-integration resolved (provisional).** The exclusion-list prefilter
  lives at the **top of the scan loop** (`contextful_search_scan`, `fir_kinds.rs:1978`): before
  `predicate.matches`, test the candidate against each active detachment pattern (reuse
  `SearchFir::matches_pattern` regex); if any matches, `continue` (skip) — the candidate never
  reaches the real predicate. Active patterns ride the Scope. Naked `<<>>`/`[*]` never enter this
  path (they use existing SFF). Remaining open: bare `<E>` meaning; do patterns match value/char
  too.
- **FOOP-04 Macros — scoping the research resolved (provisional).** First deliverable is a *research
  memo* comparing (a) declarative pattern-rewrite macros expressed as branes that transform
  branes (most Foolish-native, leans on FOOP-73 all-results + characterizations), vs (b) a distinct
  expansion phase (AST or Astn→FIR). Decision to reach in the FOOP: *which layer expansion
  happens at.* Everything else stays open.
- **FOOP-53 Inverse matcher — representation resolved (provisional).** Add `negate: bool` to the
  relevant `SearchPredicate` variants (not a `Not(Box<…>)` wrapper — flatter, and the combined
  `NameValue` needs **per-gate** negation which two independent bools express directly:
  `negate_name`, `negate_value`). In `matches`, a negated gate swaps `Approve`↔`Reject`;
  `NkStop` stays. Parser adds a `Bang` token consumed after optional `&`, before the op.
  Remaining open: negated-anchored miss outcome (defer to FOOP-43's ECONSTANIC rule).
- **FOOP-73 All-results — result shape resolved (provisional).** Each match becomes exactly the
  existing single-search result pair (`push_search_result_pair`: a value result + a `FoolRefFir`
  referent, `fir_kinds.rs`). The find-all result is a **brane whose children are those pairs, in
  scan order** — so every entry already carries its position, and a following `&`-search chains
  off any entry's `FoolRefFir` for free (FOOP-23 composition). Collect-mode scan = the existing
  loop but push-and-continue on `Approve`. Remaining open: `//` division clash (respell/drop);
  local vs parent-crossing; empty → empty brane (leaning empty-brane, not NK).
- **FOOP-93 Recursion — resequenced + discovery-driven (see §6).** Moved **after the full search
  suite** (Atlas). **NO cycle detection** (`infinite_loop.foo` non-termination is ACCEPTED). The
  FOOP's **first task = write a Fibonacci computer, then a series of standard recursive
  algorithms, in Foolish-as-it-exists-then** — the friction of writing them defines the sugar /
  upgrades needed. Design follows evidence; those programs are also the approval-test suite.
- **FOOP-43 Miss→ECONSTANIC — fix located (see §7).** One-line settlement change at
  `fir_kinds.rs:1277` (anchored miss NK→Econstanic) + verify the deepen-chain NK-check
  (`fir_kinds.rs:1252`) then only fires for found-`???`, not miss. Next step: enumerate the
  NK-survivors (found `???`; `#N` out-of-range on a settled finite brane) and scope snapshot
  churn.

---

## Recommended implementation order

Ordered by dependency and by "easy to implement / easy to test / max coverage". **Recursion
(FOOP-93) is deliberately last among features — it comes AFTER the full search suite** (Atlas).

1. **FOOP-33** (creation) — already Final; **implement it first**. Unblocks FOOP-83/34/44;
   independent of the search family, so it can run in parallel.
2. **FOOP-43 Miss→ECONSTANIC** (foundational) — small settlement change (`fir_kinds.rs:1277`)
   revising FOOP-23's anchored-miss→NK. **Do early:** prerequisite for FOOP-63 AND FOOP-44
   (char-demand "wait"), fixes a real bug (`{a=b.c.d}` → WOCONSTANIC). Expect snapshot churn.
3. **FOOP-53 Inverse matcher (`!`)** — smallest self-contained new op; establishes the
   **candidate-prefilter** locus in the matcher.
4. **FOOP-63 Detachment (`[p..]<<E>>`)** — parameterized SFF marker; exclusion-list prefilter at
   the same locus as FOOP-53. Needs FOOP-43.
5. **FOOP-73 All-results (`~~`/`??`)** — tokens already lexed; collect-mode scan. Composes with
   FOOP-53/63 (`a![p..]~~q`).
6. **FOOP-24 Beefy search (`&&`/`||`/`|`)** — matcher boolean operators + cascading connector.
   After FOOP-43/53 (cascade needs miss-signal; composes with `!`).
   — *(#2–6 are the "full search suite" recursion waits on.)*
7. **FOOP-14 Computed index (`#${...}`)** — self-contained; any time.
8. **FOOP-83 Boolean operators** — after FOOP-33 (hard dep). Reuses `OperatorFir::combine`.
9. **FOOP-34 Integer math** — `**` any time; comparisons after FOOP-83 (return True/False).
10. **FOOP-44 Primitive Characterization** — after FOOP-33 + FOOP-43 (char-demand = search that
    waits). Establishes the type system (`i'`/`s'`/`f'`).
11. **FOOP-93 Recursion Upgrades** — **AFTER the full search suite (and 34/44).** Discovery-driven:
    write ~1–2 dozen recursive algorithms (Fibonacci first) → they reveal the sugar needed. `↑`;
    no cycle detection.
12. **FOOP-04 Macros** — last; research-heavy, benefits from FOOP-73/44/33.

**Tracks that can proceed in parallel:**
- **Search track:** FOOP-43 → FOOP-53 → FOOP-63 → FOOP-73 → FOOP-24  (one-engine work; the "full
  search suite")
- **Creation/type track:** FOOP-33 → FOOP-83 → FOOP-34 → FOOP-44 (44 also needs FOOP-43)
- **Standalone:** FOOP-14 anytime.
Then **FOOP-93** (recursion — after both tracks) and **FOOP-04** (macros) last.

## Last Updated

**Date**: 2026-07-09 (rev 9 — FOOP-93 "Recursion Upgrades" resequenced + algorithm list)
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.8
**Changes (rev 9, Atlas)**: FOOP-93 retitled **"Recursion Upgrades"**, **resequenced AFTER the
full search suite** (FOOP-43/53/63/73/24 + 34/44), and **deliberately under-specified** ("we'll
know better when we get there"). Its **first task = write ~1–2 dozen distinct recursive algorithms
(Fibonacci first) as Python/Rust references, then port to Foolish** — the friction defines the
needed sugar; also the approval tests. Added a candidate algorithm list (§6) spanning distinct
recursion shapes. Trimmed the earlier speculative design detail. Rewrote the full Recommended-order
list to include all 12 FOOPs with recursion after both feature tracks.

**Date**: 2026-07-09 (rev 8 — added FOOP-34 integer math + FOOP-44 Primitive Characterization)
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.8
**Changes (rev 8, Atlas)**: Added **FOOP-34 — strengthen integer math** (§10): exponent `**`
(self-contained) + comparisons `< > <= >=` **returning boolean True/False** (→ needs FOOP-33/83);
noted `*`/`%` are ALREADY DONE; flagged `<`/`>` SF-marker parse collision and missing `<=`/`>=`
tokens. Added **FOOP-44 — Primitive Characterization** (§11, renamed/broadened from "String type"):
`i'` int / `s'` string / `f'` float primitives + operators; `get_value_primitive()` dispatched by
characterization (replaces hardcoded `as_i64()`). **CORE TENSION (Atlas): search can DEMAND a
characterization → a brane is WOCONSTANIC if it can't find a properly-characterized parameter** —
char-demand is a `SearchPredicate` dimension, a wrong-char candidate is a MISS, so **FOOP-44
deeply depends on FOOP-43** (miss→ECONSTANIC/WOCONSTANIC-wait). Key open Q: does the type tag
travel with the value FIR or the LHS Identifier. Numbers: FOOP-34 (key 43), FOOP-44 (key 44).

**Date**: 2026-07-09 (rev 7 — FOOP-24 standardized vocabulary)
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.8
**Changes (rev 7, Atlas)**: Standardized the FOOP-24 search-connective vocabulary — **`&&`/`||` =
"matcher boolean operators"** (boolean ops on matcher results; the easy part), **`|` = "cascading
connector for search"** (hard part), **`&` = "continuation connector for search"** (existing
contexted `&`). Added a **FOOP-24 plan TODO: write these three terms into README.md and code
comments.** Noted `&&`/`||` are simpler (pure matcher-result boolean logic) than the cascade `|`.

**Date**: 2026-07-09 (rev 6 — added FOOP-24 beefy/boolean-combinator search)
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.8
**Changes (rev 6, Atlas)**: Added **FOOP-24 — boolean-combinator "beefy" search** (§9): `&&`/`||`
= parse-time predicate combinators (new `SearchPredicate::And`/`Or`, generalizing the atomic
`NameValue`); `|` = cascading fallback via a **special `CascadingSearchFir` wrapper** with
**anchor-propagation** (`A|B|C`: each fallback resumes from the nearest earlier branch that
established a position — reuses FOOP-23 `FoolRefFir`). Plus a bare leading `=N` value predicate.
Parseability solved by the **leading `(...)` combine-block** (expression position, avoids the
regex-group path at `parser.rs:797`). Depends on FOOP-43 (cascade "fail" = ECONSTANIC-miss);
composes with FOOP-53 (`!`) and FOOP-73 (`~~`). Supersedes the earlier "bare `|` doesn't parse"
scoping note. Number: sort key 42, after FOOP-14.

**Date**: 2026-07-09 (rev 5 — added FOOP-14 computed index)
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.8
**Changes (rev 5, Atlas)**: Added **FOOP-14 — computed/dynamic index `#${...}`** (§8): evaluate
the brane after `$`, take its tail as a number, run `#` with that offset. Grounded in code
(`IndexFir.offset` is a fixed i32 today, `parser.rs:894` parses only literals; dynamic form needs
a computed-offset child + a Braning-wait NYES phase). Self-contained, independent of the batch.
Added to the number table (sort key 41, next after FOOP-04).

**Date**: 2026-07-09 (rev 4 — FOOP numbers, SF/SFF spectrum, triple-doc, NO cycle detection)
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.8
**Changes (rev 4, Atlas)**: (1) **Assigned real FOOP numbers** and switched all `#N` item refs to
`FOOP-<NUMBER>` per Atlas: FOOP-43=miss→ECONSTANIC (**do FIRST**), 53=inverse `!`, 63=detachment,
73=all-results, 83=boolean ops, 93=recursion, 04=macros. (2) **SF vs SFF resolved as the two
extremes of the detachment spectrum**: `<E>`≡`[]<E>` (empty; detach nothing) and `<<E>>`≡`[*]<<E>>`
(full; detach everything) — this ANSWERS the previously-open "bare `<E>`" question. (3) Added the
**mandated triple-documentation task** (README.md + code comments + snapshot tests) for the
spectrum framing, as a FOOP-63 plan checkbox. (4) **Recursion (FOOP-93): NO cycle detection** —
`infinite_loop.foo`'s non-termination is ACCEPTED behavior, not a bug; corrected §6 and the
increments away from the rev-3 "self-cycle detection" framing (that framing is superseded). (5)
FOOP-43 reaffirmed as the first thing to implement.

**Date**: 2026-07-08 (rev 3 — SFF move, alias, FOOP-43, per-FOOP increments, recursion evidence)
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.8
**Changes (rev 3, Atlas, effort=max)**: Detachment moved to the **SFF marker** primarily
(`[p..]<<E>>`), works on BOTH SF and SFF (not exclusive). **Naked `<<E>>` aliased to `[*]<<E>>`**;
implementation keeps existing naked-`<<>>`/`[*]` code unchanged and adds a new **exclusion-list**
path only for specific detachments. Reject-all/exhaustion → **ECONSTANIC** (Atlas), which
dissolves the old `[*]`≈`<<>>` seam. Added **FOOP-43 Search-miss → ECONSTANIC-not-NK** (foundational;
revises FOOP-23; key discriminator found-`???`→NK vs not-found→ECONSTANIC/WOCONSTANIC; fix at
`fir_kinds.rs:1277`); it is a prerequisite for FOOP-63. Added **recursion verify evidence**:
`infinite_loop.foo` (`{f1={f1};stuck=f1}`) spins to ITERATION-EXCEEDED — core problem is
WOCONSTANIC self-cycle detection. Added a **"Next-step increments"** section advancing all 7 FOOPs
one step each (arg-extraction, prefilter locus, macro layering, negate representation, find-all
result shape, cycle rule, NK-survivors). Updated diagram + order (FOOP-43→FOOP-53→FOOP-63→FOOP-73 search track).

**Date**: 2026-07-08 (rev 2 — detachment reframe)
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.8
**Changes (rev 2, Atlas, effort=max)**: **Reframed FOOP-63 Detachment** from `[..]{..}` brane
recoordination into a **parameterized SF-marker `[p1,p2,p3]<Expr>`** — a per-candidate search
**prefilter** (reject candidate if any p matches, before testing q). Added analysis of
`[*]<E>` vs `<<E>>` (SFF): converge for unanchored searches (both → coordination-deferred), but
diverge on anchored searches (`[*]` reject-all → NK terminal vs SFF defer) — one seam left OPEN.
Bare `<E>` equivalence left OPEN (Atlas's instruction). Constanic-copy strips the parameterized
marker like a normal SF-marker (verified `constanic_clone_at:155-179`). Moved detachment into the
**search family** (shares the candidate-prefilter locus with FOOP-53 inverse matcher). Corrected FOOP-93
recursion: `[n=n-1]` param-binding reading is gone → recursion's dep on FOOP-63 now UNCLEAR. Updated
the diagram and ordering (FOOP-53 → FOOP-63 → FOOP-73 search track). Verified against code that `is_search()`
supertype was never implemented. See memory `foop-detachment-as-parameterized-sfmarker`.

**Date**: 2026-07-08 (rev 1)
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.8
**Changes**: Initial draft-pass notes for 7 FOOPs (FOOP-33 creation base + boolean operators,
detachment, macros, inverse matcher, all-results, recursion). Code analysis of parser/lexer
(doubled tokens already lexed; ↑ and detachment already parsed; one-engine search model),
implementation touchpoints, per-FOOP open questions, and a dependency-ordered plan.
