use foolish_core::fir as core_fir;
use foolish_core::fir::{
    Alarm, AlarmLevel, AlarmSource, ConcatenationFirBuilder, ConstantIntFirBuilder,
    FirRef as CoreFirRef, IndexFirBuilder, NkFirBuilder, NormalBraneFirBuilder, Nyes,
    OperatorFirBuilder, SearchFirBuilder, StayFoolishFirBuilder, StayFullyFoolishFirBuilder,
};

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
        let ubca_firs =
            Compiler::compile(source).map_err(|e| format!("Compilation failed: {}", e))?;

        let scope = crate::fir_trait::Scope::empty();
        let mut results = Vec::new();

        for fir_ref in &ubca_firs {
            if let Err(alarm) = step_to_settled(fir_ref, &scope) {
                let alarm_msg = alarm.to_string();
                fir_ref.borrow().core().set_alarm_reason(alarm_msg.clone());
                fir_ref.borrow().core().set_nyes(Nyes::Nk);
                eprintln!("ALARM: {alarm_msg}");
            }
            let core_fir = proto_to_core_fir(fir_ref);
            results.push(core_fir::fir_to_ref(core_fir));
        }

        Ok(results)
    }
}

fn step_to_settled(
    fir_ref: &FirRef,
    scope: &crate::fir_trait::Scope,
) -> Result<(), crate::fir_trait::UbcError> {
    let mut last_step = 0;
    for step in 0..MAX_STEPS {
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
    proto_to_core_fir_inner(ubca_ref, false)
}

/// Convert an SFF body expression. Top-level searches get EMBRYONIC state
/// (shown by sequencer). Operator operands get CONSTANT state (hidden).
/// Operators get WOCONSTANIC or CONSTANT state based on operand states.
/// (@Agents, I suppose this can't be declared as implementation on something
///   associated with SFF marker like SFFMark? ditto, similar questions for the
///   other '^fn' declarations in this file.)
fn proto_to_core_fir_sff_body(ubca_ref: &FirRef) -> core_fir::Fir {
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
                .map(proto_to_core_fir_sff_operand)
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
        _ => proto_to_core_fir_inner(ubca_ref, true),
    }
}

/// Convert an SFF operator operand. Searches get CONSTANT state (no state shown).
fn proto_to_core_fir_sff_operand(ubca_ref: &FirRef) -> core_fir::Fir {
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
        _ => proto_to_core_fir_inner(ubca_ref, true),
    }
}

fn anchor_to_core_fir(ubca_ref: &FirRef) -> core_fir::Fir {
    let borrowed = ubca_ref.borrow();
    let kind = borrowed.kind();
    let state = borrowed.core().get_nyes();

    if kind == FirKind::Search {
        return SearchFirBuilder::new(borrowed.as_search_pattern().unwrap_or(""))
            .anchored(borrowed.as_search_anchored())
            .state(state)
            .build();
    }

    proto_to_core_fir_inner(ubca_ref, true)
}

fn proto_to_core_fir_inner(ubca_ref: &FirRef, preserve_search: bool) -> core_fir::Fir {
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
        FirKind::Operator => {
            // Unwrap to the result when the operator successfully computed
            // a constant value.
            if state == Nyes::Constant {
                let ubc = borrowed.core().ubc_children();
                if let Some(result) = ubc.first() {
                    return proto_to_core_fir_inner(result, preserve_search);
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
                        return proto_to_core_fir_inner(result, preserve_search);
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
                            proto_to_core_fir_inner(c, preserve_search)
                        }
                    })
                    .collect()
            } else {
                borrowed
                    .core()
                    .foolish_children()
                    .iter()
                    .map(|c| proto_to_core_fir_inner(c, preserve_search))
                    .collect()
            };
            OperatorFirBuilder::new(op)
                .operands(operand_firs)
                .state(state)
                .build()
        }
        FirKind::Statement => {
            let name = display_stmt_name(borrowed.as_stmt_searchable_name());
            let body_fir = borrowed
                .core()
                .foolish_children()
                .first()
                .map(|c| proto_to_core_fir_inner(c, preserve_search))
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
                    let cb = c.borrow();
                    let name = display_stmt_name(cb.as_stmt_searchable_name());
                    let body_fir = cb
                        .core()
                        .foolish_children()
                        .first()
                        .map(|c| proto_to_core_fir_inner(c, preserve_search))
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
                            let inner_result_fir =
                                proto_to_core_fir_inner(inner_ubc.first().unwrap(), false);
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

                    let resolved = proto_to_core_fir_inner(result, preserve_search);
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
                if has_anchor
                    && let Some(a) = children.first()
                {
                    builder = builder.anchor(proto_to_core_fir_inner(a, false));
                }
                let value_idx = if has_anchor { 1 } else { 0 };
                if let Some(v) = children.get(value_idx) {
                    builder = builder.value(proto_to_core_fir_inner(v, false));
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
                    let resolved = proto_to_core_fir_inner(result, preserve_search);
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
                        builder = builder.anchor(anchor_to_core_fir(anchor_ref));
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
                builder = builder.anchor(anchor_to_core_fir(anchor_ref));
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
                                let inner_result_fir = proto_to_core_fir_inner(result, false);
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
                                let inner_result_fir = proto_to_core_fir_inner(result, false);
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
                            let inner_result_fir = proto_to_core_fir_inner(result, false);
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
                .map(|c| proto_to_core_fir_inner(c, true))
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
                .map(proto_to_core_fir_sff_body)
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
                let count = borrowed.stmt_count().unwrap_or(0);
                let stmt_tuples: Vec<(Option<String>, core_fir::Fir)> = (0..count)
                    .filter_map(|i| {
                        let stmt = borrowed.stmt_at(i)?;
                        let sb = stmt.borrow();
                        let name = display_stmt_name(sb.as_stmt_searchable_name());
                        let body_fir = sb
                            .core()
                            .foolish_children()
                            .first()
                            .map(|c| proto_to_core_fir_inner(c, preserve_search))
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
                .map(|c| proto_to_core_fir_inner(c, preserve_search))
                .collect();
            ConcatenationFirBuilder::new()
                .elements(elem_firs)
                .state(state)
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
                    let body_fir = cb
                        .core()
                        .foolish_children()
                        .first()
                        .map(|c| proto_to_core_fir_inner(c, preserve_search))
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
        FirKind::Creation => core_fir::Fir::Creation,
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

        step_to_settled(&root, &scope).unwrap();
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
        step_to_settled(&root, &scope).unwrap();

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
            eprintln!("cb body is_brane_like() = {:?}", bb.is_brane_like());
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
