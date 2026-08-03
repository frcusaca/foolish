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
    /// Bare search — auto-wrap in SF.
    BareSearch,
    /// <search> — idempotent NOOP, build as-is.
    SfSearch,
    /// <{…}> — override auto-SFF: build without under_sff, wrap in SF.
    SfBrane,
    /// <<…>> or other — error, NK at construction.
    Error,
}

fn classify_concat_element(ast: &Astn) -> ConcatElemKind {
    match ast {
        Astn::Brane { .. } => ConcatElemKind::BareBrane,
        Astn::Identifier { .. }
        | Astn::DotSearch { .. }
        | Astn::RegexpSearch { .. }
        | Astn::Seek { .. }
        | Astn::HeadTail { .. }
        | Astn::UnanchoredSeek { .. }
        | Astn::ValueSearch { .. } => ConcatElemKind::BareSearch,
        Astn::ContextedSearch { inner } => {
            // Contexted search wraps a bare search.
            matches!(
                inner.as_ref(),
                Astn::Identifier { .. }
                    | Astn::DotSearch { .. }
                    | Astn::RegexpSearch { .. }
                    | Astn::Seek { .. }
                    | Astn::HeadTail { .. }
                    | Astn::UnanchoredSeek { .. }
                    | Astn::ValueSearch { .. }
            )
            .then(|| ConcatElemKind::BareSearch)
            .unwrap_or(ConcatElemKind::Error)
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
        ConcatElemKind::SfSearch => {
            // <search> — idempotent NOOP, build as-is.
            build_fir(ast, Some(parent), under_sff)
        }
        ConcatElemKind::SfBrane => {
            // <{…}> — override auto-SFF: build WITHOUT under_sff, wrap in SF.
            let brane_fir = build_fir(ast, Some(parent), false);
            Rc::new(RefCell::new(StayFoolishFir {
                core: ProtoBrane::new(vec![brane_fir], parent.clone(), Nyes::Prembrionic),
            }))
        }
        ConcatElemKind::Error => {
            // <<…>> or other — error: NK at construction.
            Rc::new(RefCell::new(NkFir {
                core: ProtoBrane::new(vec![], parent.clone(), Nyes::Nk),
                reason: "invalid concatenation element".to_string(),
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
                    is_value_search: false,
                    contexted: false,
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
            })
        }),
        Astn::Seek { anchor, offset } => Rc::new_cyclic(|me: &Weak<RefCell<IndexFir>>| {
            let me_dyn: Weak<RefCell<dyn Fir>> = me.clone();
            let a = build_fir(*anchor, Some(&me_dyn), under_sff);
            RefCell::new(IndexFir {
                core: ProtoBrane::new(vec![a], child_parent!(), search_nyes),
                offset,
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
                anchored: true,
                contexted: false,
            })
        }),
        Astn::UnanchoredSeek { offset } => Rc::new(RefCell::new(IndexFir {
            core: ProtoBrane::new(vec![], child_parent!(), search_nyes),
            offset,
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
                RefCell::new(StayFullyFoolishFir {
                    core: ProtoBrane::new(vec![e], child_parent!(), Nyes::Prembrionic),
                })
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

/// The name used for an anonymous statement (a bare expression with no LHS identifier).
/// The sequencer renders a statement named `???` WITHOUT a `name=` prefix (FOOP-62 #19).
pub(crate) const ANON_STMT_NAME: &str = "???";

trait AstnCompilerExt {
    fn compile_standalone(self) -> anyhow::Result<FirRef>;

    fn build_as_statement(
        self,
        parent: &Weak<RefCell<dyn Fir>>,
        line: usize,
        under_sff: bool,
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
            let body = expr.build_expr_with_operator(operator, &stmt_weak, under_sff);
            RefCell::new(StatementFir {
                core: ProtoBrane::new(vec![body], parent.clone(), Nyes::Prembrionic),
                identifier,
                line_number: line,
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
    use super::{AstnCompilerExt, ANON_STMT_NAME};
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
                characterizations: vec![],
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
        assert_eq!(anonymous.borrow().as_stmt_searchable_name(), Some(ANON_STMT_NAME));
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
}
