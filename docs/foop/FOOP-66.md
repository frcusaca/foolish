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
2. **Q1 through Q7, one section each**, each with a finding, a "no correspondence", or a "could
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
6. **A "## Last Updated" section**, per AGENTS.md §Markdown File Update Protocol.

**The assessment is written for the human and for the next agent, not for a grade.** Negative
findings are first-class: "tree calculus does not help here, because X" is a result that saves
the project time, and the FOOP is complete if that is what it honestly concludes.

### 5. Hard scope boundaries

These are stated as rules, not preferences, because the temptation to drift is real.

- **No crate is modified.** Not `foolish-core`, not `foolish-parser`, not `foolish-ubca`, not
  `foolish-ubca2`, not `foolish-cli`, not `einmo`, not `zweimomo`. If a task appears to require
  it, **STOP and report** — the study has drifted into implementation and the drift needs a
  human decision, not an agent's.
- **No einmo baseline is created, changed or promoted.** This FOOP has **no** `input/foop/66/`
  directory and **no** comprehensive snapshot test. §Test Plan explains why, and that omission is
  deliberate and reasoned, not an oversight.
- **No language change is decided here.** The study may *recommend*; it may not *adopt*. Any
  adoption is a separate FOOP with its own specification and its own human review.
- **No code is copied from the Coq development.** It is MIT-licensed, so copying would be legally
  permissible — that is not the reason. The reason is that Foolish gains nothing from transplanted
  Coq, and a copied proof would misrepresent the project's own understanding. **Ideas may be
  imported; text and code may not.** Anything learned is restated in Foolish's own terms, with the
  source cited.
- **Do not fabricate technical claims about tree calculus.** If a source is unreachable or a
  passage is not understood, write "could not determine" and say what was tried. A confidently
  wrong sentence about Jay's work is worse for this project than a blank, because the next reader
  will believe it. This rule outranks completeness.

## FIR Impact

**None.**

This FOOP adds no FIR variant, changes no NYES state or transition, touches no `FirSpec` field,
and has no serialization implication. It is a study; the FIR tree is its *subject* in Q2 and Q5,
never its *object*.

Stated explicitly rather than omitted so that a reader diffing this FOOP against the template can
see the question was asked and answered, not skipped.

## UBC Step Impact

**None.**

No step rule is added, removed or changed. No evaluator in any crate is modified. Nothing
interacts with constanic coordination, because nothing executes.

Stated explicitly for the same reason as §FIR Impact.

## Test Plan

**A study FOOP has no tests in the usual sense, and this one installs none.** There is no
behavior to pin, because no behavior changes. Manufacturing einmo cases or unit tests here would
create the *appearance* of verification while verifying nothing — which is worse than admitting
there is nothing to verify, since a green test on an unchanged evaluator is evidence of nothing
at all.

Specifically, and deliberately:

- **No `foolish-ubca/einmo_suite/input/foop/66/` directory**, and **no
  `foop/66/comprehensive.foo`.** `foop.md` §"Comprehensive FOOP Tests" grants every FOOP the right
  *and the obligation* to write one — but that obligation presupposes a *feature* to exercise
  against existing features. This FOOP ships no feature. Writing a comprehensive test here would
  mean writing a test of the status quo and signing it as this FOOP's contribution, which is a
  false record. **The obligation is discharged by this paragraph, which states the reason.** If
  the human disagrees, the fix is a one-line instruction and the plan gains a phase.
- **No `output → checked` promotion**, and therefore **no Promotion Review Gate** in the plan.
  Nothing is generated, so there is nothing to review or promote. The plan must contain no
  `einmo promote` invocation of any kind.
- **No "Establish relevant tests" checkbox on the reading phases.** `foop.md` §"Sub-Section Test
  Subsets" requires one at the start of every sub-section, and the requirement is sound for
  implementation work — but a phase whose entire content is *reading a web page and writing
  prose* has no relevant test subset, and naming an arbitrary one would be exactly the empty
  ritual the rule exists to prevent. The plan states this in-line where the checkbox would
  otherwise sit, so the omission reads as a decision rather than a lapse. **The one phase where
  a real test subset exists — the phase that writes `.foo` snippets — carries a real one.**

**What stands in place of tests:**

1. **The written assessment itself** (§4), reviewed by the human. The review question is not "did
   it pass?" but "is this an accurate account of tree calculus, and are the correspondences
   argued rather than asserted?"
2. **Source fidelity.** Every technical claim about tree calculus in the assessment cites the
   source it came from — spec page, repo file, or book chapter and page. A claim without a
   citation is a defect and is repaired before the human review, not after. This is the study's
   analogue of a passing test, and it is the one an executing agent can check itself.
3. **Worked examples as executable checks.** Any Foolish `.foo` snippet written to illustrate a
   correspondence **must actually parse and run** —
   `cargo run -p foolish-cli -- run <path>` — and its real behavior is what the assessment
   reports. A snippet that does not run is removed or fixed; it is never reported as if it ran.
   This is the only phase with a genuine test subset, and it gets one.
4. **The repository stays green.** The study changes no code, so `cargo test --workspace` must
   pass identically before and after — **791 tests passing on `jia` at `93afcee3`, the baseline
   this FOOP starts from.** The merge gate keeps a checkbox for it. If that number moves, the
   scope boundary of §5 was crossed and the crossing must be explained.

## Plan of Execution for Plan

**How this FOOP's plan gets executed, and by whom.** Model selection is **per-phase, not
per-FOOP**. This FOOP splits unusually cleanly, because reading-and-summarizing and
judging-a-correspondence are genuinely different jobs.

| Phase | Character | Needs |
|-------|-----------|-------|
| 0 — Begin, worktree | Mechanical: commit, check `begun`, `git worktree add`. | **Smaller model.** Fixed commands, given verbatim in the plan. |
| 1 — Verify §1 against the spec page (D1) | Comparison against a fixed target: does §1 match the source, line by line? | **Smaller model.** The target is written down; the job is to diff prose against a web page and report mismatches. It does **not** interpret. |
| 2 — Provisional answers to Q1–Q7 from D1 | First interpretation. Requires holding Foolish's semantics and tree calculus's side by side. | **Larger model.** These are the judgments the FOOP exists to make. |
| 3 — Read the repo (D2); transcribe the equality program and its theorem | Mostly extraction: locate definitions in Coq source, transcribe verbatim. | **Smaller model**, with a named stop condition: if the equality program cannot be located, STOP and report rather than guessing which definition it is. |
| 4 — Read the book's equality and reflection chapters (D3) | Deep reading of formal mathematics, against Foolish's own unfinished design. | **Larger model.** The hardest phase in the FOOP. |
| 5 — Write the assessment, answer Q1–Q7, rank the findings | The deliverable. Every finding is a claim about Foolish's design. | **Larger model.** Not delegable — see below. |
| 6 — Write and run the `.foo` worked examples | Fixed target: the snippet must parse and run; its output is reported as-is. | **Smaller model.** The only phase with a real test subset, and it is a mechanical one. |
| 7 — Human review, merge, cleanup | Judgment plus mechanics; gated by a human STOP. | **Larger model** for the report; the human holds the gate. |

**What makes the small-model phases safe here**, built in deliberately:

1. **Facts inline, not referenced.** §1 carries the five reduction rules, the three node names,
   the confluence claim and the encoding example verbatim, and the plan carries the URLs, the
   file list of the Coq repo, and the exact `cargo` commands. A Phase 1 or Phase 3 agent needs no
   rediscovery. **Every such fact is marked *verify, don't re-derive*.**
2. **A fixed target per phase.** Phase 1 diffs prose against a page. Phase 3 transcribes a
   definition that either exists or does not. Phase 6's snippet either parses or does not. None
   of these requires the agent to judge whether an answer is *good*.
3. **Named stop conditions.** Each small-model phase states what wrong looks like: Phase 1 — "the
   spec page is unreachable, or §1 contradicts it in a way that changes a Q3/Q5 premise"; Phase 3
   — "no equality program is identifiable in the repo"; Phase 6 — "the snippet does not parse."
   In every case the instruction is **STOP and report**, never improvise.

**What must not be delegated, at any model size** (AGENTS.md §"The agent is responsible for
correctness"):

- **Phase 5's findings and their ranking.** Each is a claim about Foolish's design and a
  recommendation about its future; this is precisely the judgment the human relies on.
- **Any statement that a synergy EXISTS.** A negative or "could not determine" finding may be
  recorded by any agent; an *affirmative* claim that tree calculus offers Foolish something must
  be argued by the judgment-phase agent, with the source cited, because it is the claim most
  likely to be believed downstream and acted on.
- **Any decision to touch a crate**, which §5 forbids outright — an agent that thinks it needs to
  stops and asks.
- **The Open Question in §Open Questions about follow-on scope.** That is the human's call, not an
  agent's, and it changes what this FOOP is.

**On sub-agents.** Phases 1 and 3 are good candidates for parallel sub-agents (two sources, no
dependency between them), and the human's own standing preference is to delegate small mechanical
work. Phases 2, 4 and 5 are not: they need the whole picture in one context, and splitting them
produces an assessment that contradicts itself between sections.

## Rejected Alternatives

### A. Do nothing — never read tree calculus

The status quo. Rejected because the cost is asymmetric. Reading is a handful of days; *not*
reading risks the project re-deriving, badly, something already proved — and FOOP-36 §N6's
equality design is queued right now and is exactly the kind of work where that would happen. The
BDFL has also explicitly asked for the study, which settles it. **Doing nothing is worse than
doing this even if the study concludes there is no synergy**, because a documented negative
finding stops the question being reopened every six months.

### B. Skip the study; go straight to an implementation FOOP importing tree-calculus ideas

Tempting because it looks like faster progress. Rejected outright: **nobody on the project has
read tree calculus**, so an implementation FOOP would be specifying against an imagined source.
The likeliest outcome is a Foolish feature justified by a misremembered claim — the exact failure
AGENTS.md §"The agent is responsible for correctness" is written against, and one that is very
expensive to unwind once a baseline is signed.

### C. Fold the study into FOOP-36 §N6's equality FOOP as a research phase

Plausible, and the two are genuinely related — §N6 is this study's highest-value consumer.
Rejected on three grounds. **Scope:** tree calculus bears on Q1–Q4, Q6 and Q7 as well, none of
which N6 is about; the study would be truncated to serve one consumer. **Sequencing:** N6 is
blocked on FOOP-36 §Open Questions **Q9** (cross-brane creation identity), a human decision that
has not been made; chaining an unblocked study behind a blocked FOOP delays it for no reason.
**Deliverable shape:** N6 produces a relation with an implementation; this produces a document in
`docs/why/`. Different artifacts, different review criteria, different readers. They should
**reference** each other — and this FOOP is written to feed N6 — not be merged.

### D. Read the book cover to cover before writing anything

Rejected as poor sequencing, not as wrong. The book is long and only some of it bears on Foolish.
The §2 outside-in plan reaches the equality question — the highest-value one — within D2, and
lets D3 be narrowed to the chapters that matter. **A study that stalls in chapter 4 of a book
nobody asked for delivers nothing**, whereas an early-terminated study that states what it did
not read delivers most of the value at a fraction of the cost. If the narrow read raises
questions that need the whole book, the plan can gain a phase; the reverse is not recoverable.

### E. Write the study as a blog-style essay rather than a FOOP

Rejected because it would not be tracked, not be reviewed, not be numbered, and not be findable.
The FOOP process exists so that design-relevant reasoning is discoverable by the next agent and
the next human. An essay outside it is invisible within a month. The `docs/why/` document *is*
essay-shaped — the FOOP is the process wrapper that gets it written, reviewed and indexed.

## Open Questions

- **Should this remain purely a study, or may it produce a follow-on implementation FOOP if a
  strong synergy is found?** §5 currently forbids adoption and confines the FOOP to
  recommendation. That is the conservative reading of the request ("learn … and perhaps discover
  synergies"), but it is **the human's call, not an agent's**, and it changes what this FOOP is.
  Three options: **(i)** study only, findings recorded, any follow-on proposed separately and
  numbered separately (the current text); **(ii)** study, plus authority to *write* a follow-on
  FOOP spec — not to implement it — if a finding warrants; **(iii)** study, plus authority to
  implement a small, bounded change if one is obviously right. **The FOOP is written for (i)** and
  the plan's final phase asks the human to confirm or change it. *Answering this before execution
  begins would be better than answering it at the end.*
- **Is `docs/why/TREE_CALCULUS.md` the right home and the right filename?** §4 argues for it from
  AGENTS.md's own description of `docs/why/`. An alternative is `docs/why/` for the philosophical
  comparison plus a separate `docs/how/` note for anything with engineering consequence, but
  splitting a short document across two directories seems worse than one document with a
  findings section. **Provisionally settled as one file; reopen only if the assessment grows past
  roughly 600 lines.**
- **How close is the "same time" claim, and does it matter?** The motivation rests on Jay and the
  BDFL having started on tree-based systems at about the same time. Jay's book is dated 2021 and
  the underlying calculus is older. **The exact chronology is not established, and this FOOP does
  not assert it as fact** — Phase 2 should record what the sources actually say about when tree
  calculus was developed, and if the parallel turns out to be loose, the motivation is corrected
  rather than defended. Nothing in the study's value depends on the dates lining up; the
  comparison stands on the ideas.
- **Does the study need a Coq toolchain?** D2 reads Coq *statements*, which is a reading task and
  needs no toolchain. **Building the proofs is explicitly out of scope** — but if a Phase 3 or
  Phase 4 question turns out to be answerable only by running Coq, that is a new dependency and a
  new tool on the machine, so it is a STOP-and-ask, not an agent's decision.
- **Should the assessment be shown to Barry Jay?** Out of scope for this FOOP and recorded only so
  it is not forgotten. If the study produces something substantive about the two systems'
  relationship, contacting the author is a reasonable next step and entirely the human's call —
  the repository lists a contact address. **No agent initiates outside contact.**

## References

**Tree calculus — primary sources** (the first was read while writing this FOOP; the second and
third are to be read *during* execution, per §2):

- **Specification** — https://treecalcul.us/specification/ — the grammar, the three node kinds,
  the five reduction rules, values, confluence, the boolean encoding. **Fetched and reachable
  2026-09-15**; §1 above is drawn from it and from the site's front page (https://treecalcul.us/).
- **Coq development** — https://github.com/barry-jay-personal/tree-calculus — machine-checked
  proofs for the book. MIT-licensed. Files noted in §1. **Not read at authoring time.**
- **Book (PDF)** — https://github.com/barry-jay-personal/tree-calculus/blob/master/tree_book.pdf —
  *Reflective Programs in Tree Calculus*, Barry Jay with Jose Vergara, 2021. **Not read at
  authoring time.**

**Foolish — the design this study feeds:**

- **FOOP-36 §N6** (`docs/foop/FOOP-36.md`) — "FIR equality — its own FOOP." **The single most
  relevant prior FOOP**, and the reason §3's Q5 is the study's highest-value target. Read
  §N6.1 (shape half), §N6.2 (value half — creations by dynamic programming over an equality
  table), §N6.3 (the table is per-brane), **§N6.4 (equality with conditions — return the mapping,
  not a boolean)**, and §N6.5 (the `EQUIVALENCE.md` obligation). Its §Open Questions **Q9**
  (cross-brane creation identity) blocks N6 and is the human's decision.
- **FOOP-62** — UBCa two-store ProtoBrane tree and uniform two-phase stepping; §Terminology is
  the authoritative definition of *constanic* and *constantew*. Background for Q4.
- **FOOP-56** — NYES groups and the four predicates (`is_preconstanic`, `is_constanic`,
  `is_constantew`, `is_conclusive`). Background for Q4.
- **FOOP-23** — value search and contexted `&`-searches; the one-engine
  cursor-source × predicate model and the `FoolRefFir` two-child invariant. Background for Q3.
- **FOOP-33** — the Creation Postulate: `⬤`, named creations, and `'True`/`'False` in
  `system.foo`. Background for Q1, Q5b, and the boolean worked example of §4.3.
- **FOOP-34** — Recursion Upgrades. Cited as **process precedent**: an existing standalone
  research FOOP, deliberately under-specified, whose content is discovered during execution.
  FOOP-66 follows its shape.

**Foolish — documentation:**

- `docs/vintage_legacy/EQUIVALENCE.md` — the vintage taxonomy of Foolish equality operators
  (`=s=`, `==`, `===`, `=n=`, `=c=`, `=v=`, …), unspecified and unimplemented. FOOP-36 §N6.5
  requires N6's FOOP to refresh it and bring it out of `vintage_legacy/`. **This FOOP does not
  move or rewrite it** — but Q5e asks what tree calculus says about `===` ("equal under ALL
  possible coordinations"), which is the member of that family with no obvious decision procedure.
- `docs/vintage_legacy/ECOSYSTEM.md` — UBC architecture; detachment and recoordination semantics.
  Background for Q2's AB/IB asymmetry.
- `docs/vintage_legacy/NAMES_SEARCHES_N_BOUNDS.md` — name resolution and search. Background for
  Q3 and Q7.
- `README.md` §"The Unknown" — NK versus ECONSTANIC. Background for Q3's miss-outcome comparison.
- `AGENTS.md` §"Foolish Terminology" and §"Searches (FOOP-23)" — the vocabulary the assessment
  must use consistently.

**Process:**

- `foop.md` — the authoritative FOOP process reference.
- `docs/foop/FOOP-1.md` — the meta-FOOP defining the process.
- `docs/foop/INDEX.md` — where FOOP-66 is listed (phase-5, with FOOP-34).

## Last Updated

**Date**: 2026-09-15
**Updated By**: Claude Code / claude-opus-5
**Changes**: Created FOOP-66 — a **learning/research FOOP** committing the Foolish team to study
Barry Jay's **tree calculus** and produce one written assessment (`docs/why/TREE_CALCULUS.md`) of
where it and Foolish correspond, where the analogy is false, and whether anything is worth
borrowing. Explicitly **not** an implementation FOOP: §FIR Impact and §UBC Step Impact are both
"None", §5 forbids touching any crate or einmo baseline, and §Test Plan states plainly that no
tests are installed and **why** — including the reasoned omission of the otherwise-obligatory
`foop/66/comprehensive.foo`. §1 is grounded in the tree calculus specification page, fetched and
reachable 2026-09-15 (grammar `E ::= △ | E E`, leaf/stem/fork, the five reduction rules, values
as irreducible expressions, confluence, the `false`/`true`/`not` encoding). §2 sets an outside-in
three-depth reading plan (spec page → Coq repo → the book's relevant chapters only) that may
terminate early. §3 poses Q1–Q7, with **Q5 (equality) as the highest-value target**, cross-
referenced in detail to **FOOP-36 §N6** — including §N6.4's equality-with-conditions design and
§N6.5's `EQUIVALENCE.md` obligation — and with Q5b and Q5d written so that a **negative** finding
(creation identity and the absence of a normal form may put Foolish's problem outside what Jay's
equality addresses) counts as a first-class result. §Plan of Execution for Plan assigns phases by
complexity: extraction and verification to a smaller model with named stop conditions, the
interpretive and assessment phases to a larger one. §Open Questions leads with the human's call
on whether the FOOP may produce a follow-on implementation FOOP; the spec is written for the
conservative "study only" reading.
