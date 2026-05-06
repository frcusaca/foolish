use std::cell::RefCell;
use std::rc::Rc;

use crate::fir::{Fir, FirRef, Nyes, StatementFir};
use crate::search;

#[derive(Debug, thiserror::Error)]
pub enum UbcError {
    #[error("evaluation error: {0}")]
    Eval(String),
}

/// Scope chain: list of (name, FirRef) pairs, most recent first.
/// Unanchored searches search backwards through this chain.
#[derive(Debug, Default, Clone)]
pub struct Scope {
    entries: Vec<(String, FirRef)>,
}

impl Scope {
    pub fn new() -> Self { Self::default() }

    pub fn push(&mut self, name: String, fir: FirRef) {
        self.entries.push((name, fir));
    }

    /// Search backwards for a name matching the regex pattern.
    /// Returns the first (most recent) match.
    pub fn search(&self, pattern: &str) -> Option<FirRef> {
        let re = regex::Regex::new(pattern).ok()?;
        for (name, fir) in self.entries.iter().rev() {
            if re.is_match(name) {
                return Some(Rc::clone(fir));
            }
        }
        None
    }
}

/// Run a FIR tree to completion with an empty scope.
pub fn run_to_completion(fir: &mut FirRef) -> Result<(), UbcError> {
    run_to_completion_with_scope(fir, &Scope::new())
}

/// Run a FIR tree to completion with a scope chain.
pub fn run_to_completion_with_scope(fir: &mut FirRef, scope: &Scope) -> Result<(), UbcError> {
    let mut max_steps = 100000;
    while !fir.borrow().state().is_constanic() && fir.borrow().state() != Nyes::Nk {
        if max_steps == 0 {
            return Err(UbcError::Eval("infinite loop detected".to_string()));
        }
        max_steps -= 1;
        let replacement = step_with_scope(fir, scope)?;
        if let Some(repl) = replacement {
            *fir = Rc::new(RefCell::new(repl));
        }
    }
    Ok(())
}

/// Step a FIR with scope. Returns Some(Fir) if the node should be replaced entirely.
pub fn step_with_scope(fir: &FirRef, scope: &Scope) -> Result<Option<Fir>, UbcError> {
    macro_rules! step_if {
        ($variant:pat, $fn:ident) => {
            if matches!(&*fir.borrow(), $variant) {
                return $fn(fir, scope);
            }
        };
    }
    step_if!(Fir::ConstantInt { .. }, step_noop);
    step_if!(Fir::Nk { .. }, step_noop);
    step_if!(Fir::NormalBrane { .. }, step_brane);
    step_if!(Fir::BinaryOp { .. }, step_binary_op);
    step_if!(Fir::UnaryOp { .. }, step_unary_op);
    step_if!(Fir::Search { .. }, step_search);
    step_if!(Fir::Index { .. }, step_index);
    step_if!(Fir::HeadTail { .. }, step_head_tail);
    step_if!(Fir::Concatenation { .. }, step_concatenation);
    Ok(None)
}

fn step_noop(_fir: &FirRef) -> Result<Option<Fir>, UbcError> { Ok(None) }

/// Clone a Box<Fir>, step it to completion, return result
fn step_boxed(child: &Box<Fir>) -> Result<Fir, UbcError> {
    let inner = (**child).clone();
    let mut ref_fir = Rc::new(RefCell::new(inner));
    run_to_completion(&mut ref_fir)?;
    Ok(ref_fir.borrow().clone())
}

/// Clone an Option<Box<Fir>>, step it to completion, return result
fn step_opt_boxed(child: &Option<Box<Fir>>) -> Result<Option<Fir>, UbcError> {
    match child {
        Some(boxed) => Ok(Some(step_boxed(boxed)?)),
        None => Ok(None),
    }
}

fn step_brane(fir: &FirRef) -> Result<Option<Fir>, UbcError> {
    let state = fir.borrow().state();
    match state {
        Nyes::Prembrionic => { fir.borrow_mut().set_state(Nyes::Embryonic); }
        Nyes::Embryonic => { fir.borrow_mut().set_state(Nyes::Braning); }
        Nyes::Braning => {
            let statements = {
                if let Fir::NormalBrane { statements, .. } = &*fir.borrow() {
                    Some(statements.clone())
                } else { None }
            };
            if let Some(stmts) = statements {
                let mut stepped = Vec::new();
                for stmt in stmts {
                    let body = step_boxed(&Box::new(stmt.body.clone()))?;
                    stepped.push(StatementFir {
                        name: stmt.name.clone(),
                        state: body.state(),
                        body,
                    });
                }
                let brane_state = compute_brane_state(&stepped);
                fir.borrow_mut().normal_brane_statements(stepped);
                fir.borrow_mut().set_state(brane_state);
            }
        }
        _ => {} // terminal
    }
    Ok(None)
}

fn step_binary_op(fir: &FirRef) -> Result<Option<Fir>, UbcError> {
    let op = {
        if let Fir::BinaryOp { op, .. } = &*fir.borrow() {
            op.clone()
        } else {
            return Ok(None);
        }
    };

    // Step left operand - clone, step, write back
    let left_fir = {
        if let Fir::BinaryOp { left, .. } = &*fir.borrow() {
            (**left).clone()
        } else { return Ok(None); }
    };
    let left_stepped = step_boxed(&Box::new(left_fir))?;

    // Step right operand
    let right_fir = {
        if let Fir::BinaryOp { right, .. } = &*fir.borrow() {
            (**right).clone()
        } else { return Ok(None); }
    };
    let right_stepped = step_boxed(&Box::new(right_fir))?;

    // Write back
    fir.borrow_mut().set_binary_operands(left_stepped, right_stepped);

    // Check results
    let ls = fir.borrow().left_state();
    let rs = fir.borrow().right_state();

    if ls == Nyes::Nk || rs == Nyes::Nk {
        fir.borrow_mut().set_state(Nyes::Nk);
        return Ok(None);
    }

    if (ls == Nyes::Constant || ls == Nyes::Independent)
        && (rs == Nyes::Constant || rs == Nyes::Independent)
    {
        if let Some((l, r)) = fir.borrow().binary_values() {
            return Ok(Some(compute_binary(&op, l, r)?));
        }
    }

    if ls.is_constanic() && rs.is_constanic() {
        fir.borrow_mut().set_state(Nyes::Woconstanic);
    }
    Ok(None)
}

fn step_unary_op(fir: &FirRef) -> Result<Option<Fir>, UbcError> {
    let op = {
        if let Fir::UnaryOp { op, .. } = &*fir.borrow() {
            op.clone()
        } else {
            return Ok(None);
        }
    };

    // Step operand - clone, step, write back
    let expr_fir = {
        if let Fir::UnaryOp { expr, .. } = &*fir.borrow() {
            (**expr).clone()
        } else { return Ok(None); }
    };
    let expr_stepped = step_boxed(&Box::new(expr_fir))?;
    fir.borrow_mut().set_unary_expr(expr_stepped);

    let es = fir.borrow().expr_state();

    match es {
        Nyes::Nk => { fir.borrow_mut().set_state(Nyes::Nk); Ok(None) }
        Nyes::Constant | Nyes::Independent => {
            if let Some(val) = fir.borrow().unary_value() {
                Ok(Some(compute_unary(&op, val)?))
            } else {
                Ok(None)
            }
        }
        _ => { fir.borrow_mut().set_state(Nyes::Woconstanic); Ok(None) }
    }
}

fn step_search(fir: &FirRef) -> Result<Option<Fir>, UbcError> {
    if fir.borrow().state().is_constanic() {
        return Ok(None);
    }
    if fir.borrow().search_anchored() {
        step_search_anchored(fir)
    } else {
        fir.borrow_mut().set_state(Nyes::Econstanic);
        Ok(None)
    }
}

fn step_search_anchored(fir: &FirRef) -> Result<Option<Fir>, UbcError> {
    let anchor = fir.borrow().search_anchor_ref();
    if let Some(mut anchor) = anchor {
        run_to_completion(&mut anchor)?;
        let anchor_state = anchor.borrow().state();
        match anchor_state {
            Nyes::Nk => fir.borrow_mut().set_state(Nyes::Nk),
            Nyes::Constant | Nyes::Independent => {
                let pattern = fir.borrow().search_pattern();
                match search::search_in_brane(&anchor, &pattern) {
                    None => fir.borrow_mut().set_state(Nyes::Nk),
                    Some(found) => {
                        let cloned = constanic_clone(&found);
                        fir.borrow_mut().set_search_target(cloned.clone());
                        let cs = cloned.borrow().state();
                        if cs == Nyes::Constant || cs == Nyes::Independent {
                            fir.borrow_mut().set_state(Nyes::Constant);
                        } else if cs.is_constanic() {
                            short_circuit(fir);
                            fir.borrow_mut().set_state(Nyes::Woconstanic);
                        }
                    }
                }
            }
            _ => fir.borrow_mut().set_state(Nyes::Nk),
        }
    } else {
        fir.borrow_mut().set_state(Nyes::Econstanic);
    }
    Ok(None)
}

fn step_index(fir: &FirRef) -> Result<Option<Fir>, UbcError> {
    let anchored = fir.borrow().index_anchored();
    if anchored {
        let anchor = fir.borrow().index_anchor_ref();
        if let Some(mut anchor) = anchor {
            run_to_completion(&mut anchor)?;
            let offset = fir.borrow().index_offset();
            match anchor.borrow().state() {
                Nyes::Nk => fir.borrow_mut().set_state(Nyes::Nk),
                Nyes::Constant | Nyes::Independent => {
                    match search::index_in_brane(&anchor, offset) {
                        None => fir.borrow_mut().set_state(Nyes::Nk),
                        Some(found) => {
                            let cloned = constanic_clone(&found);
                            return Ok(Some(cloned.borrow().clone()));
                        }
                    }
                }
                _ => fir.borrow_mut().set_state(Nyes::Nk),
            }
        }
    } else {
        fir.borrow_mut().set_state(Nyes::Econstanic);
    }
    Ok(None)
}

fn step_head_tail(fir: &FirRef) -> Result<Option<Fir>, UbcError> {
    let is_head = fir.borrow().headtail_is_head();
    let anchor = fir.borrow().headtail_anchor_ref();
    if let Some(mut anchor) = anchor {
        run_to_completion(&mut anchor)?;
        match anchor.borrow().state() {
            Nyes::Nk => fir.borrow_mut().set_state(Nyes::Nk),
            Nyes::Constant | Nyes::Independent => {
                let found = if is_head {
                    search::head_of_brane(&anchor)
                } else {
                    search::tail_of_brane(&anchor)
                };
                match found {
                    None => fir.borrow_mut().set_state(Nyes::Nk),
                    Some(f) => {
                        let cloned = constanic_clone(&f);
                        return Ok(Some(cloned.borrow().clone()));
                    }
                }
            }
            _ => fir.borrow_mut().set_state(Nyes::Nk),
        }
    }
    Ok(None)
}

fn step_concatenation(fir: &FirRef) -> Result<Option<Fir>, UbcError> {
    let elements = fir.borrow().concat_elements();

    // Step each element to completion
    let stepped: Vec<Fir> = elements
        .iter()
        .map(|e| step_boxed(&Box::new(e.clone())))
        .collect::<Result<_, _>>()?;

    // NK propagation
    if stepped.iter().any(|e| e.state() == Nyes::Nk) {
        fir.borrow_mut().set_state(Nyes::Nk);
        return Ok(None);
    }

    // Build merged brane
    let merged_statements: Vec<StatementFir> = stepped
        .iter()
        .flat_map(|elem| {
            let elem_ref = Rc::new(RefCell::new(elem.clone()));
            match &*elem_ref.borrow() {
                Fir::NormalBrane { statements, .. } => {
                    statements.iter().map(|stmt| {
                        let body_ref = Rc::new(RefCell::new(stmt.body.clone()));
                        let cloned = constanic_clone(&body_ref);
                        StatementFir {
                            name: stmt.name.clone(),
                            body: cloned.borrow().clone(),
                            state: Nyes::Embryonic,
                        }
                    }).collect::<Vec<_>>()
                }
                _ => vec![],
            }
        })
        .collect();

    let merged = Fir::NormalBrane {
        characterizations: vec![],
        statements: merged_statements,
        state: Nyes::Embryonic,
    };
    fir.borrow_mut().set_concat_merged(Rc::new(RefCell::new(merged)));
    fir.borrow_mut().set_state(Nyes::Embryonic);
    Ok(None)
}

/// Constanic clone per FOOP=7
pub fn constanic_clone(source: &FirRef) -> FirRef {
    match source.borrow().state() {
        Nyes::Constant | Nyes::Independent | Nyes::Nk => Rc::clone(source),
        Nyes::Econstanic => {
            let r = Rc::new(RefCell::new(source.borrow().clone()));
            r.borrow_mut().set_state(Nyes::Embryonic);
            r
        }
        Nyes::Woconstanic => {
            let r = Rc::new(RefCell::new(source.borrow().clone()));
            r.borrow_mut().set_state(Nyes::Braning);
            r
        }
        _ => panic!("constanic_clone called on nye FIR"),
    }
}

fn short_circuit(fir: &FirRef) {
    // Follow WOCONSTANIC chain
    let mut end_target: Option<Fir> = None;
    let mut current = fir.borrow().search_target_ref();
    loop {
        let local_rc = match current {
            Some(rc) => rc,
            None => break,
        };
        let state = local_rc.borrow().state();
        if state != Nyes::Woconstanic {
            end_target = Some(local_rc.borrow().clone());
            break;
        }
        let next_target = local_rc.borrow().search_target_ref();
        current = next_target;
    }
    if let Some(end) = end_target {
        fir.borrow_mut().set_search_target_direct(Some(Rc::new(RefCell::new(end))));
    }
}

fn compute_brane_state(statements: &[StatementFir]) -> Nyes {
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

fn compute_binary(op: &str, left: i64, right: i64) -> Result<Fir, UbcError> {
    let result = match op {
        "+" => left + right,
        "-" => left - right,
        "*" => left * right,
        "/" => {
            if right == 0 {
                return Ok(Fir::Nk {
                    reason: "division by zero".to_string(),
                    state: Nyes::Nk,
                });
            }
            left / right
        }
        _ => return Err(UbcError::Eval(format!("unknown op: {}", op))),
    };
    Ok(Fir::ConstantInt { value: result, state: Nyes::Constant })
}

fn compute_unary(op: &str, val: i64) -> Result<Fir, UbcError> {
    let result = match op {
        "-" => -val,
        "+" => val,
        _ => return Err(UbcError::Eval(format!("unknown unary op: {}", op))),
    };
    Ok(Fir::ConstantInt { value: result, state: Nyes::Constant })
}
