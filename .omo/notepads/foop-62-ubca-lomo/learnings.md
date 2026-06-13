# FOOP-62 UBCa LOMO — Learnings from Failed Implementation

## Architecture Overview (what was built)

The failed `foolish-ubca` crate implements a "ProtoBrane" architecture:

- **fir_trait.rs**: `trait Fir` — dyn-dispatch surface. Every FIR node embeds a `ProtoBrane`. `FirRef = Rc<RefCell<dyn Fir>>`.
- **fir_kinds.rs**: Concrete FIR structs (BraneFir, StatementFir, OperatorFir, SearchFir, IndexFir, HeadTailFir, StayFoolishFir, StayFullyFoolishFir, ConcatenationFir, ConstantIntFir, NkFir). Each implements `trait Fir`.
- **proto_brane.rs**: `ProtoBrane` — two-store topology carrier (foolish_children = fixed source structure, ubc_children = evaluation results). Has parent Weak refs, task queue, NYES state.
- **scope.rs**: `Scope` — capability surface for name resolution. Has current_brane, current_stmt_idx, Ignorance enum, alarm sink, flat entries Vec.
- **step.rs**: `step_fir_ref()` — free function driving single-step evaluation. Processes task queue then calls fir_op_step().
- **compiler.rs**: `compile()` — AST to ProtoBrane FirRef trees. Wires parent pointers.
- **ubca_proto_evaluator.rs**: `UbcaProtoEvaluator` — the real UBCa evaluator using ProtoBrane.
- **ubca_snapshot_tester.rs**: `UbcaEvaluator` — delegates to existing UBC (just calls Compiler::compile + ubc::run_to_completion).
- **cross_check.rs**: Cross-validation: compares UBC vs UBCa-delegates (passes) and UBC vs UBCa-Proto (105/142 failures).
- **fir_queryable_adapter.rs**: Adapter wrapping UBCa FirRef as dyn FirQueryable.

## Test Results Summary

- 67 total lib tests: 66 pass, 1 fails (cross_check_proto_all)
- cross_check_all (UBCa-delegates) PASSES
- approval_all PASSES (snapshot tests using delegation wrapper)
- cross_check_proto_all (UBCa-Proto vs UBC) 105/142 MISMATCH

## Key Insight

The crate has TWO evaluators:
1. UbcaEvaluator — delegates to foolish_core::ubc::run_to_completion() (works perfectly)
2. UbcaProtoEvaluator — uses new ProtoBrane architecture (broken on 74% of test cases)
