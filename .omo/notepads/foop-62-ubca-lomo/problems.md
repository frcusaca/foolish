# FOOP-62 UBCa LOMO — Pitfalls to Avoid in New Implementation

## 1. DO NOT Use Flat Entries for Scope

The biggest architectural mistake. Flat entries Vec loses positional information. Must use structural brane walk:
- IB search: walk current brane's foolish_children backward from current_stmt_idx
- AB search: walk parent_brane chain
- Forward bindings: a name at statement N is visible to statement N+1, N+2, etc. but NOT to N-1

## 2. DO NOT Collapse Search Results in Formatter

UBC preserves search patterns in output: `?(result=X, pattern='^foo$', UNANCHORED)`. The formatter must show both the search pattern AND the result. Proto evaluator's proto_to_core_fir() incorrectly collapses this.

## 3. SFF Must Be Lazy (Thunk-Like)

SFF expressions must NOT be evaluated at definition time. They must be re-evaluated on each access. This is fundamental to Foolish semantics — SFF creates a "template" that resolves in the caller's context.

## 4. SF Must Freeze at Evaluation Time

SF evaluates eagerly in Foolishly-ignorant scope, then the result is frozen. Subsequent accesses return the frozen value, not re-evaluate.

## 5. Econstanic Re-evaluation Must Be Correct

The "forward reference" pattern requires:
- First pass: evaluate each statement; some searches may find Econstanic (name not yet defined)
- Second pass: re-evaluate statements whose searches failed, because later-defined names may now be available
- Must handle chains: a→b→c where c is defined first

## 6. Parent Pointers Must Be Used for AB Walk

The implementation wires parent pointers but never uses them during evaluation. The evaluator must use get_parent_brane() to walk up the tree for name resolution.

## 7. Scope Must Be Positional

Scope must carry (brane, stmt_idx) to know which statements are "before" the current one. The flat entries model throws this away.

## 8. Proto-to-Core Conversion Must Preserve Search Structure

When converting ProtoBrane results to foolish_core::Fir for formatting, settled searches must remain as Search nodes with both pattern and result, not collapse to just the result.

## 9. Avoid RefCell Borrow Conflicts

The note in setup_brane_scope says: "Uses flat entries only (no current_brane) to avoid RefCell borrow conflicts when step_fir_ref recurses into children that search the scope." This is a real problem — need to be careful about holding borrows while recursing. Solution: clone Rc<FirRef> references before borrowing, or restructure to avoid holding borrows across recursive calls.

## 10. Code Quality Issues

- 9 compiler warnings (unused imports, dead code, unused variables)
- resolve_child_for_value() in fir_kinds.rs is never used
- cross_check.rs has unused imports and dead code
- Proto evaluator's step.rs has stuck-child logic that may prematurely pop tasks
