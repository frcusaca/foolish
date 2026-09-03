//! Foolish and detailed rendering for ubca2's arena FIR (FOOP-36 §1).

use foolish_core::fir::Nyes;

use crate::fvm_storage::{
    ANON_STMT_NAME, ConcatProvenance, FVMStorage, FirCursor, FirPointer, FirSpec, proto_to_core_fir,
};
use crate::nyes_ext::NyesExt;

const LINE_BUDGET: usize = 108;
const BODY_INDENT: usize = 2;
const NK_REASON_LIMIT: usize = 60;

/// Selects how an arena FIR is rendered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SequenceMode {
    /// Valid Foolish source with evaluator state confined to comments.
    #[default]
    Foolish,
    /// The legacy FIR-internal rendering used for detailed debugging.
    Detailed,
}

/// Configures Foolish sequencing without changing evaluator state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SequenceOptions {
    pub mode: SequenceMode,
    /// Soft maximum line width. Atoms and trailing annotations are not split.
    pub width: usize,
    /// Whether NK findings are appended as Foolish line comments.
    pub comment_nk: bool,
    /// Overrides `comment_nk` and every state annotation: when true, the
    /// sequencer emits no `!!` comments of its own at all. Named
    /// "sequencing comments" to distinguish this renderer's own annotations
    /// from any comment the sequencer may in future echo through from
    /// source rather than generate itself.
    pub suppress_sequencing_comments: bool,
}

impl Default for SequenceOptions {
    fn default() -> Self {
        Self {
            mode: SequenceMode::Foolish,
            width: LINE_BUDGET,
            comment_nk: true,
            suppress_sequencing_comments: false,
        }
    }
}

/// Renders ubca2's arena FIR.
#[derive(Debug, Clone, Copy, Default)]
pub struct Ubca2Sequencer;

impl Ubca2Sequencer {
    /// Formats one arena FIR in the selected mode.
    #[must_use]
    pub fn format(storage: &FVMStorage, fir: FirPointer, mode: SequenceMode) -> String {
        let options = SequenceOptions {
            mode,
            ..SequenceOptions::default()
        };
        Self::format_with(storage, fir, &options)
    }

    /// Formats one arena FIR with explicit sequencing options.
    #[must_use]
    pub fn format_with(storage: &FVMStorage, fir: FirPointer, options: &SequenceOptions) -> String {
        match options.mode {
            SequenceMode::Foolish => Renderer::new(storage, options).render(fir),
            SequenceMode::Detailed => Self::format_detailed(storage, fir),
        }
    }

    fn format_detailed(storage: &FVMStorage, fir: FirPointer) -> String {
        let core_fir = proto_to_core_fir(storage, fir);
        foolish_core::FirSequencer::format(&core_fir)
    }
}

struct Renderer<'a> {
    storage: &'a FVMStorage,
    options: &'a SequenceOptions,
}

impl<'a> Renderer<'a> {
    fn new(storage: &'a FVMStorage, options: &'a SequenceOptions) -> Self {
        Self { storage, options }
    }

    fn render(&self, fir: FirPointer) -> String {
        self.render_expr(fir, self.options.width, None, true)
            .join("\n")
    }

    fn render_expr(
        &self,
        fir: FirPointer,
        width: usize,
        current_stmt: Option<FirPointer>,
        annotate: bool,
    ) -> Vec<String> {
        let cursor = FirCursor::new(fir, self.storage);
        let mut lines = match cursor.node() {
            FirSpec::IndepInt { value } => vec![value.to_string()],
            FirSpec::Nk { .. } => vec!["???".to_string()],
            FirSpec::Creation => vec![
                cursor
                    .as_creation_display_name(current_stmt)
                    .unwrap_or_else(|| "⬤".to_string()),
            ],
            FirSpec::Operator { op } => {
                self.render_process_or_result(fir, width, current_stmt, |renderer| {
                    renderer.render_operator(fir, op, width, current_stmt)
                })
            }
            FirSpec::Comparison { .. } => {
                self.render_process_or_result(fir, width, current_stmt, |renderer| {
                    renderer.render_operator(
                        fir,
                        cursor.as_op_name().unwrap_or("?"),
                        width,
                        current_stmt,
                    )
                })
            }
            FirSpec::Statement { .. } => self.render_statement(fir, width, true),
            FirSpec::Brane { .. } | FirSpec::ConcatHelper => {
                self.render_brane(fir, width, current_stmt)
            }
            FirSpec::Search { .. } => {
                self.render_process_or_result(fir, width, current_stmt, |renderer| {
                    renderer.render_search(fir, width, current_stmt)
                })
            }
            FirSpec::Index { .. } => {
                self.render_process_or_result(fir, width, current_stmt, |renderer| {
                    renderer.render_index(fir, width, current_stmt)
                })
            }
            FirSpec::FoolRef { referent } => {
                self.render_expr(*referent, width, current_stmt, false)
            }
            FirSpec::StayFoolish => self.render_stay(fir, width, current_stmt, false),
            FirSpec::StayFullyFoolish => self.render_stay(fir, width, current_stmt, true),
            FirSpec::Concatenation { provenance } => {
                self.render_concatenation(fir, width, current_stmt, *provenance)
            }
        };

        if annotate {
            self.annotate(fir, &mut lines);
        }
        lines
    }

    fn render_process_or_result<F>(
        &self,
        fir: FirPointer,
        width: usize,
        current_stmt: Option<FirPointer>,
        written: F,
    ) -> Vec<String>
    where
        F: FnOnce(&Self) -> Vec<String>,
    {
        let cursor = FirCursor::new(fir, self.storage);
        if let Some(&result) = cursor.ubc_children().first()
            && self.storage.get_nyes(result).is_conclusive()
        {
            return self.render_expr(result.value(self.storage), width, current_stmt, false);
        }
        written(self)
    }

    fn render_operator(
        &self,
        fir: FirPointer,
        op: &str,
        width: usize,
        current_stmt: Option<FirPointer>,
    ) -> Vec<String> {
        let cursor = FirCursor::new(fir, self.storage);
        let operands: Vec<String> = cursor
            .foolish_children()
            .iter()
            .map(|&child| self.render_inline(child, width, current_stmt))
            .collect();
        let text = match operands.as_slice() {
            [only] => format!("{op}{only}"),
            [left, right] => format!("{left} {op} {right}"),
            _ => operands.join(&format!(" {op} ")),
        };
        vec![text]
    }

    fn render_search(
        &self,
        fir: FirPointer,
        width: usize,
        current_stmt: Option<FirPointer>,
    ) -> Vec<String> {
        let cursor = FirCursor::new(fir, self.storage);
        let FirSpec::Search {
            pattern,
            anchored,
            forward,
            is_value_search,
            contexted,
        } = cursor.node()
        else {
            unreachable!()
        };
        let children = cursor.foolish_children();
        let anchor =
            anchored.then(|| self.render_written_operand_inline(children[0], width, current_stmt));
        let marker = if *forward { "~" } else { "?" };
        let context = if *contexted { "&" } else { "" };

        let query = if *is_value_search {
            let value_index = usize::from(*anchored);
            let value = children
                .get(value_index)
                .map(|&child| self.render_inline(child, width, current_stmt))
                .unwrap_or_else(|| "???".to_string());
            let name = canonical_name_pattern(pattern)
                .map(|name| name.to_string())
                .unwrap_or_default();
            format!("{context}{marker}{name}={value}")
        } else if let Some(name) = canonical_name_pattern(pattern) {
            if *anchored {
                format!("{context}{marker}{name}")
            } else {
                name.to_string()
            }
        } else {
            // FOOP-75 §6.1 (current, unimplemented §6.2): the parser's
            // `parse_regexp_pattern` absorbs a parenthesized run VERBATIM,
            // parens included, into `pattern` itself — `B~(x)` stores
            // pattern `"(x)"`, not `"x"`. So a pattern that already reads as
            // parenthesized must be written back exactly as stored; wrapping
            // it in another layer (`?((ho))`) is not disambiguation, it is a
            // literal extra `(`/`)` pair the parser then reads as PART OF
            // the pattern text, which is never what was evaluated (verified:
            // `hw?(ho)` renders back as `hw?((ho))`, which fails to parse —
            // "expected primary expression, found RParen" — because the
            // outer `?(` opens a paren run whose matching `)` is consumed
            // mid-pattern, leaving a stray `)` behind).
            let wrapped = pattern.starts_with('(') && pattern.ends_with(')');
            if wrapped {
                format!("{context}{marker}{pattern}")
            } else {
                format!("{context}{marker}({pattern})")
            }
        };

        vec![match anchor {
            Some(anchor) => format!("{anchor}{query}"),
            None => query,
        }]
    }

    fn render_index(
        &self,
        fir: FirPointer,
        width: usize,
        current_stmt: Option<FirPointer>,
    ) -> Vec<String> {
        let cursor = FirCursor::new(fir, self.storage);
        let FirSpec::Index {
            offset,
            anchored,
            contexted,
        } = cursor.node()
        else {
            unreachable!()
        };
        let marker = match (*anchored, *offset) {
            (true, 0) => "^".to_string(),
            (true, -1) => "$".to_string(),
            _ => format!("#{offset}"),
        };
        let context = if *contexted { "&" } else { "" };
        if *anchored {
            let anchor = cursor
                .foolish_children()
                .first()
                .map(|&child| self.render_written_operand_inline(child, width, current_stmt))
                .unwrap_or_else(|| "???".to_string());
            vec![format!("{anchor}{context}{marker}")]
        } else {
            vec![format!("{context}{marker}")]
        }
    }

    fn render_stay(
        &self,
        fir: FirPointer,
        width: usize,
        current_stmt: Option<FirPointer>,
        fully: bool,
    ) -> Vec<String> {
        let cursor = FirCursor::new(fir, self.storage);
        let inner = cursor
            .foolish_children()
            .first()
            .map(|&child| {
                self.render_written_operand_inline(child, width.saturating_sub(4), current_stmt)
            })
            .unwrap_or_else(|| "???".to_string());
        // A `<`/`<<` wrapper must not let its content's own leading/trailing
        // `<`/`>` fuse with this wrapper's delimiter: the lexer greedily
        // pairs adjacent `<`/`>` characters two-at-a-time into `LtLt`/`GtGt`
        // tokens (`foolish-parser/src/lexer.rs`), left to right, so a run of
        // delimiter characters at a boundary mis-lexes whenever the pairing
        // does not land where THIS wrapper's own close is expected.
        //
        // Whether a given boundary is safe depends on the FULL run length at
        // that point in the fully-rendered string, not just on this call's
        // own `inner` and `close` — a boundary that looks even in isolation
        // (this wrapper's own trailing `>` count plus its own close) can
        // still mis-lex once an ENCLOSING wrapper's delimiter is appended
        // immediately afterward (verified by trying to narrow this check to
        // "only when the immediate run is odd": `b = <1 + <<b>> + <c>>` — the
        // outer SF's own boundary looked even considered alone, but the
        // enclosing statement's context still fused it). Determining safety
        // correctly would require the caller to know what follows, which
        // `render_stay` does not have. A single space unconditionally
        // whenever the content touches this wrapper's own delimiter
        // character is therefore the only reliably safe choice — it costs
        // a small amount of "clean output" in the (harmless) case where the
        // narrower check would also have been safe, in exchange for never
        // being wrong. `misc/concat_sf_f_more.foo`'s original source uses
        // exactly this separation (`<c> >`) for the same reason.
        let needs_space_after = inner.starts_with('<');
        let needs_space_before = inner.ends_with('>');
        let open = if fully { "<<" } else { "<" };
        let close = if fully { ">>" } else { ">" };
        let sep_after = if needs_space_after { " " } else { "" };
        let sep_before = if needs_space_before { " " } else { "" };
        vec![format!("{open}{sep_after}{inner}{sep_before}{close}")]
    }

    fn render_concatenation(
        &self,
        fir: FirPointer,
        width: usize,
        current_stmt: Option<FirPointer>,
        provenance: ConcatProvenance,
    ) -> Vec<String> {
        let cursor = FirCursor::new(fir, self.storage);
        if self.storage.get_nyes(fir).is_constanic()
            && (!cursor.ubc_children().is_empty() || self.storage.get_nyes(fir).is_conclusive())
        {
            return self.render_brane(fir, width, current_stmt);
        }

        let mut children: Vec<FirPointer> = cursor.foolish_children().to_vec();
        if provenance == ConcatProvenance::TailConcatenation {
            children.reverse();
        }
        let separator = if provenance == ConcatProvenance::TailConcatenation {
            "`"
        } else {
            ""
        };
        vec![
            children
                .into_iter()
                .map(|child| self.render_concat_element(child, width, current_stmt))
                .collect::<Vec<_>>()
                .join(separator),
        ]
    }

    fn render_concat_element(
        &self,
        child: FirPointer,
        width: usize,
        current_stmt: Option<FirPointer>,
    ) -> String {
        let cursor = FirCursor::new(child, self.storage);
        if matches!(cursor.node(), FirSpec::StayFoolish) {
            return cursor
                .foolish_children()
                .first()
                .map(|&inner| self.render_written_operand_inline(inner, width, current_stmt))
                .unwrap_or_else(|| "???".to_string());
        }
        self.render_inline(child, width, current_stmt)
            .replace("{ ", "{")
            .replace(" }", "}")
    }

    /// Renders `fir` and safely collapses it to ONE line, for use as an
    /// inline operand (an operator's operand, a search/index anchor, an SF
    /// wrapper's interior, a concatenation element).
    ///
    /// A naive `render_expr(..).join(" ")` is unsafe here: when `fir`
    /// renders to several lines (e.g. a brane too wide to inline, each
    /// member on its own line with its own `!!` annotation), joining those
    /// lines with plain spaces puts a `!!` comment — which the lexer reads
    /// to end-of-line — in the MIDDLE of the resulting single line, silently
    /// swallowing everything rendered after it (verified:
    /// `foop/33/boolean/comparison_non_integer.foo`'s `{1, {x=5;}, 'lt}$`
    /// anchor, whose inner statement's `!! NK: ...` annotation ate the
    /// closing `}` and the outer `;`). Each line's own annotation is
    /// dropped before joining — annotations are per-line commentary, never
    /// part of the written form an inline context needs.
    fn render_inline(
        &self,
        fir: FirPointer,
        width: usize,
        current_stmt: Option<FirPointer>,
    ) -> String {
        self.render_expr(fir, width, current_stmt, false)
            .into_iter()
            .map(|line| {
                line.split("  !!")
                    .next()
                    .unwrap_or(&line)
                    .trim()
                    .to_string()
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// [`Self::render_inline`], routed through [`Self::render_written_operand`]
    /// so a Search/Index anchor keeps its written form rather than collapsing
    /// early to a conclusive result (the same distinction
    /// `render_written_operand` already draws for multi-line callers).
    fn render_written_operand_inline(
        &self,
        fir: FirPointer,
        width: usize,
        current_stmt: Option<FirPointer>,
    ) -> String {
        self.render_written_operand(fir, width, current_stmt)
            .into_iter()
            .map(|line| {
                line.split("  !!")
                    .next()
                    .unwrap_or(&line)
                    .trim()
                    .to_string()
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn render_written_operand(
        &self,
        fir: FirPointer,
        width: usize,
        current_stmt: Option<FirPointer>,
    ) -> Vec<String> {
        match FirCursor::new(fir, self.storage).node() {
            FirSpec::Search { .. } => self.render_search(fir, width, current_stmt),
            FirSpec::Index { .. } => self.render_index(fir, width, current_stmt),
            _ => self.render_expr(fir, width, current_stmt, false),
        }
    }

    fn render_brane(
        &self,
        fir: FirPointer,
        width: usize,
        _current_stmt: Option<FirPointer>,
    ) -> Vec<String> {
        let cursor = FirCursor::new(fir, self.storage);
        let count = cursor.stmt_count().unwrap_or(0);
        let chars = cursor
            .as_brane_characterizations()
            .iter()
            .map(|component| format!("{component}'"))
            .collect::<String>();
        if count == 0 {
            return vec![format!("{chars}{{}}")];
        }

        let statements: Vec<Vec<String>> = (0..count)
            .filter_map(|index| cursor.stmt_at(index))
            .enumerate()
            .map(|(index, statement)| self.render_statement(statement, width, index + 1 == count))
            .collect();
        let can_inline = statements.iter().all(|lines| {
            lines.len() == 1 && !lines[0].contains("  !!") && !lines[0].contains('\n')
        });
        if can_inline {
            let inside = statements
                .iter()
                .map(|lines| lines[0].as_str())
                .collect::<Vec<_>>()
                .join(" ");
            let candidate = format!("{chars}{{{inside}}}");
            if candidate.chars().count() <= width {
                return vec![candidate];
            }
        }

        let mut lines = vec![format!("{chars}{{")];
        for statement in statements {
            lines.extend(
                statement
                    .into_iter()
                    .map(|line| format!("{}{line}", " ".repeat(BODY_INDENT))),
            );
        }
        lines.push("}".to_string());
        lines
    }

    fn render_statement(&self, statement: FirPointer, width: usize, is_last: bool) -> Vec<String> {
        let cursor = FirCursor::new(statement, self.storage);
        let name = cursor
            .as_stmt_identifier()
            .map(|identifier| identifier.searchable_name())
            .filter(|name| *name != ANON_STMT_NAME);
        let Some(&body) = cursor.foolish_children().first() else {
            return vec!["???".to_string()];
        };

        let body_cursor = FirCursor::new(body, self.storage);
        let attached = match body_cursor.node() {
            FirSpec::Index {
                offset,
                anchored: true,
                ..
            } if matches!(offset, 0 | -1)
                && name.is_some()
                && body_cursor
                    .foolish_children()
                    .first()
                    .is_some_and(|&anchor| self.is_safe_attached_anchor(anchor)) =>
            {
                Some(*offset)
            }
            _ => None,
        };
        let mut lines = if let Some(offset) = attached {
            let anchor = body_cursor
                .foolish_children()
                .first()
                .map(|&child| self.render_written_operand_inline(child, width, Some(statement)))
                .unwrap_or_else(|| "???".to_string());
            let marker = if offset == 0 { "^" } else { "$" };
            let mut attached = vec![format!("{} ={marker} {anchor}", name.unwrap())];
            self.annotate(body, &mut attached);
            attached
        } else {
            let prefix = name.map_or(0, |name| name.chars().count() + 3);
            let mut rendered =
                self.render_expr(body, width.saturating_sub(prefix), Some(statement), true);
            if let Some(name) = name
                && let Some(first) = rendered.first_mut()
            {
                *first = format!("{name} = {first}");
            }
            rendered
        };

        if !is_last {
            append_before_comment(lines.last_mut().expect("statement has a line"), ";");
        }
        lines
    }

    /// Whether `anchor` is safe to write as the RHS of an attached-search
    /// statement (`name =$ anchor` / `name =^ anchor`, FOOP-75 §4).
    ///
    /// The attached spelling is defined as `name =SPEC RHS` meaning
    /// `name = RHS SPEC` (FOOP-75 §1/§2): the parser records the adjacent
    /// `=$`/`=^` run, parses the REST of the line as an ordinary RHS
    /// expression, then replays the recorded suffix against it. That replay
    /// requires the RHS to parse as a complete, self-contained primary
    /// expression on its own — which fails whenever the anchor RENDERS with
    /// a leading search-operator marker (`?`, `~`, `#`, `^`, bare `$`) and no
    /// grounding identifier/brane before it, since a bare marker is not a
    /// valid standalone primary outside this same attached path (verified
    /// live: `d =$ #-1` and `same =$ ?=1` both fail with FOOP-75 §6's
    /// "attached search specification is ambiguous", while `d = #-1$` and
    /// `same = (?=1)$` both parse).
    ///
    /// This is NOT simply "the anchor's own `anchored` flag": an unanchored
    /// NAME search with a canonical pattern (a bare identifier reference
    /// like `b`) renders as plain `b` — no marker at all — via
    /// [`render_search`]'s `canonical_name_pattern` branch, and is exactly
    /// FOOP-75's own canonical example (`tail_of_b =$ b;`). Only the cases
    /// that keep a marker in their rendering are unsafe: an unanchored value
    /// search (always `?=`/`~=`-prefixed), an unanchored search whose
    /// pattern is not a bare name (`?(...)`), or any unanchored index/seek
    /// (`#N`, `^`, `$`, all marker-only with no anchor to ground them).
    fn is_safe_attached_anchor(&self, anchor: FirPointer) -> bool {
        match FirCursor::new(anchor, self.storage).node() {
            FirSpec::Search { anchored: true, .. } => true,
            FirSpec::Search {
                anchored: false,
                is_value_search: false,
                pattern,
                ..
            } => canonical_name_pattern(pattern).is_some(),
            FirSpec::Search {
                anchored: false,
                is_value_search: true,
                ..
            } => false,
            FirSpec::Index { anchored, .. } => *anchored,
            _ => true,
        }
    }

    fn annotate(&self, fir: FirPointer, lines: &mut [String]) {
        if self.options.suppress_sequencing_comments {
            return;
        }
        // A brane's own NYES is a rollup of its members (decide_nyes_due_to_children);
        // each member already carries its own accurate annotation on its own line, so
        // repeating the derived state — or worse, a borrowed child's NK reason — on the
        // opening `{` is redundant noise, not information otherwise unrecoverable from
        // the output (§4's own test for whether an annotation earns its place). The
        // written brane, re-stepped, reaches the same member states (§2.1), so nothing
        // is lost by leaving the bracket bare.
        //
        // The one exception: a DIRECT alarm on the brane itself (`alarm_reason`, a
        // per-pointer field with no recursion into children — contrast `nk_reason`,
        // which deliberately walks into children and is exactly the borrowing this
        // rule avoids). The step-cap/iteration alarm is set only on the composed
        // root, which is a brane with no member of its own carrying that reason —
        // suppressing it unconditionally would silently discard the one piece of
        // information nowhere else in the output, defeating §2.1's debugging intent.
        let cursor = FirCursor::new(fir, self.storage);
        if matches!(cursor.node(), FirSpec::Brane { .. } | FirSpec::ConcatHelper) {
            if self.options.comment_nk
                && let Some(reason) = self.storage.alarm_reason(fir)
            {
                let annotation = format!("NK: {}", sanitize_reason(reason));
                if let Some(first) = lines.first_mut() {
                    first.push_str("  !! ");
                    first.push_str(&annotation);
                }
            }
            return;
        }
        let state = self.storage.get_nyes(fir);
        let annotation = match state {
            Nyes::Prembrionic
            | Nyes::Embryonic
            | Nyes::Braning
            | Nyes::Econstanic
            | Nyes::Woconstanic => Some(state.to_string()),
            Nyes::Nk if self.options.comment_nk => Some(format!("NK: {}", self.nk_reason(fir))),
            Nyes::Constant | Nyes::Independent | Nyes::Nk => None,
        };
        if let (Some(annotation), Some(first)) = (annotation, lines.first_mut()) {
            first.push_str("  !! ");
            first.push_str(&annotation);
        }
    }

    fn nk_reason(&self, fir: FirPointer) -> String {
        let cursor = FirCursor::new(fir, self.storage);
        let reason = if let Some(reason) = cursor.as_nk_reason() {
            if reason == "division by zero" {
                "DIV-BY-ZERO: division by zero".to_string()
            } else {
                reason.to_string()
            }
        } else if let Some(reason) = self.storage.alarm_reason(fir) {
            reason.to_string()
        } else if let Some(&result) = cursor.ubc_children().first() {
            self.nk_reason(result)
        } else {
            match cursor.node() {
                FirSpec::Search { anchored: true, .. } => {
                    "anchored search found no match".to_string()
                }
                FirSpec::Index { anchored: true, .. } => cursor
                    .foolish_children()
                    .first()
                    .and_then(|&anchor| {
                        FirCursor::new(anchor.value(self.storage), self.storage).as_i64()
                    })
                    .map_or_else(
                        || "anchored index found no match".to_string(),
                        |value| format!("{value} is not a brane"),
                    ),
                _ => cursor
                    .foolish_children()
                    .iter()
                    .copied()
                    .find(|&child| self.storage.get_nyes(child) == Nyes::Nk)
                    .map_or_else(|| "unknown".to_string(), |child| self.nk_reason(child)),
            }
        };
        sanitize_reason(&reason)
    }
}

fn canonical_name_pattern(pattern: &str) -> Option<&str> {
    let unwrapped = pattern
        .strip_prefix('^')
        .and_then(|inner| inner.strip_suffix('$'))
        .unwrap_or(pattern);
    (!unwrapped.is_empty()
        && unwrapped
            .chars()
            .all(|ch| ch.is_alphanumeric() || matches!(ch, 'ˍ' | '_' | '\'')))
    .then_some(unwrapped)
}

fn append_before_comment(line: &mut String, suffix: &str) {
    if let Some(index) = line.find("  !!") {
        line.insert_str(index, suffix);
    } else {
        line.push_str(suffix);
    }
}

fn sanitize_reason(reason: &str) -> String {
    let one_line = reason
        .replace(['\n', '\r', '①'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    if one_line.chars().count() <= NK_REASON_LIMIT {
        return one_line;
    }
    let mut truncated = one_line
        .chars()
        .take(NK_REASON_LIMIT.saturating_sub(1))
        .collect::<String>();
    truncated.push('…');
    truncated
}

#[cfg(test)]
mod tests {
    use super::{SequenceMode, SequenceOptions, Ubca2Sequencer, sanitize_reason};
    use crate::fvm_storage::{
        FVMStorage, FirCursor, FirPointer, FirSpec, compose_program_with_system, program_result,
        proto_to_core_fir, step_to_constanic,
    };
    use foolish_core::fir::Nyes;

    fn evaluated_body(source: &str, statement_index: usize) -> (FVMStorage, FirPointer) {
        let mut storage = FVMStorage::new();
        let roots = compose_program_with_system(&mut storage, source).expect("source compiles");
        let composed_root = roots[0];
        step_to_constanic(&mut storage, composed_root).expect("source settles");
        let program = program_result(&storage, composed_root).expect("program result exists");
        let statement = FirCursor::new(program, &storage)
            .stmt_at(statement_index)
            .expect("statement exists");
        let body = FirCursor::new(statement, &storage).foolish_children()[0];
        (storage, body)
    }

    fn evaluated_program(source: &str) -> (FVMStorage, FirPointer) {
        let mut storage = FVMStorage::new();
        let roots = compose_program_with_system(&mut storage, source).expect("source compiles");
        let composed_root = roots[0];
        step_to_constanic(&mut storage, composed_root).expect("source settles");
        let program = program_result(&storage, composed_root).expect("program result exists");
        (storage, program)
    }

    fn assert_foolish_body(source: &str, statement_index: usize, expected: &str) {
        let (storage, fir) = evaluated_body(source, statement_index);
        assert_eq!(
            Ubca2Sequencer::format(&storage, fir, SequenceMode::Foolish),
            expected
        );
    }

    fn evaluate_and_render_program(source: &str) -> String {
        let (storage, program) = evaluated_program(source);
        Ubca2Sequencer::format(&storage, program, SequenceMode::Foolish)
    }

    fn assert_detailed_delegates(source: &str, statement_index: usize) {
        let (storage, fir) = evaluated_body(source, statement_index);
        let core_fir = proto_to_core_fir(&storage, fir);
        let expected = foolish_core::FirSequencer::format(&core_fir);

        assert_eq!(
            Ubca2Sequencer::format(&storage, fir, SequenceMode::Detailed),
            expected
        );
    }

    #[test]
    fn detailed_delegates_for_integer() {
        assert_detailed_delegates("{x=7;}", 0);
    }

    #[test]
    fn detailed_delegates_for_brane() {
        assert_detailed_delegates("{x={a=1;};}", 0);
    }

    #[test]
    fn detailed_delegates_for_operator() {
        assert_detailed_delegates("{x=1+2;}", 0);
    }

    #[test]
    fn detailed_delegates_for_resolved_search() {
        assert_detailed_delegates("{b={x=3;};r=b?x;}", 1);
    }

    #[test]
    fn detailed_delegates_for_nk() {
        assert_detailed_delegates("{x=1/0;}", 0);
    }

    #[test]
    fn foolish_collapses_conclusive_processes_to_values() {
        assert_foolish_body("{x=3+4;}", 0, "7");
        assert_foolish_body("{b={x=3;};r=b?x;}", 1, "3");
    }

    #[test]
    fn foolish_retains_inconclusive_processes_as_source() {
        assert_foolish_body("{x=1/0;}", 0, "1 / 0  !! NK: DIV-BY-ZERO: division by zero");
        assert_foolish_body("{r=missing;}", 0, "missing  !! ECONSTANIC");
        assert_foolish_body(
            "{b={x=3;};r=b?missing;}",
            1,
            "b?missing  !! NK: anchored search found no match",
        );
    }

    #[test]
    fn foolish_preserves_stay_wrappers() {
        assert_foolish_body("{x=1;sf=<x>;sff=<<x>>;}", 1, "<x>");
        assert_foolish_body("{x=1;sf=<x>;sff=<<x>>;}", 2, "<<x>>  !! WOCONSTANIC");
    }

    #[test]
    fn foolish_standardizes_attached_indexes() {
        let (storage, program) = evaluated_program("{b={x=3;};A=b$;}");
        assert_eq!(
            Ubca2Sequencer::format(&storage, program, SequenceMode::Foolish),
            "{b = {x = 3}; A =$ b}"
        );
    }

    #[test]
    fn foolish_renders_branes_and_the_supported_nested_concatenation_case() {
        assert_foolish_body("{x={a=1;b=2;};}", 0, "{a = 1; b = 2}");
        let (storage, program) = evaluated_program("{f=3;a=2;b={f1=f;f2=a;f3=not_found;};}");
        assert_eq!(
            Ubca2Sequencer::format(&storage, program, SequenceMode::Foolish),
            "{\n  f = 3;\n  a = 2;\n  b = {\n    f1 = 3;\n    f2 = 2;\n    f3 = notˍfound  !! ECONSTANIC\n  }\n}",
            "a brane's own derived state is a rollup of its members and must not repeat as a \
             comment on its opening brace — each member already carries its own accurate \
             annotation, so the outer and inner brace lines here stay bare"
        );
    }

    #[test]
    fn foolish_renders_leaves_names_and_indexes_as_foolish_source() {
        assert_foolish_body("{x=-8;}", 0, "-8");
        assert_foolish_body("{x=???;}", 0, "???  !! NK: ??? literal");
        assert_foolish_body("{b={x=3;};r=b#0;}", 1, "3");

        let (storage, program) = evaluated_program("{my_var=2;λ=3;}");
        assert_eq!(
            Ubca2Sequencer::format(&storage, program, SequenceMode::Foolish),
            "{myˍvar = 2; λ = 3}"
        );
    }

    /// §3's Creation and Characterized-brane rows: a bare creation renders
    /// `⬤`; a NAMED creation (FOOP-33's null-characterized statement)
    /// renders its original name; a characterized brane keeps its `a'b'`
    /// prefix. Previously exercised only end-to-end via the einmo contract,
    /// not by a direct unit test.
    #[test]
    fn foolish_renders_creations_and_characterized_branes() {
        // A bare creation renders `⬤`, whether anonymous or the RHS of the
        // null-characterized statement that names it (FOOP-33) — naming
        // does not change what the DEFINING statement's own RHS renders as.
        assert_foolish_body("{x=⬤;}", 0, "⬤");
        assert_foolish_body("{'k=⬤;}", 0, "⬤");

        // A statement that REFERENCES an already-named creation elsewhere
        // renders that creation's original name, not `⬤`.
        let (storage, program) = evaluated_program("{'k=⬤;j='k;}");
        assert_eq!(
            Ubca2Sequencer::format(&storage, program, SequenceMode::Foolish),
            "{'k = ⬤; j = 'k}"
        );

        let (storage, program) = evaluated_program("{characterized=a'b'{v=1};}");
        assert_eq!(
            Ubca2Sequencer::format(&storage, program, SequenceMode::Foolish),
            "{characterized = a'b'{v = 1}}"
        );
    }

    #[test]
    fn foolish_flags_change_only_annotations_and_layout() {
        let (storage, program) = evaluated_program("{x=1/0;}");
        let annotated = Ubca2Sequencer::format(&storage, program, SequenceMode::Foolish);
        let without_nk = Ubca2Sequencer::format_with(
            &storage,
            program,
            &SequenceOptions {
                comment_nk: false,
                ..SequenceOptions::default()
            },
        );
        assert_eq!(
            annotated, "{\n  x = 1 / 0  !! NK: DIV-BY-ZERO: division by zero\n}",
            "the enclosing brane's own derived NK state must not repeat the member's reason \
             on the opening brace"
        );
        assert_eq!(without_nk, "{x = 1 / 0}");

        let (storage, program) = evaluated_program("{a=1;b=2;c=3;}");
        let narrow = Ubca2Sequencer::format_with(
            &storage,
            program,
            &SequenceOptions {
                width: 8,
                ..SequenceOptions::default()
            },
        );
        assert!(
            narrow.contains("\n  a = 1;"),
            "narrow rendering was {narrow}"
        );
    }

    #[test]
    fn foolish_suppress_sequencing_comments_overrides_comment_nk_and_state_annotations() {
        let (storage, program) = evaluated_program("{x=1/0;}");
        let suppressed_nk = Ubca2Sequencer::format_with(
            &storage,
            program,
            &SequenceOptions {
                suppress_sequencing_comments: true,
                ..SequenceOptions::default()
            },
        );
        assert_eq!(
            suppressed_nk, "{x = 1 / 0}",
            "the override must silence NK's annotation just as comment_nk: false does"
        );
        let suppressed_nk_even_when_comment_nk_true = Ubca2Sequencer::format_with(
            &storage,
            program,
            &SequenceOptions {
                comment_nk: true,
                suppress_sequencing_comments: true,
                ..SequenceOptions::default()
            },
        );
        assert_eq!(
            suppressed_nk_even_when_comment_nk_true, "{x = 1 / 0}",
            "suppress_sequencing_comments must win even when comment_nk explicitly asks for \
             NK annotations — it is an override, not a peer flag"
        );

        // A WOCONSTANIC state comment is unconditional under comment_nk (§4) but must still
        // be silenced by the override, proving it is not NK-specific.
        let (storage, program) = evaluated_program("{b={x=3;};r=b?missing;}");
        let annotated = Ubca2Sequencer::format(&storage, program, SequenceMode::Foolish);
        assert!(
            annotated.contains("!!"),
            "sanity: this program normally carries a state comment; got:\n{annotated}"
        );
        let suppressed_state = Ubca2Sequencer::format_with(
            &storage,
            program,
            &SequenceOptions {
                suppress_sequencing_comments: true,
                ..SequenceOptions::default()
            },
        );
        assert!(
            !suppressed_state.contains("!!"),
            "suppress_sequencing_comments must silence state annotations too, not just NK; \
             got:\n{suppressed_state}"
        );
    }

    #[test]
    fn foolish_width_preserves_atoms_and_indents_nested_branes() {
        let source =
            "{outer={an_unsplittable_identifier_that_exceeds_the_budget=1;alpha=2;beta=3;};}";
        let (storage, program) = evaluated_program(source);
        let rendered = Ubca2Sequencer::format_with(
            &storage,
            program,
            &SequenceOptions {
                width: 24,
                ..SequenceOptions::default()
            },
        );
        assert!(
            rendered.contains("outer = {\n"),
            "nested brane did not break: {rendered}"
        );
        assert!(
            rendered.contains("    anˍunsplittableˍidentifierˍthatˍexceedsˍtheˍbudget = 1;"),
            "long atom was altered: {rendered}"
        );
        assert!(
            rendered.contains("\n    alpha = 2;"),
            "nested body was not indented: {rendered}"
        );
    }

    /// T7's remaining two width exceptions (§4.1): an annotation pushing a
    /// line over budget must not be split or otherwise mangled — it is
    /// appended AFTER the width-aware line-breaking decision, never
    /// influencing it (`render_statement` calls `annotate` on the already-
    /// rendered lines) — and a genuinely over-width echoed source statement
    /// must render exactly as written, since Foolish has no
    /// line-continuation syntax to break it with.
    #[test]
    fn foolish_width_exceptions_render_intact_not_mangled() {
        // A narrow width so a short expression's own annotation alone pushes
        // the line over budget, while the expression itself would easily fit.
        let (storage, program) = evaluated_program("{x=1/0;}");
        let annotated = Ubca2Sequencer::format_with(
            &storage,
            program,
            &SequenceOptions {
                width: 8,
                ..SequenceOptions::default()
            },
        );
        assert!(
            annotated.contains("x = 1 / 0  !! NK: DIV-BY-ZERO: division by zero"),
            "an annotation pushing a line over a narrow budget must render intact, not be \
             split across lines or truncated by the WIDTH logic (only the reason's own 60-char \
             cap applies, and this reason is under that): {annotated}"
        );

        // An identifier long enough to exceed even the default 108-column
        // budget on its own, echoed as written source (an operand, not a
        // statement name) rather than broken.
        let long_ident = "an_identifier_that_is_deliberately_constructed_to_exceed_even_the_default_one_hundred_and_eight_column_budget_all_by_itself";
        assert!(long_ident.len() > 108);
        let source = format!("{{sum={long_ident}+1;}}");
        let (storage, program) = evaluated_program(&source);
        let rendered = Ubca2Sequencer::format(&storage, program, SequenceMode::Foolish);
        let expected_operand = long_ident.replace('_', "ˍ");
        assert!(
            rendered.contains(&expected_operand),
            "an over-width echoed operand must render whole, not truncated or split \
             (Foolish has no line-continuation syntax): {rendered}"
        );
    }

    #[test]
    fn foolish_annotations_are_separator_safe() {
        assert_eq!(
            sanitize_reason("first①\nsecond\rthird"),
            "first second third"
        );
        let contract = include_str!("../einmo_suite2/input/foop/36/rendering_contract.foo");
        assert!(!contract.contains('①'));
        assert!(contract.contains("\n  !!!\n"));
        assert!(contract.contains("!!!\n\n  leaves"));
    }

    #[test]
    fn foolish_rendering_round_trips_constanic_programs() {
        for source in [
            "{x=3+4;}",
            "{r=missing;}",
            "{x=1/0;}",
            "{b={x=3;};r=b?missing;}",
            "{b={a=1;b=5;};r=b?=5;}",
            "{b={x=3;};r=b#0;}",
            "{x=1;sf=<x>;sff=<<x>>;}",
            "{x={a=1}{b=2};}",
            "{x=???;}",
            "{f=3;a=2;b={f1=f;f2=a;f3=not_found;};}",
        ] {
            let first = evaluate_and_render_program(source);
            let second = evaluate_and_render_program(&first);
            assert_eq!(second, first, "rendering drifted for {source}: {first}");
        }
    }

    #[test]
    fn foolish_preconstanic_rendering_parses_without_state_syntax() {
        let mut seen = Vec::new();

        for program in ["{a=missing;b=a+absent;}", "{x=1+2;}", "{x={a=1;b=2;};}"] {
            let mut storage = FVMStorage::new();
            let roots =
                compose_program_with_system(&mut storage, program).expect("source compiles");
            let root = roots[0];

            for _ in 0..32 {
                let root_rendered = Ubca2Sequencer::format(&storage, root, SequenceMode::Foolish);
                compose_program_with_system(&mut FVMStorage::new(), &root_rendered)
                    .expect("pre-constanic rendering parses");

                let root_cursor = FirCursor::new(root, &storage);
                let mut candidates = vec![root];
                for statement_index in 0..root_cursor.stmt_count().unwrap_or(0) {
                    if let Some(statement) = root_cursor.stmt_at(statement_index) {
                        candidates.push(statement);
                        candidates.extend(FirCursor::new(statement, &storage).foolish_children());
                    }
                }
                for candidate in candidates {
                    let state = storage.get_nyes(candidate);
                    if !matches!(state, Nyes::Prembrionic | Nyes::Embryonic | Nyes::Braning)
                        || seen.contains(&state)
                    {
                        continue;
                    }
                    let rendered =
                        Ubca2Sequencer::format(&storage, candidate, SequenceMode::Foolish);
                    for line in rendered.lines() {
                        let source = line.split("  !!").next().unwrap_or(line);
                        assert!(
                            ![
                                "PREMBRYONIC",
                                "EMBRYONIC",
                                "BRANING",
                                "ECONSTANIC",
                                "WOCONSTANIC"
                            ]
                            .iter()
                            .any(|token| source.contains(token)),
                            "state leaked into source syntax: {line}"
                        );
                    }
                    seen.push(state);
                }
                if storage.get_nyes(root).is_constanic() {
                    break;
                }
                root.step(&mut storage);
            }
        }

        let mut storage = FVMStorage::new();
        let search = storage.make_root(FirSpec::Search {
            pattern: "^missing$".to_string(),
            anchored: false,
            forward: false,
            is_value_search: false,
            contexted: false,
        });
        search.step(&mut storage);
        assert_eq!(storage.get_nyes(search), Nyes::Embryonic);
        let rendered = Ubca2Sequencer::format(&storage, search, SequenceMode::Foolish);
        let wrapped = format!("{{x={rendered}\n;}}");
        compose_program_with_system(&mut FVMStorage::new(), &wrapped)
            .expect("embryonic search rendering parses");
        assert!(rendered.contains("  !! EMBRYONIC"));
        seen.push(Nyes::Embryonic);
        seen.sort_by_key(|state| match state {
            Nyes::Prembrionic => 0,
            Nyes::Embryonic => 1,
            Nyes::Braning => 2,
            _ => unreachable!("only pre-constanic states are recorded"),
        });

        assert_eq!(
            seen,
            vec![Nyes::Prembrionic, Nyes::Embryonic, Nyes::Braning]
        );
    }

    /// T2b's fourth case: a program halted MID-STEP, short of the iteration
    /// cap that settles it NK (that settled case is covered separately by
    /// `foolish_iteration_alarm_is_a_parseable_nk_annotation`). A bounded,
    /// small number of steps on the same self-referential program leaves the
    /// root pre-constanic — the renderer must still produce parseable
    /// output with no state token as syntax, and must NOT assert
    /// idempotence (§2.1 does not require it of pre-constanic FIR).
    #[test]
    fn foolish_mid_step_snapshot_renders_without_settling() {
        let mut storage = FVMStorage::new();
        let roots = compose_program_with_system(&mut storage, "{\n  f1 = { f1 }\n  stuck = f1;\n}")
            .expect("source compiles");
        let root = roots[0];

        for _ in 0..5 {
            root.step(&mut storage);
        }
        assert!(
            !storage.get_nyes(root).is_constanic(),
            "a handful of steps on a self-referential program must not have reached the \
             iteration cap yet — this is a genuine mid-step snapshot, not the settled case"
        );

        let rendered = Ubca2Sequencer::format(&storage, root, SequenceMode::Foolish);
        for token in [
            "PREMBRYONIC",
            "EMBRYONIC",
            "BRANING",
            "ECONSTANIC",
            "WOCONSTANIC",
        ] {
            assert!(
                !rendered.lines().any(|line| line
                    .split("  !!")
                    .next()
                    .unwrap_or(line)
                    .contains(token)),
                "state leaked into source syntax via {token}: {rendered}"
            );
        }
        compose_program_with_system(&mut FVMStorage::new(), &rendered)
            .expect("mid-step rendering remains parseable Foolish source (Property 1)");
    }

    #[test]
    fn foolish_iteration_alarm_is_a_parseable_nk_annotation() {
        let (storage, roots) = crate::UbcaEvaluator
            .evaluate_arena("{\n  f1 = { f1 }\n  stuck = f1;\n}")
            .expect("evaluation returns an alarm-bearing FIR");
        let rendered = Ubca2Sequencer::format(&storage, roots[0], SequenceMode::Foolish);
        assert_eq!(
            rendered,
            "{  !! NK: Iteration exceeded 9999\n  f1 = {\n    f1  !! ECONSTANIC\n  };\n  stuck = f1  !! BRANING\n}",
            "the alarm is set DIRECTLY on the composed root (system_foo.rs's \
             non_settling_program_renders_nk_with_iteration_alarm regression guard) — the one \
             case where a brane's own annotation is not a redundant rollup of its members, \
             since no member line carries this reason. `stuck`'s own search never got a \
             chance to settle before the cap fired (still pre-constanic, BRANING), so its \
             line stays `stuck = f1` with no borrowed reason of its own; got:\n{rendered}"
        );
        compose_program_with_system(&mut FVMStorage::new(), &rendered)
            .expect("alarm rendering remains Foolish source");
    }

    /// Corpus bug (`foop/33/boolean/comparison_non_integer.foo`): an attached
    /// (`=$`/`=^`) anchor, or any inline operand, that renders MULTI-LINE
    /// (a brane too wide to inline, each member on its own `!!`-annotated
    /// line) must not be collapsed with a naive `.join(" ")` — the first
    /// line's trailing `!!` comment reads to end-of-line and swallows every
    /// line joined after it, including the anchor's own closing `}` and the
    /// statement's `;`. `render_inline`/`render_written_operand_inline`
    /// strip each line's own annotation before joining instead.
    #[test]
    fn foolish_multiline_inline_operands_drop_line_comments_before_joining() {
        let source = "{brane_operand = {1, {x = 5;}, 'lt}$;}";
        let (storage, program) = evaluated_program(source);
        let rendered = Ubca2Sequencer::format(&storage, program, SequenceMode::Foolish);
        assert!(
            !rendered.contains("}  !! NK:") || rendered.trim_end().ends_with('}'),
            "an inner statement's `!!` comment must not appear ahead of the anchor's own \
             closing brace: {rendered}"
        );
        compose_program_with_system(&mut FVMStorage::new(), &rendered).unwrap_or_else(|e| {
            panic!("multi-line attached anchor must remain parseable: {e:?}\n{rendered}")
        });
    }

    /// Corpus bugs (`foop/33/comprehensive.foo`,
    /// `misc/unanchored_seek_with_head_tail.foo`): the attached spelling
    /// (`name =$ anchor`) is only safe when the anchor renders WITHOUT a
    /// leading bare search-operator marker. An unanchored value search
    /// (`?=1`) or an unanchored seek (`#-1`) both render with a leading
    /// marker and no grounding identifier — `is_safe_attached_anchor` must
    /// refuse the attached spelling for those and fall back to the ordinary
    /// postfix form, which parses unambiguously either way.
    #[test]
    fn foolish_attached_form_falls_back_to_postfix_for_unanchored_anchors() {
        for source in ["{a=1;same = ?=a&#-1;}", "{a=1;same = ?=1;}"] {
            let (storage, program) = evaluated_program(source);
            let rendered = Ubca2Sequencer::format(&storage, program, SequenceMode::Foolish);
            compose_program_with_system(&mut FVMStorage::new(), &rendered).unwrap_or_else(|e| {
                panic!("unanchored-anchor statement must not use an ambiguous attached form: {e:?}\n{rendered}")
            });
        }

        let (storage, program) =
            evaluated_program("{a = 1; b = {10; 20; 30}; c = {10; 20; 30}; d = #-1$;}");
        let rendered = Ubca2Sequencer::format(&storage, program, SequenceMode::Foolish);
        assert!(
            !rendered.contains("=$ #-1") && !rendered.contains("=$ #"),
            "an unanchored seek anchor must not be written in the attached spelling, which is \
             ambiguous per FOOP-75 §6: {rendered}"
        );
        compose_program_with_system(&mut FVMStorage::new(), &rendered).unwrap_or_else(|e| {
            panic!(
                "unanchored seek anchor must render as parseable postfix form: {e:?}\n{rendered}"
            )
        });
    }

    /// FOOP-75's own canonical case must still use the attached spelling: a
    /// bare identifier reference (an unanchored NAME search with a canonical
    /// pattern) is safe and is exactly what `foolish_standardizes_attached_indexes`
    /// pins. This test guards the OTHER direction of the same fix — that
    /// tightening the safety check didn't overreach into refusing the safe case.
    #[test]
    fn foolish_attached_form_still_used_for_simple_identifier_anchor() {
        let (storage, program) = evaluated_program("{b={x=3;};A=b$;}");
        assert_eq!(
            Ubca2Sequencer::format(&storage, program, SequenceMode::Foolish),
            "{b = {x = 3}; A =$ b}"
        );
    }

    /// Corpus bug (`misc/concat_sf_f_more.foo`): an SF (`<...>`) whose
    /// interior itself starts or ends with `<`/`>` (from a directly nested
    /// `<<...>>` or `<...>`) must not let its own delimiter fuse with the
    /// interior's — the lexer greedily reads two adjacent `>` characters as
    /// one `GtGt` token (and two adjacent `<` as one `LtLt`), which breaks
    /// whichever wrapper expected a single-character close. A single space
    /// at the boundary prevents the fusion without changing what either
    /// wrapper reads as.
    #[test]
    fn foolish_stay_wrappers_insert_a_space_to_avoid_delimiter_fusion() {
        let source = "{a=1;b=2;c=3; f2={b= <a + <<b>> + <c> >;}; }";
        let (storage, program) = evaluated_program(source);
        let rendered = Ubca2Sequencer::format(&storage, program, SequenceMode::Foolish);
        assert!(
            !rendered.contains(">>>") && !rendered.contains("<<<"),
            "adjacent SF/SFF delimiters must never fuse into a longer run: {rendered}"
        );
        compose_program_with_system(&mut FVMStorage::new(), &rendered).unwrap_or_else(|e| {
            panic!("nested SF/SFF wrappers must remain parseable: {e:?}\n{rendered}")
        });
    }

    /// Corpus bugs (`foop/42/…hfs.foo`, `foop/62/anchored_search_suite.foo`):
    /// per FOOP-75 §6.1 (current, unimplemented §6.2), a parenthesized
    /// regexp pattern like `~(x)` is stored by the PARSER with its parens
    /// included (`pattern == "(x)"`), not stripped. Rendering must write
    /// that pattern back exactly as stored — wrapping it in a SECOND layer
    /// of parens (`?((x))`) is not disambiguation, it produces a stray
    /// unmatched `)` the parser cannot place.
    #[test]
    fn foolish_search_does_not_double_wrap_an_already_parenthesized_pattern() {
        let source = "{hw = {hello=1;world=2;};how_is_not = hw?(ho);}";
        let (storage, program) = evaluated_program(source);
        let rendered = Ubca2Sequencer::format(&storage, program, SequenceMode::Foolish);
        assert!(
            !rendered.contains("?((") && !rendered.contains("~(("),
            "an already-parenthesized pattern must not gain a second wrapping layer: {rendered}"
        );
        compose_program_with_system(&mut FVMStorage::new(), &rendered).unwrap_or_else(|e| {
            panic!("a parenthesized-pattern search must remain parseable: {e:?}\n{rendered}")
        });
    }
}
