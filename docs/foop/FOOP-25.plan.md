# FOOP-25.plan — The dot search

- [x] Begin work: commit FOOP-25.md and FOOP-25.plan.md to origin, check `begun: [x]` in frontmatter
      (2026-07-22 09:31)
- [x] Create worktree at /home/hcbusy/tmp/foolish-worktrees/foop-25-dot-search with branch `foop/foop-25-dot-search`
      (2026-07-22 09:31)
- [x] (read §Specification of FOOP-25.md — the three evaluation phases and settlement rules)
      (2026-07-22 09:31)
- [ ] Verify existing dot search approval tests pass — run `cargo test -p foolish-ubca --lib -- approval_all` and confirm zero failures
  - [x] Known P1 issue: approval_all fails on signature-only divergence (embedded timestamps + key drift). Content byte-identical. Pre-existing, not caused by FOOP-25.
        (2026-07-22 09:34)
- [x] Write `foolish-ubca/snapshot_tests/input/foop_25_comprehensive.foo` covering:
  - [x] Simple coordinate access: `a.x`
  - [x] Chained deepening: `a.b.c.d`
  - [x] Whitespace tolerance: `a . x`
  - [x] Miss (NK): `a.nonexistent`
  - [x] Contexted follow-up: `a.x&#1`
  - [x] Multiple dots with mixed spacing: `a . b.c . d`
  - [x] Nested branes with dot search through multiple levels
  - [x] Dot search on result of other search operators
      (2026-07-22 09:35)
  - **Finding**: `container?alpha.inner` fails because `parse_regexp_pattern()` consumes
    `alpha.inner` as one regex pattern. Workaround: use separate assignment
    `found_alpha = container?alpha; found_alpha.inner`. This is a parser issue, not a dot
    search issue. Filed as observation — no fix needed for this FOOP.
- [ ] Run `cargo insta test -p foolish-ubca --lib -- foop_25_comprehensive` and verify `.snap.new` output
  - [x] Ran `cargo run -p foolish-cli -- run foop_25_comprehensive.foo` — all 12 test cases correct
        (2026-07-22 09:35)
  - [ ] Generate `.snap.new` via insta (blocked by P1 — signature-only failure on first test)
- [ ] Verify all work is complete in /home/hcbusy/tmp/foolish-worktrees/foop-25-dot-search and committed to `foop/foop-25-dot-search`
- [ ] Merge `foop/foop-25-dot-search` to `alpha`
  - [ ] Check and make sure current foop has, and passes, a "comprehensive" snaptest. Input name: `foop_25_comprehensive.foo` (reserved for this foop). Agent generates and verifies; human gives final signed approval.
  - [ ] STOP! STOP!! STOP!!! ASK HUMAN to check this box before continuing. UNDER NO CIRCUMSTANCES will Agent continue past this point automatically!!
    - [ ] Present human with the `cd /home/hcbusy/tmp/foolish-worktrees/foop-25-dot-search` command and ask them to review snapshots BEFORE checking the parent checkbox.
  - [ ] Cleanup /home/hcbusy/tmp/foolish-worktrees/foop-25-dot-search
    - [ ] Check that .plan.md has all but Cleanup checkboxes completed
    - [ ] Remove /home/hcbusy/tmp/foolish-worktrees/foop-25-dot-search
    - [ ] This is the last sub-task checkbox to be checked in this block
