//! `system.foo` composition — FOOP-33 §4.
//!
//! `system.foo` (authored at the repo root, `system/system.foo`, copied into
//! `OUT_DIR` by `build.rs` and embedded here at compile time) is composed with
//! every user program as its root ancestor. The user's program becomes an
//! ordinary member of the composite root brane, bound to the plain name
//! `program` — NOT wrapped in a null-characterized statement, and NOT reached
//! via ancestral (`_ab_search`) parent-chain walking. Composition, not
//! ancestry: `system.foo`'s own parsed AST gains one more statement,
//! `program = {user source}`, appended last, and the combined AST compiles as
//! ONE self-rooting brane. The evaluator extracts the `program` member back
//! out via `stmt_at(stmt_count() - 1)` — a Rust structural accessor
//! (FOOP-13 A2), never a Foolish search — so the return path cannot be
//! perturbed by a search-engine bug.
//!
//! See FOOP-33.md §4 and the plan's Phase 5 section for the full design
//! rationale, including why `program` is retrieved positionally (last
//! statement) rather than by name.

use std::cell::RefCell;
use std::rc::{Rc, Weak};

use foolish_core::fir::Nyes;
use foolish_parser::{AssignmentOperator, Astn};

use crate::fir_kinds::IndepIntFir;
use crate::fir_trait::{Fir, FirKind, FirRef, FirRefExt, Scope, UbcError};
use crate::proto_brane::ProtoBrane;

/// The five comparison operators `system.foo` supplies (FOOP-33 §5.0).
///
/// One enum rather than five near-identical FIR types: every operator shares
/// the *entire* structure — the same two SFF-marked operand lookups
/// (`<<#-2>>`/`<<#-1>>`), the same constanic gating, the same `'True`/`'False`
/// production — and differs ONLY in which Rust comparison runs once both
/// operands are integers. A finite, closed set of behaviors distinguished by
/// one line of code is exactly what an enum is for (`rust_instructions.md`
/// §"finite word-domains → enum"); five types would be five copies of
/// [`ComparisonFir`] with one differing arm each, and adding a sixth operator
/// would mean a sixth type instead of a sixth variant.
///
/// This mirrors [`crate::fir_kinds::OperatorFir`]'s single-type-plus-op-tag
/// shape, but as a real enum rather than its `op: String` — a typo becomes a
/// compile error and the `match` in [`ComparisonOp::compare`] is exhaustive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparisonOp {
    /// `'lt` — strictly less than.
    Lt,
    /// `'gt` — strictly greater than.
    Gt,
    /// `'le` — less than or equal.
    Le,
    /// `'ge` — greater than or equal.
    Ge,
    /// `'eq` — equal.
    Eq,
}

impl ComparisonOp {
    /// Every comparison operator, paired with the `system.foo` name that
    /// declares it. The single source of truth for which names are comparison
    /// operators — both the installer and its tests read this list.
    const ALL: [(&'static str, ComparisonOp); 5] = [
        ("'lt", ComparisonOp::Lt),
        ("'gt", ComparisonOp::Gt),
        ("'le", ComparisonOp::Le),
        ("'ge", ComparisonOp::Ge),
        ("'eq", ComparisonOp::Eq),
    ];

    /// The operator's `system.foo` declaration name (e.g. `"'lt"`).
    #[must_use]
    pub fn searchable_name(self) -> &'static str {
        match self {
            ComparisonOp::Lt => "'lt",
            ComparisonOp::Gt => "'gt",
            ComparisonOp::Le => "'le",
            ComparisonOp::Ge => "'ge",
            ComparisonOp::Eq => "'eq",
        }
    }

    /// Recover an operator from the name [`ComparisonOp::searchable_name`]
    /// produced. Used when constanic-cloning, where only `as_op_name`'s
    /// string is reachable through `dyn Fir`.
    #[must_use]
    pub(crate) fn from_searchable_name(name: &str) -> Option<ComparisonOp> {
        ComparisonOp::ALL
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, op)| *op)
    }

    /// Run this operator's Rust comparison. The ONLY thing that differs
    /// between the five operators.
    #[must_use]
    fn compare(self, left: i64, right: i64) -> bool {
        match self {
            ComparisonOp::Lt => left < right,
            ComparisonOp::Gt => left > right,
            ComparisonOp::Le => left <= right,
            ComparisonOp::Ge => left >= right,
            ComparisonOp::Eq => left == right,
        }
    }
}

/// The Foolish source of a comparison's operands: the two statements
/// immediately preceding the operator, both SFF-marked.
///
/// Written as source and run through the ordinary compiler (rather than
/// assembled by hand) so `build_fir`'s `under_sff` rule — the rule that builds
/// descendant searches ECONSTANIC so they never run where they have no valid
/// neighbors — applies exactly as it does to any other Foolish. Hand-built
/// operands would have to re-implement that rule and could drift from it.
/// The brane-and-statement wrapper is only there because an SFF marker is not
/// valid at top level; `compile_stmt_body_under` discards it and keeps the
/// `<<…>>` body.
const OPERAND_SRC: [&str; 2] = ["{o = <<#-2>>;}", "{o = <<#-1>>;}"];

/// A comparison operator installed into `system.foo` (FOOP-33 §5.0).
///
/// **Postfix, two operands, both BEFORE the operator**: `<<#-2>> ⟨op⟩
/// `<<#-1>>`. Usage is an ordinary brane literal with the operator as a
/// member, and the result is read out with `$` (tail):
///
/// ```foolish
/// comparison = {1, 2, 'lt}$    !! 'True
/// ```
///
/// **How the operands find their values.** Inside `system.foo` the two
/// SFF-marked lookups sit ECONSTANIC — `system.foo` has no valid neighbors for
/// them, and an SFF-marked search never runs. When a user references `'lt`,
/// the ordinary reference-resolution path detaches a constanic clone and
/// recoordinates it into the user's brane (AGENTS.md "Detachment and
/// Coordination"), where `#-2`/`#-1` DO have neighbors and resolve against
/// them. No name-based or parse-time special-casing of `'lt` is involved: it
/// resolves by ordinary ancestral search, exactly as `'True` does. The
/// mechanism is pinned independently of this type by
/// `fir_kinds::tests::sff_index_operand_recoordinates_to_the_referencing_branes_neighbors`.
///
/// **Result.** Once both operands settle to integers, the Rust comparison runs
/// and the corresponding `'True`/`'False` creation — resolved by ancestral
/// search from this FIR's own position, so it is the SAME creation
/// `system.foo` defines — is stored as the settled result, becoming this FIR's
/// value on later reads (settle-once, read-many, like the other operators).
/// A non-integer operand yields `NK`.
#[derive(Debug)]
pub struct ComparisonFir {
    core: ProtoBrane,
    op: ComparisonOp,
    /// Self-reference, established via `Rc::new_cyclic`. Needed because
    /// resolving `'True`/`'False` ancestrally requires `_ab_search`, which
    /// takes `self_ref: &FirRef`, while `fir_op_step` receives only `&self`.
    /// Same pattern, and same reason, as `StatementFir::self_weak`.
    self_weak: Weak<RefCell<dyn Fir>>,
}

impl ComparisonFir {
    /// Build a comparison FIR with its two SFF-marked operand lookups.
    fn comparison(op: ComparisonOp, parent: Weak<RefCell<dyn Fir>>) -> FirRef {
        Rc::new_cyclic(|me: &Weak<RefCell<ComparisonFir>>| {
            let self_weak: Weak<RefCell<dyn Fir>> = me.clone();
            let mut core = ProtoBrane::new(vec![], parent, Nyes::Prembrionic);
            for src in OPERAND_SRC {
                // `push_foolish_child_sff_marked` PANICS if the SFF rule did
                // not take effect (a descendant search left runnable). That is
                // the guard we want: a violation would mean these operands
                // could run inside system.foo, where they must not.
                core.push_foolish_child_sff_marked(build_operand(src, &self_weak));
            }
            RefCell::new(ComparisonFir {
                core,
                op,
                self_weak,
            })
        })
    }

    /// Constanic-clone a comparison FIR onto `new_parent`.
    ///
    /// This is the clone that makes comparisons work: `'lt` is cloned out of
    /// `system.foo` and recoordinated into the brane that referenced it, and
    /// the clone's operand lookups then resolve against THAT brane's
    /// neighbours. Children are cloned by the shared
    /// `clone_children_for_constanic_clone` helper, exactly as `OperatorFir`'s
    /// clone does — the operands must come across as ordinary children so the
    /// recoordination applies to them too.
    pub(crate) fn constanic_clone(
        op: ComparisonOp,
        source: &std::cell::Ref<'_, dyn Fir>,
        new_parent: &Weak<RefCell<dyn Fir>>,
        nyes: Nyes,
        descendent_of_sfm_and_foolishly_ignorant: bool,
        skip_foolish_children: bool,
    ) -> FirRef {
        Rc::new_cyclic(|me: &Weak<RefCell<ComparisonFir>>| {
            let self_weak: Weak<RefCell<dyn Fir>> = me.clone();
            let core = ProtoBrane::clone_children_for_constanic_clone(
                source.core(),
                &self_weak,
                new_parent,
                nyes,
                descendent_of_sfm_and_foolishly_ignorant,
                skip_foolish_children,
            );
            RefCell::new(ComparisonFir {
                core,
                op,
                self_weak,
            })
        })
    }

    /// Resolve `'True`/`'False` by ordinary ancestral search from this FIR's
    /// own position. Because this FIR lives inside `system.foo`, the search
    /// finds the very creations `system.foo` declares — so a comparison's
    /// result is referentially identical to the `'True` a user's own `'True`
    /// reference resolves to (FOOP-33 §5: "the actual 'True/'False FIR object
    /// from system.foo, not a synthetic boolean").
    ///
    /// `_ab_search` returns the found STATEMENT (`'True = ⬤`), not its body —
    /// see `Brane::_ib_search`/`_search_brane`. Reading `.value()` on a bare
    /// statement is a no-op (`StatementFir::settled_result()` answers `None`
    /// in the common case), so the comparison would settle to the whole
    /// `{'True=⬤}` statement wrapper instead of the bare creation. Go through
    /// `statement_value_for_comparison`, the one documented accessor for "what
    /// does this statement actually resolve to", exactly as `IndexFir`'s `$`
    /// search does for its own result.
    fn resolve_boolean(&self, verdict: bool) -> Option<FirRef> {
        let name = if verdict { "'True" } else { "'False" };
        let self_ref = self.self_weak.upgrade()?;
        let (found, _) = self._ab_search(&self_ref, name)?;
        let body = crate::fir_kinds::statement_value_for_comparison(&found)?;
        Some(body.value())
    }

    /// Settle to `NK` with `reason`, storing the NK as this FIR's result.
    fn settle_nk(&self, reason: &str, scope: &Scope) {
        let nk_ref = crate::fir_kinds::NkFir::nk(reason, self.core.parent_weak());
        nk_ref.borrow().core().set_nyes(Nyes::Nk);
        self.core.push_ubc_child(ProtoBrane::constanic_clone_at(
            &nk_ref,
            &self.core.parent_weak(),
            0,
            scope.has_ancestral_sfm,
            false,
        ));
        self.core.set_alarm_reason(reason.to_owned());
        self.core.set_nyes(Nyes::Nk);
    }

    /// Run the comparison once both operands are constanic.
    fn combine(&self, scope: &Scope) -> Result<(), UbcError> {
        let operands = self.core.foolish_children().to_vec();

        // An ECONSTANIC operand means "not evaluated IN THIS CONTEXT — may
        // gain a value via recoordination". That is the state the operands sit
        // in inside system.foo itself, which has no neighbours for them. The
        // comparison must then settle ECONSTANIC TOO, and emphatically NOT NK:
        // NK is terminal and would poison the `'lt` DEFINITION, so a search for
        // `'lt` would hit `check_body_nyes`'s NkStop and never hand the
        // definition out to be recoordinated — the operator could never be used
        // anywhere. Settling ECONSTANIC keeps the definition available and
        // inert exactly as FOOP-33 §5.0 requires ("'lt's #-2/#-1 operands sit
        // ECONSTANIC inside system.foo ... and resolve against real neighbours
        // once the reference is detached and recoordinated").
        if operands.iter().any(operand_is_unevaluated_here) {
            self.core.set_nyes(Nyes::Econstanic);
            return Ok(());
        }

        // Read each operand THROUGH its SFF wrapper: `.value()` follows the
        // settled chain to whatever the recoordinated index landed on.
        let values: Vec<Option<i64>> = operands
            .iter()
            .map(|o| o.value().borrow().as_i64())
            .collect();

        let [Some(left), Some(right)] = values[..] else {
            // The operands DID evaluate here, and at least one is not an
            // integer (a brane, a creation, NK). FOOP-33 §5: "only integers are
            // comparable" — the same principle default_equal follows. Unlike
            // the ECONSTANIC case above, there is nothing more to learn from
            // recoordination, so NK is right.
            self.settle_nk("comparison: non-integer operand", scope);
            return Ok(());
        };

        let verdict = self.op.compare(left, right);
        let Some(boolean) = self.resolve_boolean(verdict) else {
            // system.foo always defines 'True/'False, so failing to find them
            // means the prelude itself is malformed — an interpreter defect,
            // not an unevaluable program.
            return Err(UbcError::InternalConsistency(format!(
                "system.foo must define 'True and 'False, but {} could not resolve one",
                self.op.searchable_name()
            )));
        };

        self.core.push_ubc_child(ProtoBrane::constanic_clone_at(
            &boolean,
            &self.core.parent_weak(),
            0,
            scope.has_ancestral_sfm,
            false,
        ));
        self.core.set_nyes(Nyes::Constant);
        Ok(())
    }
}

impl Fir for ComparisonFir {
    #[inline(always)]
    fn core(&self) -> &ProtoBrane {
        &self.core
    }

    fn kind(&self) -> FirKind {
        FirKind::Comparison
    }

    fn as_op_name(&self) -> Option<&str> {
        Some(self.op.searchable_name())
    }

    fn fir_op_step(&self, scope: &Scope) -> Result<(), UbcError> {
        // Same two-phase shape as OperatorFir: enqueue the operands, then
        // combine once they have settled.
        match self.core.get_nyes() {
            Nyes::Prembrionic | Nyes::Embryonic => {
                self.core.set_nyes(Nyes::Braning);
                for child in self.core.foolish_children().to_vec() {
                    self.core.push_task(child);
                }
                Ok(())
            }
            Nyes::Braning => self.combine(scope),
            _ => Ok(()),
        }
    }
}

/// Arithmetic operators `system.foo` supplies (FOOP-55 §1).
///
/// Mirrors [`ComparisonOp`]'s design: one enum for all arithmetic operators
/// that share the same two-SFF-operand, integer-result structure. Today only
/// `Mod`; `'div` and others can be added as variants later.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArithOp {
    /// `'mod` — integer modulo (truncating remainder).
    Mod,
}

impl ArithOp {
    pub(crate) const ALL: [(&'static str, ArithOp); 1] = [("'mod", ArithOp::Mod)];

    #[must_use]
    pub fn searchable_name(self) -> &'static str {
        match self {
            ArithOp::Mod => "'mod",
        }
    }

    #[must_use]
    pub(crate) fn from_searchable_name(name: &str) -> Option<ArithOp> {
        ArithOp::ALL
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, op)| *op)
    }

    #[must_use]
    fn compute(self, left: i64, right: i64) -> i64 {
        match self {
            ArithOp::Mod => left % right,
        }
    }
}

/// An arithmetic operator installed into `system.foo` (FOOP-55 §1).
///
/// Postfix, two operands, both BEFORE the operator — same shape as
/// [`ComparisonFir`]. Usage:
///
/// ```foolish
/// result = {7, 3, 'mod}$    !! 1
/// ```
///
/// **Result.** Once both operands settle to integers, the Rust arithmetic
/// runs and the integer result is stored as an [`IndepIntFir`].
/// A non-integer operand yields NK; divisor zero yields NK.
#[derive(Debug)]
pub struct ModuloFir {
    core: ProtoBrane,
    op: ArithOp,
    self_weak: Weak<RefCell<dyn Fir>>,
}

impl ModuloFir {
    fn modulo(op: ArithOp, parent: Weak<RefCell<dyn Fir>>) -> FirRef {
        Rc::new_cyclic(|me: &Weak<RefCell<ModuloFir>>| {
            let self_weak: Weak<RefCell<dyn Fir>> = me.clone();
            let mut core = ProtoBrane::new(vec![], parent, Nyes::Prembrionic);
            for src in OPERAND_SRC {
                core.push_foolish_child_sff_marked(build_operand(src, &self_weak));
            }
            RefCell::new(ModuloFir {
                core,
                op,
                self_weak,
            })
        })
    }

    pub(crate) fn constanic_clone(
        op: ArithOp,
        source: &std::cell::Ref<'_, dyn Fir>,
        new_parent: &Weak<RefCell<dyn Fir>>,
        nyes: Nyes,
        descendent_of_sfm_and_foolishly_ignorant: bool,
        skip_foolish_children: bool,
    ) -> FirRef {
        Rc::new_cyclic(|me: &Weak<RefCell<ModuloFir>>| {
            let self_weak: Weak<RefCell<dyn Fir>> = me.clone();
            let core = ProtoBrane::clone_children_for_constanic_clone(
                source.core(),
                &self_weak,
                new_parent,
                nyes,
                descendent_of_sfm_and_foolishly_ignorant,
                skip_foolish_children,
            );
            RefCell::new(ModuloFir {
                core,
                op,
                self_weak,
            })
        })
    }

    fn settle_nk(&self, reason: &str, scope: &Scope) {
        let nk_ref = crate::fir_kinds::NkFir::nk(reason, self.core.parent_weak());
        nk_ref.borrow().core().set_nyes(Nyes::Nk);
        self.core.push_ubc_child(ProtoBrane::constanic_clone_at(
            &nk_ref,
            &self.core.parent_weak(),
            0,
            scope.has_ancestral_sfm,
            false,
        ));
        self.core.set_alarm_reason(reason.to_owned());
        self.core.set_nyes(Nyes::Nk);
    }

    fn combine(&self, scope: &Scope) -> Result<(), UbcError> {
        let operands = self.core.foolish_children().to_vec();

        if operands.iter().any(operand_is_unevaluated_here) {
            self.core.set_nyes(Nyes::Econstanic);
            return Ok(());
        }

        let values: Vec<Option<i64>> = operands
            .iter()
            .map(|o| o.value().borrow().as_i64())
            .collect();

        let [Some(left), Some(right)] = values[..] else {
            self.settle_nk("modulo: non-integer operand", scope);
            return Ok(());
        };

        if right == 0 {
            self.settle_nk("division by zero", scope);
            return Ok(());
        }

        let result = self.op.compute(left, right);
        let self_weak = self.self_weak.clone();
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
        Ok(())
    }
}

impl Fir for ModuloFir {
    #[inline(always)]
    fn core(&self) -> &ProtoBrane {
        &self.core
    }

    fn kind(&self) -> FirKind {
        FirKind::Modulo
    }

    fn as_op_name(&self) -> Option<&str> {
        Some(self.op.searchable_name())
    }

    fn fir_op_step(&self, scope: &Scope) -> Result<(), UbcError> {
        match self.core.get_nyes() {
            Nyes::Prembrionic | Nyes::Embryonic => {
                self.core.set_nyes(Nyes::Braning);
                for child in self.core.foolish_children().to_vec() {
                    self.core.push_task(child);
                }
                Ok(())
            }
            Nyes::Braning => self.combine(scope),
            _ => Ok(()),
        }
    }
}

/// A boolean OR operator installed into `system.foo` (FOOP-55 §2).
///
/// Postfix, two operands, both BEFORE the operator — same shape as
/// [`ComparisonFir`]. Usage:
///
/// ```foolish
/// result = {'True, 'False, 'or}$    !! 'True
/// ```
///
/// **Result.** Once both operands settle, referential identity checks
/// against `'True`/`'False` creations determine the boolean value.
/// `left_is_true || right_is_true` → `'True`; both `'False` → `'False`.
/// A non-boolean operand yields NK.
#[derive(Debug)]
pub struct OrFir {
    core: ProtoBrane,
    self_weak: Weak<RefCell<dyn Fir>>,
}

impl OrFir {
    fn or(parent: Weak<RefCell<dyn Fir>>) -> FirRef {
        Rc::new_cyclic(|me: &Weak<RefCell<OrFir>>| {
            let self_weak: Weak<RefCell<dyn Fir>> = me.clone();
            let mut core = ProtoBrane::new(vec![], parent, Nyes::Prembrionic);
            for src in OPERAND_SRC {
                core.push_foolish_child_sff_marked(build_operand(src, &self_weak));
            }
            RefCell::new(OrFir { core, self_weak })
        })
    }

    pub(crate) fn constanic_clone(
        source: &std::cell::Ref<'_, dyn Fir>,
        new_parent: &Weak<RefCell<dyn Fir>>,
        nyes: Nyes,
        descendent_of_sfm_and_foolishly_ignorant: bool,
        skip_foolish_children: bool,
    ) -> FirRef {
        Rc::new_cyclic(|me: &Weak<RefCell<OrFir>>| {
            let self_weak: Weak<RefCell<dyn Fir>> = me.clone();
            let core = ProtoBrane::clone_children_for_constanic_clone(
                source.core(),
                &self_weak,
                new_parent,
                nyes,
                descendent_of_sfm_and_foolishly_ignorant,
                skip_foolish_children,
            );
            RefCell::new(OrFir { core, self_weak })
        })
    }

    fn settle_nk(&self, reason: &str, scope: &Scope) {
        let nk_ref = crate::fir_kinds::NkFir::nk(reason, self.core.parent_weak());
        nk_ref.borrow().core().set_nyes(Nyes::Nk);
        self.core.push_ubc_child(ProtoBrane::constanic_clone_at(
            &nk_ref,
            &self.core.parent_weak(),
            0,
            scope.has_ancestral_sfm,
            false,
        ));
        self.core.set_alarm_reason(reason.to_owned());
        self.core.set_nyes(Nyes::Nk);
    }

    fn resolve_boolean(&self, verdict: bool) -> Option<FirRef> {
        let name = if verdict { "'True" } else { "'False" };
        let self_ref = self.self_weak.upgrade()?;
        let (found, _) = self._ab_search(&self_ref, name)?;
        let body = crate::fir_kinds::statement_value_for_comparison(&found)?;
        Some(body.value())
    }

    /// Check whether a value is referentially identical to a `'True` or `'False`
    /// creation. Returns `Some(true)` for `'True`, `Some(false)` for `'False`,
    /// `None` if the value is not a boolean creation.
    fn is_boolean_creation(&self, val: &FirRef) -> Option<bool> {
        if let Some(true_ref) = self.resolve_boolean(true)
            && Rc::ptr_eq(val, &true_ref)
        {
            return Some(true);
        }
        if let Some(false_ref) = self.resolve_boolean(false)
            && Rc::ptr_eq(val, &false_ref)
        {
            return Some(false);
        }
        None
    }

    fn combine(&self, scope: &Scope) -> Result<(), UbcError> {
        let operands = self.core.foolish_children().to_vec();

        if operands.iter().any(operand_is_unevaluated_here) {
            self.core.set_nyes(Nyes::Econstanic);
            return Ok(());
        }

        let left_val = operands[0].value();
        let right_val = operands[1].value();

        let Some(left_is_true) = self.is_boolean_creation(&left_val) else {
            self.settle_nk("or: non-boolean operand", scope);
            return Ok(());
        };
        let Some(right_is_true) = self.is_boolean_creation(&right_val) else {
            self.settle_nk("or: non-boolean operand", scope);
            return Ok(());
        };

        let verdict = left_is_true || right_is_true;
        let Some(boolean) = self.resolve_boolean(verdict) else {
            return Err(UbcError::InternalConsistency(
                "system.foo must define 'True and 'False, but 'or could not resolve one"
                    .to_string(),
            ));
        };

        self.core.push_ubc_child(ProtoBrane::constanic_clone_at(
            &boolean,
            &self.core.parent_weak(),
            0,
            scope.has_ancestral_sfm,
            false,
        ));
        self.core.set_nyes(Nyes::Constant);
        Ok(())
    }
}

impl Fir for OrFir {
    #[inline(always)]
    fn core(&self) -> &ProtoBrane {
        &self.core
    }

    fn kind(&self) -> FirKind {
        FirKind::Or
    }

    fn as_op_name(&self) -> Option<&str> {
        Some("'or")
    }

    fn fir_op_step(&self, scope: &Scope) -> Result<(), UbcError> {
        match self.core.get_nyes() {
            Nyes::Prembrionic | Nyes::Embryonic => {
                self.core.set_nyes(Nyes::Braning);
                for child in self.core.foolish_children().to_vec() {
                    self.core.push_task(child);
                }
                Ok(())
            }
            Nyes::Braning => self.combine(scope),
            _ => Ok(()),
        }
    }
}

/// Has this operand gone unevaluated in the context it currently sits in?
///
/// The operand is an SFF wrapper (`<<#-1>>`) around an index search. The
/// WRAPPER settles constanic on its own account, so its NYES does not answer
/// the question; the SEARCH inside it does. ECONSTANIC there means "searched
/// nothing in this context — may gain a value via recoordination", which is
/// exactly the state the operands hold inside `system.foo`, where there are no
/// neighbours for them.
fn operand_is_unevaluated_here(operand: &FirRef) -> bool {
    operand
        .borrow()
        .core()
        .foolish_children()
        .first()
        .is_some_and(|inner| inner.borrow().core().get_nyes() == Nyes::Econstanic)
}

/// Compile one operand fragment directly beneath `parent`.
///
/// `OPERAND_SRC` is fixed, valid Foolish compiled in by this crate — a parse
/// or build failure is a defect in this module's own literal, not in any user
/// input, so `expect` states an invariant here rather than hiding a runtime
/// failure mode.
fn build_operand(src: &str, parent: &Weak<RefCell<dyn Fir>>) -> FirRef {
    crate::compiler::compile_stmt_body_under(src, parent)
        .expect("OPERAND_SRC is a fixed, valid Foolish expression")
}

/// Supply a [`ComparisonFir`] or [`ModuloFir`] body for each system operator's
/// statement.
///
/// `system.foo` declares `'lt = ⬤`, `'mod = ⬤`, etc. as ordinary
/// null-characterized creations. This supplies the actual operator logic in
/// place of those `⬤` placeholders as the composed root is compiled.
///
/// Matching is by the statement's null-characterized searchable name, and this
/// hook runs ONLY over `system.foo`'s own top-level statements.
fn system_body(
    identifier: &crate::identifier::Identifier,
    stmt_weak: &Weak<RefCell<dyn Fir>>,
) -> Option<FirRef> {
    let name = identifier.searchable_name();
    if let Some(op) = ComparisonOp::from_searchable_name(name) {
        return Some(ComparisonFir::comparison(op, stmt_weak.clone()));
    }
    if let Some(op) = ArithOp::from_searchable_name(name) {
        return Some(ModuloFir::modulo(op, stmt_weak.clone()));
    }
    if name == "'or" {
        return Some(OrFir::or(stmt_weak.clone()));
    }
    None
}

/// The embedded `system.foo` source, baked into the binary at compile time.
///
/// `OUT_DIR` is a Cargo build-script variable, read by `env!` at COMPILE time;
/// `include_str!` then bakes the file's contents into the binary right then.
/// Neither macro touches the filesystem at runtime — `system.foo` ships
/// inside the compiled crate with no runtime file dependency. See
/// `foolish-ubca/build.rs` (the copy into `OUT_DIR`) and FOOP-33.md §4's
/// `@human` note on this distinction.
pub const SYSTEM_FOO_SRC: &str = include_str!(concat!(env!("OUT_DIR"), "/system.foo"));

/// Errors composing a user program with `system.foo`.
#[derive(Debug, thiserror::Error)]
pub enum ComposeError {
    /// `system.foo` itself failed to parse — a build-time/embedding defect,
    /// not a user error.
    #[error("system.foo failed to parse: {0}")]
    SystemFooParse(String),
    /// `system.foo`'s parsed source was not a single top-level brane — a
    /// build-time/embedding defect (the prelude must be exactly one brane).
    #[error("system.foo must parse to exactly one top-level brane, found {0}")]
    SystemFooShape(usize),
    /// The user's source failed to parse — an ordinary user error.
    #[error("program failed to parse: {0}")]
    ProgramParse(String),
    /// Compiling the composed (system.foo + program) AST failed.
    #[error("composed program failed to compile: {0}")]
    Compile(String),
}

/// Compose `system.foo` with a single user program's AST, appended as a
/// statement named `program` (last), and compile the combined AST as one
/// self-rooting brane. Returns the composite root `FirRef` — callers extract
/// the user's result via [`program_result`].
fn compose_one(system_ast: Astn, program_ast: Astn) -> Result<FirRef, ComposeError> {
    let Astn::Brane {
        characterizations,
        mut statements,
    } = system_ast
    else {
        // Unreachable in practice (guarded by compose_with_program's shape
        // check before this is called) — kept as a defensive match arm
        // rather than an unwrap, per the "no panics on structured input" rule.
        return Err(ComposeError::SystemFooShape(0));
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
    // The comparison operators' bodies are supplied here, as the composed root
    // is built (FOOP-33 §5.0) — see `comparison_body`.
    crate::compiler::compile_root_with_body_override(composed, &system_body)
        .map_err(|e| ComposeError::Compile(e.to_string()))
}

/// Parse `system.foo` and the user's source, and compose ONE user top-level
/// item with `system.foo` per [`compose_one`]. The overwhelmingly common case
/// (a `.foo` file's source is exactly one top-level `{...}` brane) is `Ok`
/// with a `Vec` of length 1; a source with N top-level items composes each
/// one separately against its OWN fresh `system.foo` parse (mirroring
/// `UbcaEvaluator::evaluate`'s existing per-top-level-item loop).
pub fn compose_program_with_system(user_source: &str) -> Result<Vec<FirRef>, ComposeError> {
    let program_asts = foolish_parser::parse(user_source)
        .map_err(|e| ComposeError::ProgramParse(e.to_string()))?;
    program_asts
        .into_iter()
        .map(|program_ast| {
            let system_asts = foolish_parser::parse(SYSTEM_FOO_SRC)
                .map_err(|e| ComposeError::SystemFooParse(e.to_string()))?;
            let [system_ast] = <[Astn; 1]>::try_from(system_asts)
                .map_err(|v| ComposeError::SystemFooShape(v.len()))?;
            compose_one(system_ast, program_ast)
        })
        .collect()
}

/// Extract the `program` member's VALUE from a composed root — the LAST
/// statement of the composite brane (FOOP-33 §4: "program is retrieved
/// positionally, as the last statement of system.foo"), resolved through to
/// its settled body via [`FirRefExt::value`]. Structural access
/// (`stmt_count`/`stmt_at`), never a Foolish search — the return path must
/// not depend on the search engine. Returns the user's program's own root
/// brane, "whose own universe is exactly as it was before the prelude
/// existed" (FOOP-33.md §4) — not the wrapping `StatementFir`.
pub fn program_result(composed_root: &FirRef) -> Option<FirRef> {
    use crate::fir_trait::FirRefExt;
    let last_stmt = {
        let borrowed = composed_root.borrow();
        let count = borrowed.stmt_count()?;
        if count == 0 {
            return None;
        }
        borrowed.stmt_at(count - 1)?
    };
    // `.value()` on the STATEMENT itself would return the statement (a plain
    // StatementFir has no settled_result — FirRefExt::value falls to
    // Rc::clone(self) when there's nothing to follow). What the spec means by
    // "the program member" is the WRITTEN BODY's value — the user's program's
    // own root brane — so resolve through foolish_children().first() first.
    let body = last_stmt
        .borrow()
        .core()
        .foolish_children()
        .first()
        .cloned()?;
    Some(body.value())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fir_trait::{FirKind, FirRefExt, Scope};
    use std::rc::Rc;

    fn step_to_settled(node: &FirRef, scope: &Scope) {
        for _ in 0..20000 {
            let report = node.step(scope).unwrap();
            if let crate::fir_trait::StepReport::Progress(nyes) = report
                && nyes.is_constanic()
            {
                return;
            }
        }
    }

    /// Resolve a value through a statement wrapper, if it is one.
    ///
    /// `$` yields the tail STATEMENT (every search result is a statement, per
    /// the FoolRefFir two-child invariant); the value a Foolisher means is the
    /// statement's body. Unwrapping one level here is the same step every
    /// other statement-valued result needs, not something specific to
    /// comparisons.
    fn through_statement(value: FirRef) -> FirRef {
        if value.borrow().kind() != FirKind::Statement {
            return value;
        }
        let body = value.borrow().core().foolish_children().first().cloned();
        match body {
            Some(b) => b.value(),
            None => value,
        }
    }

    /// Evaluate `source` and return the value of its statement at `idx`.
    fn statement_value(source: &str, idx: usize) -> FirRef {
        let composed = compose_program_with_system(source).unwrap();
        let root = &composed[0];
        step_to_settled(root, &Scope::empty());
        let program = program_result(root).expect("program member must exist");
        let stmt = program.borrow().stmt_at(idx).expect("statement exists");
        let body = stmt.borrow().core().foolish_children()[0].clone();
        through_statement(body.value())
    }

    /// Evaluate `source` and return the value of its FIRST statement.
    fn first_statement_value(source: &str) -> FirRef {
        statement_value(source, 0)
    }

    /// The `'True`/`'False` creations `system.foo` itself declares.
    fn system_boolean(composed_root: &FirRef, name: &str) -> FirRef {
        let stmts = composed_root.borrow().core().foolish_children().to_vec();
        let stmt = stmts
            .iter()
            .find(|s| s.borrow().as_stmt_searchable_name() == Some(name))
            .expect("system.foo declares 'True and 'False");
        let body = stmt.borrow().core().foolish_children()[0].clone();
        body.value()
    }

    /// Step `node` to settled, recording its NYES after every step.
    fn nyes_trace(node: &FirRef, scope: &Scope) -> Vec<Nyes> {
        let mut trace = vec![node.borrow().core().get_nyes()];
        for _ in 0..20000 {
            let report = node.step(scope).unwrap();
            match report {
                crate::fir_trait::StepReport::Progress(nyes) => {
                    trace.push(nyes);
                    if nyes.is_constanic() {
                        break;
                    }
                }
                crate::fir_trait::StepReport::NoProgress => break,
            }
        }
        trace
    }

    /// The shared progression contract (AGENTS.md §"NYES transition tests"):
    /// start PREMBRIONIC, end constanic at `expected_terminal`, never regress
    /// from constanic back to pre-constanic.
    fn assert_progression(trace: &[Nyes], expected_terminal: Nyes, label: &str) {
        assert_eq!(
            trace.first().copied(),
            Some(Nyes::Prembrionic),
            "{label}: must start PREMBRIONIC (trace {trace:?})"
        );
        let last = *trace.last().expect("non-empty trace");
        assert!(
            last.is_constanic(),
            "{label}: must end constanic (got {last:?}, trace {trace:?})"
        );
        assert_eq!(
            last, expected_terminal,
            "{label}: wrong terminal state (trace {trace:?})"
        );
        let mut seen_constanic = false;
        for n in trace {
            if seen_constanic {
                assert!(
                    n.is_constanic(),
                    "{label}: regressed from constanic to {n:?} (trace {trace:?})"
                );
            }
            seen_constanic = n.is_constanic();
        }
    }

    /// Find `system.foo`'s own `'lt` comparison FIR in a composed root.
    fn system_comparison(composed_root: &FirRef, name: &str) -> FirRef {
        let stmts = composed_root.borrow().core().foolish_children().to_vec();
        let stmt = stmts
            .iter()
            .find(|s| s.borrow().as_stmt_searchable_name() == Some(name))
            .expect("system.foo declares the comparison operators");
        let body = stmt.borrow().core().foolish_children()[0].clone();
        body
    }

    #[test]
    fn comparison_nyes_transitions() {
        // Required by AGENTS.md: every FIR kind pins its NYES progression.
        // ComparisonFir has THREE terminal states, and which one it reaches is
        // the heart of the design -- so all three are pinned here.

        // 1. ECONSTANIC -- the state inside system.foo itself, where the
        //    operands have no neighbours. Load-bearing: NK here would poison
        //    the definition (check_body_nyes -> NkStop) and no search could
        //    ever hand 'lt out to be recoordinated.
        let composed = compose_program_with_system("{x = 1;}").unwrap();
        let root = &composed[0];
        let lt = system_comparison(root, "'lt");
        let trace = nyes_trace(&lt, &Scope::empty());
        assert_progression(
            &trace,
            Nyes::Econstanic,
            "Comparison (in system.foo, no neighbours)",
        );

        // 2. CONSTANT -- recoordinated into a brane with two integer
        //    neighbours, the comparison computes a boolean.
        let composed = compose_program_with_system("{r = {1, 2, 'lt}$;}").unwrap();
        let root = &composed[0];
        step_to_settled(root, &Scope::empty());
        let used = sift_for_comparison_in_program(root).expect("the referenced 'lt clone");
        assert_eq!(
            used.borrow().core().get_nyes(),
            Nyes::Constant,
            "a recoordinated comparison with integer operands settles CONSTANT"
        );

        // 3. NK -- recoordinated, operands evaluated, but not comparable.
        let composed = compose_program_with_system("{r = {1, {x = 5;}, 'lt}$;}").unwrap();
        let root = &composed[0];
        step_to_settled(root, &Scope::empty());
        let used = sift_for_comparison_in_program(root).expect("the referenced 'lt clone");
        assert_eq!(
            used.borrow().core().get_nyes(),
            Nyes::Nk,
            "a non-integer operand settles NK"
        );
    }

    /// Sift the `program` brane for the first `ComparisonFir` in it — the
    /// recoordinated clone of a `system.foo` operator. A plain Rust tree walk,
    /// not a Foolish search, hence `sift_` (AGENTS.md §Foolish Terminology).
    fn sift_for_comparison_in_program(composed_root: &FirRef) -> Option<FirRef> {
        fn sift(node: &FirRef, depth: usize) -> Option<FirRef> {
            if depth > 20 {
                return None;
            }
            if node.borrow().kind() == FirKind::Comparison {
                return Some(Rc::clone(node));
            }
            let children = node.borrow().core().all_children();
            children.iter().find_map(|c| sift(c, depth + 1))
        }
        let program = program_result(composed_root)?;
        sift(&program, 0)
    }

    #[test]
    fn each_comparison_operator_produces_the_right_boolean() {
        // The whole feature, end to end, for all five operators and both
        // outcomes. `{a, b, 'op}$`: the brane literal's tail is 'op, whose
        // settled value is the boolean it computed from its two preceding
        // neighbours (FOOP-33 §5.0).
        //
        // Expectations are written as the plain Rust comparison of 1 and 2 so
        // each row states WHY it is what it is, not merely what was observed:
        // 1 < 2 is true, 1 > 2 is false, and so on.
        for (op, expected) in [
            ("'lt", 1 < 2),
            ("'gt", 1 > 2),
            ("'le", 1 <= 2),
            ("'ge", 1 >= 2),
            ("'eq", 1 == 2),
        ] {
            let source = format!("{{r = {{1, 2, {op}}}$;}}");
            let composed = compose_program_with_system(&source).unwrap();
            let root = &composed[0];
            step_to_settled(root, &Scope::empty());
            let program = program_result(root).unwrap();
            let stmt = program.borrow().stmt_at(0).unwrap();
            let body = stmt.borrow().core().foolish_children()[0].clone();
            let got = through_statement(body.value());

            assert_eq!(
                got.borrow().kind(),
                FirKind::Creation,
                "{op} must produce a creation ('True/'False), not {:?}",
                got.borrow().kind()
            );
            let want = system_boolean(root, if expected { "'True" } else { "'False" });
            assert!(
                Rc::ptr_eq(&got, &want),
                "{{1, 2, {op}}}$ must be system.foo's own {} creation \
                 (referential identity, FOOP-33 §5), expected={expected}",
                if expected { "'True" } else { "'False" }
            );
        }
    }

    #[test]
    fn comparison_operands_come_from_the_referencing_brane() {
        // Operand ORDER is postfix `<<#-2>> op <<#-1>>`: the first operand is
        // the one two back, the second is the one immediately back. If the two
        // were swapped, {2, 1, 'lt} and {1, 2, 'lt} would both give the same
        // answer -- so asserting BOTH directions pins the order, not just that
        // a comparison happened.
        let true_ = |src: &str| {
            let v = first_statement_value(src);
            let kind = v.borrow().kind();
            assert_eq!(kind, FirKind::Creation, "{src} must produce a boolean");
            v
        };
        let lt_yes = true_("{r = {1, 2, 'lt}$;}");
        let lt_no = true_("{r = {2, 1, 'lt}$;}");
        assert!(
            !Rc::ptr_eq(&lt_yes, &lt_no),
            "{{1,2,'lt}} and {{2,1,'lt}} must differ -- operands are ordered, \
             not a commutative pair"
        );
    }

    #[test]
    fn comparison_reads_its_own_branes_neighbours_not_another_branes() {
        // Two independent uses in one program must each compare THEIR OWN
        // neighbours. This is the recoordination property that makes the
        // single system.foo definition reusable: each reference detaches its
        // own clone, coordinated into its own brane.
        let source = "{a = {1, 2, 'lt}$; b = {9, 4, 'lt}$;}";
        let composed = compose_program_with_system(source).unwrap();
        let root = &composed[0];
        step_to_settled(root, &Scope::empty());
        let program = program_result(root).unwrap();

        let value_of = |idx: usize| {
            let stmt = program.borrow().stmt_at(idx).unwrap();
            let body = stmt.borrow().core().foolish_children()[0].clone();
            through_statement(body.value())
        };
        let a = value_of(0); // 1 < 2 -> 'True
        let b = value_of(1); // 9 < 4 -> 'False
        assert!(
            Rc::ptr_eq(&a, &system_boolean(root, "'True")),
            "a: 1 < 2 is true"
        );
        assert!(
            Rc::ptr_eq(&b, &system_boolean(root, "'False")),
            "b: 9 < 4 is false"
        );
    }

    #[test]
    fn comparison_with_a_non_integer_operand_settles_nk() {
        // FOOP-33 §5: "only integers are comparable" -- the same principle
        // default_equal follows. A brane operand is not comparable, so the
        // comparison settles NK rather than inventing an ordering.
        let value = first_statement_value("{r = {1, {x = 5;}, 'lt}$;}");
        assert_eq!(
            value.borrow().kind(),
            FirKind::Nk,
            "a brane operand must make the comparison NK, not a boolean"
        );
    }

    #[test]
    fn comparison_operators_do_not_disturb_true_and_false() {
        // system.foo gaining five members must not change what 'True/'False
        // resolve to -- the Phase 4/5 behaviour they already have.
        let value = first_statement_value("{t = 'True;}");
        assert_eq!(value.borrow().kind(), FirKind::Creation);
    }

    #[test]
    fn a_bare_comparison_brane_without_dollar_is_not_the_result() {
        // FOOP-33 §5.0: "the brane literal by itself is not the full
        // expression" -- without `$` the value is the BRANE, and the boolean
        // is merely its tail. Pins that `$` is doing real work here and the
        // brane is not silently collapsing to its last member.
        let value = first_statement_value("{r = {1, 2, 'lt};}");
        assert_eq!(
            value.borrow().kind(),
            FirKind::Brane,
            "without $, the value is the brane itself"
        );
    }

    #[test]
    fn program_redefining_true_to_a_conflicting_value_is_refused() {
        // A user brane's `'True = 3` conflicts with system.foo's ancestral
        // `'True = ⬤` -- refused via the AB-search path (found: true, per
        // AGENTS.md "Detachment and Coordination" -- ordinary ancestral
        // search, no special-casing). `'True = 'True` (same creation,
        // resolved via search) is permitted in between.
        let source = "{restate = 'True; 'True = 'True; conflict = 'True; 'True = 3;}";
        let composed = compose_program_with_system(source).unwrap();
        let root = &composed[0];
        let scope = Scope::empty();
        step_to_settled(root, &scope);
        let program = program_result(root).expect("program member must exist");
        let stmts = program.borrow().core().foolish_children().to_vec();
        assert!(
            stmts[1].borrow().settled_result().is_none(),
            "'True = 'True (same creation) must be permitted"
        );
        let reason = stmts[3]
            .borrow()
            .settled_result()
            .expect("'True = 3 conflicts with system.foo's ancestral 'True -- must be refused")
            .borrow()
            .as_nk_reason()
            .unwrap()
            .to_owned();
        assert_eq!(reason, "'True not-foolish");
    }

    #[test]
    fn referenced_creations_own_parent_chain_reaches_its_defining_brane() {
        // Phase 5.5 precondition: when a user program references 'True (a
        // creation constructed inside system.foo's own AST), the REFERENCED
        // creation's OWN parent chain still leads back to system.foo -- NOT
        // to wherever it's read from. Constanic clone of an Independent
        // creation returns the SAME Rc (Gotcha #2), so its `core().parent()`
        // (set at ORIGINAL construction time, inside system.foo's AST) is
        // unaffected by detachment/recoordination at the reference site. This
        // is what lets a name-rendering pass, given only the creation FirRef,
        // walk `_get_my_brane` to find system.foo and search it for the
        // defining 'True statement -- without needing the reader to already
        // know which brane it came from.
        let composed = compose_program_with_system("{t1 = 'True;}").unwrap();
        let root = &composed[0];
        let scope = Scope::empty();
        step_to_settled(root, &scope);
        let program = program_result(root).expect("program member must exist");
        let stmts = program.borrow().core().foolish_children().to_vec();
        let t1_body = stmts[0].borrow().core().foolish_children()[0].clone();
        let t1_value = t1_body.value();
        assert_eq!(t1_value.borrow().kind(), FirKind::Creation);

        let home_brane = t1_value
            .borrow()
            ._get_my_brane(&t1_value)
            .expect("creation must have a home brane");
        assert!(
            Rc::ptr_eq(&home_brane, root),
            "the creation's home brane must be system.foo (the composed root), \
             not `program` (where it's merely referenced from)"
        );
    }

    #[test]
    fn system_foo_src_embeds_true_and_false() {
        assert!(SYSTEM_FOO_SRC.contains("'True"));
        assert!(SYSTEM_FOO_SRC.contains("'False"));
    }

    #[test]
    fn compose_appends_program_as_last_statement() {
        let composed = compose_program_with_system("{x = 1;}").unwrap();
        assert_eq!(composed.len(), 1);
        let root = &composed[0];
        assert_eq!(root.borrow().kind(), FirKind::Brane);
        // system.foo's own statements + 1 appended (`program`, always LAST --
        // the invariant `program_result`'s positional access depends on).
        let count = root
            .borrow()
            .stmt_count()
            .expect("composed root is a brane");
        let program_stmt = root.borrow().stmt_at(count - 1).unwrap();
        assert_eq!(
            program_stmt.borrow().as_stmt_searchable_name(),
            Some("program")
        );
    }

    #[test]
    fn composed_root_is_self_rooting() {
        let composed = compose_program_with_system("{x = 1;}").unwrap();
        let root = &composed[0];
        assert!(root.borrow().core().is_root(root));
    }

    #[test]
    fn program_resolves_true_and_false_ancestrally() {
        // The user references the null-characterized constants WITH the
        // leading quote (`'True`, `'False`) -- a bare `True` (no quote) can
        // never match, by design: the matcher compares against
        // Identifier::searchable_name() unconditionally, and a plain
        // pattern's compiled form (`^True$`) does not match the
        // null-characterized `searchable_name()` `"'True"` (FOOP-33.md line
        // ~324: "'True does not match a plainly-named True").
        let composed = compose_program_with_system("{a = 'True; b = 'False;}").unwrap();
        let root = &composed[0];
        let scope = Scope::empty();
        step_to_settled(root, &scope);
        let program = program_result(root).expect("program member must exist");
        assert_eq!(program.borrow().stmt_count(), Some(2));
        let a_stmt = program.borrow().stmt_at(0).unwrap();
        let a_body = a_stmt.borrow().core().foolish_children()[0].clone();
        assert_eq!(a_body.value().borrow().kind(), FirKind::Creation);
    }

    #[test]
    fn program_line_numbers_are_preserved() {
        // A one-line user program's statement still reports its OWN original
        // line number (0) -- system.foo's siblings do not shift it, because
        // line numbers are 0-based indices assigned PER-BRANE, and `program`'s
        // brane is a different brane than system.foo's.
        let composed = compose_program_with_system("{x = 1;}").unwrap();
        let root = &composed[0];
        let program = program_result(root).expect("program member must exist");
        let x_stmt = program.borrow().stmt_at(0).unwrap();
        assert_eq!(x_stmt.borrow().as_stmt_line_number(), Some(0));
    }

    #[test]
    fn ab_search_terminates_at_system_root_no_infinite_walk() {
        // The composite root is its own parent (self-rooting) -- _ab_search
        // must terminate there rather than looping. A bounded step count
        // settling to Independent/Constant (not hanging/panicking) is proof.
        let composed = compose_program_with_system("{a = 'True;}").unwrap();
        let root = &composed[0];
        let scope = Scope::empty();
        step_to_settled(root, &scope);
        assert!(root.borrow().core().get_nyes().is_constanic());
    }

    // ── FOOP-55: 'or (boolean OR, pure-Foolish truth table) ───────────

    #[test]
    fn or_all_four_rows() {
        // FOOP-55 §2: 'or is a truth-table brane in system.foo, applied by
        // ordinary search. All four rows must produce the RIGHT creation
        // (referential identity, not just display).
        for (a, b, expected) in [
            ("'True", "'True", true),
            ("'True", "'False", true),
            ("'False", "'True", true),
            ("'False", "'False", false),
        ] {
            let source = format!("{{r = {{{a}, {b}, 'or}}$;}}");
            let composed = compose_program_with_system(&source).unwrap();
            let root = &composed[0];
            step_to_settled(root, &Scope::empty());
            let program = program_result(root).unwrap();
            let stmt = program.borrow().stmt_at(0).unwrap();
            let body = stmt.borrow().core().foolish_children()[0].clone();
            let got = through_statement(body.value());
            let want = system_boolean(root, if expected { "'True" } else { "'False" });
            assert!(
                Rc::ptr_eq(&got, &want),
                "{{{a}, {b}, 'or}}$ must be system.foo's own {} creation \
                 (referential identity), expected={expected}",
                if expected { "'True" } else { "'False" }
            );
        }
    }

    #[test]
    fn or_non_boolean_argument_settles_nk() {
        // FOOP-55 §2: non-boolean arguments → the lookup finds no row →
        // anchored miss → NK.
        let value = first_statement_value("{r = {3, 'True, 'or}$;}");
        assert_eq!(
            value.borrow().kind(),
            FirKind::Nk,
            "a non-boolean first argument must make 'or settle NK"
        );
    }

    // ── FOOP-55: 'mod (integer modulo, system operator) ──────────────

    /// Find `system.foo`'s own `'mod` modulo FIR in a composed root.
    fn system_modulo(composed_root: &FirRef) -> FirRef {
        let stmts = composed_root.borrow().core().foolish_children().to_vec();
        let stmt = stmts
            .iter()
            .find(|s| s.borrow().as_stmt_searchable_name() == Some("'mod"))
            .expect("system.foo declares 'mod");
        stmt.borrow().core().foolish_children()[0].clone()
    }

    /// Sift the `program` brane for the first `ModuloFir` in it.
    fn sift_for_modulo_in_program(composed_root: &FirRef) -> Option<FirRef> {
        fn sift(node: &FirRef, depth: usize) -> Option<FirRef> {
            if depth > 20 {
                return None;
            }
            if node.borrow().kind() == FirKind::Modulo {
                return Some(Rc::clone(node));
            }
            let children = node.borrow().core().all_children();
            children.iter().find_map(|c| sift(c, depth + 1))
        }
        let program = program_result(composed_root)?;
        sift(&program, 0)
    }

    #[test]
    fn modulo_nyes_transitions() {
        // Required by AGENTS.md: every FIR kind pins its NYES progression.
        // ModuloFir has THREE terminal states.

        // 1. ECONSTANIC — inside system.foo itself, operands have no neighbours.
        let composed = compose_program_with_system("{x = 1;}").unwrap();
        let root = &composed[0];
        let modulo = system_modulo(root);
        let trace = nyes_trace(&modulo, &Scope::empty());
        assert_progression(
            &trace,
            Nyes::Econstanic,
            "Modulo (in system.foo, no neighbours)",
        );

        // 2. CONSTANT — recoordinated into a brane with two integer neighbours.
        let composed = compose_program_with_system("{r = {7, 3, 'mod}$;}").unwrap();
        let root = &composed[0];
        step_to_settled(root, &Scope::empty());
        let used = sift_for_modulo_in_program(root).expect("the referenced 'mod clone");
        assert_eq!(
            used.borrow().core().get_nyes(),
            Nyes::Constant,
            "a recoordinated modulo with integer operands settles CONSTANT"
        );

        // 3. NK — recoordinated, operands evaluated, but not integers.
        let composed = compose_program_with_system("{r = {1, {x = 5;}, 'mod}$;}").unwrap();
        let root = &composed[0];
        step_to_settled(root, &Scope::empty());
        let used = sift_for_modulo_in_program(root).expect("the referenced 'mod clone");
        assert_eq!(
            used.borrow().core().get_nyes(),
            Nyes::Nk,
            "a non-integer operand settles NK"
        );
    }

    /// FOOP-55 §7: `ExtremumFir` selects an order statistic from the integers
    /// that precede it in the flattened concatenation.
    ///
    /// `'min_int_val` is index 0 of the ascending sort; `'max_int_val` is
    /// index -1 — the same 0-based-with-negatives convention `#` already uses
    /// for source order.
    #[test]
    fn extremum_selects_min_and_max() {
        for (op, expected) in [("'max_int_val", 0i64), ("'min_int_val", -2i64)] {
            let src = format!("{{r = {{-1, -2, 0}}{op};}}");
            let value = first_statement_value(&src);
            assert_eq!(
                value.borrow().as_i64(),
                Some(expected),
                "{{-1, -2, 0}}{op} sorts ascending to [-2, -1, 0]; \
                 'min_int_val takes index 0 and 'max_int_val index -1"
            );
        }
    }

    /// The result is INDEPENDENT — a self-contained constant depending on
    /// nothing outside itself — not merely CONSTANT.
    #[test]
    fn extremum_result_is_independent() {
        let value = first_statement_value("{r = {-1, -2, 0}'max_int_val;}");
        assert_eq!(
            value.borrow().core().get_nyes(),
            Nyes::Independent,
            "the extremum of literal integers depends on nothing outside \
             itself, so it settles INDEPENDENT"
        );
    }

    /// Non-integer members are SKIPPED, not fatal — the deliberate difference
    /// from `'mod`/`'or`, which name their operands positionally and settle NK
    /// on a non-integer. A fold has no fixed arity, so a member that is not an
    /// integer simply is not a candidate.
    #[test]
    fn extremum_skips_non_integer_members() {
        let value = first_statement_value("{r = {1, {x=9;}, 7}'max_int_val;}");
        assert_eq!(
            value.borrow().as_i64(),
            Some(7),
            "a brane member is not an integer candidate -- it is skipped and \
             the fold continues over the integers that remain"
        );
    }

    /// No integers at all: the extremum of an empty set is not a value, and
    /// unlike a deferral there is nothing recoordination could supply. NK.
    #[test]
    fn extremum_with_no_integer_candidates_settles_nk() {
        let value = first_statement_value("{r = {{x=1;}}'max_int_val;}");
        assert_eq!(
            value.borrow().core().get_nyes(),
            Nyes::Nk,
            "no integer candidates means there is no maximum"
        );
    }

    #[test]
    fn modulo_basic_semantics() {
        // 7 % 3 = 1
        let value = first_statement_value("{r = {7, 3, 'mod}$;}");
        assert_eq!(value.borrow().as_i64(), Some(1), "7 mod 3 must be 1");
    }

    #[test]
    fn modulo_zero_dividend() {
        // 0 % 5 = 0
        let value = first_statement_value("{r = {0, 5, 'mod}$;}");
        assert_eq!(value.borrow().as_i64(), Some(0), "0 mod 5 must be 0");
    }

    #[test]
    fn modulo_negative_dividend() {
        // Rust truncating remainder: -7 % 3 = -1
        let value = first_statement_value("{r = {(-7), 3, 'mod}$;}");
        assert_eq!(
            value.borrow().as_i64(),
            Some(-1),
            "(-7) mod 3 must be -1 (Rust truncating remainder)"
        );
    }

    #[test]
    fn modulo_negative_divisor() {
        // Rust truncating remainder: 7 % -3 = 1
        let value = first_statement_value("{r = {7, (-3), 'mod}$;}");
        assert_eq!(
            value.borrow().as_i64(),
            Some(1),
            "7 mod (-3) must be 1 (Rust truncating remainder)"
        );
    }

    #[test]
    fn modulo_division_by_zero_settles_nk() {
        let value = first_statement_value("{r = {7, 0, 'mod}$;}");
        assert_eq!(
            value.borrow().kind(),
            FirKind::Nk,
            "division by zero must settle NK"
        );
    }

    #[test]
    fn modulo_non_integer_operand_settles_nk() {
        let value = first_statement_value("{r = {7, {x = 5;}, 'mod}$;}");
        assert_eq!(
            value.borrow().kind(),
            FirKind::Nk,
            "a brane operand must make the modulo NK"
        );
    }

    // ── FOOP-75: Assignment Attached Searches ──────────────────────────

    /// FOOP-54 §D.5 (a `Complete` FOOP, the in-force authority):
    /// `a =$ b` ≡ `a = b$` — "bind the value of the LAST statement of `b`
    /// to the name `a`."
    ///
    /// Measured on jia@dc6db093, BEFORE FOOP-75: this yielded the whole
    /// brane `{1;2;3}` (WOCONSTANIC), not the tail. The old `=$` built
    /// `BinaryOp("$", UnanchoredSeek{-1}, rhs)`, whose `"$"` arm in
    /// `fir_kinds.rs` validated the RHS was a brane and then returned
    /// WITHOUT extracting anything. FOOP-75 §7 routes `=$` through
    /// `IndexFir` instead, exactly as postfix `b$` always did.
    #[test]
    fn foop75_attached_tail_binds_the_tail_value() {
        let v = statement_value("{b = {1,2,3}; y =$ b}", 1);
        assert_eq!(
            v.borrow().as_i64(),
            Some(3),
            "`y =$ b` must bind b's TAIL (3), per FOOP-54 §D.5"
        );
    }

    /// FOOP-75 §7 / §Motivation defect (2): `=^` did not evaluate AT ALL
    /// before this FOOP — there was no `"^"` arm in `fir_kinds.rs` to match
    /// the `"$"` one, so the OperatorFir never settled and leaked
    /// `y=Op^({1;2;3}, {1;2;3}, WOCONSTANIC)` into rendered output.
    #[test]
    fn foop75_attached_head_binds_the_head_value() {
        let v = statement_value("{b = {1,2,3}; y =^ b}", 1);
        assert_eq!(
            v.borrow().as_i64(),
            Some(1),
            "`y =^ b` must bind b's HEAD (1)"
        );
    }

    /// FOOP-75 §2 tree identity implies VALUE identity: the attached and
    /// postfix spellings are the same program, so they must settle alike.
    #[test]
    fn foop75_attached_and_postfix_settle_identically() {
        for (attached, postfix, expected) in [
            ("{b = {1,2,3}; y =$ b}", "{b = {1,2,3}; y = b$}", 3),
            ("{b = {1,2,3}; y =^ b}", "{b = {1,2,3}; y = b^}", 1),
        ] {
            let a = statement_value(attached, 1);
            let p = statement_value(postfix, 1);
            assert_eq!(a.borrow().as_i64(), Some(expected), "{attached}");
            assert_eq!(
                a.borrow().as_i64(),
                p.borrow().as_i64(),
                "attached and postfix must agree: {attached} vs {postfix}"
            );
        }
    }

    /// FOOP-75 §8 / AGENTS.md §Searches: an ANCHORED miss settles NK. `4` is
    /// not a brane, so its tail is provably unfindable.
    ///
    /// This is the case pinned by the frozen verified baseline
    /// `regression/disappearing_brane_statements` (input `d =$ 4`). The
    /// OUTCOME is unchanged by FOOP-75 — still NK — though the rendered text
    /// changes, which is why that baseline needs a human signing decision.
    #[test]
    fn foop75_attached_tail_on_non_brane_settles_nk() {
        let composed = compose_program_with_system("{d =$ 4}").unwrap();
        let root = &composed[0];
        step_to_settled(root, &Scope::empty());
        let program = program_result(root).expect("program member must exist");
        let stmt = program.borrow().stmt_at(0).expect("statement exists");
        let body = stmt.borrow().core().foolish_children()[0].clone();
        assert_eq!(
            body.borrow().core().get_nyes(),
            Nyes::Nk,
            "a tail search anchored on the non-brane `4` settles NK \
             (AGENTS.md §Searches: anchored miss → NK)"
        );

        // FOOP-75 §7: settling NK is only half the answer — the Foolisher
        // needs to know WHY. The deleted `OperatorFir` `"$"` arm recorded
        // `"4 is not a brane"`; the `IndexFir` path must record the same
        // diagnosis, or the rendered output degrades from
        //     d =$ ??? (4 is not a brane)
        // to a bare
        //     d =$ 4 (???)
        // which says the result is unknown without saying what went wrong.
        assert_eq!(
            body.borrow().core().alarm_reason().as_deref(),
            Some("4 is not a brane"),
            "an anchored search on a non-brane must say WHY it settled NK"
        );
    }

    /// FOOP-75 §7: the non-brane diagnosis names the offending value, so
    /// distinct anchors give distinct messages rather than one generic one.
    #[test]
    fn foop75_non_brane_anchor_names_the_value() {
        for (src, expected) in [
            ("{d =$ 4}", "4 is not a brane"),
            ("{d =^ 7}", "7 is not a brane"),
        ] {
            let composed = compose_program_with_system(src).unwrap();
            let root = &composed[0];
            step_to_settled(root, &Scope::empty());
            let program = program_result(root).expect("program member must exist");
            let stmt = program.borrow().stmt_at(0).expect("statement exists");
            let body = stmt.borrow().core().foolish_children()[0].clone();
            assert_eq!(
                body.borrow().core().alarm_reason().as_deref(),
                Some(expected),
                "{src} must diagnose its own anchor"
            );
        }
    }

    /// FOOP-75 §4: the non-brane diagnosis must reach RENDERED output, not
    /// just the `alarm_reason` field — `alarm_reason` is never read on the
    /// sequencing path, so a reason recorded there alone is invisible.
    ///
    /// Pins the exact text of the frozen `verified/` baseline
    /// `regression/disappearing_brane_statements` (input `d =$ 4`). The
    /// reason travels as the search's RESULT and takes the value slot; an
    /// earlier attempt rendered `d =$ 4 (??? (4 is not a brane))`, doubling
    /// the anchor.
    #[test]
    fn foop75_non_brane_reason_reaches_rendered_output() {
        use foolish_core::{Evaluator, FirSequencer};
        let firs = crate::UbcaEvaluator.evaluate("{a = 1; d =$ 4}").unwrap();
        let rendered = firs
            .iter()
            .map(|f| FirSequencer::format(&foolish_core::clone_steppable(f)))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            rendered.contains("d =$ ??? (4 is not a brane)"),
            "the rendered statement must carry the diagnosis; got:\n{rendered}"
        );
    }

    /// A program that cannot settle must RENDER as NK with the
    /// ITERATION-EXCEEDED alarm, not as a pre-constanic `BRANING` brane.
    ///
    /// `{f1 = {f1}; stuck = f1;}` is self-referential: `f1`'s body searches
    /// for `f1`, so stepping never reaches a fixed point and the evaluator's
    /// step cap fires.
    ///
    /// Regression guard. `evaluate` sets the alarm and NK on the COMPOSED
    /// ROOT, but `program_result` then reaches past that root to the user's
    /// `program` member and renders it instead — so the error state landed on
    /// a wrapper that was discarded. Introduced when system.foo composition
    /// began extracting the program member (FOOP-33); before that the root
    /// itself was rendered. Output regressed:
    ///
    ///     {NK(ITERATION-EXCEEDED, Iteration exceeded 9999)   →   {BRANING
    #[test]
    fn non_settling_program_renders_nk_with_iteration_alarm() {
        use foolish_core::{Evaluator, FirSequencer};
        let firs = crate::UbcaEvaluator
            .evaluate("{\n  f1 = { f1 }\n  stuck = f1;\n}")
            .expect("evaluation returns a rendering even when it cannot settle");
        let rendered = firs
            .iter()
            .map(|f| FirSequencer::format(&foolish_core::clone_steppable(f)))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            rendered.contains("NK(ITERATION-EXCEEDED"),
            "a program that exceeds the step cap must render NK with the \
             alarm, not a pre-constanic state; got:\n{rendered}"
        );
        assert!(
            !rendered.starts_with("{BRANING"),
            "the rendered brane must not be left pre-constanic; got:\n{rendered}"
        );
    }
}
