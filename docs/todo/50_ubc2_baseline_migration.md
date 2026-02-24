# TODO 50: UBC2 Baseline Migration

> AI agents: read [../DOC_AGENTS.md](../DOC_AGENTS.md) before editing this file.

The UBC2 design introduces new output symbols and semantics that are incompatible with all
existing UBC1 approval test baselines. Every `.approved.foo` file must be regenerated before
UBC2 is considered complete. This document tracks what needs to change and why.

## Why All Tests Need Regeneration

UBC1 approval tests were written against UBC1 rendering conventions. UBC2 changes:

1. **Output symbols for constanic states** — the three-symbol system is new, and the
   assignment of `🧠?` vs `🧠??` to WOCONSTANIC vs CONSTANIC differs from any prior convention.
   Every test that previously rendered `⎵⎵` (CC_STR) for unresolved states will now render
   differently.

2. **NK vs CONSTANIC distinction** — UBC1 produced `🧠???` (NK) for many intra-brane search
   failures that UBC2 now correctly renders as `🧠??` (CONSTANIC). Any test where a search
   failed locally will likely change output.

3. **WOCONSTANIC is a new state** — UBC1 had no WOCONSTANIC. Searches that UBC1 rendered
   as CONSTANIC may now be WOCONSTANIC (`🧠?`) in UBC2.

## Output Symbol Changes

| Symbol | UBC2 State | Meaning |
|--------|-----------|---------|
| `🧠???` | NK | Definitively unfindable. The UBC has proven the search fails in all future contexts. Produced for arithmetic errors, depth exhaustion, and proven-impossible searches. Sequencer output only — not valid Foolish source. |
| `🧠??`  | CONSTANIC | Search performed, nothing found. May gain value when placed in a new context (e.g., via concatenation). Sequencer output only. |
| `🧠?`   | WOCONSTANIC | Found everything it was looking for, but one or more results are themselves constanic. Structurally complete, waiting on dependencies. Sequencer output only. |

The ordering is intuitive: more question marks = larger unknown. `🧠???` is the maximum —
definitively unknowable. `🧠??` is less certain — unknown in this context but potentially
findable. `🧠?` is the smallest — we know what it found, we're just waiting.

None of these symbols are valid Foolish source syntax. The 🧠 prefix is the denotational
marker — it indicates we are expressing the *meaning* of an expression, not its source form.

## Migration Tasks

- [ ] Run all UBC1 approval tests against UBC2 sequencer output
- [ ] Review each `.received.foo` diff carefully (do not force-approve)
- [ ] **Audit input files for invalid `???` usage** — scan all `.foo` test input files to
      confirm none use `???` as Foolish source syntax. It is not a valid input token.
      Any occurrence in a `.foo` input file is a bug in the test itself.
- [ ] For each test where output changed from `🧠???` to `🧠??`: verify that brane concatenation
      could plausibly provide the missing binding; if not, the NK ruling may have been correct
- [ ] For each test where output changed from `🧠???` to `🧠?`: verify the WOCONSTANIC reasoning
- [ ] Remove or rewrite detachment tests that encode incorrect permanent-blocking semantics
      (`detachmentBlockingIsApproved.foo` and related — see Open Issues item 16 in ubc2_design.md)
- [ ] Audit force-approved baselines from commit `0032e47` — these may be doubly wrong
      (wrong UBC1 semantics + now outdated symbols)
- [ ] After Java UBC2 passes all approval tests, begin Scala cross-validation

## Note on Output Symbols

All three state symbols (`🧠?`, `🧠??`, `🧠???`) are denotations — the sequencer renders them
to express the *meaning* of an expression that has not fully resolved. They are not Foolish
source syntax; the 🧠 prefix marks them as denotational.

A human writing an approval test can type these character sequences directly. But they will
never appear in Foolish program input files, because they are semantics, not syntax.

---

## Last Updated

**Date**: 2026-02-24
**Updated By**: Claude Code v1.0.0 / claude-sonnet-4-6
**Changes**: Renamed from `50_approval_test_migration.md`. Updated output symbols to use 🧠
prefix throughout. Removed erroneous claim that parser accepts `???` as source token (it does
not — all three symbols are sequencer output only). Added task to audit test input files for
invalid `???` usage.
