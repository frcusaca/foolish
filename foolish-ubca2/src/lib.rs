//! # foolish-ubca2
//!
//! UBCa — arena-backed FIR storage. Every FIR node lives in a `u32`-indexed
//! arena (`fvm_storage::FVMStorage`) addressed through the validated handle
//! type `fvm_storage::FirPointer`, in place of `foolish-ubca`'s sibling
//! `Rc<RefCell<dyn Fir>>`/`ProtoBrane` design.
//!
//! `UbcaEvaluator::evaluate` is the crate's one production-facing entry
//! point.
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
