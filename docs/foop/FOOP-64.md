---
foop: 46
title: Migrate UBCa snapshot tests to a hierarchical einmo suite
author: Atlas <hc.busy@gmail.com>
status: Draft
type: Standards
created: 2026-07-14
phase: meta
supersedes: []
begun: [x]
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
(`foop/<NUMBER>/…`, `regression/…`, `misc/…` — organized by FOOP provenance) and stored in the
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
├── einmo.toml                  # signing: computer key for output/checked; verified unset
├── MAPPING.md                  # generated: old-path → new-path, rules, dedup + dup survey
├── input/
│   ├── foop/                   # organized by FOOP provenance (the primary axis)
│   │   ├── 9/    13/    23/    41/    42/    62/    64/
│   │   └──   (comprehensives + each FOOP's development tests)
│   ├── regression/             # pinned bug reproductions
│   └── misc/                   # CATCHALL beside foop/ — not easily categorized;
│                               #   the pool to re-home as FOOPs claim their tests
├── output/                     # generated, signed (computer key)
├── checked/                    # reviewed baseline (committed) — feature-complete tier
├── flagged/                    # set-aside sink
└── verified/                   # human-signed — merge-ready tier
```

Directory names under `foop/` use the **filename digits** (the little-endian identifier):
`foop/13/` is FOOP-13, *not* the FOOP whose sort key is 13 (that would be FOOP-31).

### Placement rules

Inputs remain `.foo` files; einmo writes `<relative-path>.foo.einmo` into each stage directory.
Rules are **first-match-wins**; the executed outcome for every input is recorded in
`einmo_suite/MAPPING.md`.

| Rule | Condition | Home | Count (executed) |
|------|-----------|------|------------------|
| R1 | name declares FOOP + comprehensive (`foop_<N>_comprehensive`) | `foop/<N>/comprehensive.foo` | 2 |
| R2 | name declares its FOOP (`foop<N>_<rest>`, non-regression) | `foop/<N>/<rest>.foo` | 3 |
| R3 | regression (`regression_*`, `*_regression`) | `regression/<stem>.foo` | 4 |
| R4 | git birth-commit names a FOOP (attribution pass) | `foop/<N>/<stem>.foo` | 20 |
| R5 | everything else — catchall | `misc/<stem>.foo` | 132 |
| DEDUP | byte-identical (source + output) to another input | not migrated — keep one | 1 |

Directory names under `foop/` use the **filename digits** (the little-endian identifier):
`foop/13/` is FOOP-13, *not* the FOOP whose sort key is 13 (that would be FOOP-31). Prefix
stripping applies only where the directory already carries the FOOP (`foop9_unary_operator.foo`
→ `foop/9/unary_operator.foo`); `misc/` and `regression/` keep whole stems.

**162 old → 161 migrated**, every copy byte-identical to its source.

### One home per test (no dual-homing)

**Superseded the earlier dual-home proposal (Atlas 2026-07-15).** Every input has exactly ONE
home. The organizing axis is **FOOP provenance**: a test whose name declares its FOOP (R1/R2) or
whose git birth-commit names one (R4) lives under `foop/<N>/`; regressions live under
`regression/`; everything not easily categorized lands in **`misc/`**, a catchall beside `foop/`
that is the pool to re-home later as FOOPs claim their tests.

**Identical tests: keep one — the older.** When two inputs are byte-identical in source *and*
evaluated output, only one is migrated (age decides; where both were born in the same commit,
the base name wins over a derived `_regression`-style copy). MAPPING.md records the drop.

**Near-identical tests are NOT merged automatically.** Integration ("keep only the differing
elements of two nearly identical tests") rewrites Foolish source and can silently delete
coverage, so it is a **human authoring decision**, tracked as a plan task. The corpus survey
(recorded in `einmo_suite/MAPPING.md`) found 54 pairs ≥0.75 similar that are deliberate
axis-variations — `offset_access_backward` vs `_forward`, concat empty/single/unresolved,
`sff_basic` vs `sff_nested` — where merging would destroy failure localization. They stay
separate.

Within a single directory, genuine variants may still use einmo's dependent naming
(`base++variant.foo`, diffed via the DIFF section).

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

### Validation levels: Output, Checked, Verified (escalating, cumulative)

Validation happens at **one of three levels**. Each level performs **everything the level below
it requires**, plus its own — `Verified ⊃ Checked ⊃ Output`. This is the spine of the two gates:
the **feature-complete test suite** validates at `Checked`; the **merge-ready test suite**
validates at `Verified`. Both run against the same suite directories; only the bar rises.

**The API has no default.** The configuring test states the level it produces and validates
(`ValidationScope::{Output, Checked, Verified}`); a config that has not chosen cannot be
constructed. The **CLI** is built on top and may default (to `Checked`), with `--scope verified`
to escalate.

#### Level 1 — `Output`: the suite is well-formed and evaluates

| # | Requirement |
|---|---|
| O1 | **No extraneous files** anywhere under `input/` — a test input is a file someone deliberately named. Dot-prefixed entries (editor swap/backup, `.DS_Store`) are skipped for *discovery* so they cannot become phantom tests, and **reported as violations**. |
| O2 | **The suite is non-empty** — a run that discovered no inputs is a failure, not a vacuous pass. |
| O3 | **Every input evaluates** and its `.einmo` is written to `output/`. |
| O4 | **Every written artifact self-verifies** — its full stamp chain (compiled → configured → `stage:output`) validates against the bytes on disk immediately after writing. |
| O5 | **No orphaned `output/` artifacts** — every `.einmo` in `output/` has a corresponding `input/` file. An artifact whose input is gone is a record that can never be re-derived. |

#### Level 2 — `Checked`: …everything Output requires, plus a reviewed baseline

| # | Requirement |
|---|---|
| C1 | **All of O1–O5.** |
| C2 | **Files match up exactly** — every `output/` artifact has a `checked/` counterpart and vice versa. No one-sided files in either direction. |
| C3 | **No orphaned `checked/` artifacts** — every `checked/` artifact has an `input/` file. |
| C4 | **Every `checked/` artifact's signatures verify** — the chain now includes `stage:checked`. |
| C5 | **Content compares identical** — `output` vs `checked` INPUT + every OUTPUT section, byte for byte. STAMPS and metadata are excluded by design: they carry per-run timestamps and would make the gate structurally red (the insta defect this FOOP exists to fix). |

#### Level 3 — `Verified`: …everything Checked requires, plus human attestation

| # | Requirement |
|---|---|
| V1 | **All of C1–C5** (and therefore all of O1–O5). |
| V2 | **Files match up exactly** — every `checked/` artifact has a `verified/` counterpart and vice versa. A partially-signed corpus is an *incomplete* tier, not a passing one. |
| V3 | **No orphaned `verified/` artifacts** — every `verified/` artifact has an `input/` file. |
| V4 | **Every `verified/` artifact's signatures verify** — the chain now includes `stage:verified`. |
| V5 | **Content compares identical** — `checked` vs `verified`, same section rule as C5. |
| V6 | **Signed by the human reviewer's key** — every `stage:verified` stamp's pubkey matches the configured reviewer key. |
| V7 | **No computer-key attestation** — *zero* `stage:verified` stamps carry the well-known empty-passphrase key. An AI piping `--passphrase ""` is detected post-hoc and fails the gate. |

`flagged/` is **outside the escalation entirely**: it is the terminal sink, so a flagged
artifact with no input is a completed retirement, not an orphan, at every level.

#### What the API returns

Success, or failure **with a list of problems**. Each problem is an **enum variant plus a
description** — never an excerpt from a file (the artifacts are signed; a report must not
reproduce their content, and a reviewer reads them through `einmo body` / `poor_einmo.sh`):

```rust
pub enum ValidationScope { Output, Checked, Verified }

pub enum Problem {
    ExtraneousFile { path, .. },        // O1
    EmptySuite,                         // O2
    EvaluationFailed { path, .. },      // O3
    SelfVerifyFailed { path, .. },      // O4
    OrphanedArtifact { stage, path },   // O5 / C3 / V3
    MissingCounterpart { from, to, path },  // C2 / V2
    SignatureInvalid { stage, path, .. },   // C4 / V4
    ContentDiffers { a, b, path, sections },// C5 / V5  (names sections, not content)
    WrongSigner { path, expected, found },  // V6
    ComputerKeyAttestation { path },        // V7
}
```

Each variant carries the offending path and a one-line description of what is wrong and what to
do about it. The **CLI is complementary**: it prints the same problems (`--json` emits one
object per problem) and exits non-zero — one implementation, so the library and CLI can never
disagree about what a valid suite is.

**Mechanics of the merge tier:**

- A second test (`einmo_verified_gate`), `#[ignore]`d locally, run by
  `.github/workflows/einmo-gates.yml` on pull requests. Making that workflow a **required status
  check** is a GitHub branch-protection setting — a human act (agents do not change repository
  settings); the plan carries it as a human checkbox.
- The reviewer's public key (hex prefix) is a constant in the gate; the human derives and
  supplies it once (their passphrase never appears anywhere).

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
