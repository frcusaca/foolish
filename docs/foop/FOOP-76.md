---
foop: D67
title: Revival of Equality — FIR equivalence with conditions
author: Claude Code / claude-opus-5 (directed by the human)
status: Draft
type: Standards
created: 2026-09-15
phase: phase-4
supersedes: []
begun: [ ]
---

# FOOP-76: Revival of Equality

FOOP numbering is little-endian; the full rules live in `foop.md` at the repository root —
**read it before creating or editing a FOOP.** The `foop:` front-matter field here is the
big-endian sort key preceded by `D` (`foop: D67`, file `FOOP-76.md`, following FOOP-66's 66).

> **Broken out of FOOP-36 §N6 on 2026-09-15, at the human's request** ("Can you please break it
> out of FOOP-36 call it Revival_of_Equality"). Every substantive claim, justification and worked
> example in §N6.1–§N6.5 of `FOOP-36.md` is carried here; that section is now a pointer to this
> document. FOOP-36 keeps §2.2 and §2.2.1, which establish WHICH pair to compare — a property of
> its round trip — while this FOOP defines HOW to compare. The design below is the product of a
> long conversation with the human across 2026-09-07 and 2026-09-15; the dated parentheticals
> record whose judgment settled what, and they are load-bearing.

## Abstract

This FOOP specifies **FIR equality** for `foolish-ubca2`: a relation over two FIR subtrees that
answers not `true`/`false` but one of three things — `NO`, `YES`, or **`YES, provided
[(fvm1_creation1, fvm2_creation10), …]`**. The residual is the set of creation identifications
that would have to hold for the two subtrees to be equal, and the caller decides whether those
identifications are acceptable for its purpose. The boolean is recovered as "is the residual
empty?", so there is one function and the boolean is a thin wrapper over it.

The relation has a **shape** half (kind, arity, children over `foolish_children`, plus the
shape-bearing `FirSpec` fields) and a **value** half (integers by value; creations by a
correspondence discovered during a lockstep walk and recorded in a pair list that must be a
**bijection**). Its defining setting is **two FVMs producing two trees**, where pointer identity
is meaningless by construction and the pair list is the only thing that can relate the two sides.
Because every FVM composes the same `SYSTEM_FOO_SRC`, the pair list starts **seeded** with the
system creations rather than empty. Version 1 requires **constanic** subtrees as inputs and says
so as a precondition.

Alongside the implementation, this FOOP carries a **documentation deliverable**: refreshing
`docs/vintage_legacy/EQUIVALENCE.md` against the definitions established here and bringing it OUT
of `vintage_legacy/` into the live documentation tree.

## Motivation

**The "revival" in the title is the point.** `docs/vintage_legacy/EQUIVALENCE.md` sketched a family
of Foolish equality operators years ago — `=s=`, `==`, `===`, `=n=`/`=N=`, `=c=`/`=C=`,
`=v=`/`=V=` — as *language* operators with proposed surface syntax. None of them was ever
specified, and none was ever implemented. The document did not go away, though: **FOOP-23 has been
deferring to it ever since**, recording in its §Open Questions that value-search equality means
**integer equality only, pending an equivalence FOOP**, and leaving an unchecked task in
`FOOP-23.plan.md` §D.4 to note as much in `EQUIVALENCE.md`. A sketch that other work defers to is
not a dead document; it is an unpaid debt. **This FOOP is that equivalence FOOP.** It revives the
thread with real definitions in the places the vintage document had only examples.

**Today**, the repository has no FIR equivalence relation at all. FOOP-36 needed one and could not
have it: its Property 3 — "`R` MEANS what `P` meant" — is a stated REQUIREMENT of `Ubca2Sequencer`
that is **enforced by reading**, because the mechanical check does not exist (FOOP-36 §Rejected
Alternatives F). FOOP-36 §2.2 goes as far as naming the pair that would check it — `spp` vs `spr`,
the two stepped FIRs, two computations of the same limiting fixed point — and then stops, because
defining the comparison turned out to be a feature in its own right. Its test **T2c** is recorded
and explicitly deferred to this FOOP.

**After this FOOP**, Property 3 has a mechanical check, and more than one caller gains something
it could not previously ask for:

- **Testing** — FOOP-36's `spp` vs `spr`, and any future check that two FIRs agree. A comparison
  that returns *why* and *how close* is far more useful under test than a bare `false`.
- **Comprehending programs** — the larger reason, and the human's own framing. *"Under what
  identifications are these two branes the same?"* is a question a Foolisher asks while **READING**
  code, not only while testing it. That is what lifts this from a test utility to **language
  documentation**, and it is why `EQUIVALENCE.md` belongs in the live documentation tree rather
  than the legacy pile.
- **Subtree comparison** and *"do these two differently-written programs denote the same thing?"*
  — both natural users of the relation, and neither expressible today.

**And the residual is a better primitive than the thing the vintage document reached for.** `===`
in that taxonomy means "equal under ALL possible coordinations", which quantifies over every
context and is correspondingly hard to check. `YES, provided […]` is the **constructive** form of
the same instinct: instead of asserting equality under all coordinations, it hands back the exact
conditions under which equality holds and lets the caller judge them.

## Specification

The design is split into a shape half (§1), a value half (§2), the two-FVM setting that is its
defining case (§3), the conditional signature that generalizes both (§4), and the documentation
deliverable (§5).

### §1 The shape half — kind, arity, children

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

### §2 The value half — integers are easy, creations are dynamic programming

**Integers.** `IndepInt { value }` compares by value. Two `3`s are equal; a `3` and a `4` are not.
Nothing more is needed: an integer literal denotes itself, and re-parsing a rendered `3` yields a
`3`.

**Creations are the interesting case** (human, 2026-09-07). A creation denotes nothing but its own
identity, and identity is arena-scoped — `FirPointer` is meaningless across `spp` and `spr`. So
creation equality cannot be decided by looking at either node: it is a **correspondence discovered
during the walk**, built by dynamic programming over an equality table.

#### §2.1 The table is built with respect to IDENTICAL TREE TRAVERSAL

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

#### §2.2 The rule — three cases at every creation pair

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

#### §2.3 Reflexive pairs — an optimization inside one comparison, never a design assumption

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

#### §2.4 On redundancy — noted, not derived; and why the table ships anyway

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

### §3 Two FVMs is the defining case; name lookup is ordinary scoping

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

#### §3.1 System equality — the comparison starts seeded, not empty

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

#### §3.2 The residual must be a BIJECTION — the Creation Postulate demands it

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

#### §3.3 NYES: require constanic inputs first, generalize later

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

### §4 Equality with CONDITIONS — return the mapping, not a boolean

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

#### §4.1 What conditions being first-class opens up

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

### §5 `EQUIVALENCE.md` — refreshed, and brought out of `vintage_legacy/`

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

### §6 Scope beyond what is specified above

Recorded from FOOP-36 §N6's handover note, for whoever executes this FOOP:

- Whether the relation is a **test helper** or a **language-level notion** Foolish itself can
  express (this is Q1's larger half).
- Whether it belongs in `foolish-ubca2` or lower. The design as written reads `foolish-ubca2`'s
  `FVMStorage`, which argues for `foolish-ubca2`; a lower home would need the storage abstraction to
  move with it.
- Whether **`pp` vs `pr`** (FOOP-36 §2.2's third row — the unstepped FIRs) is worth implementing
  alongside as a debugging aid. It isolates a parse/render fault from an evaluation fault when the
  stepped comparison fails.

## FIR Impact

**The relation reads FIR and adds nothing to it.**

- **No new `FirSpec` variant.** The relation is a walk over the existing enum, dispatching on the
  discriminant and reading the shape-bearing fields §1 lists.
- **No change to any existing variant.** `FirSpec` already derives `PartialEq`
  (`foolish-ubca2/src/fvm_storage.rs:303` — `#[derive(Debug, Clone, PartialEq)]`), which is what
  makes the field comparison of §1.4 a paired walk rather than a new equivalence theory. *Verify,
  don't re-derive.*
- **No NYES state-machine change.** §3.3 makes constanic input a **precondition** of v1, so the
  relation observes NYES to check that precondition and never writes it.
- **No serialization implication.** Nothing new is stored, so no einmo envelope or snapshot shape
  changes.

The accessors the walk uses already exist and are read-only: `FVMStorage::foolish_children`
(`fvm_storage.rs:472`, and the borrowed-view form at `:1676`) and the `FirSpec` held per node. The
one thing this FOOP adds is a **new data structure of its own** — the creation pair list / residual
— which lives in the relation's module and is never attached to a FIR.

**If implementation finds that `FirSpec` must change**, that is a scope change and must be reported
to the human before it is made, not taken as an implementation convenience.

## UBC Step Impact

**None.**

The relation is **read-only over already-constanic FIR** and constructs nothing:

- **Constanic inputs are a precondition** (§3.3), so there is no stepping to interact with. Both
  subtrees have finished; what they are is what they will remain.
- **It takes `&FVMStorage`**, an immutable borrow. This mirrors FOOP-36 §N4.b's finding that
  `search_engine::contextful_search_scan` takes `&FVMStorage` — the same property that allows a
  read-only sequencer to ask a real search question. If this FOOP's relation reuses `ib_search` /
  `ab_search` for the name lookup of §3, it inherits that immutability rather than needing a new
  guarantee.
- **It creates no FIR.** The only thing built is the pair list, which is the relation's own return
  value and is never stored in the arena.

So no step rule is added, none is changed, and there is no interaction with constanic coordination.
A caller may of course step a program *before* calling the relation — FOOP-36's T2c does exactly
that — but the stepping is the caller's, not this FOOP's.

## Test Plan

Several tests are **already specified by the design** and are carried here as requirements. They are
unit tests in Rust (the relation is internal FVM machinery with no surface syntax in v1), living
alongside the relation's implementation in `foolish-ubca2`.

### T-BIJ — the bijectivity counterexample (§3.2) — REQUIRED, both directions

**The human explicitly asked that this counterexample be written down as a test**, to make certain it
is handled. Two distinct VM1 creations against one VM2 creation reached twice must answer **`NO`**:

```
[(vm1_a1, vm2_a1), (vm1_a2, vm2_a1)]        ← must be refused
```

Because `vm1_a1 ≠ vm1_a2` by the Creation Postulate while `vm2_a1 = vm2_a1` trivially, this asserts
that two things known to differ are both equal to one thing. **Checked in BOTH directions** — one
left creation paired to two different right creations is equally impossible — since the requirement
is a bijection, not a function.

### T-SEED — system equality seeds the pair list (§3.1)

Two independently-built FVMs, each composed through `compose_program_with_system` from the same
`SYSTEM_FOO_SRC`. Assert:

- Their system creations correspond — `fvm1`'s `'True` ↔ `fvm2`'s `'True`, and likewise the number
  definitions — **by construction**, with no user declaration.
- **Seeded pairs never appear in a residual.** A comparison whose only correspondences are system
  ones answers `YES`, not `YES, provided ['True ≡ 'True]`.
- **A system creation paired against a user creation is a hard `NO`**, not a condition.

### T-SIG — the three-outcome signature (§4)

One test per outcome, on inputs chosen so the outcome is unambiguous:

1. **`YES`** — the walk completes with an empty residual.
2. **`YES, provided […]`** — the walk completes with a non-empty residual, and the residual contains
   exactly the pairs the walk had to assume (no seeded pairs, no reflexive pairs — §2.3).
3. **`NO`** — three sub-cases, since §4's outcome 3 has three sources: a shape mismatch, an integer
   mismatch, and a contradicted creation pairing.

Plus the wrapper: the boolean form agrees with "is the residual empty?" on every one of the above.

### T-SHAPE — the shape half (§1)

- **Arity** catches FOOP-36's fused `f1f2` concatenation defect: a `Concatenation` over 3
  constituents is not one over 2.
- **Shape-bearing fields** are compared: `?x` against `~x` is a `NO` even though kind, arity and
  children all agree.
- **Ignored fields are ignored**: differing `Statement { line_number }`, differing `Nk { reason }`,
  and a differing `Concatenation { rendering_aid }` must not by themselves produce a `NO`.
- **Order of failure** (§2.1): a shape mismatch fails *before* the creation table is consulted at
  that position. A test that would produce both a shape failure and a table failure must report the
  shape one.

### T-SHARE — the creation table catches split sharing (§2.2)

`{orig = ⬤; ref = orig;}` — one creation reached twice — compared against a tree with TWO distinct
creations in those positions. The first comparison records the pairing; the second finds the left
creation already paired to the other right creation, and case 3 fails. **Sharing preserved passes;
sharing split fails.** Note per §2.4 that in FOOP-36's own T2c inputs the shape half tends to catch
this first (node counts 7 against 5); this test constructs the FIR so the table is genuinely the
thing under test.

### T-PRE — the constanic precondition (§3.3)

v1 states constanic inputs as a requirement. Whichever enforcement this FOOP's plan chooses — type
system, assertion, or a checked error return — there is a test that a pre-constanic input is
**rejected**, not silently compared.

### T2c — FOOP-36's deferred structural-equivalence test

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

### What is NOT tested in v1

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

The phases of this FOOP differ sharply in what they demand, so they are assigned individually rather
than sized as a block. Harness-specific names as of writing: Claude — Opus / Sonnet for judgment,
Sonnata for execution; Codex — GPT-terra; local — Qwen3.8-27B.

| Phase | Character | Needs |
|---|---|---|
| **0** — Begin, and answer Q1/Q2 | Two **blocking** scope questions for the human | **Larger model**, and it must **stop and ask**, never decide Q1 itself |
| **1** — Read the spec and `EQUIVALENCE.md`; place the relation in its taxonomy | Reading and judgment; the answer shapes §5's rewrite | **Larger model** |
| **2** — Core algorithm: lockstep walk, shape half, order of failure | The spec's §1/§2.1 are precise, but the failure-ordering invariant is subtle | **Larger model.** Getting the order wrong silently degrades every diagnosis |
| **3** — Creation table and the bijection check | The Creation-Postulate reasoning decides what is a hard `NO` vs a condition | **Larger model.** §3.2 is a language-semantics argument, not a coding task |
| **4** — The `NO`/`YES`/`YES-provided` return type and the boolean wrapper | Mechanical once §4 is read; the shape is given in the spec | Smaller model |
| **5** — System seeding (§3.1) | Walk two system branes in lockstep, pair creation to creation | Smaller model; the procedure is stated and the target is fixed |
| **6** — Test scaffolding for T-SIG, T-SHAPE, T-PRE | Fixed targets — each test's expected outcome is named in §Test Plan | Smaller model |
| **7** — T-BIJ and T-SHARE, and their expected answers | **Hand-written expectations the design depends on** | **Larger model.** Generating these and calling them hand-written would void the test |
| **8** — `EQUIVALENCE.md` refresh and relocation (§5) | Writing language documentation; marking spec'd vs sketch | **Larger model** |
| **9** — Any einmo promotion (only if Q1 puts an operator in scope) | Promotion is a correctness claim | **Larger model** |
| **10** — Merge and cleanup | Mechanical, with a human STOP | Smaller model |

**What makes the small-model phases safe.** Three properties, built in deliberately:

1. **Facts are inline, not referenced.** The plan carries the verified code facts it needs —
   `FirSpec` derives `PartialEq` at `foolish-ubca2/src/fvm_storage.rs:303`; `foolish_children` is at
   `:472`; `SYSTEM_FOO_SRC` is at `foolish-ubca2/src/system_foo.rs:106`; `compose_program_with_system`
   at `fvm_storage.rs:4944` — each marked *verify, don't re-derive*, so an executing agent spends its
   context on the work rather than on rediscovery.
2. **Each phase has a fixed target.** After Phase 7 there are hand-written expected answers; Phases
   4–6 each match a shape or an outcome that a larger model already wrote down. "Does this match the
   thing someone wrote?" is a far easier question than "is this right?".
3. **The stop conditions are named.** Phase 0's Q1 and Q2 are blocking. A standing scope guard says:
   if implementation appears to require a change to `FirSpec` (§FIR Impact) or to any crate outside
   `foolish-ubca2`, **STOP and report** — do not take it as an implementation convenience.

**What must not be delegated**, regardless of model size (AGENTS.md §"The agent is responsible for
correctness"):

- **Every `output` → `checked` promotion**, if any arise under Q1.
- **The hand-written expectations of Phase 7** — T-BIJ in particular. The human asked for that
  counterexample specifically; generating its expected answer from the implementation would test
  nothing.
- **Answering Q1** (the operator-taxonomy scope question). It is the human's call, stated as such.
- **Any decision to change `FirSpec`** or a crate this FOOP promised not to touch.
- **Marking any Verified-tier test `#[ignore]`** — never an agent's call.

## Rejected Alternatives

### A. Do nothing — leave Property 3 enforced by reading

This is the status quo FOOP-36 inherited and lives with. It is survivable but demonstrably costly:
**two defects entered through exactly this gap** — the fused `f1f2` concatenation and the `⬤`
unnamed creation (FOOP-36 §N4) — each satisfying rendering Properties 1 and 2 while meaning
something else, and each found by a human reading output rather than by a test. Doing nothing also
leaves FOOP-23's value-search equality pinned at integer-only indefinitely, and leaves
`EQUIVALENCE.md` as an unspecified sketch that other FOOPs defer to. Rejected.

### B. A plain boolean relation, with the residual added later

Simpler to write and adequate for FOOP-36's own use, which wants the strict reading anyway. Rejected
because **retrofitting a residual onto a boolean means changing the return type of every path**
(§4), and because the boolean answers `false` without saying *why* or *how close* — which is
precisely the information a caller comparing two independently-built arenas needs. The boolean is
kept, as a wrapper over the general form.

### C. Compare by pointer identity, or by arena index

The obvious implementation, and wrong by construction. `FirPointer` is **arena-scoped**, so two
creations from different FVMs are always distinct pointers even when they denote the same thing
(§3). Pointer comparison would answer `NO` for every cross-FVM pair, making the defining case of the
relation unanswerable. It survives only as an optimization *inside* one comparison — same arena, same
pointer → equal, record nothing (§2.3) — never as the design.

### D. Global creation table with no bijection check

Record pairs as they are met and never check for contradiction. Rejected from the **Creation
Postulate** (§3.2): a table that pairs `vm1_a1` and `vm1_a2` both to `vm2_a1` asserts that two things
known to differ are equal to one thing. Without the both-directions check, the relation would answer
`YES, provided …` to a condition that cannot be discharged — worse than answering `NO`, because it
looks like an answer.

### E. Per-brane creation tables, created on brane entry and discarded on exit

The earlier draft's mechanism, and the source of FOOP-36's Q9. Rejected because it **conflates two
distinct jobs** (§3): name lookup within one tree, which is ordinary nested scoping, and creation
correspondence between two trees, which is what the pair list is for. The mechanism invented a table
lifetime the language does not have, then asked what happens to a creation that outlives it. The
question dissolved when the mechanism was dropped.

### F. Compare pre-constanic FIR too, in v1

Rejected for v1 by §3.3, and the rejection is a staging decision rather than a permanent one. Two
pre-constanic subtrees may be identical now and diverge on the next step, so the answer is not
stable; and **no identified caller wants it** — FOOP-36's `spp` vs `spr` compares stepped trees, and
`EQUIVALENCE.md`'s questions are about settled meaning. §3.3 records what the later version must
decide so v1 does not foreclose it.

## Open Questions

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

## Last Updated

**Date**: 2026-09-15

**Updated By**: Claude Code / claude-opus-5

**Changes**: Created. **Broken out of FOOP-36 §N6** at the human's request ("Can you please break it
out of FOOP-36 call it Revival_of_Equality"), carrying §N6.1–§N6.5 forward as §1–§5 with every
substantive claim, justification, worked example and dated human attribution preserved. FOOP-36 §N6
is now a pointer to this document; FOOP-36 keeps §2.2/§2.2.1 (which pair to compare), and every
reference that pointed into N6's subsections was repointed here. Added the sections the spec template
requires and §N6 did not carry as such: **§FIR Impact** (the relation adds nothing to `FirSpec`,
which already derives `PartialEq`), **§UBC Step Impact** (None — read-only over constanic FIR, takes
`&FVMStorage`, constructs nothing), **§Test Plan** (T-BIJ, T-SEED, T-SIG, T-SHAPE, T-SHARE, T-PRE,
and FOOP-36's deferred T2c), **§Plan of Execution for Plan** (per-phase model assignment), and
**§Rejected Alternatives** A–F. **§Open Questions Q1** records the undecided scope call between the
relation alone and the full operator taxonomy — the human's, not this document's — and **Q2** the
sequencing question against **FOOP-66 §3 Q5**. §3.2 carries N6.3.2's **worked counterexample**
verbatim, including its Foolish code block, since the human asked specifically that it be written
down; it is also T-BIJ in §Test Plan.
