//! Three `einmo::Evaluator` implementations: Foolish, Python, JavaScript.

use einmo::Evaluator;
use foolish_core::Evaluator as FoolishEvaluator;
use foolish_core::FirSequencer;
use foolish_ubca::UbcaEvaluator;

// ─────────────────────────────────────────────────────────────────────────────
// UbcaEvaluatorAdapter — wraps the existing foolish-ubca evaluator
// ─────────────────────────────────────────────────────────────────────────────

/// Adapts `foolish_ubca::UbcaEvaluator` to `einmo::Evaluator`.
///
/// Evaluates Foolish source via the UBCa VM, then formats each returned FIR
/// through `FirSequencer::format` (the humanizing sequencer). This produces
/// hfssnap-style output — one formatted block per top-level statement.
pub struct UbcaEvaluatorAdapter;

impl Evaluator for UbcaEvaluatorAdapter {
    fn evaluate(&self, source: &str) -> Result<Vec<String>, String> {
        let inner = UbcaEvaluator;
        let firs = inner.evaluate(source)?;
        Ok(firs
            .iter()
            .map(|fir_ref| {
                let fir = foolish_core::clone_steppable(fir_ref);
                FirSequencer::format(&fir)
            })
            .collect())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// RustPythonEvaluator — pure-Rust Python via rustpython-vm 0.5.0
// ─────────────────────────────────────────────────────────────────────────────

/// Evaluates Python source via `rustpython-vm` (sandboxed: no stdlib).
///
/// Each call constructs a fresh `Interpreter` (not `Send`) so parallel
/// `evaluate_all` threads each own their interpreter. The result is
/// stringified via the Python `str()` protocol — idiomatic for Python values.
pub struct RustPythonEvaluator;

impl Evaluator for RustPythonEvaluator {
    fn evaluate(&self, source: &str) -> Result<Vec<String>, String> {
        let interp = rustpython_vm::Interpreter::without_stdlib(Default::default());
        interp.enter(|vm| {
            let scope = vm.new_scope_with_builtins();
            let result = rustpython_vm::eval::eval(vm, source, scope, "<zweimomo>")
                .map_err(|e| format!("{e:?}"))?;
            let py_str = result.str(vm).map_err(|e| format!("{e:?}"))?;
            let s = py_str.to_str().unwrap_or("<non-utf8>");
            Ok(vec![s.to_string()])
        })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// BoaEvaluator — pure-Rust JavaScript via boa_engine 0.21.1
// ─────────────────────────────────────────────────────────────────────────────

/// Evaluates JavaScript source via `boa_engine` (sandboxed: no fs/network).
///
/// Each call constructs a fresh `Context` (not `Send`). The result is
/// stringified via the JS `toString()` protocol — idiomatic for JS values.
pub struct BoaEvaluator;

impl Evaluator for BoaEvaluator {
    fn evaluate(&self, source: &str) -> Result<Vec<String>, String> {
        let mut context = boa_engine::Context::default();
        let result = context
            .eval(boa_engine::Source::from_bytes(source.as_bytes()))
            .map_err(|e| e.to_string())?;
        let s = result
            .to_string(&mut context)
            .map_err(|e| e.to_string())?
            .to_std_string_escaped();
        Ok(vec![s])
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// brane_name_perspective — extracts names, replaces values with ???
// ─────────────────────────────────────────────────────────────────────────────

/// Extracts ordinate names from a Foolish brane and replaces values with `???`.
///
/// `{a=1,b=2,c=3}` → `{a=???,b=???,c=???}`
///
/// This is a pure string transformation — no parsing, no evaluation. It handles
/// nested branes (values containing `{…}`) by tracking brace depth. Designed
/// for use as a `Perspective` in `TestConfig`.
pub fn brane_name_perspective(input: &str) -> String {
    let bytes = input.as_bytes();
    let len = bytes.len();
    let mut result = String::with_capacity(len);
    let mut i = 0;

    while i < len {
        if bytes[i].is_ascii_alphabetic() || bytes[i] == b'_' {
            // Read identifier
            let ident_start = i;
            i += 1;
            while i < len && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            let ident = &input[ident_start..i];

            // Capture optional whitespace before '='
            let ws_start = i;
            while i < len && bytes[i] == b' ' {
                i += 1;
            }
            let whitespace = &input[ws_start..i];

            // Check for '=' (but not '==' or '=$')
            if i < len && bytes[i] == b'=' && i + 1 < len && bytes[i + 1] != b'=' {
                result.push_str(ident);
                result.push_str(whitespace);
                result.push('=');
                i += 1; // skip '='
                // Preserve whitespace between '=' and value
                let ws_after = i;
                while i < len && bytes[i] == b' ' {
                    i += 1;
                }
                result.push_str(&input[ws_after..i]);
                result.push_str("???");
                skip_past_value(bytes, &mut i);
            } else {
                result.push_str(ident);
                result.push_str(whitespace);
            }
        } else {
            // Copy one byte (safe for ASCII punctuation used in Foolish syntax;
            // multi-byte UTF-8 characters pass through byte-by-byte which is
            // correct since none of their continuation bytes match the
            // identifier-start predicate).
            result.push(input[i..].chars().next().unwrap_or('?'));
            i += input[i..].chars().next().map_or(1, |c| c.len_utf8());
        }
    }

    result
}

/// Skip past a value after `=`: a nested brane `{…}`, or characters until the
/// next delimiter (`,`, `;`, `}`). Leading whitespace is already consumed.
fn skip_past_value(bytes: &[u8], i: &mut usize) {
    if *i >= bytes.len() {
        return;
    }
    if bytes[*i] == b'{' {
        // Skip nested brane, tracking depth
        let mut depth = 1usize;
        *i += 1;
        while *i < bytes.len() && depth > 0 {
            match bytes[*i] {
                b'{' => depth += 1,
                b'}' => depth -= 1,
                _ => {}
            }
            if depth > 0 {
                *i += 1;
            }
        }
        if *i < bytes.len() {
            *i += 1; // skip closing '}'
        }
    } else {
        // Skip until delimiter
        while *i < bytes.len() && !matches!(bytes[*i], b',' | b';' | b'}') {
            *i += 1;
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Python evaluator ──────────────────────────────────────────────────

    #[test]
    fn python_integer_arithmetic() {
        let eval = RustPythonEvaluator;
        let result = eval.evaluate("1 + 2").expect("should succeed");
        assert_eq!(result, vec!["3"]);
    }

    #[test]
    fn python_error_returns_err() {
        let eval = RustPythonEvaluator;
        let result = eval.evaluate("def");
        assert!(result.is_err(), "Python syntax error should return Err");
    }

    // ── JavaScript evaluator ───────────────────────────────────────────────

    #[test]
    fn js_integer_arithmetic() {
        let eval = BoaEvaluator;
        let result = eval.evaluate("1 + 2").expect("should succeed");
        assert_eq!(result, vec!["3"]);
    }

    #[test]
    fn js_error_returns_err() {
        let eval = BoaEvaluator;
        let result = eval.evaluate("throw 'error'");
        assert!(result.is_err(), "JS throw should return Err");
    }

    // ── Foolish evaluator ─────────────────────────────────────────────────

    #[test]
    fn foolish_brane_evaluates() {
        let eval = UbcaEvaluatorAdapter;
        let result = eval.evaluate("{3 + 4;}").expect("should succeed");
        assert!(!result.is_empty(), "should produce non-empty output");
        assert!(!result[0].is_empty(), "hfssnap output should be non-empty");
    }

    #[test]
    fn foolish_error_returns_err() {
        let eval = UbcaEvaluatorAdapter;
        // An unbalanced brace should fail compilation
        let result = eval.evaluate("{");
        assert!(result.is_err(), "Foolish parse error should return Err");
    }

    // ── brane_name_perspective ─────────────────────────────────────────────

    #[test]
    fn brane_name_perspective_simple() {
        let input = "{a=1,b=2,c=3}";
        let expected = "{a=???,b=???,c=???}";
        assert_eq!(brane_name_perspective(input), expected);
    }

    #[test]
    fn brane_name_perspective_spaced() {
        let input = "{a = 1; b = 2; c = 3;}";
        let expected = "{a = ???; b = ???; c = ???;}";
        assert_eq!(brane_name_perspective(input), expected);
    }

    #[test]
    fn brane_name_perspective_nested_brane_value() {
        let input = "{a = {inner = 1;}; b = 2;}";
        let expected = "{a = ???; b = ???;}";
        assert_eq!(brane_name_perspective(input), expected);
    }

    #[test]
    fn brane_name_perspective_no_bindings() {
        let input = "{3 + 4;}";
        // No name= patterns, so the string passes through unchanged
        assert_eq!(brane_name_perspective(input), input);
    }
}
