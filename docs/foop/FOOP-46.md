---
foop: D64
title: BraneConcatOp — a rewritten concatenation operator with phased search resolution
author: Claude Code / claude-opus-5 (directed by Atlas)
status: Draft
type: Standards
created: 2026-09-02
phase: phase-4
supersedes: []
begun: [ ]
---

# FOOP-46: `BraneConcatOp` — a rewritten concatenation operator

FOOP numbering is little-endian; the full rules live in `foop.md` at the repository root —
**read it before creating or editing a FOOP.** The `foop:` front-matter field here is the
big-endian sort key preceded by `D` (`foop: D64`, file `FOOP-46.md`, following FOOP-36's 63).

## Abstract

`foolish-ubca2` shall replace its concatenation implementation with a **new** operator,
`BraneConcatOp`, written against a specified search behavior rather than patched into the
existing one.

The change that motivates a rewrite rather than a repair is **phased search resolution**:

> While a concatenation's `foolish_children` are still being stepped, an IB search demanded by
> one of them must **find nothing** in the concatenation — the demanding member then looks to
> the concatenation's parent. Once the constituents are ready and their statements have been
> revived into `ubc_children`, IB searches resolve **normally** within `ubc_children`.

The concatenation therefore has two distinct search personalities in sequence, and which one is
active is a function of which phase it is in. That is not a predicate to add to the existing
code; it is the shape the operator should have been built with.

This FOOP was split out of **FOOP-26**, which specifies the operator's *contract* (it is an
operator, not a brane; it answers no brane-like question about itself; it says "don't know"
rather than zero while unsettled; it waits for an `Econstanic` constituent rather than
proceeding past it) and its *ergonomics* (the compiler marks each written constituent by its
syntactic form). **FOOP-26 §4 is the specification this FOOP implements.** Nothing here
restates it.

**Dependencies:** FOOP-26. **Order:** after it.

## Motivation

### Why a rewrite and not a repair

FOOP-26 §4.6 lists what would have to change in the existing implementation: remove three
brane-like answers, triage nine `stmt_count().unwrap_or(0)` sites, add an `Econstanic`-chain
readiness walk, add a constantew classifier arm, rename the kind. Each is individually small.

Adding phased search resolution on top of that is different in kind. It means the operator
answers a search *differently depending on which of its two child stores is live*, which is a
property of the operator's whole structure rather than of any one method. Retrofitting it means
threading a phase question through every path that can reach a concatenation during a search —
and the FOOP-55 branch's experience is directly relevant here: three successive wrong
root-cause diagnoses (its D9, D10, and the §9.2 classifier bug) all traced to concatenation
behaving inconsistently between phases, and each fix made the next one harder to see.

Writing the operator fresh, against a specification that names both phases explicitly, is
cheaper than reaching the same place by correction.

### What FOOP-26 leaves in place

FOOP-26 lands the operator's contract and ergonomics against the *existing* implementation,
because those changes are worth having on their own and because they are what make the
specification precise enough to build against. This FOOP then replaces the implementation
underneath that contract. If FOOP-26's changes prove sufficient in practice and the phased
search behavior turns out not to be needed, this FOOP can be cancelled without loss.

## Specification

### §1. The two phases

A `BraneConcatOp` is in exactly one of two phases, distinguished by whether its `ubc_children`
have been populated:

| Phase | State | Its `foolish_children` | Its `ubc_children` |
|---|---|---|---|
| **Gathering** | constituents still being stepped to constanic | live, being stepped | empty |
| **Joined** | constituents constanic, statements revived into the helper | inert | live, being stepped |

The phases are sequential and there is no return: `populate_concat_helpers` is the transition,
and it runs once.

### §2. Search behavior, per phase

**Gathering — an IB search demanded by a constituent finds nothing here.**

A constituent being stepped may demand a name. The concatenation is not yet a brane in any
meaningful sense — its `foolish_children` are the *unmerged constituents*, not the statements
they will become, so there is no partial "join so far" to search even in principle. The search
must therefore find nothing **in the concatenation**, and the demanding member looks onward to
the concatenation's **parent**, by the ordinary IB-then-AB fallback.

This is not a special case bolted onto the search engine. It is what FOOP-26 §4.5's "says
'don't know', never zero" produces: an unknown statement count makes the index lookup answer
"not found", and `SearchFir`'s existing `Embryonic` arm already treats an IB miss as "try the
parent".

**Joined — IB searches resolve normally within `ubc_children`.**

Once the constituents are ready and their statements have been revived into the helper, those
revived statements *are* the concatenation's content. An IB search resolves against them
normally, exactly as it would inside a brane — including across the boundary between statements
that came from different constituents. FOOP-26 §4.3 shows this working
(`{t = {y=2;}; a = {x=1;} t; b = a?y; b;}` gives `b = 2`).

**Why the two windows do not overlap.** Two separate reasons, one per kind of search:

- A **brane constituent's internal searches** are built already-`Econstanic` by the compile-time
  SFF rule (FOOP-26 §2.2), so they do not run during Gathering at all. They run only after
  `populate_concat_helpers` has revived them into the helper and the revive has reset their
  NYES — by which time the concatenation is Joined.
- A **bare-name constituent's own search** does run during Gathering, and §3 traces what happens
  to it: it finds nothing here and forwards outward, meeting only Gathering concatenations on
  the way, until it leaves them. It never consults a Joined concatenation, because a
  concatenation cannot be Joined while the constituent that started the search is still
  pre-constanic.

### §3. Worked example — a bare name as a constituent, and why the fall-through chains

A search may appear as a constituent in its own right, not merely inside one:

```foolish
{ huh={z=9}; a = {a=1}{b=2}({c=3}huh{d=4}){e=5}; a;}
!! a = {a=1; b=2; c=3; z=9; d=4; e=5}     (measured, foolish-ubca2, 2026-09-02)
```

**The rule in §2 produces this answer, and the route matters more than the answer.** Trace it:

1. `huh` sits in the **inner** concatenation's `foolish_children`, being stepped toward
   constanic. It demands the name `huh` from its IB.
2. The inner concatenation is **Gathering** — its own constituents are still being stepped — so
   by §2 it answers **nothing found**, and the search forwards to its parent.
3. That parent is the **outer** concatenation. It is *necessarily* Gathering too: the inner
   concatenation is one of its `foolish_children` and is still pre-constanic, which is precisely
   what "Gathering" means. So the outer one also answers **nothing found**, and forwards again.
4. The search reaches the outermost brane, where `huh={z=9}` resolves normally.

**The phases chain, and they chain in the only order they can.** A concatenation cannot be
Joined while any constituent is pre-constanic, so a search travelling outward from a
pre-constanic constituent meets Gathering concatenations all the way up until it leaves them.
There is no arrangement in which it meets a Joined concatenation on the way out — being Joined
would mean the constituent it started from had already settled.

This answers Open Question 4 (a Gathering concatenation whose parent is also Gathering) in the
affirmative and by construction, rather than by testing: it is the normal case, not an edge one.

**The shadowing case — §2's prediction, confirmed.** The discriminating arrangement is a name
bound in **both** a sibling constituent and the parent. Under §2 the concatenation answers
nothing while Gathering, so the parent's binding wins and the sibling's is never consulted.
Measured on `foolish-ubca2`, 2026-09-02:

```foolish
{ huh = {z=9;};
  a = {a=1;}({huh={z=7;};} huh {d=4;}){e=5;}; a;}
!! a = {a=1; huh={z=7}; z=9; d=4; e=5}
```

The parent's `z=9` is what `huh` resolved to. The sibling's `huh={z=7;}` was **not** consulted
as a binding — it appears in the output only as its own contributed statement. Behavior and rule
agree.

### §3.1 An unresolved constituent stops the join — and a nested one does not, today

Removing the parent binding leaves the search with nothing to find anywhere.

**The rule.** `huh` settles **ECONSTANIC** — an unanchored miss, which may still gain a value
by recoordination and is therefore not NK. A constituent that is not constanic is not ready, so
**the concatenation does not concatenate**: it holds at `Woconstanic` and produces no join. It
does not flatten past the constituent, and it does not collapse. Nothing is lost, and if the
whole concatenation is later recoordinated somewhere `huh` resolves, the join can still happen.

**Flat, this is already what happens** (measured 2026-09-02):

```foolish
{ a = {a=1;} huh {d=4;}; a;}
!! a = ⨃(elements=3, {a=1}, <ECONSTANIC ?huh>, {d=4}, WOCONSTANIC)
```

The concatenation is `WOCONSTANIC`, the unresolved `huh` is visible inside it as `ECONSTANIC`,
and all three constituents are intact. This is the `!all_brane_like → Woconstanic` branch
(`fvm_storage.rs:1047`) doing its job.

**Nested, it is not** (measured 2026-09-02):

```foolish
{ a = {a=1;}({huh={z=7;};} huh {d=4;}){e=5;}; a;}
!! shall be:  a holds at WOCONSTANIC, with the inner concatenation unjoined
!! today:     a = {a=1; e=5}    -- inner concatenation gone, huh and {d=4;} lost, no alarm
```

The outer concatenation joined anyway, treating the unfinished inner one as contributing
nothing. The sibling `huh={z=7;}` and `{d=4;}` are silently dropped.

**This is the bug, and it is nesting-specific.** The flat case proves the readiness rule is
right and already implemented; the nested case proves it does not survive one level of nesting.
The likely cause is FOOP-26 §4.5's unknown-count defect — an inner concatenation that cannot yet
say how many statements it has reports **zero** rather than "don't know"
(`stmt_count().unwrap_or(0)`), so the outer one reads it as an empty but valid brane, finds
every constituent brane-like, and joins. Confirm that before fixing; FOOP-26 §4.6's triage of
the nine `unwrap_or(0)` sites may already resolve it, in which case this becomes a regression
test rather than a change.

**The design question, now narrowed.** §2's rule is confirmed against current behavior in every
arrangement tested. What remains is whether the rule is *wanted*: a sibling constituent's binding
is invisible to its siblings during Gathering, and only becomes visible once everything is Joined
— by which time the searches that would have used it have already resolved elsewhere. The
alternative, letting a Gathering concatenation answer from its constituents, reintroduces exactly
the phase-inconsistency §Motivation says is expensive to debug. Decide, and record the reasoning
here.

Note why a *bare-name* constituent is the interesting one at all: it is SF-wrapped by the
ergonomics (FOOP-26 §4.3), and an SF defers stepping but does **not** make the search
`Econstanic`. Unlike a brane constituent's internal searches — held off entirely by the
compile-time SFF rule — this search really does run during Gathering. It is the case that
exercises §2 rather than bypassing it.

### §4. Reusing `Brane`'s code — the operator's `ubc_children` should be a brane

**The design.** Once Joined, a `BraneConcatOp` holds one brane in `ubc_children`, and everything
below the operator — stepping, IB search, `stmt_count`/`stmt_at`, rendering — is performed by
**`Brane`'s existing code**, unmodified. The operator's job ends at producing that brane. It
does not imitate a brane, delegate to one method by method, or carry a parallel implementation.

`.value()` is the whole interface. Today `FirPointer::settled_result` (`fvm_storage.rs:639`)
already returns `ubc_children[0]` once the node is constanic, and `value()` (`:649`) unwraps
through it recursively. So **a settled concatenation's `.value()` is already the brane in
`ubc_children`** — no new mechanism is needed for the Joined phase, and FOOP-26 §4.5's "a
concatenation answers no brane-like question about itself; ask its settled value" is satisfied
by the accessor that exists.

**How close the current code already is.** `ConcatHelper`'s `fir_op_step` arm
(`fvm_storage.rs:978-996`) and `Brane`'s (`:879-898`) have **identical bodies** — verified line
by line 2026-09-02, the only difference being the `FirSpec` pattern each matches. Both push
`foolish_children` as tasks on `Prembrionic`/`Embryonic` and both settle via
`decide_nyes_due_to_children` on `Braning`. The code's own comment says as much: *"Same shape as
`Brane`'s arm — a `ConcatHelper` is transparent, inheriting brane-shaped stepping."*
`FirCursor::stmt_count` and `stmt_at` likewise treat `Brane` and `ConcatHelper` as one case
(`:1653`, `:1675`).

`ConcatHelper` is therefore not a different thing from a brane; it is a duplicate of one, kept
distinct only by its `FirSpec` tag. That duplication is what this section removes.

**What to decide during implementation.** Two shapes reach the same place:

1. **Populate `ubc_children` with a `FirSpec::Brane`** and delete `FirSpec::ConcatHelper`
   entirely. Maximum reuse; every arm that matches `ConcatHelper` alongside `Brane` collapses
   to one case.
2. **Keep `ConcatHelper` as a distinct tag** whose every arm delegates to `Brane`'s. Preserves
   the ability to tell, from the FIR alone, that a brane was produced by a join rather than
   written — which matters to rendering and to debugging.

Shape 1 is the stronger form of the argument and should be tried first. The reason to hesitate
is that a brane produced by a join is **not** a `{…}` written by a Foolisher, and FOOP-26 §4.1
is emphatic that the three must not be conflated. If shape 1 makes a joined brane
indistinguishable from a written one in the rendered output, that is a real loss, and shape 2
buys it back for the cost of a tag.

**Things to check before committing to either.** A joined brane is a node Foolish cannot name,
so it is worth confirming each of these behaves:

- `get_my_brane` walks — a search inside a joined statement must find the joined brane as its
  home brane, not skip past it.
- Line numbering — `populate_concat_helpers` renumbers copied statements to their global index
  (`revive_constanic(stmt, helper, global_idx, …)`, `:2749`); that must remain true.
- Rendering — FOOP-36 (the Foolish-rendering sequencer) renders a brane as `{…}`. A joined brane
  rendering as an ordinary brane is arguably correct, since that is what it *is* once settled;
  confirm with FOOP-36 rather than assuming.
- The `provenance: ConcatProvenance` field, which records whether the operator was written as a
  juxtaposition or a tail-concatenation, stays on the **operator**, not on the produced brane.

## FIR Impact

- `FirSpec::BraneConcatOp` is rewritten. Whether it keeps `ConcatHelper` as a separate kind, or
  owns a hidden `Brane`, is §3's open question.
- No new NYES state.

## UBC Step Impact

Step counts change for concatenation cases. Every moved baseline goes through the Promotion
Review Gate.

## Test Plan

1. **FOOP-26's concatenation cases are the regression floor.** They must continue to pass
   unchanged; this FOOP replaces the implementation, not the specified behavior.
2. **One case per phase boundary** — a search demanded during Gathering that must fall through
   to the parent, and the same name resolving within `ubc_children` once Joined.
3. **§3's fall-through chain** — the bare-name constituent inside a nested concatenation,
   asserting the outermost binding is what resolves, and that it does so through two Gathering
   concatenations.
4. **§3's shadowing case** — the same name bound in a sibling constituent and in the parent
   (parent wins). Measured 2026-09-02; pin it.
5. **§3.1's pair** — the flat unresolved constituent (holds at `Woconstanic`, all constituents
   intact) and the nested one (must hold the same way; today it silently drops). The nested case
   is a **failing test to write first**.
6. **The nested-concatenation cases from FOOP-26 §4.3**, since an inner operator's phases are
   nested inside the outer one's.

## Plan of Execution for Plan

To be written when this FOOP moves out of Draft. Judgment phases — resolving §3 and §4, and every
`output` → `checked` promotion review — go to a larger model; the mechanical rewrite of the
operator against a settled specification does not.

## Rejected Alternatives

**A. Patch the existing implementation.** Rejected — see §Motivation. The phase distinction is
structural, and the FOOP-55 branch demonstrated what retrofitting it costs.

**B. Fold this into FOOP-26.** Rejected: FOOP-26 is already three changes across four crates,
and its concatenation work (contract + ergonomics) is separable from and prerequisite to this.
Keeping them apart lets FOOP-26 land and be measured before a rewrite begins.

## Open Questions

1. **§3 — does a bare-name constituent resolve against its siblings, or fall through to the
   parent?** The load-bearing question. §2's rule says fall through; current behavior is
   untested against the discriminating arrangements. Write §4's cases 2 and 3 before building
   anything.
2. **§4 — populate `ubc_children` with a real `Brane` and delete `ConcatHelper`, or keep the
   tag and delegate?** The reuse argument favours deleting it; telling a joined brane from a
   written one favours keeping it. Try deletion first.
3. **Does a contexted search need the same phase treatment?** A contexted search navigates from
   a carried position rather than doing a fresh IB/AB walk. FOOP-26 §5 flags this as untested;
   it belongs here.
4. ~~What does a search demanded during Gathering see if the concatenation's parent is itself a
   concatenation in its Gathering phase?~~ **ANSWERED by §3**: the fall-through chains, and it is
   the normal case rather than an edge one — a concatenation cannot be Joined while a constituent
   is still pre-constanic, so an outward-travelling search meets only Gathering concatenations
   until it leaves them.

## References

- **`docs/foop/FOOP-26.md` §4** — the operator's contract and ergonomics. This FOOP implements
  that specification; it does not restate it.
- FOOP-23 (searches, IB/AB fallback, the two-child invariant), FOOP-65 (tail concatenator, the
  Equivalence Law), FOOP-13 (ConcatBrane, the Equivalence Law's origin).
- `docs/foop/FOOP-55.md` on the `worktree-foop-55-event-handlers` branch — its D9, D10 and §9.2
  findings are the evidence that phase-inconsistent concatenation is expensive to debug.

## Last Updated

**Date**: 2026-09-02
**Updated By**: Claude Code / claude-opus-5
**Changes**: Created. Split out of FOOP-26 (human, 2026-09-02) so `BraneConcatOp` can be
**rewritten** against a specified phased search behavior rather than patched: during Gathering an
IB search demanded by a constituent finds nothing in the concatenation and falls through to its
parent; once Joined, IB searches resolve normally within `ubc_children`. FOOP-26 §4 keeps the
operator's contract and ergonomics and is the specification this FOOP implements.

§3 traces the fall-through for a bare-name constituent and establishes that **the phases chain,
and can only chain one way**: a concatenation cannot be Joined while any constituent is
pre-constanic, so a search travelling outward from a pre-constanic constituent meets Gathering
concatenations all the way up until it leaves them — never a Joined one.

§3.1 states the rule for a constituent that resolves to nothing: it settles **ECONSTANIC** (an
unanchored miss, recoverable by recoordination, not NK) and **the concatenation does not
concatenate** — it holds at `Woconstanic`, keeping every constituent intact. Measured
2026-09-02: the **flat** case already does this; the **nested** case silently drops the
unfinished inner concatenation and joins around it. The bug is nesting-specific, and the likely
cause is FOOP-26 §4.5's unknown-count defect.

§4 states the reuse design (human, 2026-09-02): once Joined, the operator holds a **brane** in
`ubc_children`, and all stepping, searching and rendering below it is performed by `Brane`'s
existing code. `.value()` is the whole interface, and it already works — `settled_result`
(`fvm_storage.rs:639`) returns `ubc_children[0]` once constanic, so a settled concatenation's
`.value()` is already the brane. Verified line by line that `ConcatHelper`'s `fir_op_step` arm
and `Brane`'s have identical bodies, differing only in the pattern matched, so `ConcatHelper`
is a duplicate of `Brane` rather than a different thing. Whether to delete the tag outright or
keep it delegating is left open, weighed against FOOP-26 §4.1's insistence that a written brane
and a produced one not be conflated.

Also measured: the discriminating shadowing case, where a parent binding beats a sibling
constituent's and the sibling is never consulted as a binding. Behavior and rule agree; the open
question is whether the rule is wanted.
