# FOOP-25.plan — einmo-review-session

Read `docs/foop/FOOP-25.md` before acting on any task below. Tasks run top to bottom; each phase
lands value on its own. Worktree variables, expanded:

```
WORKTREE_ORIGIN_BRANCH=jia         # confirmed by Atlas 2026-07-19 (merges target `jia`)
WORKTREE_ORIGIN_PATH=/home/hcbusy/foolish-rust
WORKTREE_BRANCH_NAME=foop-25-einmo-review-session
WORKTREE_FULL_FS_PATH=/home/hcbusy/tmp/foolish-worktrees/foop-25-einmo-review-session
```

- [ ] STOP — preconditions: FOOP-64 merged and its suite green; all workspace tests pass. Do not
      begin while any test is broken (Development Rules).
- [ ] Sanity check: consult human to resolve FOOP-25.md §Open Questions (HTTP stack, journal
      location, differing default) enough to start Phase A. Remind them: "Above message comes from FOOP-25 working to build the
      EinmoReview session object; the worktree is at
      /home/hcbusy/tmp/foolish-worktrees/foop-25-einmo-review-session. PTAL"
- [ ] Begin work: commit FOOP-25.md and FOOP-25.plan.md on the origin branch, check `begun: [x]` in
      FOOP-25.md frontmatter
- [ ] Create worktree at /home/hcbusy/tmp/foolish-worktrees/foop-25-einmo-review-session with branch
      `foop-25-einmo-review-session` (from this point ALL work, including FOOP file updates, happens
      only in the worktree)

## Phase 0 — drift re-survey (FOOP-25.md §S.10)

- [ ] Read §S.10 of FOOP-25.md, then re-survey `einmo/src/einmo_suite.rs`, `transitions.rs`,
      `signature.rs`, `verify.rs`, `format.rs`, `compare.rs` for API drift since 2026-07-19
- [ ] Touch up FOOP-25.md §S.2–§S.7 sketches to match current einmo shapes (worktree copy only);
      record notable drift in this plan as sub-tasks

## Phase A — the session library (FOOP-25.md §S.2–§S.6)

- [ ] Write the unit tests FIRST (FOOP-25.md §Test Plan: decisions, cache, signer, execute, journal)
      as failing tests against the intended `einmo::review` surface
- [ ] Implement `review::Decision` + `DecisionBook` (per-item, per-reviewer, versioned;
      replace-not-stack)
- [ ] Implement `review::VerifiedCache` (fingerprint → `Arc<OnceLock<VerifiedBody>>`, single-flight;
      verify-count test hook)
- [ ] Implement `review::Signer` / `SignerSet` (Argon2id→Ed25519 derive-once, zeroize on drop,
      computer key constructor) — §S.4 is the authority for what does NOT go in `EinmoReview`
- [ ] Implement `EinmoReview` (open/items/body/diff/decide/undecide/decision/refresh) over the above
- [ ] Implement `ExecutionPlan` + `execute`/`execute_one` (exec mutex, fingerprint re-check,
      skip-and-report drift, retract cascade, confirm token plumbed but enforced by frontends)
- [ ] Flag = plaintext, concatenating (§S.3): `flagged/` is PLAINTEXT/unsigned/transient; execute writes
      the annotated note as plaintext and CONCATENATES a dated block on top when re-flagging; concurrent
      flags serialize under the exec mutex; `flagged/` stays exempt from verification; journal records
      each. (Supersedes the earlier `einmo flag --merge` in-envelope idea — flags are plaintext now.)
- [ ] New signed `notes/` stage (§S.3): a durable, attributed sibling to `flagged/`; a note is a valid
      signed `.einmo` (stamped, verify-on-inspect, participates in signature checks); support promoting a
      flag's concatenated content into `notes/` as a signed note body. (Coordinate the `notes/` stage
      addition with FOOP-64's stage set at begun-time.)
- [ ] Flags break tests by default (§S.3): a flagged artifact fails the run (non-zero / red gate);
      `--flag-is-not-failure` downgrades to non-fatal but stderr STILL announces the flag count (no
      silent config); wire into the goal-state check (green = zero flags + signed + matching + valid
      signatures). Tests per §Test Plan "flag breaks tests".
- [ ] Implement `Journal` (append-only JSONL, replay, truncated-tail tolerance)
- [ ] All Phase A tests green; `cargo fmt` and `cargo clippy -D warnings` clean

## Phase A2 — `CorpusSigner` (section PQ attestation), CRYPTO CORE ONLY (FOOP-25.md §S.11)

Self-contained `CorpusSigner` object — NOT mixed into `EinmoReview` (§S.11). NO real-corpus writes and
NOT wired into the live promotion flow in this FOOP (that integration is a later step). Prove the
object in isolation; `EinmoReview` will merely hold and call it later.

- [ ] Read §S.11 of FOOP-25.md; add `fips205` dep (feature `slh_dsa_sha2_256s` — conservative set) to
      einmo/Cargo.toml
- [ ] Write tests FIRST (§Test Plan "CorpusSigner read strategies" + section attestation): deterministic
      manifest; digest changes on add/remove/alter/reorder; SLH-DSA sign→verify round-trip; tamper
      fails; same-passphrase dual-derivation determinism; empty-section manifest; the two read
      strategies agree bit-for-bit — all exercising `CorpusSigner` standalone (no `EinmoReview`)
- [ ] Implement `CorpusSigner` skeleton (`new`/`manifest`/`digest`/`sign`/`verify`) + the manifest
      builder (stage name + param-set id + sorted mirror-path list via the existing deterministic walk)
- [ ] Implement `ReadStrategy::ParallelBuffer` (DEFAULT): metadata→offsets→one allocation; parallel
      `read_exact` into disjoint slices, bounded worker pool; short/long-read hard error; hand the whole
      buffer to the signer (add `rayon` OR a small std thread-pool — decide at begun-time, see Open Q)
- [ ] Implement `ReadStrategy::Stream` (alternative): sequential manifest-order read feeding the hasher
      incrementally, bounded memory; assert byte-identical digest to `ParallelBuffer`
- [ ] Extend `Signer` (§S.4) to derive BOTH the Ed25519 stamp key and the section SLH-DSA key from one
      passphrase (Argon2id output expanded to the SLH-DSA seed; deterministic keygen)
- [ ] Implement `sign`/`verify` over the digest; `.section.sig` file shape defined but written only to
      fixtures/tempdirs in tests, never the real corpus
- [ ] Phase A2 tests green; `cargo fmt` / `cargo clippy -D warnings` clean; `#![forbid(unsafe_code)]`
      still holds (fips205 is pure Rust)

## Phase B — CLI verbs

- [ ] `einmo review plan|list|decide|undecide|execute` one-shot subcommands (journal-backed session
      identity) with endpoint-equivalent semantics; unit tests
- [ ] Byte-for-byte equivalence test: `review execute` promotion == existing `einmo promote`

## Phase C — the server (FOOP-25.md §S.7)

- [ ] `einmo review serve <suite>`: UDS listener by default; TCP 127.0.0.1 + bearer token behind a
      flag; suite lockfile (second server refuses)
- [ ] JSON endpoints per §S.7 table incl. If-Match/409 and SSE events; endpoint tests against a
      tempdir suite; passphrase handled only inside POST execute (derive-use-drop)
- [ ] Concurrency tests: N verifiers, single-flight verify counts, no lost updates, claims expire

## Phase D — reduce poor_einmo.sh (FOOP-25.md §S.8)

- [ ] Add server discovery + `fetch_body`/decision/plan/execute thin-client paths; keep the direct
      `einmo` fallback
- [ ] Delete the superseded state machinery (decision arrays, undo/answer bookkeeping, results
      rendering, stats computation)
- [ ] Pty-driven end-to-end tests (stub-vim technique): promote, note→flag, u-revisit keeps answer,
      gate confirm/skip, fallback-without-server
- [ ] Measure and record here: line count (target ≤350) and per-test spawn/verification counts

## Phase E — dhtml frontend (FOOP-25.md §S.9)

- [ ] Single embedded page: 4-pane view, server diff hunks, verb buttons, notes→Flag, plan view with
      typed-PROMOTE gate, SSE refresh
- [ ] Browser-path integration test (HTTP+token mode) reusing Phase C fixtures

## Comprehensive test + merge

- [ ] Comprehensive test, adapted per FOOP-25.md §Test Plan: scripted multi-verifier end-to-end
      session (two reviewers, mixed individual/batch signing, crash-resume, drift) over a fixture
      suite; stamp chains asserted with `einmo verify`. (The reserved `foop_25_comprehensive.foo`
      language test is inapplicable — meta/tooling FOOP, no FVM surface; this discharges the
      obligation.)
- [ ] Verify all work is complete in /home/hcbusy/tmp/foolish-worktrees/foop-25-einmo-review-session
      and committed to `foop-25-einmo-review-session`
- [ ] Merge `foop-25-einmo-review-session` to `jia`
  - [ ] Repair ALL tests on `jia` in /home/hcbusy/foolish-rust
  - [ ] STOP! STOP!! STOP!!! ASK HUMAN to check this box before continuing. UNDER NO CIRCUMSTANCES
        will Agent continue past this point automatically!!
    - [ ] Present human with `cd /home/hcbusy/tmp/foolish-worktrees/foop-25-einmo-review-session`
          and ask them to review before checking the parent checkbox.
  - [ ] Cleanup /home/hcbusy/tmp/foolish-worktrees/foop-25-einmo-review-session
    - [ ] Check that FOOP-25.plan.md has all but Cleanup checkboxes completed
    - [ ] Remove /home/hcbusy/tmp/foolish-worktrees/foop-25-einmo-review-session
    - [ ] This is the last sub-task checkbox to be checked in this block
