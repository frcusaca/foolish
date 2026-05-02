# Foolish Scala MVP — Phases Overview

> The MVP is built in five sequential phases. Each phase has its own document.
>
> The MVP is the entire effort. The "MVP" terminology is dropped — we now refer
> to the work as **Phase 1** through **Phase 5** (and a Phase 4.5 for deferred
> language features).

---

## Phase Roster

| Phase | Title | Document | Goal |
|-------|-------|----------|------|
| **Phase 1** | Compiler — Source to FIR JSON | [phase1_compiler.md](phase1_compiler.md) | Parse `.foo` source, translate to Foolish Internal Representation (FIR), serialize as JSON via Circe. No evaluation. |
| **Phase 2** | UBC — Step Evaluation | [phase2_ubc.md](phase2_ubc.md) | Read FIRs, step-evaluate until every node is `Constant` or `Constanic`. Approval tests live here. |
| **Phase 3** | CLI | [phase3_cli.md](phase3_cli.md) | Wire compile + step into a usable command-line tool. |
| **Phase 4** | Web Brane Browser | [phase4_browser.md](phase4_browser.md) | LOD viewer for browsing brane trees in a web UI. |
| **Phase 4.5** | Concatenation + deferred features | [phase4.5_concatenation.md](phase4.5_concatenation.md) | Add concatenation, forward search, and other features once the core rhythm is established. |
| **Phase 5** | Detachment, SF/SFF | [phase5_detachment.md](phase5_detachment.md) | The advanced language features. |

---

## Why this ordering

**Phase 1 first, no evaluation:** the contract between parser and evaluator is the
serialized FIR. By landing that contract early — and round-trip-testing it via Circe —
we eliminate an entire class of "did the parser produce what the evaluator expected?"
bugs. The compiler and the UBC can be developed independently as long as both honor
the JSON FIR format.

**Phase 2 is the hard part:** coordinating constanic values (the core UBC1 stuckness)
is isolated to one phase. By Phase 2 the AST→FIR pipeline is already proven correct,
so any approval-test failure is unambiguously an evaluation bug.

**Phase 3 (CLI) before Phase 4 (web UI):** a CLI that pipes source through compile +
step gives a daily-driver tool for testing language behavior, without the overhead
of a web stack. The web UI is a presentation layer over the same evaluator.

**Phase 4.5 / 5:** concatenation and detachment are intentionally deferred. They
interact with constanic semantics in subtle ways that benefit from a stable Phase 2
implementation as foundation.

---

## Language Scope by Phase

| Feature | Phase 1 (compile) | Phase 2 (evaluate) |
|---------|------------------|---------------------|
| Integer literals | `ConstantIntFir` (already `Constant`) | passes through |
| `???` literal | `NKFir` (already `NK`) | passes through |
| Brane `{...}` | `NormalBraneFir(Initialized)` | steps to `Constant`/`Constanic` |
| Identification `name = expr` | `StatementFir(Some(name), body)` | name lookup table |
| Arithmetic `+ - * / %` | `BinaryOpFir(Initialized)` tree, no compute | steps to `ConstantIntFir` if both sides Constant |
| Unary `-` | `UnaryOpFir(Initialized)` | steps to `ConstantIntFir` |
| Bare identifier | `SearchFir(pattern="^x$", Backward, anchored=false)` | walks IB then AB chain |
| `#-N` seek | `IndexFir(N, anchored=false)` | walks IB |
| Anchored `.`, `?` | `SearchFir(anchored=true, anchor=Some(...))` | local to anchor |
| `^` head, `$` tail | `HeadTailFir(anchored=true)` | first/last of anchor |
| Anchored `#N` index | `IndexFir(N, anchored=true)` | nth of anchor |
| Regex search | `SearchFir(pattern=..., ...)` | regex match |
| Comments | stripped at parse | n/a |
| Shebang | stripped at parse | n/a |
| Concatenation `A B` | **deferred to Phase 4.5** | — |
| `~` forward search | **deferred to Phase 4.5** | — |
| Detachment, SF/SFF | **deferred to Phase 5** | — |
| `if-then-else` | rejected at compile | — |

---

## Phase Lifecycle

For each phase:

1. **Read the phase document** (`phaseN_*.md`) end to end before writing code.
2. **Write tests first** at the layer the phase requires. Phase 1 is unit tests
   (Scala `case class` literals as expected values). Phase 2 is approval tests
   (`.foo` → `.approved.foo`).
3. **Implement** in `foolish-core-scala/src/main/scala/`.
4. **Commit per logical step** — one phase often takes 5–10 commits. Each commit's
   message names the phase and the step (e.g., "Phase 1: compile IntLitAstn to
   ConstantIntFir").
5. **Phase exit criteria:** all tests for that phase pass, the next phase's
   document has been read, and any open questions are listed in this overview.

---

## Three Test Layers (Phase 1 specifically)

Phase 1 has three independent test layers because three things can break independently:

| Layer | What it tests | File |
|-------|--------------|------|
| **AST** | `.foo` source → Scala AST node tree | `FoolishAstTest.scala` (unit tests, inline expected values) |
| **AST → FIR** | Scala AST → FIR tree (in memory) | `CompilerTest.scala` (unit tests, inline expected FIR values) |
| **FIR roundtrip** | FIR → JSON → FIR (in memory) | `FirRoundtripTest.scala` (unit tests) |

If a `.foo` file produces wrong output, you can tell from which layer is at fault by
looking at which test failed. Phase 1 has no `.foo`-driven approval tests — those
arrive in Phase 2.

---

## Last Updated

**Date**: 2026-05-01
**Updated By**: Claude Code 2.1.119 (Claude Code); Opus 4.7 (1M Context)
**Changes**: New phase-based structure replacing the MVP-numbered scheme. Phase 1
is now compile-only (FIR JSON output via Circe), Phase 2 is evaluation. Per-phase
documents introduced. Concatenation moved to Phase 4.5; detachment to Phase 5.
