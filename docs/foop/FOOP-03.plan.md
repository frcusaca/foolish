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

- [ ] Begin FOOP-03 execution
- [ ] Create worktree at `/home/hcbusy/tmp/foolish-worktrees/foop-03-repo-cleanup` with branch `foop-03-repo-cleanup` from `alpha`
- [ ] Phase 1: Remove JVM artifacts
  - [ ] Delete `.github/workflows/java-tests.yml`
  - [ ] Delete `.github/workflows/scala-tests.yml`
  - [ ] Update `.github/workflows/tests.yml` — remove any JVM cross-validation references
  - [ ] Delete `docs/ubc1/todo/scala-mvp/` directory entirely
  - [ ] Clean `.gitignore` — remove JVM-specific entries (`*.class`, `*.jar`, `*.war`, `*.ear`, `pom.xml.*`, `ivy-*`)
  - [ ] Verify: `grep -r "java\|scala\|maven\|pom\.xml" --include="*.yml" .github/` returns nothing
- [ ] Phase 2: Remove dead Rust crates
  - [ ] Delete `foolish/foolish-web/` directory
  - [ ] Delete `foolish/foolish-ubcb/` directory
  - [ ] Delete `foolish/foolish-ubcb-cli/` directory
  - [ ] Update `foolish/Cargo.toml` — remove `foolish-web`, `foolish-ubcb`, `foolish-ubcb-cli` from workspace members
  - [ ] Verify: `cargo check --workspace` succeeds from `foolish/`
- [ ] Phase 3: Flatten workspace
  - [ ] Move `foolish/Cargo.toml` to repo root (merge with or replace any existing root `Cargo.toml`)
  - [ ] Move `foolish/Cargo.lock` to repo root
  - [ ] Move `foolish/foolish-core/` to repo root
  - [ ] Move `foolish/foolish-parser/` to repo root
  - [ ] Move `foolish/foolish-cli/` to repo root
  - [ ] Delete `foolish/target/` (build cache)
  - [ ] Delete `foolish/mcp.log.*` files
  - [ ] Delete `foolish/.claude/` (redundant)
  - [ ] Delete `foolish/.omo/` (redundant)
  - [ ] Delete empty `foolish/` directory
  - [ ] Verify: `cargo build --workspace` succeeds from repo root
  - [ ] Verify: `cargo test --workspace` passes
- [ ] Phase 4: Update documentation
  - [ ] Rewrite AGENTS.md Build Requirements — remove Java/Scala/Maven/ANTLR, replace with Rust-only
  - [ ] Rewrite AGENTS.md Build Commands — remove all `cd foolish` prefixes, paths now repo-root-relative
  - [ ] Rewrite AGENTS.md Project Structure — three crates: foolish-core (UBCa), foolish-parser, foolish-cli
  - [ ] Rewrite AGENTS.md Architecture section — UBCa as sole reference implementation, FIR, Nyes states (PREMBRYONIC, EMBRYONIC, BRANING, ECONSTANIC, WOCONSTANIC, CONSTANT, INDEPENDENT, NK), constanic terminology, AB/IB semantics
  - [ ] Rewrite AGENTS.md Testing section — remove UBCb refs, update snapshot commands to foolish-core only
  - [ ] Rewrite AGENTS.md Approval Test section — update paths, remove UBCb refs
  - [ ] Rewrite AGENTS.md CLI Usage section — remove `cd foolish`
  - [ ] Update README.md — remove Java/Scala badges, remove Quick Start Java/Scala, remove Maven commands, remove Versioned Documentation table, update build/test commands to repo-root-relative
  - [ ] Update rust_instructions.md — path references
  - [ ] Update docs/DOC_AGENTS.md — path references, remove dead crate refs
  - [ ] Update docs/styleguide.md — path references if any
  - [ ] Update docs/foop/scripts/foop_check.py — verify no stale path refs after flattening
  - [ ] Update docs/foop/INDEX.md — add FOOP-03 entry, add any missing FOOPs (42, 62, 72, 82, 92), update by-status and by-phase sections
  - [ ] Search all `docs/foop/FOOP-*.md` and `docs/foop/FOOP-*.plan.md` for `foolish/` path refs and dead crate refs — add historical notes where needed
  - [ ] Update docs/ubc1/ engineering docs — remove dead crate refs
  - [ ] Verify: `grep -r "foolish/" --include="*.md" .` shows no matches outside `docs/vintage_legacy/`
  - [ ] Verify: `grep -r "foolish-web\|foolish-ubcb" --include="*.md" .` shows no matches
  - [ ] Verify: `grep -r "java\|scala\|maven" --include="*.md" .` shows no matches outside `docs/vintage_legacy/`
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
