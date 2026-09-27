//! Stepping loop and public Foolish debugger breakpoints (FOOP-96 §3).
//! The `foolish-debugging` skill uses `step_until*` to inspect FVM evaluation.
use super::{FVMStorage, FirCursor, FirPointer};

/// Steps `ptr` up to `MAX_STEPS` times, returning `Ok(())` once constanic, or an error naming the
/// iteration count if the step budget is exhausted first. Caps total top-level iterations — distinct
/// from `step_inner`'s `MAX_DEPTH`, which caps recursion depth within a single iteration.
const MAX_STEPS: usize = 10_000;

pub fn step_to_constanic(storage: &mut FVMStorage, ptr: FirPointer) -> Result<(), String> {
    let mut last_step = 0;
    for step in 0..MAX_STEPS {
        ptr.step(storage);
        last_step = step;
        if storage.get_nyes(ptr).is_constanic() {
            return Ok(());
        }
    }
    if !storage.get_nyes(ptr).is_constanic() {
        return Err(format!("Iteration exceeded {last_step}"));
    }
    Ok(())
}

/// The UBCA debugger-breakpoint equivalent — steps until `matcher`
/// accepts the front task (or `None` when there is no front task),
/// returning the step count, or an error if the FVM settles first or
/// the step budget is exhausted.
///
/// The matcher takes `&FVMStorage` explicitly alongside
/// `Option<FirPointer>` (rather than a bare `Option<FirPointer>`)
/// because a `FirPointer` carries no data of its own — it must be
/// read through the arena to be inspected.
///
/// **Public API for users of the FVM** (the human, 2026-09-26: *"the debugging code should be
/// accessible by users of the fvm"*). This is developer-facing debugger tooling rather than
/// part of `evaluate`'s own path, so it has no caller INSIDE this crate — but it is `pub`
/// because downstream code driving the FVM needs it, and a `pub` item in a `pub mod` is API,
/// never dead code. It therefore needs no `expect(dead_code)`; an earlier
/// `pub(crate)` + `expect(dead_code)` pair was papering over the too-narrow visibility.
pub fn step_until(
    storage: &mut FVMStorage,
    ptr: FirPointer,
    mut matcher: impl FnMut(&FVMStorage, Option<FirPointer>) -> bool,
) -> Result<usize, String> {
    for step in 0..MAX_STEPS {
        let front = FirCursor::new(ptr, storage).front_task();
        if matcher(storage, front) {
            return Ok(step);
        }
        if storage.get_nyes(ptr).is_constanic() {
            return Err(format!(
                "FVM went constanic (nyes={:?}) before condition was met at step {step}",
                storage.get_nyes(ptr)
            ));
        }
        ptr.step(storage);
    }
    Err(format!(
        "Step limit ({MAX_STEPS}) reached before condition was met"
    ))
}

/// Public debugger API — see `step_until`'s doc comment.
pub fn step_until_line_number(
    storage: &mut FVMStorage,
    ptr: FirPointer,
    line: usize,
) -> Result<usize, String> {
    step_until(storage, ptr, |storage, front| {
        front
            .and_then(|f| FirCursor::new(f, storage).as_stmt_line_number())
            .is_some_and(|l| l == line)
    })
}

/// Public debugger API — see `step_until`'s doc comment.
pub fn step_until_statement_name(
    storage: &mut FVMStorage,
    ptr: FirPointer,
    name: &str,
) -> Result<usize, String> {
    step_until(storage, ptr, |storage, front| {
        front
            .and_then(|f| FirCursor::new(f, storage).as_stmt_identifier())
            .is_some_and(|id| id.searchable_name() == name)
    })
}
