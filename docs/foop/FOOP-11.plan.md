# FOOP-11: Search stops at NK — Implementation Plan

## Search Step Rule Updates

- [ ] Update search FIR step rules (`SearchFir`, `IndexFir`, `HeadTailFir`, `CharacterizedRefFir`):
  - [ ] Anchor is NK → search becomes NK
  - [ ] Dereferenced target is NK → search becomes NK
  - [ ] Found statement's body is NK → search becomes NK
- [ ] Verify no `nkPolicy` field is added (behavior is implicit)

## Integration

- [ ] Confirm consistency with FOOP-7: `constanicClone(NK) = NK` (shared, not cloned)
- [ ] Confirm consistency with FOOP-01: NK propagation through constanic anchor chains

## Tests

- [ ] Approval test: `{a = 5 / 0, b = a}` → `b = NK` (search for `a` finds NK)
- [ ] Approval test: `{brane = {x = 5 / 0, y = 7}, result = brane.x}` → `result = NK`
- [ ] Approval test: `{brane = {x = 5 / 0, y = 7}, result = brane.y}` → `result = 7` (NK doesn't contaminate unrelated searches)

## Worktree

- [ ] Create worktree at `${HOME}/tmp/foolish-worktrees/6017-foop-11` with branch `foop/11-search-stops-at-nk`
- [ ] Verify all work is complete in `${HOME}/tmp/foolish-worktrees/6017-foop-11` and committed to `foop/11-search-stops-at-nk`
- [ ] Merge `foop/11-search-stops-at-nk` to alpha
