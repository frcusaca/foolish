use crate::*;
use crate::fir::{
    ConstantIntFirBuilder, NkFirBuilder, OperatorFirBuilder, SearchFirBuilder,
    IndexFirBuilder, HeadTailFirBuilder, StayFoolishFirBuilder, StayFullyFoolishFirBuilder,
    ConcatenationFirBuilder, NormalBraneFirBuilder, Nyes, SearchDirection, Alarm, AlarmLevel, AlarmSource,
};
use crate::HumanizingSequencerRef;
use crate::sequencer::format_fir_simple;

fn format_fir_ref(fir: &Fir) -> String {
    HumanizingSequencerRef::new(fir).format_for_snap_test()
}

#[test]
fn test_hs_variant_all_types() {
    assert_eq!(ConstantIntFirBuilder::new(42).build().hs_variant(), "ConstantInt");
    assert_eq!(NkFirBuilder::new("unknown").build().hs_variant(), "Nk");
    assert_eq!(OperatorFirBuilder::new("+").build().hs_variant(), "Operator");
    assert_eq!(SearchFirBuilder::new("x").build().hs_variant(), "Search");
    assert_eq!(IndexFirBuilder::new(1).build().hs_variant(), "Index");
    assert_eq!(HeadTailFirBuilder::new(true).build().hs_variant(), "HeadTail");
    assert_eq!(StayFoolishFirBuilder::new(ConstantIntFirBuilder::new(1).build()).build().hs_variant(), "StayFoolish");
    assert_eq!(StayFullyFoolishFirBuilder::new(ConstantIntFirBuilder::new(1).build()).build().hs_variant(), "StayFullyFoolish");
    assert_eq!(ConcatenationFirBuilder::new().build().hs_variant(), "Concatenation");
    assert_eq!(NormalBraneFirBuilder::new().build().hs_variant(), "NormalBrane");
}

#[test]
fn test_format_empty_brane() {
    let brane = NormalBraneFirBuilder::new().state(Nyes::Constant).build();
    assert_eq!(format_fir_simple(&brane), "Brane{}");
}

#[test]
fn test_format_constant_int() {
    let c = ConstantIntFirBuilder::new(42).build();
    assert_eq!(format_fir_simple(&c), "Int(42)");
}

#[test]
fn test_format_named_statement() {
    let body = ConstantIntFirBuilder::new(42).build();
    let brane = NormalBraneFirBuilder::new()
        .statement(Some("x".into()), body)
        .build();
    let s = format_fir_simple(&brane);
    assert!(s.contains("x = Int(42)"), "Expected 'x = Int(42)' in: {}", s);
}

#[test]
fn test_format_multi_statement() {
    let s = vec![
        (Some("a".into()), ConstantIntFirBuilder::new(1).build()),
        (Some("b".into()), ConstantIntFirBuilder::new(2).build()),
    ];
    let brane = NormalBraneFirBuilder::new().statements(s).build();
    let out = format_fir_simple(&brane);
    assert!(out.contains("a = Int(1)"), "Expected 'a = Int(1)' in: {}", out);
    assert!(out.contains("b = Int(2)"), "Expected 'b = Int(2)' in: {}", out);
    assert!(out.contains(";"), "Expected semicolon separator in: {}", out);
}

#[test]
fn test_format_search() {
    let search = SearchFirBuilder::new("x")
        .direction(SearchDirection::Backward)
        .state(Nyes::Econstanic)
        .build();
    let s = format_fir_simple(&search);
    assert!(s.starts_with("Search("), "Expected Search( in: {}", s);
    assert!(s.contains("pattern='x'"), "Expected pattern='x' in: {}", s);
}

#[test]
fn test_format_nk() {
    let nk = NkFirBuilder::new("unknown").build();
    let s = format_fir_simple(&nk);
    assert!(s.starts_with("NK("), "Expected NK( in: {}", s);
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
    assert!(s.starts_with("NK("), "Expected NK( in: {}", s);
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
    assert!(s.contains("Operator(op='+'"), "Expected Operator(op='+' in: {}", s);
    assert!(s.contains("Int(1)"), "Expected Int(1) in: {}", s);
    assert!(s.contains("Int(2)"), "Expected Int(2) in: {}", s);
}

#[test]
fn test_format_concatenation() {
    let conc = ConcatenationFirBuilder::new()
        .element(ConstantIntFirBuilder::new(1).build())
        .element(ConstantIntFirBuilder::new(2).build())
        .state(Nyes::Constant)
        .build();
    let s = format_fir_simple(&conc);
    assert!(s.starts_with("Concatenation("), "Expected Concatenation( in: {}", s);
    assert!(s.contains("elements=2"), "Expected elements=2 in: {}", s);
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
    assert!(s.contains("Index(offset=1, FREE)"), "Expected 'Index(offset=1, FREE)' in: {}", s);
}

#[test]
fn test_format_index_anchored() {
    let idx = IndexFirBuilder::new(0).anchored(true).build();
    let s = format_fir_simple(&idx);
    assert!(s.contains("Index(offset=0, ANCHORED)"), "Expected 'Index(offset=0, ANCHORED)' in: {}", s);
}

#[test]
fn test_format_headtail_head() {
    let ht = HeadTailFirBuilder::new(true).build();
    let s = format_fir_simple(&ht);
    assert!(s.contains("HeadTail(HEAD, FREE)"), "Expected 'HeadTail(HEAD, FREE)' in: {}", s);
}

#[test]
fn test_format_headtail_tail_anchored() {
    let ht = HeadTailFirBuilder::new(false).anchored(true).build();
    let s = format_fir_simple(&ht);
    assert!(s.contains("HeadTail(TAIL, ANCHORED)"), "Expected 'HeadTail(TAIL, ANCHORED)' in: {}", s);
}

#[test]
fn test_format_stay_foolish() {
    let sf = StayFoolishFirBuilder::new(ConstantIntFirBuilder::new(1).build()).build();
    let s = format_fir_simple(&sf);
    assert!(s.starts_with("StayFoolish("), "Expected StayFoolish( in: {}", s);
    assert!(s.contains("Int(1)"), "Expected Int(1) in: {}", s);
}

#[test]
fn test_format_stay_fully_foolish() {
    let sff = StayFullyFoolishFirBuilder::new(ConstantIntFirBuilder::new(2).build()).build();
    let s = format_fir_simple(&sff);
    assert!(s.starts_with("StayFullyFoolish("), "Expected StayFullyFoolish( in: {}", s);
    assert!(s.contains("Int(2)"), "Expected Int(2) in: {}", s);
}

#[test]
fn test_integration_compile_format() {
    let firs = Compiler::compile("{x = 1 + 2}").unwrap();
    let formatted = format_fir_simple(&firs[0]);

    assert!(formatted.contains("x ="), "Expected 'x =' in: {}", formatted);
    assert!(formatted.contains("Operator(op='+'"), "Expected operator in: {}", formatted);
    assert!(formatted.contains("Int(1)"), "Expected Int(1) in: {}", formatted);
    assert!(formatted.contains("Int(2)"), "Expected Int(2) in: {}", formatted);
}

#[test]
fn test_integration_multi_statement_roundtrip() {
    let firs = Compiler::compile("{a = 1; b = 2; c = 3}").unwrap();
    let formatted = format_fir_simple(&firs[0]);

    assert!(formatted.contains("a = Int(1)"), "Expected 'a = Int(1)' in: {}", formatted);
    assert!(formatted.contains("b = Int(2)"), "Expected 'b = Int(2)' in: {}", formatted);
    assert!(formatted.contains("c = Int(3)"), "Expected 'c = Int(3)' in: {}", formatted);
}

#[test]
fn test_sequencer_ref_format_constant() {
    let fir = ConstantIntFirBuilder::new(42).build();
    let ref_seq = HumanizingSequencerRef::new(&fir);
    let out = ref_seq.format_for_snap_test();
    assert!(out.contains("Int(42)"), "Expected Int(42) in: {}", out);
}

#[test]
fn test_sequencer_format_with_header() {
    let fir = ConstantIntFirBuilder::new(42).build();
    let out = Sequencer::format_with_header("{42}", &fir, 0);
    assert!(out.contains("INPUT:"));
    assert!(out.contains("STEPS:"));
}
