//! `system.foo` — shared constants and the comparison-operator enum.
//!
//! `system.foo` composition itself (`compose_program_with_system`/
//! `compose_one`/`program_result`/`comparison_body`) and comparison
//! evaluation live in `fvm_storage.rs`'s `arena_compiler` module and
//! `FirSpec::Comparison`'s `fir_op_step` dispatch arm. What remains here is
//! exactly the data those read directly (`ComparisonOp`, `OPERAND_SRC`,
//! `SYSTEM_FOO_SRC`), plus tests that exercise them via
//! `UbcaEvaluator::evaluate` directly.

/// The five comparison operators `system.foo` supplies (FOOP-33 §5.0).
///
/// One enum rather than five near-identical FIR types: every operator shares
/// the *entire* structure — the same two SFF-marked operand lookups
/// (`<<#-2>>`/`<<#-1>>`), the same constanic gating, the same `'True`/`'False`
/// production — and differs ONLY in which Rust comparison runs once both
/// operands are integers. A finite, closed set of behaviors distinguished by
/// one line of code is exactly what an enum is for (`rust_instructions.md`
/// §"finite word-domains → enum").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparisonOp {
    /// `'lt` — strictly less than.
    Lt,
    /// `'gt` — strictly greater than.
    Gt,
    /// `'le` — less than or equal.
    Le,
    /// `'ge` — greater than or equal.
    Ge,
    /// `'eq` — equal.
    Eq,
}

impl ComparisonOp {
    /// Every comparison operator, paired with the `system.foo` name that
    /// declares it. The single source of truth for which names are comparison
    /// operators — both the installer and its tests read this list.
    const ALL: [(&'static str, ComparisonOp); 5] = [
        ("'lt", ComparisonOp::Lt),
        ("'gt", ComparisonOp::Gt),
        ("'le", ComparisonOp::Le),
        ("'ge", ComparisonOp::Ge),
        ("'eq", ComparisonOp::Eq),
    ];

    /// The operator's `system.foo` declaration name (e.g. `"'lt"`).
    #[must_use]
    pub fn searchable_name(self) -> &'static str {
        match self {
            ComparisonOp::Lt => "'lt",
            ComparisonOp::Gt => "'gt",
            ComparisonOp::Le => "'le",
            ComparisonOp::Ge => "'ge",
            ComparisonOp::Eq => "'eq",
        }
    }

    /// Recover an operator from the name [`ComparisonOp::searchable_name`]
    /// produced. Used when constanic-cloning, where only the constanic node's
    /// own `op` field string is reachable.
    #[must_use]
    pub(crate) fn from_searchable_name(name: &str) -> Option<ComparisonOp> {
        ComparisonOp::ALL
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, op)| *op)
    }

    /// Run this operator's Rust comparison. The ONLY thing that differs
    /// between the five operators. `pub(crate)`, not private: called
    /// directly by `fvm_storage`'s `FirSpec::Comparison` dispatch arm, a
    /// sibling module.
    #[must_use]
    pub(crate) fn compare(self, left: i64, right: i64) -> bool {
        match self {
            ComparisonOp::Lt => left < right,
            ComparisonOp::Gt => left > right,
            ComparisonOp::Le => left <= right,
            ComparisonOp::Ge => left >= right,
            ComparisonOp::Eq => left == right,
        }
    }
}

/// The Foolish source of a comparison's operands: the two statements
/// immediately preceding the operator, both SFF-marked.
///
/// Written as source and run through the ordinary compiler (rather than
/// assembled by hand) so `build_fir`'s `under_sff` rule — the rule that builds
/// descendant searches ECONSTANIC so they never run where they have no valid
/// neighbors — applies exactly as it does to any other Foolish. Hand-built
/// operands would have to re-implement that rule and could drift from it.
/// The brane-and-statement wrapper is only there because an SFF marker is not
/// valid at top level; `arena_compiler::compile_stmt_body_under` discards it
/// and keeps the `<<…>>` body. `pub(crate)`: read directly by
/// `fvm_storage`'s arena-side `build_comparison`, a sibling module.
pub(crate) const OPERAND_SRC: [&str; 2] = ["{o = <<#-2>>;}", "{o = <<#-1>>;}"];

/// The embedded `system.foo` source, baked into the binary at compile time.
///
/// `OUT_DIR` is a Cargo build-script variable, read by `env!` at COMPILE time;
/// `include_str!` then bakes the file's contents into the binary right then.
/// Neither macro touches the filesystem at runtime — `system.foo` ships
/// inside the compiled crate with no runtime file dependency. See
/// `foolish-ubca2/build.rs` (the copy into `OUT_DIR`).
pub const SYSTEM_FOO_SRC: &str = include_str!(concat!(env!("OUT_DIR"), "/system.foo"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_foo_src_embeds_true_and_false() {
        assert!(SYSTEM_FOO_SRC.contains("'True"));
        assert!(SYSTEM_FOO_SRC.contains("'False"));
    }

    /// FOOP-75 §4: the non-brane diagnosis must reach RENDERED output, not
    /// just the internal `alarm_reason` field — `alarm_reason` is never read
    /// on the sequencing path, so a reason recorded there alone is
    /// invisible.
    ///
    /// Ported by FOOP-86 Phase 4b from the old bridge
    /// (`foolish_core::Evaluator::evaluate` + `FirSequencer::format`,
    /// asserting FIR-internal text `d =$ ??? (4 is not a brane)`) to the
    /// arena-native path — same source as the surviving suite's
    /// `regression/disappearing_brane_statements.foo`, so this also serves
    /// as a fast, non-einmo confirmation of the same behavior the signed
    /// baseline covers, in the NEW Foolish-mode rendering (`d =$ 4  !! NK: 4
    /// is not a brane`).
    #[test]
    fn foop75_non_brane_reason_reaches_rendered_output() {
        use crate::sequencer::{SequenceMode, Ubca2Sequencer};
        let (storage, firs) = crate::UbcaEvaluator
            .evaluate_arena("{a = 1; d =$ 4}")
            .unwrap();
        let rendered = firs
            .iter()
            .map(|&f| Ubca2Sequencer::format(&storage, f, SequenceMode::Foolish))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            rendered.contains("d =$ 4") && rendered.contains("4 is not a brane"),
            "the rendered statement must carry the diagnosis; got:\n{rendered}"
        );
    }

    /// A program that cannot settle must RENDER with the ITERATION-EXCEEDED
    /// alarm surfaced, not silently as a pre-constanic `BRANING` brane with
    /// no explanation.
    ///
    /// `{f1 = {f1}; stuck = f1;}` is self-referential: `f1`'s body searches
    /// for `f1`, so stepping never reaches a fixed point and the evaluator's
    /// step cap fires.
    ///
    /// Regression guard, ported by FOOP-86 Phase 4b from the old bridge
    /// (which rendered `NK(ITERATION-EXCEEDED, ...)` FIR-internal text) to
    /// the arena-native path. Same source as the surviving suite's
    /// `foop/62/infinite_loop.foo`, whose signed `checked/` baseline pins
    /// the exact banner text this test checks for. History: `evaluate` sets
    /// the alarm and NK on the COMPOSED ROOT, but `program_result` then
    /// reaches past that root to the user's `program` member and renders it
    /// instead — so the error state landed on a wrapper that was discarded.
    /// Introduced when system.foo composition began extracting the program
    /// member (FOOP-33); before that the root itself was rendered.
    #[test]
    fn non_settling_program_renders_iteration_exceeded_alarm() {
        use crate::sequencer::{SequenceMode, Ubca2Sequencer};
        let (storage, firs) = crate::UbcaEvaluator
            .evaluate_arena("{\n  f1 = { f1 }\n  stuck = f1;\n}")
            .expect("evaluation returns a result even when it cannot settle");
        let rendered = firs
            .iter()
            .map(|&f| Ubca2Sequencer::format(&storage, f, SequenceMode::Foolish))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            rendered.contains("did not complete stepping within the limit of"),
            "a program that exceeds the step cap must render the iteration-exceeded \
             banner, not silently stay pre-constanic; got:\n{rendered}"
        );
    }
}
