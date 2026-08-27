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

/// The strip budget carried down one root-to-leaf path of a constanic clone
/// (FOOP-55 §5).
///
/// A clone may remove **at most one** SF/SFF mark **per path**. Nesting is
/// therefore a deferral count — `<< <<X>> >>` strips its outer mark and keeps
/// the inner one, because both sit on the same path — while *sibling* marks
/// are independent: `'mod`'s two operands `<<#-2>>` and `<<#-1>>` are separate
/// subtrees, each with its own budget, and both resolve as they always have.
///
/// `Copy` and passed **by value** on purpose: descending into a child inherits
/// the parent's remaining budget, but spending it in one child must not affect
/// that child's siblings.
///
/// Wraps a count (not a bool) so a future higher budget (more than one strip
/// per path) is a matter of starting `constanic_clone` with a bigger number,
/// not a new type (human, 2026-08-26). The count is optional so that
/// "unlimited" is carried explicitly by `None` rather than by a sentinel
/// value (human, 2026-08-26).
#[derive(Debug, Clone, Copy)]
pub(crate) struct StripBudget {
    /// `None` = unlimited: every strip is permitted and nothing is ever
    /// decremented. `Some(n)` = `n` strips remain on this path.
    remaining: Option<u32>,
}

impl StripBudget {
    /// A budget of `n` strips available — `n=1` is "one path's worth", the
    /// ordinary case; `n=0` is the SF-enforcement case (human, 2026-08-26):
    /// SF enforcement is accomplished during `constanic_clone` — when the
    /// stepper is currently stepping INSIDE an SF mark
    /// (`scope.has_ancestral_sfm`), any clone it triggers must not strip
    /// anything, so the found content's own nested marks stay intact
    /// rather than being freed to run in a context the SF's own deferral
    /// has not yet decided is final.
    fn new(n: u32) -> Self {
        StripBudget { remaining: Some(n) }
    }

    /// A budget with no limit: every mark on the path may be stripped, and
    /// spending never exhausts it.
    ///
    /// This is what [`OpInstructions::InsideUfm`] starts with — the Unstay
    /// Foolishness Mark's whole purpose is removing ALL SF/SFF layers below
    /// it, not merely the outermost one.
    fn unlimited() -> Self {
        StripBudget { remaining: None }
    }

    /// A budget with one strip available — one path's worth, the ordinary
    /// starting point for `constanic_clone`.
    fn fresh() -> Self {
        StripBudget::new(1)
    }

    /// Spend one strip if available. Returns `(may_strip, remaining_budget)`.
    /// The caller passes `remaining_budget` down the path it descends.
    ///
    /// An unlimited budget (`None`) always permits the strip and comes back
    /// unchanged. A limited one permits the strip while any remain, and
    /// decrements with `saturating_sub` so the count can never wrap around.
    fn spend(self) -> (bool, Self) {
        match self.remaining {
            None => (true, self),
            Some(0) => (false, self),
            Some(n) => (true, StripBudget::new(n.saturating_sub(1))),
        }
    }
}

/// What the STEPPER calling [`ProtoBrane::constanic_clone`] is currently
/// stepping inside, which decides the strip budget the clone begins with
/// (FOOP-55 §5, Phase 3J).
///
/// This names the *call site's* ambient condition, never a property of the
/// clone TARGET: the same subtree cloned under different instructions keeps
/// or loses its marks accordingly. A mark encountered mid-descent is still
/// handled by [`ProtoBrane::_inner_constanic_clone`]'s own mark arm; these
/// variants only choose what budget that descent starts with.
///
/// Replaces the former `inside_sf_mark: bool` (human, 2026-08-27): a boolean
/// could name only two of the three real conditions, and UFM needs the third.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OpInstructions {
    /// An ordinary step, outside any mark. Budget of **one**: the outermost
    /// SF/SFF mark on each path strips, and a nested one survives.
    Normal,
    /// Stepping inside an SF mark (`scope.has_ancestral_sfm`). Continues with
    /// a budget of one that is **already spent**, so nothing strips: the
    /// enclosing SF has not yet decided this content's final position, so the
    /// found body's own marks stay wrapped and their NYES preserved verbatim.
    InsideSfm,
    /// Stepping inside a UFM (`<@ … @>`). Continues with an **unlimited**
    /// budget: every SF/SFF layer below strips, however deeply nested. UFM
    /// removes the effects of `<>`/`<<>>` throughout its whole subtree.
    ///
    /// Named by `UfmFir::on_foolish_op_ready` (Phase 3J) for its strip-clone.
    InsideUfm,
}

impl OpInstructions {
    /// The strip budget a clone under these instructions begins with.
    ///
    /// `InsideSfm` spends the fresh budget down by one rather than naming a
    /// zero constructor, keeping "one path's worth, already used" visible as
    /// the single formula it is (human, 2026-08-26: "you can just do
    /// budget-1 when inside an SFMark").
    fn starting_budget(self) -> StripBudget {
        match self {
            OpInstructions::Normal => StripBudget::fresh(),
            OpInstructions::InsideSfm => StripBudget::fresh().spend().1,
            OpInstructions::InsideUfm => StripBudget::unlimited(),
        }
    }
}

/// Define a FIR kind's `constanic_clone` dispatch arm (FOOP-55 §5, Phase 4B).
///
/// A kind whose clone is "rebuild me around a budgeted clone of my children"
/// gets its whole body from here. `$Kind` is the struct; each `$field` is
/// extra state carried beside `core`/`self_weak`, taken as a same-named
/// parameter ahead of the standard ones.
///
/// **This exists to make the budget un-droppable.** These bodies were
/// duplicated per kind, and `779b63f5` threaded `StripBudget` through the
/// copies in `fir_kinds.rs` while missing the ones in `system_foo.rs` — which
/// then minted `StripBudget::fresh()` and silently discarded the descending
/// budget for two months. With one body, a kind cannot get that wrong by
/// copy-paste: there is exactly one place the budget is threaded, and adding
/// a kind cannot fork it.
///
/// Macros are used sparingly here (`rust_instructions.md` §5). This is the
/// sanctioned case — unavoidable repetition, removed without loss of clarity
/// — because the bodies differ only in a struct name and an optional field.
macro_rules! budgeted_constanic_clone {
    ($Kind:ident $(, $field:ident : $FieldTy:ty)* $(,)?) => {
        /// Constanic-clone this FIR onto `new_parent`, recoordinating it.
        ///
        /// This is the clone that makes the `system.foo` operators work:
        /// `'lt` is cloned out of `system.foo` and recoordinated into the
        /// brane that referenced it, and the clone's operand lookups then
        /// resolve against THAT brane's neighbours. Children go through
        /// `clone_children_budgeted` — the operands must come across as
        /// ordinary children so the recoordination applies to them too, and
        /// so `stay_budget` keeps descending rather than being re-minted.
        pub(crate) fn constanic_clone(
            $($field: $FieldTy,)*
            source: &std::cell::Ref<'_, dyn Fir>,
            new_parent: &Weak<RefCell<dyn Fir>>,
            nyes: Nyes,
            disable_nyes_reset: bool,
            skip_foolish_children: bool,
            stay_budget: StripBudget,
        ) -> FirRef {
            Rc::new_cyclic(|me: &Weak<RefCell<$Kind>>| {
                let self_weak: Weak<RefCell<dyn Fir>> = me.clone();
                let core = ProtoBrane::clone_children_budgeted(
                    source.core(),
                    &self_weak,
                    new_parent,
                    nyes,
                    disable_nyes_reset,
                    skip_foolish_children,
                    stay_budget,
                );
                RefCell::new($Kind { core, $($field,)* self_weak })
            })
        }
    };
}

pub(crate) use budgeted_constanic_clone;

impl ProtoBrane {
    /// Clone a source core's children into a new core, carrying the
    /// in-progress clone operation's strip budget (FOOP-55 §5).
    ///
    /// `pub(crate)` for the `system_foo` kinds (`ComparisonFir`, `ModuloFir`,
    /// `OrFir`), whose `constanic_clone` dispatch arms live in that module
    /// only because their types do; they clone children exactly as the kinds
    /// in this module do, and pass down the same descending `budget`.
    pub(crate) fn clone_children_budgeted(
        source: &ProtoBrane,
        self_weak: &Weak<RefCell<dyn Fir>>,
        new_parent: &Weak<RefCell<dyn Fir>>,
        nyes: Nyes,
        sfm: bool,
        skip_foolish_children: bool,
        budget: StripBudget,
    ) -> ProtoBrane {
        let cloned_children: Vec<FirRef> = if skip_foolish_children {
            Vec::new()
        } else {
            source
                .foolish_children()
                .iter()
                .enumerate()
                .map(|(i, c)| {
                    ProtoBrane::_inner_constanic_clone(c, self_weak, i, sfm, false, budget)
                })
                .collect()
        };
        let core = ProtoBrane::new(
            cloned_children,
            new_parent.clone(),
            nyes.transform_for_clone(sfm),
        );
        for ubc in source.ubc_children() {
            core.push_ubc_child(ProtoBrane::_inner_constanic_clone(
                &ubc, self_weak, 0, sfm, false, budget,
            ));
        }
        core
    }

    /// Public entry point for a constanic clone (FOOP-55 §5, §11/D9 item 3,
    /// corrected 2026-08-26 human direction: "SFM enforcement is
    /// accomplished during constanic cloning — when a stepper is aware
    /// that it is stepping inside an SF mark, the clones do not reset
    /// nyes").
    ///
    /// Every clone is one **operation** with one **strip budget** and its own
    /// **fresh** `disable_nyes_reset = false` starting point. What IS carried
    /// in is [`OpInstructions`]: what the STEPPER calling this (not some
    /// unrelated ancestor of the clone TARGET) is currently stepping inside.
    /// That choice, and only that choice, sets the starting budget —
    /// `Normal` one strip, `InsideSfm` none, `InsideUfm` unlimited. See
    /// [`OpInstructions::starting_budget`].
    ///
    /// This is DIFFERENT from the superseded design (D9 item 3, first
    /// draft): that version ignored ambient scope entirely, which broke a
    /// search's own found-body clone reached while STILL inside the
    /// search's own SF wrapper (a concatenation element, `<<OB2>>`, whose
    /// SF has not yet let its content be used) — that content must stay
    /// frozen until the SF genuinely resolves, not be immediately stripped
    /// just because THIS particular clone call started a fresh budget.
    /// [`Self::_inner_constanic_clone`]'s own SF/SFF-mark-encounter arm is
    /// still the only place `disable_nyes_reset` ever becomes `true` for a
    /// STRIPPED mark's content — this just controls what budget it starts
    /// with, not the stripping logic itself.
    pub(crate) fn constanic_clone(
        fir_ref: &FirRef,
        new_parent: &Weak<RefCell<dyn Fir>>,
        index: usize,
        skip_foolish_children: bool,
        instructions: OpInstructions,
    ) -> FirRef {
        Self::_inner_constanic_clone(
            fir_ref,
            new_parent,
            index,
            false,
            skip_foolish_children,
            instructions.starting_budget(),
        )
    }

    /// The recursive worker behind [`Self::constanic_clone`]. `stay_budget`
    /// is this call's OWN strip budget — NOT inherited from a parent's
    /// already-spent budget (FOOP-55.md D9 item 3): every distinct child
    /// [`Self::clone_children_budgeted`] recurses into gets its OWN copy of
    /// whatever budget it was handed, so sibling marks each get their own
    /// mark stripped independently. Only a genuine strip-then-recurse INTO
    /// what a mark wraps (the same path, one layer deeper) passes the
    /// SPENT budget onward.
    pub(crate) fn _inner_constanic_clone(
        fir_ref: &FirRef,
        new_parent: &Weak<RefCell<dyn Fir>>,
        index: usize,
        disable_nyes_reset: bool,
        skip_foolish_children: bool,
        stay_budget: StripBudget,
    ) -> FirRef {
        if matches!(
            fir_ref.borrow().kind(),
            FirKind::StayFoolish | FirKind::StayFullyFoolish
        ) {
            // FOOP-55 §5 / D9 item 3: budget exhausted -> this mark STAYS,
            // and its content is genuinely still foolishly-ignorant (it has
            // not been stripped), so `disable_nyes_reset` becomes `true` for
            // THIS clone -- decided locally, from this call's own remaining
            // budget, never inherited. An unstripped mark has not searched,
            // so it holds no resolved reference to any brane and no
            // per-site state -- sharing the original node is sound.
            let (may_strip, stay_budget) = stay_budget.spend();
            if !may_strip {
                return Rc::clone(fir_ref);
            }
            let source = fir_ref.borrow();
            if source.kind() == FirKind::StayFoolish
                && let Some(constanic_result) = source.core().ubc_children().into_iter().next()
            {
                return Self::_inner_constanic_clone(
                    &constanic_result,
                    new_parent,
                    index,
                    disable_nyes_reset,
                    skip_foolish_children,
                    stay_budget,
                );
            }
            if let Some(inner) = source.core().foolish_children().first().cloned() {
                return Self::_inner_constanic_clone(
                    &inner,
                    new_parent,
                    index,
                    disable_nyes_reset,
                    skip_foolish_children,
                    stay_budget,
                );
            }
            eprintln!("ALARM: SF/SFF node has no children — cloning wrapper as-is");
        }
        let nyes = fir_ref.borrow().core().get_nyes();
        let kind_now = fir_ref.borrow().kind();
        // IndepInt/Nk never read context (no children, no scope, no
        // neighbours) -- their eventual value is fixed at construction
        // regardless of where they end up recoordinated to, so sharing the
        // SAME node is always safe, independent of their CURRENT NYES
        // (human, 2026-08-26: "it should provide a reference"). Every other
        // kind still only shares once already Constant/Independent.
        if matches!(kind_now, FirKind::IndepInt | FirKind::Nk)
            || ((nyes == Nyes::Constant || nyes == Nyes::Independent) && kind_now != FirKind::Brane)
        {
            return Rc::clone(fir_ref);
        }
        let borrowed = fir_ref.borrow();
        let kind = borrowed.kind();
        match kind {
            FirKind::IndepInt | FirKind::Nk => {
                unreachable!("IndepInt/Nk are shared by reference above, never reach this match")
            }
            FirKind::Operator => {
                let op_name = borrowed.as_op_name().unwrap_or("?").to_owned();
                Rc::new_cyclic(|me: &Weak<RefCell<OperatorFir>>| {
                    let self_weak: Weak<RefCell<dyn Fir>> = me.clone();
                    let core = ProtoBrane::clone_children_budgeted(
                        borrowed.core(),
                        &self_weak,
                        new_parent,
                        nyes,
                        disable_nyes_reset,
                        skip_foolish_children,
                        stay_budget,
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
                        disable_nyes_reset,
                        skip_foolish_children,
                        stay_budget,
                    ),
                    None => Rc::new(RefCell::new(NkFir {
                        core: ProtoBrane::new(vec![], new_parent.clone(), Nyes::Nk),
                        reason: "comparison: unknown operator".to_string(),
                    })),
                }
            }
            FirKind::Modulo => {
                let op = borrowed
                    .as_op_name()
                    .and_then(crate::system_foo::ArithOp::from_searchable_name);
                match op {
                    Some(op) => crate::system_foo::ModuloFir::constanic_clone(
                        op,
                        &borrowed,
                        new_parent,
                        nyes,
                        disable_nyes_reset,
                        skip_foolish_children,
                        stay_budget,
                    ),
                    None => Rc::new(RefCell::new(NkFir {
                        core: ProtoBrane::new(vec![], new_parent.clone(), Nyes::Nk),
                        reason: "modulo: unknown operator".to_string(),
                    })),
                }
            }
            FirKind::Or => crate::system_foo::OrFir::constanic_clone(
                &borrowed,
                new_parent,
                nyes,
                disable_nyes_reset,
                skip_foolish_children,
                stay_budget,
            ),
            FirKind::SearchPosition => Rc::new_cyclic(|me: &Weak<RefCell<SearchPositionFir>>| {
                let self_weak: Weak<RefCell<dyn Fir>> = me.clone();
                let core = ProtoBrane::clone_children_budgeted(
                    borrowed.core(),
                    &self_weak,
                    new_parent,
                    nyes,
                    disable_nyes_reset,
                    skip_foolish_children,
                    stay_budget,
                );
                RefCell::new(SearchPositionFir { core })
            }),
            FirKind::Search => {
                let clone_nyes_val = nyes.transform_for_clone(disable_nyes_reset);
                let pattern = borrowed.as_search_pattern().unwrap_or("").to_owned();
                let anchored = borrowed.as_search_anchored();
                let is_value = borrowed.as_search_is_value();
                let is_contexted = borrowed.as_search_contexted();
                let chain_econstanic = if !disable_nyes_reset && nyes == Nyes::Woconstanic {
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
                                ProtoBrane::_inner_constanic_clone(
                                    c,
                                    &self_weak,
                                    i,
                                    disable_nyes_reset,
                                    skip_foolish_children,
                                    stay_budget,
                                )
                            })
                            .collect()
                    };
                    let core = ProtoBrane::new(children, new_parent.clone(), clone_nyes_val);
                    if let Some(ref econ) = chain_econstanic {
                        core.push_ubc_child(ProtoBrane::constanic_clone(
                            econ,
                            &self_weak,
                            0,
                            false,
                            OpInstructions::Normal,
                        ));
                    } else {
                        for ubc in borrowed.core().ubc_children() {
                            core.push_ubc_child(ProtoBrane::_inner_constanic_clone(
                                &ubc,
                                &self_weak,
                                0,
                                disable_nyes_reset,
                                skip_foolish_children,
                                stay_budget,
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
                        exhausted: std::cell::Cell::new(false),
                        found_context: RefCell::new(None),
                    })
                })
            }
            FirKind::Index => {
                let offset = borrowed.as_index_offset();
                let anchored = borrowed.as_index_anchored();
                let is_contexted = borrowed.as_search_contexted();
                Rc::new_cyclic(|me: &Weak<RefCell<IndexFir>>| {
                    let self_weak: Weak<RefCell<dyn Fir>> = me.clone();
                    let core = ProtoBrane::clone_children_budgeted(
                        borrowed.core(),
                        &self_weak,
                        new_parent,
                        nyes,
                        disable_nyes_reset,
                        skip_foolish_children,
                        stay_budget,
                    );
                    RefCell::new(IndexFir {
                        core,
                        offset,
                        index_expr: None,
                        anchored,
                        contexted: is_contexted,
                    })
                })
            }
            FirKind::StayFoolish | FirKind::StayFullyFoolish => {
                unreachable!("SF/SFF stripped at fn top")
            }
            // A UFM clones like any other operator: rebuild the wrapper around
            // a budgeted clone of its children. It is NOT stripped at the fn
            // top -- only SF/SFF marks are; a UFM in a cloned tree still has
            // its own unstripping to do when it steps.
            FirKind::Ufm => Rc::new_cyclic(|me: &Weak<RefCell<UfmFir>>| {
                let self_weak: Weak<RefCell<dyn Fir>> = me.clone();
                let core = ProtoBrane::clone_children_budgeted(
                    borrowed.core(),
                    &self_weak,
                    new_parent,
                    nyes,
                    disable_nyes_reset,
                    skip_foolish_children,
                    stay_budget,
                );
                RefCell::new(UfmFir { core })
            }),
            FirKind::Concatenation => {
                let helpers_populated = !borrowed.core().ubc_children().is_empty();
                let provenance = borrowed.as_concat_provenance();
                Rc::new_cyclic(|me: &Weak<RefCell<BraneConcatOpFir>>| {
                    let self_weak: Weak<RefCell<dyn Fir>> = me.clone();
                    let core = ProtoBrane::clone_children_budgeted(
                        borrowed.core(),
                        &self_weak,
                        new_parent,
                        nyes,
                        disable_nyes_reset,
                        skip_foolish_children,
                        stay_budget,
                    );
                    RefCell::new(BraneConcatOpFir {
                        core,
                        _helpers_populated: std::cell::Cell::new(helpers_populated),
                        provenance,
                    })
                })
            }
            FirKind::ConcatHelper => Rc::new_cyclic(|me: &Weak<RefCell<ConcatHelper>>| {
                let self_weak: Weak<RefCell<dyn Fir>> = me.clone();
                let core = ProtoBrane::clone_children_budgeted(
                    borrowed.core(),
                    &self_weak,
                    new_parent,
                    nyes,
                    disable_nyes_reset,
                    skip_foolish_children,
                    stay_budget,
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
                    let core = ProtoBrane::clone_children_budgeted(
                        borrowed.core(),
                        &self_weak,
                        new_parent,
                        nyes,
                        disable_nyes_reset,
                        skip_foolish_children,
                        stay_budget,
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
                let core = ProtoBrane::clone_children_budgeted(
                    borrowed.core(),
                    &self_weak,
                    new_parent,
                    nyes,
                    disable_nyes_reset,
                    skip_foolish_children,
                    stay_budget,
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

/// How equality classifies two **not mutually identifiable** values.
///
/// Two constanic values of different kinds — a brane against an integer, an
/// integer against a creation — can never bear the same identity (FOOP-33 §2,
/// as clarified by FOOP-55 §8). Two readings are defensible:
///
/// - **`true` (current, and the only behaviour today): "not mutually
///   identifiable" IS "not equal".** A brane is never an integer, so the answer is decidable even
///   though the comparison is not meaningful. A value search Rejects the
///   candidate and keeps scanning.
/// - **`false`: "not mutually identifiable" is UNKNOWABLE.** The comparison has no answer, so
///   neither does the search.
///
/// This is a **policy**, not a fact about the values, and it is made explicit
/// here so it can be made configurable later (per-suite, or per-search) without
/// first having to find where the decision was buried. It is deliberately a
/// `const` for now: exactly one behaviour ships, and the branch below documents
/// what the alternative would mean.
///
/// **Do not flip this to `false` casually.** `rust_instructions.md` records the
/// FOOP-33 incident where a three-valued `default_equal` returned Unknowable
/// for brane-vs-integer, which made value searches **abort** on the first
/// non-comparable candidate instead of skipping it — turning a working
/// `mixed~=7` into NK and silently changing eleven baselines.
pub(crate) const NOT_MUTUALLY_IDENTIFIABLE_IS_NOT_EQUAL: bool = true;

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
    // Different non-NK constanic kinds (brane-vs-integer, integer-vs-creation,
    // etc.). Whether that counts as "not equal" or as "unknowable" is a
    // POLICY, not a fact about the values — see [`NOT_MUTUALLY_IDENTIFIABLE_IS_NOT_EQUAL`].
    if NOT_MUTUALLY_IDENTIFIABLE_IS_NOT_EQUAL {
        // A brane is never an integer: different FIR kinds, decidable. The
        // matcher Rejects (skips) and continues scanning, rather than NkStop
        // (abort) — see FOOP-55 §8 and the FOOP-33 incident in
        // rust_instructions.md, where treating this as Unknowable made value
        // searches abort on the first non-comparable candidate.
        Equality::NotEqual
    } else {
        Equality::Unknowable
    }
}

/// `anchor@` — a search result's POSITION (FOOP-55 §8).
///
/// A **continuation**: the anchor must BE a search, because only a search
/// produces a position. It has one dependency (that anchor), so it settles
/// once the anchor is constanic — hit or miss alike, since NYES comes from the
/// dependency and not from the search outcome.
///
/// | Anchor's state | `@` |
/// |----------------|-----|
/// | found | the found statement's index |
/// | `candidates_exhausted()` | **-1** |
/// | NK (never scanned) | **NK** — "where in nothing?" has no answer |
/// | not a search at all | **NK** — a malformed continuation, per §8 |
///
/// The `-1` is what makes a default branch fall out of arithmetic: `@+1` maps a
/// miss to index 0, so a table written with its default FIRST is selected by
/// the same expression that steps a hit to its adjacent `value=` row.
#[derive(Debug)]
pub struct SearchPositionFir {
    pub(crate) core: ProtoBrane,
}

impl SearchPositionFir {
    /// Pushes an NK `ubc_child` with `reason` and reports `Nk` — the shared
    /// shape behind every NK exit in [`Fir::on_foolish_op_ready`] below.
    fn settle_nk(&self, reason: &str) -> Nyes {
        let nk_ref = NkFir::nk(reason, self.core.parent_weak());
        nk_ref.borrow().core().set_nyes(Nyes::Nk);
        self.core.push_ubc_child(nk_ref);
        self.core.set_alarm_reason(reason.to_owned());
        Nyes::Nk
    }

    /// Pushes the computed index as an `IndepInt` `ubc_child` and reports
    /// `Independent` — FOOP-55 §11 (human, 2026-08-25). `@` has computed a
    /// final, self-contained integer here; there is nothing left to wait
    /// ON. Reporting `Independent` (rather than the old hardcoded
    /// `Woconstanic`, which asserted an ongoing wait that did not exist)
    /// is what lets a consuming operator (e.g. `@+1`) see this operand as
    /// genuinely ready via the standard `constantew` gate, without any
    /// `@`-specific special-casing on the consumer's side.
    fn settle_index(&self, index: i64) -> Nyes {
        let out = IndepIntFir::constant_int(index, self.core.parent_weak());
        out.borrow().core().set_nyes(Nyes::Independent);
        self.core.push_ubc_child(out);
        Nyes::Independent
    }
}

impl Fir for SearchPositionFir {
    #[inline(always)]
    fn core(&self) -> &ProtoBrane {
        &self.core
    }

    fn kind(&self) -> FirKind {
        FirKind::SearchPosition
    }

    fn fir_op_step(&self, scope: &Scope) -> Result<(), UbcError> {
        match self.core.get_nyes() {
            Nyes::Prembrionic | Nyes::Embryonic => {
                self.core.set_nyes(Nyes::Braning);
                if let Some(anchor) = self.core.foolish_children().first().cloned() {
                    self.core.push_task(anchor);
                }
            }
            Nyes::Braning => {
                if let Some(nyes) = self.on_foolish_op_ready(scope) {
                    self.core.set_nyes(nyes);
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// FOOP-55 §11 (human, 2026-08-25): `@`'s one dependency (the anchor
    /// search) is "done enough" the moment it has FOUND something
    /// (`is_found()`), independent of whether the anchor's own NYES ever
    /// reaches a terminal state — a search can be genuinely, permanently
    /// `WOCONSTANIC` (found a statement whose value never resolves) and `@`
    /// still has a real, final index to report. Without this override, the
    /// default (`is_constanic()`) would leave such an anchor stuck on the
    /// task queue forever, since `step_inner`'s dequeue gate uses this
    /// predicate directly. Falls back to the default (plain `is_constanic()`)
    /// once the anchor is NOT found — needed to let a genuine miss (NK
    /// anchor, or an exhausted scan) still dequeue and reach
    /// `on_foolish_op_ready`'s own miss-handling.
    fn is_foolish_child_constanic_enough(&self, child: &FirRef) -> bool {
        child.borrow().is_found() || child.borrow().core().get_nyes().is_constanic()
    }

    /// FOOP-55 §11: moved from `combine` (formerly called directly from the
    /// `Braning` arm), converting every `set_nyes` into `Some(nyes)`.
    fn on_foolish_op_ready(&self, _scope: &Scope) -> Option<Nyes> {
        let Some(anchor) = self.core.foolish_children().first().cloned() else {
            return Some(self.settle_nk("@: no anchor"));
        };

        // §8: the anchor must BE a search. A malformed continuation is a true
        // NK, not a compile error -- an unanswerable question, as elsewhere in
        // Foolish.
        if anchor.borrow().kind() != FirKind::Search {
            return Some(self.settle_nk("@: anchor is not a search"));
        }

        // FOOP-55 §11 (human, 2026-08-25): `@` needs to know the anchor
        // FOUND something, not that the anchor's own NYES reached any
        // particular terminal state — a search can be genuinely,
        // permanently WOCONSTANIC (found a statement whose value never
        // resolves) and `@` still has a real, final index to report. Read
        // that directly via `is_found()`/`found_context` rather than
        // waiting on `is_constanic()` or reaching into `ubc_children[1]`.
        if anchor.borrow().is_found() {
            let idx = anchor
                .borrow()
                .found_context_index()
                .expect("is_found() true implies found_context_index() is Some");
            return Some(self.settle_index(idx as i64));
        }

        // Not found (yet, or ever). Only once the anchor is itself
        // constanic can we distinguish "ran out of candidates" (a genuine
        // miss, per `candidates_exhausted()`) from "still searching."
        if !anchor.borrow().core().get_nyes().is_constanic() {
            return None; // still waiting on our one dependency
        }
        if anchor.borrow().candidates_exhausted() {
            Some(self.settle_index(-1))
        } else {
            Some(self.settle_nk("@: anchor never scanned (its own anchor was NK)"))
        }
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
                // FOOP-55 §11 (human, 2026-08-24): the any-NK-poison check
                // still runs first and separately (it must fire even when
                // NOT every child is constantew — an NK poisons regardless
                // of what else is waiting). Only once past that does
                // `are_foolish_children_ready_for_op()` gate whether the
                // real operation may be ATTEMPTED at all; when it is not
                // ready, fall back to the shared, honest
                // `_decide_nyes_due_to_children` rather than guessing.
                let children = self.core.foolish_children().to_vec();
                let any_nk = children
                    .iter()
                    .any(|c| c.borrow().core().get_nyes() == Nyes::Nk);
                let nyes = if any_nk {
                    self.on_foolish_op_ready(scope)
                        .expect("on_foolish_op_ready must report Some when an NK is present")
                } else if self.are_foolish_children_ready_for_op() {
                    self.on_foolish_op_ready(scope)
                        .expect("on_foolish_op_ready must report Some once ready")
                } else {
                    _decide_nyes_due_to_children(&children).unwrap_or(Nyes::Woconstanic)
                };
                self.core.set_nyes(nyes);
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

    /// FOOP-55 §11: moved from `combine` (formerly called directly from the
    /// `Braning` arm), converting every `set_nyes` + early `return Ok(())`
    /// into `Some(nyes)`. Preserves the existing any-`NK`-poison
    /// short-circuit's ordering — checked FIRST, separately from the
    /// values/constanew wait below, exactly as before (FOOP-55 §11's own
    /// caution about this kind: its gate is NOT simply "wait for
    /// constantew," the poison check runs first).
    fn on_foolish_op_ready(&self, scope: &Scope) -> Option<Nyes> {
        let children = self.core.foolish_children().to_vec();

        let any_nk = children
            .iter()
            .any(|c| c.borrow().core().get_nyes() == Nyes::Nk);
        if any_nk {
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
            self.push_nk_result(scope, reason);
            return Some(Nyes::Nk);
        }

        // FOOP-55 §11 (human, 2026-08-25): by the time this method runs,
        // the caller has already established every child is `constantew`
        // (either via the any-NK check above, or via
        // `are_foolish_children_ready_for_op`'s own `constantew` gate) —
        // so a child that still fails to produce an integer here is not
        // "not ready yet," it is PERMANENTLY the wrong TYPE (e.g. a brane,
        // not a number). That must settle NK with a reason naming the
        // problem, matching `BraneConcatOpFir`'s own type-error handling —
        // not the old `Woconstanic`, which wrongly implied an ongoing wait
        // for something that will never resolve.
        let non_integer_indexes: Vec<usize> = children
            .iter()
            .enumerate()
            .filter(|(_, c)| c.value().borrow().as_i64().is_none())
            .map(|(idx, _)| idx)
            .collect();
        if !non_integer_indexes.is_empty() {
            let list = non_integer_indexes
                .iter()
                .map(usize::to_string)
                .collect::<Vec<_>>()
                .join(",");
            self.push_nk_result(
                scope,
                format!("operator operand indexes that are not integers: {list}"),
            );
            return Some(Nyes::Nk);
        }

        let values: Vec<i64> = children
            .iter()
            .map(|c| c.value().borrow().as_i64().expect("checked above"))
            .collect();

        let result = match self.op.as_str() {
            "+" if values.len() == 2 => values[0] + values[1],
            "-" if values.len() == 2 => values[0] - values[1],
            "*" if values.len() == 2 => values[0] * values[1],
            "/" if values.len() == 2 => {
                if values[1] == 0 {
                    self.push_nk_result(scope, "division by zero".to_string());
                    return Some(Nyes::Nk);
                }
                values[0] / values[1]
            }
            "%" if values.len() == 2 => {
                if values[1] == 0 {
                    self.push_nk_result(scope, "division by zero".to_string());
                    return Some(Nyes::Nk);
                }
                values[0] % values[1]
            }
            "-" if values.len() == 1 => -values[0], // unary negation
            // FOOP-75 §7: the `"$"` arm that used to live here is DELETED.
            //
            // It served the old bespoke `=$` sugar, which built
            // `BinaryOp("$", UnanchoredSeek{-1}, rhs)` in the parser. It
            // validated that the RHS was a brane and then returned WITHOUT
            // extracting the tail, so `y =$ b` settled to the whole brane
            // rather than its last element -- contradicting FOOP-54 §D.5
            // ("bind the value of the last statement of `b` to `a`").
            // There was never a matching `"^"` arm at all, so `=^` never
            // settled and leaked `Op^(...)` into rendered output.
            //
            // Both spellings now compile to `IndexFir` via
            // `Astn::HeadTail`, exactly as the postfix `b$` / `b^` always
            // did, so this operator path is unreachable and both defects
            // dissolve rather than needing separate fixes.
            //
            // FOOP-55 §11 (2026-08-24): an unrecognized operator string is
            // ALSO unreachable in practice (the parser only ever
            // constructs a known operator) — this arm used to be a hard
            // `UbcError::Eval`, which `on_foolish_op_ready`'s `Option<Nyes>`
            // signature cannot express. Human direction: settle NK with an
            // explanation here; the parser should additionally reject an
            // unrecognized operator at construction time so this remains
            // unreachable (tracked separately, Phase 4B).
            op => {
                self.push_nk_result(
                    scope,
                    format!("unknown operator: {op} ({} operands)", values.len()),
                );
                return Some(Nyes::Nk);
            }
        };

        let self_weak = self.core.parent_weak();
        let result_ref: FirRef = Rc::new_cyclic(|me: &Weak<RefCell<IndepIntFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(IndepIntFir {
                core: ProtoBrane::new(vec![], parent, Nyes::Constant),
                value: result,
            })
        });
        self.core.push_ubc_child(ProtoBrane::constanic_clone(
            &result_ref,
            &self_weak,
            0,
            false,
            OpInstructions::Normal,
        ));
        Some(Nyes::Constant)
    }
}

impl OperatorFir {
    /// FOOP-55 §11 migration: pre-migration `fir_op_step` body, moved here
    /// verbatim (rename only, no logic change). Delete once every piece has
    /// moved into named handlers and `fir_op_step` no longer calls it.
    /// Pushes an NK `ubc_child` with `reason`, constanic-cloned into this
    /// operator's context — the shared shape behind every NK exit in
    /// [`Fir::on_foolish_op_ready`] below (any-operand-NK, division/modulo
    /// by zero, and — new in FOOP-55 §11's migration — an unrecognized
    /// operator, which used to be a hard `UbcError::Eval` before
    /// `on_foolish_op_ready`'s `Option<Nyes>` signature made a genuine
    /// error unrepresentable here; human direction 2026-08-24 was to make
    /// this NK with an explanation, and to additionally have the PARSER
    /// reject unknown operators at parse time so this case cannot be
    /// reached in practice — the parser-side check is tracked separately,
    /// Phase 4B).
    fn push_nk_result(&self, _scope: &Scope, reason: String) {
        let self_weak = self.core.parent_weak();
        let nk_ref: FirRef = Rc::new_cyclic(|me: &Weak<RefCell<NkFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(NkFir {
                core: ProtoBrane::new(vec![], parent, Nyes::Nk),
                reason,
            })
        });
        self.core.push_ubc_child(ProtoBrane::constanic_clone(
            &nk_ref,
            &self_weak,
            0,
            false,
            OpInstructions::Normal,
        ));
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
    /// [`BraneConcatOpFir::apply_null_const_rule_to_merged_stmt`].
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

    /// FOOP-33 (post-merge addition) — **"Named creations cannot be
    /// renamed."** A creation reached ONLY through a null-characterized name
    /// (e.g. `'True`) is that creation's **original name**; giving it a
    /// SECOND, DIFFERENT null-characterized name (`'other = 'True`) would let
    /// the same creation answer to two different protected names,
    /// undermining the point of null-characterization (a name that uniquely
    /// and durably identifies one creation — see the "named creation"/
    /// "original name" terminology in AGENTS.md/README.md). Refused the same
    /// way the null-const conflict rule is: set `nf_reason`, terminal, read
    /// by `settled_result`/`.value()` in place of the written body.
    ///
    /// Trigger, all three required: (1) this statement's own LHS is
    /// null-characterized; (2) its (constanic) body resolves (`.value()`) to
    /// a creation; (3) that creation's original name (its OWN defining
    /// statement's null-characterized name, found via
    /// `CreationFir::get_display_name` viewed from HERE) is `Some` AND
    /// DIFFERS from this statement's own name.
    ///
    /// Condition 3's "differs" clause is what distinguishes a genuine rename
    /// from a same-name REASSERTION, which must stay permitted:
    /// `{'a=⬤; 'a='a;}` — `'a='a` re-states `'a`'s own existing name, not a
    /// second name, and is allowed (mirrors the existing, separately-tested
    /// guarantee that `'True = 'True` is permitted —
    /// `system_foo::tests::program_redefining_true_to_a_conflicting_value_is_refused`).
    /// `{'a=⬤; 'b='a;}` — `'b='a` gives `'a`'s creation a SECOND name `'b`,
    /// and is refused.
    ///
    /// Does not cover concatenation, for the same reason
    /// `check_null_const_conflict` does not — see that method's doc comment.
    fn check_rename_of_named_creation(&self, body: &FirRef) {
        if self.nf_reason.borrow().is_some() {
            return; // already resolved (terminal).
        }
        if !self.identifier.is_nully_characterizing_coordinate_name() {
            return; // only a null-characterized statement can commit this offense.
        }
        let Some(self_rc) = self.self_weak.upgrade() else {
            return; // torn down; nothing to check against.
        };
        let resolved = body.value();
        if resolved.borrow().kind() != FirKind::Creation {
            return; // not a creation reference at all -- nothing to forbid.
        }
        let original_name = resolved
            .borrow()
            .as_creation_display_name(&resolved, Some(&self_rc));
        let Some(original_name) = original_name else {
            return; // the creation has no original name at all -- nothing to protect.
        };
        if original_name != self.identifier.searchable_name() {
            *self.nf_reason.borrow_mut() =
                Some(rename_nf_reason(self.identifier.identifier_name()));
        }
    }
}

/// The NF reason string for a refused attempt to give an already-named
/// creation a second, different null-characterized name. Kept separate from
/// [`null_const_nf_reason`] (a different offense, same NF mechanism) so the
/// reason text distinguishes "conflicting redefinition" from "renaming a
/// named creation" for a human reading an alarm.
fn rename_nf_reason(name: &str) -> String {
    format!("'{name} {NF_PREFIX} (Named creations cannot be renamed)")
}

/// The value a statement PRESENTS — `settled_result()` (the NF refusal NK, if
/// this statement was itself already refused by the null-const rule) if set,
/// else the raw written body. This is the ONE place "what does this statement
/// actually resolve to" is decided; every reader of a statement's value must
/// go through it rather than reaching into `foolish_children().first()`
/// directly, or it will present the pre-refusal RHS instead of the NF NK.
/// Used by `StatementFir::check_null_const_conflict` and
/// `BraneConcatOpFir::apply_null_const_rule_to_merged_stmt` (FOOP-33 §4 —
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
/// so the two trigger sites (`StatementFir`'s own step, and `BraneConcatOpFir`'s
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
                            self.check_rename_of_named_creation(body);
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
    /// statement's own `check_null_const_conflict`: `BraneConcatOpFir`'s
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
    /// FOOP-55 §8: set when the scan ran to completion and no candidate
    /// matched — as distinct from never having scanned (an NK anchor) or
    /// having stopped at a match. Read through
    /// [`Fir::candidates_exhausted`].
    pub(crate) exhausted: std::cell::Cell<bool>,
    /// FOOP-55 §11 (human, 2026-08-25): the found statement's ORIGINAL
    /// `(home brane, statement index)`, captured the moment `handle_found`
    /// discovers it — BEFORE `clone_stmt_result` constanic-clones the
    /// statement's body into `ubc_children` and reparents that clone into
    /// this search's own statement's brane. Reading a found statement's
    /// position back out of `ubc_children` positionally (as
    /// `contexted_search_from_anchor`/`SearchPositionFir::combine` still
    /// do) works only because those callers separately read
    /// `ubc_children[1]`'s `FoolRefFir`, which independently preserves the
    /// original referent — this field is a DIRECT, named alternative to
    /// that positional convention, read via [`Fir::is_found`]. `None`
    /// until a search genuinely finds something; never reset once set
    /// (a search settles once, per FOOP-62's stepping model).
    ///
    /// TODO(FOOP-55 Phase 4B, human 2026-08-25): refactor so this reference
    /// to the search result's parent brane is removed once the entire
    /// statement containing the current search is constanic — the stored
    /// `FirRef` keeps that brane alive for the search node's whole
    /// lifetime, longer than any computation still needs it, a GC hazard.
    pub(crate) found_context: RefCell<Option<(FirRef, usize)>>,
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

    /// Constanic-clone a found statement's body as this search's result.
    ///
    /// The clone starts a fresh strip budget (`constanic_clone`'s own,
    /// `inside_sf_mark = false`): whether the content under a mark is free
    /// to re-evaluate is decided by THIS clone's own budget as it descends,
    /// not by any ambient `Scope` flag the caller happened to be stepping
    /// under.
    /// Constanic-copy the found statement's body into this search's
    /// `ubc_children`.
    ///
    /// `instructions` carries `scope.has_ancestral_sfm` as
    /// [`OpInstructions::InsideSfm`]: SF enforcement
    /// happens during constanic cloning (human, 2026-08-26). When this
    /// search is itself running inside an SF mark, the copy starts with no
    /// strip budget, so a mark on the found body is PRESERVED rather than
    /// stripped and run — `{a={1,2}, b=<<#-2>>, c= a b}` copies `b`'s body
    /// into the search's `ubc_children` still `<<#-2>>`-wrapped, because
    /// that search sits inside `c`'s auto-SF-wrapped element. Outside any
    /// SF mark the budget starts fresh and the outermost mark strips as
    /// usual.
    fn clone_stmt_result(
        stmt: &FirRef,
        new_parent: &Weak<RefCell<dyn Fir>>,
        instructions: OpInstructions,
    ) -> FirRef {
        // Prefer settled_result() over the raw written body: for a plain
        // StatementFir this is None (unchanged behavior — falls through to the
        // body below); the null-characterized name constant rule (FOOP-33 §4)
        // is the ONLY thing that ever makes it Some, substituting the refusal
        // NK for the written RHS without mutating the body's own FIR/nyes.
        let body = statement_value_for_comparison(stmt).expect("statement must have a body");
        let index = stmt.borrow().as_stmt_line_number().unwrap_or(0);
        ProtoBrane::constanic_clone(&body, new_parent, index, false, instructions)
    }

    fn handle_found(&self, stmt: FirRef, _nyes: Nyes, scope: &Scope) {
        // FOOP-55 §11 (human, 2026-08-25): capture the ORIGINAL found
        // statement's home brane and index HERE, before `clone_stmt_result`
        // constanic-clones its body and `push_search_result_pair` reparents
        // that clone into this search's own statement's brane. This is the
        // direct, named record of "did I find something" — read via
        // `Fir::is_found` — independent of the found statement's own VALUE
        // ever resolving (see D9/§5.5: an ECONSTANIC/WOCONSTANIC found
        // statement is still a genuine find).
        if let Some(home_brane) = stmt.borrow()._get_my_brane(&stmt)
            && let Some(idx) = home_brane.find_stmt_index(&stmt)
        {
            *self.found_context.borrow_mut() = Some((home_brane, idx));
        }
        let self_weak = self.core.parent_weak();
        let clone = Self::clone_stmt_result(
            &stmt,
            &self_weak,
            if scope.has_ancestral_sfm {
                OpInstructions::InsideSfm
            } else {
                OpInstructions::Normal
            },
        );
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
        // FOOP-55 §5.5: if `brane` is a BraneConcatOpFir that has not yet
        // populated its helpers, `stmt_count()` honestly answers `None`
        // (never a premature `Some(0)`), so `find_stmt_index` below
        // short-circuits to `None` via `?` -- an ordinary IB miss. The
        // caller (`SearchFir`'s Embryonic arm) already treats that as "try
        // ab_search_with_engine next", which walks outward through
        // successive ancestor branes on its own. No special-case forwarding
        // is needed here: once a nested concatenation is built unmarked
        // (§9.2's corrected classify_concat_element) and steps normally, the
        // existing IB-then-AB fallback is sufficient.
        let idx = brane.find_stmt_index(&stmt)?;
        // `checked_sub` (not `saturating_sub`): index 0 has no preceding
        // range — return None rather than searching [0, 0] and self-matching
        // (as in the sibling `_ib_search`).
        let search_end = idx.checked_sub(1)?;
        // FOOP-55 §6: the candidate window is the home brane's statements
        // BEFORE this one — [0, idx-1] — walked in the direction this search
        // asks for. `?` (backward) finds the nearest preceding match; `~`
        // (forward) the earliest. Same window, opposite direction; neither
        // looks forward into statements that have not settled.
        let mut nav = BraneNavigator::new(&brane, self.forward);
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
                            // The scan decided every candidate and none matched
                            // (FOOP-55 §8). Distinct from NkStop, which never
                            // scanned.
                            self.exhausted.set(true);
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
                if let Some(nyes) = self.on_value_search_op_ready(scope) {
                    self.core.set_nyes(nyes);
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// FOOP-55 §11: moved from `value_search_step`'s `Braning` arm,
    /// converting every `self.core.set_nyes(X); return Ok(())` into
    /// `Some(X)`, and every "still waiting" `return Ok(())` into `None`.
    /// A call to `handle_found` already commits its own NYES internally
    /// (it stays Braning to drain the freshly-pushed ubc_child on the next
    /// step), so those paths report `None` after calling it — there is
    /// nothing further for the shared caller in `fir_op_step` to commit.
    ///
    /// Also applies the `is_found()`-based fix (FOOP-55 §11, human,
    /// 2026-08-25: "all searches should be like that") to the
    /// anchored-and-not-contexted branch: the anchor's own NYES may be
    /// permanently WOCONSTANIC while still holding a real, final resolved
    /// value, so wait on `is_found()` first before falling back to
    /// `is_constanic()`.
    fn on_value_search_op_ready(&self, scope: &Scope) -> Option<Nyes> {
        use contextful_search::{BraneNavigator, contextful_search_scan};
        if !self.core.ubc_children().is_empty() {
            self.settle_from_ubc_result();
            return None;
        }
        if !self.check_value_pattern_ready() {
            return None;
        }
        let predicate = self.build_value_predicate().expect("checked ready");
        let scan_outcome = if self.contexted && self.anchored {
            let anchor = Rc::clone(&self.core.foolish_children()[0]);
            let anchor_settled =
                anchor.borrow().is_found() || anchor.borrow().core().get_nyes().is_constanic();
            match self.contexted_search_from_anchor(scope) {
                Some((stmt, nyes)) => {
                    self.handle_found(stmt, nyes, scope);
                    return None;
                }
                None => {
                    if !anchor_settled {
                        return None;
                    }
                    ScanOutcome::Miss
                }
            }
        } else if self.anchored {
            let anchor = Rc::clone(&self.core.foolish_children()[0]);
            let resolved = anchor.resolve_anchor();
            let resolved_nyes = resolved.borrow().core().get_nyes();
            if resolved_nyes == Nyes::Nk {
                return Some(Nyes::Nk);
            }
            // FOOP-55 §5.5: `is_constanic_branelike`/scanning answer
            // a CONTENT question about `resolved` and require its
            // search context constanic -- a pre-constanic `resolved`
            // (e.g. a BraneConcatOpFir still Prembrionic) has not
            // been driven through its own fir_op_step gate yet, so
            // asking it now would reach the same premature-populate
            // path D10 named. Stay waiting instead of scanning.
            //
            // FOOP-55 §11 (human, 2026-08-25): but a permanently
            // WOCONSTANIC anchor (found a statement whose own value
            // never resolves) never reaches `is_constanic()` even
            // though it has a real, final resolved brane -- so try
            // `anchor.borrow().is_found()` first.
            if !anchor.borrow().is_found() && !resolved_nyes.is_constanic() {
                return None;
            }
            if !resolved.borrow().is_constanic_branelike() {
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
                None
            }
            ScanOutcome::NkStop => Some(Nyes::Nk),
            ScanOutcome::Miss => {
                // The scan decided every candidate and none matched
                // (FOOP-55 §8). NkStop above never scanned, so it does
                // NOT set this — that is the distinction @ reads back.
                self.exhausted.set(true);
                Some(if self.anchored {
                    Nyes::Nk
                } else {
                    Nyes::Econstanic
                })
            }
        }
    }
}

impl Fir for SearchFir {
    #[inline(always)]
    fn core(&self) -> &ProtoBrane {
        &self.core
    }
    fn fir_op_step(&self, scope: &Scope) -> Result<(), UbcError> {
        self._deprecating_op_step(scope)
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

    fn is_found(&self) -> bool {
        self.found_context.borrow().is_some()
    }

    fn found_context_index(&self) -> Option<usize> {
        self.found_context.borrow().as_ref().map(|(_, idx)| *idx)
    }

    /// FOOP-55 D9 item 1/Step 6, exact algorithm from FOOP-55.md §D9:
    /// `true` the moment this search (or the search it chains to) is
    /// `ECONSTANIC`; `false` once the chain bottoms out in a non-search
    /// value or an as-yet-unpopulated result.
    fn terminates_econstanic(&self) -> bool {
        if self.core.get_nyes() == Nyes::Econstanic {
            return true;
        }
        let Some(next) = self.core.ubc_children().into_iter().next() else {
            return false;
        };
        if next.borrow().kind() != FirKind::Search {
            return false;
        }
        next.borrow().terminates_econstanic()
    }

    fn candidates_exhausted(&self) -> bool {
        self.exhausted.get()
    }
    fn as_search_contexted(&self) -> bool {
        self.contexted
    }
    fn set_contexted(&mut self, contexted: bool) {
        self.contexted = contexted;
    }
}

impl SearchFir {
    /// FOOP-55 §11 migration: pre-migration `fir_op_step` body, moved here
    /// verbatim (rename only, no logic change). Delete once every piece has
    /// moved into named handlers and `fir_op_step` no longer calls it.
    fn _deprecating_op_step(&self, scope: &Scope) -> Result<(), UbcError> {
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
                if let Some(nyes) = self.on_foolish_op_ready(scope) {
                    self.core.set_nyes(nyes);
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// FOOP-55 §11: moved from `_deprecating_op_step`'s (plain, non-value)
    /// `Braning` arm, converting every `self.core.set_nyes(X)` with no
    /// further code in that arm into `Some(X)`, the implicit "do nothing"
    /// no-op arm into `None`, and a call to `handle_found` (which commits
    /// its own NYES internally) into `handle_found(...); None`.
    ///
    /// Also applies the `is_found()`-based fix (FOOP-55 §11, human,
    /// 2026-08-25: "all searches should be like that") to the
    /// anchored-and-not-contexted branch, matching
    /// `on_value_search_op_ready` and `SearchPositionFir`'s `@`: the
    /// anchor's own NYES may be permanently WOCONSTANIC while still
    /// holding a real, final resolved value.
    fn on_foolish_op_ready(&self, scope: &Scope) -> Option<Nyes> {
        use contextful_search::{
            BraneNavigator, SearchPredicate, contextful_search_scan_no_body_check,
        };
        if !self.core.ubc_children().is_empty() {
            self.settle_from_ubc_result();
            return None;
        }
        if self.contexted && self.anchored {
            let result = self.contexted_search_from_anchor(scope);
            return match result {
                Some((stmt, nyes)) => {
                    self.handle_found(stmt, nyes, scope);
                    None
                }
                None => Some(if self.anchored {
                    Nyes::Nk
                } else {
                    Nyes::Econstanic
                }),
            };
        }
        if self.anchored {
            let anchor = Rc::clone(&self.core.foolish_children()[0]);
            let resolved = anchor.resolve_anchor();
            let resolved_nyes = resolved.borrow().core().get_nyes();
            if resolved_nyes == Nyes::Nk {
                return Some(Nyes::Nk);
            }
            // FOOP-55 §5.5: `resolved` has not been driven through its own
            // fir_op_step gate yet -- its brane-likeness is not yet
            // knowable, not "no". Stay Braning and try again once it
            // settles.
            //
            // FOOP-55 §11 (human, 2026-08-25): but a permanently
            // WOCONSTANIC anchor (found a statement whose own value never
            // resolves) never reaches `is_constanic()` even though it has
            // a real, final resolved brane -- so try `is_found()` first.
            if !anchor.borrow().is_found() && !resolved_nyes.is_constanic() {
                return None;
            }
            if !resolved.borrow().is_constanic_branelike() {
                return Some(Nyes::Nk);
            }
            let mut nav = BraneNavigator::new(&resolved, self.forward);
            let predicate = SearchPredicate::Name {
                pattern: self.pattern.clone(),
            };
            return match contextful_search_scan_no_body_check(&mut nav, &predicate) {
                ScanOutcome::Found(stmt) => {
                    let nyes = stmt.borrow().core().get_nyes();
                    self.handle_found(stmt, nyes, scope);
                    None
                }
                _ => Some(Nyes::Nk),
            };
        }
        let result = self.ab_search_with_engine(scope);
        match result {
            Some((stmt, nyes)) => {
                self.handle_found(stmt, nyes, scope);
                None
            }
            None => Some(Nyes::Econstanic),
        }
    }
}

#[derive(Debug)]
pub struct IndexFir {
    pub(crate) core: ProtoBrane,
    pub(crate) offset: i32,
    /// FOOP-55 §8: `#(expr)` — a COMPUTED index. When set, this is the index
    /// search's **second dependency** (the anchor is the first): the operand
    /// must settle to an integer before navigation can happen, and its value
    /// replaces `offset`. `None` for the ordinary literal form, which keeps
    /// `tbl#1+1` parsing as `(tbl#1)+1`.
    pub(crate) index_expr: Option<FirRef>,
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
            index_expr: None,
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

impl IndexFir {
    /// The offset to navigate by: the computed operand's value when `#(expr)`
    /// was written, otherwise the literal `offset` (FOOP-55 §8).
    ///
    /// `None` means a computed operand that did not settle to an integer —
    /// there is no position to navigate to, so the caller settles NK.
    fn effective_offset(&self) -> Option<i32> {
        match &self.index_expr {
            None => Some(self.offset),
            Some(ix) => ix.value().borrow().as_i64().map(|v| v as i32),
        }
    }
}

impl Fir for IndexFir {
    #[inline(always)]
    fn core(&self) -> &ProtoBrane {
        &self.core
    }
    fn fir_op_step(&self, scope: &Scope) -> Result<(), UbcError> {
        self._deprecating_op_step(scope)
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

    /// FOOP-55 §11: moved from `_deprecating_op_step`'s `Braning` arm
    /// (after the drain check), converting every `set_nyes` + early
    /// `return Ok(())` into `Some(nyes)`, and the "still waiting, do
    /// nothing" `return Ok(())` into `None`. Two internal modes, matching
    /// `IndexFir`'s two anchored-search shapes (contexted-from-anchor vs
    /// plain-anchored) — see each inline comment for its own readiness
    /// question.
    fn on_foolish_op_ready(&self, _scope: &Scope) -> Option<Nyes> {
        use contextful_search::{
            BraneNavigator, SearchPredicate, contextful_search_scan_no_body_check,
        };
        if self.contexted && self.anchored {
            let anchor = Rc::clone(&self.core.foolish_children()[0]);
            let fool_ref_fir = {
                let borrowed = anchor.borrow();
                borrowed.core().ubc_children().get(1).cloned()
            };
            let contexted_result = fool_ref_fir.and_then(|frf| {
                let referent = frf.borrow().as_fool_ref_referent().cloned()?;
                let h_brane = referent.borrow()._get_my_brane(&referent)?;
                let p = h_brane.find_stmt_index(&referent)?;
                // A computed index (FOOP-55 §8) that did not settle to
                // an integer has no navigable position; this closure
                // reports "no result" with None.
                let effective_offset = self.effective_offset()?;
                let target = p as i32 + effective_offset;
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
                let clone = ProtoBrane::constanic_clone(
                    &body,
                    &self_weak,
                    0,
                    false,
                    OpInstructions::Normal,
                );
                push_search_result_pair(&self.core, clone, stmt);
                return Some(Nyes::Braning);
            } else if !anchor.borrow().core().get_nyes().is_constanic() {
                // Still waiting on our one dependency (the anchor).
                return None;
            } else {
                return Some(Nyes::Nk);
            }
        }
        if self.anchored {
            let anchor = Rc::clone(&self.core.foolish_children()[0]);
            let resolved = anchor.resolve_anchor();
            if !resolved.borrow().core().get_nyes().is_constanic() {
                // FOOP-55 §5.5: `resolved` has not been driven
                // through its own fir_op_step gate yet -- its
                // brane-likeness is not yet knowable, not "no" (and
                // therefore not grounds for the permanent NK below).
                return None;
            }
            if !resolved.borrow().is_constanic_branelike() {
                // FOOP-75 §7: settling NK is only half the answer —
                // record WHY. An anchored search demands its anchor
                // resolve *through* to a brane (AGENTS.md §Searches);
                // when it does not, name the offending value so the
                // rendered output reads
                //     d =$ ??? (4 is not a brane)
                // rather than a bare `d =$ 4 (???)`, which says the
                // result is unknown without saying what went wrong.
                // Diagnose only when the offending anchor can be
                // NAMED — an integer literal, as in `d =$ 4`. Then
                // the reason travels as the search's RESULT (the
                // sequencer renders that; `alarm_reason` alone is
                // never read on the rendering path), giving
                //     d =$ ??? (4 is not a brane)
                //
                // When the anchor is some other FIR — commonly another
                // search that itself settled NK — there is no value to
                // name, and `"<kind> is not a brane"` would report an
                // interpreter type rather than anything the Foolisher
                // wrote. Leaving the result unset keeps the existing
                // rendering, which shows the failed anchor itself:
                //     also_not_found =^ ?(pattern='^z$', ANCHORED, NK) (???)
                let named = resolved.borrow().as_i64().map(|v| v.to_string());
                if let Some(shown) = named {
                    let reason = format!("{} is not a brane", shown);
                    let self_weak = self.core.parent_weak();
                    let nk_ref = NkFir::nk(&reason, self_weak.clone());
                    nk_ref.borrow().core().set_nyes(Nyes::Nk);
                    self.core.push_ubc_child(ProtoBrane::constanic_clone(
                        &nk_ref,
                        &self_weak,
                        0,
                        false,
                        OpInstructions::Normal,
                    ));
                    self.core.set_alarm_reason(reason);
                }
                return Some(Nyes::Nk);
            }
            let mut nav = BraneNavigator::new(&resolved, true);
            let Some(effective_offset) = self.effective_offset() else {
                // A computed index (FOOP-55 §8) that did not settle to an
                // integer has no navigable position.
                return Some(Nyes::Nk);
            };
            let predicate = SearchPredicate::Index(effective_offset);
            return match contextful_search_scan_no_body_check(&mut nav, &predicate) {
                ScanOutcome::Found(stmt) => {
                    let body = statement_value_for_comparison(&stmt);
                    match body {
                        Some(body) => {
                            let self_weak = self.core.parent_weak();
                            let clone = ProtoBrane::constanic_clone(
                                &body,
                                &self_weak,
                                0,
                                false,
                                OpInstructions::Normal,
                            );
                            push_search_result_pair(&self.core, clone, stmt);
                            Some(Nyes::Braning)
                        }
                        None => Some(Nyes::Nk),
                    }
                }
                _ => Some(Nyes::Nk),
            };
        }
        Some(Nyes::Nk)
    }
}

impl IndexFir {
    /// FOOP-55 §11 migration: pre-migration `fir_op_step` body, moved here
    /// verbatim (rename only, no logic change). Delete once every piece has
    /// moved into named handlers and `fir_op_step` no longer calls it.
    fn _deprecating_op_step(&self, scope: &Scope) -> Result<(), UbcError> {
        match self.core.get_nyes() {
            Nyes::Prembrionic | Nyes::Embryonic => {
                if self.anchored {
                    let anchor = Rc::clone(&self.core.foolish_children()[0]);
                    self.core.push_task(anchor);
                    // FOOP-55 §8: a computed index is a SECOND dependency —
                    // enqueue it alongside the anchor so both settle before
                    // navigation. Not a new evaluation phase; just another task.
                    if let Some(ix) = self.index_expr.clone() {
                        self.core.push_task(ix);
                    }
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
                                let Some(effective_offset) = self.effective_offset() else {
                                    // A computed index that did not settle to an integer has no
                                    // navigable position (FOOP-55 §8).
                                    self.core.set_nyes(Nyes::Nk);
                                    return Ok(());
                                };
                                let target = idx as i32 + effective_offset;
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
                                                    let clone = ProtoBrane::constanic_clone(
                                                        &body,
                                                        &self_weak,
                                                        0,
                                                        false,
                                                        OpInstructions::Normal,
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
                } else if let Some(nyes) = self.on_foolish_op_ready(scope) {
                    self.core.set_nyes(nyes);
                }
            }
            _ => {}
        }
        Ok(())
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

/// UFM — the Unstay Foolishness Mark, `<<<…>>>` (FOOP-55 Phase 3J).
///
/// A **mark** to the Foolisher; an **operator** to the evaluator. It owns its
/// content in `foolish_children`, waits for that to go constanic, then
/// constanic-clones it into `ubc_children` stripping **every** SF/SFF layer
/// below (`OpInstructions::InsideUfm`, an unlimited strip budget) and lets the
/// result step again.
///
/// Where `<<x>>` removes ONE layer of detachment, `<<<x>>>` removes ALL of
/// them, on every path. It undoes SFF's compile-time detachment without
/// needing a compile-time half: SFF's detachment lives entirely in "this
/// search was born ECONSTANIC", and `Nyes::transform_for_clone(false)` maps
/// `Econstanic → Embryonic`, so re-birthing the content EMBRYONIC undoes it.
/// The governing principle stays intact — **SF and UFM affect STEPPING; SFF
/// detaches during COMPILATION.**
///
/// The UFM does not survive its own clone: it is consumed by producing its
/// result, like any other operator.
#[derive(Debug)]
pub struct UfmFir {
    pub(crate) core: ProtoBrane,
}

impl UfmFir {
    pub fn ufm(expr: FirRef, parent: Weak<RefCell<dyn Fir>>) -> FirRef {
        Rc::new(RefCell::new(UfmFir {
            core: ProtoBrane::new(vec![expr], parent, Nyes::Prembrionic),
        }))
    }
}

impl Fir for UfmFir {
    #[inline(always)]
    fn core(&self) -> &ProtoBrane {
        &self.core
    }

    fn fir_op_step(&self, scope: &Scope) -> Result<(), UbcError> {
        match self.core.get_nyes() {
            Nyes::Prembrionic | Nyes::Embryonic => {
                self.core.set_nyes(Nyes::Braning);
                for child in self.core.foolish_children() {
                    self.core.push_task(Rc::clone(child));
                }
            }
            Nyes::Braning => {
                // FOOP-55 §11 event-driven shape, same as BraneConcatOpFir:
                // pure orchestration. Ask each phase; commit what it reports;
                // `None` means "still working".
                if let Some(nyes) = self.on_foolish_op_ready(scope) {
                    self.core.set_nyes(nyes);
                    return Ok(());
                }
                if let Some(nyes) = self.on_ubc_op_ready(scope) {
                    self.core.set_nyes(nyes);
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// The UFM's readiness gate: ANY constanic content is ready.
    ///
    /// The default `are_foolish_children_ready_for_op` demands `constantew`
    /// (`Constant|Independent|Nk`), which a marked body never reaches: a
    /// nested `<<a>>` settles WOCONSTANIC precisely BECAUSE its mark is
    /// deferring the search. Waiting for it to resolve on its own is waiting
    /// for the thing this operator exists to do, so the UFM would never fire
    /// and the evaluator would spin to ITERATION-EXCEEDED. Stripping is what
    /// unblocks it, so "constanic" is the right bar here.
    fn are_foolish_children_ready_for_op(&self) -> bool {
        self.core
            .foolish_children()
            .iter()
            .all(|c| c.borrow().core().get_nyes().is_constanic())
    }

    /// Phase 1 — the content has gone constanic: strip-clone it.
    ///
    /// Reports `None` throughout: this phase never settles the UFM. Its job
    /// is to produce the stripped clone; `push_ubc_child` auto-enqueues that
    /// clone (born EMBRYONIC by `transform_for_clone`), so the re-step is
    /// free and phase 2 settles from the drained result.
    fn on_foolish_op_ready(&self, _scope: &Scope) -> Option<Nyes> {
        if !self.are_foolish_children_ready_for_op() {
            return None;
        }
        // Only strip once -- a second pass would re-clone the already-stripped
        // result and undo nothing, but would reset its NYES again and spin.
        if !self.core.ubc_children().is_empty() {
            return None;
        }
        let content = self.core.foolish_children().first().cloned()?;
        let self_weak = self.core.parent_weak();
        self.core.push_ubc_child(ProtoBrane::constanic_clone(
            &content,
            &self_weak,
            0,
            false,
            OpInstructions::InsideUfm,
        ));
        None
    }

    /// Phase 2 — the stripped clone has re-stepped: settle from it.
    fn on_ubc_op_ready(&self, _scope: &Scope) -> Option<Nyes> {
        if self.core.ubc_children().is_empty() || !self.are_ubc_children_ready_for_op() {
            return None;
        }
        let result_nyes = self
            .core
            .ubc_children()
            .into_iter()
            .next()?
            .borrow()
            .core()
            .get_nyes();
        // Same rule as SFF/SearchFir: the wrapper is not itself a search, so
        // it cannot be ECONSTANIC -- an ECONSTANIC result means it is WAITING
        // on that search.
        Some(SearchFir::nyes_from_found(result_nyes))
    }

    fn kind(&self) -> FirKind {
        FirKind::Ufm
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

    fn is_constanic_branelike(&self) -> bool {
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

/// How a concatenation was spelled in source.
/// Affects SEQUENCING ONLY — never evaluation (FOOP-65 §5.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConcatProvenance {
    /// Ordinary brane concatenation (juxtaposition): `{a}{b}{c}`.
    Juxtaposition,
    /// Tail concatenation (backtick chain): `` c`b`a `` — the elements are
    /// already stored REVERSED relative to source (FOOP-65 §5.2).
    TailConcatenation,
}

/// The brane-concatenation OPERATOR (`a b`, and the backtick tail form).
///
/// Named `BraneConcatOpFir`, not `ConcatenationFir`: a concatenation is an
/// **operator**, not a brane (FOOP-55 §10). It takes brane-like INPUTS in
/// `foolish_children` and PRODUCES a joined result — the `_ConcatHelper` in
/// `ubc_children`, which is the brane. Callers reach that result through
/// `.value()`, never by asking this node for statements: it deliberately
/// does not implement `stmt_count`/`stmt_at`.
///
/// Stepping is fully event-driven (FOOP-55 §11): `fir_op_step` is pure
/// orchestration, and the two phases live in the two handlers —
/// [`Fir::on_foolish_op_ready`] decides element readiness and, on the step
/// it first passes, builds the helpers; [`Fir::on_ubc_op_ready`] settles
/// once those helpers have drained. The dequeue gate,
/// [`Fir::is_foolish_child_constanic_enough`], is overridden so an element
/// whose search chain bottoms out `ECONSTANIC` keeps waiting rather than
/// being treated as done.
#[derive(Debug)]
pub struct BraneConcatOpFir {
    pub(crate) core: ProtoBrane,
    pub(crate) _helpers_populated: std::cell::Cell<bool>,
    /// Provenance: how this concatenation was spelled in source.
    /// Affects SEQUENCING ONLY — never evaluation (FOOP-65 §5.3).
    pub(crate) provenance: ConcatProvenance,
}

impl BraneConcatOpFir {
    pub fn concatenation(elements: Vec<FirRef>, parent: Weak<RefCell<dyn Fir>>) -> FirRef {
        Rc::new(RefCell::new(BraneConcatOpFir {
            core: ProtoBrane::new(elements, parent, Nyes::Prembrionic),
            _helpers_populated: std::cell::Cell::new(false),
            provenance: ConcatProvenance::Juxtaposition,
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
    /// FOOP-55 §10: ALWAYS builds and pushes a `_ConcatHelper`, even when
    /// there are zero elements or every element is an empty brane (zero
    /// total lines). A settled `BraneConcatOp`'s RESULT (`.value()`, reached
    /// via `settled_result()`'s universal default of "first `ubc_children`
    /// entry") must be a real `ConcatHelper` in every case where the
    /// concatenation settles successfully — an empty result is still a
    /// result, distinct from "not yet joined". Without this, an empty
    /// concatenation would settle constanic with `ubc_children` empty,
    /// making `.value()` indistinguishable from the pre-constanic case and
    /// losing the "I settled to an empty brane" fact entirely.
    fn populate_concat_helpers(&self) {
        let self_weak = self.core.parent_weak();
        let elements = self.core.foolish_children();

        let total_lines: usize = elements
            .iter()
            .map(|e| e.value().borrow().stmt_count().unwrap_or(0))
            .sum();

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
                    let clone = ProtoBrane::constanic_clone(
                        &stmt,
                        &helper_weak,
                        global_idx,
                        false,
                        OpInstructions::Normal,
                    );
                    Self::apply_null_const_rule_to_merged_stmt(&clone, &cloned_stmts);
                    cloned_stmts.push(clone);
                    global_idx += 1;
                }
            }
        }

        // Even when `cloned_stmts` is empty, replace the helper's core so it
        // is born with the correct (empty) Nyes progression rather than
        // sitting at Prembrionic with no children forever — an empty
        // ProtoBrane still needs to be stepped to Constant.
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

impl Fir for BraneConcatOpFir {
    #[inline(always)]
    fn core(&self) -> &ProtoBrane {
        &self.core
    }
    fn fir_op_step(&self, scope: &Scope) -> Result<(), UbcError> {
        match self.core.get_nyes() {
            Nyes::Prembrionic | Nyes::Embryonic => {
                // FOOP-55 §10: the zero-elements case is NOT special-cased
                // here anymore -- it transitions to Braning like any other
                // concatenation (there is simply nothing to push as a task),
                // so the Braning arm's own populate_concat_helpers() runs and
                // builds a real (empty) ConcatHelper. This is what makes
                // .value() on an empty, settled concatenation reach a
                // genuine result instead of being indistinguishable from
                // "not yet settled".
                self.core.set_nyes(Nyes::Braning);
                for child in self.core.foolish_children() {
                    self.core.push_task(Rc::clone(child));
                }
            }
            Nyes::Braning => {
                // FOOP-55 §11, event-driven shape: the two handlers ARE the
                // two phases. `on_foolish_op_ready` reports a NYES while the
                // foolish_children phase is still deciding (and performs its
                // own phase-completion work — populating the helpers — on the
                // step it finds every element ready); `on_ubc_op_ready`
                // reports once the ubc_children phase has drained. Neither
                // reporting a NYES means "still working", so this arm is pure
                // orchestration: ask, and commit whatever is reported.
                if let Some(nyes) = self.on_foolish_op_ready(scope) {
                    self.core.set_nyes(nyes);
                    return Ok(());
                }
                if let Some(nyes) = self.on_ubc_op_ready(scope) {
                    self.core.set_nyes(nyes);
                }
            }
            _ => {}
        }
        Ok(())
    }
    fn kind(&self) -> FirKind {
        FirKind::Concatenation
    }
    fn as_concat_provenance(&self) -> ConcatProvenance {
        self.provenance
    }

    fn is_constanic_branelike(&self) -> bool {
        true
    }

    /// FOOP-55 D9 item 2/Step 6: `step_inner`'s dequeue gate for this
    /// kind's `foolish_children` (the SF-wrapped element searches). The
    /// default (plain `is_constanic()`) would let a search whose found
    /// content may still resolve via recoordination (D9) dequeue and keep
    /// STEPPING on its own — running that found content's nested marks all
    /// the way to a premature, wrong-context resolution — before this
    /// concatenation's own `on_foolish_op_ready` gate ever gets a turn to
    /// decide the element's real, merged position. Ordinarily constanic,
    /// **and** not a search chain that bottoms out in `ECONSTANIC`
    /// (`terminates_econstanic()`) — that combination alone is what keeps
    /// waiting here. Specific to concatenation; the default stays
    /// `is_constanic()` everywhere else.
    fn is_foolish_child_constanic_enough(&self, child: &FirRef) -> bool {
        let is_constanic = child.borrow().core().get_nyes().is_constanic();
        let is_terminating_search =
            child.borrow().kind() == FirKind::Search && child.borrow().terminates_econstanic();
        is_constanic && !is_terminating_search
    }

    /// The `foolish_children` phase: decide whether the elements are ready
    /// to join, and — on the step they first are — perform this phase's own
    /// completion work by building the `_ConcatHelper`s.
    ///
    /// One pass over the elements accumulates two verdicts:
    ///  - `all_brane_like`: every value is a brane (can be iterated and
    ///    copied — true for any NYES, incl. WOCONSTANIC/NK).
    ///  - `type_errors`: indices of permanent non-branes (constantew but
    ///    not brane-like) — genuine errors, all reported.
    ///
    /// Returns `Some(Nk)` on a type error (wins over "not ready yet" — a
    /// real bad element is not masked by another still resolving),
    /// `Some(Woconstanic)` if not all elements are brane-like yet (not a
    /// greedy join — the sequencer renders the raw un-joined elements), or
    /// `None` once every element is cleanly brane-like. That `None` is what
    /// hands control to the `ubc_children` phase, so it is also where the
    /// helpers get built and pushed as tasks: `self` deliberately stays
    /// pre-constanic so the driver drains them before re-entry, and
    /// `_helpers_populated` makes the build happen exactly once.
    fn on_foolish_op_ready(&self, _scope: &Scope) -> Option<Nyes> {
        let mut all_brane_like = true;
        let mut type_errors: Vec<usize> = Vec::new();
        for (idx, elem) in self.core.foolish_children().iter().enumerate() {
            let brane_like = elem.value().borrow().is_constanic_branelike();
            all_brane_like &= brane_like;
            if !brane_like && elem.borrow().core().get_nyes().is_constantew() {
                type_errors.push(idx);
            }
        }

        if !type_errors.is_empty() {
            let self_weak = self.core.parent_weak();
            let list = type_errors
                .iter()
                .map(usize::to_string)
                .collect::<Vec<_>>()
                .join(",");
            let nk: FirRef = Rc::new(RefCell::new(NkFir {
                core: ProtoBrane::new(vec![], self_weak, Nyes::Nk),
                reason: format!("concatenation constituent indexes where it's not a brane: {list}"),
            }));
            self.core.push_ubc_child(nk);
            return Some(Nyes::Nk);
        }
        if !all_brane_like {
            return Some(Nyes::Woconstanic);
        }

        // Foolish phase complete. Build the helpers once, push them as
        // tasks, and report "not settling yet" so the driver drains them.
        if !self._helpers_populated.get() {
            self._helpers_populated.set(true);
            self.populate_concat_helpers();
            for helper in self.core.ubc_children() {
                self.core.push_task(helper);
            }
        }
        None
    }

    /// The `ubc_children` phase: settle from the JOINED lines (the
    /// helpers), not the elements — the recoordinated joined copies can be
    /// constant even when the original element brane was WOCONSTANIC (e.g.
    /// `{c=a+b}` → joined `c=3`). Empty (no lines joined) → Constant, per
    /// the empty-brane convention.
    ///
    /// Reports `None` while the helpers have not drained yet
    /// ([`Fir::are_ubc_children_ready_for_op`]): the step that builds them
    /// reaches here immediately, and `self` must stay pre-constanic until
    /// the driver has actually run them.
    fn on_ubc_op_ready(&self, _scope: &Scope) -> Option<Nyes> {
        if !self.are_ubc_children_ready_for_op() {
            return None;
        }
        let helpers = self.core.ubc_children();
        Some(if helpers.is_empty() {
            Nyes::Constant
        } else {
            _decide_nyes_due_to_children(&helpers).unwrap_or(Nyes::Constant)
        })
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
        index_expr: None,
        anchored,
        contexted: false,
    }))
}

pub fn concatenation(elements: Vec<FirRef>, parent: Weak<RefCell<dyn Fir>>) -> FirRef {
    Rc::new(RefCell::new(BraneConcatOpFir {
        core: ProtoBrane::new(elements, parent, Nyes::Prembrionic),
        _helpers_populated: std::cell::Cell::new(false),
        provenance: ConcatProvenance::Juxtaposition,
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
    /// **Two conditions, both required** (revised 2026-08-04 — see FOOP-33.md
    /// §"Concerns Standing Past Completion" for the incident that forced this):
    ///
    /// 1. **Reached somewhere other than its own defining statement.** A
    ///    creation's parent statement is where it was born; reporting that
    ///    same name back AT that statement reads as self-referential
    ///    (`{a = ⬤;}` must NOT sequence as `{a=a}` — that looks like a
    ///    tautology or a bug, not "a fresh creation is being introduced").
    ///    Only a reference reached elsewhere (through search, as another
    ///    statement's value) reports the name; the defining site itself
    ///    always shows the glyph. `viewed_from` is the statement CURRENTLY
    ///    being rendered (the one whose body led the caller to this
    ///    creation) — compared by `Rc::ptr_eq` against the creation's own
    ///    recorded parent statement. They are the same `Rc` only when we are
    ///    looking at the creation from its own defining statement (Gotcha
    ///    #2 means the creation itself is one shared `Rc` everywhere, so this
    ///    check must be on the STATEMENT, not the creation).
    /// 2. **The defining statement's name is null-characterized.** A bare
    ///    `a = ⬤` does not qualify — only a protected constant like
    ///    `'True = ⬤` does. Without this gate, two independent plain
    ///    creations sharing unrelated statement names read as far more
    ///    confusing once rendered: `{'a=⬤; 'a=⬤;}` (two DIFFERENT creations
    ///    that both happen to sit under the coordinate name `a`, e.g. inside
    ///    different branes) would sequence as `{'a='a; 'a='a;}` with no way
    ///    to tell from the rendering that they are not the same creation.
    ///    Restricting to null-characterized names limits this rendering to
    ///    the case it was actually designed for: protected, effectively
    ///    singleton constants like `'True`/`'False`, where the name really
    ///    does uniquely pick out one creation.
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
    #[must_use]
    pub fn get_display_name(&self, self_ref: &FirRef, viewed_from: &FirRef) -> Option<String> {
        let parent = self.core.parent()?;
        // A self-parenting node is the root; it has no defining statement.
        if Rc::ptr_eq(&parent, self_ref) {
            return None;
        }
        let parent_borrowed = parent.borrow();
        // `as_stmt_identifier` is `None` for every non-statement kind, so it
        // doubles as the "is this a statement?" discriminator.
        let identifier = parent_borrowed.as_stmt_identifier()?;
        let body = parent_borrowed.core().foolish_children().first()?;
        if !Rc::ptr_eq(body, self_ref) {
            return None;
        }
        // Condition 2: only a null-characterized (protected-constant) name
        // qualifies at all.
        if !identifier.is_nully_characterizing_coordinate_name() {
            return None;
        }
        let name = identifier.searchable_name().to_owned();
        drop(parent_borrowed);
        // Condition 1: never report the name when viewed from the creation's
        // own defining statement -- only from a different statement (a
        // reference reached elsewhere).
        if Rc::ptr_eq(&parent, viewed_from) {
            return None;
        }
        Some(name)
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
    fn as_creation_display_name(
        &self,
        self_ref: &FirRef,
        viewed_from: Option<&FirRef>,
    ) -> Option<String> {
        let viewed_from = viewed_from?;
        self.get_display_name(self_ref, viewed_from)
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
                exhausted: std::cell::Cell::new(false),
                found_context: RefCell::new(None),
            })
        })
    }

    fn make_index(offset: i32, anchored: bool, foolish_children: Vec<FirRef>) -> FirRef {
        Rc::new_cyclic(|me: &Weak<RefCell<IndexFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(IndexFir {
                core: ProtoBrane::new(foolish_children, parent, Nyes::Prembrionic),
                offset,
                index_expr: None,
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
                index_expr: None,
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
        Rc::new_cyclic(|me: &Weak<RefCell<BraneConcatOpFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(BraneConcatOpFir {
                core: ProtoBrane::new(elements, parent, Nyes::Prembrionic),
                _helpers_populated: std::cell::Cell::new(false),
                provenance: ConcatProvenance::Juxtaposition,
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

    /// FOOP-55 §10: an empty concatenation still produces a real (empty)
    /// `ConcatHelper` as its settled result -- `populate_concat_helpers`
    /// always pushes one, even with zero total lines, precisely so `.value()`
    /// on a settled-but-empty concatenation reaches a genuine result instead
    /// of being indistinguishable from "not yet settled" (which would happen
    /// if `ubc_children` stayed empty on settlement).
    #[test]
    fn concatenation_empty_elements_is_constant_empty_brane() {
        let cat = make_concatenation(vec![]);
        let scope = Scope::empty();

        let transitions = step_to_settled(&cat, &scope);
        eprintln!("Concatenation(empty) NYES transitions: {transitions:?}");

        assert_eq!(cat.borrow().core().get_nyes(), Nyes::Constant);
        let ubc = cat.borrow().core().ubc_children().to_vec();
        assert_eq!(
            ubc.len(),
            1,
            "even an empty join produces a real ConcatHelper"
        );
        assert_eq!(ubc[0].borrow().kind(), FirKind::ConcatHelper);
        assert_eq!(ubc[0].borrow().core().get_nyes(), Nyes::Constant);

        let result = cat.value();
        assert!(
            Rc::ptr_eq(&result, &ubc[0]),
            "value() must reach the empty ConcatHelper, not the operator"
        );
        assert_eq!(result.borrow().stmt_count(), Some(0));
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

        let cloned = ProtoBrane::_inner_constanic_clone(
            &op,
            &dangling_parent(),
            0,
            false,
            false,
            StripBudget::fresh(),
        );

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

        let cloned = ProtoBrane::_inner_constanic_clone(
            &op,
            &dangling_parent(),
            0,
            false,
            false,
            StripBudget::fresh(),
        );

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
        let cloned = ProtoBrane::_inner_constanic_clone(
            &woc,
            &dangling_parent(),
            0,
            true,
            false,
            StripBudget::fresh(),
        );
        assert_eq!(
            cloned.borrow().core().get_nyes(),
            Nyes::Woconstanic,
            "FICC must keep a constanic compound's state verbatim"
        );

        let econ = make_operator("+", vec![make_constant_int(1), make_constant_int(2)]);
        econ.borrow().core().set_nyes(Nyes::Econstanic);
        let cloned = ProtoBrane::_inner_constanic_clone(
            &econ,
            &dangling_parent(),
            0,
            true,
            false,
            StripBudget::fresh(),
        );
        assert_eq!(cloned.borrow().core().get_nyes(), Nyes::Econstanic);
    }

    #[test]
    fn leaf_clone_unchanged_both_modes() {
        let ci = make_constant_int(9);
        ci.borrow().core().set_nyes(Nyes::Constant);
        let n = ProtoBrane::_inner_constanic_clone(
            &ci,
            &dangling_parent(),
            0,
            false,
            false,
            StripBudget::fresh(),
        );
        assert_eq!(n.borrow().core().get_nyes(), Nyes::Constant);
        let n = ProtoBrane::_inner_constanic_clone(
            &ci,
            &dangling_parent(),
            0,
            true,
            false,
            StripBudget::fresh(),
        );
        assert_eq!(n.borrow().core().get_nyes(), Nyes::Constant);

        let nk = make_nk("gone");
        nk.borrow().core().set_nyes(Nyes::Nk);
        let n = ProtoBrane::_inner_constanic_clone(
            &nk,
            &dangling_parent(),
            0,
            false,
            false,
            StripBudget::fresh(),
        );
        assert_eq!(n.borrow().core().get_nyes(), Nyes::Nk);
        let n = ProtoBrane::_inner_constanic_clone(
            &nk,
            &dangling_parent(),
            0,
            true,
            false,
            StripBudget::fresh(),
        );
        assert_eq!(n.borrow().core().get_nyes(), Nyes::Nk);
    }

    #[test]
    fn cloning_sf_strips_the_mark() {
        let inner = make_constant_int(10);
        inner.borrow().core().set_nyes(Nyes::Econstanic); // force the clone path
        let sf = make_stay_foolish(Rc::clone(&inner));
        sf.borrow().core().set_nyes(Nyes::Econstanic);

        let normal = ProtoBrane::_inner_constanic_clone(
            &sf,
            &dangling_parent(),
            0,
            false,
            false,
            StripBudget::fresh(),
        );
        assert_ne!(
            normal.borrow().kind(),
            FirKind::StayFoolish,
            "normal clone of an SF must NOT be a StayFoolish wrapper"
        );
        assert_eq!(normal.borrow().kind(), FirKind::IndepInt);

        let foolish = ProtoBrane::_inner_constanic_clone(
            &sf,
            &dangling_parent(),
            0,
            true,
            false,
            StripBudget::fresh(),
        );
        assert_ne!(
            foolish.borrow().kind(),
            FirKind::StayFoolish,
            "even a foolish clone of an SF strips the wrapper (clones the inner)"
        );
    }

    // ── FOOP-55 D9 fix (2026-08-26): `constanic_clone`/`_inner_constanic_clone`
    // ──────────────────────────────────────────────────────────────────────
    //
    // These pin the NEW public entry point's contract before it exists —
    // `ProtoBrane::constanic_clone` (no `disable_nyes_reset`/`stay_budget`
    // parameters; always starts `stay_budget=1, disable_nyes_reset=false`).
    // They must FAIL TO COMPILE until that function is added, and then pin
    // the corrected behavior once it is. See FOOP-55.md's D9 item 3 for the
    // full spec; FOOP-55.plan.md Phase 3I for the checklist these implement.

    /// A single SF/SFF mark strips cleanly via the new entry point — same
    /// contract `cloning_sf_strips_the_mark` already pins for the old
    /// `constanic_clone_at`, restated against `constanic_clone` so the new
    /// entry point is proven to preserve this baseline behavior.
    #[test]
    fn constanic_clone_strips_a_single_mark() {
        let inner = make_constant_int(10);
        inner.borrow().core().set_nyes(Nyes::Econstanic);
        let sf = make_stay_foolish(Rc::clone(&inner));
        sf.borrow().core().set_nyes(Nyes::Econstanic);

        let cloned =
            ProtoBrane::constanic_clone(&sf, &dangling_parent(), 0, false, OpInstructions::Normal);
        assert_ne!(
            cloned.borrow().kind(),
            FirKind::StayFoolish,
            "constanic_clone must strip a single SF wrapper, same as constanic_clone_at"
        );
        assert_eq!(cloned.borrow().kind(), FirKind::IndepInt);
    }

    /// D9's exact shape: a `constanic_clone` call reaching a mark the CALLER
    /// had nothing to do with (no ambient "foolishly ignorant" state passed
    /// in) must strip that mark and reset its ECONSTANIC content to
    /// EMBRYONIC — regardless of what any OTHER, unrelated mark the calling
    /// context might itself be nested under. This is `constanic_clone`'s
    /// whole point: SF/SFF-ness of the clone TARGET is self-contained to
    /// this call, never inherited from an ambient `Scope`.
    #[test]
    fn constanic_clone_resets_a_fresh_mark_regardless_of_caller_context() {
        // b's own body: <<#-2>>-shaped stand-in — an operator (any compound
        // works; OperatorFir is the simplest available) sitting ECONSTANIC,
        // wrapped in its OWN SFF mark, exactly as `b = <<#-2>>` compiles.
        let compound = make_operator("+", vec![make_constant_int(1), make_constant_int(2)]);
        compound.borrow().core().set_nyes(Nyes::Econstanic);
        let b_body_mark = make_stay_fully_foolish(Rc::clone(&compound));
        b_body_mark.borrow().core().set_nyes(Nyes::Econstanic);

        // The clone call carries NO memory of any other mark — this models
        // `constanic_clone`'s fixed `disable_nyes_reset=false` starting
        // value, independent of whatever ambient SF wrapper a caller (like
        // the search-for-`b` in `c = a b`) happened to be stepping under.
        let cloned = ProtoBrane::constanic_clone(
            &b_body_mark,
            &dangling_parent(),
            0,
            false,
            OpInstructions::Normal,
        );

        assert_ne!(
            cloned.borrow().kind(),
            FirKind::StayFullyFoolish,
            "the mark itself must be stripped"
        );
        assert_eq!(
            cloned.borrow().core().get_nyes(),
            Nyes::Embryonic,
            "D9: a freshly-reached SFF mark's ECONSTANIC content must reset \
             to EMBRYONIC so it re-steps in its new context, never preserved \
             verbatim just because some OTHER mark was in scope somewhere \
             upstream of this call"
        );
    }

    /// Two marks nested directly (a mark wrapping a mark, same path to any
    /// leaf): one `constanic_clone` call strips exactly the OUTER mark and
    /// leaves the inner one intact — `stay_budget=1` is one layer, not
    /// unlimited stripping down a single path.
    #[test]
    fn constanic_clone_strips_exactly_one_nested_layer() {
        let leaf = make_constant_int(5);
        leaf.borrow().core().set_nyes(Nyes::Econstanic);
        let inner_mark = make_stay_fully_foolish(Rc::clone(&leaf));
        inner_mark.borrow().core().set_nyes(Nyes::Econstanic);
        let outer_mark = make_stay_foolish(Rc::clone(&inner_mark));
        outer_mark.borrow().core().set_nyes(Nyes::Econstanic);

        let cloned = ProtoBrane::constanic_clone(
            &outer_mark,
            &dangling_parent(),
            0,
            false,
            OpInstructions::Normal,
        );

        assert_ne!(
            cloned.borrow().kind(),
            FirKind::StayFoolish,
            "the outer mark must be stripped"
        );
        assert_eq!(
            cloned.borrow().kind(),
            FirKind::StayFullyFoolish,
            "the inner mark must survive UNSTRIPPED -- one layer per clone call"
        );
        assert_eq!(
            cloned.borrow().core().get_nyes(),
            Nyes::Econstanic,
            "the still-wrapped inner mark's content must be preserved verbatim \
             (disable_nyes_reset=true for the exhausted-budget clone of the \
             mark itself), not reset -- it has not actually been stripped"
        );
    }

    /// Multiple INDEPENDENT SF/SFF-marked descendants (siblings, not nested
    /// on the same path) — cloning their common, unmarked parent must strip
    /// EACH one's own mark, confirming the budget is per-child-call, never
    /// shared/threaded across siblings from one parent-level budget.
    #[test]
    fn constanic_clone_strips_each_sibling_mark_independently() {
        let left_inner = make_constant_int(1);
        left_inner.borrow().core().set_nyes(Nyes::Econstanic);
        let left_mark = make_stay_foolish(Rc::clone(&left_inner));
        left_mark.borrow().core().set_nyes(Nyes::Econstanic);

        let right_inner = make_constant_int(2);
        right_inner.borrow().core().set_nyes(Nyes::Econstanic);
        let right_mark = make_stay_fully_foolish(Rc::clone(&right_inner));
        right_mark.borrow().core().set_nyes(Nyes::Econstanic);

        let op = make_operator("+", vec![left_mark, right_mark]);
        op.borrow().core().set_nyes(Nyes::Woconstanic);

        let cloned =
            ProtoBrane::constanic_clone(&op, &dangling_parent(), 0, false, OpInstructions::Normal);

        let cloned_children = cloned.borrow().core().foolish_children().to_vec();
        assert_eq!(cloned_children.len(), 2);
        assert_ne!(
            cloned_children[0].borrow().kind(),
            FirKind::StayFoolish,
            "left sibling's own SF mark must be stripped independently"
        );
        assert_ne!(
            cloned_children[1].borrow().kind(),
            FirKind::StayFullyFoolish,
            "right sibling's own SFF mark must be stripped independently, \
             not blocked by the left sibling having already spent a budget"
        );
    }

    /// `OpInstructions::Normal` is the ordinary, unwrapped step: one strip per
    /// path, so the outermost mark comes off and a nested one survives. This
    /// restates `constanic_clone_strips_exactly_one_nested_layer` against the
    /// enum spelling, pinning `Normal` to the historical `inside_sf_mark =
    /// false` behavior.
    #[test]
    fn op_instructions_normal_strips_exactly_one_layer() {
        let leaf = make_constant_int(5);
        leaf.borrow().core().set_nyes(Nyes::Econstanic);
        let inner_mark = make_stay_fully_foolish(Rc::clone(&leaf));
        inner_mark.borrow().core().set_nyes(Nyes::Econstanic);
        let outer_mark = make_stay_foolish(Rc::clone(&inner_mark));
        outer_mark.borrow().core().set_nyes(Nyes::Econstanic);

        let cloned = ProtoBrane::constanic_clone(
            &outer_mark,
            &dangling_parent(),
            0,
            false,
            OpInstructions::Normal,
        );

        assert_eq!(
            cloned.borrow().kind(),
            FirKind::StayFullyFoolish,
            "Normal: outer mark stripped, inner mark survives -- one layer"
        );
    }

    /// `OpInstructions::InsideSfm` continues with a budget of ONE that the
    /// stepper has ALREADY spent — so nothing strips at all and the mark is
    /// preserved verbatim. This is the D9 fix: a search running inside an SF
    /// wrapper copies its found body with its own marks still intact.
    #[test]
    fn op_instructions_inside_sfm_preserves_the_mark() {
        let compound = make_operator("+", vec![make_constant_int(1), make_constant_int(2)]);
        compound.borrow().core().set_nyes(Nyes::Econstanic);
        let mark = make_stay_fully_foolish(Rc::clone(&compound));
        mark.borrow().core().set_nyes(Nyes::Econstanic);

        let cloned = ProtoBrane::constanic_clone(
            &mark,
            &dangling_parent(),
            0,
            false,
            OpInstructions::InsideSfm,
        );

        assert_eq!(
            cloned.borrow().kind(),
            FirKind::StayFullyFoolish,
            "InsideSfm: the mark must survive -- the enclosing SF has not yet \
             decided this content's final position"
        );
        assert_eq!(
            cloned.borrow().core().get_nyes(),
            Nyes::Econstanic,
            "InsideSfm: an unstripped mark's content is preserved verbatim, \
             never reset to EMBRYONIC"
        );
    }

    /// `OpInstructions::InsideUfm` continues with an INFINITE budget: every
    /// mark on the path is stripped, however deeply nested. This is what makes
    /// `<@ … @>` remove ALL SF/SFF layers below it rather than just one.
    #[test]
    fn op_instructions_inside_ufm_strips_every_nested_layer() {
        let leaf = make_constant_int(7);
        leaf.borrow().core().set_nyes(Nyes::Econstanic);
        let innermost = make_stay_fully_foolish(Rc::clone(&leaf));
        innermost.borrow().core().set_nyes(Nyes::Econstanic);
        let middle = make_stay_foolish(Rc::clone(&innermost));
        middle.borrow().core().set_nyes(Nyes::Econstanic);
        let outer = make_stay_fully_foolish(Rc::clone(&middle));
        outer.borrow().core().set_nyes(Nyes::Econstanic);

        let cloned = ProtoBrane::constanic_clone(
            &outer,
            &dangling_parent(),
            0,
            false,
            OpInstructions::InsideUfm,
        );

        assert_eq!(
            cloned.borrow().kind(),
            FirKind::IndepInt,
            "InsideUfm: ALL THREE nested marks must be stripped, exposing the \
             bare leaf -- an unlimited budget, not one layer"
        );
    }

    /// The unlimited budget must not be consumed by breadth either: an
    /// UFM-instructed clone strips each sibling's mark AND keeps stripping
    /// deeper on each path.
    #[test]
    fn op_instructions_inside_ufm_strips_siblings_and_depth_together() {
        let left_leaf = make_constant_int(1);
        left_leaf.borrow().core().set_nyes(Nyes::Econstanic);
        let left_inner = make_stay_foolish(Rc::clone(&left_leaf));
        left_inner.borrow().core().set_nyes(Nyes::Econstanic);
        let left_mark = make_stay_fully_foolish(Rc::clone(&left_inner));
        left_mark.borrow().core().set_nyes(Nyes::Econstanic);

        let right_leaf = make_constant_int(2);
        right_leaf.borrow().core().set_nyes(Nyes::Econstanic);
        let right_mark = make_stay_foolish(Rc::clone(&right_leaf));
        right_mark.borrow().core().set_nyes(Nyes::Econstanic);

        let op = make_operator("+", vec![left_mark, right_mark]);
        op.borrow().core().set_nyes(Nyes::Woconstanic);

        let cloned = ProtoBrane::constanic_clone(
            &op,
            &dangling_parent(),
            0,
            false,
            OpInstructions::InsideUfm,
        );

        let kids = cloned.borrow().core().foolish_children().to_vec();
        assert_eq!(kids.len(), 2);
        assert_eq!(
            kids[0].borrow().kind(),
            FirKind::IndepInt,
            "UFM strips BOTH of the left path's nested marks"
        );
        assert_eq!(
            kids[1].borrow().kind(),
            FirKind::IndepInt,
            "UFM strips the right sibling's mark too"
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

    /// FOOP-55 Phase 3J: the UFM's NYES progression (AGENTS.md requires a
    /// `*_nyes_transitions` test for every FIR kind).
    #[test]
    fn ufm_nyes_transitions() {
        let ufm = UfmFir::ufm(make_constant_int(42), dangling_parent());
        let trace = step_to_settled(&ufm, &Scope::empty());
        // CONSTANT, not INDEPENDENT: the UFM settles via
        // `SearchFir::nyes_from_found` over its stripped result, the same
        // rule SFF uses -- the wrapper reports "I have a value", not "I am
        // self-contained", even when the content itself is independent.
        assert_progression(&trace, Nyes::Constant, "Ufm");
    }

    /// The UFM's defining property: it removes EVERY SF/SFF layer below it,
    /// not just the outermost one. A plain SFF clone would strip one layer
    /// and leave the rest wrapped; the UFM leaves nothing wrapped.
    #[test]
    fn ufm_strips_every_nested_layer() {
        let leaf = make_constant_int(7);
        leaf.borrow().core().set_nyes(Nyes::Econstanic);
        let inner = make_stay_fully_foolish(Rc::clone(&leaf));
        inner.borrow().core().set_nyes(Nyes::Econstanic);
        let middle = make_stay_foolish(Rc::clone(&inner));
        middle.borrow().core().set_nyes(Nyes::Econstanic);
        let outer = make_stay_fully_foolish(Rc::clone(&middle));
        outer.borrow().core().set_nyes(Nyes::Econstanic);

        let ufm = UfmFir::ufm(outer, dangling_parent());
        step_to_settled(&ufm, &Scope::empty());

        let stripped = ufm
            .borrow()
            .core()
            .ubc_children()
            .into_iter()
            .next()
            .expect("UFM must produce a stripped clone");
        assert_eq!(
            stripped.borrow().kind(),
            FirKind::IndepInt,
            "all three nested marks must be gone, exposing the bare leaf"
        );
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

    /// Pins the CURRENT, CORRECT operational semantics of `{a = {1,2},
    /// b=<<#-2>>, c= a b}` (human, 2026-08-26: "D9 behaves correctly as it
    /// adheres to the operational semantics of Foolish at this moment").
    ///
    /// `c`'s `foolish_children` hold `b` as an SF-wrapped search whose
    /// result chain bottoms out `ECONSTANIC`, so `c` keeps waiting on it
    /// rather than joining: a raw, unstripped `#-2` in `ubc_children` does
    /// not mean anything at a time when the concatenation has not yet
    /// happened. `c` therefore stays pre-constanic and the root settles
    /// `WOCONSTANIC`, with `b`'s search result still `<<...>>`-wrapped.
    ///
    /// FOOP-55.md's D9 section carries an illustrative `c = {1,2,1,2}`
    /// target; that target was written before the evaluator was ever run
    /// against this input (the section flags it "illustrative; not yet an
    /// einmo case"). It is NOT what these semantics produce, and this test
    /// pins what they do produce. Revisit only if the semantics themselves
    /// are deliberately changed.
    #[test]
    fn d9_recoordinated_index_holds_b_econstanic_and_does_not_join() {
        let root = Compiler::compile("{a = {1,2}, b=<<#-2>>, c= a b}")
            .unwrap()
            .pop()
            .unwrap();
        let scope = Scope::empty();

        let mut transitions = vec![root.borrow().core().get_nyes()];
        for _ in 0..200 {
            let report = root.step(&scope).unwrap();
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
        assert_eq!(
            *transitions.last().unwrap(),
            Nyes::Woconstanic,
            "root settles WOCONSTANIC: c never joins, because b's element \
             is held on an ECONSTANIC-terminating search chain"
        );

        // `c`'s elements: `a` settles; `b`'s SF wrapper keeps waiting.
        let stmts = root.borrow().core().foolish_children().to_vec();
        let c_stmt = &stmts[2];
        let c_body = c_stmt
            .borrow()
            .core()
            .foolish_children()
            .first()
            .cloned()
            .expect("c has a body");
        assert_eq!(c_body.borrow().kind(), FirKind::Concatenation);
        let elements: Vec<(FirKind, Nyes)> = c_body
            .borrow()
            .core()
            .foolish_children()
            .iter()
            .map(|e| (e.borrow().kind(), e.borrow().core().get_nyes()))
            .collect();
        assert_eq!(
            elements,
            vec![
                (FirKind::StayFoolish, Nyes::Independent),
                (FirKind::StayFoolish, Nyes::Woconstanic),
            ],
            "a's element settles INDEPENDENT; b's element settles WOCONSTANIC \
             -- its search found b, but b's body is an ECONSTANIC-terminating \
             chain, so no join runs"
        );

        // `?b` found `b`, and its result is `b`'s body STILL SFF-wrapped:
        // the mark is not stripped while the join has not happened.
        let search_b = find_search(&root, "^b$").expect("search for b inside c");
        let ubc_b: Vec<(FirKind, Nyes)> = search_b
            .borrow()
            .core()
            .ubc_children()
            .iter()
            .map(|c| (c.borrow().kind(), c.borrow().core().get_nyes()))
            .collect();
        assert_eq!(
            ubc_b,
            vec![
                (FirKind::StayFullyFoolish, Nyes::Woconstanic),
                (FirKind::FoolRef, Nyes::Constant),
            ],
            "?b's result pair is [b's body, still <<...>>-wrapped; FoolRef to b]"
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

    /// Evaluate a program the way the CLI does — composed with `system.foo`
    /// and stepped to settlement. The module's own `step_to_settled` caps at
    /// 50 steps and does NOT compose the system brane, so it cannot settle
    /// these.
    #[cfg(test)]
    fn settle_composed(src: &str) -> FirRef {
        let root = crate::system_foo::compose_program_with_system(src)
            .unwrap()
            .pop()
            .unwrap();
        let scope = Scope::empty();
        for _ in 0..20000 {
            if root.borrow().core().get_nyes().is_constanic() {
                break;
            }
            if root.step(&scope).is_err() {
                break;
            }
        }
        root
    }

    /// Read a named top-level statement's settled integer value.
    /// Find a named statement anywhere in a composed tree.
    #[cfg(test)]
    fn named_stmt(root: &FirRef, name: &str) -> Option<FirRef> {
        fn walk(n: &FirRef, name: &str, d: usize) -> Option<FirRef> {
            if d > 8 {
                return None;
            }
            if n.borrow()
                .as_stmt_searchable_name()
                .is_some_and(|s| s == name)
                && !n.borrow().core().foolish_children().is_empty()
            {
                return Some(Rc::clone(n));
            }
            for c in n.borrow().core().foolish_children().to_vec() {
                if let Some(f) = walk(&c, name, d + 1) {
                    return Some(f);
                }
            }
            None
        }
        walk(root, name, 0)
    }

    #[cfg(test)]
    fn named_i64(root: &FirRef, name: &str) -> Option<i64> {
        // Recursive: compose_program_with_system wraps the user program, so
        // the wanted statement is not a direct child of the composed root.
        fn walk(n: &FirRef, name: &str, depth: usize) -> Option<i64> {
            if depth > 8 {
                return None;
            }
            if n.borrow()
                .as_stmt_searchable_name()
                .is_some_and(|s| s == name)
            {
                // Read through the statement's BODY, as the sibling tests do:
                // a StatementFir's own .value() is not the settled result.
                if let Some(body) = n.borrow().core().foolish_children().first() {
                    let v = body.value();
                    let out = v.borrow().as_i64();
                    if out.is_some() {
                        return out;
                    }
                }
            }
            let kids = n.borrow().core().foolish_children().to_vec();
            for c in kids {
                if let Some(found) = walk(&c, name, depth + 1) {
                    return Some(found);
                }
            }
            None
        }
        walk(root, name, 0)
    }

    /// FOOP-55 §8, 3E.3: `@` projects a search result's POSITION.
    #[test]
    fn at_yields_the_found_statements_index() {
        let root = settle_composed("{tbl = {zzz=0; key=77; other=5;}; p = tbl~key=(77)@;}");
        assert_eq!(
            named_i64(&root, "p"),
            Some(1),
            "key=77 is the statement at index 1, so @ yields 1 -- NOT 77, which \
             is what the search's VALUE would be"
        );
    }

    /// The regression guard that matters most: before 3E.3, `@` fell into the
    /// lexer's unknown-character fallback and was **silently ignored**, so
    /// `tbl~key=(77)@` and `tbl~key=(77)` both gave 77. A program written to
    /// §8 would have run and produced a plausible wrong answer.
    #[test]
    fn at_and_no_at_now_differ() {
        let root = settle_composed(
            "{tbl = {zzz=0; key=77; other=5;}; with_at = tbl~key=(77)@; without = tbl~key=(77);}",
        );
        // NB: the lexer canonicalizes identifier underscores to U+02CD, so the
        // searchable name is "with\u{02CD}at", not "with_at".
        assert_eq!(
            named_i64(&root, "with\u{02CD}at"),
            Some(1),
            "@ gives the POSITION"
        );
        assert_eq!(
            named_i64(&root, "without"),
            Some(77),
            "no @ gives the VALUE"
        );
    }

    /// A miss yields **-1**, which is what makes a default branch fall out of
    /// arithmetic: `@+1` maps a miss to index 0, so a table written with its
    /// default FIRST is selected by the same expression that steps a hit to its
    /// adjacent `value=` row.
    #[test]
    fn at_yields_minus_one_when_candidates_are_exhausted() {
        let root = settle_composed("{tbl = {p=1; q=2;}; miss = tbl~=(99)@;}");
        assert_eq!(
            named_i64(&root, "miss"),
            Some(-1),
            "the scan ran and matched nothing, so @ is -1 -- and -1+1 = 0, the \
             default row's index"
        );
    }

    /// An NK anchor propagates NK rather than yielding -1: there was no table
    /// to search, so "where in nothing?" has no answer. Distinguished from the
    /// case above by `candidates_exhausted()`, not by the NYES.
    #[test]
    fn at_propagates_nk_when_the_anchor_was_nk() {
        let root = settle_composed("{bad = {a=1;}?nope; miss = bad~=(1)@;}");
        let stmt = named_stmt(&root, "miss").expect("statement miss");
        let body = stmt.borrow().core().foolish_children()[0].clone();
        assert_eq!(
            body.value().borrow().core().get_nyes(),
            Nyes::Nk,
            "an NK anchor means there were never any candidates -- @ must NOT \
             invent -1 and silently select a default branch"
        );
    }

    /// A malformed continuation is a true NK, not a compile error (§8): `@` on
    /// a brane has no search whose position could be projected.
    #[test]
    fn at_on_a_non_search_anchor_is_nk() {
        let root = settle_composed("{r = {a=1}@;}");
        let stmt = named_stmt(&root, "r").expect("statement r");
        let body = stmt.borrow().core().foolish_children()[0].clone();
        assert_eq!(
            body.value().borrow().core().get_nyes(),
            Nyes::Nk,
            "only a search produces a position; @ on anything else is an \
             unanswerable question, which is NK"
        );
    }

    /// FOOP-55 §8, 3E.4: `#` accepts an EXPRESSION, not only a literal.
    #[test]
    fn hash_accepts_a_parenthesized_expression() {
        let root = settle_composed("{tbl = {a=1; b=2; c=3;}; r = tbl#(1+1);}");
        assert_eq!(
            named_i64(&root, "r"),
            Some(3),
            "#(1+1) is index 2 -- c=3. Before 3E.4 this was a parse error, \
             'expected integer, found LParen'"
        );
    }

    /// The operand may be a name, whose value the index waits on. That is `#`'s
    /// SECOND dependency (the anchor is the first) — not a new evaluation
    /// phase.
    #[test]
    fn hash_accepts_a_named_operand() {
        let root = settle_composed("{tbl = {a=1; b=2; c=3;}; n = 1; r = tbl#(n);}");
        assert_eq!(
            named_i64(&root, "r"),
            Some(2),
            "#(n) with n=1 selects index 1 -- b=2"
        );
    }

    /// The operand may itself be a search — which is the whole point: it is how
    /// `tbl#(tbl~key=(key)@+1)` selects a row by key.
    #[test]
    fn hash_accepts_a_search_expression_operand() {
        let root = settle_composed("{tbl = {zzz=0; key=77; val=9;}; r = tbl#(tbl~key=(77)@+1);}");
        assert_eq!(
            named_i64(&root, "r"),
            Some(9),
            "key=77 is at index 1, so @+1 = 2 -- val=9, the row beside the \
             matched key. This is the pattern-matching idiom in miniature."
        );
    }

    /// `tbl#1+1` must KEEP its current meaning, `(tbl#1)+1`. It is existing
    /// behaviour and 3E.4 must not change it: only a PARENTHESIZED operand is
    /// the new form.
    #[test]
    fn hash_literal_then_plus_keeps_its_old_meaning() {
        let root = settle_composed("{tbl = {a=1; b=2; c=3;}; r = tbl#1+1;}");
        assert_eq!(
            named_i64(&root, "r"),
            Some(3),
            "tbl#1 is b=2, then +1 gives 3 -- NOT index 2 (which is also 3, so \
             this test uses values where the readings would differ if they did)"
        );
    }

    /// FOOP-55 §8, 3E.2: `candidates_exhausted()` — "the scan ran to
    /// completion and no candidate matched".
    ///
    /// **One observable fact, not a compound claim.** The NK distinction falls
    /// out of it rather than being encoded: a real brane with no match scanned
    /// every candidate and IS exhausted; an NK anchor never ran a scan at all,
    /// so it is NOT.
    ///
    /// Today both settle a bare `Nyes::Nk` (`fir_kinds.rs`'s value-search step
    /// maps `ScanOutcome::NkStop` and an anchored `ScanOutcome::Miss` to the
    /// same state), so the scan's knowledge is discarded exactly where it is
    /// known. This is what `@` needs back.
    #[test]
    fn candidates_exhausted_true_when_the_scan_ran_and_matched_nothing() {
        let root = settle_composed("{tbl = {p=1; q=2;}; miss = tbl~=(99);}");
        let stmt = named_stmt(&root, "miss").expect("statement miss");
        let search = stmt.borrow().core().foolish_children()[0].clone();
        assert!(
            search.borrow().candidates_exhausted(),
            "the anchor is a real brane and every candidate was decided, so the \
             scan is exhausted -- @ should yield -1 here, not NK"
        );
    }

    /// The anchor was NK, so there were never any candidates and no scan ran.
    /// Not exhausted — `@` must propagate NK rather than yield `-1`, because
    /// "where in nothing?" has no answer.
    #[test]
    fn candidates_exhausted_false_when_the_anchor_was_nk() {
        let root = settle_composed("{bad = {a=1;}?nope; miss = bad~=(1);}");
        let stmt = named_stmt(&root, "miss").expect("statement miss");
        let search = stmt.borrow().core().foolish_children()[0].clone();
        assert!(
            !search.borrow().candidates_exhausted(),
            "an NK anchor means the scan never ran -- nothing was exhausted"
        );
    }

    /// A search that FOUND something is not exhausted either: the scan stopped
    /// early, at the match.
    #[test]
    fn candidates_exhausted_false_when_the_search_found_a_match() {
        let root = settle_composed("{tbl = {p=1; q=2;}; hit = tbl~=(2);}");
        let stmt = named_stmt(&root, "hit").expect("statement hit");
        let search = stmt.borrow().core().foolish_children()[0].clone();
        assert!(
            !search.borrow().candidates_exhausted(),
            "the scan stopped at a match, so it did not exhaust its candidates"
        );
    }

    /// It DOES NOT CASCADE — it reflects this search's own status only. A
    /// chained search that itself found something is not exhausted, whatever
    /// its anchor did. A consumer wanting an ancestor's answer asks that
    /// ancestor.
    #[test]
    fn candidates_exhausted_does_not_cascade() {
        let root = settle_composed("{tbl = {p=1; q=2; r=3;}; outer = (tbl?q) &#1;}");
        let stmt = named_stmt(&root, "outer").expect("statement outer");
        let search = stmt.borrow().core().foolish_children()[0].clone();
        assert!(
            !search.borrow().candidates_exhausted(),
            "the outer continuation found r=3; its own status is 'found', and \
             it must not inherit anything from the search it continues"
        );
    }

    /// FOOP-55 §9 rule 3: `@` and `#(expr)` are SEARCHES, so the classifier
    /// gives them the auto-SF wrap rather than rejecting them at construction.
    ///
    /// They still settle NK as concatenation *elements*, and correctly so — but
    /// for a **typing** reason, not a classification one: a postfix search
    /// yields a single value, and a concatenation requires brane-like elements
    /// ("each element value must be brane-like"). `tbl^` and `tbl#1` behave
    /// identically and always have, so this is not §8's omission.
    ///
    /// What the classifier must not do is treat them as *unclassifiable*. This
    /// test pins that they take the same path as the older postfix searches.
    #[test]
    fn search_position_and_computed_seek_classify_as_searches() {
        for src in [
            "{tbl = {k=1; v=9;}; c = {0} tbl~k=(1)@;}",
            "{tbl = {a=1; b=2;}; c = {0} tbl#(1);}",
            // the pre-existing postfix searches, for comparison
            "{tbl = {a=1; b=2;}; c = {0} tbl^;}",
            "{tbl = {a=1; b=2;}; c = {0} tbl#1;}",
        ] {
            let root = settle_composed(src);
            let c = named_stmt(&root, "c").expect("statement c");
            let body = c.borrow().core().foolish_children()[0].clone();
            assert_eq!(
                body.value().borrow().core().get_nyes(),
                Nyes::Nk,
                "a postfix search yields a single value, not a brane, so it \
                 cannot be a concatenation element -- all four alike: {src}"
            );
        }
    }

    /// A bare identifier resolving to a brane IS a valid element — the control
    /// showing the NK above is about the VALUE's shape, not about searches.
    #[test]
    fn a_search_resolving_to_a_brane_is_a_valid_concat_element() {
        let root = settle_composed("{tbl = {a=1; b=2;}; c = {0} tbl;}");
        let c = named_stmt(&root, "c").expect("statement c");
        let body = c.borrow().core().foolish_children()[0].clone();
        assert_ne!(
            body.value().borrow().core().get_nyes(),
            Nyes::Nk,
            "`tbl` resolves to a brane, so it concatenates: {{0; a=1; b=2}}"
        );
    }

    /// FOOP-55 §9 rule 1: an element already marked at the top is built AS
    /// WRITTEN — no second mark is added, and an SFF is not downgraded to SF.
    ///
    /// `<<{…}>>` was classified `SfBrane` and silently given SF semantics.
    /// Under rule 1 the user's doubled mark survives, so the element defers one
    /// coordination longer and the concatenation does not resolve it here.
    #[test]
    fn user_written_sff_on_a_concat_element_is_not_downgraded() {
        let root = settle_composed("{c = {1,2} <<{v=<<#-1>>;}>>;}");
        let c = named_stmt(&root, "c").expect("statement c");
        let body = c.borrow().core().foolish_children()[0].clone();
        assert_ne!(
            body.value().borrow().core().get_nyes(),
            Nyes::Nk,
            "a user-written <<{{…}}>> element must be built as written (§9 rule \
             1), not downgraded to SF and not re-wrapped"
        );
    }

    /// FOOP-55 §8, 3E.1: a continuation's anchor must BE a search.
    ///
    /// A continuation navigates *from a position*, and only a search produces
    /// one. `{a=1}&#1` asks "one past where that landed" of something that
    /// never landed anywhere — there is no position to continue from, so the
    /// answer is **NK**: an unanswerable question, not a refusal to run.
    ///
    /// Two operators are absent on purpose. `&^` and `&$` are consumed by the
    /// FOOP-75 attached-search sugar (`r = X&^` parses as `r =^ X`) before they
    /// can be read as continuations, so a malformed one cannot be written that
    /// way. And there is no bare `&=`: the value continuations are `&?=` and
    /// `&~=` ("expected search operator after &").
    #[test]
    fn continuation_on_a_non_search_anchor_is_nk() {
        for src in [
            "{r = {a=1}&#1;}",
            "{r = {a=1}&?x;}",
            "{r = {a=1}&~x;}",
            "{r = {a=1}&?=1;}",
            "{r = {a=1}&~=1;}",
        ] {
            let root = settle_composed(src);
            let r = named_stmt(&root, "r").expect("statement r");
            let body = r.borrow().core().foolish_children()[0].clone();
            assert_eq!(
                body.value().borrow().core().get_nyes(),
                Nyes::Nk,
                "a continuation whose anchor is not a search has no position to \
                 continue from, so it settles NK: {src}"
            );
        }
    }

    /// The control: a continuation on a REAL search anchor resolves normally.
    /// If this breaks, the check above is rejecting well-formed programs.
    #[test]
    fn continuation_on_a_search_anchor_resolves() {
        let root = settle_composed("{tbl = {a=1; b=2; c=3;}; r = (tbl?b) &#1;}");
        assert_eq!(
            named_i64(&root, "r"),
            Some(3),
            "(tbl?b)&#1 navigates one past the found b, to c=3"
        );
    }

    /// FOOP-55 §6: unanchored FORWARD search `~name`.
    ///
    /// Scans the home brane from the FRONT, stopping before the searching
    /// statement — the same candidate window `?name` uses, walked the other
    /// way. `?name` finds the NEAREST PRECEDING match; `~name` the EARLIEST.
    #[test]
    fn unanchored_forward_search_finds_the_earliest_match() {
        let root = settle_composed("{a=1; b=2; a=4; a=5; result = ~a;}");
        assert_eq!(
            named_i64(&root, "result"),
            Some(1),
            "~a scans from the FRONT of the brane, so it finds the FIRST a (=1)"
        );
    }

    /// The backward twin, pinned alongside so the contrast is explicit and a
    /// regression in either direction is obvious. This one passes today.
    #[test]
    fn unanchored_backward_search_finds_the_nearest_preceding_match() {
        let root = settle_composed("{a=1; b=2; a=4; a=5; result = ?a;}");
        assert_eq!(
            named_i64(&root, "result"),
            Some(5),
            "?a scans BACKWARD from the searching statement, so it finds the \
             nearest preceding a (=5)"
        );
    }

    /// The candidate window stops BEFORE the searching statement: a later
    /// same-named statement is not a candidate, and there is no self-match.
    #[test]
    fn unanchored_forward_search_does_not_see_itself_or_later_statements() {
        let root = settle_composed("{a=1; result = ~a; a=99;}");
        assert_eq!(
            named_i64(&root, "result"),
            Some(1),
            "the a=99 AFTER the search is not a candidate -- the window is \
             [0, my_index-1], so there is no self-match and nothing later"
        );
    }

    /// FOOP-55 §5: a constanic clone strips AT MOST ONE SF/SFF mark.
    ///
    /// The single-mark case is the control: `<<X>>` resolves on
    /// recoordination exactly as it always has. This test must keep passing
    /// unchanged -- if it breaks, the strip budget is spending itself where it
    /// should not.
    #[test]
    fn single_mark_strips_and_resolves_on_recoordination() {
        let root = Compiler::compile("{defn = <<#-1>>; use = {5, 9, defn};}")
            .unwrap()
            .pop()
            .unwrap();
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
            Some(9),
            "a SINGLY marked <<#-1>> resolves to the referencing brane's \
             neighbor on the first recoordination -- unchanged by FOOP-55 §5"
        );
    }

    /// FOOP-55 §5, case 1 (syntactic nesting): a DOUBLY marked term sits out
    /// one coordination.
    ///
    /// `<< <<#-1>> >>` recoordinated once must still carry an SFF mark and
    /// must NOT have searched. This is the property `'ite`'s branches depend
    /// on: they survive the coordination that builds the lookup table, so the
    /// branch the value search does not select never resolves at all.
    #[test]
    fn double_mark_defers_one_coordination() {
        let root = Compiler::compile("{defn = << <<#-1>> >>; use = {5, 9, defn};}")
            .unwrap()
            .pop()
            .unwrap();
        let scope = Scope::empty();
        let _ = step_to_settled(&root, &scope);

        // Walk to the SEARCH for `defn` inside `use`, and read the result it
        // recoordinated. That clone is the coordination event, so its result
        // is where exactly one mark must have come off.
        let stmts = root.borrow().core().foolish_children().to_vec();
        let use_brane = stmts[1].borrow().core().foolish_children()[0].clone();
        let search = use_brane.borrow().core().foolish_children()[2]
            .borrow()
            .core()
            .foolish_children()[0]
            .clone();
        let result = search.borrow().core().ubc_children()[0].clone();

        assert_eq!(
            result.borrow().kind(),
            FirKind::StayFullyFoolish,
            "after ONE coordination a doubly marked term must STILL be \
             SFF-marked -- exactly one mark comes off per constanic clone"
        );

        let inner = result.borrow().core().foolish_children()[0].clone();
        assert_eq!(
            inner.borrow().core().get_nyes(),
            Nyes::Econstanic,
            "the still-marked inner search must NOT have run: it is \
             ECONSTANIC, awaiting the next coordination"
        );
    }

    /// Count the SF/SFF marks stacked at the top of a FIR, then return the
    /// first non-mark node beneath them.
    ///
    /// Marks nest as a chain of single-child wrappers, so "how many marks
    /// survived a clone" is the length of that chain. Every step `expect`s:
    /// a mark with no child is the ALARM condition FOOP-55 §5 forbids, and a
    /// silent `None` here is exactly what made the previous version of this
    /// test vacuous.
    fn peel_marks(fir: &FirRef) -> (usize, FirRef) {
        let mut depth = 0usize;
        let mut cur = Rc::clone(fir);
        loop {
            let kind = cur.borrow().kind();
            if !matches!(kind, FirKind::StayFoolish | FirKind::StayFullyFoolish) {
                return (depth, cur);
            }
            let child = cur
                .borrow()
                .core()
                .foolish_children()
                .first()
                .cloned()
                .unwrap_or_else(|| {
                    panic!("{kind:?} has no child -- the FOOP-55 §5 ALARM condition")
                });
            depth += 1;
            cur = child;
        }
    }

    /// FOOP-55 §5: a clone may strip **at most one** mark per root-to-leaf
    /// PATH. Nesting is therefore a deferral count — `<< <<X>> >>` comes back
    /// from one clone with exactly one mark left.
    ///
    /// Navigates `ubc_children`, NOT `foolish_children`: `foolish_children` is
    /// the program as written and always shows both marks. The clone the budget
    /// governs is the search RESULT, which lives in `ubc_children[0]`. The
    /// previous version of this test navigated the written tree, got `None`,
    /// and swallowed it in an `if let` — passing while asserting nothing.
    #[test]
    fn strip_budget_spends_one_mark_per_path() {
        let root = Compiler::compile("{defn = {inner = << <<#-1>> >>;}; use = defn;}")
            .unwrap()
            .pop()
            .unwrap();
        let scope = Scope::empty();
        let _ = step_to_settled(&root, &scope);

        let stmts = root.borrow().core().foolish_children().to_vec();
        assert_eq!(stmts.len(), 2, "program has two statements");

        // As WRITTEN: two marks.
        let written = stmts[0].borrow().core().foolish_children()[0]
            .borrow()
            .core()
            .foolish_children()[0]
            .borrow()
            .core()
            .foolish_children()[0]
            .clone();
        let (written_depth, _) = peel_marks(&written);
        assert_eq!(
            written_depth, 2,
            "the SOURCE must carry both marks -- if this is not 2 the test is \
             navigating to the wrong node and everything below is meaningless"
        );

        // As CLONED by the search in `use`: one strip spent, one mark left.
        let search = stmts[1].borrow().core().foolish_children()[0].clone();
        let result = search
            .borrow()
            .core()
            .ubc_children()
            .first()
            .cloned()
            .expect("a settled search has its value in ubc_children[0]");
        let cloned_inner = result.borrow().core().foolish_children()[0]
            .borrow()
            .core()
            .foolish_children()[0]
            .clone();
        let (cloned_depth, _) = peel_marks(&cloned_inner);
        assert_eq!(
            cloned_depth,
            written_depth - 1,
            "one clone spends exactly ONE strip on this path: {written_depth} \
             marks written, so {} must survive",
            written_depth - 1
        );
    }

    /// FOOP-55 §5: sibling marks are INDEPENDENT — each subtree carries its own
    /// budget, because `StripBudget` is `Copy` and passed by value.
    ///
    /// This is the case that disproved the original per-clone-TREE design: a
    /// tree-wide budget would let the first operand strip and leave the second
    /// marked, breaking `'mod`, whose two operands are `<<#-2>>` and `<<#-1>>`.
    #[test]
    fn strip_budget_is_per_path_not_per_tree_so_siblings_are_independent() {
        let root = Compiler::compile("{defn = {l = <<#-1>>; r = <<#-1>>;}; use = defn;}")
            .unwrap()
            .pop()
            .unwrap();
        let scope = Scope::empty();
        let _ = step_to_settled(&root, &scope);

        let stmts = root.borrow().core().foolish_children().to_vec();
        let search = stmts[1].borrow().core().foolish_children()[0].clone();
        let result = search
            .borrow()
            .core()
            .ubc_children()
            .first()
            .cloned()
            .expect("a settled search has its value in ubc_children[0]");

        let cloned_stmts = result.borrow().core().foolish_children().to_vec();
        assert_eq!(
            cloned_stmts.len(),
            2,
            "the cloned brane must still hold BOTH sibling statements"
        );
        for (i, stmt) in cloned_stmts.iter().enumerate() {
            let body = stmt.borrow().core().foolish_children()[0].clone();
            let (depth, _) = peel_marks(&body);
            assert_eq!(
                depth, 0,
                "sibling {i} carries its OWN budget, so its single mark is \
                 stripped -- a tree-wide budget would leave this one marked"
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

    // ── "Named creations cannot be renamed" (FOOP-33, post-merge addition) ──

    #[test]
    fn rename_of_named_creation_settles_nf() {
        // {'a=⬤; 'b='a;} -- 'a is the creation's original name; 'b='a tries
        // to give it a SECOND, DIFFERENT null-characterized name -> refused.
        let root = Compiler::compile("{'a=⬤; 'b='a;}").unwrap().pop().unwrap();
        let scope = Scope::empty();
        let _ = step_to_settled(&root, &scope);
        let stmts = root.borrow().core().foolish_children().to_vec();

        assert!(
            stmts[0].borrow().settled_result().is_none(),
            "'a itself is never refused -- it establishes the original name"
        );
        let reason = stmts[1]
            .borrow()
            .settled_result()
            .expect("'b='a must be refused")
            .borrow()
            .as_nk_reason()
            .unwrap()
            .to_owned();
        assert!(is_nf_reason(&reason), "must be NF, got: {reason}");
        assert_eq!(reason, "'b not-foolish (Named creations cannot be renamed)");
    }

    #[test]
    fn same_name_reassertion_of_named_creation_is_permitted() {
        // {'a=⬤; 'a='a;} -- 'a='a re-states 'a's OWN existing name, not a
        // second name. Must stay permitted (mirrors the pre-existing
        // 'True='True guarantee in system_foo.rs).
        let root = Compiler::compile("{'a=⬤; 'a='a;}").unwrap().pop().unwrap();
        let scope = Scope::empty();
        let _ = step_to_settled(&root, &scope);
        let stmts = root.borrow().core().foolish_children().to_vec();
        assert!(
            stmts[1].borrow().settled_result().is_none(),
            "'a='a (same name, same creation) must be PERMITTED, not a rename"
        );
    }

    #[test]
    fn rename_of_a_plain_unnamed_creation_is_permitted() {
        // {c=⬤; 'k=c;} -- `c` is plain (not null-characterized), so it has NO
        // original name to protect. Giving it a null-characterized name for
        // the first time is not a RE-name, it's the first name -- permitted.
        let root = Compiler::compile("{c=⬤; 'k=c;}").unwrap().pop().unwrap();
        let scope = Scope::empty();
        let _ = step_to_settled(&root, &scope);
        let stmts = root.borrow().core().foolish_children().to_vec();
        assert!(
            stmts[1].borrow().settled_result().is_none(),
            "naming a previously-unnamed (plain) creation is permitted"
        );
    }

    #[test]
    fn rename_via_search_reaching_creation_through_a_third_statement_settles_nf() {
        // {'a=⬤; mid='a; 'b=mid;} -- 'b reaches 'a's creation THROUGH `mid`
        // (a plain, non-null-characterized intermediary), but the creation's
        // identity (and its original name 'a) survives the hop (Gotcha #2) --
        // still a rename, still refused.
        let root = Compiler::compile("{'a=⬤; mid='a; 'b=mid;}")
            .unwrap()
            .pop()
            .unwrap();
        let scope = Scope::empty();
        let _ = step_to_settled(&root, &scope);
        let stmts = root.borrow().core().foolish_children().to_vec();
        let reason = stmts[2]
            .borrow()
            .settled_result()
            .expect("'b=mid must be refused -- it renames 'a's creation")
            .borrow()
            .as_nk_reason()
            .unwrap()
            .to_owned();
        assert_eq!(reason, "'b not-foolish (Named creations cannot be renamed)");
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

    /// FOOP-55 §8: the "not mutually identifiable" policy is explicit and pinned.
    ///
    /// A brane against an integer can never bear the same identity. Under
    /// `NOT_MUTUALLY_IDENTIFIABLE_IS_NOT_EQUAL` (the only behaviour that ships) it is
    /// classified **NotEqual**, so a value search Rejects the candidate and
    /// keeps scanning rather than aborting.
    ///
    /// This is what stops the FOOP-33 incident recurring: returning Unknowable
    /// here made value searches abort on the first non-comparable candidate,
    /// turning a working `mixed~=7` into NK.
    #[test]
    fn not_mutually_identifiable_is_not_equal() {
        // Deliberately constant: this is a TRIPWIRE, not a runtime check. The
        // policy constant is `true` today, so clippy correctly observes the
        // assertion cannot fail — but flipping the policy must fail this test
        // loudly and force a review of value-search scanning across the
        // language, which is exactly what the assertion buys.
        #[allow(clippy::assertions_on_constants)]
        {
            assert!(
                NOT_MUTUALLY_IDENTIFIABLE_IS_NOT_EQUAL,
                "only the not-equable-is-not-equal policy ships today; flipping \
                 this constant changes value-search scanning across the language"
            );
        }
        let brane = make_brane(vec![]);
        let int = settled_int(7);
        assert_eq!(
            default_equal(&brane, &int),
            Equality::NotEqual,
            "a brane is never an integer -- decidable, so the matcher skips \
             the candidate and keeps scanning rather than aborting"
        );
    }

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
        // FOOP-55 §10: content is asked of the settled RESULT, not the
        // operator -- .value() unwraps to the ConcatHelper.
        let result_brane = cat.value();

        let result = result_brane.borrow()._search_brane("^a$", 0, 9);
        assert!(result.is_some(), "must find 'a'");
        let (idx, stmt, _nyes) = result.unwrap();
        assert_eq!(idx, 0, "global index of 'a' must be 0");
        assert_eq!(stmt.borrow().as_stmt_searchable_name(), Some("a"));

        let result = result_brane.borrow()._search_brane("^f$", 0, 9);
        assert!(result.is_some(), "must find 'f'");
        let (idx, stmt, _nyes) = result.unwrap();
        assert_eq!(idx, 5, "global index of 'f' must be 5");
        assert_eq!(stmt.borrow().as_stmt_searchable_name(), Some("f"));

        let result = result_brane.borrow()._search_brane("^j$", 9, 0);
        assert!(result.is_some(), "must find 'j' in reverse");
        let (idx, stmt, _nyes) = result.unwrap();
        assert_eq!(idx, 9, "global index of 'j' must be 9");
        assert_eq!(stmt.borrow().as_stmt_searchable_name(), Some("j"));

        let result = result_brane.borrow()._search_brane("^e$", 0, 9);
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
        assert!(cb_body.borrow().is_constanic_branelike());

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
        assert!(
            cb_body.borrow().is_constanic_branelike(),
            "cb must be brane-like"
        );

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
        // FOOP-55 §10: content is asked of the settled RESULT.
        let result_brane = cat.value();

        for i in 0..10 {
            let stmt = result_brane.borrow().stmt_at(i).unwrap();
            let found = result_brane.find_stmt_index(&stmt);
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

    /// FOOP-55 (BraneConcatOp correction): a `BraneConcatOpFir` follows the
    /// SAME universal `value()` rule every FIR follows (`fir_trait.rs`'s
    /// `settled_result` contract) -- itself while pre-constanic, its settled
    /// result (the `ConcatHelper` it pushes into `ubc_children`) once
    /// constanic. There is no `BraneConcatOpFir`-specific exception: the
    /// operator is not its own result once it has actually produced one,
    /// exactly as `OperatorFir` is not its own result once `combine()` runs.
    #[test]
    fn concat_value_is_itself_only_while_pre_constanic() {
        let cat = make_10_stmt_concat();

        // Pre-constanic: value() returns the operator node itself, per the
        // universal rule (fir_trait.rs's FirRefExt::value doc).
        assert!(!cat.borrow().core().get_nyes().is_constanic());
        assert!(cat.borrow().settled_result().is_none());
        let v_pre = cat.value();
        assert!(
            Rc::ptr_eq(&v_pre, &cat),
            "value() before settlement must return the operator itself"
        );

        settle_root(&cat);
        assert!(cat.borrow().core().get_nyes().is_constanic());

        // Once constanic, value() unwraps to the settled result -- the
        // ConcatHelper already sitting in ubc_children -- NOT the operator.
        let helpers = cat.borrow().core().ubc_children();
        assert_eq!(helpers.len(), 1, "unlimited k → single ConcatHelper");
        let helper = &helpers[0];
        let settled = cat.borrow().settled_result();
        assert!(
            settled.is_some_and(|s| Rc::ptr_eq(&s, helper)),
            "settled_result must expose the ConcatHelper once constanic"
        );
        let v_settled = cat.value();
        assert!(
            Rc::ptr_eq(&v_settled, helper),
            "value() once settled must return the ConcatHelper, not the operator"
        );
        assert_eq!(v_settled.borrow().kind(), FirKind::ConcatHelper);

        assert!(
            cat.borrow().as_i64().is_none(),
            "as_i64 on the operator node must be None"
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
        let clone = ProtoBrane::constanic_clone(&cat, &new_parent, 0, true, OpInstructions::Normal);

        // The CLONE's identity as an operator node is what's under test here
        // -- it must still be Concatenation-kinded and constanic. Content
        // comparisons, below, go through .value() (FOOP-55 §10).
        assert_eq!(clone.borrow().kind(), FirKind::Concatenation);
        assert!(clone.borrow().core().get_nyes().is_constanic());
        let cat_result = cat.value();
        let clone_result = clone.value();
        assert_eq!(
            clone_result.borrow().stmt_count(),
            cat_result.borrow().stmt_count(),
            "clone stmt_count must match original"
        );
        for i in 0..10 {
            let orig = cat_result.borrow().stmt_at(i).unwrap();
            let cloned = clone_result.borrow().stmt_at(i).unwrap();
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
        let clone =
            ProtoBrane::constanic_clone(&brane, &new_parent, 0, true, OpInstructions::Normal);

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

        // FOOP-55 §10: content is asked of the settled RESULT.
        let result = cat.value();
        assert_eq!(
            result.borrow().stmt_count(),
            Some(5),
            "total must be 5 statements"
        );

        let expected = ["a", "b", "c", "d", "e"];
        for (i, name) in expected.iter().enumerate() {
            let stmt = result.borrow().stmt_at(i).unwrap();
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
        // FOOP-55 §10: content is asked of the settled RESULT; even zero
        // total lines across all elements produces a real empty ConcatHelper.
        let result = cat.value();
        assert_eq!(
            result.borrow().stmt_count(),
            Some(0),
            "empty branes → 0 statements"
        );
        assert_eq!(
            cat.borrow().core().ubc_children().len(),
            1,
            "a real (empty) ConcatHelper for empty concat"
        );

        let val = make_constant_int(42);
        let stmt = make_statement("x", 0, Rc::clone(&val));
        let non_empty = make_brane(vec![Rc::clone(&stmt)]);
        let empty = make_brane(vec![]);
        let cat2 = make_concatenation(vec![Rc::clone(&non_empty), Rc::clone(&empty)]);
        step_to_settled(&cat2, &scope);
        let result2 = cat2.value();

        assert_eq!(
            result2.borrow().stmt_count(),
            Some(1),
            "one non-empty → 1 statement"
        );
        let first = result2.borrow().stmt_at(0).unwrap();
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
        // FOOP-55 §10: content is asked of the settled RESULT.
        assert_eq!(
            nl_body.value().borrow().stmt_count(),
            Some(3),
            "a, b, c joined"
        );
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
        assert!(
            cb.borrow().is_constanic_branelike(),
            "cb must be brane-like"
        );

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
        let clone = ProtoBrane::constanic_clone(
            &creation,
            &Rc::downgrade(&parent),
            0,
            false,
            OpInstructions::Normal,
        );
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

    // ── CreationFir::get_display_name (FOOP-33 stretch goal, revised) ──
    //
    // A creation reports a name ONLY when BOTH hold: (1) it is being viewed
    // from a statement OTHER than its own defining statement, AND (2) the
    // defining statement's name is null-characterized. These tests pin all
    // four corners of that rule, plus the payoff case: a creation reached
    // through a search still answers with its OWN defining statement's name
    // when viewed from the referencing statement, because a constanic clone
    // of an `Independent` creation returns the SAME `Rc` (FOOP-33 "Gotcha
    // #2"), so its `.parent()` chain -- set at original construction --
    // survives detachment and recoordination at the reference site.

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
    fn creation_viewed_from_its_own_defining_statement_reports_no_name() {
        // `'a = ⬤` -- the creation's parent IS the statement `'a`, and the
        // creation IS that statement's whole body -- but viewed FROM that
        // same statement (condition 1 fails), it must NOT report its name:
        // `{'a=⬤;}` sequencing as `{'a='a}` reads as circular, not as "a
        // fresh creation is being introduced" (FOOP-33.md "Concerns Standing
        // Past Completion").
        let root = Compiler::compile("{'a=⬤; b='a;}").unwrap().pop().unwrap();
        let creation = sift_for_first_creation(&root).expect("creation must exist");
        let defining_stmt = creation.borrow().core().parent().expect("has a parent");
        let name = creation
            .borrow()
            .as_creation_display_name(&creation, Some(&defining_stmt));
        assert_eq!(
            name, None,
            "a creation viewed from its OWN defining statement never reports \
             a name, even though it is null-characterized and the whole RHS"
        );
    }

    #[test]
    fn creation_viewed_from_elsewhere_reports_its_defining_statements_name() {
        // Same source as above, but viewed from a DIFFERENT statement (`b`'s)
        // -- condition 1 now holds, and `'a` is null-characterized (condition
        // 2 holds), so the name is reported.
        let root = Compiler::compile("{'a=⬤; b='a;}").unwrap().pop().unwrap();
        let creation = sift_for_first_creation(&root).expect("creation must exist");
        let stmts = root.borrow().core().foolish_children().to_vec();
        let b_stmt = &stmts[1];
        let name = creation
            .borrow()
            .as_creation_display_name(&creation, Some(b_stmt));
        assert_eq!(
            name.as_deref(),
            Some("'a"),
            "viewed from a DIFFERENT statement, a null-characterized \
             creation reports the FULL characterized name, leading quote \
             included"
        );
    }

    #[test]
    fn creation_inside_operator_expression_reports_no_name() {
        // `'a = 1 + ⬤` -- the creation's parent is the OPERATOR, not the
        // statement. The statement's name belongs to the whole `1+⬤`
        // expression, not to the creation sitting inside it -- true
        // regardless of viewpoint.
        let root = Compiler::compile("{'a=1+⬤; b='a;}").unwrap().pop().unwrap();
        let creation = sift_for_first_creation(&root).expect("creation must exist");
        let stmts = root.borrow().core().foolish_children().to_vec();
        let name = creation
            .borrow()
            .as_creation_display_name(&creation, Some(&stmts[1]));
        assert_eq!(
            name, None,
            "a creation that is only a sub-expression of the RHS reports no \
             name -- the statement name does not belong to it"
        );
    }

    #[test]
    fn creation_under_a_plain_not_null_characterized_statement_never_reports_a_name() {
        // `a = ⬤` (no leading `'`) -- the creation IS the whole RHS, and it is
        // viewed from elsewhere, but the defining statement's name is NOT
        // null-characterized (condition 2 fails). Two unrelated plain
        // creations both named `a` in different branes would otherwise be
        // indistinguishable once rendered by name; restricting to
        // null-characterized names limits this to protected constants like
        // `'True`/`'False`, where the name genuinely picks out one creation.
        let root = Compiler::compile("{a=⬤; b=a;}").unwrap().pop().unwrap();
        let creation = sift_for_first_creation(&root).expect("creation must exist");
        let stmts = root.borrow().core().foolish_children().to_vec();
        let name = creation
            .borrow()
            .as_creation_display_name(&creation, Some(&stmts[1]));
        assert_eq!(
            name, None,
            "a plain (non-null-characterized) defining statement never \
             lends its name to its creation, even viewed from elsewhere"
        );
    }

    #[test]
    fn creation_with_no_statement_parent_reports_no_name() {
        // Constructed directly under a brane -- no statement anywhere in its
        // parent chain, so there is no name to report, regardless of
        // viewpoint.
        let parent = make_brane(vec![]);
        let creation = CreationFir::creation(Rc::downgrade(&parent));
        let name = creation
            .borrow()
            .as_creation_display_name(&creation, Some(&parent));
        assert_eq!(
            name, None,
            "a creation whose parent is not a statement reports no name"
        );
    }

    #[test]
    fn creation_viewed_with_no_statement_in_scope_reports_no_name() {
        // `viewed_from = None` -- conservatively, never show a name: there is
        // no way to tell whether this is the defining site or elsewhere.
        let root = Compiler::compile("{'a=⬤; b='a;}").unwrap().pop().unwrap();
        let creation = sift_for_first_creation(&root).expect("creation must exist");
        let name = creation.borrow().as_creation_display_name(&creation, None);
        assert_eq!(
            name, None,
            "with no current statement in scope, the name is never shown"
        );
    }

    #[test]
    fn creation_reached_through_search_still_reports_its_own_name() {
        // THE PAYOFF CASE. `b='a` resolves to the SAME creation `Rc` that `'a`
        // defines (Gotcha #2), so walking that resolved value's parent chain
        // still lands on the ORIGINAL `'a` statement -- proving the parent
        // chain survives detachment/recoordination at the reference site.
        // Viewed from `b`'s own statement (condition 1 holds; `'a` is
        // null-characterized, condition 2 holds).
        let root = Compiler::compile("{'a=⬤; b='a;}").unwrap().pop().unwrap();
        let scope = Scope::empty();
        let _ = step_to_settled(&root, &scope);

        let stmts = root.borrow().core().foolish_children().to_vec();
        let b_stmt = &stmts[1];
        let b_body = b_stmt.borrow().core().foolish_children()[0].clone();
        let resolved = b_body.value();
        assert_eq!(
            resolved.borrow().kind(),
            FirKind::Creation,
            "`b='a` must resolve THROUGH the search to the creation itself"
        );

        let name = resolved
            .borrow()
            .as_creation_display_name(&resolved, Some(b_stmt));
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
        // their own creation. Each must report its own -- pointer identity,
        // not a coincidental name match, is what decides. Viewed from a
        // sibling statement in each brane (not their own defining site).
        let root = Compiler::compile("{A={'k=⬤; other=1;}; B={'k=⬤; other=2;};}")
            .unwrap()
            .pop()
            .unwrap();
        let stmts = root.borrow().core().foolish_children().to_vec();

        let names: Vec<Option<String>> = stmts
            .iter()
            .map(|outer| {
                let creation =
                    sift_for_first_creation(outer).expect("each brane defines a creation");
                let inner_brane = outer.borrow().core().foolish_children()[0].clone();
                let inner_stmts = inner_brane.borrow().core().foolish_children().to_vec();
                let sibling = &inner_stmts[1]; // `other`, not `'k` itself
                creation
                    .borrow()
                    .as_creation_display_name(&creation, Some(sibling))
            })
            .collect();

        assert_eq!(
            names,
            vec![Some("'k".to_owned()), Some("'k".to_owned())],
            "each creation, viewed from a sibling statement, reports the \
             name of the statement it is the body of"
        );

        let a_creation = sift_for_first_creation(&stmts[0]).unwrap();
        let b_creation = sift_for_first_creation(&stmts[1]).unwrap();
        assert!(
            !Rc::ptr_eq(&a_creation, &b_creation),
            "they are genuinely distinct creations despite the shared name"
        );
    }

    // ── FOOP-65 Phase 2: ConcatProvenance tests ──────────────────────────

    fn make_tail_concatenation(elements: Vec<FirRef>) -> FirRef {
        Rc::new_cyclic(|me: &Weak<RefCell<BraneConcatOpFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(BraneConcatOpFir {
                core: ProtoBrane::new(elements, parent, Nyes::Prembrionic),
                _helpers_populated: std::cell::Cell::new(false),
                provenance: ConcatProvenance::TailConcatenation,
            })
        })
    }

    #[test]
    fn tail_concat_nyes_transitions() {
        let brane1 = make_brane(vec![
            make_statement("a", 0, make_constant_int(1)),
            make_statement("b", 1, make_constant_int(2)),
        ]);
        let brane2 = make_brane(vec![
            make_statement("c", 0, make_constant_int(3)),
            make_statement("d", 1, make_constant_int(4)),
        ]);
        let cat = make_tail_concatenation(vec![brane1, brane2]);
        let trace = step_to_settled(&cat, &Scope::empty());
        assert_progression(&trace, Nyes::Constant, "Concatenation(tail)");
        assert!(
            trace.contains(&Nyes::Braning),
            "tail-flagged concatenation must pass through Braning"
        );
    }

    #[test]
    fn tail_concat_provenance_on_fir() {
        let cat = make_tail_concatenation(vec![make_constant_int(1)]);
        assert_eq!(
            cat.borrow().as_concat_provenance(),
            ConcatProvenance::TailConcatenation
        );
        let juxta = make_concatenation(vec![make_constant_int(1)]);
        assert_eq!(
            juxta.borrow().as_concat_provenance(),
            ConcatProvenance::Juxtaposition
        );
    }

    #[test]
    fn compiler_shape_tail_concat() {
        // Using identifiers matching the parser worked example (a`b`c`d e f).
        // The outer concatenation should have provenance TailConcatenation.
        let root = Compiler::compile("{r = a`b`c`d e f;}")
            .unwrap()
            .pop()
            .unwrap();
        let stmts = root.borrow().core().foolish_children().to_vec();
        let stmt = &stmts[0];
        let body = stmt.borrow().core().foolish_children()[0].clone();
        let b = body.borrow();
        assert_eq!(b.kind(), FirKind::Concatenation);
        assert_eq!(
            b.as_concat_provenance(),
            ConcatProvenance::TailConcatenation
        );

        let children = b.core().foolish_children();
        assert_eq!(
            children.len(),
            4,
            "outer has 4 elements: [Concat(d,e,f), c, b, a]"
        );
    }

    /// FOOP-65's Equivalence Law: `` 99`{x=1;y=2;} `` and `{x=1;y=2;} 99` are
    /// the same program spelled two ways, so they must settle the same.
    ///
    /// **They do not, and this test documents the divergence rather than
    /// hiding it.**
    ///
    /// | Spelling | NYES | why |
    /// |----------|------|-----|
    /// | `` 99`{…} `` | **NK** | the backtick parser accepts a bare integer, so `99` really is a constituent — and an integer is not concatenable |
    /// | `{…} 99` | **INDEPENDENT** | `is_concatenation_continuation` rejects a bare integer, so `99` never joins: it splits into its own statement and the brane is untouched |
    ///
    /// This is the same root cause as FOOP-55 §9.4(a) / plan item (a2): the
    /// juxtaposition parser will not start a continuation on a bare integer.
    /// Atlas decided (2026-08-13) that a parse error is acceptable there; that
    /// decision now has a second consequence recorded here, since the backtick
    /// spelling reaches the concatenation and the juxtaposed one does not.
    /// **Resolving (a2) should make these two agree.**
    ///
    /// **Corrected 2026-08-22 (D10 fix, §5.5):** an earlier version of this
    /// test compared `stmt_count()` on both spellings and asserted they
    /// coincided at 2 -- which relied on the NK spelling's `stmt_count()`
    /// populating helpers via the OLD, ungated `BraneConcatOpFir::stmt_count`
    /// (a side door that memoized a count even for a concatenation that
    /// never actually joined, via the `type_errors`-then-Nk path). Now that
    /// `stmt_count`/`stmt_at` only answer once helpers are genuinely
    /// populated, the NK spelling honestly reports `None` — it never joined
    /// anything, so there is no count to report. Comparing counts across an
    /// NK and a non-NK spelling was never a meaningful equivalence check;
    /// the NYES comparison below is the actual documented divergence.
    #[test]
    fn tail_concat_equivalence_brane_literal() {
        let a = Compiler::compile("{result = 99`{x=1; y=2;};}")
            .unwrap()
            .pop()
            .unwrap();
        let b = Compiler::compile("{result = {x=1; y=2;} 99;}")
            .unwrap()
            .pop()
            .unwrap();
        settle_root(&a);
        settle_root(&b);
        let a_stmts = a.borrow().core().foolish_children().to_vec();
        let b_stmts = b.borrow().core().foolish_children().to_vec();
        let a_body = a_stmts[0].borrow().core().foolish_children()[0].clone();
        let b_body = b_stmts[0].borrow().core().foolish_children()[0].clone();

        assert_eq!(
            a_body.borrow().stmt_count(),
            None,
            "the NK spelling never actually joins anything, so it has no \
             statement count to report -- None is the honest answer"
        );
        assert_eq!(
            b_body.borrow().stmt_count(),
            Some(2),
            "the juxtaposed spelling settles a real 2-statement brane"
        );

        // The documented divergence. Pinned as the current, known-wrong
        // behaviour so that fixing (a2) fails this assertion loudly and forces
        // the fix to be recorded, rather than passing silently either way.
        assert_eq!(
            (
                a_body.borrow().core().get_nyes(),
                b_body.borrow().core().get_nyes()
            ),
            (Nyes::Nk, Nyes::Independent),
            "KNOWN DIVERGENCE (FOOP-55 plan item a2): the backtick spelling \
             makes `99` a constituent and NKs, while juxtaposition refuses to \
             continue on a bare integer and splits it off instead. Equivalence \
             requires these to agree; when (a2) is resolved, update this \
             assertion to demand equality."
        );
    }

    #[test]
    fn tail_concat_chain_reversal() {
        let root = Compiler::compile("{r = f`g`h`{x=1;};}")
            .unwrap()
            .pop()
            .unwrap();
        let stmts = root.borrow().core().foolish_children().to_vec();
        let body = stmts[0].borrow().core().foolish_children()[0].clone();
        let b = body.borrow();
        assert_eq!(b.kind(), FirKind::Concatenation);
        assert_eq!(
            b.as_concat_provenance(),
            ConcatProvenance::TailConcatenation
        );
        let children = b.core().foolish_children();
        assert_eq!(children.len(), 4);
    }

    #[test]
    fn tail_concat_flag_survives_recoordination() {
        let cat = make_tail_concatenation(vec![
            make_brane(vec![make_statement("a", 0, make_constant_int(1))]),
            make_brane(vec![make_statement("b", 0, make_constant_int(2))]),
        ]);
        settle_root(&cat);

        assert!(cat.borrow().core().get_nyes().is_constanic());

        // Use a real brane as the new parent (Weak::new doesn't work for dyn Fir).
        let dummy_parent = make_brane(vec![]);
        let new_parent: std::rc::Weak<RefCell<dyn Fir>> = std::rc::Rc::downgrade(&dummy_parent);
        let cloned =
            ProtoBrane::constanic_clone(&cat, &new_parent, 0, false, OpInstructions::Normal);
        assert_eq!(
            cloned.borrow().as_concat_provenance(),
            ConcatProvenance::TailConcatenation,
            "constanic clone must preserve tail provenance"
        );
    }

    #[test]
    fn tail_concat_evaluation_inert() {
        let brane_a = make_brane(vec![make_statement("x", 0, make_constant_int(10))]);
        let brane_b = make_brane(vec![make_statement("y", 0, make_constant_int(20))]);

        let juxta = make_concatenation(vec![brane_a.clone(), brane_b.clone()]);
        let tail = make_tail_concatenation(vec![brane_a, brane_b]);

        settle_root(&juxta);
        settle_root(&tail);

        let j_nyes = juxta.borrow().core().get_nyes();
        let t_nyes = tail.borrow().core().get_nyes();
        assert_eq!(j_nyes, t_nyes, "same settled NYES");

        let j_stmts = juxta.borrow().stmt_count().unwrap_or(0);
        let t_stmts = tail.borrow().stmt_count().unwrap_or(0);
        assert_eq!(j_stmts, t_stmts, "same statement count");
    }

    #[test]
    fn tail_concat_system_operator_application() {
        let root = Compiler::compile("{result = ('lt`{1, 2})$;}")
            .unwrap()
            .pop()
            .unwrap();
        settle_root(&root);
        let stmts = root.borrow().core().foolish_children().to_vec();
        let body = stmts[0].borrow().core().foolish_children()[0].clone();
        assert!(body.borrow().core().get_nyes().is_constanic(), "settled");
    }

    /// ISOLATED illustration of the mechanism behind D10's remaining gap
    /// (found while investigating `nested_concat_as_tail_concat_last_element_
    /// flattens_completely` below). Plain English:
    ///
    /// `x y z` is a plain juxtaposition concatenation of three names. Inside
    /// a concatenation, each bare name is compiled as a SEARCH (FOOP-55 §9.2
    /// rule 3: "search — any of them — SF-marked"), NOT read directly as a
    /// value: the compiler does not know yet whether `x` names a brane in
    /// scope until the search actually runs.
    ///
    /// When `x`'s search steps, it is UNANCHORED — it looks in `x`'s own
    /// ancestor chain for a statement named `x`. If none is found there
    /// (yet), it settles **ECONSTANIC**: "no value HERE, but I may still
    /// gain one later via recoordination" (AGENTS.md's NYES glossary). This
    /// is not a failure — it is the ordinary, revivable "not yet" answer an
    /// unanchored search gives when its brane hasn't been coordinated into a
    /// context wide enough to contain the name it wants.
    ///
    /// `BraneConcatOpFir::fir_op_step`'s `Braning` arm decides whether it can
    /// join its elements by asking each element's `.value().constanic_is_
    /// brane_like()`. For a search that is still ECONSTANIC, `.value()`
    /// returns the search node itself (unresolved — see `FirRefExt::value`'s
    /// own doc: "for FIRs with no settled result, returns a clone of self"),
    /// and a bare, unresolved `SearchFir` reports `is_constanic_branelike()
    /// == false` (it has no `stmt_count` of its own to report). The classify
    /// therefore concludes `all_brane_like = false` and the concatenation
    /// gives up, settling **Woconstanic via the "not joinable yet, render
    /// raw elements" branch** (`fir_kinds.rs` ~3230-3235) — WITHOUT ever
    /// calling `populate_concat_helpers()`.
    ///
    /// The bug: that Woconstanic is indistinguishable, from the OUTSIDE, from
    /// an ordinary "I joined successfully but the result still has an
    /// unsettled dependency" Woconstanic — both just read `Nyes::Woconstanic`
    /// (a `is_constanic()` state). Once a caller sees `is_constanic() ==
    /// true`, nothing re-steps this concatenation — `step_inner`'s task-queue
    /// driver pops a constanic child and moves on (`fir_trait.rs:510`). So
    /// this concatenation is now PERMANENTLY stuck reporting zero content,
    /// even if `x`'s search would have found its target one step later, once
    /// this concatenation is spliced into a wider context by an ENCLOSING
    /// concatenation (exactly what D10's `nested_concat_as_tail_concat_last_
    /// element_flattens_completely` test below does).
    ///
    /// This is the SAME defect shape §5.5/D9 already names — an ECONSTANIC
    /// search misread as a permanent "no" — one level higher: here it is
    /// `all_brane_like`'s classify doing the misreading, and the concatenation
    /// borrowing WOCONSTANIC's terminal vocabulary for a genuinely
    /// pre-constanic "haven't attempted the join yet" condition.
    #[test]
    fn concat_element_that_is_an_econstanic_search_freezes_the_concatenation() {
        // `x y z` with NO enclosing definition of x, y, or z: each name's
        // search is unanchored and has nothing to find AT THIS SCOPE. This
        // reproduces, in isolation, exactly the state the nested `d_brn e_brn
        // f_brn` concatenation is in the moment BEFORE it is spliced into a
        // wider outer context by an enclosing concatenation.
        let root = Compiler::compile("{lonely = x y z;}")
            .unwrap()
            .pop()
            .unwrap();
        let scope = Scope::empty();
        for _ in 0..200 {
            match root.step(&scope).unwrap() {
                StepReport::Progress(nyes) if nyes.is_constanic() => break,
                StepReport::NoProgress => break,
                _ => {}
            }
        }
        let stmts = root.borrow().core().foolish_children().to_vec();
        let lonely_stmt = stmts
            .iter()
            .find(|s| s.borrow().as_stmt_searchable_name() == Some("lonely"))
            .expect("lonely statement");
        let lonely_body = lonely_stmt.borrow().core().foolish_children()[0].clone();

        // What actually happens today: the concatenation gives up on its
        // FIRST attempt (its elements' searches are ECONSTANIC, not yet
        // found) and freezes at Woconstanic with NOTHING joined --
        // stmt_count reports Some(0)/None rather than "not yet knowable".
        assert_eq!(
            lonely_body.borrow().core().get_nyes(),
            Nyes::Woconstanic,
            "demonstrates the freeze: settles Woconstanic on the FIRST attempt, \
             via the \"not joinable yet\" escape, before helpers are ever populated"
        );
        assert!(
            lonely_body.borrow().stmt_count().unwrap_or(0) == 0,
            "demonstrates the damage: once frozen, stmt_count reports no content \
             and nothing re-steps this concatenation to try again"
        );
    }

    /// FOOP-55 §9.2 (corrected 2026-08-22, per Atlas): a nested WRITTEN
    /// concatenation is NOT a Foolish Brane (a `{...}` literal typed out in
    /// the program) and must not be SFF-marked as one. It is an active
    /// joining process, meant to run and complete immediately, right where
    /// it is written -- exactly like a top-level concatenation. Only a
    /// genuine Foolish Brane literal, or a search (whatever it resolves to),
    /// gets a mark; a nested concatenation gets none, and its own
    /// constituents are classified and marked by the same table, recursively.
    ///
    /// An earlier, INCORRECT version of this rule SFF-marked a nested
    /// concatenation "because it's brane-like, treated exactly as a brane".
    /// That over-corrected: `compiler.rs`'s `build_fir` has a separate,
    /// correct rule that a search built while an SFF ancestor is under
    /// construction is compiled ALREADY SETTLED at ECONSTANIC ("the SFF body
    /// is constanic unevaluated") -- right for a brane literal's free
    /// variables, but wrong for `d_brn`/`e_brn`/`f_brn` here, which are the
    /// nested concatenation's OWN elements and must resolve normally. The
    /// incorrect SFF wrap froze them at construction, so the nested
    /// concatenation could never complete its own join (this test's earlier
    /// form pinned exactly that failure: `d_search` Econstanic-by-
    /// construction, `outer` stuck at `stmt_count = Some(1)`).
    #[test]
    fn nested_written_concat_as_constituent_joins_immediately() {
        let root = Compiler::compile(
            "{d_brn={4;}; e_brn={5;}; f_brn={6;}; c_brn={3;}; \
             outer = (d_brn`e_brn`f_brn) c_brn;}",
        )
        .unwrap()
        .pop()
        .unwrap();
        let scope = Scope::empty();
        for _ in 0..200 {
            match root.step(&scope).unwrap() {
                StepReport::Progress(nyes) if nyes.is_constanic() => break,
                StepReport::NoProgress => break,
                _ => {}
            }
        }
        let stmts = root.borrow().core().foolish_children().to_vec();
        let outer_stmt = stmts
            .iter()
            .find(|s| s.borrow().as_stmt_searchable_name() == Some("outer"))
            .expect("outer statement");
        let outer_body = outer_stmt.borrow().core().foolish_children()[0].clone();

        // The nested concatenation constituent carries NO mark of its own.
        let nested_concat = outer_body.borrow().core().foolish_children()[0].clone();
        assert_eq!(
            nested_concat.borrow().kind(),
            FirKind::Concatenation,
            "a nested written concatenation must be built UNMARKED -- it is \
             not a Foolish Brane literal and must not be wrapped in SFF"
        );
        // Its own elements ARE still individually SF-marked (search, rule 3)
        // and their searches run and resolve normally -- NOT built
        // already-Econstanic, since nothing SFF-marked sits above them now.
        let d_elem = nested_concat.borrow().core().foolish_children()[0].clone();
        assert_eq!(d_elem.borrow().kind(), FirKind::StayFoolish);
        let d_search = d_elem.borrow().core().foolish_children()[0].clone();
        assert_eq!(
            d_search.borrow().core().get_nyes(),
            Nyes::Constant,
            "d_brn's search must actually RUN and resolve to Constant -- not \
             be pre-settled Econstanic-by-construction as if it were a \
             deferred macro-body free variable"
        );

        // The nested concatenation joins its own elements immediately, and
        // the outer concatenation flattens ALL 4 statements. `d_brn`e_brn`f_brn`
        // is a tail-concatenation chain, so D11's element-reversal applies to
        // ITS OWN elements too (FOOP-65 §1 equivalence) -- print the actual
        // order rather than assume it, then pin whatever it is.
        // FOOP-55 §10: content is asked of the settled RESULT.
        let outer_result = outer_body.value();
        assert_eq!(outer_result.borrow().stmt_count(), Some(4));
        let actual: Vec<i64> = (0..4)
            .map(|i| {
                let stmt = outer_result.borrow().stmt_at(i).expect("stmt_at in range");
                statement_value_for_comparison(&stmt)
                    .and_then(|v| v.borrow().as_i64())
                    .unwrap_or(-1)
            })
            .collect();
        // `d_brn`e_brn`f_brn` is itself a TailConcatenation, so D11's
        // element-reversal (FOOP-65 §1: `fn`X` == `X fn`) applies to its own
        // elements too, flattening to f,e,d before the outer `) c_brn` adds
        // c_brn last. The regression guard that matters is that ALL FOUR
        // values are PRESENT -- D10's bug was values going MISSING, not
        // reordered.
        assert_eq!(actual, vec![6, 5, 4, 3]);
    }

    /// D10 (FOOP-55.md §Findings, §5.5, §9.2): a nested written concatenation
    /// (`d_brn e_brn f_brn`) as the last element of a TailConcatenation
    /// (`a_brn`b_brn`c_brn`d_brn e_brn f_brn`) must join immediately and
    /// contribute ALL its statements to the outer flatten — none may be
    /// silently dropped.
    ///
    /// Two independent defects on this path, both fixed:
    /// - `BraneConcatOpFir::stmt_count`/`stmt_at` used to call
    ///   `populate_concat_helpers()` unconditionally, bypassing
    ///   `fir_op_step`'s own gate; a not-yet-populated concatenation now
    ///   honestly answers "not yet knowable" (`None`) instead of a
    ///   memoized, premature `Some(0)`.
    /// - §9.2's "Concatenation → SFF-mark it" row incorrectly treated a
    ///   nested concatenation as if it were a Foolish Brane literal. A
    ///   nested concatenation is not a Foolish Brane — it is an active
    ///   joining process — and must not be SFF-marked: doing so froze its
    ///   own constituent searches at construction (`compiler.rs`'s
    ///   `under_sff` rule), so it could never complete its own join.
    #[test]
    fn nested_concat_as_tail_concat_last_element_flattens_completely() {
        let root = Compiler::compile(
            "{a_brn={1;}; b_brn={2;}; c_brn={3;}; d_brn={4;}; e_brn={5;}; f_brn={6;}; \
             chain = a_brn`b_brn`c_brn`d_brn e_brn f_brn;}",
        )
        .unwrap()
        .pop()
        .unwrap();
        let scope = Scope::empty();
        for _ in 0..2000 {
            match root.step(&scope).unwrap() {
                StepReport::Progress(nyes) if nyes.is_constanic() => break,
                StepReport::NoProgress => break,
                _ => {}
            }
        }
        assert!(
            root.borrow().core().get_nyes().is_constanic(),
            "root did not settle"
        );

        let stmts = root.borrow().core().foolish_children().to_vec();
        let chain_stmt = stmts
            .iter()
            .find(|s| s.borrow().as_stmt_searchable_name() == Some("chain"))
            .expect("chain statement");
        let chain_body = chain_stmt.borrow().core().foolish_children()[0].clone();
        // FOOP-55 §10: content is asked of the settled RESULT.
        let chain_result = chain_body.value();

        assert_eq!(
            chain_result.borrow().stmt_count(),
            Some(6),
            "expected all 6 flattened statements; the nested concatenation's \
             contribution must not be dropped"
        );
        let actual: Vec<i64> = (0..6)
            .map(|i| {
                let stmt = chain_result
                    .borrow()
                    .stmt_at(i)
                    .unwrap_or_else(|| panic!("stmt_at({i}) missing"));
                statement_value_for_comparison(&stmt)
                    .and_then(|v| v.borrow().as_i64())
                    .unwrap_or_else(|| panic!("stmt_at({i}) has no comparable value"))
            })
            .collect();
        // Every value must be PRESENT — this is the regression D10 guards
        // against (values going missing), not a claim about ORDER: this
        // chain mixes plain backtick links with a trailing juxtaposition
        // group, and D11's per-level reversal (FOOP-65 §1) applies at each
        // nesting depth independently.
        let mut sorted = actual.clone();
        sorted.sort_unstable();
        assert_eq!(
            sorted,
            vec![1, 2, 3, 4, 5, 6],
            "actual flattened order was {actual:?}; all six literals \
             (1..6) must be present exactly once"
        );
    }

    /// FOOP-55 §11 Step 1: the default `is_foolish_child_constanic_enough`/
    /// `is_ubc_child_constanic_enough` must agree with plain `is_constanic()`
    /// for both a pre-constanic and a settled child, on a kind that
    /// overrides neither (an ordinary `Operator`) — pinning that the new
    /// dequeue gate is observably identical to the old hardcoded check until
    /// a kind opts in to an override.
    #[test]
    fn constanic_enough_default_matches_is_constanic() {
        let a = make_constant_int(3);
        let b = make_constant_int(5);
        let op = make_operator("+", vec![Rc::clone(&a), Rc::clone(&b)]);
        let scope = Scope::empty();

        // Fresh, pre-constanic operand `a`: default gate says "not enough",
        // matching is_constanic() == false.
        assert!(!a.borrow().core().get_nyes().is_constanic());
        assert_eq!(
            op.borrow().is_foolish_child_constanic_enough(&a),
            a.borrow().core().get_nyes().is_constanic(),
        );
        assert!(!op.borrow().is_foolish_child_constanic_enough(&a));

        // Step `a` to settled (a leaf constant settles in one step), then
        // both sides must agree it is now "enough".
        let _ = a.step(&scope).unwrap();
        assert!(a.borrow().core().get_nyes().is_constanic());
        assert_eq!(
            op.borrow().is_foolish_child_constanic_enough(&a),
            a.borrow().core().get_nyes().is_constanic(),
        );
        assert!(op.borrow().is_foolish_child_constanic_enough(&a));

        // is_ubc_child_constanic_enough has the same default; same node,
        // same answer.
        assert_eq!(
            op.borrow().is_ubc_child_constanic_enough(&a),
            op.borrow().is_foolish_child_constanic_enough(&a),
        );
    }

    /// FOOP-55 §11 Step 2: `are_foolish_children_ready_for_op`'s default
    /// must be `false` while any `foolish_children` member is pre-constanic,
    /// and `true` only once every member has settled — a genuine
    /// whole-SET aggregation, not just a single-child check.
    #[test]
    fn are_foolish_children_ready_for_op_default_waits_for_all() {
        let a = make_constant_int(1);
        let b = make_constant_int(2);
        let brane = make_brane(vec![Rc::clone(&a), Rc::clone(&b)]);
        let scope = Scope::empty();

        // Neither child has stepped yet: not ready.
        assert!(!brane.borrow().are_foolish_children_ready_for_op());

        // Settle only `a`: still not ready — `b` is still pre-constanic.
        let _ = a.step(&scope).unwrap();
        assert!(a.borrow().core().get_nyes().is_constanic());
        assert!(!b.borrow().core().get_nyes().is_constanic());
        assert!(!brane.borrow().are_foolish_children_ready_for_op());

        // Settle `b` too: NOW the whole set is ready.
        let _ = b.step(&scope).unwrap();
        assert!(b.borrow().core().get_nyes().is_constanic());
        assert!(brane.borrow().are_foolish_children_ready_for_op());
    }

    /// FOOP-55 §11 Step 2: `are_ubc_children_ready_for_op`'s default over an
    /// EMPTY `ubc_children` store must be vacuously `true` (no children to
    /// fail the predicate) — this is what lets a kind with nothing further
    /// to compute (e.g. a simple operator) pass its ubc-phase gate
    /// trivially once it has pushed its single computed result, or before
    /// it has pushed anything at all.
    #[test]
    fn are_ubc_children_ready_for_op_vacuously_true_when_empty() {
        let brane = make_brane(vec![]);
        assert!(brane.borrow().core().ubc_children().is_empty());
        assert!(brane.borrow().are_ubc_children_ready_for_op());
    }

    /// FOOP-55 §11 Step 3: the default `on_foolish_op_ready`/`on_ubc_op_ready`
    /// return `None` (defer to the fallback) on a kind that overrides
    /// neither.
    #[test]
    fn on_op_ready_handlers_default_to_none() {
        let brane = make_brane(vec![]);
        let scope = Scope::empty();
        assert_eq!(brane.borrow().on_foolish_op_ready(&scope), None);
        assert_eq!(brane.borrow().on_ubc_op_ready(&scope), None);
    }

    /// FOOP-55 §11 (2026-08-24): an `OperatorFir` built with an unrecognized
    /// operator string settles NK with an explanation, rather than
    /// propagating a hard `UbcError::Eval` — a behavior change forced by
    /// `on_foolish_op_ready`'s `Option<Nyes>` signature (it cannot express
    /// a Rust-level error). This path is unreachable through the parser in
    /// practice (only known operators are ever constructed) but is pinned
    /// here so the FVM-level answer for a malformed `OperatorFir` is
    /// documented and does not silently drift. Parser-side validation to
    /// make this truly unreachable is tracked separately (plan Phase 4B).
    #[test]
    fn unknown_operator_settles_nk_not_hard_error() {
        let a = make_constant_int(1);
        let b = make_constant_int(2);
        let op = make_operator("¿unknown?", vec![Rc::clone(&a), Rc::clone(&b)]);
        let scope = Scope::empty();

        let trace = step_to_settled(&op, &scope);
        assert_eq!(
            *trace.last().unwrap(),
            Nyes::Nk,
            "unknown operator must settle NK, trace={trace:?}"
        );
        let reason = op.value().borrow().as_nk_reason().map(str::to_owned);
        assert!(
            reason
                .as_deref()
                .is_some_and(|r| r.contains("unknown operator")),
            "NK reason should explain the cause, got {reason:?}"
        );
    }
}
