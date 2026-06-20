use foolish_core::fir as core_fir;
use foolish_core::fir::{
    Alarm, AlarmLevel, AlarmSource, ConcatenationFirBuilder, ConstantIntFirBuilder,
    FirRef as CoreFirRef, HeadTailFirBuilder, IndexFirBuilder, NkFirBuilder, NormalBraneFirBuilder,
    Nyes, OperatorFirBuilder, SearchFirBuilder, StayFoolishFirBuilder, StayFullyFoolishFirBuilder,
};

use crate::compiler::Compiler;
use crate::fir_trait::{FirKind, FirRef, StepReport};
use crate::nyes_ext::NyesExt;

const MAX_STEPS: usize = 10_000;

pub struct UbcaEvaluator;

impl foolish_core::Evaluator for UbcaEvaluator {
    fn evaluate(&self, source: &str) -> Result<Vec<CoreFirRef>, String> {
        let ubca_firs =
            Compiler::compile(source).map_err(|e| format!("Compilation failed: {}", e))?;

        let scope = crate::fir_trait::Scope::empty();
        let mut results = Vec::new();

        for fir_ref in &ubca_firs {
            step_to_settled(fir_ref, &scope).map_err(|e| format!("Evaluation failed: {}", e))?;
            let core_fir = proto_to_core_fir(fir_ref);
            results.push(core_fir::fir_to_ref(core_fir));
        }

        Ok(results)
    }
}

fn step_to_settled(
    fir_ref: &FirRef,
    scope: &crate::fir_trait::Scope,
) -> Result<(), crate::fir_trait::UbcError> {
    for _ in 0..MAX_STEPS {
        let report = crate::step_fir_ref(fir_ref, scope)?;
        match report {
            StepReport::Progress(nyes) if nyes.is_constanic() => return Ok(()),
            StepReport::NoProgress => return Ok(()),
            _ => {}
        }
    }
    Ok(())
}

fn proto_to_core_fir(ubca_ref: &FirRef) -> core_fir::Fir {
    proto_to_core_fir_inner(ubca_ref, false)
}

/// Convert an SFF body expression. Top-level searches get EMBRYONIC state
/// (shown by sequencer). Operator operands get CONSTANT state (hidden).
/// Operators get WOCONSTANIC or CONSTANT state based on operand states.
fn proto_to_core_fir_sff_body(ubca_ref: &FirRef) -> core_fir::Fir {
    let borrowed = ubca_ref.borrow();
    let kind = borrowed.kind();
    match kind {
        FirKind::Search => SearchFirBuilder::new(borrowed.as_search_pattern().unwrap_or(""))
            .anchored(borrowed.as_search_anchored())
            .state(Nyes::Embryonic)
            .build(),
        FirKind::Operator => {
            let op = borrowed.as_op_name().unwrap_or("?").to_string();
            let operand_firs: Vec<core_fir::Fir> = borrowed
                .core()
                .foolish_children()
                .iter()
                .map(|c| proto_to_core_fir_sff_operand(c))
                .collect();
            use foolish_core::fir::FirQueryable;
            let op_state = if operand_firs.iter().any(|f| {
                let s = f.hs_state();
                s == Nyes::Econstanic || s == Nyes::Woconstanic
            }) {
                Nyes::Woconstanic
            } else {
                Nyes::Constant
            };
            OperatorFirBuilder::new(op)
                .operands(operand_firs)
                .state(op_state)
                .build()
        }
        FirKind::ConstantInt => ConstantIntFirBuilder::new(borrowed.as_i64().unwrap_or(0))
            .state(Nyes::Constant)
            .build(),
        FirKind::Nk => NkFirBuilder::new(borrowed.as_nk_reason().unwrap_or("unknown"))
            .state(Nyes::Nk)
            .build(),
        _ => proto_to_core_fir_inner(ubca_ref, true),
    }
}

/// Convert an SFF operator operand. Searches get CONSTANT state (no state shown).
fn proto_to_core_fir_sff_operand(ubca_ref: &FirRef) -> core_fir::Fir {
    let borrowed = ubca_ref.borrow();
    let kind = borrowed.kind();
    match kind {
        FirKind::Search => SearchFirBuilder::new(borrowed.as_search_pattern().unwrap_or(""))
            .anchored(borrowed.as_search_anchored())
            .state(Nyes::Econstanic)
            .build(),
        FirKind::ConstantInt => ConstantIntFirBuilder::new(borrowed.as_i64().unwrap_or(0))
            .state(Nyes::Constant)
            .build(),
        FirKind::Nk => NkFirBuilder::new(borrowed.as_nk_reason().unwrap_or("unknown"))
            .state(Nyes::Nk)
            .build(),
        _ => proto_to_core_fir_inner(ubca_ref, true),
    }
}

fn anchor_to_core_fir(ubca_ref: &FirRef) -> core_fir::Fir {
    let borrowed = ubca_ref.borrow();
    let kind = borrowed.kind();
    let state = borrowed.core().get_nyes();

    if kind == FirKind::Search {
        return SearchFirBuilder::new(borrowed.as_search_pattern().unwrap_or(""))
            .anchored(borrowed.as_search_anchored())
            .state(state)
            .build();
    }

    proto_to_core_fir_inner(ubca_ref, true)
}

fn proto_to_core_fir_inner(ubca_ref: &FirRef, preserve_search: bool) -> core_fir::Fir {
    let borrowed = ubca_ref.borrow();
    let kind = borrowed.kind();
    let state = borrowed.core().get_nyes();

    match kind {
        FirKind::ConstantInt => ConstantIntFirBuilder::new(borrowed.as_i64().unwrap_or(0))
            .state(state)
            .build(),
        FirKind::Nk => {
            let reason = borrowed.as_nk_reason().unwrap_or("unknown");
            let mut builder = NkFirBuilder::new(reason).state(state);
            if reason == "division by zero" {
                builder = builder.alarm(Alarm {
                    level: AlarmLevel::Mild,
                    code: "DIV-BY-ZERO".to_string(),
                    message: "Division by zero produces NK".to_string(),
                    source: AlarmSource::Evaluator,
                });
            }
            builder.build()
        }
        FirKind::Operator => {
            // Unwrap to the result when the operator successfully computed
            // a constant value.
            if state == Nyes::Constant {
                let ubc = borrowed.core().ubc_children();
                if let Some(result) = ubc.first() {
                    return proto_to_core_fir_inner(result, preserve_search);
                }
            }
            // When the operator itself computed NK (e.g. division by zero)
            // AND none of its operands are NK (meaning NK was computed here,
            // not propagated from an operand), unwrap to the NK result.
            // If any operand is NK, keep the operator wrapper so the
            // humanizer can display all operands and state.
            if state == Nyes::Nk {
                let any_operand_nk = borrowed
                    .core()
                    .foolish_children()
                    .iter()
                    .any(|c| c.borrow().core().get_nyes() == Nyes::Nk);
                if !any_operand_nk {
                    let ubc = borrowed.core().ubc_children();
                    if let Some(result) = ubc.first() {
                        return proto_to_core_fir_inner(result, preserve_search);
                    }
                }
            }
            let op = borrowed.as_op_name().unwrap_or("?").to_string();
            let operand_firs: Vec<core_fir::Fir> = borrowed
                .core()
                .foolish_children()
                .iter()
                .map(|c| proto_to_core_fir_inner(c, preserve_search))
                .collect();
            OperatorFirBuilder::new(op)
                .operands(operand_firs)
                .state(state)
                .build()
        }
        FirKind::Statement => {
            let name = borrowed.as_stmt_name().map(|s| s.to_string());
            let body_fir = borrowed
                .core()
                .foolish_children()
                .first()
                .map(|c| proto_to_core_fir_inner(c, preserve_search))
                .unwrap_or_else(|| NkFirBuilder::new("empty statement").build());
            NormalBraneFirBuilder::new()
                .statement(name, body_fir)
                .state(state)
                .build()
        }
        FirKind::Brane => {
            let stmt_tuples: Vec<(Option<String>, core_fir::Fir)> = borrowed
                .core()
                .foolish_children()
                .iter()
                .map(|c| {
                    let cb = c.borrow();
                    let name = cb.as_stmt_name().map(|s| s.to_string());
                    let body_fir = cb
                        .core()
                        .foolish_children()
                        .first()
                        .map(|c| proto_to_core_fir_inner(c, preserve_search))
                        .unwrap_or_else(|| NkFirBuilder::new("empty body").build());
                    (name, body_fir)
                })
                .collect();
            // Recompute brane state from converted children: if any child body
            // is ECONSTANIC or WOCONSTANIC, the brane is WOCONSTANIC.
            let mut effective_state = state;
            if state == Nyes::Constant || state == Nyes::Independent {
                use foolish_core::fir::FirQueryable;
                for (_, body) in &stmt_tuples {
                    let body_state = body.hs_state();
                    if body_state == Nyes::Econstanic || body_state == Nyes::Woconstanic {
                        effective_state = Nyes::Woconstanic;
                        break;
                    }
                    if body_state == Nyes::Nk {
                        effective_state = Nyes::Nk;
                        break;
                    }
                }
            }
            NormalBraneFirBuilder::new()
                .characterizations(borrowed.as_brane_characterizations().to_vec())
                .statements(stmt_tuples)
                .state(effective_state)
                .build()
        }
        FirKind::Search => {
            if state.is_constanic() {
                let ubc = borrowed.core().ubc_children();
                if let Some(result) = ubc.first() {
                    // When the ubc_child is a settled search whose own ubc_child
                    // is a complex type (Brane, Operator, SF, SFF), this search
                    // came from unwrapping an SF value. UBC preserves the search
                    // wrapper in this case rather than resolving to the final value.
                    let result_borrowed = result.borrow();
                    if result_borrowed.kind() == FirKind::Search
                        && result_borrowed.core().get_nyes().is_constanic()
                    {
                        let inner_ubc = result_borrowed.core().ubc_children();
                        let has_complex = inner_ubc.first().map_or(false, |r| {
                            let rb = r.borrow();
                            let is_complex_type = matches!(
                                rb.kind(),
                                FirKind::Brane
                                    | FirKind::Operator
                                    | FirKind::StayFoolish
                                    | FirKind::StayFullyFoolish
                            );
                            let has_resolved_value = rb.core().ubc_children().first().is_some();
                            is_complex_type && !has_resolved_value
                        });
                        if has_complex {
                            let inner_fir = SearchFirBuilder::new(
                                result_borrowed.as_search_pattern().unwrap_or(""),
                            )
                            .anchored(result_borrowed.as_search_anchored())
                            .state(Nyes::Econstanic)
                            .build();
                            drop(result_borrowed);
                            return SearchFirBuilder::new(
                                borrowed.as_search_pattern().unwrap_or(""),
                            )
                            .anchored(borrowed.as_search_anchored())
                            .result(inner_fir)
                            .state(Nyes::Woconstanic)
                            .build();
                        }
                        // Simple result (ConstantInt/NK): build inner search with resolved value.
                        // For other cases (Search chains), fall through to the normal path
                        // which correctly wraps in the outer search.
                        let first_inner_kind = inner_ubc.first().map(|r| r.borrow().kind());
                        let has_simple = first_inner_kind
                            .map_or(false, |k| matches!(k, FirKind::ConstantInt | FirKind::Nk));
                        if has_simple {
                            let inner_result_fir =
                                proto_to_core_fir_inner(inner_ubc.first().unwrap(), false);
                            let inner_search = SearchFirBuilder::new(
                                result_borrowed.as_search_pattern().unwrap_or(""),
                            )
                            .anchored(result_borrowed.as_search_anchored())
                            .result(inner_result_fir)
                            .state(result_borrowed.core().get_nyes())
                            .build();
                            drop(result_borrowed);
                            return inner_search;
                        }
                    }
                    drop(result_borrowed);

                    let resolved = proto_to_core_fir_inner(result, preserve_search);
                    if !preserve_search {
                        let resolved_state = result.borrow().core().get_nyes();
                        if resolved_state == Nyes::Constant || resolved_state == Nyes::Independent {
                            if let Some(sf_pat) = borrowed.as_sf_inner_pattern() {
                                return SearchFirBuilder::new(sf_pat)
                                    .result(resolved)
                                    .state(resolved_state)
                                    .build();
                            }
                            return resolved;
                        }
                    }
                    return SearchFirBuilder::new(borrowed.as_search_pattern().unwrap_or(""))
                        .anchored(borrowed.as_search_anchored())
                        .result(resolved)
                        .state(state)
                        .build();
                }
            }
            SearchFirBuilder::new(borrowed.as_search_pattern().unwrap_or(""))
                .anchored(borrowed.as_search_anchored())
                .state(state)
                .build()
        }
        FirKind::Index => {
            if state.is_constanic() {
                let ubc = borrowed.core().ubc_children();
                if let Some(result) = ubc.first() {
                    let resolved = proto_to_core_fir_inner(result, preserve_search);
                    let resolved_state = result.borrow().core().get_nyes();
                    let result_kind = result.borrow().kind();
                    // Unwrap when: constant/independent, OR the result is a Brane
                    // (index into a brane returns the brane itself, not the index wrapper)
                    if !preserve_search
                        && (resolved_state == Nyes::Constant
                            || resolved_state == Nyes::Independent
                            || result_kind == FirKind::Brane)
                    {
                        return resolved;
                    }
                    let mut builder = IndexFirBuilder::new(borrowed.as_index_offset())
                        .anchored(borrowed.as_index_anchored())
                        .state(state);
                    if borrowed.as_index_anchored() {
                        if let Some(anchor_ref) = borrowed.core().foolish_children().first() {
                            builder = builder.anchor(anchor_to_core_fir(anchor_ref));
                        }
                    }
                    return builder.build();
                }
            }
            let mut builder = IndexFirBuilder::new(borrowed.as_index_offset())
                .anchored(borrowed.as_index_anchored())
                .state(state);
            if borrowed.as_index_anchored() {
                if let Some(anchor_ref) = borrowed.core().foolish_children().first() {
                    builder = builder.anchor(anchor_to_core_fir(anchor_ref));
                }
            }
            builder.build()
        }
        FirKind::HeadTail => {
            if state.is_constanic() {
                let ubc = borrowed.core().ubc_children();
                if let Some(result) = ubc.first() {
                    let resolved = proto_to_core_fir_inner(result, preserve_search);
                    let resolved_state = result.borrow().core().get_nyes();
                    let result_kind = result.borrow().kind();
                    if !preserve_search
                        && (resolved_state == Nyes::Constant
                            || resolved_state == Nyes::Independent
                            || result_kind == FirKind::Brane)
                    {
                        return resolved;
                    }
                    let mut builder = HeadTailFirBuilder::new(borrowed.as_headtail_is_head())
                        .anchored(borrowed.as_headtail_anchored())
                        .state(state);
                    if borrowed.as_headtail_anchored() {
                        if let Some(anchor_ref) = borrowed.core().foolish_children().first() {
                            builder = builder.anchor(anchor_to_core_fir(anchor_ref));
                        }
                    }
                    return builder.build();
                }
            }
            // No ubc_child (NK from empty brane): preserve wrapper with anchor
            let mut builder = HeadTailFirBuilder::new(borrowed.as_headtail_is_head())
                .anchored(borrowed.as_headtail_anchored())
                .state(state);
            if borrowed.as_headtail_anchored() {
                if let Some(anchor_ref) = borrowed.core().foolish_children().first() {
                    builder = builder.anchor(anchor_to_core_fir(anchor_ref));
                }
            }
            builder.build()
        }
        FirKind::StayFoolish => {
            let inner = borrowed.core().foolish_children();
            let inner_ref = inner.first();
            if let Some(expr_ref) = inner_ref {
                let expr_borrowed = expr_ref.borrow();
                if expr_borrowed.kind() == FirKind::Search
                    && expr_borrowed.core().get_nyes().is_constanic()
                {
                    let ubc = expr_borrowed.core().ubc_children();
                    if let Some(result) = ubc.first() {
                        let result_kind = result.borrow().kind();
                        if result_kind == FirKind::Brane
                            || result_kind == FirKind::Operator
                            || result_kind == FirKind::StayFoolish
                            || result_kind == FirKind::StayFullyFoolish
                        {
                            if result.borrow().core().ubc_children().first().is_some() {
                                let inner_result_fir = proto_to_core_fir_inner(result, false);
                                return SearchFirBuilder::new(
                                    expr_borrowed.as_search_pattern().unwrap_or(""),
                                )
                                .anchored(expr_borrowed.as_search_anchored())
                                .result(inner_result_fir)
                                .state(expr_borrowed.core().get_nyes())
                                .build();
                            }
                            // A Brane or other complex type that is itself
                            // constanic IS the value — no ubc_children needed.
                            if result.borrow().core().get_nyes().is_constanic() {
                                let inner_result_fir = proto_to_core_fir_inner(result, false);
                                return SearchFirBuilder::new(
                                    expr_borrowed.as_search_pattern().unwrap_or(""),
                                )
                                .anchored(expr_borrowed.as_search_anchored())
                                .result(inner_result_fir)
                                .state(expr_borrowed.core().get_nyes())
                                .build();
                            }
                            let search_fir = SearchFirBuilder::new(
                                expr_borrowed.as_search_pattern().unwrap_or(""),
                            )
                            .anchored(expr_borrowed.as_search_anchored())
                            .state(Nyes::Econstanic)
                            .build();
                            return StayFoolishFirBuilder::new(search_fir)
                                .state(Nyes::Woconstanic)
                                .build();
                        }
                        if result_kind == FirKind::Search {
                            let inner_borrowed = result.borrow();
                            let inner_fir = SearchFirBuilder::new(
                                inner_borrowed.as_search_pattern().unwrap_or(""),
                            )
                            .anchored(inner_borrowed.as_search_anchored())
                            .state(Nyes::Econstanic)
                            .build();
                            drop(inner_borrowed);
                            let outer_search = SearchFirBuilder::new(
                                expr_borrowed.as_search_pattern().unwrap_or(""),
                            )
                            .anchored(expr_borrowed.as_search_anchored())
                            .result(inner_fir)
                            .state(Nyes::Woconstanic)
                            .build();
                            return StayFoolishFirBuilder::new(outer_search)
                                .state(Nyes::Woconstanic)
                                .build();
                        }
                        if result_kind == FirKind::ConstantInt || result_kind == FirKind::Nk {
                            let inner_result_fir = proto_to_core_fir_inner(result, false);
                            return SearchFirBuilder::new(
                                expr_borrowed.as_search_pattern().unwrap_or(""),
                            )
                            .anchored(expr_borrowed.as_search_anchored())
                            .result(inner_result_fir)
                            .state(expr_borrowed.core().get_nyes())
                            .build();
                        }
                    }
                }
            }
            let expr_fir = inner_ref
                .map(|c| proto_to_core_fir_inner(c, true))
                .unwrap_or_else(|| NkFirBuilder::new("empty sf").build());
            StayFoolishFirBuilder::new(expr_fir).state(state).build()
        }
        FirKind::StayFullyFoolish => {
            // In UBC, SFF stores the expression with searches at EMBRYONIC
            // (not evaluated). Force inner searches to EMBRYONIC state.
            let expr_fir = borrowed
                .core()
                .foolish_children()
                .first()
                .map(|c| proto_to_core_fir_sff_body(c))
                .unwrap_or_else(|| NkFirBuilder::new("empty sff").build());
            StayFullyFoolishFirBuilder::new(expr_fir)
                .state(state)
                .build()
        }
        FirKind::Concatenation => {
            if state.is_constanic() {
                let ubc = borrowed.core().ubc_children();
                if let Some(result) = ubc.first() {
                    return proto_to_core_fir_inner(result, preserve_search);
                }
            }
            let elem_firs: Vec<core_fir::Fir> = borrowed
                .core()
                .foolish_children()
                .iter()
                .map(|c| proto_to_core_fir_inner(c, preserve_search))
                .collect();
            ConcatenationFirBuilder::new()
                .elements(elem_firs)
                .state(state)
                .build()
        }
        FirKind::Unknown => NkFirBuilder::new("unknown fir kind").build(),
    }
}
