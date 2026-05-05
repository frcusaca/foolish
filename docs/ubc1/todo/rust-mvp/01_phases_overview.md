# Foolish Rust MVP — Phases Overview

> The MVP is built in seven sequential phases. Each phase has its own document.
>
> The MVP is the entire effort. The "MVP" terminology is dropped — we now refer
> to the work as **Phase 1** through **Phase 7**.

---

## Phase Roster

| Phase | Title | Document | Goal |
|-------|-------|----------|------|
| **Phase 1** | Compiler — Source to FIR JSON | [phase1_compiler.md](phase1_compiler.md) | Parse `.foo` source, translate to Foolish Internal Representation (FIR), serialize as JSON via serde. No evaluation. |
| **Phase 2** | UBC — Depth-First Step Evaluation | [phase2_ubc.md](phase2_ubc.md) | Read FIRs, step-evaluate depth-first sequentially. Search short-circuiting and constanic cloning. Approval tests live here. |
| **Phase 3** | Concatenation | [phase3_concatenation.md](phase3_concatenation.md) | Implement the `A B C ...` operator: produce a new merged brane of `constanicClone`'d copies; delegate further `step()`s to the merged brane. First real exercise of recoordination across context changes. |
| **Phase 4** | CLI | [phase4_cli.md](phase4_cli.md) | Wire compile + step + concatenation into a usable command-line tool with REPL. |
| **Phase 5** | UBC — Breadth-First Evaluation | [phase5_ubc_breadth_first.md](phase5_ubc_breadth_first.md) | Re-implement UBC stepping in breadth-first order. Stack-safe, supports independent subtree progress. |
| **Phase 6** | Web Brane Browser | [phase6_browser.md](phase6_browser.md) | LOD viewer for browsing brane trees in a web UI. Consumes Phase 5's breadth-first output. |
| **Phase 7** | Detachment, SF/SFF | [phase7_detachment.md](phase7_detachment.md) | The advanced language features. |

---

## Why this ordering

**Phase 1 first, no evaluation:** the contract between parser and evaluator is the
serialized FIR. By landing that contract early — and round-trip-testing it via serde —
we eliminate an entire class of "did the parser produce what the evaluator expected?"
bugs. The compiler and the UBC can be developed independently as long as both honor
the JSON FIR format.

**Phase 2 is depth-first sequential:** coordinating constanic values (the core UBC1
stuckness) is hard. Phase 2 simplifies the implementation by guaranteeing that all
dependencies are evaluated before their dependents. This makes search short-circuiting
a synchronous in-step operation rather than a wake-up-queue mechanism. See FOOP=6.

**Phase 3 is concatenation, before CLI:** concatenation is the first real exercise
of `constanicClone` (FOOP=7) across actual context changes. It belongs before the
CLI because the CLI is significantly more useful when users can compose Foolish
snippets. Promoting concatenation to Phase 3 (rather than deferring to Phase 6)
also catches recoordination bugs earlier when they're easier to find. See FOOP=3
(revised) for the algorithm.

**Phase 4 (CLI) builds on Phases 1–3:** a CLI that exposes compile + run + REPL.
The CLI doesn't depend on breadth-first ordering — depth-first is sufficient for
the daily-driver use case.

**Phase 5 introduces breadth-first:** all previous Foolish designs assumed
breadth-first. Phase 5 reintroduces it on a stable foundation, with the open
design question of how to coordinate one Foolish-machine. See FOOP=6 and the
Phase 5 TODO.

**Phase 6 (Web Browser) requires Phase 5:** the LOD viewer benefits from
independent subtree progress (Phase 5's main observable benefit) so multiple
viewports can show partial progress on different branes simultaneously.

**Phase 7 (Detachment, SF/SFF):** the most advanced features. They depend on
concatenation (Phase 3), breadth-first UBC (Phase 5), and the web browser
(Phase 6) being stable.

---

## Language Scope by Phase

| Feature | Phase 1 (compile) | Phase 2 (depth-first eval) | Phase 3 (concatenation) |
|---------|------------------|---------------------------|------------------------|
| Integer literals | `ConstantIntFir` (already `INDEPENDENT`) | passes through | shared by reference (no clone) |
| `???` literal | `NKFir` (already `NK`) | passes through | shared by reference |
| Brane `{...}` | `NormalBraneFir(EMBRYONIC)` | steps to `CONSTANT` / `WOCONSTANIC` | one of these per concatenation element |
| Identification `name = expr` | `StatementFir(Some(name), body, EMBRYONIC)` | mirrors body's state | cloned in merge per FOOP=7 |
| Arithmetic `+ - * / %` | `OperatorFir(EMBRYONIC)` per FOOP=9 (operator FIR with operand list, no search boundary) | computes if all operands CONSTANT/INDEPENDENT | as inside any expression |
| Unary `-` | `OperatorFir("-@unary", List(operand))` | same | same |
| Bare identifier | `SearchFir(pattern="^x$", Backward, anchored=false, EMBRYONIC)` | walks IB then AB chain; ECONSTANIC if not found | re-walked in merged context if cloned |
| `#-N` seek | `IndexFir(N, anchored=false)` | walks IB | re-walked in merged context |
| Anchored `.`, `?` | `SearchFir(anchored=true, anchor=Some(...))` | local to anchor (FOOP=10 rules) | as before |
| `^` head, `$` tail | `HeadTailFir(anchored=true)` | first/last of anchor | — |
| Anchored `#N` index | `IndexFir(N, anchored=true)` | nth of anchor | — |
| Regex search | `SearchFir(pattern=..., ...)` | regex match | — |
| Comments | stripped at parse | n/a | — |
| Shebang | stripped at parse | n/a | — |
| Concatenation `A B C` | rejected by Phase 1; **enabled in Phase 3** | — | `ConcatenationFir(elements)` per FOOP=3 |
| `~` forward search | rejected by Phase 1; deferred | — | — |
| Detachment, SF/SFF | rejected by Phase 1; **deferred to Phase 7** | — | — |
| `if-then-else` | rejected at compile (FOOP=2) | — | — |

---

## Phase Lifecycle

For each phase:

1. **Read the phase document** (`phaseN_*.md`) end to end before writing code.
2. **Write tests first** at the layer the phase requires. Phase 1 is unit tests
   (Rust struct literals as expected values). Phase 2 is approval tests
   (`.foo` → `.approved.foo`). Phase 3 reuses the approval test infrastructure
   for concatenation-specific tests.
3. **Implement** in the Rust crate (`foolish-core/src/`).
4. **Commit per logical step** — one phase often takes 5–10 commits. Each commit's
   message names the phase and the step (e.g., "Phase 1: compile IntLit to
   ConstantIntFir").
5. **Phase exit criteria:** all tests for that phase pass, the next phase's
   document has been read, and any open questions are listed in the FOOPs.

---

## Three Test Layers (Phase 1 specifically)

Phase 1 has three independent test layers because three things can break independently:

| Layer | What it tests | Module |
|-------|--------------|--------|
| **AST** | `.foo` source → Rust AST node tree | `tests::ast` (unit tests, inline expected values) |
| **AST → FIR** | Rust AST → FIR tree (in memory) | `tests::compiler` (unit tests, inline expected FIR values) |
| **FIR roundtrip** | FIR → JSON → FIR (in memory) | `tests::roundtrip` (unit tests) |

If a `.foo` file produces wrong output, you can tell from which layer is at fault by
looking at which test failed. Phase 1 has no `.foo`-driven approval tests — those
arrive in Phase 2.

---

## FOOPs Governing These Phases

Each phase is shaped by one or more FOOPs (Foolish Optimization Process documents
in `docs/foop/`):

| Phase | Governing FOOPs |
|-------|----------------|
| meta | FOOP=1 (the FOOP process itself) |
| Phase 1 | FOOP=2 (no if-then-else), FOOP=4 (search regex pattern), FOOP=5 (compile-time vs eval-time), FOOP=9 (operator FIR shape) |
| Phase 2 | FOOP=6 (depth-first), FOOP=7 (constanic clone contract), FOOP=8 (FIR mutability), FOOP=10 (anchored search rules), FOOP=11 (search stops at NK) |
| Phase 3 | FOOP=3 (concatenation algorithm) |
| Phase 4 | (no governing FOOP — implementation only) |
| Phase 5 | FOOP=6 also covers Phase 5 (it's the deferred-from-Phase-2 work) |
| Phase 6 | (no governing FOOP — application layer) |
| Phase 7 | (FOOPs to be written when Phase 7 design begins) |

---

## Last Updated

**Date**: 2026-05-05
**Updated By**: Claude Code; Qwen3.6-27B-AWQ-BF16-INT4
**Changes**: Initial creation — translated Scala MVP phases overview to Rust. Replaced
Circe with serde, ScalaTest with cargo test, scoop with clap, http4s with axum.
Adapted module structure and test layer descriptions for Rust conventions.
