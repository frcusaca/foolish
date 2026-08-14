use std::cell::RefCell;
use std::rc::{Rc, Weak};

use anyhow::anyhow;
use foolish_parser::{AssignmentOperator, Astn, SearchOperator};

use crate::fir_kinds::{
    BraneFir, ConcatenationFir, CreationFir, IndepIntFir, IndexFir, NkFir, OperatorFir, SearchFir,
    StatementFir, StayFoolishFir, StayFullyFoolishFir,
};
use crate::fir_trait::{Fir, FirRef};
use crate::proto_brane::ProtoBrane;
use foolish_core::fir::Nyes;

/// Element types allowed inside a ConcatBrane.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConcatElemKind {
    /// Bare brane literal — auto-wrap in SFF (under_sff = true).
    BareBrane,
    /// Bare search — auto-wrap in SF (rule 3).
    BareSearch,
    /// Already SF- or SFF-marked at the top, or constantew — build AS WRITTEN,
    /// adding nothing (rules 1 and 2).
    AsWritten,
    /// Not a valid concatenation element — error, NK at construction (rule 5).
    Error,
}

/// Classify a concatenation element per FOOP-55 §9's five rules, **tested in
/// this order**:
///
/// 1. already SF/SFF-marked at the top → [`ConcatElemKind::AsWritten`]
/// 2. constantew (constant everywhere) → [`ConcatElemKind::AsWritten`]
/// 3. any search → [`ConcatElemKind::BareSearch`] (auto-SF)
/// 4. any brane-like → [`ConcatElemKind::BareBrane`] (auto-SFF)
/// 5. otherwise → [`ConcatElemKind::Error`]
///
/// **Order matters, and rule 1 comes first by specification** — not as a fix for
/// any observed defect. FOOP-55 §9.2 says a constituent the Foolisher has
/// already marked is compiled **as written**, adding nothing; testing "is it
/// already marked?" before inspecting what the mark contains is what makes that
/// literally true. Matching `StayFoolish`/`StayFullyFoolish` in arms that look
/// inside the mark would re-wrap, adding a second mark the Foolisher did not
/// write.
///
/// An earlier revision of this comment justified the ordering by claiming
/// `<<{…}>>` had been "silently downgraded to SF semantics". **That claim was
/// measured and is false** — see FOOP-55 §9.4(c). The ordering is right; the
/// rationale was not.
///
/// # Requirements
///
/// The caller must not pass an AST node that cannot appear as a concatenation
/// element. Rule 5 returns [`ConcatElemKind::Error`] for those, and
/// `build_concat_element` turns it into an NK at construction with a reason —
/// it is a malformed program, not an interpreter fault.
fn classify_concat_element(ast: &Astn) -> ConcatElemKind {
    // Rule 1: already marked at the top — build as written, add nothing.
    if matches!(
        ast,
        Astn::StayFoolish { .. } | Astn::StayFullyFoolish { .. }
    ) {
        return ConcatElemKind::AsWritten;
    }
    // Rule 2: constantew — CONSTANT / INDEPENDENT / NK by construction. A mark
    // could defer nothing, so leave it alone. (Recognised syntactically: the
    // classifier runs on the AST, before any NYES exists.)
    if matches!(ast, Astn::IntLit(_) | Astn::UnknownLit | Astn::Creation) {
        return ConcatElemKind::AsWritten;
    }
    // Rule 3: any search — auto-SF.
    if is_search_astn(ast) {
        return ConcatElemKind::BareSearch;
    }
    // Rule 4: any brane-like — auto-SFF. A written concatenation is brane-like,
    // so a nested one is treated exactly as a brane (FOOP-55 §9.2). Omitting it
    // made `(({1}{2}) ({3}{4}))` drop its second inner concatenation: that
    // constituent fell through to rule 5 and NK'd, taking the outer
    // concatenation with it.
    if matches!(ast, Astn::Brane { .. } | Astn::Concatenation { .. }) {
        return ConcatElemKind::BareBrane;
    }
    // Rule 5.
    ConcatElemKind::Error
}

/// Whether an AST node is a **search** for FOOP-55 §9 rule 3.
///
/// `SearchPosition` (`@`) and `ComputedSeek` (`#(expr)`) are included: both were
/// added in §8 and are searches, and omitting them made them NK as
/// concatenation elements.
fn is_search_astn(ast: &Astn) -> bool {
    match ast {
        Astn::Identifier { .. }
        | Astn::DotSearch { .. }
        | Astn::RegexpSearch { .. }
        | Astn::Seek { .. }
        | Astn::ComputedSeek { .. }
        | Astn::SearchPosition { .. }
        | Astn::HeadTail { .. }
        | Astn::UnanchoredSeek { .. }
        | Astn::ValueSearch { .. } => true,
        // A contexted search wraps a bare search.
        Astn::ContextedSearch { inner } => is_search_astn(inner),
        _ => false,
    }
}

/// A short human-readable name for a constituent that is **not** valid in a
/// concatenation, used to build the NK reason (FOOP-55 §9.4(a)).
///
/// Only reached for [`ConcatElemKind::Error`], so it need not name every AST
/// node — it names the kinds a Foolisher plausibly writes by mistake and falls
/// back to a generic phrase otherwise.
fn concat_element_kind_name(ast: &Astn) -> &'static str {
    match ast {
        Astn::BinaryOp { .. } => "number",
        Astn::UnaryOp { .. } => "number",
        _ => "this expression",
    }
}

fn build_concat_element(ast: Astn, parent: &Weak<RefCell<dyn Fir>>, under_sff: bool) -> FirRef {
    let kind = classify_concat_element(&ast);
    match kind {
        ConcatElemKind::BareBrane => {
            // Bare brane literal → wrap in SFF: build with under_sff = true.
            build_fir(ast, Some(parent), true)
        }
        ConcatElemKind::BareSearch => {
            // Bare search → wrap in SF: build, then wrap in StayFoolishFir.
            let search_fir = build_fir(ast, Some(parent), under_sff);
            Rc::new(RefCell::new(StayFoolishFir {
                core: ProtoBrane::new(vec![search_fir], parent.clone(), Nyes::Prembrionic),
            }))
        }
        ConcatElemKind::AsWritten => {
            // Rules 1 and 2 — the user's own mark, or a constantew value.
            // Build exactly as written: no wrapper is added and no mark is
            // downgraded.
            build_fir(ast, Some(parent), under_sff)
        }
        ConcatElemKind::Error => {
            // Rule 5 — not a valid constituent. NK at construction, naming the
            // cause: "cannot concatenate X" reads as a diagnosis, where the bare
            // "invalid concatenation element" left the Foolisher to guess which
            // constituent and why. Detection here (classify time) is earlier
            // than settle time, which FOOP-55 §9.4(a) prefers.
            Rc::new(RefCell::new(NkFir {
                core: ProtoBrane::new(vec![], parent.clone(), Nyes::Nk),
                reason: format!("cannot concatenate {}", concat_element_kind_name(&ast)),
            }))
        }
    }
}

pub struct Compiler;

impl Compiler {
    pub fn compile(source: &str) -> anyhow::Result<Vec<FirRef>> {
        let asts = foolish_parser::parse(source)?;
        asts.into_iter()
            .map(AstnCompilerExt::compile_standalone)
            .collect()
    }
}

fn validate_astn(ast: &Astn) -> anyhow::Result<()> {
    match ast {
        Astn::IfExpr { .. } => Err(anyhow!("if-then-else: not supported (FOOP=2)")),
        Astn::UpwardSearch => Err(anyhow!("Upward search: deferred")),
        Astn::DetachmentBrane { .. } => Err(anyhow!("Detachment brane: deferred")),
        Astn::NotImplemented(r) => Err(anyhow!("Not yet implemented: {}", r)),
        Astn::SearchPosition { anchor } => validate_astn(anchor),
        Astn::ComputedSeek { anchor, index } => {
            validate_astn(anchor)?;
            validate_astn(index)
        }
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
        Astn::StayFoolish { expr } => validate_astn(expr),
        Astn::StayFullyFoolish { expr } => validate_astn(expr),
        Astn::IntLit(_)
        | Astn::UnknownLit
        | Astn::Creation
        | Astn::Identifier { .. }
        | Astn::UnanchoredSeek { .. } => Ok(()),
    }
}

/// Build a FIR from an AST node (`Astn` = AST node).
///
/// `parent`: `None` ⇒ build a ROOT node — it is its own parent (`is_root()` convention,
/// parent Weak points at self). `Some(p)` ⇒ a child whose parent is `p`. Every arm builds
/// inside `Rc::new_cyclic` so the self-Weak (`me`) is available for both rooting and for
/// wiring this node as its own children's parent.
///
/// `under_sff` (FOOP-62 #17): true when an SFF (`<<…>>`) is an ANCESTOR being built from
/// Foolish code. While true, descendant SEARCH FIRs (plain/dot/regexp searches, Index,
/// Index/HeadTail) are CONSTRUCTED at ECONSTANIC so they never run — the SFF body is constanic
/// unevaluated. This is a BUILD-FROM-CODE rule ONLY; it does NOT affect constanic-cloning of
/// an SFF child (that strips the marker and uses normal constanic-clone nyes rules).
fn build_fir(ast: Astn, parent: Option<&Weak<RefCell<dyn Fir>>>, under_sff: bool) -> FirRef {
    // Searches built under an SFF start ECONSTANIC; otherwise PREMBRIONIC.
    let search_nyes = if under_sff {
        Nyes::Econstanic
    } else {
        Nyes::Prembrionic
    };
    // Only a Brane may be a ROOT (its own parent when `parent` is None). Every other kind
    // MUST be given a parent — a non-Brane can never be the root (FOOP-62 root convention).
    macro_rules! brane_parent {
        ($me:expr) => {
            parent.cloned().unwrap_or_else(|| $me.clone())
        };
    }
    macro_rules! child_parent {
        () => {
            parent
                .expect("non-Brane FIR must have a parent — only a Brane can be root")
                .clone()
        };
    }
    match ast {
        // Leaves have no children, so they never self-root and need no self-Weak.
        Astn::IntLit(n) => Rc::new(RefCell::new(IndepIntFir {
            core: ProtoBrane::new(vec![], child_parent!(), Nyes::Independent),
            value: n as i64,
        })),
        Astn::UnknownLit => Rc::new(RefCell::new(NkFir {
            core: ProtoBrane::new(vec![], child_parent!(), Nyes::Nk),
            reason: "??? literal".to_string(),
        })),
        Astn::Creation => Rc::new(RefCell::new(CreationFir {
            core: ProtoBrane::new(vec![], child_parent!(), Nyes::Independent),
        })),
        Astn::Identifier {
            characterizations,
            id,
        } => {
            // Fold characterizations back into the search pattern (Gotcha #3).
            // A 'True reference must search for 'True, not just True.
            let full_pattern = if characterizations.is_empty() {
                id.clone()
            } else {
                let char_str: String = characterizations.iter().map(|c| format!("{c}'")).collect();
                format!("{char_str}{id}")
            };
            Rc::new(RefCell::new(SearchFir {
                core: ProtoBrane::new(vec![], child_parent!(), search_nyes),
                pattern: format!("^{full_pattern}$"),
                anchored: false,
                forward: false,
                sf_inner_pattern: RefCell::new(None),
                is_value_search: false,
                contexted: false,
                exhausted: std::cell::Cell::new(false),
            }))
        }
        Astn::Brane {
            characterizations,
            statements,
        } => Rc::new_cyclic(|me: &Weak<RefCell<BraneFir>>| {
            let me_dyn: Weak<RefCell<dyn Fir>> = me.clone();
            let children = build_stmts(statements, &me_dyn, under_sff);
            RefCell::new(BraneFir {
                core: ProtoBrane::new(children, brane_parent!(me_dyn), Nyes::Prembrionic),
                characterizations: crate::identifier::Characterizations::from_brane_parts(
                    characterizations,
                ),
            })
        }),
        Astn::BinaryOp { op, left, right } => Rc::new_cyclic(|me: &Weak<RefCell<OperatorFir>>| {
            let me_dyn: Weak<RefCell<dyn Fir>> = me.clone();
            let l = build_fir(*left, Some(&me_dyn), under_sff);
            let r = build_fir(*right, Some(&me_dyn), under_sff);
            RefCell::new(OperatorFir {
                core: ProtoBrane::new(vec![l, r], child_parent!(), Nyes::Prembrionic),
                op,
            })
        }),
        Astn::UnaryOp { op, expr } => Rc::new_cyclic(|me: &Weak<RefCell<OperatorFir>>| {
            let me_dyn: Weak<RefCell<dyn Fir>> = me.clone();
            let e = build_fir(*expr, Some(&me_dyn), under_sff);
            RefCell::new(OperatorFir {
                core: ProtoBrane::new(vec![e], child_parent!(), Nyes::Prembrionic),
                op,
            })
        }),
        Astn::DotSearch { anchor, coordinate } => {
            Rc::new_cyclic(|me: &Weak<RefCell<SearchFir>>| {
                let me_dyn: Weak<RefCell<dyn Fir>> = me.clone();
                let a = build_fir(*anchor, Some(&me_dyn), under_sff);
                RefCell::new(SearchFir {
                    core: ProtoBrane::new(vec![a], child_parent!(), search_nyes),
                    pattern: format!("^{}$", coordinate),
                    anchored: true,
                    forward: false,
                    sf_inner_pattern: RefCell::new(None),
                    is_value_search: false,
                    contexted: false,
                    exhausted: std::cell::Cell::new(false),
                })
            })
        }
        Astn::RegexpSearch {
            anchor,
            pattern,
            operator,
            ..
        } => Rc::new_cyclic(|me: &Weak<RefCell<SearchFir>>| {
            let me_dyn: Weak<RefCell<dyn Fir>> = me.clone();
            let has_anchor = anchor.is_some();
            let children: Vec<FirRef> = match anchor {
                Some(a) => vec![build_fir(*a, Some(&me_dyn), under_sff)],
                None => vec![],
            };
            RefCell::new(SearchFir {
                core: ProtoBrane::new(children, child_parent!(), search_nyes),
                pattern,
                anchored: has_anchor,
                forward: operator == SearchOperator::RegexpForward,
                sf_inner_pattern: RefCell::new(None),
                is_value_search: false,
                contexted: false,
                exhausted: std::cell::Cell::new(false),
            })
        }),
        Astn::ValueSearch {
            anchor,
            forward,
            name_pattern,
            value_pattern,
        } => Rc::new_cyclic(|me: &Weak<RefCell<SearchFir>>| {
            let me_dyn: Weak<RefCell<dyn Fir>> = me.clone();
            let has_anchor = anchor.is_some();
            let mut children: Vec<FirRef> = if let Some(a) = anchor {
                vec![build_fir(*a, Some(&me_dyn), under_sff)]
            } else {
                vec![]
            };
            let value_fir = build_fir(*value_pattern, Some(&me_dyn), under_sff);
            children.push(value_fir);
            let pattern = name_pattern.unwrap_or_default();
            RefCell::new(SearchFir {
                core: ProtoBrane::new(children, child_parent!(), search_nyes),
                pattern,
                anchored: has_anchor,
                forward,
                sf_inner_pattern: RefCell::new(None),
                is_value_search: true,
                contexted: false,
                exhausted: std::cell::Cell::new(false),
            })
        }),
        Astn::Seek { anchor, offset } => Rc::new_cyclic(|me: &Weak<RefCell<IndexFir>>| {
            let me_dyn: Weak<RefCell<dyn Fir>> = me.clone();
            let a = build_fir(*anchor, Some(&me_dyn), under_sff);
            RefCell::new(IndexFir {
                core: ProtoBrane::new(vec![a], child_parent!(), search_nyes),
                offset,
                index_expr: None,
                anchored: true,
                contexted: false,
            })
        }),
        Astn::HeadTail { is_head, anchor } => Rc::new_cyclic(|me: &Weak<RefCell<IndexFir>>| {
            let me_dyn: Weak<RefCell<dyn Fir>> = me.clone();
            let a = build_fir(*anchor, Some(&me_dyn), under_sff);
            let offset = if is_head { 0 } else { -1 };
            RefCell::new(IndexFir {
                core: ProtoBrane::new(vec![a], child_parent!(), search_nyes),
                offset,
                index_expr: None,
                anchored: true,
                contexted: false,
            })
        }),
        Astn::ComputedSeek { anchor, index } => {
            Rc::new_cyclic(|me: &Weak<RefCell<IndexFir>>| {
                let me_dyn: Weak<RefCell<dyn Fir>> = me.clone();
                let a = build_fir(*anchor, Some(&me_dyn), under_sff);
                let i = build_fir(*index, Some(&me_dyn), under_sff);
                RefCell::new(IndexFir {
                    // The anchor stays foolish_children[0], as for a literal
                    // index; the operand is held separately so the navigation
                    // code below can read it without disturbing that.
                    core: ProtoBrane::new(vec![a], child_parent!(), search_nyes),
                    offset: 0,
                    index_expr: Some(i),
                    anchored: true,
                    contexted: false,
                })
            })
        }
        Astn::SearchPosition { anchor } => {
            Rc::new_cyclic(|me: &Weak<RefCell<crate::fir_kinds::SearchPositionFir>>| {
                let me_dyn: Weak<RefCell<dyn Fir>> = me.clone();
                let a = build_fir(*anchor, Some(&me_dyn), under_sff);
                RefCell::new(crate::fir_kinds::SearchPositionFir {
                    core: ProtoBrane::new(vec![a], child_parent!(), search_nyes),
                })
            })
        }
        Astn::UnanchoredSeek { offset } => Rc::new(RefCell::new(IndexFir {
            core: ProtoBrane::new(vec![], child_parent!(), search_nyes),
            offset,
            index_expr: None,
            anchored: false,
            contexted: false,
        })),
        Astn::Concatenation { elements } => {
            Rc::new_cyclic(|me: &Weak<RefCell<ConcatenationFir>>| {
                let me_dyn: Weak<RefCell<dyn Fir>> = me.clone();
                let children: Vec<FirRef> = elements
                    .into_iter()
                    .map(|e| build_concat_element(e, &me_dyn, under_sff))
                    .collect();
                RefCell::new(ConcatenationFir {
                    core: ProtoBrane::new(children, child_parent!(), Nyes::Prembrionic),
                    _helpers_populated: std::cell::Cell::new(false),
                })
            })
        }
        Astn::StayFoolish { expr } => Rc::new_cyclic(|me: &Weak<RefCell<StayFoolishFir>>| {
            let me_dyn: Weak<RefCell<dyn Fir>> = me.clone();
            // SF does NOT make descendants econstanic — only SFF does; preserve any inherited flag.
            let e = build_fir(*expr, Some(&me_dyn), under_sff);
            RefCell::new(StayFoolishFir {
                core: ProtoBrane::new(vec![e], child_parent!(), Nyes::Prembrionic),
            })
        }),
        Astn::StayFullyFoolish { expr } => {
            Rc::new_cyclic(|me: &Weak<RefCell<StayFullyFoolishFir>>| {
                let me_dyn: Weak<RefCell<dyn Fir>> = me.clone();
                // SFF marker: from here down, searches are built ECONSTANIC.
                let e = build_fir(*expr, Some(&me_dyn), true);
                // Sanity-check that the `under_sff` rule actually reached every
                // descendant search before storing the body (debug builds only).
                let mut core = ProtoBrane::new(vec![], child_parent!(), Nyes::Prembrionic);
                core.push_foolish_child_sff_marked(e);
                RefCell::new(StayFullyFoolishFir { core })
            })
        }
        Astn::ContextedSearch { inner } => {
            let fir = build_fir(*inner, parent, under_sff);
            fir.borrow_mut().set_contexted(true);
            fir
        }
        Astn::Assignment { .. } => {
            unreachable!("standalone Assignment should be wrapped in Brane by parser")
        }
        _ => unreachable!("validate_astn should have rejected this"),
    }
}

fn build_stmts(asts: Vec<Astn>, parent: &Weak<RefCell<dyn Fir>>, under_sff: bool) -> Vec<FirRef> {
    asts.into_iter()
        .enumerate()
        .map(|(i, ast)| ast.build_as_statement(parent, i, under_sff))
        .collect()
}

/// Compile the body of the sole statement of a one-statement brane `source`,
/// built directly beneath `parent`.
///
/// Unlike [`Compiler::compile`], which roots the brane it is given, this keeps
/// only the statement BODY and parents it on an existing FIR. Used by
/// `system_foo.rs` to build the comparison operators' `<<#-2>>`/`<<#-1>>`
/// operands as real, compiler-built Foolish rather than hand-assembled FIR.
/// Going through `build_fir` is the point: the `under_sff` rule (which builds
/// descendant searches ECONSTANIC so they never run) applies exactly as it
/// does to any other Foolish, so the operands cannot drift from it.
///
/// The brane-and-statement wrapper in `source` exists only because an SFF
/// marker is not valid at top level; it is discarded here.
pub(crate) fn compile_stmt_body_under(
    source: &str,
    parent: &Weak<RefCell<dyn Fir>>,
) -> anyhow::Result<FirRef> {
    let asts = foolish_parser::parse(source)?;
    let [ast] = <[Astn; 1]>::try_from(asts)
        .map_err(|v| anyhow!("expected exactly one top-level brane, found {}", v.len()))?;
    validate_astn(&ast)?;
    let Astn::Brane { mut statements, .. } = ast else {
        return Err(anyhow!("expected a brane"));
    };
    if statements.len() != 1 {
        return Err(anyhow!(
            "expected exactly one statement, found {}",
            statements.len()
        ));
    }
    let Astn::Assignment { expr, operator, .. } = statements.remove(0) else {
        return Err(anyhow!("expected an assignment"));
    };
    Ok(expr.build_expr_with_operator(operator, parent, false))
}

/// Builds a statement's body in place of the one its AST describes.
///
/// Given a statement's [`Identifier`] and its own self-`Weak` (usable as the
/// body's parent), returns `Some(body)` to supply that body instead of
/// compiling the AST's, or `None` to compile normally.
///
/// This exists for `system.foo`, whose comparison operators are DECLARED in
/// Foolish as ordinary `'lt = ⬤` creations but whose real bodies are Rust FIR
/// kinds (FOOP-33 §5.0: "that foolishness is put into the system brane by
/// fvm + system_foo.rs"). The hook keeps brane and statement construction here
/// in the compiler — including line numbering and the `Rc::new_cyclic` parent
/// wiring — rather than duplicating it in `system_foo.rs`.
pub(crate) type BodyOverride<'a> =
    &'a dyn Fn(&crate::identifier::Identifier, &Weak<RefCell<dyn Fir>>) -> Option<FirRef>;

/// Compile a top-level brane AST as a self-rooting root, letting `override_body`
/// replace individual statements' bodies.
///
/// Identical to [`AstnCompilerExt::compile_standalone`] except for the hook;
/// see [`BodyOverride`] for why it exists.
pub(crate) fn compile_root_with_body_override(
    ast: Astn,
    override_body: BodyOverride<'_>,
) -> anyhow::Result<FirRef> {
    validate_astn(&ast)?;
    let Astn::Brane {
        characterizations,
        statements,
    } = ast
    else {
        return Err(anyhow!("only a Brane can be a top-level (root) node"));
    };
    Ok(Rc::new_cyclic(|me: &Weak<RefCell<BraneFir>>| {
        let me_dyn: Weak<RefCell<dyn Fir>> = me.clone();
        let children = statements
            .into_iter()
            .enumerate()
            .map(|(i, stmt_ast)| stmt_ast.build_as_statement_overridden(&me_dyn, i, override_body))
            .collect();
        RefCell::new(BraneFir {
            core: ProtoBrane::new(children, me_dyn.clone(), Nyes::Prembrionic),
            characterizations: crate::identifier::Characterizations::from_brane_parts(
                characterizations,
            ),
        })
    }))
}

/// The name used for an anonymous statement (a bare expression with no LHS identifier).
/// The sequencer renders a statement named `???` WITHOUT a `name=` prefix (FOOP-62 #19).
pub(crate) const ANON_STMT_NAME: &str = "???";

pub(crate) trait AstnCompilerExt {
    fn compile_standalone(self) -> anyhow::Result<FirRef>;

    fn build_as_statement(
        self,
        parent: &Weak<RefCell<dyn Fir>>,
        line: usize,
        under_sff: bool,
    ) -> FirRef;

    /// As [`Self::build_as_statement`], but consulting `override_body` first.
    /// See [`BodyOverride`].
    fn build_as_statement_overridden(
        self,
        parent: &Weak<RefCell<dyn Fir>>,
        line: usize,
        override_body: BodyOverride<'_>,
    ) -> FirRef;

    fn build_as_statement_inner(
        self,
        parent: &Weak<RefCell<dyn Fir>>,
        line: usize,
        under_sff: bool,
        override_body: Option<BodyOverride<'_>>,
    ) -> FirRef;

    fn build_expr_with_operator(
        self,
        operator: AssignmentOperator,
        parent: &Weak<RefCell<dyn Fir>>,
        under_sff: bool,
    ) -> FirRef;
}

impl AstnCompilerExt for Astn {
    fn compile_standalone(self) -> anyhow::Result<FirRef> {
        validate_astn(&self)?;
        // Only a Brane can be a ROOT node (FOOP-62 root convention: the root is its own parent).
        // A top-level non-Brane has no valid root to attach to.
        if !matches!(self, Astn::Brane { .. }) {
            return Err(anyhow!("only a Brane can be a top-level (root) node"));
        }
        Ok(build_fir(self, None, false))
    }

    fn build_as_statement(
        self,
        parent: &Weak<RefCell<dyn Fir>>,
        line: usize,
        under_sff: bool,
    ) -> FirRef {
        self.build_as_statement_inner(parent, line, under_sff, None)
    }

    fn build_as_statement_overridden(
        self,
        parent: &Weak<RefCell<dyn Fir>>,
        line: usize,
        override_body: BodyOverride<'_>,
    ) -> FirRef {
        self.build_as_statement_inner(parent, line, false, Some(override_body))
    }

    fn build_as_statement_inner(
        self,
        parent: &Weak<RefCell<dyn Fir>>,
        line: usize,
        under_sff: bool,
        override_body: Option<BodyOverride<'_>>,
    ) -> FirRef {
        // Decide the statement's name once: the LHS identifier for an assignment, else `???`
        // (anonymous bare expression). The body is built the same way regardless, via
        // build_expr_with_operator (Assign is the no-op operator), so there is ONE Rc::new_cyclic.
        let (characterizations, name, expr, operator) = match self {
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
        let identifier = crate::identifier::Identifier::from_parts(characterizations, &name);
        Rc::new_cyclic(move |me: &Weak<RefCell<StatementFir>>| {
            let stmt_weak: Weak<RefCell<dyn Fir>> = me.clone();
            let body = override_body
                .and_then(|f| f(&identifier, &stmt_weak))
                .unwrap_or_else(|| expr.build_expr_with_operator(operator, &stmt_weak, under_sff));
            RefCell::new(StatementFir {
                core: ProtoBrane::new(vec![body], parent.clone(), Nyes::Prembrionic),
                identifier,
                line_number: line,
                self_weak: stmt_weak,
                nf_reason: RefCell::new(None),
            })
        })
    }

    fn build_expr_with_operator(
        self,
        operator: AssignmentOperator,
        parent: &Weak<RefCell<dyn Fir>>,
        under_sff: bool,
    ) -> FirRef {
        // An SFF assignment operator makes its body's descendant searches econstanic.
        let body_under_sff = under_sff || operator == AssignmentOperator::SFF;
        let body = build_fir(self, Some(parent), body_under_sff);
        match operator {
            AssignmentOperator::Assign => body,
            AssignmentOperator::SF => Rc::new(RefCell::new(StayFoolishFir {
                core: ProtoBrane::new(vec![body], parent.clone(), Nyes::Prembrionic),
            })),
            AssignmentOperator::SFF => Rc::new(RefCell::new(StayFullyFoolishFir {
                core: ProtoBrane::new(vec![body], parent.clone(), Nyes::Independent),
            })),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ANON_STMT_NAME, AstnCompilerExt, ConcatElemKind, classify_concat_element};
    use std::cell::RefCell;
    use std::rc::{Rc, Weak};

    use foolish_core::fir::Nyes;
    use foolish_parser::{AssignmentOperator, Astn};

    use crate::fir_kinds::BraneFir;
    use crate::fir_trait::{Fir, FirKind};
    use crate::proto_brane::ProtoBrane;

    fn root_parent() -> Weak<RefCell<dyn Fir>> {
        let root = Rc::new_cyclic(|me: &Weak<RefCell<BraneFir>>| {
            let me_dyn: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(BraneFir {
                core: ProtoBrane::new(vec![], me_dyn, Nyes::Prembrionic),
                characterizations: crate::identifier::Characterizations::default(),
            })
        });
        Rc::downgrade(&(root as Rc<RefCell<dyn Fir>>))
    }

    #[test]
    fn build_as_statement_keeps_assignment_name_and_anonymous_fallback() {
        let parent = root_parent();

        let named = Astn::Assignment {
            characterizations: vec![],
            identifier: "x".to_string(),
            operator: AssignmentOperator::Assign,
            expr: Box::new(Astn::IntLit(1)),
        }
        .build_as_statement(&parent, 7, false);
        assert_eq!(named.borrow().kind(), FirKind::Statement);
        assert_eq!(named.borrow().as_stmt_searchable_name(), Some("x"));
        assert_eq!(named.borrow().as_stmt_line_number(), Some(7));

        let anonymous = Astn::IntLit(2).build_as_statement(&parent, 8, false);
        assert_eq!(anonymous.borrow().kind(), FirKind::Statement);
        assert_eq!(
            anonymous.borrow().as_stmt_searchable_name(),
            Some(ANON_STMT_NAME)
        );
        assert_eq!(anonymous.borrow().as_stmt_line_number(), Some(8));
    }

    #[test]
    fn compile_standalone_rejects_non_brane_root() {
        let err = Astn::IntLit(1)
            .compile_standalone()
            .expect_err("non-Brane root must be rejected");
        assert_eq!(
            err.to_string(),
            "only a Brane can be a top-level (root) node"
        );
    }

    #[test]
    fn build_expr_with_operator_wraps_sf_and_sff() {
        let parent = root_parent();

        let sf = Astn::IntLit(1).build_expr_with_operator(AssignmentOperator::SF, &parent, false);
        assert_eq!(sf.borrow().kind(), FirKind::StayFoolish);
        assert_eq!(sf.borrow().core().get_nyes(), Nyes::Prembrionic);

        let sff = Astn::IntLit(2).build_expr_with_operator(AssignmentOperator::SFF, &parent, false);
        assert_eq!(sff.borrow().kind(), FirKind::StayFullyFoolish);
        assert_eq!(sff.borrow().core().get_nyes(), Nyes::Independent);
    }

    // ---- FOOP-55 §9.2: concatenation constituent classification ----------
    //
    // These pin the CLASSIFICATION, not the evaluated value: classification is
    // where §9.2's rule lives, and a value test would also be sensitive to
    // stepping and coordination, which §9.2 does not govern.

    /// §9.2: "a concatenation is brane-like, so it is treated exactly as a
    /// brane" — hence SFF, the same as `Astn::Brane`.
    ///
    /// Regression: while `Concatenation` was missing from rule 4 it fell through
    /// to rule 5 and NK'd, so `(({1}{2}) ({3}{4}))` silently DROPPED its second
    /// inner concatenation, yielding `{NK 1; 2}` instead of a four-statement
    /// flatten.
    #[test]
    fn concat_constituent_classifies_as_brane_like() {
        let nested = Astn::Concatenation {
            elements: vec![Astn::IntLit(3), Astn::IntLit(4)],
        };
        assert_eq!(
            classify_concat_element(&nested),
            ConcatElemKind::BareBrane,
            "a nested concatenation is brane-like and must be SFF-marked, as a brane is"
        );
        // …and identically to the brane it is being equated with.
        let brane = Astn::Brane {
            characterizations: vec![],
            statements: vec![],
        };
        assert_eq!(
            classify_concat_element(&brane),
            classify_concat_element(&nested),
            "§9.2 says 'treated exactly as a brane' — the two must not diverge"
        );
    }

    /// §9.2 rule 1: a constituent the Foolisher already marked is compiled AS
    /// WRITTEN. Rule 1 is tested before anything looks inside the mark, so
    /// neither mark can pick up a second wrapper.
    #[test]
    fn marked_constituent_is_left_as_written() {
        for marked in [
            Astn::StayFoolish {
                expr: Box::new(Astn::Brane { characterizations: vec![], statements: vec![] }),
            },
            Astn::StayFullyFoolish {
                expr: Box::new(Astn::Brane { characterizations: vec![], statements: vec![] }),
            },
        ] {
            assert_eq!(
                classify_concat_element(&marked),
                ConcatElemKind::AsWritten,
                "already-marked constituent must be compiled as written: {marked:?}"
            );
        }
    }

    /// §9.2: an operator constituent is "not allowed — the compiler emits NK".
    /// The reason must NAME THE CAUSE; a bare "invalid concatenation element"
    /// leaves the Foolisher to guess which constituent and why.
    #[test]
    fn operator_constituent_is_error_naming_the_cause() {
        let op = Astn::BinaryOp {
            op: "+".to_string(),
            left: Box::new(Astn::IntLit(2)),
            right: Box::new(Astn::IntLit(3)),
        };
        assert_eq!(classify_concat_element(&op), ConcatElemKind::Error);
        assert_eq!(super::concat_element_kind_name(&op), "number");
    }
}
