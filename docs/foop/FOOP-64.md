---
foop: 46
title: Migrate UBCa snapshot tests to a hierarchical einmo suite
author: Atlas <hc.busy@gmail.com>
status: Draft
type: Standards
created: 2026-07-14
phase: meta
supersedes: []
begun: [ ]
---

# FOOP-64: Migrate UBCa snapshot tests to a hierarchical einmo suite

FOOP numbering is little-endian; the full rules live in `foop.md` at the
repository root — **read it before creating or editing a FOOP.**

> **TRACK 0 — REQUIRED BEFORE ANYTHING ELSE (Atlas, 2026-07-14).** The insta `approval_all`
> gate is structurally red: generation embeds a wall-clock `generated:` timestamp inside the
> signed, byte-compared content, so a fresh run can never byte-match the stored corpus (all 161
> snaps diverge signature-only; content byte-identical; signing-key drift `eb9604b1…` →
> `dc5f586c…`). Einmo's `compare` checks INPUT/OUTPUT sections only — STAMPS and metadata are
> excluded — so migrating to einmo is the structural fix. **No other FOOP starts until this one
> is merged and the einmo gate is green.**
>
> **Absorbed from FOOP-92 (marked Complete 2026-07-14):** Phase 11 (gate shell glue /
> `einmo compare --require-match` pre-commit + CI wiring) and Phase 12 (`einmo console-review` —
> the vimdiff-based review loop with `--reexamine-rate` sampling and `@agent` handling), both of
> which this migration needs operationally: the initial promotion of ~162 migrated outputs and
> every review after it go through these tools. FOOP-92's serve/SPA, MCP server, algorithm
> corpus, and use-case validation are NOT absorbed — deferred to future FOOP(s).

## Abstract

Migrate the 162 flat insta snapshot tests under `foolish-ubca/snapshot_tests/` into a new
**einmo suite** at `foolish-ubca/einmo_suite/`, organized as a meaningful directory hierarchy
(`foop/<NUMBER>/…`, `lang/<category>/…`, `lang/usecases/…`, `regression/…`) and stored in the
signed `.einmo` envelope format (FOOP-92). This is the inverse direction from zweimomo: there the
Foolish FVM was an evaluator used to test einmo; here einmo is the harness used to test the
Foolish UBCa FVM. The FOOP also proposes nine new feature-combination tests (§Proposed new
combination tests), re-homes the reserved comprehensive-test path for all future FOOPs,
installs the **two-tier signing gate** (§Two-tier signing gate: development requires the
**checked** stage; PR merge requires the **verified** stage — same suite directory, different
requirements, different public keys), and updates AGENTS.md / foop.md / the skills to mandate
the checked-stage gate. **This FOOP is not complete until the repository has securely migrated
off insta snapshots entirely — `insta` removed from every crate's dependencies and the
workspace** (`foolish-ubca`, `foolish-parser`, `foolish-core`, root `Cargo.toml`).

## Motivation

Today `foolish-ubca/snapshot_tests/input/` is a single flat directory of 162 `.foo` files whose
only organization is name prefixes (`alarm_*`, `sff_*`, `foop_23_*`, `regression_*`, …). The flat
namespace hides three distinct kinds of test:

1. **FOOP development tests** — born during a specific FOOP's implementation (comprehensives and
   feature probes). Their home should say which FOOP owns them.
2. **Language tests** — durable documentation of a feature area or a realistic usage demo.
3. **Regression tests** — pinned bug reproductions that must never regress.

The `.snap` format is unsigned-per-stage insta output with a bespoke Ed25519 wrapper bolted on;
FOOP-92 built einmo precisely to replace that arrangement with a staged, signed promotion pipeline
(`output/` → `checked/` → `verified/`) whose stage directories **mirror the input tree at any
depth** — which is exactly the capability a hierarchy needs. Einmo is merged and green on `jia`;
zweimomo proved the `UbcaEvaluatorAdapter` shape. The remaining step is to point einmo at the UBCa
FVM's own approval corpus.

## Specification

### Suite layout

```
foolish-ubca/einmo_suite/
├── einmo.toml                  # suite config: signing (computer key for output/checked)
├── MAPPING.md                  # generated: full old-path → new-path table, incl. dual homes
├── input/
│   ├── foop/
│   │   ├── 9/                  # foop9_* probes (FOOP-9)
│   │   ├── 13/comprehensive.foo
│   │   ├── 23/comprehensive.foo
│   │   ├── 42/comprehensive.foo
│   │   └── 64/                 # this FOOP's new tests + comprehensive.foo
│   ├── lang/
│   │   ├── basics/  operators/  alarms/  branes/  concat/  sf_sff/  sequencer/
│   │   ├── searches/
│   │   │   └── anchored/  contexted/  value/  regex/  seeks/  head_tail/
│   │   └── usecases/
│   └── regression/
├── output/                     # generated, signed (computer key)
├── checked/                    # reviewed baseline (committed)
├── flagged/                    # set-aside sink
└── verified/                   # human-signed (passphrase; never the computer key)
```

Directory names under `foop/` use the **filename digits** (the little-endian identifier):
`foop/13/` is FOOP-13, *not* the FOOP whose sort key is 13 (that would be FOOP-31).

### Placement rules

Inputs remain `.foo` files; einmo writes `<relative-path>.foo.einmo` into each stage directory.

| Rule | Old name (pattern) | New home |
|------|--------------------|----------|
| R1 | `foop_<N>_comprehensive.foo` | `foop/<N>/comprehensive.foo` |
| R2 | `foop<N>_<rest>.foo` (non-regression) | `foop/<N>/<rest>.foo` |
| R3 | `regression_<rest>.foo`, `foop<N>_*_regression.foo` | `regression/…` (keep `foop<N>` in the stem for provenance) |
| R4 | `complex_*.foo`, `infinite_loop.foo` | `lang/usecases/` |
| R5 | `alarm_*`, `zero_division`, `division_by_zero_in_nested_brane`, `operator_chain_with_division_by_zero` | `lang/alarms/` |
| R6 | `concatenation_*`, `concat_*`, `multiple_concatenation_in_sequence` | `lang/concat/` |
| R7 | `sf_*`, `sff_*`, `sf_sff_*` | `lang/sf_sff/` |
| R8 | `sequencer_*`, `hfs_*`, `hs_*` | `lang/sequencer/` |
| R9 | search family by operator: `anchored_search_*`/`search_*`/`level_skipping_*`/`nested_search_in_brane`/`assignment_anchor_search`/`contextless_deepening_chain` → `searches/anchored/`; `contexted_*` → `searches/contexted/`; `value_search_*`/`name_value_atomic` → `searches/value/`; `regex_search_*`/`simple_regex_search` → `searches/regex/`; `*seek*`/`offset_access_*` → `searches/seeks/`; `head_tail_*`/`brane_with_single_value_head_tail` → `searches/head_tail/` |
| R10 | `operator_*` (remaining) | `lang/operators/` |
| R11 | brane/scope/shadowing/forward-ref stems | `lang/branes/` |
| R12 | everything remaining (simple arithmetic, literals, identifiers, unicode) | `lang/basics/` |

Rules apply top to bottom; the first match wins. When the directory now carries the category, the
redundant stem prefix is stripped (`alarm_division_by_zero_in_brane.foo` →
`lang/alarms/division_by_zero_in_brane.foo`); the stem is kept whole when stripping would damage
meaning. The executed outcome for all 162 files is recorded in `einmo_suite/MAPPING.md`.

### Dual-home rule

When two tests are largely identical, that is often because they belong to two places — once as a
FOOP's feature probe (`foop/<N>/feature_testing.foo`) and once as living language documentation
(`lang/usecases/demonstrate_concat_in_recursive_call.foo`). Such inputs are **fully copied into
each home, not deduplicated**; each copy is signed and promoted independently. `MAPPING.md` records
dual-home pairs. Within a *single* directory, near-identical variants instead use einmo's
dependent naming (`base++variant.foo`, diffed via the DIFF section).

An attribution pass (git history of each input) may add `foop/<N>/` dual copies for tests that
were clearly born inside a FOOP but carry no `foop` prefix (e.g. the `operator_transparency_*`
trio from FOOP-9 or `concat_brane_*` from FOOP-13).

### Harness

- `foolish-ubca` gains a dev-dependency on `einmo` (einmo has zero Foolish dependencies, so no
  cycle) and a copy of zweimomo's small `UbcaEvaluatorAdapter` (`UbcaEvaluator` +
  `clone_steppable` + `FirSequencer::format`, one OUTPUT chunk per top-level statement).
- New test `einmo_approval_all` in `foolish-ubca/src/ubca_snapshot_tester.rs`, per the
  dev-compliance pattern in `einmo.README.md`: `TestConfig::new(einmo_suite).foolish_separator()
  .require_correspondence(Stage::Output, Stage::Checked)`.
- Promotion `output->checked` uses the einmo CLI with the computer key (the AI-permitted stage).
  `checked->verified` remains a human act with a real passphrase, exactly as in FOOP-92.
- The existing insta `approval_all` and `snapshot_tests/` stay untouched during the migration
  (agents must never move or alter `.snap` files) — but their retirement is **in scope and
  completion-blocking**: this FOOP ends only when the `.snap` corpus is removed (human act),
  the insta tests are deleted, and `insta` is out of every `Cargo.toml`. `foolish-parser` and
  `foolish-core` insta snapshot tests are inventoried and migrated to their own einmo suites as
  part of the same sweep (`cargo tree -i insta` must find nothing at completion).

### Cross-validation

A temporary `#[ignore]`d test `cross_validate_einmo_vs_insta` reads each migrated input's OUTPUT
section from `einmo_suite/checked/` and byte-compares it against the RESULT section of the
corresponding approved `.snap` (read-only). This proves the migration transported behavior, not
just files. The test is deleted when the human retires `snapshot_tests/`.

### Two-tier signing gate: checked for development, verified for merge

Both tiers run against the **same suite directories** — they differ in the required
correspondence stage and in **which public keys they accept**:

| Tier | Name | When | Requirement | Key check |
|------|------|------|-------------|-----------|
| **Development** | the **feature-complete test suite** | every `cargo test`, every commit | output ↔ **checked** correspondence (`einmo_approval_all`: `require_correspondence(Output, Checked)`) | `checked/` stamps may be the well-known computer key — AI promotion `output->checked` is the einmo design |
| **Merge (PR)** | the **merge-ready test suite** | GitHub PR gate | output ↔ **verified** correspondence (`einmo compare output verified --require-match`) | every `verified/` file carries a `stage:verified` stamp under the **human reviewer's public key** (`confirm-signatures verified <human-key-prefix> --require-all`), and **zero** files carry the computer key on that stamp (AI-bypass scan per the einmo draconian-gate pattern) |

Both tiers cover **all migrated suites** — `foolish-ubca/einmo_suite/` and
`foolish-core/einmo_suite/` (below) — so "feature-complete" and "merge-ready" are properties of
the whole repository, not of one crate.

**Gate failure semantics — verified empirically 2026-07-14 (sanity check, do not assume):**

| Condition | Behavior | Verdict |
|---|---|---|
| `output/` has files, `checked/` missing them | `compare --require-match` → `1 only-in-output`, burden message, **exit 1** | correct — a demanded checked file that does not exist **fails** |
| `output/` has files, `verified/` missing them | same, **exit 1** | correct |
| library `require_correspondence(Output, Checked)` with empty `checked/` | `only_in_a` → `is_clean()` false → `correspondence_failures` non-empty → `all_output_written_and_verified()` false (pinned by einmo's own `correspondence_failure_reported_until_promoted` test) | correct |
| **empty suite** (no inputs / no stage files at all) | `compare --require-match` → **exit 0** | ⚠️ **vacuous pass** |
| **`confirm-signatures --require-all` on an empty `verified/`** | "0 file(s) match, 0 do not" → **exit 0** | ⚠️ **vacuous pass** |

The two vacuous passes are edge holes einmo's CLI leaves open (they are technically correct —
"all zero files comply" — but useless as gates). **Both gate tests MUST therefore assert
non-emptiness explicitly**, per the einmo draconian-gate pattern:

- `einmo_approval_all`: `assert!(!results.files.is_empty())` — a suite that discovered no inputs
  is a failure, not a pass.
- `einmo_verified_gate`: assert the `verified/` walk is non-empty **and** that its file count
  equals the `output/` count, *before* asserting key coverage — otherwise a never-populated
  `verified/` sails through `--require-all`.

Mechanics:

- The merge tier is a second test (`einmo_verified_gate`), `#[ignore]`d in local runs and
  executed by `.github/workflows/einmo-gates.yml` on pull requests. Making that workflow a
  **required status check** is a GitHub branch-protection setting — a human act (agents do not
  change repository settings); the plan carries it as a human checkbox.
- The human reviewer's public key (hex prefix) is embedded as a constant in the gate test /
  workflow; the human derives and supplies it once (their passphrase never appears anywhere).
- Keeping the tiers on one suite directory means the *content* under test is identical — only
  the attestation bar rises from "a reviewer promoted this" to "a human signed this."

### foolish-core migration (as part of this FOOP)

`foolish-core/snapshot_tests/` migrates to a corresponding **`foolish-core/einmo_suite/`** with
the **same organizational requirements** as the UBCa suite: the `foop/<NUMBER>/…`,
`lang/<category>/…`, `lang/usecases/…`, `regression/…` hierarchy, the same placement rules
R1–R12, the same dual-home rule, and its own `MAPPING.md`. Both tiers include it: the
feature-complete suite requires its output↔checked correspondence; the merge-ready suite its
output↔verified correspondence under the human key. One inventory question is resolved during
execution: which evaluator drives foolish-core's inputs post-FOOP-62 (UBC was retired; the
corpus may be partially stale) — stale inputs are flagged to the human via `einmo flag` with
reasons rather than silently dropped. `foolish-parser`'s insta usage (inline snapshot
assertions rather than a directory corpus) is inventoried in the retirement sweep and migrated
to whatever einmo shape fits (a small suite or unit-test rewrites), so the insta removal
criterion holds workspace-wide.

### Documentation, skills, and process-gate updates

AGENTS.md, `foop.md`, and the three skills (`foop-write-plan`, `foop-use-maintain`,
`foolish-debugging`) are updated inside this FOOP's worktree to make the einmo gate the
process law:

- **AGENTS.md**: the Development Rules' "never start work when tests are broken" is restated
  as **the codebase must pass the einmo checked stage** (output matches signed `checked/`
  einmos); the Build Commands section replaces the insta workflow with the einmo flow
  (`einmo_approval_all`, `einmo compare/promote/verify`); the ⚠️ CRITICAL snapshot section is
  rewritten in einmo terms — AI may promote `output->checked` after review, AI must **never**
  produce a `verified` stamp (the empty-passphrase key on `stage:verified` is the detectable
  bypass), and the insta-specific prohibitions (`cargo insta accept`, `.snap` handling) are
  retired with insta itself.
- **`foop.md`**: comprehensive-test path re-homing (below) plus the requirement that a FOOP's
  merge criteria include the einmo checked-stage gate.
- **Skills**: every insta/`.snap` reference becomes the einmo equivalent (generate → review →
  promote flow, `console-review`, the two-tier gate).

### Comprehensive-test path re-homing

From this FOOP forward, the reserved comprehensive input for FOOP `<N>` lives at
`foolish-ubca/einmo_suite/input/foop/<N>/comprehensive.foo` (was
`foolish-ubca/snapshot_tests/input/foop_<N>_comprehensive.foo`). `foop.md` and `AGENTS.md` are
updated inside this FOOP's worktree, so the rule and the tree change land together. FOOP-64's own
comprehensive is the first test born at the new path.

## Proposed new combination tests

New tests authored by this FOOP live in `foop/64/`; those that read as demonstrations get a
dual-home copy in `lang/usecases/`. Each targets a feature *pair/triple* the current corpus
misses:

| Test | Combination | What it pins |
|------|-------------|--------------|
| `foop/64/sff_alarm_per_use.foo` | SFF × alarms | Division by zero inside an SFF body raises at each **use** site, not at definition; alarm count matches use count. |
| `foop/64/contexted_after_concat.foo` | `&`-search × concat | A contexted search anchored on a result found *through* a concatenation navigates the concatenated home brane correctly. |
| `foop/64/value_search_shadowed_concat.foo` | value search × concat × shadowing | Atomic `?name=value` selects the right candidate among shadowed duplicates across a concatenation seam. |
| `foop/64/recoordination_sff_payoff.foo` | recoordination × SFF × unanchored | An unanchored miss (ECONSTANIC) inside a brane gains a value when the brane is referenced in a new context; SFF re-resolves per use. |
| `foop/64/regex_unicode_names.foo` | regex × unicode identifiers | Regex search over mixed-script names (Greek/Cyrillic/Chinese); corpus has unicode names and regex separately, never together. |
| `foop/64/head_tail_empty_concat_chain.foo` | head/tail × concat × contexted | `^`/`$` and `&^`/`&$` on a concatenation of empty and non-empty branes. |
| `foop/64/seek_boundary_alarm.foo` | seeks × nesting × alarms | `#N` / seek out of bounds across a nested-brane boundary where the neighbor statement carries an alarm. |
| `foop/64/deep_forward_ref_sf_gate.foo` | forward `~` × SF timing | A forward reference into a brane whose SF wrapper blocked at assignment time. |
| `foop/64/comprehensive.foo` | all of the above | FOOP-64's reserved comprehensive; also dogfoods the new path rule. |

## FIR Impact

None.

## UBC Step Impact

None. This FOOP is test infrastructure only; every migrated input must produce byte-identical
evaluator output (enforced by §Cross-validation).

## Test Plan

- `einmo_approval_all` (new): every input under `einmo_suite/input/` written+verified to
  `output/` and matching `checked/` — the development tier.
- `einmo_verified_gate` (new, `#[ignore]` locally, CI on PRs): output ↔ `verified/`
  correspondence + human-key `confirm-signatures --require-all` + zero-computer-key scan on
  `stage:verified` stamps — the merge tier.
- `cross_validate_einmo_vs_insta` (new, `#[ignore]`, temporary): migrated outputs byte-match the
  approved `.snap` RESULTs.
- Existing insta `approval_all` keeps passing untouched until human retirement.
- The nine new `.foo` inputs of §Proposed new combination tests go through the normal einmo
  review: agent generates `output/`, promotes to `checked/` after self-review; human reviews the
  diff and may sign `verified/`.

## Rejected Alternatives

### A. Flat `einmo_suite/input/` (no hierarchy)
Loses the entire organizational payoff; einmo's stage mirroring of nested trees is the feature
this migration exists to use.

### B. Convert `.snap` files in place to `.einmo`
Agents may not move or alter `.snap` files, and insta signatures cannot be transplanted into an
einmo stamp chain anyway. Fresh generation + promotion is the einmo-sanctioned path; §Cross-
validation supplies the equivalence proof instead.

### C. Move inputs and delete `snapshot_tests/` immediately
Breaks the "never work with broken tests" rule during the transition and requires agent action on
protected files. Copy now, human-gated retirement later.

### D. Do nothing
The flat corpus keeps growing (162 and counting), FOOP ownership of tests stays invisible, and the
unsigned-stage insta pipeline FOOP-92 was built to replace stays load-bearing.

## Open Questions

- Should the `verified/` stage be populated for the whole migrated corpus at merge time (one human
  signing session), or lazily per release as the draconian gate demands?
- Exact home for a handful of ambiguous stems (e.g. `search_through_concatenation`,
  `seek_in_nested_result_after_concatenation`) — first-match rule places them; the attribution
  pass may add dual copies. Resolve during MAPPING.md review.
- Merge-tier key: a single human reviewer key, or a small set of accepted reviewer keys
  (workflow constant becomes a list)?
- ~~`foolish-parser` / `foolish-core` insta inventories: separate suites or shared?~~ Resolved
  (Atlas 2026-07-14): **per-crate corresponding suites** — `foolish-core/einmo_suite/` with the
  same organizational requirements, included in both tiers; parser handled in the retirement
  sweep per its actual insta shape.
- Whether `foolish_review.sh` / `accept_approved.sh` retire outright or become thin wrappers
  over `einmo console-review` / `einmo promote` (human preference).

## References

- Prior FOOPs: FOOP-92 (einmo), FOOP-13/FOOP-23/FOOP-42 (owners of migrated comprehensives),
  FOOP-9 (operator transparency probes). FOOP-72 (Draft) consolidates insta-era snapshot
  documentation into `AGENTS/snapshot.md`; if it lands, its snapshot document must describe the
  einmo suite instead.
- `einmo.README.md` — §"Appendix: Migrating an insta test to einmo" is the step-by-step recipe
  this FOOP instantiates; §"Appendix: Development Compliance Test" is the harness pattern.
- Code: `foolish-ubca/src/ubca_snapshot_tester.rs` (insta suite to mirror),
  `zweimomo/src/evaluators.rs` (`UbcaEvaluatorAdapter` to copy), `einmo/src/einmo_suite.rs`.
