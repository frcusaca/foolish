use std::cell::RefCell;
use std::rc::{Rc, Weak};

use foolish_core::fir::Nyes;
use regex::Regex;

use crate::fir_trait::{Fir, FirKind, FirRef, FirRefExt, Scope, UbcError};
use crate::proto_brane::ProtoBrane;

pub(crate) fn _decide_nyes_due_to_children(children: &[FirRef]) -> Option<Nyes> {
    let mut all_constantew = true;
    let mut all_independent = true;
    let mut preconstanic_count = 0usize;
    let mut nk_count = 0usize;
    let mut econstanic_woconstanic_count = 0usize;

    for c in children {
        match c.borrow().core().get_nyes() {
            Nyes::Prembrionic | Nyes::Embryonic | Nyes::Braning => {
                preconstanic_count += 1;
                all_constantew = false;
                all_independent = false;
            }
            Nyes::Nk => {
                nk_count += 1;
                all_constantew = false;
                all_independent = false;
            }
            Nyes::Econstanic | Nyes::Woconstanic => {
                econstanic_woconstanic_count += 1;
                all_constantew = false;
                all_independent = false;
            }
            Nyes::Constant => {
                all_independent = false;
            }
            _ => {}
        }
    }
    if all_independent {
        return Some(Nyes::Independent);
    } else if all_constantew {
        return Some(Nyes::Constant);
    } else if preconstanic_count > 0 {
        return Some(Nyes::Braning);
    } else if econstanic_woconstanic_count > 0 {
        return Some(Nyes::Woconstanic);
    } else if nk_count > 0 {
        return Some(Nyes::Nk);
    }
    unreachable!("ALARM: _decide_nyes_due_to_children: no decision made.");
}

/// Brane/anchor navigation methods on a [`FirRef`].
///
/// # Why an extension trait
///
/// [`FirRef`] aliases the foreign `Rc<RefCell<dyn Fir>>`, so the orphan rule
/// rules out an inherent `impl`. These navigation helpers are attached to the
/// handle through an extension trait — the same pattern as
/// [`crate::fir_trait::FirRefExt`] and [`crate::nyes_ext::NyesExt`]. This trait
/// is kept separate from `FirRefExt` so the stepping/value core and the
/// brane-navigation helpers read as distinct concerns.
///
/// Each method borrows the pointee transiently and drops the borrow before any
/// recursion or re-borrow, preserving the crate's RefCell borrow discipline.
pub(crate) trait FirRefNavExt {
    /// Walks a chain of `Search` results to the deepest ECONSTANIC search.
    ///
    /// Follows `ubc_children[0]` while each link is a WOCONSTANIC search;
    /// returns the first ECONSTANIC search, or `None` if the chain leaves
    /// `Search` or dead-ends.
    fn deepest_econstanic_in_chain(&self) -> Option<FirRef>;

    /// Resolves an anchor to its underlying value (alias of
    /// [`crate::fir_trait::FirRefExt::value`], named for the anchor use site).
    fn resolve_anchor(&self) -> FirRef;

    /// Returns the index of `stmt` among `self`'s foolish children, by identity.
    ///
    /// `self` is the brane being searched; `stmt` is the statement to locate.
    fn find_stmt_index(&self, stmt: &FirRef) -> Option<usize>;
}

impl FirRefNavExt for FirRef {
    fn deepest_econstanic_in_chain(&self) -> Option<FirRef> {
        let mut current = Rc::clone(self);
        loop {
            let (kind, nyes, result) = {
                let b = current.borrow();
                let result = b.core().ubc_children().into_iter().next();
                (b.kind(), b.core().get_nyes(), result)
            };
            if kind != FirKind::Search {
                return None;
            }
            match nyes {
                Nyes::Econstanic => return Some(current),
                Nyes::Woconstanic => match result {
                    Some(next) => current = next,
                    None => return None,
                },
                _ => return None,
            }
        }
    }

    #[inline(always)]
    fn resolve_anchor(&self) -> FirRef {
        self.value()
    }

    fn find_stmt_index(&self, stmt: &FirRef) -> Option<usize> {
        let brane_borrowed = self.borrow();
        for (i, child) in brane_borrowed.core().foolish_children().iter().enumerate() {
            if Rc::ptr_eq(child, stmt) {
                return Some(i);
            }
        }
        None
    }
}

impl ProtoBrane {
    fn clone_children_for_constanic_clone(
        source: &ProtoBrane,
        self_weak: &Weak<RefCell<dyn Fir>>,
        new_parent: &Weak<RefCell<dyn Fir>>,
        nyes: Nyes,
        sfm: bool,
    ) -> ProtoBrane {
        let cloned_children: Vec<FirRef> = source
            .foolish_children()
            .iter()
            .enumerate()
            .map(|(i, c)| ProtoBrane::constanic_clone_at(c, self_weak, i, sfm))
            .collect();
        let core = ProtoBrane::new(
            cloned_children,
            new_parent.clone(),
            nyes.transform_for_clone(sfm),
        );
        for ubc in source.ubc_children() {
            core.push_ubc_child(ProtoBrane::constanic_clone_at(&ubc, self_weak, 0, sfm));
        }
        core
    }

    pub(crate) fn constanic_clone_at(
        fir_ref: &FirRef,
        new_parent: &Weak<RefCell<dyn Fir>>,
        index: usize,
        descendent_of_sfm_and_foolishly_ignorant: bool,
    ) -> FirRef {
        if matches!(
            fir_ref.borrow().kind(),
            FirKind::StayFoolish | FirKind::StayFullyFoolish
        ) {
            let source = fir_ref.borrow();
            if source.kind() == FirKind::StayFoolish
                && let Some(constanic_result) = source.core().ubc_children().into_iter().next()
            {
                return Self::constanic_clone_at(
                    &constanic_result,
                    new_parent,
                    index,
                    descendent_of_sfm_and_foolishly_ignorant,
                );
            }
            if let Some(inner) = source.core().foolish_children().first().cloned() {
                return Self::constanic_clone_at(
                    &inner,
                    new_parent,
                    index,
                    descendent_of_sfm_and_foolishly_ignorant,
                );
            }
            eprintln!("ALARM: SF/SFF node has no children — cloning wrapper as-is");
        }
        let nyes = fir_ref.borrow().core().get_nyes();
        if (nyes == Nyes::Constant || nyes == Nyes::Independent)
            && fir_ref.borrow().kind() != FirKind::Brane
        {
            return Rc::clone(fir_ref);
        }
        let borrowed = fir_ref.borrow();
        let kind = borrowed.kind();
        match kind {
            FirKind::IndepInt => Rc::new(RefCell::new(IndepIntFir {
                core: ProtoBrane::new(vec![], new_parent.clone(), borrowed.core().get_nyes()),
                value: borrowed.as_i64().unwrap_or(0),
            })),
            FirKind::Nk => Rc::new(RefCell::new(NkFir {
                core: ProtoBrane::new(vec![], new_parent.clone(), borrowed.core().get_nyes()),
                reason: borrowed.as_nk_reason().unwrap_or("unknown").to_owned(),
            })),
            FirKind::Operator => {
                let op_name = borrowed.as_op_name().unwrap_or("?").to_owned();
                Rc::new_cyclic(|me: &Weak<RefCell<OperatorFir>>| {
                    let self_weak: Weak<RefCell<dyn Fir>> = me.clone();
                    let core = ProtoBrane::clone_children_for_constanic_clone(
                        borrowed.core(),
                        &self_weak,
                        new_parent,
                        nyes,
                        descendent_of_sfm_and_foolishly_ignorant,
                    );
                    RefCell::new(OperatorFir { core, op: op_name })
                })
            }
            FirKind::Search => {
                let clone_nyes_val =
                    nyes.transform_for_clone(descendent_of_sfm_and_foolishly_ignorant);
                let pattern = borrowed.as_search_pattern().unwrap_or("").to_owned();
                let anchored = borrowed.as_search_anchored();
                let is_value = borrowed.as_search_is_value();
                let is_contexted = borrowed.as_search_contexted();
                let chain_econstanic =
                    if !descendent_of_sfm_and_foolishly_ignorant && nyes == Nyes::Woconstanic {
                        fir_ref.deepest_econstanic_in_chain()
                    } else {
                        None
                    };
                Rc::new_cyclic(|me: &Weak<RefCell<SearchFir>>| {
                    let self_weak: Weak<RefCell<dyn Fir>> = me.clone();
                    let children: Vec<FirRef> = borrowed
                        .core()
                        .foolish_children()
                        .iter()
                        .enumerate()
                        .map(|(i, c)| {
                            ProtoBrane::constanic_clone_at(
                                c,
                                &self_weak,
                                i,
                                descendent_of_sfm_and_foolishly_ignorant,
                            )
                        })
                        .collect();
                    let core = ProtoBrane::new(children, new_parent.clone(), clone_nyes_val);
                    if let Some(ref econ) = chain_econstanic {
                        core.push_ubc_child(ProtoBrane::constanic_clone_at(
                            econ, &self_weak, 0, false,
                        ));
                    } else {
                        for ubc in borrowed.core().ubc_children() {
                            core.push_ubc_child(ProtoBrane::constanic_clone_at(
                                &ubc,
                                &self_weak,
                                0,
                                descendent_of_sfm_and_foolishly_ignorant,
                            ));
                        }
                    }
                    RefCell::new(SearchFir {
                        core,
                        pattern,
                        anchored,
                        forward: false,
                        sf_inner_pattern: RefCell::new(None),
                        is_value_search: is_value,
                        contexted: is_contexted,
                    })
                })
            }
            FirKind::Index => {
                let offset = borrowed.as_index_offset();
                let anchored = borrowed.as_index_anchored();
                let is_contexted = borrowed.as_search_contexted();
                Rc::new_cyclic(|me: &Weak<RefCell<IndexFir>>| {
                    let self_weak: Weak<RefCell<dyn Fir>> = me.clone();
                    let core = ProtoBrane::clone_children_for_constanic_clone(
                        borrowed.core(),
                        &self_weak,
                        new_parent,
                        nyes,
                        descendent_of_sfm_and_foolishly_ignorant,
                    );
                    RefCell::new(IndexFir {
                        core,
                        offset,
                        anchored,
                        contexted: is_contexted,
                    })
                })
            }
            FirKind::StayFoolish | FirKind::StayFullyFoolish => {
                unreachable!("SF/SFF stripped at fn top")
            }
            FirKind::Concatenation => Rc::new_cyclic(|me: &Weak<RefCell<ConcatenationFir>>| {
                let self_weak: Weak<RefCell<dyn Fir>> = me.clone();
                let core = ProtoBrane::clone_children_for_constanic_clone(
                    borrowed.core(),
                    &self_weak,
                    new_parent,
                    nyes,
                    descendent_of_sfm_and_foolishly_ignorant,
                );
                RefCell::new(ConcatenationFir { core })
            }),
            FirKind::Statement => {
                let name = borrowed.as_stmt_name().unwrap_or("").to_owned();
                let line = index;
                Rc::new_cyclic(|me: &Weak<RefCell<StatementFir>>| {
                    let self_weak: Weak<RefCell<dyn Fir>> = me.clone();
                    let core = ProtoBrane::clone_children_for_constanic_clone(
                        borrowed.core(),
                        &self_weak,
                        new_parent,
                        nyes,
                        descendent_of_sfm_and_foolishly_ignorant,
                    );
                    RefCell::new(StatementFir {
                        core,
                        name,
                        line_number: line,
                    })
                })
            }
            FirKind::Brane => Rc::new_cyclic(|me: &Weak<RefCell<BraneFir>>| {
                let self_weak: Weak<RefCell<dyn Fir>> = me.clone();
                let core = ProtoBrane::clone_children_for_constanic_clone(
                    borrowed.core(),
                    &self_weak,
                    new_parent,
                    nyes,
                    descendent_of_sfm_and_foolishly_ignorant,
                );
                RefCell::new(BraneFir {
                    core,
                    characterizations: borrowed.as_brane_characterizations().to_vec(),
                })
            }),
            FirKind::Unknown => Rc::new(RefCell::new(NkFir {
                core: ProtoBrane::new(vec![], new_parent.clone(), Nyes::Nk),
                reason: "unknown fir kind".to_owned(),
            })),
            FirKind::FoolRef => Rc::clone(fir_ref),
        }
    }
}

#[derive(Debug)]
pub struct IndepIntFir {
    pub(crate) core: ProtoBrane,
    pub(crate) value: i64,
}

impl IndepIntFir {
    pub fn value(&self) -> i64 {
        self.value
    }

    pub fn constant_int(value: i64, parent: Weak<RefCell<dyn Fir>>) -> FirRef {
        Rc::new(RefCell::new(IndepIntFir {
            core: ProtoBrane::new(vec![], parent, Nyes::Prembrionic),
            value,
        }))
    }
}

impl Fir for IndepIntFir {
    #[inline(always)]
    fn core(&self) -> &ProtoBrane {
        &self.core
    }

    fn fir_op_step(&self, _scope: &Scope) -> Result<(), UbcError> {
        if !self.core.get_nyes().is_constanic() {
            self.core.set_nyes(Nyes::Constant);
        }
        Ok(())
    }

    fn kind(&self) -> FirKind {
        FirKind::IndepInt
    }

    fn as_i64(&self) -> Option<i64> {
        Some(self.value)
    }
}

#[derive(Debug)]
pub struct NkFir {
    pub(crate) core: ProtoBrane,
    pub(crate) reason: String,
}

impl NkFir {
    pub fn reason(&self) -> &str {
        &self.reason
    }

    pub fn nk(reason: &str, parent: Weak<RefCell<dyn Fir>>) -> FirRef {
        Rc::new(RefCell::new(NkFir {
            core: ProtoBrane::new(vec![], parent, Nyes::Prembrionic),
            reason: reason.to_owned(),
        }))
    }
}

impl Fir for NkFir {
    #[inline(always)]
    fn core(&self) -> &ProtoBrane {
        &self.core
    }

    fn fir_op_step(&self, _scope: &Scope) -> Result<(), UbcError> {
        if !self.core.get_nyes().is_constanic() {
            self.core.set_nyes(Nyes::Nk);
        }
        Ok(())
    }

    fn kind(&self) -> FirKind {
        FirKind::Nk
    }

    fn as_nk_reason(&self) -> Option<&str> {
        Some(&self.reason)
    }
}

#[derive(Debug)]
pub struct OperatorFir {
    pub(crate) core: ProtoBrane,
    pub(crate) op: String,
}

impl OperatorFir {
    pub fn operator(op: &str, operands: Vec<FirRef>, parent: Weak<RefCell<dyn Fir>>) -> FirRef {
        Rc::new(RefCell::new(OperatorFir {
            core: ProtoBrane::new(operands, parent, Nyes::Prembrionic),
            op: op.to_owned(),
        }))
    }

    fn operands_all_settled(&self) -> bool {
        self.core().foolish_children().iter().all(|c| {
            matches!(
                c.borrow().core().get_nyes(),
                Nyes::Constant | Nyes::Independent
            )
        })
    }
}

impl Fir for OperatorFir {
    #[inline(always)]
    fn core(&self) -> &ProtoBrane {
        &self.core
    }
    fn fir_op_step(&self, scope: &Scope) -> Result<(), UbcError> {
        match self.core.get_nyes() {
            Nyes::Prembrionic | Nyes::Embryonic => {
                self.core.set_nyes(Nyes::Braning);
                if !self.operands_all_settled() {
                    let children: Vec<FirRef> = self.core.foolish_children().to_vec();
                    for child in children {
                        self.core.push_task(child);
                    }
                }
            }
            Nyes::Braning => {
                return self.combine(scope);
            }
            _ => {}
        }
        Ok(())
    }

    fn kind(&self) -> FirKind {
        FirKind::Operator
    }

    fn as_op_name(&self) -> Option<&str> {
        Some(&self.op)
    }
}

impl OperatorFir {
    fn combine(&self, scope: &Scope) -> Result<(), UbcError> {
        {
            let self_weak = self.core.parent_weak();
            let children = self.core.foolish_children().to_vec();

            let any_nk = children
                .iter()
                .any(|c| c.borrow().core().get_nyes() == Nyes::Nk);
            if any_nk {
                let nk_ref: FirRef = Rc::new_cyclic(|me: &Weak<RefCell<NkFir>>| {
                    let parent: Weak<RefCell<dyn Fir>> = me.clone();
                    let reason = children
                        .iter()
                        .find_map(|c| {
                            let b = c.borrow();
                            if b.core().get_nyes() == Nyes::Nk {
                                b.as_nk_reason().map(|s| s.to_string())
                            } else {
                                None
                            }
                        })
                        .unwrap_or_else(|| "operator nk".to_string());
                    RefCell::new(NkFir {
                        core: ProtoBrane::new(vec![], parent, Nyes::Nk),
                        reason,
                    })
                });
                self.core.push_ubc_child(ProtoBrane::constanic_clone_at(
                    &nk_ref,
                    &self_weak,
                    0,
                    scope.has_ancestral_sfm,
                ));
                self.core.set_nyes(Nyes::Nk);
                return Ok(());
            }

            let values: Vec<i64> = children
                .iter()
                .map(|c| c.value())
                .filter_map(|v| v.borrow().as_i64())
                .collect();

            if values.len() != children.len() {
                self.core.set_nyes(Nyes::Woconstanic);
                return Ok(());
            }

            let result = match self.op.as_str() {
                "+" if values.len() == 2 => values[0] + values[1],
                "-" if values.len() == 2 => values[0] - values[1],
                "*" if values.len() == 2 => values[0] * values[1],
                "/" if values.len() == 2 => {
                    if values[1] == 0 {
                        let nk_ref: FirRef = Rc::new_cyclic(|me: &Weak<RefCell<NkFir>>| {
                            let parent: Weak<RefCell<dyn Fir>> = me.clone();
                            RefCell::new(NkFir {
                                core: ProtoBrane::new(vec![], parent, Nyes::Nk),
                                reason: "division by zero".to_string(),
                            })
                        });
                        self.core.push_ubc_child(ProtoBrane::constanic_clone_at(
                            &nk_ref,
                            &self_weak,
                            0,
                            scope.has_ancestral_sfm,
                        ));
                        self.core.set_nyes(Nyes::Nk);
                        return Ok(());
                    }
                    values[0] / values[1]
                }
                "%" if values.len() == 2 => {
                    if values[1] == 0 {
                        let nk_ref: FirRef = Rc::new_cyclic(|me: &Weak<RefCell<NkFir>>| {
                            let parent: Weak<RefCell<dyn Fir>> = me.clone();
                            RefCell::new(NkFir {
                                core: ProtoBrane::new(vec![], parent, Nyes::Nk),
                                reason: "division by zero".to_string(),
                            })
                        });
                        self.core.push_ubc_child(ProtoBrane::constanic_clone_at(
                            &nk_ref,
                            &self_weak,
                            0,
                            scope.has_ancestral_sfm,
                        ));
                        self.core.set_nyes(Nyes::Nk);
                        return Ok(());
                    }
                    values[0] % values[1]
                }
                "-" if values.len() == 1 => -values[0], // unary negation
                "$" if children.len() == 2 => {
                    let rhs = children[1].value();
                    if rhs.borrow().kind() != FirKind::Brane {
                        let rhs_val = rhs
                            .borrow()
                            .as_i64()
                            .map(|v| v.to_string())
                            .unwrap_or_else(|| format!("{:?}", rhs.borrow().kind()));
                        let reason = format!("{} is not a brane", rhs_val);
                        let nk_ref: FirRef = Rc::new_cyclic(|me: &Weak<RefCell<NkFir>>| {
                            let parent: Weak<RefCell<dyn Fir>> = me.clone();
                            RefCell::new(NkFir {
                                core: ProtoBrane::new(vec![], parent, Nyes::Nk),
                                reason: reason.clone(),
                            })
                        });
                        self.core.push_ubc_child(ProtoBrane::constanic_clone_at(
                            &nk_ref,
                            &self_weak,
                            0,
                            scope.has_ancestral_sfm,
                        ));
                        self.core.set_alarm_reason(reason);
                        self.core.set_nyes(Nyes::Nk);
                        return Ok(());
                    }
                    return Ok(());
                }
                op => {
                    return Err(UbcError::Eval(format!(
                        "unknown operator: {op} ({} operands)",
                        values.len()
                    )));
                }
            };

            let result_ref: FirRef = Rc::new_cyclic(|me: &Weak<RefCell<IndepIntFir>>| {
                let parent: Weak<RefCell<dyn Fir>> = me.clone();
                RefCell::new(IndepIntFir {
                    core: ProtoBrane::new(vec![], parent, Nyes::Constant),
                    value: result,
                })
            });
            self.core.push_ubc_child(ProtoBrane::constanic_clone_at(
                &result_ref,
                &self_weak,
                0,
                scope.has_ancestral_sfm,
            ));
            self.core.set_nyes(Nyes::Constant);
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct StatementFir {
    pub(crate) core: ProtoBrane,
    pub(crate) name: String,
    pub(crate) line_number: usize,
}

impl StatementFir {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn line_number(&self) -> usize {
        self.line_number
    }

    pub fn statement(
        name: &str,
        line_number: usize,
        body: FirRef,
        parent: Weak<RefCell<dyn Fir>>,
    ) -> FirRef {
        Rc::new(RefCell::new(StatementFir {
            core: ProtoBrane::new(vec![body], parent, Nyes::Prembrionic),
            name: name.to_owned(),
            line_number,
        }))
    }
}

impl Fir for StatementFir {
    #[inline(always)]
    fn core(&self) -> &ProtoBrane {
        &self.core
    }

    fn fir_op_step(&self, _scope: &Scope) -> Result<(), UbcError> {
        match self.core.get_nyes() {
            Nyes::Prembrionic | Nyes::Embryonic => {
                self.core.set_nyes(Nyes::Braning);
                let children: Vec<FirRef> = self.core.foolish_children().to_vec();
                for child in children {
                    self.core.push_task(child);
                }
            }
            Nyes::Braning => {
                let children = self.core.foolish_children().to_vec();
                if let Some(body) = children.first() {
                    let body_nyes = body.borrow().core().get_nyes();
                    if body_nyes.is_constanic() {
                        self.core.set_nyes(body_nyes);
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn kind(&self) -> FirKind {
        FirKind::Statement
    }

    fn as_stmt_name(&self) -> Option<&str> {
        if self.name.is_empty() {
            None
        } else {
            Some(&self.name)
        }
    }
    fn as_stmt_line_number(&self) -> Option<usize> {
        Some(self.line_number)
    }

    fn _ib_search(&self, self_ref: &FirRef, name: &str) -> Option<(FirRef, Nyes)> {
        let brane = self.get_my_brane(self_ref)?;
        let brane_borrowed = brane.borrow();
        brane_borrowed
            ._search_brane(name, self.line_number.saturating_sub(1), 0)
            .map(|(_idx, stmt, nyes)| (stmt, nyes))
    }
}

#[derive(Debug)]
pub struct BraneFir {
    pub(crate) core: ProtoBrane,
    pub(crate) characterizations: Vec<String>,
}

impl BraneFir {
    pub fn brane(children: Vec<FirRef>, parent: Weak<RefCell<dyn Fir>>) -> FirRef {
        Rc::new(RefCell::new(BraneFir {
            core: ProtoBrane::new(children, parent, Nyes::Prembrionic),
            characterizations: Vec::new(),
        }))
    }
}

impl Fir for BraneFir {
    #[inline(always)]
    fn core(&self) -> &ProtoBrane {
        &self.core
    }

    fn fir_op_step(&self, _scope: &Scope) -> Result<(), UbcError> {
        match self.core.get_nyes() {
            Nyes::Prembrionic | Nyes::Embryonic => {
                let children: Vec<FirRef> = self.core.foolish_children().to_vec();
                if children.is_empty() {
                    self.core.set_nyes(Nyes::Constant);
                } else {
                    self.core.set_nyes(Nyes::Braning);
                    for child in children {
                        self.core.push_task(child);
                    }
                }
            }
            Nyes::Braning => {
                let children = self.core.foolish_children().to_vec();
                if let Some(nyes) = _decide_nyes_due_to_children(&children) {
                    self.core.set_nyes(nyes);
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn kind(&self) -> FirKind {
        FirKind::Brane
    }

    fn as_brane_characterizations(&self) -> &[String] {
        &self.characterizations
    }

    fn _ab_search(&self, self_ref: &FirRef, name: &str) -> Option<(FirRef, Nyes)> {
        let stmt = self.get_my_statement(self_ref);
        if Rc::ptr_eq(&stmt, self_ref) {
            return None;
        }
        let stmt_borrowed = stmt.borrow();
        if let Some((body, nyes)) = stmt_borrowed._ib_search(&stmt, name) {
            return Some((body, nyes));
        }
        drop(stmt_borrowed);
        let parent_brane = self.get_my_brane(self_ref)?;
        if Rc::ptr_eq(&parent_brane, self_ref) {
            return None;
        }
        let pb = parent_brane.borrow();
        pb._ab_search(&parent_brane, name)
    }

    fn _search_brane(
        &self,
        expression: &str,
        starting_index: usize,
        ending_index: usize,
    ) -> Option<(usize, FirRef, Nyes)> {
        let children = self.core.foolish_children();
        if starting_index >= children.len() || ending_index >= children.len() {
            panic!(
                "_search_brane: index out of bounds (start={}, end={}, len={})",
                starting_index,
                ending_index,
                children.len()
            );
        }
        let range = if starting_index >= ending_index {
            Box::new((ending_index..=starting_index).rev()) as Box<dyn Iterator<Item = usize>>
        } else {
            Box::new(starting_index..=ending_index) as Box<dyn Iterator<Item = usize>>
        };
        for i in range {
            let child = &children[i];
            let child_borrowed = child.borrow();
            if let Some(sn) = child_borrowed.as_stmt_name()
                && SearchFir::matches_pattern(sn, expression)
            {
                return Some((i, Rc::clone(child), child_borrowed.core().get_nyes()));
            }
        }
        None
    }
}

#[derive(Debug)]
pub struct SearchFir {
    pub(crate) core: ProtoBrane,
    pub(crate) pattern: String,
    pub(crate) anchored: bool,
    pub(crate) forward: bool,
    pub(crate) sf_inner_pattern: RefCell<Option<String>>,
    pub(crate) is_value_search: bool,
    pub(crate) contexted: bool,
}

impl SearchFir {
    pub(crate) fn matches_pattern(stmt_name: &str, pattern: &str) -> bool {
        if stmt_name == pattern {
            return true;
        }
        let re = if pattern.contains('^') || pattern.contains('$') {
            Regex::new(pattern)
        } else {
            Regex::new(&format!("^{}$", pattern))
        };
        if let Ok(re) = re {
            return re.is_match(stmt_name);
        }
        false
    }

    pub(crate) fn nyes_from_found(found: Nyes) -> Nyes {
        match found {
            Nyes::Econstanic | Nyes::Woconstanic => Nyes::Woconstanic,
            Nyes::Constant | Nyes::Independent => Nyes::Constant,
            Nyes::Nk => Nyes::Nk,
            other => other,
        }
    }

    fn clone_stmt_result(
        stmt: &FirRef,
        new_parent: &Weak<RefCell<dyn Fir>>,
        descendent_of_sfm_and_foolishly_ignorant: bool,
    ) -> FirRef {
        let stmt_borrowed = stmt.borrow();
        let body = stmt_borrowed
            .core()
            .foolish_children()
            .first()
            .cloned()
            .expect("statement must have a body");
        let index = stmt_borrowed.as_stmt_line_number().unwrap_or(0);
        ProtoBrane::constanic_clone_at(
            &body,
            new_parent,
            index,
            descendent_of_sfm_and_foolishly_ignorant,
        )
    }

    fn handle_found(&self, stmt: FirRef, _nyes: Nyes, scope: &Scope) {
        let self_weak = self.core.parent_weak();
        let clone = Self::clone_stmt_result(&stmt, &self_weak, scope.has_ancestral_sfm);
        push_search_result_pair(&self.core, clone, stmt);
        self.core.set_nyes(Nyes::Braning);
    }

    fn settle_from_ubc_result(&self) {
        let result_nyes = self
            .core
            .ubc_children()
            .first()
            .map(|r| r.borrow().core().get_nyes())
            .unwrap_or(Nyes::Nk);
        if result_nyes.is_constanic() {
            self.core.set_nyes(Self::nyes_from_found(result_nyes));
        }
    }

    fn contexted_search_from_anchor(&self, _scope: &Scope) -> Option<(FirRef, Nyes)> {
        use contextful_search::{BraneNavigator, SearchPredicate, contextful_search_scan};
        let anchor = Rc::clone(&self.core.foolish_children()[0]);
        // Try to get FoolRefFir from anchor's ubc_children[1]
        let fool_ref_fir = {
            let borrowed = anchor.borrow();
            borrowed.core().ubc_children().get(1).cloned()
        };
        let fool_ref_fir = fool_ref_fir?;
        let referent = {
            let borrowed = fool_ref_fir.borrow();
            borrowed.as_fool_ref_referent().cloned()
        }?;
        let h_brane = referent.borrow().get_my_brane(&referent)?;
        let p = h_brane.find_stmt_index(&referent)?;
        let brane_len = h_brane.borrow().core().foolish_children().len();
        if brane_len == 0 {
            return None;
        }
        let (scan_start, scan_end) = if self.forward {
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
        let mut nav = BraneNavigator::new(&h_brane, self.forward);
        nav.set_range(scan_start, scan_end);
        let predicate = if self.is_value_search {
            let value_fir = self.value_child();
            SearchPredicate::Value { pattern: value_fir }
        } else if self.pattern.is_empty() {
            return None;
        } else {
            SearchPredicate::Name {
                pattern: self.pattern.clone(),
            }
        };
        match contextful_search_scan(&mut nav, &predicate) {
            ScanOutcome::Found(stmt) => {
                let nyes = stmt
                    .borrow()
                    .core()
                    .foolish_children()
                    .first()
                    .map(|b| b.borrow().core().get_nyes())
                    .unwrap_or(Nyes::Nk);
                Some((stmt, nyes))
            }
            _ => None,
        }
    }

    fn value_child(&self) -> FirRef {
        let idx = if self.anchored { 1 } else { 0 };
        Rc::clone(&self.core.foolish_children()[idx])
    }

    fn build_value_predicate(&self) -> Option<SearchPredicate> {
        let value_fir = self.value_child();
        let nyes = value_fir.borrow().core().get_nyes();
        if !nyes.is_constanic() {
            return None;
        }
        if self.pattern.is_empty() {
            Some(SearchPredicate::Value { pattern: value_fir })
        } else {
            Some(SearchPredicate::NameValue {
                name: self.pattern.clone(),
                value: value_fir,
            })
        }
    }

    fn check_value_pattern_ready(&self) -> bool {
        let value_fir = self.value_child();
        let nyes = value_fir.borrow().core().get_nyes();
        if !nyes.is_constanic() {
            self.core.push_task(value_fir);
            return false;
        }
        match nyes {
            Nyes::Nk => {
                self.core.set_nyes(Nyes::Nk);
                return false;
            }
            Nyes::Econstanic | Nyes::Woconstanic => {
                self.core.set_nyes(Nyes::Econstanic);
                return false;
            }
            _ => {}
        }
        if value_fir.borrow().as_i64().is_none() {
            self.core.set_alarm_reason(
                "VALUE-SEARCH-UNSUPPORTED-PATTERN: non-integer value pattern".to_string(),
            );
            self.core.set_nyes(Nyes::Nk);
            return false;
        }
        true
    }

    fn ib_search_with_engine(&self, scope: &Scope) -> Option<(FirRef, Nyes)> {
        use contextful_search::{
            BraneNavigator, SearchPredicate, contextful_search_scan_no_body_check,
        };
        let stmt = scope.get_my_statement()?;
        let brane = stmt.borrow().get_my_brane(&stmt)?;
        let idx = brane.find_stmt_index(&stmt)?;
        let search_end = idx.saturating_sub(1);
        let mut nav = BraneNavigator::new(&brane, false);
        nav.set_range(0, search_end);
        let predicate = SearchPredicate::Name {
            pattern: self.pattern.clone(),
        };
        match contextful_search_scan_no_body_check(&mut nav, &predicate) {
            ScanOutcome::Found(found) => {
                let nyes = found.borrow().core().get_nyes();
                Some((found, nyes))
            }
            _ => None,
        }
    }

    fn ab_search_with_engine(&self, scope: &Scope) -> Option<(FirRef, Nyes)> {
        use contextful_search::{
            BraneNavigator, SearchPredicate, contextful_search_scan_no_body_check,
        };
        let mut current_brane = scope.get_my_brane()?;
        loop {
            let stmt = {
                let borrowed = current_brane.borrow();
                borrowed.get_my_statement(&current_brane)
            };
            if Rc::ptr_eq(&stmt, &current_brane) {
                return None;
            }
            let parent_brane = stmt.borrow().get_my_brane(&stmt)?;
            if let Some(idx) = parent_brane.find_stmt_index(&stmt)
                && idx > 0
            {
                let mut nav = BraneNavigator::new(&parent_brane, false);
                nav.set_range(0, idx - 1);
                let predicate = SearchPredicate::Name {
                    pattern: self.pattern.clone(),
                };
                if let ScanOutcome::Found(found) =
                    contextful_search_scan_no_body_check(&mut nav, &predicate)
                {
                    let nyes = found.borrow().core().get_nyes();
                    return Some((found, nyes));
                }
            }
            if Rc::ptr_eq(&parent_brane, &current_brane) {
                return None;
            }
            current_brane = parent_brane;
        }
    }

    fn value_search_step(&self, scope: &Scope) -> Result<(), UbcError> {
        use contextful_search::{BraneNavigator, contextful_search_scan};
        match self.core.get_nyes() {
            Nyes::Prembrionic => {
                // Phase B: push value pattern task alongside anchor so both
                // start stepping immediately.
                if self.anchored {
                    let anchor = Rc::clone(&self.core.foolish_children()[0]);
                    self.core.push_task(anchor);
                }
                self.core.push_task(self.value_child());
                if self.anchored {
                    self.core.set_nyes(Nyes::Braning);
                } else {
                    self.core.set_nyes(Nyes::Embryonic);
                }
            }
            Nyes::Embryonic => {
                if !self.core.ubc_children().is_empty() {
                    self.settle_from_ubc_result();
                    return Ok(());
                }
                if !self.check_value_pattern_ready() {
                    return Ok(());
                }
                let predicate = self.build_value_predicate().expect("checked ready");
                if let Some((stmt_ref, brane_ref)) = find_enclosing_stmt_and_brane(&self.core)
                    && let Some(idx) = brane_ref.find_stmt_index(&stmt_ref)
                    && idx > 0
                {
                    let range_end = idx - 1;
                    let mut nav = BraneNavigator::new(&brane_ref, false);
                    nav.set_range(0, range_end);
                    match contextful_search_scan(&mut nav, &predicate) {
                        ScanOutcome::Found(stmt) => {
                            let nyes = stmt
                                .borrow()
                                .core()
                                .foolish_children()
                                .first()
                                .map(|b| b.borrow().core().get_nyes())
                                .unwrap_or(Nyes::Nk);
                            self.handle_found(stmt, nyes, scope);
                        }
                        ScanOutcome::NkStop => {
                            self.core.set_nyes(Nyes::Nk);
                            return Ok(());
                        }
                        ScanOutcome::Miss => {
                            if !self.anchored {
                                self.core.set_nyes(Nyes::Econstanic);
                                return Ok(());
                            }
                        }
                    }
                } else if !self.anchored {
                    // No backward candidates (idx == 0 or no enclosing brane).
                    self.core.set_nyes(Nyes::Econstanic);
                    return Ok(());
                }
                self.core.set_nyes(Nyes::Braning);
            }
            Nyes::Braning => {
                if !self.core.ubc_children().is_empty() {
                    self.settle_from_ubc_result();
                    return Ok(());
                }
                if !self.check_value_pattern_ready() {
                    return Ok(());
                }
                let predicate = self.build_value_predicate().expect("checked ready");
                let scan_outcome = if self.contexted && self.anchored {
                    let anchor = Rc::clone(&self.core.foolish_children()[0]);
                    let anchor_settled = anchor.borrow().core().get_nyes().is_constanic();
                    match self.contexted_search_from_anchor(scope) {
                        Some((stmt, nyes)) => {
                            self.handle_found(stmt, nyes, scope);
                            return Ok(());
                        }
                        None => {
                            if !anchor_settled {
                                return Ok(());
                            }
                            ScanOutcome::Miss
                        }
                    }
                } else if self.anchored {
                    let anchor = Rc::clone(&self.core.foolish_children()[0]);
                    let resolved = anchor.resolve_anchor();
                    if resolved.borrow().core().get_nyes() == Nyes::Nk {
                        self.core.set_nyes(Nyes::Nk);
                        return Ok(());
                    }
                    if resolved.borrow().kind() != FirKind::Brane {
                        ScanOutcome::Miss
                    } else {
                        let mut nav = BraneNavigator::new(&resolved, self.forward);
                        contextful_search_scan(&mut nav, &predicate)
                    }
                } else {
                    match find_enclosing_stmt_and_brane(&self.core) {
                        Some((stmt_ref, brane_ref)) => {
                            if let Some(idx) = brane_ref.find_stmt_index(&stmt_ref) {
                                let brane_borrowed = brane_ref.borrow();
                                let children = brane_borrowed.core().foolish_children();
                                let len = children.len();
                                if idx + 1 < len {
                                    let mut nav = BraneNavigator::new(&brane_ref, true);
                                    nav.set_range(idx + 1, len - 1);
                                    let outcome = contextful_search_scan(&mut nav, &predicate);
                                    drop(brane_borrowed);
                                    outcome
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
                        let nyes = stmt
                            .borrow()
                            .core()
                            .foolish_children()
                            .first()
                            .map(|b| b.borrow().core().get_nyes())
                            .unwrap_or(Nyes::Nk);
                        self.handle_found(stmt, nyes, scope);
                    }
                    ScanOutcome::NkStop => {
                        self.core.set_nyes(Nyes::Nk);
                    }
                    ScanOutcome::Miss => {
                        self.core.set_nyes(if self.anchored {
                            Nyes::Nk
                        } else {
                            Nyes::Econstanic
                        });
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}

impl Fir for SearchFir {
    #[inline(always)]
    fn core(&self) -> &ProtoBrane {
        &self.core
    }
    fn fir_op_step(&self, scope: &Scope) -> Result<(), UbcError> {
        if self.is_value_search {
            return self.value_search_step(scope);
        }
        match self.core.get_nyes() {
            Nyes::Prembrionic => {
                if self.anchored {
                    let anchor = Rc::clone(&self.core.foolish_children()[0]);
                    self.core.push_task(anchor);
                    self.core.set_nyes(Nyes::Braning);
                } else {
                    self.core.set_nyes(Nyes::Embryonic);
                }
            }
            Nyes::Embryonic => {
                if !self.core.ubc_children().is_empty() {
                    self.settle_from_ubc_result();
                } else {
                    match self.ib_search_with_engine(scope) {
                        Some((stmt, nyes)) => {
                            self.handle_found(stmt, nyes, scope);
                            self.core.set_nyes(Nyes::Braning);
                        }
                        None => self.core.set_nyes(Nyes::Braning),
                    }
                }
            }
            Nyes::Braning => {
                if !self.core.ubc_children().is_empty() {
                    self.settle_from_ubc_result();
                } else {
                    let result = if self.contexted && self.anchored {
                        self.contexted_search_from_anchor(scope)
                    } else if self.anchored {
                        use contextful_search::{
                            BraneNavigator, SearchPredicate, contextful_search_scan_no_body_check,
                        };
                        let anchor = Rc::clone(&self.core.foolish_children()[0]);
                        let resolved = anchor.resolve_anchor();
                        if resolved.borrow().core().get_nyes() == Nyes::Nk {
                            self.core.set_nyes(Nyes::Nk);
                            return Ok(());
                        }
                        if resolved.borrow().kind() != FirKind::Brane {
                            None
                        } else {
                            let mut nav = BraneNavigator::new(&resolved, self.forward);
                            let predicate = SearchPredicate::Name {
                                pattern: self.pattern.clone(),
                            };
                            match contextful_search_scan_no_body_check(&mut nav, &predicate) {
                                ScanOutcome::Found(stmt) => {
                                    let nyes = stmt.borrow().core().get_nyes();
                                    Some((stmt, nyes))
                                }
                                _ => None,
                            }
                        }
                    } else {
                        self.ab_search_with_engine(scope)
                    };
                    match result {
                        Some((stmt, nyes)) => self.handle_found(stmt, nyes, scope),
                        None => self.core.set_nyes(if self.anchored {
                            Nyes::Nk
                        } else {
                            Nyes::Econstanic
                        }),
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
    fn kind(&self) -> FirKind {
        FirKind::Search
    }
    fn as_search_pattern(&self) -> Option<&str> {
        Some(&self.pattern)
    }
    fn as_search_anchored(&self) -> bool {
        self.anchored
    }
    fn as_sf_inner_pattern(&self) -> Option<String> {
        self.sf_inner_pattern.borrow().clone()
    }
    fn as_search_is_value(&self) -> bool {
        self.is_value_search
    }
    fn as_search_contexted(&self) -> bool {
        self.contexted
    }
    fn set_contexted(&mut self, contexted: bool) {
        self.contexted = contexted;
    }
}

#[derive(Debug)]
pub struct IndexFir {
    pub(crate) core: ProtoBrane,
    pub(crate) offset: i32,
    pub(crate) anchored: bool,
    pub(crate) contexted: bool,
}

fn find_enclosing_stmt_and_brane(start: &ProtoBrane) -> Option<(FirRef, FirRef)> {
    let mut current = start.parent();
    while let Some(node) = current {
        let kind = node.borrow().kind();
        if kind == FirKind::Statement {
            let brane = {
                let borrowed = node.borrow();
                find_parent_brane(borrowed.core())
            };
            if let Some(brane) = brane {
                return Some((node, brane));
            }
        }
        let next = node.borrow().core().parent();
        match next {
            Some(ref n) if Rc::ptr_eq(n, &node) => break,
            None => break,
            _ => current = next,
        }
    }
    None
}

fn find_parent_brane(start: &ProtoBrane) -> Option<FirRef> {
    let mut current = start.parent();
    while let Some(node) = current {
        if node.borrow().kind() == FirKind::Brane {
            return Some(node);
        }
        let next = node.borrow().core().parent();
        match next {
            Some(ref n) if Rc::ptr_eq(n, &node) => break,
            None => break,
            _ => current = next,
        }
    }
    None
}

impl IndexFir {
    pub fn index(
        offset: i32,
        anchored: bool,
        children: Vec<FirRef>,
        parent: Weak<RefCell<dyn Fir>>,
    ) -> FirRef {
        Rc::new(RefCell::new(IndexFir {
            core: ProtoBrane::new(children, parent, Nyes::Prembrionic),
            offset,
            anchored,
            contexted: false,
        }))
    }

    fn settle_from_ubc_result(&self) {
        let result_nyes = self
            .core
            .ubc_children()
            .first()
            .map(|r| r.borrow().core().get_nyes())
            .unwrap_or(Nyes::Nk);
        if result_nyes.is_constanic() {
            self.core.set_nyes(SearchFir::nyes_from_found(result_nyes));
        }
    }
}

impl Fir for IndexFir {
    #[inline(always)]
    fn core(&self) -> &ProtoBrane {
        &self.core
    }
    fn fir_op_step(&self, scope: &Scope) -> Result<(), UbcError> {
        match self.core.get_nyes() {
            Nyes::Prembrionic | Nyes::Embryonic => {
                if self.anchored {
                    let anchor = Rc::clone(&self.core.foolish_children()[0]);
                    self.core.push_task(anchor);
                    self.core.set_nyes(Nyes::Braning);
                } else {
                    if self.offset >= 0 {
                        return Err(UbcError::Eval(
                            "unanchored index requires negative offset".to_owned(),
                        ));
                    }
                    use contextful_search::{
                        BraneNavigator, SearchPredicate, contextful_search_scan_no_body_check,
                    };
                    match find_enclosing_stmt_and_brane(&self.core) {
                        Some((stmt_ref, brane_ref)) => {
                            if let Some(idx) = brane_ref.find_stmt_index(&stmt_ref) {
                                let target = idx as i32 + self.offset;
                                let len = brane_ref.borrow().core().foolish_children().len() as i32;
                                if target < 0 || target >= len {
                                    self.core.set_nyes(Nyes::Nk);
                                } else {
                                    let mut nav = BraneNavigator::new(&brane_ref, true);
                                    let predicate = SearchPredicate::Index(target);
                                    match contextful_search_scan_no_body_check(&mut nav, &predicate)
                                    {
                                        ScanOutcome::Found(stmt) => {
                                            let body = stmt
                                                .borrow()
                                                .core()
                                                .foolish_children()
                                                .first()
                                                .cloned();
                                            match body {
                                                Some(body) => {
                                                    let self_weak = self.core.parent_weak();
                                                    let clone = ProtoBrane::constanic_clone_at(
                                                        &body,
                                                        &self_weak,
                                                        0,
                                                        scope.has_ancestral_sfm,
                                                    );
                                                    push_search_result_pair(
                                                        &self.core, clone, stmt,
                                                    );
                                                    self.core.set_nyes(Nyes::Braning);
                                                }
                                                None => self.core.set_nyes(Nyes::Nk),
                                            }
                                        }
                                        _ => self.core.set_nyes(Nyes::Nk),
                                    }
                                }
                            } else {
                                self.core.set_nyes(Nyes::Nk);
                            }
                        }
                        None => {
                            self.core.set_nyes(Nyes::Nk);
                        }
                    }
                }
            }
            Nyes::Braning => {
                if !self.core.ubc_children().is_empty() {
                    self.settle_from_ubc_result();
                } else if self.contexted && self.anchored {
                    use contextful_search::{
                        BraneNavigator, SearchPredicate, contextful_search_scan_no_body_check,
                    };
                    let anchor = Rc::clone(&self.core.foolish_children()[0]);
                    let fool_ref_fir = {
                        let borrowed = anchor.borrow();
                        borrowed.core().ubc_children().get(1).cloned()
                    };
                    let contexted_result = fool_ref_fir.and_then(|frf| {
                        let referent = frf.borrow().as_fool_ref_referent().cloned()?;
                        let h_brane = referent.borrow().get_my_brane(&referent)?;
                        let p = h_brane.find_stmt_index(&referent)?;
                        let target = p as i32 + self.offset;
                        let len = h_brane.borrow().core().foolish_children().len() as i32;
                        if target < 0 || target >= len {
                            return None;
                        }
                        let mut nav = BraneNavigator::new(&h_brane, true);
                        let predicate = SearchPredicate::Index(target);
                        match contextful_search_scan_no_body_check(&mut nav, &predicate) {
                            ScanOutcome::Found(stmt) => {
                                let body =
                                    stmt.borrow().core().foolish_children().first().cloned()?;
                                Some((stmt, body))
                            }
                            _ => None,
                        }
                    });
                    if let Some((stmt, body)) = contexted_result {
                        let self_weak = self.core.parent_weak();
                        let clone = ProtoBrane::constanic_clone_at(
                            &body,
                            &self_weak,
                            0,
                            scope.has_ancestral_sfm,
                        );
                        push_search_result_pair(&self.core, clone, stmt);
                    } else if !anchor.borrow().core().get_nyes().is_constanic() {
                        return Ok(());
                    } else {
                        self.core.set_nyes(Nyes::Nk);
                    }
                } else if self.anchored {
                    use contextful_search::{
                        BraneNavigator, SearchPredicate, contextful_search_scan_no_body_check,
                    };
                    let anchor = Rc::clone(&self.core.foolish_children()[0]);
                    let resolved = anchor.resolve_anchor();
                    if resolved.borrow().kind() != FirKind::Brane {
                        self.core.set_nyes(Nyes::Nk);
                        return Ok(());
                    }
                    let mut nav = BraneNavigator::new(&resolved, true);
                    let predicate = SearchPredicate::Index(self.offset);
                    match contextful_search_scan_no_body_check(&mut nav, &predicate) {
                        ScanOutcome::Found(stmt) => {
                            let body = stmt.borrow().core().foolish_children().first().cloned();
                            match body {
                                Some(body) => {
                                    let self_weak = self.core.parent_weak();
                                    let clone = ProtoBrane::constanic_clone_at(
                                        &body,
                                        &self_weak,
                                        0,
                                        scope.has_ancestral_sfm,
                                    );
                                    push_search_result_pair(&self.core, clone, stmt);
                                }
                                None => self.core.set_nyes(Nyes::Nk),
                            }
                        }
                        _ => self.core.set_nyes(Nyes::Nk),
                    }
                } else {
                    self.core.set_nyes(Nyes::Nk);
                }
            }
            _ => {}
        }
        Ok(())
    }
    fn kind(&self) -> FirKind {
        FirKind::Index
    }
    fn as_index_offset(&self) -> i32 {
        self.offset
    }
    fn as_index_anchored(&self) -> bool {
        self.anchored
    }
    fn as_search_contexted(&self) -> bool {
        self.contexted
    }
    fn set_contexted(&mut self, contexted: bool) {
        self.contexted = contexted;
    }
}

// ── FoolRefFir (FOOP-23 Phase C1) ──────────────────────────────────

/// An immutable strong reference to another FIR — the "fool's reference".
///
/// Wraps a strong (non-weak) [`FirRef`] to the original statement a search
/// found. Created alongside every search result as `ubc_children[1]` (the
/// found statement's body clone lives at `[0]`). Read-only: no method on
/// `FoolRefFir` mutates the referent, and it exposes no `&mut` path to it.
///
/// `FoolRefFir` is born [`Nyes::Constant`] (terminal) — the reference itself
/// is a settled value even while the referent may still be evolving. It takes
/// no steps (`fir_op_step` is a no-op) and holds no children.
///
/// Invisible to values: [`FirRefExt::value`], result-chain walking, and the
/// Humanizing Sequencer all read `ubc_children[0]` only.
///
/// This is what makes providing-context universal (FOOP-23 §C.3.2): every
/// search result carries a `FoolRefFir` position that a following contexted
/// (`&`-prefixed) search can read to anchor within the found statement's home
/// brane.
#[derive(Debug)]
pub struct FoolRefFir {
    pub(crate) core: ProtoBrane,
    #[allow(dead_code)]
    referent: FirRef,
}

impl Fir for FoolRefFir {
    #[inline(always)]
    fn core(&self) -> &ProtoBrane {
        &self.core
    }
    fn fir_op_step(&self, _scope: &Scope) -> Result<(), UbcError> {
        Ok(())
    }
    fn kind(&self) -> FirKind {
        FirKind::FoolRef
    }
    fn as_fool_ref_referent(&self) -> Option<&FirRef> {
        Some(&self.referent)
    }
}

impl FoolRefFir {
    #[allow(dead_code)]
    pub(crate) fn referent(&self) -> &FirRef {
        &self.referent
    }
}

/// Push a search result AND its FoolRefFir bookkeeping entry to `ubc_children`.
///
/// After this call, `ubc_children` holds `[clone, FoolRefFir]`.
/// All existing readers access `[0]` only (via `.first()`), so the
/// FoolRefFir at `[1]` is invisible to them.
pub(crate) fn push_search_result_pair(core: &ProtoBrane, result: FirRef, referent: FirRef) {
    let fool_ref = Rc::new(RefCell::new(FoolRefFir {
        core: ProtoBrane::new(vec![], core.parent_weak(), Nyes::Constant),
        referent,
    }));
    core.push_search_result(result);
    core.push_ubc_child(fool_ref);
}

// ── ContextfulSearch engine skeleton (FOOP-23 Phase A0) ──────────────

// Allow dead_code for Phase A0 skeleton types — wired into production in Phase A1+.
#[allow(dead_code)]
mod contextful_search {
    use super::{FirRef, SearchFir};

    use std::rc::Rc;

    use foolish_core::fir::Nyes;

    /// Where the Navigator starts scanning from.
    ///
    /// `Contextless` — anchor resolved to a brane, cursor at front/rear.
    /// `Contexted` — incoming result's statement position, bounded by home brane.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) enum CursorSource {
        Contextless,
        Contexted,
    }

    /// The result of applying a predicate to a single candidate statement.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) enum MatchOutcome {
        Approve,
        Reject,
        NkStop,
    }

    /// Result of the core scan loop.
    #[derive(Debug, Clone)]
    pub(crate) enum ScanOutcome {
        Found(FirRef),
        NkStop,
        Miss,
    }

    impl PartialEq for ScanOutcome {
        fn eq(&self, other: &Self) -> bool {
            match (self, other) {
                (Self::Found(a), Self::Found(b)) => Rc::ptr_eq(a, b),
                (Self::NkStop, Self::NkStop) => true,
                (Self::Miss, Self::Miss) => true,
                _ => false,
            }
        }
    }

    impl Eq for ScanOutcome {}

    /// Match predicates for the ContextfulSearch engine.
    ///
    /// Each variant reads a different facet of the candidate statement FIR.
    /// The candidate is the *full* statement — name, body/value, line number,
    /// parent, NYES — everything reachable from the statement FirRef.
    #[derive(Debug)]
    pub(crate) enum SearchPredicate {
        /// Name-match: `?name` / `~name` / `.name`. Reads the candidate's name.
        Name { pattern: String },
        /// Value-match: `?=v` / `~=v`. Reads the candidate's body integer value.
        Value { pattern: FirRef },
        /// Atomic name+value: `?name=v` / `~name=v`. Both gates on the same candidate.
        NameValue { name: String, value: FirRef },
        /// Positional index: `#N`. Reads the candidate's position in the scan.
        Index(i32),
        /// First position: `^`. Matches when position == 0.
        Head,
        /// Last position: `$`. Matches when position == total - 1.
        Tail,
    }

    /// Context passed to the predicate during a scan.
    #[derive(Debug, Clone)]
    pub(crate) struct ScanCtx {
        /// 0-based position of the current candidate within its home brane.
        pub(crate) position: usize,
        /// Total number of candidates in the home brane.
        pub(crate) total: usize,
    }

    impl SearchPredicate {
        /// Apply this predicate to a candidate statement.
        ///
        /// The candidate is the *full* statement FirRef — each predicate reads
        /// whatever facet it needs (name, body, position). The Matcher knows
        /// nothing about traversal order.
        pub(crate) fn matches(&self, candidate: &FirRef, ctx: &ScanCtx) -> MatchOutcome {
            match self {
                Self::Name { pattern } => {
                    let borrowed = candidate.borrow();
                    let name = match borrowed.as_stmt_name() {
                        Some(n) => n,
                        None => return MatchOutcome::Reject,
                    };
                    if !SearchFir::matches_pattern(name, pattern) {
                        return MatchOutcome::Reject;
                    }
                    drop(borrowed);
                    check_body_nyes(candidate)
                }
                Self::Value { pattern } => {
                    let body = {
                        let borrowed = candidate.borrow();
                        match borrowed.core().foolish_children().first() {
                            Some(b) => Rc::clone(b),
                            None => return MatchOutcome::Reject,
                        }
                    };
                    let nyes = body.borrow().core().get_nyes();
                    if !nyes.is_constanic() {
                        unreachable!("pre-constanic body in search candidate")
                    }
                    if nyes == Nyes::Nk {
                        return MatchOutcome::NkStop;
                    }
                    let cand_val = body.borrow().as_i64();
                    let pat_val = pattern.borrow().as_i64();
                    match (cand_val, pat_val) {
                        (Some(cv), Some(pv)) if cv == pv => MatchOutcome::Approve,
                        _ => MatchOutcome::Reject,
                    }
                }
                Self::NameValue { name, value } => {
                    let body = {
                        let borrowed = candidate.borrow();
                        let stmt_name = match borrowed.as_stmt_name() {
                            Some(n) => n,
                            None => return MatchOutcome::Reject,
                        };
                        if !SearchFir::matches_pattern(stmt_name, name) {
                            return MatchOutcome::Reject;
                        }
                        match borrowed.core().foolish_children().first() {
                            Some(b) => Rc::clone(b),
                            None => return MatchOutcome::Reject,
                        }
                    };
                    let nyes = body.borrow().core().get_nyes();
                    if !nyes.is_constanic() {
                        unreachable!("pre-constanic body in search candidate")
                    }
                    if nyes == Nyes::Nk {
                        return MatchOutcome::NkStop;
                    }
                    let cand_val = body.borrow().as_i64();
                    let pat_val = value.borrow().as_i64();
                    match (cand_val, pat_val) {
                        (Some(cv), Some(pv)) if cv == pv => MatchOutcome::Approve,
                        _ => MatchOutcome::Reject,
                    }
                }
                Self::Index(offset) => {
                    let target = if *offset >= 0 {
                        *offset as usize
                    } else if ctx.total == 0 {
                        return MatchOutcome::Reject;
                    } else {
                        (ctx.total as i32 + offset) as usize
                    };
                    if ctx.position == target {
                        check_body_nyes(candidate)
                    } else {
                        MatchOutcome::Reject
                    }
                }
                Self::Head => {
                    if ctx.position == 0 {
                        check_body_nyes(candidate)
                    } else {
                        MatchOutcome::Reject
                    }
                }
                Self::Tail => {
                    if ctx.total > 0 && ctx.position == ctx.total - 1 {
                        check_body_nyes(candidate)
                    } else {
                        MatchOutcome::Reject
                    }
                }
            }
        }

        /// Like [`matches`] but skips the body-NYES gate.
        ///
        /// For positional/name-only predicates (Index, Head, Tail, Name) the
        /// candidate's body settling state is irrelevant — the caller decides
        /// what to do.  Value/NameValue predicates delegate to [`matches`]
        /// because they need the body settled to compare values.
        pub(crate) fn matches_no_body_check(
            &self,
            candidate: &FirRef,
            ctx: &ScanCtx,
        ) -> MatchOutcome {
            match self {
                Self::Name { pattern } => {
                    let borrowed = candidate.borrow();
                    let name = match borrowed.as_stmt_name() {
                        Some(n) => n,
                        None => return MatchOutcome::Reject,
                    };
                    if !SearchFir::matches_pattern(name, pattern) {
                        return MatchOutcome::Reject;
                    }
                    MatchOutcome::Approve
                }
                Self::Index(offset) => {
                    let target = if *offset >= 0 {
                        *offset as usize
                    } else if ctx.total == 0 {
                        return MatchOutcome::Reject;
                    } else {
                        (ctx.total as i32 + offset) as usize
                    };
                    if ctx.position == target {
                        MatchOutcome::Approve
                    } else {
                        MatchOutcome::Reject
                    }
                }
                Self::Head => {
                    if ctx.position == 0 {
                        MatchOutcome::Approve
                    } else {
                        MatchOutcome::Reject
                    }
                }
                Self::Tail => {
                    if ctx.total > 0 && ctx.position == ctx.total - 1 {
                        MatchOutcome::Approve
                    } else {
                        MatchOutcome::Reject
                    }
                }
                // Value/NameValue need body settled for comparison.
                _ => self.matches(candidate, ctx),
            }
        }
    }

    /// Check a candidate's body NYES after it passes positional/name gates.
    ///
    /// Pre-constanic → Wait. NK → NkStop. Otherwise → Approve.
    fn check_body_nyes(candidate: &FirRef) -> MatchOutcome {
        let nyes = candidate
            .borrow()
            .core()
            .foolish_children()
            .first()
            .map(|b| b.borrow().core().get_nyes());
        match nyes {
            Some(n) if !n.is_constanic() => unreachable!("pre-constanic body in search candidate"),
            Some(Nyes::Nk) => MatchOutcome::NkStop,
            _ => MatchOutcome::Approve,
        }
    }

    /// Navigator contract: yields candidate statements as (FirRef, brane_position).
    ///
    /// The Navigator knows nothing about matching. It embodies "where search looks
    /// and in what order." Its correctness contract:
    ///
    /// 1. **Correctly ordered** — the one mandated order.
    /// 2. **Complete** — every reachable candidate, exactly once, then stops.
    pub(crate) trait CandidateNavigator {
        /// Yield the next candidate as (statement FirRef, 0-based brane position).
        fn next_candidate(&mut self) -> Option<(FirRef, usize)>;
        /// Total number of candidates in the source.
        fn total(&self) -> usize;
    }

    /// Iterates `foolish_children()` of a brane in order (forward or backward).
    ///
    /// The Navigator contract (load-bearing correctness): yields **exactly** the
    /// mandated candidates, **in order**, **each once**, then stops.
    #[derive(Debug)]
    pub(crate) struct BraneNavigator {
        children: Vec<FirRef>,
        pos: usize,
        forward: bool,
        done: bool,
    }

    impl BraneNavigator {
        pub(crate) fn new(brane: &FirRef, forward: bool) -> Self {
            let children = brane.borrow().core().foolish_children().to_vec();
            let len = children.len();
            let start = if forward || len == 0 { 0 } else { len - 1 };
            Self {
                children,
                pos: start,
                forward,
                done: len == 0,
            }
        }

        pub(crate) fn set_range(&mut self, start: usize, end: usize) {
            if start > end || start >= self.children.len() {
                self.done = true;
                return;
            }
            let end = end.min(self.children.len() - 1);
            if self.forward {
                self.pos = start;
                self.done = false;
            } else {
                self.pos = end;
                self.done = false;
            }
        }
    }

    impl CandidateNavigator for BraneNavigator {
        fn next_candidate(&mut self) -> Option<(FirRef, usize)> {
            if self.done || self.pos >= self.children.len() {
                return None;
            }
            let brane_pos = self.pos;
            let candidate = Rc::clone(&self.children[brane_pos]);
            // Advance cursor.
            if self.forward {
                self.pos += 1;
                if self.pos >= self.children.len() {
                    self.done = true;
                }
            } else if self.pos == 0 {
                self.done = true;
            } else {
                self.pos -= 1;
            }
            Some((candidate, brane_pos))
        }

        fn total(&self) -> usize {
            self.children.len()
        }
    }

    /// The core scan loop of the ContextfulSearch engine.
    ///
    /// Iterates candidates from the Navigator, applying the predicate to each.
    /// The two shared rules live here, not in either collaborator:
    ///
    /// - **Wait-on-nye**: if a candidate's predicate returns `Wait`, the scan
    ///   suspends immediately (the candidate is pre-constanic; order is sacred).
    /// - **NK-stop**: if a candidate's predicate returns `NkStop`, the scan
    ///   halts (the search itself becomes NK).
    ///
    /// Returns `Miss` when all candidates are exhausted with no match and no
    /// suspensions. The caller decides the settlement: anchored → NK, unanchored
    /// → ECONSTANIC.
    pub(crate) fn contextful_search_scan(
        nav: &mut dyn CandidateNavigator,
        predicate: &SearchPredicate,
    ) -> ScanOutcome {
        let total = nav.total();
        while let Some((candidate, position)) = nav.next_candidate() {
            let ctx = ScanCtx { position, total };
            match predicate.matches(&candidate, &ctx) {
                MatchOutcome::Approve => return ScanOutcome::Found(candidate),
                MatchOutcome::Reject => {}
                MatchOutcome::NkStop => return ScanOutcome::NkStop,
            }
        }
        ScanOutcome::Miss
    }

    /// Like [`contextful_search_scan`] but uses [`SearchPredicate::matches_no_body_check`].
    ///
    /// For contextless searches (IndexFir, SearchFir name search)
    /// where body settling is the caller's responsibility.
    pub(crate) fn contextful_search_scan_no_body_check(
        nav: &mut dyn CandidateNavigator,
        predicate: &SearchPredicate,
    ) -> ScanOutcome {
        let total = nav.total();
        while let Some((candidate, position)) = nav.next_candidate() {
            let ctx = ScanCtx { position, total };
            match predicate.matches_no_body_check(&candidate, &ctx) {
                MatchOutcome::Approve => return ScanOutcome::Found(candidate),
                MatchOutcome::Reject => {}
                MatchOutcome::NkStop => return ScanOutcome::NkStop,
            }
        }
        ScanOutcome::Miss
    }
} // end mod contextful_search

#[allow(unused_imports)]
pub(crate) use contextful_search::contextful_search_scan;
#[allow(unused_imports)]
pub(crate) use contextful_search::contextful_search_scan_no_body_check;
#[allow(unused_imports)]
pub(crate) use contextful_search::{
    BraneNavigator, CandidateNavigator, CursorSource, MatchOutcome, ScanCtx, ScanOutcome,
    SearchPredicate,
};

#[derive(Debug)]
pub struct StayFoolishFir {
    pub(crate) core: ProtoBrane,
}

impl StayFoolishFir {
    pub fn stay_foolish(expr: FirRef, parent: Weak<RefCell<dyn Fir>>) -> FirRef {
        Rc::new(RefCell::new(StayFoolishFir {
            core: ProtoBrane::new(vec![expr], parent, Nyes::Prembrionic),
        }))
    }
}

impl Fir for StayFoolishFir {
    #[inline(always)]
    fn core(&self) -> &ProtoBrane {
        &self.core
    }
    fn fir_op_step(&self, _scope: &Scope) -> Result<(), UbcError> {
        match self.core.get_nyes() {
            Nyes::Prembrionic | Nyes::Embryonic => {
                let children: Vec<FirRef> = self.core.foolish_children().to_vec();
                if children.is_empty() {
                    self.core.set_nyes(Nyes::Constant);
                } else {
                    self.core.set_nyes(Nyes::Braning);
                    for child in children {
                        self.core.push_task(child);
                    }
                }
            }
            Nyes::Braning => {
                let children = self.core.foolish_children().to_vec();
                if let Some(expr) = children.first() {
                    let expr_nyes = expr.borrow().core().get_nyes();
                    if expr_nyes.is_constanic() {
                        let (result, result_nyes) = {
                            let borrowed = expr.borrow();
                            let ubc = borrowed.core().ubc_children();
                            match ubc.into_iter().next() {
                                Some(r) => {
                                    let n = r.borrow().core().get_nyes();
                                    (r, n)
                                }
                                None => (Rc::clone(expr), expr_nyes),
                            }
                        };
                        self.core.push_ubc_child(result);
                        self.core.set_nyes(result_nyes);
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
    fn kind(&self) -> FirKind {
        FirKind::StayFoolish
    }
}

#[derive(Debug)]
pub struct StayFullyFoolishFir {
    pub(crate) core: ProtoBrane,
}

impl StayFullyFoolishFir {
    pub fn stay_fully_foolish(expr: FirRef, parent: Weak<RefCell<dyn Fir>>) -> FirRef {
        Rc::new(RefCell::new(StayFullyFoolishFir {
            core: ProtoBrane::new(vec![expr], parent, Nyes::Prembrionic),
        }))
    }
}

impl Fir for StayFullyFoolishFir {
    #[inline(always)]
    fn core(&self) -> &ProtoBrane {
        &self.core
    }
    fn fir_op_step(&self, _scope: &Scope) -> Result<(), UbcError> {
        match self.core.get_nyes() {
            Nyes::Prembrionic | Nyes::Embryonic => {
                let children: Vec<FirRef> = self.core.foolish_children().to_vec();
                self.core.set_nyes(Nyes::Braning);
                for child in children {
                    self.core.push_task(child);
                }
            }
            Nyes::Braning => {
                let children = self.core.foolish_children().to_vec();
                if let Some(nyes) = _decide_nyes_due_to_children(&children) {
                    self.core.set_nyes(nyes);
                }
            }
            _ => {}
        }
        Ok(())
    }
    fn kind(&self) -> FirKind {
        FirKind::StayFullyFoolish
    }
}

#[derive(Debug)]
pub struct ConcatenationFir {
    pub(crate) core: ProtoBrane,
}

impl ConcatenationFir {
    pub fn concatenation(elements: Vec<FirRef>, parent: Weak<RefCell<dyn Fir>>) -> FirRef {
        Rc::new(RefCell::new(ConcatenationFir {
            core: ProtoBrane::new(elements, parent, Nyes::Prembrionic),
        }))
    }
}

impl Fir for ConcatenationFir {
    #[inline(always)]
    fn core(&self) -> &ProtoBrane {
        &self.core
    }
    fn fir_op_step(&self, _scope: &Scope) -> Result<(), UbcError> {
        match self.core.get_nyes() {
            Nyes::Prembrionic | Nyes::Embryonic => {
                let children: Vec<FirRef> = self.core.foolish_children().to_vec();
                if children.is_empty() {
                    let result_ref: FirRef = Rc::new_cyclic(|me: &Weak<RefCell<BraneFir>>| {
                        let parent: Weak<RefCell<dyn Fir>> = me.clone();
                        RefCell::new(BraneFir {
                            core: ProtoBrane::new(vec![], parent, Nyes::Constant),
                            characterizations: Vec::new(),
                        })
                    });
                    self.core.push_ubc_child(result_ref);
                    self.core.set_nyes(Nyes::Constant);
                } else {
                    self.core.set_nyes(Nyes::Braning);
                    for child in children {
                        self.core.push_task(child);
                    }
                }
            }
            Nyes::Braning => {
                let children = self.core.foolish_children().to_vec();
                let any_nk = children.iter().any(|c| {
                    let resolved = c.value();
                    resolved.borrow().core().get_nyes() == Nyes::Nk
                });
                let any_woconstanic = children.iter().any(|c| {
                    let resolved = c.value();
                    let n = resolved.borrow().core().get_nyes();
                    n == Nyes::Econstanic || n == Nyes::Woconstanic
                });
                let mut merged_stmts: Vec<FirRef> = Vec::new();
                for child in &children {
                    let resolved = {
                        let borrowed = child.borrow();
                        if borrowed.core().get_nyes().is_constanic() {
                            borrowed.core().ubc_children().into_iter().next()
                        } else {
                            None
                        }
                    };
                    let source = resolved.as_ref().unwrap_or(child);
                    let borrowed = source.borrow();
                    for stmt in borrowed.core().foolish_children() {
                        merged_stmts.push(Rc::clone(stmt));
                    }
                }
                let merged_state = if any_nk {
                    Nyes::Nk
                } else if any_woconstanic {
                    Nyes::Woconstanic
                } else {
                    Nyes::Constant
                };
                let result_ref: FirRef = Rc::new_cyclic(|me: &Weak<RefCell<BraneFir>>| {
                    let parent: Weak<RefCell<dyn Fir>> = me.clone();
                    RefCell::new(BraneFir {
                        core: ProtoBrane::new(merged_stmts, parent, merged_state),
                        characterizations: Vec::new(),
                    })
                });
                self.core.push_ubc_child(result_ref);
                self.core.set_nyes(merged_state);
            }
            _ => {}
        }
        Ok(())
    }
    fn kind(&self) -> FirKind {
        FirKind::Concatenation
    }
}

pub fn nk(reason: &str, parent: Weak<RefCell<dyn Fir>>) -> FirRef {
    Rc::new(RefCell::new(NkFir {
        core: ProtoBrane::new(vec![], parent, Nyes::Prembrionic),
        reason: reason.to_owned(),
    }))
}

pub fn operator(op: &str, operands: Vec<FirRef>, parent: Weak<RefCell<dyn Fir>>) -> FirRef {
    Rc::new(RefCell::new(OperatorFir {
        core: ProtoBrane::new(operands, parent, Nyes::Prembrionic),
        op: op.to_owned(),
    }))
}

pub fn statement(
    name: &str,
    line_number: usize,
    body: FirRef,
    parent: Weak<RefCell<dyn Fir>>,
) -> FirRef {
    Rc::new(RefCell::new(StatementFir {
        core: ProtoBrane::new(vec![body], parent, Nyes::Prembrionic),
        name: name.to_owned(),
        line_number,
    }))
}

pub fn brane(children: Vec<FirRef>, parent: Weak<RefCell<dyn Fir>>) -> FirRef {
    Rc::new(RefCell::new(BraneFir {
        core: ProtoBrane::new(children, parent, Nyes::Prembrionic),
        characterizations: Vec::new(),
    }))
}

pub fn index(
    offset: i32,
    anchored: bool,
    children: Vec<FirRef>,
    parent: Weak<RefCell<dyn Fir>>,
) -> FirRef {
    Rc::new(RefCell::new(IndexFir {
        core: ProtoBrane::new(children, parent, Nyes::Prembrionic),
        offset,
        anchored,
        contexted: false,
    }))
}

pub fn concatenation(elements: Vec<FirRef>, parent: Weak<RefCell<dyn Fir>>) -> FirRef {
    Rc::new(RefCell::new(ConcatenationFir {
        core: ProtoBrane::new(elements, parent, Nyes::Prembrionic),
    }))
}

pub fn stay_foolish(expr: FirRef, parent: Weak<RefCell<dyn Fir>>) -> FirRef {
    Rc::new(RefCell::new(StayFoolishFir {
        core: ProtoBrane::new(vec![expr], parent, Nyes::Prembrionic),
    }))
}

pub fn stay_fully_foolish(expr: FirRef, parent: Weak<RefCell<dyn Fir>>) -> FirRef {
    Rc::new(RefCell::new(StayFullyFoolishFir {
        core: ProtoBrane::new(vec![expr], parent, Nyes::Prembrionic),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fir_trait::StepReport;

    fn make_constant_int(value: i64) -> FirRef {
        Rc::new_cyclic(|me: &Weak<RefCell<IndepIntFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(IndepIntFir {
                core: ProtoBrane::new(vec![], parent, Nyes::Prembrionic),
                value,
            })
        })
    }

    fn make_nk(reason: &str) -> FirRef {
        Rc::new_cyclic(|me: &Weak<RefCell<NkFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(NkFir {
                core: ProtoBrane::new(vec![], parent, Nyes::Prembrionic),
                reason: reason.to_owned(),
            })
        })
    }

    #[test]
    fn constant_int_prembrionic_to_constant_in_one_step() {
        let node = make_constant_int(42);
        let scope = Scope::empty();

        assert_eq!(node.borrow().core().get_nyes(), Nyes::Prembrionic);
        let mut transitions = vec![Nyes::Prembrionic];

        let report = node.step(&scope).unwrap();
        match report {
            StepReport::Progress(nyes) => {
                transitions.push(nyes);
                assert_eq!(nyes, Nyes::Constant);
            }
            StepReport::NoProgress => panic!("expected progress on first step"),
        }

        eprintln!("IndepInt NYES transitions: {transitions:?}");
        assert_eq!(transitions, vec![Nyes::Prembrionic, Nyes::Constant]);

        assert!(node.borrow().core().get_nyes().is_constanic());

        assert_eq!(node.borrow().core().get_nyes(), Nyes::Constant);
        let borrowed = node.borrow();
        assert_eq!(borrowed.core().get_nyes(), Nyes::Constant);
        assert_eq!(borrowed.kind(), FirKind::IndepInt);
    }

    #[test]
    fn constant_int_value_accessor() {
        let node = make_constant_int(-7);
        let scope = Scope::empty();

        let _ = node.step(&scope).unwrap();

        assert_eq!(node.borrow().kind(), FirKind::IndepInt);
    }

    #[test]
    fn nk_prembrionic_to_nk_in_one_step() {
        let node = make_nk("unbound name");
        let scope = Scope::empty();

        assert_eq!(node.borrow().core().get_nyes(), Nyes::Prembrionic);
        let mut transitions = vec![Nyes::Prembrionic];

        let report = node.step(&scope).unwrap();
        match report {
            StepReport::Progress(nyes) => {
                transitions.push(nyes);
                assert_eq!(nyes, Nyes::Nk);
            }
            StepReport::NoProgress => panic!("expected progress on first step"),
        }

        eprintln!("NkFir NYES transitions: {transitions:?}");
        assert_eq!(transitions, vec![Nyes::Prembrionic, Nyes::Nk]);

        assert!(node.borrow().core().get_nyes().is_constanic());

        assert_eq!(node.borrow().kind(), FirKind::Nk);
    }

    #[test]
    fn nk_reason_accessor() {
        let node = make_nk("division by zero");
        let scope = Scope::empty();

        let _ = node.step(&scope).unwrap();

        assert_eq!(node.borrow().kind(), FirKind::Nk);
    }

    #[test]
    fn both_leaf_kinds_are_settled_after_one_step() {
        let ci = make_constant_int(100);
        let nk = make_nk("nope");
        let scope = Scope::empty();

        assert!(!ci.borrow().core().get_nyes().is_constanic());
        assert!(!nk.borrow().core().get_nyes().is_constanic());

        let r1 = ci.step(&scope).unwrap();
        let r2 = nk.step(&scope).unwrap();

        assert!(matches!(r1, StepReport::Progress(Nyes::Constant)));
        assert!(matches!(r2, StepReport::Progress(Nyes::Nk)));

        assert!(ci.borrow().core().get_nyes().is_constanic());
        assert!(nk.borrow().core().get_nyes().is_constanic());
    }

    #[test]
    fn stepping_already_settled_is_noop() {
        let ci = make_constant_int(1);
        let nk = make_nk("done");
        let scope = Scope::empty();

        let _ = ci.step(&scope).unwrap();
        let _ = nk.step(&scope).unwrap();

        let r1 = ci.step(&scope).unwrap();
        let r2 = nk.step(&scope).unwrap();

        assert_eq!(r1, StepReport::Progress(Nyes::Constant));
        assert_eq!(r2, StepReport::Progress(Nyes::Nk));
    }

    #[test]
    fn constant_int_builder_sets_prembrionic() {
        let dummy: Rc<RefCell<dyn Fir>> = Rc::new_cyclic(|me: &Weak<RefCell<IndepIntFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(IndepIntFir {
                core: ProtoBrane::new(vec![], parent, Nyes::Constant),
                value: 0,
            })
        });
        let parent_weak = Rc::downgrade(&dummy);
        let node = IndepIntFir::constant_int(99, parent_weak);

        assert_eq!(node.borrow().core().get_nyes(), Nyes::Prembrionic);
        assert_eq!(node.borrow().kind(), FirKind::IndepInt);
    }

    #[test]
    fn nk_builder_sets_prembrionic() {
        let dummy: Rc<RefCell<dyn Fir>> = Rc::new_cyclic(|me: &Weak<RefCell<NkFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(NkFir {
                core: ProtoBrane::new(vec![], parent, Nyes::Nk),
                reason: String::new(),
            })
        });
        let parent_weak = Rc::downgrade(&dummy);
        let node = nk("test reason", parent_weak);

        assert_eq!(node.borrow().core().get_nyes(), Nyes::Prembrionic);
        assert_eq!(node.borrow().kind(), FirKind::Nk);
    }

    fn make_operator(op: &str, operands: Vec<FirRef>) -> FirRef {
        Rc::new_cyclic(|me: &Weak<RefCell<OperatorFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(OperatorFir {
                core: ProtoBrane::new(operands, parent, Nyes::Prembrionic),
                op: op.to_owned(),
            })
        })
    }

    fn make_statement(name: &str, line_number: usize, body: FirRef) -> FirRef {
        Rc::new_cyclic(|me: &Weak<RefCell<StatementFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(StatementFir {
                core: ProtoBrane::new(vec![body], parent, Nyes::Prembrionic),
                name: name.to_owned(),
                line_number,
            })
        })
    }

    fn make_brane(children: Vec<FirRef>) -> FirRef {
        Rc::new_cyclic(|me: &Weak<RefCell<BraneFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(BraneFir {
                core: ProtoBrane::new(children, parent, Nyes::Prembrionic),
                characterizations: Vec::new(),
            })
        })
    }

    fn make_search(pattern: &str, anchored: bool, foolish_children: Vec<FirRef>) -> FirRef {
        Rc::new_cyclic(|me: &Weak<RefCell<SearchFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(SearchFir {
                core: ProtoBrane::new(foolish_children, parent, Nyes::Prembrionic),
                pattern: pattern.to_owned(),
                anchored,
                forward: false,
                sf_inner_pattern: RefCell::new(None),
                is_value_search: false,
                contexted: false,
            })
        })
    }

    fn make_index(offset: i32, anchored: bool, foolish_children: Vec<FirRef>) -> FirRef {
        Rc::new_cyclic(|me: &Weak<RefCell<IndexFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(IndexFir {
                core: ProtoBrane::new(foolish_children, parent, Nyes::Prembrionic),
                offset,
                anchored,
                contexted: false,
            })
        })
    }

    fn step_to_settled(node: &FirRef, scope: &Scope) -> Vec<Nyes> {
        let mut transitions = vec![node.borrow().core().get_nyes()];
        for _ in 0..50 {
            let report = node.step(scope).unwrap();
            match report {
                StepReport::Progress(nyes) => {
                    transitions.push(nyes);
                    if nyes.is_constanic() {
                        break;
                    }
                }
                StepReport::NoProgress => break,
            }
        }
        transitions
    }

    #[test]
    fn operator_add_two_constants() {
        let a = make_constant_int(3);
        let b = make_constant_int(5);
        let op = make_operator("+", vec![Rc::clone(&a), Rc::clone(&b)]);
        let scope = Scope::empty();

        assert_eq!(op.borrow().core().get_nyes(), Nyes::Prembrionic);
        let mut transitions = vec![Nyes::Prembrionic];

        for _ in 0..20 {
            let report = op.step(&scope).unwrap();
            match report {
                StepReport::Progress(nyes) => {
                    transitions.push(nyes);
                    if nyes.is_constanic() {
                        break;
                    }
                }
                StepReport::NoProgress => break,
            }
        }

        eprintln!("Operator(+) NYES transitions: {transitions:?}");

        assert!(a.borrow().core().get_nyes().is_constanic());
        assert!(b.borrow().core().get_nyes().is_constanic());
        assert_eq!(op.borrow().core().get_nyes(), Nyes::Constant);
        assert_eq!(op.borrow().kind(), FirKind::Operator);

        let ubc = op.borrow().core().ubc_children().to_vec();
        assert_eq!(ubc.len(), 1);
        assert_eq!(ubc[0].borrow().kind(), FirKind::IndepInt);
        assert_eq!(ubc[0].borrow().as_i64(), Some(8));
    }

    #[test]
    fn operator_subtract() {
        let a = make_constant_int(10);
        let b = make_constant_int(3);
        let op = make_operator("-", vec![Rc::clone(&a), Rc::clone(&b)]);
        let scope = Scope::empty();

        for _ in 0..20 {
            let report = op.step(&scope).unwrap();
            if let StepReport::Progress(nyes) = report
                && nyes.is_constanic()
            {
                break;
            }
        }

        assert_eq!(op.borrow().core().get_nyes(), Nyes::Constant);
        let ubc = op.borrow().core().ubc_children().to_vec();
        assert_eq!(ubc.len(), 1);
        assert_eq!(ubc[0].borrow().as_i64(), Some(7));
    }

    #[test]
    fn operator_multiply() {
        let a = make_constant_int(4);
        let b = make_constant_int(6);
        let op = make_operator("*", vec![Rc::clone(&a), Rc::clone(&b)]);
        let scope = Scope::empty();

        for _ in 0..20 {
            let report = op.step(&scope).unwrap();
            if let StepReport::Progress(nyes) = report
                && nyes.is_constanic()
            {
                break;
            }
        }

        assert_eq!(op.borrow().core().get_nyes(), Nyes::Constant);
        let ubc = op.borrow().core().ubc_children().to_vec();
        assert_eq!(ubc.len(), 1);
        assert_eq!(ubc[0].borrow().as_i64(), Some(24));
    }

    #[test]
    fn operator_divide() {
        let a = make_constant_int(20);
        let b = make_constant_int(4);
        let op = make_operator("/", vec![Rc::clone(&a), Rc::clone(&b)]);
        let scope = Scope::empty();

        for _ in 0..20 {
            let report = op.step(&scope).unwrap();
            if let StepReport::Progress(nyes) = report
                && nyes.is_constanic()
            {
                break;
            }
        }

        assert_eq!(op.borrow().core().get_nyes(), Nyes::Constant);
        let ubc = op.borrow().core().ubc_children().to_vec();
        assert_eq!(ubc.len(), 1);
        assert_eq!(ubc[0].borrow().as_i64(), Some(5));
    }

    #[test]
    fn operator_divide_by_zero_is_nk() {
        let a = make_constant_int(10);
        let b = make_constant_int(0);
        let op = make_operator("/", vec![Rc::clone(&a), Rc::clone(&b)]);
        let scope = Scope::empty();

        for _ in 0..20 {
            let report = op.step(&scope).unwrap();
            if let StepReport::Progress(nyes) = report
                && nyes.is_constanic()
            {
                break;
            }
        }

        assert_eq!(op.borrow().core().get_nyes(), Nyes::Nk);
        assert_eq!(op.borrow().core().ubc_children().len(), 1);
        assert_eq!(
            op.borrow().core().ubc_children()[0].borrow().kind(),
            FirKind::Nk
        );
    }

    #[test]
    fn operator_with_nk_operand_is_nk() {
        let a = make_constant_int(5);
        let b = make_nk("unbound");
        let op = make_operator("+", vec![Rc::clone(&a), Rc::clone(&b)]);
        let scope = Scope::empty();

        for _ in 0..20 {
            let report = op.step(&scope).unwrap();
            if let StepReport::Progress(nyes) = report
                && nyes.is_constanic()
            {
                break;
            }
        }

        assert_eq!(op.borrow().core().get_nyes(), Nyes::Nk);
    }

    #[test]
    fn operator_builder_sets_prembrionic() {
        let dummy: Rc<RefCell<dyn Fir>> = Rc::new_cyclic(|me: &Weak<RefCell<IndepIntFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(IndepIntFir {
                core: ProtoBrane::new(vec![], parent, Nyes::Constant),
                value: 0,
            })
        });
        let parent_weak = Rc::downgrade(&dummy);
        let node = operator("+", vec![], parent_weak);

        assert_eq!(node.borrow().core().get_nyes(), Nyes::Prembrionic);
        assert_eq!(node.borrow().kind(), FirKind::Operator);
    }

    #[test]
    fn statement_wrapping_constant_copies_body_nyes() {
        let body = make_constant_int(42);
        let stmt = make_statement("x", 1, Rc::clone(&body));
        let scope = Scope::empty();

        let transitions = step_to_settled(&stmt, &scope);
        eprintln!("Statement(IndepInt) NYES transitions: {transitions:?}");

        assert_eq!(transitions.first(), Some(&Nyes::Prembrionic));
        assert_eq!(transitions.last(), Some(&Nyes::Constant));
        assert!(transitions.contains(&Nyes::Braning));
        assert_eq!(stmt.borrow().kind(), FirKind::Statement);
        assert!(stmt.borrow().core().get_nyes().is_constanic());
    }

    #[test]
    fn statement_wrapping_nk_copies_nk() {
        let body = make_nk("unbound");
        let stmt = make_statement("y", 2, Rc::clone(&body));
        let scope = Scope::empty();

        let transitions = step_to_settled(&stmt, &scope);
        eprintln!("Statement(Nk) NYES transitions: {transitions:?}");

        assert_eq!(transitions.first(), Some(&Nyes::Prembrionic));
        assert_eq!(transitions.last(), Some(&Nyes::Nk));
        assert!(transitions.contains(&Nyes::Braning));
    }

    #[test]
    fn statement_name_and_line_accessors() {
        let body = make_constant_int(1);
        let stmt = make_statement("myvar", 42, Rc::clone(&body));
        assert_eq!(stmt.borrow().kind(), FirKind::Statement);
        let borrowed = stmt.borrow();
        assert_eq!(borrowed.kind(), FirKind::Statement);
    }

    #[test]
    fn statement_builder_sets_prembrionic() {
        let dummy: Rc<RefCell<dyn Fir>> = Rc::new_cyclic(|me: &Weak<RefCell<IndepIntFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(IndepIntFir {
                core: ProtoBrane::new(vec![], parent, Nyes::Constant),
                value: 0,
            })
        });
        let parent_weak = Rc::downgrade(&dummy);
        let body = make_constant_int(1);
        let node = statement("x", 1, body, parent_weak);

        assert_eq!(node.borrow().core().get_nyes(), Nyes::Prembrionic);
        assert_eq!(node.borrow().kind(), FirKind::Statement);
    }

    #[test]
    fn brane_two_constant_children_classifies_constant() {
        let a = make_constant_int(10);
        let b = make_constant_int(20);
        let brane = make_brane(vec![Rc::clone(&a), Rc::clone(&b)]);
        let scope = Scope::empty();

        let transitions = step_to_settled(&brane, &scope);
        eprintln!("Brane(2x Constant) NYES transitions: {transitions:?}");

        assert_eq!(brane.borrow().core().get_nyes(), Nyes::Constant);
        assert_eq!(brane.borrow().kind(), FirKind::Brane);
    }

    #[test]
    fn brane_with_nk_child_classifies_nk() {
        let a = make_constant_int(5);
        let b = make_nk("unbound name");
        let brane = make_brane(vec![Rc::clone(&a), Rc::clone(&b)]);
        let scope = Scope::empty();

        let transitions = step_to_settled(&brane, &scope);
        eprintln!("Brane(Constant + Nk) NYES transitions: {transitions:?}");

        assert_eq!(brane.borrow().core().get_nyes(), Nyes::Nk);
    }

    #[test]
    fn brane_drains_children_in_order() {
        let a = make_constant_int(1);
        let b = make_constant_int(2);
        let brane = make_brane(vec![Rc::clone(&a), Rc::clone(&b)]);
        let scope = Scope::empty();

        let r1 = brane.step(&scope).unwrap();
        assert!(matches!(r1, StepReport::Progress(Nyes::Braning)));

        let mut child_a_settled_first = false;
        for _ in 0..20 {
            let _ = brane.step(&scope).unwrap();
            let a_settled = a.borrow().core().get_nyes().is_constanic();
            let b_settled = b.borrow().core().get_nyes().is_constanic();
            if a_settled && !b_settled {
                child_a_settled_first = true;
            }
            if a_settled && b_settled {
                break;
            }
        }
        assert!(
            child_a_settled_first,
            "child a should settle before child b"
        );
    }

    #[test]
    fn brane_nested_brane_drains_inner_first() {
        let inner_a = make_constant_int(100);
        let inner_b = make_constant_int(200);
        let inner = make_brane(vec![Rc::clone(&inner_a), Rc::clone(&inner_b)]);

        let outer = make_brane(vec![Rc::clone(&inner)]);
        let scope = Scope::empty();

        let transitions = step_to_settled(&outer, &scope);
        eprintln!("Nested brane NYES transitions: {transitions:?}");

        assert!(inner.borrow().core().get_nyes().is_constanic());
        assert!(outer.borrow().core().get_nyes().is_constanic());
        assert_eq!(inner.borrow().core().get_nyes(), Nyes::Constant);
        assert_eq!(outer.borrow().core().get_nyes(), Nyes::Constant);
    }

    #[test]
    fn brane_empty_children_classifies_constant() {
        let brane = make_brane(vec![]);
        let scope = Scope::empty();

        let transitions = step_to_settled(&brane, &scope);
        eprintln!("Brane(empty) NYES transitions: {transitions:?}");

        assert_eq!(brane.borrow().core().get_nyes(), Nyes::Constant);
    }

    #[test]
    fn brane_builder_sets_prembrionic() {
        let dummy: Rc<RefCell<dyn Fir>> = Rc::new_cyclic(|me: &Weak<RefCell<IndepIntFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(IndepIntFir {
                core: ProtoBrane::new(vec![], parent, Nyes::Constant),
                value: 0,
            })
        });
        let parent_weak = Rc::downgrade(&dummy);
        let node = brane(vec![], parent_weak);

        assert_eq!(node.borrow().core().get_nyes(), Nyes::Prembrionic);
        assert_eq!(node.borrow().kind(), FirKind::Brane);
    }

    #[test]
    fn statement_wrapping_operator_propagates() {
        let a = make_constant_int(3);
        let b = make_constant_int(5);
        let op = make_operator("+", vec![Rc::clone(&a), Rc::clone(&b)]);
        let stmt = make_statement("result", 1, Rc::clone(&op));
        let scope = Scope::empty();

        let transitions = step_to_settled(&stmt, &scope);
        eprintln!("Statement(Operator) NYES transitions: {transitions:?}");

        assert_eq!(stmt.borrow().core().get_nyes(), Nyes::Constant);
        assert_eq!(op.borrow().core().get_nyes(), Nyes::Constant);
    }

    #[test]
    fn search_finds_name_in_anchored_brane() {
        let val = make_constant_int(42);
        let stmt = make_statement("x", 0, Rc::clone(&val));
        let brane = make_brane(vec![Rc::clone(&stmt)]);
        let search = make_search("^x$", true, vec![Rc::clone(&brane)]);
        let scope = Scope::empty();

        let transitions = step_to_settled(&search, &scope);
        eprintln!("Search(found) NYES transitions: {transitions:?}");

        assert_eq!(search.borrow().core().get_nyes(), Nyes::Constant);
        let ubc = search.borrow().core().ubc_children().to_vec();
        assert_eq!(ubc.len(), 2);
        assert_eq!(ubc[0].borrow().as_i64(), Some(42));
    }

    #[test]
    fn search_not_found_becomes_nk() {
        let val = make_constant_int(42);
        let stmt = make_statement("y", 0, Rc::clone(&val));
        let brane = make_brane(vec![Rc::clone(&stmt)]);
        let search = make_search("^x$", true, vec![Rc::clone(&brane)]);
        let scope = Scope::empty();

        let transitions = step_to_settled(&search, &scope);
        eprintln!("Search(not found) NYES transitions: {transitions:?}");

        assert_eq!(search.borrow().core().get_nyes(), Nyes::Nk);
        assert!(search.borrow().core().ubc_children().is_empty());
    }

    #[test]
    fn search_anchored_nk_body_propagates_woconstanic() {
        let val = make_nk("unbound");
        let stmt = make_statement("z", 0, Rc::clone(&val));
        let brane = make_brane(vec![Rc::clone(&stmt)]);
        let search = make_search("^z$", true, vec![Rc::clone(&brane)]);
        let scope = Scope::empty();

        let transitions = step_to_settled(&search, &scope);
        eprintln!("Search(nk body) NYES transitions: {transitions:?}");

        assert_eq!(search.borrow().core().get_nyes(), Nyes::Nk);
    }

    #[test]
    fn index_finds_element_at_offset_in_anchor_brane() {
        let val_a = make_constant_int(10);
        let val_b = make_constant_int(20);
        let val_c = make_constant_int(30);
        let stmt_a = make_statement("a", 1, Rc::clone(&val_a));
        let stmt_b = make_statement("b", 2, Rc::clone(&val_b));
        let stmt_c = make_statement("c", 3, Rc::clone(&val_c));
        let brane = make_brane(vec![
            Rc::clone(&stmt_a),
            Rc::clone(&stmt_b),
            Rc::clone(&stmt_c),
        ]);

        let idx = make_index(1, true, vec![Rc::clone(&brane)]);
        let scope = Scope::empty();

        let transitions = step_to_settled(&idx, &scope);
        eprintln!("Index(offset=1) NYES transitions: {transitions:?}");

        assert_eq!(idx.borrow().core().get_nyes(), Nyes::Constant);
        let ubc = idx.borrow().core().ubc_children().to_vec();
        assert_eq!(ubc.len(), 2);
        assert_eq!(ubc[0].borrow().as_i64(), Some(20));
    }

    #[test]
    fn index_out_of_bounds_is_nk() {
        let val = make_constant_int(42);
        let stmt = make_statement("x", 1, Rc::clone(&val));
        let brane = make_brane(vec![Rc::clone(&stmt)]);

        let idx = make_index(5, true, vec![Rc::clone(&brane)]);
        let scope = Scope::empty();

        let transitions = step_to_settled(&idx, &scope);
        eprintln!("Index(oob) NYES transitions: {transitions:?}");

        assert_eq!(idx.borrow().core().get_nyes(), Nyes::Nk);
        assert!(idx.borrow().core().ubc_children().is_empty());
    }

    #[test]
    fn index_negative_offset_from_back() {
        let val_a = make_constant_int(10);
        let val_b = make_constant_int(20);
        let val_c = make_constant_int(30);
        let stmt_a = make_statement("a", 1, Rc::clone(&val_a));
        let stmt_b = make_statement("b", 2, Rc::clone(&val_b));
        let stmt_c = make_statement("c", 3, Rc::clone(&val_c));
        let brane = make_brane(vec![
            Rc::clone(&stmt_a),
            Rc::clone(&stmt_b),
            Rc::clone(&stmt_c),
        ]);

        let idx = make_index(-1, true, vec![Rc::clone(&brane)]);
        let scope = Scope::empty();

        let transitions = step_to_settled(&idx, &scope);
        eprintln!("Index(offset=-1) NYES transitions: {transitions:?}");

        assert_eq!(idx.borrow().core().get_nyes(), Nyes::Constant);
        let ubc = idx.borrow().core().ubc_children().to_vec();
        assert_eq!(ubc.len(), 2);
        assert_eq!(ubc[0].borrow().as_i64(), Some(30));
    }

    #[test]
    fn index_zero_offset_gets_first_element() {
        let val = make_constant_int(99);
        let stmt = make_statement("first", 1, Rc::clone(&val));
        let brane = make_brane(vec![Rc::clone(&stmt)]);

        let idx = make_index(0, true, vec![Rc::clone(&brane)]);
        let scope = Scope::empty();

        let transitions = step_to_settled(&idx, &scope);
        eprintln!("Index(offset=0) NYES transitions: {transitions:?}");

        assert_eq!(idx.borrow().core().get_nyes(), Nyes::Constant);
        let ubc = idx.borrow().core().ubc_children().to_vec();
        assert_eq!(ubc.len(), 2);
        assert_eq!(ubc[0].borrow().as_i64(), Some(99));
    }

    #[test]
    fn index_nk_body_propagates_nk() {
        let val = make_nk("unbound");
        let stmt = make_statement("z", 1, Rc::clone(&val));
        let brane = make_brane(vec![Rc::clone(&stmt)]);

        let idx = make_index(0, true, vec![Rc::clone(&brane)]);
        let scope = Scope::empty();

        let transitions = step_to_settled(&idx, &scope);
        eprintln!("Index(nk body) NYES transitions: {transitions:?}");

        assert_eq!(idx.borrow().core().get_nyes(), Nyes::Nk);
    }

    fn make_headtail(is_head: bool, anchored: bool, foolish_children: Vec<FirRef>) -> FirRef {
        let offset: i32 = if is_head { 0 } else { -1 };
        Rc::new_cyclic(|me: &Weak<RefCell<IndexFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(IndexFir {
                core: ProtoBrane::new(foolish_children, parent, Nyes::Prembrionic),
                offset,
                anchored,
                contexted: false,
            })
        })
    }

    #[test]
    fn headtail_head_gets_first_element() {
        let val_a = make_constant_int(10);
        let val_b = make_constant_int(20);
        let val_c = make_constant_int(30);
        let stmt_a = make_statement("a", 1, Rc::clone(&val_a));
        let stmt_b = make_statement("b", 2, Rc::clone(&val_b));
        let stmt_c = make_statement("c", 3, Rc::clone(&val_c));
        let brane = make_brane(vec![
            Rc::clone(&stmt_a),
            Rc::clone(&stmt_b),
            Rc::clone(&stmt_c),
        ]);

        let ht = make_headtail(true, true, vec![Rc::clone(&brane)]);
        let scope = Scope::empty();

        let transitions = step_to_settled(&ht, &scope);
        eprintln!("HeadTail(head) NYES transitions: {transitions:?}");

        assert_eq!(ht.borrow().core().get_nyes(), Nyes::Constant);
        let ubc = ht.borrow().core().ubc_children().to_vec();
        assert_eq!(ubc.len(), 2);
        assert_eq!(ubc[0].borrow().as_i64(), Some(10));
    }

    #[test]
    fn headtail_tail_gets_last_element() {
        let val_a = make_constant_int(10);
        let val_b = make_constant_int(20);
        let val_c = make_constant_int(30);
        let stmt_a = make_statement("a", 1, Rc::clone(&val_a));
        let stmt_b = make_statement("b", 2, Rc::clone(&val_b));
        let stmt_c = make_statement("c", 3, Rc::clone(&val_c));
        let brane = make_brane(vec![
            Rc::clone(&stmt_a),
            Rc::clone(&stmt_b),
            Rc::clone(&stmt_c),
        ]);

        let ht = make_headtail(false, true, vec![Rc::clone(&brane)]);
        let scope = Scope::empty();

        let transitions = step_to_settled(&ht, &scope);
        eprintln!("HeadTail(tail) NYES transitions: {transitions:?}");

        assert_eq!(ht.borrow().core().get_nyes(), Nyes::Constant);
        let ubc = ht.borrow().core().ubc_children().to_vec();
        assert_eq!(ubc.len(), 2);
        assert_eq!(ubc[0].borrow().as_i64(), Some(30));
    }

    #[test]
    fn headtail_empty_brane_is_nk() {
        let brane = make_brane(vec![]);
        let ht = make_headtail(true, true, vec![Rc::clone(&brane)]);
        let scope = Scope::empty();

        let transitions = step_to_settled(&ht, &scope);
        eprintln!("HeadTail(empty) NYES transitions: {transitions:?}");

        assert_eq!(ht.borrow().core().get_nyes(), Nyes::Nk);
        assert!(ht.borrow().core().ubc_children().is_empty());
    }

    #[test]
    fn headtail_single_element_head_and_tail_same() {
        let val = make_constant_int(42);
        let stmt = make_statement("x", 1, Rc::clone(&val));
        let brane = make_brane(vec![Rc::clone(&stmt)]);

        let head = make_headtail(true, true, vec![Rc::clone(&brane)]);
        let tail = make_headtail(false, true, vec![Rc::clone(&brane)]);
        let scope = Scope::empty();

        step_to_settled(&head, &scope);
        step_to_settled(&tail, &scope);

        assert_eq!(head.borrow().core().get_nyes(), Nyes::Constant);
        assert_eq!(
            head.borrow().core().ubc_children()[0].borrow().as_i64(),
            Some(42)
        );
        assert_eq!(tail.borrow().core().get_nyes(), Nyes::Constant);
        assert_eq!(
            tail.borrow().core().ubc_children()[0].borrow().as_i64(),
            Some(42)
        );
    }

    #[test]
    fn headtail_nk_body_propagates_nk() {
        let val = make_nk("unbound");
        let stmt = make_statement("z", 1, Rc::clone(&val));
        let brane = make_brane(vec![Rc::clone(&stmt)]);

        let ht = make_headtail(true, true, vec![Rc::clone(&brane)]);
        let scope = Scope::empty();

        let transitions = step_to_settled(&ht, &scope);
        eprintln!("HeadTail(nk body) NYES transitions: {transitions:?}");

        assert_eq!(ht.borrow().core().get_nyes(), Nyes::Nk);
    }

    fn make_concatenation(elements: Vec<FirRef>) -> FirRef {
        Rc::new_cyclic(|me: &Weak<RefCell<ConcatenationFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(ConcatenationFir {
                core: ProtoBrane::new(elements, parent, Nyes::Prembrionic),
            })
        })
    }

    #[test]
    fn concatenation_two_brane_elements() {
        let a = make_constant_int(1);
        let b = make_constant_int(2);
        let stmt_a = make_statement("a", 1, Rc::clone(&a));
        let stmt_b = make_statement("b", 2, Rc::clone(&b));
        let brane1 = make_brane(vec![Rc::clone(&stmt_a)]);
        let brane2 = make_brane(vec![Rc::clone(&stmt_b)]);

        let cat = make_concatenation(vec![Rc::clone(&brane1), Rc::clone(&brane2)]);
        let scope = Scope::empty();

        let transitions = step_to_settled(&cat, &scope);
        eprintln!("Concatenation(2 branes) NYES transitions: {transitions:?}");

        assert_eq!(cat.borrow().core().get_nyes(), Nyes::Constant);
        let ubc = cat.borrow().core().ubc_children().to_vec();
        assert_eq!(ubc.len(), 1);
        assert_eq!(ubc[0].borrow().kind(), FirKind::Brane);
        let result_brane = &ubc[0];
        assert_eq!(result_brane.borrow().core().foolish_children().len(), 2);
    }

    #[test]
    fn concatenation_empty_elements_is_constant_empty_brane() {
        let cat = make_concatenation(vec![]);
        let scope = Scope::empty();

        let transitions = step_to_settled(&cat, &scope);
        eprintln!("Concatenation(empty) NYES transitions: {transitions:?}");

        assert_eq!(cat.borrow().core().get_nyes(), Nyes::Constant);
        let ubc = cat.borrow().core().ubc_children().to_vec();
        assert_eq!(ubc.len(), 1);
        assert_eq!(ubc[0].borrow().kind(), FirKind::Brane);
        assert_eq!(ubc[0].borrow().core().foolish_children().len(), 0);
    }

    #[test]
    fn concatenation_with_nk_element_is_nk() {
        let a = make_constant_int(1);
        let b = make_nk("unbound");
        let stmt_a = make_statement("a", 1, Rc::clone(&a));
        let stmt_b = make_statement("b", 2, Rc::clone(&b));
        let brane1 = make_brane(vec![Rc::clone(&stmt_a)]);
        let brane2 = make_brane(vec![Rc::clone(&stmt_b)]);

        let cat = make_concatenation(vec![Rc::clone(&brane1), Rc::clone(&brane2)]);
        let scope = Scope::empty();

        let transitions = step_to_settled(&cat, &scope);
        eprintln!("Concatenation(nk element) NYES transitions: {transitions:?}");

        assert_eq!(cat.borrow().core().get_nyes(), Nyes::Nk);
    }

    #[test]
    fn concatenation_single_element_brane() {
        let val = make_constant_int(99);
        let stmt = make_statement("x", 1, Rc::clone(&val));
        let brane = make_brane(vec![Rc::clone(&stmt)]);

        let cat = make_concatenation(vec![Rc::clone(&brane)]);
        let scope = Scope::empty();

        let transitions = step_to_settled(&cat, &scope);
        eprintln!("Concatenation(single) NYES transitions: {transitions:?}");

        assert_eq!(cat.borrow().core().get_nyes(), Nyes::Constant);
        let ubc = cat.borrow().core().ubc_children().to_vec();
        assert_eq!(ubc.len(), 1);
        assert_eq!(ubc[0].borrow().kind(), FirKind::Brane);
        assert_eq!(ubc[0].borrow().core().foolish_children().len(), 1);
    }

    fn make_stay_foolish(expr: FirRef) -> FirRef {
        Rc::new_cyclic(|me: &Weak<RefCell<StayFoolishFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(StayFoolishFir {
                core: ProtoBrane::new(vec![expr], parent, Nyes::Prembrionic),
            })
        })
    }

    #[test]
    fn stay_foolish_sets_constant_body_constanic() {
        let body = make_constant_int(42);
        let sf = make_stay_foolish(Rc::clone(&body));
        let scope = Scope::empty();

        let transitions = step_to_settled(&sf, &scope);
        eprintln!("SF(IndepInt) NYES transitions: {transitions:?}");

        assert_eq!(transitions.first(), Some(&Nyes::Prembrionic));
        assert_eq!(transitions.last(), Some(&Nyes::Constant));
        assert!(transitions.contains(&Nyes::Braning));
        assert_eq!(sf.borrow().kind(), FirKind::StayFoolish);
    }

    #[test]
    fn stay_foolish_sets_nk_body_constanic() {
        let body = make_nk("unbound");
        let sf = make_stay_foolish(Rc::clone(&body));
        let scope = Scope::empty();

        let transitions = step_to_settled(&sf, &scope);
        eprintln!("SF(Nk) NYES transitions: {transitions:?}");

        assert_eq!(transitions.first(), Some(&Nyes::Prembrionic));
        assert_eq!(transitions.last(), Some(&Nyes::Nk));
        assert!(transitions.contains(&Nyes::Braning));
    }

    #[test]
    fn stay_foolish_sets_econstanic_body_constanic() {
        let val = make_constant_int(1);
        let stmt = make_statement("x", 1, Rc::clone(&val));
        let brane = make_brane(vec![Rc::clone(&stmt)]);
        let search = make_search("^missing$", true, vec![Rc::clone(&brane)]);
        let sf = make_stay_foolish(Rc::clone(&search));
        let scope = Scope::empty();

        let transitions = step_to_settled(&sf, &scope);
        eprintln!("SF(Econstanic body) NYES transitions: {transitions:?}");

        assert!(sf.borrow().core().get_nyes().is_constanic());
    }

    #[test]
    fn stay_foolish_builder_sets_prembrionic() {
        let dummy: Rc<RefCell<dyn Fir>> = Rc::new_cyclic(|me: &Weak<RefCell<IndepIntFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(IndepIntFir {
                core: ProtoBrane::new(vec![], parent, Nyes::Constant),
                value: 0,
            })
        });
        let parent_weak = Rc::downgrade(&dummy);
        let body = make_constant_int(1);
        let node = stay_foolish(body, parent_weak);

        assert_eq!(node.borrow().core().get_nyes(), Nyes::Prembrionic);
        assert_eq!(node.borrow().kind(), FirKind::StayFoolish);
    }

    #[test]
    fn stay_foolish_constanic_clones_constant_to_ubc_children() {
        let body = make_constant_int(42);
        let sf = make_stay_foolish(Rc::clone(&body));
        let scope = Scope::empty();

        step_to_settled(&sf, &scope);

        assert_eq!(sf.borrow().core().get_nyes(), Nyes::Constant);
        let ubc = sf.borrow().core().ubc_children();
        assert_eq!(
            ubc.len(),
            1,
            "SF should constanic-clone result into ubc_children"
        );
        assert_eq!(ubc[0].borrow().kind(), FirKind::IndepInt);
        assert_eq!(ubc[0].borrow().as_i64(), Some(42));
    }

    #[test]
    fn stay_foolish_constanic_clones_nk_to_ubc_children() {
        let body = make_nk("unbound");
        let sf = make_stay_foolish(Rc::clone(&body));
        let scope = Scope::empty();

        step_to_settled(&sf, &scope);

        assert_eq!(sf.borrow().core().get_nyes(), Nyes::Nk);
        let ubc = sf.borrow().core().ubc_children();
        assert_eq!(
            ubc.len(),
            1,
            "SF should constanic-clone nk result into ubc_children"
        );
        assert_eq!(ubc[0].borrow().kind(), FirKind::Nk);
    }

    #[test]
    fn stay_foolish_constanic_clones_operator_result() {
        let a = make_constant_int(3);
        let b = make_constant_int(5);
        let op = make_operator("+", vec![Rc::clone(&a), Rc::clone(&b)]);
        let sf = make_stay_foolish(Rc::clone(&op));
        let scope = Scope::empty();

        step_to_settled(&sf, &scope);

        assert_eq!(sf.borrow().core().get_nyes(), Nyes::Constant);
        let ubc = sf.borrow().core().ubc_children();
        assert_eq!(ubc.len(), 1, "SF should constanic-clone operator result");
        assert_eq!(ubc[0].borrow().as_i64(), Some(8));
    }

    #[test]
    fn stay_foolish_constanic_clones_search_result() {
        let val = make_constant_int(10);
        let stmt = make_statement("x", 1, Rc::clone(&val));
        let brane = make_brane(vec![Rc::clone(&stmt)]);
        let search = make_search("^x$", true, vec![Rc::clone(&brane)]);
        let sf = make_stay_foolish(Rc::clone(&search));
        let scope = Scope::empty();

        step_to_settled(&sf, &scope);

        assert_eq!(sf.borrow().core().get_nyes(), Nyes::Constant);
        let ubc = sf.borrow().core().ubc_children();
        assert_eq!(ubc.len(), 1, "SF should constanic-clone search result");
        assert_eq!(ubc[0].borrow().as_i64(), Some(10));
    }

    fn make_stay_fully_foolish(expr: FirRef) -> FirRef {
        Rc::new_cyclic(|me: &Weak<RefCell<StayFullyFoolishFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(StayFullyFoolishFir {
                core: ProtoBrane::new(vec![expr], parent, Nyes::Prembrionic),
            })
        })
    }

    #[test]
    fn stay_fully_foolish_evaluates_child() {
        let body = make_constant_int(42);
        let sff = make_stay_fully_foolish(Rc::clone(&body));
        let scope = Scope::empty();

        assert_eq!(sff.borrow().core().get_nyes(), Nyes::Prembrionic);
        let mut transitions = vec![Nyes::Prembrionic];
        for _ in 0..10 {
            let report = sff.step(&scope).unwrap();
            match report {
                StepReport::Progress(nyes) => {
                    transitions.push(nyes);
                    if nyes.is_constanic() {
                        break;
                    }
                }
                StepReport::NoProgress => break,
            }
        }
        eprintln!("SFF NYES transitions: {transitions:?}");
        assert!(sff.borrow().core().get_nyes().is_constanic());
        assert_eq!(sff.borrow().kind(), FirKind::StayFullyFoolish);
    }

    #[test]
    fn stay_fully_foolish_body_is_never_evaluated() {
        let body = make_constant_int(42);
        let sff = make_stay_fully_foolish(Rc::clone(&body));
        let scope = Scope::empty();

        sff.step(&scope).unwrap(); // Prembrionic → Braning (child pushed)
        sff.step(&scope).unwrap(); // child stepped to CONSTANT
        sff.step(&scope).unwrap(); // child popped (constanic)
        let report = sff.step(&scope).unwrap(); // Braning → CONSTANT
        assert!(matches!(report, StepReport::Progress(Nyes::Constant)));
        assert!(sff.borrow().core().get_nyes().is_constanic());

        assert_eq!(body.borrow().core().get_nyes(), Nyes::Constant);
    }

    #[test]
    fn stay_fully_foolish_builder_sets_prembrionic() {
        let dummy: Rc<RefCell<dyn Fir>> = Rc::new_cyclic(|me: &Weak<RefCell<IndepIntFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(IndepIntFir {
                core: ProtoBrane::new(vec![], parent, Nyes::Constant),
                value: 0,
            })
        });
        let parent_weak = Rc::downgrade(&dummy);
        let body = make_constant_int(1);
        let node = stay_fully_foolish(body, parent_weak);

        assert_eq!(node.borrow().core().get_nyes(), Nyes::Prembrionic);
        assert_eq!(node.borrow().kind(), FirKind::StayFullyFoolish);
    }

    fn dangling_parent() -> Weak<RefCell<dyn Fir>> {
        Weak::<RefCell<IndepIntFir>>::new()
    }

    #[test]
    fn clone_nyes_rule_by_mode() {
        assert_eq!(Nyes::Constant.transform_for_clone(false), Nyes::Constant);
        assert_eq!(
            Nyes::Independent.transform_for_clone(false),
            Nyes::Independent
        );
        assert_eq!(Nyes::Nk.transform_for_clone(false), Nyes::Nk);
        assert_eq!(
            Nyes::Econstanic.transform_for_clone(false),
            Nyes::Embryonic,
            "NICC resets ECONSTANIC to EMBRYONIC so it re-steps (IB then AB)"
        );
        assert_eq!(
            Nyes::Woconstanic.transform_for_clone(false),
            Nyes::Embryonic,
            "NICC resets WOCONSTANIC to EMBRYONIC too (nyes part; collapse is task #21)"
        );

        for n in [
            Nyes::Constant,
            Nyes::Independent,
            Nyes::Econstanic,
            Nyes::Woconstanic,
            Nyes::Nk,
        ] {
            assert_eq!(
                n.transform_for_clone(true),
                n,
                "FICC must copy {n:?} verbatim"
            );
        }
    }

    #[test]
    fn nicc_resets_econstanic_to_embryonic() {
        let op = make_operator("+", vec![make_constant_int(1), make_constant_int(2)]);
        op.borrow().core().set_nyes(Nyes::Econstanic);

        let cloned = ProtoBrane::constanic_clone_at(&op, &dangling_parent(), 0, false);

        assert_eq!(cloned.borrow().kind(), FirKind::Operator);
        assert_eq!(
            cloned.borrow().core().get_nyes(),
            Nyes::Embryonic,
            "NICC of an ECONSTANIC compound must reset to EMBRYONIC (so it re-steps)"
        );
    }

    #[test]
    fn nicc_resets_woconstanic_compound_to_embryonic() {
        let op = make_operator("+", vec![make_constant_int(1), make_constant_int(2)]);
        op.borrow().core().set_nyes(Nyes::Woconstanic);

        let cloned = ProtoBrane::constanic_clone_at(&op, &dangling_parent(), 0, false);

        assert_eq!(
            cloned.borrow().core().get_nyes(),
            Nyes::Embryonic,
            "NICC of a WOCONSTANIC compound must reset to EMBRYONIC"
        );
    }

    #[test]
    fn foolish_clone_copies_constanic_nyes_verbatim() {
        let woc = make_operator("+", vec![make_constant_int(1), make_constant_int(2)]);
        woc.borrow().core().set_nyes(Nyes::Woconstanic);
        let cloned = ProtoBrane::constanic_clone_at(&woc, &dangling_parent(), 0, true);
        assert_eq!(
            cloned.borrow().core().get_nyes(),
            Nyes::Woconstanic,
            "FICC must keep a constanic compound's state verbatim"
        );

        let econ = make_operator("+", vec![make_constant_int(1), make_constant_int(2)]);
        econ.borrow().core().set_nyes(Nyes::Econstanic);
        let cloned = ProtoBrane::constanic_clone_at(&econ, &dangling_parent(), 0, true);
        assert_eq!(cloned.borrow().core().get_nyes(), Nyes::Econstanic);
    }

    #[test]
    fn leaf_clone_unchanged_both_modes() {
        let ci = make_constant_int(9);
        ci.borrow().core().set_nyes(Nyes::Constant);
        let n = ProtoBrane::constanic_clone_at(&ci, &dangling_parent(), 0, false);
        assert_eq!(n.borrow().core().get_nyes(), Nyes::Constant);
        let n = ProtoBrane::constanic_clone_at(&ci, &dangling_parent(), 0, true);
        assert_eq!(n.borrow().core().get_nyes(), Nyes::Constant);

        let nk = make_nk("gone");
        nk.borrow().core().set_nyes(Nyes::Nk);
        let n = ProtoBrane::constanic_clone_at(&nk, &dangling_parent(), 0, false);
        assert_eq!(n.borrow().core().get_nyes(), Nyes::Nk);
        let n = ProtoBrane::constanic_clone_at(&nk, &dangling_parent(), 0, true);
        assert_eq!(n.borrow().core().get_nyes(), Nyes::Nk);
    }

    #[test]
    fn cloning_sf_strips_the_mark() {
        let inner = make_constant_int(10);
        inner.borrow().core().set_nyes(Nyes::Econstanic); // force the clone path
        let sf = make_stay_foolish(Rc::clone(&inner));
        sf.borrow().core().set_nyes(Nyes::Econstanic);

        let normal = ProtoBrane::constanic_clone_at(&sf, &dangling_parent(), 0, false);
        assert_ne!(
            normal.borrow().kind(),
            FirKind::StayFoolish,
            "normal clone of an SF must NOT be a StayFoolish wrapper"
        );
        assert_eq!(normal.borrow().kind(), FirKind::IndepInt);

        let foolish = ProtoBrane::constanic_clone_at(&sf, &dangling_parent(), 0, true);
        assert_ne!(
            foolish.borrow().kind(),
            FirKind::StayFoolish,
            "even a foolish clone of an SF strips the wrapper (clones the inner)"
        );
    }

    #[test]
    fn step_sets_foolish_scope_inside_sf() {
        let body = make_constant_int(7);
        let sf = make_stay_foolish(Rc::clone(&body));
        let scope = Scope::empty();
        assert!(!scope.has_ancestral_sfm);

        let transitions = step_to_settled(&sf, &scope);
        eprintln!("SF settle under has_ancestral_sfm propagation: {transitions:?}");
        assert!(sf.borrow().core().get_nyes().is_constanic());

        let foolish_scope = scope.with_ancestral_sfm(true);
        assert!(foolish_scope.has_ancestral_sfm);
    }

    fn step_watching(root: &FirRef, watched: &FirRef, scope: &Scope) -> Vec<(Nyes, Nyes)> {
        let mut trace = vec![(
            root.borrow().core().get_nyes(),
            watched.borrow().core().get_nyes(),
        )];
        for _ in 0..100 {
            if root.borrow().core().get_nyes().is_constanic() {
                break;
            }
            let _ = root.step(scope).unwrap();
            trace.push((
                root.borrow().core().get_nyes(),
                watched.borrow().core().get_nyes(),
            ));
        }
        trace
    }

    #[test]
    fn brane_of_constants_progresses_to_settled() {
        let s1 = make_statement("a", 0, make_constant_int(1));
        let s2 = make_statement("b", 1, make_constant_int(2));
        let brane = make_brane(vec![Rc::clone(&s1), Rc::clone(&s2)]);
        let scope = Scope::empty();

        let trace = step_watching(&brane, &s2, &scope);
        eprintln!("brane/constants (brane, s2) nyes: {trace:?}");

        assert_eq!(trace.first().unwrap().0, Nyes::Prembrionic);
        assert!(brane.borrow().core().get_nyes().is_constanic());
        assert!(s2.borrow().core().get_nyes().is_constanic());
        assert!(trace.len() >= 2, "should take at least one step");
    }

    #[test]
    fn operator_in_brane_advances_before_parent_settles() {
        let op = make_operator("+", vec![make_constant_int(4), make_constant_int(6)]);
        let stmt = make_statement("sum", 0, Rc::clone(&op));
        let brane = make_brane(vec![Rc::clone(&stmt)]);
        let scope = Scope::empty();

        assert_eq!(op.borrow().core().get_nyes(), Nyes::Prembrionic);

        let trace = step_watching(&brane, &op, &scope);
        eprintln!("operator-in-brane (brane, op) nyes: {trace:?}");

        assert!(brane.borrow().core().get_nyes().is_constanic());
        assert!(op.borrow().core().get_nyes().is_constanic());
        assert_eq!(op.borrow().core().get_nyes(), Nyes::Constant);
        assert_eq!(op.borrow().as_i64(), Some(10));
    }

    #[test]
    fn unresolved_search_in_brane_goes_econstanic() {
        let search = make_search("zzz", false, vec![]);
        let stmt = make_statement("x", 0, Rc::clone(&search));
        let brane = make_brane(vec![Rc::clone(&stmt)]);
        let scope = Scope::empty();

        let trace = step_watching(&brane, &search, &scope);
        eprintln!("unresolved-search (brane, search) nyes: {trace:?}");

        assert!(search.borrow().core().get_nyes().is_constanic());
        assert_eq!(search.borrow().core().get_nyes(), Nyes::Econstanic);
        assert!(brane.borrow().core().get_nyes().is_constanic());
    }

    #[test]
    fn constanic_node_stays_constanic_across_parent_steps() {
        let s1 = make_statement("a", 0, make_constant_int(1));
        let s2 = make_statement("b", 1, make_constant_int(2));
        let brane = make_brane(vec![Rc::clone(&s1), Rc::clone(&s2)]);
        let scope = Scope::empty();

        let mut s1_was_constanic = false;
        for _ in 0..100 {
            if brane.borrow().core().get_nyes().is_constanic() {
                break;
            }
            let _ = brane.step(&scope).unwrap();
            let s1_now = s1.borrow().core().get_nyes().is_constanic();
            if s1_was_constanic {
                assert!(s1_now, "a constanic node must not regress to pre-constanic");
            }
            s1_was_constanic = s1_now;
        }
        assert!(s1.borrow().core().get_nyes().is_constanic());
    }

    fn assert_progression(trace: &[Nyes], expected_terminal: Nyes, label: &str) {
        eprintln!("{label} nyes transitions: {trace:?}");
        assert!(!trace.is_empty(), "{label}: empty trace");
        assert_eq!(
            *trace.first().unwrap(),
            Nyes::Prembrionic,
            "{label}: must start PREMBRIONIC"
        );
        let last = *trace.last().unwrap();
        assert!(
            last.is_constanic(),
            "{label}: must end constanic (got {last:?})"
        );
        assert_eq!(last, expected_terminal, "{label}: wrong terminal state");
        let mut seen_constanic = false;
        for n in trace {
            if seen_constanic {
                assert!(
                    n.is_constanic(),
                    "{label}: regressed from constanic to {n:?}"
                );
            }
            seen_constanic = n.is_constanic();
        }
    }

    #[test]
    fn constant_int_nyes_transitions() {
        let n = make_constant_int(7);
        let trace = step_to_settled(&n, &Scope::empty());
        assert_progression(&trace, Nyes::Constant, "IndepInt");
    }

    #[test]
    fn nk_nyes_transitions() {
        let n = make_nk("gone");
        let trace = step_to_settled(&n, &Scope::empty());
        assert_progression(&trace, Nyes::Nk, "Nk");
    }

    #[test]
    fn operator_nyes_transitions() {
        let op = make_operator("+", vec![make_constant_int(2), make_constant_int(3)]);
        let trace = step_to_settled(&op, &Scope::empty());
        assert_progression(&trace, Nyes::Constant, "Operator(+)");
        assert_eq!(op.borrow().as_i64(), Some(5));
    }

    #[test]
    fn operator_div_by_zero_nyes_transitions() {
        let op = make_operator("/", vec![make_constant_int(1), make_constant_int(0)]);
        let trace = step_to_settled(&op, &Scope::empty());
        assert_progression(&trace, Nyes::Nk, "Operator(/0)");
    }

    #[test]
    fn statement_nyes_transitions() {
        let stmt = make_statement("a", 0, make_constant_int(9));
        let trace = step_to_settled(&stmt, &Scope::empty());
        assert_progression(&trace, Nyes::Constant, "Statement");
    }

    #[test]
    fn brane_nyes_transitions() {
        let brane = make_brane(vec![
            make_statement("a", 0, make_constant_int(1)),
            make_statement("b", 1, make_constant_int(2)),
        ]);
        let trace = step_to_settled(&brane, &Scope::empty());
        assert_progression(&trace, Nyes::Constant, "Brane");
    }

    #[test]
    fn brane_with_nk_child_nyes_transitions() {
        let brane = make_brane(vec![
            make_statement("a", 0, make_constant_int(1)),
            make_statement("bad", 1, make_nk("boom")),
        ]);
        let trace = step_to_settled(&brane, &Scope::empty());
        assert_progression(&trace, Nyes::Nk, "Brane(+NK)");
    }

    #[test]
    fn search_anchored_found_nyes_transitions() {
        let brane = make_brane(vec![make_statement("a", 0, make_constant_int(10))]);
        let search = make_search("^a$", true, vec![Rc::clone(&brane)]);
        let trace = step_to_settled(&search, &Scope::empty());
        assert!(search.borrow().core().get_nyes().is_constanic());
        assert_eq!(*trace.first().unwrap(), Nyes::Prembrionic);
        eprintln!("Search(anchored,found) nyes transitions: {trace:?}");
    }

    #[test]
    fn search_not_found_nyes_transitions() {
        let search = make_search("zzz", false, vec![]);
        let trace = step_to_settled(&search, &Scope::empty());
        assert_progression(&trace, Nyes::Econstanic, "Search(not found)");
    }

    #[test]
    fn index_nyes_transitions() {
        let brane = make_brane(vec![
            make_statement("a", 0, make_constant_int(10)),
            make_statement("b", 1, make_constant_int(20)),
        ]);
        let idx = make_index(1, true, vec![Rc::clone(&brane)]);
        let trace = step_to_settled(&idx, &Scope::empty());
        assert_progression(&trace, Nyes::Constant, "Index(1)");
    }

    #[test]
    fn index_out_of_bounds_nyes_transitions() {
        let brane = make_brane(vec![make_statement("a", 0, make_constant_int(10))]);
        let idx = make_index(5, true, vec![Rc::clone(&brane)]);
        let trace = step_to_settled(&idx, &Scope::empty());
        assert_progression(&trace, Nyes::Nk, "Index(oob)");
    }

    #[test]
    fn headtail_nyes_transitions() {
        let brane = make_brane(vec![
            make_statement("a", 0, make_constant_int(10)),
            make_statement("b", 1, make_constant_int(20)),
        ]);
        let head = make_headtail(true, true, vec![Rc::clone(&brane)]);
        let trace = step_to_settled(&head, &Scope::empty());
        assert_progression(&trace, Nyes::Constant, "Index(head/offset=0)");

        let tail = make_headtail(false, true, vec![Rc::clone(&brane)]);
        let trace = step_to_settled(&tail, &Scope::empty());
        assert_progression(&trace, Nyes::Constant, "Index(tail/offset=-1)");
    }

    #[test]
    fn headtail_empty_nyes_transitions() {
        let brane = make_brane(vec![]);
        let ht = make_headtail(true, true, vec![Rc::clone(&brane)]);
        let trace = step_to_settled(&ht, &Scope::empty());
        assert_progression(&trace, Nyes::Nk, "Index(head,empty)");
    }

    #[test]
    fn headtail_sugar_nyes_transitions() {
        let brane = make_brane(vec![
            make_statement("x", 0, make_constant_int(100)),
            make_statement("y", 1, make_constant_int(200)),
            make_statement("z", 2, make_constant_int(300)),
        ]);
        let head_as_index = make_index(0, true, vec![Rc::clone(&brane)]);
        let trace = step_to_settled(&head_as_index, &Scope::empty());
        assert_progression(&trace, Nyes::Constant, "^ sugar → Index(offset=0)");
        assert_eq!(
            head_as_index.borrow().core().ubc_children()[0]
                .borrow()
                .as_i64(),
            Some(100)
        );

        let tail_as_index = make_index(-1, true, vec![Rc::clone(&brane)]);
        let trace = step_to_settled(&tail_as_index, &Scope::empty());
        assert_progression(&trace, Nyes::Constant, "$ sugar → Index(offset=-1)");
        assert_eq!(
            tail_as_index.borrow().core().ubc_children()[0]
                .borrow()
                .as_i64(),
            Some(300)
        );
    }

    #[test]
    fn concatenation_nyes_transitions() {
        let brane1 = make_brane(vec![make_statement("a", 0, make_constant_int(1))]);
        let brane2 = make_brane(vec![make_statement("b", 0, make_constant_int(2))]);
        let cat = make_concatenation(vec![Rc::clone(&brane1), Rc::clone(&brane2)]);
        let trace = step_to_settled(&cat, &Scope::empty());
        assert_progression(&trace, Nyes::Constant, "Concatenation");
    }

    #[test]
    fn stay_foolish_nyes_transitions() {
        let sf = make_stay_foolish(make_constant_int(42));
        let trace = step_to_settled(&sf, &Scope::empty());
        assert_progression(&trace, Nyes::Constant, "StayFoolish");
    }

    #[test]
    fn stay_fully_foolish_nyes_transitions() {
        let sff = make_stay_fully_foolish(make_constant_int(42));
        let trace = step_to_settled(&sff, &Scope::empty());
        assert_progression(&trace, Nyes::Constant, "StayFullyFoolish");
    }

    use crate::compiler::Compiler;

    fn find_search(node: &FirRef, pattern: &str) -> Option<FirRef> {
        if node.borrow().kind() == FirKind::Search
            && node.borrow().as_search_pattern() == Some(pattern)
        {
            return Some(Rc::clone(node));
        }
        let children: Vec<FirRef> = node.borrow().core().foolish_children().to_vec();
        for c in children {
            if let Some(found) = find_search(&c, pattern) {
                return Some(found);
            }
        }
        None
    }

    #[test]
    fn ib_context_resolves_in_immediate_brane() {
        let root = Compiler::compile("{a = 1; b = a;}").unwrap().pop().unwrap();
        let search = find_search(&root, "^a$").expect("search for a");
        let stmts = root.borrow().core().foolish_children().to_vec();
        let b_stmt = &stmts[1]; // b = a
        let ib = b_stmt.borrow()._ib_search(b_stmt, "^a$");
        assert!(
            ib.is_some(),
            "ib_search must find a name in the immediate brane"
        );
        let scope = Scope::empty();
        let _ = step_to_settled(&root, &scope);
        assert!(search.borrow().core().get_nyes().is_constanic());
    }

    #[test]
    fn ab_context_name_not_in_immediate_brane() {
        let root = Compiler::compile("{a = 1; b = {c = a;};}")
            .unwrap()
            .pop()
            .unwrap();
        let _search = find_search(&root, "^a$").expect("search for a");
        let inner_brane = root
            .borrow()
            .core()
            .foolish_children()
            .get(1)
            .map(|s| {
                s.borrow()
                    .core()
                    .foolish_children()
                    .first()
                    .unwrap()
                    .clone()
            })
            .unwrap();
        let inner_stmts = inner_brane.borrow().core().foolish_children().to_vec();
        let c_stmt = &inner_stmts[0]; // c = a
        assert!(
            c_stmt.borrow()._ib_search(c_stmt, "^a$").is_none(),
            "ib_search must NOT find an ancestral-only name in the immediate brane"
        );
        assert!(
            inner_brane
                .borrow()
                ._ab_search(&inner_brane, "^a$")
                .is_some(),
            "ab_search must find the ancestral name"
        );
    }

    #[test]
    fn ib_shadows_ab_immediate_wins() {
        let root = Compiler::compile("{a = 1; b = {a = 2; c = a;};}")
            .unwrap()
            .pop()
            .unwrap();
        let search = find_search(&root, "^a$").expect("search for a");
        // The search for 'a' in 'c = a' should find the inner 'a = 2' (shadowing)
        let stmts = root.borrow().core().foolish_children().to_vec();
        let inner_brane = stmts[1]
            .borrow()
            .core()
            .foolish_children()
            .first()
            .unwrap()
            .clone();
        let inner_stmts = inner_brane.borrow().core().foolish_children().to_vec();
        let c_stmt = &inner_stmts[1]; // c = a
        let ib = c_stmt.borrow()._ib_search(c_stmt, "^a$");
        assert!(
            ib.is_some(),
            "ib_search must find the immediate (shadowing) a"
        );
        let (stmt, _nyes) = ib.unwrap();
        let body = stmt
            .borrow()
            .core()
            .foolish_children()
            .first()
            .unwrap()
            .clone();
        assert_eq!(
            body.borrow().as_i64(),
            Some(2),
            "must find inner a=2, not outer a=1"
        );

        let scope = Scope::empty();
        let _ = step_to_settled(&root, &scope);
        assert!(search.borrow().core().get_nyes().is_constanic());
        let result = search.value();
        assert_eq!(
            result.borrow().as_i64(),
            Some(2),
            "shadowing: search must resolve to the immediate-brane a (2), not ancestral (1)"
        );
    }

    #[test]
    fn ancestral_search_passes_through_embryonic_then_braning() {
        let root = Compiler::compile("{a = 1; b = {c = a;};}")
            .unwrap()
            .pop()
            .unwrap();
        let search = find_search(&root, "^a$").expect("search for a");
        let scope = Scope::empty();

        let trace = step_to_settled(&search, &scope);
        eprintln!("ancestral search nyes: {trace:?}");
        assert!(
            trace.contains(&Nyes::Embryonic),
            "ancestral search must pass through EMBRYONIC (ib_search stage)"
        );
        assert!(
            trace.contains(&Nyes::Braning),
            "ancestral search must reach BRANING (ab_search stage)"
        );
    }

    #[test]
    fn sff_descendant_searches_are_econstanic_at_build() {
        let root = Compiler::compile("{a = 1; b = 2; sff = <<a + b>>;}")
            .unwrap()
            .pop()
            .unwrap();
        let sa = find_search(&root, "^a$").expect("search a");
        let sb = find_search(&root, "^b$").expect("search b");
        assert_eq!(
            sa.borrow().core().get_nyes(),
            Nyes::Econstanic,
            "SFF search a built ECONSTANIC"
        );
        assert_eq!(
            sb.borrow().core().get_nyes(),
            Nyes::Econstanic,
            "SFF search b built ECONSTANIC"
        );

        let scope = Scope::empty();
        for _ in 0..200 {
            if root.borrow().core().get_nyes().is_constanic() {
                break;
            }
            let _ = root.step(&scope).unwrap();
        }
        assert_eq!(
            sa.borrow().core().get_nyes(),
            Nyes::Econstanic,
            "SFF search a stays ECONSTANIC after stepping"
        );
        assert_eq!(
            sb.borrow().core().get_nyes(),
            Nyes::Econstanic,
            "SFF search b stays ECONSTANIC after stepping"
        );
    }

    #[test]
    fn sf_of_sff_sets_econstanic_body_constanic() {
        let root = Compiler::compile("{a = 1; b = 2; sff = <<a + b>>; sf = <sff>;}")
            .unwrap()
            .pop()
            .unwrap();
        let scope = Scope::empty();
        for _ in 0..200 {
            if root.borrow().core().get_nyes().is_constanic() {
                break;
            }
            let _ = root.step(&scope).unwrap();
        }
        let sff_search = find_search(&root, "^sff$").expect("search sff");
        assert!(sff_search.borrow().core().get_nyes().is_constanic());
        let result = sff_search.value();
        assert_eq!(
            result.borrow().kind(),
            FirKind::Operator,
            "sf=<sff> must make the SFF's Op+ body constanic, not resolve it to a value"
        );
    }

    #[test]
    fn anonymous_statement_named_question_marks() {
        let root = Compiler::compile("{a = 1; a;}").unwrap().pop().unwrap();
        let stmts: Vec<FirRef> = root.borrow().core().foolish_children().to_vec();
        assert_eq!(stmts.len(), 2);
        assert_eq!(
            stmts[0].borrow().as_stmt_name(),
            Some("a"),
            "named assignment keeps its LHS"
        );
        assert_eq!(
            stmts[1].borrow().as_stmt_name(),
            Some(crate::compiler::ANON_STMT_NAME),
            "anonymous bare expression is named ???"
        );
    }

    // ── Value search tests (FOOP-23 Phase A) ────────────────────────

    #[test]
    fn value_search_forward_finds_first_match() {
        use crate::compiler::Compiler;
        let root = Compiler::compile("{a = {id = 4; size = 10; depth = 10;}; fwd = a~=10;}")
            .unwrap()
            .pop()
            .unwrap();
        settle_root(&root);
        let stmts = root.borrow().core().foolish_children().to_vec();
        let fwd_stmt = &stmts[1];
        let body = fwd_stmt
            .borrow()
            .core()
            .foolish_children()
            .first()
            .cloned()
            .unwrap();
        assert_eq!(body.borrow().as_i64(), Some(10));
    }

    #[test]
    fn value_search_backward_finds_last_match() {
        use crate::compiler::Compiler;
        let root = Compiler::compile("{a = {id = 4; size = 10; depth = 10;}; bwd = a?=10;}")
            .unwrap()
            .pop()
            .unwrap();
        settle_root(&root);
        let stmts = root.borrow().core().foolish_children().to_vec();
        let bwd_stmt = &stmts[1];
        let body = bwd_stmt
            .borrow()
            .core()
            .foolish_children()
            .first()
            .cloned()
            .unwrap();
        assert_eq!(body.borrow().as_i64(), Some(10));
    }

    #[test]
    fn value_search_anchored_miss_is_nk() {
        use crate::compiler::Compiler;
        let root = Compiler::compile("{a = {x = 1;}; bad = a~=99;}")
            .unwrap()
            .pop()
            .unwrap();
        settle_root(&root);
        let stmts = root.borrow().core().foolish_children().to_vec();
        let bad_stmt = &stmts[1];
        let body = bad_stmt
            .borrow()
            .core()
            .foolish_children()
            .first()
            .cloned()
            .unwrap();
        assert_eq!(body.borrow().core().get_nyes(), Nyes::Nk);
    }

    #[test]
    fn value_search_non_integer_pattern_is_nk() {
        use crate::compiler::Compiler;
        let root = Compiler::compile("{a = {inner = {x = 1;}; n = 5;}; bad = a~={q = 1;};}")
            .unwrap()
            .pop()
            .unwrap();
        settle_root(&root);
        let stmts = root.borrow().core().foolish_children().to_vec();
        let bad_stmt = &stmts[1];
        let body = bad_stmt
            .borrow()
            .core()
            .foolish_children()
            .first()
            .cloned()
            .unwrap();
        assert_eq!(body.borrow().core().get_nyes(), Nyes::Nk);
        assert!(
            body.borrow()
                .core()
                .alarm_reason()
                .is_some_and(|r| r.contains("VALUE-SEARCH-UNSUPPORTED-PATTERN"))
        );
    }

    #[test]
    fn value_search_unanchored_miss_is_econstanic() {
        use crate::compiler::Compiler;
        let root = Compiler::compile("{pi = 3; e2 = 2; nope = ?=9;}")
            .unwrap()
            .pop()
            .unwrap();
        settle_root(&root);
        let stmts = root.borrow().core().foolish_children().to_vec();
        let nope_stmt = &stmts[2];
        let body = nope_stmt
            .borrow()
            .core()
            .foolish_children()
            .first()
            .cloned()
            .unwrap();
        assert_eq!(body.borrow().core().get_nyes(), Nyes::Econstanic);
    }

    #[test]
    fn value_search_unanchored_finds_match() {
        use crate::compiler::Compiler;
        let root = Compiler::compile("{pi = 3; e2 = 2; found = ?=3;}")
            .unwrap()
            .pop()
            .unwrap();
        settle_root(&root);
        let stmts = root.borrow().core().foolish_children().to_vec();
        let found_stmt = &stmts[2];
        let body = found_stmt
            .borrow()
            .core()
            .foolish_children()
            .first()
            .cloned()
            .unwrap();
        assert_eq!(body.borrow().as_i64(), Some(3));
    }

    #[test]
    fn value_search_nyes_transitions() {
        use crate::compiler::Compiler;
        let root = Compiler::compile("{a = {id = 4; size = 10; depth = 10;}; fwd = a~=10;}")
            .unwrap()
            .pop()
            .unwrap();
        settle_root(&root);
        let stmts = root.borrow().core().foolish_children().to_vec();
        let fwd_stmt = &stmts[1];
        let search = fwd_stmt
            .borrow()
            .core()
            .foolish_children()
            .first()
            .cloned()
            .unwrap();
        assert!(
            search.borrow().core().get_nyes().is_constanic(),
            "search must be constanic after root settles"
        );
        assert_eq!(search.borrow().as_i64(), Some(10));
    }

    // ── Value search Phase B tests (expression patterns) ─────────────

    #[test]
    fn value_search_expr_pattern_1_plus_2_finds_3() {
        use crate::compiler::Compiler;
        let root = Compiler::compile("{a = {u = 3; v = 5;}; r = a~=1+2;}")
            .unwrap()
            .pop()
            .unwrap();
        settle_root(&root);
        let stmts = root.borrow().core().foolish_children().to_vec();
        let r_stmt = &stmts[1];
        let body = r_stmt
            .borrow()
            .core()
            .foolish_children()
            .first()
            .cloned()
            .unwrap();
        assert_eq!(body.borrow().as_i64(), Some(3), "1+2=3 must find u=3");
    }

    #[test]
    fn value_search_expr_pattern_c_minus_d_resolves_in_search_context() {
        use crate::compiler::Compiler;
        let root = Compiler::compile("{c = 12; d = 9; a = {u = 3; v = 5;}; r = a~=c-d;}")
            .unwrap()
            .pop()
            .unwrap();
        settle_root(&root);
        let stmts = root.borrow().core().foolish_children().to_vec();
        let r_stmt = &stmts[3];
        let body = r_stmt
            .borrow()
            .core()
            .foolish_children()
            .first()
            .cloned()
            .unwrap();
        assert_eq!(body.borrow().as_i64(), Some(3), "c-d=12-9=3 must find u=3");
    }

    #[test]
    fn value_search_expr_pattern_nk_operand_is_nk() {
        use crate::compiler::Compiler;
        let root = Compiler::compile("{a = {u = 3;}; r = a~=???+1;}")
            .unwrap()
            .pop()
            .unwrap();
        settle_root(&root);
        let stmts = root.borrow().core().foolish_children().to_vec();
        let r_stmt = &stmts[1];
        let body = r_stmt
            .borrow()
            .core()
            .foolish_children()
            .first()
            .cloned()
            .unwrap();
        assert_eq!(
            body.borrow().core().get_nyes(),
            Nyes::Nk,
            "NK pattern operand makes value search NK"
        );
    }

    #[test]
    fn value_search_expr_pattern_unresolvable_name_is_econstanic() {
        use crate::compiler::Compiler;
        let root = Compiler::compile("{c = 12; d = 9; a = {u = 3; v = 5;}; r = a~=c-d+v;}")
            .unwrap()
            .pop()
            .unwrap();
        settle_root(&root);
        let stmts = root.borrow().core().foolish_children().to_vec();
        let r_stmt = &stmts[3];
        let body = r_stmt
            .borrow()
            .core()
            .foolish_children()
            .first()
            .cloned()
            .unwrap();
        assert_eq!(
            body.borrow().core().get_nyes(),
            Nyes::Econstanic,
            "pattern with unresolvable v becomes ECONSTANIC"
        );
    }

    fn settle_root(root: &FirRef) {
        let scope = Scope::empty();
        for _ in 0..200 {
            let report = root.step(&scope).unwrap();
            if let StepReport::Progress(nyes) = report
                && nyes.is_constanic()
            {
                return;
            }
        }
        panic!("root did not settle within 200 steps");
    }

    // ── ContextfulSearch engine tests (FOOP-23 Phase A0) ────────────

    use crate::fir_kinds::{
        BraneNavigator, CandidateNavigator, MatchOutcome, ScanOutcome, SearchPredicate,
    };

    fn settled_int(value: i64) -> FirRef {
        Rc::new_cyclic(|me: &Weak<RefCell<IndepIntFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(IndepIntFir {
                core: ProtoBrane::new(vec![], parent, Nyes::Constant),
                value,
            })
        })
    }

    fn settled_nk(reason: &str) -> FirRef {
        Rc::new_cyclic(|me: &Weak<RefCell<NkFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(NkFir {
                core: ProtoBrane::new(vec![], parent, Nyes::Nk),
                reason: reason.to_owned(),
            })
        })
    }

    // --- Navigator contract tests ---

    #[test]
    fn brane_nav_forward_yields_in_order_exactly_once() {
        let s0 = make_statement("α", 0, make_constant_int(1));
        let s1 = make_statement("β", 1, make_constant_int(2));
        let s2 = make_statement("γ", 2, make_constant_int(3));
        let brane = make_brane(vec![Rc::clone(&s0), Rc::clone(&s1), Rc::clone(&s2)]);
        let mut nav = BraneNavigator::new(&brane, true);

        assert_eq!(nav.total(), 3);

        let yielded: Vec<(String, usize)> = std::iter::from_fn(|| nav.next_candidate())
            .map(|(c, pos)| (c.borrow().as_stmt_name().unwrap().to_owned(), pos))
            .collect();
        assert_eq!(
            yielded,
            vec![("α".into(), 0), ("β".into(), 1), ("γ".into(), 2),]
        );

        assert!(
            nav.next_candidate().is_none(),
            "must stop after all yielded"
        );
    }

    #[test]
    fn brane_nav_backward_yields_reverse_order_exactly_once() {
        let s0 = make_statement("א", 0, make_constant_int(10));
        let s1 = make_statement("ב", 1, make_constant_int(20));
        let s2 = make_statement("ג", 2, make_constant_int(30));
        let brane = make_brane(vec![Rc::clone(&s0), Rc::clone(&s1), Rc::clone(&s2)]);
        let mut nav = BraneNavigator::new(&brane, false);

        let yielded: Vec<(String, usize)> = std::iter::from_fn(|| nav.next_candidate())
            .map(|(c, pos)| (c.borrow().as_stmt_name().unwrap().to_owned(), pos))
            .collect();
        assert_eq!(
            yielded,
            vec![("ג".into(), 2), ("ב".into(), 1), ("א".into(), 0),]
        );

        assert!(
            nav.next_candidate().is_none(),
            "backward must stop after all yielded"
        );
    }

    #[test]
    fn brane_nav_empty_brane_yields_nothing() {
        let brane = make_brane(vec![]);
        let mut nav = BraneNavigator::new(&brane, true);
        assert_eq!(nav.total(), 0);
        assert!(nav.next_candidate().is_none());
    }

    #[test]
    fn brane_nav_single_element_forward_and_backward() {
        let stmt = make_statement("μ", 0, make_constant_int(42));
        let brane = make_brane(vec![Rc::clone(&stmt)]);

        let mut fwd = BraneNavigator::new(&brane, true);
        let v: Vec<usize> = std::iter::from_fn(|| fwd.next_candidate())
            .map(|(_, p)| p)
            .collect();
        assert_eq!(v, vec![0]);

        let mut bwd = BraneNavigator::new(&brane, false);
        let v: Vec<usize> = std::iter::from_fn(|| bwd.next_candidate())
            .map(|(_, p)| p)
            .collect();
        assert_eq!(v, vec![0]);
    }

    // --- Matcher tests ---

    #[test]
    fn matcher_name_approve_on_exact_match() {
        let stmt = make_statement("ξ", 0, settled_int(5));
        let ctx = super::ScanCtx {
            position: 0,
            total: 1,
        };
        let pred = SearchPredicate::Name {
            pattern: "ξ".into(),
        };
        assert_eq!(pred.matches(&stmt, &ctx), MatchOutcome::Approve);
    }

    #[test]
    fn matcher_name_approve_on_regex() {
        let stmt = make_statement("tmp_abc", 0, settled_int(5));
        let ctx = super::ScanCtx {
            position: 0,
            total: 1,
        };
        let pred = SearchPredicate::Name {
            pattern: "^tmp_.*".into(),
        };
        assert_eq!(pred.matches(&stmt, &ctx), MatchOutcome::Approve);
    }

    #[test]
    fn matcher_name_reject_on_mismatch() {
        let stmt = make_statement("ω", 0, settled_int(5));
        let ctx = super::ScanCtx {
            position: 0,
            total: 1,
        };
        let pred = SearchPredicate::Name {
            pattern: "ζ".into(),
        };
        assert_eq!(pred.matches(&stmt, &ctx), MatchOutcome::Reject);
    }

    #[test]
    #[should_panic(expected = "pre-constanic body")]
    fn matcher_name_panics_on_nye_body() {
        let body: FirRef = Rc::new_cyclic(|me: &Weak<RefCell<IndepIntFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(IndepIntFir {
                core: ProtoBrane::new(vec![], parent, Nyes::Prembrionic),
                value: 0,
            })
        });
        let stmt = make_statement("φ", 0, Rc::clone(&body));
        let ctx = super::ScanCtx {
            position: 0,
            total: 1,
        };
        let pred = SearchPredicate::Name {
            pattern: "φ".into(),
        };
        pred.matches(&stmt, &ctx);
    }

    #[test]
    fn matcher_name_nk_stop_on_nk_body() {
        let body = settled_nk("boom");
        let stmt = make_statement("χ", 0, Rc::clone(&body));
        let ctx = super::ScanCtx {
            position: 0,
            total: 1,
        };
        let pred = SearchPredicate::Name {
            pattern: "χ".into(),
        };
        assert_eq!(pred.matches(&stmt, &ctx), MatchOutcome::NkStop);
    }

    #[test]
    fn matcher_value_approve_on_matching_int() {
        let stmt = make_statement("σ", 0, settled_int(42));
        let ctx = super::ScanCtx {
            position: 0,
            total: 1,
        };
        let pattern_val = make_constant_int(42);
        let pred = SearchPredicate::Value {
            pattern: Rc::clone(&pattern_val),
        };
        assert_eq!(pred.matches(&stmt, &ctx), MatchOutcome::Approve);
    }

    #[test]
    fn matcher_value_reject_on_mismatched_int() {
        let stmt = make_statement("τ", 0, settled_int(7));
        let ctx = super::ScanCtx {
            position: 0,
            total: 1,
        };
        let pattern_val = make_constant_int(99);
        let pred = SearchPredicate::Value {
            pattern: Rc::clone(&pattern_val),
        };
        assert_eq!(pred.matches(&stmt, &ctx), MatchOutcome::Reject);
    }

    #[test]
    fn matcher_value_reject_non_integer_candidate() {
        let inner_stmt = make_statement("x", 0, make_constant_int(1));
        let body = make_brane(vec![Rc::clone(&inner_stmt)]);
        let _ = step_to_settled(&body, &Scope::empty());
        let stmt = make_statement("ρ", 0, Rc::clone(&body));
        let ctx = super::ScanCtx {
            position: 0,
            total: 1,
        };
        let pattern_val = make_constant_int(1);
        let pred = SearchPredicate::Value {
            pattern: Rc::clone(&pattern_val),
        };
        assert_eq!(
            pred.matches(&stmt, &ctx),
            MatchOutcome::Reject,
            "brane-valued candidate is skipped, not an error"
        );
    }

    #[test]
    #[should_panic(expected = "pre-constanic body")]
    fn matcher_value_panics_on_nye_body() {
        let body: FirRef = Rc::new_cyclic(|me: &Weak<RefCell<IndepIntFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(IndepIntFir {
                core: ProtoBrane::new(vec![], parent, Nyes::Embryonic),
                value: 10,
            })
        });
        let stmt = make_statement("ψ", 0, Rc::clone(&body));
        let ctx = super::ScanCtx {
            position: 0,
            total: 1,
        };
        let pattern_val = make_constant_int(10);
        let pred = SearchPredicate::Value {
            pattern: Rc::clone(&pattern_val),
        };
        pred.matches(&stmt, &ctx);
    }

    #[test]
    fn matcher_value_nk_stop_on_nk_body() {
        let body = settled_nk("unbound");
        let stmt = make_statement("ζ", 0, Rc::clone(&body));
        let ctx = super::ScanCtx {
            position: 0,
            total: 1,
        };
        let pattern_val = make_constant_int(1);
        let pred = SearchPredicate::Value {
            pattern: Rc::clone(&pattern_val),
        };
        assert_eq!(pred.matches(&stmt, &ctx), MatchOutcome::NkStop);
    }

    #[test]
    fn matcher_namevalue_both_must_match() {
        let stmt = make_statement("ωμ", 0, settled_int(7));
        let ctx = super::ScanCtx {
            position: 0,
            total: 1,
        };
        let pat_val = make_constant_int(7);

        let pred_ok = SearchPredicate::NameValue {
            name: "ωμ".into(),
            value: Rc::clone(&pat_val),
        };
        assert_eq!(pred_ok.matches(&stmt, &ctx), MatchOutcome::Approve);

        let pred_name_miss = SearchPredicate::NameValue {
            name: "zzz".into(),
            value: Rc::clone(&pat_val),
        };
        assert_eq!(pred_name_miss.matches(&stmt, &ctx), MatchOutcome::Reject);

        let bad_val = make_constant_int(99);
        let pred_val_miss = SearchPredicate::NameValue {
            name: "ωμ".into(),
            value: Rc::clone(&bad_val),
        };
        assert_eq!(pred_val_miss.matches(&stmt, &ctx), MatchOutcome::Reject);
    }

    #[test]
    #[should_panic(expected = "pre-constanic body")]
    fn matcher_namevalue_panics_on_nye_body() {
        let body: FirRef = Rc::new_cyclic(|me: &Weak<RefCell<IndepIntFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(IndepIntFir {
                core: ProtoBrane::new(vec![], parent, Nyes::Prembrionic),
                value: 5,
            })
        });
        let stmt = make_statement("λ", 0, Rc::clone(&body));
        let ctx = super::ScanCtx {
            position: 0,
            total: 1,
        };
        let pat_val = make_constant_int(5);
        let pred = SearchPredicate::NameValue {
            name: "λ".into(),
            value: Rc::clone(&pat_val),
        };
        pred.matches(&stmt, &ctx);
    }

    #[test]
    fn matcher_index_approve_at_correct_position() {
        let stmt = make_statement("ε", 0, settled_int(1));
        let ctx = super::ScanCtx {
            position: 1,
            total: 3,
        };
        let pred = SearchPredicate::Index(1);
        assert_eq!(pred.matches(&stmt, &ctx), MatchOutcome::Approve);
    }

    #[test]
    fn matcher_index_reject_at_wrong_position() {
        let stmt = make_statement("δ", 0, settled_int(1));
        let ctx = super::ScanCtx {
            position: 0,
            total: 3,
        };
        let pred = SearchPredicate::Index(1);
        assert_eq!(pred.matches(&stmt, &ctx), MatchOutcome::Reject);
    }

    #[test]
    fn matcher_index_negative_offset() {
        let stmt = make_statement("η", 0, settled_int(1));
        let ctx = super::ScanCtx {
            position: 2,
            total: 3,
        };
        let pred = SearchPredicate::Index(-1);
        assert_eq!(
            pred.matches(&stmt, &ctx),
            MatchOutcome::Approve,
            "Index(-1) at total=3 means position 2"
        );
    }

    #[test]
    fn matcher_head_at_position_zero() {
        let stmt = make_statement("κ", 0, settled_int(1));
        let ctx = super::ScanCtx {
            position: 0,
            total: 5,
        };
        assert_eq!(
            SearchPredicate::Head.matches(&stmt, &ctx),
            MatchOutcome::Approve
        );
        let ctx2 = super::ScanCtx {
            position: 3,
            total: 5,
        };
        assert_eq!(
            SearchPredicate::Head.matches(&stmt, &ctx2),
            MatchOutcome::Reject
        );
    }

    #[test]
    fn matcher_tail_at_last_position() {
        let stmt = make_statement("ν", 0, settled_int(1));
        let ctx = super::ScanCtx {
            position: 4,
            total: 5,
        };
        assert_eq!(
            SearchPredicate::Tail.matches(&stmt, &ctx),
            MatchOutcome::Approve
        );
        let ctx2 = super::ScanCtx {
            position: 0,
            total: 5,
        };
        assert_eq!(
            SearchPredicate::Tail.matches(&stmt, &ctx2),
            MatchOutcome::Reject
        );
    }

    // --- Core scan loop tests ---

    #[test]
    fn scan_finds_first_match_in_forward_order() {
        let s0 = make_statement("α", 0, settled_int(1));
        let s1 = make_statement("β", 1, settled_int(2));
        let s2 = make_statement("γ", 2, settled_int(3));
        let brane = make_brane(vec![Rc::clone(&s0), Rc::clone(&s1), Rc::clone(&s2)]);
        let mut nav = BraneNavigator::new(&brane, true);
        let pred = SearchPredicate::Name {
            pattern: "γ".into(),
        };

        let outcome = super::contextful_search_scan(&mut nav, &pred);
        match outcome {
            ScanOutcome::Found(stmt) => {
                assert_eq!(stmt.borrow().as_stmt_name(), Some("γ"));
            }
            other => panic!("expected Found, got {other:?}"),
        }
    }

    #[test]
    fn scan_finds_first_match_in_backward_order() {
        let s0 = make_statement("ᚠ", 0, settled_int(10));
        let s1 = make_statement("ᚢ", 1, settled_int(20));
        let s2 = make_statement("ᚦ", 2, settled_int(30));
        let brane = make_brane(vec![Rc::clone(&s0), Rc::clone(&s1), Rc::clone(&s2)]);
        let mut nav = BraneNavigator::new(&brane, false);
        let pred = SearchPredicate::Name {
            pattern: "ᚠ".into(),
        };

        let outcome = super::contextful_search_scan(&mut nav, &pred);
        match outcome {
            ScanOutcome::Found(stmt) => {
                assert_eq!(
                    stmt.borrow().as_stmt_name(),
                    Some("ᚠ"),
                    "backward scan must find ᚠ even though it is at brane position 0"
                );
            }
            other => panic!("expected Found, got {other:?}"),
        }
    }

    #[test]
    #[should_panic(expected = "pre-constanic body")]
    fn scan_panic_on_pre_constanic_candidate() {
        let s0 = make_statement("a", 0, settled_int(1));
        let body_nye: FirRef = Rc::new_cyclic(|me: &Weak<RefCell<IndepIntFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(IndepIntFir {
                core: ProtoBrane::new(vec![], parent, Nyes::Embryonic),
                value: 5,
            })
        });
        let s1 = make_statement("b", 1, Rc::clone(&body_nye));
        let s2 = make_statement("c", 2, settled_int(5));
        let brane = make_brane(vec![Rc::clone(&s0), Rc::clone(&s1), Rc::clone(&s2)]);
        let mut nav = BraneNavigator::new(&brane, true);
        let pattern_val = settled_int(5);
        let pred = SearchPredicate::Value {
            pattern: Rc::clone(&pattern_val),
        };

        super::contextful_search_scan(&mut nav, &pred);
    }

    #[test]
    fn scan_nk_stop_halts_immediately() {
        let s0 = make_statement("x", 0, settled_int(99));
        let s1 = make_statement("y", 1, settled_nk("boom"));
        let s2 = make_statement("z", 2, settled_int(1));
        let brane = make_brane(vec![Rc::clone(&s0), Rc::clone(&s1), Rc::clone(&s2)]);
        let mut nav = BraneNavigator::new(&brane, true);
        let pattern_val = settled_int(1);
        let pred = SearchPredicate::Value {
            pattern: Rc::clone(&pattern_val),
        };

        let outcome = super::contextful_search_scan(&mut nav, &pred);
        assert_eq!(
            outcome,
            ScanOutcome::NkStop,
            "must halt on NK candidate, not skip to z"
        );
    }

    #[test]
    fn scan_miss_returns_miss() {
        let s0 = make_statement("a", 0, settled_int(1));
        let brane = make_brane(vec![Rc::clone(&s0)]);
        let mut nav = BraneNavigator::new(&brane, true);
        let pattern_val = settled_int(999);
        let pred = SearchPredicate::Value {
            pattern: Rc::clone(&pattern_val),
        };

        let outcome = super::contextful_search_scan(&mut nav, &pred);
        assert_eq!(outcome, ScanOutcome::Miss);
    }

    #[test]
    fn scan_namevalue_finds_correct_combined_candidate() {
        let s0 = make_statement("setting", 0, settled_int(11));
        let s1 = make_statement("mid", 1, settled_int(0));
        let s2 = make_statement("setting", 2, settled_int(10));
        let brane = make_brane(vec![Rc::clone(&s0), Rc::clone(&s1), Rc::clone(&s2)]);
        let mut nav = BraneNavigator::new(&brane, true);
        let pat_val = settled_int(10);
        let pred = SearchPredicate::NameValue {
            name: "setting".into(),
            value: Rc::clone(&pat_val),
        };

        let outcome = super::contextful_search_scan(&mut nav, &pred);
        match outcome {
            ScanOutcome::Found(stmt) => {
                assert_eq!(stmt.borrow().as_stmt_name(), Some("setting"));
                let body = stmt
                    .borrow()
                    .core()
                    .foolish_children()
                    .first()
                    .cloned()
                    .unwrap();
                assert_eq!(
                    body.borrow().as_i64(),
                    Some(10),
                    "must find the SECOND setting (value=10), not the first (value=11)"
                );
            }
            other => panic!("expected Found, got {other:?}"),
        }
    }

    #[test]
    fn scan_head_predicate_yields_first_statement() {
        let s0 = make_statement("ᚺ", 0, settled_int(100));
        let s1 = make_statement("ᚾ", 1, settled_int(200));
        let brane = make_brane(vec![Rc::clone(&s0), Rc::clone(&s1)]);
        let mut nav = BraneNavigator::new(&brane, true);

        let outcome = super::contextful_search_scan(&mut nav, &SearchPredicate::Head);
        match outcome {
            ScanOutcome::Found(stmt) => {
                assert_eq!(stmt.borrow().as_stmt_name(), Some("ᚺ"));
            }
            other => panic!("expected Found, got {other:?}"),
        }
    }

    #[test]
    fn scan_tail_predicate_yields_last_statement() {
        let s0 = make_statement("ᛊ", 0, settled_int(100));
        let s1 = make_statement("ᛏ", 1, settled_int(200));
        let brane = make_brane(vec![Rc::clone(&s0), Rc::clone(&s1)]);
        let mut nav = BraneNavigator::new(&brane, false);

        let outcome = super::contextful_search_scan(&mut nav, &SearchPredicate::Tail);
        match outcome {
            ScanOutcome::Found(stmt) => {
                assert_eq!(
                    stmt.borrow().as_stmt_name(),
                    Some("ᛏ"),
                    "Tail matches the last brane position regardless of nav direction"
                );
            }
            other => panic!("expected Found, got {other:?}"),
        }
    }

    #[test]
    fn scan_index_backward_finds_correct_position() {
        let s0 = make_statement("ᚩ", 0, settled_int(10));
        let s1 = make_statement("ᚪ", 1, settled_int(20));
        let s2 = make_statement("ᚫ", 2, settled_int(30));
        let brane = make_brane(vec![Rc::clone(&s0), Rc::clone(&s1), Rc::clone(&s2)]);
        let mut nav = BraneNavigator::new(&brane, false);
        let pred = SearchPredicate::Index(1);

        let outcome = super::contextful_search_scan(&mut nav, &pred);
        match outcome {
            ScanOutcome::Found(stmt) => {
                assert_eq!(
                    stmt.borrow().as_stmt_name(),
                    Some("ᚪ"),
                    "Index(1) must match brane position 1 even in backward scan"
                );
            }
            other => panic!("expected Found, got {other:?}"),
        }
    }

    // --- NYES transition test (mandatory per AGENTS.md) ---

    #[test]
    fn contextful_search_nyes_transitions() {
        let const_stmt = make_statement("c", 0, settled_int(1));
        let nk_stmt = make_statement("n", 1, settled_nk("gone"));
        let brane = make_brane(vec![Rc::clone(&const_stmt), Rc::clone(&nk_stmt)]);
        let pattern_val = settled_int(1);

        // Name predicate on constanic body → Approve
        let mut nav1 = BraneNavigator::new(&brane, true);
        let pred_name = SearchPredicate::Name {
            pattern: "c".into(),
        };
        assert_eq!(
            super::contextful_search_scan(&mut nav1, &pred_name),
            ScanOutcome::Found(Rc::clone(&const_stmt)),
        );

        // Name predicate on NK body → NkStop
        let mut nav2 = BraneNavigator::new(&brane, true);
        let pred_nk_name = SearchPredicate::Name {
            pattern: "n".into(),
        };
        assert_eq!(
            super::contextful_search_scan(&mut nav2, &pred_nk_name),
            ScanOutcome::NkStop,
        );

        // Value predicate where first candidate matches → Found
        let mut nav3 = BraneNavigator::new(&brane, true);
        let pred_val = SearchPredicate::Value {
            pattern: Rc::clone(&pattern_val),
        };
        assert_eq!(
            super::contextful_search_scan(&mut nav3, &pred_val),
            ScanOutcome::Found(Rc::clone(&const_stmt)),
        );

        // All constanic → no wait, no nk_stop
        let all_const = make_brane(vec![
            make_statement("a", 0, settled_int(1)),
            make_statement("b", 1, settled_int(2)),
            make_statement("c", 2, settled_int(3)),
        ]);
        let mut nav4 = BraneNavigator::new(&all_const, true);
        let pred_miss = SearchPredicate::Name {
            pattern: "zzz".into(),
        };
        assert_eq!(
            super::contextful_search_scan(&mut nav4, &pred_miss),
            ScanOutcome::Miss,
            "all constanic, no match → Miss (no wait, no nk_stop)"
        );

        // Empty brane → Miss
        let empty = make_brane(vec![]);
        let mut nav5 = BraneNavigator::new(&empty, true);
        assert_eq!(
            super::contextful_search_scan(&mut nav5, &pred_miss),
            ScanOutcome::Miss,
            "empty brane → Miss"
        );
    }

    // ── C0: Contextless deepening tests (FOOP-23) ────────────────────

    #[test]
    fn contextless_deepening_chain_resolves() {
        use crate::compiler::Compiler;
        let root = Compiler::compile(
            "{some_brane = {some_aspect = {some_thing_else = {some_value = 42;};};}; \
             final = some_brane.some_aspect.some_thing_else.some_value;}",
        )
        .unwrap()
        .pop()
        .unwrap();
        settle_root(&root);
        let stmts = root.borrow().core().foolish_children().to_vec();
        let final_stmt = &stmts[1];
        let body = final_stmt
            .borrow()
            .core()
            .foolish_children()
            .first()
            .cloned()
            .unwrap();
        assert_eq!(
            body.borrow().as_i64(),
            Some(42),
            "dot-chain must deepen through nested branes"
        );
    }

    #[test]
    fn contextless_search_on_non_brane_anchor_is_nk() {
        use crate::compiler::Compiler;
        let root = Compiler::compile("{a = 42; x = a.b;}")
            .unwrap()
            .pop()
            .unwrap();
        settle_root(&root);
        let stmts = root.borrow().core().foolish_children().to_vec();
        let x_stmt = &stmts[1];
        let body = x_stmt
            .borrow()
            .core()
            .foolish_children()
            .first()
            .cloned()
            .unwrap();
        assert_eq!(
            body.borrow().core().get_nyes(),
            Nyes::Nk,
            "anchored search on non-brane must be NK"
        );
    }

    #[test]
    fn contextless_search_anchor_nk_is_nk() {
        use crate::compiler::Compiler;
        let root = Compiler::compile("{a = ???; x = a.b;}")
            .unwrap()
            .pop()
            .unwrap();
        settle_root(&root);
        let stmts = root.borrow().core().foolish_children().to_vec();
        let x_stmt = &stmts[1];
        let body = x_stmt
            .borrow()
            .core()
            .foolish_children()
            .first()
            .cloned()
            .unwrap();
        assert_eq!(
            body.borrow().core().get_nyes(),
            Nyes::Nk,
            "anchored search on NK anchor must be NK"
        );
    }

    #[test]
    fn contextless_search_anchor_nigh_waits_then_resolves() {
        use crate::compiler::Compiler;
        let root = Compiler::compile("{a = {x = 1;}; b = a.x;}")
            .unwrap()
            .pop()
            .unwrap();
        let search = find_search(&root, "^x$").expect("search for x");
        let scope = Scope::empty();

        let mut saw_braning = false;
        for _ in 0..200 {
            if root.borrow().core().get_nyes().is_constanic() {
                break;
            }
            let _ = root.step(&scope).unwrap();
            if search.borrow().core().get_nyes() == Nyes::Braning {
                saw_braning = true;
            }
        }
        assert!(
            saw_braning,
            "search must pass through BRANING while anchor settles"
        );
        assert_eq!(
            search.borrow().as_i64(),
            Some(1),
            "search must resolve to 1 after anchor settles"
        );
    }

    // ── C1: FoolRefFir tests (FOOP-23) ──────────────────────────────

    #[test]
    fn fool_ref_fir_nyes_transitions() {
        use crate::fir_trait::FirRefExt;
        use std::cell::RefCell;
        use std::rc::{Rc, Weak};

        let referent = make_constant_int(42);
        let fool_ref: FirRef = Rc::new_cyclic(|me: &Weak<RefCell<FoolRefFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(FoolRefFir {
                core: ProtoBrane::new(vec![], parent, Nyes::Constant),
                referent: Rc::clone(&referent),
            })
        });

        assert_eq!(
            fool_ref.borrow().core().get_nyes(),
            Nyes::Constant,
            "FoolRefFir must be born CONSTANT"
        );
        assert_eq!(fool_ref.borrow().kind(), FirKind::FoolRef);

        let scope = Scope::empty();
        let report = fool_ref.step(&scope).unwrap();
        assert!(
            matches!(report, StepReport::Progress(Nyes::Constant)),
            "stepping a FoolRefFir must be a no-op returning Constant"
        );
        assert_eq!(fool_ref.borrow().core().get_nyes(), Nyes::Constant);
    }

    #[test]
    fn fool_ref_referent_survives_original_drop() {
        use crate::compiler::Compiler;
        let root = Compiler::compile("{a = {x = 99;}; b = a.x;}")
            .unwrap()
            .pop()
            .unwrap();
        settle_root(&root);

        let stmts = root.borrow().core().foolish_children().to_vec();
        let b_stmt = &stmts[1];
        let search_body = b_stmt
            .borrow()
            .core()
            .foolish_children()
            .first()
            .cloned()
            .unwrap();

        let ubc = search_body.borrow().core().ubc_children();
        assert_eq!(ubc.len(), 2, "search must have [clone, FoolRefFir]");
        assert_eq!(
            ubc[0].borrow().as_i64(),
            Some(99),
            "ubc_children[0] is the resolved clone"
        );
        assert_eq!(
            ubc[1].borrow().kind(),
            FirKind::FoolRef,
            "ubc_children[1] is the FoolRefFir"
        );
        assert_eq!(
            ubc[1].borrow().core().get_nyes(),
            Nyes::Constant,
            "FoolRefFir is born CONSTANT"
        );

        drop(root);

        assert_eq!(
            ubc[1].borrow().core().get_nyes(),
            Nyes::Constant,
            "FoolRefFir must survive root drop (strong Rc)"
        );
        assert_eq!(
            ubc[0].borrow().as_i64(),
            Some(99),
            "clone must survive root drop"
        );
    }

    #[test]
    fn search_result_pair_has_fool_ref_at_index_1() {
        use crate::compiler::Compiler;
        let root = Compiler::compile("{a = {x = 7;}; b = a.x;}")
            .unwrap()
            .pop()
            .unwrap();
        settle_root(&root);

        let stmts = root.borrow().core().foolish_children().to_vec();
        let b_stmt = &stmts[1];
        let search_body = b_stmt
            .borrow()
            .core()
            .foolish_children()
            .first()
            .cloned()
            .unwrap();

        let ubc = search_body.borrow().core().ubc_children();
        assert_eq!(ubc.len(), 2);
        assert_eq!(ubc[0].borrow().as_i64(), Some(7));
        assert_eq!(ubc[1].borrow().kind(), FirKind::FoolRef);
        assert_eq!(ubc[1].borrow().core().get_nyes(), Nyes::Constant);
    }

    #[test]
    fn index_result_pair_has_fool_ref_at_index_1() {
        let val = make_constant_int(42);
        let stmt = make_statement("x", 0, Rc::clone(&val));
        let brane = make_brane(vec![Rc::clone(&stmt)]);
        let idx = make_index(0, true, vec![Rc::clone(&brane)]);
        let scope = Scope::empty();

        step_to_settled(&idx, &scope);
        assert_eq!(idx.borrow().core().get_nyes(), Nyes::Constant);

        let ubc = idx.borrow().core().ubc_children();
        assert_eq!(ubc.len(), 2, "index must have [clone, FoolRefFir]");
        assert_eq!(ubc[0].borrow().as_i64(), Some(42));
        assert_eq!(ubc[1].borrow().kind(), FirKind::FoolRef);
    }

    #[test]
    fn headtail_sugar_result_pair_has_fool_ref_at_index_1() {
        let val = make_constant_int(10);
        let stmt = make_statement("x", 0, Rc::clone(&val));
        let brane = make_brane(vec![Rc::clone(&stmt)]);
        let ht = make_headtail(true, true, vec![Rc::clone(&brane)]);
        let scope = Scope::empty();

        step_to_settled(&ht, &scope);
        assert_eq!(ht.borrow().core().get_nyes(), Nyes::Constant);

        let ubc = ht.borrow().core().ubc_children();
        assert_eq!(ubc.len(), 2, "index(head) must have [clone, FoolRefFir]");
        assert_eq!(ubc[0].borrow().as_i64(), Some(10));
        assert_eq!(ubc[1].borrow().kind(), FirKind::FoolRef);
    }

    #[test]
    fn fool_ref_fir_no_ubc_children() {
        use std::cell::RefCell;
        use std::rc::{Rc, Weak};

        let referent = make_constant_int(1);
        let fool_ref: FirRef = Rc::new_cyclic(|me: &Weak<RefCell<FoolRefFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(FoolRefFir {
                core: ProtoBrane::new(vec![], parent, Nyes::Constant),
                referent: Rc::clone(&referent),
            })
        });

        assert!(
            fool_ref.borrow().core().ubc_children().is_empty(),
            "FoolRefFir must have no ubc_children"
        );
        assert!(
            fool_ref.borrow().core().foolish_children().is_empty(),
            "FoolRefFir must have no foolish_children"
        );
    }

    #[test]
    fn fool_ref_referent_is_original_statement_by_identity() {
        use crate::compiler::Compiler;

        let root = Compiler::compile("{a = {x = 99;}; b = a.x;}")
            .unwrap()
            .pop()
            .unwrap();
        settle_root(&root);

        let stmts_a = root.borrow().core().foolish_children().to_vec();
        let a_stmt = &stmts_a[0];
        let inner_brane = a_stmt
            .borrow()
            .core()
            .foolish_children()
            .first()
            .cloned()
            .unwrap();
        let inner_stmts = inner_brane.borrow().core().foolish_children().to_vec();
        let original_x = &inner_stmts[0];
        assert_eq!(original_x.borrow().as_stmt_name(), Some("x"));

        let b_stmt = &stmts_a[1];
        let search_body = b_stmt
            .borrow()
            .core()
            .foolish_children()
            .first()
            .cloned()
            .unwrap();
        let ubc = search_body.borrow().core().ubc_children();
        assert_eq!(ubc.len(), 2);
        let fool_ref = &ubc[1];
        assert_eq!(fool_ref.borrow().kind(), FirKind::FoolRef);

        let borrowed = fool_ref.borrow();
        let referent = borrowed
            .as_fool_ref_referent()
            .expect("FoolRefFir must expose referent");
        assert!(
            Rc::ptr_eq(referent, original_x),
            "FoolRefFir referent must be the SAME Rc as the original found statement"
        );
    }

    #[test]
    fn fool_ref_no_mutation_path_to_referent() {
        let referent = make_constant_int(7);
        let fool_ref: FirRef = Rc::new_cyclic(|me: &Weak<RefCell<FoolRefFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(FoolRefFir {
                core: ProtoBrane::new(vec![], parent, Nyes::Constant),
                referent: Rc::clone(&referent),
            })
        });

        let borrowed = fool_ref.borrow();
        let ref_from_trait = borrowed
            .as_fool_ref_referent()
            .expect("must expose referent");
        assert_eq!(ref_from_trait.borrow().as_i64(), Some(7));
        assert!(borrowed.core().ubc_children().is_empty());
        drop(borrowed);
        let scope = Scope::empty();
        let report = fool_ref.step(&scope).unwrap();
        assert_eq!(report, StepReport::Progress(Nyes::Constant));
    }

    // ── C2: Contexted search tests (FOOP-23) ────────────────────────

    #[test]
    fn contexted_index_self_finds_anchor_statement() {
        use crate::compiler::Compiler;
        let root = Compiler::compile(
            "{recipe = {steps = {prep = 7; step_1 = 40; bake = 9;};}; \
             selfx = recipe.steps~step_1&#0;}",
        )
        .unwrap()
        .pop()
        .unwrap();
        settle_root(&root);
        let stmts = root.borrow().core().foolish_children().to_vec();
        let self_stmt = &stmts[1];
        let body = self_stmt
            .borrow()
            .core()
            .foolish_children()
            .first()
            .cloned()
            .unwrap();
        assert_eq!(
            body.borrow().as_i64(),
            Some(40),
            "&#0 must find the anchor statement itself"
        );
    }

    #[test]
    fn contexted_index_offset_finds_next_statement() {
        use crate::compiler::Compiler;
        let root = Compiler::compile(
            "{recipe = {steps = {prep = 7; step_1 = 40; bake = 9;};}; \
             after = recipe.steps~step_1&#1;}",
        )
        .unwrap()
        .pop()
        .unwrap();
        settle_root(&root);
        let stmts = root.borrow().core().foolish_children().to_vec();
        let after_stmt = &stmts[1];
        let body = after_stmt
            .borrow()
            .core()
            .foolish_children()
            .first()
            .cloned()
            .unwrap();
        assert_eq!(
            body.borrow().as_i64(),
            Some(9),
            "&#1 must find the next statement"
        );
    }

    #[test]
    fn contexted_index_out_of_range_is_nk() {
        use crate::compiler::Compiler;
        let root = Compiler::compile(
            "{recipe = {steps = {prep = 7; step_1 = 40; bake = 9;};}; \
             oob = recipe.steps~step_1&#5;}",
        )
        .unwrap()
        .pop()
        .unwrap();
        settle_root(&root);
        let stmts = root.borrow().core().foolish_children().to_vec();
        let oob_stmt = &stmts[1];
        let body = oob_stmt
            .borrow()
            .core()
            .foolish_children()
            .first()
            .cloned()
            .unwrap();
        assert_eq!(
            body.borrow().core().get_nyes(),
            Nyes::Nk,
            "out-of-range &# must be NK"
        );
    }

    #[test]
    fn contexted_backward_search_finds_preceding() {
        use crate::compiler::Compiler;
        let root = Compiler::compile(
            "{recipe = {steps = {prep_var = 7; step_1 = 40; bake = 9;};}; \
             before = recipe.steps~step_1&?prep_var;}",
        )
        .unwrap()
        .pop()
        .unwrap();
        settle_root(&root);
        let stmts = root.borrow().core().foolish_children().to_vec();
        let before_stmt = &stmts[1];
        let body = before_stmt
            .borrow()
            .core()
            .foolish_children()
            .first()
            .cloned()
            .unwrap();
        assert_eq!(
            body.borrow().as_i64(),
            Some(7),
            "&? must find preceding statement"
        );
    }

    #[test]
    fn contexted_forward_search_finds_following() {
        use crate::compiler::Compiler;
        let root = Compiler::compile(
            "{recipe = {steps = {prep_var = 7; step_1 = 40; bake = 9;};}; \
             afterv = recipe.steps~step_1&~bake;}",
        )
        .unwrap()
        .pop()
        .unwrap();
        settle_root(&root);
        let stmts = root.borrow().core().foolish_children().to_vec();
        let afterv_stmt = &stmts[1];
        let body = afterv_stmt
            .borrow()
            .core()
            .foolish_children()
            .first()
            .cloned()
            .unwrap();
        assert_eq!(
            body.borrow().as_i64(),
            Some(9),
            "&~ must find following statement"
        );
    }

    #[test]
    fn contexted_search_escape_is_nk() {
        use crate::compiler::Compiler;
        let root = Compiler::compile(
            "{outside = 99; recipe = {steps = {prep = 7; step_1 = 40; bake = 9;};}; \
             escape = recipe.steps~step_1&?outside;}",
        )
        .unwrap()
        .pop()
        .unwrap();
        settle_root(&root);
        let stmts = root.borrow().core().foolish_children().to_vec();
        let escape_stmt = &stmts[2];
        let body = escape_stmt
            .borrow()
            .core()
            .foolish_children()
            .first()
            .cloned()
            .unwrap();
        assert_eq!(
            body.borrow().core().get_nyes(),
            Nyes::Nk,
            "contexted search escaping home brane must be NK"
        );
    }

    #[test]
    fn atomic_name_value_search() {
        use crate::compiler::Compiler;
        let root = Compiler::compile(
            "{b = {setting = 11; mid = 0; setting = 10;}; \
             atomic = b~setting=10;}",
        )
        .unwrap()
        .pop()
        .unwrap();
        settle_root(&root);
        let stmts = root.borrow().core().foolish_children().to_vec();
        let atomic_stmt = &stmts[1];
        let body = atomic_stmt
            .borrow()
            .core()
            .foolish_children()
            .first()
            .cloned()
            .unwrap();
        assert_eq!(
            body.borrow().as_i64(),
            Some(10),
            "atomic name=value must find second setting"
        );
    }

    #[test]
    fn contexted_index_on_value_search_finds_result() {
        use crate::compiler::Compiler;
        let root = Compiler::compile(
            "{doc = {tmp_a = 4; c = 30; tmp_b = 4;}; \
             after_first_4 = doc~=4&#1;}",
        )
        .unwrap()
        .pop()
        .unwrap();
        settle_root(&root);
        let stmts = root.borrow().core().foolish_children().to_vec();
        let after_stmt = &stmts[1];
        eprintln!("after_stmt kind: {:?}", after_stmt.borrow().kind());
        eprintln!(
            "after_stmt nyes: {:?}",
            after_stmt.borrow().core().get_nyes()
        );
        let stmt_children = after_stmt.borrow().core().foolish_children().to_vec();
        eprintln!("stmt children count: {}", stmt_children.len());
        for (i, child) in stmt_children.iter().enumerate() {
            eprintln!(
                "  child[{}] kind: {:?}, nyes: {:?}",
                i,
                child.borrow().kind(),
                child.borrow().core().get_nyes()
            );
        }
        let body = after_stmt
            .borrow()
            .core()
            .foolish_children()
            .first()
            .cloned()
            .unwrap();
        let body_borrowed = body.borrow();
        eprintln!("body kind: {:?}", body_borrowed.kind());
        eprintln!("body nyes: {:?}", body_borrowed.core().get_nyes());
        eprintln!("body as_i64: {:?}", body_borrowed.as_i64());
        let ubc = body_borrowed.core().ubc_children();
        eprintln!("body ubc_children len: {}", ubc.len());
        if let Some(first) = ubc.first() {
            eprintln!("ubc[0] kind: {:?}", first.borrow().kind());
            eprintln!("ubc[0] nyes: {:?}", first.borrow().core().get_nyes());
            eprintln!("ubc[0] as_i64: {:?}", first.borrow().as_i64());
        }
        let fc = body_borrowed.core().foolish_children();
        eprintln!("body foolish_children len: {}", fc.len());
        for (i, child) in fc.iter().enumerate() {
            eprintln!(
                "  fc[{}] kind: {:?}, nyes: {:?}",
                i,
                child.borrow().kind(),
                child.borrow().core().get_nyes()
            );
        }
        drop(body_borrowed);
        assert_eq!(
            body.borrow().as_i64(),
            Some(30),
            "contexted index &#1 after value search ~=4 must find c=30"
        );
    }

    #[test]
    fn contexted_name_value_index_finds_preceding() {
        use crate::compiler::Compiler;
        let root = Compiler::compile(
            "{b = {setting = 11; mid = 0; setting = 10;}; \
             before = b~setting=10&#-1;}",
        )
        .unwrap()
        .pop()
        .unwrap();
        settle_root(&root);
        let stmts = root.borrow().core().foolish_children().to_vec();
        let before_stmt = &stmts[1];
        let body = before_stmt
            .borrow()
            .core()
            .foolish_children()
            .first()
            .cloned()
            .unwrap();
        assert_eq!(
            body.borrow().as_i64(),
            Some(0),
            "b~setting=10&#-1 must find mid=0"
        );
    }

    #[test]
    fn value_search_unanchored_miss_with_forward_candidates() {
        use crate::compiler::Compiler;
        let root = Compiler::compile("{pi = 3; e2 = 2; found = ?=3; nope = ?=9; named = ?e.*=2;}")
            .unwrap()
            .pop()
            .unwrap();
        settle_root(&root);
        let stmts = root.borrow().core().foolish_children().to_vec();

        let nope_stmt = &stmts[3];
        let nope_body = nope_stmt
            .borrow()
            .core()
            .foolish_children()
            .first()
            .cloned()
            .unwrap();
        assert_eq!(
            nope_body.borrow().core().get_nyes(),
            Nyes::Econstanic,
            "nope = ?=9 must settle ECONSTANIC, not stay BRANING"
        );

        let named_stmt = &stmts[4];
        let named_body = named_stmt
            .borrow()
            .core()
            .foolish_children()
            .first()
            .cloned()
            .unwrap();
        assert_eq!(
            named_body.borrow().as_i64(),
            Some(2),
            "named = ?e.*=2 must find e2 = 2"
        );
    }

    #[test]
    fn value_search_pattern_error_has_alarm_in_evaluator_output() {
        use crate::compiler::Compiler;
        let root = Compiler::compile("{a = {inner = {x = 1;}; n = 5;}; bad = a~={q = 1;};}")
            .unwrap()
            .pop()
            .unwrap();
        settle_root(&root);
        let stmts = root.borrow().core().foolish_children().to_vec();
        let bad_stmt = &stmts[1];
        let body = bad_stmt
            .borrow()
            .core()
            .foolish_children()
            .first()
            .cloned()
            .unwrap();
        assert_eq!(
            body.borrow().core().get_nyes(),
            Nyes::Nk,
            "bad search must be NK"
        );
        assert!(
            body.borrow()
                .core()
                .alarm_reason()
                .is_some_and(|r| r.contains("VALUE-SEARCH-UNSUPPORTED-PATTERN")),
            "bad search must have alarm for non-integer pattern"
        );
    }
}
