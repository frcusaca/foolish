# FOOP-10: Anchored search through constanic anchors — Implementation Plan

## Search Step Rule

- [ ] Implement `dereference(anchor)` helper: walk `.target` chain through WOCONSTANIC searches
- [ ] Update anchored `SearchFir` step rule to dispatch on anchor's resolved kind:
  - [ ] CONSTANT brane → local search; miss → NK
  - [ ] WOCONSTANIC/BRANING brane → local search against statement-list; miss → NK (shape is fixed)
  - [ ] WOCONSTANIC search → dereference chain, recurse on result
  - [ ] ECONSTANIC search → result is WOCONSTANIC (wait on chain)
  - [ ] NK → NK propagates
  - [ ] nigh states → wait (Phase 4 only)

## Tests

- [ ] Approval test: `{b = {x = unknown, y = 5}, c = b.y}` → `c = 5`
- [ ] Approval test: `{b = {x = unknown, y = 5}, c = b.q}` → `c = NK`
- [ ] Approval test: `{b = unknown, c = b.x}` → `c = WOCONSTANIC`
- [ ] Approval test: `{b = {x = 5, y = 6}, c = b.q}` → `c = NK`
- [ ] Review and update existing `anchoredSearchOnConstanic.foo` and `anchoredSearchFailsOnConstant.foo`

## Worktree

- [ ] Create worktree at `${HOME}/tmp/foolish-worktrees/4928-foop-01` with branch `foop/01-anchored-constanic-search`
- [ ] Verify all work is complete in `${HOME}/tmp/foolish-worktrees/4928-foop-01` and committed to `foop/01-anchored-constanic-search`
- [ ] Merge `foop/01-anchored-constanic-search` to alpha
