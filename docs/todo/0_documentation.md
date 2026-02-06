# Documentation Reorganization Plan

This document tracks the phased reorganization of Foolish documentation from the
legacy flat structure (now in `docs/old/`) into the new four-directory taxonomy.

## Directory Taxonomy

| Directory | Title | Content Style |
|-----------|-------|---------------|
| `docs/howto` | How to Express it in Foolish | Literate `.foo` files with documentation in comments |
| `docs/why` | Philosophy of Foolish | English prose on origins, inspirations, design philosophy |
| `docs/how` | Engineering Documentation | Operational semantics, implementation details, reference |
| `docs/todo` | Project Documentation | Active project tracking and growth plans |
| `docs/old` | Legacy Documentation | Pre-reorganization files, being migrated |

## Phase 1: Structure and Migration (COMPLETE)

- [x] Create directory structure: docs/howto, docs/why, docs/how, docs/todo, docs/old
- [x] Move all docs/ files to docs/old/
- [x] Move all projects/ files to docs/old/
- [x] Move SYMBOL_TABLE.md to docs/how/ as permanent reference
- [x] Update README.md links to point to docs/old/
- [x] Update AGENTS.md documentation section
- [x] Update CLAUDE.md documentation section
- [x] Create this plan document

## Phase 2: Populate `docs/how/` (Engineering Documentation)

Content: Operational semantics expressed in UBC steps. Denotational semantics forthcoming.
Implementation details.

Source material from `docs/old/`:
- ECOSYSTEM.md - UBC architecture, brane reference semantics, typing
- UBC_FEATURES.md - UBC feature documentation
- search-semantics.md - Search system operational semantics
- NAMES_SEARCHES_N_BOUNDS.md - Name resolution and search semantics
- RELATIONAL_COORDINATES.md - Coordinate system semantics
- ARITHMETIC_ERRORS.md - Error handling semantics
- EQUIVALENCE.md - Equivalence semantics
- DIFFERENTIATION.md - Differentiation semantics
- LOGIC.md - Logic system semantics
- LSP_WORK_DOC.md - LSP implementation engineering
- DEVELOPMENT.md / DEVELOPMENT_NOTES.md - Development engineering notes
- SYMBOL_TABLE.md - Already migrated

Source material from `docs/old/` (formerly projects/):
- 003-Detachment_Project.md - Detachment engineering
- 004-nyes-state-simplification.md - State simplification engineering
- 005-step-counting-refactoring.md - Step counting engineering
- 006-search-semantics-implementation-summary.md - Search implementation
- 007-unanchored-seek-implementation.md - Seek implementation
- 008-cmfir-nyes-state-review.md - CMFIR state review
- 009-Concatenation_Project.md - Concatenation engineering
- 010-AlarmCode_Engineering.md - Alarm code engineering

Tasks:
- [ ] Identify content that describes operational semantics and extract/reorganize
- [ ] Write new docs/how/ files with proper structure
- [ ] Update README.md links from docs/old/ to docs/how/ as content migrates

## Phase 3: Populate `docs/why/` (Philosophy of Foolish)

Content: English prose explaining how things came about, what the inspirations are.

Topics to cover:
- [ ] Openness - open replication, transparency
- [ ] Humility - learning from what works, taking success patterns from nature and biology
- [ ] Principles from nature:
  - The nature of our life is that source code is well protected by memBRANEs
  - Content can be expressed differently in different contexts, producing different proteins
    and different organisms
  - Foolish uses branes to contain information, permits completely open replication, and
    branes express differently depending on how they are coordinated
- [ ] Other programming languages as naturally occurring languages - their designs, features,
  and what we learn from them
- [ ] Origins story of Foolish

Source material from `docs/old/`:
- CREATION.md - Creation postulate, origin concepts
- DEVELOPMENT_NOTES.md - Foundation philosophy, historical context
- STYLES.md - Design philosophy, terminology
- SCHOLIA.md - Scholarly notes

## Phase 4: Populate `docs/howto/` (How to Express it in Foolish)

Content: Literate programming tutorials. Each tutorial is a Foolish `.foo` file where
documentation lives inside comments. Cookbook style with decreasing documentation density.
Eventually points to standard libraries with normal documentation for a Foolish-literate reader.

Structure:
- [ ] Introductory tutorials (branes, values, names) - heavily documented
- [ ] Intermediate tutorials (search, detachment, concatenation) - moderate documentation
- [ ] Advanced tutorials (recursion, patterns, composition) - light documentation
- [ ] Standard library pointers - minimal documentation, assumes fluency

Source material:
- README.md tutorial sections (Branes, Sizes, Comments, Expressions, Names, The Unknown)
- docs/old/ADVANCED_FEATURES.md code examples
- docs/old/NAMES_SEARCHES_N_BOUNDS.md code examples
- Existing test input .foo files for reference patterns

## Phase 5: Populate `docs/todo/` (Project Documentation)

Content: How we're growing Foolish. Active project tracking.

Source material from `docs/old/` (formerly projects/):
- 000-TODO_FEATURES.md - Feature TODO list and roadmap
- 001-GO_PLAN.md - Go implementation plan
- 002-todo.md - General TODO tracking
- 009-detachment-semantics-discussion-and-planning.md - Planning
- 007-foolish-indentation-style-guide.md - Style guide (may also fit docs/how/)

Tasks:
- [ ] Review and consolidate active TODO items
- [ ] Archive completed project items
- [ ] Create new project tracking documents for active work

## Phase 6: Rewrite README.md

Once the new documentation directories are populated, README.md should be rewritten as a
summary composed of the first chapters from each documentation area:
- Opening from `docs/why/` (philosophy, why Foolish exists)
- Quick start from `docs/howto/` (first tutorial excerpts)
- Architecture overview from `docs/how/` (engineering summary)
- Roadmap from `docs/todo/` (where we're going)

## Phase 7: Cleanup

- [ ] Verify all docs/old/ content has been migrated or explicitly archived
- [ ] Remove docs/old/ files that are fully superseded
- [ ] Final review of all cross-references
- [ ] Remove docs/old/ directory when empty

## Last Updated

**Date**: 2026-02-06
**Updated By**: Claude Code v1.0.0 / claude-opus-4-6
**Changes**: Initial creation of documentation reorganization plan.
