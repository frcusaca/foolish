//! # foolish-ubca2
//!
//! UBCa — arena-backed FIR storage (FOOP-16), replacing the original
//! `foolish-ubca` crate's `Rc<RefCell<dyn Fir>>`/`ProtoBrane` design with a
//! `u32`-indexed arena (`fvm_storage::FVMStorage`) addressed through the
//! validated handle type `fvm_storage::FirPointer`.
//!
//! Phase 5's cutover (see `docs/foop/FOOP-16.plan.md`) deleted the crate's
//! original `Rc`-based `Fir` trait/`ProtoBrane`/per-kind struct machinery
//! (`fir_trait.rs`, `proto_brane.rs`, `fir_kinds.rs`, `compiler.rs`'s and
//! `system_foo.rs`'s `Rc`-based construction) once `UbcaEvaluator::evaluate`
//! — the crate's one genuinely production-facing entry point (confirmed by
//! direct research: `foolish-ubca2` has zero real external callers in this
//! workspace) — was fully and correctly rewired onto the arena path, proven
//! by the complete `einmo_gate_checked` suite passing end-to-end through it.
//!
//! - **`FVMStorage`**: the arena; owns every node reachable from any
//!   `FirPointer` it minted.
//! - **`FirSpec`**: one variant per FIR kind, dispatched on by `fir_op_step`
//!   (enum dispatch, not `dyn Fir` — rust_instructions.md §7).
//! - **`NyesExt`**: adds `is_settled()` to `Nyes` (`is_constanic() || == Nk`).

pub mod evaluator;
pub mod fvm_storage;
pub(crate) mod identifier;
pub mod nyes_ext;
pub mod system_foo;

pub use evaluator::UbcaEvaluator;
pub use nyes_ext::NyesExt;

#[cfg(test)]
mod ubca_snapshot_tester;
