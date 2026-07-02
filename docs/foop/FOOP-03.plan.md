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
- [ ] Phase 1: Remove JVM artifacts
  - [ ] Delete `.github/workflows/java-tests.yml`
  - [ ] Delete `.github/workflows/scala-tests.yml`
  - [ ] Update `.github/workflows/tests.yml` — remove any JVM cross-validation references
  - [ ] Delete `docs/ubc1/todo/scala-mvp/` directory entirely
  - [ ] Clean `.gitignore` — remove JVM-specific entries (`*.class`, `*.jar`, `*.war`, `*.ear`, `pom.xml.*`, `ivy-*`)
  - [ ] Verify: `grep -r "java\|scala\|maven\|pom\.xml" --include="*.yml" .github/` returns nothing
  - [ ] Remove Java/Scala/Cross-Validation CI badges from README.md (deferred from Phase 4 — same underlying JVM removal)
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
  - [-] Delete `foolish/.claude/` — NOT deleted. Verified NOT redundant: differs materially from root's `.claude/settings.json` (narrower, Rust-only permission set vs. root's broader Java/Maven-era + plugin config). Left in place at `foolish/.claude/`. Disposition open — ask human before deleting or merging.
        (2026-07-02 14:32)
  - [-] Delete `foolish/.omo/` — NOT deleted. Verified NOT redundant: `foolish/.omo/tasks/` holds 5 task JSON records with IDs absent from root's `.omo/tasks/`, and `foolish/.omo/notepads/` has FOOP-62-specific session notes. Left in place at `foolish/.omo/`. Disposition open — ask human before deleting or merging.
        (2026-07-02 14:32)
  - [ ] Delete empty `foolish/` directory — BLOCKED: `foolish/` is not empty (`.claude/` and `.omo/` remain per above). Revisit once their disposition is decided.
  - [x] Verify: `cargo build --workspace` succeeds from repo root
        (2026-07-02 14:34)
  - [ ] Verify: `cargo test --workspace` passes — NOT run as a gate this session (tests use non-standard insta/signature machinery; human asked to check script output directly instead — see `verify_signatures` runs in session log). 2 pre-existing snapshot/test issues carried over from baseline `alpha`, unrelated to this Phase.
- [ ] Phase 4: Update documentation
  - [ ] Rewrite AGENTS.md Build Requirements — remove Java/Scala/Maven/ANTLR, replace with Rust-only (deferred to Phase 1 JVM-removal pass)
  - [x] Rewrite AGENTS.md Build Commands — workspace-root path now repo-root-relative (no `cd foolish` prefixes existed; fixed the one absolute `/home/hcbusy/foolish-rust/foolish` root-path sentence instead)
        (2026-07-02 14:34)
  - [ ] Rewrite AGENTS.md Project Structure — crates: foolish-core, foolish-ubca, foolish-parser, foolish-cli (four, not three — see note above)
  - [ ] Rewrite AGENTS.md Architecture section — UBCa as reference implementation, FIR, Nyes states, constanic terminology, AB/IB semantics (BLOCKED on FOOP-62 retirement gate — foolish-core is not yet fully retired, see FOOP-62.plan.md backburner note)
  - [x] Rewrite AGENTS.md Testing section — corrected stale `foolish-ubcb` snapshot references (left over from before Phase 2 deleted that crate) to `foolish-ubca`
        (2026-07-02 14:34)
  - [x] Rewrite AGENTS.md Approval Test section — updated paths (workspace-root, signature-verification walkthrough) and stale UBCb refs
        (2026-07-02 14:34)
  - [x] Rewrite AGENTS.md CLI Usage section — no `cd foolish` prefixes existed here; nothing to change
        (2026-07-02 14:34)
  - [x] Update README.md — corrected directory reference and stale `foolish-ubcb-cli` test commands (now `foolish-ubca`); build/test commands now repo-root-relative
        (2026-07-02 14:34)
  - [ ] Update README.md — remove Java/Scala badges, Quick Start Java/Scala, Maven commands, Versioned Documentation table (deferred to Phase 1 JVM-removal pass — badges/workflows still exist on disk)
  - [x] Update rust_instructions.md — path references
        (2026-07-02 14:34)
  - [ ] Update docs/DOC_AGENTS.md — path references, remove dead crate refs
  - [ ] Update docs/styleguide.md — path references if any
  - [ ] Update docs/foop/scripts/foop_check.py — verify no stale path refs after flattening
  - [ ] Update docs/foop/INDEX.md — add FOOP-03 entry, add any missing FOOPs (42, 62, 72, 82, 92), update by-status and by-phase sections
  - [x] Search all `docs/foop/FOOP-*.md` and `docs/foop/FOOP-*.plan.md` for `foolish/` path refs and dead crate refs — per explicit instruction, historical/completed FOOPs are LEFT ALONE (their `foolish/` paths correctly describe the repo state when written); only FOOP-03's own files were updated
        (2026-07-02 14:34)
  - [ ] Update docs/ubc1/ engineering docs — remove dead crate refs (these are largely Java/Scala historical docs — needs a closer look to separate historical from live content)
  - [ ] Verify: `grep -r "foolish/" --include="*.md" .` shows no matches outside `docs/vintage_legacy/` — NOT clean yet: `docs/ubc1/` (non-vintage_legacy) still has historical Java/Scala path refs; historical `docs/foop/` refs also remain by design (see above), so this verify step as originally worded is too strict — needs re-scoping
  - [ ] Verify: `grep -r "foolish-web\|foolish-ubcb" --include="*.md" .` shows no matches — NOT clean yet: historical FOOP docs and docs/ubc1/ still reference them, by design
  - [ ] Verify: `grep -r "java\|scala\|maven" --include="*.md" .` shows no matches outside `docs/vintage_legacy/` — NOT run (Phase 1 JVM removal not done)
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
