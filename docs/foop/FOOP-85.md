---
foop: D85
title: The einmo Foolish separator collides with Foolish block comments
author: Sisyphus / claude-opus-5 (directed by Atlas)
status: Draft
type: Standards
created: 2026-08-07
phase: meta
supersedes: []
begun: [ ]
---

# FOOP-85: The einmo Foolish separator collides with Foolish block comments

FOOP numbering is little-endian; the full rules live in `foop.md` at the
repository root — **read it before creating or editing a FOOP.** The one
template-specific note: the `foop:` front-matter field may either match the
filename digits directly, or give the big-endian decimal value preceded by
`D` (this file: `foop: D85` — digits `85` reversed = sort key 58). In all
cases, the `FOOP-85.md` file name is ultimately the right numbering.

## Abstract

The einmo Foolish-suite section separator is `"!!\n"` — `!!` being the
Foolish **line** comment. Foolish's **block** comment delimiter is `!!!`,
and every `!!!` line therefore *ends with* the separator. Because
`EinmoFile::serialize`'s collision check is a plain substring test, any
Foolish source containing a block comment is **unserializable**, which fails
the entire UBCa einmo suite rather than the one file. This FOOP changes the
separator to `"\n!!!EINMO!!!\n"` — newline-delimited on both sides, so it
matches only a whole line, never a suffix of one. The change is one
constant. It is **backward compatible**: each `.einmo` file records its own
separator in its header and `parse` reads it from there, so existing
baselines keep verifying against their original signatures.

## Motivation

Discovered while gating FOOP-75. `cargo test --workspace` on `jia@dc6db093`
failed three einmo tests:

```
test ubca_snapshot_tester::einmo_tests::einmo_gate_output   ... FAILED
test ubca_snapshot_tester::einmo_tests::einmo_gate_checked  ... FAILED
test ubca_snapshot_tester::einmo_tests::einmo_gate_verified ... FAILED

exercises/project_euler/1.foolish.einmo was not written+verified:
  write/serialize error: section "INPUT" contains the configured
  separator; configure a different one
```

The offending input opens with a Foolish block comment:

```foolish
!!!
# Multiples of 3 or 5
...
!!!
```

Raw bytes of line 1 are `! ! ! \n`, which contains `!!\n` at offset 1. The
`!!!` delimiter **ends with** the separator.

This is a **latent trap, not a one-file problem**. `!!!` is Foolish's block
comment (`lexer.rs:100-104` distinguishes it from `!!` by checking
`peek_at(2)`), so *any* `.foo` file using block comments is unserializable.
`exercises/project_euler/1.foolish` was simply the first in the suite to use
one — a survey found it was the only file in the corpus with a line ending
in `!!`.

The blast radius is the whole suite, not the file: `serialize()` returning
`Err` fails the gate, so one block comment anywhere takes down all 313
tests.

## Specification

### §1. The new separator

```rust
// einmo/src/format.rs
pub(crate) const FOOLISH_SEPARATOR: &str = "\n!!!EINMO!!!\n";
```

Previously `"!!\n"`.

### §2. Why the leading newline is the load-bearing part

The collision check (`format.rs:427-440`) is:

```rust
if section.body.contains(&self.separator) { /* refuse */ }
```

A plain substring test. With `"!!\n"`, a body line ending in `!!` matches.
Wrapping the token in newlines on **both** sides means the separator can
only match when `!!!EINMO!!!` occupies a line by itself — which converts the
substring test into an effective **whole-line** test without touching
signing-critical code.

### §3. Why this token

`!!!EINMO!!!` is itself a well-formed Foolish block-comment delimiter line,
so if it ever did appear in a `.foo` source it would be inert to the
Foolish lexer. But it is deliberately implausible in a real test — it names
the tool that owns it.

### §4. Backward compatibility

**Existing baselines are unaffected and need no re-signing.**

Each `.einmo` file records its separator in its own header
(`format.rs:373-379`, rendered by `escape_separator`), and `EinmoFile::parse`
reads the separator **from that header** (`format.rs:467-473`) rather than
from the ambient config. Existing files carry `separator=!!\n` and continue
to parse and verify against their original signatures.

The new value applies only to newly written files. Verified empirically:
after the change, only two suite entries differed, and **neither** was a
separator re-serialization (§Test Plan).

Note `escape_separator` (`format.rs:517`) replaces **all** `\n`, not merely
a trailing one, so the leading newline escapes correctly into the header
line and round-trips through `unescape_separator`.

## FIR Impact

None. This FOOP does not touch Foolish, the FVM, or FIR — only the einmo
snapshot envelope format's separator constant.

## UBC Step Impact

None.

## Test Plan

### Already verified (during discovery, on the working change)

- `cargo test -p einmo` — **133 passed, 0 failed.** The existing
  `roundtrip_foolish_separator` test (`format.rs:643`) references the
  constant symbolically, so it exercises the new value automatically.
- `cargo test -p foolish-ubca --lib -- einmo_gate_output` — **passes**
  (was failing). The `1.foolish` input now serializes.
- `cargo test --workspace` — went from **3 failures to 2**. The two
  remaining are unrelated to this FOOP:
  - `exercises/project_euler/1.py.einmo` missing from `checked/` — a
    Python reference input being fed to the Foolish evaluator (a suite
    hygiene issue; the human is moving non-`.foo` files out of the input
    tree).
  - `foop/62/infinite_loop.foo.einmo` — OUTPUT regressed
    `NK(ITERATION-EXCEEDED, Iteration exceeded 9999)` → `BRANING`.
    **Verified by stashing this change and re-running on clean `jia`: the
    regression is pre-existing** and belongs to FOOP-62.

### To add

- A unit test in `einmo/src/format.rs` pinning the class of bug: a section
  body containing a `!!!` block-comment line must serialize successfully
  under `FOOLISH_SEPARATOR`. This is the regression guard — without it, a
  future separator change could silently re-arm the trap.
- A unit test asserting a body containing a **standalone** `!!!EINMO!!!`
  line is still correctly refused (the collision check must still work).

## Rejected Alternatives

### A. Do nothing

Leave the separator at `"!!\n"`. Rejected: the entire UBCa einmo suite is
red, and any future `.foo` file using a block comment re-breaks it. This
was the state that blocked FOOP-75's Phase 0 gate.

### B. Rewrite the offending exercise's block comments as `!!` line comments

The minimal edit — change `!!!…!!!` to per-line `!!` in
`exercises/project_euler/1.foolish`. Rejected: it fixes one file and leaves
the trap armed for the next author who uses a block comment. Foolish
*has* block comments; the snapshot format should not forbid them.

### C. Anchor the collision check to whole lines in `serialize`

Change `body.contains(&separator)` to a line-wise match. This is the
conceptually correct fix — the separator *is* a line. Rejected **for now**:
`serialize()` is signing-critical code on the path that produces every
signed artifact, and changing it deserves its own review. §2's
newline-wrapping achieves the same effect through the constant, at zero
risk to the signing path. If C is ever done, this FOOP's separator remains
valid and the two are compatible.

### D. Change the separator to a non-Foolish glyph (revert toward `①`)

Use the default `①\n` for the Foolish suite too. Rejected: the suite chose
a Foolish comment deliberately, because `.foo` sources may legitimately
contain `①` (the reason `foolish_separator()` exists at all —
`ubca_snapshot_tester.rs:53-56`). Reverting reintroduces the collision it
was created to avoid, in the other direction.

## Open Questions

- **Should the collision check become line-anchored (Alternative C)?**
  Recommended as a follow-up, not required by this FOOP. It would make the
  separator's line-ness explicit in code rather than implicit in the
  constant's value.
- **Should non-`.foo` files be excluded from UBCa input discovery?**
  Raised during this investigation and **decided against** by the human:
  einmo's discovery stays extension-agnostic (its cross-language design,
  `stage.rs:99`), and the `einmo_suite/input/` tree is instead kept free of
  extraneous files. Recorded here because the alternative was prototyped
  and reverted; it is not part of this FOOP.

## References

- Code anchors: `einmo/src/format.rs:37` (the constant), `:427-440`
  (`serialize`'s collision check), `:373-379` (`header_line` /
  `escape_separator`), `:467-473` (`parse` reading the per-file separator),
  `:517-523` (escape/unescape), `:643` (`roundtrip_foolish_separator`);
  `einmo/src/config.rs:232` (`TestConfig::foolish_separator`);
  `foolish-ubca/src/ubca_snapshot_tester.rs:53-56` (the suite opting in);
  `foolish-parser/src/lexer.rs:100-104` (`!!!` vs `!!` block/line comment).
- Related FOOPs: FOOP-64 / FOOP-54 / FOOP-92 (einmo itself); FOOP-75
  (Assignment Attached Searches — blocked by this bug at its Phase 0 gate,
  which is how it was found); FOOP-55 (owns
  `exercises/project_euler/1.foolish`); FOOP-62 (owns the unrelated
  `infinite_loop` regression noted in the Test Plan).
- Docs: `AGENTS.md` §"Approval Tests (einmo)" and §"Non-regression
  invariant"; `rust_instructions.md` §"Phase-by-phase testing discipline".

## Appendix A — the change, as generated

This is the exact working-tree diff produced during discovery, preserved
here so the FOOP is self-contained and the change can be reapplied verbatim
if the working tree is reverted before the plan runs.

```diff
diff --git a/einmo/src/format.rs b/einmo/src/format.rs
index e6ca7864..c8273af5 100644
--- a/einmo/src/format.rs
+++ b/einmo/src/format.rs
@@ -34,8 +34,23 @@ const FORMAT_VERSION: u32 = 1;
 /// The default section separator: `①` (U+2460) followed by LF.
 pub(crate) const DEFAULT_SEPARATOR: &str = "①\n";
 
-/// The Foolish-suite separator: `!!` (a Foolish line comment) followed by LF.
-pub(crate) const FOOLISH_SEPARATOR: &str = "!!\n";
+/// The Foolish-suite separator: a standalone `!!!EINMO!!!` line.
+///
+/// Written as LF + `!!!EINMO!!!` + LF so it matches only a *whole line*,
+/// never a suffix of one.
+///
+/// The previous value was `"!!\n"` — `!!` being the Foolish line comment.
+/// That collided with Foolish's **block** comment `!!!`: because the
+/// collision check in [`EinmoFile::serialize`] is a plain substring test,
+/// any line ending in `!!` contains `"!!\n"`, and a `!!!` block-comment
+/// delimiter always does. The first suite input to use block comments
+/// (`exercises/project_euler/1.foolish`, lines 1 and 5) made the whole
+/// suite unserializable.
+///
+/// `!!!EINMO!!!` is itself a valid Foolish block-comment delimiter line, so
+/// it stays inert if it ever appears in a `.foo` source, but it is
+/// deliberately implausible in an actual test.
+pub(crate) const FOOLISH_SEPARATOR: &str = "\n!!!EINMO!!!\n";
 
 /// The metadata `status` field — whether the *harness* ran normally.
 ///
```

## Last Updated

**Date**: 2026-08-07
**Updated By**: Claude Code / claude-opus-5
**Changes**: Initial draft. Records the `"!!\n"` separator's collision with
Foolish's `!!!` block-comment delimiter (any `!!!` line ends with the
separator, and the collision check is a substring test), and the fix:
`"\n!!!EINMO!!!\n"`, newline-wrapped so it matches only whole lines.
Backward-compatible — each `.einmo` records its own separator in its header
and `parse` reads it from there. Appendix A preserves the generated diff so
the working tree can be reverted to a clean build before the plan runs.
