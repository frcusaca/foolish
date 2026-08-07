# FOOP-75 — Tests written during design

These tests are written **as part of the specification**, per AGENTS.md
§"Development process" ("AI should always write the tests first"). They are
the executable form of FOOP-75's §2, §3, §4, §5, and §6, and they are
copied into their target files by the plan's implementation phases.

Every test below is written against the **specified** behavior, so all of
them **fail on `jia` today**. That is intended: each failure is the defect
the corresponding section repairs. Where a test pins *current* (defective
or limited) behavior deliberately, its name and comment say so explicitly.

---

## A. Parser — tree identity (§2)

Target: `foolish-parser/src/parser.rs`, tests module.

The core property. `Astn` derives `PartialEq`, so §2's claim — that the
attached and postfix spellings build the *same tree* — is assertable
directly rather than approximated by per-operator structural checks.

```rust
/// FOOP-75 §2: `LHS =SEARCH_SPEC RHS` builds the SAME tree as
/// `LHS = RHS SEARCH_SPEC`. This single property is the whole of the
/// parse-direction specification; if it holds for an operator, that
/// operator needs no further parse test.
#[test]
fn foop75_attached_search_builds_same_tree_as_postfix() {
    // (attached form, postfix form)
    let pairs = [
        ("{B={1,2,3}; A =$ B;}",      "{B={1,2,3}; A = B$;}"),
        ("{B={1,2,3}; A =^ B;}",      "{B={1,2,3}; A = B^;}"),
        ("{B={1,2,3}; A =#1 B;}",     "{B={1,2,3}; A = B#1;}"),
        ("{B={1,2,3}; A =#-2 B;}",    "{B={1,2,3}; A = B#-2;}"),
        ("{B={1,2,3}; A =?x B;}",     "{B={1,2,3}; A = B?x;}"),
        ("{B={1,2,3}; A =~x B;}",     "{B={1,2,3}; A = B~x;}"),
        ("{B={1,2,3}; A =.x B;}",     "{B={1,2,3}; A = B.x;}"),
        ("{B={1,2,3}; A =~=5 B;}",    "{B={1,2,3}; A = B~=5;}"),
        ("{B={1,2,3}; A =?=5 B;}",    "{B={1,2,3}; A = B?=5;}"),
    ];
    for (attached, postfix) in pairs {
        let a = parse_single(attached)
            .unwrap_or_else(|e| panic!("attached form failed to parse: {attached}: {e:?}"));
        let p = parse_single(postfix)
            .unwrap_or_else(|e| panic!("postfix form failed to parse: {postfix}: {e:?}"));
        assert_eq!(a, p, "trees differ:\n  attached: {attached}\n  postfix:  {postfix}");
    }
}

/// FOOP-75 §3: a chain of attached searches builds the same left-nested
/// spine as the equivalent postfix chain. The leftmost search in the
/// attached sequence is the INNERMOST node of the spine.
#[test]
fn foop75_attached_search_chains_match_postfix_chains() {
    let pairs = [
        ("{B={1,2,3}; A =#-2$ B;}",  "{B={1,2,3}; A = B#-2$;}"),
        ("{B={1,2,3}; A =$#1 B;}",   "{B={1,2,3}; A = B$#1;}"),
        ("{B={1,2,3}; A =^#1 B;}",   "{B={1,2,3}; A = B^#1;}"),
        // From the corpus: test-resources/.../test_syntax.foo:4 already
        // contains `c =$#-1;` — an attached chain written before this FOOP
        // existed. Found by the plan's Phase 1 survey.
        ("{a=1; b=2; c =$#-1;}",     "{a=1; b=2; c = #-1$;}"),
    ];
    for (attached, postfix) in pairs {
        let a = parse_single(attached).expect(attached);
        let p = parse_single(postfix).expect(postfix);
        assert_eq!(a, p, "chain trees differ: {attached} vs {postfix}");
    }
}
```

---

## B. Parser — the space rule (§5)

```rust
/// FOOP-75 §5.1(1): attachment requires ADJACENCY. `= $` (with a space
/// after the `=`) has no attached search.
///
/// NOTE: this test cannot pass until §5.3's lexer change lands — verified
/// on jia@dc6db093 that "{a =$ b}", "{a = $ b}" and "{a =   $ b}" lex to
/// BYTE-IDENTICAL token streams, because skip_whitespace advances `pos`
/// without incrementing `column`.
#[test]
fn foop75_space_after_equals_means_no_attached_search() {
    let attached = parse_single("{B={1,2,3}; A =$ B;}").expect("attached form parses");
    let spaced   = parse_single("{B={1,2,3}; A = $ B;}");
    match spaced {
        Ok(t) => assert_ne!(
            t, attached,
            "`= $` must NOT be treated as an attached search (§5.1(1))"
        ),
        Err(_) => { /* also acceptable: `= $` may simply be a parse error */ }
    }
}

/// FOOP-75 §5.1(3): an attached search MUST be terminated by a space.
/// Terminating with `;`, `}`, `,`, `)` or EOF is a PARSE ERROR — not a
/// statement with an empty RHS. An attached search is a promise that an
/// RHS follows.
#[test]
fn foop75_attached_search_must_be_space_terminated() {
    for src in [
        "{B={1,2,3}; A =$;}",
        "{B={1,2,3}; A =$}",
        "{B={1,2,3}; A =#-2;}",
        "{B={1,2,3}; A =^,}",
    ] {
        let err = parse_single(src)
            .expect_err(&format!("expected a parse error for unterminated attached search: {src}"));
        let msg = format!("{err}");
        assert!(
            msg.contains("space"),
            "the error must name the rule (\"attached search must be terminated \
             by a space\"); got: {msg}"
        );
    }
}

/// FOOP-75 §5.4: `&` is NOT in the trigger set. `=&?x` does not take the
/// attached-search path. (What it DOES do is not specified by this FOOP;
/// this test only pins that it is not silently treated as attached.)
#[test]
fn foop75_ampersand_is_not_an_attached_search_trigger() {
    let attached_like = parse_single("{B={1,2,3}; A =&?x B;}");
    let postfix       = parse_single("{B={1,2,3}; A = B&?x;}");
    if let (Ok(a), Ok(p)) = (attached_like, postfix) {
        assert_ne!(a, p, "`=&?x` must not be rewritten as an attached search (§5.4)");
    }
}
```

---

## C. Lexer — adjacency information (§5.3)

Target: `foolish-parser/src/lexer.rs`, tests module.

```rust
/// FOOP-75 §5.3: the lexer must record whether whitespace preceded each
/// token, because `column` cannot answer the adjacency question (it does
/// not count skipped whitespace).
///
/// Regression guard for the exact defect measured on jia@dc6db093:
/// "{a =$ b}" and "{a = $ b}" produced byte-identical token streams.
#[test]
fn foop75_lexer_records_preceding_space() {
    let toks = Lexer::new("{a =$ b}").tokenize();
    let dollar = toks.iter().find(|t| t.token == Token::Dollar).expect("has $");
    assert!(!dollar.preceded_by_space, "`=$`: the $ is adjacent to the =");

    let toks = Lexer::new("{a = $ b}").tokenize();
    let dollar = toks.iter().find(|t| t.token == Token::Dollar).expect("has $");
    assert!(dollar.preceded_by_space, "`= $`: the $ is NOT adjacent to the =");

    let toks = Lexer::new("{a =   $ b}").tokenize();
    let dollar = toks.iter().find(|t| t.token == Token::Dollar).expect("has $");
    assert!(dollar.preceded_by_space, "`=   $`: multiple spaces, still not adjacent");
}

/// FOOP-75 §5.3: newlines and tabs count as space for this purpose.
#[test]
fn foop75_lexer_counts_tab_and_newline_as_space() {
    for src in ["{a =\t$ b}", "{a =\n$ b}"] {
        let toks = Lexer::new(src).tokenize();
        let dollar = toks.iter().find(|t| t.token == Token::Dollar).unwrap();
        assert!(dollar.preceded_by_space, "tab/newline must count as space: {src:?}");
    }
}
```

---

## D. Evaluation — value correctness (§7, FOOP-54 §D.5)

Target: `foolish-ubca` unit tests.

These encode the three defects measured on `jia@dc6db093`. Each currently
produces the "actual" column of §Motivation's table.

```rust
/// FOOP-54 §D.5 (in-force): `a =$ b` ≡ `a = b$` — bind the value of the
/// LAST statement of `b` to `a`.
///
/// Measured on jia@dc6db093: yields the whole brane `{1;2;3}` (WOCONSTANIC),
/// NOT the tail. FOOP-75 §7 dissolves this by routing `=$` through IndexFir.
#[test]
fn foop75_attached_tail_binds_the_tail_value() {
    assert_settles("{b = {1,2,3}; y =$ b}", "y", "3");
}

/// FOOP-75 §7 / §Motivation defect (2): `=^` does not evaluate at all on
/// jia@dc6db093 — there is NO "^" arm in fir_kinds.rs to match "$" at :713,
/// so the OperatorFir never settles and leaks as `Op^(...)`.
#[test]
fn foop75_attached_head_binds_the_head_value() {
    assert_settles("{b = {1,2,3}; y =^ b}", "y", "1");
}

/// FOOP-75 §2 tree identity implies VALUE identity: the two spellings are
/// the same program, so they must settle to the same value.
#[test]
fn foop75_attached_and_postfix_settle_identically() {
    for (attached, postfix) in [
        ("{b = {1,2,3}; y =$ b}",   "{b = {1,2,3}; y = b$}"),
        ("{b = {1,2,3}; y =^ b}",   "{b = {1,2,3}; y = b^}"),
        ("{b = {1,2,3}; y =#1 b}",  "{b = {1,2,3}; y = b#1}"),
    ] {
        assert_eq!(settle(attached), settle(postfix), "{attached} vs {postfix}");
    }
}

/// FOOP-75 §8 / AGENTS.md §Searches: an ANCHORED miss settles NK. `4` is
/// not a brane, so its tail is provably unfindable.
#[test]
fn foop75_attached_tail_on_non_brane_settles_nk() {
    assert_nk("{d =$ 4}", "d");
}
```

---

## E. Sequencer — the reverse direction (§4)

Target: `foolish-core/src/sequencer_tests.rs`.

```rust
/// FOOP-75 §4: the sequencer walks the body's anchor spine, lifts the whole
/// run of searches out, and renders it immediately after the `=`.
#[test]
fn foop75_sequencer_lifts_search_spine_to_attached_position() {
    assert_sequences("{B={1,2,3}; A = B~=5#-2}", "A =~=5#-2 B");
}

/// FOOP-75 §4 NORMALIZATION: `A = B$` and `A =$ B` are the same tree (§2),
/// so they MUST render identically. The attached form is canonical.
///
/// Measured on jia@dc6db093: `z = b$` renders `z=3` — the `$` vanishes
/// entirely, because `b$` compiles to IndexFir while the sequencer's sugar
/// branch (sequencer.rs:650) is gated on hs_operator() == Some(("$", ..)).
#[test]
fn foop75_postfix_normalizes_to_attached_in_output() {
    assert_same_render("{b={1,2,3}; z = b$}", "{b={1,2,3}; z =$ b}");
}

/// FOOP-75 §4: the fallback is TOTAL — a statement whose body is not a
/// search renders exactly as it does today.
#[test]
fn foop75_non_search_statements_render_unchanged() {
    assert_sequences("{a = 1}", "a=1");
}

/// FOOP-75 §2+§4 round-trip: parse → evaluate → sequence → re-parse yields
/// the same tree. This is the property that makes the two directions
/// mutually consistent rather than independently plausible.
#[test]
fn foop75_round_trips() {
    for src in [
        "{b={1,2,3}; y =$ b}",
        "{b={1,2,3}; y =^ b}",
        "{b={1,2,3}; y =#-2 b}",
    ] {
        let once  = sequence(src);
        let twice = sequence(&once);
        assert_eq!(once, twice, "not idempotent under re-sequencing: {src}");
    }
}
```

---

## F. einmo — the documentary howto (§6)

Target:
`foolish-ubca/einmo_suite/input/foop/75/search_operator_inside_patterns_howto.foo`

Written as a **teaching document first, test second**. Its value is its
comments; a reader must be able to answer "can I put a `$` in a pattern?"
from this file alone, without reading FOOP-75.

```foolish
!!! FOOP-75 §6 — Search operators inside search patterns: a HOWTO
!!!
!!! Some characters are BOTH a Foolish search operator AND a regexp
!!! metacharacter. `$` is the clearest case: it means "tail search" in
!!! Foolish and "end of string" in a regexp. When a `$` appears after a
!!! `~` or `?`, which one is it?
!!!
!!! The rule: a pattern runs until a SPACE (FOOP-75 §5), so a bare `$` is
!!! part of the pattern. To chain a real tail search onto a name search,
!!! delimit the pattern with parentheses. !!!

{
	haystack = {alpha = 1; beta_asdf = 2; gamma = 3};

	!! (1) BARE — the trailing `$` is a regexp END-ANCHOR, part of the
	!!     pattern. This searches for names ENDING in "asdf". It is NOT a
	!!     tail search. One search, not a chain.
	ends_with_asdf = haystack~.*asdf$;

	!! (2) PARENTHESIZED — the parens DELIMIT the pattern. The pattern is
	!!     exactly `.*asdf$` (still end-anchored). The parens are not part
	!!     of the pattern text. Same meaning as (1), written unambiguously.
	same_as_1 = haystack~(.*asdf$);

	!! (3) PARENTHESIZED, THEN CHAINED — the pattern is `.*asdf`, and the
	!!     `$` AFTER the close paren is a real TAIL search, chained onto
	!!     the name search's result. This is the case that is impossible
	!!     to write without parens.
	tail_of_match = haystack~(.*asdf)$;

	!! (4) PATTERN THEN INDEX — same idea with `#`.
	two_before = haystack?(gamma)#-2;

	!! (5) `^` as a regexp START-anchor, not a head search.
	starts_with_a = haystack~(^a.*);

	!! (6) ATTACHED FORM — every case above may be written attached to the
	!!     `=` instead (FOOP-75 §2). The tree, and therefore the value, is
	!!     IDENTICAL to the postfix spelling. Compare with (3).
	attached_tail_of_match =~(.*asdf)$ haystack;

	!! (7) The space is REQUIRED to end an attached search (FOOP-75 §5.1).
	!!     `attached =$;` would be a PARSE ERROR, not an empty RHS.
	attached_plain_tail =$ haystack
}
```

**Note on §6.2 deferral**: if the §6.3 survey defers the parenthetical
terminator to its own FOOP, cases (2)–(6) change meaning and this file must
be rewritten to pin the §6.4 behavior instead, with comments stating that
the parenthetical form is not yet available and naming the FOOP that would
add it. The file is written either way — only its expected output differs.

---

## F2. Cross-FOOP — interaction with FOOP-65 (§9.3)

**Only applicable once FOOP-65 has landed.** If FOOP-75 lands first, this
section is carried forward into FOOP-65's test plan instead (§9.4).

```rust
/// FOOP-75 §9.3: an attached search applies to the WHOLE RHS, whatever its
/// internal structure. After FOOP-65, the RHS may be a backtick chain —
/// and since the backtick is the WEAKEST operator (FOOP-65 §2), the chain
/// IS the whole RHS.
///
/// This also answers FOOP-65's Open Question ("$-after-backtick
/// ergonomics"): `(fn`X)$` and `=$ fn`X` are the same tree.
#[test]
fn foop75_attached_search_applies_to_whole_backtick_chain() {
    let attached    = parse_single("{fn={r=1}; A =$ fn`{a,b};}").unwrap();
    let parenthesized = parse_single("{fn={r=1}; A = (fn`{a,b})$;}").unwrap();
    assert_eq!(
        attached, parenthesized,
        "§9.3 reading (i): the attached search takes the whole application, \
         NOT the chain's last operand"
    );
}

/// FOOP-75 §9.4: FOOP-65's new Backtick token must populate the
/// `preceded_by_space` field added by §5.3. Guards the one-line omission
/// whichever FOOP lands second could make.
#[test]
fn foop75_backtick_token_carries_adjacency_flag() {
    let toks = Lexer::new("{a = fn`{x}}").tokenize();
    let bt = toks.iter().find(|t| t.token == Token::Backtick).expect("has backtick");
    assert!(!bt.preceded_by_space, "`fn`{{x}}`: backtick is adjacent to fn");
}
```

---

## G. Tests that pin CURRENT behavior deliberately

These do **not** describe desired behavior. They exist so a future change
is deliberate rather than accidental. Each must carry a comment saying so.

```rust
/// PINS A KNOWN LIMITATION — not desired behavior. See FOOP-75 §6.3.
///
/// On jia@dc6db093, `~(x)` yields the pattern "(x)" (parens absorbed INTO
/// the pattern text), and `~(a|b)c` yields "(a|b)c" as ONE pattern. Under
/// §6.2 these become "x" and a CHAIN respectively — a meaning change.
///
/// If you are reading this because the test failed: you are changing
/// pattern-boundary semantics. That is FOOP-75 §6.2's job. Confirm the
/// §6.3 survey was done and the change is intended; do not "fix" this test.
#[test]
fn foop75_pins_current_paren_pattern_absorption() {
    // ... assert pattern == "(x)" and "(a|b)c" ...
}
```

---

## Last Updated

**Date**: 2026-08-07
**Updated By**: Claude Code / claude-opus-5
**Changes**: Initial set, written during design per AGENTS.md §"Development
process". Encodes §2 tree identity (the central property, assertable via
`Astn: PartialEq`), §3 chains, §5 space rule incl. the §5.3 lexer
prerequisite, §4 sequencer lifting and normalization, §7 value corrections
against FOOP-54 §D.5, and the §6 documentary howto. Section G isolates
tests that deliberately pin current defective behavior.
