# Foolish Documentation

> AI agents: read [DOC_AGENTS.md](DOC_AGENTS.md) before editing files in this directory.

Documentation is organized in purpose-driven subdirectories:

| Directory | Title | Description |
|-----------|-------|-------------|
| [ubc1/how/](ubc1/how/) | UBC1 Engineering Documentation | Operational semantics, implementation details, and reference. |
| [ubc1/todo/](ubc1/todo/) | UBC1 Project Documentation | Active project tracking, growth plans, and roadmap. |
| [ubc0_1/how/](ubc0_1/how/) | UBC0_1 Engineering Documentation | UBC0 semantics with UBC1 microstate definitions. |
| [ubc0_1/todo/](ubc0_1/todo/) | UBC0_1 Project Documentation | UBC0_1 tracking and development plans. |
| [howto/](howto/) | How to Express it in Foolish | Literate programming tutorials as `.foo` files. Cookbook style. |
| [why/](why/) | Philosophy of Foolish | Origins, inspirations, and design philosophy in English prose. |
| [vintage_legacy/](vintage_legacy/) | Legacy Documentation | Pre-reorganization files, being migrated into the above. |

See [ubc1/todo/0_documentation.md](ubc1/todo/0_documentation.md) for the reorganization plan.

## Getting Started

Start with the tutorials in `howto/`:
1. [Chapter 1: The Basics](howto/01_howto_foolish.foo) — Branes, values, names, arithmetic, shadowing, nesting, scope, dot access, head/tail, index access, unanchored seek
2. [Chapter 2: Searches and Patterns](howto/02_howto_foolish_more.foo) — Localized search (?), forward search (~), regex patterns, anchored vs unanchored, shadowing resolution, constanic state
3. [Chapter 3: Topics To Be Written](howto/03_howto_foolish_todo.foo) — Concatenation, liberation, characterization, creation, equivalence, logic, differentiation

## Philosophy

- [Brainstorm](why/brainstorm.md) — Raw collection of motivating thoughts and design rationale
- [The Creation Postulate](why/creation_postulate.md) — The foundational axiom: you can always create something new

## Engineering Reference

- [UBC Engineering Reference](ubc1/how/ubc_engineering.md) — Consolidated reference for the UBC1 implementation: FIR hierarchy, state machine, search system, context manipulation
- [UBC2 Design Specification](ubc1/how/ubc2_design.md) — Next-generation brane computer design: message-passing architecture, FIR lifecycle stages (PREMBRYONIC → EMBRYONIC → BRANING → constanic)
- [Style Guide](ubc1/how/foolish_style_guide.md) — Formatting, naming conventions, terminology, testing
- [Symbol Table](ubc1/how/SYMBOL_TABLE.md) — Complete symbol reference

## Last Updated

Date: 2026-03-07
Updated By: Claude Code / cyankiwi/Qwen3.5-27B-AWQ-BF16-INT8
Changes: Updated all path references to use semantic versioning: `how/` → `ubc1/how/`,
`todo/` → `ubc1/todo/`. Added ubc0_1 version directories.
Previous (2026-02-16): Added UBC engineering reference and UBC2 design specification links.
