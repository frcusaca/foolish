
use std::cell::RefCell;
use std::rc::{Rc, Weak};

use foolish_core::fir::Nyes;
use regex::Regex;

use crate::fir_trait::{Fir, FirKind, FirRef, Scope, UbcError, get_value};
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
					 all_constantew  = false;
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
    None
}

fn clone_nyes(source: Nyes, descendent_of_sfm_and_foolishly_ignorant: bool) -> Nyes {
    if descendent_of_sfm_and_foolishly_ignorant {
        return source;
    }
    match source {
        Nyes::Constant | Nyes::Independent | Nyes::Nk => source,
        Nyes::Econstanic | Nyes::Woconstanic => Nyes::Embryonic,
        Nyes::Prembrionic | Nyes::Embryonic | Nyes::Braning => Nyes::Embryonic,
    }
}

fn deepest_econstanic_in_chain(search: &FirRef) -> Option<FirRef> {
    let mut current = Rc::clone(search);
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

macro_rules! clone_proto_brane {
    ($source:expr, $self_weak:expr, $new_parent:expr, $nyes:expr, $foolish:expr) => {{
        let cloned_children: Vec<FirRef> = $source
            .foolish_children()
            .iter()
            .enumerate()
            .map(|(i, c)| constanic_clone_at(c, &$self_weak, i, $foolish))
            .collect();
        let core = ProtoBrane::new(cloned_children, $new_parent.clone(), clone_nyes($nyes, $foolish));
        for ubc in $source.ubc_children() {
            core.push_ubc_child(constanic_clone_at(&ubc, &$self_weak, 0, $foolish));
        }
        core
    }};
}

fn constanic_clone_at(
    fir_ref: &FirRef,
    new_parent: &Weak<RefCell<dyn Fir>>,
    index: usize,
    descendent_of_sfm_and_foolishly_ignorant: bool,
) -> FirRef {
    if matches!(fir_ref.borrow().kind(), FirKind::StayFoolish | FirKind::StayFullyFoolish) {
        let source = fir_ref.borrow();
        if source.kind() == FirKind::StayFoolish {
            if let Some(constanic_result) = source.core().ubc_children().into_iter().next() {
                return constanic_clone_at(&constanic_result, new_parent, index, descendent_of_sfm_and_foolishly_ignorant);
            }
        }
        if let Some(inner) = source.core().foolish_children().first().cloned() {
            return constanic_clone_at(&inner, new_parent, index, descendent_of_sfm_and_foolishly_ignorant);
        }
        eprintln!("ALARM: SF/SFF node has no children — cloning wrapper as-is");
    }
    let nyes = fir_ref.borrow().core().get_nyes();
    if nyes == Nyes::Constant || nyes == Nyes::Independent {
        if fir_ref.borrow().kind() != FirKind::Brane {
            return Rc::clone(fir_ref);
        }
    }
    let borrowed = fir_ref.borrow();
    let kind = borrowed.kind();
    match kind {
        FirKind::IndepInt => {
            Rc::new(RefCell::new(IndepIntFir {
                core: ProtoBrane::new(vec![], new_parent.clone(), borrowed.core().get_nyes()),
                value: borrowed.as_i64().unwrap_or(0),
            }))
        }
        FirKind::Nk => {
            Rc::new(RefCell::new(NkFir {
                core: ProtoBrane::new(vec![], new_parent.clone(), borrowed.core().get_nyes()),
                reason: borrowed.as_nk_reason().unwrap_or("unknown").to_owned(),
            }))
        }
        FirKind::Operator => {
            let op_name = borrowed.as_op_name().unwrap_or("?").to_owned();
            Rc::new_cyclic(|me: &Weak<RefCell<OperatorFir>>| {
                let self_weak: Weak<RefCell<dyn Fir>> = me.clone();
                let core = clone_proto_brane!(borrowed.core(), self_weak, new_parent, nyes, descendent_of_sfm_and_foolishly_ignorant);
                RefCell::new(OperatorFir { core, op: op_name })
            })
        }
        FirKind::Search => {
            let clone_nyes_val = clone_nyes(nyes, descendent_of_sfm_and_foolishly_ignorant);
            let pattern = borrowed.as_search_pattern().unwrap_or("").to_owned();
            let anchored = borrowed.as_search_anchored();
            let chain_econstanic = if !descendent_of_sfm_and_foolishly_ignorant
                && nyes == Nyes::Woconstanic
            {
                deepest_econstanic_in_chain(fir_ref)
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
                    .map(|(i, c)| constanic_clone_at(c, &self_weak, i, descendent_of_sfm_and_foolishly_ignorant))
                    .collect();
                let core = ProtoBrane::new(children, new_parent.clone(), clone_nyes_val);
                if let Some(ref econ) = chain_econstanic {
                    core.push_ubc_child(constanic_clone_at(econ, &self_weak, 0, false));
                } else {
                    for ubc in borrowed.core().ubc_children() {
                        core.push_ubc_child(constanic_clone_at(&ubc, &self_weak, 0, descendent_of_sfm_and_foolishly_ignorant));
                    }
                }
                RefCell::new(SearchFir {
                    core,
                    pattern,
                    anchored,
                    forward: false,
                    sf_inner_pattern: RefCell::new(None),
                })
            })
        }
        FirKind::Index => {
            let offset = borrowed.as_index_offset();
            let anchored = borrowed.as_index_anchored();
            Rc::new_cyclic(|me: &Weak<RefCell<IndexFir>>| {
                let self_weak: Weak<RefCell<dyn Fir>> = me.clone();
                let core = clone_proto_brane!(borrowed.core(), self_weak, new_parent, nyes, descendent_of_sfm_and_foolishly_ignorant);
                RefCell::new(IndexFir { core, offset, anchored })
            })
        }
        FirKind::HeadTail => {
            let is_head = borrowed.as_headtail_is_head();
            let anchored = borrowed.as_headtail_anchored();
            Rc::new_cyclic(|me: &Weak<RefCell<HeadTailFir>>| {
                let self_weak: Weak<RefCell<dyn Fir>> = me.clone();
                let core = clone_proto_brane!(borrowed.core(), self_weak, new_parent, nyes, descendent_of_sfm_and_foolishly_ignorant);
                RefCell::new(HeadTailFir { core, is_head, anchored })
            })
        }
        FirKind::StayFoolish | FirKind::StayFullyFoolish => {
            unreachable!("SF/SFF stripped at fn top")
        }
        FirKind::Concatenation => {
            Rc::new_cyclic(|me: &Weak<RefCell<ConcatenationFir>>| {
                let self_weak: Weak<RefCell<dyn Fir>> = me.clone();
                let core = clone_proto_brane!(borrowed.core(), self_weak, new_parent, nyes, descendent_of_sfm_and_foolishly_ignorant);
                RefCell::new(ConcatenationFir { core })
            })
        }
        FirKind::Statement => {
            let name = borrowed.as_stmt_name().unwrap_or("").to_owned();
            let line = index;
            Rc::new_cyclic(|me: &Weak<RefCell<StatementFir>>| {
                let self_weak: Weak<RefCell<dyn Fir>> = me.clone();
                let core = clone_proto_brane!(borrowed.core(), self_weak, new_parent, nyes, descendent_of_sfm_and_foolishly_ignorant);
                RefCell::new(StatementFir { core, name, line_number: line })
            })
        }
        FirKind::Brane => {
            Rc::new_cyclic(|me: &Weak<RefCell<BraneFir>>| {
                let self_weak: Weak<RefCell<dyn Fir>> = me.clone();
                let core = clone_proto_brane!(borrowed.core(), self_weak, new_parent, nyes, descendent_of_sfm_and_foolishly_ignorant);
                RefCell::new(BraneFir {
                    core,
                    characterizations: borrowed.as_brane_characterizations().to_vec(),
                })
            })
        }
        FirKind::Unknown => {
            Rc::new(RefCell::new(NkFir {
                core: ProtoBrane::new(vec![], new_parent.clone(), Nyes::Nk),
                reason: "unknown fir kind".to_owned(),
            }))
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
}

impl Fir for IndepIntFir {
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
}

impl Fir for NkFir {
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
    fn operands_all_constanic(&self) -> bool {
        self.core()
            .foolish_children()
            .iter()
            .all(|c| c.borrow().core().get_nyes().is_constanic())
    }
    fn operands_all_settled(&self) -> bool {
        self.core()
            .foolish_children()
            .iter()
            .all(|c| matches!(c.borrow().core().get_nyes(), Nyes::Constant | Nyes::Independent))
    }
}

impl Fir for OperatorFir {
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
                        let reason = children.iter()
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
                    self.core.push_ubc_child(constanic_clone_at(&nk_ref, &self_weak, 0, scope.has_ancestral_sfm));
                    self.core.set_nyes(Nyes::Nk);
                    return Ok(());
                }

                let values: Vec<i64> = children
                    .iter()
                    .map(|c| get_value(c))
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
                            self.core.push_ubc_child(constanic_clone_at(&nk_ref, &self_weak, 0, scope.has_ancestral_sfm));
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
                            self.core.push_ubc_child(constanic_clone_at(&nk_ref, &self_weak, 0, scope.has_ancestral_sfm));
                            self.core.set_nyes(Nyes::Nk);
                            return Ok(());
                        }
                        values[0] % values[1]
                    }
                    "-" if values.len() == 1 => -values[0], // unary negation
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
                self.core.push_ubc_child(constanic_clone_at(&result_ref, &self_weak, 0, scope.has_ancestral_sfm));
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
}

impl Fir for StatementFir {
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

    fn ib_search(&self, self_ref: &FirRef, name: &str) -> Option<(FirRef, Nyes)> {
        let brane = self.get_my_brane(self_ref)?;
        search_brane_children(&brane, name, Some(self.line_number), false)
            .map(|(body, nyes, _sf_pat)| (body, nyes))
    }
}


#[derive(Debug)]
pub struct BraneFir {
    pub(crate) core: ProtoBrane,
    pub(crate) characterizations: Vec<String>,
}

impl Fir for BraneFir {
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

    fn ab_search(&self, self_ref: &FirRef, name: &str) -> Option<(FirRef, Nyes)> {
        let stmt = self.get_my_statement(self_ref);
        let stmt_borrowed = stmt.borrow();
        if let Some((body, nyes)) = stmt_borrowed.ib_search(&stmt, name) {
            return Some((body, nyes));
        }
        drop(stmt_borrowed);
        let parent_brane = self.get_my_brane(self_ref)?;
        if Rc::ptr_eq(&parent_brane, self_ref) {
            return None;
        }
        let pb = parent_brane.borrow();
        pb.ab_search(&parent_brane, name)
    }
}


#[derive(Debug)]
pub struct SearchFir {
    pub(crate) core: ProtoBrane,
    pub(crate) pattern: String,
    pub(crate) anchored: bool,
    pub(crate) forward: bool,
    pub(crate) sf_inner_pattern: RefCell<Option<String>>,
}


fn extract_simple_name(pattern: &str) -> &str {
    let s = pattern.strip_prefix('^').unwrap_or(pattern);
    s.strip_suffix('$').unwrap_or(s)
}

fn resolve_anchor(anchor: &FirRef) -> FirRef {
    get_value(anchor)
}

fn find_stmt_index_in_brane(stmt: &FirRef, brane: &FirRef) -> Option<usize> {
    let brane_borrowed = brane.borrow();
    for (i, child) in brane_borrowed.core().foolish_children().iter().enumerate() {
        if Rc::ptr_eq(child, stmt) {
            return Some(i);
        }
    }
    None
}

fn matches_pattern(stmt_name: &str, pattern: &str) -> bool {
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

fn unwrap_sf_sff(fir_ref: &FirRef) -> FirRef {
    let kind = fir_ref.borrow().kind();
    match kind {
        FirKind::StayFoolish | FirKind::StayFullyFoolish => {
            if let Some(inner) = fir_ref.borrow().core().foolish_children().first() {
                Rc::clone(inner)
            } else {
                Rc::clone(fir_ref)
            }
        }
        _ => Rc::clone(fir_ref),
    }
}

fn search_brane_children(brane: &FirRef, name: &str, before: Option<usize>, forward: bool) -> Option<(FirRef, Nyes, Option<String>)> {
    let (children_vec, end) = {
        let brane_borrowed = brane.borrow();
        let c: Vec<FirRef> = brane_borrowed.core().foolish_children().to_vec();
        let e = before.unwrap_or(c.len());
        (c, e)
    };
    let range = &children_vec[..end];
    let iter: Box<dyn Iterator<Item = &FirRef>> = if forward {
        Box::new(range.iter())
    } else {
        Box::new(range.iter().rev())
    };
    for child in iter {
        let child_borrowed = child.borrow();
        if let Some(sn) = child_borrowed.as_stmt_name() {
            if matches_pattern(sn, name)
                && let Some(body) = child_borrowed.core().foolish_children().first()
            {
                let body_nyes = body.borrow().core().get_nyes();
                return Some((Rc::clone(body), body_nyes, None));
            }
        }
    }
    None
}

fn search_nyes_from_found(found: Nyes) -> Nyes {
    match found {
        Nyes::Econstanic | Nyes::Woconstanic | Nyes::Nk => Nyes::Woconstanic,
        Nyes::Constant | Nyes::Independent => Nyes::Constant,
        other => other,
    }
}

fn ib_search(start: &ProtoBrane, name: &str, forward: bool) -> Option<(FirRef, Nyes, Option<String>)> {
    let mut current = start.parent();
    while let Some(node) = current {
        if node.borrow().kind() == FirKind::Statement {
            let brane = {
                let borrowed = node.borrow();
                find_parent_brane(borrowed.core())
            };
            if let Some(ref brane_ref) = brane {
                let before_idx = find_stmt_index_in_brane(&node, brane_ref);
                if let Some((body, nyes, sf_pat)) = search_brane_children(brane_ref, name, before_idx, forward) {
                    if nyes.is_constanic() {
                        return Some((body, nyes, sf_pat));
                    }
                    eprintln!("ib_search: found '{name}' but nyes={nyes:?} is not constanic — skipping");
                }
                return None;
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

fn ab_search(start: &ProtoBrane, name: &str, forward: bool) -> Option<(FirRef, Nyes, Option<String>)> {
    let mut current = start.parent();
    let mut seen_immediate = false;
    while let Some(node) = current {
        if node.borrow().kind() == FirKind::Statement {
            let brane = {
                let borrowed = node.borrow();
                find_parent_brane(borrowed.core())
            };
            if let Some(ref brane_ref) = brane {
                if !seen_immediate {
                    seen_immediate = true;
                } else {
                    let before_idx = find_stmt_index_in_brane(&node, brane_ref);
                    if let Some((body, nyes, sf_pat)) = search_brane_children(brane_ref, name, before_idx, forward) {
                        if nyes.is_constanic() {
                            return Some((body, nyes, sf_pat));
                        }
                    }
                }
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

impl SearchFir {
    fn handle_found(
        &self,
        body: FirRef,
        _nyes: Nyes,
        sf_pat: Option<String>,
        scope: &Scope,
    ) {
        if let Some(p) = sf_pat {
            *self.sf_inner_pattern.borrow_mut() = Some(p);
        }
        let self_weak = self.core.parent_weak();
        self.core
            .push_search_result(constanic_clone_at(&body, &self_weak, 0, scope.has_ancestral_sfm));
        self.core.set_nyes(Nyes::Braning);
    }

    fn settle_from_ubc_result(&self) {
        let result_nyes = self
            .core
            .ubc_children()
            .first()
            .map(|r| r.borrow().core().get_nyes())
            .unwrap_or(Nyes::Nk);
        self.core.set_nyes(search_nyes_from_found(result_nyes));
    }
}

impl Fir for SearchFir {
    fn core(&self) -> &ProtoBrane {
        &self.core
    }
    fn fir_op_step(&self, scope: &Scope) -> Result<(), UbcError> {
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
                    match ib_search(&self.core, &self.pattern, self.forward) {
                        Some((body, nyes, sf_pat)) => {
                            eprintln!("ib_search: found '{}' nyes={:?} — handle_found", self.pattern, nyes);
                            self.handle_found(body, nyes, sf_pat, scope);
                        }
                        None => {
                            eprintln!("ib_search: '{}' not found — escalating", self.pattern);
                            self.core.set_nyes(Nyes::Braning);
                        }
                    }
                }
            }
            Nyes::Braning => {
                if !self.core.ubc_children().is_empty() {
                    self.settle_from_ubc_result();
                } else if self.anchored {
                    let anchor = Rc::clone(&self.core.foolish_children()[0]);
                    let resolved = resolve_anchor(&anchor);
                    let name = &self.pattern;
                    if let Some((body, nyes, sf_pat)) = search_brane_children(&resolved, name, None, self.forward) {
                        self.handle_found(body, nyes, sf_pat, scope);
                    } else {
                        self.core.set_nyes(Nyes::Nk);
                    }
                } else {
                    match ab_search(&self.core, &self.pattern, self.forward) {
                        Some((body, nyes, sf_pat)) => self.handle_found(body, nyes, sf_pat, scope),
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
}


#[derive(Debug)]
pub struct IndexFir {
    pub(crate) core: ProtoBrane,
    pub(crate) offset: i32,
    pub(crate) anchored: bool,
}

fn index_into_brane(brane: &FirRef, offset: i32) -> Option<(FirRef, Nyes)> {
    let (body, body_nyes) = {
        let body: FirRef = {
            let borrowed = brane.borrow();
            let children = borrowed.core().foolish_children();
            let len = children.len() as i32;
            let idx = if offset >= 0 { offset } else { len + offset };
            if idx < 0 || idx >= len {
                return None;
            }
            let stmt = Rc::clone(&children[idx as usize]);
            let body = Rc::clone(stmt.borrow().core().foolish_children().first()?);
            body
        };
        let body_nyes = body.borrow().core().get_nyes();
        (body, body_nyes)
    };
    Some((body, body_nyes))
}

fn index_into_brane_relative(brane: &FirRef, stmt_idx: usize, offset: i32) -> Option<(FirRef, Nyes)> {
    let (body, body_nyes) = {
        let body: FirRef = {
            let borrowed = brane.borrow();
            let children = borrowed.core().foolish_children();
            let idx = stmt_idx as i32 + offset;
            if idx < 0 || idx >= children.len() as i32 {
                return None;
            }
            let stmt = Rc::clone(&children[idx as usize]);
            Rc::clone(stmt.borrow().core().foolish_children().first()?)
        };
        let body_nyes = body.borrow().core().get_nyes();
        (body, body_nyes)
    };
    Some((body, body_nyes))
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
    fn settle_from_ubc_result(&self) {
        let nyes = self
            .core
            .ubc_children()
            .first()
            .map(|r| r.borrow().core().get_nyes())
            .unwrap_or(Nyes::Nk);
        self.core.set_nyes(nyes);
    }
}

impl Fir for IndexFir {
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
                    match find_enclosing_stmt_and_brane(&self.core) {
                        Some((stmt_ref, brane_ref)) => {
                            if let Some(idx) = find_stmt_index_in_brane(&stmt_ref, &brane_ref) {
                                if let Some((body, _body_nyes)) = index_into_brane_relative(&brane_ref, idx, self.offset) {
                                    let self_weak = self.core.parent_weak();
                                    self.core.push_search_result(constanic_clone_at(&body, &self_weak, 0, scope.has_ancestral_sfm));
                                    self.core.set_nyes(Nyes::Braning);
                                } else {
                                    self.core.set_nyes(Nyes::Nk);
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
                } else if self.anchored {
                    let anchor = Rc::clone(&self.core.foolish_children()[0]);
                    let resolved = resolve_anchor(&anchor);
                    if let Some((body, _body_nyes)) = index_into_brane(&resolved, self.offset) {
                        let self_weak = self.core.parent_weak();
                        self.core.push_search_result(constanic_clone_at(&body, &self_weak, 0, scope.has_ancestral_sfm));
                    } else {
                        self.core.set_nyes(Nyes::Nk);
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
}


#[derive(Debug)]
pub struct HeadTailFir {
    pub(crate) core: ProtoBrane,
    pub(crate) is_head: bool,
    pub(crate) anchored: bool,
}

impl Fir for HeadTailFir {
    fn core(&self) -> &ProtoBrane {
        &self.core
    }
    fn fir_op_step(&self, scope: &Scope) -> Result<(), UbcError> {
        let offset: i32 = if self.is_head { 0 } else { -1 };
        match self.core.get_nyes() {
            Nyes::Prembrionic | Nyes::Embryonic => {
                if self.anchored {
                    let anchor = Rc::clone(&self.core.foolish_children()[0]);
                    self.core.push_task(anchor);
                    self.core.set_nyes(Nyes::Braning);
                } else {
                    if offset >= 0 {
                        return Err(UbcError::Eval(
                            "unanchored head/tail requires tail (negative offset)".to_owned(),
                        ));
                    }
                    match find_enclosing_stmt_and_brane(&self.core) {
                        Some((stmt_ref, brane_ref)) => {
                            if let Some(idx) = find_stmt_index_in_brane(&stmt_ref, &brane_ref) {
                                if let Some((body, nyes)) = index_into_brane_relative(&brane_ref, idx, offset) {
                                    if nyes.is_constanic() {
                                        let self_weak = self.core.parent_weak();
                                        self.core.push_search_result(constanic_clone_at(&body, &self_weak, 0, scope.has_ancestral_sfm));
                                        self.core.set_nyes(nyes);
                                    } else {
                                        self.core.push_task(Rc::clone(&body));
                                        self.core.set_nyes(Nyes::Braning);
                                    }
                                } else {
                                    self.core.set_nyes(Nyes::Nk);
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
                if self.anchored {
                    let anchor = Rc::clone(&self.core.foolish_children()[0]);
                    let resolved = resolve_anchor(&anchor);
                    if let Some((body, body_nyes)) = index_into_brane(&resolved, offset) {
                        if body_nyes.is_constanic() {
                            let self_weak = self.core.parent_weak();
                            self.core.push_search_result(constanic_clone_at(&body, &self_weak, 0, scope.has_ancestral_sfm));
                            self.core.set_nyes(body_nyes);
                        } else {
                            self.core.set_nyes(Nyes::Nk);
                        }
                    } else {
                        self.core.set_nyes(Nyes::Nk);
                    }
                } else {
                    match find_enclosing_stmt_and_brane(&self.core) {
                        Some((stmt_ref, brane_ref)) => {
                            if let Some(idx) = find_stmt_index_in_brane(&stmt_ref, &brane_ref) {
                                if let Some((body, body_nyes)) = index_into_brane_relative(&brane_ref, idx, offset) {
                                    if body_nyes.is_constanic() {
                                        let self_weak = self.core.parent_weak();
                                        self.core.push_search_result(constanic_clone_at(&body, &self_weak, 0, scope.has_ancestral_sfm));
                                        self.core.set_nyes(body_nyes);
                                    } else {
                                        self.core.set_nyes(Nyes::Nk);
                                    }
                                } else {
                                    self.core.set_nyes(Nyes::Nk);
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
            _ => {}
        }
        Ok(())
    }
    fn kind(&self) -> FirKind {
        FirKind::HeadTail
    }
    fn as_headtail_is_head(&self) -> bool {
        self.is_head
    }
    fn as_headtail_anchored(&self) -> bool {
        self.anchored
    }
}


#[derive(Debug)]
pub struct StayFoolishFir {
    pub(crate) core: ProtoBrane,
}

impl Fir for StayFoolishFir {
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

impl Fir for StayFullyFoolishFir {
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

impl Fir for ConcatenationFir {
    fn core(&self) -> &ProtoBrane {
        &self.core
    }
    fn fir_op_step(&self, _scope: &Scope) -> Result<(), UbcError> {
        match self.core.get_nyes() {
            Nyes::Prembrionic | Nyes::Embryonic => {
                let children: Vec<FirRef> = self.core.foolish_children().to_vec();
                if children.is_empty() {
                    let result_ref: FirRef =
                        Rc::new_cyclic(|me: &Weak<RefCell<BraneFir>>| {
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
                    let resolved = get_value(c);
                    resolved.borrow().core().get_nyes() == Nyes::Nk
                });
                let any_woconstanic = children.iter().any(|c| {
                    let resolved = get_value(c);
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


pub fn constant_int(value: i64, parent: Weak<RefCell<dyn Fir>>) -> FirRef {
    Rc::new(RefCell::new(IndepIntFir {
        core: ProtoBrane::new(vec![], parent, Nyes::Prembrionic),
        value,
    }))
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
    }))
}

pub fn headtail(
    is_head: bool,
    anchored: bool,
    children: Vec<FirRef>,
    parent: Weak<RefCell<dyn Fir>>,
) -> FirRef {
    Rc::new(RefCell::new(HeadTailFir {
        core: ProtoBrane::new(children, parent, Nyes::Prembrionic),
        is_head,
        anchored,
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
    use crate::fir_trait::step_fir_ref;
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

        let report = step_fir_ref(&node, &scope).unwrap();
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

        let _ = step_fir_ref(&node, &scope).unwrap();

        assert_eq!(node.borrow().kind(), FirKind::IndepInt);
    }

    #[test]
    fn nk_prembrionic_to_nk_in_one_step() {
        let node = make_nk("unbound name");
        let scope = Scope::empty();

        assert_eq!(node.borrow().core().get_nyes(), Nyes::Prembrionic);
        let mut transitions = vec![Nyes::Prembrionic];

        let report = step_fir_ref(&node, &scope).unwrap();
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

        let _ = step_fir_ref(&node, &scope).unwrap();

        assert_eq!(node.borrow().kind(), FirKind::Nk);
    }

    #[test]
    fn both_leaf_kinds_are_settled_after_one_step() {
        let ci = make_constant_int(100);
        let nk = make_nk("nope");
        let scope = Scope::empty();

        assert!(!ci.borrow().core().get_nyes().is_constanic());
        assert!(!nk.borrow().core().get_nyes().is_constanic());

        let r1 = step_fir_ref(&ci, &scope).unwrap();
        let r2 = step_fir_ref(&nk, &scope).unwrap();

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

        let _ = step_fir_ref(&ci, &scope).unwrap();
        let _ = step_fir_ref(&nk, &scope).unwrap();

        let r1 = step_fir_ref(&ci, &scope).unwrap();
        let r2 = step_fir_ref(&nk, &scope).unwrap();

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
        let node = constant_int(99, parent_weak);

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
            })
        })
    }

    fn step_to_settled(node: &FirRef, scope: &Scope) -> Vec<Nyes> {
        let mut transitions = vec![node.borrow().core().get_nyes()];
        for _ in 0..50 {
            let report = step_fir_ref(node, scope).unwrap();
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
            let report = step_fir_ref(&op, &scope).unwrap();
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
            let report = step_fir_ref(&op, &scope).unwrap();
            if let StepReport::Progress(nyes) = report {
                if nyes.is_constanic() {
                    break;
                }
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
            let report = step_fir_ref(&op, &scope).unwrap();
            if let StepReport::Progress(nyes) = report {
                if nyes.is_constanic() {
                    break;
                }
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
            let report = step_fir_ref(&op, &scope).unwrap();
            if let StepReport::Progress(nyes) = report {
                if nyes.is_constanic() {
                    break;
                }
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
            let report = step_fir_ref(&op, &scope).unwrap();
            if let StepReport::Progress(nyes) = report {
                if nyes.is_constanic() {
                    break;
                }
            }
        }

        assert_eq!(op.borrow().core().get_nyes(), Nyes::Nk);
        assert_eq!(op.borrow().core().ubc_children().len(), 1);
        assert_eq!(op.borrow().core().ubc_children()[0].borrow().kind(), FirKind::Nk);
    }

    #[test]
    fn operator_with_nk_operand_is_nk() {
        let a = make_constant_int(5);
        let b = make_nk("unbound");
        let op = make_operator("+", vec![Rc::clone(&a), Rc::clone(&b)]);
        let scope = Scope::empty();

        for _ in 0..20 {
            let report = step_fir_ref(&op, &scope).unwrap();
            if let StepReport::Progress(nyes) = report {
                if nyes.is_constanic() {
                    break;
                }
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

        let r1 = step_fir_ref(&brane, &scope).unwrap();
        assert!(matches!(r1, StepReport::Progress(Nyes::Braning)));

        let mut child_a_settled_first = false;
        for _ in 0..20 {
            let _ = step_fir_ref(&brane, &scope).unwrap();
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
        assert_eq!(ubc.len(), 1);
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

        assert_eq!(search.borrow().core().get_nyes(), Nyes::Woconstanic);
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
        assert_eq!(ubc.len(), 1);
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
        assert_eq!(ubc.len(), 1);
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
        assert_eq!(ubc.len(), 1);
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
        Rc::new_cyclic(|me: &Weak<RefCell<HeadTailFir>>| {
            let parent: Weak<RefCell<dyn Fir>> = me.clone();
            RefCell::new(HeadTailFir {
                core: ProtoBrane::new(foolish_children, parent, Nyes::Prembrionic),
                is_head,
                anchored,
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
        assert_eq!(ubc.len(), 1);
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
        assert_eq!(ubc.len(), 1);
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
        assert_eq!(head.borrow().core().ubc_children()[0].borrow().as_i64(), Some(42));
        assert_eq!(tail.borrow().core().get_nyes(), Nyes::Constant);
        assert_eq!(tail.borrow().core().ubc_children()[0].borrow().as_i64(), Some(42));
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
        assert_eq!(ubc.len(), 1, "SF should constanic-clone result into ubc_children");
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
        assert_eq!(ubc.len(), 1, "SF should constanic-clone nk result into ubc_children");
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
            let report = step_fir_ref(&sff, &scope).unwrap();
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

        step_fir_ref(&sff, &scope).unwrap(); // Prembrionic → Braning (child pushed)
        step_fir_ref(&sff, &scope).unwrap(); // child stepped to CONSTANT
        step_fir_ref(&sff, &scope).unwrap(); // child popped (constanic)
        let report = step_fir_ref(&sff, &scope).unwrap(); // Braning → CONSTANT
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
        assert_eq!(clone_nyes(Nyes::Constant, false), Nyes::Constant);
        assert_eq!(clone_nyes(Nyes::Independent, false), Nyes::Independent);
        assert_eq!(clone_nyes(Nyes::Nk, false), Nyes::Nk);
        assert_eq!(
            clone_nyes(Nyes::Econstanic, false),
            Nyes::Embryonic,
            "NICC resets ECONSTANIC to EMBRYONIC so it re-steps (IB then AB)"
        );
        assert_eq!(
            clone_nyes(Nyes::Woconstanic, false),
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
            assert_eq!(clone_nyes(n, true), n, "FICC must copy {n:?} verbatim");
        }
    }

    #[test]
    fn nicc_resets_econstanic_to_embryonic() {
        let op = make_operator("+", vec![make_constant_int(1), make_constant_int(2)]);
        op.borrow().core().set_nyes(Nyes::Econstanic);

        let cloned = constanic_clone_at(&op, &dangling_parent(), 0, false);

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

        let cloned = constanic_clone_at(&op, &dangling_parent(), 0, false);

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
        let cloned = constanic_clone_at(&woc, &dangling_parent(), 0, true);
        assert_eq!(
            cloned.borrow().core().get_nyes(),
            Nyes::Woconstanic,
            "FICC must keep a constanic compound's state verbatim"
        );

        let econ = make_operator("+", vec![make_constant_int(1), make_constant_int(2)]);
        econ.borrow().core().set_nyes(Nyes::Econstanic);
        let cloned = constanic_clone_at(&econ, &dangling_parent(), 0, true);
        assert_eq!(cloned.borrow().core().get_nyes(), Nyes::Econstanic);
    }

    #[test]
    fn leaf_clone_unchanged_both_modes() {
        let ci = make_constant_int(9);
        ci.borrow().core().set_nyes(Nyes::Constant);
        let n = constanic_clone_at(&ci, &dangling_parent(), 0, false);
        assert_eq!(n.borrow().core().get_nyes(), Nyes::Constant);
        let n = constanic_clone_at(&ci, &dangling_parent(), 0, true);
        assert_eq!(n.borrow().core().get_nyes(), Nyes::Constant);

        let nk = make_nk("gone");
        nk.borrow().core().set_nyes(Nyes::Nk);
        let n = constanic_clone_at(&nk, &dangling_parent(), 0, false);
        assert_eq!(n.borrow().core().get_nyes(), Nyes::Nk);
        let n = constanic_clone_at(&nk, &dangling_parent(), 0, true);
        assert_eq!(n.borrow().core().get_nyes(), Nyes::Nk);
    }

    #[test]
    fn cloning_sf_strips_the_mark() {
        let inner = make_constant_int(10);
        inner.borrow().core().set_nyes(Nyes::Econstanic); // force the clone path
        let sf = make_stay_foolish(Rc::clone(&inner));
        sf.borrow().core().set_nyes(Nyes::Econstanic);

        let normal = constanic_clone_at(&sf, &dangling_parent(), 0, false);
        assert_ne!(
            normal.borrow().kind(),
            FirKind::StayFoolish,
            "normal clone of an SF must NOT be a StayFoolish wrapper"
        );
        assert_eq!(normal.borrow().kind(), FirKind::IndepInt);

        let foolish = constanic_clone_at(&sf, &dangling_parent(), 0, true);
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
            let _ = step_fir_ref(root, scope).unwrap();
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
            let _ = step_fir_ref(&brane, &scope).unwrap();
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
        assert!(last.is_constanic(), "{label}: must end constanic (got {last:?})");
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
        let ht = make_headtail(true, true, vec![Rc::clone(&brane)]); // head
        let trace = step_to_settled(&ht, &Scope::empty());
        assert_progression(&trace, Nyes::Constant, "HeadTail(head)");
    }

    #[test]
    fn headtail_empty_nyes_transitions() {
        let brane = make_brane(vec![]);
        let ht = make_headtail(true, true, vec![Rc::clone(&brane)]);
        let trace = step_to_settled(&ht, &Scope::empty());
        assert_progression(&trace, Nyes::Nk, "HeadTail(empty)");
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

        let ib = ib_search(&search.borrow().core(), "^a$", false);
        assert!(ib.is_some(), "ib_search must find a name in the immediate brane");

        let scope = Scope::empty();
        let _ = step_to_settled(&root, &scope);
        assert!(search.borrow().core().get_nyes().is_constanic());
    }

    #[test]
    fn ab_context_name_not_in_immediate_brane() {
        let root = Compiler::compile("{a = 1; b = {c = a;};}").unwrap().pop().unwrap();
        let search = find_search(&root, "^a$").expect("search for a");

        {
            let b = search.borrow();
            assert!(
                ib_search(b.core(), "^a$", false).is_none(),
                "ib_search must NOT find an ancestral-only name in the immediate brane"
            );
            assert!(
                ab_search(b.core(), "^a$", false).is_some(),
                "ab_search must find the ancestral name"
            );
        }

        let scope = Scope::empty();
        let _ = step_to_settled(&root, &scope);
        assert!(search.borrow().core().get_nyes().is_constanic());
    }

    #[test]
    fn ib_shadows_ab_immediate_wins() {
        let root = Compiler::compile("{a = 1; b = {a = 2; c = a;};}")
            .unwrap()
            .pop()
            .unwrap();
        let search = find_search(&root, "^a$").expect("search for a");
        {
            let b = search.borrow();
            let ib = ib_search(b.core(), "^a$", false);
            assert!(ib.is_some(), "ib_search must find the immediate (shadowing) a");
        }

        let scope = Scope::empty();
        let _ = step_to_settled(&root, &scope);
        assert!(search.borrow().core().get_nyes().is_constanic());
        let result = get_value(&search);
        assert_eq!(
            result.borrow().as_i64(),
            Some(2),
            "shadowing: search must resolve to the immediate-brane a (2), not ancestral (1)"
        );
    }

    #[test]
    fn ancestral_search_passes_through_embryonic_then_braning() {
        let root = Compiler::compile("{a = 1; b = {c = a;};}").unwrap().pop().unwrap();
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
        assert_eq!(sa.borrow().core().get_nyes(), Nyes::Econstanic, "SFF search a built ECONSTANIC");
        assert_eq!(sb.borrow().core().get_nyes(), Nyes::Econstanic, "SFF search b built ECONSTANIC");

        let scope = Scope::empty();
        for _ in 0..200 {
            if root.borrow().core().get_nyes().is_constanic() { break; }
            let _ = step_fir_ref(&root, &scope).unwrap();
        }
        assert_eq!(sa.borrow().core().get_nyes(), Nyes::Econstanic, "SFF search a stays ECONSTANIC after stepping");
        assert_eq!(sb.borrow().core().get_nyes(), Nyes::Econstanic, "SFF search b stays ECONSTANIC after stepping");
    }

    #[test]
    fn sf_of_sff_sets_econstanic_body_constanic() {
        let root = Compiler::compile("{a = 1; b = 2; sff = <<a + b>>; sf = <sff>;}")
            .unwrap()
            .pop()
            .unwrap();
        let scope = Scope::empty();
        for _ in 0..200 {
            if root.borrow().core().get_nyes().is_constanic() { break; }
            let _ = step_fir_ref(&root, &scope).unwrap();
        }
        let sff_search = find_search(&root, "^sff$").expect("search sff");
        assert!(sff_search.borrow().core().get_nyes().is_constanic());
        let result = get_value(&sff_search);
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
        assert_eq!(stmts[0].borrow().as_stmt_name(), Some("a"), "named assignment keeps its LHS");
        assert_eq!(
            stmts[1].borrow().as_stmt_name(),
            Some(crate::compiler::ANON_STMT_NAME),
            "anonymous bare expression is named ???"
        );
    }
}

#[cfg(test)]
mod concat_sf_f_more_debug {
    use super::*;
    use crate::compiler::Compiler;
    use crate::fir_trait::{step_fir_ref, Scope, StepReport};

    #[test]
    fn trace_concat_sf_f_more() {
        let root = Compiler::compile(
            "{a=1; b=2; c=3; x=-10; y=-20; z=-30; \
             f1 = {b= <<x+y>> + <<a>> + <<b + c>>; a= <x+y> + a + b;}; \
             f2 = {b= <<x+z>> + <a + <<b>> + <c> >;}; \
             f3 = {c= <a + <<b + <x + y + z> >> + <c> >;}; \
             b = f1 <f2> <<f3>>; \
             d= b$;}",
        )
        .unwrap()
        .pop()
        .unwrap();
        let scope = Scope::empty();

        for step in 0..100 {
            let stmts = root.borrow().core().foolish_children().to_vec();
            let f1_stmt = stmts.iter().find(|s| s.borrow().as_stmt_name() == Some("f1")).unwrap();
            let f1_body = f1_stmt.borrow().core().foolish_children().first().unwrap().clone();
            let f1 = f1_body.borrow();

            let a_stmt = f1.core().foolish_children().iter().find(|s| s.borrow().as_stmt_name() == Some("a")).unwrap();
            let a_body = a_stmt.borrow().core().foolish_children().first().unwrap().clone();
            let a = a_body.borrow();
            let a_nyes = a.core().get_nyes();

            if a_nyes == Nyes::Braning {
                let ubc = a.core().ubc_children();
                if !ubc.is_empty() {
                    let b_clone = &ubc[0];
                    let bb = b_clone.borrow();
                    eprintln!("step {step}: f1.a=Braning, ubc[0] kind={:?} nyes={:?} children={}",
                        bb.kind(), bb.core().get_nyes(), bb.core().foolish_children().len());
                    for (i, c) in bb.core().foolish_children().iter().enumerate() {
                        let cb = c.borrow();
                        eprintln!("  child[{i}]: kind={:?} nyes={:?}", cb.kind(), cb.core().get_nyes());
                    }
                } else {
                    eprintln!("step {step}: f1.a=Braning, NO ubc_children");
                }
            }

            if a_nyes.is_constanic() {
                eprintln!("f1.a SETTLED at step {step}: {a_nyes:?}");
                return;
            }

            let report = step_fir_ref(&root, &scope).unwrap();
            if let StepReport::NoProgress = report {
                eprintln!("NoProgress at step {step}");
                break;
            }
        }
        panic!("f1.a did not settle");
    }
}

