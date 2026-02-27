# Foolish Documentation

> AI agents: read [DOC_AGENTS.md](DOC_AGENTS.md) before editing files in this directory.

Documentation is organized in purpose-driven subdirectories:

| Directory | Title | Description |
|-----------|-------|-------------|
| [howto/](howto/) | How to Express it in Foolish | Literate programming tutorials as `.foo` files. Cookbook style. |
| [why/](why/) | Philosophy of Foolish | Origins, inspirations, and design philosophy in English prose. |
| [how/](how/) | Engineering Documentation | Operational semantics, implementation details, and reference. |
| [todo/](todo/) | Project Documentation | Active project tracking, growth plans, and roadmap. |
| [vintage_legacy/](vintage_legacy/) | Legacy Documentation | Pre-reorganization files, being migrated into the above. |

See [docs/todo/0_documentation.md](todo/0_documentation.md) for the reorganization plan.

## Getting Started

Start with the tutorials in `howto/`:
1. [Chapter 1: The Basics](howto/01_howto_foolish.foo) — Branes, values, names, arithmetic, shadowing, nesting, scope, dot access, head/tail, index access, unanchored seek
2. [Chapter 2: Searches and Patterns](howto/02_howto_foolish_more.foo) — Localized search (?), forward search (~), regex patterns, anchored vs unanchored, shadowing resolution, constanic state
3. [Chapter 3: Topics To Be Written](howto/03_howto_foolish_todo.foo) — Concatenation, liberation, characterization, creation, equivalence, logic, differentiation

## Philosophy

- [Brainstorm](why/brainstorm.md) — Raw collection of motivating thoughts and design rationale
- [The Creation Postulate](why/creation_postulate.md) — The foundational axiom: you can always create something new

## Engineering Reference

- [UBC Engineering Reference](how/ubc_engineering.md) — Consolidated reference for the UBC1 implementation: FIR hierarchy, state machine, search system, context manipulation
- [UBC2 Design Specification](how/ubc2_design.md) — Next-generation brane computer design: message-passing architecture, FIR lifecycle stages (PREMBRYONIC → EMBRYONIC → BRANING → constanic)
- [Style Guide](how/foolish_style_guide.md) — Formatting, naming conventions, terminology, testing
- [Symbol Table](how/SYMBOL_TABLE.md) — Complete symbol reference

## Last Updated

Date: 2026-02-16
Updated By: Claude Code v1.0.0 / claude-opus-4-6
Changes: Added UBC engineering reference and UBC2 design specification links.
