use anyhow::anyhow;
use std::cell::RefCell;
use std::rc::Rc;
use foolish_parser::Astn;
use crate::fir::{Fir, FirRef, Nyes, SearchDirection, StatementFir, Steppable,
    ConstantIntFir, NkFir, SearchFir, NormalBraneFir,
    OperatorFir, IndexFir, HeadTailFir,
    StayFoolishFir, StayFullyFoolishFir, ConcatenationFir, Alarm, AlarmLevel, AlarmSource};

/// Helper: wrap a Fir in a FirRef (Rc<RefCell<dyn Steppable>>)
fn to_ref(fir: Fir) -> FirRef { Rc::new(RefCell::new(fir)) }

pub struct Compiler;

impl Compiler {
    pub fn compile(source: &str) -> anyhow::Result<Vec<Fir>> {
        let asts = foolish_parser::parse(source)?;
        let mut firs = Vec::new();
        for ast in asts {
            firs.push(Self::compile_astn(ast)?);
        }
        Ok(firs)
    }

    pub fn compile_astn(ast: Astn) -> anyhow::Result<Fir> {
        match ast {
            Astn::IntLit(n) => Ok(Fir::ConstantInt(Box::new(ConstantIntFir {
                value: n as i64,
                state: Nyes::Independent,
            }))),

            Astn::UnknownLit => Ok(Fir::Nk(Box::new(NkFir {
                reason: "??? literal".to_string(),
                state: Nyes::Nk,
                alarm: None,
            }))),

            Astn::Identifier { id, .. } => {
                Ok(Fir::Search(Box::new(SearchFir {
                    pattern: format!("^{}$", id),
                    direction: SearchDirection::Backward,
                    anchored: false,
                    anchor: None,
                    target: None,
                    parent: None,
                    state: Nyes::Embryonic,
                })))
            }

            Astn::Brane { characterizations, statements } => {
                let mut stmt_firs = Vec::new();
                for stmt in statements {
                    match stmt {
                        Astn::Assignment { identifier, operator, expr, .. } => {
                            let body_fir = Self::compile_astn(*expr)?;
                            let (name, body, state) = match operator {
                                foolish_parser::AssignmentOperator::Assign => {
                                    let state = if body_fir.state().is_constanic() {
                                        body_fir.state()
                                    } else {
                                        Nyes::Embryonic
                                    };
                                    (Some(identifier), to_ref(body_fir), state)
                                }
                                foolish_parser::AssignmentOperator::SF => {
                                    (
                                        Some(identifier),
                                        to_ref(Fir::StayFoolish(Box::new(StayFoolishFir {
                                            expr: to_ref(body_fir),
                                            state: Nyes::Embryonic,
                                        }))),
                                        Nyes::Embryonic,
                                    )
                                }
                                foolish_parser::AssignmentOperator::SFF => {
                                    (
                                        Some(identifier),
                                        to_ref(Fir::StayFullyFoolish(Box::new(StayFullyFoolishFir {
                                            expr: to_ref(body_fir),
                                            state: Nyes::Independent,
                                        }))),
                                        Nyes::Independent,
                                    )
                                }
                            };
                            stmt_firs.push(StatementFir { name, body, state });
                        }
                        other => {
                            let body_fir = Self::compile_astn(other)?;
                            stmt_firs.push(StatementFir::anonymous(to_ref(body_fir)));
                        }
                    }
                }
                Ok(Fir::NormalBrane(Box::new(NormalBraneFir {
                    characterizations,
                    statements: stmt_firs,
                    state: Nyes::Embryonic,
                    parent: None,
                })))
            }

            Astn::Assignment { identifier, operator, expr, .. } => {
                let body_fir = Self::compile_astn(*expr)?;
                let (name, body, state) = match operator {
                    foolish_parser::AssignmentOperator::Assign => {
                        let state = if body_fir.state().is_constanic() {
                            body_fir.state()
                        } else {
                            Nyes::Embryonic
                        };
                        (Some(identifier), to_ref(body_fir), state)
                    }
                    foolish_parser::AssignmentOperator::SF => {
                        (
                            Some(identifier),
                            to_ref(Fir::StayFoolish(Box::new(StayFoolishFir {
                                expr: to_ref(body_fir),
                                state: Nyes::Embryonic,
                            }))),
                            Nyes::Embryonic,
                        )
                    }
                    foolish_parser::AssignmentOperator::SFF => {
                        (
                            Some(identifier),
                            to_ref(Fir::StayFullyFoolish(Box::new(StayFullyFoolishFir {
                                expr: to_ref(body_fir),
                                state: Nyes::Independent,
                            }))),
                            Nyes::Independent,
                        )
                    }
                };
                Ok(Fir::NormalBrane(Box::new(NormalBraneFir {
                    characterizations: vec![],
                    statements: vec![StatementFir {
                        name,
                        body,
                        state,
                    }],
                    state: Nyes::Embryonic,
                    parent: None,
                })))
            }

            Astn::BinaryOp { op, left, right } => {
                let left_fir = Self::compile_astn(*left)?;
                let right_fir = Self::compile_astn(*right)?;
                Ok(Fir::Operator(Box::new(OperatorFir {
                    op,
                    operands: vec![to_ref(left_fir), to_ref(right_fir)],
                    state: Nyes::Embryonic,
                })))
            }

            Astn::UnaryOp { op, expr } => {
                let expr_fir = Self::compile_astn(*expr)?;
                Ok(Fir::Operator(Box::new(OperatorFir {
                    op,
                    operands: vec![to_ref(expr_fir)],
                    state: Nyes::Embryonic,
                })))
            }

            Astn::DotSearch { anchor, coordinate } => {
                let anchor_fir = Self::compile_astn(*anchor)?;
                Ok(Fir::Search(Box::new(SearchFir {
                    pattern: format!("^{}$", coordinate),
                    direction: SearchDirection::Backward,
                    anchored: true,
                    anchor: Some(to_ref(anchor_fir)),
                    target: None,
                    parent: None,
                    state: Nyes::Embryonic,
                })))
            }

            Astn::RegexpSearch { anchor, operator, pattern } => {
                let anchor_fir = Self::compile_astn(*anchor)?;
                Ok(Fir::Search(Box::new(SearchFir {
                    pattern,
                    direction: match operator {
                        foolish_parser::SearchOperator::RegexpLocal => SearchDirection::Backward,
                        foolish_parser::SearchOperator::RegexpForward => SearchDirection::Forward,
                        _ => SearchDirection::Backward,
                    },
                    anchored: true,
                    anchor: Some(to_ref(anchor_fir)),
                    target: None,
                    parent: None,
                    state: Nyes::Embryonic,
                })))
            }

            Astn::Seek { anchor, offset } => {
                let anchor_fir = Self::compile_astn(*anchor)?;
                Ok(Fir::Index(Box::new(IndexFir {
                    offset,
                    anchored: true,
                    anchor: Some(to_ref(anchor_fir)),
                    state: Nyes::Embryonic,
                })))
            }

            Astn::HeadTail { is_head, anchor } => {
                let anchor_fir = Self::compile_astn(*anchor)?;
                Ok(Fir::HeadTail(Box::new(HeadTailFir {
                    is_head,
                    anchored: true,
                    anchor: Some(to_ref(anchor_fir)),
                    state: Nyes::Embryonic,
                })))
            }

            Astn::UnanchoredSeek { offset } => {
                Ok(Fir::Index(Box::new(IndexFir {
                    offset,
                    anchored: false,
                    anchor: None,
                    state: Nyes::Embryonic,
                })))
            }

            Astn::Concatenation { elements } => {
                let refs: Vec<FirRef> = elements.into_iter()
                    .map(|e| {
                        let f = Self::compile_astn(e)?;
                        anyhow::Ok(to_ref(f))
                    })
                    .collect::<anyhow::Result<Vec<_>>>()?;
                Ok(Fir::Concatenation(Box::new(ConcatenationFir {
                    elements: refs,
                    merged: None,
                    state: Nyes::Embryonic,
                })))
            }

            Astn::IfExpr { .. } => {
                Err(anyhow!("if-then-else: not supported (FOOP=2)"))
            }

            Astn::UpwardSearch => {
                Err(anyhow!("Upward search (↑): deferred to Phase 7"))
            }

            Astn::StayFoolish { expr } => {
                let inner = Self::compile_astn(*expr)?;
                Ok(Fir::StayFoolish(Box::new(StayFoolishFir {
                    expr: to_ref(inner),
                    state: Nyes::Embryonic,
                })))
            }

            Astn::StayFullyFoolish { expr } => {
                let inner = Self::compile_astn(*expr)?;
                Ok(Fir::StayFullyFoolish(Box::new(StayFullyFoolishFir {
                    expr: to_ref(inner),
                    state: Nyes::Independent,
                })))
            }

            Astn::DetachmentBrane { .. } => {
                Err(anyhow!("Detachment brane: deferred to Phase 7"))
            }

            Astn::NotImplemented(reason) => {
                Err(anyhow!("Not yet implemented: {}", reason))
            }
        }
    }
}
