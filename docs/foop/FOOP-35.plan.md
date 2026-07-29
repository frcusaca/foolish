# FOOP-35.plan — einmo-ship (library + cargo command, own repo, crates.io)

Read `docs/foop/FOOP-35.md` before acting on any task below; the plan stages mirror the spec's
§S.1–§S.8 walkthrough one-to-one and run top to bottom. Worktree variables, expanded:

```
WORKTREE_ORIGIN_BRANCH=jia
WORKTREE_ORIGIN_PATH=/home/hcbusy/foolish-rust
WORKTREE_BRANCH_NAME=foop-35-einmo-ship
WORKTREE_FULL_FS_PATH=/home/hcbusy/tmp/foolish-worktrees/foop-35-einmo-ship
```

- [ ] STOP — preconditions: FOOP-64 merged to jia and the einmo suite green; all workspace tests
      pass (Development Rules). FOOP-25 need NOT be done (spec §S.1 #7: publish before FOOP-25).
- [ ] Begin work: commit FOOP-35.md and FOOP-35.plan.md on `jia`, check `begun: [x]` in FOOP-35.md
      frontmatter
- [ ] Create worktree at /home/hcbusy/tmp/foolish-worktrees/foop-35-einmo-ship with branch
      `foop-35-einmo-ship` (from this point ALL work, including FOOP file updates, happens only in
      the worktree)

## Stage 1 — the decisions ledger (FOOP-35.md §S.1)

- [ ] Read §S.1 of FOOP-35.md
- [ ] Check crates.io availability of the name `einmo` (`cargo search einmo` + web UI)
- [ ] ASK HUMAN to ratify the ledger: crate name (or fallback), repo host + owner, version 0.1.0,
      MSRV target, publish-before-FOOP-25. Remind them: "Above message comes from FOOP-35 working
      to ship einmo as a library + cargo command; the worktree is at
      /home/hcbusy/tmp/foolish-worktrees/foop-35-einmo-ship. PTAL"
- [ ] Record the ratified answers HERE as sub-bullets (they parameterize every later stage)

## Stage 2 — the `test` subcommand (FOOP-35.md §S.2)

- [ ] Write the integration tests FIRST (`assert_cmd` against fixture suites): exit 0 green /
      1 drift / 2 tamper / 101 config; `--level checked|verified`; `--filter`; `--json` schema;
      `cargo einmo test` argv-strip path
- [ ] Implement `einmo test` (suite discovery via einmo.toml walk-up, evaluator from config or
      `--evaluator`, `--jobs`, verify-on-inspect as hard failure)
- [ ] Dogfood in-tree: `cargo run -p einmo -- test foolish-ubca/einmo_suite --level checked`
      matches the FOOP-64 gate verdict


## Stage 3 — crate completeness (FOOP-35.md §S.3)

- [ ] Add LICENSE-MIT and LICENSE-APACHE; Cargo.toml metadata (repository, homepage, readme,
      keywords, categories, rust-version, include/exclude)
- [ ] Public library surface audit + `#![forbid(unsafe_code)]` + `#![deny(missing_docs)]`;
      rustdoc examples on all public items; `examples/` (drive-a-suite, verify-programmatically)
- [ ] External-audience README (library quickstart + CLI quickstart)

## Stage 4 — repository extraction (FOOP-35.md §S.4)

- [ ] `git filter-repo --subdirectory-filter einmo/` on a fresh clone (fallback: subtree split);
      graft README/LICENSE/CI; push to the ratified remote; tag `v0.1.0-rc0`
- [ ] Monorepo switch: remove `einmo/` from the workspace, depend via
      `einmo = { git = "<remote>" }` + `[patch]` path override for co-development; suite, gates,
      and poor_einmo.sh verified against the git-dep binary
- [ ] STOP! ASK HUMAN before force-pushing or deleting anything in the monorepo history

## Stage 5 — local install dogfood (FOOP-35.md §S.5)

- [ ] `cargo install --path . --locked` from the new repo; from a clean shell run
      `cargo einmo test` on the Foolish suite; `einmo self-check` wired as the new repo's smoke test

## Stage 6 — crates.io registration and publish (FOOP-35.md §S.6)

- [ ] crates.io account + verified email + scoped publish token (`cargo login`) — HUMAN performs;
      agent prepares everything up to the button
- [ ] `cargo publish --dry-run --locked` clean; `cargo package --list` eyeballed (no keys, no
      corpora)
- [ ] STOP! STOP!! ASK HUMAN to run `cargo publish` (an irreversible public registration)
- [ ] Post-publish: docs.rs build green; `cargo install einmo` from a machine without the repo;
      tag `v0.1.0`; `cargo owner --add` a second owner; monorepo dependency switched to
      `einmo = "0.1"`

## Stage 7 — the testing battery + CI (FOOP-35.md §S.7)

- [ ] CI matrix {linux,macos,windows} × {stable,MSRV}: fmt, clippy -D warnings, test, doc test
- [ ] `proptest` roundtrips (envelope/stamps serialize∘parse, body extraction, promotion
      idempotence)
- [ ] `cargo-fuzz` targets: envelope parser, STAMPS parser, signature-line decoder (corpus seeded
      from fixtures; short fuzz-smoke job in CI)
- [ ] `cargo-mutants` run on signature.rs/verify.rs — kill or justify every survivor
- [ ] `cargo-deny` + `cargo audit` + `cargo-msrv verify` jobs; `cargo llvm-cov` ratchet
- [ ] Release workflow: tag → dry-run → publish → GitHub release binaries (cargo-dist or matrix);
      binstall metadata

## Stage 8 — documentation hand-back (FOOP-35.md §S.8)

- [ ] Monorepo `einmo.README.md` shrinks to a pointer + Foolish-specific suite conventions;
      AGENTS.md build commands updated to `cargo einmo test`; CHANGELOG.md at 0.1.0 in the new repo

## Comprehensive test + merge

- [ ] Comprehensive test, adapted per FOOP-35.md §Test Plan (meta/tooling FOOP — the reserved
      `foop_35_comprehensive.foo` does not apply): scripted clean-container dogfood —
      `cargo install einmo`, clone monorepo, `cargo einmo test foolish-ubca/einmo_suite
      --level checked` reproduces the in-tree verdict
- [ ] Verify all work is complete in /home/hcbusy/tmp/foolish-worktrees/foop-35-einmo-ship and
      committed to `foop-35-einmo-ship`
- [ ] Merge `foop-35-einmo-ship` to `jia`
  - [ ] Repair ALL tests on `jia` in /home/hcbusy/foolish-rust
  - [ ] STOP! STOP!! STOP!!! ASK HUMAN to check this box before continuing. UNDER NO CIRCUMSTANCES
        will Agent continue past this point automatically!!
    - [ ] Present human with `cd /home/hcbusy/tmp/foolish-worktrees/foop-35-einmo-ship` and ask
          them to review BEFORE checking the parent checkbox.
  - [ ] Cleanup /home/hcbusy/tmp/foolish-worktrees/foop-35-einmo-ship
    - [ ] Check that FOOP-35.plan.md has all but Cleanup checkboxes completed
    - [ ] Remove /home/hcbusy/tmp/foolish-worktrees/foop-35-einmo-ship
    - [ ] This is the last sub-task checkbox to be checked in this block
