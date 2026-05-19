use foolish_core::{clone_steppable, FirRef};
use foolish_core::sequencer::format_fir_simple_with_indent;

use crate::{EvaluationResult, StatementResult};

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
    let output = format_fir_simple_with_indent(&cloned_fir, indent);
    if states {
        format!("{} [{}]", output, fir.borrow().state())
    } else {
        output
    }
}
