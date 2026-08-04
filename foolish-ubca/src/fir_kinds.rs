use std::cell::RefCell;
use std::rc::{Rc, Weak};

use foolish_core::fir::Nyes;
use regex::Regex;

use crate::identifier::{Characterizations, Identifier};

use crate::fir_trait::{Fir, FirKind, FirRef, FirRefExt, Scope, UbcError};
use crate::nyes_ext::NyesExt;
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
                Nyes::Woconstanic => current = result?,
                _ => return None,
            }
        }
    }

    #[inline(always)]
    fn resolve_anchor(&self) -> FirRef {
        self.value()
    }

    fn find_stmt_index(&self, stmt: &FirRef) -> Option<usize> {
        let count = self.borrow().stmt_count()?;
        for i in 0..count {
            if let Some(c) = self.borrow().stmt_at(i)
                && Rc::ptr_eq(&c, stmt)
            {
                return Some(i);
            }
        }
        None
    }
}

impl ProtoBrane {
    /// `pub(crate)` for `system_foo::ComparisonFir::constanic_clone`, which
    /// clones its children exactly as the kinds in this module do.
    pub(crate) fn clone_children_for_constanic_clone(
        source: &ProtoBrane,
        self_weak: &Weak<RefCell<dyn Fir>>,
        new_parent: &Weak<RefCell<dyn Fir>>,
        nyes: Nyes,
        sfm: bool,
        skip_foolish_children: bool,
    ) -> ProtoBrane {
        let cloned_children: Vec<FirRef> = if skip_foolish_children {
            Vec::new()
        } else {
            source
                .foolish_children()
                .iter()
                .enumerate()
                .map(|(i, c)| ProtoBrane::constanic_clone_at(c, self_weak, i, sfm, false))
                .collect()
        };
        let core = ProtoBrane::new(
            cloned_children,
            new_parent.clone(),
            nyes.transform_for_clone(sfm),
        );
        for ubc in source.ubc_children() {
            core.push_ubc_child(ProtoBrane::constanic_clone_at(
                &ubc, self_weak, 0, sfm, false,
            ));
        }
        core
    }

    pub(crate) fn constanic_clone_at(
        fir_ref: &FirRef,
        new_parent: &Weak<RefCell<dyn Fir>>,
        index: usize,
        descendent_of_sfm_and_foolishly_ignorant: bool,
        skip_foolish_children: bool,
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
                    skip_foolish_children,
                );
            }
            if let Some(inner) = source.core().foolish_children().first().cloned() {
                return Self::constanic_clone_at(
                    &inner,
                    new_parent,
                    index,
                    descendent_of_sfm_and_foolishly_ignorant,
                    skip_foolish_children,
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
                        skip_foolish_children,
                    );
                    RefCell::new(OperatorFir { core, op: op_name })
                })
            }
            // The clone that MAKES a comparison work. `'lt` is cloned out of
            // system.foo and recoordinated into the referencing brane; the
            // clone's SFF-marked operand lookups then resolve against that
            // brane's real neighbors (FOOP-33 §5.0). Same shape as Operator
            // above, plus the operator identity, which `as_op_name` carries.
            FirKind::Comparison => {
                let op = borrowed
                    .as_op_name()
                    .and_then(crate::system_foo::ComparisonOp::from_searchable_name);
                match op {
                    Some(op) => crate::system_foo::ComparisonFir::constanic_clone(
                        op,
                        &borrowed,
                        new_parent,
                        nyes,
                        descendent_of_sfm_and_foolishly_ignorant,
                        skip_foolish_children,
                    ),
                    // Unreachable: a ComparisonFir always reports one of the
                    // five names. Degrading to NK rather than panicking keeps
                    // a construction defect from taking down the evaluator.
                    None => Rc::new(RefCell::new(NkFir {
                        core: ProtoBrane::new(vec![], new_parent.clone(), Nyes::Nk),
                        reason: "comparison: unknown operator".to_string(),
                    })),
                }
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
                    let children: Vec<FirRef> = if skip_foolish_children {
                        Vec::new()
                    } else {
                        borrowed
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
                                    skip_foolish_children,
                                )
                            })
                            .collect()
                    };
                    let core = ProtoBrane::new(children, new_parent.clone(), clone_nyes_val);
                    if let Some(ref econ) = chain_econstanic {
                        core.push_ubc_child(ProtoBrane::constanic_clone_at(
                            econ, &self_weak, 0, false, false,
                        ));
                    } else {
                        for ubc in borrowed.core().ubc_children() {
                            core.push_ubc_child(ProtoBrane::constanic_clone_at(
                                &ubc,
                                &self_weak,
                                0,
                                descendent_of_sfm_and_foolishly_ignorant,
                                skip_foolish_children,
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
                        skip_foolish_children,
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
            FirKind::Concatenation => {
                let helpers_populated = !borrowed.core().ubc_children().is_empty();
                Rc::new_cyclic(|me: &Weak<RefCell<ConcatenationFir>>| {
                    let self_weak: Weak<RefCell<dyn Fir>> = me.clone();
                    let core = ProtoBrane::clone_children_for_constanic_clone(
                        borrowed.core(),
                        &self_weak,
                        new_parent,
                        nyes,
                        descendent_of_sfm_and_foolishly_ignorant,
                        skip_foolish_children,
                    );
                    RefCell::new(ConcatenationFir {
                        core,
                        _helpers_populated: std::cell::Cell::new(helpers_populated),
                    })
                })
            }
            FirKind::ConcatHelper => Rc::new_cyclic(|me: &Weak<RefCell<ConcatHelper>>| {
                let self_weak: Weak<RefCell<dyn Fir>> = me.clone();
                let core = ProtoBrane::clone_children_for_constanic_clone(
                    borrowed.core(),
                    &self_weak,
                    new_parent,
                    nyes,
                    descendent_of_sfm_and_foolishly_ignorant,
                    skip_foolish_children,
                );
                RefCell::new(ConcatHelper { core })
            }),
            FirKind::Statement => {
                let identifier = borrowed
                    .as_stmt_identifier()
                    .cloned()
                    .unwrap_or_else(|| Identifier::from_parts(vec![], ""));
                let line = index;
                Rc::new_cyclic(|me: &Weak<RefCell<StatementFir>>| {
                    let self_weak: Weak<RefCell<dyn Fir>> = me.clone();
                    let core = ProtoBrane::clone_children_for_constanic_clone(
                        borrowed.core(),
                        &self_weak,
                        new_parent,
                        nyes,
                        descendent_of_sfm_and_foolishly_ignorant,
                        skip_foolish_children,
                    );
                    RefCell::new(StatementFir {
                        core,
                        identifier,
                        line_number: line,
                        self_weak,
                        nf_reason: RefCell::new(None),
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
                    skip_foolish_children,
                );
                RefCell::new(BraneFir {
                    core,
                    characterizations: Characterizations::from_brane_parts(
                        borrowed.as_brane_characterizations().to_vec(),
                    ),
                })
            }),
            FirKind::Unknown => Rc::new(RefCell::new(NkFir {
                core: ProtoBrane::new(vec![], new_parent.clone(), Nyes::Nk),
                reason: "unknown fir kind".to_owned(),
            })),
            FirKind::FoolRef => Rc::clone(fir_ref),
            FirKind::Creation => Rc::clone(fir_ref),
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

/// NF (Not Foolish) — a sub-condition of NK for violations of Foolish's own rules.
/// Used as a reason string prefix on NkFir. NF is terminal and behaves identically
/// to NK in all downstream machinery — it is a semantic label, not a new control flow.
pub const NF_PREFIX: &str = "not-foolish";

/// Check if an NK reason string indicates an NF (Not Foolish) condition.
pub fn is_nf_reason(reason: &str) -> bool {
    reason.contains(NF_PREFIX)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Equality {
    Equal,
    NotEqual,
    Unknowable,
}

pub fn default_equal(a: &FirRef, b: &FirRef) -> Equality {
    let a_borrowed = a.borrow();
    let b_borrowed = b.borrow();
    if a_borrowed.core().get_nyes() == Nyes::Nk || b_borrowed.core().get_nyes() == Nyes::Nk {
        return Equality::Unknowable;
    }
    if let (Some(av), Some(bv)) = (a_borrowed.as_i64(), b_borrowed.as_i64()) {
        return if av == bv {
            Equality::Equal
        } else {
            Equality::NotEqual
        };
    }
    drop(a_borrowed);
    drop(b_borrowed);
    // Resolve through to the settled value (e.g. a search reference to a
    // creation resolves to the CreationFir it found) before comparing kinds.
    // `.value()` is a no-op for FIRs that are already their own value.
    let a_resolved = a.value();
    let b_resolved = b.value();
    let a_borrowed = a_resolved.borrow();
    let b_borrowed = b_resolved.borrow();
    if a_borrowed.kind() == FirKind::Creation && b_borrowed.kind() == FirKind::Creation {
        return if Rc::ptr_eq(&a_resolved, &b_resolved) {
            Equality::Equal
        } else {
            Equality::NotEqual
        };
    }
    // Two branes: brane-vs-brane equivalence is unspecified (FOOP-23) → genuinely unknowable.
    if a_borrowed.kind() == FirKind::Brane && b_borrowed.kind() == FirKind::Brane {
        return Equality::Unknowable;
    }
    // Different non-NK constanic kinds (brane-vs-integer, integer-vs-creation, etc.)
    // are provably not equal — a brane is never an integer (different FIR kinds, decidable).
    // The matcher should Reject (skip) and continue scanning, not NkStop (abort).
    Equality::NotEqual
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
                    false,
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
                            false,
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
                            false,
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
                            false,
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
                false,
            ));
            self.core.set_nyes(Nyes::Constant);
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct StatementFir {
    pub(crate) core: ProtoBrane,
    pub(crate) identifier: Identifier,
    pub(crate) line_number: usize,
    /// Self-reference, established at construction via `Rc::new_cyclic` (same
    /// pattern as `ProtoBrane.parent`, one level up). Needed ONLY by the
    /// null-characterized name constant rule (FOOP-33 §4): `fir_op_step(&self,
    /// scope: &Scope)` has no `self_ref` parameter, but detecting an ancestral
    /// conflict requires calling `_ib_search`/`_ab_search` (which take
    /// `self_ref: &FirRef`) FROM this statement's own position. Every other use
    /// of `StatementFir` needs no self-reference; this field exists solely to
    /// make that one check possible without threading `self_ref` through the
    /// whole `Fir` trait.
    pub(crate) self_weak: Weak<RefCell<dyn Fir>>,
    /// Set once, by this statement's own `fir_op_step`, when the null-
    /// characterized name constant rule (FOOP-33 §4) refuses a conflicting
    /// redefinition. `None` in the overwhelmingly common case (not a
    /// null-characterized name, or no conflict) — readers must keep reading
    /// `foolish_children().first()` (the written body) exactly as before.
    /// `Some(reason)` means `get_value()` (via `settled_result`/`.value()`)
    /// must present a fresh `NkFir` with this reason INSTEAD of the written
    /// body — this is what distinguishes "the rule refused this redefinition"
    /// from "the body itself genuinely settled to NK" (e.g. `k = ???`), which
    /// must keep presenting the real NK body unchanged.
    pub(crate) nf_reason: RefCell<Option<String>>,
}

impl StatementFir {
    pub fn name(&self) -> &str {
        self.identifier.identifier_name()
    }

    pub fn identifier(&self) -> &Identifier {
        &self.identifier
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
        Self::statement_with_identifier(
            Identifier::from_parts(vec![], name),
            line_number,
            body,
            parent,
        )
    }

    pub fn statement_with_identifier(
        identifier: Identifier,
        line_number: usize,
        body: FirRef,
        parent: Weak<RefCell<dyn Fir>>,
    ) -> FirRef {
        Rc::new_cyclic(|me: &Weak<RefCell<StatementFir>>| {
            let self_weak: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(StatementFir {
                core: ProtoBrane::new(vec![body], parent, Nyes::Prembrionic),
                identifier,
                line_number,
                self_weak,
                nf_reason: RefCell::new(None),
            })
        })
    }

    /// FOOP-33 §4 — the null-characterized name constant rule's own check,
    /// run by a null-characterized statement ON ITSELF once its body is
    /// constanic. Sets `self.nf_reason` (once, terminal) iff a prior
    /// null-characterized statement of the same `searchable_name()` exists
    /// (via IB — earlier in this same brane — then AB — an ancestor brane)
    /// AND its value is not `Equal` (by `default_equal`) to this statement's
    /// own body. Does NOT touch `self.core`'s nyes here — the caller
    /// (`fir_op_step`) still sets it from `body_nyes` afterward; `nf_reason`
    /// alone is what `settled_result` consults to substitute the NK.
    ///
    /// **Does not cover concatenation.** A concatenation-merged statement is
    /// constructed via `constanic_clone_at`, which — for an already-constanic
    /// source (the overwhelmingly common case) — builds the clone DIRECTLY at
    /// its terminal `Nyes` (`transform_for_clone`), skipping `Prembrionic`/
    /// `Embryonic`/`Braning` entirely, so THIS check (which lives in the
    /// `Braning` arm of `fir_op_step`) never runs for it. Concatenation has
    /// its own, separate application of the same rule:
    /// [`ConcatenationFir::apply_null_const_rule_to_merged_stmt`].
    fn check_null_const_conflict(&self, body: &FirRef) {
        if self.nf_reason.borrow().is_some() {
            return; // already resolved (terminal, no re-alarm — Gotcha #5a).
        }
        let Some(self_rc) = self.self_weak.upgrade() else {
            return; // torn down; nothing to check against.
        };
        let pattern = self.identifier.searchable_name();
        let prior = self_rc
            .borrow()
            ._ib_search(&self_rc, pattern)
            .or_else(|| self_rc.borrow()._ab_search(&self_rc, pattern));
        let Some((prior_stmt, _prior_nyes)) = prior else {
            return; // no earlier definition -- this statement establishes the constant.
        };
        let Some(prior_body) = statement_value_for_comparison(&prior_stmt) else {
            return; // malformed statement with no body -- nothing to compare.
        };
        if !prior_body.borrow().core().get_nyes().is_constanic() {
            return; // prior definition not yet settled -- nothing to compare yet.
        }
        if default_equal(body, &prior_body) != Equality::Equal {
            *self.nf_reason.borrow_mut() =
                Some(null_const_nf_reason(self.identifier.identifier_name()));
        }
    }
}

/// The value a statement PRESENTS — `settled_result()` (the NF refusal NK, if
/// this statement was itself already refused by the null-const rule) if set,
/// else the raw written body. This is the ONE place "what does this statement
/// actually resolve to" is decided; every reader of a statement's value must
/// go through it rather than reaching into `foolish_children().first()`
/// directly, or it will present the pre-refusal RHS instead of the NF NK.
/// Used by `StatementFir::check_null_const_conflict` and
/// `ConcatenationFir::apply_null_const_rule_to_merged_stmt` (FOOP-33 §4 —
/// poisoning must be transitive: comparing against an ALREADY-refused prior
/// statement's original RHS would let a later-but-equal-to-the-invalid-one
/// redefinition wrongly slip through) and by `evaluator.rs`'s
/// `proto_to_core_fir_inner` (the sequencer/einmo rendering path — without
/// this, the NF refusal is enforced internally but never actually rendered).
pub(crate) fn statement_value_for_comparison(stmt: &FirRef) -> Option<FirRef> {
    let borrowed = stmt.borrow();
    borrowed
        .settled_result()
        .or_else(|| borrowed.core().foolish_children().first().cloned())
}

/// The NF (Not Foolish) reason string for a refused null-characterized name
/// constant redefinition — `"'<name> not-foolish"`. One place constructs this
/// so the two trigger sites (`StatementFir`'s own step, and `ConcatenationFir`'s
/// merge) can never drift out of sync with `NF_PREFIX`/`is_nf_reason`.
fn null_const_nf_reason(name: &str) -> String {
    format!("'{name} {NF_PREFIX}")
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
                        if self.identifier.is_nully_characterizing_coordinate_name() {
                            self.check_null_const_conflict(body);
                        }
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

    /// `None` in the overwhelmingly common case — a plain `StatementFir` has no
    /// separate result child; readers (`clone_stmt_result` and its two `IndexFir`
    /// counterparts) fall through to the written body (`foolish_children().first()`)
    /// exactly as before this override existed. `Some(nk)` ONLY when the null-
    /// characterized name constant rule (FOOP-33 §4) refused a conflicting
    /// redefinition — this is what makes `get_value()` present the refusal NK
    /// instead of the written RHS, without ever touching the written body itself
    /// (the immutable `foolish_children` topology) or another FIR's own nyes.
    ///
    /// Built directly at `Nyes::Nk` (NOT via `NkFir::nk`, which starts
    /// `Prembrionic` and needs a step to reach `Nk`): `settled_result`'s own
    /// contract is "applies the constanic gate itself," so what it returns
    /// must already BE constanic — callers (including the concatenation
    /// merge's own null-const comparison, which reads a freshly-built prior
    /// statement's `settled_result()` without ever stepping it) rely on that.
    fn settled_result(&self) -> Option<FirRef> {
        let reason = self.nf_reason.borrow().clone()?;
        Some(Rc::new(RefCell::new(NkFir {
            core: ProtoBrane::new(vec![], self.core.parent_weak(), Nyes::Nk),
            reason,
        })))
    }

    /// FOOP-33 §4 — the ONLY writer of `nf_reason` other than this
    /// statement's own `check_null_const_conflict`: `ConcatenationFir`'s
    /// merge-collision check, which cannot call `check_null_const_conflict`
    /// (the clone was built already-constanic, so its `Braning` step —
    /// where that check lives — never runs). Terminal like the other path:
    /// once set, later calls are ignored (first refusal wins).
    fn set_nf_reason(&self, reason: String) {
        let mut slot = self.nf_reason.borrow_mut();
        if slot.is_none() {
            *slot = Some(reason);
        }
    }

    fn as_stmt_identifier(&self) -> Option<&Identifier> {
        Some(&self.identifier)
    }
    fn as_stmt_line_number(&self) -> Option<usize> {
        Some(self.line_number)
    }

    fn _ib_search(&self, self_ref: &FirRef, name: &str) -> Option<(FirRef, Nyes)> {
        // `checked_sub` (not `saturating_sub`): a first statement (line 0)
        // has no preceding range, so return None rather than searching [0, 0]
        // and finding itself.
        let backward_end = self.line_number.checked_sub(1)?;
        let brane = self._get_my_brane(self_ref)?;
        let brane_borrowed = brane.borrow();
        brane_borrowed
            ._search_brane(name, backward_end, 0)
            .map(|(_idx, stmt, nyes)| (stmt, nyes))
    }
}

#[derive(Debug)]
pub struct BraneFir {
    pub(crate) core: ProtoBrane,
    pub(crate) characterizations: Characterizations,
}

impl BraneFir {
    pub fn brane(children: Vec<FirRef>, parent: Weak<RefCell<dyn Fir>>) -> FirRef {
        Rc::new(RefCell::new(BraneFir {
            core: ProtoBrane::new(children, parent, Nyes::Prembrionic),
            characterizations: Characterizations::default(),
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

    fn stmt_count(&self) -> Option<usize> {
        Some(self.core.foolish_children().len())
    }

    fn stmt_at(&self, idx: usize) -> Option<FirRef> {
        self.core.foolish_children().get(idx).cloned()
    }

    fn as_brane_characterizations(&self) -> &[String] {
        self.characterizations.components()
    }

    fn _ab_search(&self, self_ref: &FirRef, name: &str) -> Option<(FirRef, Nyes)> {
        let stmt = self._get_my_statement(self_ref);
        if Rc::ptr_eq(&stmt, self_ref) {
            return None;
        }
        let stmt_borrowed = stmt.borrow();
        if let Some((body, nyes)) = stmt_borrowed._ib_search(&stmt, name) {
            return Some((body, nyes));
        }
        drop(stmt_borrowed);
        let parent_brane = self._get_my_brane(self_ref)?;
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
            // Every name-search matches against searchable_name — the full
            // characterized LHS as one string. A plain pattern (`^x$`) naturally
            // won't match a characterized searchable_name (`"tag'x"`) under this
            // full-string anchoring, and a `'`-bearing pattern matches only the
            // identically-characterized name. One projection, one comparison.
            let candidate = child_borrowed
                .as_stmt_identifier()
                .map(|id| id.searchable_name());
            if let Some(sn) = candidate
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
        // Prefer settled_result() over the raw written body: for a plain
        // StatementFir this is None (unchanged behavior — falls through to the
        // body below); the null-characterized name constant rule (FOOP-33 §4)
        // is the ONLY thing that ever makes it Some, substituting the refusal
        // NK for the written RHS without mutating the body's own FIR/nyes.
        let body = statement_value_for_comparison(stmt).expect("statement must have a body");
        let index = stmt.borrow().as_stmt_line_number().unwrap_or(0);
        ProtoBrane::constanic_clone_at(
            &body,
            new_parent,
            index,
            descendent_of_sfm_and_foolishly_ignorant,
            false,
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
        let h_brane = referent.borrow()._get_my_brane(&referent)?;
        let p = h_brane.find_stmt_index(&referent)?;
        let brane_len = h_brane.borrow().stmt_count().unwrap_or(0);
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
            // A WOCONSTANIC value expression is *waiting on its constanics* — it
            // found its dependencies (e.g. a nested search resolved), they are
            // just not yet concrete values. The value search inherits that: it
            // is WOCONSTANIC (may gain a value via recoordination), NOT a miss.
            // Only a genuinely ECONSTANIC value expression (an unanchored miss
            // inside the pattern) collapses the search to ECONSTANIC.
            // (FOOP-23 rendering appendix: r3 = a~=c-d+v must settle WOCONSTANIC.)
            Nyes::Woconstanic => {
                self.core.set_nyes(Nyes::Woconstanic);
                return false;
            }
            Nyes::Econstanic => {
                self.core.set_nyes(Nyes::Econstanic);
                return false;
            }
            _ => {}
        }
        let resolved_kind = value_fir.value().borrow().kind();
        if value_fir.borrow().as_i64().is_none() && resolved_kind != FirKind::Creation {
            self.core.set_alarm_reason(
                "VALUE-SEARCH-UNSUPPORTED-PATTERN: pattern is neither integer nor creation"
                    .to_string(),
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
        let brane = stmt.borrow()._get_my_brane(&stmt)?;
        let idx = brane.find_stmt_index(&stmt)?;
        // `checked_sub` (not `saturating_sub`): index 0 has no preceding
        // range — return None rather than searching [0, 0] and self-matching
        // (as in the sibling `_ib_search`).
        let search_end = idx.checked_sub(1)?;
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
                borrowed._get_my_statement(&current_brane)
            };
            if Rc::ptr_eq(&stmt, &current_brane) {
                return None;
            }
            let parent_brane = stmt.borrow()._get_my_brane(&stmt)?;
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
                    if !resolved.borrow().is_brane_like() {
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
                if self.anchored {
                    let anchor = Rc::clone(&self.core.foolish_children()[0]);
                    self.core.push_task(anchor);
                    self.core.set_nyes(Nyes::Braning);
                } else if !self.core.ubc_children().is_empty() {
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
                } else if self.contexted && self.anchored {
                    let result = self.contexted_search_from_anchor(scope);
                    match result {
                        Some((stmt, nyes)) => self.handle_found(stmt, nyes, scope),
                        None => self.core.set_nyes(if self.anchored {
                            Nyes::Nk
                        } else {
                            Nyes::Econstanic
                        }),
                    }
                } else if self.anchored {
                    use contextful_search::{
                        BraneNavigator, SearchPredicate, contextful_search_scan_no_body_check,
                    };
                    let anchor = Rc::clone(&self.core.foolish_children()[0]);
                    let resolved = anchor.resolve_anchor();
                    if resolved.borrow().core().get_nyes() == Nyes::Nk
                        || !resolved.borrow().is_brane_like()
                    {
                        self.core.set_nyes(Nyes::Nk);
                    } else {
                        let mut nav = BraneNavigator::new(&resolved, self.forward);
                        let predicate = SearchPredicate::Name {
                            pattern: self.pattern.clone(),
                        };
                        match contextful_search_scan_no_body_check(&mut nav, &predicate) {
                            ScanOutcome::Found(stmt) => {
                                let nyes = stmt.borrow().core().get_nyes();
                                self.handle_found(stmt, nyes, scope);
                            }
                            _ => self.core.set_nyes(Nyes::Nk),
                        }
                    }
                } else {
                    let result = self.ab_search_with_engine(scope);
                    match result {
                        Some((stmt, nyes)) => self.handle_found(stmt, nyes, scope),
                        None => self.core.set_nyes(Nyes::Econstanic),
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
        if node.borrow().kind() == FirKind::Statement {
            let brane = node.borrow()._get_my_brane(&node)?;
            return Some((node, brane));
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

#[allow(dead_code)]
fn find_parent_brane(start: &ProtoBrane) -> Option<FirRef> {
    start.parent().and_then(|p| p.borrow()._get_my_brane(&p))
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
                                let len = brane_ref.borrow().stmt_count().unwrap_or(0) as i32;
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
                                                        false,
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
                        let h_brane = referent.borrow()._get_my_brane(&referent)?;
                        let p = h_brane.find_stmt_index(&referent)?;
                        let target = p as i32 + self.offset;
                        let len = h_brane.borrow().stmt_count().unwrap_or(0) as i32;
                        if target < 0 || target >= len {
                            return None;
                        }
                        let mut nav = BraneNavigator::new(&h_brane, true);
                        let predicate = SearchPredicate::Index(target);
                        match contextful_search_scan_no_body_check(&mut nav, &predicate) {
                            ScanOutcome::Found(stmt) => {
                                let body = statement_value_for_comparison(&stmt)?;
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
                            false,
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
                    if !resolved.borrow().is_brane_like() {
                        self.core.set_nyes(Nyes::Nk);
                        return Ok(());
                    }
                    let mut nav = BraneNavigator::new(&resolved, true);
                    let predicate = SearchPredicate::Index(self.offset);
                    match contextful_search_scan_no_body_check(&mut nav, &predicate) {
                        ScanOutcome::Found(stmt) => {
                            let body = statement_value_for_comparison(&stmt);
                            match body {
                                Some(body) => {
                                    let self_weak = self.core.parent_weak();
                                    let clone = ProtoBrane::constanic_clone_at(
                                        &body,
                                        &self_weak,
                                        0,
                                        scope.has_ancestral_sfm,
                                        false,
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
    use super::{Equality, FirRef, SearchFir, default_equal};

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
                    // Matches against searchable_name (the full characterized LHS as
                    // one string) — a plain pattern naturally won't match a
                    // characterized name, and a '-bearing pattern matches only the
                    // identically-characterized name. See Identifier::searchable_name.
                    let name = match borrowed.as_stmt_searchable_name() {
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
                    match default_equal(&body, pattern) {
                        Equality::Equal => MatchOutcome::Approve,
                        Equality::NotEqual => MatchOutcome::Reject,
                        Equality::Unknowable => MatchOutcome::NkStop,
                    }
                }
                Self::NameValue { name, value } => {
                    let body = {
                        let borrowed = candidate.borrow();
                        let stmt_name = match borrowed.as_stmt_searchable_name() {
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
                    match default_equal(&body, value) {
                        Equality::Equal => MatchOutcome::Approve,
                        Equality::NotEqual => MatchOutcome::Reject,
                        Equality::Unknowable => MatchOutcome::NkStop,
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
                    let name = match borrowed.as_stmt_searchable_name() {
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
            let len = brane.borrow().stmt_count().unwrap_or(0);
            let children: Vec<FirRef> =
                (0..len).filter_map(|i| brane.borrow().stmt_at(i)).collect();
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
                // Resolve to the inner expression's value (like SF): once the
                // inner expr is constanic, expose its resolved value via
                // ubc_children so `.value()` unwraps to it. A search that
                // found nothing has no resolved child, so the search itself
                // becomes the value (still not brane-like — keeps the
                // ConcatBrane join gate correct for SFF-wrapped searches).
                let children = self.core.foolish_children().to_vec();
                if _decide_nyes_due_to_children(&children).is_some()
                    && let Some(expr) = children.first()
                {
                    let (result, result_nyes) = {
                        let borrowed = expr.borrow();
                        match borrowed.core().ubc_children().into_iter().next() {
                            Some(r) => {
                                let n = r.borrow().core().get_nyes();
                                (r, n)
                            }
                            None => (Rc::clone(expr), borrowed.core().get_nyes()),
                        }
                    };
                    self.core.push_ubc_child(result);
                    // The SFF wrapper is not a search — it can't be ECONSTANIC.
                    // An ECONSTANIC result means it's WAITING on that search
                    // (WOCONSTANIC), while the pushed result keeps its own
                    // ECONSTANIC. Same rule as SearchFir::nyes_from_found.
                    self.core.set_nyes(SearchFir::nyes_from_found(result_nyes));
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

/// Internal storage brane for ConcatBrane.
/// Holds constanic-cloned statements from concatenated elements.
/// Transparent: inherits all defaults, BraneFir-shaped stepping.
#[derive(Debug)]
pub struct ConcatHelper {
    pub(crate) core: ProtoBrane,
}

impl ConcatHelper {
    pub fn concat_helper(children: Vec<FirRef>, parent: Weak<RefCell<dyn Fir>>) -> FirRef {
        Rc::new(RefCell::new(ConcatHelper {
            core: ProtoBrane::new(children, parent, Nyes::Prembrionic),
        }))
    }
}

impl Fir for ConcatHelper {
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
        FirKind::ConcatHelper
    }

    fn stmt_count(&self) -> Option<usize> {
        Some(self.core.foolish_children().len())
    }

    fn stmt_at(&self, idx: usize) -> Option<FirRef> {
        self.core.foolish_children().get(idx).cloned()
    }

    fn is_brane_like(&self) -> bool {
        true
    }

    fn _search_brane(
        &self,
        expression: &str,
        starting_index: usize,
        ending_index: usize,
    ) -> Option<(usize, FirRef, Nyes)> {
        let children = self.core.foolish_children();
        if starting_index >= children.len() || ending_index >= children.len() {
            return None;
        }
        let range = if starting_index >= ending_index {
            Box::new((ending_index..=starting_index).rev()) as Box<dyn Iterator<Item = usize>>
        } else {
            Box::new(starting_index..=ending_index) as Box<dyn Iterator<Item = usize>>
        };
        for i in range {
            let child = &children[i];
            let child_borrowed = child.borrow();
            // See _search_brane above: every name-search matches against searchable_name.
            let candidate = child_borrowed
                .as_stmt_identifier()
                .map(|id| id.searchable_name());
            if let Some(sn) = candidate
                && SearchFir::matches_pattern(sn, expression)
            {
                return Some((i, Rc::clone(child), child_borrowed.core().get_nyes()));
            }
        }
        None
    }
}

#[derive(Debug)]
pub struct ConcatenationFir {
    pub(crate) core: ProtoBrane,
    pub(crate) _helpers_populated: std::cell::Cell<bool>,
}

impl ConcatenationFir {
    pub fn concatenation(elements: Vec<FirRef>, parent: Weak<RefCell<dyn Fir>>) -> FirRef {
        Rc::new(RefCell::new(ConcatenationFir {
            core: ProtoBrane::new(elements, parent, Nyes::Prembrionic),
            _helpers_populated: std::cell::Cell::new(false),
        }))
    }

    /// Populate _ConcatHelpers from settled element values.
    ///
    /// Phase A: single _ConcatHelper with all lines (no MAX_BRANE_SIZE limit yet).
    /// Performs settle-time typing check (each element value must be brane-like).
    /// Constanic-clones every element's lines into one `_ConcatHelper`,
    /// pushed to `ubc_children`. Structural only — the caller decides NYES.
    /// Callers guarantee every element's value is brane-like first (the
    /// join-readiness gate in `fir_op_step`). Empty (no lines) → no helper.
    fn populate_concat_helpers(&self) {
        let self_weak = self.core.parent_weak();
        let elements = self.core.foolish_children();

        let total_lines: usize = elements
            .iter()
            .map(|e| e.value().borrow().stmt_count().unwrap_or(0))
            .sum();

        if total_lines == 0 {
            return;
        }

        // Build the _ConcatHelper first so its Weak becomes the parent of
        // every cloned line — cross-element search resolution walks to it.
        let helper: Rc<RefCell<ConcatHelper>> = Rc::new(RefCell::new(ConcatHelper {
            core: ProtoBrane::new(vec![], self_weak.clone(), Nyes::Prembrionic),
        }));
        let helper_fir: FirRef = helper.clone();
        let helper_weak = Rc::downgrade(&helper_fir);
        let mut cloned_stmts: Vec<FirRef> = Vec::with_capacity(total_lines);
        let mut global_idx: usize = 0;

        for elem in elements {
            let resolved = elem.value();
            let count = resolved.borrow().stmt_count().unwrap_or(0);
            for i in 0..count {
                if let Some(stmt) = resolved.borrow().stmt_at(i) {
                    let clone = ProtoBrane::constanic_clone_at(
                        &stmt,
                        &helper_weak,
                        global_idx,
                        false,
                        false,
                    );
                    Self::apply_null_const_rule_to_merged_stmt(&clone, &cloned_stmts);
                    cloned_stmts.push(clone);
                    global_idx += 1;
                }
            }
        }

        if cloned_stmts.is_empty() {
            return;
        }

        *helper.borrow_mut() = ConcatHelper {
            core: ProtoBrane::new(cloned_stmts, self_weak.clone(), Nyes::Prembrionic),
        };

        self.core.push_ubc_child(helper_fir);
    }

    /// FOOP-33 §4 — the null-characterized name constant rule, applied at
    /// concatenation merge time. `StatementFir::check_null_const_conflict`
    /// (the ordinary same-brane/ancestral path, run from `fir_op_step`'s
    /// `Braning` arm) does NOT fire for `new_stmt`: it was built via
    /// `constanic_clone_at` from an already-constanic source, which
    /// constructs the clone DIRECTLY at its terminal `Nyes`
    /// (`Nyes::transform_for_clone`) — `Prembrionic`/`Embryonic`/`Braning`
    /// (and therefore the check that lives there) never run. Concatenation
    /// must therefore enforce the SAME rule itself, here, against the
    /// statements already merged BEFORE `new_stmt` in the growing helper.
    ///
    /// `already_merged` is searched in REVERSE (nearest-first) so a chain of
    /// three-or-more same-name clones compares each new one against the
    /// NEAREST prior — which, via `statement_value_for_comparison`'s
    /// settled_result()-first read, transitively carries any earlier refusal
    /// forward (Gotcha #5a: one rule, one NK mechanism, two trigger sites).
    fn apply_null_const_rule_to_merged_stmt(new_stmt: &FirRef, already_merged: &[FirRef]) {
        let is_nully = new_stmt
            .borrow()
            .as_stmt_identifier()
            .is_some_and(Identifier::is_nully_characterizing_coordinate_name);
        if !is_nully {
            return;
        }
        let pattern = new_stmt
            .borrow()
            .as_stmt_searchable_name()
            .map(str::to_owned);
        let Some(pattern) = pattern else { return };
        let Some(prior_stmt) = already_merged
            .iter()
            .rev()
            .find(|s| s.borrow().as_stmt_searchable_name() == Some(pattern.as_str()))
        else {
            return; // first occurrence of this null-const name in the merge -- permitted.
        };
        let Some(new_body) = statement_value_for_comparison(new_stmt) else {
            return;
        };
        let Some(prior_body) = statement_value_for_comparison(prior_stmt) else {
            return;
        };
        if !new_body.borrow().core().get_nyes().is_constanic()
            || !prior_body.borrow().core().get_nyes().is_constanic()
        {
            return; // one side not yet settled -- nothing to compare yet.
        }
        if default_equal(&new_body, &prior_body) != Equality::Equal {
            let name = new_stmt
                .borrow()
                .as_stmt_identifier()
                .map(|id| id.identifier_name().to_owned());
            if let Some(name) = name {
                new_stmt.borrow().set_nf_reason(null_const_nf_reason(&name));
            }
        }
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
                    // Empty ConcatBrane settles as empty constant brane immediately.
                    self.core.set_nyes(Nyes::Constant);
                } else {
                    // Call 1: push elements as tasks, transition to Braning.
                    self.core.set_nyes(Nyes::Braning);
                    for child in children {
                        self.core.push_task(child);
                    }
                }
            }
            Nyes::Braning => {
                // One pass over the elements, accumulating two verdicts:
                //  - all_brane_like: every value is a brane (can be iterated
                //    and copied — true for any NYES, incl. WOCONSTANIC/NK).
                //  - type_errors: indices of permanent non-branes (constantew
                //    but not brane-like) — genuine errors, all reported.
                let mut all_brane_like = true;
                let mut type_errors: Vec<usize> = Vec::new();
                for (idx, elem) in self.core.foolish_children().iter().enumerate() {
                    let brane_like = elem.value().borrow().is_brane_like();
                    all_brane_like &= brane_like;
                    if !brane_like && elem.borrow().core().get_nyes().is_constantew() {
                        type_errors.push(idx);
                    }
                }

                // A type error wins over "not ready yet" (a real bad element
                // is not masked by another still resolving).
                if !type_errors.is_empty() {
                    let self_weak = self.core.parent_weak();
                    let list = type_errors
                        .iter()
                        .map(usize::to_string)
                        .collect::<Vec<_>>()
                        .join(",");
                    let nk: FirRef = Rc::new(RefCell::new(NkFir {
                        core: ProtoBrane::new(vec![], self_weak, Nyes::Nk),
                        reason: format!(
                            "concatenation constituent indexes where it's not a brane: {list}"
                        ),
                    }));
                    self.core.push_ubc_child(nk);
                    self.core.set_nyes(Nyes::Nk);
                    return Ok(());
                }
                if !all_brane_like {
                    // Not a greedy join: some element isn't joinable yet.
                    // Settle WOCONSTANIC; the sequencer renders the raw
                    // un-joined elements.
                    self.core.set_nyes(Nyes::Woconstanic);
                    return Ok(());
                }

                if !self._helpers_populated.get() {
                    // First pass: build helpers, push them as tasks. Don't
                    // settle self's NYES yet — self must stay pre-constanic
                    // so the driver drains the helper tasks before re-entry.
                    self._helpers_populated.set(true);
                    self.populate_concat_helpers();
                    for helper in self.core.ubc_children() {
                        self.core.push_task(helper);
                    }
                } else {
                    // Helpers drained. Settle from the JOINED lines (the
                    // helpers), not the elements: the recoordinated joined
                    // copies can be constant even when the original element
                    // brane was WOCONSTANIC (e.g. `{c=a+b}` → joined `c=3`).
                    // Empty (no lines joined) → Constant, per the empty-brane
                    // convention.
                    let helpers = self.core.ubc_children();
                    let settled = if helpers.is_empty() {
                        Nyes::Constant
                    } else {
                        _decide_nyes_due_to_children(&helpers).unwrap_or(Nyes::Constant)
                    };
                    self.core.set_nyes(settled);
                }
            }
            _ => {}
        }
        Ok(())
    }
    fn kind(&self) -> FirKind {
        FirKind::Concatenation
    }

    fn stmt_count(&self) -> Option<usize> {
        if !self._helpers_populated.get() {
            if self.core.foolish_children().is_empty() {
                return Some(0);
            }
            self._helpers_populated.set(true);
            self.populate_concat_helpers();
        }
        let total: usize = self
            .core
            .ubc_children()
            .iter()
            .map(|h| h.borrow().stmt_count().unwrap_or(0))
            .sum();
        Some(total)
    }

    fn stmt_at(&self, idx: usize) -> Option<FirRef> {
        if !self._helpers_populated.get() {
            return None;
        }
        let mut remaining = idx;
        for helper in self.core.ubc_children() {
            let count = helper.borrow().stmt_count().unwrap_or(0);
            if remaining < count {
                return helper.borrow().stmt_at(remaining);
            }
            remaining -= count;
        }
        None
    }

    fn settled_result(&self) -> Option<FirRef> {
        // ConcatBrane IS its own value — no separate result child.
        None
    }

    fn is_brane_like(&self) -> bool {
        true
    }

    fn _search_brane(
        &self,
        expression: &str,
        starting_index: usize,
        ending_index: usize,
    ) -> Option<(usize, FirRef, Nyes)> {
        if !self._helpers_populated.get() {
            return None;
        }
        let total = self.stmt_count().unwrap_or(0);
        if starting_index >= total || ending_index >= total {
            return None;
        }
        let (from, to) = if starting_index >= ending_index {
            (ending_index, starting_index)
        } else {
            (starting_index, ending_index)
        };
        let mut offset = 0;
        for helper in self.core.ubc_children() {
            let count = helper.borrow().stmt_count().unwrap_or(0);
            let helper_end = offset + count;
            if from < helper_end {
                let local_start = from.saturating_sub(offset);
                let local_end = if to < helper_end {
                    to - offset
                } else {
                    count - 1
                };
                if let Some((local_idx, stmt, nyes)) =
                    helper
                        .borrow()
                        ._search_brane(expression, local_start, local_end)
                {
                    return Some((offset + local_idx, stmt, nyes));
                }
            }
            offset = helper_end;
            if offset > to {
                break;
            }
        }
        None
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
    Rc::new_cyclic(|me: &Weak<RefCell<StatementFir>>| {
        let self_weak: Weak<RefCell<dyn Fir>> = me.clone();
        RefCell::new(StatementFir {
            core: ProtoBrane::new(vec![body], parent, Nyes::Prembrionic),
            identifier: Identifier::from_parts(vec![], name),
            line_number,
            self_weak,
            nf_reason: RefCell::new(None),
        })
    })
}

pub fn brane(children: Vec<FirRef>, parent: Weak<RefCell<dyn Fir>>) -> FirRef {
    Rc::new(RefCell::new(BraneFir {
        core: ProtoBrane::new(children, parent, Nyes::Prembrionic),
        characterizations: Characterizations::default(),
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
        _helpers_populated: std::cell::Cell::new(false),
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

#[derive(Debug)]
pub struct CreationFir {
    pub(crate) core: ProtoBrane,
}

impl CreationFir {
    pub fn creation(parent: Weak<RefCell<dyn Fir>>) -> FirRef {
        Rc::new(RefCell::new(CreationFir {
            core: ProtoBrane::new(vec![], parent, Nyes::Independent),
        }))
    }

    /// The name this creation reports for itself, if any (FOOP-33).
    ///
    /// A creation names itself **only when it is the entire right-hand side of
    /// a named statement**. Concretely: its parent must be a statement, and
    /// this creation must be that statement's body. Then the statement's full
    /// characterized name is returned — `'a` for a null-characterized name,
    /// leading quote included, exactly as every name-search matches it.
    ///
    /// A creation sitting *inside* a larger expression reports `None`: in
    /// `'a = 1 + ⬤` the parent is the `+` operator, and the statement's name
    /// belongs to the whole `1 + ⬤` expression, not to the creation within it.
    ///
    /// This works even for a creation reached through a search from another
    /// brane. A constanic clone of an `Independent` creation returns the *same*
    /// `Rc` (FOOP-33 "Gotcha #2"), so the parent chain set at original
    /// construction survives detachment and recoordination at the reference
    /// site — the creation still finds its own defining statement. Creation
    /// identity is pointer identity, so the body check uses `Rc::ptr_eq`.
    ///
    /// `self_ref` must be the `FirRef` wrapping `self`; the parent is reached
    /// through it, following the same convention as [`Fir::_get_my_brane`].
    ///
    /// **CAVEAT (FOOP-33.md §"Concerns Standing Past Completion"):** this rule
    /// applies uniformly at the DEFINING site too, which reads oddly to a
    /// human even though it is the design working as specified. `{a = {*};}`
    /// sequences as `{a=a}`, not `{a={*}}`/`{a=⬤}` — the statement's own name
    /// is reported for its own creation, which looks self-referential rather
    /// than "here is a fresh, still-glyph creation being introduced." Whether
    /// the defining site should be special-cased to show the glyph (only
    /// *references reached elsewhere* would then show the name) is an open
    /// design question, not resolved by this implementation — see the FOOP-33
    /// doc section for the full writeup and status.
    #[must_use]
    pub fn get_display_name(&self, self_ref: &FirRef) -> Option<String> {
        let parent = self.core.parent()?;
        // A self-parenting node is the root; it has no defining statement.
        if Rc::ptr_eq(&parent, self_ref) {
            return None;
        }
        let parent = parent.borrow();
        // `as_stmt_searchable_name` is `None` for every non-statement kind, so
        // it doubles as the "is this a statement?" discriminator.
        let name = parent.as_stmt_searchable_name()?;
        let body = parent.core().foolish_children().first()?;
        Rc::ptr_eq(body, self_ref).then(|| name.to_owned())
    }
}

impl Fir for CreationFir {
    fn core(&self) -> &ProtoBrane {
        &self.core
    }
    fn fir_op_step(&self, _scope: &Scope) -> Result<(), UbcError> {
        Ok(())
    }
    fn kind(&self) -> FirKind {
        FirKind::Creation
    }
    fn as_creation_display_name(&self, self_ref: &FirRef) -> Option<String> {
        self.get_display_name(self_ref)
    }
}

pub fn creation(parent: Weak<RefCell<dyn Fir>>) -> FirRef {
    CreationFir::creation(parent)
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
            let self_weak: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(StatementFir {
                core: ProtoBrane::new(vec![body], parent, Nyes::Prembrionic),
                identifier: Identifier::from_parts(vec![], name),
                line_number,
                self_weak,
                nf_reason: RefCell::new(None),
            })
        })
    }

    fn make_brane(children: Vec<FirRef>) -> FirRef {
        Rc::new_cyclic(|me: &Weak<RefCell<BraneFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(BraneFir {
                core: ProtoBrane::new(children, parent, Nyes::Prembrionic),
                characterizations: Characterizations::default(),
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
                _helpers_populated: std::cell::Cell::new(false),
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
        assert_eq!(ubc[0].borrow().kind(), FirKind::ConcatHelper);
        let helper = &ubc[0];
        assert_eq!(helper.borrow().core().foolish_children().len(), 2);
    }

    #[test]
    fn concatenation_empty_elements_is_constant_empty_brane() {
        let cat = make_concatenation(vec![]);
        let scope = Scope::empty();

        let transitions = step_to_settled(&cat, &scope);
        eprintln!("Concatenation(empty) NYES transitions: {transitions:?}");

        assert_eq!(cat.borrow().core().get_nyes(), Nyes::Constant);
        let ubc = cat.borrow().core().ubc_children().to_vec();
        assert_eq!(ubc.len(), 0);
        assert_eq!(cat.borrow().stmt_count(), Some(0));
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
        assert_eq!(ubc[0].borrow().kind(), FirKind::ConcatHelper);
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

        let cloned = ProtoBrane::constanic_clone_at(&op, &dangling_parent(), 0, false, false);

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

        let cloned = ProtoBrane::constanic_clone_at(&op, &dangling_parent(), 0, false, false);

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
        let cloned = ProtoBrane::constanic_clone_at(&woc, &dangling_parent(), 0, true, false);
        assert_eq!(
            cloned.borrow().core().get_nyes(),
            Nyes::Woconstanic,
            "FICC must keep a constanic compound's state verbatim"
        );

        let econ = make_operator("+", vec![make_constant_int(1), make_constant_int(2)]);
        econ.borrow().core().set_nyes(Nyes::Econstanic);
        let cloned = ProtoBrane::constanic_clone_at(&econ, &dangling_parent(), 0, true, false);
        assert_eq!(cloned.borrow().core().get_nyes(), Nyes::Econstanic);
    }

    #[test]
    fn leaf_clone_unchanged_both_modes() {
        let ci = make_constant_int(9);
        ci.borrow().core().set_nyes(Nyes::Constant);
        let n = ProtoBrane::constanic_clone_at(&ci, &dangling_parent(), 0, false, false);
        assert_eq!(n.borrow().core().get_nyes(), Nyes::Constant);
        let n = ProtoBrane::constanic_clone_at(&ci, &dangling_parent(), 0, true, false);
        assert_eq!(n.borrow().core().get_nyes(), Nyes::Constant);

        let nk = make_nk("gone");
        nk.borrow().core().set_nyes(Nyes::Nk);
        let n = ProtoBrane::constanic_clone_at(&nk, &dangling_parent(), 0, false, false);
        assert_eq!(n.borrow().core().get_nyes(), Nyes::Nk);
        let n = ProtoBrane::constanic_clone_at(&nk, &dangling_parent(), 0, true, false);
        assert_eq!(n.borrow().core().get_nyes(), Nyes::Nk);
    }

    #[test]
    fn cloning_sf_strips_the_mark() {
        let inner = make_constant_int(10);
        inner.borrow().core().set_nyes(Nyes::Econstanic); // force the clone path
        let sf = make_stay_foolish(Rc::clone(&inner));
        sf.borrow().core().set_nyes(Nyes::Econstanic);

        let normal = ProtoBrane::constanic_clone_at(&sf, &dangling_parent(), 0, false, false);
        assert_ne!(
            normal.borrow().kind(),
            FirKind::StayFoolish,
            "normal clone of an SF must NOT be a StayFoolish wrapper"
        );
        assert_eq!(normal.borrow().kind(), FirKind::IndepInt);

        let foolish = ProtoBrane::constanic_clone_at(&sf, &dangling_parent(), 0, true, false);
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
    fn brane_fir_reports_its_own_characterizations() {
        // Pins the BraneFir.characterizations: Vec<String> → Characterizations
        // migration (FOOP-33 Phase 1 leftover item): `as_brane_characterizations()`
        // must still return the raw, ordered components for a characterized brane
        // literal, exactly as before the migration.
        let root = Compiler::compile("{x = a'b'{y = 1;};}")
            .unwrap()
            .pop()
            .unwrap();
        let x_stmt = &root.borrow().core().foolish_children().to_vec()[0];
        let brane = x_stmt.borrow().core().foolish_children().to_vec()[0].clone();
        assert_eq!(brane.borrow().kind(), FirKind::Brane);
        assert_eq!(brane.borrow().as_brane_characterizations(), &["a", "b"]);
    }

    #[test]
    fn brane_fir_with_no_characterizations_reports_empty() {
        let root = Compiler::compile("{y = 1;}").unwrap().pop().unwrap();
        assert_eq!(root.borrow().kind(), FirKind::Brane);
        assert!(root.borrow().as_brane_characterizations().is_empty());
    }

    #[test]
    fn bare_unanchored_index_out_of_bounds_settles_nk() {
        // A BARE (un-SFF-marked) unanchored index RUNS, misses, and settles
        // terminal NK -- correct behavior: it genuinely searched and genuinely
        // found nothing. IndexFir::fir_op_step's unanchored-Prembrionic arm
        // sets Nyes::Nk on an out-of-bounds target (`if target < 0 || target
        // >= len`).
        //
        // NOTE (2026-08-04): an earlier reading took this to mean FOOP-33
        // Phase 6's design was blocked, since 'lt's #-2/#-1 operand lookups
        // have no valid neighbors inside system.foo alone and NK is terminal
        // (never revived by recoordination). That conclusion was WRONG -- it
        // tested the wrong construct. Phase 6 specifies SFF-MARKED operands
        // (`<<#-1>>`), which never run at all and are built ECONSTANIC. See
        // the companion test `sff_marked_unanchored_index_out_of_bounds_
        // settles_econstanic` immediately below.
        let root = Compiler::compile("{only = #-1;}").unwrap().pop().unwrap();
        let scope = Scope::empty();
        let _ = step_to_settled(&root, &scope);
        let stmts = root.borrow().core().foolish_children().to_vec();
        let only_body = stmts[0].borrow().core().foolish_children()[0].clone();
        assert_eq!(only_body.borrow().kind(), FirKind::Index);
        assert_eq!(
            only_body.borrow().core().get_nyes(),
            Nyes::Nk,
            "a BARE out-of-bounds unanchored index runs, misses, and settles NK"
        );
    }

    #[test]
    fn sff_marked_unanchored_index_out_of_bounds_settles_econstanic() {
        // The construct FOOP-33 Phase 6 actually specifies: an SFF-marked
        // unanchored index. `compiler::build_fir`'s `under_sff` rule builds
        // descendant search kinds ECONSTANIC so they NEVER RUN -- so unlike
        // the bare form above, there is no miss and no NK. ECONSTANIC is
        // precisely "not evaluated in this context, may gain a value via
        // recoordination", which is the state Phase 6's design depends on:
        // 'lt's #-2/#-1 operands sit ECONSTANIC inside system.foo (no valid
        // neighbors there), then resolve against real neighbors once the
        // reference is detached and recoordinated into the user's brane.
        //
        // This pins that Phase 6's mechanism is sound as designed.
        let root = Compiler::compile("{only = <<#-1>>;}")
            .unwrap()
            .pop()
            .unwrap();
        let scope = Scope::empty();
        let _ = step_to_settled(&root, &scope);

        // Walk to the index inside the SFF wrapper.
        let stmts = root.borrow().core().foolish_children().to_vec();
        let sff_body = stmts[0].borrow().core().foolish_children()[0].clone();
        assert_eq!(sff_body.borrow().kind(), FirKind::StayFullyFoolish);
        let index = sff_body.borrow().core().foolish_children()[0].clone();
        assert_eq!(index.borrow().kind(), FirKind::Index);
        assert_eq!(
            index.borrow().core().get_nyes(),
            Nyes::Econstanic,
            "an SFF-marked out-of-bounds unanchored index is built ECONSTANIC \
             and never runs -- it can still gain a value via recoordination"
        );
    }

    /// The exact mechanism FOOP-33 Phase 6's comparison operators rest on,
    /// pinned in pure Foolish with no comparison machinery involved.
    ///
    /// A statement whose body is an SFF-marked unanchored index sits
    /// ECONSTANIC where it is defined (no valid neighbor there). When that
    /// statement is REFERENCED by name from another brane, the ordinary
    /// reference-resolution path detaches a constanic clone and RECOORDINATES
    /// it into the referencing brane -- where `#-2`/`#-1` now DO have
    /// neighbors, and resolve against them (AGENTS.md "Detachment and
    /// Coordination": "previously failed name searches can now resolve in the
    /// new context").
    ///
    /// This is what lets `'lt`'s operand lookups, defined inertly inside
    /// `system.foo`, pick up `1` and `2` when `'lt` is referenced from a user's
    /// `{1, 2, 'lt}`. If this test breaks, the comparison operators lose the
    /// ground they stand on -- the failure is here, not in `system_foo.rs`.
    #[test]
    fn sff_index_operand_recoordinates_to_the_referencing_branes_neighbors() {
        for (offset, expected) in [("#-2", 5), ("#-1", 9)] {
            let source = format!("{{defn = <<{offset}>>; use = {{5, 9, defn}};}}");
            let root = Compiler::compile(&source).unwrap().pop().unwrap();
            let scope = Scope::empty();
            let _ = step_to_settled(&root, &scope);

            let stmts = root.borrow().core().foolish_children().to_vec();
            let use_brane = stmts[1].borrow().core().foolish_children()[0]
                .clone()
                .value();
            let referenced = use_brane.borrow().core().foolish_children()[2].clone();
            let body = referenced.borrow().core().foolish_children()[0].clone();

            assert_eq!(
                body.value().borrow().as_i64(),
                Some(expected),
                "<<{offset}>> defined elsewhere must resolve to the REFERENCING \
                 brane's neighbor after recoordination"
            );
        }
    }

    #[test]
    fn stmt_ib_search_finds_earlier_null_characterized_sibling_by_searchable_name() {
        // FOOP-33 Phase 4 precondition: the null-constant rule's ancestral-conflict
        // check (BraneFir's own step) needs to find a same-name null-characterized
        // statement either earlier in the SAME brane (IB) or in an ancestor brane
        // (AB), using each statement's own FirRef (which BraneFir already holds
        // via foolish_children()) -- no self_ref parameter is needed or available
        // inside fir_op_step. Pins that `stmt._ib_search(&stmt, "'k")` / `_ab_search`
        // (the default Fir trait methods, called directly on a statement's own
        // FirRef) do exactly this, matching only the null-characterized searchable
        // name (not a plain name).
        let root = Compiler::compile("{'k=1; 'k=2;}").unwrap().pop().unwrap();
        let stmts = root.borrow().core().foolish_children().to_vec();
        let first_k = Rc::clone(&stmts[0]);
        let second_k = Rc::clone(&stmts[1]);
        let scope = Scope::empty();
        // Settle the bodies first (constanic gate).
        let _ = step_to_settled(&root, &scope);

        // second_k should find first_k via its own IB search for the exact
        // searchable_name pattern "'k" (matches only null-characterized 'k,
        // not a plain k).
        let ib_hit = second_k.borrow()._ib_search(&second_k, "'k");
        eprintln!("second_k IB search for \"'k\": {:?}", ib_hit.is_some());
        assert!(
            ib_hit.is_some(),
            "same-brane IB search must find the earlier 'k statement"
        );
        let (found_stmt, _found_nyes) = ib_hit.unwrap();
        assert!(
            Rc::ptr_eq(&found_stmt, &first_k),
            "IB search must find the FIRST 'k statement specifically"
        );

        // first_k has nothing before it -> IB search finds nothing, AB search
        // (climbing to the enclosing statement, of which there is none at root)
        // also finds nothing. This is the "no prior definition" case.
        let first_ib = first_k.borrow()._ib_search(&first_k, "'k");
        let first_ab = first_k.borrow()._ab_search(&first_k, "'k");
        eprintln!(
            "first_k IB={:?} AB={:?}",
            first_ib.is_some(),
            first_ab.is_some()
        );
        assert!(
            first_ib.is_none() && first_ab.is_none(),
            "the first 'k statement has no prior definition via IB or AB"
        );
    }

    #[test]
    fn stmt_ab_search_finds_ancestral_null_characterized_definition() {
        // Cross-brane case: {'k=1; b={'k=2;};} -- inner 'k has no earlier
        // statement in ITS OWN brane (IB), but must find the outer 'k via AB.
        // Companion to the IB case above.
        let root = Compiler::compile("{'k=1; b={'k=2;};}")
            .unwrap()
            .pop()
            .unwrap();
        let stmts = root.borrow().core().foolish_children().to_vec();
        let b_stmt = &stmts[1];
        let inner_brane = b_stmt
            .borrow()
            .core()
            .foolish_children()
            .first()
            .cloned()
            .unwrap();
        let inner_stmts = inner_brane.borrow().core().foolish_children().to_vec();
        let inner_k = Rc::clone(&inner_stmts[0]);
        let scope = Scope::empty();
        let _ = step_to_settled(&root, &scope);

        let ib = inner_k.borrow()._ib_search(&inner_k, "'k");
        eprintln!("inner_k IB for \"'k\": {:?}", ib.is_some());
        assert!(
            ib.is_none(),
            "inner 'k has no earlier statement in its OWN brane"
        );
        let ab = inner_k.borrow()._ab_search(&inner_k, "'k");
        eprintln!("inner_k AB for \"'k\": {:?}", ab.is_some());
        assert!(
            ab.is_some(),
            "inner 'k must find the outer 'k via ancestral (AB) search"
        );
        let outer_k = Rc::clone(&stmts[0]);
        assert!(Rc::ptr_eq(&ab.unwrap().0, &outer_k));
    }

    // --- FOOP-33 Phase 4: null-characterized name constants ---

    #[test]
    fn null_const_first_definition_is_permitted() {
        // A single 'k=1 establishes the constant -- no conflict, ordinary value.
        let root = Compiler::compile("{'k=1;}").unwrap().pop().unwrap();
        let scope = Scope::empty();
        let _ = step_to_settled(&root, &scope);
        let stmts = root.borrow().core().foolish_children().to_vec();
        assert!(
            stmts[0].borrow().settled_result().is_none(),
            "first definition of a null-const is never refused"
        );
        assert_eq!(stmts[0].borrow().core().get_nyes(), Nyes::Independent);
    }

    #[test]
    fn null_const_same_brane_conflicting_redefinition_settles_nf() {
        // {'k=1; 'k=2;} -- second 'k conflicts (1 != 2) -> NF.
        let root = Compiler::compile("{'k=1; 'k=2;}").unwrap().pop().unwrap();
        let scope = Scope::empty();
        let _ = step_to_settled(&root, &scope);
        let stmts = root.borrow().core().foolish_children().to_vec();

        assert!(
            stmts[0].borrow().settled_result().is_none(),
            "the FIRST 'k is never refused -- it establishes the constant"
        );

        let second_result = stmts[1]
            .borrow()
            .settled_result()
            .expect("second 'k must be refused (NF)");
        assert_eq!(second_result.borrow().kind(), FirKind::Nk);
        let reason = second_result.borrow().as_nk_reason().unwrap().to_owned();
        assert!(
            is_nf_reason(&reason),
            "reason must be an NF (not-foolish) condition, got: {reason}"
        );
        assert_eq!(reason, "'k not-foolish");
    }

    #[test]
    fn null_const_get_value_via_value_returns_the_nf_nk() {
        // The spec's own framing: get_value() (here, .value()) on the offending
        // statement's BODY yields the NF NK, not the written RHS `2`.
        let root = Compiler::compile("{'k=1; 'k=2;}").unwrap().pop().unwrap();
        let scope = Scope::empty();
        let _ = step_to_settled(&root, &scope);
        let stmts = root.borrow().core().foolish_children().to_vec();
        let second_value = stmts[1].value();
        assert_eq!(second_value.borrow().kind(), FirKind::Nk);
    }

    #[test]
    fn null_const_same_creation_redefinition_is_permitted() {
        // {c=⬤; 'k=c; 'k=c;} -- both 'k's resolve (via search) to the SAME
        // creation Rc -> default_equal Equal -> both permitted.
        let root = Compiler::compile("{c=⬤; 'k=c; 'k=c;}")
            .unwrap()
            .pop()
            .unwrap();
        let scope = Scope::empty();
        let _ = step_to_settled(&root, &scope);
        let stmts = root.borrow().core().foolish_children().to_vec();
        assert!(
            stmts[1].borrow().settled_result().is_none(),
            "first 'k=c is never refused"
        );
        assert!(
            stmts[2].borrow().settled_result().is_none(),
            "second 'k=c must be PERMITTED: same creation, default_equal == Equal"
        );
    }

    #[test]
    fn null_const_ancestral_conflict_via_ab_search() {
        // {'k=1; b={'k=2;};} -- inner 'k conflicts with the outer ancestor.
        let root = Compiler::compile("{'k=1; b={'k=2;};}")
            .unwrap()
            .pop()
            .unwrap();
        let scope = Scope::empty();
        let _ = step_to_settled(&root, &scope);
        let stmts = root.borrow().core().foolish_children().to_vec();
        let b_stmt = &stmts[1];
        let inner_brane = b_stmt
            .borrow()
            .core()
            .foolish_children()
            .first()
            .cloned()
            .unwrap();
        let inner_k = inner_brane.borrow().core().foolish_children().to_vec()[0].clone();
        let reason = inner_k
            .borrow()
            .settled_result()
            .expect("inner 'k must be refused via ancestral conflict")
            .borrow()
            .as_nk_reason()
            .unwrap()
            .to_owned();
        assert_eq!(reason, "'k not-foolish");
    }

    #[test]
    fn null_const_poison_scope_sibling_brane_unaffected() {
        // A sibling brane that resolves the SAME plain name differently (or not
        // at all) must be completely unaffected by a conflict elsewhere. Two
        // independent 'k definitions in UNRELATED branches (neither is an
        // ancestor of the other) must NOT conflict with each other.
        let root = Compiler::compile("{a={'k=1;}; b={'k=2;};}")
            .unwrap()
            .pop()
            .unwrap();
        let scope = Scope::empty();
        let _ = step_to_settled(&root, &scope);
        let stmts = root.borrow().core().foolish_children().to_vec();
        let a_brane = stmts[0]
            .borrow()
            .core()
            .foolish_children()
            .first()
            .cloned()
            .unwrap();
        let b_brane = stmts[1]
            .borrow()
            .core()
            .foolish_children()
            .first()
            .cloned()
            .unwrap();
        let a_k = a_brane.borrow().core().foolish_children().to_vec()[0].clone();
        let b_k = b_brane.borrow().core().foolish_children().to_vec()[0].clone();
        assert!(
            a_k.borrow().settled_result().is_none(),
            "a's 'k=1 is a fresh definition in an unrelated brane -- not poisoned"
        );
        assert!(
            b_k.borrow().settled_result().is_none(),
            "b's 'k=2 is a fresh definition in an unrelated (sibling) brane -- not poisoned"
        );
    }

    #[test]
    fn null_const_descendant_query_true_for_ancestor_null_const_false_otherwise() {
        // "Is this name a null-characterized coordinate name (a constant) here?"
        // is answered by is_nully_characterizing_coordinate_name() on whatever
        // statement a search finds -- true for 'k, false for a plain k.
        let root = Compiler::compile("{'k=1; m=2;}").unwrap().pop().unwrap();
        let stmts = root.borrow().core().foolish_children().to_vec();
        let k_is_nully = stmts[0]
            .borrow()
            .as_stmt_identifier()
            .unwrap()
            .is_nully_characterizing_coordinate_name();
        let m_is_nully = stmts[1]
            .borrow()
            .as_stmt_identifier()
            .unwrap()
            .is_nully_characterizing_coordinate_name();
        assert!(k_is_nully, "'k IS a null-characterized coordinate name");
        assert!(
            !m_is_nully,
            "plain m is NOT a null-characterized coordinate name"
        );
    }

    #[test]
    fn null_const_rule_does_not_fire_on_plain_names() {
        // Regression guard: k=1; k=2 (no leading ') must NOT be refused -- the
        // rule only fires on null-characterized coordinate names.
        let root = Compiler::compile("{k=1; k=2;}").unwrap().pop().unwrap();
        let scope = Scope::empty();
        let _ = step_to_settled(&root, &scope);
        let stmts = root.borrow().core().foolish_children().to_vec();
        assert!(
            stmts[0].borrow().settled_result().is_none(),
            "plain k=1 must never be refused by the null-const rule"
        );
        assert!(
            stmts[1].borrow().settled_result().is_none(),
            "plain k=2 must never be refused by the null-const rule"
        );
        let k2_body = stmts[1].borrow().core().foolish_children()[0].clone();
        assert_eq!(k2_body.value().borrow().as_i64(), Some(2));
    }

    #[test]
    fn null_const_concatenation_collision_later_duplicates_settle_nf() {
        // {A={'a=1;}; B = A A A;} -- the merged B is {'a=1, 'a=1(NF), 'a=1(NF)}:
        // the first 'a establishes the constant, each LATER 'a is a conflicting
        // redefinition (integer 1 vs the SAME creation... no, here they're both
        // literally `1`, but each is a FRESH clone from a FRESH A -- and 1 == 1
        // by default_equal (same integer value) -- so this specific case must
        // actually be PERMITTED (Equal), not NF. Use distinct values instead.
        let root = Compiler::compile("{A={'a=1;}; B = A A A;}")
            .unwrap()
            .pop()
            .unwrap();
        let scope = Scope::empty();
        let _ = step_to_settled(&root, &scope);
        let stmts = root.borrow().core().foolish_children().to_vec();
        let b_body = stmts[1].borrow().core().foolish_children()[0].clone();
        let b_value = b_body.value();
        assert_eq!(b_value.borrow().stmt_count(), Some(3));
        for i in 0..3 {
            let merged_a = b_value.borrow().stmt_at(i).unwrap();
            assert!(
                merged_a.borrow().settled_result().is_none(),
                "merged 'a[{i}]=1 must be PERMITTED -- every clone is the same \
                 integer 1, and default_equal(1,1) == Equal"
            );
        }
    }

    #[test]
    fn null_const_concatenation_collision_with_conflicting_values_settles_nf() {
        // Concatenating THREE DIFFERENT branes, each defining 'a to a
        // DIFFERENT value: the first 'a=1 establishes the constant; the
        // second 'a=2 and third 'a=3 both conflict and settle NF.
        //
        // NOTE: step_to_settled's 50-iteration cap is insufficient for this
        // shape (4 top-level statements, 3-way concatenation, each nested
        // brane needing its own settle pass) -- confirmed by tracing: with
        // only 50 steps the third concat element (Z) is still Braning when
        // the helper gives up, silently merging only 2 of 3 statements. This
        // is a pre-existing property of the shared step_to_settled test
        // helper (unrelated to the null-const rule itself), so step directly
        // with a larger budget here rather than changing the shared helper.
        let root = Compiler::compile("{X={'a=1;}; Y={'a=2;}; Z={'a=3;}; B = X Y Z;}")
            .unwrap()
            .pop()
            .unwrap();
        let scope = Scope::empty();
        for _ in 0..300 {
            let _ = root.step(&scope);
        }
        let stmts = root.borrow().core().foolish_children().to_vec();
        let b_body = stmts[3].borrow().core().foolish_children()[0].clone();
        let b_value = b_body.value();
        assert_eq!(b_value.borrow().stmt_count(), Some(3));

        let first_a = b_value.borrow().stmt_at(0).unwrap();
        assert!(
            first_a.borrow().settled_result().is_none(),
            "the FIRST merged 'a establishes the constant -- never refused"
        );

        let second_a = b_value.borrow().stmt_at(1).unwrap();
        let second_reason = second_a
            .borrow()
            .settled_result()
            .expect("second merged 'a=2 conflicts with 'a=1 -- must be NF")
            .borrow()
            .as_nk_reason()
            .unwrap()
            .to_owned();
        assert_eq!(second_reason, "'a not-foolish");

        let third_a = b_value.borrow().stmt_at(2).unwrap();
        let third_reason = third_a
            .borrow()
            .settled_result()
            .expect("third merged 'a=3 conflicts with the established constant -- must be NF")
            .borrow()
            .as_nk_reason()
            .unwrap()
            .to_owned();
        assert_eq!(third_reason, "'a not-foolish");
    }

    #[test]
    fn null_const_concatenation_same_creation_is_permitted_value_sensitive() {
        // {A={c=⬤; 'a=c;}; B = A A;} -- both merged 'a's resolve to the SAME
        // creation (both clones reference the SAME original c via search, and
        // constanic clone of an Independent creation preserves identity per
        // Gotcha #2) -- default_equal Equal -- both permitted. Proves the rule
        // is VALUE-sensitive, not "duplicate name = NF" by fiat.
        let root = Compiler::compile("{A={c=⬤; 'a=c;}; B = A A;}")
            .unwrap()
            .pop()
            .unwrap();
        let scope = Scope::empty();
        let _ = step_to_settled(&root, &scope);
        let stmts = root.borrow().core().foolish_children().to_vec();
        let b_body = stmts[1].borrow().core().foolish_children()[0].clone();
        let b_value = b_body.value();
        assert_eq!(b_value.borrow().stmt_count(), Some(4));
        // Statements: c, 'a, c, 'a (two merged copies of A's two statements).
        let first_a = b_value.borrow().stmt_at(1).unwrap();
        let second_a = b_value.borrow().stmt_at(3).unwrap();
        assert!(
            first_a.borrow().settled_result().is_none(),
            "first merged 'a=c is never refused"
        );
        assert!(
            second_a.borrow().settled_result().is_none(),
            "second merged 'a=c must be PERMITTED: same creation, default_equal == Equal"
        );
    }

    #[test]
    fn null_const_concatenation_empty_and_single_operand_merge_without_spurious_nf() {
        // Regression guard: an empty concatenation operand, or a single-operand
        // concatenation, must merge without any spurious NF -- the collision
        // check must not misfire when there's nothing (or only one thing) to
        // collide with.
        let root = Compiler::compile("{A={}; B={'a=1;}; C = A B;}")
            .unwrap()
            .pop()
            .unwrap();
        let scope = Scope::empty();
        let _ = step_to_settled(&root, &scope);
        let stmts = root.borrow().core().foolish_children().to_vec();
        let c_body = stmts[2].borrow().core().foolish_children()[0].clone();
        let c_value = c_body.value();
        assert_eq!(c_value.borrow().stmt_count(), Some(1));
        let merged_a = c_value.borrow().stmt_at(0).unwrap();
        assert!(
            merged_a.borrow().settled_result().is_none(),
            "single 'a merged from a concatenation with an empty operand must not be NF"
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
            stmts[0].borrow().as_stmt_searchable_name(),
            Some("a"),
            "named assignment keeps its LHS"
        );
        assert_eq!(
            stmts[1].borrow().as_stmt_searchable_name(),
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
    fn value_search_expr_pattern_woconstanic_value_is_woconstanic() {
        // FOOP-23 rendering appendix (2026-07-22): a value search whose value
        // EXPRESSION is WOCONSTANIC (here `c-d+v`, an Op+ waiting on the
        // ECONSTANIC search for `v`) must itself settle WOCONSTANIC — it is
        // "waiting on constanics" and may gain a value via recoordination, NOT a
        // miss. (Previously this wrongly collapsed to ECONSTANIC.)
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
            Nyes::Woconstanic,
            "value search with a WOCONSTANIC value expression settles WOCONSTANIC"
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
            .map(|(c, pos)| {
                (
                    c.borrow().as_stmt_searchable_name().unwrap().to_owned(),
                    pos,
                )
            })
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
            .map(|(c, pos)| {
                (
                    c.borrow().as_stmt_searchable_name().unwrap().to_owned(),
                    pos,
                )
            })
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
    fn value_search_pattern_referencing_a_creation_finds_matching_creation() {
        use crate::compiler::Compiler;

        // `diff` value-searches for w's (== y's) creation. `z` (== x's, a
        // DIFFERENT creation) sits between y/w and diff in scan order and
        // must be skipped; only w/y's creation may match.
        let root = Compiler::compile("{x = ⬤; y = {*}; z = x; w = y; diff = ?=w;}")
            .unwrap()
            .pop()
            .unwrap();
        settle_root(&root);

        let stmts = root.borrow().core().foolish_children().to_vec();
        let body_of = |i: usize| -> FirRef {
            stmts[i]
                .borrow()
                .core()
                .foolish_children()
                .first()
                .cloned()
                .unwrap()
        };
        let y_creation = body_of(1).value();
        let x_creation = body_of(0).value();
        assert_eq!(y_creation.borrow().kind(), FirKind::Creation);
        assert_eq!(x_creation.borrow().kind(), FirKind::Creation);
        assert!(
            !Rc::ptr_eq(&x_creation, &y_creation),
            "x and y must be distinct creations"
        );

        let diff_body = body_of(4);
        assert_eq!(
            diff_body.borrow().core().get_nyes(),
            Nyes::Constant,
            "diff must settle Constant, found the matching creation"
        );
        let diff_value = diff_body.value();
        assert!(
            Rc::ptr_eq(&diff_value, &y_creation),
            "diff must resolve to y's creation, not x's or any other"
        );
    }

    #[test]
    fn value_search_pattern_referencing_a_creation_rejects_distinct_creation() {
        use crate::compiler::Compiler;

        // `nomatch` is the ONLY statement in its home brane (`inner`), so
        // its unanchored backward scan has nothing to look at — not even a
        // self-match. `y`'s creation is a DIFFERENT creation than the
        // pattern would need, and it lives outside `inner` entirely, so it
        // is not even a candidate; the search must genuinely miss.
        let root = Compiler::compile("{y = {*}; inner = {nomatch = ?=y;};}")
            .unwrap()
            .pop()
            .unwrap();
        settle_root(&root);

        let stmts = root.borrow().core().foolish_children().to_vec();
        let inner_brane = stmts[1]
            .borrow()
            .core()
            .foolish_children()
            .first()
            .cloned()
            .unwrap();
        let inner_stmts = inner_brane.borrow().core().foolish_children().to_vec();
        let nomatch_body = inner_stmts[0]
            .borrow()
            .core()
            .foolish_children()
            .first()
            .cloned()
            .unwrap();
        // Unanchored miss -> ECONSTANIC (may still gain a value via
        // recoordination), not NK — see AGENTS.md "NK vs ECONSTANIC miss
        // outcomes".
        assert_eq!(
            nomatch_body.borrow().core().get_nyes(),
            Nyes::Econstanic,
            "nomatch must be an unanchored miss (ECONSTANIC), not a false match or NK"
        );
    }

    // --- default_equal truth table (FOOP-33 §2, Phase 3 gap) ---
    //
    // Direct tests of `default_equal` in isolation, one case per rule. These
    // complement the indirect matcher-level coverage below (which exercises
    // default_equal through SearchPredicate::Value/NameValue) by pinning the
    // primitive's own three-valued outcomes without going through search.

    #[test]
    fn default_equal_same_integer_value_is_equal() {
        let a = settled_int(7);
        let b = settled_int(7);
        assert_eq!(default_equal(&a, &b), Equality::Equal);
    }

    #[test]
    fn default_equal_different_integers_is_not_equal() {
        let a = settled_int(7);
        let b = settled_int(8);
        assert_eq!(default_equal(&a, &b), Equality::NotEqual);
    }

    #[test]
    fn default_equal_same_creation_rc_is_equal() {
        let parent = make_brane(vec![]);
        let creation = CreationFir::creation(Rc::downgrade(&parent));
        // `x = ⬤; y = x` resolves both to the SAME Rc via constanic clone
        // (Gotcha #2) — model that directly here without going through the
        // compiler, since default_equal only cares about the settled Rc.
        assert_eq!(default_equal(&creation, &creation), Equality::Equal);
    }

    #[test]
    fn default_equal_distinct_creations_is_not_equal() {
        let parent = make_brane(vec![]);
        let a = CreationFir::creation(Rc::downgrade(&parent));
        let b = CreationFir::creation(Rc::downgrade(&parent));
        assert_eq!(default_equal(&a, &b), Equality::NotEqual);
    }

    #[test]
    fn default_equal_either_operand_nk_is_unknowable() {
        let nk = settled_nk("unbound");
        let int_val = settled_int(1);
        assert_eq!(
            default_equal(&nk, &int_val),
            Equality::Unknowable,
            "NK vs integer: unknowable, not merely not-equal"
        );
        assert_eq!(
            default_equal(&int_val, &nk),
            Equality::Unknowable,
            "argument order must not matter"
        );
    }

    #[test]
    fn default_equal_same_nk_rc_is_still_unknowable() {
        // NKs are never equal to each other, even the exact same Rc (FOOP-23):
        // the NK guard fires before any identity check.
        let nk = settled_nk("unbound");
        assert_eq!(default_equal(&nk, &nk), Equality::Unknowable);
    }

    #[test]
    fn default_equal_creation_vs_integer_is_not_equal() {
        // Every integer is itself a creation (human ruling 2026-08-03, plan
        // Phase 3): a NEW, distinct creation can never equal any integer —
        // decidably NotEqual, not Unknowable.
        let parent = make_brane(vec![]);
        let creation = CreationFir::creation(Rc::downgrade(&parent));
        let int_val = settled_int(1);
        assert_eq!(default_equal(&creation, &int_val), Equality::NotEqual);
        assert_eq!(default_equal(&int_val, &creation), Equality::NotEqual);
    }

    #[test]
    fn default_equal_brane_vs_integer_is_not_equal() {
        // A settled brane is provably never an integer (different FIR kinds,
        // decidable) — NotEqual, matcher Rejects (skips) rather than NkStops.
        let brane = make_brane(vec![]);
        let _ = step_to_settled(&brane, &Scope::empty());
        let int_val = settled_int(1);
        assert_eq!(default_equal(&brane, &int_val), Equality::NotEqual);
        assert_eq!(default_equal(&int_val, &brane), Equality::NotEqual);
    }

    #[test]
    fn default_equal_two_branes_is_unknowable() {
        // Brane-vs-brane equivalence is unspecified (FOOP-23) — genuinely
        // unknowable, unlike the provably-different-kinds cases above.
        let brane_a = make_brane(vec![]);
        let brane_b = make_brane(vec![]);
        let _ = step_to_settled(&brane_a, &Scope::empty());
        let _ = step_to_settled(&brane_b, &Scope::empty());
        assert_eq!(default_equal(&brane_a, &brane_b), Equality::Unknowable);
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
            "brane-vs-integer is NotEqual → Reject (skip)"
        );
    }

    #[test]
    fn matcher_value_with_embryonic_body_with_value() {
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
        assert_eq!(
            pred.matches(&stmt, &ctx),
            MatchOutcome::Approve,
            "embryonic body with as_i64() matches via default_equal"
        );
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
    fn matcher_namevalue_with_nye_body_with_value() {
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
        assert_eq!(
            pred.matches(&stmt, &ctx),
            MatchOutcome::Approve,
            "pre-constanic body with value matches via default_equal"
        );
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
                assert_eq!(stmt.borrow().as_stmt_searchable_name(), Some("γ"));
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
                    stmt.borrow().as_stmt_searchable_name(),
                    Some("ᚠ"),
                    "backward scan must find ᚠ even though it is at brane position 0"
                );
            }
            other => panic!("expected Found, got {other:?}"),
        }
    }

    #[test]
    fn scan_embryonic_candidate_with_value() {
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

        let outcome = super::contextful_search_scan(&mut nav, &pred);
        match outcome {
            ScanOutcome::Found(stmt) => {
                assert_eq!(stmt.borrow().as_stmt_searchable_name(), Some("b"));
            }
            other => panic!("expected Found, got {other:?}"),
        }
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
                assert_eq!(stmt.borrow().as_stmt_searchable_name(), Some("setting"));
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
                assert_eq!(stmt.borrow().as_stmt_searchable_name(), Some("ᚺ"));
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
                    stmt.borrow().as_stmt_searchable_name(),
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
                    stmt.borrow().as_stmt_searchable_name(),
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
        assert_eq!(original_x.borrow().as_stmt_searchable_name(), Some("x"));

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

    /// A candidate-generator computing a scan range from a statement's own
    /// index must treat "first/last statement" as an EMPTY range, not wrap
    /// onto the statement's own slot. Otherwise a self-referential search at
    /// index 0 (e.g. `{a = a + 1;}`) recurses forever: the search finds
    /// itself, clones its body (still self-searching), and repeats unbounded.
    ///
    /// Five call sites compute such a range; the four reachable from compiled
    /// Foolish source are exercised here at their boundary:
    /// 1. `_ib_search` (fir_kinds.rs, the `self_ref`-parent-walk path)
    /// 2. `SearchFir::ib_search_with_engine` (scope-cached runtime path — the
    ///    one `{a=a+1;}` actually steps through)
    /// 3. `SearchFir::ab_search_with_engine` (backward, ancestor climb)
    /// 4. `SearchFir::contexted_search_from_anchor` (both directions)
    /// 5. `SearchFir::value_search_step`'s backward value-search range
    ///    (its forward+unanchored counterpart is UNREACHABLE from any valid
    ///    Foolish program — see the note at the end of this test — so it is
    ///    not exercised)
    #[test]
    fn index_zero_boundary_does_not_self_reference_across_all_candidate_generators() {
        use crate::compiler::Compiler;

        // (1) + (2): _ib_search AND the live ib_search_with_engine runtime
        // path, exercised together via full stepping — the ORIGINAL bug.
        // `a` is index 0 of its own (only) brane; its self-search for "a"
        // must find nothing there, fall through (unanchored miss ->
        // ECONSTANIC/NK per the current miss-settlement rule), and the
        // program must settle rather than hang.
        {
            let root = Compiler::compile("{a = a + 1;}").unwrap().pop().unwrap();
            let scope = Scope::empty();
            let mut settled = false;
            for _ in 0..500 {
                if root.borrow().core().get_nyes().is_constanic() {
                    settled = true;
                    break;
                }
                let _ = root.step(&scope).unwrap();
            }
            assert!(
                settled,
                "(1)+(2) {{a = a + 1;}} must settle -- a's own self-search at \
                 index 0 must not find itself and loop forever"
            );

            // Direct _ib_search evidence too (not just the end-to-end settle).
            let stmts = root.borrow().core().foolish_children().to_vec();
            let a_stmt = &stmts[0];
            assert_eq!(a_stmt.borrow().as_stmt_line_number(), Some(0));
            let found = a_stmt.borrow()._ib_search(a_stmt, "a");
            assert!(
                found.is_none(),
                "(1) _ib_search at index 0 must not find self"
            );
        }

        // (3): ab_search_with_engine already guarded with `idx > 0` --
        // confirm a statement at index 0 of the ANCESTOR brane (the one
        // ab_search climbs into) does not find itself when the inner
        // statement's own name matches the outer one.
        {
            // outer `x` is index 0 of the root brane; inner `y = x;` is
            // index 0 of `inner`'s brane, so ab_search climbs to the root
            // and must find outer `x` (index 0 there) WITHOUT the inner
            // statement (also effectively "at index 0" in its own brane)
            // ever mistaking itself for the ancestor's index-0 statement.
            let root = Compiler::compile("{x = 1; inner = {y = x;};}")
                .unwrap()
                .pop()
                .unwrap();
            settle_root(&root);
            let stmts = root.borrow().core().foolish_children().to_vec();
            let inner_stmt = &stmts[1];
            let inner_body = inner_stmt
                .borrow()
                .core()
                .foolish_children()
                .first()
                .cloned()
                .unwrap();
            let inner_stmts = inner_body.borrow().core().foolish_children().to_vec();
            let y_body = inner_stmts[0]
                .borrow()
                .core()
                .foolish_children()
                .first()
                .cloned()
                .unwrap();
            assert_eq!(
                y_body.borrow().as_i64(),
                Some(1),
                "(3) ab_search_with_engine: y = x must resolve to outer x = 1 \
                 (outer x is index 0 of the root brane)"
            );
        }

        // (4): contexted_search_from_anchor, backward direction, anchor at
        // index 0 of its brane (`p == 0` guard) -- `&?` from the FIRST
        // statement must find nothing backward, not wrap onto itself.
        {
            let root = Compiler::compile("{blk = {first = 1; second = 2;}; r = blk.first&?first;}")
                .unwrap()
                .pop()
                .unwrap();
            settle_root(&root);
            let stmts = root.borrow().core().foolish_children().to_vec();
            let r_stmt = &stmts[1];
            let r_body = r_stmt
                .borrow()
                .core()
                .foolish_children()
                .first()
                .cloned()
                .unwrap();
            assert_eq!(
                r_body.borrow().core().get_nyes(),
                Nyes::Nk,
                "(4) contexted_search_from_anchor backward from index-0 anchor \
                 must NOT find itself (should be NK: anchored miss, nothing \
                 precedes the first statement)"
            );
        }

        // (5a): value_search_step backward range, statement at index 0 --
        // `idx > 0` guard. A value-search AT index 0 must not find itself
        // even when its own (still-unsettled) value could coincidentally
        // match its own pattern.
        {
            let root = Compiler::compile("{found9 = ?=9; other = 9;}")
                .unwrap()
                .pop()
                .unwrap();
            settle_root(&root);
            let stmts = root.borrow().core().foolish_children().to_vec();
            let found9_stmt = &stmts[0];
            assert_eq!(found9_stmt.borrow().as_stmt_line_number(), Some(0));
            let found9_body = found9_stmt
                .borrow()
                .core()
                .foolish_children()
                .first()
                .cloned()
                .unwrap();
            assert_eq!(
                found9_body.borrow().core().get_nyes(),
                Nyes::Econstanic,
                "(5a) value_search_step backward from index 0: found9 = ?=9 \
                 must NOT find itself -- no preceding statement has value 9, \
                 so it settles ECONSTANIC (unanchored value-search miss)"
            );
        }

        // NOTE: the sibling forward+unanchored value-search range at
        // fir_kinds.rs (`idx + 1 < len` guard, inside the `!self.anchored`
        // branch of `value_search_step`) is NOT exercised here. It is
        // unreachable from any valid Foolish source: every `forward: true`
        // SearchFir the parser produces has `anchor: Some(...)` (postfix
        // continuations `a~pattern`, `a~=value`) -- there is no primary-
        // position unanchored-forward token. This matches the documented
        // language rule "no unanchored forward form" (AGENTS.md / FOOP-23
        // §Searches: "Foolish cannot look forward in its own brane").
        // Confirmed by inspection (Atlas, FOOP-13 triage) rather than tested,
        // since testing it requires hand-building a SearchFir via struct
        // literal for a shape no compiled program can ever produce.
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

    fn make_concat_helper(children: Vec<FirRef>) -> FirRef {
        Rc::new_cyclic(|me: &Weak<RefCell<ConcatHelper>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(ConcatHelper {
                core: ProtoBrane::new(children, parent, Nyes::Prembrionic),
            })
        })
    }

    fn make_10_stmt_concat() -> FirRef {
        let mut brane1_stmts = Vec::new();
        let mut brane2_stmts = Vec::new();
        for i in 0..5 {
            let name = format!("{}", (b'a' + i as u8) as char);
            let val = make_constant_int(i as i64 + 1);
            let stmt = make_statement(&name, i, Rc::clone(&val));
            brane1_stmts.push(stmt);
        }
        for i in 5..10 {
            let name = format!("{}", (b'a' + i as u8) as char);
            let val = make_constant_int(i as i64 + 1);
            let stmt = make_statement(&name, i, Rc::clone(&val));
            brane2_stmts.push(stmt);
        }
        let brane1 = make_brane(brane1_stmts);
        let brane2 = make_brane(brane2_stmts);
        make_concatenation(vec![brane1, brane2])
    }

    // ── Equivalence Law and search ─────────────────────────────────────

    #[test]
    fn concat_equals_big_brane() {
        let big = Compiler::compile(
            "{a = 1; b = 2; c = 3; d = 4; e = 5; \
             f = 6; g = 7; h = 8; i = 9; j = 10;}",
        )
        .unwrap()
        .pop()
        .unwrap();
        let cat = Compiler::compile(
            "{a = 1; b = 2; c = 3; d = 4; e = 5; \
             f = 6; g = 7; h = 8; i = 9; j = 10;}",
        )
        .unwrap()
        .pop()
        .unwrap();

        settle_root(&big);
        settle_root(&cat);

        let big_count = big.borrow().stmt_count().unwrap();
        let cat_count = cat.borrow().stmt_count().unwrap();
        assert_eq!(big_count, cat_count, "both must have same stmt_count");

        for i in 0..big_count {
            let bs = big.borrow().stmt_at(i).unwrap();
            let cs = cat.borrow().stmt_at(i).unwrap();
            let b_name = bs.borrow().as_stmt_searchable_name().unwrap().to_owned();
            let c_name = cs.borrow().as_stmt_searchable_name().unwrap().to_owned();
            assert_eq!(b_name, c_name, "stmt[{i}] name mismatch");
            let b_val = bs.borrow().core().foolish_children()[0].borrow().as_i64();
            let c_val = cs.borrow().core().foolish_children()[0].borrow().as_i64();
            assert_eq!(b_val, c_val, "stmt[{i}] value mismatch");
        }
    }

    #[test]
    fn concat_search_brane_translates_global_indices() {
        let cat = make_10_stmt_concat();
        settle_root(&cat);
        assert_eq!(cat.borrow().kind(), FirKind::Concatenation);
        assert!(cat.borrow().core().get_nyes().is_constanic());

        let result = cat.borrow()._search_brane("^a$", 0, 9);
        assert!(result.is_some(), "must find 'a'");
        let (idx, stmt, _nyes) = result.unwrap();
        assert_eq!(idx, 0, "global index of 'a' must be 0");
        assert_eq!(stmt.borrow().as_stmt_searchable_name(), Some("a"));

        let result = cat.borrow()._search_brane("^f$", 0, 9);
        assert!(result.is_some(), "must find 'f'");
        let (idx, stmt, _nyes) = result.unwrap();
        assert_eq!(idx, 5, "global index of 'f' must be 5");
        assert_eq!(stmt.borrow().as_stmt_searchable_name(), Some("f"));

        let result = cat.borrow()._search_brane("^j$", 9, 0);
        assert!(result.is_some(), "must find 'j' in reverse");
        let (idx, stmt, _nyes) = result.unwrap();
        assert_eq!(idx, 9, "global index of 'j' must be 9");
        assert_eq!(stmt.borrow().as_stmt_searchable_name(), Some("j"));

        let result = cat.borrow()._search_brane("^e$", 0, 9);
        assert!(result.is_some(), "must find 'e'");
        let (idx, stmt, _nyes) = result.unwrap();
        assert_eq!(idx, 4, "global index of 'e' must be 4");
        assert_eq!(stmt.borrow().as_stmt_searchable_name(), Some("e"));
    }

    #[test]
    fn concat_ib_search_crosses_segments() {
        let root = Compiler::compile("{cb = {a = 10;}{b = a;};}")
            .unwrap()
            .pop()
            .unwrap();
        settle_root(&root);

        let stmts = root.borrow().core().foolish_children().to_vec();
        let cb_body = stmts[0].borrow().core().foolish_children()[0].value();
        assert!(cb_body.borrow().is_brane_like());

        let mut found_val = None;
        let count = cb_body.borrow().stmt_count().unwrap_or(0);
        for i in 0..count {
            let stmt = cb_body.borrow().stmt_at(i).unwrap();
            if stmt.borrow().as_stmt_searchable_name() == Some("b") {
                let body = stmt.borrow().core().foolish_children()[0].value();
                found_val = body.borrow().as_i64();
                break;
            }
        }
        assert_eq!(found_val, Some(10), "b must resolve a=10 across segments");
    }

    #[test]
    fn concat_ab_search_reaches_outward() {
        let root = Compiler::compile("{x = 99; cb = {a = 1; b = x;};}")
            .unwrap()
            .pop()
            .unwrap();
        settle_root(&root);

        let stmts = root.borrow().core().foolish_children().to_vec();
        let cb_body = stmts[1].borrow().core().foolish_children()[0].value();
        assert!(cb_body.borrow().is_brane_like(), "cb must be brane-like");

        let cb_count = cb_body.borrow().stmt_count().unwrap_or(0);
        let mut found_val = None;
        for i in 0..cb_count {
            let stmt = cb_body.borrow().stmt_at(i).unwrap();
            if stmt.borrow().as_stmt_searchable_name() == Some("b") {
                let body = stmt.borrow().core().foolish_children()[0].value();
                found_val = body.borrow().as_i64();
                break;
            }
        }
        assert_eq!(
            found_val,
            Some(99),
            "b must resolve x=99 from enclosing brane"
        );
    }

    #[test]
    fn concat_contexted_search_spans_segments() {
        let root = Compiler::compile(
            "{data = {a = 1; b = 2; c = 3; d = 4; e = 5; \
             f = 6; g = 7; h = 8; i = 9; j = 10;}; \
             found = data~f;}",
        )
        .unwrap()
        .pop()
        .unwrap();
        settle_root(&root);

        let stmts = root.borrow().core().foolish_children().to_vec();
        let found_stmt = &stmts[1];
        let found_body = found_stmt.borrow().core().foolish_children()[0].value();
        assert_eq!(
            found_body.borrow().as_i64(),
            Some(6),
            "data~f must find f=6 in ConcatBrane"
        );
    }

    // ── Indexing ───────────────────────────────────────────────────────

    #[test]
    fn concat_index_spans_segments() {
        let cat = make_10_stmt_concat();
        settle_root(&cat);

        let idx9 = make_index(9, true, vec![Rc::clone(&cat)]);
        let scope = Scope::empty();
        step_to_settled(&idx9, &scope);
        assert_eq!(
            idx9.borrow().core().get_nyes(),
            Nyes::Constant,
            "#9 into 10-stmt concat must be Constant"
        );
        let ubc = idx9.borrow().core().ubc_children().to_vec();
        assert_eq!(ubc[0].borrow().as_i64(), Some(10), "#9 must be j=10");

        let idx99 = make_index(99, true, vec![Rc::clone(&cat)]);
        step_to_settled(&idx99, &scope);
        assert_eq!(
            idx99.borrow().core().get_nyes(),
            Nyes::Nk,
            "#99 out of range must be NK"
        );
    }

    #[test]
    fn concat_find_stmt_index_is_global() {
        let cat = make_10_stmt_concat();
        settle_root(&cat);

        for i in 0..10 {
            let stmt = cat.borrow().stmt_at(i).unwrap();
            let found = cat.find_stmt_index(&stmt);
            assert_eq!(
                found,
                Some(i),
                "find_stmt_index must return global index {i}"
            );
        }
    }

    // ── Structure, value, and clone ────────────────────────────────────

    #[test]
    fn concat_statement_parents_point_at_concat_helper() {
        let cat = make_10_stmt_concat();
        settle_root(&cat);

        let helpers = cat.borrow().core().ubc_children().to_vec();
        assert_eq!(helpers.len(), 1, "unlimited k → single _ConcatHelper");
        let helper = &helpers[0];
        assert_eq!(helper.borrow().kind(), FirKind::ConcatHelper);
        assert_eq!(helper.borrow().stmt_count().unwrap(), 10);
    }

    #[test]
    fn concat_value_is_itself() {
        let cat = make_10_stmt_concat();
        settle_root(&cat);

        assert!(cat.borrow().core().get_nyes().is_constanic());

        assert!(
            cat.borrow().settled_result().is_none(),
            "ConcatBrane settled_result must be None"
        );
        let v = cat.value();
        assert!(
            Rc::ptr_eq(&v, &cat),
            "value() of ConcatBrane must return itself"
        );

        assert!(
            cat.borrow().as_i64().is_none(),
            "as_i64 on ConcatBrane must be None"
        );
    }

    #[test]
    fn concat_constanic_clone_rewires_and_recoordinates() {
        let cat = make_10_stmt_concat();
        settle_root(&cat);

        let dummy: Rc<RefCell<dyn Fir>> = Rc::new_cyclic(|me: &Weak<RefCell<IndepIntFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(IndepIntFir {
                core: ProtoBrane::new(vec![], parent, Nyes::Constant),
                value: 0,
            })
        });
        let new_parent = Rc::downgrade(&dummy);
        let clone = ProtoBrane::constanic_clone_at(&cat, &new_parent, 0, false, true);

        assert_eq!(clone.borrow().kind(), FirKind::Concatenation);
        assert!(clone.borrow().core().get_nyes().is_constanic());
        assert_eq!(
            clone.borrow().stmt_count(),
            cat.borrow().stmt_count(),
            "clone stmt_count must match original"
        );
        for i in 0..10 {
            let orig = cat.borrow().stmt_at(i).unwrap();
            let cloned = clone.borrow().stmt_at(i).unwrap();
            assert_eq!(
                orig.borrow().as_stmt_searchable_name(),
                cloned.borrow().as_stmt_searchable_name(),
                "stmt[{i}] name must match"
            );
        }
    }

    #[test]
    fn settled_search_clone_skips_foolish_children() {
        let brane = make_brane(vec![
            make_statement("a", 0, make_constant_int(1)),
            make_statement("b", 1, make_constant_int(2)),
        ]);
        settle_root(&brane);

        let dummy: Rc<RefCell<dyn Fir>> = Rc::new_cyclic(|me: &Weak<RefCell<IndepIntFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(IndepIntFir {
                core: ProtoBrane::new(vec![], parent, Nyes::Constant),
                value: 0,
            })
        });
        let new_parent = Rc::downgrade(&dummy);
        let clone = ProtoBrane::constanic_clone_at(&brane, &new_parent, 0, false, true);

        assert_eq!(clone.borrow().kind(), FirKind::Brane);
        assert!(
            clone.borrow().core().foolish_children().is_empty(),
            "skip_foolish_children must drop brane children"
        );
    }

    #[test]
    fn concat_arrangement_is_function_of_n_and_k() {
        let a = make_constant_int(1);
        let b = make_constant_int(2);
        let sa = make_statement("a", 0, Rc::clone(&a));
        let sb = make_statement("b", 1, Rc::clone(&b));
        let brane1 = make_brane(vec![Rc::clone(&sa), Rc::clone(&sb)]);

        let c = make_constant_int(3);
        let d = make_constant_int(4);
        let e = make_constant_int(5);
        let sc = make_statement("c", 0, Rc::clone(&c));
        let sd = make_statement("d", 1, Rc::clone(&d));
        let se = make_statement("e", 2, Rc::clone(&e));
        let inner_brane = make_brane(vec![Rc::clone(&sc), Rc::clone(&sd), Rc::clone(&se)]);
        let inner_cat = make_concatenation(vec![Rc::clone(&inner_brane)]);

        let cat = make_concatenation(vec![Rc::clone(&brane1), Rc::clone(&inner_cat)]);
        settle_root(&cat);

        assert!(cat.borrow().core().get_nyes().is_constanic());
        let helpers = cat.borrow().core().ubc_children().to_vec();
        assert_eq!(
            helpers.len(),
            1,
            "unlimited k must produce single _ConcatHelper"
        );

        assert_eq!(
            cat.borrow().stmt_count(),
            Some(5),
            "total must be 5 statements"
        );

        let expected = ["a", "b", "c", "d", "e"];
        for (i, name) in expected.iter().enumerate() {
            let stmt = cat.borrow().stmt_at(i).unwrap();
            assert_eq!(
                stmt.borrow().as_stmt_searchable_name(),
                Some(*name),
                "stmt[{i}] must be named {name}"
            );
        }
    }

    #[test]
    fn concatenation_of_empty_branes() {
        let empty1 = make_brane(vec![]);
        let empty2 = make_brane(vec![]);
        let cat = make_concatenation(vec![Rc::clone(&empty1), Rc::clone(&empty2)]);
        let scope = Scope::empty();
        step_to_settled(&cat, &scope);

        assert_eq!(cat.borrow().core().get_nyes(), Nyes::Constant);
        assert_eq!(
            cat.borrow().stmt_count(),
            Some(0),
            "empty branes → 0 statements"
        );
        assert!(
            cat.borrow().core().ubc_children().is_empty(),
            "no _ConcatHelpers for empty concat"
        );

        let val = make_constant_int(42);
        let stmt = make_statement("x", 0, Rc::clone(&val));
        let non_empty = make_brane(vec![Rc::clone(&stmt)]);
        let empty = make_brane(vec![]);
        let cat2 = make_concatenation(vec![Rc::clone(&non_empty), Rc::clone(&empty)]);
        step_to_settled(&cat2, &scope);

        assert_eq!(
            cat2.borrow().stmt_count(),
            Some(1),
            "one non-empty → 1 statement"
        );
        let first = cat2.borrow().stmt_at(0).unwrap();
        assert_eq!(first.borrow().as_stmt_searchable_name(), Some("x"));
    }

    // ── Protocol (element typing, auto-wrapping, copy-and-coordinate) ──

    #[test]
    fn concat_element_typing_rejects_non_brane() {
        let int_val = make_constant_int(99);
        let cat = make_concatenation(vec![Rc::clone(&int_val)]);
        let scope = Scope::empty();
        step_to_settled(&cat, &scope);

        assert_eq!(
            cat.borrow().core().get_nyes(),
            Nyes::Nk,
            "non-brane element must settle NK"
        );
    }

    /// All non-brane constituent indices are reported, not just the first.
    #[test]
    fn concat_reports_all_non_brane_indices() {
        let cat = make_concatenation(vec![
            make_constant_int(1), // 0: non-brane
            make_brane(vec![]),   // 1: brane
            make_constant_int(2), // 2: non-brane
        ]);
        step_to_settled(&cat, &Scope::empty());
        assert_eq!(cat.borrow().core().get_nyes(), Nyes::Nk);
        let nk_child = cat
            .borrow()
            .core()
            .ubc_children()
            .into_iter()
            .next()
            .unwrap();
        let reason = nk_child.borrow().as_nk_reason().unwrap_or("").to_string();
        assert!(
            reason.contains("0,2"),
            "reason must list both non-brane indices, got: {reason}"
        );
    }

    /// A WOCONSTANIC brane element still joins, and the ConcatBrane settles
    /// from the JOINED lines (all Constant here), not the element's own
    /// WOCONSTANIC state: `{c=a+b}` is WOCONSTANIC but its joined copy `c=3`
    /// is Constant.
    #[test]
    fn concat_joins_woconstanic_brane_element_and_settles_from_joined_lines() {
        let root = Compiler::compile("{b1 = {a=1, b=2}; nl = b1 {c = a + b};}")
            .unwrap()
            .pop()
            .unwrap();
        settle_root(&root);
        let stmts = root.borrow().core().foolish_children().to_vec();
        let nl = stmts
            .iter()
            .find(|s| s.borrow().as_stmt_searchable_name() == Some("nl"))
            .unwrap();
        let nl_body = nl
            .borrow()
            .core()
            .foolish_children()
            .first()
            .cloned()
            .unwrap();
        assert_eq!(
            nl_body.borrow().core().get_nyes(),
            Nyes::Constant,
            "joined lines are all constant → ConcatBrane is Constant"
        );
        assert_eq!(nl_body.borrow().stmt_count(), Some(3), "a, b, c joined");
    }

    #[test]
    fn concat_construction_auto_wraps() {
        let root = Compiler::compile("{{a = 1;} {b = 2;}}")
            .unwrap()
            .pop()
            .unwrap();
        settle_root(&root);

        let stmts = root.borrow().core().foolish_children().to_vec();
        let first_body = stmts[0].borrow().core().foolish_children()[0].clone();
        let kind = first_body.borrow().kind();
        assert_eq!(
            kind,
            FirKind::Concatenation,
            "juxtaposition must be Concatenation"
        );
        let elements = first_body.borrow().core().foolish_children().to_vec();
        assert_eq!(elements.len(), 2, "concat must have 2 elements");
    }

    #[test]
    fn concat_cross_element_reference_resolves() {
        let root = Compiler::compile("{cb = {a = 1; b = 2;}{c = a + b;};}")
            .unwrap()
            .pop()
            .unwrap();
        settle_root(&root);

        let stmts = root.borrow().core().foolish_children().to_vec();
        let mut cb_val: Option<FirRef> = None;
        for s in &stmts {
            if s.borrow().as_stmt_searchable_name() == Some("cb") {
                cb_val = Some(s.borrow().core().foolish_children()[0].value());
                break;
            }
        }
        let cb = cb_val.expect("must find cb");
        assert!(cb.borrow().is_brane_like(), "cb must be brane-like");

        let cb_count = cb.borrow().stmt_count().unwrap_or(0);
        let mut c_val = None;
        for i in 0..cb_count {
            let stmt = cb.borrow().stmt_at(i).unwrap();
            if stmt.borrow().as_stmt_searchable_name() == Some("c") {
                let body = stmt.borrow().core().foolish_children()[0].value();
                c_val = body.borrow().as_i64();
                break;
            }
        }
        assert_eq!(c_val, Some(3), "c = a + b must resolve to 3");
    }

    #[test]
    fn concat_sff_born_searches_revive_embryonic() {
        let root = Compiler::compile("{a = 42; b = a;}")
            .unwrap()
            .pop()
            .unwrap();
        settle_root(&root);

        let stmts = root.borrow().core().foolish_children().to_vec();
        let b_body = stmts[1].borrow().core().foolish_children()[0].value();
        assert_eq!(
            b_body.borrow().as_i64(),
            Some(42),
            "b=a must resolve in same brane"
        );
    }

    #[test]
    fn concat_sf_on_search_is_noop() {
        let r1 = Compiler::compile("{a = 1; b = 2;}").unwrap().pop().unwrap();
        let r2 = Compiler::compile("{a = 1; b = 2;}").unwrap().pop().unwrap();

        settle_root(&r1);
        settle_root(&r2);

        assert_eq!(
            r1.borrow().stmt_count(),
            r2.borrow().stmt_count(),
            "identical sources must produce same stmt_count"
        );
    }

    #[test]
    fn concat_sf_marked_literal_prepares_locally() {
        let root = Compiler::compile("{sf_brane = <{a = 1;}>; b = 2;}")
            .unwrap()
            .pop()
            .unwrap();
        settle_root(&root);

        let stmts = root.borrow().core().foolish_children().to_vec();
        let names: Vec<String> = stmts
            .iter()
            .filter_map(|s| s.borrow().as_stmt_searchable_name().map(|n| n.to_owned()))
            .collect();
        assert!(names.contains(&"b".to_string()), "must contain 'b'");
    }

    #[test]
    fn concat_explicit_sff_element_is_error() {
        let int_val = make_constant_int(99);
        let nk_elem = Rc::new(RefCell::new(NkFir {
            core: ProtoBrane::new(vec![], Rc::downgrade(&int_val), Nyes::Nk),
            reason: "invalid concatenation element".to_string(),
        }));
        let cat = make_concatenation(vec![nk_elem]);
        settle_root(&cat);

        assert_eq!(
            cat.borrow().core().get_nyes(),
            Nyes::Nk,
            "NK element → NK concat"
        );
    }

    // ── NYES transitions ───────────────────────────────────────────────

    #[test]
    fn concat_helper_nyes_transitions() {
        let stmt = make_statement("x", 0, make_constant_int(42));
        let helper = make_concat_helper(vec![stmt]);
        let trace = step_to_settled(&helper, &Scope::empty());
        assert_progression(&trace, Nyes::Constant, "ConcatHelper");
    }

    #[test]
    fn concatenation_nyes_transitions() {
        let brane1 = make_brane(vec![
            make_statement("a", 0, make_constant_int(1)),
            make_statement("b", 1, make_constant_int(2)),
        ]);
        let brane2 = make_brane(vec![
            make_statement("c", 0, make_constant_int(3)),
            make_statement("d", 1, make_constant_int(4)),
        ]);
        let cat = make_concatenation(vec![brane1, brane2]);
        let trace = step_to_settled(&cat, &Scope::empty());
        assert_progression(&trace, Nyes::Constant, "Concatenation(extended)");
        assert!(
            trace.contains(&Nyes::Braning),
            "extended concatenation must pass through Braning"
        );
    }

    /// An SFF wrapping an unfindable search: the inner search is ECONSTANIC,
    /// but the SFF wrapper itself must be WOCONSTANIC (it is not a search, so
    /// it can't be ECONSTANIC — it is waiting on one).
    #[test]
    fn sff_wrapper_is_woconstanic_when_inner_search_is_econstanic() {
        let root = Compiler::compile("{sff = <<nope>>;}")
            .unwrap()
            .pop()
            .unwrap();
        let scope = Scope::empty();
        for _ in 0..50 {
            if root.borrow().core().get_nyes().is_constanic() {
                break;
            }
            let _ = root.step(&scope).unwrap();
        }
        let stmts = root.borrow().core().foolish_children().to_vec();
        let sff = stmts[0]
            .borrow()
            .core()
            .foolish_children()
            .first()
            .cloned()
            .unwrap();
        let inner = sff
            .borrow()
            .core()
            .foolish_children()
            .first()
            .cloned()
            .unwrap();
        assert_eq!(
            sff.borrow().core().get_nyes(),
            Nyes::Woconstanic,
            "SFF wrapper must be WOCONSTANIC (it is not a search)"
        );
        assert_eq!(
            inner.borrow().core().get_nyes(),
            Nyes::Econstanic,
            "inner search stays ECONSTANIC"
        );
    }

    #[test]
    fn creation_nyes_transitions() {
        let parent = make_brane(vec![]);
        let creation = CreationFir::creation(Rc::downgrade(&parent));
        let trace = step_to_settled(&creation, &Scope::empty());
        assert!(trace.iter().all(|n| *n == Nyes::Independent));
        assert!(trace.first().unwrap().is_constanic());
    }

    #[test]
    fn creation_constanic_clone_preserves_identity() {
        let parent = make_brane(vec![]);
        let creation = CreationFir::creation(Rc::downgrade(&parent));
        let clone =
            ProtoBrane::constanic_clone_at(&creation, &Rc::downgrade(&parent), 0, false, false);
        assert!(
            Rc::ptr_eq(&creation, &clone),
            "constanic clone of CreationFir must return same Rc"
        );
        let creation2 = CreationFir::creation(Rc::downgrade(&parent));
        assert!(
            !Rc::ptr_eq(&creation, &creation2),
            "two distinct creations must not be ptr_eq"
        );
    }

    // ── CreationFir::get_display_name (FOOP-33 stretch goal) ──
    //
    // A creation reports a name ONLY when it is the ENTIRE right-hand side of a
    // named statement. These tests pin both halves of that rule, plus the
    // payoff case: a creation reached through a search still answers with its
    // OWN defining statement's name, because a constanic clone of an
    // `Independent` creation returns the SAME `Rc` (FOOP-33 "Gotcha #2"), so
    // its `.parent()` chain -- set at original construction -- survives
    // detachment and recoordination at the reference site.

    /// Rust-side tree walk (a *sift*, not a Foolish search -- see AGENTS.md
    /// §Foolish Terminology) finding the first `Creation` in the foolish store.
    fn sift_for_first_creation(node: &FirRef) -> Option<FirRef> {
        if node.borrow().kind() == FirKind::Creation {
            return Some(Rc::clone(node));
        }
        let children: Vec<FirRef> = node.borrow().core().foolish_children().to_vec();
        children.iter().find_map(sift_for_first_creation)
    }

    #[test]
    fn creation_as_whole_statement_body_reports_its_name() {
        // `'a = ⬤` -- the creation's parent IS the statement `'a`, and the
        // creation IS that statement's whole body. It names itself.
        let root = Compiler::compile("{'a=⬤; b='a;}").unwrap().pop().unwrap();
        let creation = sift_for_first_creation(&root).expect("creation must exist");
        let name = creation.borrow().as_creation_display_name(&creation);
        assert_eq!(
            name.as_deref(),
            Some("'a"),
            "a creation that is the whole body of statement `'a` reports the \
             FULL characterized name, leading quote included"
        );
    }

    #[test]
    fn creation_inside_operator_expression_reports_no_name() {
        // `'a = 1 + ⬤` -- the creation's parent is the OPERATOR, not the
        // statement. The statement's name belongs to the whole `1+⬤`
        // expression, not to the creation sitting inside it.
        let root = Compiler::compile("{'a=1+⬤; b='a;}").unwrap().pop().unwrap();
        let creation = sift_for_first_creation(&root).expect("creation must exist");
        let name = creation.borrow().as_creation_display_name(&creation);
        assert_eq!(
            name, None,
            "a creation that is only a sub-expression of the RHS reports no \
             name -- the statement name does not belong to it"
        );
    }

    #[test]
    fn creation_under_plainly_named_statement_reports_that_name() {
        // The rule is not specific to null-characterized (`'`-prefixed) names.
        let root = Compiler::compile("{a=⬤;}").unwrap().pop().unwrap();
        let creation = sift_for_first_creation(&root).expect("creation must exist");
        let name = creation.borrow().as_creation_display_name(&creation);
        assert_eq!(name.as_deref(), Some("a"));
    }

    #[test]
    fn creation_with_no_statement_parent_reports_no_name() {
        // Constructed directly under a brane -- no statement anywhere in its
        // parent chain, so there is no name to report.
        let parent = make_brane(vec![]);
        let creation = CreationFir::creation(Rc::downgrade(&parent));
        let name = creation.borrow().as_creation_display_name(&creation);
        assert_eq!(
            name, None,
            "a creation whose parent is not a statement reports no name"
        );
    }

    #[test]
    fn creation_reached_through_search_still_reports_its_own_name() {
        // THE PAYOFF CASE. `b='a` resolves to the SAME creation `Rc` that `'a`
        // defines (Gotcha #2), so walking that resolved value's parent chain
        // still lands on the ORIGINAL `'a` statement -- proving the parent
        // chain survives detachment/recoordination at the reference site.
        let root = Compiler::compile("{'a=⬤; b='a;}").unwrap().pop().unwrap();
        let scope = Scope::empty();
        let _ = step_to_settled(&root, &scope);

        let stmts = root.borrow().core().foolish_children().to_vec();
        let b_body = stmts[1].borrow().core().foolish_children()[0].clone();
        let resolved = b_body.value();
        assert_eq!(
            resolved.borrow().kind(),
            FirKind::Creation,
            "`b='a` must resolve THROUGH the search to the creation itself"
        );

        let name = resolved.borrow().as_creation_display_name(&resolved);
        assert_eq!(
            name.as_deref(),
            Some("'a"),
            "a creation reached through a search from a DIFFERENT statement \
             still reports its OWN defining statement's name"
        );
    }

    #[test]
    fn distinct_creations_report_their_own_statement_names() {
        // Two same-named statements (`'k`) in two different branes each define
        // their own creation. Each must report its own -- pointer identity, not
        // a coincidental name match, is what decides.
        let root = Compiler::compile("{A={'k=⬤;}; B={'k=⬤;};}")
            .unwrap()
            .pop()
            .unwrap();
        let stmts = root.borrow().core().foolish_children().to_vec();

        let names: Vec<Option<String>> = stmts
            .iter()
            .map(|outer| {
                let creation =
                    sift_for_first_creation(outer).expect("each brane defines a creation");
                creation.borrow().as_creation_display_name(&creation)
            })
            .collect();

        assert_eq!(
            names,
            vec![Some("'k".to_owned()), Some("'k".to_owned())],
            "each creation reports the name of the statement it is the body of"
        );

        let a_creation = sift_for_first_creation(&stmts[0]).unwrap();
        let b_creation = sift_for_first_creation(&stmts[1]).unwrap();
        assert!(
            !Rc::ptr_eq(&a_creation, &b_creation),
            "they are genuinely distinct creations despite the shared name"
        );
    }
}
