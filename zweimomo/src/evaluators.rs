//! The three pure-Rust [`einmo::Evaluator`] implementations (FOOP-92 §D.3).
//!
//! Each adapter constructs its (`!Send`) interpreter *inside* `evaluate`, per
//! call, so einmo's parallel `evaluate_all` can share one adapter across
//! threads (the adapters are unit structs — `Send + Sync`).

use einmo::Evaluator;

/// Wraps the existing `foolish_ubca::UbcaEvaluator` (used as-is, never
/// modified) and formats each returned FIR via `FirSequencer::format`.
///
/// **Serialization choice:** the idiomatic Foolish rendering is the
/// humanizing-sequencer `hfssnap` output — the same format the project's
/// approval tests already use — one OUTPUT chunk per top-level statement.
#[derive(Debug, Default, Clone, Copy)]
pub struct UbcaEvaluatorAdapter;

impl Evaluator for UbcaEvaluatorAdapter {
    fn evaluate(&self, source: &str) -> Result<Vec<String>, String> {
        use foolish_core::Evaluator as CoreEvaluator;
        let inner = foolish_ubca::UbcaEvaluator;
        let firs = inner.evaluate(source)?;
        Ok(firs
            .iter()
            .map(|fir_ref| {
                let fir = foolish_core::clone_steppable(fir_ref);
                foolish_core::FirSequencer::format(&fir)
            })
            .collect())
    }
}

/// Evaluates Python via `rustpython-vm` 0.5.0 in a sandboxed (`without_stdlib`)
/// interpreter — no `os`/`sys`/file I/O.
///
/// **Serialization choice:** the idiomatic Python rendering of a single
/// expression's value is `str(value)`; the adapter evaluates the source in
/// `Eval` mode and returns one OUTPUT chunk (`str(result)`).
#[derive(Debug, Default, Clone, Copy)]
pub struct RustPythonEvaluator;

impl Evaluator for RustPythonEvaluator {
    fn evaluate(&self, source: &str) -> Result<Vec<String>, String> {
        use rustpython_vm::Interpreter;
        // A fresh, sandboxed interpreter per call (Interpreter is !Send).
        let interp = Interpreter::without_stdlib(Default::default());
        interp.enter(|vm| {
            let scope = vm.new_scope_with_builtins();
            let code = vm
                .compile(
                    source,
                    rustpython_vm::compiler::Mode::Eval,
                    "<zweimomo>".to_owned(),
                )
                .map_err(|err| format!("compile error: {err}"))?;
            let result = vm
                .run_code_obj(code, scope)
                .map_err(|exc| py_exception_message(vm, &exc))?;
            let text = result
                .str(vm)
                .map_err(|exc| py_exception_message(vm, &exc))?
                .to_string_lossy()
                .into_owned();
            Ok(vec![text])
        })
    }
}

/// Render a Python exception into a single-line message.
fn py_exception_message(
    vm: &rustpython_vm::VirtualMachine,
    exc: &rustpython_vm::builtins::PyBaseExceptionRef,
) -> String {
    let mut s = String::new();
    if vm.write_exception(&mut s, exc).is_ok() {
        s.trim().replace('\n', "; ")
    } else {
        "python exception".to_owned()
    }
}

/// Evaluates JavaScript via `boa_engine` 0.21.1 in a fresh `Context` — no
/// `fs`/network/Node APIs.
///
/// **Serialization choice:** the idiomatic JS rendering of a value is
/// `String(value)`; the adapter evaluates the source and returns one OUTPUT
/// chunk (`value.to_string()`).
#[derive(Debug, Default, Clone, Copy)]
pub struct BoaEvaluator;

impl Evaluator for BoaEvaluator {
    fn evaluate(&self, source: &str) -> Result<Vec<String>, String> {
        use boa_engine::{Context, Source};
        // A fresh Context per call (Context is !Send).
        let mut context = Context::default();
        let value = context
            .eval(Source::from_bytes(source))
            .map_err(|err| err.to_string())?;
        let text = value
            .to_string(&mut context)
            .map_err(|err| err.to_string())?
            .to_std_string_escaped();
        Ok(vec![text])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn python_arithmetic_smoke() {
        let out = RustPythonEvaluator.evaluate("2 + 3 * 4 - 5").unwrap();
        assert_eq!(out, vec!["9".to_string()]);
    }

    #[test]
    fn python_integer_division() {
        let out = RustPythonEvaluator.evaluate("7 // 2").unwrap();
        assert_eq!(out, vec!["3".to_string()]);
    }

    #[test]
    fn python_error_is_err_not_panic() {
        let result = RustPythonEvaluator.evaluate("1 / 0");
        assert!(
            result.is_err(),
            "division by zero must be an Err, got {result:?}"
        );
        assert!(result.unwrap_err().to_lowercase().contains("division"));
    }

    #[test]
    fn python_syntax_error_is_err() {
        assert!(RustPythonEvaluator.evaluate("def (").is_err());
    }

    #[test]
    fn js_arithmetic_smoke() {
        let out = BoaEvaluator.evaluate("2 + 3 * 4 - 5").unwrap();
        assert_eq!(out, vec!["9".to_string()]);
    }

    #[test]
    fn js_floor_division_matches_integer() {
        let out = BoaEvaluator.evaluate("Math.floor(7 / 2)").unwrap();
        assert_eq!(out, vec!["3".to_string()]);
    }

    #[test]
    fn js_divide_by_zero_is_infinity_value() {
        // JS `10/0` is `Infinity` — a *value*, not an error (documents the
        // cross-language asymmetry).
        let out = BoaEvaluator.evaluate("10 / 0").unwrap();
        assert_eq!(out, vec!["Infinity".to_string()]);
    }

    #[test]
    fn js_throw_is_err() {
        assert!(BoaEvaluator.evaluate("throw new Error('boom')").is_err());
    }

    #[test]
    fn foolish_arithmetic_produces_output() {
        let out = UbcaEvaluatorAdapter.evaluate("{3 + 4;}").unwrap();
        assert_eq!(out.len(), 1);
        assert!(
            !out[0].trim().is_empty(),
            "hfssnap output must be non-empty"
        );
    }

    #[test]
    fn foolish_parse_error_is_err() {
        // An unmatched brace should fail compilation → Err, not panic.
        let result = UbcaEvaluatorAdapter.evaluate("{ this is not foolish ((((");
        assert!(
            result.is_err(),
            "malformed Foolish must be an Err, got {result:?}"
        );
    }
}
