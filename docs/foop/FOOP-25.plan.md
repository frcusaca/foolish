# FOOP-25.plan — The dot search

- [ ] Begin work: commit FOOP-25.md and FOOP-25.plan.md to origin, check `begun: [x]` in frontmatter
      (YYYY-MM-DD HH:MM)
- [ ] Create worktree at /home/hcbusy/tmp/foolish-worktrees/foop-25-dot-search with branch `foop/foop-25-dot-search`
- [ ] (read §Specification of FOOP-25.md — the three evaluation phases and settlement rules)
- [ ] Verify existing dot search approval tests pass — run `cargo test -p foolish-ubca --lib -- approval_all` and confirm zero failures
- [ ] Write `foolish-ubca/snapshot_tests/input/foop_25_comprehensive.foo` covering:
  - Simple coordinate access: `a.x`
  - Chained deepening: `a.b.c.d`
  - Whitespace tolerance: `a . x`
  - Miss (NK): `a.nonexistent`
  - Contexted follow-up: `a.x&#1`
  - Multiple dots with mixed spacing: `a . b.c . d`
  - Nested branes with dot search through multiple levels
  - Dot search on result of other search operators
- [ ] Run `cargo insta test -p foolish-ubca --lib -- foop_25_comprehensive` and verify `.snap.new` output
- [ ] Verify all work is complete in /home/hcbusy/tmp/foolish-worktrees/foop-25-dot-search and committed to `foop/foop-25-dot-search`
- [ ] Merge `foop/foop-25-dot-search` to `alpha`
  - [ ] Check and make sure current foop has, and passes, a "comprehensive" snaptest. Input name: `foop_25_comprehensive.foo` (reserved for this foop). Agent generates and verifies; human gives final signed approval.
  - [ ] STOP! STOP!! STOP!!! ASK HUMAN to check this box before continuing. UNDER NO CIRCUMSTANCES will Agent continue past this point automatically!!
    - [ ] Present human with the `cd /home/hcbusy/tmp/foolish-worktrees/foop-25-dot-search` command and ask them to review snapshots BEFORE checking the parent checkbox.
  - [ ] Cleanup /home/hcbusy/tmp/foolish-worktrees/foop-25-dot-search
    - [ ] Check that .plan.md has all but Cleanup checkboxes completed
    - [ ] Remove /home/hcbusy/tmp/foolish-worktrees/foop-25-dot-search
    - [ ] This is the last sub-task checkbox to be checked in this block
