//! AST→FIR construction — the compiler `UbcaEvaluator::evaluate` drives (via
//! `compose_program_with_system`/`program_result`, re-exported at this file's top level).
use super::{
    ANON_STMT_NAME, ConcatProvenance, ConcatRenderingAid, FVMStorage, FirCursor, FirCursorMut, FirPointer,
    FirSpec, StayMarker,
};

use foolish_core::fir::Nyes;
use foolish_parser::{AssignmentOperator, Astn, SearchOperator};

use crate::identifier::{Characterizations, Identifier};

/// Element types allowed inside a ConcatBrane.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConcatElemKind {
    BareBrane,
    BareConcat,
    BareSearch,
    SfSearch,
    SfBrane,
    Error,
}

/// Rejects AST shapes this crate doesn't support (before any FIR construction begins) — a plain,
/// storage-independent AST walk with no `FirPointer` involvement.
fn validate_astn(ast: &Astn) -> anyhow::Result<()> {
    match ast {
        Astn::IfExpr { .. } => anyhow::bail!("if-then-else: not supported (FOOP=2)"),
        Astn::UpwardSearch => anyhow::bail!("Upward search: deferred"),
        Astn::DetachmentBrane { .. } => anyhow::bail!("Detachment brane: deferred"),
        Astn::NotImplemented(r) => anyhow::bail!("Not yet implemented: {}", r),
        Astn::Brane { statements, .. } => {
            for s in statements {
                validate_astn(s)?;
            }
            Ok(())
        }
        Astn::Assignment { expr, .. } => validate_astn(expr),
        Astn::BinaryOp { left, right, .. } => {
            validate_astn(left)?;
            validate_astn(right)
        }
        Astn::UnaryOp { expr, .. } => validate_astn(expr),
        Astn::DotSearch { anchor, .. } => validate_astn(anchor),
        Astn::RegexpSearch { anchor, .. } => {
            if let Some(a) = anchor {
                validate_astn(a)?;
            }
            Ok(())
        }
        Astn::ValueSearch {
            anchor,
            value_pattern,
            ..
        } => {
            if let Some(a) = anchor {
                validate_astn(a)?;
            }
            validate_astn(value_pattern)
        }
        Astn::Seek { anchor, .. } => validate_astn(anchor),
        Astn::HeadTail { anchor, .. } => validate_astn(anchor),
        Astn::ContextedSearch { inner } => validate_astn(inner),
        Astn::Concatenation { elements } => {
            for e in elements {
                validate_astn(e)?;
            }
            Ok(())
        }
        Astn::TailConcatenation { elements } => {
            for e in elements {
                validate_astn(e)?;
            }
            Ok(())
        }
        Astn::StayFoolish { expr } => validate_astn(expr),
        Astn::StayFullyFoolish { expr } => validate_astn(expr),
        Astn::IntLit(_)
        | Astn::UnknownLit
        | Astn::Creation
        | Astn::Identifier { .. }
        | Astn::UnanchoredSeek { .. } => Ok(()),
    }
}

/// Byte-for-byte copy of `compiler.rs`'s real (private) `classify_concat_element` — plain AST
/// classification, no `FirPointer` involvement, duplicated for the same reason as
/// `ConcatElemKind`/`validate_astn` above.
fn classify_concat_element(ast: &Astn) -> ConcatElemKind {
    match ast {
        Astn::Brane { .. } => ConcatElemKind::BareBrane,
        Astn::Concatenation { .. } => ConcatElemKind::BareConcat,
        Astn::Identifier { .. }
        | Astn::DotSearch { .. }
        | Astn::RegexpSearch { .. }
        | Astn::Seek { .. }
        | Astn::HeadTail { .. }
        | Astn::UnanchoredSeek { .. }
        | Astn::ValueSearch { .. } => ConcatElemKind::BareSearch,
        Astn::ContextedSearch { inner } => {
            if matches!(
                inner.as_ref(),
                Astn::Identifier { .. }
                    | Astn::DotSearch { .. }
                    | Astn::RegexpSearch { .. }
                    | Astn::Seek { .. }
                    | Astn::HeadTail { .. }
                    | Astn::UnanchoredSeek { .. }
                    | Astn::ValueSearch { .. }
            ) {
                ConcatElemKind::BareSearch
            } else {
                ConcatElemKind::Error
            }
        }
        Astn::StayFoolish { expr } => match expr.as_ref() {
            Astn::Brane { .. } => ConcatElemKind::SfBrane,
            Astn::Identifier { .. }
            | Astn::DotSearch { .. }
            | Astn::RegexpSearch { .. }
            | Astn::Seek { .. }
            | Astn::HeadTail { .. }
            | Astn::UnanchoredSeek { .. }
            | Astn::ValueSearch { .. }
            | Astn::ContextedSearch { .. } => ConcatElemKind::SfSearch,
            _ => ConcatElemKind::Error,
        },
        Astn::StayFullyFoolish { expr } => match expr.as_ref() {
            Astn::Brane { .. } => ConcatElemKind::SfBrane,
            Astn::Identifier { .. }
            | Astn::DotSearch { .. }
            | Astn::RegexpSearch { .. }
            | Astn::Seek { .. }
            | Astn::HeadTail { .. }
            | Astn::UnanchoredSeek { .. }
            | Astn::ValueSearch { .. }
            | Astn::ContextedSearch { .. } => ConcatElemKind::SfSearch,
            _ => ConcatElemKind::Error,
        },
        _ => ConcatElemKind::Error,
    }
}

/// `parent` is the ALREADY-CREATED arena parent (the `Concatenation`/`ConcatHelper` node), so each
/// wrapper here is one `create_child` call. Returns the marker this element WROTE in source, if any —
/// `None` when the wrapper was synthesized here rather than written. Recorded into the concatenation's
/// [`ConcatRenderingAid`] so the renderer can put back exactly the markers the Foolisher typed.
fn build_concat_element(
    storage: &mut FVMStorage,
    ast: Astn,
    parent: FirPointer,
    under_sff: bool,
) -> Option<StayMarker> {
    let written = match &ast {
        Astn::StayFoolish { .. } => Some(StayMarker::Sf),
        Astn::StayFullyFoolish { .. } => Some(StayMarker::Sff),
        _ => None,
    };
    match classify_concat_element(&ast) {
        ConcatElemKind::BareBrane => {
            build_fir(storage, ast, Some(parent), true);
        }
        ConcatElemKind::BareConcat => {
            build_fir(storage, ast, Some(parent), under_sff);
        }
        ConcatElemKind::BareSearch => {
            let sf = parent.create_child(storage, FirSpec::StayFoolish);
            build_fir(storage, ast, Some(sf), under_sff);
        }
        ConcatElemKind::SfSearch => {
            build_fir(storage, ast, Some(parent), under_sff);
        }
        ConcatElemKind::SfBrane => {
            let sff = parent.create_child(storage, FirSpec::StayFullyFoolish);
            build_fir(storage, ast, Some(sff), false);
        }
        ConcatElemKind::Error => {
            parent.create_child(
                storage,
                FirSpec::Nk {
                    reason: "invalid concatenation element".to_string(),
                },
            );
        }
    }
    written
}

/// Writes the collected aid onto an already-built concatenation node.
fn set_concat_rendering_aid(storage: &mut FVMStorage, node: FirPointer, aid: ConcatRenderingAid) {
    if aid.is_empty() {
        return;
    }
    storage.with_mut(node, |fir| fir.set_concat_rendering_aid(aid));
}

/// `parent: None` means build a ROOT (self-parented via `FVMStorage::make_root`); `Some(p)` means a
/// child of `p` (via `create_child`). Every arm that recurses builds its OWN node FIRST —
/// `create_child`/`make_root` need no placeholder-then-mutate step, since they need only the node's
/// final field values (tree structure is handled generically by the arena itself) — so the order is:
/// construct this node, getting its `FirPointer` immediately, THEN build children as its
/// `create_child`s.
fn build_fir(
    storage: &mut FVMStorage,
    ast: Astn,
    parent: Option<FirPointer>,
    under_sff: bool,
) -> FirPointer {
    let search_nyes = if under_sff {
        Nyes::Econstanic
    } else {
        Nyes::Prembrionic
    };
    macro_rules! child_parent {
        () => {
            parent.expect("non-Brane FIR must have a parent — only a Brane can be root")
        };
    }
    match ast {
        Astn::IntLit(n) => child_parent!().create_child(storage, FirSpec::IndepInt { value: n as i64 }),
        Astn::UnknownLit => child_parent!().create_child(
            storage,
            FirSpec::Nk {
                reason: "??? literal".to_string(),
            },
        ),
        Astn::Creation => child_parent!().create_child(storage, FirSpec::Creation),
        Astn::Identifier {
            characterizations,
            id,
        } => {
            // Fold characterizations back into the search pattern (Gotcha #3).
            let full_pattern = if characterizations.is_empty() {
                id.clone()
            } else {
                let char_str: String = characterizations.iter().map(|c| format!("{c}'")).collect();
                format!("{char_str}{id}")
            };
            let node = child_parent!().create_child(
                storage,
                FirSpec::Search {
                    pattern: format!("^{full_pattern}$"),
                    anchored: false,
                    forward: false,
                    is_value_search: false,
                    contexted: false,
                },
            );
            storage.with_mut(node, |fir| fir.set_nyes(search_nyes));
            node
        }
        Astn::Brane {
            characterizations,
            statements,
        } => {
            let brane = match parent {
                Some(p) => p.create_child(
                    storage,
                    FirSpec::Brane {
                        characterizations: Characterizations::from_brane_parts(characterizations),
                    },
                ),
                None => storage.make_root(FirSpec::Brane {
                    characterizations: Characterizations::from_brane_parts(characterizations),
                }),
            };
            build_stmts(storage, statements, brane, under_sff);
            brane
        }
        Astn::BinaryOp { op, left, right } => {
            let node = child_parent!().create_child(storage, FirSpec::Operator { op });
            build_fir(storage, *left, Some(node), under_sff);
            build_fir(storage, *right, Some(node), under_sff);
            node
        }
        Astn::UnaryOp { op, expr } => {
            let node = child_parent!().create_child(storage, FirSpec::Operator { op });
            build_fir(storage, *expr, Some(node), under_sff);
            node
        }
        Astn::DotSearch { anchor, coordinate } => {
            let node = child_parent!().create_child(
                storage,
                FirSpec::Search {
                    pattern: format!("^{coordinate}$"),
                    anchored: true,
                    forward: false,
                    is_value_search: false,
                    contexted: false,
                },
            );
            storage.with_mut(node, |fir| fir.set_nyes(search_nyes));
            build_fir(storage, *anchor, Some(node), under_sff);
            node
        }
        Astn::RegexpSearch {
            anchor,
            pattern,
            operator,
            ..
        } => {
            let has_anchor = anchor.is_some();
            let node = child_parent!().create_child(
                storage,
                FirSpec::Search {
                    pattern,
                    anchored: has_anchor,
                    forward: operator == SearchOperator::RegexpForward,
                    is_value_search: false,
                    contexted: false,
                },
            );
            storage.with_mut(node, |fir| fir.set_nyes(search_nyes));
            if let Some(a) = anchor {
                build_fir(storage, *a, Some(node), under_sff);
            }
            node
        }
        Astn::ValueSearch {
            anchor,
            forward,
            name_pattern,
            value_pattern,
        } => {
            let has_anchor = anchor.is_some();
            let pattern = name_pattern.unwrap_or_default();
            let node = child_parent!().create_child(
                storage,
                FirSpec::Search {
                    pattern,
                    anchored: has_anchor,
                    forward,
                    is_value_search: true,
                    contexted: false,
                },
            );
            storage.with_mut(node, |fir| fir.set_nyes(search_nyes));
            if let Some(a) = anchor {
                build_fir(storage, *a, Some(node), under_sff);
            }
            build_fir(storage, *value_pattern, Some(node), under_sff);
            node
        }
        Astn::Seek { anchor, offset } => {
            let node = child_parent!().create_child(
                storage,
                FirSpec::Index {
                    offset,
                    anchored: true,
                    contexted: false,
                },
            );
            storage.with_mut(node, |fir| fir.set_nyes(search_nyes));
            build_fir(storage, *anchor, Some(node), under_sff);
            node
        }
        Astn::HeadTail { is_head, anchor } => {
            let offset = if is_head { 0 } else { -1 };
            let node = child_parent!().create_child(
                storage,
                FirSpec::Index {
                    offset,
                    anchored: true,
                    contexted: false,
                },
            );
            storage.with_mut(node, |fir| fir.set_nyes(search_nyes));
            build_fir(storage, *anchor, Some(node), under_sff);
            node
        }
        Astn::UnanchoredSeek { offset } => {
            let node = child_parent!().create_child(
                storage,
                FirSpec::Index {
                    offset,
                    anchored: false,
                    contexted: false,
                },
            );
            storage.with_mut(node, |fir| fir.set_nyes(search_nyes));
            node
        }
        Astn::Concatenation { elements } => {
            let node = child_parent!().create_child(
                storage,
                FirSpec::Concatenation {
                    provenance: ConcatProvenance::Juxtaposition,
                    rendering_aid: ConcatRenderingAid::default(),
                },
            );
            let mut aid = ConcatRenderingAid::default();
            for (i, e) in elements.into_iter().enumerate() {
                if let Some(marker) = build_concat_element(storage, e, node, under_sff) {
                    aid.record(i, marker);
                }
            }
            set_concat_rendering_aid(storage, node, aid);
            node
        }
        Astn::TailConcatenation { elements } => {
            let node = child_parent!().create_child(
                storage,
                FirSpec::Concatenation {
                    provenance: ConcatProvenance::TailConcatenation,
                    rendering_aid: ConcatRenderingAid::default(),
                },
            );
            // Elements are stored REVERSED (FOOP-65 §5.2), so the index recorded here is the STORED
            // index, matching what the renderer walks.
            let mut aid = ConcatRenderingAid::default();
            for (i, e) in elements.into_iter().rev().enumerate() {
                if let Some(marker) = build_concat_element(storage, e, node, under_sff) {
                    aid.record(i, marker);
                }
            }
            set_concat_rendering_aid(storage, node, aid);
            node
        }
        Astn::StayFoolish { expr } => {
            let node = child_parent!().create_child(storage, FirSpec::StayFoolish);
            build_fir(storage, *expr, Some(node), under_sff);
            node
        }
        Astn::StayFullyFoolish { expr } => {
            let node = child_parent!().create_child(storage, FirSpec::StayFullyFoolish);
            // SFF marker: from here down, searches are built ECONSTANIC.
            let e = build_fir(storage, *expr, Some(node), true);
            // Sanity-check that `under_sff` actually reached every descendant search — mirrors the real
            // `push_foolish_child_sff_marked` call exactly (the arena's `create_child` above already
            // did the "push" half; this is purely the invariant CHECK, run after the fact since the
            // arena wires parent/child atomically at construction).
            let cursor = FirCursorMut::new(node, storage);
            cursor.check_sff_marked_child(e);
            node
        }
        Astn::ContextedSearch { inner } => {
            let node = build_fir(storage, *inner, parent, under_sff);
            storage.with_mut(node, |fir| fir.set_contexted(true));
            node
        }
        Astn::Assignment { .. } => {
            unreachable!("standalone Assignment should be wrapped in Brane by parser")
        }
        _ => unreachable!("validate_astn should have rejected this"),
    }
}

/// Direct translation of `compiler.rs`'s real `build_stmts`.
fn build_stmts(storage: &mut FVMStorage, asts: Vec<Astn>, parent: FirPointer, under_sff: bool) {
    for (i, ast) in asts.into_iter().enumerate() {
        build_as_statement(storage, ast, parent, i, under_sff);
    }
}

/// Direct translation of `compiler.rs`'s real `AstnCompilerExt:: build_as_statement_inner` (the shared
/// body behind `build_as_statement`/`build_as_statement_overridden`; the `override_body` hook itself —
/// `system_foo.rs`'s comparison-operator injection — is NOT translated here, since `system_foo.rs`'s
/// own arena migration is out of this task's scope; only the ordinary, unoverridden path is
/// implemented).
fn build_as_statement(
    storage: &mut FVMStorage,
    ast: Astn,
    parent: FirPointer,
    line: usize,
    under_sff: bool,
) -> FirPointer {
    let (characterizations, name, expr, operator) = match ast {
        Astn::Assignment {
            characterizations,
            identifier,
            operator,
            expr,
        } => (characterizations, identifier, *expr, operator),
        other => (
            vec![],
            ANON_STMT_NAME.to_string(),
            other,
            AssignmentOperator::Assign,
        ),
    };
    let identifier = Identifier::from_parts(characterizations, &name);
    let stmt = parent.create_child(
        storage,
        FirSpec::Statement {
            identifier,
            line_number: line,
        },
    );
    build_expr_with_operator(storage, expr, operator, stmt, under_sff);
    stmt
}

/// Direct translation of `compiler.rs`'s real `AstnCompilerExt:: build_expr_with_operator`.
fn build_expr_with_operator(
    storage: &mut FVMStorage,
    ast: Astn,
    operator: AssignmentOperator,
    parent: FirPointer,
    under_sff: bool,
) -> FirPointer {
    let body_under_sff = under_sff || operator == AssignmentOperator::SFF;
    match operator {
        AssignmentOperator::Assign => build_fir(storage, ast, Some(parent), body_under_sff),
        AssignmentOperator::SF => {
            let sf = parent.create_child(storage, FirSpec::StayFoolish);
            build_fir(storage, ast, Some(sf), body_under_sff);
            sf
        }
        AssignmentOperator::SFF => {
            let sff = parent.create_child(storage, FirSpec::StayFullyFoolish);
            build_fir(storage, ast, Some(sff), body_under_sff);
            sff
        }
    }
}

/// A plain, no-system.foo compile path.
///
/// No production caller: `UbcaEvaluator::evaluate`'s real body goes
/// through `compose_program_with_system`/`compose_one`/
/// `compile_root_with_body_override` instead — system.foo composition
/// is not opt-in (FOOP-33 §4). Kept and exercised by this file's own
/// tests.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "no production caller — evaluate goes through compose_program_with_system, \
                  not this plain compile path; exercised only by this file's own tests"
    )
)]
pub(crate) fn compile_standalone(storage: &mut FVMStorage, ast: Astn) -> anyhow::Result<FirPointer> {
    validate_astn(&ast)?;
    if !matches!(ast, Astn::Brane { .. }) {
        anyhow::bail!("only a Brane can be a top-level (root) node");
    }
    Ok(build_fir(storage, ast, None, false))
}

/// No production caller — see `compile_standalone`'s doc comment.
#[cfg_attr(not(test), expect(dead_code, reason = "see compile_standalone's doc comment"))]
pub(crate) fn compile(storage: &mut FVMStorage, source: &str) -> anyhow::Result<Vec<FirPointer>> {
    let asts = foolish_parser::parse(source)?;
    asts.into_iter()
        .map(|ast| compile_standalone(storage, ast))
        .collect()
}

/// Parses `source`, requires it to be exactly one top-level brane with exactly one (assignment)
/// statement, then builds ONLY that statement's body under `parent` via `build_expr_with_operator` —
/// never wrapping it in a `Statement`/`Brane` of its own. Used by `system_foo`'s comparison-operator
/// installer to compile each fixed `OPERAND_SRC` fragment (`"{o = <<#-2>>;}"`) directly beneath the
/// `ComparisonFir` node.
pub(crate) fn compile_stmt_body_under(
    storage: &mut FVMStorage,
    source: &str,
    parent: FirPointer,
) -> anyhow::Result<FirPointer> {
    let asts = foolish_parser::parse(source)?;
    let [ast] = <[Astn; 1]>::try_from(asts)
        .map_err(|v| anyhow::anyhow!("expected exactly one top-level brane, found {}", v.len()))?;
    validate_astn(&ast)?;
    let Astn::Brane { mut statements, .. } = ast else {
        anyhow::bail!("expected a brane");
    };
    if statements.len() != 1 {
        anyhow::bail!("expected exactly one statement, found {}", statements.len());
    }
    let Astn::Assignment { expr, operator, .. } = statements.remove(0) else {
        anyhow::bail!("expected an assignment");
    };
    Ok(build_expr_with_operator(storage, *expr, operator, parent, false))
}

/// A body-override hook: takes `&mut FVMStorage` (needed to construct a replacement body) and the
/// STATEMENT's own `FirPointer`. Returns `Some(body)` to supply that body INSTEAD of the ordinary
/// compiled one, or `None` to fall through to normal construction.
pub(crate) type ArenaBodyOverride<'a> =
    &'a dyn Fn(&Identifier, &mut FVMStorage, FirPointer) -> Option<FirPointer>;

/// Builds ONE statement, consulting `override_body` first (by the statement's OWN identifier) before
/// falling through to the ordinary `build_expr_with_operator` path `build_as_statement` uses. Kept as
/// its own function since `override_body` is threaded ONLY at the top level of
/// `compile_root_with_body_override`'s own statement loop — system.foo's own top-level statements are
/// the only call site in this module (nested branes, concatenation elements, etc.) needs this this
/// crate that ever needs this override parameter at all.
fn build_as_statement_overridden(
    storage: &mut FVMStorage,
    ast: Astn,
    parent: FirPointer,
    line: usize,
    override_body: ArenaBodyOverride<'_>,
) -> FirPointer {
    let (characterizations, name, expr, operator) = match ast {
        Astn::Assignment {
            characterizations,
            identifier,
            operator,
            expr,
        } => (characterizations, identifier, *expr, operator),
        other => (
            vec![],
            ANON_STMT_NAME.to_string(),
            other,
            AssignmentOperator::Assign,
        ),
    };
    let identifier = Identifier::from_parts(characterizations, &name);
    let stmt = parent.create_child(
        storage,
        FirSpec::Statement {
            identifier: identifier.clone(),
            line_number: line,
        },
    );
    match override_body(&identifier, storage, stmt) {
        Some(_body) => {
            // The override already built its replacement body AS a child of `stmt` (matching this
            // arena's strictly-top-down construction discipline — see `compile_stmt_body_under`'s own
            // `parent` parameter). Nothing more to do here.
        }
        None => {
            build_expr_with_operator(storage, expr, operator, stmt, false);
        }
    }
    stmt
}

/// Arena counterpart to `compiler.rs`'s real `compile_root_with_body_ override`: compile a top-level
/// brane AST as a self-rooting root, letting `override_body` replace individual statements' bodies.
/// Identical to `compile_standalone` except for the per-statement hook.
pub(crate) fn compile_root_with_body_override(
    storage: &mut FVMStorage,
    ast: Astn,
    override_body: ArenaBodyOverride<'_>,
) -> anyhow::Result<FirPointer> {
    validate_astn(&ast)?;
    let Astn::Brane {
        characterizations,
        statements,
    } = ast
    else {
        anyhow::bail!("only a Brane can be a top-level (root) node");
    };
    let root = storage.make_root(FirSpec::Brane {
        characterizations: Characterizations::from_brane_parts(characterizations),
    });
    for (i, stmt_ast) in statements.into_iter().enumerate() {
        build_as_statement_overridden(storage, stmt_ast, root, i, override_body);
    }
    Ok(root)
}

/// Builds a `FirSpec::Comparison` node with its two SFF-marked operand lookups, compiled from
/// `system_foo::OPERAND_SRC`'s fixed Foolish source via `compile_stmt_body_under`. The operands are
/// compiled from source, not hand-built, specifically so `build_fir`'s `under_sff` rule applies to them
/// exactly like any other SFF-marked expression — no separate panic-guard is needed beyond the ordinary
/// `under_sff` propagation through `build_fir`/`build_expr_with_operator`.
pub(crate) fn build_comparison(
    storage: &mut FVMStorage,
    op: crate::system_foo::ComparisonOp,
    parent: FirPointer,
) -> FirPointer {
    let cmp = parent.create_child(storage, FirSpec::Comparison { op });
    for src in crate::system_foo::OPERAND_SRC {
        compile_stmt_body_under(storage, src, cmp)
            .expect("OPERAND_SRC is a fixed, valid Foolish expression");
    }
    cmp
}

/// Supplies a `Comparison`-shaped body for each comparison operator's `system.foo` statement, matched
/// by the statement's OWN null-characterized searchable name against `ComparisonOp::ALL`. Returns
/// `None` (fall through to ordinary construction) for every other statement — this hook runs ONLY over
/// `system.foo`'s own top-level statements, never over user source.
pub(crate) fn comparison_body(
    identifier: &Identifier,
    storage: &mut FVMStorage,
    stmt: FirPointer,
) -> Option<FirPointer> {
    let name = identifier.searchable_name();
    let op = crate::system_foo::ComparisonOp::from_searchable_name(name)?;
    Some(build_comparison(storage, op, stmt))
}

/// Composes `system.foo` with a single user program's AST, appended as a statement named `program`
/// (last), and compiles the combined AST as one self-rooting brane via
/// `compile_root_with_body_override` with `comparison_body` as the hook.
pub(crate) fn compose_one(
    storage: &mut FVMStorage,
    system_ast: Astn,
    program_ast: Astn,
) -> anyhow::Result<FirPointer> {
    let Astn::Brane {
        characterizations,
        mut statements,
    } = system_ast
    else {
        anyhow::bail!("system.foo must parse to exactly one top-level brane, found 0");
    };
    statements.push(Astn::Assignment {
        characterizations: vec![],
        identifier: "program".to_string(),
        operator: AssignmentOperator::Assign,
        expr: Box::new(program_ast),
    });
    let composed = Astn::Brane {
        characterizations,
        statements,
    };
    compile_root_with_body_override(storage, composed, &comparison_body)
}

/// Parses `system.foo` and the user's source, composing each of the
/// user's top-level items with `system.foo` per [`compose_one`].
pub(crate) fn compose_program_with_system(
    storage: &mut FVMStorage,
    user_source: &str,
) -> anyhow::Result<Vec<FirPointer>> {
    let program_asts = foolish_parser::parse(user_source)?;
    program_asts
        .into_iter()
        .map(|program_ast| {
            let system_asts = foolish_parser::parse(crate::system_foo::SYSTEM_FOO_SRC)?;
            let [system_ast] = <[Astn; 1]>::try_from(system_asts).map_err(|v| {
                anyhow::anyhow!(
                    "system.foo must parse to exactly one top-level brane, found {}",
                    v.len()
                )
            })?;
            compose_one(storage, system_ast, program_ast)
        })
        .collect()
}

/// Extracts the `program` member's VALUE from a composed root — the LAST statement of the composite
/// brane (FOOP-33 §4). Structural access (`stmt_count`/`stmt_at`), never a Foolish search. `.value()`
/// on the STATEMENT itself would just return the statement (a plain `Statement` has no constanic result
/// in the common case), so this resolves through `foolish_children().first()` (the written body) first,
/// THEN `.value()`.
pub(crate) fn program_result(storage: &FVMStorage, composed_root: FirPointer) -> Option<FirPointer> {
    let count = FirCursor::new(composed_root, storage).stmt_count()?;
    if count == 0 {
        return None;
    }
    let last_stmt = FirCursor::new(composed_root, storage).stmt_at(count - 1)?;
    let body = storage.foolish_children(last_stmt).first().copied()?;
    Some(body.value(storage))
}
