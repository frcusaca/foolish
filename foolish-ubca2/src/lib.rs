//! # foolish-ubca2
//!
//! UBCa — arena-backed FIR storage, and the reference implementation of
//! Foolish evaluation (FOOP-86: `foolish-ubca`, a second, independent
//! `Rc<RefCell<dyn Fir>>`-based implementation this crate was once kept
//! honest against, has been retired). Every FIR node lives in a
//! `u32`-indexed arena (`fvm_storage::FVMStorage`) addressed through the
//! validated handle type `fvm_storage::FirPointer`, stepped and dispatched
//! by kind through `FirSpec`'s enum dispatch — never `dyn Fir`.
//!
//! `UbcaEvaluator::evaluate_arena` is the crate's one production-facing
//! entry point, retaining the full arena so callers (the `foolish-cli`
//! binary, this crate's own einmo suite) can render through
//! `Ubca2Sequencer` without losing source-form metadata a compatibility
//! conversion would drop.
//!
//! - **`FVMStorage`**: the arena; owns every node reachable from any
//!   `FirPointer` it minted.
//! - **`FirSpec`**: one variant per FIR kind, dispatched on by `fir_op_step`
//!   (enum dispatch, not `dyn Fir` — rust_instructions.md §7).
//! - **`NyesExt`**: adds the four NYES-group predicates to `Nyes` —
//!   `is_preconstanic()`/`is_nye()`, `is_constanic()`, `is_constantew()`,
//!   `is_conclusive()`. No `is_settled()` — "settled" is qualified with its
//!   group everywhere in this crate rather than named as its own predicate.

pub mod evaluator;
pub mod fvm_storage;
pub(crate) mod identifier;
pub mod nyes_ext;
pub mod sequencer;
pub mod system_foo;

pub use evaluator::UbcaEvaluator;
pub use nyes_ext::NyesExt;
pub use sequencer::{SequenceMode, SequenceOptions, Ubca2Sequencer};

#[cfg(test)]
mod einmo_gates;
