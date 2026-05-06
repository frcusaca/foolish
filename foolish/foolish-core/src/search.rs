use std::cell::RefCell;
use std::rc::Rc;
use crate::fir::{Fir, FirRef};

/// Search within a brane's statements for a name matching the pattern.
/// Returns the first match (writing-order: earliest in brane).
pub fn search_in_brane(brane: &FirRef, pattern: &str) -> Option<FirRef> {
    if let Fir::NormalBrane { statements, .. } = &*brane.borrow() {
        let re = regex::Regex::new(pattern).ok()?;
        for stmt in statements {
            if let Some(ref name) = stmt.name {
                if re.is_match(name) {
                    return Some(Rc::new(RefCell::new(stmt.body.clone())));
                }
            }
        }
    }
    None
}

/// Get statement at index in a brane (positive = from start, negative = from end).
pub fn index_in_brane(brane: &FirRef, offset: i32) -> Option<FirRef> {
    if let Fir::NormalBrane { statements, .. } = &*brane.borrow() {
        let len = statements.len() as i32;
        let idx = if offset < 0 {
            (len + offset).max(0) as usize
        } else {
            offset as usize
        };
        if idx < statements.len() {
            Some(Rc::new(RefCell::new(statements[idx].body.clone())))
        } else {
            None
        }
    } else {
        None
    }
}

/// Get first statement body from a brane.
pub fn head_of_brane(brane: &FirRef) -> Option<FirRef> {
    if let Fir::NormalBrane { statements, .. } = &*brane.borrow() {
        statements.first().map(|s| Rc::new(RefCell::new(s.body.clone())))
    } else {
        None
    }
}

/// Get last statement body from a brane.
pub fn tail_of_brane(brane: &FirRef) -> Option<FirRef> {
    if let Fir::NormalBrane { statements, .. } = &*brane.borrow() {
        statements.last().map(|s| Rc::new(RefCell::new(s.body.clone())))
    } else {
        None
    }
}
