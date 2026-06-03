use crate::fir::{Fir, FirQueryable, StatementSimple};
use std::fmt::Write;

#[derive(Default)]
pub struct Sequencer {
    steps: u64,
}

impl Sequencer {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn steps(&self) -> u64 {
        self.steps
    }
    pub fn format(fir: &Fir) -> String {
        let mut buf = String::new();
        format_fir_q(&mut buf, fir, 0);
        buf.trim_end().to_string()
    }
    pub fn format_with_header(source: &str, fir: &Fir, steps: u64) -> String {
        let body = Self::format(fir);
        format!(
            "INPUT: {}\nPARSED:\n{}\nSTEPS: {}",
            source.trim(),
            body,
            steps
        )
    }
}

/// Human-readable sequencer for any FirQueryable (borrowed).
pub struct HumanizingSequencerRef<'a> {
    fir: &'a dyn FirQueryable,
}

impl<'a> HumanizingSequencerRef<'a> {
    pub fn new(fir: &'a dyn FirQueryable) -> Self {
        Self { fir }
    }

    pub fn format_for_snap_test(&self) -> String {
        let mut buf = String::new();
        format_fir_q(&mut buf, self.fir, 0);
        buf.trim_end().to_string()
    }

    pub fn format_with_indent(&self, indent: usize) -> String {
        let mut buf = String::new();
        format_fir_q(&mut buf, self.fir, indent);
        buf.trim_end().to_string()
    }

    pub fn format_for_repl(&self) -> String {
        self.format_for_snap_test()
    }
}

/// Format a FirQueryable matching the old Steppable::format output style.
fn format_fir_q(buf: &mut String, fir: &dyn FirQueryable, depth: usize) {
    let indent = "  ".repeat(depth);
    let state = fir.hs_state();
    let state_sfx = if state.should_show_nyes() {
        format!(", [{}]", state)
    } else {
        String::new()
    };

    if let Some(value) = fir.hs_constant_int() {
        let _ = writeln!(buf, "{}Int({})", indent, value);
        return;
    }
    if let Some((reason, _alarm)) = fir.hs_nk() {
        let _ = writeln!(buf, "{}??? ({}{})", indent, reason, state_sfx);
        return;
    }
    if let Some((op, operands)) = fir.hs_operator() {
        let _ = writeln!(buf, "{}Operator({}{})", indent, op, state_sfx);
        for operand in &operands {
            format_fir_q(buf, &**operand, depth + 1);
        }
        return;
    }
    if let Some((pattern, direction, anchored, _anchor, target)) = fir.hs_search() {
        let anchor_str = if anchored { "ANCHORED" } else { "UNANCHORED" };
        let _ = writeln!(
            buf,
            "{}Search(pattern='{}', dir={}, {}{})",
            indent, pattern, direction, anchor_str, state_sfx
        );
        if let Some(ref t) = target {
            format_fir_q(buf, &**t, depth + 1);
        }
        return;
    }
    if let Some((offset, anchored, _anchor)) = fir.hs_index() {
        let anchor_str = if anchored { "ANCHORED" } else { "UNANCHORED" };
        let _ = writeln!(
            buf,
            "{}Index(offset={}, {}{})",
            indent, offset, anchor_str, state_sfx
        );
        return;
    }
    if let Some((is_head, anchored, _anchor)) = fir.hs_head_tail() {
        let ht = if is_head { "HEAD" } else { "TAIL" };
        let anchor_str = if anchored { "ANCHORED" } else { "UNANCHORED" };
        let _ = writeln!(
            buf,
            "{}HeadTail({}, {}{})",
            indent, ht, anchor_str, state_sfx
        );
        return;
    }
    if let Some(ref expr) = fir.hs_stay_foolish() {
        let _ = writeln!(buf, "{}StayFoolish({})", indent, state_sfx);
        format_fir_q(buf, &**expr, depth + 1);
        return;
    }
    if let Some(ref expr) = fir.hs_stay_fully_foolish() {
        let _ = writeln!(buf, "{}StayFullyFoolish({})", indent, state_sfx);
        format_fir_q(buf, &**expr, depth + 1);
        return;
    }
    if let Some((elements, merged)) = fir.hs_concatenation() {
        let _ = writeln!(
            buf,
            "{}Concatenation(elements={}{})",
            indent,
            elements.len(),
            state_sfx
        );
        for elem in &elements {
            format_fir_q(buf, &**elem, depth + 1);
        }
        if let Some(ref m) = merged {
            format_fir_q(buf, &**m, depth + 1);
        }
        return;
    }
    if let Some((characterizations, statements)) = fir.hs_brane() {
        let chars = if characterizations.is_empty() {
            String::new()
        } else {
            format!("{}'", characterizations.join(" "))
        };
        let brane_state = if state.should_show_nyes() {
            format!(" [{}]", state)
        } else {
            String::new()
        };
        let _ = writeln!(buf, "{}{}Brane{}", indent, chars, brane_state);
        for stmt in &statements {
            if let Some(ref name) = stmt.name {
                let _ = writeln!(buf, "{}{} = ", "  ".repeat(depth + 1), name);
            }
            format_fir_q(buf, &*stmt.body, depth + 1);
        }
        return;
    }
    let _ = writeln!(buf, "{}Unknown({})", indent, fir.hs_variant());
}

/// Format a StatementSimple inline for compact output (character-based indent).
pub fn format_statement_simple(stmt: &StatementSimple, indent: usize) -> String {
    let value = format_fir_simple_indent(&*stmt.body, indent);
    match &stmt.name {
        Some(name) => format!("{} = {}", name, value),
        None => value,
    }
}

/// Format a FirQueryable inline using the compact style.
pub fn format_fir_simple(fir: &dyn FirQueryable) -> String {
    format_fir_simple_indent(fir, 0)
}

/// Format a FirQueryable inline with character-based indent (for snapshot output).
pub fn format_fir_simple_with_indent(fir: &dyn FirQueryable, indent: usize) -> String {
    format_fir_simple_indent(fir, indent)
}

fn format_fir_simple_indent(fir: &dyn FirQueryable, indent: usize) -> String {
    let state = fir.hs_state();
    let state_sfx = if state.should_show_nyes() {
        format!(", [{}]", state)
    } else {
        String::new()
    };

    if let Some(value) = fir.hs_constant_int() {
        return format!("Int({})", value);
    }
    if let Some((reason, alarm)) = fir.hs_nk() {
        return if let Some(a) = alarm {
            format!("NK({}; {}{})", reason, a, state_sfx)
        } else {
            format!("NK({}{})", reason, state_sfx)
        };
    }
    if let Some((op, operands)) = fir.hs_operator() {
        let child_fmts: Vec<String> = operands
            .iter()
            .map(|o| format_fir_simple_indent(&**o, indent))
            .collect();
        format!(
            "Operator(op='{}', operands=[{}]{})",
            op,
            child_fmts.join(", "),
            state_sfx
        )
    } else if let Some((pattern, direction, anchored, _anchor, target)) = fir.hs_search() {
        let anchor_str = if anchored { "ANCHORED" } else { "UNANCHORED" };
        if let Some(ref t) = target {
            let t_fmt = format_fir_simple_indent(&**t, indent);
            format!(
                "Search(result={}, pattern='{}', direction={}, {}{})",
                t_fmt, pattern, direction, anchor_str, state_sfx
            )
        } else {
            format!(
                "Search(pattern='{}', direction={}, {}{})",
                pattern, direction, anchor_str, state_sfx
            )
        }
    } else if let Some((offset, anchored, _anchor)) = fir.hs_index() {
        let anchor_str = if anchored { "ANCHORED" } else { "UNANCHORED" };
        format!("Index(offset={}, {}{})", offset, anchor_str, state_sfx)
    } else if let Some((is_head, anchored, _anchor)) = fir.hs_head_tail() {
        let ht = if is_head { "HEAD" } else { "TAIL" };
        let anchor_str = if anchored { "ANCHORED" } else { "UNANCHORED" };
        format!("HeadTail({}, {}{})", ht, anchor_str, state_sfx)
    } else if let Some(ref expr) = fir.hs_stay_foolish() {
        let inner = format_fir_simple_indent(&**expr, indent);
        format!("StayFoolish({}{})", inner, state_sfx)
    } else if let Some(ref expr) = fir.hs_stay_fully_foolish() {
        let inner = format_fir_simple_indent(&**expr, indent);
        format!("StayFullyFoolish({}{})", inner, state_sfx)
    } else if let Some((elements, merged)) = fir.hs_concatenation() {
        if let Some(ref m) = merged {
            let m_fmt = format_fir_simple_indent(&**m, indent);
            format!(
                "Concatenation(elements={}, merged={}{})",
                elements.len(),
                m_fmt,
                state_sfx
            )
        } else {
            format!("Concatenation(elements={}{})", elements.len(), state_sfx)
        }
    } else if let Some((characterizations, statements)) = fir.hs_brane() {
        let chars = if characterizations.is_empty() {
            String::new()
        } else {
            format!("{}'", characterizations.join(" "))
        };
        let state_display = if state.should_show_nyes() {
            format!("[{}]", state)
        } else {
            String::new()
        };
        if statements.is_empty() {
            format!("{}Brane{}{{}}", chars, state_display)
        } else if statements.len() == 1 {
            let stmt_fmt = format_statement_simple(&statements[0], indent + 2);
            format!(
                "{}Brane{}{{\n{}\n{}}}",
                chars,
                state_display,
                format!("{}{}", " ".repeat(indent + 2), stmt_fmt),
                " ".repeat(indent + 2)
            )
        } else {
            let inner_pad = " ".repeat(indent + 2);
            let stmts: Vec<String> = statements
                .iter()
                .map(|s| format!("{}{};", inner_pad, format_statement_simple(s, indent + 2)))
                .collect();
            format!(
                "{}Brane{}{{\n{}\n{}}}",
                chars,
                state_display,
                stmts.join("\n"),
                inner_pad
            )
        }
    } else {
        format!("Unknown({})", fir.hs_variant())
    }
}
