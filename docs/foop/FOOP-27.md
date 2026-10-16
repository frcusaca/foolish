---
foop: D72
title: Equivalences of Foolish — round trips, FIR equivalence, and a new EQUIVALENCE.md
author: hcbusy <hc.busy@gmail.com>, with Claude Code / claude-opus-5-5
status: Draft
type: Standards
created: 2026-10-16
phase: phase-4
supersedes: [FOOP-76]
begun: [ ]
---

# FOOP-27: Equivalences of Foolish — round trips, FIR equivalence, and a new EQUIVALENCE.md

FOOP numbering is little-endian; the full rules live in `foop.md` at the repository root — **read
it before creating or editing a FOOP.** In prose this is "FOOP 27"; it is NOT FOOP 72 (`foop: D72`
is its sort key).

> **This FOOP absorbs FOOP-76 ("Revival of Equality"), deprecated 2026-10-16 at the human's
> request** ("Let's deprecate FOOP-76 and move it's content to FOOP-27"). FOOP-76 §1–§6 are carried
> here as **Part C, §C.1–§C.6**, verbatim apart from the heading numbers, with every dated human
> attribution preserved; they are load-bearing. FOOP-76's tests, rejected alternatives, open
> questions and references are carried into the matching sections below, marked *(from FOOP-76)*.
> Inside Part C, a bare reference such as "§3.2" means **§C.3.2** — FOOP-76's own numbering.

## Abstract

Foolish needs more than one notion of "the same", and the project has been inventing them one at a
time — a textual fixed point in FOOP-36, a FIR relation with creation conditions in FOOP-76, a
vintage operator taxonomy in `docs/vintage_legacy/EQUIVALENCE.md`, integer-only value-search
equality in FOOP-23. This FOOP is **design first**: it makes one serious pass at **collecting every
equivalence we wish to document** (Part A) — more than will be implemented immediately — and
proposes them in order. The **first two** are the two round trips the human stated on 2026-10-16:

```
E1 (text):  sequence(ubca(sequence(ubca(P))))  ==  sequence(ubca(P))
E2 (FVM):            ubca(sequence(ubca(P)))   ≅   ubca(P)
```

**Both E1 and E2 are required deliverables of this FOOP, and both are implemented here** (the human,
2026-10-16). Neither is optional, and neither waits on the other. E1 compares two renderings byte for
byte. E2 compares two FVM results under a **Foolish FIR
equivalence**, which must be defined — Part C (FOOP-76's design) is its basis — and which carries
**parameters**: whether NK equals NK, whether names are compared, how far to run, and how constanic
subtrees compare (§A.2). Every rendering difficulty observed so far is **listed as a design input**
(Part B) so the design accounts for it once rather than being revisited. The design **may change the
FVM and FIR structures as needed, without reducing language capability** (§D.2). The design phase's
outputs are a **new `EQUIVALENCE.md`** in the live documentation tree and an **implementation plan**
(§D.1). Implementation is scheduled **immediately after FOOP-26**.

## Motivation

### Why one FOOP for all of them

Every one of these relations answers the same question — *which differences matter?* — with a
different answer, and the answers interact. E1 is only meaningful once we know which rendering
differences are allowed; E2 cannot be defined without deciding whether names, NK and unresolved
searches count; `EQUIVALENCE.md`'s `=n=`/`=v=` family is the same decision surfaced as Foolish
operators. Designing them separately produced two documents (FOOP-36 §N6 → FOOP-76, and the vintage
taxonomy) that each had to be reconciled with the other. One catalogue, designed together, avoids a
third.

### FOOP-76's motivation, carried *(from FOOP-76)*

**The "revival" is the point.** `docs/vintage_legacy/EQUIVALENCE.md` sketched a family of Foolish
equality operators years ago — `=s=`, `==`, `===`, `=n=`/`=N=`, `=c=`/`=C=`, `=v=`/`=V=` — as
*language* operators with proposed surface syntax. None was ever specified or implemented. **FOOP-23
has been deferring to it ever since**: its §Open Questions record that value-search equality means
**integer equality only, pending an equivalence FOOP**, and `FOOP-23.plan.md` §D.4 carries an
unchecked task to say so in `EQUIVALENCE.md`. **This FOOP is that equivalence FOOP.**

**Today** the repository has no FIR equivalence relation. FOOP-36's Property 3 — "`R` MEANS what `P`
meant" — is a stated REQUIREMENT of `Ubca2Sequencer` **enforced by reading**, because no mechanical
check exists (FOOP-36 §Rejected Alternatives F). FOOP-36 §2.2 names the pair that would check it —
`spp` vs `spr` — and its test **T2c** is deferred here.

**The callers** (human's framing): **testing** (FOOP-36's `spp` vs `spr`; any check that two FIRs
agree), and **comprehending programs** — *"under what identifications are these two branes the
same?"* is a question a Foolisher asks while READING code. That is what makes this language
documentation, not a test utility.

### Measured today: E1 does not hold (`jia` at `eec1dfc4`, 2026-10-16)

FOOP-36 defines E1 as its **Property 2** but checks it on only **ten hand-written programs**
(`sequencer::tests::foolish_rendering_round_trips_constanic_programs`). Its only corpus-wide test,
T3 (`einmo_corpus_wide_foolish_rendering_parses`), checks Property 1 (parses), deliberately.

Method: for every `foolish-ubca2/einmo_suite/input/**/*.foo`, `foolish-cli run P > R1`, then
`foolish-cli run R1 > R2`, compare byte for byte. (`foolish-cli run` is
`UbcaEvaluator::evaluate_arena` + `Ubca2Sequencer` in `Foolish` mode — the renderer einmo uses.)
Result: **179 fixed points, 4 drifts, 0 failures to run or re-parse.**

| Case | `R1` | `R2` | Part B item |
|---|---|---|---|
| `foop/23/contexted_index` | `oob = recipe?steps~stepˍ1&#5  !! NK: 40 is not a brane` | `oob = recipe?(steps~stepˍ1)&#5  !! NK: anchored index found no match` | B1, B2 |
| `foop/23/contexted_search` | `escape = recipe?steps~stepˍ1&?outside` | `escape = recipe?(steps~stepˍ1)&?outside` | B1, B2 |
| `foop/33/creation/referential_equality` | `crossˍsame = ba~=bb?v;` | `crossˍsame = ba~={v = shared}?v;  !! NK: …` | B1, B2 |
| `foop/86/unsteppable_concat_merge` | `!! NK: 'C already defined in context` | `!! NK: unsteppable — 'C already defined in context` | B4 |

Sources: `recipe.steps~step_1&#5`, `recipe.steps~step_1&?outside`, `ba~=(bb.v)`. Three of the four
**evaluate to something else on the second pass** — a different NK reason in one, NK where `R1` was
not NK in another. Property 1 passed every time; only E1 caught them.

## Specification

The specification has four parts:
- **Part A** catalogues the equivalences, with the first two specified in detail.
- **Part B** lists the known rendering difficulties the design must account for.
- **Part C** is FOOP-76's FIR-equivalence design, the basis for E2.
- **Part D** states the design phase's deliverables and constraints.

## Part A — The catalogue of equivalences

The design phase's first deliverable is this catalogue, completed. It is deliberately larger than
what will be implemented at first. Each entry records what it compares, what it ignores, its
parameters, and its status: **proposed-first** (E1, E2), **catalogued** (wanted, specified here at
least in outline, not scheduled), or **sketch** (named, not yet understood well enough to specify).

**E1 and E2 are both REQUIRED, and both are implemented by this FOOP** (the human, 2026-10-16). They
are independent: E1 is a byte comparison of text and needs nothing from Part C; E2 needs Part C's FIR
equivalence. Phase 4 delivers both, and the FOOP is not complete until both gates run green over
the corpus. E3–E13 are catalogued for documentation and need not be implemented here.

| | E1 — text round trip | E2 — FVM round trip |
|---|---|---|
| Specified in | §A.1 | §A.2 |
| Design inputs | Part B | Part C, plus Part B where rendering affects meaning |
| Gate | corpus-wide byte equality of `R1` and `R2` | corpus-wide FIR equivalence, strict corner (§A.4) |
| Status | **required** | **required** |

### §A.1 E1 — the text round trip (sequence fixed point) — required

`sequence(ubca(sequence(ubca(P)))) == sequence(ubca(P))`, byte for byte. This is FOOP-36's Property 2.

- **Domain.** `P` whose program FIR is constanic after `ubca(P)`. Pre-constanic results (a step
  budget hit, an `ALARM:` halt) are exempt per FOOP-36 §2.1. The exemption is decided by the FIR's
  NYES, never by a list of case names.
- **Gate.** A test beside T3 in `foolish-ubca2/src/einmo_gates.rs`, walking the same inputs, that
  reports **every** drifting case in one failure message (case, `R1` line, `R2` line). **No
  allow-list.** A drift that cannot be repaired is handled with the human under AGENTS.md
  §"Deferring an incidental bug".
- **Repair rules** (they apply to every rendering, not only to E1's failures):
  1. **Grouping.** Emit parentheses exactly where re-parsing without them would build a different
     tree. The rule is structural: compare each child's binding strength with what the parser would
     bind in that operand position. The parser's precedence table is **consulted, not copied**.
     Never add parentheses where they are not needed (B2).
  2. **NK reasons come from the final FIR's state**, not from the route that produced it (B4).
  3. **Where the FIR does not record what is needed to render faithfully, record it** — see §D.2 and
     B1/B7. Do not guess a spelling.
- **Parameters** (§A.4): `annotations` (are `!!` comments part of the equality? Recommended **yes**,
  Q-A1) and `depth`.

### §A.2 E2 — the FVM round trip (FIR equivalence) — required

`ubca(sequence(ubca(P))) ≅ ubca(P)`, where `≅` is the **Foolish FIR equivalence**. Its basis is
Part C: a lockstep shape walk (§C.1), integers by value and creations by a bijective correspondence
(§C.2–§C.3), and the three-outcome `NO` / `YES` / `YES, provided [pairs]` signature (§C.4). This is
FOOP-36's **T2c** (`spp` vs `spr`). With the strictest parameters it is the check for FOOP-36's
Property 3.

E2 is not limited to round trips. The same relation compares any two FIR subtrees, from one FVM or
two (§C.3). The round trip is its first caller.

**Parameters**, all new as of 2026-10-16 except where marked. Part C as written hard-codes one
setting of each; the design must turn each into a stated parameter with a stated default:

| Parameter | Settings | Part C's hard-coded setting |
|---|---|---|
| `nk` | **`nk_equal`**: NK ≅ NK. **`nk_strict`**: NK ≇ NK, NaN-style; an NK anywhere makes the answer `NO`. | `nk_equal` (NK compared by kind; `Nk { reason }` ignored, §C.1) |
| `names` | **`names_compared`**: `Statement { identifier }` is shape. **`names_ignored`**: statements match by position and value only, so `{a=1;b=2}`, `{1;2}` and `{b=1;a=2}` are equivalent. | `names_compared` (§C.1 item 4) |
| `depth` | Run or compare only down to depth X. **Two readings, both plausible** (Q-A3): (i) step each FVM only until X; (ii) compare the trees only down to brane depth X. | unbounded |
| `constanics` | **`constanic_identical`**: both sides constanic **and** their FIR trees match (shape walk) ⇒ equal. For example, the same unresolved search on both sides. **`constanic_any`**: both sides constanic ⇒ equal, checking NYES only, whatever the subtrees. | precondition only: both inputs must be constanic; no rule for comparing them (§C.3.3) |

**Interactions the design must settle** (recorded, not decided):
- `nk_strict` with `constanic_any`. NK is constanic, so `constanic_any` would equate two NKs that
  `nk_strict` forbids. Proposed: `nk` is consulted first, so `nk_strict` wins (Q-A4).
- `names_ignored` with §C.3's name lookup. Ignoring names in the comparison does not change how each
  tree resolved its own names; the two jobs stay separate, as §C.3 insists.
- `depth` with §C.3.3. A depth-bounded run may hand over pre-constanic subtrees. The `constanics`
  parameter, or a rule that pre-constanic nodes at the boundary are compared by shape only, must say
  what happens. This is where §C.3.3's "generalize later" is cashed in.
- `names_ignored` is not any operator in the vintage taxonomy. `=V=` is "same values *against the
  same names*", its opposite. The new `EQUIVALENCE.md` must say so.

### §A.3 Further equivalences — catalogued or sketch

Carried from `EQUIVALENCE.md` and FOOP-76 §5, plus what this conversation surfaced. The design phase
completes this list. It does not implement it.

| Id | Relation | Source | Status |
|---|---|---|---|
| E3 | `=s=` — syntactic: bitwise-identical source | EQUIVALENCE.md | catalogued |
| E4 | `==` — equal once neither side is nye | EQUIVALENCE.md | catalogued; E2 is a `==`-like relation (§C.5 point 1) |
| E5 | `===` — semantic: equal under ALL possible coordinations | EQUIVALENCE.md | sketch; may have no decision procedure. E2's `YES, provided […]` is its constructive form (§C.4) |
| E6 | `=n=` / `=N=` — same names / in the same order | EQUIVALENCE.md | catalogued |
| E7 | `=c=` / `=C=` — same characterized names / in the same order | EQUIVALENCE.md | catalogued |
| E8 | `=v=` / `=V=` — same values appear / same values against the same names | EQUIVALENCE.md | catalogued |
| E9 | positional value equality (E2 with `names_ignored`) as an operator in its own right | human, 2026-10-16 | catalogued |
| E10 | conditional equality — the residual as a first-class answer, composition, caller-supplied assumptions | FOOP-76 §4.1 | catalogued |
| E11 | value-search equality (`~=` / `?=`) beyond integers | FOOP-23 §Open Questions | catalogued; today integer-only |
| E12 | `pp` vs `pr` — the UN-stepped trees of a round trip, to separate parse/render faults from evaluation faults | FOOP-36 §2.2, FOOP-76 Q4 | catalogued |
| E13 | the FOOP-95 pair: `normalize(resequence(parse(src))) == normalize(src)` and resequencer idempotence | FOOP-95 | **owned by FOOP-95**; listed so the catalogue is complete |

Whether any of E3–E12 become Foolish **surface syntax** is FOOP-76's Q1, carried as Q-C1.

### §A.4 Parameters common to E1 and E2

`depth` applies to both. `annotations` applies to E1 only. `nk`, `names` and `constanics` apply to
E2; their analogues for text, if any, are for the design phase to state. **Defaults** are open (Q-A2).
The strict corner — `nk_strict`, `names_compared`, unbounded `depth`, `constanic_identical`, empty
residual required — is what the round-trip gates use.

## Part B — Known rendering difficulties (design inputs) — working towards E1 (and E2)

> **This part is working towards E1** (§A.1). Every item here is a way the text round trip fails or
> could fail. Items that change what `R1` *means* (B1, B2, B3, B6) also break E2, since
> `ubca(R1)` is then a different tree.

The human's instruction: list every difficulty observed so far and design against it, **so we do not
have to come back to fix them**. Each item names its evidence and its current owner, if it has one.

| # | Difficulty | Evidence | Owner today |
|---|---|---|---|
| **B1** | **`.` renders as `?`**: they lower to the same FIR, so the spelling is lost, and `?`'s operand then re-parses differently. | `recipe.steps~step_1&#5` → `recipe?steps~stepˍ1&#5` → re-parses as `recipe?(steps~stepˍ1)`. FOOP-36 §N1/§N2. | FOOP-07 (`.` and `?` become distinct; "`.` renders as `.`") |
| **B2** | **Parentheses dropped**: the rendered text re-parses as a different tree. | `ba~=(bb.v)` → `ba~=bb?v`, which re-parses as `(ba~=bb)?v`. `(r.s?fry).oil` loses its parentheses (FOOP-07 planner). | none |
| **B3** | **`&` dropped**: a contexted index renders as an attached plain one. | `{a=1;b=2}&#0` renders as `=^ {…}` (FOOP-07 planner). | none |
| **B4** | **NK reason depends on route**: the same final FIR gets a different `!! NK:` text depending on how it arose. | `foop/86/unsteppable_concat_merge` (measured above). | none |
| **B5** | **Fused concatenation**: adjacent constituents run together into one identifier. | `o = f1 <f2> <<f3>>` → `o = f1f2<<f3>>`. FOOP-36 §2. | fixed in FOOP-36; keep a regression test |
| **B6** | **Unnamed creation rendered as `⬤`**: a reference became a fresh creation. | `what_was_a = a` → `whatˍwasˍa = ⬤`. FOOP-36 §N4. | resolved 2026-09-07 (revert to original Foolish); keep a regression test |
| **B7** | **Spelling not recorded in FIR**: equivalent spellings (attached vs postfix, `#0` vs `^`, parenthesized vs bare patterns) reach the renderer indistinguishable. | FOOP-36 §N1 proposes a sequencing-aid element on the FIR. | unowned; §D.2 now permits it |
| **B8** | **`=$` / `=^` sugars**: `=$` yields the whole brane; `=^` never settles and leaks `Op^(…)` into output. | FOOP-75 §Motivation. | FOOP-75 |
| **B9** | **Backtick concatenation renders only while embryonic**: its provenance flag affects sequencing only. | FOOP-65. | FOOP-65 / FOOP-95 |
| **B10** | **Pre-constanic output is not a fixed point** by definition. | FOOP-36 §2.1. | stays exempt; the E1 domain says so |
| **B11** | **No line continuation**, so long statements cannot wrap. | FOOP-36 §N3. | future EOL-continuation FOOP |
| **B12** | **A planned rendering that may not round-trip**: FOOP-07 §4.3 plans `ba~=(bb.v)` → `ba~=bb.v`. | B2's measurement suggests it re-parses as `(ba~=bb).v`. | raise with FOOP-07 before it is Final (Q-B1) |
| **B13** | **Ignored-by-design differences** that E2 must not count. | `Statement { line_number }`, `Nk { reason }`, `Concatenation { rendering_aid }`, `FoolRef { referent }`. §C.1. | Part C |

**How the design uses this list.** For each item, the design phase records one of three things:
- the representation or rendering rule that makes it impossible;
- the owner FOOP that will, with a check that this FOOP's gates will catch a regression;
- an explicit exemption with its reason.

No item is left as "noticed".

**Contexted search without a search context** (human ruling, 2026-10-16). A contexted search whose
anchor provides no search context — no brane plus statement position, as with a literal `{…}` — is
**NK**. So today's `{…}&?a` → NK is **correct**. FOOP-23 §"Two degeneracies" (*"Contexted on a bare
brane ≡ contextless"*) and AGENTS.md's matching paragraph are **wrong**. FOOP-07's draft already has
the rule, so FOOP-07 owns the documentation correction. Under this ruling B3's `{…}&#0` is NK for a
literal anchor, but its rendering must still keep the `&`. Separately, and **not a rendering
problem**: `&^` / `&$` resolve to the anchor's own statement and the one before it, rather than to
the head and tail of the home brane. That is unowned (Q-B2).

## Part C — FIR equivalence design (carried from FOOP-76 §1–§6) — working towards E2

> **This part is working towards E2** (§A.2), the FVM round trip `ubca(sequence(ubca(P))) ≅ ubca(P)`.
> E2's `≅` is the relation designed here. It may not yet be complete for E2: §A.2's parameters are new
> and Part C does not yet carry them (see the note below). E1 (§A.1) is textual and does not depend on
> this part.

> **Read with §A.2.** Part C hard-codes one setting of each E2 parameter. Where it says names are
> shape (§C.1 item 4), that is `names_compared`. Where it compares NK by kind and ignores
> `Nk { reason }`, that is `nk_equal`. Where it requires constanic inputs (§C.3.3), §A.2's
> `constanics` and `depth` parameters now generalize it. Where it says the relation adds nothing to
> `FirSpec` (§C.6 and FOOP-76's FIR Impact), §D.2 now permits FIR and FVM changes. Bare
> §-references inside Part C use FOOP-76's numbering: "§3.2" means §C.3.2.

### §C.1 The shape half — kind, arity, children

Equality on FIR is not merely hard to implement; taken naively it is **ill-defined** (human,
2026-09-07). Too strict — comparing arena indices, NYES, or produced values — fails on *correct*
renderings, because recoordination may legitimately resolve an ECONSTANIC differently in a new
context (FOOP-36 §Rejected Alternatives F's surviving objection). Too loose, and it certifies
nothing. Every choice of what to ignore is therefore a judgment about what a comparison must
preserve, which restates the property rather than proving it.

So the relation is split into a **shape** half (§1) and a **value** half (§2), each decidable on
its own terms. **Structural equivalence** compares, recursively over `foolish_children` (the
*written* structure, which recoordination does not perturb):

1. **Kind** — the same `FirSpec` discriminant.
2. **Arity** — `foolish_children().len()` agrees. A 3-statement brane is not equivalent to a
   5-statement one; a `Concatenation` over 2 constituents is not one over 3.
3. **Children**, pairwise, in order.
4. **Shape-bearing `FirSpec` fields** — `Search { pattern, anchored, forward, is_value_search,
   contexted }`, `Index { offset, anchored, contexted }`, `Operator { op }`,
   `Comparison { op }`, `Statement { identifier }`. Rendering `?x` as `~x` leaves the shape
   unchanged while changing the program, so these are part of shape.

It deliberately does **not** compare:

- **Values** — `IndepInt { value }` and creation identity. Deferred at the time §1 was first
  written; **§2 now settles both** (human, 2026-09-07).
- **`Statement { line_number }`** — a sequencer reformats to one statement per line (FOOP-36
  §4.1), so these differ by design.
- **`FoolRef { referent }`** — a raw `FirPointer`, meaningless across two arenas.
- **`Nk { reason }`** — prose, not structure.
- **`Concatenation { rendering_aid }`** — sequencing-only by construction (FOOP-36 §N1).
- **`ubc_children`** and NYES — produced values, which is the value half.

**What each half catches.** FOOP-36's fused `f1f2` concatenation defect is caught by **arity** (3
elements against 2). Its `⬤` unnamed-creation defect (FOOP-36 §N4) is caught by **§2's creation
table** — shape alone cannot see it, since both sides render a `Creation` node and the fault is
one of identity.

### §C.2 The value half — integers are easy, creations are dynamic programming

**Integers.** `IndepInt { value }` compares by value. Two `3`s are equal; a `3` and a `4` are not.
Nothing more is needed: an integer literal denotes itself, and re-parsing a rendered `3` yields a
`3`.

**Creations are the interesting case** (human, 2026-09-07). A creation denotes nothing but its own
identity, and identity is arena-scoped — `FirPointer` is meaningless across `spp` and `spr`. So
creation equality cannot be decided by looking at either node: it is a **correspondence discovered
during the walk**, built by dynamic programming over an equality table.

#### §C.2.1 The table is built with respect to IDENTICAL TREE TRAVERSAL

**It is built incrementally, but with respect to IDENTICAL TREE TRAVERSAL** (human, 2026-09-07).
This qualifier is what makes the table well-defined rather than arbitrary. The two trees are walked
in **lockstep** — the same order, position by position, `foolish_children` index by index — so when
the walk arrives at a pair of creations, those two nodes occupy *the same position in their
respective trees*. That is the only reason it is meaningful to call them corresponding.

The consequence is that the shape half is not merely a separate check that happens to run
alongside: it is the **precondition** for the value half. The lockstep walk is what §1's kind and
arity comparisons enforce, and it is what lets a creation pair mean anything at all. If the walk
ever had to guess which right-hand node matches a given left-hand node, the table would be
searching for an isomorphism rather than verifying one, and the linear-time incremental
construction below would not apply.

**Hence a strict order of failure: the tree match would have to fail first** (human, 2026-09-07).
At every position the walk checks kind, then arity, then — only if both agree — descends or
compares values. A shape mismatch **fails immediately and the walk stops**; it never reaches the
creation table at that position or below it. So a creation-table failure (case 3 below) is only
ever reported for two trees whose shape has already matched everywhere the walk has been. That
makes the diagnosis unambiguous: a shape failure says *the structure differs*, and a table failure
says *the structure agrees but the sharing does not* — which is exactly the distinction FOOP-36
§N4's defect turned on, and it would be lost if the two halves could fail in either order.

#### §C.2.2 The rule — three cases at every creation pair

Whenever the walk compares two elements and both are creations, consult a table of pairs
`(left creation, right creation)`:

1. **Neither is in the table** — they are **equal**, and the pairing is **recorded**.
2. **The pair is in the table** — great, **equal**.
3. **Either is in the table but paired to something else** — **fail**: the two trees are not the
   same.

Case 3 is the whole point. It is what makes the table a **bijection** rather than a mere mapping: a
left creation may correspond to exactly one right creation and vice versa, so the check must be
consulted in both directions. §3.2 justifies that requirement from the Creation Postulate rather
than leaving it as an implementation convention.

**Why this catches FOOP-36's `⬤` defect.** For `{orig = ⬤; ref = orig;}`, the left tree has ONE
creation reached twice — once at `orig`'s definition and once through `ref`. A correct rendering
also yields one creation reached twice: the first comparison records the pairing, the second finds
exactly that pairing already present, and case 2 approves. The defective `⬤` rendering yields TWO
distinct creations on the right; the second comparison finds the left creation already paired to
the OTHER right creation, and case 3 fails. Sharing preserved passes; sharing split fails — which
is precisely FOOP-36 §2's "the same sharing, the same creations".

Note what the table does NOT require: it never asks that a creation occupy the same arena index, or
be reached by the same route. Only that the *pattern* of sharing agree. That is what makes it sound
across two independently-built arenas.

#### §C.2.3 Reflexive pairs — an optimization inside one comparison, never a design assumption

**Does the table ever need an identity entry, `A ≡ A`?** (human, 2026-09-07). **Not when the two
sides are literally the same node** — if the walk reaches the same `FirPointer` on both sides,
which can only happen when both subtrees are drawn from the SAME `FVMStorage`, the pair is
trivially satisfied and recording it wastes an entry. Skipping reflexive pairs is a sound
optimization, and it also keeps the residual of §4 minimal: a condition of the form "`A` must equal
`A`" is no condition at all and should never appear in an answer handed to a caller.

**But the table cannot be built on the assumption that identity is expressible.** Two creations from
DIFFERENT arenas are always distinct `FirPointer`s even when they denote the same thing —
`FirPointer` is arena-scoped by construction — so `A ≡ A` is not a case that arises there at all.
That is the case §4 exists for, and it is the general one. The rule to implement:

- **Same arena, same pointer** → equal, record nothing.
- **Otherwise** → the three-case check above, on a pair of genuinely distinct pointers.

The distinction matters because it says where the shortcut lives: it is an optimization inside the
comparison of one pair, NOT a property the table's design may rely on.

#### §C.2.4 On redundancy — noted, not derived; and why the table ships anyway

**On redundancy — noted, not derived** (human, 2026-09-07: "explore formalism but don't spend
cycles deriving"). In T2c's particular use — two whole programs from the same source, stepped the
same way, walked from their roots — the table looks redundant: the lockstep walk tends to hit a
shape difference before it ever reaches a creation pair. A spot check bears that out; the correct
tree for `{orig = ⬤; ref = orig;}` against the defective `⬤` rendering differs in node count (7
against 5), because a `Search` carries its anchor child and a bare `Creation` does not. So **both
of FOOP-36's recorded defects are caught by the shape half**, which corrects an earlier draft
claiming the table catches §N4.

That is as far as it is worth taking the argument. Proving it properly is slippery — stepping
rewrites the tree as it goes — and the conclusion would not change what gets built. Treat it as an
observation about T2c's inputs.

**Include the table regardless**, for reasons that do not depend on the observation holding:

- **Out-of-order execution** (how the trees are BUILT) and **out-of-order comparison** (how they
  are WALKED) each break the assumption that position implies identity. The human's concrete case:
  comparing every one of a brane's children **in parallel**. There the table becomes shared state,
  and two children racing to record `(L→R₁)` and `(L→R₂)` is a real violation that can be LOST
  unless check-and-record is one atomic step. The resulting bijection is then also run-to-run
  nondeterministic, so pairing detail in a failure message is diagnostic only.
- **§3's two-FVM setting** — subtrees from different arenas, and non-identical source — where
  pointer identity is meaningless and the pair list is load-bearing from the first comparison.

It is cheap (a map consulted at creation nodes), correct today, and already correct under those
changes. The one thing that must not happen is a future reader seeing an assertion that rarely
fires, concluding it is dead, and deleting it — hence this note.

### §C.3 Two FVMs is the defining case; name lookup is ordinary scoping

**Take the most disjoint case: TWO FVMs produce TWO trees, and we are given a subtree from each**
(human, 2026-09-15). That is the general setting, and every other use is a specialization of it.
Framing the relation this way settles several things at once that earlier drafts got wrong.

**Pointer identity is meaningless across the pair, by construction.** `FirPointer` is arena-scoped,
so `fvm1_creation1` and `fvm2_creation10` are incomparable as values even when they denote the same
thing. A record of *pairs* is therefore not an optimization — it is the only thing that can relate
the two sides at all.

**Two mechanisms, two distinct jobs.** An earlier draft conflated them, and the conflation is what
produced FOOP-36's now-dissolved Q9:

| mechanism | job | scope |
|---|---|---|
| name lookup | what does this name mean **here**? | *within* one tree |
| the pair list | which creation corresponds to which? | *between* two trees |

**Name lookup is ordinary nested scoping — nothing bespoke is needed** (human, 2026-09-15). The
conceptual model: every brane builds a map as it walks its statements in order; a name is looked up
in the current brane's map first and falls back outward to the parent's. Re-stating a name in one
brane overwrites, which is correct, because that is the evaluation order. **Or, more simply, use
the FVM's own search language** — `ib_search`, `ab_search`, backward search — which already
implements exactly this: IB is the context accumulated so far, AB is the parent chain. Preferring
the existing machinery over a reimplementation is the same discipline FOOP-36 §N4.b followed ("we
shoudl use search when possible", human 2026-09-07), and for the same reason: the language already
answers this question correctly, and a parallel implementation can only drift from it.

**FOOP-36's Q9 is DISSOLVED, not answered.** The earlier draft specified per-brane creation tables
created on brane entry and discarded on exit, then asked what happens to a creation that outlives
the table which recorded it — e.g. `{shared = ⬤; ba = {v = shared;}; bb = {v = shared;};}`, where
one creation is reached from two sibling branes. **That question was an artifact of the invented
mechanism, not of the language.** `shared` is created once and referenced from every context below
it; nothing in Foolish splits it. Under ordinary scoping both `ba` and `bb` miss locally, fall back
outward, and reach the same defining statement — there is no table lifetime, so nothing escapes it.
The three candidate rules the draft offered were three ways to patch a self-inflicted problem. See
FOOP-36 §Open Questions Q9 for the record.

#### §C.3.1 System equality — the comparison starts seeded, not empty

**FVM1's definition of numbers is already equal to FVM2's definition of numbers** (human,
2026-09-15). Every FVM composes the *same* `system_foo::SYSTEM_FOO_SRC` — one compile-time source
string, run through the same deterministic composer (`compose_program_with_system`). So two FVMs
are never fully disjoint: they share a common ancestor, and their system creations correspond by
construction rather than by anyone's declaration.

**This is a seeding rule.** Before comparing two subtrees from different FVMs, the pair list starts
**pre-populated** with every system creation paired to its counterpart — `fvm1`'s `'True` ↔
`fvm2`'s `'True`, their number definitions, and so on. Three consequences:

- **Seeded pairs never appear in the residual.** `YES-provided […]` lists only what the *users'*
  programs introduced. A residual cluttered with "provided `'True` equals `'True`" would be noise,
  and would bury the conditions that actually matter.
- **A system creation paired against a non-system one is a hard NO**, not a condition. If `fvm1`'s
  `'True` would have to equal some user creation in `fvm2`, no caller judgment can rescue it — it
  is a contradiction, not a negotiable identification.
- **The seeding is mechanical.** Because the system brane is composed identically in both, the
  pairs can be established by walking the two system branes in lockstep and pairing creation to
  creation. Names are not needed for this, though they make it checkable.

Without this shared ancestry every comparison of two independently-built trees would begin with
zero known correspondences. System equality is what gives the relation a foothold to start from.

#### §C.3.2 The residual must be a BIJECTION — the Creation Postulate demands it

**The creation pairs must be bijective within creations** (human, 2026-09-15). This residual is
**not** producible, and an attempt to produce it is a `NO`:

```
[(vm1_a1, vm2_a1), (vm1_a2, vm2_a1)]        ← IMPOSSIBLE
```

**Why, from the Creation Postulate** (`docs/why/creation_postulate.md`, specified in FOOP-33). Every
`⬤` is a genuinely new thing, distinct from every creation before it. So within VM1, `vm1_a1 ≠
vm1_a2` — that is the postulate, not an implementation accident. Meanwhile `vm2_a1 = vm2_a1`
trivially. The residual above therefore asserts that two things *known to differ* are both equal to
one single thing, which is a contradiction no caller judgment can discharge. It is not a condition
that happens to be unattractive; it is not a condition at all.

The same argument runs in the other direction — one left creation paired to two different right
creations is equally impossible — so the requirement is a **bijection**, and the check is made in
**both** directions. This is why the pair list must be consulted before recording, and why a
contradicted pairing is a hard `NO` rather than something to be reported and left to the caller.

**Worked counterexample, to be a test.** The implementation of this relation must carry this case
explicitly:

```foolish
{ a1 = ⬤; a2 = ⬤; }        !! in VM1 — two DISTINCT creations, by the postulate
{ a1 = ⬤; }                !! in VM2 — one creation
```

Comparing a structure that reaches both of VM1's creations against one that reaches VM2's single
creation twice must answer **NO**. If an implementation instead returns
`[(vm1_a1, vm2_a1), (vm1_a2, vm2_a1)]`, it has silently identified two distinct creations, which is
exactly the collapse FOOP-36 §2's Property 3 exists to prevent — and it is the same failure shape as
the `⬤` defect of FOOP-36 §N4, one level up. Note this is also where §3.1's seeding is load-bearing
in reverse: the seeded system pairs are already a bijection, so a user creation colliding with a
seeded one is caught by the same check.

**The human asked that this counterexample be written down to make certain it is handled**, so it is
carried as a **REQUIRED test** (§Test Plan T-BIJ), checked in both directions.

#### §C.3.3 NYES: require constanic inputs first, generalize later

**Necessarily, the relation must either CHECK NYES equality or REQUIRE constanic subtrees as
inputs** (human, 2026-09-15). There is no third option: a pre-constanic subtree is mid-flight, so
two such subtrees may be structurally identical right now and diverge on the next step. Comparing
them without regard to NYES would answer a question nobody asked.

**The staging: define "constanic equivalence given creations" first; define the non-static version,
where NYES states are not constanic, later** (human). Two reasons this ordering is the right one,
not merely the easier one:

- **The constanic case is the one with a stable answer.** Both sides have finished; what they are is
  what they will remain. The relation is then a statement about two settled objects, which is what
  an equivalence ought to be.
- **It is what every known caller needs.** FOOP-36's own use (its §2.2's `spp` vs `spr`) compares two
  *stepped* trees; the `EQUIVALENCE.md` questions ask whether two programs denote the same thing,
  which is likewise a question about settled meaning. No identified caller wants to compare two
  half-evaluated trees.

So **v1 takes constanic subtrees and states that as a precondition.** Whether it is enforced by the
type system, asserted, or checked and returned as an error is an implementation choice for this
FOOP's plan; what matters is that it is a stated requirement rather than an unexamined assumption.

**What the later, non-static version must then decide** — recorded now so v1 does not foreclose it:

- Does NYES equality mean *the same state*, or *compatible* states? Two subtrees both BRANING at
  different depths are in the same state but not obviously equivalent.
- **ECONSTANIC is the interesting one.** It exists because a name might resolve later, in another
  context (FOOP-23). Two ECONSTANIC subtrees are *unresolved in the same way*, which may be a
  legitimate equivalence — or may be exactly where a residual should report a condition, since
  "equal provided these two unresolved searches resolve alike" is the same shape of answer as "equal
  provided these two creations are the same."
- Does a residual over pre-constanic trees need to carry conditions about *searches* as well as
  creations? If so, §4's pair list generalizes to pairs of unresolved things, not only pairs of
  creations — which is a larger design and a good reason to keep it out of v1.

### §C.4 Equality with CONDITIONS — return the mapping, not a boolean

**The interesting resultant system** (human, 2026-09-07): take two subtrees, compare them, and
**return the mapping of which creations would have to be equal for the two FIRs to be equal.**
Equality then comes *with conditions*, and the caller decides whether those conditions are
acceptable for their purpose.

**The signature, from the human's own worked example** (2026-09-15) — two FVMs, two subtrees:

```
NO
YES
YES, provided [(fvm1_creation1, fvm2_creation10), (fvm1_creation3, fvm2_creation2), ...]
```

`YES` is just the third answer with an empty list, so there is **one** function returning the
residual and the boolean is a convenience wrapper asking "is it empty?". That is why the residual
must be designed in from the start: retrofitting it onto a boolean changes every return path. Per
§3.1 the list is seeded with the system creations and those pairs are never reported, so a residual
names only what the users' own programs introduced.

This inverts the relation's shape. §1–§2 answer a yes/no question by building the creation table
internally and discarding it. Here the table **is the answer**:

| | question | result |
|---|---|---|
| §1–§2 | are these two FIRs equal? | `true` / `false` |
| §4 | under what identifications are they equal? | a set of required creation pairs, or "impossible" |

Three outcomes rather than two:

1. **Unconditionally equal** — the walk completes with an empty residual. Nothing needed to be
   assumed.
2. **Conditionally equal** — the walk completes, and the residual is the set of creation pairs it
   had to assume. "These are equal *provided* `L₁≡R₁` and `L₂≡R₂`."
3. **Not equal** — a shape mismatch, or an integer mismatch, or a creation pairing that contradicts
   one already required. No set of identifications can rescue it.

**Why this is more useful than a boolean.** Two subtrees plucked from the same FVM — or from
different ones — will routinely reach creations that are distinct objects, so a plain comparison
answers `false` and tells the caller nothing about *why* or *how close*. The residual says exactly
what would have to hold, and **under some conditions the user may decide certain creations really
are equal for their purpose**. The relation supplies the facts; the caller supplies the judgment.
That is a much better division than baking one notion of creation identity into the comparison and
forcing every user to accept it.

**It subsumes the boolean.** §1–§2's answer is just "is the residual empty?" — so this is a
generalization, not a competing design, and the boolean version should be implemented as a thin
wrapper over it rather than as separate code. Notably, **the FOOP-36 use wants the strict reading**:
for its Property 3, a non-empty residual is a FAILURE, because a rendering that requires two
creations to be identified is a rendering that lost the distinction. Other callers will want the
residual itself.

#### §C.4.1 What conditions being first-class opens up

Once conditions are first-class, natural follow-ons appear — none of which need deciding now, but
which this FOOP should consider so the return type does not have to change later:

- **Composing residuals.** Comparing many pairs of subtrees and asking whether their conditions are
  jointly satisfiable — a union-find over creations across comparisons.
- **Caller-supplied assumptions.** Seeding the table before the walk: "treat `L₁` and `R₁` as the
  same creation, now compare." Falls out of the same machinery, since seeding is just pre-populating
  the residual — and §3.1's system seeding is already an instance of it.
- **Minimality.** Whether the residual returned is guaranteed to be the *smallest* set of
  identifications sufficient for equality, or merely *a* sufficient set. With the lockstep walk and
  ordinary scoping the natural construction is already minimal, but that should be stated and tested
  rather than assumed.
- **Cross-arena comparison.** The residual is a set of pairs, so it is meaningful whether both
  subtrees came from one `FVMStorage` or two — which is what makes "descendant FIRs that may or may
  not share FIR from the same FVM" a coherent thing to ask about.

**Interaction with §3.** The residual records CROSS-TREE creation pairs and carries no scoping duty
— name resolution inside each tree is ordinary nested lookup (§3). Conflating those two roles is
what produced FOOP-36's now-dissolved Q9. Per §3.1 the list is seeded with system creations, which
are never reported, so a residual names only what the users' programs introduced.

### §C.5 `EQUIVALENCE.md` — refreshed, and brought out of `vintage_legacy/`

**`docs/vintage_legacy/EQUIVALENCE.md` still exists, and this FOOP must update it as part of its
work** (human, 2026-09-07). It is not a stale note to ignore: it is a **taxonomy of equivalence
relations as Foolish LANGUAGE OPERATORS**, with proposed surface syntax, and it is the closest thing
the project has to a prior design for what this FOOP is now specifying. Writing this FOOP without
reconciling it would leave two competing accounts of Foolish equality in the repository.

**What it sketches** — an operator family, none of it specified or implemented:

| operator | relation |
|---|---|
| `=s=` | syntactic — bitwise-identical source |
| `==` | equal once the compared branes are no longer nye |
| `===` | **semantic** — equal under ALL possible coordinations |
| `=n=` / `=N=` | same names / same names in the same order |
| `=c=` / `=C=` | same characterized names / in the same order |
| `=v=` / `=V=` | same values appear / same values against the same names |

**Why it is directly relevant, not merely adjacent.** Three points of contact:

1. **This FOOP's relation is a `==`-like relation** in that taxonomy — structural, over
   already-stepped FIR — and should say so explicitly, in the document's own vocabulary, rather than
   inventing a parallel one.
2. **§4's conditional equality has no entry in the table**, and is arguably a better primitive than
   several that do. `===` ("equal in all possible coordinations") **quantifies over every context**,
   which is correspondingly hard to check. `YES-provided […]` is the **constructive** form of that
   same instinct: rather than asserting equality under all coordinations, it hands back the exact
   conditions under which equality holds, and lets the caller judge them. The refreshed document
   should say so, and should carry §3.1's system-equality rule, which is what makes a cross-FVM
   comparison start seeded rather than empty.
3. **FOOP-23 already defers to it.** Its §Open Questions record that value-search equality currently
   means **integer equality only, pending an equivalence FOOP**, and `FOOP-23.plan.md` §D.4 carries
   an **unchecked** task: *"Note in `EQUIVALENCE.md` (or leave a pointer) that value-search equality
   currently means integer equality only, pending an equivalence FOOP."* **This FOOP is that
   equivalence FOOP**, so it inherits that task and should close it.

**REFRESH THE DOCUMENT AND BRING IT OUT OF `vintage_legacy/`** (human, 2026-09-07). Not a side
errand: it is a **deliverable of this FOOP**, because this FOOP is what makes the refresh possible.
The reason is that there are now **several substantive equivalence definitions** — §1's structural
relation, §2's creation correspondence, §3's two-FVM framing, §4's conditional/residual form — where
the vintage document had only sketches. They are useful for two distinct purposes, and the human
named both:

- **For testing** — the FOOP-36 use, its Property 3, and any future check that two FIRs agree.
- **For COMPREHENDING PROGRAMS** — the larger reason. "Under what identifications are these two
  branes the same?" is a question a Foolisher asks while *reading* code, not only while testing it.
  That is what lifts this from a test utility to language documentation, and it is why the document
  belongs in the live tree rather than in the legacy pile.

`vintage_legacy/` is explicitly transitional — `docs/README.md` describes it as "Pre-reorganization
files, being migrated into the above" — so promoting `EQUIVALENCE.md` out of it is exactly the
migration that directory exists to enable, not a special case.

**Concretely, this FOOP should:**

- Read `EQUIVALENCE.md` before designing, and state where this FOOP's relation sits in its taxonomy.
- **Rewrite it against the definitions this FOOP actually establishes**, and **move it out of
  `vintage_legacy/`** into the live documentation tree. Destination is this FOOP's call —
  `docs/ubc1/how/` if it reads as engineering reference, `docs/why/` if the emphasis is the design
  rationale for what equality MEANS in Foolish — and `docs/README.md`'s index must be updated with
  it.
- Mark clearly which operators are specified-and-implemented, which are specified-only, and which
  remain sketch, so a reader is never left believing `===` exists when it does not.
- Note that `docs/howto/03_howto_foolish_todo.foo` already lists **equivalence** among its unwritten
  chapters. A refreshed document makes that tutorial writable, and this FOOP should consider whether
  writing it is in scope or a follow-on.
- Close `FOOP-23.plan.md` §D.4's outstanding `EQUIVALENCE.md` checkbox, and revisit FOOP-23 §Open
  Questions' "equality maturation" note, which anticipates exactly this work.
- Decide whether the vintage operators are still wanted as Foolish surface syntax at all, or whether
  this FOOP's relation is a Rust-side one only. **That is a scope question for the human** — see
  §Open Questions Q1 — and the document's existence is the reason it must be asked.

### §C.6 Scope beyond what is specified above

Recorded from FOOP-36 §N6's handover note, for whoever executes this FOOP:

- Whether the relation is a **test helper** or a **language-level notion** Foolish itself can
  express (this is Q1's larger half).
- Whether it belongs in `foolish-ubca2` or lower. The design as written reads `foolish-ubca2`'s
  `FVMStorage`, which argues for `foolish-ubca2`; a lower home would need the storage abstraction to
  move with it.
- Whether **`pp` vs `pr`** (FOOP-36 §2.2's third row — the unstepped FIRs) is worth implementing
  alongside as a debugging aid. It isolates a parse/render fault from an evaluation fault when the
  stepped comparison fails.

## Part D — Phases, deliverables and constraints

### §D.1 The four phases (human, 2026-10-16)

| Phase | Work | Output, written into the FOOP or plan |
|---|---|---|
| **1 — Semantic design** | Settle what each equivalence MEANS: complete Part A; give E1 and E2 their parameters and defaults; work through Part B item by item; reconcile with Part C. | This spec, with Open Questions emptied for E1/E2 |
| **2 — Illustrative tests** | Write, **into this FOOP**, the tests that best illustrate each equivalence and each parameter: Foolish programs (pairs, plus the expected answer under each parameter setting) or Rust unit-test bodies. These are the specification's worked examples **and** the tests. | A new §Illustrations section |
| **3 — Coding plan** | Plan the code, and **develop all the test cases**: every einmo input and unit test, written out, with expected results hand-written from the spec. Phase 2's illustrations are promoted into real tests here. | `FOOP-27.plan.md` (implementation part); the test files themselves |
| **4 — Code** | Implement until the Phase 3 tests pass. **Both the E1 and the E2 gates must be green**; neither is optional. | Code |

**Why tests come so early here.** The human's observation: this is one case where test writing is
**easiest**, and it can be done ahead of implementation. An equivalence is *defined* by which pairs it
accepts and which it rejects, so the illustrating pairs **are** the specification. Part C's T-BIJ and
T-SHARE already show this: each is a worked example that doubles as a required test. And E1's corpus
gate needs no hand-written expectations at all. Its expected answer is "unchanged".

The design phase's two named deliverables are a **new `EQUIVALENCE.md`**, moved out of
`vintage_legacy/` per §C.5, and the **implementation plan** (Phase 3). If the plan proves too large
for one FOOP, the design may split the catalogued items (E3–E12) into follow-on FOOPs. E1 and E2 stay
here.

### §D.2 The design may change the FVM and FIR — without reducing language capability

The human (2026-10-16): **"The design itself may change the FVM/ubca and fir structures as needed
without reducing language capabilities."** This supersedes FOOP-76's "the relation reads FIR and adds
nothing to it" and FOOP-36 §FIR Impact's "None". It is what makes B1 and B7 solvable at the source:
- FOOP-36 §N1's sequencing-aid element (record the written spelling on the FIR) is now in scope as a
  design option;
- so are equivalence-specific annotations, if E2 needs them.

**The constraint is a hard one.** Every FIR or FVM change the design proposes must state:
- what it adds;
- that every program accepted before is still accepted and means the same thing;
- which einmo cases would move, if any.

A change that would remove or narrow a capability is out of scope and goes to the human.

### §D.3 Scheduling

Phase 4 (code) and the Part B re-measurement run **immediately after FOOP-26** (the human's call).
FOOP-07, FOOP-46 and FOOP-26 each change rendering (B1, concatenation, marks/UFM), so the drift list
must be re-taken on `jia` after 26 merges.

**Phases 1–3 run NOW, in parallel with those efforts** (the human, 2026-10-16: planned in the main
session while the earlier FOOPs are coded elsewhere). That makes this FOOP's planning **concurrent
with other in-progress work, and it must stay aware of it.**

### §D.4 Awareness of concurrent efforts (standing rule for Phases 1–3)

While this FOOP is planned, other FOOPs are being planned or coded on other branches and worktrees.
Their specs and plans are visible in `docs/foop/` and on their branches. **This FOOP's design must
not silently assume a state of `jia` that a concurrent FOOP is about to change.** Concretely:

1. **At the start of every phase, and before every commit**, run `git log --oneline <recorded>..jia`
   and `git branch -a`. Read the current spec and plan of every concurrent FOOP below whose status or
   branch has moved since the last check. Record the `jia` commit this phase was prepared against.
2. **Every Part B item and every illustration in Phase 2 that depends on a concurrent FOOP's outcome
   names that FOOP and the assumption.** For example: "assumes FOOP-07 Q4/Q6 as recommended:
   `no statement named x`". A later change in that FOOP is then a findable, mechanical update, not a
   rediscovery.
3. **Findings that belong to a concurrent FOOP go to it, not here.** B12 (FOOP-07 §4.3) is the model:
   record it, then raise it with that FOOP's planner or the human. Do not edit another FOOP's files
   from this one.
4. **Phase 3 (coding plan) re-baselines after FOOP-26 merges.** Its expected outputs are written
   against `jia` with 07/46/26 landed, and it STOPs if any of them has not.

**Concurrent efforts known at creation (2026-10-16):**

| FOOP | State | Touches this FOOP's ground |
|---|---|---|
| FOOP-07 | plan drafted on `worktree-agent-a5e87dc31591ca72a` (`2b0aec55`), awaiting Q1–Q7; next to execute | B1, B12; the `.` vs `?` split; NK reasons (B4); the contexted-search ruling |
| FOOP-46 | plan drafted on `worktree-agent-ae848d2712c666948` (`c1bc7930`), awaiting H1–H6 | concatenation rendering and FIR shape (B5; E2's shape half over `BraneConcatOp`) |
| FOOP-26 | plan drafted on `worktree-agent-ad3e1d633707bbe7f` (`d4bb0d51`, `cfe0cd27`), awaiting Q-A–Q-C | mark rendering, UFM `<<<…>>>`, `CloneMarks` (new FIR shape for E2) |
| FOOP-96 | merged to `jia` (`dee80c72`); close-out open | moved the code Part C's line numbers point to |
| FOOP-17 | Draft, after this FOOP in Track 6 | none directly |

## FIR Impact

**To be determined by Phase 1, under §D.2's constraint.** Candidates already visible:
- a sequencing-aid element recording the written spelling (B1, B7; FOOP-36 §N1);
- no change at all, if FOOP-07's `.`/`?` split plus structural parenthesization (§A.1 rule 1) cover
  every Part B item.

The facts Part C relied on (*verify, don't re-derive*, and re-check after FOOP-96's split moved the
code):
- `FirSpec` derives `PartialEq`;
- `foolish_children` and `ubc_children` are read-only accessors on `FVMStorage`;
- `SYSTEM_FOO_SRC` is in `foolish-ubca2/src/system_foo.rs`.

FOOP-76's line numbers into `fvm_storage.rs` **predate FOOP-96** and are stale.

## UBC Step Impact

**None expected.** The E2 relation is read-only over FIR (§C, FOOP-76's analysis: it takes
`&FVMStorage` and constructs no FIR). E1 needs no evaluator change, and §A.1 forbids one: if a drift's
root cause is an evaluator bug, it belongs to the FOOP that owns that behaviour. Two things could
change this, and either must be reported as a scope change:
- a `depth` reading (i) that needs a new stepping entry point;
- a §D.2 FIR addition that the stepper must maintain.

## Test Plan

Phases 2 and 3 produce the full test inventory (§D.1). Required now:

- **E1 corpus gate** (§A.1), plus one unit test per Part B item that the design resolves, each a
  minimal program with its expected `R1`. Class B2 needs at least `recipe.steps~step_1&#5`,
  `ba~=(bb.v)` and `(r.s?fry).oil`, plus one case per precedence pair the parser distinguishes, enumerated
  from the parser's table in Phase 3, not guessed. Extend
  `foolish_rendering_round_trips_constanic_programs` with each.
- **E2 parameter illustrations** (Phase 2). For each parameter, a pair that it decides. For example
  `{a=1;b=2}` vs `{b=1;a=2}` is `NO` under `names_compared` and `YES` under `names_ignored`; `{x=1/0}`
  vs `{x=1/0}` is `YES` under `nk_equal` and `NO` under `nk_strict`.
- **E2 as T2c**: the round trip `ubca(sequence(ubca(P))) ≅ ubca(P)` over the einmo corpus, strict
  corner (§A.4).
- **Part C's tests, carried below**, re-stated against §A.2's parameters where they touch one.

### Carried from FOOP-76 §Test Plan

#### T-BIJ — the bijectivity counterexample (§3.2) — REQUIRED, both directions

**The human explicitly asked that this counterexample be written down as a test**, to make certain it
is handled. Two distinct VM1 creations against one VM2 creation reached twice must answer **`NO`**:

```
[(vm1_a1, vm2_a1), (vm1_a2, vm2_a1)]        ← must be refused
```

Because `vm1_a1 ≠ vm1_a2` by the Creation Postulate while `vm2_a1 = vm2_a1` trivially, this asserts
that two things known to differ are both equal to one thing. **Checked in BOTH directions** — one
left creation paired to two different right creations is equally impossible — since the requirement
is a bijection, not a function.

#### T-SEED — system equality seeds the pair list (§3.1)

Two independently-built FVMs, each composed through `compose_program_with_system` from the same
`SYSTEM_FOO_SRC`. Assert:

- Their system creations correspond — `fvm1`'s `'True` ↔ `fvm2`'s `'True`, and likewise the number
  definitions — **by construction**, with no user declaration.
- **Seeded pairs never appear in a residual.** A comparison whose only correspondences are system
  ones answers `YES`, not `YES, provided ['True ≡ 'True]`.
- **A system creation paired against a user creation is a hard `NO`**, not a condition.

#### T-SIG — the three-outcome signature (§4)

One test per outcome, on inputs chosen so the outcome is unambiguous:

1. **`YES`** — the walk completes with an empty residual.
2. **`YES, provided […]`** — the walk completes with a non-empty residual, and the residual contains
   exactly the pairs the walk had to assume (no seeded pairs, no reflexive pairs — §2.3).
3. **`NO`** — three sub-cases, since §4's outcome 3 has three sources: a shape mismatch, an integer
   mismatch, and a contradicted creation pairing.

Plus the wrapper: the boolean form agrees with "is the residual empty?" on every one of the above.

#### T-SHAPE — the shape half (§1)

- **Arity** catches FOOP-36's fused `f1f2` concatenation defect: a `Concatenation` over 3
  constituents is not one over 2.
- **Shape-bearing fields** are compared: `?x` against `~x` is a `NO` even though kind, arity and
  children all agree.
- **Ignored fields are ignored**: differing `Statement { line_number }`, differing `Nk { reason }`,
  and a differing `Concatenation { rendering_aid }` must not by themselves produce a `NO`.
- **Order of failure** (§2.1): a shape mismatch fails *before* the creation table is consulted at
  that position. A test that would produce both a shape failure and a table failure must report the
  shape one.

#### T-SHARE — the creation table catches split sharing (§2.2)

`{orig = ⬤; ref = orig;}` — one creation reached twice — compared against a tree with TWO distinct
creations in those positions. The first comparison records the pairing; the second finds the left
creation already paired to the other right creation, and case 3 fails. **Sharing preserved passes;
sharing split fails.** Note per §2.4 that in FOOP-36's own T2c inputs the shape half tends to catch
this first (node counts 7 against 5); this test constructs the FIR so the table is genuinely the
thing under test.

#### T-PRE — the constanic precondition (§3.3)

v1 states constanic inputs as a requirement. Whichever enforcement this FOOP's plan chooses — type
system, assertion, or a checked error return — there is a test that a pre-constanic input is
**rejected**, not silently compared.

#### T2c — FOOP-36's deferred structural-equivalence test

FOOP-36's §Test Plan records **T2c — Structural equivalence of the stepped FIRs — DEFERRED to §N6's
FOOP**, and this is that FOOP. Its §2.2 identifies `spp` vs `spr` as the pair that would check its
Property 3, and its T2 procedure already builds both arenas and discards them, **so the hook is
cheap**. This FOOP supplies the relation; wiring T2c is then a small addition on FOOP-36's side.

Two notes carried from FOOP-36's own T2c text:

- FOOP-36's §2.2 table **names the pair**; this FOOP **implements the comparison**. The division is
  deliberate — which pair to compare is a property of FOOP-36's round trip, how to compare is this
  FOOP's subject.
- **The FOOP-36 use wants the strict reading** (§4): a non-empty residual is a FAILURE there,
  because a rendering that requires two creations to be identified is a rendering that lost the
  distinction.

Whether T2c lands in FOOP-36's crate or as a test here depends on whether FOOP-36 has merged when
this FOOP executes; the plan should check rather than assume.

#### What is NOT tested in v1

- **Pre-constanic comparison** — out of scope by §3.3; the tests assert the precondition instead.
- **ECONSTANIC equivalence** — §3.3's open question, deliberately deferred.
- **Any surface-syntax operator** from §5's taxonomy, unless §Open Questions Q1 is answered in a way
  that puts one in scope. If an operator lands, it gets einmo cases; a Rust-side-only relation does
  not, since it has no Foolish syntax to exercise.

**Einmo coverage is contingent on Q1.** If this FOOP stays Rust-side, there is no `.foo` input to
write and the comprehensive einmo case would be vacuous — the plan says so explicitly at that
checkbox rather than generating an empty case. If an operator does land, `input/foop/76/` is
reserved for it.

## Plan of Execution for Plan

| Phase | Character | Needs |
|---|---|---|
| **1 — Semantic design** | Every deliverable is a decision; several are the human's (Q-A*, Q-C1). | **Larger model**, which must **stop and ask** on human calls |
| **2 — Illustrative tests** | Hand-written expected answers the design depends on. | **Larger model.** Never generated from an implementation, since none exists yet. |
| **3a — Coding plan** | Decomposition, file placement, stop conditions. | Larger model |
| **3b — Write out the test cases** | Mechanical transcription of Phase 2's illustrations and the Part B unit tests into einmo inputs and Rust tests, against expectations already written. | **Smaller model**, against fixed text |
| **4 — Code** | Implement until the Phase 3 tests pass: the E1 gate, the parenthesization rule, the E2 relation and its parameters. | Smaller model for code given or targeted by tests; **larger** for the shape-walk failure ordering (§C.2.1) and the bijection (§C.3.2), as FOOP-76 assigned |
| **Promotions, merge** | Correctness claims. | Larger model; human STOP |

Phase 4 is safe for a smaller model because its target is a test suite a larger model wrote first.
Named stop conditions:
- an evaluator change (§A.1, UBC Step Impact);
- a FIR or FVM change not designed in Phase 1 (§D.2);
- a drift not in the post-26 re-measurement;
- any einmo INPUT edit;
- any capability narrowed.

**Never delegated** (AGENTS.md §"The agent is responsible for correctness"):
- every `output` → `checked` promotion;
- Phase 2's expected answers, T-BIJ above all;
- the human's calls Q-A1–Q-A4 and Q-C1;
- any decision to change `FirSpec`;
- marking any Verified-tier test `#[ignore]`.

## Rejected Alternatives

### R1. Keep E1 and E2 in separate FOOPs (this FOOP as text-only, FOOP-76 for FIR)

That was the arrangement until 2026-10-16. Rejected by the human. The equivalences share their hard
decisions (which differences count), and separate FOOPs had already produced two accounts of
equality needing reconciliation.

### R2. Do nothing; T3's parse check is enough for E1

Rejected by measurement: all four drifts parse, and three of them evaluate to something else.

### R3. Fold E1 into FOOP-95

FOOP-95 round-trips the UN-stepped tree through a new, separate resequencer. E1 concerns the settled
rendering einmo already uses. FOOP-95's equalities are catalogued here as E13 and stay FOOP-95's.

### R4. Always parenthesize every compound operand

Trivially satisfies B2, but makes every rendering unreadable and moves every baseline, all with
`verified/` twins. FOOP-36 §4 asks for output a Foolisher could have typed.

### R5. Implement first, write illustrative tests afterwards

Rejected by the human's phasing (§D.1). For an equivalence the accepted and rejected pairs ARE the
definition, so writing them first is cheaper and catches design gaps before any code exists.

### Carried from FOOP-76 §Rejected Alternatives (A–F)

#### A. Do nothing — leave Property 3 enforced by reading

This is the status quo FOOP-36 inherited and lives with. It is survivable but demonstrably costly:
**two defects entered through exactly this gap** — the fused `f1f2` concatenation and the `⬤`
unnamed creation (FOOP-36 §N4) — each satisfying rendering Properties 1 and 2 while meaning
something else, and each found by a human reading output rather than by a test. Doing nothing also
leaves FOOP-23's value-search equality pinned at integer-only indefinitely, and leaves
`EQUIVALENCE.md` as an unspecified sketch that other FOOPs defer to. Rejected.

#### B. A plain boolean relation, with the residual added later

Simpler to write and adequate for FOOP-36's own use, which wants the strict reading anyway. Rejected
because **retrofitting a residual onto a boolean means changing the return type of every path**
(§4), and because the boolean answers `false` without saying *why* or *how close* — which is
precisely the information a caller comparing two independently-built arenas needs. The boolean is
kept, as a wrapper over the general form.

#### C. Compare by pointer identity, or by arena index

The obvious implementation, and wrong by construction. `FirPointer` is **arena-scoped**, so two
creations from different FVMs are always distinct pointers even when they denote the same thing
(§3). Pointer comparison would answer `NO` for every cross-FVM pair, making the defining case of the
relation unanswerable. It survives only as an optimization *inside* one comparison — same arena, same
pointer → equal, record nothing (§2.3) — never as the design.

#### D. Global creation table with no bijection check

Record pairs as they are met and never check for contradiction. Rejected from the **Creation
Postulate** (§3.2): a table that pairs `vm1_a1` and `vm1_a2` both to `vm2_a1` asserts that two things
known to differ are equal to one thing. Without the both-directions check, the relation would answer
`YES, provided …` to a condition that cannot be discharged — worse than answering `NO`, because it
looks like an answer.

#### E. Per-brane creation tables, created on brane entry and discarded on exit

The earlier draft's mechanism, and the source of FOOP-36's Q9. Rejected because it **conflates two
distinct jobs** (§3): name lookup within one tree, which is ordinary nested scoping, and creation
correspondence between two trees, which is what the pair list is for. The mechanism invented a table
lifetime the language does not have, then asked what happens to a creation that outlives it. The
question dissolved when the mechanism was dropped.

#### F. Compare pre-constanic FIR too, in v1

Rejected for v1 by §3.3, and the rejection is a staging decision rather than a permanent one. Two
pre-constanic subtrees may be identical now and diverge on the next step, so the answer is not
stable; and **no identified caller wants it** — FOOP-36's `spp` vs `spr` compares stepped trees, and
`EQUIVALENCE.md`'s questions are about settled meaning. §3.3 records what the later version must
decide so v1 does not foreclose it.

## Open Questions

**New with this FOOP (2026-10-16):**

- **Q-A1 — Are `!!` annotations part of E1?** Recommended: yes. If re-evaluating the text changes
  the stated reason, the reader of `R1` was told something `R1` does not mean. Today this affects only
  B4.
- **Q-A2 — Default for each parameter.** The strict corner for the gates (§A.4) is not in question.
  The defaults a Foolisher sees, for any surface operator under Q-C1, are open.
- **Q-A3 — What does `depth` mean?** (i) Step each FVM only until depth X, then compare. (ii) Compare
  the trees only down to brane depth X. These are different tools: (i) bounds evaluation, (ii) bounds
  the claim. Both may be wanted.
- **Q-A4 — `nk_strict` versus `constanic_any`.** Proposed: `nk` is consulted first, so `nk_strict`
  wins.
- **Q-B1 — FOOP-07 §4.3's planned `ba~=(bb.v)` → `ba~=bb.v`.** Probably B2. Raise it with the FOOP-07
  planner before FOOP-07 is Final, so 28 baselines are not re-signed around a rendering that does not
  round-trip.
- **Q-B2 — `&^` / `&$` resolve to the wrong statements.** A search bug, not a rendering one. It needs
  an owner: a FOOP-07 follow-up, FOOP-84 (Search Engine Refactor), or a new FOOP.
- **Q-B3 — `verified/` re-signing.** Every einmo case whose `R1` gains parentheses or loses a
  route-dependent reason moves OUTPUT and has a `verified/` twin. Count them in the post-26
  re-measurement; the human decides then.
- **Q-B4 — CLI versus in-process harness.** The measurement used `foolish-cli run`. Phase 3 confirms
  the gate's in-process pipeline reproduces the same drifts.
- **Q-D1 — ANSWERED 2026-10-16:** Phases 1–3 run now, in this session, in parallel with 07/46/26.
  §D.4's awareness rule is the mitigation for staleness, and Phase 3 re-baselines after 26.

**Carried from FOOP-76 §Open Questions** (renumbered Q-C1 to Q-C5; FOOP-76's text kept below):

- **Q-C1** = FOOP-76 **Q1**: the relation only, or the operator taxonomy as surface syntax? **Partly
  answered 2026-10-16.** The design catalogues the whole taxonomy (Part A), and E1 and E2 are
  implemented first. Whether any catalogued relation becomes Foolish surface syntax is still the
  human's call.
- **Q-C2** = FOOP-76 **Q2**: wait on FOOP-66's Q5 (tree calculus equality), or proceed in parallel?
- **Q-C3** = FOOP-76 **Q3**: where the relation lives.
- **Q-C4** = FOOP-76 **Q4**: `pp` vs `pr`. Now catalogued as **E12**.
- **Q-C5** = FOOP-76 **Q5**: is the `docs/howto/` equivalence chapter in scope?

FOOP-76's original text of these questions:

- **Q1 — How far does the scope go: the relation only, or the whole operator taxonomy?
  HUMAN'S CALL, and it is blocking for §5.** This FOOP carries §5's documentation deliverable as
  **in scope** — `EQUIVALENCE.md` is refreshed against real definitions and moved out of
  `vintage_legacy/`. What is NOT settled is **how far the operator taxonomy goes**, and the two
  readings are very different sizes:

  - **(a) The relation only.** Implement the Rust-side relation of §1–§4, and refresh
    `EQUIVALENCE.md` to describe it honestly — marking `=s=`, `===`, `=n=`, `=c=`, `=v=` as *sketch,
    not implemented*. This is **contained**: it delivers FOOP-36's T2c (`spp` vs `spr`), closes
    `FOOP-23.plan.md` §D.4, and adds no surface syntax, no lexer change, and no einmo cases.
  - **(b) The full vocabulary as language operators.** Specify and implement `=s=`, `==`, `===`,
    `=n=`/`=N=`, `=c=`/`=C=`, `=v=`/`=V=` as Foolish operators with surface syntax. This is
    **considerably larger**: each needs a defined semantics (and `===`, "equal under all possible
    coordinations", may have no decision procedure at all — see §5's point 2), lexer and parser work,
    FIR representation, step rules, and an einmo suite. It is plausibly several FOOPs.

  The design below is written for (a) and does not foreclose (b). **This FOOP does not decide it.**
  §5's last bullet already frames it as a question for the human; this entry is where the answer
  gets recorded.

- **Q2 — Does this FOOP wait on FOOP-66's Q5, or proceed in parallel? HUMAN'S CALL.**
  **FOOP-66 §3 Q5** ("Equality — the sharpest candidate synergy, and this FOOP's highest-value
  target") studies whether tree calculus's equality machinery transfers to FIR equality. It names
  this design directly: **Q5c** asks whether tree calculus has anything resembling §4's
  equality-with-conditions, noting a residual of required identifications is close to **unification**
  and that the framing may be worth importing even if the machinery is not; **Q5b** predicts the
  honest answer for **creation identity** is *no*, since an unlabeled tree is entirely determined by
  its shape; **Q5d** asks whether the absence of a normal form makes Foolish equality a
  *fundamentally different* question rather than a harder version of the same one — which, if yes,
  bounds what any import could achieve; and **Q5e** asks whether tree calculus informs
  `EQUIVALENCE.md`'s taxonomy, `===` especially. FOOP-66's own INDEX entry calls the dependency
  **soft**: reading it first "costs days and saves a possible rewrite."

  Two options: **wait** for Q5's finding before Phase 2, or **proceed in parallel** and incorporate
  the finding if it lands in time. Proceeding is defensible — FOOP-66 is a study with no code, and
  Q5b's expected answer is negative on the hard case — but Q5c and Q5d could change §4's framing,
  which is the expensive thing to change late.

- **Q3 — Where does the relation live?** `foolish-ubca2` (it reads `FVMStorage`) or lower. §6 records
  the tradeoff; this is an implementation judgment rather than a human decision, but it should be
  made deliberately in Phase 1 rather than by default.

- **Q4 — Is `pp` vs `pr` worth implementing alongside?** FOOP-36 §2.2's third row, as a debugging aid
  that isolates a parse/render fault from an evaluation fault. Cheap once the relation exists. §6.

- **Q5 — Is the `docs/howto/` equivalence chapter in scope?** `docs/howto/03_howto_foolish_todo.foo`
  lists **equivalence** among its unwritten chapters, and a refreshed `EQUIVALENCE.md` makes that
  tutorial writable. §5 flags it as "in scope or a follow-on" without deciding.

## References

- **FOOP-76** (deprecated 2026-10-16, absorbed here as Part C). Its own references, carried:

- **FOOP-36** §N6 — **the source of this document.** Broken out 2026-09-15 at the human's request;
  §N6 is now a pointer here. FOOP-36 keeps **§2.2** and **§2.2.1** (WHICH pair to compare — `spp` vs
  `spr`, and why it is the load-bearing one), its **§Test Plan T2c** (deferred here), its
  **§Rejected Alternatives F** (the superseded position, and the record that Property 3 was enforced
  by reading), and its **§Open Questions Q9** (DISSOLVED — see §3).
- **FOOP-33** — The Creation Postulate, which §3.2's bijection requirement is derived from.
- **FOOP-23** — value search; its §Open Questions pin value-search equality at **integer equality
  only, pending an equivalence FOOP**. `FOOP-23.plan.md` §D.4 carries an unchecked `EQUIVALENCE.md`
  task this FOOP inherits and should close (§5).
- **FOOP-66** §3 **Q5** (and Q5a–Q5e) — Learn Tree Calculus, whose equality findings are a soft input
  to this FOOP. See §Open Questions Q2.
- **FOOP-62** §Terminology, **FOOP-56** — constanic / constantew / conclusive, the vocabulary §3.3's
  precondition is stated in.
- `docs/why/creation_postulate.md` — the postulate itself.
- `docs/vintage_legacy/EQUIVALENCE.md` — the vintage operator taxonomy this FOOP revives, refreshes,
  and relocates (§5).
- `docs/howto/03_howto_foolish_todo.foo` — lists **equivalence** among unwritten tutorial chapters.
- `foolish-ubca2/src/fvm_storage.rs` — `FirSpec` (`:304`, deriving `PartialEq` at `:303`),
  `foolish_children` (`:472`, `:1676`), `ubc_children` (`:1680`),
  `compose_program_with_system` (`:4944`), `ib_search_with_engine` (`:2908`),
  `ab_search_with_engine` (`:2948`).
- `foolish-ubca2/src/system_foo.rs` `:106` — `SYSTEM_FOO_SRC`, the shared ancestor §3.1 seeds from.
- `AGENTS.md` §"The agent is responsible for correctness"; `foop.md` §"Promotion Review Gate".

  *(FOOP-76's `fvm_storage.rs` line numbers predate FOOP-96's split and are stale; see §FIR Impact.)*
- **FOOP-36**: §2 (Properties 1–3), §2.1 (pre-constanic exemption), §2.2 (`spp`/`spr`/`pp`/`pr`),
  §T2, §T2c, §T3, §N1–§N4 (rendering difficulties B1, B5–B7, B11).
- **FOOP-07**: §2 and §4.3 (planning draft on branch `worktree-agent-a5e87dc31591ca72a`, commit
  `2b0aec55`). Part B's B1, B12, and the contexted-search ruling.
- **FOOP-23** §"Two degeneracies": wrong on bare-brane contexted search per the human's 2026-10-16
  ruling (Part B).
- **FOOP-65**, **FOOP-75**, **FOOP-95**: Part B's B8, B9, and catalogue item E13.
- Code: `foolish-ubca2/src/sequencer.rs` (`foolish_rendering_round_trips_constanic_programs`),
  `foolish-ubca2/src/einmo_gates.rs` (`einmo_corpus_wide_foolish_rendering_parses`).

## Last Updated

**Date**: 2026-10-16
**Updated By**: Claude Code / claude-opus-5-5
**Changes**: States plainly that **E1 and E2 are both required and both implemented by this FOOP**,
in the Abstract, in a table at the head of Part A, in §A.1/§A.2's headings ("required"), and in §D.1's
Phase 4 row. Part B is headlined "working towards E1 (and E2)"; Part C is headlined "working towards
E2". Earlier the same day: created; absorbed FOOP-76 as Part C; Part A catalogue E1–E13 with E2's four
parameters; Part B difficulties B1–B13; Part D's four phases and the §D.4 concurrent-efforts rule.
