//! # foolish-ubca2
//!
//! UBCa — arena-backed FIR storage. Every FIR node lives in a `u32`-indexed
//! arena (`fvm_storage::FVMStorage`) addressed through the validated handle
//! type `fvm_storage::FirPointer`.
//!
//! This crate and `foolish-ubca` are two **independent implementations of the
//! same Foolish evaluator** — neither depends on or calls into the other, and
//! what they share is only what both build on: `foolish-parser`'s AST and
//! `foolish-core`'s FIR, `Nyes`, sequencer, and `Evaluator` trait. They are
//! kept honest against each other through that trait and the einmo baselines
//! both are evaluated against; a disagreement about what a program means is a
//! bug in at least one of them. `foolish-ubca` builds its tree from
//! `Rc<RefCell<dyn Fir>>` nodes with vtable dispatch; this crate stores the
//! same tree as arena slots with enum dispatch.
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
