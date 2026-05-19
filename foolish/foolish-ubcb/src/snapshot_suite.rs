use foolish_core::{clone_steppable, FirRef, Evaluator};
use foolish_core::sequencer::{format_fir_simple_with_indent, HumanizingSequencerRef};

use crate::{EvaluationResult, StatementResult, UbcbEngine};

/// Format an EvaluationResult as a multiline string using HumanizingSequencer.
pub fn format_result(result: &EvaluationResult, states: bool) -> String {
    if result.statements.is_empty() {
        return "{}".to_string();
    }

    let stmts: Vec<String> = result.statements.iter()
        .map(|s| format!("  {};", fmt_stmt(s, 2, states)))
        .collect();

    format!("\n{}\n}}", stmts.join("\n"))
}

fn fmt_stmt(stmt: &StatementResult, indent: usize, states: bool) -> String {
    let value = fmt_fir_inline(&stmt.fir, indent, states);
    match &stmt.name {
        Some(name) => format!("{name} = {value}"),
        None => value,
    }
}

fn fmt_fir_inline(fir: &FirRef, indent: usize, states: bool) -> String {
    let cloned_fir = clone_steppable(fir);
    let _seq = HumanizingSequencerRef::new(&cloned_fir);
    let output = format_fir_simple_with_indent(&cloned_fir, indent);
    if states {
        format!("{} [{}]", output, fir.borrow().state())
    } else {
        output
    }
}

/// UBCb evaluator adapter for SnapshotSuite.
pub struct UbcbEvaluator;

impl Evaluator for UbcbEvaluator {
    fn evaluate(&self, source: &str) -> Result<Vec<FirRef>, String> {
        let mut engine = UbcbEngine::new();
        let result = engine.evaluate(source)
            .map_err(|e| format!("Evaluation failed: {}", e))?;
        let mut firs = Vec::new();
        for stmt in result.statements {
            firs.push(stmt.fir);
        }
        Ok(firs)
    }
}

#[cfg(test)]
mod approval_tests {
    use super::*;
    use std::path::PathBuf;

    fn suite() -> foolish_core::SnapshotSuite {
        foolish_core::SnapshotSuite::new(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../foolish-ubcb-cli/snapshot_tests/input"),
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../foolish-ubcb-cli/snapshot_tests/approved"),
        ).expect("SnapshotSuite initialization failed")
    }

    #[test]
    fn approval_all() {
        let eval = UbcbEvaluator;
        let suite = suite();
        let evaluations = suite.evaluate_all(num_cpus::get(), &eval);
        let mut settings = insta::Settings::clone_current();
        settings.set_snapshot_path(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../foolish-ubcb-cli/snapshot_tests/approved"),
        );
        settings.set_prepend_module_to_snapshot(false);
        settings.set_omit_expression(true);
        settings.bind(|| {
            for (name, result) in evaluations {
                match result {
                    Ok(output) => {
                        insta::assert_snapshot!(format!("{}.foo", name), output);
                    }
                    Err(msg) => {
                        panic!("Evaluation error for {}: {}", name, msg);
                    }
                }
            }
        });
    }

    #[test]
    fn approval_all_states() {
        let eval = UbcbEvaluator;
        let suite = suite();
        let evaluations = suite.evaluate_all(num_cpus::get(), &eval);
        let mut settings = insta::Settings::clone_current();
        settings.set_snapshot_path(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../foolish-ubcb-cli/snapshot_tests/approved"),
        );
        settings.set_prepend_module_to_snapshot(false);
        settings.set_omit_expression(true);
        settings.bind(|| {
            for (name, result) in evaluations {
                match result {
                    Ok(output) => {
                        insta::assert_snapshot!(format!("{}_states.foo", name), output);
                    }
                    Err(msg) => {
                        panic!("Evaluation error for {}: {}", name, msg);
                    }
                }
            }
        });
    }
}
