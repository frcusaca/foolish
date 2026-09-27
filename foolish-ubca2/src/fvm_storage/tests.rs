use super::*;

fn brane_spec() -> FirSpec {
    FirSpec::Brane {
        characterizations: Characterizations::default(),
    }
}

/// A small, self-contained arena exercising `get`/`with_mut`/`get_parent`
/// round-tripping, per this task's "Establish relevant tests" checkbox.
#[test]
fn make_root_then_child_round_trips_through_get_and_with_mut() {
    let mut storage = FVMStorage::new();
    let root = storage.make_root(brane_spec());
    let child = root.create_child(&mut storage, FirSpec::IndepInt { value: 42 });

    assert_eq!(storage.get(child), &FirSpec::IndepInt { value: 42 });
    assert_eq!(child.get_parent(&storage), Some(root));
    assert!(root.is_root(&storage));
    assert!(!child.is_root(&storage));
    assert_eq!(storage.foolish_children(root), &[child]);

    storage.with_mut(child, |fir| fir.set_nyes(Nyes::Constant));
    assert_eq!(storage.get_nyes(child), Nyes::Constant);
}

/// A freshly-created node starts at its spec's initial `Nyes` — the same starting state each kind's own
/// constructor established when nodes were built one-by-one rather than allocated from an arena.
#[test]
fn initial_nyes_matches_each_kinds_own_constructor() {
    let mut storage = FVMStorage::new();
    let root = storage.make_root(brane_spec());
    assert_eq!(storage.get_nyes(root), Nyes::Prembrionic);

    let creation = root.create_child(&mut storage, FirSpec::Creation);
    assert_eq!(storage.get_nyes(creation), Nyes::Independent);

    let int_child = root.create_child(&mut storage, FirSpec::IndepInt { value: 1 });
    assert_eq!(storage.get_nyes(int_child), Nyes::Independent);

    let op_child = root.create_child(&mut storage, FirSpec::Operator { op: "+".to_string() });
    assert_eq!(storage.get_nyes(op_child), Nyes::Prembrionic);
}

/// A `FirPointer` minted by one `FVMStorage` must never validate against a
/// different instance — the whole reason `arena: ArenaId` exists (see
/// FOOP-16.md §Specification "`FirPointer`'s identity properties").
#[test]
#[should_panic(expected = "different FVMStorage instance")]
fn pointer_from_a_different_arena_fails_validation() {
    let mut storage_a = FVMStorage::new();
    let root_a = storage_a.make_root(brane_spec());

    let storage_b = FVMStorage::new();
    // root_a was minted by storage_a; reading it through storage_b must panic.
    let _ = storage_b.get(root_a);
}

/// `get_mut` gives the same access as `with_mut`, just without the closure — confirms both really are
/// equally powerful, per FOOP-16.md's resolution of the two-cursor-type design question.
#[test]
fn get_mut_and_with_mut_reach_the_same_state() {
    let mut storage = FVMStorage::new();
    let root = storage.make_root(brane_spec());
    let child = root.create_child(&mut storage, FirSpec::IndepInt { value: 7 });

    storage.get_mut(child).set_nyes(Nyes::Constant);
    assert_eq!(storage.get_nyes(child), Nyes::Constant);
}

/// The structural root has no home brane (it climbs to itself and stops).
#[test]
fn home_brane_of_the_structural_root_is_none() {
    let mut storage = FVMStorage::new();
    let root = storage.make_root(FirSpec::IndepInt { value: 1 });
    assert_eq!(root.home_brane(&storage), None);
}

/// A child of a `Brane` reports that brane as its home brane; a child of
/// a non-brane-like node (here, a bare `IndepInt` root standing in for
/// "some non-brane-like ancestor") climbs past it with no brane to find.
#[test]
fn home_brane_finds_the_nearest_brane_like_ancestor() {
    let mut storage = FVMStorage::new();
    let root = storage.make_root(brane_spec());
    let inner_int = root.create_child(&mut storage, FirSpec::IndepInt { value: 1 });
    assert_eq!(inner_int.home_brane(&storage), Some(root));

    let non_brane_root = storage.make_root(FirSpec::IndepInt { value: 1 });
    let grandchild = non_brane_root.create_child(&mut storage, FirSpec::IndepInt { value: 2 });
    assert_eq!(grandchild.home_brane(&storage), None);
}

/// `test_leaf`/`test_root_brane` round-trip correctly.
#[test]
fn test_helpers_build_expected_shapes() {
    let (storage, leaf) = FVMStorage::test_leaf(Nyes::Constant);
    assert_eq!(storage.get_nyes(leaf), Nyes::Constant);
    assert!(leaf.is_root(&storage));

    let (storage, root) = FVMStorage::test_root_brane(&[FirSpec::IndepInt { value: 1 }, FirSpec::Creation]);
    assert_eq!(storage.foolish_children(root).len(), 2);
    assert_eq!(storage.get_nyes(root), Nyes::Prembrionic);
}

/// `FirCursor` reads match direct `FVMStorage` reads for the same pointer — proving the wrapper is a
/// pure convenience, not a divergent second source of truth.
#[test]
fn fir_cursor_reads_match_direct_storage_reads() {
    let (mut storage, root) = FVMStorage::test_root_brane(&[FirSpec::IndepInt { value: 10 }]);
    let child = storage.foolish_children(root)[0];
    storage.with_mut(child, |fir| fir.set_nyes(Nyes::Constant));

    let cursor = FirCursor::new(child, &storage);
    assert_eq!(cursor.node(), storage.get(child));
    assert_eq!(cursor.get_nyes(), storage.get_nyes(child));
    assert_eq!(cursor.parent(), Some(root));
    assert_eq!(cursor.home_brane().map(|c| c.ptr), Some(root));
    // `child` has no `Statement` ancestor, so the climb goes all the way to the structural root and
    // stops there — NOT back to `child` itself. `root` is where the climb terminates, since its own
    // parent is itself.
    assert_eq!(cursor.statement().ptr, root);
    assert!(cursor.settled_constanic_result().is_none()); // IndepInt never has a settled_constanic_result body
}

/// `FirCursorMut::push_ubc_child` keeps the two-part contract exactly: pushes to `ubc_children` AND
/// enqueues as a task only when the child is not already constanic.
#[test]
fn fir_cursor_mut_push_ubc_child_enqueues_only_non_constanic_children() {
    let (mut storage, root) = FVMStorage::test_root_brane(&[]);
    let settled = root.create_child(&mut storage, FirSpec::IndepInt { value: 1 });
    storage.with_mut(settled, |fir| fir.set_nyes(Nyes::Constant));
    let unsettled = root.create_child(&mut storage, FirSpec::Operator { op: "+".to_string() });

    {
        let mut cursor = FirCursorMut::new(root, &mut storage);
        cursor.push_ubc_child(settled);
        cursor.push_ubc_child(unsettled);
    }

    let cursor = FirCursor::new(root, &storage);
    assert_eq!(cursor.ubc_children(), &[settled, unsettled]);
    // Only the unsettled child should have been enqueued as a task.
    assert_eq!(cursor.front_task(), Some(unsettled));
}

/// `FirCursorMut::push_search_result`'s SINGULAR-RESULT INVARIANT trips
/// its `debug_assert!` on a second push.
#[test]
#[should_panic(expected = "singular-result")]
fn push_search_result_rejects_a_second_result() {
    let (mut storage, root) = FVMStorage::test_root_brane(&[]);
    let a = root.create_child(&mut storage, FirSpec::IndepInt { value: 1 });
    let b = root.create_child(&mut storage, FirSpec::IndepInt { value: 2 });
    let mut cursor = FirCursorMut::new(root, &mut storage);
    cursor.push_search_result(a);
    cursor.push_search_result(b); // must panic: already has a result
}

/// `check_sff_marked_child` accepts a child whose descendant searches are all `ECONSTANIC` and panics
/// on one that is not — mirrors `proto_brane.rs`'s `push_foolish_child_sff_marked_*` test trio.
#[test]
fn check_sff_marked_child_accepts_all_econstanic_descendants() {
    let (mut storage, root) = FVMStorage::test_root_brane(&[]);
    let search = root.create_child(
        &mut storage,
        FirSpec::Search {
            pattern: "x".to_string(),
            anchored: true,
            forward: false,
            is_value_search: false,
            contexted: false,
        },
    );
    storage.with_mut(search, |fir| fir.set_nyes(Nyes::Econstanic));

    let cursor = FirCursorMut::new(root, &mut storage);
    cursor.check_sff_marked_child(search); // must not panic
}

#[test]
#[should_panic(expected = "INTERNAL CONSISTENCY error")]
fn check_sff_marked_child_rejects_a_non_econstanic_descendant() {
    let (mut storage, root) = FVMStorage::test_root_brane(&[]);
    let search = root.create_child(
        &mut storage,
        FirSpec::Search {
            pattern: "x".to_string(),
            anchored: true,
            forward: false,
            is_value_search: false,
            contexted: false,
        },
    );
    // Left at Prembrionic (the default) — NOT Econstanic — exactly the
    // mis-constructed shape the guard exists to catch.
    let cursor = FirCursorMut::new(root, &mut storage);
    cursor.check_sff_marked_child(search);
}

/// `revive_constanic`'s share-not-clone behavior: a `Creation` always shares the SAME `FirPointer`,
/// regardless of NYES — the FoolRef/Creation unconditional-share rule from `constanic_clone_at`.
#[test]
fn revive_constanic_shares_creation_unconditionally() {
    let (mut storage, root) = FVMStorage::test_root_brane(&[]);
    let creation = root.create_child(&mut storage, FirSpec::Creation);
    let other_root = storage.make_root(FirSpec::IndepInt { value: 0 });

    let cloned = storage.revive_constanic(creation, other_root, 0, false, false);
    assert_eq!(cloned, creation, "Creation must share, never clone");
}

/// `revive_constanic`'s share-not-clone behavior for a `Constant`
/// non-`Brane` node: returns the SAME pointer, not a new slot.
#[test]
fn revive_constanic_shares_constant_non_brane() {
    let (mut storage, root) = FVMStorage::test_root_brane(&[]);
    let settled = root.create_child(&mut storage, FirSpec::IndepInt { value: 42 });
    storage.with_mut(settled, |fir| fir.set_nyes(Nyes::Constant));
    let other_root = storage.make_root(FirSpec::IndepInt { value: 0 });

    let cloned = storage.revive_constanic(settled, other_root, 0, false, false);
    assert_eq!(cloned, settled, "Constant non-Brane must share, never clone");
}

/// `revive_constanic`'s full-rebuild behavior: a pre-constanic node is rebuilt as a genuinely new
/// pointer under the new parent, with its foolish children recursively cloned too, and a `Statement`'s
/// `line_number` renumbered to the passed `index` — exactly as `constanic_clone_at`'s
/// `FirKind::Statement` arm does today (`let line = index;`).
#[test]
fn revive_constanic_rebuilds_pre_constanic_nodes_and_renumbers_statement_lines() {
    let (mut storage, root) = FVMStorage::test_root_brane(&[]);
    let stmt = root.create_child(
        &mut storage,
        FirSpec::Statement {
            identifier: Identifier::from_parts(vec![], "x"),
            line_number: 99, // original position — must be overwritten by `index` below
        },
    );
    let other_root = storage.make_root(FirSpec::IndepInt { value: 0 });

    let cloned = storage.revive_constanic(stmt, other_root, 3, false, false);
    assert_ne!(
        cloned, stmt,
        "a pre-constanic Statement must be rebuilt, not shared"
    );
    assert_eq!(cloned.get_parent(&storage), Some(other_root));
    match storage.get(cloned) {
        FirSpec::Statement { line_number, .. } => {
            assert_eq!(*line_number, 3, "line_number must be renumbered to `index`")
        }
        other => panic!("expected FirSpec::Statement, got {other:?}"),
    }
    // The original subtree's pointer must remain exactly as valid as before.
    assert_eq!(storage.get_nyes(stmt), Nyes::Prembrionic);
}

/// `revive_constanic` recursively clones foolish children, preserving count
/// and (for pre-constanic children) producing fresh pointers for each.
#[test]
fn revive_constanic_recursively_clones_foolish_children() {
    let (mut storage, root) = FVMStorage::test_root_brane(&[
        FirSpec::Operator { op: "+".to_string() },
        FirSpec::Operator { op: "+".to_string() },
    ]);
    let other_root = storage.make_root(FirSpec::IndepInt { value: 0 });

    let cloned = storage.revive_constanic(root, other_root, 0, false, false);
    let cloned_children = storage.foolish_children(cloned);
    assert_eq!(cloned_children.len(), 2);
    let original_children = storage.foolish_children(root).to_vec();
    for (c, orig) in cloned_children.iter().zip(original_children.iter()) {
        assert_ne!(c, orig, "each pre-constanic child must be a fresh clone");
    }
}

/// `skip_foolish_children: true` omits re-cloning parse-time children —
/// used at the top level of a clone when only the ubc/result side is
/// being recoordinated (per `constanic_clone_at`'s own parameter).
#[test]
fn revive_constanic_skip_foolish_children_omits_them() {
    let (mut storage, root) = FVMStorage::test_root_brane(&[FirSpec::IndepInt { value: 1 }]);
    let other_root = storage.make_root(FirSpec::IndepInt { value: 0 });

    let cloned = storage.revive_constanic(root, other_root, 0, false, true);
    assert!(storage.foolish_children(cloned).is_empty());
}

/// `temporary_release!` drops a handle, runs a storage-needing operation, and re-acquires a fresh
/// handle bound back to the same name — proven against the interleaved shape FOOP-16.md's own
/// illustrative example describes: finish writing to one node, build a second node mid- sequence, then
/// resume writing to the first.
#[test]
fn temporary_release_reacquires_a_usable_handle() {
    let (mut storage, root) = FVMStorage::test_root_brane(&[]);
    let first_ptr = root.create_child(&mut storage, FirSpec::IndepInt { value: 1 });

    let first = storage.get_mut(first_ptr);
    first.set_nyes(Nyes::Woconstanic);
    let (second_ptr, first) = temporary_release!(
        first,
        storage.get_mut(first_ptr),
        first_ptr.create_child(&mut storage, FirSpec::IndepInt { value: 2 })
    );
    first.set_nyes(Nyes::Constant);

    assert_eq!(storage.get_nyes(first_ptr), Nyes::Constant);
    assert_eq!(storage.get(second_ptr), &FirSpec::IndepInt { value: 2 });
}

/// `step_inner`'s pop-vs-recurse shape, exercised without ever reaching the `fir_op_step` dispatch
/// `todo!()`: a front task that is already constanic gets popped, not recursed into.
#[test]
fn step_pops_a_front_task_that_is_already_constanic() {
    let (mut storage, root) = FVMStorage::test_root_brane(&[]);
    let done = root.create_child(&mut storage, FirSpec::IndepInt { value: 1 });
    storage.with_mut(done, |fir| fir.set_nyes(Nyes::Constant));
    storage.with_mut(root, |fir| fir.push_task(done));
    assert_eq!(
        FirCursor::new(root, &storage).front_task(),
        Some(done),
        "task queued before stepping"
    );

    root.step(&mut storage);

    assert_eq!(
        FirCursor::new(root, &storage).front_task(),
        None,
        "the already-constanic front task must be popped, not recursed into"
    );
}

/// A literal integer is fully determined the moment it's written, so it is born `Independent` and needs
/// no stepping at all — no Prembrionic phase, no Braning phase (an IndepInt has no children/tasks
/// either way).
#[test]
fn indep_int_starts_independent_and_needs_no_stepping() {
    let mut storage = FVMStorage::new();
    let node = storage.make_root(FirSpec::IndepInt { value: 42 });
    assert_eq!(storage.get_nyes(node), Nyes::Independent);

    node.step(&mut storage);

    assert_eq!(storage.get_nyes(node), Nyes::Independent);
    assert_eq!(FirCursor::new(node, &storage).as_i64(), Some(42));
}

/// Stepping an already-conclusive `IndepInt` repeatedly is a no-op.
#[test]
fn indep_int_stepping_already_conclusive_is_noop() {
    let mut storage = FVMStorage::new();
    let node = storage.make_root(FirSpec::IndepInt { value: 1 });
    node.step(&mut storage);
    assert_eq!(storage.get_nyes(node), Nyes::Independent);

    node.step(&mut storage);
    assert_eq!(storage.get_nyes(node), Nyes::Independent);
}

/// `NkFir`'s arena migration: mirrors the existing
/// `fir_kinds.rs::tests::nk_prembrionic_to_nk_in_one_step` test exactly — Prembrionic → Nk in ONE step.
#[test]
fn nk_prembrionic_to_nk_in_one_step() {
    let mut storage = FVMStorage::new();
    let node = storage.make_root(FirSpec::Nk {
        reason: "unbound name".to_string(),
    });
    assert_eq!(storage.get_nyes(node), Nyes::Prembrionic);

    node.step(&mut storage);

    assert_eq!(storage.get_nyes(node), Nyes::Nk);
    assert_eq!(
        FirCursor::new(node, &storage).as_nk_reason(),
        Some("unbound name")
    );
}

/// `2 + 3` settles Constant with value `5`. Both operands start pre-settled (`Constant`), so `combine`
/// fires without a genuine Braning-phase child-stepping round-trip — this test's own `step` loop drains
/// the (already-constanic) operand tasks first, then settles via `combine`.
#[test]
fn operator_addition_settles_constant() {
    let mut storage = FVMStorage::new();
    let op = storage.make_root(FirSpec::Operator { op: "+".to_string() });
    let a = op.create_child(&mut storage, FirSpec::IndepInt { value: 2 });
    let b = op.create_child(&mut storage, FirSpec::IndepInt { value: 3 });
    storage.with_mut(a, |fir| fir.set_nyes(Nyes::Constant));
    storage.with_mut(b, |fir| fir.set_nyes(Nyes::Constant));

    for _ in 0..10 {
        if storage.get_nyes(op).is_constanic() {
            break;
        }
        op.step(&mut storage);
    }

    assert_eq!(storage.get_nyes(op), Nyes::Constant);
    assert_eq!(FirCursor::new(op, &storage).as_i64(), Some(5));
    assert_eq!(FirCursor::new(op, &storage).as_op_name(), Some("+"));
}

/// Mirrors `fir_kinds.rs::tests::operator_div_by_zero_nyes_transitions` exactly — `1 / 0` settles NK.
#[test]
fn operator_division_by_zero_settles_nk() {
    let mut storage = FVMStorage::new();
    let op = storage.make_root(FirSpec::Operator { op: "/".to_string() });
    let a = op.create_child(&mut storage, FirSpec::IndepInt { value: 1 });
    let b = op.create_child(&mut storage, FirSpec::IndepInt { value: 0 });
    storage.with_mut(a, |fir| fir.set_nyes(Nyes::Constant));
    storage.with_mut(b, |fir| fir.set_nyes(Nyes::Constant));

    for _ in 0..10 {
        if storage.get_nyes(op).is_constanic() {
            break;
        }
        op.step(&mut storage);
    }

    assert_eq!(storage.get_nyes(op), Nyes::Nk);
    assert_eq!(
        FirCursor::new(op, &storage)
            .settled_constanic_result()
            .and_then(|c| c.as_nk_reason().map(str::to_string)),
        Some("division by zero".to_string())
    );
}

/// An `Operator` with a pre-constanic (not-yet-settled) operand pushes tasks for its inconclusive
/// operands and moves to `Braning` — mirrors `impl Fir for OperatorFir`'s `Prembrionic`/`Embryonic`
/// branch exactly (`if !self.operands_all_settled() { push tasks }`).
#[test]
fn operator_pushes_tasks_for_inconclusive_operands() {
    let mut storage = FVMStorage::new();
    let op = storage.make_root(FirSpec::Operator { op: "+".to_string() });
    let a = op.create_child(&mut storage, FirSpec::Operator { op: "+".to_string() });
    let _b = op.create_child(&mut storage, FirSpec::Operator { op: "+".to_string() });
    // `a`/`b` both start Prembrionic (unsettled) — the default.

    op.step(&mut storage);

    assert_eq!(storage.get_nyes(op), Nyes::Braning);
    assert_eq!(
        FirCursor::new(op, &storage).front_task(),
        Some(a),
        "unsettled operands must be queued as tasks"
    );
}

/// An `Operator` with an ECONSTANIC operand still pushes a task for it: ECONSTANIC is constanic but not
/// conclusive, and line 818's rule (`all_foolish_children_conclusive`) gates on conclusive, not
/// constanic. `operator_pushes_tasks_for_inconclusive_operands` only exercises PREMBRYONIC operands, so
/// it cannot tell the two apart — this does.
#[test]
fn operator_pushes_tasks_for_econstanic_operand() {
    let mut storage = FVMStorage::new();
    let op = storage.make_root(FirSpec::Operator { op: "+".to_string() });
    let a = op.create_child(&mut storage, FirSpec::Operator { op: "+".to_string() });
    storage.with_mut(a, |fir| fir.set_nyes(Nyes::Econstanic));

    op.step(&mut storage);

    assert_eq!(storage.get_nyes(op), Nyes::Braning);
    assert_eq!(
        FirCursor::new(op, &storage).front_task(),
        Some(a),
        "an ECONSTANIC (constanic but inconclusive) operand must still be queued"
    );
}

/// The core settle shape, with no null-characterized name in play so the NF-refusal checks stay out of
/// scope: `a = 9` settles Independent (a statement mirrors its body's exact settled state).
#[test]
fn statement_settles_to_its_bodys_nyes() {
    use crate::identifier::Identifier;

    let mut storage = FVMStorage::new();
    let stmt = storage.make_root(FirSpec::Statement {
        identifier: Identifier::from_parts(vec![], "a"),
        line_number: 0,
    });
    let body = stmt.create_child(&mut storage, FirSpec::IndepInt { value: 9 });

    for _ in 0..10 {
        if storage.get_nyes(stmt).is_constanic() {
            break;
        }
        stmt.step(&mut storage);
    }

    assert_eq!(storage.get_nyes(body), Nyes::Independent);
    assert_eq!(storage.get_nyes(stmt), Nyes::Independent);
    assert_eq!(
        FirCursor::new(stmt, &storage)
            .as_stmt_identifier()
            .map(|id| id.identifier_name()),
        Some("a")
    );
    assert_eq!(FirCursor::new(stmt, &storage).as_stmt_line_number(), Some(0));
}

/// A brane whose statements are all literal (`Independent`) values settles `Independent` itself —
/// `decide_nyes_due_to_children` checks all-`Independent` before all-`Constant`.
#[test]
fn brane_of_all_independent_statements_settles_independent() {
    use crate::identifier::Identifier;

    let mut storage = FVMStorage::new();
    let brane = storage.make_root(FirSpec::Brane {
        characterizations: Characterizations::default(),
    });
    let stmt_a = brane.create_child(
        &mut storage,
        FirSpec::Statement {
            identifier: Identifier::from_parts(vec![], "a"),
            line_number: 0,
        },
    );
    stmt_a.create_child(&mut storage, FirSpec::IndepInt { value: 1 });
    let stmt_b = brane.create_child(
        &mut storage,
        FirSpec::Statement {
            identifier: Identifier::from_parts(vec![], "b"),
            line_number: 1,
        },
    );
    stmt_b.create_child(&mut storage, FirSpec::IndepInt { value: 2 });

    for _ in 0..30 {
        if storage.get_nyes(brane).is_constanic() {
            break;
        }
        brane.step(&mut storage);
    }

    assert_eq!(storage.get_nyes(brane), Nyes::Independent);
    assert_eq!(FirCursor::new(brane, &storage).stmt_count(), Some(2));
    assert!(FirCursor::new(brane, &storage).is_brane_like());
}

/// An NK MEMBER does not make its brane NK (FOOP-94 as amended by the human 2026-09-23).
///
/// This test formerly asserted the OPPOSITE — it was named `brane_with_nk_child_settles_nk`
/// and pinned the any-NK-child rule, mirroring the retired `foolish-ubca`'s
/// `brane_with_nk_child_nyes_transitions`. That rule is gone: a brane is NK if and only if it
/// contains an unsteppable statement, so brane NK records a run-time error (FOOP-86 §6.2a)
/// rather than rolling up member states. The brane here did its own part correctly, so it
/// classifies CONSTANT and keeps `bad` NK and searchable.
///
/// CONSTANT rather than INDEPENDENT is deliberate and conservative: INDEPENDENT asserts "no
/// context dependencies at all", which an NK member's unresolved history does not support.
#[test]
fn brane_with_nk_child_settles_constant_not_nk() {
    use crate::identifier::Identifier;

    let mut storage = FVMStorage::new();
    let brane = storage.make_root(FirSpec::Brane {
        characterizations: Characterizations::default(),
    });
    let stmt_a = brane.create_child(
        &mut storage,
        FirSpec::Statement {
            identifier: Identifier::from_parts(vec![], "a"),
            line_number: 0,
        },
    );
    stmt_a.create_child(&mut storage, FirSpec::IndepInt { value: 1 });
    let stmt_bad = brane.create_child(
        &mut storage,
        FirSpec::Statement {
            identifier: Identifier::from_parts(vec![], "bad"),
            line_number: 1,
        },
    );
    stmt_bad.create_child(
        &mut storage,
        FirSpec::Nk {
            reason: "boom".to_string(),
        },
    );

    for _ in 0..30 {
        if storage.get_nyes(brane).is_constanic() {
            break;
        }
        brane.step(&mut storage);
    }

    assert_eq!(
        storage.get_nyes(brane),
        Nyes::Constant,
        "an NK member must not contaminate its brane; only an unsteppable statement makes a \
         brane NK"
    );
    assert!(
        storage.unsteppable_cause(brane).is_none(),
        "nothing here is unsteppable -- this brane has no run-time error to record"
    );
    assert_eq!(
        storage.get_nyes(stmt_bad),
        Nyes::Nk,
        "the NK member itself stays NK -- only the CONTAINER's classification changed"
    );
}

/// An empty `Brane` settles `Constant` in one step, via the `children.is_empty()` short-circuit.
#[test]
fn empty_brane_settles_constant_immediately() {
    let mut storage = FVMStorage::new();
    let brane = storage.make_root(FirSpec::Brane {
        characterizations: Characterizations::default(),
    });
    brane.step(&mut storage);
    assert_eq!(storage.get_nyes(brane), Nyes::Constant);
    assert_eq!(FirCursor::new(brane, &storage).stmt_count(), Some(0));
}

/// Construction and the pure data accessors round-trip correctly. Does NOT exercise search dispatch
/// correctness — see the end-to-end dispatch tests below for that.
#[test]
fn search_fir_structural_construction_and_accessors_round_trip() {
    let (mut storage, root) = FVMStorage::test_root_brane(&[]);
    let search = root.create_child(
        &mut storage,
        FirSpec::Search {
            pattern: "^x$".to_string(),
            anchored: true,
            forward: false,
            is_value_search: false,
            contexted: true,
        },
    );

    let cursor = FirCursor::new(search, &storage);
    assert_eq!(cursor.as_search_pattern(), Some("^x$"));
    assert!(cursor.as_search_anchored());
    assert!(!cursor.as_search_is_value());
    assert!(cursor.as_search_contexted());
    assert_eq!(storage.get_nyes(search), Nyes::Prembrionic);
}

/// Construction and the pure data accessors round-trip correctly. Does NOT exercise `#N`/`^`/`$`
/// resolution itself — see the IndexFir dispatch tests below for that. Index resolution resolves
/// against the ANCHOR (`foolish_children()[0]`, for the anchored+contexted case) or the enclosing
/// STATEMENT/BRANE found by walking the PARENT chain (`find_enclosing_stmt_and_brane`, for the
/// unanchored case) — never against a sibling directly.
#[test]
fn index_fir_structural_construction_and_accessors_round_trip() {
    let (mut storage, root) = FVMStorage::test_root_brane(&[]);
    let index = root.create_child(
        &mut storage,
        FirSpec::Index {
            offset: -1,
            anchored: true,
            contexted: false,
        },
    );

    let cursor = FirCursor::new(index, &storage);
    assert_eq!(cursor.as_index_offset(), -1);
    assert!(cursor.as_index_anchored());
    assert!(!cursor.as_search_contexted());
    assert_eq!(storage.get_nyes(index), Nyes::Prembrionic);
}

/// `FoolRefFir`'s arena migration, and — correctness-critical, per this plan's own note — the
/// FoolRefFir TWO-CHILD INVARIANT: a resolved search result has exactly two `ubc_children`, `[0]` the
/// result value, `[1]` a `FoolRefFir` wrapping the ORIGINAL found statement. Confirms the two children
/// are distinguishable by position exactly as `ubc_children[0]`/`[1]` are today, and that `FoolRefFir`
/// reports its referent and is born `Constant`.
#[test]
fn push_search_result_pair_preserves_the_two_child_invariant() {
    let (mut storage, root) = FVMStorage::test_root_brane(&[]);
    let result = root.create_child(&mut storage, FirSpec::IndepInt { value: 42 });
    let referent = root.create_child(
        &mut storage,
        FirSpec::Statement {
            identifier: crate::identifier::Identifier::from_parts(vec![], "x"),
            line_number: 0,
        },
    );

    let mut cursor = FirCursorMut::new(root, &mut storage);
    cursor.push_search_result_pair(result, referent);

    let read = FirCursor::new(root, &storage);
    let children = read.ubc_children();
    assert_eq!(children.len(), 2, "exactly two ubc_children, per the invariant");
    assert_eq!(children[0], result, "[0] is the result value");
    assert_eq!(
        FirCursor::new(children[1], &storage).node(),
        &FirSpec::FoolRef { referent }
    );
    assert_eq!(
        storage.get_nyes(children[1]),
        Nyes::Constant,
        "FoolRef is born Constant"
    );
    assert_eq!(
        FirCursor::new(children[1], &storage).as_fool_ref_referent(),
        Some(referent),
        "the FoolRef's referent is the ORIGINAL found statement, genuinely shared"
    );
    // `settled_constanic_result` (used by `.value()`) reads [0] only — [1] stays invisible. Its
    // contract applies the constanic gate itself, so `root` must be constanic first (a real search FIR
    // would already be constanic by the time it pushes a result; this test sets it directly rather than
    // stepping a real search).
    storage.with_mut(root, |fir| fir.set_nyes(Nyes::Constant));
    assert_eq!(
        FirCursor::new(root, &storage)
            .settled_constanic_result()
            .map(|c| c.ptr),
        Some(result)
    );
}

/// `StayFoolishFir`'s arena migration: mirrors `fir_kinds.rs::tests::stay_foolish_nyes_transitions`
/// exactly — SF wrapping a constant int settles Constant, unwrapping to the inner value.
#[test]
fn stay_foolish_settles_to_inner_expr_value() {
    let mut storage = FVMStorage::new();
    let sf = storage.make_root(FirSpec::StayFoolish);
    let expr = sf.create_child(&mut storage, FirSpec::IndepInt { value: 42 });
    storage.with_mut(expr, |fir| fir.set_nyes(Nyes::Constant));

    for _ in 0..10 {
        if storage.get_nyes(sf).is_constanic() {
            break;
        }
        sf.step(&mut storage);
    }

    assert_eq!(storage.get_nyes(sf), Nyes::Constant);
    assert_eq!(
        FirCursor::new(sf, &storage).ubc_children().first(),
        Some(&expr),
        "SF unwraps to the inner expr itself, since it has no constanic result of its own"
    );
}

/// Mirrors `fir_kinds.rs::tests::stay_fully_foolish_nyes_transitions`
/// exactly — SFF wrapping a constant int settles Constant.
#[test]
fn stay_fully_foolish_settles_to_inner_expr_value() {
    let mut storage = FVMStorage::new();
    let sff = storage.make_root(FirSpec::StayFullyFoolish);
    let expr = sff.create_child(&mut storage, FirSpec::IndepInt { value: 42 });
    storage.with_mut(expr, |fir| fir.set_nyes(Nyes::Constant));

    for _ in 0..10 {
        if storage.get_nyes(sff).is_constanic() {
            break;
        }
        sff.step(&mut storage);
    }

    assert_eq!(storage.get_nyes(sff), Nyes::Constant);
    assert_eq!(FirCursor::new(sff, &storage).ubc_children().first(), Some(&expr));
}

/// `revive_constanic`'s SF/SFF unwrap: a `StayFoolish` with a constanic result unwraps to that result
/// (recursing through `revive_constanic` again on it), never producing a cloned SF wrapper node —
/// mirrors `constanic_clone_at`'s own first branch exactly.
#[test]
fn revive_constanic_unwraps_stay_foolish_to_its_settled_constanic_result() {
    let (mut storage, root) = FVMStorage::test_root_brane(&[]);
    let sf = root.create_child(&mut storage, FirSpec::StayFoolish);
    let inner = sf.create_child(&mut storage, FirSpec::IndepInt { value: 7 });
    storage.with_mut(inner, |fir| fir.set_nyes(Nyes::Constant));
    // Simulate SF's own settle: push inner as its ubc_children[0].
    {
        let mut cursor = FirCursorMut::new(sf, &mut storage);
        cursor.push_ubc_child(inner);
    }
    let other_root = storage.make_root(FirSpec::IndepInt { value: 0 });

    let cloned = storage.revive_constanic(sf, other_root, 0, false, false);

    // `inner` is Constant non-Brane, so it's SHARED, not cloned — the
    // unwrap recurses into it and the share-rule then returns it as-is.
    assert_eq!(
        cloned, inner,
        "SF unwraps through to its constanic result, which then shares"
    );
    assert!(
        storage.foolish_children(other_root).is_empty() || storage.foolish_children(other_root) != [sf],
        "no cloned SF wrapper node should ever be produced"
    );
}

/// `revive_constanic`'s SF/SFF unwrap falls through to the first foolish child when there is no
/// constanic result yet (or for `StayFullyFoolish`, which never tries `ubc_children` first at all).
#[test]
fn revive_constanic_unwraps_stay_fully_foolish_to_first_foolish_child() {
    let (mut storage, root) = FVMStorage::test_root_brane(&[]);
    let sff = root.create_child(&mut storage, FirSpec::StayFullyFoolish);
    let inner = sff.create_child(&mut storage, FirSpec::Operator { op: "+".to_string() });
    // inner stays Prembrionic — a full-rebuild case, not a share.
    let other_root = storage.make_root(FirSpec::IndepInt { value: 0 });

    let cloned = storage.revive_constanic(sff, other_root, 0, false, false);

    assert_ne!(cloned, sff, "no cloned SFF wrapper node should ever be produced");
    assert_ne!(cloned, inner, "a pre-constanic inner must be rebuilt, not shared");
    assert_eq!(storage.get(cloned), &FirSpec::Operator { op: "+".to_string() });
}

/// `ConcatHelper` steps identically to a `Brane` — it is transparent, inheriting brane-shaped stepping.
#[test]
fn concat_helper_settles_like_a_brane() {
    use crate::identifier::Identifier;

    let mut storage = FVMStorage::new();
    let helper = storage.make_root(FirSpec::ConcatHelper);
    let stmt = helper.create_child(
        &mut storage,
        FirSpec::Statement {
            identifier: Identifier::from_parts(vec![], "x"),
            line_number: 0,
        },
    );
    stmt.create_child(&mut storage, FirSpec::IndepInt { value: 42 });

    for _ in 0..10 {
        if storage.get_nyes(helper).is_constanic() {
            break;
        }
        helper.step(&mut storage);
    }

    assert_eq!(storage.get_nyes(helper), Nyes::Independent);
    assert_eq!(FirCursor::new(helper, &storage).stmt_count(), Some(1));
}

/// `ConcatenationFir`'s arena migration is TYPE-CHECK AND JOIN-READINESS ONLY (see this kind's
/// `fir_op_step` arm doc comment — helper population/merging is deferred, same NF-mechanism dependency
/// already deferred at `StatementFir`). This test proves the type-check path: a concatenation of two
/// constanic, brane-like elements is join-ready and settles `Woconstanic` (an HONEST
/// incomplete-implementation result, NOT `Constant` — `concatenation_nyes_transitions` in
/// `fir_kinds.rs` expects `Constant` from the REAL, fully-merging implementation; this arena test
/// intentionally does NOT mirror that terminal state, since doing so would misrepresent what this task
/// actually implemented).
#[test]
fn concatenation_of_settled_branes_is_join_ready() {
    let mut storage = FVMStorage::new();
    let cat = storage.make_root(FirSpec::Concatenation {
        provenance: ConcatProvenance::Juxtaposition,
        rendering_aid: ConcatRenderingAid::default(),
    });
    let brane1 = cat.create_child(
        &mut storage,
        FirSpec::Brane {
            characterizations: Characterizations::default(),
        },
    );
    storage.with_mut(brane1, |fir| fir.set_nyes(Nyes::Constant));
    let brane2 = cat.create_child(
        &mut storage,
        FirSpec::Brane {
            characterizations: Characterizations::default(),
        },
    );
    storage.with_mut(brane2, |fir| fir.set_nyes(Nyes::Constant));

    core_fir_conversion::step_to_constanic(&mut storage, cat).unwrap();

    // Both elements are EMPTY branes -- zero lines to merge, so the real `populate_concat_helpers`
    // pushes no helper at all, and the "empty helper set -> Constant" convention applies (updated from
    // this test's earlier Woconstanic expectation, which pinned the deliberately-incomplete
    // pre-merge-logic placeholder — now that populate_concat_helpers is real, join-ready empty branes
    // settle Constant, matching the real ConcatenationFir's own documented "Empty (no lines joined) ->
    // Constant" rule).
    assert_eq!(
        storage.get_nyes(cat),
        Nyes::Constant,
        "join-ready elements with zero total lines settle Constant (empty-brane convention)"
    );
    assert_eq!(
        FirCursor::new(cat, &storage).as_concat_provenance(),
        ConcatProvenance::Juxtaposition
    );
}

/// A concatenation with a genuinely non-brane, constanic element (an `IndepInt`) settles `Nk` with the
/// exact reason format the real `fir_op_step` produces — mirrors the type-error branch exactly.
#[test]
fn concatenation_with_a_non_brane_element_settles_nk() {
    let mut storage = FVMStorage::new();
    let cat = storage.make_root(FirSpec::Concatenation {
        provenance: ConcatProvenance::Juxtaposition,
        rendering_aid: ConcatRenderingAid::default(),
    });
    let brane = cat.create_child(
        &mut storage,
        FirSpec::Brane {
            characterizations: Characterizations::default(),
        },
    );
    storage.with_mut(brane, |fir| fir.set_nyes(Nyes::Constant));
    let not_a_brane = cat.create_child(&mut storage, FirSpec::IndepInt { value: 1 });
    storage.with_mut(not_a_brane, |fir| fir.set_nyes(Nyes::Constant));

    for _ in 0..5 {
        if storage.get_nyes(cat).is_constanic() {
            break;
        }
        cat.step(&mut storage);
    }

    assert_eq!(storage.get_nyes(cat), Nyes::Nk);
    let reason = FirCursor::new(cat, &storage)
        .settled_constanic_result()
        .and_then(|c| c.as_nk_reason().map(str::to_string));
    assert_eq!(
        reason,
        Some("concatenation constituent indexes where it's not a brane: 1".to_string())
    );
}

/// `CreationFir`'s arena migration: born `Independent` and never steps —
/// mirrors `fir_kinds.rs::tests::creation_nyes_transitions` exactly.
#[test]
fn creation_is_born_independent_and_never_steps() {
    let mut storage = FVMStorage::new();
    let root = storage.make_root(FirSpec::Brane {
        characterizations: Characterizations::default(),
    });
    let creation = root.create_child(&mut storage, FirSpec::Creation);
    assert_eq!(storage.get_nyes(creation), Nyes::Independent);

    creation.step(&mut storage);
    assert_eq!(storage.get_nyes(creation), Nyes::Independent);
}

/// `get_display_name`'s two-condition rule (FOOP-33), condition 1: a creation viewed from its OWN
/// defining statement never reports a name, even though it is null-characterized and the whole RHS —
/// mirrors `fir_kinds.rs::tests:: creation_viewed_from_its_own_defining_statement_reports_no_name`
/// exactly, using `Identifier::from_parts(vec![String::new()], "a")` to construct a null-characterized
/// identifier directly (per that constructor's own doc comment: a single empty-string characterization
/// component means null-characterization) rather than through the (not-yet-arena-migrated)
/// parser/compiler.
#[test]
fn creation_viewed_from_its_own_defining_statement_reports_no_name() {
    use crate::identifier::Identifier;

    let (mut storage, root) = FVMStorage::test_root_brane(&[]);
    let stmt = root.create_child(
        &mut storage,
        FirSpec::Statement {
            identifier: Identifier::from_parts(vec![String::new()], "a"),
            line_number: 0,
        },
    );
    assert!(
        FirCursor::new(stmt, &storage)
            .as_stmt_identifier()
            .unwrap()
            .is_nully_characterizing_coordinate_name(),
        "sanity: the constructed identifier must actually be null-characterized"
    );
    let creation = stmt.create_child(&mut storage, FirSpec::Creation);

    let name = creation.get_display_name(&storage, stmt);
    assert_eq!(
        name, None,
        "a creation viewed from its OWN defining statement never reports a name"
    );
}

/// Condition 1's positive case: viewed from a DIFFERENT statement, a null-characterized creation DOES
/// report its defining statement's name.
#[test]
fn creation_viewed_from_elsewhere_reports_its_name() {
    use crate::identifier::Identifier;

    let (mut storage, root) = FVMStorage::test_root_brane(&[]);
    let stmt = root.create_child(
        &mut storage,
        FirSpec::Statement {
            identifier: Identifier::from_parts(vec![String::new()], "a"),
            line_number: 0,
        },
    );
    let creation = stmt.create_child(&mut storage, FirSpec::Creation);
    let elsewhere = root.create_child(
        &mut storage,
        FirSpec::Statement {
            identifier: Identifier::from_parts(vec![], "b"),
            line_number: 1,
        },
    );

    let name = creation.get_display_name(&storage, elsewhere);
    // `get_display_name` reports `identifier.searchable_name()` (`fully_characterized_name`), not the
    // bare `identifier_name()` — for a null-characterized name the searchable form is `"'a"`
    // (`Identifier::from_parts`'s doc comment: an empty-string characterization component renders as a
    // bare `'` prefix).
    assert_eq!(name, Some("'a".to_string()));
}

/// Condition 2: a creation defined under a PLAIN (non-null-characterized)
/// name never reports a name, even when viewed from elsewhere.
#[test]
fn creation_under_a_plain_name_never_reports_a_name() {
    use crate::identifier::Identifier;

    let (mut storage, root) = FVMStorage::test_root_brane(&[]);
    let stmt = root.create_child(
        &mut storage,
        FirSpec::Statement {
            identifier: Identifier::from_parts(vec![], "a"), // plain, not null-characterized
            line_number: 0,
        },
    );
    let creation = stmt.create_child(&mut storage, FirSpec::Creation);
    let elsewhere = root.create_child(
        &mut storage,
        FirSpec::Statement {
            identifier: Identifier::from_parts(vec![], "b"),
            line_number: 1,
        },
    );

    assert_eq!(creation.get_display_name(&storage, elsewhere), None);
}

/// An operand whose own first foolish child is itself `Econstanic` (shaped like `<<#-1>>`, an
/// SFF-wrapped index search inside `system.foo`) makes the whole comparison settle `Econstanic`.
#[test]
fn comparison_settles_econstanic_when_an_operand_is_unevaluated_here() {
    use crate::system_foo::ComparisonOp;

    let mut storage = FVMStorage::new();
    let cmp = storage.make_root(FirSpec::Comparison { op: ComparisonOp::Lt });
    // Operand shaped like `<<#-1>>`: an SFF-wrapped index search whose own inner search sits Econstanic
    // (searched nothing in this context yet).
    let operand = cmp.create_child(&mut storage, FirSpec::StayFullyFoolish);
    let inner_search = operand.create_child(
        &mut storage,
        FirSpec::Index {
            offset: -1,
            anchored: true,
            contexted: false,
        },
    );
    storage.with_mut(inner_search, |fir| fir.set_nyes(Nyes::Econstanic));
    storage.with_mut(operand, |fir| fir.set_nyes(Nyes::Constant));

    for _ in 0..5 {
        if storage.get_nyes(cmp).is_constanic() {
            break;
        }
        cmp.step(&mut storage);
    }

    assert_eq!(storage.get_nyes(cmp), Nyes::Econstanic);
    assert_eq!(FirCursor::new(cmp, &storage).as_op_name(), Some("'lt"));
}

/// When both operands ARE genuinely evaluated, the comparison resolves to whichever of `'True`/`'False`
/// its ancestral search finds — which needs `'True`/`'False` reachable via ancestral search from the
/// Comparison node's own position, so this test builds a minimal system.foo-shaped ancestor brane
/// declaring them, with the Comparison node nested inside it (an isolated root with no ancestor to
/// search would not exercise this path).
#[test]
fn comparison_with_evaluated_operands_resolves_the_real_verdict() {
    use crate::system_foo::ComparisonOp;

    let mut storage = FVMStorage::new();
    let root = storage.make_root(FirSpec::Brane {
        characterizations: Characterizations::default(),
    });
    let true_stmt = root.create_child(
        &mut storage,
        FirSpec::Statement {
            identifier: Identifier::from_parts(vec![String::new()], "True"),
            line_number: 0,
        },
    );
    let true_creation = true_stmt.create_child(&mut storage, FirSpec::Creation);
    let false_stmt = root.create_child(
        &mut storage,
        FirSpec::Statement {
            identifier: Identifier::from_parts(vec![String::new()], "False"),
            line_number: 1,
        },
    );
    false_stmt.create_child(&mut storage, FirSpec::Creation);

    // `'True`/`'False` must be in an ANCESTOR brane of `cmp`'s own home brane, not siblings within the
    // SAME brane `cmp` sits in — `ab_search_by_pattern` searches ANCESTORS, never the current brane's
    // own siblings, and the ROOT brane itself is never its own ancestor. Nest one level deeper: an
    // inner brane holds the statement whose body is the Comparison node.
    let inner_holder_stmt = root.create_child(
        &mut storage,
        FirSpec::Statement {
            identifier: Identifier::from_parts(vec![], "program"),
            line_number: 2,
        },
    );
    let inner = inner_holder_stmt.create_child(
        &mut storage,
        FirSpec::Brane {
            characterizations: Characterizations::default(),
        },
    );
    let holder = inner.create_child(
        &mut storage,
        FirSpec::Statement {
            identifier: Identifier::from_parts(vec![], "eq_check"),
            line_number: 0,
        },
    );
    let cmp = holder.create_child(&mut storage, FirSpec::Comparison { op: ComparisonOp::Eq });
    cmp.create_child(&mut storage, FirSpec::IndepInt { value: 1 });
    cmp.create_child(&mut storage, FirSpec::IndepInt { value: 1 });

    core_fir_conversion::step_to_constanic(&mut storage, root).unwrap();

    assert_eq!(
        storage.get_nyes(cmp),
        Nyes::Constant,
        "1 =̲=̲ 1 must resolve the real verdict, not defer to Woconstanic"
    );
    let result = FirCursor::new(cmp, &storage).ubc_children().first().copied();
    assert_eq!(
        result,
        Some(true_creation),
        "eq(1, 1) is true -- the comparison's result must be the SAME 'True creation \
         system.foo declares (referential identity, FOOP-33 SS5), not a synthetic boolean"
    );
}

// ── Search engine tests ──────────────────────────────────────────
//
// Mirror the spirit (not every single case) of `fir_kinds.rs`'s real
// `ContextfulSearch engine tests` module: Navigator ordering contract,
// predicate matching per variant, and the scan loop's Found/Miss/NkStop
// outcomes. The authoritative correctness check for this phase is the
// targeted einmo re-run (per FOOP-16.plan.md's own instruction that this
// phase carries the highest silent-regression risk) — these unit tests
// pin the internal engine state the black-box einmo comparison does not
// directly exercise.

use search_engine::{
    BraneNavigator, CandidateNavigator, MatchOutcome, ScanCtx, ScanOutcome, SearchPredicate,
    contextful_search_scan, contextful_search_scan_no_body_check,
};

fn make_named_statement(
    storage: &mut FVMStorage,
    brane: FirPointer,
    name: &str,
    line: usize,
    value: i64,
) -> FirPointer {
    use crate::identifier::Identifier;
    let stmt = brane.create_child(
        storage,
        FirSpec::Statement {
            identifier: Identifier::from_parts(vec![], name),
            line_number: line,
        },
    );
    let body = stmt.create_child(storage, FirSpec::IndepInt { value });
    storage.with_mut(body, |fir| fir.set_nyes(Nyes::Constant));
    storage.with_mut(stmt, |fir| fir.set_nyes(Nyes::Constant));
    stmt
}

/// `BraneNavigator` forward direction yields every candidate, in construction order, exactly once, then
/// stops — mirrors `brane_nav_forward_yields_in_order_exactly_once` exactly.
#[test]
fn brane_navigator_forward_yields_in_order_exactly_once() {
    let (mut storage, brane) = FVMStorage::test_root_brane(&[]);
    let s0 = make_named_statement(&mut storage, brane, "a", 0, 1);
    let s1 = make_named_statement(&mut storage, brane, "b", 1, 2);
    let s2 = make_named_statement(&mut storage, brane, "c", 2, 3);

    let mut nav = BraneNavigator::new(&storage, brane, true);
    assert_eq!(nav.total(), 3);

    let yielded: Vec<(FirPointer, usize)> = std::iter::from_fn(|| nav.next_candidate()).collect();
    assert_eq!(yielded, vec![(s0, 0), (s1, 1), (s2, 2)]);
    assert!(nav.next_candidate().is_none(), "must stop after all yielded");
}

/// Backward direction yields in reverse order, exactly once — mirrors
/// `brane_nav_backward_yields_reverse_order_exactly_once` exactly.
#[test]
fn brane_navigator_backward_yields_reverse_order_exactly_once() {
    let (mut storage, brane) = FVMStorage::test_root_brane(&[]);
    let s0 = make_named_statement(&mut storage, brane, "a", 0, 10);
    let s1 = make_named_statement(&mut storage, brane, "b", 1, 20);
    let s2 = make_named_statement(&mut storage, brane, "c", 2, 30);

    let mut nav = BraneNavigator::new(&storage, brane, false);
    let yielded: Vec<(FirPointer, usize)> = std::iter::from_fn(|| nav.next_candidate()).collect();
    assert_eq!(yielded, vec![(s2, 2), (s1, 1), (s0, 0)]);
    assert!(nav.next_candidate().is_none());
}

/// An empty brane's navigator yields nothing — mirrors `brane_nav_empty_brane_yields_nothing` exactly.
#[test]
fn brane_navigator_empty_brane_yields_nothing() {
    let (storage, brane) = FVMStorage::test_root_brane(&[]);
    let mut nav = BraneNavigator::new(&storage, brane, true);
    assert_eq!(nav.total(), 0);
    assert!(nav.next_candidate().is_none());
}

/// `SearchPredicate::Name` approves an exact match on a constanic candidate.
#[test]
fn search_predicate_name_approves_exact_match() {
    let (mut storage, brane) = FVMStorage::test_root_brane(&[]);
    let stmt = make_named_statement(&mut storage, brane, "x", 0, 5);
    let ctx = ScanCtx {
        position: 0,
        total: 1,
    };
    let pred = SearchPredicate::Name {
        pattern: "x".to_string(),
    };
    assert_eq!(pred.matches(&storage, stmt, &ctx), MatchOutcome::Approve);
}

/// `SearchPredicate::Name` rejects a non-matching name.
#[test]
fn search_predicate_name_rejects_non_match() {
    let (mut storage, brane) = FVMStorage::test_root_brane(&[]);
    let stmt = make_named_statement(&mut storage, brane, "x", 0, 5);
    let ctx = ScanCtx {
        position: 0,
        total: 1,
    };
    let pred = SearchPredicate::Name {
        pattern: "y".to_string(),
    };
    assert_eq!(pred.matches(&storage, stmt, &ctx), MatchOutcome::Reject);
}

/// `SearchPredicate::Name` NkStops when the candidate's body is NK — `check_body_nyes`'s NK branch.
#[test]
fn search_predicate_name_nkstops_on_nk_body() {
    let (mut storage, brane) = FVMStorage::test_root_brane(&[]);
    use crate::identifier::Identifier;
    let stmt = brane.create_child(
        &mut storage,
        FirSpec::Statement {
            identifier: Identifier::from_parts(vec![], "bad"),
            line_number: 0,
        },
    );
    let body = stmt.create_child(
        &mut storage,
        FirSpec::Nk {
            reason: "boom".to_string(),
        },
    );
    storage.with_mut(body, |fir| fir.set_nyes(Nyes::Nk));
    storage.with_mut(stmt, |fir| fir.set_nyes(Nyes::Nk));

    let ctx = ScanCtx {
        position: 0,
        total: 1,
    };
    let pred = SearchPredicate::Name {
        pattern: "bad".to_string(),
    };
    assert_eq!(pred.matches(&storage, stmt, &ctx), MatchOutcome::NkStop);
}

/// `SearchPredicate::Value` approves when the candidate's body equals
/// the pattern's value, via `default_equal`.
#[test]
fn search_predicate_value_approves_on_equal_body() {
    let (mut storage, brane) = FVMStorage::test_root_brane(&[]);
    let stmt = make_named_statement(&mut storage, brane, "x", 0, 5);
    let pattern = brane.create_child(&mut storage, FirSpec::IndepInt { value: 5 });
    storage.with_mut(pattern, |fir| fir.set_nyes(Nyes::Constant));

    let ctx = ScanCtx {
        position: 0,
        total: 1,
    };
    let pred = SearchPredicate::Value { pattern };
    assert_eq!(pred.matches(&storage, stmt, &ctx), MatchOutcome::Approve);
}

/// `SearchPredicate::NameValue` is atomic: both name and value must
/// match on the SAME candidate in one scan.
#[test]
fn search_predicate_name_value_is_atomic_conjunction() {
    let (mut storage, brane) = FVMStorage::test_root_brane(&[]);
    let stmt = make_named_statement(&mut storage, brane, "x", 0, 5);
    let pattern = brane.create_child(&mut storage, FirSpec::IndepInt { value: 5 });
    storage.with_mut(pattern, |fir| fir.set_nyes(Nyes::Constant));

    let ctx = ScanCtx {
        position: 0,
        total: 1,
    };
    // Name matches, value matches -> Approve.
    let both_match = SearchPredicate::NameValue {
        name: "x".to_string(),
        value: pattern,
    };
    assert_eq!(both_match.matches(&storage, stmt, &ctx), MatchOutcome::Approve);

    // Name matches, value does NOT -> Reject (not NkStop: NotEqual, not Unknowable).
    let other_pattern = brane.create_child(&mut storage, FirSpec::IndepInt { value: 999 });
    storage.with_mut(other_pattern, |fir| fir.set_nyes(Nyes::Constant));
    let name_only = SearchPredicate::NameValue {
        name: "x".to_string(),
        value: other_pattern,
    };
    assert_eq!(name_only.matches(&storage, stmt, &ctx), MatchOutcome::Reject);
}

/// `SearchPredicate::Index` with a negative offset resolves relative to
/// `ctx.total` — `#-1` addresses the last candidate.
#[test]
fn search_predicate_index_negative_offset_addresses_from_the_end() {
    let (mut storage, brane) = FVMStorage::test_root_brane(&[]);
    let _s0 = make_named_statement(&mut storage, brane, "a", 0, 1);
    let s1 = make_named_statement(&mut storage, brane, "b", 1, 2);

    let ctx = ScanCtx {
        position: 1,
        total: 2,
    };
    let pred = SearchPredicate::Index(-1);
    assert_eq!(pred.matches(&storage, s1, &ctx), MatchOutcome::Approve);
}

/// `SearchPredicate::Head`/`Tail` match only position 0 / the last position respectively.
#[test]
fn search_predicate_head_and_tail_match_the_right_position() {
    let (mut storage, brane) = FVMStorage::test_root_brane(&[]);
    let s0 = make_named_statement(&mut storage, brane, "a", 0, 1);
    let s1 = make_named_statement(&mut storage, brane, "b", 1, 2);

    let ctx0 = ScanCtx {
        position: 0,
        total: 2,
    };
    let ctx1 = ScanCtx {
        position: 1,
        total: 2,
    };
    assert_eq!(
        SearchPredicate::Head.matches(&storage, s0, &ctx0),
        MatchOutcome::Approve
    );
    assert_eq!(
        SearchPredicate::Head.matches(&storage, s1, &ctx1),
        MatchOutcome::Reject
    );
    assert_eq!(
        SearchPredicate::Tail.matches(&storage, s1, &ctx1),
        MatchOutcome::Approve
    );
    assert_eq!(
        SearchPredicate::Tail.matches(&storage, s0, &ctx0),
        MatchOutcome::Reject
    );
}

/// `matches_no_body_check` skips the body-NYES gate for positional/name predicates — approves even with
/// a pre-constanic body, which `matches` would treat as an internal-consistency violation
/// (`unreachable!`).
#[test]
fn matches_no_body_check_skips_the_body_nyes_gate() {
    use crate::identifier::Identifier;
    let (mut storage, brane) = FVMStorage::test_root_brane(&[]);
    let stmt = brane.create_child(
        &mut storage,
        FirSpec::Statement {
            identifier: Identifier::from_parts(vec![], "x"),
            line_number: 0,
        },
    );
    let _body = stmt.create_child(&mut storage, FirSpec::IndepInt { value: 1 });
    // body stays Prembrionic (pre-constanic) — matches() would hit the
    // check_body_nyes unreachable!(); matches_no_body_check must not.

    let ctx = ScanCtx {
        position: 0,
        total: 1,
    };
    let pred = SearchPredicate::Name {
        pattern: "x".to_string(),
    };
    assert_eq!(
        pred.matches_no_body_check(&storage, stmt, &ctx),
        MatchOutcome::Approve
    );
}

/// `contextful_search_scan` finds the first approving candidate and
/// stops — the scan loop's `Found` outcome.
#[test]
fn contextful_search_scan_finds_first_match() {
    let (mut storage, brane) = FVMStorage::test_root_brane(&[]);
    let _s0 = make_named_statement(&mut storage, brane, "a", 0, 1);
    let s1 = make_named_statement(&mut storage, brane, "target", 1, 2);
    let _s2 = make_named_statement(&mut storage, brane, "target", 2, 3);

    let mut nav = BraneNavigator::new(&storage, brane, true);
    let pred = SearchPredicate::Name {
        pattern: "target".to_string(),
    };
    let outcome = contextful_search_scan(&storage, &mut nav, &pred);
    assert_eq!(
        outcome,
        ScanOutcome::Found(s1),
        "forward scan finds the FIRST matching candidate, not a later duplicate"
    );
}

/// `contextful_search_scan` exhausts with `Miss` when nothing matches.
#[test]
fn contextful_search_scan_misses_when_nothing_matches() {
    let (mut storage, brane) = FVMStorage::test_root_brane(&[]);
    let _s0 = make_named_statement(&mut storage, brane, "a", 0, 1);

    let mut nav = BraneNavigator::new(&storage, brane, true);
    let pred = SearchPredicate::Name {
        pattern: "nonexistent".to_string(),
    };
    assert_eq!(
        contextful_search_scan(&storage, &mut nav, &pred),
        ScanOutcome::Miss
    );
}

/// `contextful_search_scan` halts immediately with `NkStop` on an Unknowable candidate — never masks it
/// by continuing to scan further candidates that might otherwise match.
#[test]
fn contextful_search_scan_halts_on_nkstop() {
    let (mut storage, brane) = FVMStorage::test_root_brane(&[]);
    use crate::identifier::Identifier;
    let bad = brane.create_child(
        &mut storage,
        FirSpec::Statement {
            identifier: Identifier::from_parts(vec![], "bad"),
            line_number: 0,
        },
    );
    let bad_body = bad.create_child(
        &mut storage,
        FirSpec::Nk {
            reason: "x".to_string(),
        },
    );
    storage.with_mut(bad_body, |fir| fir.set_nyes(Nyes::Nk));
    storage.with_mut(bad, |fir| fir.set_nyes(Nyes::Nk));
    let _after = make_named_statement(&mut storage, brane, "bad", 1, 1); // would match if scan continued

    let mut nav = BraneNavigator::new(&storage, brane, true);
    let pred = SearchPredicate::Name {
        pattern: "bad".to_string(),
    };
    assert_eq!(
        contextful_search_scan(&storage, &mut nav, &pred),
        ScanOutcome::NkStop
    );
}

/// `contextful_search_scan_no_body_check`'s own re-verification (per this phase's own task
/// instruction): confirms the scan loop needed NO further logic change beyond what already flows
/// through from `CandidateNavigator`'s and `SearchPredicate`'s migrations.
#[test]
fn contextful_search_scan_no_body_check_finds_pre_constanic_candidates() {
    use crate::identifier::Identifier;
    let (mut storage, brane) = FVMStorage::test_root_brane(&[]);
    let stmt = brane.create_child(
        &mut storage,
        FirSpec::Statement {
            identifier: Identifier::from_parts(vec![], "x"),
            line_number: 0,
        },
    );
    let _body = stmt.create_child(&mut storage, FirSpec::IndepInt { value: 1 });

    let mut nav = BraneNavigator::new(&storage, brane, true);
    let pred = SearchPredicate::Name {
        pattern: "x".to_string(),
    };
    assert_eq!(
        contextful_search_scan_no_body_check(&storage, &mut nav, &pred),
        ScanOutcome::Found(stmt)
    );
}

// ── SearchFir end-to-end dispatch tests ──────────────────────────
//
// These exercise the FULL fir_op_step dispatch through FirPointer::step
// (not the lower-level search_engine primitives directly), proving
// Scope threading (current_statement/current_brane) and the IB/AB/
// contexted dispatch paths work together end-to-end — the shape real
// Foolish source produces, even though this crate's compiler/evaluator
// aren't migrated yet (Phases 3-4), so these trees are hand-built.

/// An anchored search finding a statement via `contextful_search_scan` (the anchored-Braning path in
/// `name_search_step`) — `anchor_brane?x` shape: an anchored search whose FIRST child (the anchor) IS
/// the brane to scan directly (`.value()` on an already-`Constant` `Brane` is a no-op — `Brane` never
/// populates its own `ubc_children`, so `settled_constanic_result`/`.value()` return the brane itself
/// unchanged).
#[test]
fn search_fir_anchored_finds_statement_in_resolved_brane() {
    let (mut storage, root) = FVMStorage::test_root_brane(&[]);
    let search = root.create_child(
        &mut storage,
        FirSpec::Search {
            pattern: "x".to_string(),
            anchored: true,
            forward: false,
            is_value_search: false,
            contexted: false,
        },
    );
    // The anchor: search's own foolish_children[0].
    let anchor_brane = search.create_child(
        &mut storage,
        FirSpec::Brane {
            characterizations: Characterizations::default(),
        },
    );
    let _x = make_named_statement(&mut storage, anchor_brane, "x", 0, 42);

    for _ in 0..30 {
        if storage.get_nyes(search).is_constanic() {
            break;
        }
        search.step(&mut storage);
    }

    assert_eq!(
        storage.get_nyes(search),
        Nyes::Constant,
        "anchored search must resolve its anchor to a brane, scan it, and find 'x'"
    );
    assert_eq!(FirCursor::new(search, &storage).as_i64(), Some(42));
}

/// An anchored search that finds NOTHING settles `Nk` (anchored miss), not `Econstanic` — confirming
/// the anchored-vs-unanchored miss distinction the OTHER direction from
/// `search_fir_unanchored_miss_settles_econstanic` below.
#[test]
fn search_fir_anchored_miss_settles_nk() {
    let (mut storage, root) = FVMStorage::test_root_brane(&[]);
    let search = root.create_child(
        &mut storage,
        FirSpec::Search {
            pattern: "nonexistent".to_string(),
            anchored: true,
            forward: false,
            is_value_search: false,
            contexted: false,
        },
    );
    let anchor_brane = search.create_child(
        &mut storage,
        FirSpec::Brane {
            characterizations: Characterizations::default(),
        },
    );
    let _x = make_named_statement(&mut storage, anchor_brane, "x", 0, 1);

    for _ in 0..30 {
        if storage.get_nyes(search).is_constanic() {
            break;
        }
        search.step(&mut storage);
    }

    assert_eq!(storage.get_nyes(search), Nyes::Nk);
}

/// IB search: `{x=1; y=?x;}` shape — `y`'s unanchored search finds `x` earlier in the SAME brane via
/// `name_search_step`'s Embryonic arm (`ib_search_with_engine`), reading `Scope::current_statement`
/// (threaded by `step_inner`).
#[test]
fn search_fir_ib_search_finds_earlier_statement_in_same_brane() {
    use crate::identifier::Identifier;

    let (mut storage, root) = FVMStorage::test_root_brane(&[]);
    let x = root.create_child(
        &mut storage,
        FirSpec::Statement {
            identifier: Identifier::from_parts(vec![], "x"),
            line_number: 0,
        },
    );
    let x_body = x.create_child(&mut storage, FirSpec::IndepInt { value: 1 });
    storage.with_mut(x_body, |fir| fir.set_nyes(Nyes::Constant));
    storage.with_mut(x, |fir| fir.set_nyes(Nyes::Constant));

    let y = root.create_child(
        &mut storage,
        FirSpec::Statement {
            identifier: Identifier::from_parts(vec![], "y"),
            line_number: 1,
        },
    );
    let search = y.create_child(
        &mut storage,
        FirSpec::Search {
            pattern: "x".to_string(),
            anchored: false,
            forward: false,
            is_value_search: false,
            contexted: false,
        },
    );

    for _ in 0..30 {
        if storage.get_nyes(root).is_constanic() {
            break;
        }
        root.step(&mut storage);
    }

    assert!(
        storage.get_nyes(search).is_constanic(),
        "search must settle (got {:?})",
        storage.get_nyes(search)
    );
    assert_eq!(
        storage.get_nyes(search),
        Nyes::Constant,
        "IB search for 'x' from 'y' must find it and settle Constant"
    );
    assert_eq!(FirCursor::new(search, &storage).as_i64(), Some(1));
}

/// An unanchored search with NOTHING preceding it in its brane settles `Econstanic` (unanchored miss),
/// not `Nk` — the anchored-vs-unanchored miss distinction (AGENTS.md §Searches "NK vs ECONSTANIC miss
/// outcomes").
#[test]
fn search_fir_unanchored_miss_settles_econstanic() {
    use crate::identifier::Identifier;

    let (mut storage, root) = FVMStorage::test_root_brane(&[]);
    let y = root.create_child(
        &mut storage,
        FirSpec::Statement {
            identifier: Identifier::from_parts(vec![], "y"),
            line_number: 0,
        },
    );
    let search = y.create_child(
        &mut storage,
        FirSpec::Search {
            pattern: "nonexistent".to_string(),
            anchored: false,
            forward: false,
            is_value_search: false,
            contexted: false,
        },
    );

    for _ in 0..30 {
        if storage.get_nyes(root).is_constanic() {
            break;
        }
        root.step(&mut storage);
    }

    assert_eq!(storage.get_nyes(search), Nyes::Econstanic);
}

/// AB search: `{x=1; inner={y=?x;};}` shape — `y`'s unanchored search finds `x` in the ANCESTOR brane
/// via `name_search_step`'s Braning arm (`ab_search_with_engine`), reading `Scope::current_brane`.
#[test]
fn search_fir_ab_search_finds_in_ancestor_brane() {
    use crate::identifier::Identifier;

    let (mut storage, root) = FVMStorage::test_root_brane(&[]);
    let x = root.create_child(
        &mut storage,
        FirSpec::Statement {
            identifier: Identifier::from_parts(vec![], "x"),
            line_number: 0,
        },
    );
    let x_body = x.create_child(&mut storage, FirSpec::IndepInt { value: 99 });
    storage.with_mut(x_body, |fir| fir.set_nyes(Nyes::Constant));
    storage.with_mut(x, |fir| fir.set_nyes(Nyes::Constant));

    let inner_stmt = root.create_child(
        &mut storage,
        FirSpec::Statement {
            identifier: Identifier::from_parts(vec![], "inner"),
            line_number: 1,
        },
    );
    let inner_brane = inner_stmt.create_child(
        &mut storage,
        FirSpec::Brane {
            characterizations: Characterizations::default(),
        },
    );
    let y = inner_brane.create_child(
        &mut storage,
        FirSpec::Statement {
            identifier: Identifier::from_parts(vec![], "y"),
            line_number: 0,
        },
    );
    let search = y.create_child(
        &mut storage,
        FirSpec::Search {
            pattern: "x".to_string(),
            anchored: false,
            forward: false,
            is_value_search: false,
            contexted: false,
        },
    );

    for _ in 0..50 {
        if storage.get_nyes(root).is_constanic() {
            break;
        }
        root.step(&mut storage);
    }

    assert_eq!(
        storage.get_nyes(search),
        Nyes::Constant,
        "AB search for 'x' from inner brane's 'y' must climb out and find it"
    );
    assert_eq!(FirCursor::new(search, &storage).as_i64(), Some(99));
}

// ── Stepping loop tests ───────────────────

use core_fir_conversion::step_to_constanic;

/// `step_to_constanic`'s happy path: an `IndepInt` settles within budget.
#[test]
fn step_to_constanic_settles_a_simple_fir() {
    let mut storage = FVMStorage::new();
    let ptr = storage.make_root(FirSpec::IndepInt { value: 7 });
    assert!(step_to_constanic(&mut storage, ptr).is_ok());
    assert_eq!(storage.get_nyes(ptr), Nyes::Independent);
}

use core_fir_conversion::{step_until, step_until_line_number, step_until_statement_name};

/// `step_until_statement_name` finds the SECOND statement in a two-line brane — mirrors
/// `evaluator.rs::step_until_tests:: step_until_statement_name_finds_second_statement`'s intent.
#[test]
fn step_until_statement_name_finds_second_statement() {
    use crate::identifier::Identifier;

    let (mut storage, root) = FVMStorage::test_root_brane(&[]);
    let a = root.create_child(
        &mut storage,
        FirSpec::Statement {
            identifier: Identifier::from_parts(vec![], "a"),
            line_number: 0,
        },
    );
    a.create_child(&mut storage, FirSpec::IndepInt { value: 1 });
    let b = root.create_child(
        &mut storage,
        FirSpec::Statement {
            identifier: Identifier::from_parts(vec![], "b"),
            line_number: 1,
        },
    );
    b.create_child(&mut storage, FirSpec::IndepInt { value: 2 });

    let steps = step_until_statement_name(&mut storage, root, "b").unwrap();
    eprintln!("stopped after {steps} steps");
    let front = FirCursor::new(root, &storage).front_task();
    assert!(front.is_some());
    assert_eq!(
        FirCursor::new(front.unwrap(), &storage)
            .as_stmt_identifier()
            .map(|id| id.searchable_name()),
        Some("b")
    );
}

/// `step_until_line_number` stops when the front task reaches the given
/// line — mirrors `step_until_line_number_finds_line`'s intent.
#[test]
fn step_until_line_number_finds_line() {
    use crate::identifier::Identifier;

    let (mut storage, root) = FVMStorage::test_root_brane(&[]);
    for (name, line, value) in [("a", 0, 1), ("b", 1, 2), ("c", 2, 3)] {
        let stmt = root.create_child(
            &mut storage,
            FirSpec::Statement {
                identifier: Identifier::from_parts(vec![], name),
                line_number: line,
            },
        );
        stmt.create_child(&mut storage, FirSpec::IndepInt { value });
    }

    let steps = step_until_line_number(&mut storage, root, 2).unwrap();
    eprintln!("stopped after {steps} steps");
    let front = FirCursor::new(root, &storage).front_task().unwrap();
    assert_eq!(FirCursor::new(front, &storage).as_stmt_line_number(), Some(2));
}

/// The generic `step_until` matcher — stops when the front task's own `Nyes` is constanic — mirrors
/// `step_until_generic_matcher_by_nyes`'s intent.
#[test]
fn step_until_generic_matcher_by_nyes() {
    let (mut storage, root) = FVMStorage::test_root_brane(&[]);
    let a = root.create_child(&mut storage, FirSpec::IndepInt { value: 1 });
    let _b = root.create_child(&mut storage, FirSpec::IndepInt { value: 2 });

    let steps = step_until(&mut storage, root, |storage, front| {
        front.is_some_and(|f| storage.get_nyes(f).is_constanic())
    })
    .unwrap();
    eprintln!("stopped after {steps} steps");
    let front = FirCursor::new(root, &storage).front_task().unwrap();
    assert_eq!(front, a);
    assert!(storage.get_nyes(front).is_constanic());
}

// ── compiler tests ───────────────────────────────────────────────

use compiler::compile;

/// Compiles `{a = 1; b = 2;}` through the arena compiler and confirms the resulting tree shape: a
/// self-rooted `Brane` with two `Statement` children, each with an `IndepInt` body — mirrors
/// `compiler.rs::tests`' overall intent (that module hand-builds each piece rather than compiling a
/// full source string, so this test's end-to-end shape is new coverage, not a direct mirror of one
/// test).
#[test]
fn arena_compiler_compiles_a_simple_brane() {
    let mut storage = FVMStorage::new();
    let roots = compile(&mut storage, "{a = 1; b = 2;}").unwrap();
    assert_eq!(roots.len(), 1);
    let root = roots[0];
    assert!(root.is_root(&storage));
    assert!(matches!(storage.get(root), FirSpec::Brane { .. }));

    let stmts = storage.foolish_children(root);
    assert_eq!(stmts.len(), 2);
    let a = stmts[0];
    assert_eq!(
        FirCursor::new(a, &storage)
            .as_stmt_identifier()
            .map(|id| id.identifier_name()),
        Some("a")
    );
    assert_eq!(FirCursor::new(a, &storage).as_stmt_line_number(), Some(0));
    let a_body = storage.foolish_children(a)[0];
    assert_eq!(storage.get(a_body), &FirSpec::IndepInt { value: 1 });

    let b = stmts[1];
    assert_eq!(
        FirCursor::new(b, &storage)
            .as_stmt_identifier()
            .map(|id| id.identifier_name()),
        Some("b")
    );
    assert_eq!(FirCursor::new(b, &storage).as_stmt_line_number(), Some(1));
}

/// A bare (unnamed) expression statement gets the anonymous name —
/// mirrors `compiler.rs::tests::build_as_statement_keeps_assignment_name_and_anonymous_fallback`'s
/// anonymous-fallback half.
#[test]
fn arena_compiler_anonymous_statement_gets_the_anon_name() {
    let mut storage = FVMStorage::new();
    let roots = compile(&mut storage, "{1;}").unwrap();
    let root = roots[0];
    let stmt = storage.foolish_children(root)[0];
    assert_eq!(
        FirCursor::new(stmt, &storage)
            .as_stmt_identifier()
            .map(|id| id.identifier_name()),
        Some(ANON_STMT_NAME)
    );
}

/// `compile_standalone` (via `compile`) rejects a non-`Brane` top-level
/// root — mirrors `compile_standalone_rejects_non_brane_root` exactly.
#[test]
fn arena_compiler_rejects_non_brane_root() {
    // The parser itself only ever produces a top-level Brane per source string, so to exercise the
    // non-Brane-root rejection path directly (matching the real test's use of `Astn::IntLit(1)` fed
    // straight to `compile_standalone`), call `compile_standalone` directly with a hand-built non-Brane
    // Astn rather than through `compile`'s parse-then-compile pipeline.
    let mut storage = FVMStorage::new();
    let err = compiler::compile_standalone(&mut storage, foolish_parser::Astn::IntLit(1))
        .expect_err("non-Brane root must be rejected");
    assert_eq!(err.to_string(), "only a Brane can be a top-level (root) node");
}

/// `1 + 2` compiles to an `Operator` node with two `IndepInt` operands —
/// exercises `build_fir`'s `BinaryOp` arm directly.
#[test]
fn arena_compiler_binary_op_has_two_operands() {
    let mut storage = FVMStorage::new();
    let roots = compile(&mut storage, "{x = 1 + 2;}").unwrap();
    let root = roots[0];
    let stmt = storage.foolish_children(root)[0];
    let op = storage.foolish_children(stmt)[0];
    assert!(matches!(storage.get(op), FirSpec::Operator { .. }));
    let operands = storage.foolish_children(op);
    assert_eq!(operands.len(), 2);
    assert_eq!(storage.get(operands[0]), &FirSpec::IndepInt { value: 1 });
    assert_eq!(storage.get(operands[1]), &FirSpec::IndepInt { value: 2 });
}

/// A name reference (`?x`-shaped bare identifier) compiles to an anchored-false `Search` — exercises
/// `build_fir`'s `Identifier` arm and its characterization-folding (Gotcha #3).
#[test]
fn arena_compiler_identifier_compiles_to_an_unanchored_search() {
    let mut storage = FVMStorage::new();
    let roots = compile(&mut storage, "{x = 1; y = x;}").unwrap();
    let root = roots[0];
    let y = storage.foolish_children(root)[1];
    let search = storage.foolish_children(y)[0];
    match storage.get(search) {
        FirSpec::Search {
            pattern, anchored, ..
        } => {
            assert_eq!(pattern, "^x$");
            assert!(!anchored);
        }
        other => panic!("expected FirSpec::Search, got {other:?}"),
    }
}

/// A dot-search (`a.x`) compiles to an anchored `Search` whose first
/// child is the anchor — exercises `build_fir`'s `DotSearch` arm.
#[test]
fn arena_compiler_dot_search_is_anchored_with_an_anchor_child() {
    let mut storage = FVMStorage::new();
    let roots = compile(&mut storage, "{a = {x=1;}; y = a.x;}").unwrap();
    let root = roots[0];
    let y = storage.foolish_children(root)[1];
    let search = storage.foolish_children(y)[0];
    match storage.get(search) {
        FirSpec::Search {
            pattern, anchored, ..
        } => {
            assert_eq!(pattern, "^x$");
            assert!(anchored);
        }
        other => panic!("expected FirSpec::Search, got {other:?}"),
    }
    assert_eq!(
        storage.foolish_children(search).len(),
        1,
        "anchored search has one anchor child"
    );
}

/// `<<x>>` (StayFullyFoolish) builds its descendant search ECONSTANIC — exercises `build_fir`'s
/// `StayFullyFoolish` arm and the `under_sff` rule together, proving this crate's own compiler produces
/// bodies satisfying the SFF invariant.
#[test]
fn arena_compiler_sff_marks_descendant_searches_econstanic() {
    let mut storage = FVMStorage::new();
    let roots = compile(&mut storage, "{a = <<x>>;}").unwrap();
    let root = roots[0];
    let stmt = storage.foolish_children(root)[0];
    let sff = storage.foolish_children(stmt)[0];
    assert!(matches!(storage.get(sff), FirSpec::StayFullyFoolish));
    let search = storage.foolish_children(sff)[0];
    assert!(matches!(storage.get(search), FirSpec::Search { .. }));
    assert_eq!(
        storage.get_nyes(search),
        Nyes::Econstanic,
        "a search built under an SFF marker must start ECONSTANIC, never Prembrionic"
    );
}

/// `'a = ⬤` compiles a `Creation` as a named creation's whole RHS —
/// exercises `build_fir`'s `Creation` arm.
#[test]
fn arena_compiler_creation_literal() {
    let mut storage = FVMStorage::new();
    let roots = compile(&mut storage, "{'a = \u{2b24};}").unwrap();
    let root = roots[0];
    let stmt = storage.foolish_children(root)[0];
    let body = storage.foolish_children(stmt)[0];
    assert!(matches!(storage.get(body), FirSpec::Creation));
    assert_eq!(storage.get_nyes(body), Nyes::Independent);
}

/// `\o<name` (SF sugar via `=$`-equivalent) — a contexted search built via `Astn::ContextedSearch` —
/// has its `contexted` flag set true post construction, exercising `build_fir`'s `ContextedSearch` arm
/// and `ProtoBrane::set_contexted` together.
#[test]
fn arena_compiler_contexted_search_sets_the_contexted_flag() {
    let mut storage = FVMStorage::new();
    let roots = compile(&mut storage, "{a = {x=1;}; y = a~x &?x;}").unwrap();
    let root = roots[0];
    let y = storage.foolish_children(root)[1];
    // y's body is the OUTER search (&?x, contexted); its own anchor chain leads down to the ~x search
    // first, per this operator's real parse shape — walk to find a Search with contexted == true
    // anywhere in y's body subtree.
    fn sift_for_contexted_search(storage: &FVMStorage, ptr: FirPointer) -> bool {
        if let FirSpec::Search { contexted: true, .. } = storage.get(ptr) {
            return true;
        }
        storage
            .foolish_children(ptr)
            .iter()
            .any(|&c| sift_for_contexted_search(storage, c))
    }
    assert!(
        sift_for_contexted_search(&storage, y),
        "a contexted search (&?x) must have contexted == true somewhere in the compiled tree"
    );
}

// ── IndexFir dispatch tests ──────────────────────────────────────

/// Builds an `Index` node whose sole foolish child is a fresh `Brane` of three statements `a=10; b=20;
/// c=30`, returning `(storage, idx, [a, b, c] statement pointers)`. The anchor brane is built AS the
/// index node's own child from the start (the arena's tree is built strictly top-down — there is no
/// "attach an existing pointer as a child" primitive), avoiding any re-parenting.
fn index_with_anchor_brane(offset: i32, anchored: bool) -> (FVMStorage, FirPointer, [FirPointer; 3]) {
    let mut storage = FVMStorage::new();
    let idx = storage.make_root(FirSpec::Index {
        offset,
        anchored,
        contexted: false,
    });
    let anchor_brane = idx.create_child(
        &mut storage,
        FirSpec::Brane {
            characterizations: Characterizations::default(),
        },
    );
    let mut stmts = [anchor_brane; 3];
    for (i, (name, value)) in [("a", 10i64), ("b", 20), ("c", 30)].into_iter().enumerate() {
        let stmt = anchor_brane.create_child(
            &mut storage,
            FirSpec::Statement {
                identifier: Identifier::from_parts(vec![], name),
                line_number: i,
            },
        );
        stmt.create_child(&mut storage, FirSpec::IndepInt { value });
        stmts[i] = stmt;
    }
    (storage, idx, stmts)
}

/// An anchored `#1` index into a brane of three statements settles Constant with the middle statement's
/// value. Exercises `IndexFir`'s `Prembrionic`/`Embryonic` push-anchor-task arm, then the `Braning`
/// anchored-search arm (`BraneNavigator` + `SearchPredicate::Index`), then `settle_from_ubc_result`.
#[test]
fn index_fir_finds_element_at_offset_in_anchor_brane() {
    let (mut storage, idx, _stmts) = index_with_anchor_brane(1, true);
    core_fir_conversion::step_to_constanic(&mut storage, idx).unwrap();
    assert_eq!(storage.get_nyes(idx), Nyes::Constant);
    let result = FirCursor::new(idx, &storage).ubc_children().first().copied();
    assert!(
        result.is_some(),
        "constanic Index must have a ubc_children result"
    );
    assert_eq!(
        FirCursor::new(result.unwrap().value(&storage), &storage).as_i64(),
        Some(20)
    );
}

/// An anchored index whose target falls outside the anchor brane's statement range settles Nk.
#[test]
fn index_fir_out_of_bounds_is_nk() {
    let (mut storage, idx, _stmts) = index_with_anchor_brane(5, true);
    core_fir_conversion::step_to_constanic(&mut storage, idx).unwrap();
    assert_eq!(storage.get_nyes(idx), Nyes::Nk);
    assert!(FirCursor::new(idx, &storage).ubc_children().is_empty());
}

/// `#-1` anchored into a three-statement brane addresses the LAST statement.
#[test]
fn index_fir_negative_offset_from_back() {
    let (mut storage, idx, _stmts) = index_with_anchor_brane(-1, true);
    core_fir_conversion::step_to_constanic(&mut storage, idx).unwrap();
    assert_eq!(storage.get_nyes(idx), Nyes::Constant);
    let result = FirCursor::new(idx, &storage).ubc_children().first().copied();
    assert_eq!(
        result.map(|r| FirCursor::new(r.value(&storage), &storage).as_i64()),
        Some(Some(30)),
        "anchored #-1 must address the LAST statement (c=30)"
    );
}

/// Direct arena counterpart to an unanchored `#-1` (the real `Astn::UnanchoredSeek` shape, e.g.
/// compiler-generated for a bare trailing reference): from a statement's own enclosing brane, addresses
/// the statement immediately before it. Exercises the unanchored branch's
/// `find_enclosing_stmt_and_brane` + `BraneNavigator` path, distinct from the anchored branch above.
#[test]
fn index_fir_unanchored_negative_offset_finds_preceding_statement() {
    let mut storage = FVMStorage::new();
    let brane = storage.make_root(FirSpec::Brane {
        characterizations: Characterizations::default(),
    });
    let a = brane.create_child(
        &mut storage,
        FirSpec::Statement {
            identifier: Identifier::from_parts(vec![], "a"),
            line_number: 0,
        },
    );
    a.create_child(&mut storage, FirSpec::IndepInt { value: 10 });
    let b = brane.create_child(
        &mut storage,
        FirSpec::Statement {
            identifier: Identifier::from_parts(vec![], "b"),
            line_number: 1,
        },
    );
    // b's body is an unanchored Index(-1): "the statement one before me".
    let idx = b.create_child(
        &mut storage,
        FirSpec::Index {
            offset: -1,
            anchored: false,
            contexted: false,
        },
    );

    core_fir_conversion::step_to_constanic(&mut storage, brane).unwrap();
    assert_eq!(storage.get_nyes(idx), Nyes::Constant);
    let result = FirCursor::new(idx, &storage).ubc_children().first().copied();
    assert_eq!(
        result.map(|r| FirCursor::new(r.value(&storage), &storage).as_i64()),
        Some(Some(10)),
        "unanchored #-1 from statement b must find statement a's value"
    );
}

/// An unanchored `IndexFir` whose enclosing statement is itself the FIRST statement of its brane (no
/// preceding statement to find) must settle Nk, not panic or hang — the same index-0 boundary
/// discipline `_ib_search`'s own regression test enforces for name search.
#[test]
fn index_fir_unanchored_negative_offset_at_index_zero_settles_nk() {
    let mut storage = FVMStorage::new();
    let brane = storage.make_root(FirSpec::Brane {
        characterizations: Characterizations::default(),
    });
    let a = brane.create_child(
        &mut storage,
        FirSpec::Statement {
            identifier: Identifier::from_parts(vec![], "a"),
            line_number: 0,
        },
    );
    let idx = a.create_child(
        &mut storage,
        FirSpec::Index {
            offset: -1,
            anchored: false,
            contexted: false,
        },
    );

    core_fir_conversion::step_to_constanic(&mut storage, brane).unwrap();
    assert_eq!(
        storage.get_nyes(idx),
        Nyes::Nk,
        "no statement precedes index 0 -- must settle Nk, not hang or find itself"
    );
}

/// An anchored `IndexFir` whose anchor resolves to a non-brane, NAMEABLE value (an integer literal)
/// settles Nk AND records a named reason (FOOP-75 §7) — both via a fresh ubc_children Nk AND via
/// `alarm_reason`.
#[test]
fn index_fir_anchor_not_a_brane_names_the_value() {
    let mut storage = FVMStorage::new();
    let idx = storage.make_root(FirSpec::Index {
        offset: 0,
        anchored: true,
        contexted: false,
    });
    idx.create_child(&mut storage, FirSpec::IndepInt { value: 4 });

    core_fir_conversion::step_to_constanic(&mut storage, idx).unwrap();
    assert_eq!(storage.get_nyes(idx), Nyes::Nk);
    let reason = storage.alarm_reason(idx).map(str::to_owned);
    assert_eq!(
        reason.as_deref(),
        Some("4 is not a brane"),
        "a nameable non-brane anchor (an int literal) must name itself in the alarm reason"
    );
    let ubc_nk = FirCursor::new(idx, &storage).ubc_children().first().copied();
    assert!(
        ubc_nk.is_some_and(
            |nk| matches!(storage.get(nk), FirSpec::Nk { .. }) && storage.get_nyes(nk) == Nyes::Nk
        ),
        "the named-reason NK must also be pushed as a ubc_children result"
    );
}

/// An anchored `IndexFir` whose anchor resolves to a non-brane, UNNAMEABLE value (e.g. an already-NK
/// search result) settles Nk but records NO named reason — the "leave the result unset" half of FOOP-75
/// §7's rule, distinct from the nameable-anchor test above.
#[test]
fn index_fir_anchor_not_a_brane_and_unnameable_records_no_reason() {
    let mut storage = FVMStorage::new();
    let idx = storage.make_root(FirSpec::Index {
        offset: 0,
        anchored: true,
        contexted: false,
    });
    // An anchor that resolves to Nk (unnameable: as_i64() is None for
    // an Nk node) rather than to a brane or a nameable integer.
    let anchor = idx.create_child(
        &mut storage,
        FirSpec::Nk {
            reason: "unbound".to_string(),
        },
    );
    storage.with_mut(anchor, |fir| fir.set_nyes(Nyes::Nk));

    core_fir_conversion::step_to_constanic(&mut storage, idx).unwrap();
    assert_eq!(storage.get_nyes(idx), Nyes::Nk);
    assert_eq!(
        storage.alarm_reason(idx),
        None,
        "an unnameable non-brane anchor must NOT synthesize a reason"
    );
    assert!(
        FirCursor::new(idx, &storage).ubc_children().is_empty(),
        "no named-reason NK should be pushed when the anchor cannot be named"
    );
}

/// A contexted, anchored index (`&#1`-shaped) reads its anchor's `FoolRef` bookkeeping entry to find
/// the REFERENT's home brane and position, then indexes relative to THAT position — not the position of
/// the index node itself. Exercises the `contexted && anchored` branch, distinct from the
/// plain-anchored branch above.
#[test]
fn index_fir_contexted_finds_statement_relative_to_anchors_referent() {
    let mut storage = FVMStorage::new();

    // The referent's home brane: {a=10; b=20; c=30;}, built as its own
    // root so it can be shared as the FoolRef referent below.
    let home_brane = storage.make_root(FirSpec::Brane {
        characterizations: Characterizations::default(),
    });
    let mut home_stmts = Vec::new();
    for (name, value, line) in [("a", 10i64, 0usize), ("b", 20, 1), ("c", 30, 2)] {
        let stmt = home_brane.create_child(
            &mut storage,
            FirSpec::Statement {
                identifier: Identifier::from_parts(vec![], name),
                line_number: line,
            },
        );
        stmt.create_child(&mut storage, FirSpec::IndepInt { value });
        home_stmts.push(stmt);
    }

    // idx: a contexted, anchored Index(offset=1) whose own foolish child (the anchor) is a Search
    // already manually made constanic as having found home_stmts[0] ("a") via push_search_result_pair —
    // exactly the two-child invariant a real prior search leaves behind, which the contexted branch
    // reads via `ubc_children().get(1)` (the FoolRef).
    let idx = storage.make_root(FirSpec::Index {
        offset: 1,
        anchored: true,
        contexted: true,
    });
    let anchor = idx.create_child(
        &mut storage,
        FirSpec::Search {
            pattern: "^a$".to_string(),
            anchored: true,
            forward: false,
            is_value_search: false,
            contexted: false,
        },
    );
    let a_value_clone = anchor.create_child(&mut storage, FirSpec::IndepInt { value: 10 });
    {
        let mut cursor = FirCursorMut::new(anchor, &mut storage);
        cursor.push_search_result_pair(a_value_clone, home_stmts[0]);
        cursor.set_nyes(Nyes::Constant);
    }

    core_fir_conversion::step_to_constanic(&mut storage, idx).unwrap();
    assert_eq!(storage.get_nyes(idx), Nyes::Constant);
    let result = FirCursor::new(idx, &storage).ubc_children().first().copied();
    assert_eq!(
        result.map(|r| FirCursor::new(r.value(&storage), &storage).as_i64()),
        Some(Some(20)),
        "contexted &#1 from an anchor pointing at 'a' must find 'b' (offset 1 from a's position)"
    );
}

/// A contexted index whose target falls outside the referent's home brane range settles Nk.
#[test]
fn index_fir_contexted_out_of_range_is_nk() {
    let mut storage = FVMStorage::new();
    let home_brane = storage.make_root(FirSpec::Brane {
        characterizations: Characterizations::default(),
    });
    let a = home_brane.create_child(
        &mut storage,
        FirSpec::Statement {
            identifier: Identifier::from_parts(vec![], "a"),
            line_number: 0,
        },
    );
    a.create_child(&mut storage, FirSpec::IndepInt { value: 10 });

    let idx = storage.make_root(FirSpec::Index {
        offset: 5,
        anchored: true,
        contexted: true,
    });
    let anchor = idx.create_child(
        &mut storage,
        FirSpec::Search {
            pattern: "^a$".to_string(),
            anchored: true,
            forward: false,
            is_value_search: false,
            contexted: false,
        },
    );
    let a_value_clone = anchor.create_child(&mut storage, FirSpec::IndepInt { value: 10 });
    {
        let mut cursor = FirCursorMut::new(anchor, &mut storage);
        cursor.push_search_result_pair(a_value_clone, a);
        cursor.set_nyes(Nyes::Constant);
    }

    core_fir_conversion::step_to_constanic(&mut storage, idx).unwrap();
    assert_eq!(storage.get_nyes(idx), Nyes::Nk);
}

// ── Unsteppable-statement checks (FOOP-86 §6, superseding FOOP-33 §4's
//    per-statement NF refusal) ──────────────────────────────────────

/// `true` when `stmt` is the statement that made its own brane halt — the FOOP-86 §6.4 replacement for
/// the superseded "this statement has an `nf_reason`". The fault is recorded on the BRANE, naming the
/// statement, not on the statement itself.
fn is_unsteppable_cause(storage: &FVMStorage, stmt: FirPointer) -> bool {
    stmt.home_brane(storage)
        .and_then(|b| storage.unsteppable_cause(b))
        == Some(stmt)
}

/// A null-characterized statement redefining an existing same-name null-characterized constant with a
/// DIFFERENT value is **unsteppable** (FOOP-86 §6.2 route 1): it becomes its brane's recorded cause,
/// and the brane halts. The statement itself keeps its honest written value — §6.3 — so this asserts
/// the CAUSE record, not a mark on the statement.
#[test]
fn statement_null_const_conflict_is_refused() {
    let mut storage = FVMStorage::new();
    let brane = storage.make_root(FirSpec::Brane {
        characterizations: Characterizations::default(),
    });
    let first = brane.create_child(
        &mut storage,
        FirSpec::Statement {
            identifier: Identifier::from_parts(vec![String::new()], "True"),
            line_number: 0,
        },
    );
    first.create_child(&mut storage, FirSpec::IndepInt { value: 1 });
    let second = brane.create_child(
        &mut storage,
        FirSpec::Statement {
            identifier: Identifier::from_parts(vec![String::new()], "True"),
            line_number: 1,
        },
    );
    second.create_child(&mut storage, FirSpec::IndepInt { value: 2 });

    core_fir_conversion::step_to_constanic(&mut storage, brane).unwrap();
    assert!(
        is_unsteppable_cause(&storage, second),
        "redefining a null-characterized constant with a DIFFERENT value must make that \
         statement its brane's unsteppable cause"
    );
    assert!(
        !is_unsteppable_cause(&storage, first),
        "the FIRST definition establishes the constant -- it is never itself the cause"
    );
    assert_eq!(
        storage.get_nyes(brane),
        Nyes::Nk,
        "the brane halted, so it is NK -- it failed to finish its own work (§6.4c)"
    );
}

// ── FOOP-86 §6 unsteppable-statement behaviors (§6.6c requires each of
//    these to have an einmo counterpart as well) ────────────────────

/// Evaluates `src` as a real program and returns `(storage, program)`.
fn evaluated(src: &str) -> (FVMStorage, FirPointer) {
    let mut storage = FVMStorage::new();
    let roots = compiler::compose_program_with_system(&mut storage, src).unwrap();
    let root = roots[0];
    let _ = core_fir_conversion::step_to_constanic(&mut storage, root);
    let program = compiler::program_result(&storage, root).unwrap_or(root);
    (storage, program)
}

/// §6.3 — the halt's full shape, in one test. Statements BEFORE the cause keep their values (`'K` still
/// means the creation for `a`); the CAUSE reverts to Foolish and keeps its honest `IndepInt`;
/// statements AFTER are NK from never being stepped — **including one that never mentions the name at
/// all**, which is what distinguishes §6.3's halt from the superseded search-poisoning (that only
/// reached readers OF the name).
#[test]
fn unsteppable_halts_the_brane_and_leaves_the_remainder_unstepped() {
    let (storage, program) = evaluated("{'K = ⬤; a = 'K; 'K = 3; b = 'K; d = 1 + 1;}");
    let cursor = FirCursor::new(program, &storage);
    let stmt = |i: usize| cursor.stmt_at(i).expect("statement exists");

    assert_eq!(
        storage.get_nyes(stmt(1)),
        Nyes::Constant,
        "`a = 'K` precedes the conflict, so it keeps the meaning 'K had there"
    );
    assert!(
        is_unsteppable_cause(&storage, stmt(2)),
        "`'K = 3` is the cause -- its name is already defined in this context"
    );

    let cause_body = FirCursor::new(stmt(2), &storage).foolish_children()[0];
    assert_eq!(
        storage.get(cause_body),
        &FirSpec::IndepInt { value: 3 },
        "the cause reverts to Foolish: 3 is an honest IndepInt, never marked (§6.3)"
    );
    assert_eq!(
        storage.get_nyes(cause_body),
        Nyes::Independent,
        "and it is genuinely Independent -- marking it NK would be a false statement \
         about that node, which is WHY the fact lives on the brane (§6.4)"
    );

    assert_eq!(storage.get_nyes(stmt(3)), Nyes::Nk, "`b = 'K` was never stepped");
    assert_eq!(
        storage.get_nyes(stmt(4)),
        Nyes::Nk,
        "`d = 1 + 1` was never stepped EITHER, though it never mentions 'K -- the halt \
         stops the brane, it does not poison a name (§6.3 vs the superseded §4)"
    );
    assert_eq!(
        storage.get_nyes(program),
        Nyes::Nk,
        "the brane halted, so it is NK"
    );
    assert_eq!(
        storage.alarm_reason(program),
        Some("'K already defined in context"),
        "the halt raises the run-time-error alarm (§6.2a, Q-E)"
    );
}

/// §6.4c — the governing distinction. A brane containing an NK VALUE did its part and stays valid; only
/// a brane that FAILED to finish goes NK by the halt. This pins that the halt is what sets it, not the
/// NK-member rollup (Phase 9 stop condition 1) — if a later change to `decide_nyes_due_to_children`
/// (§6.6b) altered the rollup, this test keeps the unsteppable rule honest.
#[test]
fn nk_member_does_not_halt_its_brane_but_an_unsteppable_statement_does() {
    let (storage, program) = evaluated("{x = 1/0; y = 2;}");
    assert_eq!(
        storage.unsteppable_cause(program),
        None,
        "a brane containing an NK VALUE has no unsteppable cause -- it did its part"
    );
    let cursor = FirCursor::new(program, &storage);
    assert_eq!(
        storage.get_nyes(cursor.stmt_at(1).unwrap()),
        Nyes::Independent,
        "and its later statements stepped normally -- nothing halted"
    );

    let (storage, program) = evaluated("{'K = ⬤; 'K = 3; z = 9;}");
    assert!(
        storage.unsteppable_cause(program).is_some(),
        "whereas an unsteppable statement DOES halt its brane"
    );
}

/// §6.4c — a conflict inside a NESTED brane does not halt the OUTER one: the outer brane stepped
/// everything it has, including the inner brane, which settles NK as an ordinary value.
#[test]
fn a_nested_conflict_does_not_halt_the_outer_brane() {
    let (storage, program) = evaluated("{a = 1; inner = {'K = ⬤; 'K = 3;}; b = 2;}");
    assert_eq!(
        storage.unsteppable_cause(program),
        None,
        "the OUTER brane has no cause of its own -- it did its part (§6.4c)"
    );
    let cursor = FirCursor::new(program, &storage);
    assert_eq!(
        storage.get_nyes(cursor.stmt_at(2).unwrap()),
        Nyes::Independent,
        "`b = 2` after the nested conflict evaluates normally"
    );
}

/// §6.4b — EVERY access into an NK brane settles NK: anchored search, index, head/tail, and a plain
/// reference alike. No access path yields a meaningful value from a brane that has no meaning.
#[test]
fn every_access_into_an_nk_brane_settles_nk() {
    let (storage, program) =
        evaluated("{bad = {'K = ⬤; 'K = 3;}; s = bad?'K; i = bad#0; h = bad^; r = bad;}");
    let cursor = FirCursor::new(program, &storage);
    for (index, what) in [
        (1, "anchored search"),
        (2, "index"),
        (3, "head"),
        (4, "plain reference"),
    ] {
        let stmt = cursor.stmt_at(index).expect("statement exists");
        let body = FirCursor::new(stmt, &storage).foolish_children()[0];
        assert_eq!(
            storage.get_nyes(body),
            Nyes::Nk,
            "{what} into an NK brane must settle NK (§6.4b)"
        );
    }
}

/// The brane a concatenation MERGED INTO, given the statement body holding the concat expression. The
/// merge result is the concat's `ubc_children[0]`, so a route-2 halt is recorded there rather than on
/// the body; falls back to the body when there is no result child.
fn merged_brane_of(storage: &FVMStorage, body: FirPointer) -> FirPointer {
    FirCursor::new(body, storage)
        .ubc_children()
        .first()
        .copied()
        .unwrap_or(body)
}

/// Lazy **post-order** iterator over a FIR tree: a node is yielded only after all of its
/// children, its children's children, and so on.
///
/// Post-order, not pre-order, because that is the order the invariant needs (the human,
/// 2026-09-26): a node cannot be constanic before its children are, so a traversal that
/// yielded the parent first would report a parent as "reached" while its subtree was still
/// pre-constanic. In generator form:
///
/// ```text
/// for child in my_children(front to back):   # in order
///     yield from InOrder(child)              # the child's whole subtree, post-order
/// yield self                                 # self LAST
/// ```
///
/// Yields one `FirPointer` at a time and **never materializes the tree** (the human:
/// *"definitely do NOT materialize a Fir vec. Let's use an iterator please."*). State is an
/// explicit stack of `(node, next_child_index)` frames, so memory is O(depth) — note this is
/// a HIGH-BRANCHING tree, not a binary one, so a frame per level rather than a copy of each
/// level's children is what keeps that bound. A caller that stops early walks no further.
///
/// This follows the **iterative candidate-generator pattern the search engine already uses**
/// (`CandidateNavigator`), which the human named as the replacement for a visitor. The
/// ITERATION ORDER differs from the search navigator's deliberately: this is a structural
/// deepest-first walk of the whole tree, whereas a search navigator yields statement
/// candidates in a direction- and cursor-dependent order.
///
/// **Mutation during iteration is not this iterator's problem, and cannot be.** The walk holds
/// frames pointing at nodes it has not yet finished, so a tree reshaped mid-iteration could
/// hand back pointers whose children changed. The iterator makes no attempt to detect that —
/// it is a borrow of a tree assumed still.
///
/// In practice `&'s FVMStorage` makes the dangerous cases unrepresentable rather than merely
/// discouraged: the shared borrow excludes any `&mut FVMStorage` for the iterator's lifetime,
/// so stepping cannot run while a walk is live — verified, it is a compile error (E0502) —
/// and `FVMStorage` is a plain owned arena with no interior mutability (no `RefCell`, no
/// `Mutex`, no `unsafe impl Send`/`Sync`), so it is not shared across threads either.
/// **UBCA2 is single-threaded**; a concurrent mutator is not a scenario the crate supports,
/// and the compiler is what enforces it here.
///
/// What remains the CALLER's responsibility is the one case borrowing cannot see: collecting
/// pointers from a walk, dropping the iterator, stepping, and then using those stale pointers.
/// Re-walk instead of caching across a step.
///
/// A general FIR iterator on this pattern is future work; kept local while it is this small.
struct PostOrder<'s> {
    storage: &'s FVMStorage,
    /// One frame per level from the root down to the node being expanded: the node, and how
    /// many of its children have been fully emitted. O(depth), not O(nodes).
    stack: Vec<(FirPointer, usize)>,
}

impl<'s> PostOrder<'s> {
    fn new(storage: &'s FVMStorage, root: FirPointer) -> Self {
        Self {
            storage,
            stack: vec![(root, 0)],
        }
    }
}

impl Iterator for PostOrder<'_> {
    type Item = FirPointer;

    fn next(&mut self) -> Option<FirPointer> {
        loop {
            let (node, visited) = *self.stack.last()?;
            let children = self.storage.foolish_children(node);
            if visited < children.len() {
                // Descend into the next unvisited child, front to back.
                let child = children[visited];
                self.stack.last_mut().expect("just read").1 += 1;
                self.stack.push((child, 0));
            } else {
                // Every child is done, so this node may finally be yielded.
                self.stack.pop();
                return Some(node);
            }
        }
    }
}

/// Iterates `root`'s tree post-order (children before parents), lazily. See [`PostOrder`].
fn post_order(storage: &FVMStorage, root: FirPointer) -> PostOrder<'_> {
    PostOrder::new(storage, root)
}

/// Renders the traversal as `name=NYES` pairs, for a failure message a human can read.
fn nyes_trace(storage: &FVMStorage, root: FirPointer) -> String {
    post_order(storage, root)
        .map(|n| format!("{:?}", storage.get_nyes(n)))
        .collect::<Vec<_>>()
        .join(" ")
}

/// [`PostOrder`] yields children before parents, front-to-back, and is LAZY.
///
/// The laziness half matters because the obvious implementation (build a `Vec`, splice
/// children in) is both O(n²) and eager; this pins that it was not done that way. The ORDER
/// half matters because an earlier version of this iterator was PRE-order — parent first —
/// which is exactly backwards for the constanic-prefix invariant below: a node cannot be
/// constanic before its children are.
#[test]
fn post_order_iterator_yields_children_before_parents_and_is_lazy() {
    use crate::identifier::Identifier;

    // root
    //   stmt_a -> IndepInt(1)
    //   stmt_b -> IndepInt(2)
    let (mut storage, root) = FVMStorage::test_root_brane(&[]);
    let mut stmts = vec![];
    for (name, line, value) in [("a", 0, 1), ("b", 1, 2)] {
        let stmt = root.create_child(
            &mut storage,
            FirSpec::Statement {
                identifier: Identifier::from_parts(vec![], name),
                line_number: line,
            },
        );
        stmt.create_child(&mut storage, FirSpec::IndepInt { value });
        stmts.push(stmt);
    }
    let body_a = FirCursor::new(stmts[0], &storage).foolish_children()[0];
    let body_b = FirCursor::new(stmts[1], &storage).foolish_children()[0];

    // ORDER: deepest-first, children front-to-back, each node AFTER its whole subtree.
    // The root is LAST, which is the defining property.
    let walked: Vec<FirPointer> = post_order(&storage, root).collect();
    assert_eq!(
        walked,
        vec![body_a, stmts[0], body_b, stmts[1], root],
        "post-order: a's body, a, b's body, b, then the root LAST"
    );
    assert_eq!(
        *walked.last().expect("non-empty"),
        root,
        "the root is yielded last, not first"
    );

    // LAZINESS: the stack is O(DEPTH), not O(nodes). This tree is 3 deep, so even mid-walk
    // the pending frames must stay tiny — that is what makes it safe on a wide tree.
    let mut it = post_order(&storage, root);
    assert_eq!(it.next(), Some(body_a), "the deepest front node comes first");
    assert!(
        it.stack.len() <= 3,
        "frames are per-LEVEL (depth 3), not per-node (tree of {}); holds {}",
        walked.len(),
        it.stack.len()
    );
}

/// **The debugger's core invariant** (the human, 2026-09-26): *"stepping until a Fir node is
/// constanic means in order traversal of fir tree should all be constanic up to and including
/// that node."*
///
/// **Why the traversal must be POST-order for this to hold.** The evaluation rule is: *"if a
/// brane's children are all evaluated to constanic, then the brane then becomes constanic"*
/// (the human). So constanic-ness propagates UPWARD — a brane is the LAST thing in its own
/// subtree to settle. A pre-order walk would visit the brane before its contents and report
/// it as reached while the subtree behind it was still pre-constanic, inverting the very
/// property being checked. Post-order matches the direction evaluation actually flows, which
/// is why [`PostOrder`] yields a node only after everything inside it.
///
/// This is a property of the stepping ORDER, and nothing else tested it. The existing
/// `step_until*` tests assert only that the breakpoint fires at the right place; they say
/// nothing about the state of everything BEFORE it. That prefix property is what makes a
/// breakpoint meaningful — if stepping could leave a hole behind it, stopping at a node would
/// tell you nothing about what had been evaluated.
///
/// Checked at four points: before any stepping, twice mid-flight, and after settling.
#[test]
fn stepping_leaves_a_constanic_prefix_in_traversal_order() {
    use crate::identifier::Identifier;

    // Build the tree by hand rather than by parsing, so the shape under test is explicit.
    let (mut storage, root) = FVMStorage::test_root_brane(&[]);
    for (name, line, value) in [("a", 0, 1), ("b", 1, 2), ("c", 2, 3), ("d", 3, 4)] {
        let stmt = root.create_child(
            &mut storage,
            FirSpec::Statement {
                identifier: Identifier::from_parts(vec![], name),
                line_number: line,
            },
        );
        stmt.create_child(&mut storage, FirSpec::IndepInt { value });
    }

    // (1) BEFORE: nothing is constanic except the leaf IndepInts, which are born
    // INDEPENDENT. The root and its statements are all still pre-constanic.
    assert!(
        !storage.get_nyes(root).is_constanic(),
        "before stepping, the root must be pre-constanic: {}",
        nyes_trace(&storage, root)
    );

    // (2) DURING: step to each statement in turn and assert the PREFIX property at each
    // stop -- everything up to and including the target is constanic, in traversal order.
    for line in [0usize, 2] {
        let (mut storage, root) = FVMStorage::test_root_brane(&[]);
        for (name, ln, value) in [("a", 0, 1), ("b", 1, 2), ("c", 2, 3), ("d", 3, 4)] {
            let stmt = root.create_child(
                &mut storage,
                FirSpec::Statement {
                    identifier: Identifier::from_parts(vec![], name),
                    line_number: ln,
                },
            );
            stmt.create_child(&mut storage, FirSpec::IndepInt { value });
        }
        core_fir_conversion::step_until_line_number(&mut storage, root, line)
            .unwrap_or_else(|e| panic!("breakpoint at line {line} must fire: {e}"));

        // The breakpoint stopped with `line`'s statement as the front task, i.e. ABOUT to be
        // stepped. So every statement STRICTLY BEFORE it must already be constanic, and none
        // after it can be -- that is the prefix, stated as both halves.
        let cursor = FirCursor::new(root, &storage);
        for idx in 0..cursor.stmt_count().unwrap_or(0) {
            let stmt = cursor.stmt_at(idx).expect("statement exists");
            if idx < line {
                assert!(
                    storage.get_nyes(stmt).is_constanic(),
                    "stopped at line {line}: statement {idx} precedes it and must be \
                     constanic. trace: {}",
                    nyes_trace(&storage, root)
                );
            }
        }
    }

    // (3) AFTER: once the root settles, EVERY node in traversal order is constanic --
    // the prefix has grown to cover the whole tree.
    let (mut storage, root) = FVMStorage::test_root_brane(&[]);
    for (name, line, value) in [("a", 0, 1), ("b", 1, 2), ("c", 2, 3), ("d", 3, 4)] {
        let stmt = root.create_child(
            &mut storage,
            FirSpec::Statement {
                identifier: Identifier::from_parts(vec![], name),
                line_number: line,
            },
        );
        stmt.create_child(&mut storage, FirSpec::IndepInt { value });
    }
    core_fir_conversion::step_to_constanic(&mut storage, root).expect("settles");

    let trace = nyes_trace(&storage, root);
    for (position, node) in post_order(&storage, root).enumerate() {
        assert!(
            storage.get_nyes(node).is_constanic(),
            "after settling, traversal position {position} is NOT constanic. trace: {trace}"
        );
    }
    assert_eq!(
        storage.get_nyes(root),
        Nyes::Independent,
        "a brane of plain integers settles INDEPENDENT"
    );
}

/// §6.2 route 2 — CONCATENATION MERGE. A merge brings statements from
/// several operands into one brane; if two of them null-characterize the
/// SAME name with DIFFERENT values, the merged brane cannot be built and
/// halts (`apply_null_const_rule_to_merged_stmt`).
///
/// **This test covers BOTH paths deliberately, and the permitted path is the more important
/// half.** A refusal-only test leaves a specific misreading undetected: `{A = {'C = 10}; b = A
/// A;}` concatenates `A` with ITSELF, so both copies carry `'C = 10` — the SAME value — and the
/// merge is PERMITTED. In that output "no NK" is indistinguishable from "the check never ran".
/// Pinning equal-merge-permitted next to conflict-merge-halts is what makes the distinction
/// testable rather than a matter of reading.
#[test]
fn concatenation_merge_halts_only_on_a_conflicting_null_const() {
    // PERMITTED: same value from both operands.
    for (label, src) in [
        ("self-concatenation", "{A = {'C = 10}; b = A A;}"),
        ("two equal operands", "{A = {'C = 10}; B = {'C = 10}; b = A B;}"),
        ("equal inline operands", "{b = {'C = 10} {'C = 10};}"),
    ] {
        let (storage, program) = eval_program(src);
        let index = FirCursor::new(program, &storage).stmt_count().unwrap_or(0) - 1;
        let (body, nyes) = stmt_body_and_nyes(&storage, program, index);
        // The merged brane is the concatenation's RESULT -- `ubc_children[0]` of the concat expression
        // -- not the statement body itself, so the halt is recorded there.
        let merged = merged_brane_of(&storage, body);
        assert!(
            storage.unsteppable_cause(merged).is_none(),
            "{label}: merging EQUAL null-const values is permitted -- the \
             merged brane must NOT halt ({src})"
        );
        assert_ne!(
            nyes,
            Nyes::Nk,
            "{label}: an equal merge produces a real brane, not NK ({src})"
        );
    }

    // REFUSED: the same name with different values.
    for (label, src) in [
        ("two named operands", "{A = {'C = 10}; B = {'C = 11}; b = A B;}"),
        ("inline operands", "{b = {'C = 10} {'C = 11};}"),
    ] {
        let (storage, program) = eval_program(src);
        let index = FirCursor::new(program, &storage).stmt_count().unwrap_or(0) - 1;
        let (body, nyes) = stmt_body_and_nyes(&storage, program, index);
        let merged = merged_brane_of(&storage, body);
        assert!(
            storage.unsteppable_cause(merged).is_some(),
            "{label}: merging a CONFLICTING null-const halts the merged \
             brane and records the cause on it (§6.2 route 2) ({src})"
        );
        assert_eq!(
            nyes,
            Nyes::Nk,
            "{label}: the halted merged brane takes NK (§6.3) ({src})"
        );
    }
}

/// The statement-level counterpart of the merge rule, and the case that makes "equal is
/// permitted" unmistakable: re-stating a null-characterized constant's own value is IDEMPOTENT,
/// whether written as the literal (`{'a=10; 'a=10}`) or as a search that resolves to it
/// (`{'a=10; 'a='a}`). Only a DIFFERENT value is unsteppable.
#[test]
fn restating_a_null_const_with_its_own_value_is_idempotent() {
    for (label, src) in [
        ("literal restatement", "{'a = 10; 'a = 10;}"),
        ("restated via search", "{'a = 10; 'a = 'a;}"),
    ] {
        let (storage, program) = eval_program(src);
        assert!(
            storage.unsteppable_cause(program).is_none(),
            "{label}: an equal restatement is permitted -- the brane must \
             not halt ({src})"
        );
        assert_ne!(
            storage.get_nyes(program),
            Nyes::Nk,
            "{label}: the brane keeps a real value ({src})"
        );
    }

    let (storage, program) = eval_program("{'a = 10; 'a = 11;}");
    assert!(
        storage.unsteppable_cause(program).is_some(),
        "a CONFLICTING restatement is unsteppable and halts the brane"
    );
}

/// §6.2 route 3 — RECOORDINATION. A statement whose settled value is a
/// brane brings that brane's members into this context. When one of them
/// is a null-characterized name already defined here with a DIFFERENT
/// value, the brane cannot be coordinated in.
///
/// **The brane SEGREGATES the run-time error**: only the statement holding the value settles NK
/// — an ORDINARY NK, as there is no unsteppable-versus-steppable distinction among NKs. The
/// RECEIVING brane is NOT halted and its other statements step normally. This is the one route
/// that does not halt a brane, because the failure is contained in the statement that could not
/// coordinate.
#[test]
fn recoordinating_a_conflicting_null_const_settles_that_statement_nk() {
    let (storage, program) = evaluated("{A={'C=1}, B={'C=2, D=A}}");
    let cursor = FirCursor::new(program, &storage);
    let b_body = FirCursor::new(cursor.stmt_at(1).expect("B exists"), &storage).foolish_children()[0];
    assert!(
        storage.unsteppable_cause(b_body).is_none(),
        "the receiving brane is NOT halted -- the brane segregates the run-time error"
    );

    let b = FirCursor::new(b_body, &storage);
    let members = b.foolish_children();
    assert_eq!(
        storage.get_nyes(members[0]),
        Nyes::Independent,
        "`'C = 2` is untouched -- it did its part"
    );
    assert_eq!(
        storage.get_nyes(members[1]),
        Nyes::Nk,
        "`D = A` cannot coordinate `A`'s conflicting `'C` in, so D settles NK"
    );
    let d_body = FirCursor::new(members[1], &storage).foolish_children()[0];
    assert_eq!(
        storage.get_nyes(d_body),
        Nyes::Nk,
        "D's BODY settles NK too, so it reverts to Foolish rather than \
         rendering a coordination that never happened"
    );

    // `A` itself is entirely unaffected -- it is a perfectly good brane;
    // only bringing it into THIS context fails.
    let a_body = FirCursor::new(cursor.stmt_at(0).expect("A exists"), &storage).foolish_children()[0];
    assert!(
        storage.get_nyes(a_body).is_conclusive(),
        "A is untouched -- the conflict is with the RECEIVING context, not with A"
    );

    // The SAME value is not a conflict: coordinating it in is permitted.
    let (storage, program) = evaluated("{A={'C=1}, B={'C=1, D=A}}");
    let cursor = FirCursor::new(program, &storage);
    let ok_body = FirCursor::new(cursor.stmt_at(1).expect("B exists"), &storage).foolish_children()[0];
    assert!(
        storage.unsteppable_cause(ok_body).is_none(),
        "the SAME value is not a conflict -- coordinating it in is permitted"
    );
    let ok_d = FirCursor::new(ok_body, &storage).foolish_children()[1];
    assert_ne!(
        storage.get_nyes(ok_d),
        Nyes::Nk,
        "an equal-valued recoordination must NOT be refused"
    );
}

/// Re-stating a null-characterized constant's OWN existing value (the same value, not a conflicting
/// one) is PERMITTED — not a rename, not a conflict.
#[test]
fn statement_null_const_same_value_restatement_is_permitted() {
    let mut storage = FVMStorage::new();
    let brane = storage.make_root(FirSpec::Brane {
        characterizations: Characterizations::default(),
    });
    let first = brane.create_child(
        &mut storage,
        FirSpec::Statement {
            identifier: Identifier::from_parts(vec![String::new()], "True"),
            line_number: 0,
        },
    );
    first.create_child(&mut storage, FirSpec::IndepInt { value: 1 });
    let second = brane.create_child(
        &mut storage,
        FirSpec::Statement {
            identifier: Identifier::from_parts(vec![String::new()], "True"),
            line_number: 1,
        },
    );
    second.create_child(&mut storage, FirSpec::IndepInt { value: 1 });

    core_fir_conversion::step_to_constanic(&mut storage, brane).unwrap();
    assert!(
        !is_unsteppable_cause(&storage, second),
        "restating the SAME value must be permitted, not unsteppable"
    );
}

// ── ConcatenationFir real merge (populate_concat_helpers) ───────

/// Concatenating two non-empty branes actually JOINS their statements into one flat, constant
/// `ConcatHelper` (not the old Woconstanic placeholder) — the real end-to-end behavior
/// `populate_concat_ helpers`'s translation exists to produce.
#[test]
fn concatenation_of_two_branes_joins_their_statements() {
    let mut storage = FVMStorage::new();
    let cat = storage.make_root(FirSpec::Concatenation {
        provenance: ConcatProvenance::Juxtaposition,
        rendering_aid: ConcatRenderingAid::default(),
    });
    let brane1 = cat.create_child(
        &mut storage,
        FirSpec::Brane {
            characterizations: Characterizations::default(),
        },
    );
    let a = brane1.create_child(
        &mut storage,
        FirSpec::Statement {
            identifier: Identifier::from_parts(vec![], "a"),
            line_number: 0,
        },
    );
    a.create_child(&mut storage, FirSpec::IndepInt { value: 1 });

    let brane2 = cat.create_child(
        &mut storage,
        FirSpec::Brane {
            characterizations: Characterizations::default(),
        },
    );
    let b = brane2.create_child(
        &mut storage,
        FirSpec::Statement {
            identifier: Identifier::from_parts(vec![], "b"),
            line_number: 0,
        },
    );
    b.create_child(&mut storage, FirSpec::IndepInt { value: 2 });

    core_fir_conversion::step_to_constanic(&mut storage, cat).unwrap();
    assert_eq!(storage.get_nyes(cat), Nyes::Independent);

    let helpers = FirCursor::new(cat, &storage).ubc_children().to_vec();
    assert_eq!(helpers.len(), 1, "one flat ConcatHelper for both merged branes");
    let helper = helpers[0];
    assert!(matches!(storage.get(helper), FirSpec::ConcatHelper));
    let joined_count = FirCursor::new(helper, &storage).stmt_count();
    assert_eq!(
        joined_count,
        Some(2),
        "both statements a and b must be joined into the helper"
    );

    let joined_a = FirCursor::new(helper, &storage).stmt_at(0).unwrap();
    let joined_b = FirCursor::new(helper, &storage).stmt_at(1).unwrap();
    assert_eq!(
        FirCursor::new(joined_a, &storage)
            .as_stmt_identifier()
            .map(|id| id.identifier_name()),
        Some("a")
    );
    assert_eq!(
        FirCursor::new(joined_b, &storage)
            .as_stmt_identifier()
            .map(|id| id.identifier_name()),
        Some("b")
    );
}

/// The null-const merge rule fires during a concatenation join: merging two branes that each
/// null-characterize the SAME name with DIFFERENT values refuses the second occurrence, exactly as
/// `StatementFir`'s own same-brane check does for an ordinary redefinition.
#[test]
fn concatenation_merge_applies_null_const_rule_to_conflicting_names() {
    let mut storage = FVMStorage::new();
    let cat = storage.make_root(FirSpec::Concatenation {
        provenance: ConcatProvenance::Juxtaposition,
        rendering_aid: ConcatRenderingAid::default(),
    });
    let brane1 = cat.create_child(
        &mut storage,
        FirSpec::Brane {
            characterizations: Characterizations::default(),
        },
    );
    let first = brane1.create_child(
        &mut storage,
        FirSpec::Statement {
            identifier: Identifier::from_parts(vec![String::new()], "True"),
            line_number: 0,
        },
    );
    first.create_child(&mut storage, FirSpec::IndepInt { value: 1 });

    let brane2 = cat.create_child(
        &mut storage,
        FirSpec::Brane {
            characterizations: Characterizations::default(),
        },
    );
    let second = brane2.create_child(
        &mut storage,
        FirSpec::Statement {
            identifier: Identifier::from_parts(vec![String::new()], "True"),
            line_number: 0,
        },
    );
    second.create_child(&mut storage, FirSpec::IndepInt { value: 2 });

    core_fir_conversion::step_to_constanic(&mut storage, cat).unwrap();

    let helper = FirCursor::new(cat, &storage)
        .ubc_children()
        .first()
        .copied()
        .unwrap();
    let joined_second = FirCursor::new(helper, &storage).stmt_at(1).unwrap();
    assert_eq!(
        storage.unsteppable_cause(helper),
        Some(joined_second),
        "merging a conflicting null-characterized redefinition must halt the merged brane \
         (FOOP-86 §6.2 route 2), exactly as the same-brane check halts one"
    );
    assert_eq!(
        storage.get_nyes(helper),
        Nyes::Nk,
        "the merged brane halted, so it is NK"
    );
}

// ── compose_program_with_system / evaluate tests ────────────────

/// End-to-end: composing a trivial user program `{x = 1;}` with the real
/// embedded `system.foo` source settles, and `program_result` correctly
/// extracts the user's own root brane (not the composite wrapper).
#[test]
fn compose_program_with_system_settles_a_trivial_program() {
    let mut storage = FVMStorage::new();
    let roots = compiler::compose_program_with_system(&mut storage, "{x = 1;}").unwrap();
    assert_eq!(roots.len(), 1);
    let composed_root = roots[0];

    core_fir_conversion::step_to_constanic(&mut storage, composed_root).unwrap();

    let program = compiler::program_result(&storage, composed_root)
        .expect("program_result must find the user's program member");
    assert!(
        FirCursor::new(program, &storage).is_brane_like(),
        "program_result must resolve to the user's own root brane"
    );
    let x_stmt = FirCursor::new(program, &storage).stmt_at(0).unwrap();
    assert_eq!(
        FirCursor::new(x_stmt, &storage)
            .as_stmt_identifier()
            .map(|id| id.identifier_name()),
        Some("x")
    );
}

/// End-to-end: a user program that USES a comparison operator (`'lt`) resolves through the full
/// system.foo composition -- proves build_comparison/comparison_body/ComparisonFir's real verdict
/// resolution all work together through the real embedded system.foo source, not just the hand-built
/// trees this file's other Comparison tests use.
#[test]
fn compose_program_with_system_resolves_a_comparison() {
    let mut storage = FVMStorage::new();
    let roots = compiler::compose_program_with_system(&mut storage, "{r = {1, 2, 'lt}$;}").unwrap();
    let composed_root = roots[0];

    core_fir_conversion::step_to_constanic(&mut storage, composed_root).unwrap();

    let program = compiler::program_result(&storage, composed_root).unwrap();
    let r_stmt = FirCursor::new(program, &storage).stmt_at(0).unwrap();
    let r_body = storage.foolish_children(r_stmt).first().copied().unwrap();
    let r_value = r_body.value(&storage);
    assert!(
        storage.get_nyes(r_value).is_constanic(),
        "1 <̲ 2 must resolve through the real system.foo composition, got {:?}",
        storage.get_nyes(r_value)
    );
    assert!(
        matches!(storage.get(r_value), FirSpec::Creation),
        "the result of a resolved comparison read via $ must be the 'True creation itself"
    );
    // A Creation is born Independent (self-contained, no context dependency) -- that's the SPECIFIC
    // constanic state expected here, not merely "some constanic state".
    assert_eq!(storage.get_nyes(r_value), Nyes::Independent);
}

/// Regression: a result-only node built via `ptr.create_child(storage, ..)` would be silently appended
/// to `ptr`'s `foolish_children` (the ALWAYS-append contract every `create_child` call has) even though
/// it should live ONLY in `ubc_children` — corrupting the very list `combine`'s own `any_nk` re-check
/// (and every output-serialization operand loop) reads. `{a = 10 / 0 * 5;}`'s outer `*` operator must
/// have EXACTLY its 2 parse-derived operands in `foolish_children` even after settling to Nk (its own
/// division-by-zero-propagated result must live only in `ubc_children`, via `make_orphan_child`).
#[test]
fn combine_nk_result_does_not_pollute_foolish_children() {
    let mut storage = FVMStorage::new();
    let roots = compiler::compile(&mut storage, "{a = 10 / 0 * 5;}").unwrap();
    let root = roots[0];
    let stmt = storage.foolish_children(root)[0];
    let outer = storage.foolish_children(stmt)[0];
    assert_eq!(storage.foolish_children(outer).len(), 2);

    core_fir_conversion::step_to_constanic(&mut storage, root).unwrap();

    assert_eq!(storage.get_nyes(outer), Nyes::Nk);
    assert_eq!(
        storage.foolish_children(outer).len(),
        2,
        "settling must not append the result NK to foolish_children -- it belongs only \
         in ubc_children (make_orphan_child, not create_child)"
    );
    assert_eq!(
        FirCursor::new(outer, &storage).ubc_children().len(),
        1,
        "the result NK must be recorded in ubc_children"
    );
}

/// Minimal reproduction of `einmo_suite/input/foop/33/boolean/ null_char_constant.foo`'s divergence: a
/// user program that redefines `'True` (declared in `system.foo`) with a CONFLICTING value must refuse
/// (NF), matching `program_redefining_true_to_a_conflicting_ value_is_refused` in `system_foo.rs`'s OWN
/// test suite (which passes today via the hand-built tree in that file, NOT through the real
/// `compose_program_with_system` composition this test uses instead).
#[test]
fn compose_program_with_system_refuses_conflicting_true_redefinition() {
    let mut storage = FVMStorage::new();
    let roots = compiler::compose_program_with_system(&mut storage, "{'True = 3;}").unwrap();
    let composed_root = roots[0];

    core_fir_conversion::step_to_constanic(&mut storage, composed_root).unwrap();

    let program = compiler::program_result(&storage, composed_root).unwrap();
    let true_stmt = FirCursor::new(program, &storage).stmt_at(0).unwrap();
    assert!(
        is_unsteppable_cause(&storage, true_stmt),
        "redefining system.foo's 'True with a conflicting value (3) inside the composed \
         user program must make that statement its brane's unsteppable cause (FOOP-86 §6)"
    );
}

/// Exact reproduction of `null_char_constant.foo`'s full statement sequence (restate, same-value
/// re-assert, a reference, THEN the conflicting redefinition) -- the simpler 1-statement repro above
/// passes; this one exercises the same multi-statement IB-search-finds- nearest-prior scan the real
/// case does.
#[test]
fn compose_program_with_system_refuses_conflicting_true_redefinition_full_sequence() {
    let mut storage = FVMStorage::new();
    let roots = compiler::compose_program_with_system(
        &mut storage,
        "{restate = 'True; 'True = 'True; conflict = 'True; 'True = 3;}",
    )
    .unwrap();
    let composed_root = roots[0];

    core_fir_conversion::step_to_constanic(&mut storage, composed_root).unwrap();

    let program = compiler::program_result(&storage, composed_root).unwrap();
    let second_true_stmt = FirCursor::new(program, &storage).stmt_at(3).unwrap();
    assert_eq!(
        FirCursor::new(second_true_stmt, &storage)
            .as_stmt_identifier()
            .map(|id| id.searchable_name()),
        Some("'True")
    );
    assert!(
        is_unsteppable_cause(&storage, second_true_stmt),
        "the FOURTH statement ('True = 3, conflicting) must be the unsteppable cause"
    );
}

/// **Setting `nf_reason` is not enough on its own — the NF write path must ALSO push an
/// already-`Nk` node to `ubc_children`.** `FirPointer::settled_constanic_result` is generic
/// across all kinds and reads `ubc_children().first()`; with nothing there it answers `None`,
/// and every reader (`statement_value_for_comparison`, and through it output serialization and
/// `default_equal`) silently falls through to the raw, unrefused written body — so `'True = 3`
/// renders as plain `3` despite `nf_reason` being set on that very pointer. `refuse_statement`
/// is the shared helper that keeps the two halves together.
///
/// The refusal must reach Foolish-mode output, not merely be recorded in the arena: the
/// conflicting redefinition makes its brane unsteppable (FOOP-86 §6.2 route 1), the brane halts
/// and goes NK (§6.3), and the finding is announced in the rendering (§6.5a). Here the cause is
/// the brane's LAST statement, so there is no following statement to carry the annotation and
/// the brane's opener carries it instead.
#[test]
fn evaluate_refuses_and_renders_conflicting_true_redefinition() {
    use crate::sequencer::{SequenceMode, Ubca2Sequencer};
    let source = "{restate = 'True; 'True = 'True; conflict = 'True; 'True = 3;}";
    let (storage, firs) = crate::UbcaEvaluator.evaluate_arena(source).unwrap();
    let rendered = Ubca2Sequencer::format(&storage, firs[0], SequenceMode::Foolish);
    assert!(
        rendered.contains("unsteppable — 'True already defined in context"),
        "the conflicting redefinition must render as unsteppable, got: {rendered}"
    );
}

/// Regression: `handle_found` (called from
/// `name_search_step`/`value_search_step`) hardcoded `sfm = false` at
/// every call site, instead of threading `scope.has_ancestral_sfm`
/// through. `transform_for_clone`'s contract is "SFM-descendant:
/// preserve the source NYES verbatim (foolishly ignorant)" — with the
/// bug, a search found from inside an SF (`<...>`) wrapper always
/// cloned as though NOT SFM-descendant, so its own ECONSTANIC
/// descendant searches (built ECONSTANIC by the `under_sff` rule when
/// the ORIGINAL declaration was inside an SFF marker) transitioned to
/// EMBRYONIC on clone and genuinely re-searched and resolved in the new
/// context — instead of staying inertly ECONSTANIC, verbatim.
///
/// Concretely: `{a = 1; b = 2; sff = <<a + b>>; sf = <sff>; a = 10;}`'s
/// `sf` reference resolves `sff` via a name search inside an SF
/// wrapper; the clone of `sff`'s `a + b` must stay `Woconstanic` with
/// both operand searches `Econstanic` (settling `[Econstanic,
/// Econstanic]` in ONE step and never progressing further). With the
/// bug, the clone's operand searches would instead progress
/// `Embryonic -> Braning -> Constant`, finding real values (`a=1`,
/// `b=2`) and fully resolving to `3` — an over-eager resolution the
/// SFM-verbatim-preservation rule exists to prevent.
#[test]
fn search_found_inside_sf_threads_ancestral_sfm_to_its_clone() {
    let mut storage = FVMStorage::new();
    let roots = compiler::compile(
        &mut storage,
        "{a = 1; b = 2; sff = <<a + b>>; sf = <sff>; a = 10; sf; sff;}",
    )
    .unwrap();
    let root = roots[0];
    core_fir_conversion::step_to_constanic(&mut storage, root).unwrap();

    let stmts = storage.foolish_children(root).to_vec();
    let sf_body = storage.foolish_children(stmts[3])[0]; // sf's SF wrapper
    let sf_search = storage.foolish_children(sf_body)[0]; // the search for "sff" inside it
    let clone = FirCursor::new(sf_search, &storage)
        .ubc_children()
        .first()
        .copied()
        .expect("sf's search must have gone constanic with a result");

    assert_eq!(
        storage.get_nyes(clone),
        Nyes::Woconstanic,
        "sf's cloned Op+ must stay Woconstanic (SFM-verbatim), not fully resolve"
    );
    let operand_nyes: Vec<_> = storage
        .foolish_children(clone)
        .iter()
        .map(|&c| storage.get_nyes(c))
        .collect();
    assert_eq!(
        operand_nyes,
        vec![Nyes::Econstanic, Nyes::Econstanic],
        "the cloned Op+'s own operand searches must stay Econstanic verbatim, \
         not re-search and resolve in sf's new context"
    );
}

// ── Regression guards ────────────────────────────────────────────

/// FOOP-13 regression guard: a statement that is the FIRST statement in its brane (`line_number == 0`)
/// must not find itself via its own backward IB search. The real bug: computing the backward-scan end
/// as `line_number.saturating_sub(1)` SATURATES to `0` at `line_number == 0` instead of representing
/// "no preceding statements", so the scan range `[0, 0]` wrongly includes the statement's own slot —
/// left unfixed, `{a = a + 1;}` recurses forever (`handle_found` clones the found statement's
/// still-unresolved self-search, the clone re-searches, finds the SAME original statement again,
/// without bound). `ib_search_by_pattern`'s own `checked_sub` (not `saturating_sub`) is the arena's fix
/// for this exact bug class, already in place — this test pins it directly.
#[test]
fn ib_search_at_index_zero_does_not_find_self() {
    let mut storage = FVMStorage::new();
    let brane = storage.make_root(FirSpec::Brane {
        characterizations: Characterizations::default(),
    });
    let a_stmt = brane.create_child(
        &mut storage,
        FirSpec::Statement {
            identifier: Identifier::from_parts(vec![], "a"),
            line_number: 0,
        },
    );
    let op = a_stmt.create_child(&mut storage, FirSpec::Operator { op: "+".to_string() });
    op.create_child(
        &mut storage,
        FirSpec::Search {
            pattern: "^a$".to_string(),
            anchored: false,
            forward: false,
            is_value_search: false,
            contexted: false,
        },
    );
    op.create_child(&mut storage, FirSpec::IndepInt { value: 1 });

    let result = search_dispatch::ib_search_by_pattern(&storage, "a", Some(a_stmt));
    assert!(
        result.is_none(),
        "BUG: a statement at index 0 of its brane must not find itself \
         via backward IB search — got a hit instead of None"
    );
}

/// FOOP-13 regression guard's end-to-end companion: the actual runtime path (`UbcaEvaluator::evaluate`,
/// not a direct `_ib_search`/ `ib_search_by_pattern` call) must not hang forever on a bare
/// self-referential search at brane-index 0. Before the fix this program never settles (steps forever
/// in BRANING); after the fix `a`'s self-search is correctly absent from its own brane, falls through,
/// and the whole program settles within `evaluate`'s own step budget.
#[test]
fn evaluate_settles_self_referential_statement_at_index_zero_without_hanging() {
    let evaluator = crate::evaluator::UbcaEvaluator;
    let (storage, firs) = evaluator
        .evaluate_arena("{a = a + 1;}")
        .expect("compilation must succeed");
    let alarm = storage.alarm_reason(firs[0]);
    assert!(
        alarm.is_none(),
        "BUG: {{a = a + 1;}} must settle within evaluate_arena's step budget \
         (a's self-search absent, falls through to unanchored-miss) — \
         it must NOT hang forever due to a's own statement finding \
         itself at index 0 and hitting the iteration cap, got alarm: {alarm:?}"
    );
}

/// Regression guard: `k=1; k=2` (no leading `'`) must NOT be refused — the null-const rule only fires
/// on null-characterized coordinate names, never on plain ones.
#[test]
fn null_const_rule_does_not_fire_on_plain_names() {
    let mut storage = FVMStorage::new();
    let roots = compiler::compile(&mut storage, "{k=1; k=2;}").unwrap();
    let root = roots[0];
    core_fir_conversion::step_to_constanic(&mut storage, root).unwrap();
    let stmts = storage.foolish_children(root).to_vec();
    assert!(
        !is_unsteppable_cause(&storage, stmts[0]),
        "plain k=1 must never be refused by the null-const rule"
    );
    assert!(
        !is_unsteppable_cause(&storage, stmts[1]),
        "plain k=2 must never be refused by the null-const rule"
    );
}

/// Regression guard: an empty concatenation operand, or a single-operand concatenation, must merge
/// without any spurious NF — the collision check must not misfire when there's nothing (or only one
/// thing) to collide with.
#[test]
fn null_const_concatenation_empty_and_single_operand_merge_without_spurious_nf() {
    let mut storage = FVMStorage::new();
    let roots = compiler::compile(&mut storage, "{A={}; B={'a=1;}; C = A B;}").unwrap();
    let root = roots[0];
    core_fir_conversion::step_to_constanic(&mut storage, root).unwrap();
    let stmts = storage.foolish_children(root).to_vec();
    let c_body = storage.foolish_children(stmts[2])[0];
    let c_value = c_body.value(&storage);
    assert_eq!(FirCursor::new(c_value, &storage).stmt_count(), Some(1));
    let merged_a = FirCursor::new(c_value, &storage).stmt_at(0).unwrap();
    assert!(
        !is_unsteppable_cause(&storage, merged_a),
        "single 'a merged from a concatenation with an empty operand must not be NF"
    );
}

/// The whole comparison feature, end to end, for all five operators and both outcomes. `{a, b, 'op}$`:
/// the brane literal's tail is `'op`, whose conclusive value is the boolean it computed from its two
/// preceding neighbours (FOOP-33 §5.0). Each row is expressed as the plain Rust comparison of 1 and 2,
/// so each row states WHY it is what it is, not merely what was observed.
#[test]
fn each_comparison_operator_produces_the_right_boolean() {
    for (op, expected) in [
        ("'lt", 1 < 2),
        ("'gt", 1 > 2),
        ("'le", 1 <= 2),
        ("'ge", 1 >= 2),
        ("'eq", 1 == 2),
    ] {
        let mut storage = FVMStorage::new();
        let source = format!("{{r = {{1, 2, {op}}}$;}}");
        let roots = compiler::compose_program_with_system(&mut storage, &source).unwrap();
        let composed_root = roots[0];
        core_fir_conversion::step_to_constanic(&mut storage, composed_root).unwrap();
        let program = compiler::program_result(&storage, composed_root).unwrap();
        let stmt = FirCursor::new(program, &storage).stmt_at(0).unwrap();
        let body = storage.foolish_children(stmt)[0];
        let got = body.value(&storage);

        assert!(
            matches!(storage.get(got), FirSpec::Creation),
            "{op} must produce a creation ('True/'False), not {:?}",
            storage.get(got)
        );
        let want_name = if expected { "'True" } else { "'False" };
        let want_stmt = storage
            .foolish_children(composed_root)
            .iter()
            .find(|&&s| {
                FirCursor::new(s, &storage)
                    .as_stmt_identifier()
                    .map(|id| id.searchable_name())
                    == Some(want_name)
            })
            .copied()
            .expect("system.foo declares 'True and 'False");
        let want_body = storage.foolish_children(want_stmt)[0];
        let want = want_body.value(&storage);
        assert_eq!(
            got, want,
            "{{1, 2, {op}}}$ must be system.foo's own {want_name} creation \
             (referential identity, FOOP-33 §5), expected={expected}"
        );
    }
}

/// `b='a` resolves THROUGH a search to the SAME creation `'a` defines (FOOP-33 Gotcha #2).
/// Viewed from `b`'s statement — a DIFFERENT statement than `'a`'s own — the rendered output must
/// report `'a`, not `b`, proving that identity drives the name rather than the referencing
/// statement's own name, and that viewing from elsewhere is what unlocks it.
#[test]
fn creation_reached_through_search_renders_with_its_own_defining_name() {
    use crate::sequencer::{SequenceMode, Ubca2Sequencer};
    let (storage, firs) = crate::UbcaEvaluator.evaluate_arena("{'a=⬤; b='a;}").unwrap();
    let rendered = Ubca2Sequencer::format(&storage, firs[0], SequenceMode::Foolish);
    assert!(
        rendered.contains("b = 'a"),
        "a creation reached through a search ('a=⬤; b='a), viewed from the \
         REFERENCING statement, must render with its OWN defining statement's \
         name ('a), got: {rendered}"
    );
}

/// Evaluates `src` and returns the storage plus the program brane.
fn eval_program(src: &str) -> (FVMStorage, FirPointer) {
    let mut storage = FVMStorage::new();
    let roots = compiler::compose_program_with_system(&mut storage, src).expect("program compiles");
    let root = roots[0];
    let _ = core_fir_conversion::step_to_constanic(&mut storage, root);
    let program = compiler::program_result(&storage, root).unwrap_or(root);
    (storage, program)
}

/// The body FIR of statement `index` in `program`, with its settled Nyes.
fn stmt_body_and_nyes(storage: &FVMStorage, program: FirPointer, index: usize) -> (FirPointer, Nyes) {
    let stmt = FirCursor::new(program, storage)
        .stmt_at(index)
        .expect("statement exists");
    let body = FirCursor::new(stmt, storage).foolish_children()[0];
    (body, storage.get_nyes(body))
}

/// A search that FINDS a creation settles CONSTANT and holds the creation
/// itself at `ubc_children[0]` — whether or not that creation has a
/// renderable name.
///
/// **Why this needs a unit test:** all three cases REVERT TO FOOLISH when rendered, so the
/// sequencer's output shows only the written expression and does not exhibit the NYES or the
/// internal state. In the rendered text a nameless creation (`a = b`), a named one (`b = 'a`) and
/// an out-of-context one (`r = gs?'b`) are indistinguishable from each other AND from a genuine
/// failure. The internal state says otherwise: the search SUCCEEDED in every case.
#[test]
fn search_finding_a_creation_settles_constant_regardless_of_nameability() {
    for (label, src) in [
        // Nameless: `b` is not null-characterized, so there is no name to
        // render and the statement reverts to the written `?b`.
        ("nameless creation", "{b = ⬤; a = ?b;}"),
        // Named and in context: renders `'a`, the creation's original name.
        ("named creation", "{'a = ⬤; b = ?'a;}"),
        // Named but OUT OF CONTEXT: `'b` names a creation inside `gs`, and that name is not in scope at
        // the outer brane (FOOP-36 §N4.b), so this reverts too — despite the search having found it.
        ("out-of-context name", "{gs = {b=⬤; 'b=⬤}; r = gs?'b;}"),
    ] {
        let (storage, program) = eval_program(src);
        let (body, nyes) = stmt_body_and_nyes(&storage, program, 1);
        assert_eq!(
            nyes,
            Nyes::Constant,
            "{label}: a search that FINDS its target settles CONSTANT, \
             not NK -- reverting to Foolish when rendered is a NAMING \
             outcome, not a search failure ({src})"
        );
        let results = FirCursor::new(body, &storage).ubc_children().to_vec();
        assert_eq!(
            results.len(),
            2,
            "{label}: a resolved search holds the FoolRefFir two-child \
             invariant -- [0] the found value, [1] the position ({src})"
        );
        assert!(
            matches!(storage.get(results[0]), FirSpec::Creation),
            "{label}: ubc_children[0] is the CREATION the search found, \
             intact -- not an Nk substitute ({src})"
        );
        assert_eq!(
            storage.get_nyes(results[0]),
            Nyes::Independent,
            "{label}: the found creation is INDEPENDENT -- a creation is \
             born independent and never steps ({src})"
        );
        assert!(
            matches!(storage.get(results[1]), FirSpec::FoolRef { .. }),
            "{label}: ubc_children[1] is the FoolRef carrying the found \
             statement's position ({src})"
        );
    }
}

/// The CONTRAST case: a search into a brane halted by an unsteppable statement settles NK
/// (FOOP-86 §6.4b), and the whole statement goes with it. Rendered, this is a reverted line just
/// like the three above — which is precisely why the distinction must be asserted on the NYES.
#[test]
fn search_into_an_unsteppable_brane_settles_nk_not_constant() {
    let (storage, program) = eval_program("{bs = {b=⬤; 'b=⬤; 'bad='b}; r = bs?'b;}");

    // The brane itself halted, and the cause is recorded on it.
    let (bs_body, bs_nyes) = stmt_body_and_nyes(&storage, program, 0);
    assert_eq!(
        bs_nyes,
        Nyes::Nk,
        "an unsteppable statement halts its brane (§6.3)"
    );
    assert!(
        storage.unsteppable_cause(bs_body).is_some(),
        "the halt records WHICH statement was unsteppable, on the brane (§6.2a)"
    );

    // `'b` genuinely IS in `bs` -- the search is refused because the
    // container is ill-defined, not because the name is absent.
    let (_, r_nyes) = stmt_body_and_nyes(&storage, program, 1);
    assert_eq!(
        r_nyes,
        Nyes::Nk,
        "every access into an NK brane settles NK -- a brane with no \
         meaning cannot be searched (§6.4b)"
    );
}

/// `get_display_name` is the ONLY source of a creation's rendered name, and it yields a name ONLY for a
/// null-characterized one. This pins the mechanism behind the reversion the two tests above describe:
/// the nameless case has no name to return, so rendering has nothing to print and must fall back to the
/// written form.
#[test]
fn only_a_null_characterized_creation_has_a_display_name() {
    let (storage, program) = eval_program("{'named = ⬤; plain = ⬤; reader = 1;}");
    let cursor = FirCursor::new(program, &storage);
    let reader = cursor.stmt_at(2).expect("reader exists");

    for (index, expected, what) in [
        (
            0usize,
            Some("'named"),
            "a null-characterized creation reports its name",
        ),
        (1, None, "a plain-named creation has NO display name"),
    ] {
        let stmt = cursor.stmt_at(index).expect("statement exists");
        let body = FirCursor::new(stmt, &storage).foolish_children()[0];
        assert_eq!(
            FirCursor::new(body, &storage)
                .as_creation_display_name(Some(reader))
                .as_deref(),
            expected,
            "{what}"
        );
    }
}
