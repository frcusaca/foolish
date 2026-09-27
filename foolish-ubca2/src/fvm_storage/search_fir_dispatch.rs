//! `SearchFir`'s own predicate-building and dispatch logic. Free functions taking `FirPointer` + `&mut
//! FVMStorage` explicitly, matching this module's `fir_op_step`/`combine` convention, rather than methods
//! on `FirPointer` itself — these are `SearchFir`-specific, not generic arena operations every kind needs.

use super::search_engine::{
    BraneNavigator, ScanOutcome, SearchPredicate, contextful_search_scan,
    contextful_search_scan_no_body_check,
};
use super::{FVMStorage, FirCursor, FirPointer, FirSpec};

use foolish_core::fir::Nyes;

fn nyes_from_found(found: Nyes) -> Nyes {
    super::nyes_from_found(found)
}

/// The statement a search found presents its written body, cloned via
/// `revive_constanic`.
///
/// **FOOP-86 §6.4 removed an NF-substitution branch here.** It used to
/// check the statement's `nf_reason` and present a fresh `Nk` node
/// instead of the written body — which is precisely the leak §6 exists
/// to close: a statement refused under FOOP-33 §4 poisoned every later
/// reader that resolved to it, destroying the very meaning the rule
/// exists to preserve. Under §6.3 the offending statement is NOT
/// refused at all (it reverts to Foolish and keeps its honest value);
/// the BRANE halts instead, so there is no poisoned value for a reader
/// to find.
pub(super) fn clone_stmt_result(
    storage: &mut FVMStorage,
    stmt: FirPointer,
    new_parent: FirPointer,
    sfm: bool,
) -> FirPointer {
    let body = storage
        .foolish_children(stmt)
        .first()
        .copied()
        .expect("statement must have a body");
    let index = FirCursor::new(stmt, storage).as_stmt_line_number().unwrap_or(0);
    storage.revive_constanic(body, new_parent, index, sfm, false)
}

/// Clones the found statement's value under `self`, pairs it with a `FoolRef` wrapping the ORIGINAL
/// statement (the two-child invariant), and moves to `Braning`.
fn handle_found(storage: &mut FVMStorage, ptr: FirPointer, stmt: FirPointer, sfm: bool) {
    let clone = clone_stmt_result(storage, stmt, ptr, sfm);
    // FOOP-86 §6.4b: if what was found is a HALTED brane, this access settles NK. `revive_constanic`
    // carries the cause onto the clone, so a plain reference (`r = bad`) is caught here the same way an
    // anchored search or an index is caught at its own anchor check — no access path yields a
    // meaningful value from a meaningless brane.
    if let Some(cause) = storage.unsteppable_cause(clone) {
        // The search RESOLVED -- it found `stmt` -- but what it found is
        // a halted brane, so the access settles NK (§6.4b). This is not a
        // miss: it found something, and that something is NK, which is
        // constantew, so no recoordination can ever change it.
        //
        // The NK is pushed as the search's RESULT, not merely set on this
        // node, so it settles through the ordinary path:
        // `settle_from_ubc_result` reads `ubc_children[0]`'s NYES and
        // maps it with `nyes_from_found` (Nk -> Nk).
        let reason = storage
            .alarm_reason(clone)
            .map_or_else(|| "unsteppable brane".to_string(), str::to_owned);
        let _ = cause;
        let nk = storage.make_orphan_child(ptr, FirSpec::Nk { reason });
        storage.with_mut(nk, |fir| fir.set_nyes(Nyes::Nk));
        let mut cursor = super::FirCursorMut::new(ptr, storage);
        cursor.push_search_result_pair(nk, stmt);
        cursor.set_nyes(Nyes::Nk);
        return;
    }
    let mut cursor = super::FirCursorMut::new(ptr, storage);
    cursor.push_search_result_pair(clone, stmt);
    cursor.set_nyes(Nyes::Braning);
}

/// The value a statement PRESENTS: its `settled_constanic_result()` (the NF-refusal NK, if already
/// refused) if set, else the raw written body. Used by the two NF-refusal checks below, which must
/// compare against what a PRIOR statement already presents, not its raw RHS — poisoning must be
/// transitive (FOOP-33 §4).
pub(super) fn statement_value_for_comparison(storage: &FVMStorage, stmt: FirPointer) -> Option<FirPointer> {
    stmt.settled_constanic_result(storage)
        .or_else(|| storage.foolish_children(stmt).first().copied())
}

/// Route 1 (FOOP-86 §6.2): a null-characterized statement checks ITSELF, once its body is constanic,
/// against any EARLIER same-name null-characterized statement (IB, then AB). If the two values are not
/// `Equal`, the statement is **unsteppable** — its name is already defined in this context — and its
/// brane halts (`halt_brane_at`). Terminal: does nothing once the brane is already halted.
pub(super) fn check_null_const_conflict(
    storage: &mut FVMStorage,
    stmt: FirPointer,
    body: FirPointer,
    current_statement: Option<FirPointer>,
    current_brane: Option<FirPointer>,
) {
    if stmt
        .home_brane(storage)
        .is_some_and(|b| storage.unsteppable_cause(b).is_some())
    {
        return;
    }
    let pattern = match storage.get(stmt) {
        FirSpec::Statement { identifier, .. } => identifier.searchable_name().to_string(),
        _ => return,
    };
    let prior = ib_search_by_pattern(storage, &pattern, current_statement)
        .or_else(|| ab_search_by_pattern(storage, &pattern, current_brane));
    let Some((prior_stmt, _)) = prior else {
        return; // no earlier definition -- this statement establishes the constant.
    };
    let Some(prior_body) = statement_value_for_comparison(storage, prior_stmt) else {
        return;
    };
    if !storage.get_nyes(prior_body).is_constanic() {
        return; // prior definition not yet constanic -- nothing to compare yet.
    }
    if super::default_equal(storage, body, prior_body) != super::Equality::Equal {
        let name = match storage.get(stmt) {
            FirSpec::Statement { identifier, .. } => identifier.identifier_name().to_string(),
            _ => return,
        };
        halt_brane_at(storage, stmt, format!("'{name} already defined in context"));
    }
}

/// Shared write path for every route to unsteppability (FOOP-86 §6.2's
/// four routes): records `stmt` as its home brane's **unsteppable
/// cause** and raises the run-time-error alarm on that brane (§6.2a).
///
/// **The statement itself is left alone** — no `Nk`, no marking, no
/// `ubc_children` push. That is the whole design (§6.3/§6.4): `3` in
/// `'K = 3` is an honest `IndepInt`/`Independent` and reverts to
/// Foolish; what failed is the BRANE, which cannot finish stepping in a
/// context where a null-characterized name was given a meaning it
/// cannot have. The superseded `refuse_statement` marked the statement,
/// which put the fault exactly where readers resolve to it and leaked
/// NK into every later reader of the name.
///
/// The actual halt (stop draining, set the brane `Nk`, leave the
/// remainder unstepped) happens in the brane's own `fir_op_step` arm,
/// which consults `unsteppable_cause`; this function only records.
fn halt_brane_at(storage: &mut FVMStorage, stmt: FirPointer, reason: String) {
    let Some(brane) = stmt.home_brane(storage) else {
        return;
    };
    if storage.unsteppable_cause(brane).is_some() {
        return; // already halted -- first cause wins (§6.4).
    }
    storage.set_unsteppable_cause(brane, stmt);
    storage.with_mut(brane, |fir| fir.set_alarm_reason(reason));
}

/// Route 3 (FOOP-86 §6.2): **recoordination**. A statement whose settled
/// value is a BRANE brings that brane's members into this context. If any
/// of those members is a null-characterized name that is ALREADY defined
/// here — with a different value — the brane cannot be coordinated in.
///
/// **The unsteppable statement is the one HOLDING the value**, not any member inside it: in
/// `{A={'C=10}, B={'C=11; A}}` it is `B#1`, the anonymous `A`, that goes NK. That is what makes
/// route 3 tractable — the members are nested inside an anonymous statement's brane rather than
/// being siblings, so nothing would find them by an ordinary prior-statement search, but the
/// statement that would introduce them is right here, and it is the thing that cannot step.
///
/// Compares against the enclosing brane's EARLIER statements only
/// (`already` is truncated at `stmt`'s own index): a brane coordinated in
/// before any conflicting definition exists is fine, exactly as routes
/// 1–2 permit a first definition and refuse only a later conflicting one.
pub(super) fn check_recoordinated_null_const_conflict(
    storage: &mut FVMStorage,
    stmt: FirPointer,
    body: FirPointer,
) {
    let Some(brane) = stmt.home_brane(storage) else {
        return;
    };
    if storage.unsteppable_cause(brane).is_some() {
        return;
    }
    let value = body.value(storage);
    if !FirCursor::new(value, storage).is_brane_like() || value == body {
        return; // not a brane, or not a REFERENCE to one -- nothing coordinated in.
    }
    let siblings: Vec<FirPointer> = storage.foolish_children(brane).to_vec();
    let Some(own_index) = siblings.iter().position(|&s| s == stmt) else {
        return;
    };
    let incoming: Vec<FirPointer> = storage.foolish_children(value).to_vec();
    for member in incoming {
        let Some(pattern) = (match storage.get(member) {
            FirSpec::Statement { identifier, .. } => identifier
                .is_nully_characterizing_coordinate_name()
                .then(|| identifier.searchable_name().to_string()),
            _ => None,
        }) else {
            continue;
        };
        let Some(&prior) = siblings[..own_index].iter().find(|&&s| {
            matches!(storage.get(s), FirSpec::Statement { identifier, .. }
                if identifier.searchable_name() == pattern)
        }) else {
            continue; // that name is not defined here yet -- coordinating it in is fine.
        };
        let (Some(prior_body), Some(member_body)) = (
            statement_value_for_comparison(storage, prior),
            statement_value_for_comparison(storage, member),
        ) else {
            continue;
        };
        if !storage.get_nyes(prior_body).is_constanic() || !storage.get_nyes(member_body).is_constanic() {
            continue;
        }
        if super::default_equal(storage, member_body, prior_body) != super::Equality::Equal {
            let name = match storage.get(member) {
                FirSpec::Statement { identifier, .. } => identifier.identifier_name().to_string(),
                _ => continue,
            };
            // The conflict belongs to THIS statement -- it is the one that cannot be stepped, because
            // coordinating the brane in would give `'{name}` a second meaning here. The statement
            // settles NK; the RECEIVING brane is untouched and keeps stepping its other statements:
            // `{A={'C=1}, B={'C=2, D=A}}` must yield `{A={'C=1}, B={'C=2, D=NK}}`. The brane segregates
            // the runtime error — outside this statement only an ordinary NK value propagates, by the
            // ordinary rules.
            let reason = format!("'{name} already defined in context");
            // The statement and its body settle NK, and the reason is
            // recorded as an alarm on each. `ubc_children` is deliberately
            // LEFT ALONE: the search genuinely FOUND its target, and `[0]`
            // is the true record of what it found. What failed is
            // COORDINATING that brane into this context, which is a fact
            // about this statement, not about the search's result.
            //
            // This also preserves the FoolRefFir two-child invariant
            // (`[0]` value, `[1]` FoolRef) that `&`-searches and result
            // chains depend on. The render-side consequence is handled
            // where it belongs, in `render_process_or_result`: a node that
            // itself settled NK while its result stayed conclusive reverts to its written
            // form. The search FIR carries the NK; it does not need to change `ubc_children`.
            for target in [body, stmt] {
                storage.with_mut(target, |fir| {
                    fir.set_nyes(Nyes::Nk);
                    fir.set_alarm_reason(reason.clone());
                });
            }
            return;
        }
    }
}

/// Route 4 (FOOP-86 §6.2): a null-characterized statement whose constanic value resolves to a
/// creation that ALREADY has a DIFFERENT original name is **unsteppable** — named creations
/// cannot be renamed (FOOP-33) — and its brane halts. It shares the routes 1–3 mechanism
/// (FOOP-86 §6.6 Q-C) because it is the same kind of fault: a null-characterized name given a
/// meaning it cannot have. Terminal, same guard as `check_null_const_conflict`.
pub(super) fn check_rename_of_named_creation(storage: &mut FVMStorage, stmt: FirPointer, body: FirPointer) {
    if stmt
        .home_brane(storage)
        .is_some_and(|b| storage.unsteppable_cause(b).is_some())
    {
        return;
    }
    let is_nully = match storage.get(stmt) {
        FirSpec::Statement { identifier, .. } => identifier.is_nully_characterizing_coordinate_name(),
        _ => return,
    };
    if !is_nully {
        return;
    }
    let resolved = body.value(storage);
    if !matches!(storage.get(resolved), FirSpec::Creation) {
        return; // not a creation reference at all -- nothing to forbid.
    }
    let Some(original_name) = resolved.get_display_name(storage, stmt) else {
        return; // the creation has no original name at all -- nothing to protect.
    };
    let pattern = match storage.get(stmt) {
        FirSpec::Statement { identifier, .. } => identifier.searchable_name().to_string(),
        _ => return,
    };
    if original_name != pattern {
        let name = match storage.get(stmt) {
            FirSpec::Statement { identifier, .. } => identifier.identifier_name().to_string(),
            _ => return,
        };
        halt_brane_at(storage, stmt, format!("'{name} is already a named creation"));
    }
}

/// The null-characterized name-constant rule (FOOP-33 §4), applied at concatenation merge time:
/// `check_null_const_conflict`'s own `fir_op_step` gate never fires for a merge-cloned statement
/// (`revive_constanic` builds it already-constanic, skipping `Prembrionic`/`Embryonic`/`Braning`
/// entirely), so this enforces the same rule directly, against statements already merged BEFORE
/// `new_stmt`. `already_merged` is searched in REVERSE (nearest-first) so a same-name chain compares
/// each new one against the NEAREST prior, transitively carrying any earlier refusal forward via
/// `statement_value_for_comparison`'s constanic-result-first read.
pub(super) fn apply_null_const_rule_to_merged_stmt(
    storage: &mut FVMStorage,
    new_stmt: FirPointer,
    already_merged: &[FirPointer],
    merged_brane: FirPointer,
) {
    let (is_nully, pattern) = match storage.get(new_stmt) {
        FirSpec::Statement { identifier, .. } => (
            identifier.is_nully_characterizing_coordinate_name(),
            identifier.searchable_name().to_string(),
        ),
        _ => return,
    };
    if !is_nully {
        return;
    }
    let Some(&prior_stmt) = already_merged.iter().rev().find(|&&s| {
        matches!(storage.get(s), FirSpec::Statement { identifier, .. }
            if identifier.searchable_name() == pattern)
    }) else {
        return; // first occurrence of this null-const name in the merge -- permitted.
    };
    let Some(new_body) = statement_value_for_comparison(storage, new_stmt) else {
        return;
    };
    let Some(prior_body) = statement_value_for_comparison(storage, prior_stmt) else {
        return;
    };
    if !storage.get_nyes(new_body).is_constanic() || !storage.get_nyes(prior_body).is_constanic() {
        return; // one side not yet constanic -- nothing to compare yet.
    }
    if super::default_equal(storage, new_body, prior_body) != super::Equality::Equal {
        let name = match storage.get(new_stmt) {
            FirSpec::Statement { identifier, .. } => identifier.identifier_name().to_string(),
            _ => return,
        };
        // Route 2 (FOOP-86 §6.2): the conflict arose during a concatenation merge rather than being
        // written directly, but it is the same fault and takes the same halt. `merged_brane` (the
        // `ConcatHelper` these clones are being added to) is passed explicitly rather than derived via
        // `home_brane`, because the clone's parent chain is still being built at this point.
        if storage.unsteppable_cause(merged_brane).is_none() {
            storage.set_unsteppable_cause(merged_brane, new_stmt);
            storage.with_mut(merged_brane, |fir| {
                fir.set_alarm_reason(format!("'{name} already defined in context"))
            });
        }
    }
}

/// Builds a single `ConcatHelper` holding ALL merged lines (no `MAX_BRANE_SIZE` limit).
/// Constanic-clones every element's statements (in order, via each element's OWN resolved `.value()`)
/// into one flat `ConcatHelper`, applying the null-const merge rule to each clone as it is added, then
/// pushes the helper as `ptr`'s sole `ubc_children` entry. Structural only — the CALLER decides `ptr`'s
/// own NYES. A no-op when there are no lines at all.
pub(super) fn populate_concat_helpers(storage: &mut FVMStorage, ptr: FirPointer) {
    let elements: Vec<FirPointer> = storage.foolish_children(ptr).to_vec();

    let total_lines: usize = elements
        .iter()
        .map(|&e| {
            let resolved = e.value(storage);
            FirCursor::new(resolved, storage).stmt_count().unwrap_or(0)
        })
        .sum();
    if total_lines == 0 {
        return;
    }

    // Build the (empty) helper first so its pointer becomes the parent of every cloned line —
    // cross-element search resolution walks to it. `make_orphan_child`, not `create_child`: the helper
    // belongs ONLY in `ubc_children`, never `foolish_children`.
    let helper = storage.make_orphan_child(ptr, FirSpec::ConcatHelper);

    let mut cloned_stmts: Vec<FirPointer> = Vec::with_capacity(total_lines);
    for &elem in &elements {
        let resolved = elem.value(storage);
        let count = FirCursor::new(resolved, storage).stmt_count().unwrap_or(0);
        for i in 0..count {
            let Some(stmt) = FirCursor::new(resolved, storage).stmt_at(i) else {
                continue;
            };
            let global_idx = cloned_stmts.len();
            let clone = storage.revive_constanic(stmt, helper, global_idx, false, false);
            apply_null_const_rule_to_merged_stmt(storage, clone, &cloned_stmts, helper);
            cloned_stmts.push(clone);
        }
    }

    if cloned_stmts.is_empty() {
        return;
    }
    let mut cursor = super::FirCursorMut::new(ptr, storage);
    cursor.push_ubc_child(helper);
}

pub(super) fn settle_from_ubc_result(storage: &mut FVMStorage, ptr: FirPointer) {
    let result_nyes = FirCursor::new(ptr, storage)
        .ubc_children()
        .first()
        .map(|&r| storage.get_nyes(r))
        .unwrap_or(Nyes::Nk);
    if result_nyes.is_constanic() {
        storage.with_mut(ptr, |fir| fir.set_nyes(nyes_from_found(result_nyes)));
    }
}

/// `self`'s value operand: index `1` if anchored (the anchor occupies `[0]`), else index `0`.
fn value_child(storage: &FVMStorage, ptr: FirPointer) -> FirPointer {
    let anchored = matches!(storage.get(ptr), FirSpec::Search { anchored: true, .. });
    let idx = if anchored { 1 } else { 0 };
    storage.foolish_children(ptr)[idx]
}

/// An immediate-brane name search, scanning backward from (but excluding) the current statement's own
/// position. `checked_sub`, not `saturating_sub`: a statement at position 0 has no preceding range at
/// all, and the `?` on `None` is exactly the self-hit guard.
fn ib_search_with_engine(
    storage: &FVMStorage,
    ptr: FirPointer,
    current_statement: Option<FirPointer>,
) -> Option<(FirPointer, Nyes)> {
    let pattern = match storage.get(ptr) {
        FirSpec::Search { pattern, .. } => pattern.clone(),
        _ => return None,
    };
    ib_search_by_pattern(storage, &pattern, current_statement)
}

/// Generalization of [`ib_search_with_engine`] taking the search pattern directly rather than reading
/// it off a `FirSpec::Search` node — needed by `check_null_const_conflict` (FOOP-33 §4), which searches
/// by the STATEMENT's own `searchable_name()`, not by a `Search` node's pattern (the statement itself
/// is not a `Search`).
pub(super) fn ib_search_by_pattern(
    storage: &FVMStorage,
    pattern: &str,
    current_statement: Option<FirPointer>,
) -> Option<(FirPointer, Nyes)> {
    let stmt = current_statement?;
    let brane = stmt.home_brane(storage)?;
    let idx = brane.find_stmt_index(storage, stmt)?;
    let search_end = idx.checked_sub(1)?;
    let mut nav = BraneNavigator::new(storage, brane, false);
    nav.set_range(0, search_end);
    let predicate = SearchPredicate::Name {
        pattern: pattern.to_string(),
    };
    match contextful_search_scan_no_body_check(storage, &mut nav, &predicate) {
        ScanOutcome::Found(found) => Some((found, storage.get_nyes(found))),
        _ => None,
    }
}

/// An ancestral-brane name search, climbing outward one brane at a time, scanning each ancestor's
/// statements strictly BEFORE the position the climb entered it from.
fn ab_search_with_engine(
    storage: &FVMStorage,
    ptr: FirPointer,
    current_brane: Option<FirPointer>,
) -> Option<(FirPointer, Nyes)> {
    let pattern = match storage.get(ptr) {
        FirSpec::Search { pattern, .. } => pattern.clone(),
        _ => return None,
    };
    ab_search_by_pattern(storage, &pattern, current_brane)
}

/// Generalization of [`ab_search_with_engine`] taking the search pattern directly — see
/// [`ib_search_by_pattern`]'s doc comment for why `StatementFir`'s NF-refusal checks need this shape.
pub(super) fn ab_search_by_pattern(
    storage: &FVMStorage,
    pattern: &str,
    current_brane: Option<FirPointer>,
) -> Option<(FirPointer, Nyes)> {
    let mut current_brane = current_brane?;
    loop {
        let stmt = current_brane.get_my_statement(storage);
        if stmt == current_brane {
            return None;
        }
        let parent_brane = stmt.home_brane(storage)?;
        if let Some(idx) = parent_brane.find_stmt_index(storage, stmt)
            && idx > 0
        {
            let mut nav = BraneNavigator::new(storage, parent_brane, false);
            nav.set_range(0, idx - 1);
            let predicate = SearchPredicate::Name {
                pattern: pattern.to_string(),
            };
            if let ScanOutcome::Found(found) =
                contextful_search_scan_no_body_check(storage, &mut nav, &predicate)
            {
                return Some((found, storage.get_nyes(found)));
            }
        }
        if parent_brane == current_brane {
            return None;
        }
        current_brane = parent_brane;
    }
}

/// Reads the anchor's `FoolRef` bookkeeping entry (`ubc_children[1]`, per the two-child invariant),
/// resolves ITS referent's home brane and position, then scans a range strictly AFTER (forward) or
/// BEFORE (backward) that position within the SAME home brane — a contexted search never leaves the
/// home brane (AGENTS.md §Searches).
fn contexted_search_from_anchor(
    storage: &FVMStorage,
    ptr: FirPointer,
    forward: bool,
) -> Option<(FirPointer, Nyes)> {
    let anchor = storage.foolish_children(ptr)[0];
    let fool_ref_fir = FirCursor::new(anchor, storage).ubc_children().get(1).copied()?;
    let referent = FirCursor::new(fool_ref_fir, storage).as_fool_ref_referent()?;
    let h_brane = referent.home_brane(storage)?;
    let p = h_brane.find_stmt_index(storage, referent)?;
    let brane_len = FirCursor::new(h_brane, storage).stmt_count().unwrap_or(0);
    if brane_len == 0 {
        return None;
    }
    let (scan_start, scan_end) = if forward {
        if p + 1 >= brane_len {
            return None;
        }
        (p + 1, brane_len - 1)
    } else {
        if p == 0 {
            return None;
        }
        (0, p - 1)
    };
    let mut nav = BraneNavigator::new(storage, h_brane, forward);
    nav.set_range(scan_start, scan_end);
    let (is_value_search, pattern) = match storage.get(ptr) {
        FirSpec::Search {
            is_value_search,
            pattern,
            ..
        } => (*is_value_search, pattern.clone()),
        _ => return None,
    };
    let predicate = if is_value_search {
        let value_fir = value_child(storage, ptr);
        SearchPredicate::Value { pattern: value_fir }
    } else if pattern.is_empty() {
        return None;
    } else {
        SearchPredicate::Name { pattern }
    };
    match contextful_search_scan(storage, &mut nav, &predicate) {
        ScanOutcome::Found(stmt) => {
            let nyes = storage
                .foolish_children(stmt)
                .first()
                .map(|&b| storage.get_nyes(b))
                .unwrap_or(Nyes::Nk);
            Some((stmt, nyes))
        }
        _ => None,
    }
}

/// The NAME-SEARCH path — the `is_value_search` branch is [`value_search_step`] below.
pub(crate) fn name_search_step(
    storage: &mut FVMStorage,
    ptr: FirPointer,
    current_statement: Option<FirPointer>,
    current_brane: Option<FirPointer>,
    has_ancestral_sfm: bool,
) {
    let (anchored, forward, contexted) = match storage.get(ptr) {
        FirSpec::Search {
            anchored,
            forward,
            contexted,
            ..
        } => (*anchored, *forward, *contexted),
        other => unreachable!("name_search_step called on non-Search spec: {other:?}"),
    };
    match storage.get_nyes(ptr) {
        Nyes::Prembrionic => {
            if anchored {
                let anchor = storage.foolish_children(ptr)[0];
                storage.with_mut(ptr, |fir| fir.push_task(anchor));
                storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Braning));
            } else {
                storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Embryonic));
            }
        }
        Nyes::Embryonic => {
            if anchored {
                let anchor = storage.foolish_children(ptr)[0];
                storage.with_mut(ptr, |fir| fir.push_task(anchor));
                storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Braning));
            } else if !FirCursor::new(ptr, storage).ubc_children().is_empty() {
                settle_from_ubc_result(storage, ptr);
            } else {
                match ib_search_with_engine(storage, ptr, current_statement) {
                    Some((stmt, _nyes)) => {
                        handle_found(storage, ptr, stmt, has_ancestral_sfm);
                        // Do NOT clobber a terminal state `handle_found` already reached. It settles NK
                        // when what it found is a halted brane (FOOP-86 §6.4b); unconditionally writing
                        // `Braning` here would regress that constanic node to pre-constanic and send it
                        // on to the AB stage, where an unanchored miss settles ECONSTANIC — leaving `r
                        // = bad` ECONSTANIC instead of NK.
                        if !storage.get_nyes(ptr).is_constanic() {
                            storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Braning));
                        }
                    }
                    None => storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Braning)),
                }
            }
        }
        Nyes::Braning => {
            if !FirCursor::new(ptr, storage).ubc_children().is_empty() {
                settle_from_ubc_result(storage, ptr);
            } else if contexted && anchored {
                match contexted_search_from_anchor(storage, ptr, forward) {
                    Some((stmt, _nyes)) => handle_found(storage, ptr, stmt, has_ancestral_sfm),
                    // `anchored` is always true in this branch, so a miss settles Nk (an unanchored
                    // miss would settle Econstanic instead).
                    None => storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Nk)),
                }
            } else if anchored {
                let anchor = storage.foolish_children(ptr)[0];
                let resolved = anchor.value(storage);
                if storage.get_nyes(resolved) == Nyes::Nk
                    || storage.unsteppable_cause(resolved).is_some()
                    || !FirCursor::new(resolved, storage).is_brane_like()
                {
                    storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Nk));
                } else {
                    let mut nav = BraneNavigator::new(storage, resolved, forward);
                    let pattern = match storage.get(ptr) {
                        FirSpec::Search { pattern, .. } => pattern.clone(),
                        _ => unreachable!(),
                    };
                    let predicate = SearchPredicate::Name { pattern };
                    match contextful_search_scan_no_body_check(storage, &mut nav, &predicate) {
                        ScanOutcome::Found(stmt) => {
                            handle_found(storage, ptr, stmt, has_ancestral_sfm);
                        }
                        _ => storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Nk)),
                    }
                }
            } else {
                match ab_search_with_engine(storage, ptr, current_brane) {
                    Some((stmt, _nyes)) => handle_found(storage, ptr, stmt, has_ancestral_sfm),
                    None => storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Econstanic)),
                }
            }
        }
        _ => {}
    }
}

/// Builds a `Value` predicate if the pattern is empty (`?=`/`~=`), else a `NameValue` predicate
/// (`?name=v`/`~name=v`). `None` if the value operand is not yet constanic — the caller
/// ([`check_value_pattern_ready`]) is responsible for confirming readiness first.
fn build_value_predicate(storage: &FVMStorage, ptr: FirPointer) -> Option<SearchPredicate> {
    let value_fir = value_child(storage, ptr);
    if !storage.get_nyes(value_fir).is_constanic() {
        return None;
    }
    let pattern = match storage.get(ptr) {
        FirSpec::Search { pattern, .. } => pattern.clone(),
        _ => return None,
    };
    if pattern.is_empty() {
        Some(SearchPredicate::Value { pattern: value_fir })
    } else {
        Some(SearchPredicate::NameValue {
            name: pattern,
            value: value_fir,
        })
    }
}

/// Gates the value-search dispatch on the value operand's own NYES (FOOP-23): pre-constanic → push as
/// task, not ready; NK → Nk; WOCONSTANIC → inherit Woconstanic (waiting on constanics, not a miss);
/// ECONSTANIC → inherit Econstanic; else confirm the resolved value is either an integer or a creation
/// (the two comparable value kinds), else Nk.
fn check_value_pattern_ready(storage: &mut FVMStorage, ptr: FirPointer) -> bool {
    let value_fir = value_child(storage, ptr);
    let nyes = storage.get_nyes(value_fir);
    if !nyes.is_constanic() {
        storage.with_mut(ptr, |fir| fir.push_task(value_fir));
        return false;
    }
    match nyes {
        Nyes::Nk => {
            storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Nk));
            return false;
        }
        Nyes::Woconstanic => {
            storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Woconstanic));
            return false;
        }
        Nyes::Econstanic => {
            storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Econstanic));
            return false;
        }
        _ => {}
    }
    let resolved = value_fir.value(storage);
    let resolved_is_creation = matches!(storage.get(resolved), FirSpec::Creation);
    if FirCursor::new(value_fir, storage).as_i64().is_none() && !resolved_is_creation {
        storage.with_mut(ptr, |fir| {
            fir.set_alarm_reason(
                "VALUE-SEARCH-UNSUPPORTED-PATTERN: pattern is neither integer nor creation".to_string(),
            )
        });
        storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Nk));
        return false;
    }
    true
}

/// The value-search dispatch (`?=`/`~=`/`?name=v`/`~name=v`), a distinct three-phase shape from
/// [`name_search_step`]'s two phases: `Prembrionic` pushes BOTH the anchor (if anchored) and the value
/// operand as tasks together (unlike name-search, which pushes only the anchor); `Embryonic`
/// (unanchored only — anchored searches skip straight to `Braning`) does the IB-equivalent backward
/// scan bounded to the enclosing statement's own position; `Braning` does the
/// contexted/anchored/unanchored (AB-style) dispatch, mirroring `name_search_step`'s `Braning` arm
/// shape closely but scanning with the value predicate instead of a name predicate.
pub(crate) fn value_search_step(storage: &mut FVMStorage, ptr: FirPointer, has_ancestral_sfm: bool) {
    let (anchored, forward, contexted) = match storage.get(ptr) {
        FirSpec::Search {
            anchored,
            forward,
            contexted,
            ..
        } => (*anchored, *forward, *contexted),
        other => unreachable!("value_search_step called on non-Search spec: {other:?}"),
    };
    match storage.get_nyes(ptr) {
        Nyes::Prembrionic => {
            if anchored {
                let anchor = storage.foolish_children(ptr)[0];
                storage.with_mut(ptr, |fir| fir.push_task(anchor));
            }
            let value_fir = value_child(storage, ptr);
            storage.with_mut(ptr, |fir| fir.push_task(value_fir));
            if anchored {
                storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Braning));
            } else {
                storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Embryonic));
            }
        }
        Nyes::Embryonic => {
            if !FirCursor::new(ptr, storage).ubc_children().is_empty() {
                settle_from_ubc_result(storage, ptr);
                return;
            }
            if !check_value_pattern_ready(storage, ptr) {
                return;
            }
            let predicate = build_value_predicate(storage, ptr).expect("checked ready");
            if let Some((stmt_ref, brane_ref)) = ptr.find_enclosing_stmt_and_brane(storage)
                && let Some(idx) = brane_ref.find_stmt_index(storage, stmt_ref)
                && idx > 0
            {
                let range_end = idx - 1;
                let mut nav = BraneNavigator::new(storage, brane_ref, false);
                nav.set_range(0, range_end);
                match contextful_search_scan(storage, &mut nav, &predicate) {
                    ScanOutcome::Found(stmt) => {
                        handle_found(storage, ptr, stmt, has_ancestral_sfm);
                    }
                    ScanOutcome::NkStop => {
                        storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Nk));
                        return;
                    }
                    ScanOutcome::Miss => {
                        if !anchored {
                            storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Econstanic));
                            return;
                        }
                    }
                }
            } else if !anchored {
                storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Econstanic));
                return;
            }
            storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Braning));
        }
        Nyes::Braning => {
            if !FirCursor::new(ptr, storage).ubc_children().is_empty() {
                settle_from_ubc_result(storage, ptr);
                return;
            }
            if !check_value_pattern_ready(storage, ptr) {
                return;
            }
            let predicate = build_value_predicate(storage, ptr).expect("checked ready");
            let scan_outcome = if contexted && anchored {
                let anchor = storage.foolish_children(ptr)[0];
                let anchor_constanic = storage.get_nyes(anchor).is_constanic();
                match contexted_search_from_anchor(storage, ptr, forward) {
                    Some((stmt, _nyes)) => {
                        handle_found(storage, ptr, stmt, has_ancestral_sfm);
                        return;
                    }
                    None => {
                        if !anchor_constanic {
                            return;
                        }
                        ScanOutcome::Miss
                    }
                }
            } else if anchored {
                let anchor = storage.foolish_children(ptr)[0];
                let resolved = anchor.value(storage);
                if storage.get_nyes(resolved) == Nyes::Nk || storage.unsteppable_cause(resolved).is_some() {
                    storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Nk));
                    return;
                }
                if !FirCursor::new(resolved, storage).is_brane_like() {
                    ScanOutcome::Miss
                } else {
                    let mut nav = BraneNavigator::new(storage, resolved, forward);
                    contextful_search_scan(storage, &mut nav, &predicate)
                }
            } else {
                match ptr.find_enclosing_stmt_and_brane(storage) {
                    Some((stmt_ref, brane_ref)) => {
                        if let Some(idx) = brane_ref.find_stmt_index(storage, stmt_ref) {
                            let len = storage.foolish_children(brane_ref).len();
                            if idx + 1 < len {
                                let mut nav = BraneNavigator::new(storage, brane_ref, true);
                                nav.set_range(idx + 1, len - 1);
                                contextful_search_scan(storage, &mut nav, &predicate)
                            } else {
                                ScanOutcome::Miss
                            }
                        } else {
                            ScanOutcome::Miss
                        }
                    }
                    None => ScanOutcome::Miss,
                }
            };
            match scan_outcome {
                ScanOutcome::Found(stmt) => {
                    handle_found(storage, ptr, stmt, has_ancestral_sfm);
                }
                ScanOutcome::NkStop => {
                    storage.with_mut(ptr, |fir| fir.set_nyes(Nyes::Nk));
                }
                ScanOutcome::Miss => {
                    let settle = if anchored { Nyes::Nk } else { Nyes::Econstanic };
                    storage.with_mut(ptr, |fir| fir.set_nyes(settle));
                }
            }
        }
        _ => {}
    }
}
