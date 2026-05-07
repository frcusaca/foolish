use crate::fir::{Fir, Steppable};

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
