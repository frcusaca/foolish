# FOOP-7: Constanic Clone — recoordination contract — Implementation Plan

## Core Implementation

- [ ] Implement `constanicClone(original: Fir): Fir` with per-state dispatch
  - [ ] CONSTANT, INDEPENDENT, NK → return original (share, do not clone)
  - [ ] ECONSTANIC → deep copy, reset state to EMBRYONIC
  - [ ] WOCONSTANIC → deep copy with recursively-cloned constanic children, reset to BRANING
  - [ ] Nigh states → throw (caller invariant violation)
- [ ] Implement caller protocol: after `constanicClone`, assign `.parent` on returned clone
- [ ] Integrate into search step: every search result goes through `constanicClone` before assignment to `.target`

## Tests

- [ ] Unit test: `constanicClone` sharing for CONSTANT/INDEPENDENT/NK (same instance returned)
- [ ] Unit test: `constanicClone` produces distinct clone for ECONSTANIC
- [ ] Unit test: `constanicClone` produces distinct clone for WOCONSTANIC with recursive children
- [ ] Approval test: worked example `{y=z, x=y, w=x, v=w+x, u=v+w}` produces documented final-state table
- [ ] Unit test: every SearchFir's `.target` post-step is output of `constanicClone` (not raw reference)

## Worktree

- [ ] Create worktree at `/tmp/foolish-rust-foop7` with branch `foop/7-constanic-clone`
- [ ] Verify all work is complete in `/tmp/foolish-rust-foop7` and committed to `foop/7-constanic-clone`
- [ ] Merge `foop/7-constanic-clone` to alpha
