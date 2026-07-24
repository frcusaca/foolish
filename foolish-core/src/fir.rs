use std::cell::RefCell;
use std::rc::Rc;

// Re-export for external use
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Diagnostic severity levels for compiler and evaluator alarms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlarmLevel {
    Info,  // Trace-level: useful for debugging
    Warn,  // Potential issue in user code
    Mild,  // Notable event (division by zero, etc.)
    Panic, // Internal error — should never happen
}

impl std::fmt::Display for AlarmLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlarmLevel::Info => write!(f, "INFO"),
            AlarmLevel::Warn => write!(f, "WARN"),
            AlarmLevel::Mild => write!(f, "MILD"),
            AlarmLevel::Panic => write!(f, "PANIC"),
        }
    }
}

/// Source of an alarm (compiler or evaluator)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlarmSource {
    Compiler,
    Evaluator,
}

/// A structured diagnostic message
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Alarm {
    pub level: AlarmLevel,
    pub code: String,
    pub message: String,
    pub source: AlarmSource,
}

impl std::fmt::Display for Alarm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}: {}", self.level, self.code, self.message)
    }
}

/// Trait for recording alarms
pub trait AlarmSink {
    fn record(&self, alarm: Alarm);
}

/// Collects alarms into a Vec
pub struct VecAlarmSink {
    alarms: RefCell<Vec<Alarm>>,
}

impl VecAlarmSink {
    pub fn new() -> Self {
        Self {
            alarms: RefCell::new(Vec::new()),
        }
    }

    pub fn get_alarms(&self) -> Vec<Alarm> {
        self.alarms.borrow().clone()
    }
}

impl AlarmSink for VecAlarmSink {
    fn record(&self, alarm: Alarm) {
        self.alarms.borrow_mut().push(alarm);
    }
}

impl Default for VecAlarmSink {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for VecAlarmSink {
    fn clone(&self) -> Self {
        Self {
            alarms: RefCell::new(self.alarms.borrow().clone()),
        }
    }
}

impl std::fmt::Debug for VecAlarmSink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VecAlarmSink")
            .field("alarms", &self.alarms.borrow())
            .finish()
    }
}

/// Result of a single step operation (for step_members reporting)
#[derive(Debug, Clone, PartialEq)]
pub enum StepResult {
    NoOp,
    MadeProgress,
    NewChildren,
    Blocked,
}

impl StepResult {
    pub fn is_terminal(&self) -> bool {
        matches!(self, StepResult::NoOp)
    }
}

/// Shared state for all FIRs
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Nyes {
    #[serde(rename = "PREMBRIONIC")]
    Prembrionic,
    #[serde(rename = "EMBRYONIC")]
    Embryonic,
    #[serde(rename = "BRANING")]
    Braning,
    #[serde(rename = "ECONSTANIC")]
    Econstanic,
    #[serde(rename = "WOCONSTANIC")]
    Woconstanic,
    #[serde(rename = "CONSTANT")]
    Constant,
    #[serde(rename = "INDEPENDENT")]
    Independent,
    #[serde(rename = "NK")]
    Nk,
}

impl Nyes {
    pub fn is_constanic(&self) -> bool {
        matches!(
            self,
            Nyes::Econstanic | Nyes::Woconstanic | Nyes::Constant | Nyes::Independent | Nyes::Nk
        )
    }

    pub fn is_nye(&self) -> bool {
        matches!(self, Nyes::Prembrionic | Nyes::Embryonic | Nyes::Braning)
    }

    /// Constanic but NOT NK (FOOP-62 Terminology) — for the HFS search display rule.
    pub fn is_nnk_constanic(&self) -> bool {
        self.is_constanic() && !matches!(self, Nyes::Nk)
    }

    /// Returns true when the NYES should be displayed in the humanizing
    /// sequencer output.
    ///
    /// Rules (FOOP-62 §9):
    /// - CONSTANT/INDEPENDENT → no nyes (always omitted)
    /// - else → show nyes (including EMBRYONIC, NK, WOCONSTANIC, etc.)
    pub fn should_show_nyes(&self) -> bool {
        !matches!(self, Nyes::Constant | Nyes::Independent)
    }

    /// HFS NYES display rule for search / Index renders (FOOP-62 §9.x).
    /// These nodes carry a result (the resolved result/anchor), so the display
    /// rule depends on whether a result is present:
    /// - a) result present AND nnk_constanic → do NOT show nyes (the reader infers
    ///   it from the result object).
    /// - b) no result AND EMBRYONIC → do NOT show nyes.
    /// - otherwise → show nyes (even PREMBRYONIC). NK still shows (with its reason).
    pub fn should_show_search_nyes(&self, has_result: bool) -> bool {
        if has_result && matches!(self, Nyes::Constant | Nyes::Independent) {
            return false;
        }
        if !has_result && matches!(self, Nyes::Embryonic) {
            return false;
        }
        true
    }

    /// Transform NYES for constanic-clone: the clone's initial state.
    ///
    /// When a FIR is cloned during evaluation (constanic-clone), the clone
    /// needs a fresh NYES that reflects its role as a "re-evaluation seed":
    /// - SFM-descendant: preserve the source NYES verbatim (foolishly ignorant).
    /// - CONSTANT / INDEPENDENT / NK: keep as-is (terminal, no re-evaluation).
    /// - Everything else → EMBRYONIC (clone must re-evaluate in new context).
    pub fn transform_for_clone(self, descendent_of_sfm_and_foolishly_ignorant: bool) -> Nyes {
        if descendent_of_sfm_and_foolishly_ignorant {
            return self;
        }
        match self {
            Nyes::Constant | Nyes::Independent | Nyes::Nk => self,
            Nyes::Econstanic
            | Nyes::Woconstanic
            | Nyes::Prembrionic
            | Nyes::Embryonic
            | Nyes::Braning => Nyes::Embryonic,
        }
    }
}

impl std::fmt::Display for Nyes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Nyes::Prembrionic => write!(f, "PREMBRIONIC"),
            Nyes::Embryonic => write!(f, "EMBRYONIC"),
            Nyes::Braning => write!(f, "BRANING"),
            Nyes::Econstanic => write!(f, "ECONSTANIC"),
            Nyes::Woconstanic => write!(f, "WOCONSTANIC"),
            Nyes::Constant => write!(f, "CONSTANT"),
            Nyes::Independent => write!(f, "INDEPENDENT"),
            Nyes::Nk => write!(f, "NK"),
        }
    }
}

/// Search direction for anchored searches
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SearchDirection {
    Forward,
    #[default]
    Backward,
}

impl std::fmt::Display for SearchDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchDirection::Forward => write!(f, "FORWARD"),
            SearchDirection::Backward => write!(f, "BACKWARD"),
        }
    }
}

// Forward decl — structs use FirRef, enum wraps structs
pub type FirRef = Rc<RefCell<dyn Steppable>>;

/// Statement: name -> body
#[derive(Debug, Clone)]
pub struct StatementFir {
    pub(crate) name: Option<String>,
    pub(crate) body: FirRef,
    pub(crate) state: Nyes,
}

impl StatementFir {
    pub fn new(name: Option<String>, body: FirRef) -> Self {
        Self {
            name,
            body,
            state: Nyes::Embryonic,
        }
    }

    pub fn anonymous(body: FirRef) -> Self {
        Self::new(None, body)
    }

    pub fn name(&self) -> &Option<String> {
        &self.name
    }
    pub fn body(&self) -> &FirRef {
        &self.body
    }
    pub fn state(&self) -> Nyes {
        self.state
    }
}

// ==================== FIR Struct Definitions ====================

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ConstantIntFir {
    pub(crate) value: i64,
    pub(crate) state: Nyes,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NkFir {
    pub(crate) reason: String,
    pub(crate) state: Nyes,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) alarm: Option<Alarm>,
}

impl NkFir {
    pub fn with_reason(reason: String) -> Self {
        Self {
            reason,
            state: Nyes::Nk,
            alarm: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct OperatorFir {
    pub(crate) op: String,
    pub(crate) operands: Vec<FirRef>,
    pub(crate) state: Nyes,
}

impl OperatorFir {
    /// Return the operator name (e.g. "+", "-", "*").
    pub fn op_name(&self) -> &str {
        &self.op
    }

    /// Return the operand list.
    pub fn operands(&self) -> &[FirRef] {
        &self.operands
    }
}

#[derive(Debug, Clone)]
pub struct SearchFir {
    pub(crate) pattern: String,
    pub(crate) direction: SearchDirection,
    pub(crate) anchored: bool,
    pub(crate) anchor: Option<FirRef>,
    pub(crate) result: Option<FirRef>,
    pub(crate) parent: Option<FirRef>,
    pub(crate) state: Nyes,
    pub(crate) alarm: Option<Alarm>,
    /// True for a value search (`~=` / `?=`): it matches on a statement's VALUE,
    /// not its name. Renders as `=(anchor=…, value=…, …)` (FOOP-23 appendix).
    pub(crate) is_value: bool,
    /// The value-search's value expression (`c-d+v` in `a~=c-d+v`). Only set
    /// when `is_value`; carries its own nested state for rendering.
    pub(crate) value: Option<FirRef>,
}

#[derive(Debug, Clone)]
pub struct IndexFir {
    pub(crate) offset: i32,
    pub(crate) anchored: bool,
    pub(crate) anchor: Option<FirRef>,
    /// The FIR this index PRODUCED (the element found at the offset). The anchor is
    /// what we index INTO; the result is what indexing yields. Rendered as `result=`.
    pub(crate) result: Option<FirRef>,
    pub(crate) state: Nyes,
}

#[derive(Debug, Clone)]
pub struct StayFoolishFir {
    pub(crate) expr: FirRef,
    pub(crate) state: Nyes,
}

#[derive(Debug, Clone)]
pub struct StayFullyFoolishFir {
    pub(crate) expr: FirRef,
    pub(crate) state: Nyes,
}

#[derive(Debug, Clone)]
pub struct ConcatenationFir {
    pub(crate) elements: Vec<FirRef>,
    pub(crate) merged: Option<FirRef>,
    pub(crate) state: Nyes,
}

#[derive(Debug, Clone)]
pub struct NormalBraneFir {
    pub(crate) characterizations: Vec<String>,
    pub(crate) statements: Vec<StatementFir>,
    pub(crate) state: Nyes,
    pub(crate) alarm: Option<Alarm>,
}

impl NormalBraneFir {
    pub fn statements(&self) -> &Vec<StatementFir> {
        &self.statements
    }
}

// ==================== Steppable Trait ====================

/// Trait for all FIRs — the OOP interface.
/// step_one() returns None if self was mutated in-place,
/// or Some(replacement_fir) if self should be replaced entirely.
/// The shared FIR query/accessor surface.
///
/// This is the object-safe base trait behind `FirRef = Rc<RefCell<dyn Steppable>>`.
/// It carries the FIR's NYES state, its cloned-enum form, and the narrow leaf-data
/// accessors the humanizing sequencer reads through `FirQueryable`. It deliberately
/// carries NO evaluation/stepping surface: the UBC evaluator that once lived here
/// (`step_one`/`step_members` over a `ubc::Scope`) has been retired. Evaluation now
/// lives entirely in `foolish-ubca`, which builds these nodes and hands them to the
/// shared sequencer. Multiple engines can share this IR + sequencer through the
/// `snapshot_suite::Evaluator` seam.
pub trait Steppable: std::fmt::Debug {
    fn state(&self) -> Nyes;
    fn set_state(&mut self, state: Nyes);

    /// Return owned clones of all child FirRefs (for formatting / query purposes).
    fn fir_children(&self) -> Vec<FirRef> {
        match self.clone_into_fir() {
            Fir::Operator(v) => v.operands,
            Fir::Search(v) => {
                let mut c = vec![];
                if let Some(a) = v.anchor {
                    c.push(a);
                }
                if let Some(t) = v.result {
                    c.push(t);
                }
                c
            }
            Fir::Index(v) => v.anchor.into_iter().collect(),
            Fir::StayFoolish(v) => vec![v.expr],
            Fir::StayFullyFoolish(v) => vec![v.expr],
            Fir::Concatenation(v) => {
                let mut c = v.elements;
                if let Some(m) = v.merged {
                    c.push(m);
                }
                c
            }
            Fir::NormalBrane(v) => v.statements.iter().map(|s| Rc::clone(&s.body)).collect(),
            Fir::ConstantInt(_) | Fir::Nk(_) => vec![],
        }
    }

    /// Clone this Steppable into a Fir enum (bypasses dyn Clone limitation).
    fn clone_into_fir(&self) -> Fir;

    /// Return variant identifier (for type-based dispatch on trait objects).
    fn fir_variant(&self) -> &'static str {
        ""
    }

    // --- Accessors (defaults return None/empty — overridden by Fir impl) ---
    fn as_int(&self) -> Option<i64> {
        None
    }
    fn binary_values(&self) -> Option<(i64, i64)> {
        None
    }
    fn search_result_ref(&self) -> Option<FirRef> {
        None
    }
    fn search_parent_ref(&self) -> Option<FirRef> {
        None
    }
    fn search_pattern(&self) -> Option<String> {
        None
    }
    fn search_anchored(&self) -> bool {
        false
    }
    fn search_anchor_ref(&self) -> Option<FirRef> {
        None
    }
    fn index_offset(&self) -> i32 {
        0
    }
    fn index_anchored(&self) -> bool {
        false
    }
    fn index_anchor_ref(&self) -> Option<FirRef> {
        None
    }
    fn left_state(&self) -> Nyes {
        Nyes::Nk
    }
    fn right_state(&self) -> Nyes {
        Nyes::Nk
    }
    fn expr_state(&self) -> Nyes {
        Nyes::Nk
    }
    fn unary_value(&self) -> Option<i64> {
        None
    }
    fn concat_merged_ref(&self) -> Option<FirRef> {
        None
    }
    fn concat_elements(&self) -> Vec<FirRef> {
        vec![]
    }
    fn normal_brane_statements(&self) -> Vec<StatementFir> {
        vec![]
    }
    fn as_brane_statements(&self) -> Option<Vec<StatementFir>> {
        None
    }
    fn brane_statement_at(&self, _idx: usize) -> Option<FirRef> {
        None
    }
    fn brane_statement_count(&self) -> usize {
        0
    }

    // Mutation methods (defaults: no-op)
    fn set_binary_operands(&mut self, _left: Fir, _right: Fir) {}
    fn set_unary_expr(&mut self, _expr: Fir) {}
    fn set_search_result(&mut self, _result: FirRef) {}
    fn set_search_result_direct(&mut self, _result: Option<FirRef>) {}
    fn set_search_parent(&mut self, _parent: FirRef) {}
    fn set_concat_merged(&mut self, _merged: FirRef) {}
    fn set_concat_merged_direct(&mut self, _merged: Option<FirRef>) {}
}

// ==================== Fir Enum ====================

/// Clone a FirRef into a Fir enum value.
pub fn clone_steppable(fir: &FirRef) -> Fir {
    fir.borrow().clone_into_fir()
}

/// Wrap a Fir into a FirRef (Rc<RefCell<dyn Steppable>>).
pub fn fir_to_ref(fir: Fir) -> FirRef {
    Rc::new(RefCell::new(fir))
}

#[derive(Debug, Clone)]
pub enum Fir {
    ConstantInt(Box<ConstantIntFir>),
    Nk(Box<NkFir>),
    Operator(Box<OperatorFir>),
    Search(Box<SearchFir>),
    Index(Box<IndexFir>),
    StayFoolish(Box<StayFoolishFir>),
    StayFullyFoolish(Box<StayFullyFoolishFir>),
    Concatenation(Box<ConcatenationFir>),
    NormalBrane(Box<NormalBraneFir>),
}

impl Fir {
    pub fn into_ref(self) -> FirRef {
        Rc::new(RefCell::new(self))
    }
}

// ==================== FirQueryable Trait ====================

/// An optional boxed child in the query surface (an anchor or a result).
pub type QueryChild = Option<Box<dyn FirQueryable>>;
/// `hs_search` payload: (pattern, direction, anchored, anchor, result,
/// is_value, value). `is_value` marks a value search (`~=` / `?=`); `value` is
/// its value expression (FOOP-23 rendering appendix).
pub type SearchQuery = (
    String,
    SearchDirection,
    bool,
    QueryChild,
    QueryChild,
    bool,
    QueryChild,
);
/// `hs_index` payload: (offset, anchored, anchor, result).
pub type IndexQuery = (i32, bool, QueryChild, QueryChild);
/// `hs_concatenation` payload: (input elements, merged result brane).
pub type ConcatenationQuery = (Vec<Box<dyn FirQueryable>>, QueryChild);

/// Trait for querying FIR properties without mutation.
/// Used by HumanizingSequencer for format dispatch.
pub trait FirQueryable: std::fmt::Debug {
    /// Return variant identifier string.
    fn hs_variant(&self) -> &'static str;
    /// Return the NYES state.
    fn hs_state(&self) -> Nyes;

    // Accessors — each returns Option, Some only for matching variant:
    fn hs_constant_int(&self) -> Option<i64>;
    fn hs_nk(&self) -> Option<(String, Option<Alarm>)>;
    fn hs_operator(&self) -> Option<(String, Vec<Box<dyn FirQueryable>>)>;
    fn hs_search(&self) -> Option<SearchQuery>;
    fn hs_index(&self) -> Option<IndexQuery>;
    fn hs_stay_foolish(&self) -> Option<Box<dyn FirQueryable>>;
    fn hs_stay_fully_foolish(&self) -> Option<Box<dyn FirQueryable>>;
    fn hs_concatenation(&self) -> Option<ConcatenationQuery>;
    fn hs_brane(&self) -> Option<(Vec<String>, Vec<StatementSimple>)>;
    fn hs_alarm(&self) -> Option<&Alarm> {
        None
    }
}

/// Wrapper for FirRef (Rc<RefCell<dyn Steppable>>) that implements FirQueryable.
pub struct FirChildRef {
    inner: FirRef,
}

impl FirChildRef {
    pub fn new(inner: FirRef) -> Self {
        Self { inner }
    }
}

impl std::fmt::Debug for FirChildRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fir = clone_steppable(&self.inner);
        fir.fmt(f)
    }
}

impl FirQueryable for FirChildRef {
    fn hs_variant(&self) -> &'static str {
        self.inner.borrow().fir_variant()
    }
    fn hs_state(&self) -> Nyes {
        self.inner.borrow().state()
    }
    fn hs_constant_int(&self) -> Option<i64> {
        self.inner.borrow().as_int()
    }
    fn hs_nk(&self) -> Option<(String, Option<Alarm>)> {
        let fir = clone_steppable(&self.inner);
        if let Fir::Nk(i) = fir {
            Some((i.reason, i.alarm))
        } else {
            None
        }
    }
    fn hs_operator(&self) -> Option<(String, Vec<Box<dyn FirQueryable>>)> {
        let fir = clone_steppable(&self.inner);
        fir.hs_operator()
    }
    fn hs_search(&self) -> Option<SearchQuery> {
        let fir = clone_steppable(&self.inner);
        fir.hs_search()
    }
    fn hs_index(
        &self,
    ) -> Option<(
        i32,
        bool,
        Option<Box<dyn FirQueryable>>,
        Option<Box<dyn FirQueryable>>,
    )> {
        let fir = clone_steppable(&self.inner);
        fir.hs_index()
    }
    fn hs_stay_foolish(&self) -> Option<Box<dyn FirQueryable>> {
        let fir = clone_steppable(&self.inner);
        fir.hs_stay_foolish()
    }
    fn hs_stay_fully_foolish(&self) -> Option<Box<dyn FirQueryable>> {
        let fir = clone_steppable(&self.inner);
        fir.hs_stay_fully_foolish()
    }
    fn hs_concatenation(
        &self,
    ) -> Option<(Vec<Box<dyn FirQueryable>>, Option<Box<dyn FirQueryable>>)> {
        let fir = clone_steppable(&self.inner);
        fir.hs_concatenation()
    }
    fn hs_brane(&self) -> Option<(Vec<String>, Vec<StatementSimple>)> {
        let fir = clone_steppable(&self.inner);
        fir.hs_brane()
    }
}

/// Fir implements FirQueryable by matching variants and wrapping children in FirChildRef.
impl FirQueryable for Fir {
    fn hs_variant(&self) -> &'static str {
        match self {
            Fir::ConstantInt(_) => "ConstantInt",
            Fir::Nk(_) => "Nk",
            Fir::Operator(_) => "Operator",
            Fir::Search(_) => "Search",
            Fir::Index(_) => "Index",
            Fir::StayFoolish(_) => "StayFoolish",
            Fir::StayFullyFoolish(_) => "StayFullyFoolish",
            Fir::Concatenation(_) => "Concatenation",
            Fir::NormalBrane(_) => "NormalBrane",
        }
    }
    fn hs_state(&self) -> Nyes {
        match self {
            Fir::ConstantInt(i) => i.state,
            Fir::Nk(i) => i.state,
            Fir::Operator(i) => i.state,
            Fir::Search(i) => i.state,
            Fir::Index(i) => i.state,
            Fir::StayFoolish(i) => i.state,
            Fir::StayFullyFoolish(i) => i.state,
            Fir::Concatenation(i) => i.state,
            Fir::NormalBrane(i) => i.state,
        }
    }
    fn hs_constant_int(&self) -> Option<i64> {
        if let Fir::ConstantInt(i) = self {
            Some(i.value)
        } else {
            None
        }
    }
    fn hs_nk(&self) -> Option<(String, Option<Alarm>)> {
        if let Fir::Nk(i) = self {
            Some((i.reason.clone(), i.alarm.clone()))
        } else {
            None
        }
    }
    fn hs_operator(&self) -> Option<(String, Vec<Box<dyn FirQueryable>>)> {
        if let Fir::Operator(i) = self {
            Some((
                i.op.clone(),
                i.operands
                    .iter()
                    .map(|o| Box::new(FirChildRef::new(Rc::clone(o))) as Box<dyn FirQueryable>)
                    .collect(),
            ))
        } else {
            None
        }
    }
    fn hs_search(&self) -> Option<SearchQuery> {
        if let Fir::Search(i) = self {
            Some((
                i.pattern.clone(),
                i.direction,
                i.anchored,
                i.anchor
                    .as_ref()
                    .map(|a| Box::new(FirChildRef::new(Rc::clone(a))) as Box<dyn FirQueryable>),
                i.result
                    .as_ref()
                    .map(|t| Box::new(FirChildRef::new(Rc::clone(t))) as Box<dyn FirQueryable>),
                i.is_value,
                i.value
                    .as_ref()
                    .map(|v| Box::new(FirChildRef::new(Rc::clone(v))) as Box<dyn FirQueryable>),
            ))
        } else {
            None
        }
    }
    fn hs_index(
        &self,
    ) -> Option<(
        i32,
        bool,
        Option<Box<dyn FirQueryable>>,
        Option<Box<dyn FirQueryable>>,
    )> {
        if let Fir::Index(i) = self {
            Some((
                i.offset,
                i.anchored,
                i.anchor
                    .as_ref()
                    .map(|a| Box::new(FirChildRef::new(Rc::clone(a))) as Box<dyn FirQueryable>),
                i.result
                    .as_ref()
                    .map(|t| Box::new(FirChildRef::new(Rc::clone(t))) as Box<dyn FirQueryable>),
            ))
        } else {
            None
        }
    }
    fn hs_stay_foolish(&self) -> Option<Box<dyn FirQueryable>> {
        if let Fir::StayFoolish(i) = self {
            Some(Box::new(FirChildRef::new(Rc::clone(&i.expr))) as Box<dyn FirQueryable>)
        } else {
            None
        }
    }
    fn hs_stay_fully_foolish(&self) -> Option<Box<dyn FirQueryable>> {
        if let Fir::StayFullyFoolish(i) = self {
            Some(Box::new(FirChildRef::new(Rc::clone(&i.expr))) as Box<dyn FirQueryable>)
        } else {
            None
        }
    }
    fn hs_concatenation(
        &self,
    ) -> Option<(Vec<Box<dyn FirQueryable>>, Option<Box<dyn FirQueryable>>)> {
        if let Fir::Concatenation(i) = self {
            Some((
                i.elements
                    .iter()
                    .map(|e| Box::new(FirChildRef::new(Rc::clone(e))) as Box<dyn FirQueryable>)
                    .collect(),
                i.merged
                    .as_ref()
                    .map(|m| Box::new(FirChildRef::new(Rc::clone(m))) as Box<dyn FirQueryable>),
            ))
        } else {
            None
        }
    }
    fn hs_brane(&self) -> Option<(Vec<String>, Vec<StatementSimple>)> {
        if let Fir::NormalBrane(i) = self {
            Some((
                i.characterizations.clone(),
                i.statements
                    .iter()
                    .map(|s| StatementSimple {
                        name: s.name.clone(),
                        body: Box::new(FirChildRef::new(Rc::clone(&s.body)))
                            as Box<dyn FirQueryable>,
                    })
                    .collect(),
            ))
        } else {
            None
        }
    }
    fn hs_alarm(&self) -> Option<&Alarm> {
        match self {
            Fir::NormalBrane(i) => i.alarm.as_ref(),
            Fir::Search(i) => i.alarm.as_ref(),
            _ => None,
        }
    }
}

// ==================== Fir: Steppable (dispatches to inner struct) ====================

impl Steppable for Fir {
    fn state(&self) -> Nyes {
        match self {
            Fir::ConstantInt(i) => i.state(),
            Fir::Nk(i) => i.state(),
            Fir::Operator(i) => i.state(),
            Fir::Search(i) => i.state(),
            Fir::Index(i) => i.state(),
            Fir::StayFoolish(i) => i.state(),
            Fir::StayFullyFoolish(i) => i.state(),
            Fir::Concatenation(i) => i.state(),
            Fir::NormalBrane(i) => i.state(),
        }
    }

    fn set_state(&mut self, s: Nyes) {
        match self {
            Fir::ConstantInt(i) => i.set_state(s),
            Fir::Nk(i) => i.set_state(s),
            Fir::Operator(i) => i.set_state(s),
            Fir::Search(i) => i.set_state(s),
            Fir::Index(i) => i.set_state(s),
            Fir::StayFoolish(i) => i.set_state(s),
            Fir::StayFullyFoolish(i) => i.set_state(s),
            Fir::Concatenation(i) => i.set_state(s),
            Fir::NormalBrane(i) => i.set_state(s),
        }
    }

    fn clone_into_fir(&self) -> Fir {
        self.clone()
    }

    fn fir_variant(&self) -> &'static str {
        match self {
            Fir::ConstantInt(_) => "ConstantInt",
            Fir::Nk(_) => "Nk",
            Fir::Operator(_) => "Operator",
            Fir::Search(_) => "Search",
            Fir::Index(_) => "Index",
            Fir::StayFoolish(_) => "StayFoolish",
            Fir::StayFullyFoolish(_) => "StayFullyFoolish",
            Fir::Concatenation(_) => "Concatenation",
            Fir::NormalBrane(_) => "NormalBrane",
        }
    }

    // Accessor overrides
    fn as_int(&self) -> Option<i64> {
        if let Fir::ConstantInt(inner) = self {
            Some(inner.value)
        } else {
            None
        }
    }

    fn binary_values(&self) -> Option<(i64, i64)> {
        if let Fir::Operator(inner) = self {
            if inner.operands.len() != 2 {
                return None;
            }
            let l = inner.operands[0].borrow().as_int()?;
            let r = inner.operands[1].borrow().as_int()?;
            Some((l, r))
        } else {
            None
        }
    }

    fn search_result_ref(&self) -> Option<FirRef> {
        if let Fir::Search(inner) = self {
            inner.result.clone()
        } else {
            None
        }
    }

    fn search_parent_ref(&self) -> Option<FirRef> {
        if let Fir::Search(inner) = self {
            inner.parent.clone()
        } else {
            None
        }
    }

    fn search_pattern(&self) -> Option<String> {
        if let Fir::Search(inner) = self {
            Some(inner.pattern.clone())
        } else {
            None
        }
    }

    fn search_anchored(&self) -> bool {
        if let Fir::Search(inner) = self {
            inner.anchored
        } else {
            false
        }
    }

    fn search_anchor_ref(&self) -> Option<FirRef> {
        if let Fir::Search(inner) = self {
            inner.anchor.clone()
        } else {
            None
        }
    }

    fn index_offset(&self) -> i32 {
        if let Fir::Index(inner) = self {
            inner.offset
        } else {
            0
        }
    }

    fn index_anchored(&self) -> bool {
        if let Fir::Index(inner) = self {
            inner.anchored
        } else {
            false
        }
    }

    fn index_anchor_ref(&self) -> Option<FirRef> {
        if let Fir::Index(inner) = self {
            inner.anchor.clone()
        } else {
            None
        }
    }

    fn left_state(&self) -> Nyes {
        if let Fir::Operator(inner) = self {
            if inner.operands.is_empty() {
                Nyes::Nk
            } else {
                inner.operands[0].borrow().state()
            }
        } else {
            Nyes::Nk
        }
    }

    fn right_state(&self) -> Nyes {
        if let Fir::Operator(inner) = self {
            if inner.operands.len() < 2 {
                Nyes::Nk
            } else {
                inner.operands[1].borrow().state()
            }
        } else {
            Nyes::Nk
        }
    }

    fn expr_state(&self) -> Nyes {
        if let Fir::Operator(inner) = self {
            if inner.operands.is_empty() {
                Nyes::Nk
            } else {
                inner.operands[0].borrow().state()
            }
        } else {
            Nyes::Nk
        }
    }

    fn unary_value(&self) -> Option<i64> {
        if let Fir::Operator(inner) = self {
            if inner.operands.is_empty() {
                None
            } else {
                inner.operands[0].borrow().as_int()
            }
        } else {
            None
        }
    }

    fn concat_merged_ref(&self) -> Option<FirRef> {
        if let Fir::Concatenation(inner) = self {
            inner.merged.clone()
        } else {
            None
        }
    }

    fn concat_elements(&self) -> Vec<FirRef> {
        if let Fir::Concatenation(inner) = self {
            inner.elements.iter().map(Rc::clone).collect()
        } else {
            vec![]
        }
    }

    fn normal_brane_statements(&self) -> Vec<StatementFir> {
        if let Fir::NormalBrane(inner) = self {
            inner.statements.clone()
        } else {
            vec![]
        }
    }

    fn as_brane_statements(&self) -> Option<Vec<StatementFir>> {
        if let Fir::NormalBrane(inner) = self {
            Some(inner.statements.clone())
        } else {
            None
        }
    }

    fn brane_statement_at(&self, idx: usize) -> Option<FirRef> {
        if let Fir::NormalBrane(inner) = self {
            inner.statements.get(idx).map(|s| Rc::clone(&s.body))
        } else {
            None
        }
    }

    fn brane_statement_count(&self) -> usize {
        if let Fir::NormalBrane(inner) = self {
            inner.statements.len()
        } else {
            0
        }
    }

    fn set_binary_operands(&mut self, left: Fir, right: Fir) {
        if let Fir::Operator(inner) = self {
            inner.operands = vec![fir_to_ref(left), fir_to_ref(right)];
        }
    }

    fn set_unary_expr(&mut self, expr: Fir) {
        if let Fir::Operator(inner) = self {
            inner.operands = vec![fir_to_ref(expr)];
        }
    }

    fn set_search_result(&mut self, result: FirRef) {
        if let Fir::Search(inner) = self {
            inner.result = Some(result);
        }
    }

    fn set_search_result_direct(&mut self, result: Option<FirRef>) {
        if let Fir::Search(inner) = self {
            inner.result = result;
        }
    }

    fn set_search_parent(&mut self, parent: FirRef) {
        if let Fir::Search(inner) = self {
            inner.parent = Some(parent);
        }
    }

    fn set_concat_merged(&mut self, merged: FirRef) {
        if let Fir::Concatenation(inner) = self {
            inner.merged = Some(Rc::clone(&merged));
        }
    }

    fn set_concat_merged_direct(&mut self, merged: Option<FirRef>) {
        if let Fir::Concatenation(inner) = self {
            inner.merged = merged;
        }
    }
}

// Mutation methods on Fir (not on trait — enum knows its own shape)
impl Fir {
    pub fn set_stay_expr(&mut self, expr: Fir) {
        if let Fir::StayFoolish(inner) = self {
            inner.expr = fir_to_ref(expr);
        }
    }

    pub fn normal_brane_set_statements(&mut self, statements: Vec<StatementFir>) {
        if let Fir::NormalBrane(inner) = self {
            inner.statements = statements;
        }
    }
}

// ==================== Struct Steppable Implementations ====================

impl Steppable for ConstantIntFir {
    fn state(&self) -> Nyes {
        self.state
    }
    fn set_state(&mut self, s: Nyes) {
        self.state = s;
    }
    fn clone_into_fir(&self) -> Fir {
        Fir::ConstantInt(Box::new(self.clone()))
    }
    fn fir_variant(&self) -> &'static str {
        "ConstantInt"
    }
}

impl Steppable for NkFir {
    fn state(&self) -> Nyes {
        self.state
    }
    fn set_state(&mut self, s: Nyes) {
        self.state = s;
    }
    fn clone_into_fir(&self) -> Fir {
        Fir::Nk(Box::new(self.clone()))
    }
    fn fir_variant(&self) -> &'static str {
        "Nk"
    }
}

impl Steppable for OperatorFir {
    fn state(&self) -> Nyes {
        self.state
    }
    fn set_state(&mut self, s: Nyes) {
        self.state = s;
    }
    fn clone_into_fir(&self) -> Fir {
        Fir::Operator(Box::new(self.clone()))
    }
    fn fir_variant(&self) -> &'static str {
        "Operator"
    }
}

impl Steppable for SearchFir {
    fn state(&self) -> Nyes {
        self.state
    }
    fn set_state(&mut self, s: Nyes) {
        self.state = s;
    }
    fn search_pattern(&self) -> Option<String> {
        Some(self.pattern.clone())
    }
    fn search_anchored(&self) -> bool {
        self.anchored
    }
    fn search_result_ref(&self) -> Option<FirRef> {
        self.result.clone()
    }
    fn search_parent_ref(&self) -> Option<FirRef> {
        self.parent.clone()
    }
    fn search_anchor_ref(&self) -> Option<FirRef> {
        self.anchor.clone()
    }
    fn clone_into_fir(&self) -> Fir {
        Fir::Search(Box::new(self.clone()))
    }
    fn fir_variant(&self) -> &'static str {
        "Search"
    }
}

impl Steppable for IndexFir {
    fn state(&self) -> Nyes {
        self.state
    }
    fn set_state(&mut self, s: Nyes) {
        self.state = s;
    }
    fn index_offset(&self) -> i32 {
        self.offset
    }
    fn index_anchored(&self) -> bool {
        self.anchored
    }
    fn index_anchor_ref(&self) -> Option<FirRef> {
        self.anchor.clone()
    }
    fn clone_into_fir(&self) -> Fir {
        Fir::Index(Box::new(self.clone()))
    }
    fn fir_variant(&self) -> &'static str {
        "Index"
    }
}

impl IndexFir {}

impl Steppable for StayFoolishFir {
    fn state(&self) -> Nyes {
        self.state
    }
    fn set_state(&mut self, s: Nyes) {
        self.state = s;
    }
    fn clone_into_fir(&self) -> Fir {
        Fir::StayFoolish(Box::new(self.clone()))
    }
    fn fir_variant(&self) -> &'static str {
        "StayFoolish"
    }
}

impl Steppable for StayFullyFoolishFir {
    fn state(&self) -> Nyes {
        self.state
    }
    fn set_state(&mut self, s: Nyes) {
        self.state = s;
    }
    fn clone_into_fir(&self) -> Fir {
        Fir::StayFullyFoolish(Box::new(self.clone()))
    }
    fn fir_variant(&self) -> &'static str {
        "StayFullyFoolish"
    }
}

impl Steppable for ConcatenationFir {
    fn state(&self) -> Nyes {
        self.state
    }
    fn set_state(&mut self, s: Nyes) {
        self.state = s;
    }
    fn concat_merged_ref(&self) -> Option<FirRef> {
        self.merged.clone()
    }
    fn clone_into_fir(&self) -> Fir {
        Fir::Concatenation(Box::new(self.clone()))
    }
    fn fir_variant(&self) -> &'static str {
        "Concatenation"
    }
    fn concat_elements(&self) -> Vec<FirRef> {
        self.elements.iter().map(Rc::clone).collect()
    }
}

impl Steppable for NormalBraneFir {
    fn state(&self) -> Nyes {
        self.state
    }
    fn set_state(&mut self, s: Nyes) {
        self.state = s;
    }
    fn normal_brane_statements(&self) -> Vec<StatementFir> {
        self.statements.clone()
    }
    fn as_brane_statements(&self) -> Option<Vec<StatementFir>> {
        Some(self.statements.clone())
    }
    fn brane_statement_count(&self) -> usize {
        self.statements.len()
    }
    fn clone_into_fir(&self) -> Fir {
        Fir::NormalBrane(Box::new(self.clone()))
    }
    fn fir_variant(&self) -> &'static str {
        "NormalBrane"
    }
}

// ==================== Helper: extract struct from enum ====================

impl Fir {
    pub fn as_const_int(&self) -> Option<&ConstantIntFir> {
        if let Fir::ConstantInt(inner) = self {
            Some(inner)
        } else {
            None
        }
    }
    pub fn as_nk(&self) -> Option<&NkFir> {
        if let Fir::Nk(inner) = self {
            Some(inner)
        } else {
            None
        }
    }
    pub fn as_operator(&self) -> Option<&OperatorFir> {
        if let Fir::Operator(inner) = self {
            Some(inner)
        } else {
            None
        }
    }
    pub fn as_search(&self) -> Option<&SearchFir> {
        if let Fir::Search(inner) = self {
            Some(inner)
        } else {
            None
        }
    }
    pub fn as_index(&self) -> Option<&IndexFir> {
        if let Fir::Index(inner) = self {
            Some(inner)
        } else {
            None
        }
    }
    pub fn as_stay_foolish(&self) -> Option<&StayFoolishFir> {
        if let Fir::StayFoolish(inner) = self {
            Some(inner)
        } else {
            None
        }
    }
    pub fn as_stay_fully_foolish(&self) -> Option<&StayFullyFoolishFir> {
        if let Fir::StayFullyFoolish(inner) = self {
            Some(inner)
        } else {
            None
        }
    }
    pub fn as_concatenation(&self) -> Option<&ConcatenationFir> {
        if let Fir::Concatenation(inner) = self {
            Some(inner)
        } else {
            None
        }
    }
    pub fn as_normal_brane(&self) -> Option<&NormalBraneFir> {
        if let Fir::NormalBrane(inner) = self {
            Some(inner)
        } else {
            None
        }
    }
}

// ==================== Manual Serde for Fir ====================

fn to_json_val<T: Serialize>(v: &T) -> serde_json::Value {
    serde_json::to_value(v).expect("serde_json::to_value should succeed on serializable types")
}

fn fir_to_json(fir: &Fir) -> serde_json::Value {
    use serde_json::{Map, Value};
    match fir {
        Fir::ConstantInt(inner) => {
            let mut m = Map::new();
            m.insert("type".into(), Value::String("ConstantInt".into()));
            m.insert("value".into(), Value::Number(inner.value.into()));
            m.insert("state".into(), to_json_val(&inner.state));
            Value::Object(m)
        }
        Fir::Nk(inner) => {
            let mut m = Map::new();
            m.insert("type".into(), Value::String("Nk".into()));
            m.insert("reason".into(), Value::String(inner.reason.clone()));
            m.insert("state".into(), to_json_val(&inner.state));
            if let Some(ref alarm) = inner.alarm {
                m.insert("alarm".into(), to_json_val(alarm));
            }
            Value::Object(m)
        }
        Fir::Operator(inner) => {
            let mut m = Map::new();
            m.insert("type".into(), Value::String("Operator".into()));
            m.insert("op".into(), Value::String(inner.op.clone()));
            m.insert(
                "operands".into(),
                Value::Array(
                    inner
                        .operands
                        .iter()
                        .map(|o| fir_to_json(&clone_steppable(o)))
                        .collect(),
                ),
            );
            m.insert("state".into(), to_json_val(&inner.state));
            Value::Object(m)
        }
        Fir::Search(inner) => {
            let mut m = Map::new();
            m.insert("type".into(), Value::String("Search".into()));
            m.insert("pattern".into(), Value::String(inner.pattern.clone()));
            m.insert("direction".into(), to_json_val(&inner.direction));
            m.insert("anchored".into(), Value::Bool(inner.anchored));
            m.insert("state".into(), to_json_val(&inner.state));
            if let Some(ref a) = inner.anchor {
                m.insert("anchor".into(), fir_to_json(&clone_steppable(a)));
            }
            if let Some(ref t) = inner.result {
                m.insert("result".into(), fir_to_json(&clone_steppable(t)));
            }
            Value::Object(m)
        }
        Fir::Index(inner) => {
            let mut m = Map::new();
            m.insert("type".into(), Value::String("Index".into()));
            m.insert("offset".into(), Value::Number(inner.offset.into()));
            m.insert("anchored".into(), Value::Bool(inner.anchored));
            m.insert("state".into(), to_json_val(&inner.state));
            if let Some(ref a) = inner.anchor {
                m.insert("anchor".into(), fir_to_json(&clone_steppable(a)));
            }
            if let Some(ref t) = inner.result {
                m.insert("result".into(), fir_to_json(&clone_steppable(t)));
            }
            Value::Object(m)
        }
        Fir::StayFoolish(inner) => {
            let mut m = Map::new();
            m.insert("type".into(), Value::String("StayFoolish".into()));
            m.insert("expr".into(), fir_to_json(&clone_steppable(&inner.expr)));
            m.insert("state".into(), to_json_val(&inner.state));
            Value::Object(m)
        }
        Fir::StayFullyFoolish(inner) => {
            let mut m = Map::new();
            m.insert("type".into(), Value::String("StayFullyFoolish".into()));
            m.insert("expr".into(), fir_to_json(&clone_steppable(&inner.expr)));
            m.insert("state".into(), to_json_val(&inner.state));
            Value::Object(m)
        }
        Fir::Concatenation(inner) => {
            let mut m = Map::new();
            m.insert("type".into(), Value::String("Concatenation".into()));
            m.insert(
                "elements".into(),
                Value::Array(
                    inner
                        .elements
                        .iter()
                        .map(|e| fir_to_json(&clone_steppable(e)))
                        .collect(),
                ),
            );
            m.insert("state".into(), to_json_val(&inner.state));
            if let Some(ref mg) = inner.merged {
                m.insert("merged".into(), fir_to_json(&clone_steppable(mg)));
            }
            Value::Object(m)
        }
        Fir::NormalBrane(inner) => {
            let mut m = Map::new();
            m.insert("type".into(), Value::String("NormalBrane".into()));
            m.insert(
                "characterizations".into(),
                Value::Array(
                    inner
                        .characterizations
                        .iter()
                        .map(|c| Value::String(c.clone()))
                        .collect(),
                ),
            );
            let stmts: Vec<Value> = inner
                .statements
                .iter()
                .map(|s| {
                    let mut sm = Map::new();
                    sm.insert("body".into(), fir_to_json(&clone_steppable(&s.body)));
                    sm.insert("state".into(), to_json_val(&s.state));
                    if let Some(ref n) = s.name {
                        sm.insert("name".into(), Value::String(n.clone()));
                    }
                    Value::Object(sm)
                })
                .collect();
            m.insert("statements".into(), Value::Array(stmts));
            m.insert("state".into(), to_json_val(&inner.state));
            Value::Object(m)
        }
    }
}

impl Serialize for Fir {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        fir_to_json(self).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Fir {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(deserializer)?;
        let obj = value
            .as_object()
            .ok_or_else(|| serde::de::Error::custom("expected JSON object"))?;
        let type_name = obj
            .get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| serde::de::Error::custom("missing type field"))?;
        let state = obj
            .get("state")
            .and_then(|v| serde_json::from_value::<Nyes>(v.clone()).ok())
            .ok_or_else(|| serde::de::Error::custom("missing or invalid state field"))?;
        match type_name {
            "ConstantInt" => {
                let value = obj
                    .get("value")
                    .and_then(|v| v.as_i64())
                    .ok_or_else(|| serde::de::Error::custom("missing value field"))?;
                Ok(Fir::ConstantInt(Box::new(ConstantIntFir { value, state })))
            }
            "Nk" => {
                let reason = obj
                    .get("reason")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| serde::de::Error::custom("missing reason field"))?
                    .to_string();
                let alarm = obj
                    .get("alarm")
                    .and_then(|v| serde_json::from_value::<Alarm>(v.clone()).ok());
                Ok(Fir::Nk(Box::new(NkFir {
                    reason,
                    state,
                    alarm,
                })))
            }
            "Operator" => {
                let op = obj
                    .get("op")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| serde::de::Error::custom("missing op field"))?
                    .to_string();
                let operands = obj
                    .get("operands")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| serde::de::Error::custom("missing operands field"))?
                    .iter()
                    .map(|v| {
                        serde_json::from_value::<Fir>(v.clone())
                            .map_err(serde::de::Error::custom)
                            .map(fir_to_ref)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Fir::Operator(Box::new(OperatorFir {
                    op,
                    operands,
                    state,
                })))
            }
            "Search" => {
                let pattern = obj
                    .get("pattern")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| serde::de::Error::custom("missing pattern field"))?
                    .to_string();
                let direction = obj
                    .get("direction")
                    .and_then(|v| serde_json::from_value::<SearchDirection>(v.clone()).ok())
                    .unwrap_or(SearchDirection::Backward);
                let anchored = obj
                    .get("anchored")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let anchor = obj
                    .get("anchor")
                    .and_then(|v| serde_json::from_value::<Fir>(v.clone()).ok())
                    .map(fir_to_ref);
                let result = obj
                    .get("result")
                    .and_then(|v| serde_json::from_value::<Fir>(v.clone()).ok())
                    .map(fir_to_ref);
                let is_value = obj
                    .get("is_value")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let value = obj
                    .get("value")
                    .and_then(|v| serde_json::from_value::<Fir>(v.clone()).ok())
                    .map(fir_to_ref);
                Ok(Fir::Search(Box::new(SearchFir {
                    pattern,
                    direction,
                    anchored,
                    anchor,
                    result,
                    parent: None,
                    state,
                    alarm: None,
                    is_value,
                    value,
                })))
            }
            "Index" => {
                let offset = obj
                    .get("offset")
                    .and_then(|v| v.as_i64())
                    .map(|n| n as i32)
                    .ok_or_else(|| serde::de::Error::custom("missing offset field"))?;
                let anchored = obj
                    .get("anchored")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let anchor = obj
                    .get("anchor")
                    .and_then(|v| serde_json::from_value::<Fir>(v.clone()).ok())
                    .map(fir_to_ref);
                let result = obj
                    .get("result")
                    .and_then(|v| serde_json::from_value::<Fir>(v.clone()).ok())
                    .map(fir_to_ref);
                Ok(Fir::Index(Box::new(IndexFir {
                    offset,
                    anchored,
                    anchor,
                    result,
                    state,
                })))
            }
            "HeadTail" => {
                // Legacy: deserialize HeadTail as Index (offset=0 for head, -1 for tail)
                let is_head = obj.get("is_head").and_then(|v| v.as_bool()).unwrap_or(true);
                let offset: i32 = if is_head { 0 } else { -1 };
                let anchored = obj
                    .get("anchored")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let anchor = obj
                    .get("anchor")
                    .and_then(|v| serde_json::from_value::<Fir>(v.clone()).ok())
                    .map(fir_to_ref);
                let result = obj
                    .get("result")
                    .and_then(|v| serde_json::from_value::<Fir>(v.clone()).ok())
                    .map(fir_to_ref);
                Ok(Fir::Index(Box::new(IndexFir {
                    offset,
                    anchored,
                    anchor,
                    result,
                    state,
                })))
            }
            "StayFoolish" => {
                let expr = obj
                    .get("expr")
                    .ok_or_else(|| serde::de::Error::custom("missing expr field"))
                    .and_then(|v| {
                        serde_json::from_value::<Fir>(v.clone()).map_err(serde::de::Error::custom)
                    })?;
                Ok(Fir::StayFoolish(Box::new(StayFoolishFir {
                    expr: fir_to_ref(expr),
                    state,
                })))
            }
            "StayFullyFoolish" => {
                let expr = obj
                    .get("expr")
                    .ok_or_else(|| serde::de::Error::custom("missing expr field"))
                    .and_then(|v| {
                        serde_json::from_value::<Fir>(v.clone()).map_err(serde::de::Error::custom)
                    })?;
                Ok(Fir::StayFullyFoolish(Box::new(StayFullyFoolishFir {
                    expr: fir_to_ref(expr),
                    state,
                })))
            }
            "Concatenation" => {
                let elements = obj
                    .get("elements")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| serde::de::Error::custom("missing elements field"))?
                    .iter()
                    .map(|v| {
                        serde_json::from_value::<Fir>(v.clone())
                            .map_err(serde::de::Error::custom)
                            .map(fir_to_ref)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let merged = obj
                    .get("merged")
                    .and_then(|v| serde_json::from_value::<Fir>(v.clone()).ok())
                    .map(fir_to_ref);
                Ok(Fir::Concatenation(Box::new(ConcatenationFir {
                    elements,
                    merged,
                    state,
                })))
            }
            "NormalBrane" => {
                let characterizations = obj
                    .get("characterizations")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str())
                            .map(|s| s.to_string())
                            .collect()
                    })
                    .unwrap_or_default();
                let statements = obj
                    .get("statements")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| serde::de::Error::custom("missing statements field"))?
                    .iter()
                    .map(|v| {
                        let stmt_obj = v
                            .as_object()
                            .ok_or_else(|| serde::de::Error::custom("expected statement object"))?;
                        let body = stmt_obj
                            .get("body")
                            .ok_or_else(|| serde::de::Error::custom("missing body field"))
                            .and_then(|bv| {
                                serde_json::from_value::<Fir>(bv.clone())
                                    .map_err(serde::de::Error::custom)
                            })?;
                        let s = stmt_obj
                            .get("state")
                            .and_then(|sv| serde_json::from_value::<Nyes>(sv.clone()).ok())
                            .ok_or_else(|| serde::de::Error::custom("missing statement state"))?;
                        let name = stmt_obj
                            .get("name")
                            .and_then(|nv| nv.as_str())
                            .map(|n| n.to_string());
                        Ok(StatementFir {
                            name,
                            body: fir_to_ref(body),
                            state: s,
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Fir::NormalBrane(Box::new(NormalBraneFir {
                    characterizations,
                    statements,
                    state,
                    alarm: None,
                })))
            }
            _ => Err(serde::de::Error::custom(format!(
                "unknown Fir type: {}",
                type_name
            ))),
        }
    }
}

// ==================== StatementSimple ====================

/// Simple statement (name + body) used as return value of hs_brane().
/// Replaces SequenceableStatement; body is a Box<dyn FirQueryable>.
#[derive(Debug)]
pub struct StatementSimple {
    pub name: Option<String>,
    pub body: Box<dyn FirQueryable>,
}

// ==================== FIR Builders ====================

/// Builder for ConstantIntFir.
pub struct ConstantIntFirBuilder {
    value: i64,
    state: Nyes,
}

impl ConstantIntFirBuilder {
    pub fn new(value: i64) -> Self {
        Self {
            value,
            state: Nyes::Constant,
        }
    }
    pub fn state(mut self, state: Nyes) -> Self {
        self.state = state;
        self
    }
    pub fn build(self) -> Fir {
        Fir::ConstantInt(Box::new(ConstantIntFir {
            value: self.value,
            state: self.state,
        }))
    }
}

/// Builder for NkFir.
pub struct NkFirBuilder {
    reason: String,
    state: Nyes,
    alarm: Option<Alarm>,
}

impl NkFirBuilder {
    pub fn new(reason: impl Into<String>) -> Self {
        Self {
            reason: reason.into(),
            state: Nyes::Nk,
            alarm: None,
        }
    }
    pub fn state(mut self, state: Nyes) -> Self {
        self.state = state;
        self
    }
    pub fn alarm(mut self, alarm: Alarm) -> Self {
        self.alarm = Some(alarm);
        self
    }
    pub fn build(self) -> Fir {
        Fir::Nk(Box::new(NkFir {
            reason: self.reason,
            state: self.state,
            alarm: self.alarm,
        }))
    }
}

/// Builder for OperatorFir.
pub struct OperatorFirBuilder {
    op: String,
    operands: Vec<FirRef>,
    state: Nyes,
}

impl OperatorFirBuilder {
    pub fn new(op: impl Into<String>) -> Self {
        Self {
            op: op.into(),
            operands: Vec::new(),
            state: Nyes::Embryonic,
        }
    }
    pub fn operand(mut self, child: Fir) -> Self {
        self.operands.push(fir_to_ref(child));
        self
    }
    pub fn operands(mut self, children: Vec<Fir>) -> Self {
        self.operands = children.into_iter().map(fir_to_ref).collect();
        self
    }
    pub fn state(mut self, state: Nyes) -> Self {
        self.state = state;
        self
    }
    pub fn build(self) -> Fir {
        Fir::Operator(Box::new(OperatorFir {
            op: self.op,
            operands: self.operands,
            state: self.state,
        }))
    }
}

/// Builder for SearchFir.
pub struct SearchFirBuilder {
    pattern: String,
    direction: SearchDirection,
    anchored: bool,
    anchor: Option<FirRef>,
    result: Option<FirRef>,
    parent: Option<FirRef>,
    state: Nyes,
    alarm: Option<Alarm>,
    is_value: bool,
    value: Option<FirRef>,
}

impl SearchFirBuilder {
    pub fn new(pattern: impl Into<String>) -> Self {
        Self {
            pattern: pattern.into(),
            direction: SearchDirection::Backward,
            anchored: false,
            anchor: None,
            result: None,
            parent: None,
            state: Nyes::Embryonic,
            alarm: None,
            is_value: false,
            value: None,
        }
    }
    /// Mark this as a value search (`~=` / `?=`). Renders `=(anchor=…, value=…)`.
    pub fn is_value(mut self, is_value: bool) -> Self {
        self.is_value = is_value;
        self
    }
    /// The value-search's value expression (the RHS of `~=`).
    pub fn value(mut self, value: Fir) -> Self {
        self.value = Some(fir_to_ref(value));
        self
    }
    pub fn direction(mut self, direction: SearchDirection) -> Self {
        self.direction = direction;
        self
    }
    pub fn anchored(mut self, anchored: bool) -> Self {
        self.anchored = anchored;
        self
    }
    pub fn anchor(mut self, anchor: Fir) -> Self {
        self.anchor = Some(fir_to_ref(anchor));
        self
    }
    pub fn result(mut self, result: Fir) -> Self {
        self.result = Some(fir_to_ref(result));
        self
    }
    pub fn parent(mut self, parent: Fir) -> Self {
        self.parent = Some(fir_to_ref(parent));
        self
    }
    pub fn state(mut self, state: Nyes) -> Self {
        self.state = state;
        self
    }
    pub fn alarm(mut self, alarm: Alarm) -> Self {
        self.alarm = Some(alarm);
        self
    }
    pub fn build(self) -> Fir {
        Fir::Search(Box::new(SearchFir {
            pattern: self.pattern,
            direction: self.direction,
            anchored: self.anchored,
            anchor: self.anchor,
            result: self.result,
            parent: self.parent,
            state: self.state,
            alarm: self.alarm,
            is_value: self.is_value,
            value: self.value,
        }))
    }
}

/// Builder for IndexFir.
pub struct IndexFirBuilder {
    offset: i32,
    anchored: bool,
    anchor: Option<FirRef>,
    result: Option<FirRef>,
    state: Nyes,
}

impl IndexFirBuilder {
    pub fn new(offset: i32) -> Self {
        Self {
            offset,
            anchored: false,
            anchor: None,
            result: None,
            state: Nyes::Embryonic,
        }
    }
    pub fn anchored(mut self, anchored: bool) -> Self {
        self.anchored = anchored;
        self
    }
    pub fn anchor(mut self, anchor: Fir) -> Self {
        self.anchor = Some(fir_to_ref(anchor));
        self
    }
    pub fn result(mut self, result: Fir) -> Self {
        self.result = Some(fir_to_ref(result));
        self
    }
    pub fn state(mut self, state: Nyes) -> Self {
        self.state = state;
        self
    }
    pub fn build(self) -> Fir {
        Fir::Index(Box::new(IndexFir {
            offset: self.offset,
            anchored: self.anchored,
            anchor: self.anchor,
            result: self.result,
            state: self.state,
        }))
    }
}

/// Builder for StayFoolishFir.
pub struct StayFoolishFirBuilder {
    expr: FirRef,
    state: Nyes,
}

impl StayFoolishFirBuilder {
    pub fn new(expr: Fir) -> Self {
        Self {
            expr: fir_to_ref(expr),
            state: Nyes::Embryonic,
        }
    }
    pub fn state(mut self, state: Nyes) -> Self {
        self.state = state;
        self
    }
    pub fn build(self) -> Fir {
        Fir::StayFoolish(Box::new(StayFoolishFir {
            expr: self.expr,
            state: self.state,
        }))
    }
}

/// Builder for StayFullyFoolishFir.
pub struct StayFullyFoolishFirBuilder {
    expr: FirRef,
    state: Nyes,
}

impl StayFullyFoolishFirBuilder {
    pub fn new(expr: Fir) -> Self {
        Self {
            expr: fir_to_ref(expr),
            state: Nyes::Embryonic,
        }
    }
    pub fn state(mut self, state: Nyes) -> Self {
        self.state = state;
        self
    }
    pub fn build(self) -> Fir {
        Fir::StayFullyFoolish(Box::new(StayFullyFoolishFir {
            expr: self.expr,
            state: self.state,
        }))
    }
}

/// Builder for ConcatenationFir.
pub struct ConcatenationFirBuilder {
    elements: Vec<FirRef>,
    merged: Option<FirRef>,
    state: Nyes,
}

impl Default for ConcatenationFirBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ConcatenationFirBuilder {
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
            merged: None,
            state: Nyes::Embryonic,
        }
    }
    pub fn element(mut self, child: Fir) -> Self {
        self.elements.push(fir_to_ref(child));
        self
    }
    pub fn elements(mut self, children: Vec<Fir>) -> Self {
        self.elements = children.into_iter().map(fir_to_ref).collect();
        self
    }
    pub fn merged(mut self, merged: Fir) -> Self {
        self.merged = Some(fir_to_ref(merged));
        self
    }
    pub fn state(mut self, state: Nyes) -> Self {
        self.state = state;
        self
    }
    pub fn build(self) -> Fir {
        Fir::Concatenation(Box::new(ConcatenationFir {
            elements: self.elements,
            merged: self.merged,
            state: self.state,
        }))
    }
}

/// Builder for NormalBraneFir.
pub struct NormalBraneFirBuilder {
    characterizations: Vec<String>,
    statements: Vec<StatementFir>,
    state: Nyes,
    alarm: Option<Alarm>,
}

impl Default for NormalBraneFirBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl NormalBraneFirBuilder {
    pub fn new() -> Self {
        Self {
            characterizations: Vec::new(),
            statements: Vec::new(),
            state: Nyes::Embryonic,
            alarm: None,
        }
    }
    pub fn characterization(mut self, c: impl Into<String>) -> Self {
        self.characterizations.push(c.into());
        self
    }
    pub fn characterizations(mut self, chars: Vec<String>) -> Self {
        self.characterizations = chars;
        self
    }
    pub fn statement(mut self, name: Option<String>, body: Fir) -> Self {
        self.statements
            .push(StatementFir::new(name, fir_to_ref(body)));
        self
    }
    pub fn anonymous_statement(mut self, body: Fir) -> Self {
        self.statements
            .push(StatementFir::anonymous(fir_to_ref(body)));
        self
    }
    pub fn statements(mut self, stmts: Vec<(Option<String>, Fir)>) -> Self {
        self.statements = stmts
            .into_iter()
            .map(|(name, body)| StatementFir::new(name, fir_to_ref(body)))
            .collect();
        self
    }
    pub fn state(mut self, state: Nyes) -> Self {
        self.state = state;
        self
    }
    pub fn alarm(mut self, alarm: Alarm) -> Self {
        self.alarm = Some(alarm);
        self
    }
    pub fn build(self) -> Fir {
        Fir::NormalBrane(Box::new(NormalBraneFir {
            characterizations: self.characterizations,
            statements: self.statements,
            state: self.state,
            alarm: self.alarm,
        }))
    }
}

/// Builder for StatementFir.
pub struct StatementFirBuilder {
    name: Option<String>,
    body: FirRef,
    state: Nyes,
}

impl StatementFirBuilder {
    pub fn new(name: Option<String>, body: Fir) -> Self {
        Self {
            name,
            body: fir_to_ref(body),
            state: Nyes::Embryonic,
        }
    }
    pub fn anonymous(body: Fir) -> Self {
        Self {
            name: None,
            body: fir_to_ref(body),
            state: Nyes::Embryonic,
        }
    }
    pub fn state(mut self, state: Nyes) -> Self {
        self.state = state;
        self
    }
    pub fn build(self) -> StatementFir {
        StatementFir {
            name: self.name,
            body: self.body,
            state: self.state,
        }
    }
}

// ==================== Builder Unit Tests ====================

#[cfg(test)]
mod builder_tests {
    use super::*;

    #[test]
    fn test_constant_int_builder() {
        let fir = ConstantIntFirBuilder::new(42).build();
        assert!(matches!(fir, Fir::ConstantInt(ref i) if i.value == 42));
        if let Fir::ConstantInt(i) = fir {
            assert_eq!(i.state, Nyes::Constant);
        }
    }

    #[test]
    fn test_constant_int_builder_state() {
        let fir = ConstantIntFirBuilder::new(7).state(Nyes::Embryonic).build();
        if let Fir::ConstantInt(i) = fir {
            assert_eq!(i.state, Nyes::Embryonic);
        } else {
            panic!("Expected ConstantInt");
        }
    }

    #[test]
    fn test_nk_builder() {
        let fir = NkFirBuilder::new("unknown").build();
        if let Fir::Nk(i) = fir {
            assert_eq!(i.reason, "unknown");
            assert_eq!(i.state, Nyes::Nk);
            assert!(i.alarm.is_none());
        } else {
            panic!("Expected Nk");
        }
    }

    #[test]
    fn test_nk_builder_with_alarm() {
        let alarm = Alarm {
            level: AlarmLevel::Mild,
            code: "TEST".into(),
            message: "test".into(),
            source: AlarmSource::Evaluator,
        };
        let fir = NkFirBuilder::new("err")
            .alarm(alarm)
            .state(Nyes::Constant)
            .build();
        if let Fir::Nk(i) = fir {
            assert!(i.alarm.is_some());
            assert_eq!(i.state, Nyes::Constant);
        } else {
            panic!("Expected Nk");
        }
    }

    #[test]
    fn test_operator_builder() {
        let left = ConstantIntFirBuilder::new(1).build();
        let right = ConstantIntFirBuilder::new(2).build();
        let fir = OperatorFirBuilder::new("+")
            .operand(left)
            .operand(right)
            .state(Nyes::Constant)
            .build();
        if let Fir::Operator(i) = fir {
            assert_eq!(i.op, "+");
            assert_eq!(i.operands.len(), 2);
            assert_eq!(i.state, Nyes::Constant);
        } else {
            panic!("Expected Operator");
        }
    }

    #[test]
    fn test_operator_builder_vec() {
        let ops = vec![
            ConstantIntFirBuilder::new(1).build(),
            ConstantIntFirBuilder::new(2).build(),
            ConstantIntFirBuilder::new(3).build(),
        ];
        let fir = OperatorFirBuilder::new("*").operands(ops).build();
        if let Fir::Operator(i) = fir {
            assert_eq!(i.operands.len(), 3);
        } else {
            panic!("Expected Operator");
        }
    }

    #[test]
    fn test_search_builder() {
        let fir = SearchFirBuilder::new("^x$")
            .direction(SearchDirection::Forward)
            .anchored(true)
            .state(Nyes::Econstanic)
            .build();
        if let Fir::Search(i) = fir {
            assert_eq!(i.pattern, "^x$");
            assert_eq!(i.direction, SearchDirection::Forward);
            assert!(i.anchored);
            assert_eq!(i.state, Nyes::Econstanic);
            assert!(i.result.is_none());
        } else {
            panic!("Expected Search");
        }
    }

    #[test]
    fn test_search_builder_with_result() {
        let result = ConstantIntFirBuilder::new(42).build();
        let fir = SearchFirBuilder::new("x").result(result).build();
        if let Fir::Search(i) = fir {
            assert!(i.result.is_some());
        } else {
            panic!("Expected Search");
        }
    }

    #[test]
    fn test_index_builder() {
        let fir = IndexFirBuilder::new(5)
            .anchored(true)
            .state(Nyes::Constant)
            .build();
        if let Fir::Index(i) = fir {
            assert_eq!(i.offset, 5);
            assert!(i.anchored);
            assert_eq!(i.state, Nyes::Constant);
        } else {
            panic!("Expected Index");
        }
    }

    #[test]
    fn test_stay_foolish_builder() {
        let inner = ConstantIntFirBuilder::new(99).build();
        let fir = StayFoolishFirBuilder::new(inner)
            .state(Nyes::Constant)
            .build();
        if let Fir::StayFoolish(i) = fir {
            assert_eq!(i.state, Nyes::Constant);
            assert_eq!(i.expr.borrow().as_int(), Some(99));
        } else {
            panic!("Expected StayFoolish");
        }
    }

    #[test]
    fn test_stay_fully_foolish_builder() {
        let inner = NkFirBuilder::new("nope").build();
        let fir = StayFullyFoolishFirBuilder::new(inner).build();
        if let Fir::StayFullyFoolish(i) = fir {
            assert_eq!(i.expr.borrow().fir_variant(), "Nk");
        } else {
            panic!("Expected StayFullyFoolish");
        }
    }

    #[test]
    fn test_concatenation_builder() {
        let e1 = ConstantIntFirBuilder::new(1).build();
        let e2 = ConstantIntFirBuilder::new(2).build();
        let merged = ConstantIntFirBuilder::new(3).build();
        let fir = ConcatenationFirBuilder::new()
            .element(e1)
            .element(e2)
            .merged(merged)
            .state(Nyes::Constant)
            .build();
        if let Fir::Concatenation(i) = fir {
            assert_eq!(i.elements.len(), 2);
            assert!(i.merged.is_some());
        } else {
            panic!("Expected Concatenation");
        }
    }

    #[test]
    fn test_concatenation_builder_vec() {
        let elems = vec![
            ConstantIntFirBuilder::new(1).build(),
            ConstantIntFirBuilder::new(2).build(),
        ];
        let fir = ConcatenationFirBuilder::new().elements(elems).build();
        if let Fir::Concatenation(i) = fir {
            assert_eq!(i.elements.len(), 2);
        } else {
            panic!("Expected Concatenation");
        }
    }

    #[test]
    fn test_normal_brane_builder() {
        let body = ConstantIntFirBuilder::new(42).build();
        let fir = NormalBraneFirBuilder::new()
            .statement(Some("x".into()), body)
            .state(Nyes::Constant)
            .build();
        if let Fir::NormalBrane(i) = fir {
            assert_eq!(i.statements.len(), 1);
            assert_eq!(i.statements[0].name, Some("x".into()));
            assert_eq!(i.state, Nyes::Constant);
        } else {
            panic!("Expected NormalBrane");
        }
    }

    #[test]
    fn test_normal_brane_builder_multi() {
        let s = vec![
            (Some("a".into()), ConstantIntFirBuilder::new(1).build()),
            (Some("b".into()), ConstantIntFirBuilder::new(2).build()),
        ];
        let fir = NormalBraneFirBuilder::new().statements(s).build();
        if let Fir::NormalBrane(i) = fir {
            assert_eq!(i.statements.len(), 2);
        } else {
            panic!("Expected NormalBrane");
        }
    }

    #[test]
    fn test_normal_brane_empty() {
        let fir = NormalBraneFirBuilder::new().build();
        if let Fir::NormalBrane(i) = fir {
            assert!(i.statements.is_empty());
            assert!(i.characterizations.is_empty());
        } else {
            panic!("Expected NormalBrane");
        }
    }

    #[test]
    fn test_statement_builder() {
        let body = ConstantIntFirBuilder::new(10).build();
        let stmt = StatementFirBuilder::new(Some("y".into()), body)
            .state(Nyes::Constant)
            .build();
        assert_eq!(stmt.name, Some("y".into()));
        assert_eq!(stmt.state, Nyes::Constant);
        assert_eq!(stmt.body.borrow().as_int(), Some(10));
    }

    #[test]
    fn test_statement_anonymous() {
        let body = ConstantIntFirBuilder::new(10).build();
        let stmt = StatementFirBuilder::anonymous(body).build();
        assert!(stmt.name.is_none());
    }

    // ── Builder + FirQueryable integration ──

    #[test]
    fn test_builder_constant_int_queryable() {
        let fir = ConstantIntFirBuilder::new(42).build();
        assert_eq!(fir.hs_variant(), "ConstantInt");
        assert_eq!(fir.hs_constant_int(), Some(42));
        assert_eq!(fir.hs_state(), Nyes::Constant);
    }

    #[test]
    fn test_builder_nested_brane_queryable() {
        let body = OperatorFirBuilder::new("+")
            .operand(ConstantIntFirBuilder::new(1).build())
            .operand(ConstantIntFirBuilder::new(2).build())
            .build();
        let fir = NormalBraneFirBuilder::new()
            .statement(Some("x".into()), body)
            .build();
        assert_eq!(fir.hs_variant(), "NormalBrane");
        if let Some((chars, stmts)) = fir.hs_brane() {
            assert!(chars.is_empty());
            assert_eq!(stmts.len(), 1);
            assert_eq!(stmts[0].name, Some("x".into()));
            assert_eq!(stmts[0].body.hs_variant(), "Operator");
        } else {
            panic!("Expected hs_brane to return Some");
        }
    }
}
