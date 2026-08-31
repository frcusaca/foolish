//! `UbcaEvaluator` — the crate's one genuinely production-facing entry
//! point (confirmed by direct research at Phase 5's cutover: `foolish-ubca2`
//! has zero real external callers in this workspace; every other public
//! item existed only to serve this crate's own, now-superseded, `Rc`-based
//! test suite).
//!
//! Before Phase 5, this file also held: a parallel `Rc`-based stepping/
//! output-serialization implementation (`step_to_settled`,
//! `proto_to_core_fir`/`_sff_body`/`_sff_operand`/`_inner`,
//! `display_stmt_name`) that `evaluate` used to call into; and a set of
//! developer-facing debugger breakpoint helpers (`step_until`/
//! `step_until_line_number`/`step_until_statement_name`) that were never
//! used by production code even before this cutover (they were `pub` but
//! only ever called by this file's own tests). All of that — and its own
//! ~9 tests — was DELETED once `evaluate`'s body below was rewired onto
//! the arena path (`crate::fvm_storage`'s `FVMStorage`/`arena_compiler`/
//! `core_fir_conversion`), which is a complete, independently-tested
//! replacement for the same functionality (see `fvm_storage.rs`'s own
//! `core_fir_conversion` module and its test suite).

use foolish_core::fir as core_fir;
use foolish_core::fir::{FirRef as CoreFirRef, Nyes};

pub struct UbcaEvaluator;

impl foolish_core::Evaluator for UbcaEvaluator {
    /// Phase 5 cutover: this body runs on the arena path (`crate::
    /// fvm_storage`'s `FVMStorage`/`FirPointer`/`arena_compiler`/
    /// `core_fir_conversion`) — the crate's only remaining evaluation
    /// implementation, the original `Rc`/`RefCell`-based `Compiler`/`Fir`
    /// machinery (`compiler.rs`/`fir_kinds.rs`/`fir_trait.rs`) having been
    /// deleted once this cutover was proven correct end-to-end (the full
    /// `einmo_gate_checked` suite passing through it, `e856a206`).
    fn evaluate(&self, source: &str) -> Result<Vec<CoreFirRef>, String> {
        let mut storage = crate::fvm_storage::FVMStorage::new();

        // FOOP-33 §4: system.foo is implicitly composed as the root ancestor
        // of every program, not opt-in. The user's program becomes an
        // ordinary member of the composite root brane, named `program`; the
        // FVM steps the WHOLE composite to settlement, then extracts the
        // `program` member's result structurally (never via a Foolish
        // search) — see the arena's `compose_program_with_system` and
        // `program_result`.
        let composed_roots = crate::fvm_storage::compose_program_with_system(&mut storage, source)
            .map_err(|e| format!("Compilation failed: {}", e))?;

        let mut results = Vec::new();

        for composed_root in composed_roots {
            let failure = crate::fvm_storage::step_to_settled(&mut storage, composed_root).err();
            let program_fir = crate::fvm_storage::program_result(&storage, composed_root)
                .unwrap_or(composed_root);

            if let Some(alarm_msg) = failure {
                // Record the failure on BOTH the composed root and the
                // `program` member.
                //
                // The root is what failed to settle, so it carries the state
                // truthfully. But `program_result` reaches PAST the root to
                // the user's program member, and that member is what gets
                // rendered — so marking only the root puts the alarm on a
                // wrapper that is then discarded, and the output shows a
                // pre-constanic brane (`{BRANING`) with no explanation of why
                // evaluation stopped.
                for &target in &[composed_root, program_fir] {
                    storage.with_mut(target, |fir| {
                        fir.set_alarm_reason(alarm_msg.clone());
                        fir.set_nyes(Nyes::Nk);
                    });
                }
                eprintln!("ALARM: {alarm_msg}");
            }

            let core_fir = crate::fvm_storage::proto_to_core_fir(&storage, program_fir);
            results.push(core_fir::fir_to_ref(core_fir));
        }

        Ok(results)
    }
}
