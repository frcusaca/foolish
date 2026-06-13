use std::cell::RefCell;
use std::rc::{Rc, Weak};

use anyhow::anyhow;
use foolish_parser::{AssignmentOperator, Astn};

use crate::fir_kinds::{
    BraneFir, ConcatenationFir, ConstantIntFir, HeadTailFir, IndexFir, NkFir, OperatorFir,
    SearchFir, StatementFir, StayFoolishFir, StayFullyFoolishFir,
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
    Ok(build_standalone(ast))
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

fn build_standalone(ast: Astn) -> FirRef {
    match ast {
        Astn::IntLit(n) => Rc::new_cyclic(|me: &Weak<RefCell<ConstantIntFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(ConstantIntFir {
                core: ProtoBrane::new(vec![], parent, Nyes::Independent),
                value: n as i64,
            })
        }),
        Astn::UnknownLit => Rc::new_cyclic(|me: &Weak<RefCell<NkFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(NkFir {
                core: ProtoBrane::new(vec![], parent, Nyes::Nk),
                reason: "??? literal".to_string(),
            })
        }),
        Astn::Brane {
            characterizations: _,
            statements,
        } => Rc::new_cyclic(|me: &Weak<RefCell<BraneFir>>| {
            let self_weak: Weak<RefCell<dyn Fir>> = me.clone();
            let children = build_stmts(statements, &self_weak);
            RefCell::new(BraneFir {
                core: ProtoBrane::new(children, self_weak, Nyes::Prembrionic),
            })
        }),
        Astn::BinaryOp { op, left, right } => Rc::new_cyclic(|me: &Weak<RefCell<OperatorFir>>| {
            let self_weak: Weak<RefCell<dyn Fir>> = me.clone();
            let l = build_astn(*left, &self_weak);
            let r = build_astn(*right, &self_weak);
            RefCell::new(OperatorFir {
                core: ProtoBrane::new(vec![l, r], self_weak, Nyes::Prembrionic),
                op,
            })
        }),
        Astn::UnaryOp { op, expr } => Rc::new_cyclic(|me: &Weak<RefCell<OperatorFir>>| {
            let self_weak: Weak<RefCell<dyn Fir>> = me.clone();
            let e = build_astn(*expr, &self_weak);
            RefCell::new(OperatorFir {
                core: ProtoBrane::new(vec![e], self_weak, Nyes::Prembrionic),
                op,
            })
        }),
        Astn::Identifier { id, .. } => Rc::new_cyclic(|me: &Weak<RefCell<SearchFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(SearchFir {
                core: ProtoBrane::new(vec![], parent, Nyes::Prembrionic),
                pattern: format!("^{}$", id),
                anchored: false,
                found_body: RefCell::new(None),
            })
        }),
        Astn::DotSearch { anchor, coordinate } => {
            Rc::new_cyclic(|me: &Weak<RefCell<SearchFir>>| {
                let self_weak: Weak<RefCell<dyn Fir>> = me.clone();
                let a = build_astn(*anchor, &self_weak);
                RefCell::new(SearchFir {
                    core: ProtoBrane::new(vec![a], self_weak, Nyes::Prembrionic),
                    pattern: format!("^{}$", coordinate),
                    anchored: true,
                    found_body: RefCell::new(None),
                })
            })
        }
        Astn::RegexpSearch {
            anchor, pattern, ..
        } => Rc::new_cyclic(|me: &Weak<RefCell<SearchFir>>| {
            let self_weak: Weak<RefCell<dyn Fir>> = me.clone();
            let a = build_astn(*anchor, &self_weak);
            RefCell::new(SearchFir {
                core: ProtoBrane::new(vec![a], self_weak, Nyes::Prembrionic),
                pattern,
                anchored: true,
                found_body: RefCell::new(None),
            })
        }),
        Astn::Seek { anchor, offset } => Rc::new_cyclic(|me: &Weak<RefCell<IndexFir>>| {
            let self_weak: Weak<RefCell<dyn Fir>> = me.clone();
            let a = build_astn(*anchor, &self_weak);
            RefCell::new(IndexFir {
                core: ProtoBrane::new(vec![a], self_weak, Nyes::Prembrionic),
                offset,
                anchored: true,
            })
        }),
        Astn::HeadTail { is_head, anchor } => Rc::new_cyclic(|me: &Weak<RefCell<HeadTailFir>>| {
            let self_weak: Weak<RefCell<dyn Fir>> = me.clone();
            let a = build_astn(*anchor, &self_weak);
            RefCell::new(HeadTailFir {
                core: ProtoBrane::new(vec![a], self_weak, Nyes::Prembrionic),
                is_head,
                anchored: true,
            })
        }),
        Astn::UnanchoredSeek { offset } => Rc::new_cyclic(|me: &Weak<RefCell<IndexFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(IndexFir {
                core: ProtoBrane::new(vec![], parent, Nyes::Prembrionic),
                offset,
                anchored: false,
            })
        }),
        Astn::Concatenation { elements } => {
            Rc::new_cyclic(|me: &Weak<RefCell<ConcatenationFir>>| {
                let self_weak: Weak<RefCell<dyn Fir>> = me.clone();
                let children: Vec<FirRef> = elements
                    .into_iter()
                    .map(|e| build_astn(e, &self_weak))
                    .collect();
                RefCell::new(ConcatenationFir {
                    core: ProtoBrane::new(children, self_weak, Nyes::Prembrionic),
                })
            })
        }
        Astn::StayFoolish { expr } => Rc::new_cyclic(|me: &Weak<RefCell<StayFoolishFir>>| {
            let self_weak: Weak<RefCell<dyn Fir>> = me.clone();
            let e = build_astn(*expr, &self_weak);
            RefCell::new(StayFoolishFir {
                core: ProtoBrane::new(vec![e], self_weak, Nyes::Prembrionic),
            })
        }),
        Astn::StayFullyFoolish { expr } => {
            Rc::new_cyclic(|me: &Weak<RefCell<StayFullyFoolishFir>>| {
                let self_weak: Weak<RefCell<dyn Fir>> = me.clone();
                let e = build_astn(*expr, &self_weak);
                RefCell::new(StayFullyFoolishFir {
                    core: ProtoBrane::new(vec![e], self_weak, Nyes::Prembrionic),
                })
            })
        }
        Astn::Assignment { .. } => {
            unreachable!("standalone Assignment should be wrapped in Brane by parser")
        }
        _ => unreachable!("validate_astn should have rejected this"),
    }
}

fn build_astn(ast: Astn, parent: &Weak<RefCell<dyn Fir>>) -> FirRef {
    match ast {
        Astn::IntLit(n) => Rc::new(RefCell::new(ConstantIntFir {
            core: ProtoBrane::new(vec![], parent.clone(), Nyes::Independent),
            value: n as i64,
        })),
        Astn::UnknownLit => Rc::new(RefCell::new(NkFir {
            core: ProtoBrane::new(vec![], parent.clone(), Nyes::Nk),
            reason: "??? literal".to_string(),
        })),
        Astn::Identifier { id, .. } => Rc::new(RefCell::new(SearchFir {
            core: ProtoBrane::new(vec![], parent.clone(), Nyes::Prembrionic),
            pattern: format!("^{}$", id),
            anchored: false,
            found_body: RefCell::new(None),
        })),
        Astn::Brane {
            characterizations: _,
            statements,
        } => Rc::new_cyclic(|me: &Weak<RefCell<BraneFir>>| {
            let self_weak: Weak<RefCell<dyn Fir>> = me.clone();
            let children = build_stmts(statements, &self_weak);
            RefCell::new(BraneFir {
                core: ProtoBrane::new(children, parent.clone(), Nyes::Prembrionic),
            })
        }),
        Astn::BinaryOp { op, left, right } => Rc::new_cyclic(|me: &Weak<RefCell<OperatorFir>>| {
            let self_weak: Weak<RefCell<dyn Fir>> = me.clone();
            let l = build_astn(*left, &self_weak);
            let r = build_astn(*right, &self_weak);
            RefCell::new(OperatorFir {
                core: ProtoBrane::new(vec![l, r], parent.clone(), Nyes::Prembrionic),
                op,
            })
        }),
        Astn::UnaryOp { op, expr } => Rc::new_cyclic(|me: &Weak<RefCell<OperatorFir>>| {
            let self_weak: Weak<RefCell<dyn Fir>> = me.clone();
            let e = build_astn(*expr, &self_weak);
            RefCell::new(OperatorFir {
                core: ProtoBrane::new(vec![e], parent.clone(), Nyes::Prembrionic),
                op,
            })
        }),
        Astn::DotSearch { anchor, coordinate } => {
            Rc::new_cyclic(|me: &Weak<RefCell<SearchFir>>| {
                let self_weak: Weak<RefCell<dyn Fir>> = me.clone();
                let a = build_astn(*anchor, &self_weak);
                RefCell::new(SearchFir {
                    core: ProtoBrane::new(vec![a], parent.clone(), Nyes::Prembrionic),
                    pattern: format!("^{}$", coordinate),
                    anchored: true,
                    found_body: RefCell::new(None),
                })
            })
        }
        Astn::RegexpSearch {
            anchor, pattern, ..
        } => Rc::new_cyclic(|me: &Weak<RefCell<SearchFir>>| {
            let self_weak: Weak<RefCell<dyn Fir>> = me.clone();
            let a = build_astn(*anchor, &self_weak);
            RefCell::new(SearchFir {
                core: ProtoBrane::new(vec![a], parent.clone(), Nyes::Prembrionic),
                pattern,
                anchored: true,
                found_body: RefCell::new(None),
            })
        }),
        Astn::Seek { anchor, offset } => Rc::new_cyclic(|me: &Weak<RefCell<IndexFir>>| {
            let self_weak: Weak<RefCell<dyn Fir>> = me.clone();
            let a = build_astn(*anchor, &self_weak);
            RefCell::new(IndexFir {
                core: ProtoBrane::new(vec![a], parent.clone(), Nyes::Prembrionic),
                offset,
                anchored: true,
            })
        }),
        Astn::HeadTail { is_head, anchor } => Rc::new_cyclic(|me: &Weak<RefCell<HeadTailFir>>| {
            let self_weak: Weak<RefCell<dyn Fir>> = me.clone();
            let a = build_astn(*anchor, &self_weak);
            RefCell::new(HeadTailFir {
                core: ProtoBrane::new(vec![a], parent.clone(), Nyes::Prembrionic),
                is_head,
                anchored: true,
            })
        }),
        Astn::UnanchoredSeek { offset } => Rc::new(RefCell::new(IndexFir {
            core: ProtoBrane::new(vec![], parent.clone(), Nyes::Prembrionic),
            offset,
            anchored: false,
        })),
        Astn::Concatenation { elements } => {
            Rc::new_cyclic(|me: &Weak<RefCell<ConcatenationFir>>| {
                let self_weak: Weak<RefCell<dyn Fir>> = me.clone();
                let children: Vec<FirRef> = elements
                    .into_iter()
                    .map(|e| build_astn(e, &self_weak))
                    .collect();
                RefCell::new(ConcatenationFir {
                    core: ProtoBrane::new(children, parent.clone(), Nyes::Prembrionic),
                })
            })
        }
        Astn::StayFoolish { expr } => Rc::new_cyclic(|me: &Weak<RefCell<StayFoolishFir>>| {
            let self_weak: Weak<RefCell<dyn Fir>> = me.clone();
            let e = build_astn(*expr, &self_weak);
            RefCell::new(StayFoolishFir {
                core: ProtoBrane::new(vec![e], parent.clone(), Nyes::Prembrionic),
            })
        }),
        Astn::StayFullyFoolish { expr } => {
            Rc::new_cyclic(|me: &Weak<RefCell<StayFullyFoolishFir>>| {
                let self_weak: Weak<RefCell<dyn Fir>> = me.clone();
                let e = build_astn(*expr, &self_weak);
                RefCell::new(StayFullyFoolishFir {
                    core: ProtoBrane::new(vec![e], parent.clone(), Nyes::Prembrionic),
                })
            })
        }
        other => build_standalone(other),
    }
}

fn build_stmts(asts: Vec<Astn>, parent: &Weak<RefCell<dyn Fir>>) -> Vec<FirRef> {
    asts.into_iter()
        .enumerate()
        .map(|(i, ast)| build_as_statement(ast, parent, i))
        .collect()
}

fn build_as_statement(ast: Astn, parent: &Weak<RefCell<dyn Fir>>, line: usize) -> FirRef {
    match ast {
        Astn::Assignment {
            identifier,
            operator,
            expr,
            ..
        } => Rc::new_cyclic(move |me: &Weak<RefCell<StatementFir>>| {
            let stmt_weak: Weak<RefCell<dyn Fir>> = me.clone();
            let body = build_expr_with_operator(*expr, operator, &stmt_weak);
            RefCell::new(StatementFir {
                core: ProtoBrane::new(vec![body], parent.clone(), Nyes::Prembrionic),
                name: identifier,
                line_number: line,
            })
        }),
        other => Rc::new_cyclic(move |me: &Weak<RefCell<StatementFir>>| {
            let stmt_weak: Weak<RefCell<dyn Fir>> = me.clone();
            let body = build_astn(other, &stmt_weak);
            RefCell::new(StatementFir {
                core: ProtoBrane::new(vec![body], parent.clone(), Nyes::Prembrionic),
                name: String::new(),
                line_number: line,
            })
        }),
    }
}

fn build_expr_with_operator(
    expr: Astn,
    operator: AssignmentOperator,
    parent: &Weak<RefCell<dyn Fir>>,
) -> FirRef {
    let body = build_astn(expr, parent);
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
