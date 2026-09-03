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
}

impl Default for SequenceOptions {
    fn default() -> Self {
        Self {
            mode: SequenceMode::Foolish,
            width: LINE_BUDGET,
            comment_nk: true,
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
            .map(|&child| {
                self.render_expr(child, width, current_stmt, false)
                    .join(" ")
            })
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
        let anchor = anchored.then(|| {
            self.render_written_operand(children[0], width, current_stmt)
                .join(" ")
        });
        let marker = if *forward { "~" } else { "?" };
        let context = if *contexted { "&" } else { "" };

        let query = if *is_value_search {
            let value_index = usize::from(*anchored);
            let value = children
                .get(value_index)
                .map(|&child| {
                    self.render_expr(child, width, current_stmt, false)
                        .join(" ")
                })
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
            format!("{context}{marker}({pattern})")
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
                .map(|&child| {
                    self.render_written_operand(child, width, current_stmt)
                        .join(" ")
                })
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
                self.render_written_operand(child, width.saturating_sub(4), current_stmt)
                    .join(" ")
            })
            .unwrap_or_else(|| "???".to_string());
        if fully {
            vec![format!("<<{inner}>>")]
        } else {
            vec![format!("<{inner}>")]
        }
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
                .map(|&inner| {
                    self.render_written_operand(inner, width, current_stmt)
                        .join(" ")
                })
                .unwrap_or_else(|| "???".to_string());
        }
        self.render_expr(child, width, current_stmt, false)
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
            .replace("{ ", "{")
            .replace(" }", "}")
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
            } if matches!(offset, 0 | -1) && name.is_some() => Some(*offset),
            _ => None,
        };
        let mut lines = if let Some(offset) = attached {
            let anchor = body_cursor
                .foolish_children()
                .first()
                .map(|&child| {
                    self.render_written_operand(child, width, Some(statement))
                        .join(" ")
                })
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

    fn annotate(&self, fir: FirPointer, lines: &mut [String]) {
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
            "{  !! WOCONSTANIC\n  f = 3;\n  a = 2;\n  b = {  !! WOCONSTANIC\n    f1 = 3;\n    f2 = 2;\n    f3 = notˍfound  !! ECONSTANIC\n  }\n}"
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
            annotated,
            "{  !! NK: DIV-BY-ZERO: division by zero\n  x = 1 / 0  !! NK: DIV-BY-ZERO: division by zero\n}"
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
}
