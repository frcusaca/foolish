# Foolish Scala MVP — Phases Overview

> The MVP is built in seven sequential phases. Each phase has its own document.
>
> The MVP is the entire effort. The "MVP" terminology is dropped — we now refer
> to the work as **Phase 1** through **Phase 7**.

---

## Phase Roster

| Phase | Title | Document | Goal |
|-------|-------|----------|------|
| **Phase 1** | Compiler — Source to FIR JSON | [phase1_compiler.md](phase1_compiler.md) | Parse `.foo` source, translate to Foolish Internal Representation (FIR), serialize as JSON via Circe. No evaluation. |
| **Phase 2** | UBC — Depth-First Step Evaluation | [phase2_ubc.md](phase2_ubc.md) | Read FIRs, step-evaluate depth-first sequentially. Search short-circuiting and constanic cloning. Approval tests live here. |
| **Phase 3** | CLI | [phase3_cli.md](phase3_cli.md) | Wire compile + step into a usable command-line tool. |
| **Phase 4** | UBC — Breadth-First Evaluation | [phase4_ubc_breadth_first.md](phase4_ubc_breadth_first.md) | Re-implement UBC stepping in breadth-first order. Stack-safe, supports independent subtree progress. |
| **Phase 5** | Web Brane Browser | [phase5_browser.md](phase5_browser.md) | LOD viewer for browsing brane trees in a web UI. |
| **Phase 6** | Concatenation + deferred features | [phase6_concatenation.md](phase6_concatenation.md) | Add concatenation, forward search, and other features once the core rhythm is established. |
| **Phase 7** | Detachment, SF/SFF | [phase7_detachment.md](phase7_detachment.md) | The advanced language features. |

---

## Why this ordering

**Phase 1 first, no evaluation:** the contract between parser and evaluator is the
serialized FIR. By landing that contract early — and round-trip-testing it via Circe —
we eliminate an entire class of "did the parser produce what the evaluator expected?"
bugs. The compiler and the UBC can be developed independently as long as both honor
the JSON FIR format.

**Phase 2 is depth-first sequential:** coordinating constanic values (the core UBC1
stuckness) is hard. Phase 2 simplifies the implementation by guaranteeing that all
dependencies are evaluated before their dependents. This makes search short-circuiting
a synchronous in-step operation rather than a wake-up-queue mechanism. See FOOP-6.

**Phase 3 (CLI) before Phase 4 (breadth-first):** a CLI that pipes source through
compile + Phase-2 step gives a daily-driver tool for testing language behavior, without
having to solve breadth-first first. The CLI doesn't depend on breadth-first ordering.

**Phase 4 introduces breadth-first:** all previous Foolish designs assumed
breadth-first. Phase 4 reintroduces it on a stable foundation, with the open
design question of how to coordinate one Foolish-machine. See FOOP-6 and the Phase 4
TODO.

**Phase 5 (Web Browser) requires Phase 4:** the LOD viewer benefits from independent
subtree progress (Phase 4's main observable benefit) so multiple viewports can show
partial progress on different branes simultaneously.

**Phases 6 and 7:** concatenation and detachment are intentionally deferred. They
interact with constanic semantics in subtle ways that benefit from a stable
Phase 2 + Phase 4 foundation. Concatenation is the first feature where
`constanicClone` recoordinates across actual context changes (in earlier phases
recoordination is a no-op-equivalent because contexts don't change).

---

## Language Scope by Phase

| Feature | Phase 1 (compile) | Phase 2 (depth-first eval) |
|---------|------------------|---------------------------|
| Integer literals | `ConstantIntFir` (already `INDEPENDENT`) | passes through |
| `???` literal | `NKFir` (already `NK`) | passes through |
| Brane `{...}` | `NormalBraneFir(EMBRYONIC)` | steps to `CONSTANT` / `WOCONSTANIC` |
| Identification `name = expr` | `StatementFir(Some(name), body, EMBRYONIC)` | mirrors body's state |
| Arithmetic `+ - * / %` | `BinaryOpFir(EMBRYONIC)` tree, no compute | computes if both sides CONSTANT/INDEPENDENT |
| Unary `-` | `UnaryOpFir(EMBRYONIC)` | same |
| Bare identifier | `SearchFir(pattern="^x$", Backward, anchored=false, EMBRYONIC)` | walks IB then AB chain; ECONSTANIC if not found |
| `#-N` seek | `IndexFir(N, anchored=false)` | walks IB |
| Anchored `.`, `?` | `SearchFir(anchored=true, anchor=Some(...))` | local to anchor |
| `^` head, `$` tail | `HeadTailFir(anchored=true)` | first/last of anchor |
| Anchored `#N` index | `IndexFir(N, anchored=true)` | nth of anchor |
| Regex search | `SearchFir(pattern=..., ...)` | regex match |
| Comments | stripped at parse | n/a |
| Shebang | stripped at parse | n/a |
| Concatenation `A B` | **deferred to Phase 6** | — |
| `~` forward search | **deferred to Phase 6** | — |
| Detachment, SF/SFF | **deferred to Phase 7** | — |
| `if-then-else` | rejected at compile (FOOP-2) | — |

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
**Changes**: Inserted new Phase 4 (breadth-first UBC). Renumbered: web browser
4→5, concatenation 5→6, detachment 6→7. Phase 2 now explicitly named
"Depth-First Step Evaluation." Adopted UBC2 Nyes terminology in scope table.
