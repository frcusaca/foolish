# FOOP-21: Alarms — diagnostic levels emitted by compiler and evaluator — Implementation Plan

## Alarm Infrastructure

- [x] Canceled. This feature should be later respecified and reimplemented.
      (2026-07-03 18:23)
- [-] Add `AlarmLevel` enum: `INFO`, `WARN`, `MILD`, `PANIC`
- [-] Add `Alarm` case class with `level`, `code`, `message`, `source`, `context` fields
- [-] Add `AlarmSink` trait with `emit`, `alarms`, `hasPanic`, `hasMildOrWorse`
- [-] Derive Circe codecs for `Alarm`

## FIR Impact

- [-] Add `alarm: Option[Alarm]` field to `NKFir` (default `None`)
- [-] Add `NKFir` alarm field roundtrip test

## Compiler Integration (Phase 1)

- [-] Thread `AlarmSink` through `Compiler.compileToJson(source, sink)`
- [-] Emit `FOOP-2-IF-REJECTED` (MILD) when source contains if-then-else
- [-] Emit `PARSE-ERROR` (MILD) for ANTLR parser errors
- [-] Emit `PARSER-INTERNAL` (PANIC) for malformed parse trees
- [-] Emit `UNSUPPORTED-CHARACTERIZATION` (WARN) for partially handled characterizations
- [-] Implement PANIC fail-fast: return immediately, partial FIR tree optional

## Evaluator Integration (Phase 2)

- [-] Thread `AlarmSink` through `Ubc.runToCompletion(fir, sink, maxDepth)`
- [-] Emit `DIV-BY-ZERO` (MILD) — attach alarm to produced `NKFir`
- [-] Emit `DEPTH-EXCEEDED` (MILD) when nesting > configured limit
- [-] Emit `STEP-LIMIT-EXCEEDED` (MILD) on step budget exhaustion
- [-] Emit `CONSTANIC-CLONE-INVARIANT` (PANIC) if clone called on nigh FIR
- [-] Emit `CIRCULAR-PARENT` (PANIC) if parent chain cycle detected

## Tests

- [-] Unit test: if-then-else emits MILD alarm, produces NK, no panic
- [-] Unit test: parse error emits MILD alarm
- [-] Unit test: division by zero emits MILD alarm
- [-] Unit test: depth limit exceeded emits DEPTH-EXCEEDED
- [-] Unit test: NKFir alarm field roundtrips

## Worktree

- [-] Create worktree at `${HOME}/tmp/foolish-worktrees/2845-foop-12` with branch `foop/12-alarms`
- [-] Verify all work is complete in `${HOME}/tmp/foolish-worktrees/2845-foop-12` and committed to `foop/12-alarms`
- [-] Merge `foop/12-alarms` to alpha
