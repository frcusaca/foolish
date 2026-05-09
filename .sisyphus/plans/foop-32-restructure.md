# FOOP-32: Write UBCb Design Spec with WODEP, StatementFir, EMBRYONIC Search

## TL;DR

> **Quick Summary**: Rewrite `FOOP-32.md` — compress existing NYES analysis into compact findings, then write the complete UBCb design spec: WODEP state, StatementFir subtype, EMBRYONIC search resolution with full AB walk, UBCb compiler with persistence, and computation-as-process model.
>
> **Deliverables**: One restructured `docs/foop/FOOP-32.md` (~400 lines)
>
> **Estimated Effort**: Medium (single file, but substantial new content)
> **Parallel Execution**: NO — sequential edits on one file
> **Critical Path**: Compress analysis → Write design → Polish flow

---

## Context

### Original Request
"Update the desired behavior right here and implement it in FOOP-32. Put all the summaries into their own section, and create a UBCb design section that outlines how we think it might be accomplished. Compact and prepare for a description of implementation for UBCb."

Followed by detailed UBCb design specification covering:
- NYES chain with WODEP (waiting on dependencies)
- StatementFir as FIR subtype with AST pointers
- EMBRYONIC doing search resolution with full AB walk
- UBCb compiler with persistence at EMBRYONIC/BRANING
- Computation as process, not single state

### Design Captured in Draft
All design decisions recorded in `.sisyphus/drafts/foop-32-design.md`:
- WODEP semantics + worked example (`{a={b=c;}; d=a.b;}`)
- EMBRYONIC stages: step children → gather searches → resolve IB → resolve AB → WODEP or BRANING
- StatementFir: FIR subtype, immutable array, code subspan + AST pointer
- Parent invariant: brane must be at least EMBRYONIC before children processed
- Compiler: `-O PREMBRYONIC` vs `-O CONSTANIC` persistence levels

### Current File Structure
- `docs/foop/FOOP-32.md` — 354 lines: Abstract, Motivation, Architecture Overview, Step Taxonomy, Development Phases, Governing FOOPs, New FOOPs, Test Plan, Research Log (200+ lines), Rejected Alternatives, Open Questions, References

---

## Work Objectives

### Core Objective
Transform FOOP-32 from research-document into design-specification: compress analysis, write the concrete UBCb design the BDFL provided.

### Concrete Deliverables
- `docs/foop/FOOP-32.md` — restructured with WODEP design, StatementFir, EMBRYONIC search, compiler persistence

### Definition of Done
- [ ] NYES analysis compressed from ~130 lines to ~60 lines ("UBC Analysis & Findings" section)
- [ ] NYES state chain updated: PREMBRYONIC → EMBRYONIC → (BRANING | WODEP) with full transition table
- [ ] StatementFir documented: FIR subtype, fields, immutable array, AST storage
- [ ] EMBRYONIC search resolution documented: step children, gather searches, resolve IB, resolve full AB, WODEP or BRANING
- [ ] WODEP semantics: definition, transitions, worked example
- [ ] UBCb compiler: shared parser, persistence levels, optimization
- [ ] Parent/child invariant: brane must be at least EMBRYONIC before children processed
- [ ] Computation-as-process model documented
- [ ] BDFL caveat preserved (stage-wise fairness subject to FOOP-31)
- [ ] Checkpoint definitions (CP-0 to CP-4) unchanged
- [ ] Governing FOOPs table unchanged
- [ ] "## Last Updated" entry added

### Must NOT Have (Guardrails)
- Do NOT alter checkpoint definitions (CP-0 through CP-4)
- Do NOT alter Governing FOOPs table
- Do NOT remove BDFL / FOOP-31 caveat
- Do NOT write actual Rust code — design documentation only
- Do NOT change FOOP frontmatter (status, phase, etc.)
- Do NOT alter the Rejected Alternatives section

---

## Verification Strategy

### QA Scenarios

```
Scenario: WODEP design section present and complete
  Tool: Bash (grep)
  Steps:
    1. grep "WODEP" docs/foop/FOOP-32.md → expect >= 8 occurrences
    2. grep "waiting on dependencies" docs/foop/FOOP-32.md → expect >= 1
    3. grep "cannot make progress" docs/foop/FOOP-32.md → expect >= 1 (WODEP definition)
    4. grep "WOCONSTANIC" docs/foop/FOOP-32.md → expect >= 3
  Expected: All counts met
  Evidence: .sisyphus/evidence/task-1-wodep.txt

Scenario: StatementFir and EMBRYONIC design present
  Tool: Bash (grep)
  Steps:
    1. grep "StatementFir" docs/foop/FOOP-32.md → expect >= 4 occurrences
    2. grep "immutable array" docs/foop/FOOP-32.md → expect >= 1
    3. grep "resolve.*AB" docs/foop/FOOP-32.md → expect >= 2 (resolve in AB chain)
    4. grep "FIR subtype" docs/foop/FOOP-32.md → expect >= 1
  Expected: All counts met
  Evidence: .sisyphus/evidence/task-1-statementfir.txt

Scenario: Compiler persistence documented
  Tool: Bash (grep)
  Steps:
    1. grep "PREMBRYONIC.*persist\|persist.*PREMBRYONIC" docs/foop/FOOP-32.md → expect >= 1
    2. grep "CONSTANIC.*persist\|persist.*CONSTANIC" docs/foop/FOOP-32.md → expect >= 1
    3. grep "shared parser" docs/foop/FOOP-32.md → expect >= 1
  Expected: All greps succeed
  Evidence: .sisyphus/evidence/task-1-compiler.txt

Scenario: Checkpoints and FOOPs unchanged
  Tool: Bash (grep)
  Steps:
    1. grep "Checkpoint-0" docs/foop/FOOP-32.md → must exist
    2. grep "Checkpoint-4" docs/foop/FOOP-32.md → must exist
    3. grep "FOOP-31" docs/foop/FOOP-32.md → must exist (caveat)
  Expected: All present, tables structurally intact
  Evidence: .sisyphus/evidence/task-1-checkpoints.txt
```

---

## Execution Strategy

### Execution Waves

```
Wave 1 (single task — sequential edits on one file):
└── Task 1: Compress analysis + write UBCb design into FOOP-32.md [writing]

Total: 1 task, 3 sequential edits within that task
```

### Dependency Matrix

- **1**: - - complete

---

## TODOs

- [ ] 1. Restructure FOOP-32 with UBCb Design Spec

  **What to do**:

  Read `docs/foop/FOOP-32.md` and `.sisyphus/drafts/foop-32-design.md`, then perform sequential edits:

  **Edit A — Compress Research Log (lines ~186–314):**
  Replace the verbose "Research Log" section with a compact "## UBC Analysis & Findings" section containing:
  - NYES State Summary table (6 rows: PREMBRYONIC→EMBRYONIC, EMBRYONIC→BRANING, BRANING, ECONSTANIC, WOCONSTANIC, CONSTANT/INDEPENDENT/NK — columns: Constant-Time?, Sync, UBCb Role)
  - BRANING cost table (condensed to 7 rows)
  - Synchronization points table (4 rows)
  - Stage-wise Fairness pseudocode + BDFL caveat
  - 4 Key Findings (one line each)
  - Target: ~60 lines replacing ~130 lines

  **Edit B — Insert UBCb Design Spec after Architecture Overview (after line ~74):**

  Write "## UBCb Design" section with these subsections:

  **NYES State Chain:**
  ```
  PREMBRYONIC → EMBRYONIC → (BRANING | WODEP)
  WODEP → BRANING | WOCONSTANIC | ECONSTANIC | CONSTANT | INDEPENDENT
  BRANING → WOCONSTANIC | ECONSTANIC | CONSTANT | INDEPENDENT | NK
  ```
  Include transition table showing all valid transitions with conditions.

  **WODEP — Definition & Semantics:**
  - "WODEP = I cannot make progress AND I cannot declare WOCONSTANIC."
  - WODEP is transient — always resolves to another state
  - WODEP vs WOCONSTANIC: timing (dep still NYE) vs semantics (dep IS constanic)
  - WODEP processing: how it wakes and re-evaluates when dependencies change

  **Worked Example:**
  ```
  {a={b=c;}; d=a.b;}
  ```
  Step-by-step: root EMBRYONIC → inner brane EMBRYONIC → `d` WODEP → root BRANING → inner WOCONSTANIC → WODEP fires → `d` WOCONSTANIC → root WOCONSTANIC.

  **StatementFir — FIR Subtype:**
  - Is a FIR with its own NYES state
  - Fields: `code` (subspan), `ast` (pointer to parent AST), `name` (CharacterizedName, string equality for SPA1)
  - Placed in immutable array (fixed size, members never change)
  - Enables: regexp name search, seek (`#-2`)

  **AST Storage in Every FIR:**
  - Every FIR carries both String code subspan and AST pointer
  - Parser produces AST; compiler attaches AST pointers during FIR creation

  **EMBRYONIC Stage — Search Resolution:**
  1. Step all children to EMBRYONIC
  2. Gather searches without entering another brane (linear time — bounded by brane size)
  3. Resolve searches within IB (immediate brane)
  4. Resolve searches in entire AB chain (all ancestors, no sibling entry)
  5. If search depends on pre-constanic FIR → transition to WODEP
  6. Otherwise → transition to BRANING

  **BRANING Stage:**
  - Perform operator activities if all members complete
  - Check dependencies, move to constanic state when ready
  - WODEP processing: re-evaluate dependencies when their state changes

  **Parent/Child Invariant:**
  - Brane must be at least EMBRYONIC before children can be looked at or processed
  - AB walk will never encounter a PREMBRYONIC ancestor
  - This is a design constraint that WODEP resolves

  **UBCb Compiler:**
  - Own compiler using UBCb's FVM; parser is shared with UBC
  - `-O PREMBRYONIC`: persist raw AST + source (minimal stepping)
  - `-O CONSTANIC`: step via FVM as far as possible, persist FIRs
  - Persisted FIRs allow comparison with UBC output

  **Computation as Process:**
  - WODEP is not a final state — it's a transient waiting point
  - Computation progresses through state transitions triggered by dependency resolution
  - Multiple branes step simultaneously on parallel cores (future: memory-coherent CPU)

  **Edit C — Reorder sections & add Last Updated:**
  - Final order: Abstract → Motivation → Architecture Overview → UBCb Design → Step Taxonomy → UBC Analysis & Findings → Development Phases → Governing FOOPs → New FOOPs → Test Plan → Rejected Alternatives → Open Questions → References → Last Updated
  - Add "## Last Updated" at end with current date, agent info, change summary

  **Must Not do**:
  - Do not alter checkpoint definitions (CP-0 through CP-4)
  - Do not alter Governing FOOPs table
  - Do not remove BDFL caveat
  - Do not write Rust implementation code
  - Do not alter Rejected Alternatives

  **Recommended Agent Profile**:
  - **Category**: `writing`
    - Reason: Substantial technical documentation authoring — not a simple edit, but a structured rewrite with new content
  - **Skills**: None needed
  - **Skills Evaluated but Omitted**: none — pure documentation

  **Parallelization**:
  - **Can Run In Parallel**: NO — sequential edits on one file
  - **Parallel Group**: Sequential (Wave 1, only task)
  - **Blocks**: Final Verification
  - **Blocked By**: None

  **References**:

  **Primary Design Source (CRITICAL — use this as the spec):**
  - `.sisyphus/drafts/foop-32-design.md` — Contains ALL confirmed design decisions: WODEP semantics with worked example, EMBRYONIC search resolution, StatementFir fields, parent/child invariant, compiler persistence, NYES chain. Read this BEFORE writing FOOP-32.

  **Current FOOP-32 (source file):**
  - `docs/foop/FOOP-32.md:1-185` — Existing sections to keep (Abstract through Test Plan)
  - `docs/foop/FOOP-32.md:186-314` — Research Log section to compress
  - `docs/foop/FOOP-32.md:315-354` — Rejected Alternatives through References (keep as-is)

  **Design Context (for cross-referencing):**
  - `docs/ubc1/how/ubc2_design.md:60-100` — UBC2 design principles (message passing, staging, FIR lifecycle)
  - `docs/ubc1/how/ubc2_message_protocol.md:70-140` — Message types (FulfillSearch, RespondToSearch, StateChange)

  **WHY Each Reference Matters**:
  - Draft file is the **authoritative spec** — copy its content into FOOP-32, expanding prose for readability
  - Current FOOP-32 provides the existing sections to preserve and the analysis to compress
  - UBC2 docs provide context for the "How UBCb Differs from UBC" comparison

  **Acceptance Criteria**:

  **Content presence:**
  - [ ] "WODEP" appears >= 8 times
  - [ ] "StatementFir" appears >= 4 times
  - [ ] "immutable array" appears >= 1 time
  - [ ] "cannot make progress" appears >= 1 time (WODEP definition)
  - [ ] "resolve" + "AB" appears >= 2 times
  - [ ] Compiler persistence documented with both -O levels
  - [ ] Worked example `{a={b=c;}; d=a.b;}` present with step-by-step trace

  **Structural integrity:**
  - [ ] Checkpoint definitions (CP-0 through CP-4) unchanged
  - [ ] Governing FOOPs table unchanged
  - [ ] BDFL / FOOP-31 caveat preserved
  - [ ] Rejected Alternatives section unchanged
  - [ ] "## Last Updated" entry added

  **QA Scenarios**:

  ```
  Scenario: WODEP design present and complete
    Tool: Bash (grep)
    Steps:
      1. grep -c "WODEP" docs/foop/FOOP-32.md → expect >= 8
      2. grep "cannot make progress" docs/foop/FOOP-32.md → must match
      3. grep "WOCONSTANIC" docs/foop/FOOP-32.md → expect >= 3
    Expected: All checks pass
    Evidence: .sisyphus/evidence/task-1-wodep.txt

  Scenario: StatementFir and EMBRYONIC design present
    Tool: Bash (grep)
    Steps:
      1. grep -c "StatementFir" docs/foop/FOOP-32.md → expect >= 4
      2. grep "immutable array" docs/foop/FOOP-32.md → must match
      3. grep -c "resolve.*AB\|AB.*resolve" docs/foop/FOOP-32.md → expect >= 2
      4. grep "FIR subtype" docs/foop/FOOP-32.md → must match
    Expected: All checks pass
    Evidence: .sisyphus/evidence/task-1-statementfir.txt

  Scenario: Compiler persistence documented
    Tool: Bash (grep)
    Steps:
      1. grep "PREMBRYONIC" docs/foop/FOOP-32.md → expect >= 1 in compiler section
      2. grep "CONSTANIC" docs/foop/FOOP-32.md → expect >= 1 in compiler section
      3. grep "shared parser" docs/foop/FOOP-32.md → must match
    Expected: All checks pass
    Evidence: .sisyphus/evidence/task-1-compiler.txt

  Scenario: Checkpoints and tables unchanged
    Tool: Bash (grep)
    Steps:
      1. grep "Checkpoint-0" docs/foop/FOOP-32.md → must exist
      2. grep "Checkpoint-4" docs/foop/FOOP-32.md → must exist
      3. grep "FOOP-31" docs/foop/FOOP-32.md → must exist
    Expected: All exist, tables structurally intact
    Evidence: .sisyphus/evidence/task-1-checkpoints.txt

  Scenario: Worked example present
    Tool: Bash (grep)
    Steps:
      1. grep "a={b=c;}" docs/foop/FOOP-32.md → must match
      2. grep "d=a.b" docs/foop/FOOP-32.md → must match (in worked example context)
    Expected: Both match — worked example is present
    Evidence: .sisyphus/evidence/task-1-example.txt
    Failure: Either grep fails → worked example missing or garbled
  ```

  **Evidence to Capture**:
  - [ ] .sisyphus/evidence/task-1-wodep.txt
  - [ ] .sisyphus/evidence/task-1-statementfir.txt
  - [ ] .sisyphus/evidence/task-1-compiler.txt
  - [ ] .sisyphus/evidence/task-1-checkpoints.txt
  - [ ] .sisyphus/evidence/task-1-example.txt

  **Commit**: YES
  - Message: `docs(foop-32): write UBCb design spec with WODEP, StatementFir, EMBRYONIC search resolution, and compiler persistence`
  - Files: `docs/foop/FOOP-32.md`

---

## Final Verification Wave

- [ ] F1. **Plan Compliance Audit** — `oracle`
  Read plan vs final FOOP-32.md. Verify: WODEP design present, StatementFir documented, EMBRYONIC search resolution complete, compiler persistence included, checkpoints/FOOPs unchanged, BDFL caveat preserved, worked example present.
  Output: `Must Have [N/N] | Must NOT Have [N/N] | VERDICT: APPROVE/REJECT`

- [ ] F2. **Content Quality Review** — `writing`
  Read FOOP-32.md for: prose quality (is it readable?), design accuracy (does it match draft?), internal consistency (WODEP transitions make sense), terminology consistency throughout.
  Output: `Readability [CLEAN/N issues] | Accuracy [CLEAN/N issues] | Consistency [CLEAN/N issues] | VERDICT`

- [ ] F3. **Structure Verification** — `quick`
  Run all 5 QA scenario grep suites. Verify all evidence files exist.
  Output: `Scenarios [N/N pass] | VERDICT`

- [ ] F4. **Scope Fidelity** — `deep`
  Compare git diff against plan. Verify: only FOOP-32.md touched, checkpoints unchanged, Governing FOOPs unchanged, Rejected Alternatives unchanged, all design content from draft is present.
  Output: `Scope [CLEAN/N drift] | VERDICT`

---

## Commit Strategy

- **1**: `docs(foop-32): write UBCb design spec with WODEP, StatementFir, EMBRYONIC search resolution, and compiler persistence`
  - Files: `docs/foop/FOOP-32.md`

---

## Success Criteria

### Verification Commands
```bash
grep -c "WODEP" docs/foop/FOOP-32.md                    # expect >= 8
grep -c "StatementFir" docs/foop/FOOP-32.md             # expect >= 4
grep "immutable array" docs/foop/FOOP-32.md             # must match
grep "shared parser" docs/foop/FOOP-32.md               # must match
grep "a={b=c;}" docs/foop/FOOP-32.md                    # worked example present
grep "Checkpoint-0" docs/foop/FOOP-32.md                # checkpoints preserved
grep "FOOP-31" docs/foop/FOOP-32.md                     # caveat preserved
```

### Final Checklist
- [ ] WODEP section with definition, transitions, worked example
- [ ] StatementFir documented as FIR subtype with immutable array
- [ ] EMBRYONIC search resolution with full AB walk
- [ ] BRANING stage documented
- [ ] Parent/child invariant stated
- [ ] UBCb compiler with persistence levels
- [ ] Computation-as-process model
- [ ] Analysis compressed to ~60 lines
- [ ] Checkpoint definitions unchanged
- [ ] Governing FOOPs table unchanged
- [ ] BDFL caveat preserved
- [ ] Last Updated entry added
