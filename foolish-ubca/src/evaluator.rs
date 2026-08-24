use std::rc::Rc;

use foolish_core::fir as core_fir;
use foolish_core::fir::{
    Alarm, AlarmLevel, AlarmSource, ConcatenationFirBuilder, ConstantIntFirBuilder,
    CreationFirBuilder, FirRef as CoreFirRef, IndexFirBuilder, NkFirBuilder, NormalBraneFirBuilder,
    Nyes, OperatorFirBuilder, SearchFirBuilder, StayFoolishFirBuilder, StayFullyFoolishFirBuilder,
};

#[cfg(test)]
use crate::compiler::Compiler;
use crate::fir_trait::{FirKind, FirRef, FirRefExt, StepReport};

// ── UBCA debugging facilities ──

/// Step the FVM until a condition on the front task is met.
///
/// This is the UBCA equivalent of a debugger breakpoint: it lets you stop
/// execution precisely when a certain statement reaches the job queue front,
/// then inspect the NYES state of any FIR in the graph.
///
/// Returns the number of steps taken, or an error if the FVM settled before
/// the condition was met, or the step limit was exceeded.
///
/// # Example
///
/// ```ignore
/// // Step until a statement named "extended" reaches the job queue front:
/// let steps = step_until(&root, &scope, |front| {
///     front.as_ref().map(|f| f.borrow().as_stmt_searchable_name() == Some("extended")).unwrap_or(false)
/// })?;
/// eprintln!("Stopped after {steps} steps, front task: {front:?}");
/// ```
pub fn step_until<F>(
    root: &FirRef,
    scope: &crate::fir_trait::Scope,
    mut matcher: F,
) -> Result<usize, crate::fir_trait::UbcError>
where
    F: FnMut(Option<&FirRef>) -> bool,
{
    for step in 0..MAX_STEPS {
        // Check if the front task matches before stepping.
        let front = root.borrow().debug_front_task();
        if let Some(ref front_ref) = front {
            if matcher(Some(front_ref)) {
                return Ok(step);
            }
        } else if matcher(None) {
            return Ok(step);
        }

        // Check if already settled.
        if root.borrow().core().get_nyes().is_constanic() {
            return Err(crate::fir_trait::UbcError::Eval(format!(
                "FVM settled (nyes={:?}) before condition was met at step {step}",
                root.borrow().core().get_nyes()
            )));
        }

        // Step once.
        let _ = root.step(scope)?;
    }
    Err(crate::fir_trait::UbcError::Eval(format!(
        "Step limit ({MAX_STEPS}) reached before condition was met"
    )))
}

/// Step until a statement at the given source line number reaches the job queue front.
///
/// Convenience wrapper around `step_until` for the common case of breakpointing
/// on a specific source line.
pub fn step_until_line_number(
    root: &FirRef,
    scope: &crate::fir_trait::Scope,
    line: usize,
) -> Result<usize, crate::fir_trait::UbcError> {
    step_until(root, scope, |front| {
        front
            .map(|f| f.borrow().as_stmt_line_number() == Some(line))
            .unwrap_or(false)
    })
}

/// Step until a statement with the given name reaches the job queue front.
///
/// Convenience wrapper around `step_until` for the common case of breakpointing
/// on a named statement.
pub fn step_until_statement_name(
    root: &FirRef,
    scope: &crate::fir_trait::Scope,
    name: &str,
) -> Result<usize, crate::fir_trait::UbcError> {
    step_until(root, scope, |front| {
        front
            .map(|f| f.borrow().as_stmt_searchable_name() == Some(name))
            .unwrap_or(false)
    })
}

const MAX_STEPS: usize = 10_000;
const MAX_STEPS_HARD_LIMIT: usize = 50_000_000;

/// Parse `@einmo set iteration depth to N` from the first 3 lines.
/// Directive may appear inside a `!!` line comment.
/// Returns clamped limit or `MAX_STEPS` if no directive found.
fn parse_iteration_depth(source: &str) -> usize {
    for line in source.lines().take(3) {
        let trimmed = line.trim().strip_prefix("!!").unwrap_or(line).trim();
        if let Some(rest) = trimmed.strip_prefix("@einmo set iteration depth to ") {
            let num_str: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
            if let Ok(n) = num_str.parse::<usize>() {
                return n.min(MAX_STEPS_HARD_LIMIT);
            }
        }
    }
    MAX_STEPS
}

/// The display name for a statement (FOOP-62 #19): an anonymous statement is named `???`
/// (`compiler::ANON_STMT_NAME`), which the sequencer must render WITHOUT a `name=` prefix.
/// Map `???` (and any empty name) to `None` so no prefix is emitted; a real LHS identifier
/// passes through as `Some(name)`.
fn display_stmt_name(name: Option<&str>) -> Option<String> {
    match name {
        Some(n) if n.is_empty() || n == crate::compiler::ANON_STMT_NAME => None,
        Some(n) => Some(n.to_string()),
        None => None,
    }
}

pub struct UbcaEvaluator;

impl foolish_core::Evaluator for UbcaEvaluator {
    fn evaluate(&self, source: &str) -> Result<Vec<CoreFirRef>, String> {
        // FOOP-33 §4: system.foo is implicitly composed as the root ancestor
        // of every program, not opt-in. The user's program becomes an
        // ordinary member of the composite root brane, named `program`; the
        // FVM steps the WHOLE composite to settlement, then extracts the
        // `program` member's result structurally (never via a Foolish
        // search) — see `system_foo::compose_program_with_system` and
        // `system_foo::program_result`.
        let composed_roots = crate::system_foo::compose_program_with_system(source)
            .map_err(|e| format!("Compilation failed: {}", e))?;

        let scope = crate::fir_trait::Scope::empty();
        let max_steps = parse_iteration_depth(source);
        let mut results = Vec::new();

        for composed_root in &composed_roots {
            let failure = step_to_settled(composed_root, &scope, max_steps).err();
            let program_fir = crate::system_foo::program_result(composed_root)
                .unwrap_or_else(|| Rc::clone(composed_root));

            if let Some(alarm) = failure {
                let alarm_msg = alarm.to_string();
                // Record the failure on BOTH the composed root and the
                // `program` member.
                //
                // The root is what failed to settle, so it carries the state
                // truthfully. But `program_result` reaches PAST the root to
                // the user's program member, and that member is what gets
                // rendered — so marking only the root puts the alarm on a
                // wrapper that is then discarded, and the output shows a
                // pre-constanic brane (`{BRANING`) with no explanation of why
                // evaluation stopped.
                for target in [composed_root, &program_fir] {
                    target.borrow().core().set_alarm_reason(alarm_msg.clone());
                    target.borrow().core().set_nyes(Nyes::Nk);
                }
                eprintln!("ALARM: {alarm_msg}");
            }

            let core_fir = proto_to_core_fir(&program_fir);
            results.push(core_fir::fir_to_ref(core_fir));
        }

        Ok(results)
    }
}

fn step_to_settled(
    fir_ref: &FirRef,
    scope: &crate::fir_trait::Scope,
    max_steps: usize,
) -> Result<(), crate::fir_trait::UbcError> {
    let mut last_step = 0;
    for step in 0..max_steps {
        let report = fir_ref.step(scope)?;
        last_step = step;
        match report {
            StepReport::Progress(nyes) if nyes.is_constanic() => return Ok(()),
            StepReport::NoProgress => break,
            _ => {}
        }
    }
    if !fir_ref.borrow().core().get_nyes().is_constanic() {
        return Err(crate::fir_trait::UbcError::Eval(format!(
            "Iteration exceeded {}",
            last_step
        )));
    }
    Ok(())
}

fn proto_to_core_fir(ubca_ref: &FirRef) -> core_fir::Fir {
    proto_to_core_fir_inner(ubca_ref, false, None)
}

/// Convert an SFF body expression. Top-level searches get EMBRYONIC state
/// (shown by sequencer). Operator operands get CONSTANT state (hidden).
/// Operators get WOCONSTANIC or CONSTANT state based on operand states.
/// (@Agents, I suppose this can't be declared as implementation on something
///   associated with SFF marker like SFFMark? ditto, similar questions for the
///   other '^fn' declarations in this file.)
///
/// `current_stmt` (FOOP-33 §"Concerns Standing Past Completion"): the
/// statement whose body is currently being converted, threaded through so
/// `CreationFir::get_display_name` can tell whether a creation is being
/// rendered from its own defining statement (name suppressed) or from
/// elsewhere (name shown, when the other conditions hold). It changes ONLY
/// where a statement's body conversion begins (`FirKind::Statement`, each
/// brane/concatenation/concat-helper statement loop) — every other call
/// site threads its caller's `current_stmt` through unchanged, since no new
/// statement is being entered.
fn proto_to_core_fir_sff_body(ubca_ref: &FirRef, current_stmt: Option<&FirRef>) -> core_fir::Fir {
    let borrowed = ubca_ref.borrow();
    let kind = borrowed.kind();
    match kind {
        FirKind::Search => SearchFirBuilder::new(borrowed.as_search_pattern().unwrap_or(""))
            .anchored(borrowed.as_search_anchored())
            .state(Nyes::Embryonic)
            .build(),
        FirKind::Operator => {
            let op = borrowed.as_op_name().unwrap_or("?").to_string();
            let operand_firs: Vec<core_fir::Fir> = borrowed
                .core()
                .foolish_children()
                .iter()
                .map(|c| proto_to_core_fir_sff_operand(c, current_stmt))
                .collect();
            use foolish_core::fir::FirQueryable;
            let op_state = if operand_firs.iter().any(|f| {
                let s = f.hs_state();
                s == Nyes::Econstanic || s == Nyes::Woconstanic
            }) {
                Nyes::Woconstanic
            } else {
                Nyes::Constant
            };
            OperatorFirBuilder::new(op)
                .operands(operand_firs)
                .state(op_state)
                .build()
        }
        FirKind::IndepInt => ConstantIntFirBuilder::new(borrowed.as_i64().unwrap_or(0))
            .state(Nyes::Constant)
            .build(),
        FirKind::Nk => NkFirBuilder::new(borrowed.as_nk_reason().unwrap_or("unknown"))
            .state(Nyes::Nk)
            .build(),
        _ => proto_to_core_fir_inner(ubca_ref, true, current_stmt),
    }
}

/// Convert an SFF operator operand. Searches get CONSTANT state (no state shown).
/// See `proto_to_core_fir_sff_body` for `current_stmt`.
fn proto_to_core_fir_sff_operand(
    ubca_ref: &FirRef,
    current_stmt: Option<&FirRef>,
) -> core_fir::Fir {
    let borrowed = ubca_ref.borrow();
    let kind = borrowed.kind();
    match kind {
        FirKind::Search => SearchFirBuilder::new(borrowed.as_search_pattern().unwrap_or(""))
            .anchored(borrowed.as_search_anchored())
            .state(Nyes::Econstanic)
            .build(),
        FirKind::IndepInt => ConstantIntFirBuilder::new(borrowed.as_i64().unwrap_or(0))
            .state(Nyes::Constant)
            .build(),
        FirKind::Nk => NkFirBuilder::new(borrowed.as_nk_reason().unwrap_or("unknown"))
            .state(Nyes::Nk)
            .build(),
        _ => proto_to_core_fir_inner(ubca_ref, true, current_stmt),
    }
}

/// See `proto_to_core_fir_sff_body` for `current_stmt`.
fn anchor_to_core_fir(ubca_ref: &FirRef, current_stmt: Option<&FirRef>) -> core_fir::Fir {
    let borrowed = ubca_ref.borrow();
    let kind = borrowed.kind();
    let state = borrowed.core().get_nyes();

    if kind == FirKind::Search {
        return SearchFirBuilder::new(borrowed.as_search_pattern().unwrap_or(""))
            .anchored(borrowed.as_search_anchored())
            .state(state)
            .build();
    }

    proto_to_core_fir_inner(ubca_ref, true, current_stmt)
}

/// See `proto_to_core_fir_sff_body` for `current_stmt`.
fn proto_to_core_fir_inner(
    ubca_ref: &FirRef,
    preserve_search: bool,
    current_stmt: Option<&FirRef>,
) -> core_fir::Fir {
    let borrowed = ubca_ref.borrow();
    let kind = borrowed.kind();
    let state = borrowed.core().get_nyes();

    match kind {
        FirKind::IndepInt => ConstantIntFirBuilder::new(borrowed.as_i64().unwrap_or(0))
            .state(state)
            .build(),
        FirKind::Nk => {
            let reason = borrowed.as_nk_reason().unwrap_or("unknown");
            let mut builder = NkFirBuilder::new(reason).state(state);
            if reason == "division by zero" {
                builder = builder.alarm(Alarm {
                    level: AlarmLevel::Mild,
                    code: "DIV-BY-ZERO".to_string(),
                    message: "Division by zero produces NK".to_string(),
                    source: AlarmSource::Evaluator,
                });
            }
            builder.build()
        }
        // A comparison renders as its RESULT — the 'True/'False creation it
        // produced, or the NK from a non-integer operand (FOOP-33 §5.0). It
        // never renders a wrapper of its own: the operands are the referencing
        // brane's OWN statements, already rendered in their own right, so
        // showing them again under the operator would duplicate them.
        FirKind::Comparison => {
            if let Some(result) = borrowed.core().ubc_children().first() {
                return proto_to_core_fir_inner(result, preserve_search, current_stmt);
            }
            NkFirBuilder::new(
                borrowed
                    .core()
                    .alarm_reason()
                    .as_deref()
                    .unwrap_or("comparison"),
            )
            .state(state)
            .build()
        }
        FirKind::Modulo => {
            if let Some(result) = borrowed.core().ubc_children().first() {
                return proto_to_core_fir_inner(result, preserve_search, current_stmt);
            }
            NkFirBuilder::new(
                borrowed
                    .core()
                    .alarm_reason()
                    .as_deref()
                    .unwrap_or("modulo"),
            )
            .state(state)
            .build()
        }
        FirKind::Or => {
            if let Some(result) = borrowed.core().ubc_children().first() {
                return proto_to_core_fir_inner(result, preserve_search, current_stmt);
            }
            NkFirBuilder::new(borrowed.core().alarm_reason().as_deref().unwrap_or("or"))
                .state(state)
                .build()
        }
        FirKind::SearchPosition => {
            if let Some(result) = borrowed.core().ubc_children().first() {
                return proto_to_core_fir_inner(result, preserve_search, current_stmt);
            }
            NkFirBuilder::new(borrowed.core().alarm_reason().as_deref().unwrap_or("@"))
                .state(state)
                .build()
        }
        FirKind::Operator => {
            // Unwrap to the result when the operator successfully computed
            // a constant value.
            if state == Nyes::Constant {
                let ubc = borrowed.core().ubc_children();
                if let Some(result) = ubc.first() {
                    return proto_to_core_fir_inner(result, preserve_search, current_stmt);
                }
            }
            // When the operator itself computed NK (e.g. division by zero)
            // AND none of its operands are NK (meaning NK was computed here,
            // not propagated from an operand), unwrap to the NK result.
            // If any operand is NK, keep the operator wrapper so the
            // humanizer can display all operands and state.
            if state == Nyes::Nk {
                let op_name = borrowed.as_op_name().unwrap_or("");
                let any_operand_nk = borrowed
                    .core()
                    .foolish_children()
                    .iter()
                    .any(|c| c.borrow().core().get_nyes() == Nyes::Nk);
                if !any_operand_nk && op_name != "$" {
                    let ubc = borrowed.core().ubc_children();
                    if let Some(result) = ubc.first() {
                        return proto_to_core_fir_inner(result, preserve_search, current_stmt);
                    }
                }
            }
            let op = borrowed.as_op_name().unwrap_or("?").to_string();
            let operand_firs: Vec<core_fir::Fir> = if op == "$" {
                borrowed
                    .core()
                    .foolish_children()
                    .iter()
                    .enumerate()
                    .map(|(i, c)| {
                        if i == 0 {
                            IndexFirBuilder::new(-1)
                                .anchored(false)
                                .state(Nyes::Econstanic)
                                .build()
                        } else {
                            proto_to_core_fir_inner(c, preserve_search, current_stmt)
                        }
                    })
                    .collect()
            } else {
                borrowed
                    .core()
                    .foolish_children()
                    .iter()
                    .map(|c| proto_to_core_fir_inner(c, preserve_search, current_stmt))
                    .collect()
            };
            OperatorFirBuilder::new(op)
                .operands(operand_firs)
                .state(state)
                .build()
        }
        FirKind::Statement => {
            let name = display_stmt_name(borrowed.as_stmt_searchable_name());
            drop(borrowed);
            // Prefer settled_result() over the raw written body — see
            // crate::fir_kinds::statement_value_for_comparison's doc comment
            // (FOOP-33 §4). Without this, the null-const rule's refusal is
            // enforced internally but never actually rendered: `'True = 3`
            // would still SHOW `3` instead of the NF NK.
            //
            // `current_stmt = Some(ubca_ref)` here: we are now converting
            // THIS statement's own body, so any creation found directly as
            // that body is at its own defining site (see
            // `CreationFir::get_display_name`'s condition 1).
            let body_fir = crate::fir_kinds::statement_value_for_comparison(ubca_ref)
                .map(|c| proto_to_core_fir_inner(&c, preserve_search, Some(ubca_ref)))
                .unwrap_or_else(|| NkFirBuilder::new("empty statement").build());
            NormalBraneFirBuilder::new()
                .statement(name, body_fir)
                .state(state)
                .build()
        }
        FirKind::Brane => {
            let stmt_tuples: Vec<(Option<String>, core_fir::Fir)> = borrowed
                .core()
                .foolish_children()
                .iter()
                .map(|c| {
                    let name = display_stmt_name(c.borrow().as_stmt_searchable_name());
                    // Prefer settled_result() over the raw written body — see
                    // crate::fir_kinds::statement_value_for_comparison's doc
                    // comment (FOOP-33 §4). `current_stmt = Some(c)`: `c` is
                    // the statement whose body is being converted here.
                    let body_fir = crate::fir_kinds::statement_value_for_comparison(c)
                        .map(|body| proto_to_core_fir_inner(&body, preserve_search, Some(c)))
                        .unwrap_or_else(|| NkFirBuilder::new("empty body").build());
                    (name, body_fir)
                })
                .collect();
            // Recompute brane state from converted children: if any child body
            // is ECONSTANIC or WOCONSTANIC, the brane is WOCONSTANIC.
            let mut effective_state = state;
            if state == Nyes::Constant || state == Nyes::Independent {
                use foolish_core::fir::FirQueryable;
                for (_, body) in &stmt_tuples {
                    let body_state = body.hs_state();
                    if body_state == Nyes::Econstanic || body_state == Nyes::Woconstanic {
                        effective_state = Nyes::Woconstanic;
                        break;
                    }
                    if body_state == Nyes::Nk {
                        effective_state = Nyes::Nk;
                        break;
                    }
                }
            }
            let mut builder = NormalBraneFirBuilder::new()
                .characterizations(borrowed.as_brane_characterizations().to_vec())
                .statements(stmt_tuples)
                .state(effective_state);
            if let Some(alarm_reason) = borrowed.core().alarm_reason() {
                builder = builder.alarm(Alarm {
                    level: AlarmLevel::Mild,
                    code: "ITERATION-EXCEEDED".to_string(),
                    message: alarm_reason.replace("ubca evaluation error: ", ""),
                    source: AlarmSource::Evaluator,
                });
            }
            builder.build()
        }
        FirKind::Search => {
            if state.is_constanic() {
                let ubc = borrowed.core().ubc_children();
                if let Some(result) = ubc.first() {
                    // When the ubc_child is a settled search whose own ubc_child
                    // is a complex type (Brane, Operator, SF, SFF), this search
                    // came from unwrapping an SF value. UBC preserves the search
                    // wrapper in this case rather than resolving to the final value.
                    let result_borrowed = result.borrow();
                    if result_borrowed.kind() == FirKind::Search
                        && result_borrowed.core().get_nyes().is_constanic()
                    {
                        let inner_ubc = result_borrowed.core().ubc_children();
                        let has_complex = inner_ubc.first().is_some_and(|r| {
                            let rb = r.borrow();
                            let is_complex_type = matches!(
                                rb.kind(),
                                FirKind::Brane
                                    | FirKind::Operator
                                    | FirKind::StayFoolish
                                    | FirKind::StayFullyFoolish
                            );
                            let has_resolved_value = !rb.core().ubc_children().is_empty();
                            is_complex_type && !has_resolved_value
                        });
                        if has_complex {
                            let inner_fir = SearchFirBuilder::new(
                                result_borrowed.as_search_pattern().unwrap_or(""),
                            )
                            .anchored(result_borrowed.as_search_anchored())
                            .state(Nyes::Econstanic)
                            .build();
                            drop(result_borrowed);
                            return SearchFirBuilder::new(
                                borrowed.as_search_pattern().unwrap_or(""),
                            )
                            .anchored(borrowed.as_search_anchored())
                            .result(inner_fir)
                            .state(Nyes::Woconstanic)
                            .build();
                        }
                        // Simple result (IndepInt/NK): build inner search with resolved value.
                        // For other cases (Search chains), fall through to the normal path
                        // which correctly wraps in the outer search.
                        let first_inner_kind = inner_ubc.first().map(|r| r.borrow().kind());
                        let has_simple = first_inner_kind
                            .is_some_and(|k| matches!(k, FirKind::IndepInt | FirKind::Nk));
                        if has_simple {
                            let inner_result_fir = proto_to_core_fir_inner(
                                inner_ubc.first().unwrap(),
                                false,
                                current_stmt,
                            );
                            let inner_search = SearchFirBuilder::new(
                                result_borrowed.as_search_pattern().unwrap_or(""),
                            )
                            .anchored(result_borrowed.as_search_anchored())
                            .result(inner_result_fir)
                            .state(result_borrowed.core().get_nyes())
                            .build();
                            drop(result_borrowed);
                            return inner_search;
                        }
                    }
                    drop(result_borrowed);

                    let resolved = proto_to_core_fir_inner(result, preserve_search, current_stmt);
                    if !preserve_search {
                        let resolved_state = result.borrow().core().get_nyes();
                        if resolved_state == Nyes::Constant || resolved_state == Nyes::Independent {
                            if let Some(sf_pat) = borrowed.as_sf_inner_pattern() {
                                return SearchFirBuilder::new(sf_pat)
                                    .result(resolved)
                                    .state(resolved_state)
                                    .build();
                            }
                            return resolved;
                        }
                    }
                    return SearchFirBuilder::new(borrowed.as_search_pattern().unwrap_or(""))
                        .anchored(borrowed.as_search_anchored())
                        .result(resolved)
                        .state(state)
                        .build();
                }
            }
            let mut builder = SearchFirBuilder::new(borrowed.as_search_pattern().unwrap_or(""))
                .anchored(borrowed.as_search_anchored())
                .state(state);
            // A value search (`~=` / `?=`) carries a value EXPRESSION as a child
            // (anchor first if present, value last). Surface both so the
            // sequencer renders `=(anchor=…, value=…)` instead of a degenerate
            // empty-pattern search (FOOP-23 rendering appendix).
            if borrowed.as_search_is_value() {
                builder = builder.is_value(true);
                let children = borrowed.core().foolish_children();
                let has_anchor = borrowed.as_search_anchored();
                if has_anchor && let Some(a) = children.first() {
                    builder = builder.anchor(proto_to_core_fir_inner(a, false, current_stmt));
                }
                let value_idx = if has_anchor { 1 } else { 0 };
                if let Some(v) = children.get(value_idx) {
                    builder = builder.value(proto_to_core_fir_inner(v, false, current_stmt));
                }
            }
            if let Some(alarm_reason) = borrowed.core().alarm_reason() {
                builder = builder.alarm(Alarm {
                    level: AlarmLevel::Mild,
                    code: "VALUE-SEARCH-UNSUPPORTED-PATTERN".to_string(),
                    message: alarm_reason,
                    source: AlarmSource::Evaluator,
                });
            }
            builder.build()
        }
        FirKind::Index => {
            if state.is_constanic() {
                let ubc = borrowed.core().ubc_children();
                if let Some(result) = ubc.first() {
                    let resolved = proto_to_core_fir_inner(result, preserve_search, current_stmt);
                    let resolved_state = result.borrow().core().get_nyes();
                    let result_kind = result.borrow().kind();
                    // Unwrap when: constant/independent, OR the result is a Brane
                    // (index into a brane returns the brane itself, not the index wrapper)
                    if !preserve_search
                        && (resolved_state == Nyes::Constant
                            || resolved_state == Nyes::Independent
                            || result_kind == FirKind::Brane)
                    {
                        return resolved;
                    }
                    // `resolved` is the indexed RESULT — it goes in the result= slot.
                    let mut builder = IndexFirBuilder::new(borrowed.as_index_offset())
                        .anchored(borrowed.as_index_anchored())
                        .result(resolved)
                        .state(state);
                    if borrowed.as_index_anchored()
                        && let Some(anchor_ref) = borrowed.core().foolish_children().first()
                    {
                        builder = builder.anchor(anchor_to_core_fir(anchor_ref, current_stmt));
                    }
                    return builder.build();
                }
            }
            let mut builder = IndexFirBuilder::new(borrowed.as_index_offset())
                .anchored(borrowed.as_index_anchored())
                .state(state);
            if borrowed.as_index_anchored()
                && let Some(anchor_ref) = borrowed.core().foolish_children().first()
            {
                builder = builder.anchor(anchor_to_core_fir(anchor_ref, current_stmt));
            }
            builder.build()
        }
        FirKind::StayFoolish => {
            let inner = borrowed.core().foolish_children();
            let inner_ref = inner.first();
            if let Some(expr_ref) = inner_ref {
                let expr_borrowed = expr_ref.borrow();
                if expr_borrowed.kind() == FirKind::Search
                    && expr_borrowed.core().get_nyes().is_constanic()
                {
                    let ubc = expr_borrowed.core().ubc_children();
                    if let Some(result) = ubc.first() {
                        let result_kind = result.borrow().kind();
                        if result_kind == FirKind::Brane
                            || result_kind == FirKind::Operator
                            || result_kind == FirKind::StayFoolish
                            || result_kind == FirKind::StayFullyFoolish
                        {
                            if !result.borrow().core().ubc_children().is_empty() {
                                let inner_result_fir =
                                    proto_to_core_fir_inner(result, false, current_stmt);
                                return SearchFirBuilder::new(
                                    expr_borrowed.as_search_pattern().unwrap_or(""),
                                )
                                .anchored(expr_borrowed.as_search_anchored())
                                .result(inner_result_fir)
                                .state(expr_borrowed.core().get_nyes())
                                .build();
                            }
                            // A Brane or other complex type that is itself
                            // constanic IS the value — no ubc_children needed.
                            if result.borrow().core().get_nyes().is_constanic() {
                                let inner_result_fir =
                                    proto_to_core_fir_inner(result, false, current_stmt);
                                return SearchFirBuilder::new(
                                    expr_borrowed.as_search_pattern().unwrap_or(""),
                                )
                                .anchored(expr_borrowed.as_search_anchored())
                                .result(inner_result_fir)
                                .state(expr_borrowed.core().get_nyes())
                                .build();
                            }
                            let search_fir = SearchFirBuilder::new(
                                expr_borrowed.as_search_pattern().unwrap_or(""),
                            )
                            .anchored(expr_borrowed.as_search_anchored())
                            .state(Nyes::Econstanic)
                            .build();
                            return StayFoolishFirBuilder::new(search_fir)
                                .state(Nyes::Woconstanic)
                                .build();
                        }
                        if result_kind == FirKind::Search {
                            let inner_borrowed = result.borrow();
                            let inner_fir = SearchFirBuilder::new(
                                inner_borrowed.as_search_pattern().unwrap_or(""),
                            )
                            .anchored(inner_borrowed.as_search_anchored())
                            .state(Nyes::Econstanic)
                            .build();
                            drop(inner_borrowed);
                            let outer_search = SearchFirBuilder::new(
                                expr_borrowed.as_search_pattern().unwrap_or(""),
                            )
                            .anchored(expr_borrowed.as_search_anchored())
                            .result(inner_fir)
                            .state(Nyes::Woconstanic)
                            .build();
                            return StayFoolishFirBuilder::new(outer_search)
                                .state(Nyes::Woconstanic)
                                .build();
                        }
                        if result_kind == FirKind::IndepInt || result_kind == FirKind::Nk {
                            let inner_result_fir =
                                proto_to_core_fir_inner(result, false, current_stmt);
                            return SearchFirBuilder::new(
                                expr_borrowed.as_search_pattern().unwrap_or(""),
                            )
                            .anchored(expr_borrowed.as_search_anchored())
                            .result(inner_result_fir)
                            .state(expr_borrowed.core().get_nyes())
                            .build();
                        }
                    }
                }
            }
            let expr_fir = inner_ref
                .map(|c| proto_to_core_fir_inner(c, true, current_stmt))
                .unwrap_or_else(|| NkFirBuilder::new("empty sf").build());
            StayFoolishFirBuilder::new(expr_fir).state(state).build()
        }
        FirKind::StayFullyFoolish => {
            // In UBC, SFF stores the expression with searches at EMBRYONIC
            // (not evaluated). Force inner searches to EMBRYONIC state.
            let expr_fir = borrowed
                .core()
                .foolish_children()
                .first()
                .map(|c| proto_to_core_fir_sff_body(c, current_stmt))
                .unwrap_or_else(|| NkFirBuilder::new("empty sff").build());
            StayFullyFoolishFirBuilder::new(expr_fir)
                .state(state)
                .build()
        }
        FirKind::Concatenation => {
            // Render the joined brane once the join actually ran (helper
            // present), or for a genuinely-empty concatenation (settled
            // Constant/Independent with no lines). Otherwise the join was
            // blocked by an un-joinable element (`fir_op_step` left
            // ubc_children empty and settled WOCONSTANIC/NK) → render the
            // raw un-joined elements, same as the pre-constanic branch below.
            let joined = !borrowed.core().ubc_children().is_empty();
            let empty_done = matches!(state, Nyes::Constant | Nyes::Independent);
            if state.is_constanic() && (joined || empty_done) {
                // FOOP-55 §10: content is asked of the settled RESULT, not
                // the operator -- `.value()` unwraps to the ConcatHelper
                // that `joined`/`empty_done` just confirmed exists.
                drop(borrowed);
                let result = ubca_ref.value();
                let borrowed = result.borrow();
                let count = borrowed.stmt_count().unwrap_or(0);
                let stmt_tuples: Vec<(Option<String>, core_fir::Fir)> = (0..count)
                    .filter_map(|i| {
                        let stmt = borrowed.stmt_at(i)?;
                        let sb = stmt.borrow();
                        let name = display_stmt_name(sb.as_stmt_searchable_name());
                        // `current_stmt = Some(&stmt)`: `stmt` is the statement
                        // whose body is being converted here.
                        let body_fir = sb
                            .core()
                            .foolish_children()
                            .first()
                            .map(|c| proto_to_core_fir_inner(c, preserve_search, Some(&stmt)))
                            .unwrap_or_else(|| NkFirBuilder::new("empty body").build());
                        drop(sb);
                        Some((name, body_fir))
                    })
                    .collect();
                let mut effective_state = state;
                if state == Nyes::Constant || state == Nyes::Independent {
                    use foolish_core::fir::FirQueryable;
                    for (_, body) in &stmt_tuples {
                        let body_state = body.hs_state();
                        if body_state == Nyes::Econstanic || body_state == Nyes::Woconstanic {
                            effective_state = Nyes::Woconstanic;
                            break;
                        }
                        if body_state == Nyes::Nk {
                            effective_state = Nyes::Nk;
                            break;
                        }
                    }
                }
                return NormalBraneFirBuilder::new()
                    .statements(stmt_tuples)
                    .state(effective_state)
                    .build();
            }
            let elem_firs: Vec<core_fir::Fir> = borrowed
                .core()
                .foolish_children()
                .iter()
                .map(|c| proto_to_core_fir_inner(c, preserve_search, current_stmt))
                .collect();
            let is_tail = borrowed.as_concat_provenance()
                == crate::fir_kinds::ConcatProvenance::TailConcatenation;
            ConcatenationFirBuilder::new()
                .elements(elem_firs)
                .state(state)
                .is_tail_concatenation(is_tail)
                .build()
        }
        FirKind::ConcatHelper => {
            let stmt_tuples: Vec<(Option<String>, core_fir::Fir)> = borrowed
                .core()
                .foolish_children()
                .iter()
                .map(|c| {
                    let cb = c.borrow();
                    let name = display_stmt_name(cb.as_stmt_searchable_name());
                    // `current_stmt = Some(c)`: `c` is the statement whose
                    // body is being converted here.
                    let body_fir = cb
                        .core()
                        .foolish_children()
                        .first()
                        .map(|body| proto_to_core_fir_inner(body, preserve_search, Some(c)))
                        .unwrap_or_else(|| NkFirBuilder::new("empty body").build());
                    (name, body_fir)
                })
                .collect();
            NormalBraneFirBuilder::new()
                .statements(stmt_tuples)
                .state(state)
                .build()
        }
        FirKind::Unknown | FirKind::FoolRef => NkFirBuilder::new("unknown fir kind").build(),
        // FOOP-33 Phase 9: resolve the display name HERE, at the conversion
        // boundary, because this is the one place ubca `Rc` identity and the
        // parent chain (which `as_creation_display_name` walks) still exist
        // alongside the `foolish-core` conversion target. `borrowed` is the
        // creation itself; `ubca_ref` is its own `FirRef`, exactly the
        // `self_ref` the FVM-side method needs to find its defining
        // statement (see `CreationFir::get_display_name`).
        FirKind::Creation => {
            let mut builder = CreationFirBuilder::new();
            if let Some(name) = borrowed.as_creation_display_name(ubca_ref, current_stmt) {
                builder = builder.name(name);
            }
            builder.build()
        }
    }
}

#[cfg(test)]
mod step_until_tests {
    use super::*;
    use crate::fir_trait::Scope;
    use std::rc::Rc;

    #[test]
    fn step_until_statement_name_finds_second_statement() {
        let firs = Compiler::compile("{a = 1; b = 2;}").unwrap();
        let root = firs[0].clone();
        let scope = Scope::empty();

        let steps = step_until_statement_name(&root, &scope, "b").unwrap();
        eprintln!(
            "step_until_statement_name('b') stopped after {} steps",
            steps
        );
        let front = root.borrow().debug_front_task();
        assert!(front.is_some(), "front task should exist");
        let f = front.unwrap();
        assert_eq!(
            f.borrow().as_stmt_searchable_name(),
            Some("b"),
            "front task should be 'b'"
        );
    }

    #[test]
    fn step_until_statement_name_finds_first_statement() {
        let firs = Compiler::compile("{x = 42; y = 10;}").unwrap();
        let root = firs[0].clone();
        let scope = Scope::empty();

        let steps = step_until_statement_name(&root, &scope, "x").unwrap();
        eprintln!(
            "step_until_statement_name('x') stopped after {} steps",
            steps
        );
        let front = root.borrow().debug_front_task();
        assert!(front.is_some());
        let f = front.unwrap();
        assert_eq!(f.borrow().as_stmt_searchable_name(), Some("x"));
    }

    #[test]
    fn step_until_line_number_finds_line() {
        let firs = Compiler::compile("{a = 1; b = 2; c = 3;}").unwrap();
        let root = firs[0].clone();
        let scope = Scope::empty();

        let steps = step_until_line_number(&root, &scope, 2).unwrap();
        eprintln!("step_until_line_number(2) stopped after {} steps", steps);
        let front = root.borrow().debug_front_task();
        assert!(front.is_some());
        let f = front.unwrap();
        assert_eq!(
            f.borrow().as_stmt_line_number(),
            Some(2),
            "front should be statement at line 2"
        );
    }

    #[test]
    fn step_until_generic_matcher_by_nyes() {
        let firs = Compiler::compile("{a = 1; b = 2;}").unwrap();
        let root = firs[0].clone();
        let scope = Scope::empty();

        let steps = step_until(&root, &scope, |front| {
            front
                .map(|f| f.borrow().core().get_nyes().is_constanic())
                .unwrap_or(false)
        })
        .unwrap();
        eprintln!(
            "step_until(by nyes constanic) stopped after {} steps",
            steps
        );
        let front = root.borrow().debug_front_task();
        assert!(
            front
                .map(|f| f.borrow().core().get_nyes().is_constanic())
                .unwrap_or(false),
            "front should be constanic"
        );
    }

    #[test]
    fn step_until_settles_before_condition_returns_error() {
        let firs = Compiler::compile("{a = 1;}").unwrap();
        let root = firs[0].clone();
        let scope = Scope::empty();

        step_to_settled(&root, &scope, MAX_STEPS).unwrap();
        let result = step_until_statement_name(&root, &scope, "nonexistent");
        assert!(
            result.is_err(),
            "should error when root is settled before condition is met"
        );
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("settled") || err_msg.contains("condition"),
            "error message: {err_msg}"
        );
    }

    /// Diagnostic test: use step_until to debug x=cb.shadow returning NK
    /// in concat_brane_nested_shadowed_resolution.
    /// Steps until `extended` reaches the job queue front, then inspects
    /// the NYES of `cb` and the floating brane element.
    #[test]
    fn diag_concat_cb_shadow_uses_step_until() {
        use std::path::PathBuf;

        let input = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("einmo_suite")
            .join("input")
            .join("foop")
            .join("13")
            .join("concat_brane_nested_shadowed_resolution.foo");
        let source = std::fs::read_to_string(&input)
            .unwrap_or_else(|_| panic!("{} not found", input.display()));

        let firs = Compiler::compile(&source).unwrap();
        let root = firs[0].clone();
        let scope = Scope::empty();

        // Step until "extended" reaches the job queue front.
        let steps = step_until_statement_name(&root, &scope, "extended")
            .unwrap_or_else(|e| panic!("step_until('extended') failed: {e}"));
        eprintln!("step_until('extended') stopped after {} steps", steps);

        // At this point, "extended" is at the front of the queue but not yet stepped.
        // Inspect the root brane children.
        let root_children: Vec<FirRef> = root.borrow().core().foolish_children().to_vec();
        eprintln!("root has {} children", root_children.len());
        for (i, child) in root_children.iter().enumerate() {
            let cb = child.borrow();
            let name = cb.as_stmt_searchable_name().unwrap_or("(anon)");
            let nyes = cb.core().get_nyes();
            let kind = cb.kind();
            eprintln!("  [{}] {} (kind={:?}, nyes={:?})", i, name, kind, nyes);
        }

        // Find the "cb" statement and inspect its NYES and value.
        let cb_stmt = root_children
            .iter()
            .find(|c| c.borrow().as_stmt_searchable_name() == Some("cb"))
            .cloned()
            .expect("cb statement not found in root");
        let cb_nyes = cb_stmt.borrow().core().get_nyes();
        eprintln!("cb NYES before extended steps: {:?}", cb_nyes);
        assert!(
            cb_nyes.is_constanic(),
            "cb should be constanic before 'extended' is stepped (cb is earlier in brane)"
        );

        // Now step to settled and check the x=cb.shadow search result.
        step_to_settled(&root, &scope, MAX_STEPS).unwrap();

        // Find the search for "^shadow$" in the entire root.
        let shadow_searches = {
            let pattern = "^shadow$";
            let mut all = Vec::new();
            for c in root_children.iter() {
                fn recurse(node: &FirRef, pattern: &str, out: &mut Vec<FirRef>) {
                    if node.borrow().kind() == FirKind::Search
                        && node.borrow().as_search_pattern() == Some(pattern)
                    {
                        out.push(Rc::clone(node));
                    }
                    for ch in node.borrow().core().foolish_children().iter() {
                        recurse(ch, pattern, out);
                    }
                }
                recurse(c, pattern, &mut all);
            }
            all
        };

        eprintln!(
            "found {} searches for '^shadow$' in root children",
            shadow_searches.len()
        );
        for (i, s) in shadow_searches.iter().enumerate() {
            let sb = s.borrow();
            let nyes = sb.core().get_nyes();
            eprintln!("  shadow search [{}] NYES={:?}", i, nyes);
        }

        // The key diagnostic: is there a shadow search that settled NK?
        let nk_shadow = shadow_searches
            .iter()
            .any(|s| s.borrow().core().get_nyes() == foolish_core::fir::Nyes::Nk);
        eprintln!("any shadow search settled NK: {}", nk_shadow);

        // Also check: does cb.stmt_count() work?
        let cb_body = cb_stmt.borrow().core().foolish_children().first().cloned();
        if let Some(body) = cb_body {
            let bb = body.borrow();
            eprintln!(
                "cb body kind={:?}, nyes={:?}",
                bb.kind(),
                bb.core().get_nyes()
            );
            eprintln!("cb body stmt_count() = {:?}", bb.stmt_count());
            eprintln!(
                "cb body is_constanic_branelike() = {:?}",
                bb.is_constanic_branelike()
            );
            eprintln!(
                "cb body ubc_children count = {:?}",
                bb.core().ubc_children().len()
            );
            eprintln!(
                "cb body foolish_children count = {:?}",
                bb.core().foolish_children().len()
            );

            // Call stmt_count again to force lazy population:
            let sc_after = bb.stmt_count();
            eprintln!("cb body stmt_count() after first call = {:?}", sc_after);
            eprintln!(
                "cb body ubc_children after = {:?}",
                bb.core().ubc_children().len()
            );

            if let Some(sc) = sc_after {
                eprintln!("cb body has {} statements", sc);
                if let Some(s0) = bb.stmt_at(0) {
                    let s0_b = s0.borrow();
                    eprintln!("  stmt_at(0) name = {:?}", s0_b.as_stmt_searchable_name());
                }
            } else {
                eprintln!("cb body stmt_count() = None — BraneNavigator will get 0 candidates!");
            }

            // Now trace: what does the cb.search resolve to?
            // The "cb" Search inside {x=cb.shadow} — find it.
        }

        // Find the {x=cb.shadow} brane inside "extended" statement body.
        let extended_stmt = root_children
            .iter()
            .find(|c| c.borrow().as_stmt_searchable_name() == Some("extended"))
            .cloned()
            .expect("extended statement not found");
        let extended_body = extended_stmt
            .borrow()
            .core()
            .foolish_children()
            .first()
            .cloned();
        if let Some(ext_brane) = extended_body {
            let ext_kind = ext_brane.borrow().kind();
            eprintln!("extended body kind = {:?}", ext_kind);

            // The extended brane is itself a concatenation (multiple elements on one line).
            // Find the {x=cb.shadow} element inside it.
            if ext_kind == FirKind::Concatenation {
                let ext_children: Vec<FirRef> =
                    ext_brane.borrow().core().foolish_children().to_vec();
                eprintln!("extended ConcatBrane has {} elements", ext_children.len());
                for (i, elem) in ext_children.iter().enumerate() {
                    let eb = elem.borrow();
                    eprintln!("  elem[{}] kind={:?}", i, eb.kind());
                    // If it's a Brane, look for x=cb.shadow inside.
                    if eb.kind() == FirKind::Brane {
                        let stmts: Vec<FirRef> = eb.core().foolish_children().to_vec();
                        for s in &stmts {
                            let sb = s.borrow();
                            if sb.as_stmt_searchable_name() == Some("x") {
                                let x_body = sb
                                    .core()
                                    .foolish_children()
                                    .first()
                                    .cloned()
                                    .expect("x has no body");
                                let xb = x_body.borrow();
                                eprintln!(
                                    "  x body kind={:?}, pattern={:?}, anchored={}",
                                    xb.kind(),
                                    xb.as_search_pattern(),
                                    xb.as_search_anchored()
                                );
                                // The anchor of the dot-search:
                                if xb.kind() == FirKind::Search {
                                    let anchor = xb.core().foolish_children().first().cloned();
                                    if let Some(a) = anchor {
                                        let ab = a.borrow();
                                        eprintln!(
                                            "    anchor kind={:?}, nyes={:?}, pattern={:?}",
                                            ab.kind(),
                                            ab.core().get_nyes(),
                                            ab.as_search_pattern()
                                        );
                                        // What does value() return?
                                        let resolved = a.value();
                                        let rb = resolved.borrow();
                                        eprintln!(
                                            "    anchor.value() kind={:?}, nyes={:?}",
                                            rb.kind(),
                                            rb.core().get_nyes()
                                        );
                                        if rb.kind() == FirKind::Concatenation {
                                            eprintln!(
                                                "    resolved stmt_count={:?}",
                                                rb.stmt_count()
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// FOOP-33 Phase 9 — the `evaluator.rs` → `foolish-core` conversion boundary
/// resolves `CreationFir::get_display_name` and carries it through as
/// `core_fir::Fir::Creation { name }`. These tests exercise
/// `proto_to_core_fir` directly (rather than the whole `system.foo`-composed
/// `UbcaEvaluator::evaluate` pipeline) because the display-name rule is a
/// property of the raw compiled+stepped FIR tree, not of program composition.
#[cfg(test)]
mod creation_display_name_conversion_tests {
    use super::*;
    use crate::fir_trait::Scope;
    use foolish_core::FirQueryable;

    #[test]
    fn defining_site_creation_converts_unnamed() {
        // `'a = ⬤` — the creation is the whole RHS of statement `'a`. Convert
        // through the WHOLE composed root (`proto_to_core_fir`, exercising
        // the real `FirKind::Statement`/`FirKind::Brane` arms that thread
        // `current_stmt`), not the creation in isolation: at 'a's own
        // statement, `current_stmt` becomes 'a's own statement, so per the
        // revised two-condition rule (FOOP-33.md "Concerns Standing Past
        // Completion") it must NOT report a name: `{'a=⬤;}` sequencing as
        // `{'a='a}` reads as circular, not as "a fresh creation is being
        // introduced."
        let firs = Compiler::compile("{'a=⬤; b='a;}").unwrap();
        let root = firs[0].clone();
        let scope = Scope::empty();
        step_to_settled(&root, &scope, MAX_STEPS).unwrap();

        let converted = proto_to_core_fir(&root);
        assert_eq!(converted.hs_variant(), "NormalBrane");
        let core_fir::Fir::NormalBrane(brane) = &converted else {
            unreachable!("checked hs_variant() above");
        };
        let a_stmt = &brane.statements()[0];
        assert_eq!(a_stmt.name().as_deref(), Some("'a"));
        assert_eq!(
            a_stmt.body().borrow().clone_into_fir().hs_creation_name(),
            None,
            "the creation defining 'a, rendered at its OWN statement, \
             converts with NO name — only a reference reached elsewhere \
             shows the name"
        );
    }

    #[test]
    fn creation_reached_through_search_converts_with_its_own_defining_name() {
        // `b='a` resolves THROUGH a search to the SAME creation `Rc` that
        // `'a` defines (FOOP-33 Gotcha #2) — viewed from `b`'s statement
        // (a DIFFERENT statement than 'a's own), the conversion boundary
        // must report `'a`, not `b`, proving identity (not the referencing
        // statement's own name) drives the name, and that viewing from
        // elsewhere is what unlocks it.
        let firs = Compiler::compile("{'a=⬤; b='a;}").unwrap();
        let root = firs[0].clone();
        let scope = Scope::empty();
        step_to_settled(&root, &scope, MAX_STEPS).unwrap();

        let stmts = root.borrow().core().foolish_children().to_vec();
        let b_stmt = &stmts[1];
        let b_body = b_stmt.borrow().core().foolish_children()[0].clone();
        let resolved = b_body.value();
        assert_eq!(resolved.borrow().kind(), FirKind::Creation);

        // Convert `resolved` the same way the real Statement/Brane arms do:
        // with `current_stmt` set to the statement whose body is being
        // rendered (`b`'s statement).
        let converted = proto_to_core_fir_inner(&resolved, false, Some(b_stmt));
        assert_eq!(converted.hs_variant(), "Creation");
        assert_eq!(
            converted.hs_creation_name().as_deref(),
            Some("'a"),
            "a creation reached through a search, viewed from the \
             REFERENCING statement, converts with its OWN defining \
             statement's name"
        );
    }

    #[test]
    fn operator_operand_creation_converts_unnamed() {
        // `'a = 1 + ⬤` — the creation is only an OPERAND of `+`, not the
        // whole RHS, so it must convert with no name (glyph fallback).
        let firs = Compiler::compile("{'a=1+⬤; b='a;}").unwrap();
        let root = firs[0].clone();
        let scope = Scope::empty();
        step_to_settled(&root, &scope, MAX_STEPS).unwrap();

        let stmts = root.borrow().core().foolish_children().to_vec();
        let a_body = stmts[0].borrow().core().foolish_children()[0].clone();
        // a_body is the `+` operator; its second operand is the creation.
        let operator_children = a_body.borrow().core().foolish_children().to_vec();
        let creation = operator_children
            .iter()
            .find(|c| c.borrow().kind() == FirKind::Creation)
            .expect("the + operator must have a creation operand")
            .clone();

        let converted = proto_to_core_fir(&creation);
        assert_eq!(converted.hs_variant(), "Creation");
        assert_eq!(
            converted.hs_creation_name(),
            None,
            "a creation that is only an operand of an operator converts \
             with NO name — the statement's name belongs to the whole \
             expression, not to the creation inside it"
        );
    }
}

#[cfg(test)]
mod iteration_depth_tests {
    use super::*;

    #[test]
    fn returns_default_when_no_directive() {
        assert_eq!(parse_iteration_depth("{x = 1;}"), MAX_STEPS);
    }

    #[test]
    fn parses_directive_in_first_line() {
        assert_eq!(
            parse_iteration_depth("!! @einmo set iteration depth to 40000\n{x = 1;}"),
            40_000
        );
    }

    #[test]
    fn parses_directive_in_second_line() {
        assert_eq!(
            parse_iteration_depth("!! comment\n!! @einmo set iteration depth to 50000\n{x = 1;}"),
            50_000
        );
    }

    #[test]
    fn clamps_to_hard_limit() {
        assert_eq!(
            parse_iteration_depth("!! @einmo set iteration depth to 999999999\n{x = 1;}"),
            MAX_STEPS_HARD_LIMIT
        );
    }

    #[test]
    fn ignores_directive_after_third_line() {
        assert_eq!(
            parse_iteration_depth(
                "!! a\n!! b\n!! c\n!! @einmo set iteration depth to 40000\n{x = 1;}"
            ),
            MAX_STEPS
        );
    }
}
