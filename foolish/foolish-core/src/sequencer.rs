use crate::fir::Fir;

/// Format a FIR tree as human-readable output (for approval tests).
pub struct Sequencer {
    depth: usize,
    steps: u64,
    lines: Vec<String>,
}

impl Sequencer {
    pub fn new() -> Self {
        Self {
            depth: 0,
            steps: 0,
            lines: Vec::new(),
        }
    }

    pub fn steps(&self) -> u64 { self.steps }

    pub fn format(fir: &Fir) -> String {
        let mut seq = Self::new();
        seq.format_fir(fir);
        seq.lines.join("\n")
    }

    pub fn format_with_header(source: &str, fir: &Fir, steps: u64) -> String {
        let mut seq = Self { steps, ..Self::new() };
        seq.lines.push(format!("INPUT: {}", source.trim()));
        seq.lines.push("PARSED:".to_string());
        seq.format_fir(fir);
        seq.lines.push(format!("STEPS: {}", steps));
        seq.lines.join("\n")
    }

    fn indent(&self) -> String {
        "  ".repeat(self.depth)
    }

    fn format_fir(&mut self, fir: &Fir) {
        match fir {
            Fir::ConstantInt { value, state } => {
                self.lines.push(format!("{}Int({}) [{}]", self.indent(), value, state));
            }
            Fir::Nk { reason, state } => {
                self.lines.push(format!("??? ({}) [{}]", reason, state));
            }
            Fir::NormalBrane { characterizations, statements, state } => {
                let chars = if characterizations.is_empty() {
                    String::new()
                } else {
                    format!("{}'", characterizations.join(" "))
                };
                self.lines.push(format!("{}{}Brane [{}]", self.indent(), chars, state));
                self.depth += 1;
                for stmt in statements {
                    if let Some(ref name) = stmt.name {
                        self.lines.push(format!("{}{} = ", self.indent(), name));
                    }
                    self.format_fir(&stmt.body);
                }
                self.depth -= 1;
            }
            Fir::BinaryOp { op, left, right, state } => {
                self.lines.push(format!("{}BinaryOp({}) [{}]", self.indent(), op, state));
                self.depth += 1;
                self.format_fir(left);
                self.format_fir(right);
                self.depth -= 1;
            }
            Fir::UnaryOp { op, expr, state } => {
                self.lines.push(format!("{}UnaryOp({}) [{}]", self.indent(), op, state));
                self.depth += 1;
                self.format_fir(expr);
                self.depth -= 1;
            }
            Fir::Search { pattern, direction, anchored, target, state, .. } => {
                let anchor_str = if *anchored { "ANCHORED" } else { "FREE" };
                self.lines.push(format!(
                    "{}Search(pattern='{}', dir={}, {}) [{}]",
                    self.indent(),
                    pattern,
                    direction,
                    anchor_str,
                    state
                ));
                if let Some(t) = target {
                    self.depth += 1;
                    self.format_fir(t);
                    self.depth -= 1;
                }
            }
            Fir::Index { offset, anchored, state, .. } => {
                let anchor_str = if *anchored { "ANCHORED" } else { "FREE" };
                self.lines.push(format!(
                    "{}Index(offset={}, {}) [{}]",
                    self.indent(),
                    offset,
                    anchor_str,
                    state
                ));
            }
            Fir::HeadTail { is_head, anchored, state, .. } => {
                let ht = if *is_head { "HEAD" } else { "TAIL" };
                let anchor_str = if *anchored { "ANCHORED" } else { "FREE" };
                self.lines.push(format!(
                    "{}HeadTail({}, {}) [{}]",
                    self.indent(),
                    ht,
                    anchor_str,
                    state
                ));
            }
            Fir::Concatenation { elements, state, .. } => {
                self.lines.push(format!(
                    "{}Concatenation(elements={}) [{}]",
                    self.indent(),
                    elements.len(),
                    state
                ));
                self.depth += 1;
                for elem in elements {
                    self.format_fir(elem);
                }
                self.depth -= 1;
            }
        }
    }
}
