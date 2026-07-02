use std::collections::HashMap;
use std::rc::Rc;

use foolish_core::{Fir, FirRef, Nyes, fir_to_ref, clone_steppable, ubc, Compiler};

use crate::luid::{Luid, LuidGenerator};
use crate::messages::UbcbMessage;
use crate::channel::MessageChannel;

/// A single statement result from evaluation.
/// A single statement result from evaluation.
#[derive(Debug)]
pub struct StatementResult {
    /// Optional name (ordinate) of the statement.
    pub name: Option<String>,
    /// The computed FIR after evaluation completes.
    pub fir: FirRef,
    /// The NYES state of the FIR.
    pub state: Nyes,
}

pub struct EvaluationResult {
    pub statements: Vec<StatementResult>,
    pub brane_state: Nyes,
}

pub struct UbcbEngine {
    firs: HashMap<Luid, FirRef>,
    channels: MessageChannel,
    next_luid: LuidGenerator,
    local_scope: HashMap<String, Luid>,
}

impl UbcbEngine {
    pub fn new() -> Self {
        Self {
            firs: HashMap::new(),
            channels: MessageChannel::new(),
            next_luid: LuidGenerator::new(),
            local_scope: HashMap::new(),
        }
    }

    pub fn register(&mut self, fir: FirRef) -> Luid {
        let luid = self.next_luid.next();
        self.firs.insert(luid, fir);
        luid
    }

    pub fn evaluate(&mut self, source: &str) -> Result<EvaluationResult, EvalError> {
        self.firs.clear();
        self.channels = MessageChannel::new();
        self.local_scope.clear();

        let firs = Compiler::compile(source).map_err(|e| EvalError::Compile(e.to_string()))?;
        if firs.is_empty() {
            return Ok(EvaluationResult {
                statements: Vec::new(),
                brane_state: Nyes::Constant,
            });
        }

        match &firs[0] {
            Fir::NormalBrane(brane) => self.evaluate_brane(brane),
            other => {
                let fir_ref: FirRef = fir_to_ref(clone_steppable(&fir_to_ref(other.clone())));
                self.evaluate_single(fir_ref)
            }
        }
    }

    fn evaluate_single(&mut self, fir: FirRef) -> Result<EvaluationResult, EvalError> {
        let luid = self.register(fir);
        self.run_loop();
        let state = self.firs[&luid].borrow().state();
        let fir_ref = self.firs[&luid].clone();
        Ok(EvaluationResult {
            statements: vec![StatementResult { name: None, fir: fir_ref, state }],
            brane_state: state,
        })
    }

    fn evaluate_brane(&mut self, brane: &foolish_core::fir::NormalBraneFir) -> Result<EvaluationResult, EvalError> {
        let mut stmt_names: Vec<Option<String>> = Vec::new();
        let mut stmt_luids: Vec<Luid> = Vec::new();

        for stmt in brane.statements() {
            let body = fir_to_ref(clone_steppable(stmt.body()));
            let luid = self.register(body);
            stmt_luids.push(luid);
            stmt_names.push(stmt.name().clone());
            if let Some(name_str) = stmt.name() {
                self.local_scope.insert(name_str.clone(), luid);
            }
        }

        self.run_loop();

        let statements: Vec<StatementResult> = stmt_luids.iter()
            .zip(stmt_names.into_iter())
            .map(|(luid_ref, name)| {
                let luid = *luid_ref;
                let fir_ref = self.firs[&luid].clone();
                let state = fir_ref.borrow().state();
                StatementResult { name, fir: fir_ref, state }
            })
            .collect();

        let brane_state = compute_brane_state(&statements);

        Ok(EvaluationResult {
            brane_state,
            statements,
        })
    }

    fn run_loop(&mut self) {
        let max_steps = 100000;
        for _ in 0..max_steps {
            let changes = self.step_all();
            if changes.is_empty() {
                break;
            }
            if self.all_terminal() {
                break;
            }
        }
    }

    fn step_all(&mut self) -> Vec<(Luid, Nyes, Nyes)> {
        let mut changes = Vec::new();

        // Phase 1: Embryonic — local resolution within brane scope
        let luids: Vec<Luid> = self.firs.keys().copied().collect();
        for luid in luids {
            let fir = self.firs[&luid].clone();
            let old_state = fir.borrow().state();

            if old_state == Nyes::Embryonic {
                Self::resolve_search_local(&fir, &self.local_scope, &self.firs);
                Self::resolve_search_operands_local(&fir, &self.local_scope, &self.firs);
            }

            let new_state = fir.borrow().state();
            if new_state != old_state {
                changes.push((luid, old_state, new_state));
            }
        }

        // Phase 2: Braning — parent resolution for unresolved searches
        let luids: Vec<Luid> = self.firs.keys().copied().collect();
        for luid in luids {
            let fir = self.firs[&luid].clone();
            let old_state = fir.borrow().state();

            if old_state == Nyes::Braning {
                Self::resolve_search_parent(&fir, &self.local_scope, &self.firs);
                Self::resolve_search_operands_parent(&fir, &self.local_scope, &self.firs);
            }

            let new_state = fir.borrow().state();
            if new_state != old_state {
                changes.push((luid, old_state, new_state));
            }
        }

        // Phase 3: Operators
        let luids: Vec<Luid> = self.firs.keys().copied().collect();
        for luid in luids {
            let old_state = self.firs[&luid].borrow().state();
            self.compute_operator(luid);
            let new_state = self.firs[&luid].borrow().state();
            if new_state != old_state {
                changes.push((luid, old_state, new_state));
            }
        }

        for (luid, old_s, new_s) in &changes {
            self.channels.send(*luid, UbcbMessage::StateChange {
                source_luid: *luid,
                old_state: *old_s,
                new_state: *new_s,
            });
        }

        changes
    }

    fn resolve_search_local(fir: &FirRef, local_scope: &HashMap<String, Luid>, firs: &HashMap<Luid, FirRef>) {
        if fir.borrow().search_anchored() {
            return;
        }
        let pattern = match fir.borrow().search_pattern() {
            Some(p) => p,
            None => return,
        };
        let name = strip_anchors(&pattern);

        if let Some(&target_luid) = local_scope.get(name) {
            let result = firs[&target_luid].clone();
            let target_state = result.borrow().state();
            if target_state == Nyes::Constant || target_state == Nyes::Independent {
                fir.borrow_mut().set_search_result(result);
                fir.borrow_mut().set_state(target_state);
            } else if target_state.is_constanic() {
                fir.borrow_mut().set_search_result(result);
                fir.borrow_mut().set_state(Nyes::Woconstanic);
            }
        } else {
            fir.borrow_mut().set_state(Nyes::Econstanic);
        }
    }

    fn resolve_search_operands_local(fir: &FirRef, local_scope: &HashMap<String, Luid>, firs: &HashMap<Luid, FirRef>) {
        let children: Vec<FirRef> = {
            let mut guard = fir.borrow_mut();
            guard.children_mut().into_iter().map(|c| Rc::clone(c)).collect()
        };

        for child in children {
            if child.borrow().fir_variant() == "Search" && !child.borrow().search_anchored() {
                Self::resolve_search_local(&child, local_scope, firs);
            }
        }
    }

    fn resolve_search_parent(fir: &FirRef, local_scope: &HashMap<String, Luid>, firs: &HashMap<Luid, FirRef>) {
        if fir.borrow().search_anchored() {
            return;
        }
        let pattern = match fir.borrow().search_pattern() {
            Some(p) => p,
            None => return,
        };
        let name = strip_anchors(&pattern);

        // Local resolution
        if let Some(&target_luid) = local_scope.get(name) {
            let result = firs[&target_luid].clone();
            let target_state = result.borrow().state();
            if target_state == Nyes::Constant || target_state == Nyes::Independent {
                fir.borrow_mut().set_search_result(result);
                fir.borrow_mut().set_state(target_state);
            } else if target_state.is_constanic() {
                fir.borrow_mut().set_search_result(result);
                fir.borrow_mut().set_state(Nyes::Woconstanic);
            }
        }
        // Parent resolution
        else if let Some(parent_ref) = fir.borrow().search_parent_ref() {
            // Search in parent brane's scope via trait method
            let parent_stmts = parent_ref.borrow().normal_brane_statements();
            let mut found = None;
            for stmt in parent_stmts {
                if let Some(stmt_name) = stmt.name() {
                    if strip_anchors(&format!("^{}$", stmt_name)) == name {
                        found = Some(stmt.body().clone());
                        break;
                    }
                }
            }
            if let Some(result) = found {
                let target_state = result.borrow().state();
                if target_state == Nyes::Constant || target_state == Nyes::Independent {
                    fir.borrow_mut().set_search_result(result);
                    fir.borrow_mut().set_state(target_state);
                } else if target_state.is_constanic() {
                    fir.borrow_mut().set_search_result(result);
                    fir.borrow_mut().set_state(Nyes::Woconstanic);
                }
            }
        }
    }

    fn resolve_search_operands_parent(fir: &FirRef, local_scope: &HashMap<String, Luid>, firs: &HashMap<Luid, FirRef>) {
        let children: Vec<FirRef> = {
            let mut guard = fir.borrow_mut();
            guard.children_mut().into_iter().map(|c| Rc::clone(c)).collect()
        };

        for child in children {
            if child.borrow().fir_variant() == "Search" && !child.borrow().search_anchored() {
                Self::resolve_search_parent(&child, local_scope, firs);
            }
        }
    }

    fn compute_operator(&mut self, luid: Luid) {
        let fir_clone = clone_steppable(&self.firs[&luid]);

        let (op_name, operand_refs): (String, Vec<FirRef>) = match &fir_clone {
            Fir::Operator(op) => (op.op_name().to_string(), op.operands().iter().map(Rc::clone).collect()),
            _ => return,
        };

        let operand_vals: Vec<i64> = operand_refs.iter().filter_map(|o| get_value(o)).collect();
        if operand_vals.len() != operand_refs.len() {
            return;
        }

        match ubc::compute_operator(&op_name, &operand_vals) {
            Ok(result) => {
                self.firs.insert(luid, fir_to_ref(result));
            }
            Err(_) => {
                self.firs.insert(luid, fir_to_ref(Fir::Nk(Box::new(foolish_core::fir::NkFir::with_reason(
                    format!("operator error: {}", op_name),
                )))));
            }
        }
    }

    fn all_terminal(&self) -> bool {
        self.firs.values().all(|f| f.borrow().state().is_constanic())
    }
}

fn strip_anchors(pattern: &str) -> &str {
    pattern.strip_prefix('^').and_then(|s| s.strip_suffix('$')).unwrap_or(pattern)
}

fn get_value(operand: &FirRef) -> Option<i64> {
    let mut current = Rc::clone(operand);
    let mut depth = 0;
    loop {
        if depth > 100 { return None; }
        depth += 1;
        let variant = current.borrow().fir_variant();
        match variant {
            "Search" => {
                let result = current.borrow().search_result_ref()?;
                let st = current.borrow().state();
                if st == Nyes::Constant || st == Nyes::Independent {
                    current = result;
                    continue;
                }
                return None;
            }
            "StayFullyFoolish" | "StayFoolish" => {
                current = ubc::resolve_to_value(&current);
                continue;
            }
            _ => return current.borrow().as_int(),
        }
    }
}

/// Compute the aggregate brane state from statement results.
fn compute_brane_state(statements: &[StatementResult]) -> Nyes {
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

#[derive(Debug, thiserror::Error)]
pub enum EvalError {
    #[error("compile error: {0}")]
    Compile(String),
}
