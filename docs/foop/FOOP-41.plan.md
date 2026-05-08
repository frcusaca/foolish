# FOOP-(41): UBCb — Message-passing brane computer; SPA1 parity plan — Implementation Plan

## CP-0: Parser and FIR (shared with UBC)

- [ ] Share UBC's parser, compiler, and FIR algebra
- [ ] Add message-passing fields to FIR types (LUID, message queue)
- [ ] Verify FIR roundtrip tests pass (shared with UBC)
- [ ] Governing FOOPs: FOOP-2, FOOP-4, FOOP-5, FOOP-9, FOOP-12

## CP-1: Basic evaluation (new UBCb FOOPs needed)

- [ ] Write UBCb Message Protocol FOOP (new FOOP to be created)
- [ ] Implement brane stepping via messages (no search, no constanic cloning)
- [ ] Implement literal value propagation
- [ ] Implement identification resolution within single brane
- [ ] Implement arithmetic reduction (all operands constant)
- [ ] UBCb produces identical output to UBC on literal-only branes

## CP-2: Search and constanic coordination (new UBCb FOOPs needed)

- [ ] Write UBCb Constanic Coordination FOOP (new FOOP to be created)
- [ ] Implement search resolution via messages
- [ ] Implement wake-up message queue and dependency tracking
- [ ] Implement constanic cloning asynchronously
- [ ] UBCb passes all 60+ Phase 2 approval tests

## CP-3: Concatenation (new UBCb FOOP needed)

- [ ] Write UBCb Concatenation Protocol FOOP (new FOOP to be created)
- [ ] Implement concatenation merge via message-passing
- [ ] UBCb passes all Phase 3 concatenation tests

## CP-4: Full SPA1 parity

- [ ] UBCb passes complete SPA1 test suite
- [ ] Cross-validation: byte-for-byte comparison with UBC approved baselines
- [ ] Decide: shared CLI binary with VM flag, or separate binary?

## New FOOPs to Create (deferred)

- [ ] Create "UBCb Message Protocol" FOOP — message types, channels, scheduling
- [ ] Create "UBCb Constanic Coordination" FOOP — wake-up queue, dependency tracking
- [ ] Create "UBCb Concatenation Protocol" FOOP — message-driven merge

## Worktree

- [ ] Create worktree at `${HOME}/tmp/foolish-worktrees/5394-foop-14` with branch `foop/14-ubcb-spa1`
- [ ] Verify all work is complete in `${HOME}/tmp/foolish-worktrees/5394-foop-14` and committed to `foop/14-ubcb-spa1`
- [ ] Merge `foop/14-ubcb-spa1` to alpha
