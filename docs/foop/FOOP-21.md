---
foop: 12
title: Alarms — diagnostic levels emitted by compiler and evaluator
author: hc <hc.busy@gmail.com>
status: Deprecated
type: Standards
created: 2026-05-04
phase: phase-1
supersedes: []
---

# FOOP-21: Alarms — diagnostic levels emitted by compiler and evaluator

> **Status: Deprecated** (2026-07-03 18:23)
>
> Canceled as it stands. This feature should be later respecified and reimplemented.

## Abstract

Foolish has an **alarm system**: structured diagnostic messages emitted
by the compiler (Phase 1) and evaluator (Phase 2 onward) when they
encounter conditions worth surfacing to the user. Alarms have **levels**;
some halt the run, others are warnings.

The alarm system starts in Phase 1 (the compiler) so it's available to
all later phases by the time they need it. Compiler alarms cover parse
errors, unsupported language features, and other source-level issues.
Evaluator alarms cover runtime conditions like depth-limit exceeded,
useless detachment, division by zero (when not silently producing NK),
and PANIC-level invariant violations.

## Motivation

The current scala-mvp plan throws `RuntimeException` or returns NK when
something goes wrong, with no consistent reporting structure. This is
fine for prototyping but brittle:

- Some "errors" are recoverable (a deferred-feature compile error
  shouldn't crash a CLI session that was building toward a multi-line
  REPL command).
- Some are not recoverable (depth limit exceeded means the FIR tree is
  truncated in a way the user must know about).
- Some should be silent in normal operation but visible when a user
  asks (diagnostic mode).

UBC2 docs already mention alarms (`docs/ubc1/how/ubc2_design.md` §14:
"depth exhaustion should produce an alarm through the message channel
so the parent brane is aware evaluation was truncated"; `d0_4` §"Useless
Detachment"; `d0_2` PANIC for circular messages). FOOP-21 codifies
these into a uniform mechanism, starting at compile time.

A unified alarm system also makes the CLI (Phase 4) easier: one error
display path instead of separate exception handlers per call site.

## Specification

### Alarm levels

Four levels, ordered by severity:

| Level | Severity | Behavior |
|-------|----------|----------|
| `INFO` | Informational | Logged; never halts. Diagnostic output for users who ask. Examples: "compiler used default for X." |
| `WARN` | Warning | Logged; never halts. The result is still produced and may be correct. Examples: "deprecated syntax used"; "potentially unintended shadowing"; "useless detachment per d0_4." |
| `MILD` | Recoverable error | Logged; the affected FIR becomes NK with the alarm as its `reason`. The surrounding brane survives — other statements continue. Examples: depth-limit exceeded; division by zero (unless suppressed to silent NK); anchored search miss producing NK. |
| `PANIC` | Fatal | Halts the run immediately. The FIR tree is left in whatever state it was; the user gets the alarm and a stack trace if relevant. Examples: parser internal failure; circular parent chain detected; invariant violation in `constanicClone`. |

### Alarm structure

```scala
case class Alarm(
  level:    AlarmLevel,
  code:     String,         // stable identifier, e.g., "FOOP-2-IF-REJECTED" or "DEPTH-EXCEEDED"
  message:  String,         // human-readable description
  source:   Option[SourceLocation],  // file/line if known (parser provides this)
  context:  Map[String, String] = Map.empty   // optional structured details
)

enum AlarmLevel:
  case INFO, WARN, MILD, PANIC
```

The `code` field is a stable identifier callers can match on. Codes
follow the convention `<SUBSYSTEM>-<CONDITION>` for evaluator alarms
(e.g., `DEPTH-EXCEEDED`, `DIV-BY-ZERO`, `USELESS-DETACHMENT`) and
`<FOOP-N>-<CONDITION>` for compiler alarms tied to a specific FOOP
decision (e.g., `FOOP-2-IF-REJECTED`).

### Alarm sink

Alarms are collected in an `AlarmSink`:

```scala
trait AlarmSink:
  def emit(alarm: Alarm): Unit
  def alarms: List[Alarm]
  def hasPanic: Boolean = alarms.exists(_.level == AlarmLevel.PANIC)
  def hasMildOrWorse: Boolean = alarms.exists(a => a.level == AlarmLevel.MILD || a.level == AlarmLevel.PANIC)
```

The sink is threaded through compilation and evaluation. Phase 1's
`Compiler.compileToJson(source: String, sink: AlarmSink): String`
takes one. Phase 2's `Ubc.runToCompletion(fir: Fir, sink: AlarmSink):
Fir` takes one. Phase 4's CLI provides a sink that prints to stderr
(WARN+) and exits non-zero on PANIC.

### NKFir carries an optional Alarm

When a MILD alarm produces an NK FIR, the FIR carries the alarm as
diagnostic context:

```scala
case class NKFir(
  reason: String,
  alarm:  Option[Alarm] = None,
  state:  Nyes = Nyes.NK
) extends Fir
```

The sink still receives the alarm independently — the `alarm` field on
`NKFir` is for tree-traversal consumers (e.g., the sequencer rendering
`🧠???` with a tooltip) that want the diagnostic without re-querying the
sink.

### PANIC behavior

A PANIC alarm halts the run:
- In Phase 1: `Compiler.compileToJson` returns immediately (the partial
  FIR tree may be returned along with the alarm; the caller decides
  what to do).
- In Phase 2: `Ubc.runToCompletion` returns immediately. The FIR is left
  in whatever Nyes state it had at the moment of PANIC.
- In Phase 4 CLI: the CLI prints the alarm and exits with a non-zero
  status code.

PANIC is reserved for invariant violations and conditions where
continuing would produce nonsensical output. It is NOT for ordinary
errors like "missing identifier" (that's CONSTANIC, not even an alarm)
or "div by zero" (that's MILD-and-NK).

### Compiler alarm sources (Phase 1)

The compiler emits alarms for:

| Code | Level | Trigger |
|------|-------|---------|
| `FOOP-2-IF-REJECTED` | MILD | Source contains `if-then-else` |
| `FOOP-3-CONCATENATION-DEFERRED` | MILD | Source contains concatenation before Phase 3 ships |
| `FOOP-7-DEFERRED-FEATURE` | MILD | Source uses SF, SFF, detachment, ↑, or other Phase 7 features |
| `PARSE-ERROR` | MILD | ANTLR parser produced an error |
| `PARSER-INTERNAL` | PANIC | Parse tree is malformed in a way that breaks AST construction |
| `UNSUPPORTED-CHARACTERIZATION` | WARN | A characterization is used in a way the current implementation doesn't fully handle (defer-and-warn for forward compat) |

The compiler fails fast on PANIC and accumulates MILD/WARN/INFO into
the sink. A run with MILD alarms still produces a FIR tree (with NK
nodes for the rejected sub-expressions); the caller can choose to use
it (e.g., for partial syntax highlighting) or discard it.

### Evaluator alarm sources (Phase 2 onward)

| Code | Level | Trigger |
|------|-------|---------|
| `DEPTH-EXCEEDED` | MILD | Brane nesting depth > 96,485 (configurable). The exceeding FIR becomes NK. |
| `DIV-BY-ZERO` | MILD | Integer division/modulo by zero. The OperatorFir becomes NK. |
| `STEP-LIMIT-EXCEEDED` | MILD | A driver-defined step budget exceeded (prevents infinite loops in degenerate cases). |
| `CONSTANIC-CLONE-INVARIANT` | PANIC | `constanicClone` called on a nigh FIR (caller bug). |
| `CIRCULAR-PARENT` | PANIC | Parent chain has a cycle (caught by FOOP-8 invariant; should never happen). |
| `USELESS-DETACHMENT` | WARN | Phase 7 detachment is on a name already resolved (per d0_4 §"Useless Detachment"). |

### Configuration

Alarm levels are NOT runtime-configurable in MVP. The level for each
code is fixed by this FOOP. (Future FOOP can add `--strict` mode that
promotes WARN → MILD, etc.)

The depth limit is configurable: `Ubc.runToCompletion(fir, sink,
maxDepth = 96485)`. Default matches UBC2.

The alarm sink is per-run, not global. Each compile and each evaluation
gets its own sink; the CLI maintains a session sink that aggregates
across REPL lines.

### Output format

Alarms render to stderr in the CLI as:

```
[MILD] FOOP-2-IF-REJECTED at line 3:5: if-then-else has been removed (FOOP-2)
[WARN] USELESS-DETACHMENT at line 7:12: detachment of 'a' has no effect; 'a' was already resolved
```

JSON serialization for tooling:

```json
{
  "level": "MILD",
  "code":  "FOOP-2-IF-REJECTED",
  "message": "if-then-else has been removed (FOOP-2)",
  "source": {"line": 3, "column": 5},
  "context": {}
}
```

Circe codecs are derived for `Alarm`. Sinks may be JSON-serializable
for testing and replay.

## FIR Impact

`NKFir` gains an optional `alarm: Option[Alarm]` field. Backward
compatible; default `None`. Roundtrip test added.

No other FIR variants change.

## UBC Step Impact

`Ubc.step` and `Ubc.runToCompletion` accept an `AlarmSink`. Step rules
that produce NK due to a runtime condition (DIV-BY-ZERO,
DEPTH-EXCEEDED, STEP-LIMIT-EXCEEDED) emit a MILD alarm and produce an
`NKFir` with the alarm attached. Step rules that detect invariant
violations (CONSTANIC-CLONE-INVARIANT, CIRCULAR-PARENT) emit PANIC and
the driver halts.

The depth check happens in the brane-stepping code: each recursion
into a nested brane increments a depth counter; exceeding the configured
maximum emits DEPTH-EXCEEDED and returns NK.

## Test Plan

Phase 1 unit tests:

```scala
test("FOOP-21: if-then-else emits MILD alarm and produces NK") {
  val sink = new TestAlarmSink
  val fir  = Compiler.compileToJson("{ if 1 then 2 else 3 fi }", sink)
  sink.alarms.map(_.code) should contain ("FOOP-2-IF-REJECTED")
  sink.hasPanic shouldBe false
}

test("FOOP-21: parse error emits MILD alarm") {
  val sink = new TestAlarmSink
  Compiler.compileToJson("{ unclosed", sink)
  sink.alarms.map(_.code) should contain ("PARSE-ERROR")
}

test("FOOP-21: NKFir alarm field roundtrips") {
  val nk = NKFir(
    reason = "if-then-else removed",
    alarm  = Some(Alarm(AlarmLevel.MILD, "FOOP-2-IF-REJECTED",
                         "if-then-else has been removed",
                         Some(SourceLocation(3, 5))))
  )
  roundtrip(nk)
}
```

Phase 2 unit tests:

```scala
test("FOOP-21: division by zero emits MILD alarm and produces NK") {
  val sink = new TestAlarmSink
  val fir  = Ubc.runToCompletion(Compiler.compileSource("{x = 5 / 0}"), sink)
  sink.alarms.map(_.code) should contain ("DIV-BY-ZERO")
}

test("FOOP-21: depth limit produces DEPTH-EXCEEDED MILD alarm") {
  val sink = new TestAlarmSink
  val deeplyNested = generateNestedBrane(depth = 100000)
  Ubc.runToCompletion(deeplyNested, sink, maxDepth = 96485)
  sink.alarms.map(_.code) should contain ("DEPTH-EXCEEDED")
}
```

Phase 4 CLI tests verify the stderr output format and exit code on
PANIC.

## Rejected Alternatives

### A. Throw exceptions; let callers catch

The Java idiom. **Rejected**: forces every call site to know the
exception types; mixes "errors I should report" with "exceptions
indicating bugs." Alarms are structured data; exceptions are not.

### B. Embed alarms only on FIRs (no central sink)

Walk the FIR tree to find all alarms. **Rejected**: makes "did this run
have any alarms?" require a tree traversal. The sink is cheap and the
FIR-level alarm field complements it (one for tree consumers, one for
"did anything go wrong" queries).

### C. Defer alarm system to Phase 4 (CLI)

Wait until the CLI is being built to add alarms. **Rejected**: the
compiler (Phase 1) needs to report at least parse errors and feature
deferral cleanly. Without alarms, Phase 1 falls back to throwing
RuntimeException, which Phase 2/3 inherit. Easier to start with the
right structure.

### D. Use sl4j or another logging framework

**Rejected**: logging frameworks are about humans reading log lines.
Alarms are about structured data that the CLI displays, the test
harness asserts on, and the REPL re-presents. Different requirements.
A small dedicated alarm system is simpler.

### E. Configurable level promotion (--strict mode)

Allow users to promote WARN → MILD or MILD → PANIC. **Rejected for MVP**:
adds a knob users won't initially need. Can be added later via a new
FOOP if the language gets a serious user base.

## Open Questions

- **Should INFO alarms be emitted by default?** Probably yes for the
  CLI in verbose mode, suppressed otherwise. Implementation detail.
- **Does the parent FIR see child alarms automatically?** Per UBC2
  d0_2 the alarm "propagates through the message channel." Since we
  don't have message channels (FOOP-6 rejected message passing), the
  sink is the only propagation. This is fine — the sink is the single
  source of truth.
- **What happens to alarms during constanicClone of an NKFir?** NK is
  shared, not cloned (FOOP-7), so the alarm field is shared too. This
  is correct — the alarm refers to the original error site.

## References

- `docs/ubc1/how/ubc2_design.md` §11, §14: UBC2 alarm intent.
- `docs/ubc1/how/d0_4_detachment.md` §"Useless Detachment": the
  WARN-level alarm pattern.
- `docs/ubc1/how/d0_2_system_operator.md`: PANIC for circular messages.
- FOOP-7: `constanicClone` callable invariant — PANIC if violated.
- FOOP-8: parent chain invariant — PANIC if cycle detected.
- FOOP-11: `NKFir` is the carrier for MILD alarm context.
- `Fir.scala`: `NKFir` gets `alarm: Option[Alarm]` field.

---

## Last Updated

**Date**: 2026-07-04
**Updated By**: Claude Code 2.1.119 (Claude Code); Fable 5 (claude-fable-5)
**Changes**: Status -> Deprecated. Canceled as it stands per user request; feature should be later respecified and reimplemented. Added Deprecation Notice section.
