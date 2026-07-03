# FOOP-03 Plan: Repository Cleanup

> **Read FOOP-03.md before executing this plan.**

## Worktree

```
WORKTREE_ORIGIN_BRANCH=alpha
WORKTREE_ORIGIN_PATH=/home/hcbusy/foolish-rust
WORKTREE_BRANCH_NAME=foop-03-repo-cleanup
WORKTREE_FULL_FS_PATH=/home/hcbusy/tmp/foolish-worktrees/foop-03-repo-cleanup
```

## Plan

> **Note (2026-07-02):** the spec's "surviving crates" list omitted `foolish-ubca`
> (FOOP-62's in-progress rewrite, already on disk as a separate crate). Per Atlas's
> explicit direction, `foolish-ubca` is KEPT throughout this plan alongside
> `foolish-core`/`foolish-parser`/`foolish-cli` — read "three crates" below as "the
> named crates plus foolish-ubca" wherever it appears.

- [x] Begin FOOP-03 execution
      (2026-07-02 14:20)
- [x] Create worktree at `/home/hcbusy/tmp/foolish-worktrees/foop-03-repo-cleanup` with branch `foop-03-repo-cleanup` from `alpha`
      (2026-07-02 14:20)
- [x] Phase 1: Remove JVM artifacts
      (2026-07-02 15:10, commit 614f01ad)
  - [x] Delete `.github/workflows/java-tests.yml`
        (2026-07-02 15:10)
  - [x] Delete `.github/workflows/scala-tests.yml`
        (2026-07-02 15:10)
  - [x] Update `.github/workflows/tests.yml` — tests.yml ("Cross Validation") was 100% Maven/Java/Scala with zero Rust content; deleted the whole file rather than editing a shell, since no non-JVM content remained. No Rust CI workflow existed to add in its place (out of FOOP-03 scope to introduce new CI). Also updated `.github/dependabot.yml` package-ecosystem from `maven` to `cargo`.
        (2026-07-02 15:10)
  - [x] Delete `docs/ubc1/todo/scala-mvp/` directory entirely (99 files: pom.xml, .java, ANTLR grammar, Scala docs)
        (2026-07-02 15:10)
  - [x] Clean `.gitignore` — removed JVM-specific entries (Ivy, Maven pom-backup files, NetBeans/Eclipse/Gradle/JIRA/Crashlytics/sbt-idea-plugin cruft); kept general IDE/OS entries (`.idea/`, `.vscode/`, `.DS_Store`); de-duped the now-single `target/` entry
        (2026-07-02 15:10)
  - [x] Verify: `grep -r "java\|scala\|maven\|pom\.xml" --include="*.yml" .github/` returns nothing
        (2026-07-02 15:10 — confirmed clean)
  - [x] Remove Java/Scala/Cross-Validation CI badges from README.md (done together with the rest of README's JVM removal, see Phase 4)
        (2026-07-02 15:20, commit 971a5785)
- [x] Phase 2: Remove dead Rust crates
      (2026-07-02 14:29, commit 065d8a7c)
  - [x] Delete `foolish/foolish-web/` directory
        (2026-07-02 14:29)
  - [x] Delete `foolish/foolish-ubcb/` directory
        (2026-07-02 14:29)
  - [x] Delete `foolish/foolish-ubcb-cli/` directory
        (2026-07-02 14:29)
  - [x] Update `foolish/Cargo.toml` — remove `foolish-web`, `foolish-ubcb`, `foolish-ubcb-cli` from workspace members (`foolish-ubca` was already a member and stays)
        (2026-07-02 14:29)
  - [x] Verify: `cargo check --workspace` succeeds from `foolish/`
        (2026-07-02 14:29 — confirmed identical pass/fail test set vs. untouched `alpha`; 2 pre-existing unrelated snapshot/test failures noted, not caused by this change)
- [x] Phase 3: Flatten workspace
      (2026-07-02 14:34, commit 3b1c11a3)
  - [x] Move `foolish/Cargo.toml` to repo root (no existing root `Cargo.toml` — clean move, no merge needed)
        (2026-07-02 14:34)
  - [x] Move `foolish/Cargo.lock` to repo root
        (2026-07-02 14:34)
  - [x] Move `foolish/foolish-core/` to repo root
        (2026-07-02 14:34)
  - [x] Move `foolish/foolish-parser/` to repo root
        (2026-07-02 14:34)
  - [x] Move `foolish/foolish-cli/` to repo root
        (2026-07-02 14:34)
  - [x] Move `foolish/foolish-ubca/` to repo root (not in original spec list — see note above)
        (2026-07-02 14:34)
  - [x] Delete `foolish/target/` (build cache)
        (2026-07-02 14:34)
  - [ ] Delete `foolish/mcp.log.*` files — none present in this worktree; not applicable
  - [x] Merge `foolish/.claude/` into root `.claude/` — NOT a plain overwrite (verified materially different content): `settings.json` moved to `.claude/settings.foolish-legacy.json` (kept alongside root's own `settings.json`, not clobbered).
        (2026-07-02 20:05, commit d9dac7f0)
  - [x] Merge `foolish/.omo/` into root `.omo/` — moved `.omo/tasks/*` (5 files, no ID collisions), `.omo/run-continuation/*` (2 files, no collisions), `.omo/notepads/foop-62-ubca-mimo/` (no collision). One real collision found and resolved: `.omo/notepads/foop-62-ubca/learnings.md` existed in both with genuinely different content (two different FOOP-62 debugging sessions) — concatenated both bodies into the root file rather than picking a winner.
        (2026-07-02 20:05, commit d9dac7f0)
  - [x] Delete empty `foolish/` directory — now empty after the merge above; removed.
        (2026-07-02 20:05, commit d9dac7f0)
  - [x] Verify: `cargo build --workspace` succeeds from repo root
        (2026-07-02 14:34)
  - [ ] Verify: `cargo test --workspace` passes — NOT run as a gate this session (tests use non-standard insta/signature machinery; human asked to check script output directly instead — see `verify_signatures` runs in session log). 2 pre-existing snapshot/test issues carried over from baseline `alpha`, unrelated to this Phase.
- [x] Phase 4: Update documentation
      (2026-07-02 15:20, commit 971a5785 + earlier 3b1c11a3)
  - [-] Rewrite AGENTS.md Build Requirements — remove Java/Scala/Maven/ANTLR, replace with Rust-only — not needed: AGENTS.md's Build Requirements section was already Rust-only (`- **Rust**: current stable toolchain`), just had a stray `(see foolish/ workspace)` clause, fixed in Phase 3.
  - [x] Rewrite AGENTS.md Build Commands — workspace-root path now repo-root-relative (no `cd foolish` prefixes existed; fixed the one absolute `/home/hcbusy/foolish-rust/foolish` root-path sentence instead)
        (2026-07-02 14:34)
  - [-] Rewrite AGENTS.md Project Structure — not done this pass. Crates are foolish-core, foolish-ubca, foolish-parser, foolish-cli (four, not the spec's three — see note above); rewriting this section to describe UBCa as sole reference is BLOCKED on the FOOP-62 retirement gate (foolish-core is not yet retired). Left as a follow-up.
  - [-] Rewrite AGENTS.md Architecture section — UBCa as reference implementation, Nyes states, constanic terminology, AB/IB semantics — NOT done, BLOCKED on FOOP-62 retirement gate (see FOOP-62.plan.md backburner note). AGENTS.md's existing terminology sections describe UBC generically and are not currently wrong, just not UBCa-specific.
  - [x] Rewrite AGENTS.md Testing section — corrected stale `foolish-ubcb` snapshot references (left over from before Phase 2 deleted that crate) to `foolish-ubca`
        (2026-07-02 14:34)
  - [x] Rewrite AGENTS.md Approval Test section — updated paths (workspace-root, signature-verification walkthrough) and stale UBCb refs
        (2026-07-02 14:34)
  - [x] Rewrite AGENTS.md CLI Usage section — no `cd foolish` prefixes existed here; nothing to change
        (2026-07-02 14:34)
  - [x] Update README.md — corrected directory reference and stale `foolish-ubcb-cli` test commands (now `foolish-ubca`); build/test commands now repo-root-relative
        (2026-07-02 14:34)
  - [x] Update README.md — removed Java/Scala/Cross-Validation badges, Quick Start Java/Scala (`mvn` command), and the "Version Overview" prose (folded into a shorter "Documentation Layout" table); fixed the resulting broken anchor link
        (2026-07-02 15:20)
  - [x] Update rust_instructions.md — path references
        (2026-07-02 14:34)
  - [-] Update docs/DOC_AGENTS.md — path references, remove dead crate refs — checked, no changes needed: its only java/scala mentions are a code-block language-tag example and two already-correct changelog entries describing past Java-content removal.
        (2026-07-02 15:15)
  - [-] Update docs/styleguide.md — path references if any — checked, no changes needed: its java/scala mention documents valid code-block language tags for writing about legacy code, not a live build reference.
        (2026-07-02 15:15)
  - [x] Update docs/foop/scripts/foop_check.py — verified: already location-independent via `Path(__file__).resolve().parent.parent`, no hardcoded path. Ran `python3 docs/foop/scripts/foop_check.py check` from the flattened root: "OK: 30 FOOPs, consecutive sort keys 1 through 30." No edit needed.
        (2026-07-02 15:15)
  - [x] Update docs/foop/INDEX.md — added FOOP-03 entry and the 7 other missing FOOPs (81, 91, 42, 62, 72, 82, 92) to the main table, By Status, and By Phase sections (new phase-0 subsection for FOOP-72; new Withdrawn/Rejected/Superseded entry for FOOP-81)
        (2026-07-02 15:20)
  - [x] Search all `docs/foop/FOOP-*.md` and `docs/foop/FOOP-*.plan.md` for `foolish/` path refs and dead crate refs — per explicit instruction, historical/completed FOOPs are LEFT ALONE (their `foolish/` paths correctly describe the repo state when written); only FOOP-03's own files were updated
        (2026-07-02 14:34)
  - [-] Update docs/ubc1/ engineering docs — remove dead crate refs — NOT done: human instruction is to leave `docs/ubc1` as-is for now. Deferred entirely, not scoped into this FOOP pass.
        (2026-07-02 15:05)
  - [x] Verify: `grep -r "foolish/" --include="*.md" .` shows no matches outside `docs/vintage_legacy/` — RE-SCOPED per human instruction: also excluding `docs/foop/` (historical FOOPs, left alone by design) and `docs/ubc1/`+`docs/ubc0_1/` (left as-is by explicit instruction). With that scoping: clean, aside from agent/session state (`.claude/agent-memory/`, `.omo/notepads/`) which is not project documentation.
        (2026-07-02 15:20)
  - [x] Verify: `grep -r "foolish-web\|foolish-ubcb" --include="*.md" .` shows no matches — RE-SCOPED same as above: clean outside historical FOOP docs / docs/ubc1 / docs/ubc0_1.
        (2026-07-02 15:20)
  - [x] Verify: `grep -r "java\|scala\|maven" --include="*.md" .` shows no matches outside `docs/vintage_legacy/` — RE-SCOPED same as above; remaining hits are either already-correct changelog entries, code-block language-tag documentation, generic examples (AGENTS.md's restricted-actions list), or design-philosophy comparisons in `docs/why/brainstorm.md` (not build instructions) — none are live JVM build references.
        (2026-07-02 15:20)
- [ ] Phase 5: Final verification
  - [ ] `cargo build --workspace` succeeds from repo root
  - [ ] `cargo test --workspace` passes
  - [ ] `cargo clippy --workspace` clean
  - [ ] `cargo fmt --check` clean
  - [ ] No `foolish/` directory exists
  - [ ] Only three crates: `foolish-core/`, `foolish-parser/`, `foolish-cli/`
  - [ ] All documentation reflects new structure
  - [ ] `docs/foop/INDEX.md` is up to date with all FOOPs listed
  - [ ] `docs/foop/scripts/foop_check.py` works from repo root
  - [ ] Commit all changes to `foop-03-repo-cleanup` branch
- [ ] Phase 6: Merge and branch rename
  - [ ] STOP! ASK HUMAN to review all changes before merging
  - [ ] Merge `foop-03-repo-cleanup` to `alpha` in `/home/hcbusy/foolish-rust`
  - [ ] Repair any merge conflicts in `alpha`
  - [ ] Verify: all tests pass in `alpha`
  - [ ] Rename main branch to `jia` on GitHub (requires admin)
  - [ ] Update local clone: `git branch -m main jia && git fetch origin && git branch -u origin/jia jia`
  - [ ] Update branch references in `.github/workflows/`, `README.md`, `AGENTS.md`
  - [ ] Transfer branch protection rules from `main` to `jia`
  - [ ] Notify contributors to update their local clones
- [ ] Cleanup worktree
  - [ ] Check that FOOP-03.plan.md has all but Cleanup checkboxes completed
  - [ ] Remove `/home/hcbusy/tmp/foolish-worktrees/foop-03-repo-cleanup`
