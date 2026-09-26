//! The FVM debugger entry points, exercised from OUTSIDE the crate.
//!
//! **Why an integration test and not a unit test** (the human, 2026-09-26: *"the debugging code
//! should be accessible by users of the fvm"*, and *"I wonder if this needs some tests"*): an
//! integration test compiles as a SEPARATE crate, so it can only reach genuinely `pub` items.
//! The in-crate unit tests in `fvm_storage.rs` cannot prove reachability — they would keep
//! passing if every one of these functions were narrowed back to `pub(crate)`, which is exactly
//! how the earlier `pub(crate)` + `expect(dead_code)` state went unnoticed.
//!
//! This file is therefore the regression test for the VISIBILITY, not for the stepping logic.
//! If someone narrows `step_until*` or `mod core_fir_conversion`, this file stops COMPILING —
//! a louder failure than a test assertion, and the right one.

use foolish_ubca2::UbcaEvaluator;
use foolish_ubca2::fvm_storage::core_fir_conversion::{
    step_to_constanic, step_until, step_until_line_number, step_until_statement_name,
};

/// Every debugger entry point is callable from outside the crate.
///
/// The assertions are deliberately weak — reachability is the property under test, and the
/// stepping behaviour itself is pinned by the unit tests in `fvm_storage.rs`.
#[test]
fn debugger_entry_points_are_reachable_from_outside_the_crate() {
    let (mut storage, roots) = UbcaEvaluator
        .evaluate_arena("{a = 1; b = 2; c = 3;}")
        .expect("program evaluates");
    let root = roots[0];

    // `evaluate_arena` already stepped this to settled, so a breakpoint cannot fire and each
    // call returns the "FVM settled first" error. That the call COMPILES and returns a typed
    // Result is the point: the API is reachable.
    let by_line = step_until_line_number(&mut storage, root, 2);
    assert!(
        by_line.is_ok() || by_line.is_err(),
        "step_until_line_number must be callable from outside the crate"
    );

    let by_name = step_until_statement_name(&mut storage, root, "b");
    assert!(
        by_name.is_ok() || by_name.is_err(),
        "step_until_statement_name must be callable from outside the crate"
    );

    let generic = step_until(&mut storage, root, |_storage, front| front.is_none());
    assert!(
        generic.is_ok() || generic.is_err(),
        "step_until must be callable from outside the crate"
    );

    // The production stepping loop is idempotent on an already-settled program.
    step_to_constanic(&mut storage, root).expect("stepping an already-settled program succeeds");
}
