//! Foolish and detailed rendering for ubca2's arena FIR (FOOP-36 §1).

use crate::fvm_storage::{FVMStorage, FirPointer, proto_to_core_fir};

/// Selects how an arena FIR is rendered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SequenceMode {
    /// Valid Foolish source with evaluator state confined to comments.
    #[default]
    Foolish,
    /// The legacy FIR-internal rendering used for detailed debugging.
    Detailed,
}

/// Renders ubca2's arena FIR.
#[derive(Debug, Clone, Copy, Default)]
pub struct Ubca2Sequencer;

impl Ubca2Sequencer {
    /// Formats one arena FIR in the selected mode.
    #[must_use]
    pub fn format(storage: &FVMStorage, fir: FirPointer, mode: SequenceMode) -> String {
        match mode {
            // Phase 2 establishes the delegation boundary. Phase 3 replaces
            // this arm with the Foolish renderer.
            SequenceMode::Foolish | SequenceMode::Detailed => Self::format_detailed(storage, fir),
        }
    }

    fn format_detailed(storage: &FVMStorage, fir: FirPointer) -> String {
        let core_fir = proto_to_core_fir(storage, fir);
        foolish_core::FirSequencer::format(&core_fir)
    }
}

#[cfg(test)]
mod tests {
    use super::{SequenceMode, Ubca2Sequencer};
    use crate::fvm_storage::{
        FVMStorage, FirCursor, FirPointer, compose_program_with_system, program_result,
        proto_to_core_fir, step_to_constanic,
    };

    fn evaluated_body(source: &str, statement_index: usize) -> (FVMStorage, FirPointer) {
        let mut storage = FVMStorage::new();
        let roots = compose_program_with_system(&mut storage, source).expect("source compiles");
        let composed_root = roots[0];
        step_to_constanic(&mut storage, composed_root).expect("source settles");
        let program = program_result(&storage, composed_root).expect("program result exists");
        let statement = FirCursor::new(program, &storage)
            .stmt_at(statement_index)
            .expect("statement exists");
        let body = FirCursor::new(statement, &storage).foolish_children()[0];
        (storage, body)
    }

    fn assert_detailed_delegates(source: &str, statement_index: usize) {
        let (storage, fir) = evaluated_body(source, statement_index);
        let core_fir = proto_to_core_fir(&storage, fir);
        let expected = foolish_core::FirSequencer::format(&core_fir);

        assert_eq!(
            Ubca2Sequencer::format(&storage, fir, SequenceMode::Detailed),
            expected
        );
    }

    #[test]
    fn detailed_delegates_for_integer() {
        assert_detailed_delegates("{x=7;}", 0);
    }

    #[test]
    fn detailed_delegates_for_brane() {
        assert_detailed_delegates("{x={a=1;};}", 0);
    }

    #[test]
    fn detailed_delegates_for_operator() {
        assert_detailed_delegates("{x=1+2;}", 0);
    }

    #[test]
    fn detailed_delegates_for_resolved_search() {
        assert_detailed_delegates("{b={x=3;};r=b?x;}", 1);
    }

    #[test]
    fn detailed_delegates_for_nk() {
        assert_detailed_delegates("{x=1/0;}", 0);
    }
}
