---
foop: D66
title: Learn Tree Calculus — a study of Barry Jay's tree-based calculus and its synergies with Foolish
author: Claude Code / claude-opus-5 (directed by the human)
status: Draft
type: Standards
created: 2026-09-15
phase: phase-5
supersedes: []
begun: [ ]
---

# FOOP-66: Learn Tree Calculus

FOOP numbering is little-endian; the full rules live in `foop.md` at the repository root —
**read it before creating or editing a FOOP.** The `foop:` front-matter field here is the
big-endian sort key preceded by `D` (`foop: D66`, file `FOOP-66.md`, following FOOP-56's 65).

## Abstract

**This is a learning FOOP. Its deliverable is understanding, written down — not code, not a
language change.** It commits the Foolish team to a structured study of **tree calculus**, the
tree-based theory of computation discovered by Barry Jay and set out in his book *Reflective
Programs in Tree Calculus* (2021), and to producing **one written assessment** of where tree
calculus and Foolish genuinely correspond, where the analogy is false, and whether anything in
Jay's rigorous treatment is worth borrowing.

It proposes **no** FIR variant, **no** step rule, **no** syntax, and **no** change to any crate.
Nothing in `foolish-core`, `foolish-parser`, `foolish-ubca`, `foolish-ubca2`, `foolish-cli`, or
either einmo suite is modified by executing this FOOP. The output is prose in `docs/`, plus a
small number of illustrative `.foo` snippets written only to make a correspondence concrete.

The one place where a concrete payoff is already visible is **equality**. Foolish has an open
design question here right now — FOOP-36 §N6 recommends a dedicated FIR-equality FOOP and §N6.4
proposes that the relation return *the mapping of creations that would have to be equal* rather
than a boolean. Tree calculus is rigorous about tree structure and, per its author's own summary,
contains "an equality program that can decide its own equality." Whether that machinery
transfers to FIR equality is precisely the kind of question this study exists to answer, and it
is the study's highest-value target. **But the FOOP does not assume the answer is yes.** Finding
that tree calculus has nothing to offer Foolish is an acceptable and useful result, provided the
finding is argued.

**The study aims at programmable and computable systems, while keeping some concepts open for
implementation by approximation** (human, 2026-09-15). Some things are most honestly stated in an
idealized form — a countably-long `integers.foo` creating the integers, an `aleph_1`-cardinality
`reals.foo` creating the reals — and then implemented approximately in the FVM with IEEE integers
and floats. Tree calculus is rigorous about exactly the layer where that idealization lives, so
how it separates a definition from its implementation is a question this study asks directly
(§3 Q8). The practical target is the *seam*: how an ideal definition and its finite stand-in
relate, and where the approximation must be visible to a Foolisher.

## Motivation

**Two people started building tree-based systems at about the same time, and took opposite
roads.** Barry Jay took the rigorous one: a calculus with a single primitive, five reduction
rules, confluence, a machine-checked Coq development, and a book. The Foolish BDFL took the
other: a language grown from use, whose central object — the **brane** — is a containment
structure discovered by writing programs in it rather than derived from an axiom set.

Neither road is the correct one in general, but they are unusually complementary here, and the
parallel is close enough to be worth taking seriously:

- Both make **trees the primitive thing**, not a data structure layered over something else.
  Foolish's FIR is a tree; a brane is a node in it; `.parent` chains and `foolish_children` are
  the whole substrate. Tree calculus's expressions *are* unlabeled binary trees, and its values
  are exactly the irreducible ones.
- Both are **intensional** — programs can look at the structure of other programs. Foolish does
  this with searches (`?` `~` `.` `#` `^` `$`, value searches, `&`-contexted searches), which
  read a brane's statements as data and match against their names, positions and values. Tree
  calculus does it with its fork rules (3a–3c below), which branch on whether the argument is a
  leaf, a stem or a fork. Neither needs Gödel numbering or quotation to do it.
- Both have wrestled with **what it means for two structures to be the same**. Foolish's account
  is unfinished (FOOP-36 §N6; `docs/vintage_legacy/EQUIVALENCE.md`, which is a sketch of an
  operator family, unspecified and unimplemented). Jay's account is finished and proved.

**What the world looks like today.** The Foolish team has no shared understanding of tree
calculus at all. When a design question arises that a formal tree theory might have already
answered — the equality question is the live one — nobody can say whether prior art exists,
because nobody has read it. Choices that are actually forced get made as though they were free,
and choices that are genuinely free get agonized over as though they were forced.

**What it looks like after.** A document in `docs/why/` that a Foolisher can read in one sitting
and come away knowing: what tree calculus is, which of its ideas have a Foolish counterpart,
which do not and why not, and a short ranked list of anything worth pursuing — each item either
promoted to a follow-on FOOP or explicitly declined with a reason. The team gains the ability to
say "tree calculus already settles this" or "tree calculus is not about this" and be right.

**Why this is worth the team's attention now, specifically.** FOOP-36 §N6 is queued and not yet
begun; its §N6.4 design (equality-with-conditions, returning a residual of required creation
identifications) is unusual enough that a check against prior art is cheap insurance. Reading
Jay before writing N6 costs days; discovering after N6 ships that a well-proved alternative
existed costs a rewrite. This FOOP should therefore land — or at least reach its §3 equality
finding — **before** N6's FOOP is written.

## Specification

For a study FOOP, the "specification" is *what the study covers, in what order, and what it
produces.* This section fixes the reading plan, the questions to answer, and the shape of the
deliverable, so that the FOOP can be judged complete or incomplete rather than merely "ongoing."

### 1. What tree calculus is — the grounding already established

**This subsection was written from the specification page (https://treecalcul.us/specification/),
fetched 2026-09-15 and reachable.** It is a summary, not a substitute for reading the source. It
is recorded here so the study starts from facts rather than from memory, and so that an executing
agent can *verify* rather than re-derive. **Everything below is to be checked against the source
during Phase 2; anything found wrong is corrected in place, and the correction noted.**

**One primitive.** The single symbol `△` (delta). The grammar is:

```
E ::= △ | E E
```

Application is binary and associates to the left. There are no variables, no binders, and no
second constant.

**Three node kinds**, which are the site's own names:

| Form | Name |
|------|------|
| `△` | *leaf* |
| `△ a` | *stem of a* |
| `△ a b` | *fork of a and b* |

**Five reduction rules.** A redex exists exactly when some subexpression has the shape `△ a b c`
— i.e. the operator is saturated at three arguments:

```
1.  △ △ y z              ⟶  y
2.  △ (△ x) y z          ⟶  x z (y z)
3a. △ (△ w x) y △        ⟶  w
3b. △ (△ w x) y (△ u)    ⟶  x u
3c. △ (△ w x) y (△ u v)  ⟶  y u v
```

Rules 1 and 2 are the site's stated analogues of combinatory logic's **K** and **S**, which is
where Turing completeness comes from. Rules 3a–3c are the *intensional* half: they branch on
whether the third argument is a leaf, a stem or a fork, and so let a program observe the shape of
a value it is handed.

**Values.** "We call irreducible expressions *values* or *programs*." Values are exactly the
binary trees — leafs, stems and forks — because "as long as there is a subexpression `(△ a b c)`,
one of the reduction rules apply."

**Confluence.** The site states: "Tree Calculus is confluent: The five reduction rules are
non-overlapping and only allow observing values, not reducible expressions." The parenthetical
matters — intensionality is confined to *values*, which is what keeps it confluent. A calculus
that let a program inspect an unreduced expression would not be.

**Encoding.** Data is trees. The site's own worked example: `false = △`, `true = △ △`, and
`not = △ (△ (△ △) (△ △ △)) △`.

**Reflection, per the project's own front page** (https://treecalcul.us/, fetched 2026-09-15):
"Since programs are also values, intensional programs can be self-applied, to achieve
introspection and reflection." The front page's worked demonstration is `size size`, which
evaluates to `168` — a program computing its own size, with no quotation mechanism.

**What the book adds, per its published description** (not yet read; to be confirmed in Phase 4):
*Reflective Programs in Tree Calculus*, Barry Jay with Jose Vergara, 2021, self-published, with
machine-checked Coq proofs in the companion repository. The description advertises three
self-applying programs: a **size** program that computes its own size, an **equality** program
that decides its own equality, and a **self-evaluator** that evaluates itself — all "without the
usual outside machinery like Gödel numbering or quotation." **The equality program is this FOOP's
primary target.** Flagged explicitly: this paragraph is publisher-description provenance, the
weakest tier used anywhere in this FOOP. It is a *reason to read*, not a finding, and no
recommendation may rest on it until Phase 4 confirms it against the book or the Coq source.

**The Coq development** (https://github.com/barry-jay-personal/tree-calculus) contains, among
others: `Tree_Calculus.v`, `Reflective_Programs.v`, `Extensional_Programs.v`,
`Intensional_Programs.v`, `Rewriting_partI.v`, `Rewriting_theorems.v`,
`Incompleteness_of_Combinatory_Logic.v`, plus `tree_book.pdf` and `Reflective_Programs.txt`. The
proofs are MIT-licensed. **No Foolish code derives from them and none may** — see §5.

### 2. The reading plan — three depths, in this order

The study proceeds **outside-in**, cheapest source first, and each depth is allowed to terminate
the study early if it answers the questions. This is deliberate: the book is long, and reading it
is only justified once the shallower sources have shown there is something there.

| Depth | Source | Goal | Stop condition |
|-------|--------|------|----------------|
| **D1** | https://treecalcul.us/specification/ and the site's front page | Verify §1 above, line by line. Correct anything wrong. Write the §3 questions' *provisional* answers. | Complete when §1 is confirmed or corrected and every §3 question has a provisional answer or an explicit "the spec page does not say." |
| **D2** | https://github.com/barry-jay-personal/tree-calculus — `README`, `Reflective_Programs.txt`, and the **statements** (not proofs) in `Tree_Calculus.v`, `Intensional_Programs.v`, `Reflective_Programs.v` | Find the equality program and the self-evaluator. Read what is *claimed*, in the form of Coq theorem statements, which are precise and short. | Complete when the equality program's definition and its stated theorem are transcribed into the assessment, or when it is established that no such program is in the repo. |
| **D3** | https://github.com/barry-jay-personal/tree-calculus/blob/master/tree_book.pdf — **the relevant chapters only** | Read the equality chapter and the reflection chapter in full. Read others only if D1/D2 raised a question that needs them. | Complete when §3's **Q5 (equality)** is answered with an argued yes/no, and each remaining §3 question is answered or explicitly declared out of reach. |

**Reading the whole book is NOT a deliverable.** If D1 and D2 answer the questions, D3 is
narrowed to the equality chapter and the study ends there. Say so in the assessment; an
early-terminated study that states why it terminated is a complete study.

### 3. The questions the study must answer

These are the assessment's skeleton. Each gets its own section in the deliverable, and each gets
an answer of one of three kinds: **a finding** (with the evidence), **"no correspondence"** (with
the argument for why the analogy fails), or **"could not determine"** (with what was read and
what would be needed). No question may be silently dropped.

**Q1 — Primitives and irreducible core.** What does tree calculus take as irreducible, and why
those choices? Foolish's irreducible core is the brane, the statement, and the creation `⬤`
(FOOP-33). Compare the two inventories. Is Foolish's core larger than it needs to be, and if so
is that a cost or a deliberate purchase of expressiveness?

**Q2 — Tree versus brane.** How does a tree-calculus tree relate to a Foolish brane? **State
where the correspondence genuinely holds and where it is false.** Some known asymmetries to test
rather than assume:

- Tree calculus nodes are **unlabeled** and arity is fixed at ≤2; Foolish statements carry
  **names**, and a brane is n-ary.
- Tree calculus has no analogue of a **statement position** in the Foolish sense — yet `#N`,
  `^`, `$` and every `&`-search are built on position. Is position expressible in tree calculus,
  and at what cost?
- Foolish has **AB/IB context** and recoordination. Tree calculus's reduction is context-free in
  the technical sense — a redex reduces the same way wherever it sits. This looks like the
  sharpest difference of all; confirm it, and say what it implies.

**Q3 — Intensionality: fork rules versus searches.** Tree calculus inspects structure by
branching on leaf/stem/fork (rules 3a–3c); Foolish inspects structure by search. Compare them on:
what each can observe, what each cannot, and what each does on a miss. Note specifically that
tree calculus's rules 3a–3c are **total** — every value is a leaf, a stem or a fork, so the
inspection always succeeds — whereas a Foolish search can **miss**, and the miss has two distinct
outcomes (anchored ⇒ NK, unanchored ⇒ ECONSTANIC). **Is the totality of tree calculus's
inspection an advantage Foolish gave up, or a consequence of it having nothing like a name to
look up?**

**Q4 — NYES and the constanic ladder.** Does tree calculus have anything resembling Foolish's
NYES states, or the pre-constanic / constanic / constantew / conclusive cuts (FOOP-62 §Terminology,
FOOP-56)? The prior expectation is **no**: tree calculus distinguishes only *reducible* from
*irreducible*, a two-state machine where Foolish has eight. If that is right, the interesting
question is the converse one — **what does Foolish buy with the extra states, and is ECONSTANIC
(the state that exists because a name might resolve later, in another context) the one that has
no tree-calculus counterpart at all?** Answer against FOOP-62 and FOOP-56, not from memory.

**Q5 — Equality. The sharpest candidate synergy, and this FOOP's highest-value target.**

Foolish's equality story is **open right now**, and this FOOP exists partly to inform it:

- **FOOP-36 §N6** recommends a dedicated FIR-equality FOOP and carries the design worked out so
  far: a **shape** half (§N6.1 — kind, arity, children, shape-bearing `FirSpec` fields) and a
  **value** half (§N6.2 — integers by value; **creations by a correspondence discovered during
  the walk**, built by dynamic programming over an equality table).
- **§N6.3** scopes that table **per-brane**, not globally.
- **§N6.4** is the design most worth testing against Jay: rather than a boolean, the relation
  returns **the mapping of creations that would have to be equal for the two FIRs to be equal** —
  three outcomes (unconditionally equal / conditionally equal with a residual / not equal), with
  the boolean recovered as "is the residual empty?".
- **§Open Questions Q9** of FOOP-36 — how a creation shared *across* brane boundaries is checked
  — blocks N6 and is explicitly the human's call.
- **§N6.5** requires that N6's FOOP reconcile with and refresh
  `docs/vintage_legacy/EQUIVALENCE.md`, a vintage taxonomy of Foolish equality *operators*
  (`=s=`, `==`, `===`, `=n=`, `=c=`, `=v=`, …), none specified or implemented.

Against that, the questions to put to tree calculus:

- **Q5a** — Does tree calculus's equality program decide **structural** equality of trees, and is
  that all it decides? Transcribe its actual definition and its Coq theorem statement.
- **Q5b** — Foolish's hard case is **creation identity**: `⬤` denotes nothing but itself, and
  identity is arena-scoped, so it cannot be read off either node. Does tree calculus have
  *anything* whose identity is not determined by its structure? The expectation is **no** — an
  unlabeled tree is entirely determined by its shape — in which case **the honest finding is that
  Jay's equality does not address Foolish's actual difficulty**, and saying so clearly is the
  valuable result. Do not manufacture a synergy to avoid a negative finding.
- **Q5c** — Is there anything resembling §N6.4's **equality-with-conditions**? A residual of
  required identifications is close to unification, and unification is well-trodden; if tree
  calculus (or the literature it cites) frames it that way, that framing is worth importing into
  N6's FOOP even if the machinery is not.
- **Q5d** — Tree calculus is **confluent**, so every expression has a unique normal form, so
  equality can be decided by *normalizing and comparing*. Foolish has no such guarantee: an
  ECONSTANIC is settled but not conclusive, and recoordination may still give it a value. **Does
  the absence of a normal form make "Foolish equality" a fundamentally different question rather
  than a harder version of the same one?** If yes, that is a first-class finding and it belongs at
  the top of the assessment, because it bounds what any import from Jay could achieve.
- **Q5e** — Does anything in tree calculus's equality inform `EQUIVALENCE.md`'s operator taxonomy
  — specifically `===`, "equal under ALL possible coordinations", which is the one relation in
  that family with no obvious decision procedure?

**Q6 — Simplification, or a check on a decision already made.** Is there anything in tree
calculus that suggests Foolish is carrying complexity it does not need — or, equally valuable,
that **confirms** a Foolish design choice that has so far been justified only by taste? A
confirmation is a finding. Candidates to weigh: the number of search operators; whether `.`-deepen
and `&`-navigate are genuinely two things; whether the eight NYES states are all load-bearing.

**Q7 — What tree calculus is NOT about.** A study that only reports similarities is not a study.
Name the things Foolish is centrally concerned with that tree calculus does not address at all —
candidates: names and name resolution, statement order and position, context and recoordination,
alarms and diagnostics, source rendering. This section protects the team from over-applying the
analogy later, and is expected to be one of the longer ones.

**Q8 — Idealized definition versus approximated implementation.** (human, 2026-09-15) **The
study must lead toward programmable and computable systems, while keeping some concepts open for
implementation by approximation.** Foolish already has this pattern in view: one can imagine a
countably-long `integers.foo` that *creates* the integers, and an `aleph_1`-cardinality
`reals.foo` that creates the reals — idealized definitions that say what the things ARE — while
the FVM implements them approximately, with IEEE integers and floats. The idealized definition is
the specification; the machine representation is a finite stand-in for it.

This is a question tree calculus is well placed to inform, because it is rigorous about exactly
the layer where the idealization lives. Ask:

- Does tree calculus distinguish an idealized definition from its implementation, and if so
  where does it draw the line? Its values are finite trees, so an infinite construction cannot be
  a value — how does the theory talk about such objects, if at all?
- Foolish's `⬤` creation is already a "make a new thing" primitive with no internal structure.
  Is it the right foundation for an idealized `integers.foo`, or does tree calculus suggest a
  different construction?
- **Where must an approximation be VISIBLE?** If the FVM substitutes an IEEE float for an
  idealized real, a Foolisher reading the program should be able to tell. Does tree calculus
  offer any discipline for marking the seam between a definition and its approximation — or is
  this a place where Foolish needs an idea that tree calculus does not supply?
- What does it cost to state the ideal and implement the approximation *separately*, rather than
  defining numbers as whatever the machine does? This is the question that decides whether the
  pattern is worth adopting broadly.

**The point of the question is the seam, not the arithmetic.** Numbers are the clearest example,
but the pattern generalizes: any construction whose honest definition is infinite, or merely
larger than a machine can hold, will need an idealized statement and an approximate
implementation. Establishing how those two relate is the practical payoff this study is aiming
at — programmable and computable systems, with the idealizations kept open rather than defined
away.

### 4. The deliverable

**One document: `docs/why/TREE_CALCULUS.md`.**

`docs/why/` is the right home — AGENTS.md describes it as "Philosophy of Foolish — origins,
inspirations, design philosophy," and a comparative study of a parallel-invented tree theory is
exactly that. It is not `how/` (no engineering procedure), not `howto/` (not a tutorial), and it
must not go to `vintage_legacy/`, which is for documents being moved *out*.

Required structure:

1. **What tree calculus is** — a correct, self-contained summary, at the depth of §1 above,
   written so a Foolisher who has never heard of it can follow the rest. Corrected against the
   sources actually read.
2. **Q1 through Q8, one section each**, each with a finding, a "no correspondence", or a "could
   not determine" — and its evidence.
3. **Worked correspondences.** At least **two**, at most about five: a small tree-calculus
   expression set beside the nearest Foolish program, with an explicit statement of what the
   translation preserves and what it loses. The `false`/`true`/`not` encoding is one natural
   candidate (Foolish has `'True`/`'False` from FOOP-33's `system.foo`); the fork rules' leaf/
   stem/fork dispatch beside a Foolish search is another. **Any Foolish snippet must be written
   in the Unicode operator forms** (`⬤`, `<̲`, `=̲=̲`, …) per AGENTS.md §Code Style, and must
   actually parse — check it, do not eyeball it.
4. **Findings, ranked.** A short list, most valuable first. Each item is exactly one of:
   **(a)** promote to a follow-on FOOP — name what it would specify; **(b)** feed into FOOP-36
   §N6's FOOP as an input — state precisely which sub-section (N6.1/N6.2/N6.3/N6.4/N6.5) and what
   it changes; or **(c)** decline, with the reason. **An empty list is a legitimate outcome** if
   the reason is argued.
5. **What was not read, and why.** Explicit. A study that quietly skipped the book's second half
   must say so.
6. **A "## Last Updated

**Date**: 2026-09-15

**Updated By**: Claude Code / claude-opus-5

**Changes**: Added **§3 Q8 — idealized definition versus approximated implementation** (human,
2026-09-15), and stated the study's practical aim in the Abstract: **it must lead toward
programmable and computable systems, while keeping some concepts open for implementation by
approximation.** The worked example is numbers — a countably-long `integers.foo` creating the
integers, an `aleph_1`-cardinality `reals.foo` creating the reals, idealized definitions saying
what the things ARE, with the FVM implementing them approximately as IEEE integers and floats.
Q8 asks whether tree calculus distinguishes an idealized definition from its implementation and
where it draws the line (its values are finite trees, so an infinite construction cannot be a
value); whether `⬤` is the right foundation for an idealized `integers.foo`; **where an
approximation must be VISIBLE to a Foolisher**; and what it costs to state the ideal and
implement the approximation separately rather than defining numbers as whatever the machine does.
The question is about **the seam**, not the arithmetic — any construction whose honest definition
is infinite will need the same treatment. Q8 is threaded through the deliverable (§4), the
Plan of Execution table, and the plan's Phase 2 and Phase 4 checkboxes, so it cannot be silently
dropped.

Prior entry: FOOP-66 authored — specification and plan for a structured study of Barry Jay's tree
calculus, grounded in the specification page fetched 2026-09-15, with **Q5 (equality)** as the
highest-value target cross-referenced to FOOP-36 §N6.
