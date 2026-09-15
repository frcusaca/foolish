# FOOP-66.plan — learn-tree-calculus

**Read `docs/foop/FOOP-66.md` before executing this plan.** The plan is derived from the
specification and assumes its context; the `§`-pointers below refer to that file.

**Worktree variables, expanded:**

```
WORKTREE_ORIGIN_BRANCH  = jia
WORKTREE_ORIGIN_PATH    = /yolo/foolish
WORKTREE_BRANCH_NAME    = foop-66-learn-tree-calculus
WORKTREE_FULL_FS_PATH   = /yolo/foolish_worktrees/foop-66-learn-tree-calculus
```

---

## Scope guard, standing for the whole plan

**This FOOP writes prose. It changes no code.** Per §5 of the specification, it does **not**
modify:

- `foolish-core/**`, `foolish-parser/**`, `foolish-ubca/**`, `foolish-ubca2/**`,
  `foolish-cli/**`, `einmo/**`, `zweimomo/**` — not one line;
- any einmo suite, in any crate — no input, no `output/`, no `checked/`, no `verified/`;
- `docs/vintage_legacy/EQUIVALENCE.md` — FOOP-36 §N6.5 assigns that refresh to N6's FOOP,
  **not to this one**.

**If a task appears to require touching any of those, STOP and report.** It means the study has
drifted into implementation, and that drift is a human decision.

The files this FOOP is permitted to create or change:

| Path | What |
|------|------|
| `docs/why/TREE_CALCULUS.md` | **The deliverable** (§4). New file. |
| `docs/foop/FOOP-66.md` | This FOOP's spec — corrections and findings recorded as the study proceeds. |
| `docs/foop/FOOP-66.plan.md` | This plan — checkboxes and timestamps. |
| `docs/foop/INDEX.md` | Status updates only. |
| `/yolo/tmp/**` (scratch) | Throwaway `.foo` snippets while drafting Phase 6. Nothing here is committed. |

Anything else is out of scope.

---

## No test gates in the usual sense — and why

**§Test Plan of `FOOP-66.md` is the authority; this is its operational restatement.** Read it
before wondering where the gates went.

- **No "Establish relevant tests" checkbox on Phases 0–5 and 7.** `foop.md` §"Sub-Section Test
  Subsets" requires one at the start of every sub-section. Those phases read web pages and write
  prose; there is no test subset that their work could break, and naming an arbitrary one would
  be the empty ritual the rule exists to prevent. **The omission is a decision, recorded here.**
- **Phase 6 DOES carry a real one**, because it writes Foolish `.foo` snippets and runs them.
- **No Promotion Review Gate anywhere in this plan**, and **no `einmo promote` invocation of any
  kind.** Nothing is generated, so there is nothing to review or promote. A plan that promotes
  nothing needs no gate — and installing an empty one would falsify the record.
- **No `foolish-ubca/einmo_suite/input/foop/66/` and no `comprehensive.foo`.** `foop.md`
  §"Comprehensive FOOP Tests" makes one obligatory for a FOOP that ships a feature. This FOOP
  ships none; a comprehensive test here would test the status quo and sign it as this FOOP's
  contribution. **Discharged by §Test Plan, which states the reason.** If the human disagrees, it
  is a one-line instruction and this plan gains a phase.
- **The repository must still be green at merge.** The study changes no code, so
  `cargo test --workspace` must pass **identically** before and after. The baseline is **791
  tests passing on `jia` at `93afcee3`**. If that number moves, §5's scope boundary was crossed
  and the crossing must be explained, not papered over. Phase 7 keeps the checkbox.

---

## How to work this plan

1. **Read `docs/foop/FOOP-66.md` first.** §1 (what tree calculus is), §2 (the reading plan), §3
   (the questions Q1–Q8) and §5 (the scope boundaries) are load-bearing for every phase.
2. **Work top to bottom.** Phases are ordered by dependency: each depth of reading feeds the next.
3. **Check each box as you finish it, with a timestamp on the next indented line** —
   `(YYYY-MM-DD HH:MM)`.
4. **When a checkbox says STOP, stop.** Those mark decisions that are the human's.
5. **Accumulate doubts; report them once, at the end of a phase** (AGENTS.md §"The agent is
   responsible for correctness"). Do not interrupt per item.
6. **Never fabricate a claim about tree calculus.** If a source is unreachable or a passage is not
   understood, write "could not determine" and say what was tried. §5 of the spec makes this rule
   outrank completeness, and it is the one rule most likely to be quietly broken under time
   pressure.
7. **Cite every technical claim** — spec page, repo file, or book chapter and page. An uncited
   claim is a defect, repaired before human review.

---

## Orientation — the facts you need, so you need not go find them

*Established while `FOOP-66.md` was written, current as of 2026-09-15.*
***Verify before relying on any of them; do not re-derive them from scratch.***

**The three sources** (§References):

| | URL | Read at authoring time? |
|---|---|---|
| Spec page | https://treecalcul.us/specification/ | **Yes** — reachable 2026-09-15; §1 is drawn from it |
| Front page | https://treecalcul.us/ | **Yes** — source of the "reflective" claims and `size size` → `168` |
| Coq repo | https://github.com/barry-jay-personal/tree-calculus | No |
| Book PDF | https://github.com/barry-jay-personal/tree-calculus/blob/master/tree_book.pdf | No |

**The five reduction rules**, as §1 records them — *verify, don't re-derive*:

```
1.  △ △ y z              ⟶  y
2.  △ (△ x) y z          ⟶  x z (y z)
3a. △ (△ w x) y △        ⟶  w
3b. △ (△ w x) y (△ u)    ⟶  x u
3c. △ (△ w x) y (△ u v)  ⟶  y u v
```

Grammar `E ::= △ | E E`, left-associating. Node kinds: *leaf* `△`, *stem* `△ a`, *fork* `△ a b`.
Rules 1 and 2 are the site's stated analogues of **K** and **S**. Booleans: `false = △`,
`true = △ △`, `not = △ (△ (△ △) (△ △ △)) △`.

**Coq files in the repo**, per the GitHub listing — *verify, don't re-derive*:
`Tree_Calculus.v`, `Reflective_Programs.v`, `Extensional_Programs.v`, `Intensional_Programs.v`,
`Rewriting_partI.v`, `Rewriting_theorems.v`, `Lambda_Abstraction_in_VA_Calculus.v`,
`Divide_and_Conquer_in_SF_Calculus.v`, `Incompleteness_of_Combinatory_Logic.v`, plus
`Reflective_Programs.txt`, `tree_book.pdf`, `tree_book_bw.pdf`, `tweets_on_trees.pdf`, and a
`trees/` directory. MIT-licensed. **Ideas may be imported; text and code may not** (§5).

**Foolish files to read, with what each is for:**

| Path | Why | Which question |
|------|-----|----------------|
| `docs/foop/FOOP-36.md` §N6 (≈ lines 1903–2230) | FIR equality: shape half, value half, per-brane table, **§N6.4 equality-with-conditions**, §N6.5 `EQUIVALENCE.md` obligation | **Q5** |
| `docs/foop/FOOP-36.md` §Open Questions **Q9** | Cross-brane creation identity — blocks N6, human's call | Q5 |
| `docs/vintage_legacy/EQUIVALENCE.md` | The vintage operator taxonomy (`=s=`, `==`, `===`, …) — **read only; do not edit** | Q5e |
| `docs/vintage_legacy/ECOSYSTEM.md` | AB/IB, detachment, recoordination | Q2 |
| `docs/vintage_legacy/NAMES_SEARCHES_N_BOUNDS.md` | Name resolution and search | Q3, Q7 |
| `README.md` §"The Unknown" | NK vs ECONSTANIC miss outcomes | Q3 |
| `AGENTS.md` §"Foolish Terminology", §"Searches (FOOP-23)" | The vocabulary the assessment must use | all |
| `docs/foop/FOOP-62.md` §Terminology | *constanic*, *constantew* | Q4 |
| `docs/foop/FOOP-56.md` | The four NYES predicates | Q4 |
| `docs/foop/FOOP-33.md` | `⬤`, named creations, `'True`/`'False` | Q1, Q5b |

**Commands, exact forms:**

```bash
# Run a Foolish snippet (Phase 6 only)
cargo run -p foolish-cli -- run /yolo/tmp/claude-1001/tc_snippet.foo

# The green-at-merge check (Phase 7). Baseline: 791 passing on jia @ 93afcee3.
cargo test --workspace

# FOOP numbering sanity
python3 docs/foop/scripts/foop_check.py check
```

---

## Phase 0 — Begin

*Mechanical. Smaller model. No judgment.*

- [ ] Read `docs/foop/FOOP-66.md` in full — all of it, including §5 (scope boundaries) and
      §Test Plan (why there are no tests). You cannot execute this plan without it.
- [ ] Read `foop.md` §"Plan execution", §"Worktree Branch Tracking", and §"Checkbox Format".
- [ ] Check `begun: [x]` in `docs/foop/FOOP-66.md`'s frontmatter, on `jia`, and commit —
      message: `Major: FOOP-66 Learn Tree Calculus, Phase: Begin--work commences`.
- [ ] Create the worktree:
      ```bash
      cd /yolo/foolish
      git worktree add -b foop-66-learn-tree-calculus \
          /yolo/foolish_worktrees/foop-66-learn-tree-calculus
      cd /yolo/foolish_worktrees/foop-66-learn-tree-calculus
      ```
- [ ] **From here on, ALL work — including edits to `FOOP-66.md` and this plan — happens ONLY in
      `/yolo/foolish_worktrees/foop-66-learn-tree-calculus`.** Nothing is written to
      `/yolo/foolish` until the merge. (`foop.md` §"Plan execution".)
- [ ] Confirm the worktree is green before starting: `cargo test --workspace` →
      **791 passing** expected. If it is not green, STOP — AGENTS.md forbids starting Phase+
      work on a broken suite.

---

## Phase 1 — D1: verify §1 against the specification page

*Comparison against a fixed target. **Smaller model.** No interpretation — that is Phase 2.*

**The job:** §1 of `FOOP-66.md` was written from the spec page. Read the page again and check
§1 against it, claim by claim. You are diffing prose against a source, not judging whether the
claims are interesting.

- [ ] Fetch https://treecalcul.us/specification/ and read it in full.
- [ ] Fetch https://treecalcul.us/ (the front page) and read it in full.
- [ ] Verify each §1 claim against the sources, and record the verdict for each:
  - [ ] The grammar `E ::= △ | E E`, and left-associating application.
  - [ ] The three node names — *leaf* `△`, *stem of a* `△ a`, *fork of a and b* `△ a b`.
  - [ ] **All five reduction rules, character by character.** These are the most load-bearing
        facts in the FOOP; a transcription error here poisons Q3 and Q5. Check each of rules 1,
        2, 3a, 3b, 3c separately.
  - [ ] Rules 1 and 2 as the stated K and S analogues.
  - [ ] "We call irreducible expressions *values* or *programs*", and values = binary trees.
  - [ ] The confluence claim, **including its qualifier** — "only allow observing values, not
        reducible expressions." The qualifier is the interesting part and must not be dropped.
  - [ ] The boolean encoding `false = △`, `true = △ △`,
        `not = △ (△ (△ △) (△ △ △)) △`.
  - [ ] The front page's four claims (minimal / Turing-complete / reflective / modular) and the
        `size size` → `168` example.
- [ ] Record, verbatim, **what the sources say about when tree calculus was developed** — dates,
      version history, any "since" or "first published" statement. This feeds §Open Questions'
      chronology item. If the sources say nothing, record that they say nothing.
- [ ] **Correct `FOOP-66.md` §1 in place** for anything found wrong, and note each correction in
      a short list at the end of §1 so the change is visible rather than silent.
- [ ] **STOP CONDITION** — if the spec page is **unreachable**, or if §1 contradicts it in a way
      that changes a **Q3 or Q5 premise** (e.g. the fork rules are not as transcribed, or the
      confluence qualifier is different), **STOP and report to the human** rather than
      improvising a correction. A changed premise changes the study's questions.
- [ ] Commit: `Major: FOOP-66 Learn Tree Calculus, Phase: D1 spec-page verification--complete`.

---

## Phase 2 — D1: provisional answers to Q1–Q8

*First interpretation. **Larger model.** This is where the FOOP starts earning its keep.*

**The job:** holding Foolish's semantics and tree calculus's side by side, write a *provisional*
answer to each of §3's seven questions from the spec page alone. Provisional means: good enough
to direct the deeper reading, and explicitly marked as not yet checked against the repo or book.

- [ ] Read the Foolish background for the questions — the Orientation table above says which file
      serves which question. At minimum: `AGENTS.md` §"Foolish Terminology" and §"Searches
      (FOOP-23)", `README.md` §"The Unknown", and **`FOOP-36.md` §N6 in full**.
- [ ] Draft `docs/why/TREE_CALCULUS.md` with the §4 structure: (1) what tree calculus is,
      (2) Q1–Q8 one section each, (3) worked correspondences, (4) ranked findings, (5) what was
      not read and why, (6) "## Last Updated". Sections 3–5 are stubs at this stage.
- [ ] Write section (1) — "What tree calculus is" — self-contained, at §1's depth, for a
      Foolisher who has never heard of it. Every claim cited.
- [ ] Provisional **Q1** — primitives and irreducible core (`△` vs brane/statement/`⬤`).
- [ ] Provisional **Q2** — tree vs brane. Test, do not assume, the three asymmetries §3 names:
      unlabeled-and-binary vs named-and-n-ary; no statement position vs `#N`/`^`/`$`/`&`;
      context-free reduction vs AB/IB recoordination.
- [ ] Provisional **Q3** — fork rules vs searches. Address the totality point head-on: rules
      3a–3c always succeed; a Foolish search can miss, with two distinct outcomes.
- [ ] Provisional **Q4** — NYES and the constanic ladder. The prior expectation is "no
      counterpart"; if that holds, the real content is what Foolish buys with eight states, and
      whether **ECONSTANIC** is the one with no analogue whatsoever.
- [ ] Provisional **Q5** — equality. Write the Foolish side in full here (N6.1–N6.5 and Q9), and
      state precisely what would have to be true of Jay's equality for it to bear on Foolish's
      problem. **This is the specification against which Phases 3 and 4 read.**
- [ ] Provisional **Q6** — simplification, or confirmation of a choice already made.
- [ ] Provisional **Q8** — idealized definition vs approximated implementation (human,
      2026-09-15). Does tree calculus distinguish the two, and where does it draw the line? Its
      values are finite trees, so an infinite construction cannot be a value — how does the
      theory talk about such objects, if at all? Carry the `integers.foo` / `reals.foo` vs
      IEEE example as the concrete case, and focus on **the seam**: where must an approximation
      be VISIBLE to a Foolisher?
- [ ] Provisional **Q7** — what tree calculus is NOT about. Expected to be one of the longer
      sections; name the Foolish concerns with no counterpart at all.
- [ ] Mark every provisional answer clearly as provisional, with the depth it rests on (D1).
- [ ] Record the **chronology** finding from Phase 1 against §Open Questions' third item. **If the
      "same time" parallel turns out to be loose, correct `FOOP-66.md` §Motivation rather than
      defending it** — §Open Questions already says the study's value does not depend on the dates.
- [ ] Report accumulated doubts from Phases 1–2 to the human in ONE statement — or record
      "no doubts".
- [ ] Commit: `Major: FOOP-66 Learn Tree Calculus, Phase: D1 provisional Q1-Q8--complete`.

---

## Phase 3 — D2: the Coq repository

*Mostly extraction. **Smaller model**, with a hard stop condition.*

**The job:** find the equality program and the self-evaluator in the repo, and transcribe what is
*claimed* about them — Coq theorem statements are precise and short, which is exactly why this
depth exists. **You are not proving anything and not evaluating anything.**

- [ ] Read the repo's `README` / `README.md` and `Reflective_Programs.txt`.
- [ ] Map chapters to proof files, as far as the repo states it. If it does not state it, say so.
- [ ] Open `Tree_Calculus.v` and transcribe the **definition of the calculus itself** — the
      reduction relation as Coq states it. Compare against §1's five rules and report any
      discrepancy. **A discrepancy here is significant**: it means the spec page and the formal
      development differ, which is itself a finding.
- [ ] Open `Intensional_Programs.v` and `Reflective_Programs.v`. Locate:
  - [ ] the **equality program** — its definition and its stated theorem, transcribed verbatim
        into the assessment's Q5 section;
  - [ ] the **size program** (the front page's `size size` → `168`);
  - [ ] the **self-evaluator**.
- [ ] Note what `Incompleteness_of_Combinatory_Logic.v` claims. It is likely to bear on **Q6**
      (what a tree calculus can express that combinatory logic cannot), and possibly on Q3.
- [ ] Record the **license and provenance** of everything transcribed, and confirm that only
      *statements* were copied for quotation with citation — **no Coq code enters any Foolish
      source file** (§5).
- [ ] **STOP CONDITION** — if **no equality program is identifiable** in the repo, or if what is
      found does not match the book description's claim of "an equality program that can decide
      its own equality", **STOP and report.** Do not guess which definition is meant. §1 flags
      that claim as publisher-description provenance, the weakest tier in the FOOP; this phase is
      where it is confirmed or withdrawn.
- [ ] Update the provisional Q1–Q8 answers with everything D2 established, marking each upgraded
      answer with its new depth.
- [ ] Commit: `Major: FOOP-66 Learn Tree Calculus, Phase: D2 Coq repository--complete`.

---

## Phase 4 — D3: the book, relevant chapters only

*Deep reading of formal mathematics against Foolish's own unfinished design. **Larger model.**
The hardest phase in the FOOP.*

**The job:** answer **Q5** with an argued yes or no, and answer or explicitly retire the rest.
**Reading the whole book is not a deliverable** (§2) — read the equality chapter and the
reflection chapter, and read others only where D1/D2 raised a question that needs them.

- [ ] Fetch `tree_book.pdf` and identify its table of contents. Record the chapter list in the
      assessment's "what was not read" section, so the reader can see what was skipped.
- [ ] Read the **equality** chapter in full.
- [ ] Read the **reflection / self-application** chapter in full.
- [ ] Answer **Q5a** — does the equality program decide *structural* equality of trees, and is
      that all it decides? Cite chapter and page.
- [ ] Answer **Q5b** — does tree calculus have anything whose identity is **not** determined by
      its structure (the analogue of Foolish's `⬤` creation identity)? **A "no" here is a
      first-class finding, not a failure**: it means Jay's equality does not address Foolish's
      actual difficulty, which is exactly what FOOP-36 §N6 needs to know. **Do not manufacture a
      synergy to avoid a negative result.**
- [ ] Answer **Q5c** — anything resembling §N6.4's equality-with-conditions (a residual of
      required identifications)? Check whether the book or its citations frame it as unification;
      if so, that framing is importable into N6's FOOP even if no machinery is.
- [ ] Answer **Q5d** — **does the absence of a normal form in Foolish make "Foolish equality" a
      different question rather than a harder one?** Tree calculus is confluent, so equality can
      be decided by normalizing and comparing; Foolish has ECONSTANIC, which is settled but not
      conclusive and may still gain a value under recoordination. **If the answer is yes, this
      goes at the TOP of the ranked findings**, because it bounds what any import from Jay could
      achieve.
- [ ] Answer **Q5e** — does anything here inform `EQUIVALENCE.md`'s `===` ("equal under ALL
      possible coordinations"), the member of that taxonomy with no obvious decision procedure?
      **Record the answer in this FOOP's assessment only — do not edit `EQUIVALENCE.md`**; FOOP-36
      §N6.5 assigns that refresh to N6's FOOP.
- [ ] Revisit Q1, Q2, Q3, Q4, Q6, Q7, Q8 with whatever the book added; where it added nothing, say so
      and close the question at its D1/D2 answer.
- [ ] Explicitly declare any question **"could not determine"**, with what was read and what would
      be needed. This is a legitimate outcome and must not be disguised as a weak finding.
- [ ] **STOP CONDITION** — if answering a question requires **building or running the Coq proofs**,
      STOP and ask. That is a new toolchain dependency and §Open Questions makes it the human's
      decision, not an agent's.
- [ ] Report accumulated doubts from Phases 3–4 in ONE statement — or record "no doubts".
- [ ] Commit: `Major: FOOP-66 Learn Tree Calculus, Phase: D3 book--complete`.

---

## Phase 5 — Write the assessment and rank the findings

***Larger model. NOT delegable*** *(§Plan of Execution for Plan). Every finding here is a claim
about Foolish's design.*

- [ ] Finalize `docs/why/TREE_CALCULUS.md` sections (1) and (2) — the summary and Q1–Q8 — so each
      question carries **a finding, a "no correspondence" with its argument, or a "could not
      determine" with what was tried.** No question silently dropped.
- [ ] **Source-fidelity pass** — walk the document and confirm **every technical claim about tree
      calculus cites its source** (spec page, repo file, or book chapter and page). An uncited
      claim is a defect; repair it now, before the human sees it. *This is the study's analogue of
      a passing test, and the one an agent can check on itself.*
- [ ] Write section (4) — **the ranked findings**, most valuable first. Each item is exactly one
      of:
  - [ ] **(a)** promote to a follow-on FOOP — naming what it would specify;
  - [ ] **(b)** an input to FOOP-36 §N6's FOOP — naming **which sub-section**
        (N6.1 / N6.2 / N6.3 / N6.4 / N6.5) and **what it changes**;
  - [ ] **(c)** decline, with the reason.
        **An empty list is a legitimate outcome** if the reason is argued. Do not pad it.
- [ ] Write section (5) — **what was not read, and why.** Explicit and honest.
- [ ] Add section (6) — the "## Last Updated" block, per AGENTS.md §Markdown File Update Protocol
      (single newest entry, REPLACE not append).
- [ ] Update `FOOP-66.md` §Open Questions: resolve the chronology item, resolve the
      `docs/why/TREE_CALCULUS.md` home/filename item, and leave the **follow-on scope** item for
      the human (Phase 7).
- [ ] Commit: `Major: FOOP-66 Learn Tree Calculus, Phase: Assessment written--complete`.

---

## Phase 6 — The worked correspondences (`.foo` snippets)

*Fixed target: the snippet parses and runs, or it does not. **Smaller model.** The only phase in
this FOOP with a genuine test subset.*

- [ ] **Establish relevant tests for this phase.** This phase writes Foolish source, so it has a
      real subset. Use [these instructions](../../README.md#running-specific-tests) to run unit
      tests: `foolish-parser` (the whole crate — the snippets exercise the lexer and parser) and
      `foolish-cli`. **No einmo case is relevant**: this phase adds no suite input and changes no
      baseline. Run the subset after each snippet.
- [ ] Draft snippets in the scratch directory (`/yolo/tmp/claude-1001/…`), **not** in the repo.
      Nothing under `foolish-ubca/einmo_suite/**` is created (§5).
- [ ] Write **at least two, at most about five** worked correspondences. Natural candidates:
  - [ ] the `false`/`true`/`not` encoding beside Foolish's `'True`/`'False` from FOOP-33's
        `system.foo`;
  - [ ] the fork rules' leaf/stem/fork dispatch beside a Foolish search that distinguishes cases.
- [ ] **Every Foolish snippet uses the Unicode operator forms** — `⬤` not `{*}`, `<̲`, `>̲`,
      `<̲=̲`, `>̲=̲`, `=̲=̲` — per AGENTS.md §Code Style. The `\o` prefix is keyboard input only and
      must not appear.
- [ ] **Run every snippet** — `cargo run -p foolish-cli -- run <path>` — and report its **actual**
      behavior in the assessment. A snippet that does not run is fixed or removed; **it is never
      reported as if it ran.**
- [ ] For each correspondence, state explicitly **what the translation preserves and what it
      loses.** The losses are the interesting half and are the reason these examples exist.
- [ ] **STOP CONDITION** — if a snippet does not parse and the reason is not obvious, **STOP and
      report** rather than reshaping the example until something compiles. A snippet bent to fit
      the compiler no longer illustrates the correspondence it was written for.
- [ ] Paste the final snippets and their real output into `docs/why/TREE_CALCULUS.md` section (3).
- [ ] Run the phase's test subset one more time; then `cargo test --workspace` — **791 passing**,
      unchanged. Any movement means code was touched; STOP and explain.
- [ ] Commit: `Major: FOOP-66 Learn Tree Calculus, Phase: Worked correspondences--complete`.

---

## Phase 7 — Review, merge, cleanup

- [ ] Re-read `docs/why/TREE_CALCULUS.md` end to end, as a reader who has never seen tree
      calculus. Fix anything that only makes sense to someone who just read the sources.
- [ ] Confirm the §5 scope boundaries held: `git diff --stat jia...foop-66-learn-tree-calculus`
      shows **only** `docs/why/TREE_CALCULUS.md`, `docs/foop/FOOP-66.md`,
      `docs/foop/FOOP-66.plan.md`, and `docs/foop/INDEX.md`. **Anything else is a scope breach —
      STOP and report it rather than merging it.**
- [ ] Update `docs/foop/INDEX.md`: status `Draft` → `Complete` in the master table, and move the
      FOOP-66 entry from the "Draft" list to the "Complete" list with a one-paragraph summary of
      what the study concluded.
- [ ] Run `python3 docs/foop/scripts/foop_check.py check` — numbering must be consistent.
- [ ] Run all tests — old and new — and make sure they all pass correctly.
      (`cargo test --workspace` → **791 passing**, matching the `93afcee3` baseline exactly. This
      FOOP adds no tests, so an *increase* is as suspect as a decrease.)
- [ ] **Present the findings to the human**, in one message: the ranked findings list, the
      "what was not read" section, and every accumulated doubt from all phases.
- [ ] **STOP! STOP!! STOP!!! ASK HUMAN to check this box before continuing. UNDER NO
      CIRCUMSTANCES will Agent continue past this point automatically!!**
  - [ ] Present the human with
        `cd /yolo/foolish_worktrees/foop-66-learn-tree-calculus` and ask them to read
        `docs/why/TREE_CALCULUS.md` BEFORE checking the parent checkbox.
  - [ ] **Ask the human to settle §Open Questions' first item** — study only (i), study plus
        authority to write a follow-on FOOP spec (ii), or study plus a bounded implementation
        (iii). The FOOP is written for **(i)**. Record the answer in `FOOP-66.md`.
  - [ ] Remind them of the standard form: "Above message comes from FOOP-66, the tree calculus
        study; the worktree is at `/yolo/foolish_worktrees/foop-66-learn-tree-calculus`. PTAL"
- [ ] Verify all work is complete in `/yolo/foolish_worktrees/foop-66-learn-tree-calculus` and
      committed to `foop-66-learn-tree-calculus`.
- [ ] Merge `foop-66-learn-tree-calculus` to `jia`
  - [ ] Confirm `jia..foop-66-learn-tree-calculus` is empty after the merge and the worktree has
        no uncommitted changes.
  - [ ] Run all tests — old and new — and make sure they all pass correctly, on `jia` after the
        merge (**791 passing**).
- [ ] Cleanup `/yolo/foolish_worktrees/foop-66-learn-tree-calculus`
  - [ ] Check that this plan has all but the Cleanup checkboxes completed
  - [ ] Remove `/yolo/foolish_worktrees/foop-66-learn-tree-calculus`
  - [ ] This is the last sub-task checkbox to be checked in this block

---

## Last Updated

**Date**: 2026-09-15
**Updated By**: Claude Code / claude-opus-5
**Changes**: Created the plan for FOOP-66 (tree calculus study). Eight phases following §2's
outside-in reading plan: Phase 0 begin + worktree; Phase 1 verify §1 against the spec page
(smaller model, pure comparison); Phase 2 provisional Q1–Q8 (larger model, first interpretation);
Phase 3 the Coq repo, transcribing the equality program's definition and theorem (smaller model,
hard stop condition if no equality program is identifiable); Phase 4 the book's equality and
reflection chapters only (larger model — the hardest phase, where **Q5b** and **Q5d** may return
first-class *negative* findings); Phase 5 the assessment and ranked findings (larger model, not
delegable); Phase 6 the `.foo` worked correspondences — **the one phase with a real
"Establish relevant tests" checkbox**, since it is the only one that writes Foolish source;
Phase 7 review, human STOP, merge, cleanup. The plan states plainly, in its own §"No test gates in
the usual sense", **why** there is no Promotion Review Gate, no `einmo promote`, no
`foop/66/comprehensive.foo`, and no test checkbox on the prose phases — each omission is a
recorded decision rather than a lapse — while keeping the green-at-merge gate against the
**791-test `93afcee3` baseline**, where an *increase* is as suspect as a decrease. A standing
scope guard lists the four files this FOOP may touch and directs a STOP on anything else,
including `EQUIVALENCE.md`, whose refresh FOOP-36 §N6.5 assigns to N6's FOOP and not to this one.
