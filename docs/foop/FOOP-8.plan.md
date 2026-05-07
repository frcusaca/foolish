# FOOP-8: FIRs are mutable; parent pointers are post-clone — Implementation Plan

## Fir.scala Rewrite

- [ ] Convert `Fir` base trait to use `var state: Nyes` and `var parent: Option[Fir]`
- [ ] Convert each FIR variant to declare mutable `state`, `parent`, and variant-specific mutable fields
- [ ] Convert `SearchFir` to use `var target: Option[Fir]`
- [ ] Update all existing FIR variants to follow the new pattern

## Equality and Structural Comparison

- [ ] Decide and document: override `equals`/`hashCode` vs dedicated `structurallyEquivalent` method
- [ ] Implement chosen approach

## Circe Serialization

- [ ] Implement custom Circe codec that excludes `parent` on encode
- [ ] On decode, default `parent` to `None`; consumer re-establishes parent pointers post-decode
- [ ] Update `FirRoundtripTest` to use structural equivalence instead of default `==`

## Tests

- [ ] Unit test: `parent` excluded from JSON (encode FIR with parent set, decode → parent is None)
- [ ] Unit test: `step()` mutates in place (same instance returned)
- [ ] Roundtrip tests continue to pass with structural comparison

## Worktree

- [ ] Create worktree at `${HOME}/tmp/foolish-worktrees/7295-foop-8` with branch `foop/8-mutable-fir`
- [ ] Verify all work is complete in `${HOME}/tmp/foolish-worktrees/7295-foop-8` and committed to `foop/8-mutable-fir`
- [ ] Merge `foop/8-mutable-fir` to alpha
