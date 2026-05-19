use crate::*;

/// UBC evaluator adapter for SnapshotSuite.
/// Compiles source via `Compiler`, evaluates to completion with `run_to_completion`.
pub struct UbcEvaluator;

impl Evaluator for UbcEvaluator {
    fn evaluate(&self, source: &str) -> Result<Vec<FirRef>, String> {
        let firs = Compiler::compile(source)
            .map_err(|e| format!("Compilation failed: {}", e))?;
        let mut result = Vec::new();
        for fir in firs {
            let mut fir_ref = fir_to_ref(fir);
            ubc::run_to_completion(&mut fir_ref)
                .map_err(|e| format!("Evaluation failed: {}", e))?;
            result.push(fir_ref);
        }
        Ok(result)
    }
}

#[cfg(test)]
mod approval_tests {
    use super::*;
    use crate::*;
    use std::path::Path;

    fn test_resources_dir() -> &'static str {
        env!("CARGO_MANIFEST_DIR")
    }

    fn run_foo(source: &str) -> String {
        let firs = Compiler::compile(source)
            .unwrap_or_else(|e| panic!("Failed to compile '{}': {}", source, e));
        let mut lines = vec![];
        lines.push(signature::sign_input_line(source));
        lines.push(format!("INPUT: {}", source.lines().next().unwrap_or(source)));
        for (i, fir) in firs.iter().enumerate() {
            lines.push(format!("[{}] PARSED:", i));
            lines.push(Sequencer::format(fir));
            let mut fir_ref = fir_to_ref(fir.clone());
            let result = ubc::run_to_completion(&mut fir_ref);
            let final_fir = clone_steppable(&fir_ref);
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
            .expect(&format!("test input file required: {}", input_path.display()));
        let output = run_foo(&source);
        insta::assert_snapshot!(name, output);
    }

    #[test]
    fn empty_brane() { approval_test("emptyBraneIsApproved"); }

    #[test]
    fn chained_arithmetic() { approval_test("chainedArithmeticIsApproved"); }

    #[test]
    fn anchored_search_on_constant() { approval_test("anchoredSearchOnConstant"); }

    #[test]
    fn scope_resolution() {
        let output = run_foo("{ x = 42; y = x + 8; }");
        insta::assert_snapshot!("scope_resolution", output);
    }

    #[test]
    fn concatenation_basic() {
        let output = run_foo("{ a={p=1;q=2}; b={r=3}; c = a b; }");
        insta::assert_snapshot!("concatenation_basic", output);
    }

    #[test]
    fn concat_shadow_scope() { approval_test("concat_shadow_scope"); }

    #[test]
    fn concat_search_failures() { approval_test("concat_search_failures"); }

    #[test]
    fn sff_basic() {
        let output = run_foo("{a=1,b=2; c=<<a+b>>; c; c;}");
        insta::assert_snapshot!("sff_basic", output);
    }

    #[test]
    fn sff_nested() {
        let output = run_foo("{a=1,b=2; c=<<a+<<b>>>>; c; c;}");
        insta::assert_snapshot!("sff_nested", output);
    }

    #[test]
    fn sf_brane_blocking() {
        let output = run_foo("{a={x=1}; b=<a>; a=42; d=<a>;}");
        insta::assert_snapshot!("sf_brane_blocking", output);
    }

    #[test]
    fn unanchored_seek() {
        let output = run_foo("{a=1,b=2,c=#-1;}");
        insta::assert_snapshot!("unanchored_seek", output);
    }

    #[test]
    fn anchored_seek_positive_negative() {
        let output = run_foo("{a={1,2,3}; a#1; a#-1;}");
        insta::assert_snapshot!("anchored_seek_positive_negative", output);
    }

    #[test]
    fn sff_in_binary_op() {
        let output = run_foo("{a=<<3>>; a + 7;}");
        insta::assert_snapshot!("sff_in_binary_op", output);
    }

    #[test]
    fn sf_non_brane_resolves() {
        let output = run_foo("{x=42; b=<x>; b;}");
        insta::assert_snapshot!("sf_non_brane_resolves", output);
    }

    #[test]
    fn seek_negative_clamping() {
        let output = run_foo("{a=1,b=2; c=#-99;}");
        insta::assert_snapshot!("seek_negative_clamping", output);
    }

    #[test]
    fn seek_beyond_start() {
        let output = run_foo("{a=1; c=#-99;}");
        insta::assert_snapshot!("seek_beyond_start", output);
    }

    #[test]
    fn foop9_operator_search_transparency() {
        let output = run_foo("{x=5, y=7, z=#-2 + #-1;}");
        insta::assert_snapshot!("foop9_operator_search_transparency", output);
    }

    #[test]
    fn foop9_unary_operator() {
        let output = run_foo("{a=-42;}");
        insta::assert_snapshot!("foop9_unary_operator", output);
    }

    #[test]
    fn simple_addition() {
        let output = run_foo("{3 + 4;}");
        insta::assert_snapshot!("simple_addition", output);
    }

    #[test]
    fn simple_subtraction() {
        let output = run_foo("{10 - 3;}");
        insta::assert_snapshot!("simple_subtraction", output);
    }

    #[test]
    fn simple_multiplication() {
        let output = run_foo("{6 * 7;}");
        insta::assert_snapshot!("simple_multiplication", output);
    }

    #[test]
    fn simple_division() {
        let output = run_foo("{15 / 3;}");
        insta::assert_snapshot!("simple_division", output);
    }

    #[test]
    fn zero_division() {
        let output = run_foo("{10 / 0;}");
        insta::assert_snapshot!("zero_division", output);
    }

    #[test]
    fn operator_precedence() {
        let output = run_foo("{2 + 3 * 4 - 5;}");
        insta::assert_snapshot!("operator_precedence", output);
    }

    #[test]
    fn nested_arithmetic() {
        let output = run_foo("{((2 + 3) * (4 - 1)) / 5;}");
        insta::assert_snapshot!("nested_arithmetic", output);
    }

    #[test]
    fn negative_result() {
        let output = run_foo("{5 - 10;}");
        insta::assert_snapshot!("negative_result", output);
    }

    #[test]
    fn mixed_operators() {
        let output = run_foo("{10 + 5 - 3 * 2;}");
        insta::assert_snapshot!("mixed_operators", output);
    }

    #[test]
    fn simple_integer() {
        let output = run_foo("{5;}");
        insta::assert_snapshot!("simple_integer", output);
    }

    #[test]
    fn multiple_expressions() {
        let output = run_foo("{1; 2; 3;}");
        insta::assert_snapshot!("multiple_expressions", output);
    }

    #[test]
    fn multiple_arithmetic_expressions() {
        let output = run_foo("{5 + 3; 10 - 4; 2 * 6;}");
        insta::assert_snapshot!("multiple_arithmetic_expressions", output);
    }

    #[test]
    fn mixed_expressions() {
        let output = run_foo("{42; (3 + 4) * 2; -15; 100 / 5;}");
        insta::assert_snapshot!("mixed_expressions", output);
    }

    #[test]
    fn single_parenthesized_expression() {
        let output = run_foo("{((((5))));}");
        insta::assert_snapshot!("single_parenthesized_expression", output);
    }

    #[test]
    fn simple_unary_minus() {
        let output = run_foo("{-42;}");
        insta::assert_snapshot!("simple_unary_minus", output);
    }

    #[test]
    fn simple_identifier() {
        let output = run_foo("{x = 42; x;}");
        insta::assert_snapshot!("simple_identifier", output);
    }

    #[test]
    fn identifier_in_expression() {
        let output = run_foo("{x = 10; y = 20; x + y;}");
        insta::assert_snapshot!("identifier_in_expression", output);
    }

    #[test]
    fn identifier_shadowing() {
        let output = run_foo("{x = 10; x; x = 20; x;}");
        insta::assert_snapshot!("identifier_shadowing", output);
    }

    #[test]
    fn multiple_identifiers() {
        let output = run_foo("{x = 5; y = 3; z = 2; x * y + z;}");
        insta::assert_snapshot!("multiple_identifiers", output);
    }

    #[test]
    fn undeclared_identifier() {
        let output = run_foo("{x = non_existent;}");
        insta::assert_snapshot!("undeclared_identifier", output);
    }

    #[test]
    fn chained_undeclared() {
        let output = run_foo("{bad = undeclared; y = bad; z = y;}");
        insta::assert_snapshot!("chained_undeclared", output);
    }

    #[test]
    fn nested_branes() {
        let output = run_foo("{5; {10; 15}; 20;}");
        insta::assert_snapshot!("nested_branes", output);
    }

    #[test]
    fn deeply_nested_branes() {
        let output = run_foo("{{{1;}; 2}; 3;}");
        insta::assert_snapshot!("deeply_nested_branes", output);
    }

    #[test]
    fn nested_branes_with_arithmetic() {
        let output = run_foo("{2 + 3; {4 * 5; {6 - 1}; 7 + 8}; 9 / 3;}");
        insta::assert_snapshot!("nested_branes_with_arithmetic", output);
    }

    #[test]
    fn level_skipping_search_found() {
        let output = run_foo("{x = 42; y = x; nested = {z = x};}");
        insta::assert_snapshot!("level_skipping_search_found", output);
    }

    #[test]
    fn level_skipping_search_not_found() {
        let output = run_foo("{x = undeclared; outer = {inner = {y = missing};};}");
        insta::assert_snapshot!("level_skipping_search_not_found", output);
    }

    #[test]
    fn nested_brane_boundary() {
        let output = run_foo("{a = 1; b = {c = #-1; d = 2; e = #-1}; f = #-1;}");
        insta::assert_snapshot!("nested_brane_boundary", output);
    }

    #[test]
    fn simple_regex_search() {
        let output = run_foo("{x = 5; result = {y = 1;}?y;}");
        insta::assert_snapshot!("simple_regex_search", output);
    }

    #[test]
    fn regex_search_pattern() {
        let output = run_foo("{result = {alice = 1; bob = 2; charlie = 3;}?(a.*);}");
        insta::assert_snapshot!("regex_search_pattern", output);
    }

    #[test]
    fn regex_search_not_found() {
        let output = run_foo("{result = {x = 100; y = 200;}?notfound;}");
        insta::assert_snapshot!("regex_search_not_found", output);
    }

    #[test]
    fn regex_search_anchor_start() {
        let output = run_foo("{result = {alice = 1; adam = 2; bob = 3;}?^a;}");
        insta::assert_snapshot!("regex_search_anchor_start", output);
    }

    #[test]
    fn regex_search_anchor_end() {
        let output = run_foo("{result = {alice = 1; charlie = 2; bob = 3;}?e$;}");
        insta::assert_snapshot!("regex_search_anchor_end", output);
    }

    #[test]
    fn assignment_anchor_search() {
        let output = run_foo("{brn = {alice = 2; bob = 3; charlie = 4}; result1 = brn?a.*; result2 = brn?b.*;}");
        insta::assert_snapshot!("assignment_anchor_search", output);
    }

    #[test]
    fn search_pattern_basics() {
        let output = run_foo("{simple = {x=10; y=20; z=30;}?y; notfound = {a=1; b=2;}?missing;}");
        insta::assert_snapshot!("search_pattern_basics", output);
    }

    #[test]
    fn head_tail_basic() {
        let output = run_foo("{x = {10; 20; 30}^; y = {10; 20; 30}$;}");
        insta::assert_snapshot!("head_tail_basic", output);
    }

    #[test]
    fn head_tail_empty_brane() {
        let output = run_foo("{e = {}^; f = {}$;}");
        insta::assert_snapshot!("head_tail_empty_brane", output);
    }

    #[test]
    fn head_tail_on_named_brane() {
        let output = run_foo("{brane = {1; 2; 3}; val = brane^; last = brane$;}");
        insta::assert_snapshot!("head_tail_on_named_brane", output);
    }

    #[test]
    fn head_tail_nested_brane() {
        let output = run_foo("{b = {{1; 2}; {3; 4}}$; c = b^; d = b$;}");
        insta::assert_snapshot!("head_tail_nested_brane", output);
    }

    #[test]
    fn offset_access_forward() {
        let output = run_foo("{data = {a=10; b=20; c=30; d=40; e=50}; first = data#0; second = data#1;}");
        insta::assert_snapshot!("offset_access_forward", output);
    }

    #[test]
    fn offset_access_backward() {
        let output = run_foo("{data = {a=10; b=20; c=30; d=40; e=50}; last = data#-1; second_last = data#-2;}");
        insta::assert_snapshot!("offset_access_backward", output);
    }

    #[test]
    fn offset_access_out_of_bounds() {
        let output = run_foo("{data = {a=10; b=20; c=30; d=40; e=50}; oob = data#5; oob_neg = data#-6;}");
        insta::assert_snapshot!("offset_access_out_of_bounds", output);
    }

    #[test]
    fn offset_access_empty_brane() {
        let output = run_foo("{empty = {}; result = empty#0;}");
        insta::assert_snapshot!("offset_access_empty_brane", output);
    }

    #[test]
    fn unanchored_seek_basic() {
        let output = run_foo("{a = 1; b = 2; c = #-1 + #-2;}");
        insta::assert_snapshot!("unanchored_seek_basic", output);
    }

    #[test]
    fn unanchored_seek_chain() {
        let output = run_foo("{val1 = 3; val2 = 4; val3 = 5; sum = #-1 + #-2 + #-3;}");
        insta::assert_snapshot!("unanchored_seek_chain", output);
    }

    #[test]
    fn unanchored_seek_with_head_tail() {
        let output = run_foo("{a = 1; b = {10; 20; 30}; c = #-1; d = #-1$;}");
        insta::assert_snapshot!("unanchored_seek_with_head_tail", output);
    }

    #[test]
    fn anchored_search_fails_on_constant() {
        let output = run_foo("{b = {x = 100; y = 200}; notFound = b?γ;}");
        insta::assert_snapshot!("anchored_search_fails_on_constant", output);
    }

    #[test]
    fn anchored_search_on_constanic() {
        let output = run_foo("{emptyBrane = {}; chained = emptyBrane^;}");
        insta::assert_snapshot!("anchored_search_on_constanic", output);
    }

    #[test]
    fn concatenation_inline_branes() {
        let output = run_foo("{c = {a=1, b=2, c=3}{e=4, f=5, g=6};}");
        insta::assert_snapshot!("concatenation_inline_branes", output);
    }

    #[test]
    fn concatenation_references() {
        let output = run_foo("{b1={a=1, b=2, c=3}; b2={e=4, f=5, g=6}; c = b1 b2;}");
        insta::assert_snapshot!("concatenation_references", output);
    }

    #[test]
    fn concatenation_mixed() {
        let output = run_foo("{b1={a=1, b=2, c=3}; c = {x=10} b1 {y=20};}");
        insta::assert_snapshot!("concatenation_mixed", output);
    }

    #[test]
    fn concatenation_three_way() {
        let output = run_foo("{b1={a=1, b=2}; b2={c=3, d=4}; b3={e=5, f=6}; c = b1 b2 b3;}");
        insta::assert_snapshot!("concatenation_three_way", output);
    }

    #[test]
    fn alarm_division_by_zero_in_brane() {
        let output = run_foo("{a = 10 / 2; b = 10 / 0; c = 20 / 4;}");
        insta::assert_snapshot!("alarm_division_by_zero_in_brane", output);
    }

    #[test]
    fn alarm_unknown_literal_no_alarm() {
        let output = run_foo("{x = ???; y = 42;}");
        insta::assert_snapshot!("alarm_unknown_literal_no_alarm", output);
    }

    #[test]
    fn alarm_nested_division_by_zero() {
        let output = run_foo("{outer = {inner = 5 / 0};}");
        insta::assert_snapshot!("alarm_nested_division_by_zero", output);
    }

    #[test]
    fn operator_transparency_deep_chain() {
        let output = run_foo("{a=1, b=2, c=3, d=4, e=5; result = #-1 + #-2 + #-3 + #-4 + #-5;}");
        insta::assert_snapshot!("operator_transparency_deep_chain", output);
    }

    #[test]
    fn operator_transparency_mixed_ops() {
        let output = run_foo("{a=1, b=2, c=3; result = #-1 * #-2 + #-3;}");
        insta::assert_snapshot!("operator_transparency_mixed_ops", output);
    }

    #[test]
    fn operator_transparency_unary_in_brane() {
        let output = run_foo("{a=5, b=-a;}");
        insta::assert_snapshot!("operator_transparency_unary_in_brane", output);
    }

    #[test]
    fn complex_negative_results() {
        let output = run_foo("{a = 5 - 10; b = 3 * 2; c = 15 + 7;}");
        insta::assert_snapshot!("complex_negative_results", output);
    }

    #[test]
    fn complex_nested_scope() {
        let output = run_foo("{a = 10; b = 20; outer = {c = 30; ac = a + c; inner = {a = 40; abc = a + b + c}; ab = a + b};}");
        insta::assert_snapshot!("complex_nested_scope", output);
    }

    #[test]
    fn complex_search_and_concatenation() {
        let output = run_foo("{target = {a=1; c=2; c={a=1, b=2, c=3}}; b1 = {x=10}; result = b1 target.c;}");
        insta::assert_snapshot!("complex_search_and_concatenation", output);
    }

    #[test]
    fn complex_sff_in_nested_brane() {
        let output = run_foo("{a=1, b=2; inner = {c = <<a+b>>; c}; inner;}");
        insta::assert_snapshot!("complex_sff_in_nested_brane", output);
    }

    #[test]
    fn complex_sf_in_expression() {
        let output = run_foo("{x=10; y=<x>; z=y + 5;}");
        insta::assert_snapshot!("complex_sf_in_expression", output);
    }

    #[test]
    fn complex_multiple_seeks_in_brane() {
        let output = run_foo("{a=1, b=2, c=3, d=4, e=5; s1 = #-1; s2 = #-2; s3 = #-3;}");
        insta::assert_snapshot!("complex_multiple_seeks_in_brane", output);
    }

    #[test]
    fn complex_brane_with_operations_and_search() {
        let output = run_foo("{x=10; y=20; z=30; sum = x + y + z; avg = sum / 3;}");
        insta::assert_snapshot!("complex_brane_with_operations_and_search", output);
    }

    #[test]
    fn file_shebang() { approval_test("shebang"); }

    #[test]
    fn file_simple_addition() { approval_test("simpleAdditionIsApproved"); }

    #[test]
    fn file_simple_integer() { approval_test("simpleIntegerIsApproved"); }

    #[test]
    fn file_simple_division() { approval_test("simpleDivisionIsApproved"); }

    #[test]
    fn file_zero_division() { approval_test("zeroDivisionIsApproved"); }

    #[test]
    fn file_simple_subtraction() { approval_test("simpleSubtractionIsApproved"); }

    #[test]
    fn file_simple_multiplication() { approval_test("simpleMultiplicationIsApproved"); }

    #[test]
    fn file_simple_unary_minus() { approval_test("simpleUnaryMinusIsApproved"); }

    #[test]
    fn file_operator_precedence() { approval_test("operatorPrecedenceIsApproved"); }

    #[test]
    fn file_nested_arithmetic() { approval_test("nestedArithmeticIsApproved"); }

    #[test]
    fn file_single_expression() { approval_test("singleExpressionIsApproved"); }

    #[test]
    fn file_multiple_expressions() { approval_test("multipleExpressionsIsApproved"); }

    #[test]
    fn file_multiple_arithmetic() { approval_test("multipleArithmeticExpressionsIsApproved"); }

    #[test]
    fn file_mixed_operators() { approval_test("mixedOperatorsIsApproved"); }

    #[test]
    fn file_mixed_expressions() { approval_test("mixedExpressionsIsApproved"); }

    #[test]
    fn file_negative_results() { approval_test("negativeResultsIsApproved"); }

    #[test]
    fn file_simple_identifier() { approval_test("simpleIdentifierIsApproved"); }

    #[test]
    fn file_identifier_in_expression() { approval_test("identifierInExpressionIsApproved"); }

    #[test]
    fn file_identifier_shadowing() { approval_test("identifierShadowingIsApproved"); }

    #[test]
    fn file_multiple_identifiers() { approval_test("multipleIdentifiersIsApproved"); }

    #[test]
    fn file_nested_branes() { approval_test("nestedBranesIsApproved"); }

    #[test]
    fn file_deeply_nested_branes() { approval_test("deeplyNestedBranesIsApproved"); }

    #[test]
    fn file_nested_branes_with_arithmetic() { approval_test("nestedBranesWithArithmeticIsApproved"); }

    #[test]
    fn file_simple_regex_search() { approval_test("simpleRegexSearchIsApproved"); }

    #[test]
    fn file_regex_search_with_pattern() { approval_test("regexSearchWithPatternIsApproved"); }

    #[test]
    fn file_regex_search_not_found() { approval_test("regexSearchNotFoundIsApproved"); }

    #[test]
    fn file_search_pattern_basics() { approval_test("searchPatternBasicsIsApproved"); }

    #[test]
    fn file_search_regex_patterns() { approval_test("searchRegexPatternsIsApproved"); }

    #[test]
    fn file_search_localized_vs_globalized() { approval_test("searchLocalizedVsGlobalizedIsApproved"); }

    #[test]
    fn file_assignment_anchor() { approval_test("assignmentAnchor"); }

    #[test]
    fn file_constantic_rendering() { approval_test("constanticRendering"); }

    #[test]
    fn file_anchored_search_fails_on_constant() { approval_test("anchoredSearchFailsOnConstant"); }

    #[test]
    fn file_anchored_search_on_constanic() { approval_test("anchoredSearchOnConstanic"); }

    #[test]
    fn file_offset_access() { approval_test("offsetAccess"); }

    #[test]
    fn file_unanchored_seek_basic() { approval_test("unanchoredSeekBasic"); }

    #[test]
    fn file_one_shot_search() { approval_test("oneShotSearchIsApproved"); }

    #[test]
    fn file_level_skipping_found() { approval_test("levelSkippingSearchFound"); }

    #[test]
    fn file_level_skipping_constanic() { approval_test("levelSkippingSearchConstanic"); }

    #[test]
    fn file_level_skipping_not_found() { approval_test("levelSkippingSearchNotFound"); }

    #[test]
    fn file_concatenation_basics() { approval_test("concatenationBasics"); }

    #[test]
    fn file_concatenation_search() { approval_test("concatenationSearch"); }

    #[test]
    fn file_test_unanchored_oneshot() { approval_test("test_unanchored_oneshot"); }

    #[test]
    fn file_test_tilde() { approval_test("testTilde"); }

    #[test]
    fn file_test_nested_brane_boundary() { approval_test("test_nested_brane_boundary"); }

    #[test]
    fn file_test_syntax() { approval_test("test_syntax"); }

    #[test]
    fn file_comment_ends_statement() { approval_test("commentEndsStatement"); }

    #[test]
    fn file_regex_search_shadowy() { approval_test("regexSearchShadowy"); }

    #[test]
    fn file_nested_scope_identifier() { approval_test("nestedScopeIdentifierIsApproved"); }

    #[test]
    fn file_nested_scope_shadowing() { approval_test("nestedScopeShadowingIsApproved"); }

    #[test]
    fn file_complex_identifier_scope() { approval_test("complexIdentifierScopeIsApproved"); }

    #[test]
    fn file_four_level_nested_branes() { approval_test("fourLevelNestedBranesWithNamesIsApproved"); }

    #[test]
    fn file_very_deep_nesting() { approval_test("veryDeepNestingIsApproved"); }

    #[test]
    fn large_numbers() {
        let output = run_foo("{a = 1000000; b = 999999; c = a - b;}");
        insta::assert_snapshot!("large_numbers", output);
    }

    #[test]
    fn division_exact_and_remainder() {
        let output = run_foo("{a = 10 / 3; b = 9 / 3;}");
        insta::assert_snapshot!("division_exact_and_remainder", output);
    }

    #[test]
    fn brane_with_single_value_head_tail() {
        let output = run_foo("{b = {42}; h = b^; t = b$;}");
        insta::assert_snapshot!("brane_with_single_value_head_tail", output);
    }

    #[test]
    fn sff_resolves_on_each_use() {
        let output = run_foo("{a=1; b=2; s=<<a+b>>; a=10; s;}");
        insta::assert_snapshot!("sff_resolves_on_each_use", output);
    }

    #[test]
    fn sf_blocks_brane_at_assignment_time() {
        let output = run_foo("{a={x=1, y=2}; s=<a>; a=99; s;}");
        insta::assert_snapshot!("sf_blocks_brane_at_assignment_time", output);
    }

    #[test]
    fn concatenation_with_search_result() {
        let output = run_foo("{target = {a=1, b=2, c={x=10}}; result = {y=5} target.c;}");
        insta::assert_snapshot!("concatenation_with_search_result", output);
    }

    #[test]
    fn seek_in_nested_result_after_concatenation() {
        let output = run_foo("{OB1={a=1; message=2; only=1}; OB2={which_1 = #-1 + 1}; OB = OB1 OB2;}");
        insta::assert_snapshot!("seek_in_nested_result_after_concatenation", output);
    }

    #[test]
    fn chained_unary_operators() {
        let output = run_foo("{a = -(-5); b = -(-(-3));}");
        insta::assert_snapshot!("chained_unary_operators", output);
    }

    #[test]
    fn nested_search_in_brane() {
        let output = run_foo("{x = 42; b = {y = x + 1};}");
        insta::assert_snapshot!("nested_search_in_brane", output);
    }

    #[test]
    fn multiple_concatenation_in_sequence() {
        let output = run_foo("{a={1;2}; b={3;4}; c={5;6}; r1 = a b; r2 = b c; r3 = a c;}");
        insta::assert_snapshot!("multiple_concatenation_in_sequence", output);
    }

    #[test]
    fn search_through_concatenation() {
        let output = run_foo("{b={x=10}; c = b {y=20}; result = c.x + c.y;}");
        insta::assert_snapshot!("search_through_concatenation", output);
    }

    #[test]
    fn unicode_identifiers_basic() {
        let output = run_foo("{x = 1; π = 3; Б = 7; sum = x + π + Б;}");
        insta::assert_snapshot!("unicode_identifiers_basic", output);
    }

    #[test]
    fn seek_zero_offset() {
        let output = run_foo("{b = {10; 20; 30}; first = b#0;}");
        insta::assert_snapshot!("seek_zero_offset", output);
    }

    #[test]
    fn unanchored_seek_large_negative() {
        let output = run_foo("{a=1; b=2; c=3; d=#-100;}");
        insta::assert_snapshot!("unanchored_seek_large_negative", output);
    }

    #[test]
    fn brane_with_operations_and_underscores() {
        let output = run_foo("{my_var = 10; another_var = 20; result = my_var + another_var;}");
        insta::assert_snapshot!("brane_with_operations_and_underscores", output);
    }

    #[test]
    fn forward_reference_basic() {
        let output = run_foo("{y = x; x = 42;}");
        insta::assert_snapshot!("forward_reference_basic", output);
    }

    #[test]
    fn forward_reference_in_nested_brane() {
        let output = run_foo("{outer = {val = x}; x = 100;}");
        insta::assert_snapshot!("forward_reference_in_nested_brane", output);
    }

    #[test]
    fn scope_shadowing_multiple_levels() {
        let output = run_foo("{x = 1; b = {x = 2; c = {x = 3; val = x}; val2 = x}; val3 = x;}");
        insta::assert_snapshot!("scope_shadowing_multiple_levels", output);
    }

    #[test]
    fn cross_scope_reference_chain() {
        let output = run_foo("{a = 1; b = {c = a + 1; d = c + 1};}");
        insta::assert_snapshot!("cross_scope_reference_chain", output);
    }

    #[test]
    fn concatenation_of_empty_branes() {
        let output = run_foo("{a = {}; b = {}; c = a b;}");
        insta::assert_snapshot!("concatenation_of_empty_branes", output);
    }

    #[test]
    fn concatenation_with_single_element() {
        let output = run_foo("{a = {x=1}; b = {y=2}; c = a b;}");
        insta::assert_snapshot!("concatenation_with_single_element", output);
    }

    #[test]
    fn concatenation_repeated_reference() {
        let output = run_foo("{a = {x=1}; c = a a;}");
        insta::assert_snapshot!("concatenation_repeated_reference", output);
    }

    #[test]
    fn concatenation_with_unresolved_search() {
        let output = run_foo("{a = {x=ref}; b = {y=2}; c = a b;}");
        insta::assert_snapshot!("concatenation_with_unresolved_search", output);
    }

    #[test]
    fn operator_with_zero_operands_edge() {
        let output = run_foo("{a = 0 * 100; b = 0 + 999;}");
        insta::assert_snapshot!("operator_with_zero_operands_edge", output);
    }

    #[test]
    fn division_by_zero_in_nested_brane() {
        let output = run_foo("{outer = {inner = {a = 5 / 0; b = 10 / 2};};}");
        insta::assert_snapshot!("division_by_zero_in_nested_brane", output);
    }

    #[test]
    fn operator_chain_with_division_by_zero() {
        let output = run_foo("{a = 10 / 0 * 5;}");
        insta::assert_snapshot!("operator_chain_with_division_by_zero", output);
    }

    #[test]
    fn operator_with_unary_and_binary() {
        let output = run_foo("{a = -3 * 4; b = -3 + 4;}");
        insta::assert_snapshot!("operator_with_unary_and_binary", output);
    }

    #[test]
    fn operator_in_nested_brane_with_scope() {
        let output = run_foo("{x = 10; b = {y = x * 2 + 3};}");
        insta::assert_snapshot!("operator_in_nested_brane_with_scope", output);
    }

    #[test]
    fn head_tail_on_two_element_brane() {
        let output = run_foo("{b = {10; 20}; h = b^; t = b$;}");
        insta::assert_snapshot!("head_tail_on_two_element_brane", output);
    }

    #[test]
    fn head_tail_on_search_result() {
        let output = run_foo("{b = {x = {1; 2; 3}}; h = b.x^; t = b.x$;}");
        insta::assert_snapshot!("head_tail_on_search_result", output);
    }

    #[test]
    fn head_tail_chained_on_nested() {
        let output = run_foo("{b = {{{1; 2}; 3}; 4}; h = b^; hh = b^^;}");
        insta::assert_snapshot!("head_tail_chained_on_nested", output);
    }

    #[test]
    fn seek_from_first_statement() {
        let output = run_foo("{a = 1; b = #-1;}");
        insta::assert_snapshot!("seek_from_first_statement", output);
    }

    #[test]
    fn seek_across_nested_brane_boundary() {
        let output = run_foo("{a = 1; b = 2; inner = {x = #-1}; c = #-1;}");
        insta::assert_snapshot!("seek_across_nested_brane_boundary", output);
    }

    #[test]
    fn anchored_seek_positive_boundary() {
        let output = run_foo("{b = {10; 20; 30}; first = b#0; second = b#1; third = b#2; oob = b#3;}");
        insta::assert_snapshot!("anchored_seek_positive_boundary", output);
    }

    #[test]
    fn anchored_seek_negative_boundary() {
        let output = run_foo("{b = {10; 20; 30}; last = b#-1; second = b#-2; first = b#-3; oob = b#-4;}");
        insta::assert_snapshot!("anchored_seek_negative_boundary", output);
    }

    #[test]
    fn sff_vs_sf_timing_difference() {
        let output = run_foo("{x = 1; sf = <x>; sff = <<x>>; x = 10; sf; sff;}");
        insta::assert_snapshot!("sff_vs_sf_timing_difference", output);
    }

    #[test]
    fn sff_in_assignment_chain() {
        let output = run_foo("{a = 1; b = 2; c = <<a + b>>; a = 100; c;}");
        insta::assert_snapshot!("sff_in_assignment_chain", output);
    }

    #[test]
    fn sf_of_sff() {
        let output = run_foo("{a = 1; b = 2; sff = <<a + b>>; sf = <sff>; a = 10; sf; sff;}");
        insta::assert_snapshot!("sf_of_sff", output);
    }

    #[test]
    fn search_pattern_with_complex_regex() {
        let output = run_foo("{b = {tmp_a = 1; result = 2; tmp_b = 3; output = 4}; r1 = b?tmp_.*; r2 = b?result;}");
        insta::assert_snapshot!("search_pattern_with_complex_regex", output);
    }

    #[test]
    fn search_pattern_matching_nested_brane() {
        let output = run_foo("{b = {a = {x = 1}; b = 2}; r = b?a;}");
        insta::assert_snapshot!("search_pattern_matching_nested_brane", output);
    }

    #[test]
    fn search_with_multiple_matches() {
        let output = run_foo("{b = {a1 = 1; a2 = 2; a3 = 3}; r = b?a.*;}");
        insta::assert_snapshot!("search_with_multiple_matches", output);
    }

    #[test]
    fn alarm_multiple_divisions_by_zero() {
        let output = run_foo("{a = 1 / 0; b = 2 / 0; c = 3 / 0;}");
        insta::assert_snapshot!("alarm_multiple_divisions_by_zero", output);
    }

    #[test]
    fn alarm_division_by_zero_deeply_nested() {
        let output = run_foo("{l1 = {l2 = {l3 = {bad = 1 / 0; good = 42};};};}");
        insta::assert_snapshot!("alarm_division_by_zero_deeply_nested", output);
    }

    #[test]
    fn alarm_mixed_alarms_and_normals() {
        let output = run_foo("{good = 42; bad = 1 / 0; also_good = 10 + 20;}");
        insta::assert_snapshot!("alarm_mixed_alarms_and_normals", output);
    }

    #[test]
    fn complex_full_program_with_all_features() {
        let output = run_foo("{a = 10; b = 20; sum = a + b; nested = {inner = sum / 2}; result = nested.inner;}");
        insta::assert_snapshot!("complex_full_program_with_all_features", output);
    }

    #[test]
    fn complex_search_concat_and_seeks() {
        let output = run_foo("{data = {x=1, y=2, z=3}; copy = data {w=4}; last = copy#-1; first = copy#0;}");
        insta::assert_snapshot!("complex_search_concat_and_seeks", output);
    }

    #[test]
    fn complex_sff_with_nested_scope() {
        let output = run_foo("{x = 5; y = 10; inner = {calc = <<x + y>>; doubled = calc * 2};}");
        insta::assert_snapshot!("complex_sff_with_nested_scope", output);
    }

    #[test]
    fn complex_unanchored_seeks_with_operations() {
        let output = run_foo("{a=10, b=20, c=30, result=#-1 + #-2, result2=#-1 * #-2, result3=#-1 - #-2;}");
        insta::assert_snapshot!("complex_unanchored_seeks_with_operations", output);
    }

    #[test]
    fn complex_forward_refs_in_nested_branes() {
        let output = run_foo("{nested = {inner = {val = x}}; x = 42;}");
        insta::assert_snapshot!("complex_forward_refs_in_nested_branes", output);
    }

    #[test]
    fn complex_brane_with_all_operator_types() {
        let output = run_foo("{add = 1 + 2; sub = 10 - 3; mul = 4 * 5; div = 20 / 4; neg = -7;}");
        insta::assert_snapshot!("complex_brane_with_all_operator_types", output);
    }

    #[test]
    fn complex_concat_with_operations() {
        let output = run_foo("{a = {x=1+2}; b = {y=3*4}; c = a b;}");
        insta::assert_snapshot!("complex_concat_with_operations", output);
    }

    #[test]
    fn named_brane_basic() {
        let output = run_foo("{x = 10; myname'{val = x};}");
        insta::assert_snapshot!("named_brane_basic", output);
    }

    #[test]
    fn named_brane_with_search() {
        let output = run_foo("{x = 42; outer'{inner = {y = x}; val = inner.y};}");
        insta::assert_snapshot!("named_brane_with_search", output);
    }

    #[test]
    fn named_brane_shadowing() {
        let output = run_foo("{x = 100; inner'{x = 200; x}; x;}");
        insta::assert_snapshot!("named_brane_shadowing", output);
    }

    #[test]
    fn regression_regression_disappearing_brane() {
        let output = run_foo("{a = 1; b = 2; c = 3;}");
        insta::assert_snapshot!("regression_regression_disappearing_brane", output);
    }

    #[test]
    fn regression_deep_nesting_does_not_lose_values() {
        let output = run_foo("{{{{x = 42; x};};};}");
        insta::assert_snapshot!("regression_deep_nesting_does_not_lose_values", output);
    }

    #[test]
    fn regression_operator_does_not_block_search() {
        let output = run_foo("{a=1, b=2, c=3, d=4, e=5, f=6; expr = #-1 + #-2 + #-3 + #-4 + #-5 + #-6;}");
        insta::assert_snapshot!("regression_operator_does_not_block_search", output);
    }

    #[test]
    fn crossval_simple_addition() { approval_test("simpleAdditionIsApproved"); }

    #[test]
    fn crossval_simple_subtraction() { approval_test("simpleSubtractionIsApproved"); }

    #[test]
    fn crossval_simple_multiplication() { approval_test("simpleMultiplicationIsApproved"); }

    #[test]
    fn crossval_simple_division() { approval_test("simpleDivisionIsApproved"); }

    #[test]
    fn crossval_simple_integer() { approval_test("simpleIntegerIsApproved"); }

    #[test]
    fn crossval_empty_brane() { approval_test("emptyBraneIsApproved"); }

    #[test]
    fn crossval_chained_arithmetic() { approval_test("chainedArithmeticIsApproved"); }

    #[test]
    fn crossval_simple_identifier() { approval_test("simpleIdentifierIsApproved"); }

    #[test]
    fn crossval_identifier_shadowing() { approval_test("identifierShadowingIsApproved"); }

    #[test]
    fn crossval_nested_branes() { approval_test("nestedBranesIsApproved"); }

    #[test]
    fn crossval_zero_division() { approval_test("zeroDivisionIsApproved"); }

    #[test]
    fn crossval_simple_unary_minus() { approval_test("simpleUnaryMinusIsApproved"); }

    #[test]
    fn crossval_simple_regex_search() { approval_test("simpleRegexSearchIsApproved"); }

    #[test]
    fn crossval_anchored_search_on_constant() { approval_test("anchoredSearchOnConstant"); }

    #[test]
    fn file_complex_arithmetic() { approval_test("complexArithmeticIsApproved"); }

    #[test]
    fn regression_disappearing_brane_statements() {
        let output = run_foo("{a = 1; b = 2; d =$ 4; e = 5; f = 6; g = 7;}");
        insta::assert_snapshot!("regression_disappearing_brane_statements", output);
    }
}

#[cfg(test)]
mod ubc_approval_tests {
    use super::*;
    use crate::*;
    use std::path::PathBuf;

    fn suite() -> SnapshotSuite {
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        SnapshotSuite::new(
            manifest.join("snapshot_tests").join("input"),
            manifest.join("snapshot_tests").join("approved"),
        )
        .expect("SnapshotSuite initialization failed")
    }

    #[test]
    fn approval_all() {
        let suite = suite();
        let evaluations = suite.evaluate_all(num_cpus::get(), &UbcEvaluator);
        let mut settings = insta::Settings::clone_current();
        settings.set_snapshot_path(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("snapshot_tests")
                .join("approved"),
        );
        settings.set_prepend_module_to_snapshot(false);
        settings.set_omit_expression(true);
        settings.bind(|| {
            for (name, result) in evaluations {
                match result {
                    Ok(output) => {
                        insta::assert_snapshot!(format!("{}.foo", name), output);
                    }
                    Err(msg) => {
                        panic!("Evaluation error for {}: {}", name, msg);
                    }
                }
            }
        });
    }
}
