# FOOP-64.plan — einmo-suite (migrate UBCa snapshots to hierarchical einmo)

Read `docs/foop/FOOP-64.md` before executing. Execute top to bottom. Once work begins, all
updates — including to this plan — happen ONLY in the worktree until merge.

Worktree literals (origin branch is `jia`: this clone has no `alpha`; `jia` is the integration
branch used by the FOOP-62 and FOOP-92 merges):

- Origin branch: `jia`, origin path: `/home/hcbusy/foolish-rust`
- Branch: `foop-64-einmo-suite`
- Worktree: `/home/hcbusy/tmp/foolish-worktrees/foop-64-einmo-suite`

---

- [x] Begin work: verified `cargo test --workspace` on `jia` — all suites green except
      `approval_all`, red signature-only (161/161; the P1 disease this FOOP cures — documented
      exception, not a blocker). Committed FOOP-64.md + FOOP-64.plan.md (6764f6ed) and the
      `begun: [x]` marker (3414805a).
      (2026-07-14 22:52)
- [x] Create worktree at /home/hcbusy/tmp/foolish-worktrees/foop-64-einmo-suite with branch
      `foop-64-einmo-suite`, from `jia` at /home/hcbusy/foolish-rust
      (2026-07-14 22:53)
- [ ] (read §Suite layout and §Harness of FOOP-64.md)
- [ ] Scaffold `foolish-ubca/einmo_suite/` — `input/` hierarchy dirs, `einmo.toml`
      (`[signing.output]` / `[signing.checked]` passphrase `""`; `verified` unset)
- [ ] Harness: add `einmo` dev-dependency to `foolish-ubca/Cargo.toml`; copy
      `UbcaEvaluatorAdapter` from `zweimomo/src/evaluators.rs` into
      `foolish-ubca/src/ubca_snapshot_tester.rs`; add `einmo_approval_all` test
      (`TestConfig::new(...).foolish_separator().require_correspondence(Stage::Output,
      Stage::Checked)`) — insta `approval_all` stays untouched
- [ ] (read §Placement rules and §Dual-home rule of FOOP-64.md)
- [ ] Attribution pass: `git log --follow` each of the 162 inputs; apply rules R1–R12; write
      `foolish-ubca/einmo_suite/MAPPING.md` (old path → new path, prefix strips, dual-home pairs)
- [ ] Copy all inputs from `foolish-ubca/snapshot_tests/input/` into
      `foolish-ubca/einmo_suite/input/` per MAPPING.md (copy, never move; `snapshot_tests/`
      remains fully intact and green)
- [ ] (read §Proposed new combination tests of FOOP-64.md)

## HUMAN INSPECTION — near-identical test clusters (FOOP-64, flagged 2026-07-15)

@human: these are the near-identical groups found by a similarity survey of the 161 migrated
inputs (transitive closure of >=0.75 character-similarity on whitespace-normalized source).
**Nothing here has been merged or deleted** — integration ("keep only the differing elements of
two nearly identical tests") rewrites Foolish source and can silently delete coverage, so it is
your call, per cluster.

**Agent's read, for what it's worth:**
- The **large loose groups** (the 12-member alarm/concat cluster, the 4-member seek cluster) are
  mostly **false positives**: Foolish programs are terse, so `{a = 1; b = 2; c = 3;}` scores
  ~0.8 against `{a = 10 / 2; b = 10 / 0;}` on characters while testing something entirely
  different. Recommend: **keep all**, no action.
- The **tight same-family groups** are the real candidates, but each looks like a deliberate
  **axis-variation** that earns its keep by localizing failures:
  - head_tail: single-value / two-element / nested / chained / on-search-result — five distinct
    anchors for `^`/`$`.
  - sff: basic / nested / in-assignment-chain / resolves-on-each-use / sf-of-sff — the SFF
    timing axis.
  - concat: inline / references / three-way — arity and reference-vs-literal axes.
  Recommend: **keep**, unless you see redundancy the agent does not.
- Already actioned: exactly one pair was byte-identical in source AND output
  (`foop9_operator_search_transparency{,_regression}`) — deduplicated, base kept.

Check a box to mean "inspected, no change needed"; annotate with `@agent integrate X into Y` to
order an integration.

  - [ ] **alarm_division_by_zero_in_brane** (group of 12, 0.75–0.94 similar)
        - `alarm_division_by_zero_in_brane.foo` → `misc/alarm_division_by_zero_in_brane.foo`
          `{a = 10 / 2; b = 10 / 0; c = 20 / 4;}`
        - `alarm_multiple_divisions_by_zero.foo` → `misc/alarm_multiple_divisions_by_zero.foo`
          `{a = 1 / 0; b = 2 / 0; c = 3 / 0;}`
        - `complex_concat_with_operations.foo` → `misc/complex_concat_with_operations.foo`
          `{a = {x=1+2}; b = {y=3*4}; c = a b;}`
        - `complex_negative_results.foo` → `misc/complex_negative_results.foo`
          `{a = 5 - 10; b = 3 * 2; c = 15 + 7;}`
        - `concatenation_basic.foo` → `misc/concatenation_basic.foo`
          `{ a={p=1;q=2}; b={r=3}; c = a b; }`
        - `concatenation_of_empty_branes.foo` → `misc/concatenation_of_empty_branes.foo`
          `{a = {}; b = {}; c = a b;}`
        - `concatenation_repeated_reference.foo` → `misc/concatenation_repeated_reference.foo`
          `{a = {x=1}; c = a a;}`
        - `concatenation_with_single_element.foo` → `misc/concatenation_with_single_element.foo`
          `{a = {x=1}; b = {y=2}; c = a b;}`
        - `concatenation_with_unresolved_search.foo` → `misc/concatenation_with_unresolved_search.foo`
          `{a = {x=ref}; b = {y=2}; c = a b;}`
        - `regression_regression_disappearing_brane.foo` → `regression/regression_disappearing_brane.foo`
          `{a = 1; b = 2; c = 3;}`
        - `search_pattern_matching_nested_brane.foo` → `misc/search_pattern_matching_nested_brane.foo`
          `{b = {a = {x = 1}; b = 2}; r = b?a;}`
        - `unanchored_seek_basic.foo` → `misc/unanchored_seek_basic.foo`
          `{a = 1; b = 2; c = #-1 + #-2;}`
  - [ ] **brane_with_single_value_head_tail** (group of 5, 0.75–0.90 similar)
        - `brane_with_single_value_head_tail.foo` → `misc/brane_with_single_value_head_tail.foo`
          `{b = {42}; h = b^; t = b$;}`
        - `head_tail_chained_on_nested.foo` → `misc/head_tail_chained_on_nested.foo`
          `{b = {{{1; 2}; 3}; 4}; h = b^; hh = b^^;}`
        - `head_tail_nested_brane.foo` → `misc/head_tail_nested_brane.foo`
          `{b = {{1; 2}; {3; 4}}$; c = b^; d = b$;}`
        - `head_tail_on_search_result.foo` → `misc/head_tail_on_search_result.foo`
          `{b = {x = {1; 2; 3}}; h = b.x^; t = b.x$;}`
        - `head_tail_on_two_element_brane.foo` → `misc/head_tail_on_two_element_brane.foo`
          `{b = {10; 20}; h = b^; t = b$;}`
  - [ ] **sf_of_sff** (group of 5, 0.76–0.93 similar)
        - `sf_of_sff.foo` → `misc/sf_of_sff.foo`
          `{a = 1; b = 2; sff = <<a + b>>; sf = <sff>; a = 10; sf; sff;}`
        - `sff_basic.foo` → `misc/sff_basic.foo`
          `{a=1,b=2; c=<<a+b>>; c; c;}`
        - `sff_in_assignment_chain.foo` → `misc/sff_in_assignment_chain.foo`
          `{a = 1; b = 2; c = <<a + b>>; a = 100; c;}`
        - `sff_nested.foo` → `misc/sff_nested.foo`
          `{a=1,b=2; c=<<a+<<b>>>>; c; c;}`
        - `sff_resolves_on_each_use.foo` → `misc/sff_resolves_on_each_use.foo`
          `{a=1; b=2; s=<<a+b>>; a=10; s;}`
  - [ ] **complex_multiple_seeks_in_brane** (group of 4, 0.75–0.86 similar)
        - `complex_multiple_seeks_in_brane.foo` → `misc/complex_multiple_seeks_in_brane.foo`
          `{a=1, b=2, c=3, d=4, e=5; s1 = #-1; s2 = #-2; s3 = #-3;}`
        - `operator_transparency_deep_chain.foo` → `misc/operator_transparency_deep_chain.foo`
          `{a=1, b=2, c=3, d=4, e=5; result = #-1 + #-2 + #-3 + #-4 + #-5;}`
        - `operator_transparency_mixed_ops.foo` → `misc/operator_transparency_mixed_ops.foo`
          `{a=1, b=2, c=3; result = #-1 * #-2 + #-3;}`
        - `regression_operator_does_not_block_search.foo` → `regression/operator_does_not_block_search.foo`
          `{a=1, b=2, c=3, d=4, e=5, f=6; expr = #-1 + #-2 + #-3 + #-4 + #-5 + #-6;}`
  - [ ] **concatenation_inline_branes** (triple, 0.76–0.77 similar)
        - `concatenation_inline_branes.foo` → `misc/concatenation_inline_branes.foo`
          `{c = {a=1, b=2, c=3}{e=4, f=5, g=6};}`
        - `concatenation_references.foo` → `misc/concatenation_references.foo`
          `{b1={a=1, b=2, c=3}; b2={e=4, f=5, g=6}; c = b1 b2;}`
        - `concatenation_three_way.foo` → `misc/concatenation_three_way.foo`
          `{b1={a=1, b=2}; b2={c=3, d=4}; b3={e=5, f=6}; c = b1 b2 b3;}`
  - [ ] **constant_int_literal** (triple, 0.80–0.86 similar)
        - `constant_int_literal.foo` → `misc/constant_int_literal.foo`
          `{42}`
        - `foop9_unary_operator.foo` → `foop/9/unary_operator.foo`
          `{a=-42;}`
        - `simple_unary_minus.foo` → `misc/simple_unary_minus.foo`
          `{-42;}`
  - [ ] **nested_search_in_brane** (triple, 0.75–0.82 similar)
        - `nested_search_in_brane.foo` → `misc/nested_search_in_brane.foo`
          `{x = 42; b = {y = x + 1};}`
        - `operator_in_nested_brane_with_scope.foo` → `misc/operator_in_nested_brane_with_scope.foo`
          `{x = 10; b = {y = x * 2 + 3};}`
        - `scope_resolution.foo` → `misc/scope_resolution.foo`
          `{ x = 42; y = x + 8; }`
  - [ ] **offset_access_backward** (triple, 0.84–0.89 similar)
        - `offset_access_backward.foo` → `misc/offset_access_backward.foo`
          `{data = {a=10; b=20; c=30; d=40; e=50}; last = data#-1; second_last = data#-2;}`
        - `offset_access_forward.foo` → `foop/41/offset_access_forward.foo`
          `{data = {a=10; b=20; c=30; d=40; e=50}; first = data#0; second = data#1;}`
        - `offset_access_out_of_bounds.foo` → `misc/offset_access_out_of_bounds.foo`
          `{data = {a=10; b=20; c=30; d=40; e=50}; oob = data#5; oob_neg = data#-6;}`
  - [ ] **regex_search_anchor_end** (triple, 0.79–0.86 similar)
        - `regex_search_anchor_end.foo` → `misc/regex_search_anchor_end.foo`
          `{result = {alice = 1; charlie = 2; bob = 3;}?e$;}`
        - `regex_search_anchor_start.foo` → `misc/regex_search_anchor_start.foo`
          `{result = {alice = 1; adam = 2; bob = 3;}?^a;}`
        - `regex_search_pattern.foo` → `misc/regex_search_pattern.foo`
          `{result = {alice = 1; bob = 2; charlie = 3;}?(^a.*);}`
  - [ ] **seek_beyond_start** (triple, 0.82–0.88 similar)
        - `seek_beyond_start.foo` → `misc/seek_beyond_start.foo`
          `{a=1; c=#-99;}`
        - `seek_negative_clamping.foo` → `misc/seek_negative_clamping.foo`
          `{a=1,b=2; c=#-99;}`
        - `unanchored_seek.foo` → `misc/unanchored_seek.foo`
          `{a=1,b=2,c=#-1;}`
  - [ ] **simple_division** (triple, 0.78 similar)
        - `simple_division.foo` → `misc/simple_division.foo`
          `{15 / 3;}`
        - `simple_subtraction.foo` → `misc/simple_subtraction.foo`
          `{10 - 3;}`
        - `zero_division.foo` → `misc/zero_division.foo`
          `{10 / 0;}`
  - [ ] **alarm_unknown_literal_no_alarm** (pair, 0.76 similar)
        - `alarm_unknown_literal_no_alarm.foo` → `misc/alarm_unknown_literal_no_alarm.foo`
          `{x = ???; y = 42;}`
        - `forward_reference_basic.foo` → `misc/forward_reference_basic.foo`
          `{y = x; x = 42;}`
  - [ ] **anchored_search_fails_on_constant** (pair, 0.77 similar)
        - `anchored_search_fails_on_constant.foo` → `misc/anchored_search_fails_on_constant.foo`
          `{b = {x = 100; y = 200}; notFound = b?γ;}`
        - `regex_search_not_found.foo` → `misc/regex_search_not_found.foo`
          `{result = {x = 100; y = 200;}?notfound;}`
  - [ ] **anchored_seek_negative_boundary** (pair, 0.86 similar)
        - `anchored_seek_negative_boundary.foo` → `misc/anchored_seek_negative_boundary.foo`
          `{b = {10; 20; 30}; last = b#-1; second = b#1; first = b#-3; oob = b#-4;}`
        - `anchored_seek_positive_boundary.foo` → `misc/anchored_seek_positive_boundary.foo`
          `{b = {10; 20; 30}; first = b#0; second = b#1; third = b#2; oob = b#3;}`
  - [ ] **deeply_nested_branes** (pair, 0.80 similar)
        - `deeply_nested_branes.foo` → `misc/deeply_nested_branes.foo`
          `{{{1;}; 2}; 3;}`
        - `multiple_expressions.foo` → `misc/multiple_expressions.foo`
          `{1; 2; 3;}`
  - [ ] **foop9_operator_search_transparency** (pair, 1.00 similar)
        - `foop9_operator_search_transparency.foo` → `foop/9/operator_search_transparency.foo`
          `{x=5, y=7, z=#-2 + #-1;}`
        - `foop9_operator_search_transparency_regression.foo` → `—`
          `{x=5, y=7, z=#-2 + #-1;}`
  - [ ] **identifier_in_expression** (pair, 0.81 similar)
        - `identifier_in_expression.foo` → `misc/identifier_in_expression.foo`
          `{x = 10; y = 20; x + y;}`
        - `identifier_shadowing.foo` → `misc/identifier_shadowing.foo`
          `{x = 10; x; x = 20; x;}`
  - [ ] **nested_arithmetic** (pair, 0.76 similar)
        - `nested_arithmetic.foo` → `misc/nested_arithmetic.foo`
          `{((2 + 3) * (4 - 1)) / 5;}`
        - `operator_precedence.foo` → `misc/operator_precedence.foo`
          `{2 + 3 * 4 - 5;}`
  - [ ] **regression_deep_nesting_does_not_lose_values** (pair, 0.75 similar)
        - `regression_deep_nesting_does_not_lose_values.foo` → `regression/deep_nesting_does_not_lose_values.foo`
          `{{{{x = 42; x};};};}`
        - `simple_identifier.foo` → `misc/simple_identifier.foo`
          `{x = 42; x;}`

  - [ ] **Small arithmetic / literal / identifier tests** (25 one-liners, all in `misc/`)
        @human: flagged as a group per your request — each is a one-line probe of one
        primitive. Agent's read: they are cheap, fast, and each names exactly what it
        pins, so a failure localizes instantly; consolidating them into one `basics.foo`
        would trade that for a single opaque diff. Recommend keep — your determination.
        Candidate sub-groupings if you do want consolidation: (a) pure literals,
        (b) binary ops, (c) unary/precedence/parens, (d) identifiers/undeclared.
        - `simple_integer.foo` → `misc/simple_integer.foo`  ·  `{5;}`
        - `constant_int_literal.foo` → `misc/constant_int_literal.foo`  ·  `{42}`
        - `simple_addition.foo` → `misc/simple_addition.foo`  ·  `{3 + 4;}`
        - `simple_subtraction.foo` → `misc/simple_subtraction.foo`  ·  `{10 - 3;}`
        - `simple_multiplication.foo` → `misc/simple_multiplication.foo`  ·  `{6 * 7;}`
        - `simple_division.foo` → `misc/simple_division.foo`  ·  `{15 / 3;}`
        - `zero_division.foo` → `misc/zero_division.foo`  ·  `{10 / 0;}`
        - `division_exact_and_remainder.foo` → `misc/division_exact_and_remainder.foo`  ·  `{a = 10 / 3; b = 9 / 3;}`
        - `simple_unary_minus.foo` → `misc/simple_unary_minus.foo`  ·  `{-42;}`
        - `negative_result.foo` → `misc/negative_result.foo`  ·  `{5 - 10;}`
        - `chained_unary_operators.foo` → `misc/chained_unary_operators.foo`  ·  `{a = -(-5); b = -(-(-3));}`
        - `operator_with_unary_and_binary.foo` → `misc/operator_with_unary_and_binary.foo`  ·  `{a = -3 * 4; b = -3 + 4;}`
        - `operator_precedence.foo` → `misc/operator_precedence.foo`  ·  `{2 + 3 * 4 - 5;}`
        - `mixed_operators.foo` → `misc/mixed_operators.foo`  ·  `{10 + 5 - 3 * 2;}`
        - `nested_arithmetic.foo` → `misc/nested_arithmetic.foo`  ·  `{((2 + 3) * (4 - 1)) / 5;}`
        - `single_parenthesized_expression.foo` → `misc/single_parenthesized_expression.foo`  ·  `{((((5))));}`
        - `large_numbers.foo` → `misc/large_numbers.foo`  ·  `{a = 1000000; b = 999999; c = a - b;}`
        - `multiple_expressions.foo` → `misc/multiple_expressions.foo`  ·  `{1; 2; 3;}`
        - `multiple_arithmetic_expressions.foo` → `misc/multiple_arithmetic_expressions.foo`  ·  `{5 + 3; 10 - 4; 2 * 6;}`
        - `mixed_expressions.foo` → `misc/mixed_expressions.foo`  ·  `{42; (3 + 4) * 2; -15; 100 / 5;}`
        - `simple_identifier.foo` → `misc/simple_identifier.foo`  ·  `{x = 42; x;}`
        - `identifier_in_expression.foo` → `misc/identifier_in_expression.foo`  ·  `{x = 10; y = 20; x + y;}`
        - `multiple_identifiers.foo` → `misc/multiple_identifiers.foo`  ·  `{x = 5; y = 3; z = 2; x * y + z;}`
        - `undeclared_identifier.foo` → `misc/undeclared_identifier.foo`  ·  `{x = non_existent;}`
        - `chained_undeclared.foo` → `misc/chained_undeclared.foo`  ·  `{bad = undeclared; y = bad; z = y;}`

- [ ] Author the eight new combination `.foo` inputs under `input/foop/64/` (+ dual-home
      `lang/usecases/` copies where they read as demonstrations)
- [ ] Write and verify `foolish-ubca/einmo_suite/input/foop/64/comprehensive.foo` (first
      comprehensive at the new reserved path)
- [ ] Generate + self-review: run `einmo_approval_all` to fill `output/`; inspect every new
      `.einmo`; `cargo build -p einmo` then
      `./target/debug/einmo promote "output->checked" foolish-ubca/einmo_suite/` (computer key);
      commit `checked/`
- [ ] Cross-validate: add `#[ignore]`d `cross_validate_einmo_vs_insta` (checked OUTPUT sections
      byte-match approved `.snap` RESULT sections, read-only on `.snap`); run it; fix any
      transport drift until clean
- [ ] foolish-core migration (same treatment, per FOOP-64.md §foolish-core migration):
      scaffold `foolish-core/einmo_suite/` (same hierarchy + einmo.toml); inventory
      `foolish-core/snapshot_tests/` — determine the post-FOOP-62 evaluator for its inputs and
      `einmo flag` stale inputs with reasons for the human; attribution pass → its own
      MAPPING.md; copy inputs per rules R1–R12 (dual-home rule applies); add its
      `einmo_approval_all`; generate, self-review, promote `output->checked`; cross-validate
      against its approved `.snap` corpus
- [x] SANITY CHECK (done up-front 2026-07-14, before implementation): does einmo actually fail
      when a demanded checked/verified file is missing? **Yes** — `compare --require-match`
      exits 1 with `only-in-output` + burden message for both stages, and the library's
      `require_correspondence` reports `correspondence_failures` (einmo's own
      `correspondence_failure_reported_until_promoted` test pins it). **But two vacuous passes
      found**: an empty suite passes `compare --require-match` (exit 0), and
      `confirm-signatures --require-all` over an empty `verified/` passes (exit 0). Both gate
      tests must assert non-emptiness — recorded in FOOP-64.md §Two-tier signing gate.
      (2026-07-14 22:41)
- [ ] Two-tier gate, development tier ("feature-complete test suite"): the `einmo_approval_all`
      tests (both suites) are the checked-stage gate — confirm each fails on any
      output↔checked divergence; **assert `!results.files.is_empty()`** (anti-vacuity, per the
      sanity check above)
- [ ] Two-tier gate, merge tier ("merge-ready test suite"): add `einmo_verified_gate` test
      (`#[ignore]` locally), covering BOTH suites (`foolish-ubca/einmo_suite/`,
      `foolish-core/einmo_suite/`): output ↔ `verified/` correspondence (`--require-match`
      semantics) + `confirm_signatures(verified, <human-key-prefix>) --require-all` +
      zero-computer-key scan on `stage:verified` stamps; **assert `verified/` is non-empty AND
      its file count == `output/` count BEFORE the key assertions** (anti-vacuity, per the
      sanity check — `--require-all` passes trivially on an empty directory)
  - [ ] Obtain the human reviewer's public-key hex prefix (human derives it once; passphrase
        never recorded); embed as the gate constant
- [ ] Add `.github/workflows/einmo-gates.yml`: on PR — run `einmo_verified_gate`; on push —
      `einmo verify --all` over the suite (absorbed FOOP-92 Phase 11)
- [ ] STOP: ASK HUMAN to (a) run the initial verified signing sessions —
      `einmo promote "checked->verified" foolish-ubca/einmo_suite/ --interactive` and
      `einmo promote "checked->verified" foolish-core/einmo_suite/ --interactive` over the
      reviewed corpora (the merge-ready tier cannot pass without them) — and (b) enable the
      GitHub branch-protection setting making the einmo-gates workflow a required status check
      (repository settings are human-only)
- [ ] Docs & skills (in-worktree): update `AGENTS.md` — Development Rules restated as "codebase
      must pass the einmo checked stage (output matches signed checked einmos)"; Build Commands
      → einmo flow; ⚠️ CRITICAL section rewritten in einmo terms (AI may promote
      output->checked after review; AI must NEVER produce a verified stamp; insta-specific
      prohibitions retired with insta). Update `foop.md` — comprehensive path re-homing + FOOP
      merge criteria include the checked-stage gate. Update the three skills
      (`.opencode/skills/{foop-write-plan,foop-use-maintain,foolish-debugging}/SKILL.md`) —
      insta/.snap references → einmo flow. Stamp all Last Updated sections
- [ ] Full gates in worktree: `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`,
      `cargo test --workspace` (insta suite AND einmo suite both green)
- [ ] Verify all work is complete in /home/hcbusy/tmp/foolish-worktrees/foop-64-einmo-suite and
      committed to `foop-64-einmo-suite`
- [ ] Merge `foop-64-einmo-suite` to `jia`
  - [ ] Confirm `foop/64/comprehensive.foo` exists, is reviewed, and passes in the einmo suite
  - [ ] Merge breaking changes from `jia` into the worktree first; resolve; re-run full gates
  - [ ] Repair ALL tests on `jia` in /home/hcbusy/foolish-rust after merge
  - [ ] STOP! STOP!! STOP!!! ASK HUMAN to check this box before continuing. UNDER NO
        CIRCUMSTANCES will Agent continue past this point automatically!!
    - [ ] Present human with `cd /home/hcbusy/tmp/foolish-worktrees/foop-64-einmo-suite` and ask
          them to review `einmo_suite/checked/` (diff vs old snaps via
          `cross_validate_einmo_vs_insta`, and `einmo show` on new foop/64 tests) BEFORE checking
          the parent checkbox. (The verified signing session already happened at the two-tier
          gate STOP above — confirm `einmo_verified_gate` passes before merging.)
- [ ] Retirement — **completion-blocking**: this FOOP is not complete until the repository has
      securely migrated off insta snapshots (insta removed from all dependencies)
  - [ ] Inventory remaining insta usage in `foolish-parser` (inline snapshot assertions);
        migrate to whatever einmo shape fits (small suite or unit-test rewrites) —
        `foolish-core` is already migrated by its dedicated task above
  - [ ] Cross-validate the parser migration the same way (content byte-match where applicable)
  - [ ] STOP: human deletes the `.snap` corpora (`foolish-ubca/snapshot_tests/`, and the
        parser/core equivalents) — agents may not move or alter `.snap` files; agent suggests
        the exact commands with case-inverted first words at that time
  - [ ] Agent: delete the insta test code (`approval_all` and parser/core insta tests) and
        `cross_validate_einmo_vs_insta`; remove `insta` from `foolish-ubca/Cargo.toml`,
        `foolish-parser/Cargo.toml`, `foolish-core/Cargo.toml`, and `[workspace.dependencies]`
  - [ ] Acceptance: `cargo tree -i insta` reports "package not found"; `cargo test --workspace`
        green; `foolish_review.sh` / `accept_approved.sh` disposition per the Open Question
- [ ] Cleanup /home/hcbusy/tmp/foolish-worktrees/foop-64-einmo-suite
  - [ ] Check that FOOP-64.plan.md has all but Cleanup checkboxes completed
  - [ ] Remove /home/hcbusy/tmp/foolish-worktrees/foop-64-einmo-suite
        (`git worktree remove ...`)
  - [ ] This is the last sub-task checkbox to be checked in this block
