# FOOP-9: Operators are brane-like FIRs with positional unnamed operands — Implementation Plan

## Compiler Changes (Phase 1)

- [ ] Remove `BinaryOpFir` and `UnaryOpFir` from `Fir.scala`
- [ ] Add `OperatorFir(op: String, operands: List[Fir])` with mutable `state` and `parent`
- [ ] Update `Compiler.compileExpr` to emit `OperatorFir` for `BinaryExprAstn` and `UnaryExprAstn`
- [ ] Decide and document: arity-by-length vs `@unary` tag for op disambiguation
- [ ] Update Circe codecs for the new `OperatorFir` variant
- [ ] Add `OperatorFir` roundtrip test to `FirRoundtripTest`

## Shared Container Logic

- [ ] Extract shared children-stepping logic (trait or helper) usable by both `NormalBraneFir` and `OperatorFir`
- [ ] Implement `OperatorFir.selfStep`: scalar reduction when all operands CONSTANT/INDEPENDENT
- [ ] Decide and document: result field vs identity mutation after reduction

## Search Transparency

- [ ] Implement `immediateBrane(fir)` helper: walk up through `OperatorFir` nodes to nearest `NormalBraneFir`
- [ ] Update search step rules to skip `OperatorFir` when identifying IB

## Tests

- [ ] Unit test: `1 + 2` compiles to `OperatorFir("+", [Const(1), Const(2)])`
- [ ] Unit test: `-42` compiles to `OperatorFir("-@unary", [Const(42)])`
- [ ] Approval test: `operatorSearchTransparency.foo` — `#-2 + #-1` resolves to `12`
- [ ] Verify existing arithmetic approval tests still pass

## Worktree

- [ ] Create worktree at `/tmp/foolish-rust-foop9` with branch `foop/9-operator-fir`
- [ ] Verify all work is complete in `/tmp/foolish-rust-foop9` and committed to `foop/9-operator-fir`
- [ ] Merge `foop/9-operator-fir` to alpha
