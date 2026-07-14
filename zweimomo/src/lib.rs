//! Zweimomo — einmo's companion test crate (FOOP-92 §Use Case D).
//!
//! It proves einmo's [`einmo::Evaluator`] trait is language-agnostic by
//! embedding three **pure-Rust** interpreters — Foolish (`foolish-ubca`),
//! Python (`rustpython-vm`), JavaScript (`boa_engine`) — as `Evaluator` impls
//! and running parallel test inputs through einmo's signed-snapshot pipeline.
//!
//! Serialization is zweimomo's responsibility: each adapter renders its
//! interpreter's values into text chunks using **what is most colloquial in
//! that language** (§D.2). Einmo never interprets body content.

pub mod aspects;
pub mod evaluators;
pub mod perspectives;

pub use aspects::{aspects_perspective, compute_aspects};
pub use evaluators::{BoaEvaluator, RustPythonEvaluator, UbcaEvaluatorAdapter};
pub use perspectives::brane_name_perspective;
