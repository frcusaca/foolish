use crate::fir::{Fir, FirQueryable, SearchDirection, StatementSimple};

/// A formatted line with its prefix (indent level in characters).
/// Only the outermost level materializes spaces.
pub type FormattedLine = (usize, String);

/// Result from a formatter: list of (prefix, text) pairs.
pub type FormattedLines = Vec<FormattedLine>;

/// Fixed body indent constant.
const B_DENT: usize = 2;

/// Maximum line width for single-lining decisions.
const LINE_BUDGET: usize = 128;

// ──────────────────────────────────────────────
// Public structs (unchanged API)
// ──────────────────────────────────────────────

#[derive(Default)]
pub struct FirSequencer {
    steps: u64,
}

impl FirSequencer {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn steps(&self) -> u64 {
        self.steps
    }
    pub fn format(fir: &Fir) -> String {
        materialize(&render_fir(fir, 0, 0, LINE_BUDGET))
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
pub struct HumanizingFirSequencerRef<'a> {
    fir: &'a dyn FirQueryable,
}

impl<'a> HumanizingFirSequencerRef<'a> {
    pub fn new(fir: &'a dyn FirQueryable) -> Self {
        Self { fir }
    }

    pub fn format_for_snap_test(&self) -> String {
        materialize(&render_fir(self.fir, 0, 0, LINE_BUDGET))
    }

    pub fn format_with_indent(&self, indent: usize) -> String {
        materialize_indented(&render_fir(self.fir, 0, 0, LINE_BUDGET), indent)
    }

    pub fn format_for_repl(&self) -> String {
        self.format_for_snap_test()
    }
}

// ──────────────────────────────────────────────
// Materialization: (prefix, text) pairs -> String
// ──────────────────────────────────────────────

fn materialize(lines: &FormattedLines) -> String {
    if lines.is_empty() {
        return String::new();
    }
    lines
        .iter()
        .map(|(prefix, text)| format!("{}{}", " ".repeat(*prefix), text))
        .collect::<Vec<String>>()
        .join("\n")
}

fn materialize_indented(lines: &FormattedLines, base_indent: usize) -> String {
    if lines.is_empty() {
        return String::new();
    }
    lines
        .iter()
        .map(|(prefix, text)| format!("{}{}", " ".repeat(base_indent + prefix), text))
        .collect::<Vec<String>>()
        .join("\n")
}

// ──────────────────────────────────────────────
// Indent computation helper
// ──────────────────────────────────────────────

fn body_indent_compute(open_indent: usize, close_indent: usize) -> usize {
    std::cmp::min(
        open_indent.saturating_sub(close_indent) + B_DENT,
        2 * B_DENT,
    )
}

// ──────────────────────────────────────────────
// Proto-brane formatter (items-based)
// ──────────────────────────────────────────────

/// Proto-brane formatter for comma-separated items.
///
/// Returns opener at prefix 0, body items at body_indent, closer at prefix 0.
/// For single-line: returns one line with opener + items + closer at prefix 0.
/// The opener IS included in the output.
fn proto_brane_formatter(
    opener: &str,
    closer: &str,
    open_indent: usize,
    close_indent: usize,
    items: &[String],
    line_hint: usize,
) -> FormattedLines {
    let body_indent = body_indent_compute(open_indent, close_indent);

    if items.is_empty() {
        return vec![(0, format!("{}{}", opener, closer))];
    }

    let joined = items.join(", ");
    let single_len = opener.len() + joined.len() + closer.len();

    if single_len <= line_hint {
        // Single-line: opener + items + closer at prefix 0
        vec![(0, format!("{}{}{}", opener, joined, closer))]
    } else {
        // Multi-line: opener at 0, items at body_indent, closer at 0
        let mut lines: FormattedLines = Vec::new();
        lines.push((0, opener.to_string()));
        let last = items.len() - 1;
        for (i, item) in items.iter().enumerate() {
            let suffix = if i < last { "," } else { "" };
            lines.push((body_indent, format!("{}{}", item, suffix)));
        }
        lines.push((0, closer.to_string()));
        lines
    }
}

// ──────────────────────────────────────────────
// Proto-brane formatter (body-based, with state)
// ──────────────────────────────────────────────

/// Proto-brane formatter for multi-line body content (branes, SF, SFF).
/// Returns opener at prefix 0, body lines at their own prefixes, closer at 0.
/// Caller adds body_indent to all returned prefixes.
fn proto_brane_formatter_body(
    closer: &str,
    opener_text: &str,
    body: &FormattedLines,
) -> FormattedLines {
    let mut lines: FormattedLines = Vec::new();
    lines.push((0, opener_text.to_string()));
    for (prefix, text) in body {
        lines.push((*prefix, text.clone()));
    }
    lines.push((0, closer.to_string()));
    lines
}

// ──────────────────────────────────────────────
// Proto-brane formatter with deferred result
// ──────────────────────────────────────────────

/// Proto-brane formatter for Search/HeadTail/Index with deferred result generation.
/// Result first, then non-result items, then closer.
///
/// Returns opener at prefix 0, body items at body_indent, closer at prefix 0.
/// For single-line: returns one line with opener + result + non-result + closer at prefix 0.
/// The opener IS included in the output.
fn proto_brane_formatter_with_result(
    opener: &str,
    closer: &str,
    open_indent: usize,
    close_indent: usize,
    non_result_items: &[String],
    result: Option<&dyn FirQueryable>,
    line_hint: usize,
) -> FormattedLines {
    let body_indent = body_indent_compute(open_indent, close_indent);

    // Compute result lines first (if any) — no trailing comma yet
    let mut result_lines: FormattedLines = if let Some(target) = result {
        let result_label = "result=";
        let inner_open = result_label.len();
        let inner_close = 0;
        let mut rl = render_fir(
            target,
            inner_open,
            inner_close,
            line_hint.saturating_sub(body_indent),
        );

        // Prepend "result=" label to first result line
        if let Some((_, first_text)) = rl.iter_mut().next() {
            *first_text = format!("{}{}", result_label, first_text);
        }

        // Add body_indent to all result prefixes so they sit at body level
        for (prefix, _text) in &mut rl {
            *prefix += body_indent;
        }
        rl
    } else {
        FormattedLines::new()
    };

    // Check if everything fits on one line
    let result_str = materialize(&result_lines);
    let result_has_newlines = result_str.contains('\n');
    let non_result_str = non_result_items.join(", ");

    let single_content = if result_lines.is_empty() {
        non_result_str.clone()
    } else {
        format!("{}, {}", result_str.trim(), non_result_str)
    };
    let can_single_line = !result_has_newlines
        && result_lines.len() <= 1
        && (opener.len() + single_content.len() + closer.len()) <= line_hint;

    if can_single_line {
        // Single-line: opener + result + non-result + closer
        // (join adds commas, no trailing comma on result needed)
        let mut parts = Vec::new();
        if !result_lines.is_empty() {
            // Strip any trailing comma from result before joining
            let r = result_str.trim().to_string();
            parts.push(r.trim_end_matches(',').to_string());
        }
        parts.extend(non_result_items.iter().cloned());
        vec![(0, format!("{}{}{}", opener, parts.join(", "), closer))]
    } else {
        // Multi-line: opener on its own line, body items at body_indent
        let mut lines: FormattedLines = Vec::new();
        lines.push((0, opener.to_string()));

        // Result first — add trailing comma if there are non-result items
        if !result_lines.is_empty() && !non_result_items.is_empty() {
            if let Some((_, last_text)) = result_lines.last_mut() {
                if !last_text.ends_with(',') {
                    *last_text = format!("{},", last_text);
                }
            }
        }
        lines.extend(result_lines);

        // Non-result items at body_indent
        let last_nr = non_result_items.len().saturating_sub(1);
        for (i, item) in non_result_items.iter().enumerate() {
            let suffix = if i < last_nr { "," } else { "" };
            lines.push((body_indent, format!("{}{}", item, suffix)));
        }

        // Closer at prefix 0
        lines.push((0, closer.to_string()));

        lines
    }
}

// ──────────────────────────────────────────────
// Main dispatch: render_fir
// ──────────────────────────────────────────────

/// Render a FirQueryable into formatted lines.
///
/// Prefixes returned are RELATIVE to the formatter's origin (opener position).
/// The parent adds its body_indent to ALL child prefixes.
fn render_fir(
    fir: &dyn FirQueryable,
    open_indent: usize,
    close_indent: usize,
    line_hint: usize,
) -> FormattedLines {
    let state = fir.hs_state();
    let show_state = state.should_show_nyes();

    // ── 1. ConstantInt ──
    if let Some(value) = fir.hs_constant_int() {
        return vec![(0, value.to_string())];
    }

    // ── 2. NK ──
    if let Some((reason, alarm)) = fir.hs_nk() {
        let mut msg = format!("??? ({})", reason);
        if let Some(a) = alarm {
            msg.push_str(&format!(", {}: {}", a.code, a.message));
        }
        return vec![(0, msg)];
    }

    // ── 3. Operator ──
    if let Some((op, operands)) = fir.hs_operator() {
        // Transparent when CONSTANT/INDEPENDENT
        if !show_state {
            if let Some(first) = operands.first() {
                return render_fir(&**first, open_indent, close_indent, line_hint);
            }
            return vec![(0, op.clone())];
        }

        let body_indent = body_indent_compute(open_indent, close_indent);
        let operand_lines: Vec<FormattedLines> = operands
            .iter()
            .map(|o| render_fir(&**o, 0, 0, line_hint.saturating_sub(body_indent)))
            .collect();

        let any_multi = operand_lines.iter().any(|l| l.len() > 1);

        if !any_multi {
            // All single-line: use proto_brane_formatter
            let body_items: Vec<String> = operand_lines.iter().map(|l| materialize(l)).collect();
            let mut items = body_items;
            items.push(state.to_string());
            let opener = format!("Op{}(", op);
            return proto_brane_formatter(
                &opener,
                ")",
                open_indent,
                close_indent,
                &items,
                line_hint,
            );
        }

        // Multi-line: opener on its own line, operands at body_indent
        let mut lines: FormattedLines = Vec::new();
        lines.push((0, format!("Op{}(", op)));
        let last_op = operands.len().saturating_sub(1);
        for (oi, oplines) in operand_lines.into_iter().enumerate() {
            let is_last_op = oi == last_op;
            let last_li = oplines.len().saturating_sub(1);
            for (pi, (prefix, text)) in oplines.into_iter().enumerate() {
                let is_last_line = pi == last_li;
                let mut t = text;
                // Comma after last line of every operand (state follows all operands)
                if is_last_line {
                    t.push(',');
                }
                lines.push((body_indent + prefix, t));
            }
        }
        lines.push((body_indent, state.to_string()));
        lines.push((0, ")".to_string()));
        return lines;
    }

    // ── 4. Search ──
    if let Some((pattern, direction, anchored, _anchor, target)) = fir.hs_search() {
        let pbid = if direction == SearchDirection::Backward {
            "?"
        } else {
            "/"
        };
        let anchor_str = if anchored { "ANCHORED" } else { "UNANCHORED" };

        let mut non_result_items = vec![format!("pattern='{}'", pattern), anchor_str.to_string()];
        if show_state {
            non_result_items.push(state.to_string());
        }

        let opener = format!("{}{}", pbid, "(");
        return proto_brane_formatter_with_result(
            &opener,
            ")",
            open_indent,
            close_indent,
            &non_result_items,
            target.as_deref(),
            line_hint,
        );
    }

    // ── 5. HeadTail ──
    if let Some((is_head, _anchored, anchor)) = fir.hs_head_tail() {
        let pbid = if is_head { "^" } else { "$" };
        let mut non_result_items = Vec::new();

        if show_state {
            non_result_items.push(state.to_string());
        }

        let opener = format!("{}{}", pbid, "(");
        return proto_brane_formatter_with_result(
            &opener,
            ")",
            open_indent,
            close_indent,
            &non_result_items,
            anchor.as_deref(),
            line_hint,
        );
    }

    // ── 6. Index ──
    if let Some((offset, anchored, anchor)) = fir.hs_index() {
        let anchor_str = if anchored { "ANCHORED" } else { "UNANCHORED" };
        let mut non_result_items = vec![format!("offset={}", offset), anchor_str.to_string()];

        if show_state {
            non_result_items.push(state.to_string());
        }

        let opener = "#(".to_string();
        return proto_brane_formatter_with_result(
            &opener,
            ")",
            open_indent,
            close_indent,
            &non_result_items,
            anchor.as_deref(),
            line_hint,
        );
    }

    // ── 7. StayFoolish ──
    if let Some(expr) = fir.hs_stay_foolish() {
        if !show_state {
            return render_fir(&*expr, open_indent, close_indent, line_hint);
        }
        let body = render_fir(&*expr, 0, 0, line_hint.saturating_sub(B_DENT));
        let bi = body_indent_compute(open_indent, close_indent);
        let mut lines: FormattedLines = Vec::new();
        lines.push((0, format!("<{}", state)));
        for (prefix, text) in body {
            lines.push((bi + prefix, text));
        }
        lines.push((0, ">".to_string()));
        return lines;
    }

    // ── 8. StayFullyFoolish ──
    if let Some(expr) = fir.hs_stay_fully_foolish() {
        if !show_state {
            return render_fir(&*expr, open_indent, close_indent, line_hint);
        }
        let body = render_fir(&*expr, 0, 0, line_hint.saturating_sub(2 * B_DENT));
        let bi = body_indent_compute(open_indent, close_indent);
        let mut lines: FormattedLines = Vec::new();
        lines.push((0, format!("<<{}", state)));
        for (prefix, text) in body {
            lines.push((bi + prefix, text));
        }
        lines.push((0, ">>".to_string()));
        return lines;
    }

    // ── 9. Concatenation ──
    if let Some((elements, merged)) = fir.hs_concatenation() {
        if !show_state {
            if let Some(m) = &merged {
                let mut brane_lines = render_fir(&**m, 0, 0, line_hint);
                if let Some((_, first)) = brane_lines.iter_mut().next() {
                    *first = format!("⨃{}", first);
                }
                return brane_lines;
            }
            let bi = body_indent_compute(0, 0);
            let mut body_lines: FormattedLines = Vec::new();
            for (ei, elem) in elements.iter().enumerate() {
                let is_last = ei == elements.len() - 1;
                let elem_lines = render_fir(&**elem, 0, 0, line_hint);
                for (prefix, text) in elem_lines {
                    body_lines.push((bi + prefix, text));
                }
                if !is_last {
                    if let Some((_, last_text)) =
                        body_lines.iter_mut().rev().find(|(p, _t)| *p == bi)
                    {
                        if !last_text.ends_with(';')
                            && !last_text.ends_with('}')
                            && !last_text.ends_with(')')
                        {
                            last_text.push(';');
                        }
                    }
                }
            }
            let mut lines = vec![(0, "{".to_string())];
            lines.extend(body_lines);
            lines.push((0, "}".to_string()));
            if let Some((_, first)) = lines.iter_mut().next() {
                *first = format!("⨃{}", first);
            }
            return lines;
        }
        // Non-terminal: items-based proto-brane
        let mut items = vec![format!("elements={}", elements.len())];
        for elem in &elements {
            items.push(materialize(&render_fir(&**elem, 0, 0, line_hint)));
        }
        if let Some(m) = &merged {
            items.push(format!(
                "merged={}",
                materialize(&render_fir(&**m, 0, 0, line_hint))
            ));
        }
        items.push(state.to_string());
        let opener = "⨃(".to_string();
        return proto_brane_formatter(&opener, ")", open_indent, close_indent, &items, line_hint);
    }

    // ── 10. NormalBrane ──
    if let Some((characterizations, statements)) = fir.hs_brane() {
        let chars = if characterizations.is_empty() {
            String::new()
        } else {
            format!("{}'", characterizations.join(" "))
        };

        if statements.is_empty() {
            let opener = if chars.is_empty() { "{" } else { &chars };
            return vec![(0, format!("{}{}", opener, "}"))];
        }

        let body_indent = body_indent_compute(open_indent, close_indent);
        let stmt_lines = render_statements(
            &statements,
            body_indent,
            close_indent,
            line_hint.saturating_sub(body_indent),
        );

        let mut lines: FormattedLines = Vec::new();

        // Opener: chars + { + optional bare state token
        let opener_text = if chars.is_empty() {
            if show_state {
                format!("{}{}", "{", state)
            } else {
                "{".to_string()
            }
        } else {
            if show_state {
                format!("{}{}{}", chars, "{", state)
            } else {
                format!("{}{{", chars)
            }
        };
        lines.push((0, opener_text));

        lines.extend(stmt_lines);
        lines.push((0, "}".to_string()));

        return lines;
    }

    // ── Fallback ──
    vec![(0, format!("Unknown({})", fir.hs_variant()))]
}

// ──────────────────────────────────────────────
// Statement rendering
// ──────────────────────────────────────────────

/// Render brane statements into formatted lines.
///
/// For each statement:
/// 1. Call render_fir for the body (open_indent = name.len()+1 for named, 0 otherwise)
/// 2. Prepend name= to first line (for named statements)
/// 3. Add ; to last line if not the last statement
/// 4. Add body_indent to ALL child prefixes
fn render_statements(
    statements: &[StatementSimple],
    body_indent: usize,
    close_indent: usize,
    line_hint: usize,
) -> FormattedLines {
    let mut lines: FormattedLines = Vec::new();
    let total_stmts = statements.len();

    for (idx, stmt) in statements.iter().enumerate() {
        let is_last_stmt = idx == total_stmts - 1;

        let child_open = if let Some(ref name) = stmt.name {
            name.len() + 1
        } else {
            0
        };

        let child_lines = render_fir(
            &*stmt.body,
            child_open,
            close_indent,
            line_hint.saturating_sub(body_indent),
        );

        let mut merged = child_lines;
        if let Some(ref name) = stmt.name {
            if let Some((_, first_text)) = merged.iter_mut().next() {
                *first_text = format!("{}={}", name, first_text);
            }
        }

        let last_idx = merged.len().saturating_sub(1);
        for (i, (prefix, text)) in merged.into_iter().enumerate() {
            let text = if i == last_idx && !is_last_stmt {
                format!("{};", text)
            } else {
                text
            };
            lines.push((body_indent + prefix, text));
        }
    }

    lines
}

// ──────────────────────────────────────────────
// Public simple formatting functions
// ──────────────────────────────────────────────

pub fn format_statement_simple(stmt: &StatementSimple, indent: usize) -> String {
    let value = format_fir_simple_indent(&*stmt.body, indent);
    match &stmt.name {
        Some(name) => format!("{} = {}", name, value),
        None => value,
    }
}

pub fn format_fir_simple(fir: &dyn FirQueryable) -> String {
    format_fir_simple_indent(fir, 0)
}

pub fn format_fir_simple_with_indent(fir: &dyn FirQueryable, indent: usize) -> String {
    format_fir_simple_indent(fir, indent)
}

fn format_fir_simple_indent(fir: &dyn FirQueryable, indent: usize) -> String {
    materialize_indented(&render_fir(fir, 0, 0, LINE_BUDGET), indent)
}
