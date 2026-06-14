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
            StepReport::Progress(nyes) if nyes.is_settled() => return Ok(()),
            StepReport::NoProgress => return Ok(()),
            _ => {}
        }
    }
    Ok(())
}

fn proto_to_core_fir(ubca_ref: &FirRef) -> core_fir::Fir {
    proto_to_core_fir_inner(ubca_ref, false)
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
            NormalBraneFirBuilder::new()
                .characterizations(borrowed.as_brane_characterizations().to_vec())
                .statements(stmt_tuples)
                .state(state)
                .build()
        }
        FirKind::Search => {
            if state.is_settled() {
                let ubc = borrowed.core().ubc_children();
                if let Some(result) = ubc.first() {
                    let resolved = proto_to_core_fir_inner(result, preserve_search);
                    if !preserve_search {
                        let resolved_state = result.borrow().core().get_nyes();
                        if resolved_state == Nyes::Constant || resolved_state == Nyes::Independent {
                            return resolved;
                        }
                    }
                    return SearchFirBuilder::new(borrowed.as_search_pattern().unwrap_or(""))
                        .anchored(borrowed.as_search_anchored())
                        .target(resolved)
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
            if state.is_settled() {
                let ubc = borrowed.core().ubc_children();
                if let Some(result) = ubc.first() {
                    let resolved = proto_to_core_fir_inner(result, preserve_search);
                    let resolved_state = result.borrow().core().get_nyes();
                    if !preserve_search
                        && (resolved_state == Nyes::Constant || resolved_state == Nyes::Independent)
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
            if state.is_settled() {
                let ubc = borrowed.core().ubc_children();
                if let Some(result) = ubc.first() {
                    let resolved = proto_to_core_fir_inner(result, preserve_search);
                    let resolved_state = result.borrow().core().get_nyes();
                    if !preserve_search
                        && (resolved_state == Nyes::Constant || resolved_state == Nyes::Independent)
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
            let expr_fir = borrowed
                .core()
                .foolish_children()
                .first()
                .map(|c| proto_to_core_fir_inner(c, true))
                .unwrap_or_else(|| NkFirBuilder::new("empty sf").build());
            StayFoolishFirBuilder::new(expr_fir).state(state).build()
        }
        FirKind::StayFullyFoolish => {
            let expr_fir = borrowed
                .core()
                .foolish_children()
                .first()
                .map(|c| proto_to_core_fir_inner(c, true))
                .unwrap_or_else(|| NkFirBuilder::new("empty sff").build());
            StayFullyFoolishFirBuilder::new(expr_fir)
                .state(state)
                .build()
        }
        FirKind::Concatenation => {
            if state.is_settled() {
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
