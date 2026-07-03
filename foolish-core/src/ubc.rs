use std::rc::Rc;

use crate::fir::{Fir, FirRef, Nyes, StatementFir, Steppable, NormalBraneFir, clone_steppable, fir_to_ref};

#[derive(Debug, thiserror::Error)]
pub enum UbcError {
    #[error("evaluation error: {0}")]
    Eval(String),
}

/// Resolve a FIR to its concrete value. For resolved searches, returns the result.
/// For SFF/SF, strips the wrapper.
pub fn resolve_to_value(fir: &FirRef) -> FirRef {
    match fir.borrow().fir_variant() {
        "Search" => {
            if let Some(result) = fir.borrow().search_result_ref() {
                let st = fir.borrow().state();
                if st == Nyes::Constant || st == Nyes::Independent {
                    return result;
                }
            }
            Rc::clone(fir)
        }
        "StayFullyFoolish" | "StayFoolish" => {
            let fir_clone = fir.borrow().clone_into_fir();
            match fir_clone {
                Fir::StayFullyFoolish(inner) => Rc::clone(&inner.expr),
                Fir::StayFoolish(inner) => Rc::clone(&inner.expr),
                _ => Rc::clone(fir),
            }
        }
        _ => Rc::clone(fir),
    }
}

/// Scope chain: list of (name, FirRef) pairs, most recent first.
pub struct Scope {
    entries: Vec<(String, FirRef)>,
    current_brane: Option<FirRef>,
    current_stmt_idx: Option<usize>,
    block_brane_searches: bool,
    #[allow(dead_code)]
    alarms: Option<Rc<dyn crate::fir::AlarmSink>>,
}

impl std::fmt::Debug for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Scope")
            .field("entries", &self.entries)
            .field("current_brane", &self.current_brane)
            .field("current_stmt_idx", &self.current_stmt_idx)
            .field("block_brane_searches", &self.block_brane_searches)
            .field("alarms", &self.alarms.as_ref().map(|_| "AlarmSink"))
            .finish()
    }
}

impl Default for Scope {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            current_brane: None,
            current_stmt_idx: None,
            block_brane_searches: false,
            alarms: None,
        }
    }
}

impl Clone for Scope {
    fn clone(&self) -> Self {
        Self {
            entries: self.entries.clone(),
            current_brane: self.current_brane.clone(),
            current_stmt_idx: self.current_stmt_idx,
            block_brane_searches: self.block_brane_searches,
            alarms: self.alarms.clone(),
        }
    }
}

impl Scope {
    pub fn new() -> Self { Self::default() }

    pub fn push(&mut self, name: String, fir: FirRef) {
        self.entries.push((name, fir));
    }

    pub fn with_brane(mut self, brane: FirRef, stmt_idx: usize) -> Self {
        self.current_brane = Some(brane);
        self.current_stmt_idx = Some(stmt_idx);
        self
    }

    pub fn search(&self, pattern: &str) -> Option<FirRef> {
        let re = regex::Regex::new(pattern).ok()?;
        for (name, fir) in self.entries.iter().rev() {
            if re.is_match(name) {
                return Some(Rc::clone(fir));
            }
        }
        None
    }

    pub fn block_brane_searches(&self) -> bool { self.block_brane_searches }

    pub fn set_block_brane_searches(&mut self, v: bool) { self.block_brane_searches = v; }

    pub fn current_brane(&self) -> Option<FirRef> {
        self.current_brane.as_ref().map(Rc::clone)
    }

    pub fn current_stmt_idx(&self) -> Option<usize> { self.current_stmt_idx }

    pub fn with_alarms(mut self, sink: Rc<dyn crate::fir::AlarmSink>) -> Self {
        self.alarms = Some(sink);
        self
    }

    pub fn emit(&self, alarm: crate::fir::Alarm) {
        if let Some(ref sink) = self.alarms {
            sink.record(alarm);
        }
    }
}

/// Run a FIR tree to completion with an empty scope.
pub fn run_to_completion(fir: &mut FirRef) -> Result<(), UbcError> {
    run_to_completion_with_scope(fir, &Scope::new())
}

/// Run a FIR tree to completion with a scope chain.
pub fn run_to_completion_with_scope(fir: &mut FirRef, scope: &Scope) -> Result<(), UbcError> {
    let mut max_steps = 100000;
    loop {
        if max_steps == 0 {
            return Err(UbcError::Eval("infinite loop detected".to_string()));
        }
        max_steps -= 1;
        let prev_state = fir.borrow().state();
        if prev_state == Nyes::Constant || prev_state == Nyes::Independent || prev_state == Nyes::Nk {
            break;
        }
        let replacement = step_with_scope(fir, scope)?;
        if let Some(repl) = replacement {
            *fir = fir_to_ref(repl);
        }
        let new_state = fir.borrow().state();
        if prev_state == new_state {
            break;
        }
        if new_state == Nyes::Woconstanic && !has_unresolved_forward_refs(fir) {
            break;
        }
    }
    Ok(())
}

/// Check if any descendant FIR has ECONSTANIC state.
pub fn has_unresolved_forward_refs(fir: &FirRef) -> bool {
    match fir.borrow().fir_variant() {
        "NormalBrane" => {
            let f = fir.borrow().clone_into_fir();
            if let Fir::NormalBrane(inner) = f {
                inner.statements.iter().any(|s| s.state == Nyes::Econstanic || has_unresolved_forward_refs_in_fir(&s.body))
            } else { false }
        }
        "Operator" => {
            let f = fir.borrow().clone_into_fir();
            if let Fir::Operator(inner) = f {
                inner.operands.iter().any(has_unresolved_forward_refs_in_fir)
            } else { false }
        }
        "Search" => {
            let f = fir.borrow().clone_into_fir();
            if let Fir::Search(inner) = f {
                inner.state == Nyes::Econstanic
            } else { false }
        }
        "Concatenation" => {
            let f = fir.borrow().clone_into_fir();
            if let Fir::Concatenation(inner) = f {
                inner.elements.iter().any(has_unresolved_forward_refs_in_fir)
            } else { false }
        }
        "StayFullyFoolish" => false,
        "StayFoolish" => {
            let f = fir.borrow().clone_into_fir();
            if let Fir::StayFoolish(inner) = f {
                has_unresolved_forward_refs_in_fir(&inner.expr)
            } else { false }
        }
        _ => false,
    }
}

fn has_unresolved_forward_refs_in_fir(fir: &FirRef) -> bool {
    has_unresolved_forward_refs(fir)
}

/// Step a FIR with scope. Returns Some(Fir) if the node should be replaced.
pub fn step_with_scope(fir: &FirRef, scope: &Scope) -> Result<Option<Fir>, UbcError> {
    let repl = fir.borrow_mut().step_one(scope)?;
    Ok(repl)
}

/// Clone a FIR, step it to completion with scope, return result as Fir.
pub fn step_boxed(fir: &FirRef, scope: &Scope) -> Result<Fir, UbcError> {
    let inner = fir.borrow().clone_into_fir();
    let mut ref_fir = fir_to_ref(inner);
    run_to_completion_with_scope(&mut ref_fir, scope)?;
    Ok(ref_fir.borrow().clone_into_fir())
}

/// Recompute brane state from statements and update the brane in-place.
pub fn re_step_brane_bodies(brane: &mut NormalBraneFir, scope: &Scope) -> Result<(), UbcError> {
    let statements: Vec<StatementFir> = brane.statements.clone();

    let statements: Vec<StatementFir> = statements.into_iter().map(|s| {
        let body_fir = clone_steppable(&s.body);
        let body = reset_searches(body_fir);
        StatementFir {
            name: s.name,
            body: fir_to_ref(body),
            state: Nyes::Embryonic,
        }
    }).collect();

    let mut local_scope = scope.clone();
    for stmt in &statements {
        if let Some(ref name) = stmt.name {
            local_scope.push(name.clone(), Rc::clone(&stmt.body));
        }
    }

    let mut stepped = Vec::new();
    for (idx, stmt) in statements.iter().enumerate() {
        let scoped = local_scope.clone()
            .with_brane(fir_to_ref(Fir::NormalBrane(Box::new(NormalBraneFir {
                characterizations: brane.characterizations.clone(),
                statements: brane.statements.clone(),
                state: brane.state,
                parent: None,
            alarm: None,
            }))), idx);
        let body = step_boxed(&stmt.body, &scoped)?;
        stepped.push(StatementFir {
            name: stmt.name.clone(),
            state: body.state(),
            body: fir_to_ref(body),
        });
    }

    let brane_state = compute_brane_state(&stepped);
    brane.statements = stepped;
    brane.state = brane_state;
    Ok(())
}

/// Recursively reset all Search FIRs to EMBRYONIC state.
fn reset_searches(fir: Fir) -> Fir {
    match fir {
        Fir::Search(inner) => {
            let mut s = *inner;
            s.result = None;
            s.state = Nyes::Embryonic;
            s.anchor = s.anchor.map(|a| fir_to_ref(reset_searches(clone_steppable(&a))));
            Fir::Search(Box::new(s))
        }
        Fir::Operator(inner) => {
            let mut op = *inner;
            op.operands = op.operands.into_iter().map(|e| {
                fir_to_ref(reset_searches(clone_steppable(&e)))
            }).collect();
            op.state = Nyes::Embryonic;
            Fir::Operator(Box::new(op))
        }
        Fir::NormalBrane(inner) => {
            let mut nb = *inner;
            nb.statements = nb.statements.into_iter().map(|s| {
                StatementFir {
                    name: s.name,
                    body: fir_to_ref(reset_searches(clone_steppable(&s.body))),
                    state: Nyes::Embryonic,
                }
            }).collect();
            nb.state = Nyes::Embryonic;
            Fir::NormalBrane(Box::new(nb))
        }
        Fir::Index(inner) => {
            let mut ix = *inner;
            ix.anchor = ix.anchor.map(|a| fir_to_ref(reset_searches(clone_steppable(&a))));
            ix.state = Nyes::Embryonic;
            Fir::Index(Box::new(ix))
        }
        Fir::HeadTail(inner) => {
            let mut ht = *inner;
            ht.anchor = ht.anchor.map(|a| fir_to_ref(reset_searches(clone_steppable(&a))));
            ht.state = Nyes::Embryonic;
            Fir::HeadTail(Box::new(ht))
        }
        Fir::Concatenation(inner) => {
            let mut c = *inner;
            c.elements = c.elements.into_iter().map(|e| {
                fir_to_ref(reset_searches(clone_steppable(&e)))
            }).collect();
            c.merged = c.merged.map(|m| fir_to_ref(reset_searches(clone_steppable(&m))));
            c.state = Nyes::Embryonic;
            Fir::Concatenation(Box::new(c))
        }
        Fir::StayFullyFoolish(inner) => {
            let mut sff = *inner;
            sff.state = Nyes::Independent;
            Fir::StayFullyFoolish(Box::new(sff))
        }
        Fir::StayFoolish(inner) => {
            let mut sf = *inner;
            sf.expr = fir_to_ref(reset_searches(clone_steppable(&sf.expr)));
            sf.state = Nyes::Embryonic;
            Fir::StayFoolish(Box::new(sf))
        }
        _ => fir,
    }
}

/// Strip SFF/SF wrappers from a FIR, recursively.
fn strip_sf_wrapper(fir: Fir) -> Fir {
    match fir {
        Fir::StayFullyFoolish(inner) => strip_sf_wrapper(clone_steppable(&inner.expr)),
        Fir::StayFoolish(inner) => strip_sf_wrapper(clone_steppable(&inner.expr)),
        other => other,
    }
}

/// Step a FIR but block searches that would resolve to brane targets.
pub fn step_except_brane_searches(fir: &FirRef, scope: &Scope) -> Result<Fir, UbcError> {
    let inner = fir.borrow().clone_into_fir();
    let mut ref_fir = fir_to_ref(inner);
    step_except_brane_searches_ref(&mut ref_fir, scope)?;
    Ok(ref_fir.borrow().clone_into_fir())
}

fn step_except_brane_searches_ref(fir: &mut FirRef, scope: &Scope) -> Result<(), UbcError> {
    let mut max_steps = 10000;
    loop {
        if max_steps == 0 { break; }
        max_steps -= 1;
        let prev = fir.borrow().state();
        if prev.is_constanic() { break; }
        let repl = step_except_brane_one(fir, scope)?;
        if let Some(r) = repl {
            *fir = fir_to_ref(r);
        }
        if fir.borrow().state() == prev { break; }
    }
    Ok(())
}

/// Extract variant data without holding RefCell borrows for mutations.
enum Variant {
    UnanchoredSearch(String),
    AnchoredSearch,
    Operator(String, Vec<Fir>),
    StayFullyFoolish,
    Terminal,
    Other,
}

fn extract_variant(fir: &FirRef) -> Variant {
    let fir_clone = fir.borrow().clone_into_fir();
    match fir_clone {
        Fir::Search(inner) => {
            if inner.anchored {
                Variant::AnchoredSearch
            } else {
                Variant::UnanchoredSearch(inner.pattern)
            }
        }
        Fir::Operator(inner) => {
            Variant::Operator(inner.op, inner.operands.iter().map(|o| clone_steppable(o)).collect())
        }
        Fir::StayFullyFoolish(_) => Variant::StayFullyFoolish,
        Fir::ConstantInt(_) | Fir::Nk(_) => Variant::Terminal,
        _ => Variant::Other,
    }
}

fn step_except_brane_one(fir: &FirRef, scope: &Scope) -> Result<Option<Fir>, UbcError> {
    let variant = extract_variant(fir);

    match variant {
        Variant::UnanchoredSearch(pattern) => {
            match scope.search(&pattern) {
                Some(found) => {
                    let is_brane = found.borrow().fir_variant() == "NormalBrane";
                    if is_brane {
                        fir.borrow_mut().set_state(Nyes::Econstanic);
                    } else {
                        let stripped = strip_sf_wrapper(clone_steppable(&found));
                        let mut found_rc: FirRef = fir_to_ref(stripped);
                        run_to_completion_with_scope(&mut found_rc, scope)?;
                        fir.borrow_mut().set_search_result(Rc::clone(&found_rc));
                        let cs = found_rc.borrow().state();
                        fir.borrow_mut().set_state(
                            if cs == Nyes::Constant || cs == Nyes::Independent {
                                Nyes::Constant
                            } else {
                                Nyes::Woconstanic
                            }
                        );
                    }
                }
                None => fir.borrow_mut().set_state(Nyes::Econstanic),
            }
            Ok(None)
        }
        Variant::AnchoredSearch => {
            let repl = fir.borrow_mut().step_one(scope)?;
            Ok(repl)
        }
        Variant::Operator(op, operands) => {
            let stepped: Vec<Fir> = operands.iter().map(|o| {
                let oref: FirRef = fir_to_ref(o.clone());
                step_except_brane_searches(&oref, scope)
            }).collect::<Result<_, _>>()?;
            let states: Vec<Nyes> = stepped.iter().map(|s| s.state()).collect();

            let all_constant = states.iter().all(|s| *s == Nyes::Constant || *s == Nyes::Independent);
            if all_constant {
                let vals: Vec<i64> = stepped.iter().filter_map(|s| s.as_int()).collect();
                if vals.len() == operands.len() {
                    return Ok(Some(compute_operator(&op, &vals)?));
                }
            }
            // Reconstruct operator with stepped operands
            let new_state = if states.iter().all(|s| s.is_constanic()) {
                Nyes::Woconstanic
            } else {
                fir.borrow().state()
            };
            let new_op = Fir::Operator(Box::new(crate::fir::OperatorFir {
                op,
                operands: stepped.into_iter().map(fir_to_ref).collect(),
                state: new_state,
            }));
            Ok(Some(new_op))
        }
        Variant::StayFullyFoolish => Ok(None),
        Variant::Terminal => Ok(None),
        Variant::Other => {
            if !fir.borrow().state().is_constanic() {
                fir.borrow_mut().set_state(Nyes::Woconstanic);
            }
            Ok(None)
        }
    }
}

/// Constanic clone per FOOP=7.
pub fn constanic_clone(source: &FirRef, permit_nye: bool) -> FirRef {
    if source.borrow().fir_variant() == "StayFullyFoolish" {
        let f = source.borrow().clone_into_fir();
        if let Fir::StayFullyFoolish(inner) = f {
            return constanic_clone(&inner.expr, permit_nye);
        }
    }
    match source.borrow().state() {
        Nyes::Constant | Nyes::Independent | Nyes::Nk => Rc::clone(source),
        Nyes::Econstanic => {
            let r = fir_to_ref(source.borrow().clone_into_fir());
            r.borrow_mut().set_state(Nyes::Embryonic);
            r
        }
        Nyes::Woconstanic => {
            let r = fir_to_ref(source.borrow().clone_into_fir());
            r.borrow_mut().set_state(Nyes::Braning);
            r
        }
        _ => {
            if permit_nye {
                fir_to_ref(source.borrow().clone_into_fir())
            } else {
                fir_to_ref(Fir::Nk(Box::new(crate::fir::NkFir {
                    reason: "constanic_clone called on NYE FIR".to_string(),
                    state: Nyes::Nk,
                    alarm: Some(crate::fir::Alarm {
                        level: crate::fir::AlarmLevel::Panic,
                        code: "INVARIANT-VIOLATED".to_string(),
                        message: "constanic_clone called on NYE FIR".to_string(),
                        source: crate::fir::AlarmSource::Evaluator,
                    }),
                })))
            }
        }
    }
}

/// Follow WOCONSTANIC chain for short-circuiting.
pub fn short_circuit(fir: &FirRef) {
    let mut end_result: Option<FirRef> = None;
    let mut current = fir.borrow().search_result_ref();
    loop {
        let local_rc = match current {
            Some(rc) => rc,
            None => break,
        };
        let state = local_rc.borrow().state();
        if state != Nyes::Woconstanic {
            end_result = Some(local_rc);
            break;
        }
        let next_result = local_rc.borrow().search_result_ref();
        current = next_result;
    }
    if let Some(end) = end_result {
        fir.borrow_mut().set_search_result_direct(Some(Rc::clone(&end)));
    }
}

/// Compute brane state from statements.
pub fn compute_brane_state(statements: &[StatementFir]) -> Nyes {
    if statements.is_empty() {
        return Nyes::Constant;
    }
    if statements.iter().all(|s| matches!(s.state, Nyes::Constant | Nyes::Independent)) {
        Nyes::Constant
    } else if statements.iter().any(|s| s.state == Nyes::Nk) {
        Nyes::Nk
    } else if statements.iter().any(|s| matches!(s.state, Nyes::Econstanic | Nyes::Woconstanic)) {
        Nyes::Woconstanic
    } else {
        Nyes::Braning
    }
}

/// Compute operator result from operand values.
pub fn compute_operator(op: &str, operands: &[i64]) -> Result<Fir, UbcError> {
    let result = match op {
        "+" => {
            if operands.len() == 1 { operands[0] }
            else { operands.iter().copied().sum() }
        }
        "-" | "*" => {
            if operands.len() == 1 {
                if op == "-" { -operands[0] } else { operands[0] }
            } else if operands.len() >= 2 {
                let mut acc = operands[0];
                for &v in &operands[1..] {
                    acc = if op == "-" { acc - v } else { acc * v };
                }
                acc
            } else {
                return Err(UbcError::Eval(format!("op {} needs >=2 operands", op)));
            }
        }
        "/" => {
            if operands.len() == 1 {
                operands[0]
            } else if operands.len() >= 2 {
                let mut acc = operands[0];
                for &v in &operands[1..] {
                    if v == 0 {
                        return Ok(Fir::Nk(Box::new(crate::fir::NkFir {
                            reason: "division by zero".to_string(),
                            state: Nyes::Nk,
                            alarm: Some(crate::fir::Alarm {
                                level: crate::fir::AlarmLevel::Mild,
                                code: "DIV-BY-ZERO".to_string(),
                                message: "Division by zero produces NK".to_string(),
                                source: crate::fir::AlarmSource::Evaluator,
                            }),
                        })));
                    }
                    acc = acc / v;
                }
                acc
            } else {
                return Err(UbcError::Eval(format!("op {} needs >=2 operands", op)));
            }
        }
        _ => return Err(UbcError::Eval(format!("unknown op: {}", op))),
    };
    Ok(Fir::ConstantInt(Box::new(crate::fir::ConstantIntFir {
        value: result,
        state: Nyes::Constant,
    })))
}
