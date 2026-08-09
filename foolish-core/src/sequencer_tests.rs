use crate::fir::{
    Alarm, AlarmLevel, AlarmSource, ConcatenationFirBuilder, ConstantIntFirBuilder,
    CreationFirBuilder, IndexFirBuilder, NkFirBuilder, NormalBraneFirBuilder, Nyes,
    OperatorFirBuilder, SearchDirection, SearchFirBuilder, StayFoolishFirBuilder,
    StayFullyFoolishFirBuilder,
};
use crate::sequencer::format_fir_simple;
use crate::*;

#[test]
fn test_hs_variant_all_types() {
    assert_eq!(
        ConstantIntFirBuilder::new(42).build().hs_variant(),
        "ConstantInt"
    );
    assert_eq!(NkFirBuilder::new("unknown").build().hs_variant(), "Nk");
    assert_eq!(
        OperatorFirBuilder::new("+").build().hs_variant(),
        "Operator"
    );
    assert_eq!(SearchFirBuilder::new("x").build().hs_variant(), "Search");
    assert_eq!(IndexFirBuilder::new(1).build().hs_variant(), "Index");
    assert_eq!(
        StayFoolishFirBuilder::new(ConstantIntFirBuilder::new(1).build())
            .build()
            .hs_variant(),
        "StayFoolish"
    );
    assert_eq!(
        StayFullyFoolishFirBuilder::new(ConstantIntFirBuilder::new(1).build())
            .build()
            .hs_variant(),
        "StayFullyFoolish"
    );
    assert_eq!(
        ConcatenationFirBuilder::new().build().hs_variant(),
        "Concatenation"
    );
    assert_eq!(
        NormalBraneFirBuilder::new().build().hs_variant(),
        "NormalBrane"
    );
}

#[test]
fn test_format_empty_brane() {
    let brane = NormalBraneFirBuilder::new().state(Nyes::Constant).build();
    assert_eq!(format_fir_simple(&brane), "{}");
}

#[test]
fn test_format_constant_int() {
    let c = ConstantIntFirBuilder::new(42).build();
    assert_eq!(format_fir_simple(&c), "42");
}

#[test]
fn test_format_named_statement() {
    let body = ConstantIntFirBuilder::new(42).build();
    let brane = NormalBraneFirBuilder::new()
        .statement(Some("x".into()), body)
        .build();
    let s = format_fir_simple(&brane);
    assert!(s.contains("x=42"), "Expected 'x=42' in: {}", s);
}

#[test]
fn test_format_multi_statement() {
    let s = vec![
        (Some("a".into()), ConstantIntFirBuilder::new(1).build()),
        (Some("b".into()), ConstantIntFirBuilder::new(2).build()),
    ];
    let brane = NormalBraneFirBuilder::new().statements(s).build();
    let out = format_fir_simple(&brane);
    assert!(out.contains("a=1"), "Expected 'a=1' in: {}", out);
    assert!(out.contains("b=2"), "Expected 'b=2' in: {}", out);
}

#[test]
fn test_format_search() {
    let search = SearchFirBuilder::new("x")
        .direction(SearchDirection::Backward)
        .state(Nyes::Econstanic)
        .build();
    let s = format_fir_simple(&search);
    assert!(s.starts_with("?("), "Expected ?( in: {}", s);
    assert!(s.contains("pattern='x'"), "Expected pattern='x' in: {}", s);
}

#[test]
fn test_format_nk() {
    let nk = NkFirBuilder::new("unknown").build();
    let s = format_fir_simple(&nk);
    assert!(s.starts_with("???"), "Expected ??? in: {}", s);
}

#[test]
fn test_format_nk_with_alarm() {
    let alarm = Alarm {
        level: AlarmLevel::Mild,
        code: "TEST".to_string(),
        message: "test alarm".to_string(),
        source: AlarmSource::Evaluator,
    };
    let nk = NkFirBuilder::new("div-by-zero").alarm(alarm).build();
    let s = format_fir_simple(&nk);
    assert!(s.starts_with("???"), "Expected ??? in: {}", s);
    assert!(s.contains("test alarm"), "Expected alarm message in: {}", s);
}

#[test]
fn test_format_operator() {
    let op = OperatorFirBuilder::new("+")
        .operand(ConstantIntFirBuilder::new(1).build())
        .operand(ConstantIntFirBuilder::new(2).build())
        .state(Nyes::Constant)
        .build();
    let s = format_fir_simple(&op);
    // CONSTANT state operator is transparent - renders computed first operand value
    assert!(
        s.contains("1"),
        "Expected '1' (transparent CONSTANT) in: {}",
        s
    );
}

#[test]
fn test_format_concatenation() {
    let conc = ConcatenationFirBuilder::new()
        .element(ConstantIntFirBuilder::new(1).build())
        .element(ConstantIntFirBuilder::new(2).build())
        .state(Nyes::Constant)
        .build();
    let s = format_fir_simple(&conc);
    // CONSTANT concatenation renders as ⨃{...} brane form
    assert!(s.starts_with("⨃{"), "Expected ⨃{{ in: {}", s);
    assert!(s.contains("1"), "Expected '1' in: {}", s);
    assert!(s.contains("2"), "Expected '2' in: {}", s);
}

#[test]
fn test_format_concatenation_merged() {
    let conc = ConcatenationFirBuilder::new()
        .element(ConstantIntFirBuilder::new(1).build())
        .merged(ConstantIntFirBuilder::new(99).build())
        .build();
    let s = format_fir_simple(&conc);
    // Embryonic + no result → no nyes → transparent: renders merged with ⨃ prefix
    assert!(s.starts_with("⨃"), "Expected ⨃ prefix in: {}", s);
    assert!(s.contains("99"), "Expected '99' in: {}", s);
}

#[test]
fn test_format_index() {
    let idx = IndexFirBuilder::new(1).build();
    let s = format_fir_simple(&idx);
    // Per should_show_search_nyes (FOOP-62 §9.x sequencer HFS rule): a search-like FIR
    // with NO result in EMBRYONIC hides its NYES. A freshly-built unanchored IndexFir
    // is EMBRYONIC with no result, so the state is NOT rendered.
    assert!(
        s.contains("#(offset=1, UNANCHORED)"),
        "Expected '#(offset=1, UNANCHORED)' in: {}",
        s
    );
    assert!(
        !s.contains("EMBRYONIC"),
        "EMBRYONIC must be hidden for a no-result EMBRYONIC index, got: {}",
        s
    );
}

#[test]
fn test_format_index_anchored() {
    let idx = IndexFirBuilder::new(0).anchored(true).build();
    let s = format_fir_simple(&idx);
    assert!(s.starts_with("^("), "Expected '^(' in: {}", s);
}

#[test]
fn test_format_headtail_head() {
    let idx = IndexFirBuilder::new(0).anchored(true).build();
    let s = format_fir_simple(&idx);
    assert!(s.starts_with("^("), "Expected '^(' in: {}", s);
}

#[test]
fn test_format_headtail_tail_anchored() {
    let idx = IndexFirBuilder::new(-1).anchored(true).build();
    let s = format_fir_simple(&idx);
    assert!(s.starts_with("$("), "Expected '$(' in: {}", s);
}

#[test]
fn test_format_stay_foolish() {
    let sf = StayFoolishFirBuilder::new(ConstantIntFirBuilder::new(1).build()).build();
    let s = format_fir_simple(&sf);
    // Embryonic + no result → no nyes → transparent: renders inner expression
    assert!(s.contains("1"), "Expected '1' in: {}", s);
}

#[test]
fn test_format_stay_fully_foolish() {
    let sff = StayFullyFoolishFirBuilder::new(ConstantIntFirBuilder::new(2).build()).build();
    let s = format_fir_simple(&sff);
    // Embryonic + no result → no nyes → transparent: renders inner expression
    assert!(s.contains("2"), "Expected '2' in: {}", s);
}

// These tests exercise the shared sequencer over builder-constructed `Fir`
// trees. (They formerly compiled `.foo` source through the retired UBC
// compiler; UBCa is now the sole engine and lives in a downstream crate, so
// foolish-core builds the FIR directly via the shared builders instead.)
#[test]
fn test_integration_statement_with_operator_format() {
    // Equivalent of `{x = 1 + 2}` before evaluation: statement `x` whose body
    // is an Embryonic `+` operator over 1 and 2.
    let op = OperatorFirBuilder::new("+")
        .operand(ConstantIntFirBuilder::new(1).build())
        .operand(ConstantIntFirBuilder::new(2).build())
        .build();
    let brane = NormalBraneFirBuilder::new()
        .statement(Some("x".into()), op)
        .build();
    let formatted = format_fir_simple(&brane);

    assert!(formatted.contains("x="), "Expected 'x=' in: {}", formatted);
    // Operator is Embryonic → shows full operator with operands
    assert!(
        formatted.contains("Op+("),
        "Expected 'Op+(' in: {}",
        formatted
    );
}

#[test]
fn test_integration_multi_statement_roundtrip() {
    // Equivalent of `{a = 1; b = 2; c = 3}`.
    let brane = NormalBraneFirBuilder::new()
        .statements(vec![
            (Some("a".into()), ConstantIntFirBuilder::new(1).build()),
            (Some("b".into()), ConstantIntFirBuilder::new(2).build()),
            (Some("c".into()), ConstantIntFirBuilder::new(3).build()),
        ])
        .build();
    let formatted = format_fir_simple(&brane);

    assert!(
        formatted.contains("a=1"),
        "Expected 'a=1' in: {}",
        formatted
    );
    assert!(
        formatted.contains("b=2"),
        "Expected 'b=2' in: {}",
        formatted
    );
    assert!(
        formatted.contains("c=3"),
        "Expected 'c=3' in: {}",
        formatted
    );
}

#[test]
fn test_sequencer_ref_format_constant() {
    let fir = ConstantIntFirBuilder::new(42).build();
    let ref_seq = HumanizingFirSequencerRef::new(&fir);
    let out = ref_seq.format_for_snap_test();
    assert!(out.contains("42"), "Expected 42 in: {}", out);
}

#[test]
fn test_sequencer_format_with_header() {
    let fir = ConstantIntFirBuilder::new(42).build();
    let out = FirSequencer::format_with_header("{42}", &fir, 0);
    assert!(out.contains("INPUT:"));
    assert!(out.contains("STEPS:"));
}

// ── FOOP-33 Phase 9: sequencer renders creation names ──

#[test]
fn test_format_unnamed_creation_renders_glyph() {
    let fir = CreationFirBuilder::new().build();
    assert_eq!(format_fir_simple(&fir), "\u{2B24}");
}

#[test]
fn test_format_named_creation_renders_its_name_not_the_glyph() {
    let fir = CreationFirBuilder::new().name("'a").build();
    assert_eq!(format_fir_simple(&fir), "'a");
}

#[test]
fn test_format_named_creation_as_statement_body() {
    // `{ 'a = <creation named 'a> }` — the realistic shape: a creation that
    // names itself sitting as the body of the very statement it names.
    let creation = CreationFirBuilder::new().name("'a").build();
    let brane = NormalBraneFirBuilder::new()
        .statement(Some("'a".into()), creation)
        .build();
    let formatted = format_fir_simple(&brane);
    assert!(
        formatted.contains("'a='a"),
        "Expected \"'a='a\" in: {}",
        formatted
    );
}

#[test]
fn test_format_comparison_result_renders_true_false_names() {
    // The Phase 9 payoff for FOOP-33's own comparison operators (Phase 6):
    // a comparison settles to the ACTUAL 'True/'False creation, which now
    // renders by name instead of the bare glyph.
    let true_creation = CreationFirBuilder::new().name("'True").build();
    let false_creation = CreationFirBuilder::new().name("'False").build();
    assert_eq!(format_fir_simple(&true_creation), "'True");
    assert_eq!(format_fir_simple(&false_creation), "'False");
}

// ── FOOP-65 Phase 2: sequencer backtick rendering ──────────────────────

#[test]
fn tail_concat_embryonic_renders_backtick() {
    // Use NormalBraneFirBuilder with Embryonic state so elements pass the
    // all-embryonic gate (Prembrionic || Embryonic).
    let a = NormalBraneFirBuilder::new().state(Nyes::Embryonic).build();
    let b = NormalBraneFirBuilder::new().state(Nyes::Embryonic).build();
    let c = NormalBraneFirBuilder::new().state(Nyes::Embryonic).build();
    let cat = ConcatenationFirBuilder::new()
        .element(a)
        .element(b)
        .element(c)
        .state(Nyes::Embryonic)
        .is_tail_concatenation(true)
        .build();
    let rendered = format_fir_simple(&cat);
    assert!(
        rendered.contains('`'),
        "tail-flagged all-embryonic should render with backtick: {}",
        rendered
    );
    assert!(
        rendered.starts_with("⨃{"),
        "should have concatenation prefix: {}",
        rendered
    );
}

#[test]
fn juxta_concat_embryonic_renders_ordinary() {
    let a = ConstantIntFirBuilder::new(1).build();
    let b = ConstantIntFirBuilder::new(2).build();
    let c = ConstantIntFirBuilder::new(3).build();
    let cat = ConcatenationFirBuilder::new()
        .element(a)
        .element(b)
        .element(c)
        .state(Nyes::Embryonic)
        .is_tail_concatenation(false)
        .build();
    let rendered = format_fir_simple(&cat);
    assert!(
        !rendered.contains('`'),
        "juxtaposition should NOT render with backtick: {}",
        rendered
    );
    assert!(
        rendered.starts_with("⨃("),
        "should show proto-brane form: {}",
        rendered
    );
}

#[test]
fn tail_concat_settled_renders_identical_to_juxta() {
    let a = ConstantIntFirBuilder::new(10).build();
    let b = ConstantIntFirBuilder::new(20).build();
    let merged = NormalBraneFirBuilder::new()
        .statement(Some("x".into()), ConstantIntFirBuilder::new(10).build())
        .statement(Some("y".into()), ConstantIntFirBuilder::new(20).build())
        .state(Nyes::Constant)
        .build();

    let tail = ConcatenationFirBuilder::new()
        .element(ConstantIntFirBuilder::new(10).build())
        .element(ConstantIntFirBuilder::new(20).build())
        .merged(merged.clone())
        .state(Nyes::Constant)
        .is_tail_concatenation(true)
        .build();
    let juxta = ConcatenationFirBuilder::new()
        .element(a)
        .element(b)
        .merged(merged)
        .state(Nyes::Constant)
        .is_tail_concatenation(false)
        .build();

    let tail_s = format_fir_simple(&tail);
    let juxta_s = format_fir_simple(&juxta);
    assert_eq!(tail_s, juxta_s, "settled forms must be byte-identical");
}

/// Temporary: print the all-embryonic rendering for FOOP-65 §6.1 inspection.
/// Remove after human review.
#[test]
fn foop65_embryonic_inspection() {
    use crate::fir::*;
    use crate::Nyes;

    // §3.1 worked example: a`b`c`d e f — all elements Embryonic
    let d = NormalBraneFirBuilder::new().state(Nyes::Embryonic).build();
    let e = NormalBraneFirBuilder::new().state(Nyes::Embryonic).build();
    let f = NormalBraneFirBuilder::new().state(Nyes::Embryonic).build();
    let inner = ConcatenationFirBuilder::new()
        .element(d)
        .element(e)
        .element(f)
        .state(Nyes::Embryonic)
        .build();
    let a = NormalBraneFirBuilder::new().state(Nyes::Embryonic).build();
    let b = NormalBraneFirBuilder::new().state(Nyes::Embryonic).build();
    let c = NormalBraneFirBuilder::new().state(Nyes::Embryonic).build();
    let outer = ConcatenationFirBuilder::new()
        .element(inner)
        .element(a)
        .element(b)
        .element(c)
        .state(Nyes::Embryonic)
        .is_tail_concatenation(true)
        .build();
    let rendered = format_fir_simple(&outer);
    eprintln!("\n=== FOOP-65 §6.1: ALL-EMBRYONIC TAIL CONCATENATION ===");
    eprintln!("Input: a`b`c`d e f");
    eprintln!("Storage order: [Concat(d,e,f), c, b, a]");
    eprintln!("Backtick rendering (reversed to source order):");
    eprintln!("  {}", rendered);
    eprintln!("=== END ===\n");
}
