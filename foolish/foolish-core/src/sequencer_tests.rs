use crate::fir::{
    Alarm, AlarmLevel, AlarmSource, ConcatenationFirBuilder, ConstantIntFirBuilder,
    HeadTailFirBuilder, IndexFirBuilder, NkFirBuilder, NormalBraneFirBuilder, Nyes,
    OperatorFirBuilder, SearchDirection, SearchFirBuilder, StayFoolishFirBuilder,
    StayFullyFoolishFirBuilder,
};
use crate::sequencer::format_fir_simple;
use crate::HumanizingFirSequencerRef;
use crate::*;

fn format_fir_ref(fir: &Fir) -> String {
    HumanizingFirSequencerRef::new(fir).format_for_snap_test()
}

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
        HeadTailFirBuilder::new(true).build().hs_variant(),
        "HeadTail"
    );
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
    assert!(s.contains("merged="), "Expected merged= in: {}", s);
}

#[test]
fn test_format_index() {
    let idx = IndexFirBuilder::new(1).build();
    let s = format_fir_simple(&idx);
    assert!(
        s.contains("#(offset=1, UNANCHORED,"),
        "Expected '#(offset=1, UNANCHORED,' in: {}",
        s
    );
}

#[test]
fn test_format_index_anchored() {
    let idx = IndexFirBuilder::new(0).anchored(true).build();
    let s = format_fir_simple(&idx);
    assert!(
        s.contains("#(offset=0, ANCHORED,"),
        "Expected '#(offset=0, ANCHORED,' in: {}",
        s
    );
}

#[test]
fn test_format_headtail_head() {
    let ht = HeadTailFirBuilder::new(true).build();
    let s = format_fir_simple(&ht);
    assert!(s.starts_with("^("), "Expected '^(' in: {}", s);
}

#[test]
fn test_format_headtail_tail_anchored() {
    let ht = HeadTailFirBuilder::new(false).anchored(true).build();
    let s = format_fir_simple(&ht);
    assert!(s.starts_with("$("), "Expected '$(' in: {}", s);
}

#[test]
fn test_format_stay_foolish() {
    let sf = StayFoolishFirBuilder::new(ConstantIntFirBuilder::new(1).build()).build();
    let s = format_fir_simple(&sf);
    assert!(s.starts_with("<"), "Expected '<' in: {}", s);
    assert!(s.contains("1"), "Expected '1' in: {}", s);
}

#[test]
fn test_format_stay_fully_foolish() {
    let sff = StayFullyFoolishFirBuilder::new(ConstantIntFirBuilder::new(2).build()).build();
    let s = format_fir_simple(&sff);
    assert!(s.starts_with("<<"), "Expected '<<' in: {}", s);
    assert!(s.contains("2"), "Expected '2' in: {}", s);
}

#[test]
fn test_integration_compile_format() {
    let firs = Compiler::compile("{x = 1 + 2}").unwrap();
    let formatted = format_fir_simple(&firs[0]);

    assert!(formatted.contains("x="), "Expected 'x=' in: {}", formatted);
    assert!(
        formatted.contains("+(1, 2,"),
        "Expected operator in: {}",
        formatted
    );
}

#[test]
fn test_integration_multi_statement_roundtrip() {
    let firs = Compiler::compile("{a = 1; b = 2; c = 3}").unwrap();
    let formatted = format_fir_simple(&firs[0]);

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
