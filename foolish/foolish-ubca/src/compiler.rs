use std::cell::RefCell;
use std::rc::{Rc, Weak};

use anyhow::anyhow;
use foolish_parser::{AssignmentOperator, Astn, SearchOperator};

use crate::fir_kinds::{
    BraneFir, ConcatenationFir, ConstantIntFir, IndexFir, NkFir, OperatorFir, SearchFir,
    StatementFir, StayFoolishFir, StayFullyFoolishFir,
};
use crate::fir_trait::{Fir, FirRef};
use crate::proto_brane::ProtoBrane;
use foolish_core::fir::Nyes;

pub struct Compiler;

impl Compiler {
    pub fn compile(source: &str) -> anyhow::Result<Vec<FirRef>> {
        let asts = foolish_parser::parse(source)?;
        asts.into_iter().map(compile_standalone).collect()
    }
}

fn compile_standalone(ast: Astn) -> anyhow::Result<FirRef> {
    validate_astn(&ast)?;
    // Only a Brane can be a ROOT node (FOOP-62 root convention: the root is its own parent).
    // A top-level non-Brane has no valid root to attach to.
    if !matches!(ast, Astn::Brane { .. }) {
        return Err(anyhow!("only a Brane can be a top-level (root) node"));
    }
    Ok(build_fir(ast, None, false))
}

fn validate_astn(ast: &Astn) -> anyhow::Result<()> {
    match ast {
        Astn::IfExpr { .. } => Err(anyhow!("if-then-else: not supported (FOOP=2)")),
        Astn::UpwardSearch => Err(anyhow!("Upward search: deferred")),
        Astn::DetachmentBrane { .. } => Err(anyhow!("Detachment brane: deferred")),
        Astn::NotImplemented(r) => Err(anyhow!("Not yet implemented: {}", r)),
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
        Astn::RegexpSearch { anchor, .. } => validate_astn(anchor),
        Astn::Seek { anchor, .. } => validate_astn(anchor),
        Astn::HeadTail { anchor, .. } => validate_astn(anchor),
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
/// HeadTail) are CONSTRUCTED at ECONSTANIC so they never run — the SFF body is frozen
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
        Astn::IntLit(n) => Rc::new(RefCell::new(ConstantIntFir {
            core: ProtoBrane::new(vec![], child_parent!(), Nyes::Independent),
            value: n as i64,
        })),
        Astn::UnknownLit => Rc::new(RefCell::new(NkFir {
            core: ProtoBrane::new(vec![], child_parent!(), Nyes::Nk),
            reason: "??? literal".to_string(),
        })),
        Astn::Identifier { id, .. } => Rc::new(RefCell::new(SearchFir {
            core: ProtoBrane::new(vec![], child_parent!(), search_nyes),
            pattern: format!("^{}$", id),
            anchored: false,
            forward: false,
            sf_inner_pattern: RefCell::new(None),
        })),
        Astn::Brane {
            characterizations,
            statements,
        } => Rc::new_cyclic(|me: &Weak<RefCell<BraneFir>>| {
            let me_dyn: Weak<RefCell<dyn Fir>> = me.clone();
            let children = build_stmts(statements, &me_dyn, under_sff);
            RefCell::new(BraneFir {
                core: ProtoBrane::new(children, brane_parent!(me_dyn), Nyes::Prembrionic),
                characterizations,
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
            let a = build_fir(*anchor, Some(&me_dyn), under_sff);
            RefCell::new(SearchFir {
                core: ProtoBrane::new(vec![a], child_parent!(), search_nyes),
                pattern,
                anchored: true,
                forward: operator == SearchOperator::RegexpForward,
                sf_inner_pattern: RefCell::new(None),
            })
        }),
        Astn::Seek { anchor, offset } => Rc::new_cyclic(|me: &Weak<RefCell<IndexFir>>| {
            let me_dyn: Weak<RefCell<dyn Fir>> = me.clone();
            let a = build_fir(*anchor, Some(&me_dyn), under_sff);
            RefCell::new(IndexFir {
                core: ProtoBrane::new(vec![a], child_parent!(), search_nyes),
                offset,
                anchored: true,
            })
        }),
        Astn::HeadTail { is_head, anchor } => Rc::new_cyclic(|me: &Weak<RefCell<IndexFir>>| {
            let me_dyn: Weak<RefCell<dyn Fir>> = me.clone();
            let a = build_fir(*anchor, Some(&me_dyn), under_sff);
            let offset = if is_head { 0 } else { -1 };
            RefCell::new(IndexFir {
                core: ProtoBrane::new(vec![a], child_parent!(), search_nyes),
                offset,
                anchored: true,
            })
        }),
        Astn::UnanchoredSeek { offset } => Rc::new(RefCell::new(IndexFir {
            core: ProtoBrane::new(vec![], child_parent!(), search_nyes),
            offset,
            anchored: false,
        })),
        Astn::Concatenation { elements } => {
            Rc::new_cyclic(|me: &Weak<RefCell<ConcatenationFir>>| {
                let me_dyn: Weak<RefCell<dyn Fir>> = me.clone();
                let children: Vec<FirRef> = elements
                    .into_iter()
                    .map(|e| build_fir(e, Some(&me_dyn), under_sff))
                    .collect();
                RefCell::new(ConcatenationFir {
                    core: ProtoBrane::new(children, child_parent!(), Nyes::Prembrionic),
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
                RefCell::new(StayFullyFoolishFir {
                    core: ProtoBrane::new(vec![e], child_parent!(), Nyes::Prembrionic),
                })
            })
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
        .map(|(i, ast)| build_as_statement(ast, parent, i, under_sff))
        .collect()
}

/// The name used for an anonymous statement (a bare expression with no LHS identifier).
/// The sequencer renders a statement named `???` WITHOUT a `name=` prefix (FOOP-62 #19).
pub(crate) const ANON_STMT_NAME: &str = "???";

fn build_as_statement(
    ast: Astn,
    parent: &Weak<RefCell<dyn Fir>>,
    line: usize,
    under_sff: bool,
) -> FirRef {
    // Decide the statement's name once: the LHS identifier for an assignment, else `???`
    // (anonymous bare expression). The body is built the same way regardless, via
    // build_expr_with_operator (Assign is the no-op operator), so there is ONE Rc::new_cyclic.
    let (name, expr, operator) = match ast {
        Astn::Assignment {
            identifier,
            operator,
            expr,
            ..
        } => (identifier, *expr, operator),
        other => (
            ANON_STMT_NAME.to_string(),
            other,
            AssignmentOperator::Assign,
        ),
    };
    Rc::new_cyclic(move |me: &Weak<RefCell<StatementFir>>| {
        let stmt_weak: Weak<RefCell<dyn Fir>> = me.clone();
        let body = build_expr_with_operator(expr, operator, &stmt_weak, under_sff);
        RefCell::new(StatementFir {
            core: ProtoBrane::new(vec![body], parent.clone(), Nyes::Prembrionic),
            name,
            line_number: line,
        })
    })
}

fn build_expr_with_operator(
    expr: Astn,
    operator: AssignmentOperator,
    parent: &Weak<RefCell<dyn Fir>>,
    under_sff: bool,
) -> FirRef {
    // An SFF assignment operator makes its body's descendant searches econstanic.
    let body_under_sff = under_sff || operator == AssignmentOperator::SFF;
    let body = build_fir(expr, Some(parent), body_under_sff);
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
