# FOOP-45.plan — Parenthetical search chaining and contexted search after parenthetical

**Track 2 (Search Family) sub-task**: This FOOP is part of Track 2 in the Implementation Roadmap,
which covers the search family of FOOPs. It fixes parser issues with parenthetical search chaining
and contexted search after parenthetical expressions.

- [ ] Begin work: commit FOOP-45.md and FOOP-45.plan.md to origin, check `begun: [x]` in frontmatter
      (2026-07-24 15:10)
- [ ] Create worktree at /home/hcbusy/tmp/foolish-worktrees/foop-45-parenthetical-search-chaining with branch `foop/foop-45-parenthetical-search-chaining`
- [ ] Read §Specification of FOOP-45.md to understand the three issues and proposed grammar changes
- [ ] Read §Test Plan of FOOP-45.md to understand the test requirements

## Phase 1: Parser Investigation

- [ ] Investigate current parser behavior for `(expr)~name1~name2`
  - [ ] Locate parser code in foolish-ubca/src/ (likely in parser.rs or similar)
  - [ ] Add debug logging to trace how `(expr)~name1~name2` is parsed
  - [ ] Identify where the parser treats `~name1~name2` as a single pattern
  - [ ] Document the current grammar rules for search expressions
- [ ] Investigate parser behavior for contexted search after parenthetical
  - [ ] Trace how `(expr)~name&#1` is parsed
  - [ ] Identify where `&#` is not recognized as a separate operator
  - [ ] Document the current grammar rules for contexted search

## Phase 2: Parser Fixes

- [ ] Fix Issue 1: Parenthetical search chaining
  - [ ] Modify parser to recognize `~` as search operator termination
  - [ ] Update grammar to support chained searches: `search_expression '~' name_pattern`
  - [ ] Add parser tests for `(expr)~name1~name2` syntax
  - [ ] Verify parser correctly parses `(expr)~name1~name2` as chained searches
- [ ] Fix Issue 2: Contexted search after parenthetical
  - [ ] Modify parser to recognize `&#` as contexted search operator
  - [ ] Update grammar to support contexted search after search results
  - [ ] Add parser tests for `(expr)~name&#1` syntax
  - [ ] Verify parser correctly parses `(expr)~name&#1` as search + contexted search
- [ ] Fix Issue 3: Value search after parenthetical
  - [ ] Modify parser to recognize `~=` as value search operator
  - [ ] Update grammar to support value search after search results
  - [ ] Add parser tests for `(expr)~name~=0` syntax
  - [ ] Verify parser correctly parses `(expr)~name~=0` as search + value search

## Phase 3: Evaluator Fixes

- [ ] Update evaluator step rules for chained searches
  - [ ] Locate evaluator code in foolish-ubca/src/ (likely in evaluator.rs)
  - [ ] Modify step rules to handle chained search evaluation
  - [ ] Ensure search results are properly anchored for subsequent searches
  - [ ] Add unit tests for chained search evaluation
- [ ] Update evaluator for contexted search after parenthetical
  - [ ] Modify step rules to handle contexted search after search results
  - [ ] Ensure FoolRefFir is properly carried through chained searches
  - [ ] Add unit tests for contexted search after parenthetical
- [ ] Update evaluator for value search after parenthetical
  - [ ] Modify step rules to handle value search after search results
  - [ ] Ensure value search works on brane results from previous searches
  - [ ] Add unit tests for value search after parenthetical

## Phase 4: Test Updates

- [ ] Update `search_based_navigation.foo` test file
  - [ ] Fix expected values for paths 20-28 (parenthetical + contexted search)
  - [ ] Fix expected values for paths 43-45 (parenthetical + value search)
  - [ ] Fix expected values for paths 48, 50-51 (complex parenthetical)
  - [ ] Run test and verify all paths pass
- [ ] Write `foop_45_comprehensive.foo` test file (search upgrade comprehensive test)
  - [ ] Test single parenthetical with chained searches
  - [ ] Test double parenthetical with chained searches
  - [ ] Test contexted search after parenthetical
  - [ ] Test value search after parenthetical
  - [ ] Test mixed dot access and parenthetical searches
  - [ ] Test edge cases: empty branes, NK results, nested parenthetical
  - [ ] Run test and verify all paths pass

## Phase 5: Verification and Cleanup

- [ ] Run full test suite to ensure no regressions
  - [ ] Run `cargo test -p foolish-ubca --lib` and verify all tests pass
  - [ ] Run `cargo test -p foolish-core --lib` and verify all tests pass
  - [ ] Check for any new warnings or errors
- [ ] Update documentation
  - [ ] Update AGENTS.md with new search chaining syntax
  - [ ] Update howto files with examples of parenthetical search chaining
  - [ ] Add comments to parser code explaining the new grammar rules
- [ ] Verify all work is complete in /home/hcbusy/tmp/foolish-worktrees/foop-45-parenthetical-search-chaining and committed to `foop/foop-45-parenthetical-search-chaining`
- [ ] Merge `foop/foop-45-parenthetical-search-chaining` to `alpha`
- [ ] Cleanup worktree at /home/hcbusy/tmp/foolish-worktrees/foop-45-parenthetical-search-chaining

## Success Criteria

1. `(expr)~name1~name2` correctly chains searches
2. `(expr)~name&#1` correctly applies contexted search after parenthetical
3. `(expr)~name~=0` correctly applies value search after parenthetical
4. All existing tests continue to pass
5. New comprehensive test covers all edge cases
6. Documentation updated with new syntax examples

## Risks and Mitigations

1. **Risk**: Parser changes may break existing behavior
   **Mitigation**: Run full test suite after each change, revert if regressions found

2. **Risk**: Evaluator changes may affect performance
   **Mitigation**: Profile search-heavy tests before and after changes

3. **Risk**: Grammar changes may be ambiguous
   **Mitigation**: Write comprehensive parser tests to disambiguate edge cases

4. **Risk**: Contexted search chaining may have complex state management
   **Mitigation**: Carefully track FoolRefFir through search chains, add unit tests for state transitions
