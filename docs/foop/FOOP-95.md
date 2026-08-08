---
foop: D95
title: Add Embryonic and Resequencing EINMO Sections
author: Sisyphus / claude-opus-5 (directed by Atlas)
status: Draft
type: Standards
created: 2026-08-08
phase: phase-2
supersedes: []
begun: [ ]
---

# FOOP-95: Add Embryonic and Resequencing EINMO Sections

FOOP numbering is little-endian; the full rules live in `foop.md` at the
repository root — **read it before creating or editing a FOOP.** The one
template-specific note: the `foop:` front-matter field may either match the
filename digits directly, or give the big-endian decimal value preceded by
`D` (this file: `foop: D95` — digits `95` reversed = sort key 59). In all
cases, the `FOOP-95.md` file name is ultimately the right numbering.

## Abstract

This FOOP adds **two new sections to every einmo test**. They are:

- **EMBRYONIC** (§1) — the program sequenced **as it was parsed, before any
  step has been taken**. It shows what the parser and `build_fir` actually
  produced (precedence groupings, operand splits, statement structure)
  separately from what evaluation did with it. Its job is to be *readable*:
  it displays the layout and nesting of FIRs.
- **RESEQUENCED** (§6) — the same un-stepped FIR tree emitted as **parsable
  Foolish**, by a new, separate sequencer: the **Foolish Resequencer**. Its
  job is to be *re-parsable*, which makes the whole corpus a round-trip
  property test of the parser and FIR-gen phases.

Both render the same un-stepped tree; they differ in what they optimize
for, which is why they are two sections and two components rather than one
of each.

RESEQUENCED brings **normalization** with it — a canonical textual form
this FOOP also introduces — and is checked by two equalities: the
resequenced Foolish must match the original input after normalization
(*fidelity*), and resequencing its own output must reproduce it exactly
(*idempotence*). It can reproduce a creation's syntax but never its
identity (`⬤`), which is inherent, not a defect.

Delivering EMBRYONIC requires repairing a pre-existing defect:
`ConcatenationFir::stmt_count` **mutates** — asking an un-stepped
concatenation how many statements it has silently forces the concatenation
join. A query that mutates cannot be used to photograph un-stepped state,
so `stmt_count` is **split into two methods**: a pure `stmt_count` and an
explicitly-named `ensure_joined_stmt_count`, with every existing call site
classified to one or the other.

The bulk of the work, and the reason this is its own FOOP rather than a
section of [FOOP-65](FOOP-65.md), is that **rendering an un-stepped tree
faithfully is not currently something the sequencer does well**. A
significant, explicitly-planned step is therefore **inspection of embryonic
Foolish for reasonably informative rendering — by agent AND by human**
(§4): looking at real pre-step output across the corpus and deciding what
"informative" means, before freezing it into baselines.

**§6 lands last**, after §1-§5 are green.

## Motivation

### What you cannot see today

Every einmo baseline is produced by stepping a program to settlement and
rendering the result. That means the corpus can only ever answer *"what did
this evaluate to?"* — never *"what did the compiler build?"*

Consequences:

- **Parser and precedence regressions are invisible until they change a
  value.** A mis-grouped operand that happens to settle to the same brane
  is undetectable; one that settles differently reports the failure at the
  wrong altitude, as a value diff rather than a structure diff.
- **Structure-only features are untestable through einmo.** The immediate
  case is [FOOP-65](FOOP-65.md)'s tail concatenator: its §5.3.1 renders a
  backtick chain in reversed form **only while all constituents are still
  embryonic**, so no settled baseline can ever display it. FOOP-65 depends
  on this FOOP for that visibility.
- **Development debugging has no cheap vantage.** Diagnosing "why did this
  brane evaluate that way?" usually starts with "what was it before
  stepping?", which today requires writing a throwaway Rust unit test (see
  the `foolish-debugging` skill). A standing pre-step rendering makes the
  first question free.
- **The parser and FIR-gen phases have no end-to-end test of their own.**
  Every test exercises them only incidentally, through whatever the
  program evaluates to. Nothing checks the stronger property: that the FIR
  tree retained *everything the source meant* — which is exactly what
  regenerating the Foolish from the tree and comparing it to the input
  would prove. That is RESEQUENCED (§6), and it is why the corpus becoming
  a round-trip property test is worth the work.

### Why both sections, and why they are one FOOP

EMBRYONIC answers *"what did the compiler build?"* and RESEQUENCED answers
the stronger *"did the FIR tree keep everything the source meant?"* They
share a source — the same un-stepped tree — and a prerequisite: a
**read-only** traversal of a tree that has not been stepped, which the code
cannot currently perform (§3). Building that once serves both.

They are nonetheless **two components and two sections**, because their
obligations conflict. A readable rendering is free to elide, summarize, and
annotate; a re-parsable one must emit exactly what the parser will read
back. Merging them would compromise both (Rejected Alternative E).

This work is scoped as its own FOOP, separate from [FOOP-65](FOOP-65.md),
because straightening up the sequencer's pre-step paths is substantial in
its own right — §4 exists precisely because the answer is not knowable in
advance.

## Specification

### §1. The EMBRYONIC section — the program as parsed

The first of this FOOP's two new sections. **EMBRYONIC** renders the
program sequenced **as parsed**, with **zero steps taken** — the
un-stepped view of the FIR tree. (The second, **RESEQUENCED**, is §6; both
render the same un-stepped tree, and §6.1 tabulates how they differ.)

- The existing OUTPUT is unchanged in content and position — it remains the
  settled result, and every existing baseline's settled section must stay
  **byte-identical** (§5).
- EMBRYONIC is **purely additive**.
- It is added to **EVERY test**, not only to tests of features that need
  it. Its value is general: any test can now show what was built, not only
  what it evaluated to.

Its job is to be **readable**: it displays the layout and nesting of FIRs,
and is under no obligation to be valid Foolish. That obligation belongs to
RESEQUENCED (§6).

The section's exact delimiter must survive einmo's Foolish separator
convention (`TestConfig::foolish_separator`, `ubca_snapshot_tester.rs:55`);
its name and ordering are fixed by §1.1.

#### §1.1 Section order — `METADATA, OUTPUT, EMBRYONIC, RESEQUENCED, INPUT, COMMENTS, STAMPS`

The envelope's sections are reordered so the reader meets them in the order
they are wanted. `INPUT` keeps its wire name — it moves, it is not renamed
— so the `compare.rs` always-compared hazard below does not arise for it.

Today an einmo envelope declares (verified on
`checked/foop/13/concat_brane_test_basic.foo.einmo`) a header block —
`#einmo 1 …`, `test:`, `suite:`, `producer:`, `producer-diff:`,
`generated:`, `status:`, `status-detail:` — followed by

```
sections: INPUT, OUTPUT, COMMENTS, STAMPS
```

with the section bodies in that declared order. This FOOP changes the
order to:

| # | Section | What it is | Change |
|---|---------|------------|--------|
| 1 | **METADATA** | the existing header block (`test:`, `suite:`, `producer:`, `generated:`, `status:`, …) | unchanged |
| 2 | **OUTPUT** | the settled result | content unchanged; **moves ahead of INPUT** |
| 3 | **EMBRYONIC** | §1: the program sequenced before any step, showing FIR layout/nesting | **new** |
| 4 | **RESEQUENCED** | the FIR tree re-emitted as *parsable Foolish* (§6) | **new**, added last (§6) |
| 5 | **INPUT** | the original Foolish source | content unchanged; **moves after EMBRYONIC** |
| 6 | **COMMENTS** | as today | unchanged |
| 7 | **STAMPS** | the signatures | unchanged; **must remain last** — it signs what precedes it |

So the declaration line becomes

```
sections: OUTPUT, EMBRYONIC, RESEQUENCED, INPUT, COMMENTS, STAMPS
```

(from today's `sections: INPUT, OUTPUT, COMMENTS, STAMPS`). It is
machine-read, so a mismatch between it and the emitted bodies is a hard
error, not cosmetic.

**RESEQUENCED is specified in §6 and lands LAST.** Implementers may land §1-§5 with the
order `OUTPUT, EMBRYONIC, INPUT, COMMENTS, STAMPS` and insert RESEQUENCED
afterwards — but doing so rewrites every baseline twice. **Preferred:**
decide the final order once, and if §6 is going to land at all, reserve
its slot from the start.

**Rationale for the order.** The reader's question is almost always "what
did this produce?", so OUTPUT comes first among the bodies. EMBRYONIC sits
immediately after it as the explanatory companion — *"and here is the
structure that produced it"* — with RESEQUENCED next to it, the two
un-stepped views adjacent. INPUT moves to the end because it is the one
section the reader already has (it is their own source file); it becomes
reference material rather than the lead. The name **EMBRYONIC** matches the
NYES vocabulary for the pre-stepped states it renders (FOOP-65 §5.3.1 gates
on exactly this condition).

**This reorder touches every baseline** — on top of the appended section —
which is a further reason the corpus-wide re-promotion is its own reviewed
step (§5). The **content** of OUTPUT and INPUT must be byte-identical
to today's; only their position and the `sections:` line change.

**Tooling impact — `einmo` is in-workspace and must be updated too.**
`einmo` is a workspace member (`/storage1/human/hcbusy/foolish/einmo`,
`Cargo.toml:7`, depended on by path from `foolish-ubca/Cargo.toml:17`), so
this is fully in scope rather than an upstream request. Two verified
touch-points:

- **`einmo/src/compare.rs:69-71`** hardcodes which sections are always
  compared:
  ```rust
  let is_output = name == "OUTPUT" || (name.starts_with("OUTPUT[") && name.ends_with(']'));
  let always = name == "INPUT" || name == "DIFF" || is_output;
  ```
  **`EMBRYONIC` must be added to this always-compared set.** If it is not,
  the section is written into every baseline but never diffed — the
  section would be snapshot-pinned in name only, and a regression in
  pre-step rendering would pass every gate silently. This is the single
  most important tooling change in this FOOP; assert it with a test that
  mutates only the EMBRYONIC body and confirms `einmo compare` reports a
  difference.
  (`INPUT` keeps its name and stays in `always` untouched — §1.1.)
- **The section-name constants and fixtures** in `einmo/src/verify.rs`
  (257-265), `transitions.rs` (461-476), and `cli.rs` (1151) enumerate
  section names and ordering, and need updating in step.
- **Ordering assumptions in the envelope parser/writer** must be checked:
  anything that assumes `INPUT` is the first body needs fixing. Verify
  rather than presume — `einmo compare`, `einmo promote`, and the
  round-trip parse must all survive the reorder.

### §2. Where it comes from

The pre-step FIR is already reachable — no new construction path is needed.
`UbcaEvaluator::evaluate` (`foolish-ubca/src/evaluator.rs:118-149`) today:

1. calls `compose_program_with_system(source)` → the composed root, **fully
   built and wholly un-stepped**;
2. calls `step_to_settled(...)`;
3. extracts `program_result(...)` and converts via `proto_to_core_fir`.

The EMBRYONIC rendering is the same conversion applied at the end of
step 1 rather than step 3. The evaluator gains a way to return both
renderings; the einmo adapter (`ubca_snapshot_tester.rs:36-48`, which today
maps each settled FIR through `FirSequencer::format`) emits the settled
chunk then the as-parsed chunk.

The conversion used for the pre-step rendering must be **purely
read-only** — it may not populate helpers, set flags, or step anything.
That is what §3 makes possible; without it, this FOOP cannot be
implemented correctly.

### §3. `stmt_count` must not mutate — split it into two methods

**The defect, verified in the code (2026-08-08).**
`ConcatenationFir::stmt_count` (`fir_kinds.rs:2840-2855`) is **not a pure
read**. On a concatenation whose helpers are not yet populated it performs

```rust
self._helpers_populated.set(true);
self.populate_concat_helpers();      // constanic-clones every element's lines
```

— i.e. **asking an un-stepped concatenation how many statements it has
forces the join**. A query mutates the thing queried. `stmt_count` is
declared on the `Fir` trait (`fir_trait.rs:346`) as `fn stmt_count(&self)
-> Option<usize>` — an `&self` read — and every other implementation
honours that; only this one does hidden work behind `Cell`/interior
mutability.

**This is a pre-existing latent bug, not merely an obstacle for §1.** The
evidence that the forcing is accidental rather than designed is in
`ConcatenationFir` itself: its two *sibling* accessors both **guard** on
the same flag and decline to force it —

- `stmt_at` (`fir_kinds.rs:2857-2860`): `if !self._helpers_populated.get()
  { return None; }`
- `_search_brane` (`fir_kinds.rs:2887-2889`): the identical early return

So `stmt_at` says "not populated ⇒ I have nothing", while `stmt_count` says
"not populated ⇒ let me populate". The two disagree about what an
un-populated concatenation *is*, and callers can observe the inconsistency
(`stmt_count()` returning N while `stmt_at(0)` returns `None`, until the
`stmt_count` call itself silently repairs it). This FOOP is what finally
exposes the divergence, because it is the first consumer that renders a FIR
tree which is *supposed* to stay un-populated.

**The fix — two methods with separate, honest contracts:**

```rust
// Pure read. NEVER mutates. Safe on any FIR in any NYES state.
// An un-populated concatenation reports what it can see WITHOUT joining
// — consistent with `stmt_at`/`_search_brane`, which already decline.
fn stmt_count(&self) -> Option<usize>;

// Explicitly performs the join if it has not happened yet, THEN counts.
// Named so the side effect is visible at every call site.
fn ensure_joined_stmt_count(&self) -> Option<usize>;
```

(The second name is the implementer's to finalise —
`force_join_stmt_count`, `join_then_stmt_count`, etc. What is **required**
is that the mutating behaviour is reachable only through a name that *says
so*, and that plain `stmt_count` becomes a pure read.)

**Migration — every existing caller must be classified, not blanket-renamed.**
There are ~20 non-test `stmt_count()` call sites (`fir_kinds.rs`,
`evaluator.rs`, `system_foo.rs`). Each must be inspected and assigned:

- Callers that **need the join to have happened** — principally the
  settled-rendering path (`evaluator.rs:713-716`), `program_result`
  (`system_foo.rs:486`), and the search/navigation paths that today rely on
  the forcing to make a concatenation navigable — move to
  `ensure_joined_stmt_count()`. **These preserve today's behaviour exactly**;
  that is the point of doing it call-site by call-site.
- Callers that are genuine queries — the new pre-step walk, diagnostics,
  `is_brane_like` (`fir_trait.rs:366`) — keep `stmt_count()`.

**This is behaviour-preserving by construction.** Done correctly, no
existing einmo OUTPUT moves: every site that used to force the join still
forces it, just under a name that admits it. **Any baseline that does move
means a call site was misclassified** — that is a bug to fix, never a
baseline to promote (AGENTS.md §"Non-regression invariant").

### §4. Inspection of embryonic Foolish for reasonably informative rendering — for purposes of future development, writing and maintaining Foolish programs

**This is a first-class step of this FOOP, by agent AND by human — not a
review afterthought.**

**Who this rendering is for.** Not primarily the FVM implementor debugging
Rust internals — it is for the **Foolisher writing and maintaining Foolish
programs**, now and in the future. That sets the bar: the pre-step
rendering must read as *Foolish*, recognisably related to the source the
developer wrote, and must help answer the questions a program author
actually asks — "did that group the way I meant?", "what did my operand
split turn into?", "is this the statement I think it is?". Internal FVM
vocabulary (helper branes, `items=` debug forms, NYES bookkeeping beyond
what a reader needs) is a rendering defect against this criterion, not a
neutral implementation detail.

**Why it is significant.** The sequencer has been developed, tuned, and
snapshot-pinned almost entirely against **settled** FIR trees. Its
pre-constanic paths exist but are comparatively unexercised: they are what
you get when a value could not settle, not something anyone has designed
*for*. Turning them into a standing, corpus-wide output makes them
first-class — and the honest expectation is that a good deal of
sequencer repair surfaces here. **The rendering is not assumed correct or
informative until it has actually been looked at.**

**What the step consists of:**

1. **Generate** the EMBRYONIC rendering across a broad slice of the
   existing einmo corpus (not just new tests) — nested branes,
   concatenations, searches, SF/SFF markers, operators, creations,
   comparisons, if-expressions.
2. **Agent inspection.** The agent reads the pre-step renderings and, for
   each construct, answers concretely: does this show what was actually
   built? Can a **program author** read the operand grouping and precedence
   off it? Does it read as Foolish, recognisably related to the source they
   wrote? Is anything rendered as a bare placeholder, an internal name, a
   debug `items=` form, or an empty shape where structure exists? Is
   anything *missing* that the FIR plainly contains? Findings are recorded
   as defects with the construct that provoked them.
3. **Human inspection.** The human reviews samples against the governing
   criterion: **is this reasonably informative for the purposes of future
   development, writing and maintaining Foolish programs?** The test is not
   "does it match the FIR" but "would a Foolisher looking at this
   understand their own program better". The human's judgement governs; the
   agent may not promote pre-step baselines on its own assessment.
4. **Repair, then re-inspect.** Sequencer fixes land, then the loop runs
   again. Only when the human is satisfied does the rendering format
   freeze and baselines get promoted.

**Discipline:** this step must not be short-circuited by promoting whatever
the sequencer currently emits. Per AGENTS.md §"The einmo review workflow"
step 4, every OUTPUT line must be justifiable in the agent's own words —
"the evaluator emitted this" is not a justification, and that rule applies
with extra force here, where the output is *new* and there is no prior
baseline to disagree with.

**Expected outputs:** a list of sequencer defects (each fixed or explicitly
deferred with a reason), a fixed rendering format for the section, and
human sign-off that the pre-step rendering is fit for development use.

### §5. Corpus impact — every baseline changes

Adding a section to **every** test means **every** `checked/` baseline in
the einmo suite gains new lines. This is a deliberate, human-directed
corpus-wide change, and it interacts with two standing rules:

- **AGENTS.md §"Non-regression invariant (hard rule)"** forbids a FOOP from
  changing another FOOP's einmo OUTPUT. This FOOP rewrites every baseline
  twice over: it *appends* the EMBRYONIC section and *reorders* the
  existing ones (§1.1). It is permissible **only** because the change is
  **content-preserving** — every
  section's bytes are unchanged; only their order changes, plus one new
  section. **Any baseline whose OUTPUT or INPUT *content* changes is a
  regression, not an expected update** (an OUTPUT change means a §3 call
  site was misclassified). Reviewers should diff with section-aware tooling
  rather than raw line diffs, since a pure reorder makes every baseline
  look heavily changed to `diff`.
- **`verified/` baselines are frozen** and require the human reviewer's
  key. Re-verifying them is a human action; the agent may not perform it.
  The implementer must **enumerate the `verified/` twins before starting**
  and present them for human review, because those cannot be re-promoted
  automatically.

#### §5.1 The re-promotion inspection gate — agent AND human, again

This is a **second, separate inspection gate**, distinct from §4's. §4 asks
*"is the pre-step rendering informative?"* on a sample, while designing it;
§5.1 asks *"did we break anything, and is it still good?"* across the
**whole corpus**, when the format is frozen and every baseline is rewritten.
Both are required, and neither substitutes for the other.

**Gate A — nothing that existed changed.** For every baseline in the suite,
confirm that the **OUTPUT, INPUT, and COMMENTS bodies are byte-identical**
to their pre-FOOP content. Only their *position* may differ (§1.1).
- Do this **mechanically first**: extract each section from the old and new
  envelopes and compare bodies pairwise across the entire corpus. A
  section-aware comparison is mandatory — raw `diff` on reordered files is
  noise, and eyeballing thousands of lines invites exactly the miss this
  gate exists to catch.
- **Any** OUTPUT difference means a §3 `stmt_count` call site was
  misclassified. Fix the code — never promote past it.
- An INPUT or COMMENTS difference means the envelope writer is mangling
  content during the reorder — likewise a bug.
- The agent reports this as a **positive, quantified result** ("N baselines
  checked, N with byte-identical OUTPUT/INPUT/COMMENTS"), not as an absence
  of complaints.

**Gate B — the new sections are worth having.** The human reviews the
EMBRYONIC bodies — and, once §6 has landed, the RESEQUENCED bodies —
across a representative spread of the corpus against §4's governing
criterion: **is this reasonably informative for the purposes of future
development, writing and maintaining Foolish programs?** §4 and §6.5
validate that on the constructs they sample; Gate B is where the whole
corpus gets looked at, and where constructs those steps missed will
surface.

**Both gates are human-gated.** Per AGENTS.md §"The einmo review workflow"
step 4, the agent must be able to justify every new line in its own words
before promotion; "the evaluator emitted this" is not a justification.
Human sign-off is required on both A and B before any `checked/`
promotion, and `verified/` twins additionally need the human key.

### §6. The Foolish Resequencer — regenerate parsable Foolish from the FIR tree

**This is the last step of this FOOP.** It builds on §1-§5 and must not
begin until they are green.

#### §6.1 What it is, and how it differs from EMBRYONIC

Two renderings of the same un-stepped FIR tree, with different jobs:

| | **EMBRYONIC** (§1) | **RESEQUENCED** (§6) |
|---|---|---|
| Audience | a developer inspecting structure | the **parser**, and a developer reading Foolish |
| Shows | the layout/nesting of FIRs | valid, **parsable Foolish** |
| Must round-trip | no | **yes** — two equality checks (§6.3) |
| Format | the existing sequencer's shape | normalized Foolish (§6.2) |

The **Foolish Resequencer** is a **separate sequencer** — a new component,
not a mode of the humanizing sequencer. It takes a FIR tree and emits
Foolish source. Keeping it separate is the point: the humanizing
sequencer's job is to be *readable*, the resequencer's job is to be
*re-parsable*, and those goals conflict often enough that one component
serving both would compromise both.

#### §6.2 Normalization — a new, shared definition

A round-trip comparison is meaningless without a normal form, since the
FIR tree does not retain comments, spacing, or the author's line breaks.
This FOOP therefore introduces **Foolish normalization**: a canonical
textual form that both the original source and the resequenced output are
reduced to before comparison.

The rules below are a **starting point, to be finalized in §6.5** — not a
frozen list:

- remove all comments;
- runs of whitespace **outside strings** collapse to a single space;
- remove empty lines;
- `,` and `;` both normalize to `;`;
- all special characters printed in their **Unicode** forms (per AGENTS.md:
  `⬤` not `{*}`, `<̲` not `\o<`, etc.);
- a brane always starts on a new line;
- indent 2 spaces per nesting level.

**Normalization is a specified artifact of this FOOP, not a test helper.**
It must live in a documented module with its own unit tests, because two
separate things depend on it agreeing with itself (§6.3), and because a
subtly wrong normalizer produces either false round-trip failures or —
worse — false passes.

**Open for §6.5:** whether normalization is defined over *text* (a source
transformation) or derived from a *canonical resequencing* (normalize by
parsing and re-emitting). The latter is more robust but makes the first
equality check partly circular; see Open Questions.

#### §6.3 The two equality checks

The RESEQUENCED section carries the resequenced Foolish, and the test
asserts **two** properties:

1. **Fidelity:** `normalize(resequence(parse(source)))` == `normalize(source)`
   — the resequenced Foolish matches the original input after
   normalization. *This is the real test of the parser and FIR-gen phases:
   it says the FIR tree retained everything the source meant.*
2. **Idempotence / stability:** parsing the resequenced output and
   resequencing *that* yields byte-identical text —
   `resequence(parse(r))` == `r` where `r = resequence(parse(source))`.
   *This says the resequencer has a fixed point and the parser agrees with
   it. A resequencer can satisfy (1) by accident on simple inputs while
   still being unstable; (2) catches that.*

Check 2 is strictly the cheaper and more robust of the two — it needs no
normalizer — and it should be implemented and passing first.

#### §6.4 Known limit — creations cannot be recreated

A creation (`⬤` / `{*}`) has **identity**, not just form: `CreationFir` is
born Independent and compared by `Rc::ptr_eq` (FOOP-33). Two textually
identical `⬤` tokens are two *different* creations. So resequencing can
reproduce the creation's **syntax** but never re-establish the identity of
the original object — re-parsing the output produces a new, distinct
creation.

Consequences to specify rather than discover:

- Fidelity (check 1) compares **text after normalization**, so a creation
  compares equal to a creation — the identity difference is invisible to
  the check, and check 1 remains meaningful.
- Named creations (FOOP-33) render via `hs_creation_name`
  (`sequencer.rs:606-609`), so a creation reached through a
  null-characterized statement resequences under its original name. The
  resequencer must not emit a name that would be read as a *rename* on
  re-parse — FOOP-33 refuses a second, different null-characterized name
  for an already-named creation.
- **The round-trip is textual, never identity-preserving.** Any future
  feature that assumes otherwise is mistaken; this limit is inherent, not
  a defect to fix.

#### §6.5 Inspection step — again, agent AND human

As with §4, the resequenced output must be **inspected before it is
frozen**, by agent and by human, against the same governing criterion:
*reasonably informative for the purposes of future development, writing
and maintaining Foolish programs.* Specifically:

- Is the normalized form one a Foolisher would accept as a faithful
  rendering of their program?
- Are the normalization rules (§6.2) right, and complete? The list above is
  explicitly rough; this step is where it is finalized.
- Where fidelity (check 1) fails, is the fault in the resequencer, the
  normalizer, or **a genuine parser/FIR-gen information loss**? The third
  case is the valuable find — it is a real bug this FOOP exists to surface
  — and must be reported, not normalized away.

**Discipline:** a fidelity failure must never be "fixed" by weakening the
normalizer until it passes. That converts a bug detector into a rubber
stamp. If a rule must be relaxed, the relaxation is justified in its own
words and reviewed.

## FIR Impact

- **No new FIR kind, no new `FirKind` variant, no NYES change.** This FOOP
  changes how existing FIRs are *read and rendered*, never what they are.
- **`stmt_count` becomes a pure read** on the `Fir` trait
  (`fir_trait.rs:346`); the join-forcing behaviour moves to a new,
  explicitly-named `ensure_joined_stmt_count` (§3). `ConcatenationFir` is
  the only implementation that behaves differently today, but the trait
  method's contract changes for everyone.
- **Sequencer pre-constanic paths become load-bearing** and are expected to
  need repair (§4).
- **A new component: the Foolish Resequencer** (§6) — FIR tree → parsable
  Foolish. Separate from the humanizing sequencer, reading the same
  `FirQueryable` surface. Plus a **normalization** module (§6.2) with its
  own tests. Neither changes any FIR.

## UBC Step Impact

- **No stepping change whatsoever.** Both new renderings are taken before
  `step_to_settled` runs and must not step, populate, or set any flag.
- **`UbcaEvaluator::evaluate` gains a pre-step rendering path**
  (`evaluator.rs:118-149`, §2) and the einmo adapter
  (`ubca_snapshot_tester.rs:36-48`) emits the new chunks per test.
- **~20 `stmt_count()` call sites are reclassified** (§3), each preserving
  its current behaviour.
- **The resequencer re-enters the parser** (`foolish-parser::parse`,
  `parser.rs:31`) for the idempotence check (§6.3) — read-only, on a
  separate tree, with no effect on the program under test.

## Test Plan

Tests first, per `rust_instructions.md`.

**Unit — `stmt_count` purity (foolish-ubca, §3):**
- `stmt_count()` on an un-stepped concatenation leaves
  `_helpers_populated == false` and `ubc_children()` empty. **This test
  fails against today's code** — it is the regression test for the defect
  being fixed.
- `stmt_count()` and `stmt_at(..)` agree about emptiness on an un-populated
  concatenation (the sibling consistency the split restores, cf.
  `fir_kinds.rs:2857`/`2887`).
- `ensure_joined_stmt_count()` populates exactly as today's `stmt_count`
  did, and is idempotent.
- **The migration guard:** evaluating any corpus input produces a settled
  result byte-identical to the pre-split code.

**Unit — the pre-step walk (§2):**
- Producing the EMBRYONIC rendering leaves the settled OUTPUT
  byte-identical to the same input evaluated without it (the anti-mutation
  guarantee).
- After producing it, the composed root is still wholly un-stepped — no
  NYES has advanced, no helper is populated.

**Unit — sequencer pre-constanic rendering (foolish-core, §4):**
- One test per major construct asserting the pre-step rendering shows its
  structure: nested branes, concatenation (element grouping visible),
  searches, SF/SFF markers, operators, if-expressions.
- These are written **during** §4's inspection loop, as findings are
  resolved — they are the record of what "informative" was decided to mean.

**Unit — einmo tooling (§1.1):**
- `einmo compare` reports a difference when only the EMBRYONIC body
  differs. **This is the test that proves the new section is actually
  gated** rather than merely written (the `compare.rs:69-71` hazard).
- An envelope round-trips (parse → write) under the new section order with
  every body byte-identical.
- `sections:` declares exactly the emitted order.

**Unit — normalization (§6.2):**
- Each rule gets its own test: comments removed; whitespace runs collapse
  **but not inside strings** (the rule most likely to be got wrong);
  empty lines removed; `,`/`;` → `;`; Unicode operator forms; brane on a
  new line; 2-space indent.
- **Idempotence:** `normalize(normalize(x)) == normalize(x)`.
- Adversarial inputs: a string literal containing `!!`, `;`, or runs of
  spaces must survive untouched.

**Unit — the Foolish Resequencer (§6.3):**
- **Check 2 first (idempotence):** `resequence(parse(r)) == r`. Cheaper,
  needs no normalizer, and should pass before check 1 is attempted.
- **Check 1 (fidelity):**
  `normalize(resequence(parse(src))) == normalize(src)` across a broad
  construct set — nested branes, concatenations, searches, SF/SFF markers,
  operators, comparisons, if-expressions, creations.
- **Creations (§6.4):** a `⬤` resequences to valid syntax and re-parses to
  a *new, distinct* creation — assert the identity is NOT preserved, so
  the limit is pinned rather than assumed.
- A named creation resequences under its original name and does **not**
  re-parse as a FOOP-33 rename.

**Einmo approval tests:**
- `foolish-ubca/einmo_suite/input/foop/95/comprehensive.foo` (reserved
  name) — a program exercising every construct whose pre-step rendering
  §4 examined, so the format is pinned in one place.
- **Corpus-wide round-trip (§6):** once RESEQUENCED lands, both equality
  checks run over **every** input in the suite. This is the payoff — the
  entire corpus becomes a parser/FIR-gen property test. Expect genuine
  parser bugs to surface here; each is a real find to report, never
  something to normalize away (§6.5).
- **The corpus-wide re-promotion is its own, separately-reviewed step**
  (§5) behind the two-part inspection gate of §5.1. Gate A is mechanizable
  and must be run as a script over the whole corpus — extract
  OUTPUT/INPUT/COMMENTS from each old and new envelope and assert
  body-level byte equality — with the result reported quantitatively.
  Gate B is human judgement on the new sections. Enumerate `verified/`
  twins first and present them for human review.

## Rejected Alternatives

### A. Keep `stmt_count` mutating; take the pre-step snapshot from a deep copy

Render the pre-step section from a clone of the composed root, so the forcing
mutation lands on the throwaway copy. Rejected: it leaves a query-that-
mutates in the codebase as a trap for the next caller, keeps `stmt_count`
inconsistent with its own `stmt_at`/`_search_brane` siblings, and pays a
whole-tree deep copy per test to work around a bug rather than fix it. The
split (§3) fixes the defect instead of routing around it.

### B. Blanket-rename every `stmt_count` call site to the joining version

Mechanical and safe-looking, but it would preserve the defect under a new
name and leave the pure `stmt_count` with no callers — including the ones
that genuinely want a non-forcing read. The call-site classification (§3)
is the actual work and cannot be skipped.

### C. Emit the new sections only for tests that need them (e.g. foop/65)

Cheaper: no corpus-wide re-promotion, no `verified/` problem. Rejected
because the general motivation (§Motivation) is the larger half of the
value: parser/precedence regressions anywhere in the corpus become
directly visible rather than surfacing as value diffs. A facility that
only one feature can use is not worth a new section.

### D. Promote the pre-step rendering as-is, skip the inspection step

Fastest path to green. Rejected as the specific failure mode §4 exists to
prevent: the sequencer's pre-constanic paths are unexercised, so
"whatever it currently emits" is precisely what must **not** be frozen into
~every baseline in the suite sight-unseen.

### E. Make RESEQUENCED a mode of the humanizing sequencer rather than a separate component

Reuses the traversal and avoids a new module. Rejected because the two
have conflicting goals — the humanizing
sequencer optimizes for *readability* and is free to elide, summarize, and
annotate with NYES state, while the resequencer must emit **exactly what
re-parses to the same tree**. Threading a "must be parsable" flag through
the humanizing sequencer would constrain every future readability
improvement and make both jobs harder to reason about.

### F. Round-trip on the AST instead of on text

Compare `parse(resequence(tree))` to `tree` structurally, skipping
normalization entirely. Tempting — it is simpler and needs no normalizer.
Rejected as testing less: it verifies the resequencer is self-consistent
but says nothing about whether the output resembles the Foolish the author
wrote, and a resequencer emitting valid-but-unrecognizable Foolish would
pass. The comparison that matters is against the *original input*, which
requires normalization. (The structural check is a fine **additional**
test; it is not a substitute.)

## Open Questions

- **Rendering format inside EMBRYONIC.** Fixed during §4, not before — the
  inspection determines what the rendering should look like. (The section's
  *name and position* are already settled by §1.1; only its contents are
  open.) The delimiter must coexist with einmo's Foolish separator.
- **Do the new sections render the composed root or just the `program`
  member?** The settled OUTPUT renders the `program` member
  (`program_result`, per the FOOP-33 composition). Rendering `system.foo`'s
  pre-step form too would be noise in every test; rendering only `program`
  is the presumed choice, to be confirmed in §4.
- **Ordering against [FOOP-65](FOOP-65.md).** FOOP-65 depends on this FOOP
  for the einmo visibility of its §5.3.1 backtick rendering. This FOOP has
  no dependency on FOOP-65. Landing this one first is preferable; confirm
  the intended order before starting.
- **Scope of sequencer repair.** §4 will surface defects; which are fixed
  here versus deferred to their own FOOPs is a judgement call to be made
  with the human reviewer once the inspection has produced the list.
- **Is normalization textual or canonical-resequencing-derived?** (§6.2)
  A text transformation is simple and independent, but must re-implement
  lexing rules (strings, comments) and can drift from the real lexer.
  Deriving it by parse-and-re-emit is more robust but makes fidelity check
  1 partly circular. **Recommendation:** textual, built on the *real*
  lexer's token stream rather than on regexes, so it cannot drift.
  Decide in §6.5.
- **Does RESEQUENCED reserve its slot from the start?** (§1.1) Landing
  §6 later means rewriting every baseline a second time. Reserving the
  slot up front avoids that but bakes in a section that is empty or absent
  until §6 lands. Decide before Phase 3 freezes the order.
- **Should §6 be its own FOOP?** It is separable — it depends on §1-§5
  only for a place to put its output, and the resequencer/normalizer are
  independently useful. If this FOOP grows further, §6 is the clean cut
  point.

## References

- **Dependent FOOP: [FOOP-65](FOOP-65.md)** (the tail concatenator) — its
  §5.3.1 backtick rendering is observable only through this FOOP's
  EMBRYONIC section; FOOP-65 declares `depends_on: [FOOP-95]`.
- Prior FOOPs: FOOP-13 (concatenation semantics / ConcatBrane — the join
  whose forcing §3 repairs); FOOP-33 §4 (`system.foo` composition — why
  the composed root and the `program` member differ); FOOP-64 (the einmo
  suite and its escalating validation levels).
- Process: `foop.md`; `rust_instructions.md` §"Phase-by-phase testing
  discipline"; AGENTS.md §"The einmo review workflow" (step 4 — justify
  every OUTPUT line) and §"Non-regression invariant (hard rule)"; the
  `foolish-debugging` skill (the throwaway-unit-test workflow this FOOP
  makes cheaper).
- Code anchors (verified 2026-08-08, `jia` @ `dc6db093`):
  `foolish-ubca/src/fir_kinds.rs` — `ConcatenationFir::stmt_count`
  2840-2855 (**the mutating query**), `stmt_at` 2857-2869 (guards),
  `_search_brane` 2887-2893 (guards), `populate_concat_helpers` 2632-2674;
  `foolish-ubca/src/fir_trait.rs` — `stmt_count` trait decl 346,
  `is_brane_like` 366; `foolish-ubca/src/evaluator.rs` —
  `UbcaEvaluator::evaluate` 118-149, `FirKind::Concatenation` render arm
  706-764; `foolish-ubca/src/system_foo.rs` — `program_result` 482-504;
  `foolish-ubca/src/ubca_snapshot_tester.rs` — the einmo adapter 36-48;
  `foolish-core/src/sequencer.rs` — concatenation rendering 496-545 (the
  pre-constanic branch §4 inspects), `FirSequencer::format` 32-34 (the
  humanizing entry point the resequencer sits *beside*, not inside),
  creation rendering 602-609 (`hs_creation_name`, §6.4);
  `foolish-parser/src/parser.rs` — `parse` 31 (the public entry the
  idempotence check §6.3 re-enters).

## Last Updated

**Date**: 2026-08-08
**Updated By**: Claude Code / claude-opus-5
**Changes**: Created (Draft). Adds two einmo sections rendering the
un-stepped FIR tree: **EMBRYONIC** (§1-§2), the program as parsed, showing
what the compiler built; and **RESEQUENCED** (§6), the same tree emitted as
parsable Foolish by a new **Foolish Resequencer**, with **normalization**
and two equality checks (fidelity to the normalized input; idempotence),
making the corpus a round-trip property test of the parser/FIR-gen phases —
creation *identity* (`⬤`) is outside what a round-trip can restore (§6.4).
§3 repairs a real defect the work exposed: `ConcatenationFir::stmt_count`
forces the concatenation join, disagreeing with its `stmt_at` and
`_search_brane` siblings which both guard and decline; it splits into a
pure `stmt_count` and an explicit `ensure_joined_stmt_count`, ~20 call
sites classified individually, behaviour preserved. §1.1 reorders the
envelope to `METADATA, OUTPUT, EMBRYONIC, RESEQUENCED, INPUT, COMMENTS,
STAMPS`, keeping `INPUT`'s wire name and flagging that the new sections
MUST join `compare.rs`'s always-compared set or they are pinned in name
only. Two human-gated inspections: §4 (is the embryonic rendering
informative for writing and maintaining Foolish programs?) and §5.1
Gates A/B (mechanically confirm OUTPUT/INPUT/COMMENTS unchanged corpus-wide;
human review of the new sections). §6 lands last and is flagged in Open
Questions as the clean cut point should this FOOP need splitting.
