//! # foolish-ubca
//!
//! UBCa — Two-Store ProtoBrane Tree and Uniform Two-Phase Stepping.
//!
//! This crate implements the new FIR evaluation model described in FOOP-62:
//! - **ProtoBrane**: a shared field-holder with two provenance-separated child
//!   stores (`foolish_children` for parse-derived, `ubc_children` for
//!   compute-derived).
//! - **Fir trait**: the narrow dyn-dispatch surface (`core()`,
//!   `fir_op_step()`, `kind()`).
//! - **FirRefExt**: extension trait adding `value()` and `step()` to `FirRef`,
//!   using transient borrows (critical for RefCell borrow discipline).
//! - **NyesExt**: adds `is_settled()` to `Nyes` (`is_constanic() || == Nk`).

pub mod compiler;
pub mod evaluator;
pub mod fir_kinds;
pub mod fir_trait;
pub(crate) mod identifier;
pub mod nyes_ext;
pub mod proto_brane;

pub use evaluator::UbcaEvaluator;
pub use fir_trait::{Fir, FirKind, FirRef, FirRefExt, Scope, StepReport, UbcError};
pub use nyes_ext::NyesExt;
pub use proto_brane::ProtoBrane;

#[cfg(test)]
mod ubca_snapshot_tester;
