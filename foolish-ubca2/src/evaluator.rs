//! `UbcaEvaluator` — the crate's one production-facing entry point.

use foolish_core::fir::Nyes;

use crate::fvm_storage::{FVMStorage, FirPointer};

pub struct UbcaEvaluator;

impl UbcaEvaluator {
    /// Evaluates source while retaining ubca2's complete arena representation.
    ///
    /// Foolish-mode sequencing uses this boundary because the compatibility
    /// conversion to `foolish_core::Fir` intentionally omits some source-form
    /// metadata, including search direction and contexting.
    pub fn evaluate_arena(&self, source: &str) -> Result<(FVMStorage, Vec<FirPointer>), String> {
        let mut storage = FVMStorage::new();

        let composed_roots = crate::fvm_storage::compose_program_with_system(&mut storage, source)
            .map_err(|e| format!("Compilation failed: {e}"))?;

        let mut results = Vec::with_capacity(composed_roots.len());
        for composed_root in composed_roots {
            let failure = crate::fvm_storage::step_to_constanic(&mut storage, composed_root).err();
            let program_fir = crate::fvm_storage::program_result(&storage, composed_root)
                .unwrap_or(composed_root);

            if let Some(alarm_msg) = failure {
                for &target in &[composed_root, program_fir] {
                    storage.with_mut(target, |fir| {
                        fir.set_alarm_reason(alarm_msg.clone());
                        fir.set_nyes(Nyes::Nk);
                    });
                }
                eprintln!("ALARM: {alarm_msg}");
            }

            results.push(program_fir);
        }

        Ok((storage, results))
    }
}
