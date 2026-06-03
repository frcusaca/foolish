use crate::fir::{FirRef, SearchDirection};
use std::rc::Rc;

/// Search within a brane's statements for a name matching the pattern.
/// Backward direction (default): last match wins (most recent binding).
/// Forward direction: first match wins (earliest in brane).
pub fn search_in_brane(
    brane: &FirRef,
    pattern: &str,
    direction: SearchDirection,
) -> Option<FirRef> {
    let statements = brane.borrow().as_brane_statements()?;
    let re = regex::Regex::new(pattern).ok()?;
    match direction {
        SearchDirection::Backward => {
            for stmt in statements.iter().rev() {
                if let Some(ref name) = stmt.name {
                    if re.is_match(name) {
                        return Some(Rc::clone(&stmt.body));
                    }
                }
            }
        }
        SearchDirection::Forward => {
            for stmt in &statements {
                if let Some(ref name) = stmt.name {
                    if re.is_match(name) {
                        return Some(Rc::clone(&stmt.body));
                    }
                }
            }
        }
    }
    None
}

/// Get statement at index in a brane (positive = from start, negative = from end).
pub fn index_in_brane(brane: &FirRef, offset: i32) -> Option<FirRef> {
    let len = brane.borrow().brane_statement_count() as i32;
    let idx = if offset < 0 {
        let adjusted = len + offset;
        if adjusted < 0 {
            return None;
        }
        adjusted as usize
    } else {
        offset as usize
    };
    let statements = brane.borrow().as_brane_statements()?;
    statements.get(idx).map(|s| Rc::clone(&s.body))
}

/// Get first statement body from a brane.
pub fn head_of_brane(brane: &FirRef) -> Option<FirRef> {
    brane.borrow().brane_statement_at(0)
}

/// Get last statement body from a brane.
pub fn tail_of_brane(brane: &FirRef) -> Option<FirRef> {
    let len = brane.borrow().brane_statement_count();
    if len > 0 {
        brane.borrow().brane_statement_at(len - 1)
    } else {
        None
    }
}
