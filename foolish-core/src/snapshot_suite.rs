use crate::FirRef;

/// Trait for evaluators that can evaluate a Foolish source file.
/// Returns the evaluated FIRs (final brane and statements).
pub trait Evaluator {
    /// Evaluate source code and return the final evaluated FIRs.
    fn evaluate(&self, source: &str) -> Result<Vec<FirRef>, String>;
}
