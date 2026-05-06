pub mod fir;
pub mod serialization;
pub mod compiler;
pub mod ubc;
pub mod search;
pub mod sequencer;

pub use fir::*;
pub use serialization::*;
pub use compiler::*;
pub use ubc::*;
pub use sequencer::*;

#[cfg(test)]
mod approval_tests {
    use super::*;
    use std::cell::RefCell;
    use std::path::Path;
    use std::rc::Rc;

    fn test_resources_dir() -> &'static str {
        env!("CARGO_MANIFEST_DIR")
    }

    fn run_foo(source: &str) -> String {
        let firs = Compiler::compile(source).unwrap();
        let mut lines = vec![format!("INPUT: {}", source.lines().next().unwrap_or(source))];
        for (i, fir) in firs.iter().enumerate() {
            lines.push(format!("[{}] PARSED:", i));
            lines.push(Sequencer::format(fir));
            // Evaluate
            let mut fir_ref = Rc::new(RefCell::new(fir.clone()));
            let result = ubc::run_to_completion(&mut fir_ref);
            let final_fir = fir_ref.borrow().clone();
            lines.push("RESULT:".to_string());
            lines.push(Sequencer::format(&final_fir));
            if let Err(e) = result {
                lines.push(format!("ERROR: {}", e));
            }
        }
        lines.join("\n")
    }

    fn approval_test(name: &str) {
        let input_dir = format!("{}/../../test-resources/org/foolish/fvm/inputs", test_resources_dir());
        let input_path = Path::new(&input_dir).join(format!("{}.foo", name));
        let source = std::fs::read_to_string(&input_path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {}", input_path.display(), e));
        let output = run_foo(&source);
        insta::assert_snapshot!(name, output);
    }

    #[test]
    fn empty_brane() { approval_test("emptyBraneIsApproved"); }

    #[test]
    fn chained_arithmetic() { approval_test("chainedArithmeticIsApproved"); }

    #[test]
    fn anchored_search_on_constant() { approval_test("anchoredSearchOnConstant"); }
}
