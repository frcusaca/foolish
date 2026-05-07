# FOOP-13: SPA1 — UBC reference implementation (depth-first) — Implementation Plan

## Phase 1: Compiler (FOOP-9, FOOP-12)

- [ ] Complete FOOP-9: OperatorFir — replace BinaryOpFir/UnaryOpFir with unified OperatorFir
  - [ ] Remove BinaryOpFir, UnaryOpFir from Fir.scala
  - [ ] Add OperatorFir(op, operands) with mutable state/parent
  - [ ] Update Compiler.compileExpr for BinaryExprAstn, UnaryExprAstn
  - [ ] Update Circe codecs, add roundtrip test
  - [ ] Implement shared Container trait for children-stepping
  - [ ] Implement OperatorFir.selfStep (scalar reduction)
- [ ] Complete FOOP-12: Alarms — diagnostic levels
  - [ ] Add AlarmLevel enum, Alarm case class, AlarmSink trait
  - [ ] Add alarm field to NKFir
  - [ ] Thread AlarmSink through Compiler.compileToJson
  - [ ] Implement compiler alarm sources (FOOP-2-IF-REJECTED, PARSE-ERROR, etc.)
  - [ ] Add alarm roundtrip test
- [ ] Verify all Phase 1 unit tests pass (AST, AST→FIR, roundtrip)

## Phase 2: UBC (FOOP-6, FOOP-7, FOOP-8, FOOP-10, FOOP-11)

- [ ] Complete FOOP-8: FIRs are mutable; parent pointers are post-clone
  - [ ] Convert Fir base trait to mutable state/parent
  - [ ] Implement structural equivalence for tests
  - [ ] Implement Circe codec that excludes parent
- [ ] Complete FOOP-7: Constanic Clone contract
  - [ ] Implement constanicClone() with per-state dispatch
  - [ ] Integrate into search step (always call constanicClone)
  - [ ] Unit tests for each Nyes state
- [ ] Complete FOOP-10: Anchored search through constanic anchors
  - [ ] Implement dereference() helper
  - [ ] Update anchored SearchFir step rule dispatch
- [ ] Complete FOOP-11: Search stops at NK
  - [ ] Update search FIR step rules for NK propagation
- [ ] Complete FOOP-6: Depth-first evaluator (assumed complete)
- [ ] Verify all 60+ Phase 2 approval tests pass

## Phase 3: Concatenation (FOOP-3)

- [ ] Complete FOOP-3: Concatenation algorithm
  - [ ] Add ConcatenationFir variant
  - [ ] Implement step_concatenation (merge brane, constanicClone, delegate)
  - [ ] Update compiler to accept concatenation (no longer reject)
  - [ ] Verify all concatenation tests pass

## Phase 4: CLI

- [ ] Implement foolish binary: compile, run, step, repl
- [ ] REPL: multiline input, brace-depth tracking, persistent session
- [ ] Verify CLI functional tests pass

## Alarm System Integration (FOOP-12 evaluator alarms)

- [ ] Thread AlarmSink through Ubc.runToCompletion
- [ ] Implement evaluator alarm sources (DIV-BY-ZERO, DEPTH-EXCEEDED, etc.)

## Final Verification

- [ ] Run complete test suite: Phase 1 + Phase 2 + Phase 3 + Phase 4
- [ ] Verify SPA1 exit criteria: all governing FOOPs Final/Implementing, all tests pass
- [ ] Tag SPA1 milestone

## Worktree

- [ ] Create worktree at `${HOME}/tmp/foolish-worktrees/8172-foop-13` with branch `foop/13-ubc-spa1`
- [ ] Verify all work is complete in `${HOME}/tmp/foolish-worktrees/8172-foop-13` and committed to `foop/13-ubc-spa1`
- [ ] Merge `foop/13-ubc-spa1` to alpha
