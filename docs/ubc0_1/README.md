# UBC0_1 - Reimplementation of UBC0 with Clarified Microstates

## Overview

UBC0_1 is a **reimplementation of the original UBC0 semantics** using the **clarified microstate definitions** developed for UBC1. This version serves as a bridge between the legacy UBC0 design and the refined UBC1 message-passing infrastructure.

## Purpose

UBC0_1 demonstrates that the original UBC0 design goals can be achieved with more rigorous state machine definitions. By combining:

1. **UBC0 semantics** - The original computational model and behavior expectations
2. **UBC1 microstates** - Clarified state definitions and transition rules

We can produce a more predictable and analyzable implementation of the original design.

## Directory Structure

```
ubc0_1/
├── how/          - Engineering documentation (to be populated)
├── todo/         - Project tracking and development tasks
│   └── 0_design_migration.md  - P0: Reverse-engineer UBC0_1 design using CMfir
└── README.md     - This file
```

## Development Goals

### Primary Task: FVM Respecification

The immediate next step for UBC0_1 is to **respecify the Foolish Virtual Machine (FVM) using CMfir with UBC1's microstates**. This involves:

1. Analyzing the original FVM semantics from vintage_legacy documentation
2. Mapping FVM operations to UBC1's clarified microstate model
3. Using CMfir (the refined FIR subtype system) as the foundation
4. Documenting the respecification in `todo/` and `how/`

### Expected Outcomes

- Clearer understanding of UBC0's computational model
- Validation that UBC1 microstates can express UBC0 semantics
- Foundation for potential UBC0 compatibility mode in UBC1
- Documentation of design evolution and lessons learned

## Relationship to Other Versions

| Version | Relationship to UBC0_1 |
|---------|------------------------|
| **ubc0** (legacy) | Source of semantics. UBC0_1 reimplements its behavior with clarified states. |
| **ubc1** | Source of microstate definitions. Provides the rigorous state model used in UBC0_1. |

## Getting Started

1. Review UBC0 documentation in `../vintage_legacy/` to understand original semantics
2. Study UBC1 microstate definitions in `../ubc1/how/`
3. Consult the task tracking in `todo/` for specific development items

---

## Last Updated

**Date**: 2026-03-07
**Updated By**: Claude Code / cyankiwi/Qwen3.5-27B-AWQ-BF16-INT8
**Changes**: Created `todo/0_design_migration.md` P0 task for reverse-engineering UBC0_1
design using CMfir from UBC1. Updated directory structure to reflect new task file.
Previous (2026-03-07): Created UBC0_1 directory structure with parallel how/ and todo/ directories.
Primary task is FVM respecification using CMfir with UBC1 microstates.
