use foolish_core::fir as core_fir;
use foolish_core::fir::{
    ConcatenationFirBuilder, ConstantIntFirBuilder, FirRef as CoreFirRef, HeadTailFirBuilder,
    IndexFirBuilder, NkFirBuilder, NormalBraneFirBuilder, Nyes, OperatorFirBuilder,
    SearchFirBuilder, StayFoolishFirBuilder, StayFullyFoolishFirBuilder,
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
    let borrowed = ubca_ref.borrow();
    let kind = borrowed.kind();
    let state = borrowed.core().get_nyes();

    match kind {
        FirKind::ConstantInt => ConstantIntFirBuilder::new(borrowed.as_i64().unwrap_or(0))
            .state(state)
            .build(),
        FirKind::Nk => NkFirBuilder::new(borrowed.as_nk_reason().unwrap_or("unknown"))
            .state(state)
            .build(),
        FirKind::Operator => {
            if state == Nyes::Nk {
                return NkFirBuilder::new("operator nk").build();
            }
            if state.is_constanic() {
                let ubc = borrowed.core().ubc_children();
                if let Some(result) = ubc.first() {
                    return proto_to_core_fir(result);
                }
            }
            let op = borrowed.as_op_name().unwrap_or("?").to_string();
            let operand_firs: Vec<core_fir::Fir> = borrowed
                .core()
                .foolish_children()
                .iter()
                .map(proto_to_core_fir)
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
                .map(proto_to_core_fir)
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
                        .map(proto_to_core_fir)
                        .unwrap_or_else(|| NkFirBuilder::new("empty body").build());
                    (name, body_fir)
                })
                .collect();
            NormalBraneFirBuilder::new()
                .statements(stmt_tuples)
                .state(state)
                .build()
        }
        FirKind::Search => SearchFirBuilder::new(borrowed.as_search_pattern().unwrap_or(""))
            .anchored(borrowed.as_search_anchored())
            .state(state)
            .build(),
        FirKind::Index => IndexFirBuilder::new(borrowed.as_index_offset())
            .anchored(borrowed.as_index_anchored())
            .state(state)
            .build(),
        FirKind::HeadTail => HeadTailFirBuilder::new(borrowed.as_headtail_is_head())
            .anchored(borrowed.as_headtail_anchored())
            .state(state)
            .build(),
        FirKind::StayFoolish => {
            let expr_fir = borrowed
                .core()
                .foolish_children()
                .first()
                .map(proto_to_core_fir)
                .unwrap_or_else(|| NkFirBuilder::new("empty sf").build());
            StayFoolishFirBuilder::new(expr_fir).state(state).build()
        }
        FirKind::StayFullyFoolish => {
            let expr_fir = borrowed
                .core()
                .foolish_children()
                .first()
                .map(proto_to_core_fir)
                .unwrap_or_else(|| NkFirBuilder::new("empty sff").build());
            StayFullyFoolishFirBuilder::new(expr_fir)
                .state(state)
                .build()
        }
        FirKind::Concatenation => {
            let elem_firs: Vec<core_fir::Fir> = borrowed
                .core()
                .foolish_children()
                .iter()
                .map(proto_to_core_fir)
                .collect();
            ConcatenationFirBuilder::new()
                .elements(elem_firs)
                .state(state)
                .build()
        }
        FirKind::Unknown => NkFirBuilder::new("unknown fir kind").build(),
    }
}
