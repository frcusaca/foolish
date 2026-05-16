use crate::fir::{Fir, Steppable, SequenceableFir, SequenceableStatement, Nyes, SearchDirection};

/// Format a FIR tree as human-readable output (for approval tests).
#[derive(Default)]
pub struct Sequencer {
    steps: u64,
}

impl Sequencer {
    pub fn new() -> Self { Self::default() }

    pub fn steps(&self) -> u64 { self.steps }

    pub fn format(fir: &Fir) -> String {
        let mut buf = String::new();
        let _ = format_fir(fir, &mut buf, 0);
        buf.trim_end().to_string()
    }

    pub fn format_with_header(source: &str, fir: &Fir, steps: u64) -> String {
        let body = Self::format(fir);
        format!("INPUT: {}\nPARSED:\n{}\nSTEPS: {}", source.trim(), body, steps)
    }
}


fn format_fir(fir: &dyn Steppable, buf: &mut String, depth: usize) -> std::fmt::Result {
    fir.format(buf, depth)
}

/// Human-readable sequencer for SequenceableFir.
/// Produces formatted output for snapshot tests and REPL.
pub struct HumanizingSequencer {
    fir: SequenceableFir,
}

impl HumanizingSequencer {
    pub fn new(fir: SequenceableFir) -> Self {
        Self { fir }
    }

    pub fn fir(&self) -> &SequenceableFir { &self.fir }

    /// Format for snapshot test approval (deterministic, state-aware).
    pub fn format_for_snap_test(&self, indent: usize) -> String {
        self.format_fir(&self.fir, indent, true)
    }

    /// Format for REPL display (human-friendly, may omit internal state).
    pub fn format_for_repl(&self, indent: usize) -> String {
        self.format_fir(&self.fir, indent, false)
    }

    fn format_fir(&self, fir: &SequenceableFir, indent: usize, _with_states: bool) -> String {
        match fir {
            SequenceableFir::ConstantInt { value, .. } => {
                format!("Int({})", value)
            }
            SequenceableFir::Nk { reason, alarm, .. } => {
                if let Some(a) = alarm {
                    format!("NK({}; {})", reason, a)
                } else {
                    format!("NK({})", reason)
                }
            }
            SequenceableFir::Operator { op, operands, .. } => {
                let child_fmts: Vec<String> = operands.iter()
                    .map(|o| Self::new(SequenceableFir::clone(o)).format_for_snap_test(indent + 2))
                    .collect();
                format!("Operator(op='{}', operands=[{}])", op, child_fmts.join(", "))
            }
            SequenceableFir::Search { pattern, direction, anchored, anchor, target, .. } => {
                let anchor_str = if *anchored { "ANCHORED" } else { "FREE" };
                let anchor_part = match anchor {
                    Some(a) => format!(", anchor=\"{}\"", self.format_anchor(&**a)),
                    None => String::new(),
                };
                if let Some(t) = target {
                    let t_fmt = Self::new(SequenceableFir::clone(t.as_ref())).format_for_snap_test(indent + 2);
                    format!("Search(result={}, pattern='{}', direction={}, {}{})",
                        t_fmt, pattern, direction, anchor_str, anchor_part)
                } else {
                    format!("Search(pattern='{}', direction={}, {}{})",
                        pattern, direction, anchor_str, anchor_part)
                }
            }
            SequenceableFir::Index { offset, anchored, .. } => {
                let anchor_str = if *anchored { "ANCHORED" } else { "FREE" };
                format!("Index(offset={}, {})", offset, anchor_str)
            }
            SequenceableFir::HeadTail { is_head, anchored, .. } => {
                let ht = if *is_head { "HEAD" } else { "TAIL" };
                let anchor_str = if *anchored { "ANCHORED" } else { "FREE" };
                format!("HeadTail({}, {})", ht, anchor_str)
            }
            SequenceableFir::StayFoolish { expr, .. } => {
                let inner = Self::new(SequenceableFir::clone(expr.as_ref())).format_for_snap_test(indent + 2);
                format!("StayFoolish({})", inner)
            }
            SequenceableFir::StayFullyFoolish { expr, .. } => {
                let inner = Self::new(SequenceableFir::clone(expr.as_ref())).format_for_snap_test(indent + 2);
                format!("StayFullyFoolish({})", inner)
            }
            SequenceableFir::Concatenation { elements, merged, .. } => {
                if let Some(m) = merged {
                    let m_fmt = Self::new(SequenceableFir::clone(m.as_ref())).format_for_snap_test(indent + 2);
                    format!("Concatenation(elements={}, merged={})", elements.len(), m_fmt)
                } else {
                    format!("Concatenation(elements={})", elements.len())
                }
            }
            SequenceableFir::NormalBrane { characterizations, statements, .. } => {
                let chars = if characterizations.is_empty() {
                    String::new()
                } else {
                    format!("{}'", characterizations.join(" "))
                };
                if statements.is_empty() {
                    format!("{}Brane{{}}", chars)
                } else if statements.len() == 1 {
                    let stmt_fmt = self.format_statement(&statements[0], indent + 2);
                    format!("{}Brane{{{}}}", chars, stmt_fmt)
                } else {
                    let stmts: Vec<String> = statements.iter()
                        .map(|s| self.format_statement(s, indent + 2))
                        .collect();
                    format!("{}Brane{{{};}}", chars, stmts.join("; "))
                }
            }
        }
    }

    fn format_statement(&self, stmt: &SequenceableStatement, indent: usize) -> String {
        let value = Self::new(stmt.body.clone()).format_for_snap_test(indent);
        match &stmt.name {
            Some(name) => format!("{} = {}", name, value),
            None => value,
        }
    }

    fn format_anchor(&self, anchor: &SequenceableFir) -> String {
        match anchor {
            SequenceableFir::Search { pattern, .. } => {
                pattern.strip_prefix('^').unwrap_or(pattern)
                    .strip_suffix('$').unwrap_or(pattern)
                    .to_string()
            }
            SequenceableFir::ConstantInt { value, .. } => format!("{}", value),
            _ => Self::new(anchor.clone()).format_for_snap_test(0),
        }
    }
}
