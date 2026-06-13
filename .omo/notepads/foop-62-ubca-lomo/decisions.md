# FOOP-62 UBCa LOMO — Key Architectural Decisions Made

## Decision 1: ProtoBrane as Universal Topology Carrier

Every FIR node embeds a ProtoBrane with two stores (foolish_children for source structure, ubc_children for eval results). This is sound in principle — it provides a uniform interface. But the implementation doesn't properly separate "source structure" from "evaluation context".

## Decision 2: Dyn-Dispatch via trait Fir

Using `Rc<RefCell<dyn Fir>>` for FirRef provides flexibility but causes RefCell borrow conflicts during recursive evaluation. The implementation notes this problem in setup_brane_scope.

## Decision 3: Free Function step_fir_ref() Instead of Trait Method

The step function is a free function to ensure transient borrows. This is correct — borrowing Rc<RefCell<dyn Fir>>, doing work, then releasing before next call avoids holding borrows across recursive calls.

## Decision 4: Flat Entries Scope Model

The biggest wrong decision. Scope uses Vec<(String, FirRef)> instead of structural brane walk. This fundamentally breaks nested scope semantics.

## Decision 5: Two Evaluators

Having UbcaEvaluator (delegates to UBC) and UbcaProtoEvaluator (new architecture) as separate implementations. The delegation approach works but defeats the purpose of building a new evaluator.

## Decision 6: ProtoBrane Compiler Wires Parent Pointers

Parent pointers are wired during compilation but never used for AB (ancestral brane) search. This is unused infrastructure.
