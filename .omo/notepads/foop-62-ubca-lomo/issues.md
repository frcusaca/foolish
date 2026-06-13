# FOOP-62 UBCa LOMO — Issues Found in Failed Implementation

## Critical Semantic Failures (from cross-check mismatches)

### 1. Scope Model: Flat Entries vs. Structural Brane Walk

`setup_brane_scope()` in ubca_proto_evaluator.rs dumps ALL statement names into a flat `entries` Vec. This breaks:
- **Nested scope shadowing**: flat entries cannot properly shadow names in nested branes (scope_shadowing_multiple_levels, identifier_shadowing, named_brane_shadowing)
- **Positional IB search**: UBC walks brane structure to find names before the current statement; flat entries search most-recent-first without positional awareness
- **Cross-brane boundary**: flat scope lets searches "leak" through brane boundaries incorrectly (complex_nested_scope, named_brane_with_search)

### 2. SFF (StayFullyFoolish) Semantics Completely Wrong

- Proto: SFF immediately sets Nyes::Independent, never re-evaluates
- UBC: SFF holds unevaluated expressions that are re-resolved on each access (thunk semantics)
- Affected: sff_basic, sff_resolves_on_each_use, sff_in_assignment_chain, sff_nested, sff_vs_sf_timing_difference, sf_of_sff, sf_sff_nested_combined, complex_sff_in_nested_brane, complex_sff_with_nested_scope

### 3. SF (StayFoolish) Semantics Partially Wrong

- Proto: SF evaluates in Foolishly ignorant scope, but doesn't properly propagate results through scope
- UBC: SF evaluates eagerly, result is "frozen" at evaluation time
- Affected: sf_brane_blocking, sf_blocks_brane_at_assignment_time, sf_of_sff

### 4. Forward References / Econstanic Re-evaluation

evaluate_brane() tries sequential evaluation then re-evaluates constanic statements up to 100 times. Problems:
- reset_searches_proto() doesn't properly handle re-evaluation with new bindings
- Econstanic searches that become resolvable after later bindings appear are not re-evaluated correctly
- Affected: chained_undeclared, cross_scope_reference_chain, complex_brane_with_operations_and_search, foop42_humanizing_sequencer_formatting_exhaustive_aka_hfs

### 5. Anchored Search / Proto-to-Core Conversion Loses Structure

proto_to_core_fir() collapses settled searches to just the result, losing the original search pattern that UBC preserves in its formatted output.
- UBC output: `?(result=..., pattern='...', UNANCHORED)`
- Proto output: just the resolved value (e.g., `42`, `{...}`)
- Affected: head_of_empty, tail_of_empty, offset_access_empty_brane, anchored_seek_positive_boundary, etc.

### 6. Concatenation + Search Interaction

- search_through_concatenation: Proto can't find names through concatenation results
- complex_search_concat_and_seeks: concat result body handling differs
- concatenation_with_unresolved_search: brane state after concat differs

### 7. Head/Tail on Nested Branes

head_tail_nested_brane: Proto returns the full brane for head/tail instead of extracting first/last element. The anchor resolution doesn't unwrap brane structures correctly.

### 8. Deep Nesting Loses Values

regression_deep_nesting_does_not_lose_values: deeply nested branes fail because the evaluator doesn't properly propagate scope through multiple brane levels.
