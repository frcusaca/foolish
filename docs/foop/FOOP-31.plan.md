# FOOP-31: SPA1 — UBC reference implementation (depth-first) — Implementation Plan

## Current State (2026-05-08)

- All 21 parser tests pass
- All 32 core tests pass (18 approval + 14 unit)
- CLI: `compile`, `run`, `step`, `repl` commands implemented
- FIR types: all 10 variants (ConstantInt, Nk, Operator, Search, Index, HeadTail, StayFoolish, StayFullyFoolish, Concatenation, NormalBrane)
- UBC evaluator: depth-first stepping, constanic clone, short-circuiting, search transparency
- JSON serialization/deserialization: complete with manual Serde impl
- FOOP-9 complete: BinaryOpFir/UnaryOpFir replaced with unified OperatorFir
- FOOP-12 complete: Alarm system with AlarmLevel, Alarm, AlarmSink, NKFir alarm integration

## Phase 1: Compiler (FOOP-9, FOOP-12)

- [x] Complete FOOP-9: OperatorFir — replace BinaryOpFir/UnaryOpFir with unified OperatorFir
  - [x](2026-05-08 22:31) Add `OperatorFir(op, operands: Vec<FirRef>)` to `fir.rs`
  - [x](2026-05-08 22:31) Remove `BinaryOpFir`, `UnaryOpFir` from `fir.rs`
  - [x](2026-05-08 22:31) Update `Steppable` trait — remove binary/unary specific accessors, add generic operand accessors
  - [x](2026-05-08 22:31) Update `Compiler.compileExpr` for binary/unary AST → OperatorFir
  - [x](2026-05-08 22:31) Update JSON serialization/deserialization in `fir.rs`
  - [x](2026-05-08 22:31) Update `ubc.rs`: `compute_binary`, `compute_unary` → generic `compute_operator`
  - [x](2026-05-08 22:31) Update approval tests, verify all pass
  - [x](2026-05-08 22:31) Add 5 unit tests (binary, unary, roundtrip, chained, search transparency regression)
- [x] Complete FOOP-12: Alarms — diagnostic levels
  - [x](2026-05-08 23:00) Write FOOP-12 spec document (FOOP012.md)
  - [x](2026-05-08 23:00) Add `AlarmLevel` enum (Info, Warn, Mild, Panic)
  - [x](2026-05-08 23:00) Add `Alarm` struct and `AlarmSink` trait
  - [x](2026-05-08 23:00) Add `alarm` field to `NkFir`
  - [x](2026-05-08 23:00) Thread `AlarmSink` through `Scope` (Rc<dyn AlarmSink>)
  - [x](2026-05-08 23:00) Implement evaluator alarm sources (DIV-BY-ZERO, UNBOUND-NAME, INVARIANT-VIOLATED)
  - [x](2026-05-08 23:00) Add 10 unit tests (level display, serialization, display, VecAlarmSink, NKFir roundtrip, div-by-zero, unknown literal, scope emit, scope without sink)
- [ ] Verify all Phase 1 tests pass

## Phase 2: UBC (FOOP-6, FOOP-7, FOOP-8, FOOP-10, FOOP-11)

- [x] Complete FOOP-8: FIRs are mutable; parent pointers are post-clone (already implemented — state is mutable field)
- [x] Complete FOOP-7: Constanic Clone contract (`constanic_clone()` implemented in `ubc.rs:434`)
- [x] Complete FOOP-10: Anchored search through constanic anchors (implemented in `SearchFir::step_anchored`)
- [x] Complete FOOP-11: Search stops at NK (NK propagation implemented in SearchFir step rules)
- [x] Complete FOOP-6: Depth-first evaluator (implemented in `run_to_completion_with_scope`)
- [ ] Run 60+ Phase 2 approval tests (currently 16 tests exist — need more test input files)

## Phase 3: Concatenation (FOOP-3)

- [x] Add ConcatenationFir variant (implemented in `fir.rs`)
- [x] Implement step_concatenation (merge brane, constanicClone, delegate)
- [x] Compiler accepts concatenation (implemented in parser)
- [ ] Add more concatenation-specific approval tests

## Phase 4: CLI

- [x] Implement `foolish compile` — outputs FIR JSON
- [x] Implement `foolish run` — evaluates and prints result
- [x] Implement `foolish step` — debug output showing parsed + result
- [x] Implement `foolish repl` — multiline input with brace-depth tracking
- [ ] Add CLI functional tests

## Final Verification

- [ ] Run complete test suite: all modules
- [ ] Verify SPA1 exit criteria: all governing FOOPs Final/Implementing, all tests pass
- [ ] Tag SPA1 milestone

## Worktree

- [ ] Create worktree at `${HOME}/tmp/foolish-worktrees/8172-foop-31` with branch `foop/31-ubc-spa1`
- [ ] Verify all work is complete in `${HOME}/tmp/foolish-worktrees/8172-foop-31` and committed to `foop/31-ubc-spa1`
- [ ] Merge `foop/31-ubc-spa1` to alpha
