# UBC1 - Unicellular Brane Computer (Message-Passing Infrastructure)

## Overview

UBC1 is the current development version of the Foolish Unicellular Brane Computer, implementing evaluation using a **message-passing infrastructure**. This represents the refined design with clarified microstates and a rigorous state machine model.

## Design Philosophy

UBC1 builds on the core Foolish concepts of:
- **Proximity creates combination** - adjacent branes combine through concatenation
- **Containment enables organization** - nested branes create scope hierarchies
- **Microstate-driven evaluation** - each evaluation step is a well-defined state transition

The message-passing approach treats evaluation as communication between brane components, where each microstate represents a message type and the evaluation engine routes messages according to the program structure.

## Directory Structure

```
ubc1/
├── how/          - Engineering documentation and implementation details
├── todo/         - Active project tracking and development roadmap
└── README.md     - This file
```

### `how/` - Engineering Documentation

- **ubc_engineering.md** - UBC implementation architecture and design
- **ubc2_design.md** - Detailed UBC2 design specifications
- **ubc2_message_protocol.md** - Message protocol specifications for the infrastructure
- **d0_fir_subtype_contracts.md** - FIR subtype system contracts
- **systems.md** - Systems overview and interactions
- **SYMBOL_TABLE.md** - Foolish symbols and Unicode mappings
- **working_doc_prompts.txt** - Working documentation and prompts

### `todo/` - Project Tracking

- **human_todo_index.md** - Human-readable task index
- **agents.status.md** - Agent collaboration status
- **0_documentation.md** - Documentation tasks
- **0.a.concatenation.woes.md** - Concatenation implementation challenges
- **50_ubc2_baseline_migration.md** - UBC2 migration tasks
- **999_rest_of_line_context.md** - Context handling tasks

## Relationship to Other Versions

| Version | Relationship |
|---------|--------------|
| **ubc0** (legacy) | Original implementation. UBC1 refines its concepts with clarified microstates. |
| **ubc0_1** | Reimplementation of ubc0 semantics using UBC1's microstate definitions. |

## Getting Started

1. Review **systems.md** for an overview of the UBC1 architecture
2. Study **ubc2_message_protocol.md** to understand the message-passing model
3. Consult **ubc_engineering.md** for implementation details

---

## Last Updated

**Date**: 2026-03-07
**Updated By**: Claude Code / cyankiwi/Qwen3.5-27B-AWQ-BF16-INT8
**Changes**: Created UBC1 directory structure by moving how/ and todo/ from docs/. Added this README explaining the message-passing infrastructure approach.
