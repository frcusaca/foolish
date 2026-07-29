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
installs the **escalating validation levels** (§The escalating validation levels: development requires the
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
├── checked/                    # reviewed baseline (committed) — the Checked level
├── flagged/                    # set-aside sink
└── verified/                   # human-signed — the Verified level
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

### The escalating validation levels: Output → Checked → Verified

Validation happens at **one of three escalating levels**. Each level performs **everything the
level below it requires**, plus its own — the levels escalate, they do not replace:
`Verified ⊃ Checked ⊃ Output`. This is the spine of the two gates: the **feature-complete test
suite** validates at the `Checked` level; the **merge-ready test suite** escalates to the
`Verified` level. Both run against the same suite directories; only the level rises.

**The API has no default level.** The configuring test states which level it produces and
validates (`ValidationLevel::{Output, Checked, Verified}`); a config that has not chosen a level
cannot be constructed. The **CLI** is built on top of that API and may default (to the `Checked`
level), with `--level verified` to escalate.

#### Level 1 (base) — `Output`: the suite is well-formed and evaluates

| # | Requirement |
|---|---|
| O1 | **No extraneous files** anywhere under `input/` — a test input is a file someone deliberately named. Dot-prefixed entries (editor swap/backup, `.DS_Store`) are skipped for *discovery* so they cannot become phantom tests, and **reported as violations**. |
| O2 | **The suite is non-empty** — a run that discovered no inputs is a failure, not a vacuous pass. |
| O3 | **Every input evaluates** and its `.einmo` is written to `output/`. |
| O4 | **Every written artifact self-verifies** — its full stamp chain (compiled → configured → `stage:output`) validates against the bytes on disk immediately after writing. |
| O5 | **No orphaned `output/` artifacts** — every `.einmo` in `output/` has a corresponding `input/` file. An artifact whose input is gone is a record that can never be re-derived. |

#### Level 2 (escalates from Output) — `Checked`: …everything the Output level requires, plus a reviewed baseline

| # | Requirement |
|---|---|
| C1 | **Everything the Output level requires (O1–O5).** |
| C2 | **Files match up exactly** — every `output/` artifact has a `checked/` counterpart and vice versa. No one-sided files in either direction. |
| C3 | **No orphaned `checked/` artifacts** — every `checked/` artifact has an `input/` file. |
| C4 | **Every `checked/` artifact's signatures verify** — the chain now includes `stage:checked`. |
| C5 | **Content compares identical** — `output` vs `checked` INPUT + every OUTPUT section, byte for byte. STAMPS and metadata are excluded by design: they carry per-run timestamps and would make the gate structurally red (the insta defect this FOOP exists to fix). |

#### Level 3 (escalates from Checked) — `Verified`: …everything the Checked level requires, plus human attestation

| # | Requirement |
|---|---|
| V1 | **Everything the Checked level requires (C1–C5), and therefore the Output level too (O1–O5).** |
| V2 | **Files match up exactly** — every `checked/` artifact has a `verified/` counterpart and vice versa. A partially-signed corpus is an *incomplete* level, not a passing one. |
| V3 | **No orphaned `verified/` artifacts** — every `verified/` artifact has an `input/` file. |
| V4 | **Every `verified/` artifact's signatures verify** — the chain now includes `stage:verified`. |
| V5 | **Content compares identical** — `checked` vs `verified`, same section rule as C5. |
| V6 | **Signed by the human reviewer's key** — every `stage:verified` stamp's pubkey matches the configured reviewer key. |
| V7 | **No computer-key attestation** — *zero* `stage:verified` stamps carry the well-known empty-passphrase key. An AI piping `--passphrase ""` is detected post-hoc and fails the gate. |

`flagged/` sits **outside the escalation entirely**: it is the terminal sink, so a flagged
artifact with no input is a completed retirement, not an orphan — at every level.

#### What the API returns

Success, or failure **with a list of problems**. Each problem is an **enum variant plus a
description** — never an excerpt from a file (the artifacts are signed; a report must not
reproduce their content, and a reviewer reads them through `einmo body` / `poor_einmo.sh`):

```rust
/// The escalating validation levels. No default: a suite states its level.
pub enum ValidationLevel { Output, Checked, Verified }

/// How much to look for before giving up. Default: FailAtEnd.
pub enum FailurePolicy {
    FailFast,   // stop at the first failure
    FailAtEnd,  // run everything, report every problem together (default)
}

pub enum Problem {
    // ── Output level ────────────────────────────────────────────────────
    ExtraneousInputFile { path },                 // O1 — cruft IN input/;
                                                  //      nothing generated it
    EmptySuite,                                   // O2
    ArtifactUnsound { path, detail },             // O3 / O4
    OrphanedStageArtifact { stage, path },        // O5/C3/V3 — a generated
                                                  //   artifact in a STAGE dir
                                                  //   whose input is gone

    // ── pairwise: `left` is the side escalated FROM, `right` the side TO ──
    LeftMissingEntirely  { left, right, path },   // C2 / V2
    RightMissingEntirely { left, right, path },   // C2 / V2
    SectionDifference { left, right, path, section },  // C5/V5 — names the
                                                  //   section, never content
    SignatureDoesNotVerify { stage, path, detail },    // C4/V4 — the bytes do
                                                  //   not match the signature

    // ── Verified level: three DISTINCT facts about the signer ───────────
    SignedByUnexpectedKey { path, expected_prefix, found },  // V6 — verifies,
                                                  //   but the wrong person
    KeyDerivedFromEmptyPassphrase { path },       // V7 — verifies, well-formed,
                                                  //   and IS the well-known
                                                  //   empty-passphrase key
}
```

**Two distinctions the enum makes deliberately:**

| Pair | The difference |
|---|---|
| `ExtraneousInputFile` vs `OrphanedStageArtifact` | **Which tree.** The first is cruft *in `input/`* that nothing generated (an editor swap file wandered in). The second is a file einmo *did* generate, *in a stage directory*, whose input has since disappeared — a signed record that can never be re-derived. Different faults, different remedies. |
| `SignatureDoesNotVerify` vs `SignedByUnexpectedKey` vs `KeyDerivedFromEmptyPassphrase` | **What is actually wrong.** (1) the signature does not match the bytes it covers — tampering, regardless of whose key it is. (2) the signature verifies perfectly, but the public key is not the reviewer's — the right bytes, the wrong person. (3) the signature verifies, the key is well-formed, and it **is the key generated from the empty passphrase** — the well-known computer key. That last one is *why* the AI bypass is detectable: einmo derives that key itself and compares. |

Each variant carries the offending path and a one-line description of what is wrong and what to
do about it. The **CLI is complementary**: it prints the same problems (`--json` emits one
object per problem) and exits non-zero — one implementation, so the library and CLI can never
disagree about what a valid suite is.

**Mechanics of the merge gate (the Verified level):**

- A second test (`einmo_verified_gate`), `#[ignore]`d locally, run by
  `.github/workflows/einmo-gates.yml` on pull requests. Making that workflow a **required status
  check** is a GitHub branch-protection setting — a human act (agents do not change repository
  settings); the plan carries it as a human checkbox.
- The reviewer's public key (hex prefix) is a constant in the gate; the human derives and
  supplies it once (their passphrase never appears anywhere).

### Retraction (demotion): pulling a promotion back for re-examination

The pipeline could **promote** up (output → checked → verified) and **flag** aside, but it had
no way to say *"un-promote this — it needs another look."* Flagging is the wrong tool: it means
"set aside as wrong" and sends the artifact to the terminal `flagged/` sink. Retraction means
"this baseline is provisional again," which is a different act.

**`retract(config, stage, files)`** removes an artifact from `checked/` or `verified/` so it is
no longer part of that baseline. The input still produces fresh `output/`, which then shows as
needing re-review against the now-absent higher stage — exactly the re-examination that was
wanted. Retraction from `output/` is meaningless (it is regenerated every run) and refused.

**The cascade (the load-bearing rule).** Retracting a `checked/` artifact **also removes its
`verified/` counterpart** if one exists. A `verified/` stamp attests that a human reviewed *this
checked baseline*; pull the baseline and the attestation is dangling — it certifies content that
no longer stands. So retraction cascades **downward through the stages it invalidates**:

| Retract from | Also removes |
|--------------|--------------|
| `verified` | (nothing — it is the top) |
| `checked`  | the matching `verified/` artifact, if present |

Retraction is **not** a stamp operation and appends nothing: the artifacts are *removed*, and
git history preserves what they were. This keeps the trust chain honest — a stage never contains
a baseline that a lower stage no longer supports. Both `console-review` and `poor_einmo.sh`
surface it: a reviewer who realizes a promoted artifact needs re-examination retracts it (and its
downstream) rather than living with a baseline they no longer trust.

### Flagging: break-and-demand-attention, and dated accumulation

A **flag breaks the test and demands attention** — it is not a diff. Flagging moves an artifact into
`flagged/` with an advisory reason (`einmo flag <suite> <stage> --reason "<note>" -- <files>`); the
flagged test is a red mark that a human must resolve, and it does **not** diff against any baseline
(there is nothing to compare — the point is "this is wrong, stop and look"). The advisory reason is
the reviewer's note in full, kept **in context**: reviewers annotate the rendered body right where the
error is (e.g. an `@agent` comment beside the failing output line), and that surrounding body is what
makes the note actionable for a human or AI, so the whole edited pane is recorded, not just the added
line. `poor_einmo.sh` composes this reason and prints the `einmo flag` command; it never flags for you
(the corpus is mutated only by a command you run).

**`flagged/` is plaintext and transient (settled by FOOP 25 §S.3).** Flagging writes a **plaintext,
unsigned** message — a development-process marker, not a durable signed record. Re-flagging an existing
`flagged/<test>` **concatenates**: the new dated block on top, the prior content below, in the same
path. Because it is plaintext by design there is no envelope to corrupt and `flagged/` stays exempt from
verification. Durable, attributable observations do NOT go here — FOOP 25 adds a **signed `notes/`
stage** for those (a proper stamped `.einmo`); a flag's concatenated content can be promoted into
`notes/` to become a signed note. See FOOP 25 §S.3 for the flag-breaks-tests-by-default rule and the
`flagged/` vs `notes/` split.

### foolish-core migration (as part of this FOOP)

`foolish-core/snapshot_tests/` migrates to a corresponding **`foolish-core/einmo_suite/`** with
the **same organizational requirements** as the UBCa suite: organized by FOOP provenance
(`foop/<NUMBER>/…`, `regression/…`, `misc/…` catchall), the same placement rules R1–R5, **one
home per test** (no dual-homing; identical tests keep the older), and its own `MAPPING.md`. Both
gates cover it: the feature-complete test suite validates it at the Checked level; the
merge-ready test suite escalates it to the Verified level under the reviewer's key. One inventory question is resolved during
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
  promote flow, `console-review`, the escalating validation levels).

### Comprehensive-test path re-homing

From this FOOP forward, the reserved comprehensive input for FOOP `<N>` lives at
`foolish-ubca/einmo_suite/input/foop/<N>/comprehensive.foo` (was
`foolish-ubca/snapshot_tests/input/foop_<N>_comprehensive.foo`). `foop.md` and `AGENTS.md` are
updated inside this FOOP's worktree, so the rule and the tree change land together. FOOP-64's own
comprehensive is the first test born at the new path.

## Security Considerations

The whole point of einmo is that a snapshot is a *signed, attributable* artifact. Two places
handle secrets or signed content and therefore need stating plainly.

### Signing keys at rest — the KEK

Promotion signs, and signing needs a key derived from a passphrase via Argon2id (~1.8s by
design). A batch signs every file with the **same** key, so the key is derived **once** and
cached for the batch. A cached secret would otherwise sit in plaintext in process memory for the
whole run.

Instead the derived seed is **sealed** with XChaCha20-Poly1305 under a random per-process
**key-encryption key (KEK)**. `StageKeypair::with_signing_key` unwraps it, signs, and zeroizes
the plaintext before returning — so the seed is in the clear only for the microseconds of one
signature, not for minutes. `Debug` never renders key material.

**Honest scope (stated so no one over-trusts it):** the KEK lives in the same process's memory,
so this does **not** stop an attacker who can already read our address space. It is
defense-in-depth that shrinks the plaintext window — heap dumps, core files, swap, a long-lived
batch. Real isolation (an OS keyring, an HSM, a separate privileged signer) is out of scope for
this FOOP. The `verified`-stage passphrase is never cached or stored: `einmo.toml` leaves it
unset, so the cascade falls through to an interactive `/dev/tty` prompt, and a non-interactive
`checked->verified` fails rather than signing.

### Review scratch is signed content — "police your temp's"

A text review (`poor_einmo.sh`, and the future `einmo console-review`) renders the artifacts
under review into temporary files, lets an editor open them, and keeps reviewer notes. **All of
that is the signed material under review** — the panes, the notes, and every editor
swap/backup/undo file. On a shared host, a world- or group-readable temp directory leaks it.

The rule — *police your temp's*, in the sense of policing your brass: **you leave no readable
trace on shared ground.** Concretely, every review tool MUST:

1. **Set `umask 077` before creating any scratch**, so every file is born `600` and every
   directory `700` — private at birth, no per-file chmod chase. This is the process-wide
   default for how new files are made; setting it once governs all scratch (the pane renders,
   the `.orig` baselines, the notes, and the editor's swap/backup/undo files).
2. **Enforce it on existing directories, not assume it.** A user-supplied scratch path may
   predate our `umask` or arrive with contents already group/world-readable. The tool
   `chmod -R go-rwx`s the directory and everything under it, verifies **nothing** beneath it
   keeps a group/other bit, and **refuses to run** if that did not take (e.g. a directory the
   user cannot chmod). Fix if we can; refuse only if we can't — the material is too sensitive to
   leave to a default.
3. **Harden before any early exit,** so a user-supplied private directory is secured the moment
   it is chosen, even on a run that finds no tests.
4. **Offer a private-location override** (`poor_einmo.sh`'s `POOR_EINMO_DIR`; console-review's
   equivalent) so a reviewer can point scratch at an encrypted or tmpfs-backed path of their own
   rather than shared `/tmp` — still forced to `700`.
5. **Remove all scratch on exit** (the tool traps and cleans up), so nothing outlives the
   review.

`poor_einmo.sh` implements all five; `einmo console-review`, when built, MUST inherit this
section verbatim — it is a review tool over the same signed content and has the same exposure.

## Proposed new combination tests

New tests authored by this FOOP live in `foop/64/` — one home each, per §"One home per test".
Each targets a feature *pair/triple* the current corpus misses:

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
  `output/` and matching `checked/` — the **feature-complete test suite**, at the Checked level.
- `einmo_verified_gate` (new, `#[ignore]` locally, CI on PRs): output ↔ `verified/`
  correspondence + human-key `confirm-signatures --require-all` + zero-computer-key scan on
  `stage:verified` stamps — the **merge-ready test suite**, at the Verified level.
- `cross_validate_einmo_vs_insta` (new, `#[ignore]`, temporary): migrated outputs byte-match the
  approved `.snap` RESULTs.
- Existing insta `approval_all` keeps passing untouched until human retirement.
- The nine new `.foo` inputs of §Proposed new combination tests go through the normal einmo
  review: agent generates `output/`, promotes to `checked/` after self-review; human reviews the
  diff and may sign `verified/`.
- **Flag = plaintext, concatenating** (§Flagging; details in FOOP 25 §S.3): flagging writes a plaintext,
  unsigned note; re-flagging concatenates a new dated block on top of the prior content in the same
  path; `flagged/` stays exempt from verification. Durable signed observations use the new `notes/`
  stage (FOOP 25), not `flagged/`.

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
- Reviewer key (V6): a single human reviewer key, or a small set of accepted reviewer keys
  (workflow constant becomes a list)?
- ~~`foolish-parser` / `foolish-core` insta inventories: separate suites or shared?~~ Resolved
  (Atlas 2026-07-14): **per-crate corresponding suites** — `foolish-core/einmo_suite/` with the
  same organizational requirements, covered by both gates; parser handled in the retirement
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
