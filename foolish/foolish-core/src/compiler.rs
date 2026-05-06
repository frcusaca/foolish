use anyhow::anyhow;
use foolish_parser::Astn;
use crate::fir::{Fir, Nyes, SearchDirection, StatementFir};

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
            Astn::IntLit(n) => Ok(Fir::ConstantInt {
                value: n as i64,
                state: Nyes::Independent,
            }),

            Astn::UnknownLit => Ok(Fir::Nk {
                reason: "??? literal".to_string(),
                state: Nyes::Nk,
            }),

            Astn::Identifier { id, .. } => {
                Ok(Fir::Search {
                    pattern: format!("^{}$", id),
                    direction: SearchDirection::Backward,
                    anchored: false,
                    anchor: None,
                    target: None,
                    state: Nyes::Embryonic,
                })
            }

            Astn::Brane { characterizations, statements } => {
                let mut stmt_firs = Vec::new();
                for stmt in statements {
                    match stmt {
                        Astn::Assignment { identifier, operator, expr, .. } => {
                            let body = Self::compile_astn(*expr)?;
                            let name = match operator {
                                foolish_parser::AssignmentOperator::Assign => Some(identifier),
                                foolish_parser::AssignmentOperator::SF => {
                                    return Err(anyhow!("SF assignment (<=>): deferred to Phase 7"))
                                }
                                foolish_parser::AssignmentOperator::SFF => {
                                    return Err(anyhow!("SFF assignment (<<=>>>): deferred to Phase 7"))
                                }
                            };
                            let state = if body.state().is_constanic() {
                                body.state()
                            } else {
                                Nyes::Embryonic
                            };
                            stmt_firs.push(StatementFir {
                                name,
                                body,
                                state,
                            });
                        }
                        other => {
                            let body = Self::compile_astn(other)?;
                            stmt_firs.push(StatementFir::anonymous(body));
                        }
                    }
                }
                Ok(Fir::NormalBrane {
                    characterizations,
                    statements: stmt_firs,
                    state: Nyes::Embryonic,
                })
            }

            Astn::Assignment { identifier, operator, expr, .. } => {
                let body = Self::compile_astn(*expr)?;
                let name = match operator {
                    foolish_parser::AssignmentOperator::Assign => Some(identifier),
                    foolish_parser::AssignmentOperator::SF => {
                        return Err(anyhow!("SF assignment (<=>): deferred to Phase 7"))
                    }
                    foolish_parser::AssignmentOperator::SFF => {
                        return Err(anyhow!("SFF assignment (<<=>>>): deferred to Phase 7"))
                    }
                };
                let state = if body.state().is_constanic() {
                    body.state()
                } else {
                    Nyes::Embryonic
                };
                Ok(Fir::NormalBrane {
                    characterizations: vec![],
                    statements: vec![StatementFir {
                        name,
                        body,
                        state,
                    }],
                    state: Nyes::Embryonic,
                })
            }

            Astn::BinaryOp { op, left, right } => {
                let left_fir = Self::compile_astn(*left)?;
                let right_fir = Self::compile_astn(*right)?;
                Ok(Fir::BinaryOp {
                    op,
                    left: Box::new(left_fir),
                    right: Box::new(right_fir),
                    state: Nyes::Embryonic,
                })
            }

            Astn::UnaryOp { op, expr } => {
                let expr_fir = Self::compile_astn(*expr)?;
                Ok(Fir::UnaryOp {
                    op,
                    expr: Box::new(expr_fir),
                    state: Nyes::Embryonic,
                })
            }

            Astn::DotSearch { anchor, coordinate } => {
                let anchor_fir = Self::compile_astn(*anchor)?;
                Ok(Fir::Search {
                    pattern: format!("^{}$", coordinate),
                    direction: SearchDirection::Backward,
                    anchored: true,
                    anchor: Some(Box::new(anchor_fir)),
                    target: None,
                    state: Nyes::Embryonic,
                })
            }

            Astn::RegexpSearch { anchor, operator, pattern } => {
                let anchor_fir = Self::compile_astn(*anchor)?;
                Ok(Fir::Search {
                    pattern,
                    direction: match operator {
                        foolish_parser::SearchOperator::RegexpLocal => SearchDirection::Backward,
                        foolish_parser::SearchOperator::RegexpForward => SearchDirection::Forward,
                        _ => SearchDirection::Backward,
                    },
                    anchored: true,
                    anchor: Some(Box::new(anchor_fir)),
                    target: None,
                    state: Nyes::Embryonic,
                })
            }

            Astn::Seek { anchor, offset } => {
                let anchor_fir = Self::compile_astn(*anchor)?;
                Ok(Fir::Index {
                    offset,
                    anchored: true,
                    anchor: Some(Box::new(anchor_fir)),
                    state: Nyes::Embryonic,
                })
            }

            Astn::HeadTail { is_head, anchor } => {
                let anchor_fir = Self::compile_astn(*anchor)?;
                Ok(Fir::HeadTail {
                    is_head,
                    anchored: true,
                    anchor: Some(Box::new(anchor_fir)),
                    state: Nyes::Embryonic,
                })
            }

            Astn::UnanchoredSeek { offset } => {
                Ok(Fir::Index {
                    offset,
                    anchored: false,
                    anchor: None,
                    state: Nyes::Embryonic,
                })
            }

            Astn::Concatenation { .. } => {
                Err(anyhow!("Concatenation: deferred to Phase 3"))
            }

            Astn::IfExpr { .. } => {
                Err(anyhow!("if-then-else: not supported (FOOP=2)"))
            }

            Astn::UpwardSearch => {
                Err(anyhow!("Upward search (↑): deferred to Phase 7"))
            }

            Astn::StayFoolish { .. } => {
                Err(anyhow!("SF marker: deferred to Phase 7"))
            }

            Astn::StayFullyFoolish { .. } => {
                Err(anyhow!("SFF marker: deferred to Phase 7"))
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
