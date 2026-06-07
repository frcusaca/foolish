# FOOP-52 Plan: FVM scope/search rework + repair WIP snapshot bugs

**Created**: 2026-06-06
**Rewritten**: 2026-06-07
**Status**: Draft (plan under review — no code or worktree created yet)
**Type**: Major (reworks scope/search across the FVM; the 15 WIP bugs are its acceptance test)
**Bugs**: 15 (6 groups, includes Bug 15 from FOOP-32)

> **For the implementing session:** read AGENTS.md and all three FOOP-52 docs
> (`.md`, `.plan.md`, `.bugs.md`) first. The per-checkbox notes below carry the
> reasoning, file:line anchors, and the gotchas worked out during plan review —
> they exist so you don't have to re-derive them. (@human all file:line anchors
> are as of 2026-06-07 on branch `alpha`; verify before editing — line numbers
> drift.)

---

## Framing (why this plan is shaped this way)

The FVM is a **parent-linked FIR graph**, kept on `Rc<RefCell<Fir>>`:
- **Children — `Rc<RefCell<Fir>>`.** FIRs are shared, not uniquely owned: a resolved
  search holds a read-only `Rc` reference to the immutable CONSTANT/INDEPENDENT node
  it found — shared, never deep-copied (`constanic_clone` returns `Rc::clone(source)`
  for CONSTANT/INDEPENDENT/NK, `ubc.rs:467`). FIRs mutate through `RefCell`.
- **Parents — `Weak<RefCell<Fir>>`.** Every FIR has a readable, non-owning back-pointer
  to its IMMEDIATE STRUCTURAL parent — usually NOT a brane (`x`,`y` → `+` → statement →
  brane → parent brane's statement → …). Finding the nearest enclosing *brane* is
  `get_brane()` (walk up to a brane), distinct from `get_parent()`. These fields exist
  today but are VESTIGIAL (set in a couple places, never read); this FOOP makes them
  real and used.
- **Access up and down, mutate only self.** Evaluation trickles DOWN from the root
  (`child.step()`); a FIR reads children and ancestors to compute but writes only
  itself; the outermost owning call commits.

The rework does NOT change the ownership model (it's already `Rc<RefCell>`). It:
1. makes parent pointers real `Weak` and uses them,
2. replaces the accumulating flat `Scope` with three recursive search methods on the
   brane (`iterate_ib_statements` / `search_immediate_brane` /
   `search_ancestral_branes`) — see FOOP-52.md "The three brane search methods",
3. kills the O(n²) per-statement clones in `braning_step`.

(`Box<Fir>` owned bodies were considered and REJECTED: shared read-only CONSTANT
nodes + readable parent pointers don't fit unique ownership without deep-copying
immutable nodes or `unsafe` self-referential raw pointers. `Rc`/`Weak` is idiomatic,
safe, and the smaller change. For UBCb — which shuffles/replaces FIRs — `Rc` is right
anyway, so one model serves both.)

The plan has one spine:

1. **Rework so the existing 64 `.snap` pass — byte-identical — then check the box.**
   Byte-identical output is the *proof* the rework was behavior-preserving. If a
   snapshot moves, the search/scope rework changed semantics — investigate before any
   bug work (`wo_short_circuit` is the likeliest culprit).
2. **Then repair the 15 WIP files, one bug group at a time.**

This separates "did I preserve what worked" (Phase 1, oracle = 64 snapshots) from
"did I fix what was broken" (Phase 2+, oracle = the 15 WIP files).

**Test baseline (verified 2026-06-07):** `cargo insta test -p foolish-core --lib`
is GREEN — insta defers snapshot mismatches to `.snap.new` rather than failing.
Plain `cargo test -p foolish-core --lib` FAILS on the 15 WIP inputs that lack a
`.snap` (insta stops on the first, alphabetically `anchored_seek_negative_boundary`).
Both are expected. Snapshot dirs: input = `foolish-core/snapshot_tests/input/`
(singular "input"), approved = `foolish-core/snapshot_tests/approved/`.

**EXCEPTION to AGENTS.md "no failing tests" rule:** AGENTS.md says never start Major
work with broken tests — FOOP-52 is explicitly excepted, because the 15 failing WIP
tests ARE the work (see FOOP-52.md §"Exception to the 'no failing tests' rule"). The
green oracle is `cargo insta test` (64 approved snapshots). Do NOT halt on the rule;
do NOT make the WIP files pass except by fixing the bugs. The exception is narrow —
NEW breakage (any of the 64, or any unit test) still halts work as usual.

---

## The Plan (top-level spine)

- [ ] **Task 0: Organize AGENTS.md** — consolidate the existing Rust best-practice
      guidance and add a terse encapsulation rule (behavior-on-data / self-mutate,
      owner swaps on type change / no leaked mutable aliases) + the Foolish-semantic-
      immutability-vs-FIR-state principle. Done before Phase 1 because the rework must
      follow the rules it documents. (DONE — both written into AGENTS.md 2026-06-07.)

- [ ] **Phase 1: Rework scope/search — `Weak` parents made real + used, three
      recursive brane search methods replace the flat `Scope`, kill the O(n²)
      `braning_step` clones. Keep `Rc<RefCell<Fir>>` children; mutate-self. GATE: all
      64 existing `.snap` pass BYTE-IDENTICAL.** (The 15 WIP files are still expected
      to FAIL here — they have no `.snap`; they are the Phase 2+ acceptance test.)

- [ ] **Phase 2+: Repair WIP — one bug group per phase, promote each WIP file
      to `.snap` as it is fixed.**
      - backward-search / source-order (Bugs 1.1–1.3, 2.3, 6.1–6.2) + Bug 15
      - boundary crossing (Bug 2.1)
      - search-tree resolution (Bugs 2.2, 3.2, 4.1)
      - concatenation precedence (Bug 3.1)
      - SF/SFF markers (Bugs 5.1–5.3) — `is_search()` FIRs → ECONSTANIC in SFF

- [ ] **Final: 64 + 15 = 79 snapshots pass; WIP markers removed; FOOP-52 status → Final.**

---

## Phase detail

### Task 0: Organize AGENTS.md

- [x] Add `### Encapsulation` subsection under "How To Write Rust Code" (near "Enum
      Dispatch" / "Traits and Generics"). DONE 2026-06-07 — the rule is written into
      AGENTS.md: struct owns its data + self-mutating methods + reasoning about itself
      + reporting methods; self-mutate via `&mut self`, return replacement only on type
      change; answer questions about self via methods not tag-matches; no leaked mutable
      aliases. This rule GOVERNS the rewrite: it is WHY the `ubc.rs` free functions
      (`re_step_brane_bodies`, `reset_searches`, `step_except_brane_one`,
      `strip_sf_wrapper`) move onto the types, and WHY string dispatch becomes
      enum/method dispatch (and `wo_short_circuit` becomes a query on the Search FIR).
      (2026-06-07)
- [ ] (Optional, low priority) Consolidate any remaining scattered Rust best-practice
      duplication under "How To Write Rust Code" — the section is already good
      (General Rust Style, Enum Dispatch, Traits and Generics, Error Handling); only
      tidy if duplication is found. Not blocking Phase 1.

### Phase 1: Parent-linked FIR graph + recursive search (GATE: 64 snapshots byte-identical)

**Goal:** behavior-preserving rework of scope/search. Keep `Rc<RefCell<Fir>>`
children; make parents real `Weak<RefCell<Fir>>` back-pointers and USE them; replace
the flat accumulating `Scope` with the three recursive brane search methods. Do NOT
fix any of the 15 bugs here — only establish the search machinery + predicates the
bug fixes need. Keep behavior identical so the 64-snapshot gate is meaningful.

OWNERSHIP MODEL (settled — see FOOP-52.md "Architecture"): children `Rc<RefCell<Fir>>`
(shared; resolved searches hold a read-only `Rc` ref to immutable CONSTANT nodes,
never copied); parents `Weak<RefCell<Fir>>` (readable non-owning up-edge, no cycle);
access up and down, mutate only self. NOT `Box<Fir>` (rejected — shared CONSTANT
nodes + readable parent pointers don't fit unique ownership).

- [ ] Change brane statement storage: `NormalBraneFir.statements` from
      `Vec<StatementFir>` (`fir.rs:309`) to a FIXED-SIZE `Vec<Rc<RefCell<StatementFir>>>`.
      Allocated once when the brane is built from the AST; length never changes during
      eval (statements stepped/replaced in place, never appended/removed). `StatementFir`
      already exists (`fir.rs:195`: `name: Option<String>` LHS, `body: FirRef` RHS,
      `state`) — no new type. Update the compiler (`compiler.rs` builds `StatementFir`)
      and the builder (`StatementFirBuilder`, `fir.rs:1921`) to wrap in `Rc<RefCell>`.
- [ ] Add TWO fields to `StatementFir`, both set at construction (and re-set on
      recoordination/clone): `parent: Weak<RefCell<Fir>>` (the OWNING brane) and
      `line_number: usize` (its own 0-based index into the parent's `statements` vec —
      so `parent.statements[line_number]` is itself). Construction signature becomes
      `StatementFir::new(name, body, parent, line_number)` (or builder equivalent).
      These make widening in `search_ancestral_branes` a field read, not a scan: a
      brane's enclosing statement knows its `line_number` and its brane directly.
      `line_of_child` collapses to `child.line_number()`; keep `Rc::ptr_eq` only as a
      debug-assert that `parent.statements[line_number]` is the child.
- [ ] Make parents real and used. `SearchFir.parent` (`fir.rs:267`) and
      `NormalBraneFir.parent` (`fir.rs:311`) become `Weak<RefCell<Fir>>`, SET during
      construction / brane recoordination, and READ by search (they are vestigial
      today — set in a couple places, never read). Every FIR has a parent: `x`,`y` →
      `+` → statement → brane → parent brane's statement → … (root brane's parent is
      an empty `Weak`). Statement → brane is the `StatementFir.parent` above.
      - NOTE: parent pointers must be on EVERY FIR, not just Search/Brane. Either add
        `parent: Weak<RefCell<Fir>>` to all FIR structs, or store it once in a shared
        FIR header. The two trait methods below depend on every node answering.
- [ ] Add `get_parent` / `get_brane` trait methods (every FIR answers — encapsulation
      rule). `get_parent(&self) -> Option<FirRef>` = `upgrade()` of the `Weak`.
      `get_brane(&self) -> Option<FirRef>` = recursive: "return `get_parent()` if it
      is a brane (`kind() == FirKind::NormalBrane`), else `get_parent().get_brane()`".
      For `{b = 1 + c}`, `c`'s `get_brane()` skips `+` and statement `b`, returns the
      containing brane — the brane whose earlier statements `c` searches. This is how a
      Search FIR STARTS resolution: `self.get_brane()` → take the enclosing statement's
      stored `line_number` → `home.search_ancestral_branes(pattern, line_number)`.
- [ ] Implement the three brane search methods on `NormalBraneFir` (normative code in
      FOOP-52.md "The three brane search methods"). Each touches only local members.
      Statements are `Rc<RefCell<StatementFir>>` handles — the iterator yields handles
      (can't return borrows out of `RefCell`); callers `borrow()` to read name/body:
      - `iterate_ib_statements(from_line, direction)` — walk own statements from
        `from_line`, inclusive, in `direction`, yield a handle to each NAMED statement. Backward
        = `(0..from_line).rev()`; forward = `(from_line)..len-1`. SHARED by anchored and
        unanchored search — this factoring is required, not optional.
      - `search_immediate_brane(pattern, from_line, direction)` — for each handle from
        the iterator, `borrow()` and regex-match `name()`; on hit return `Rc::clone` of
        the statement's `body()`. ANCHORED search (`a.foo`) calls this on the anchor's
        brane; also called at each level of ancestral search.
      - `search_ancestral_branes(pattern, from_line)` — own brane backward first, else
        widen: this brane's enclosing STATEMENT → read its stored `line_number` → that
        statement's brane → recurse `outer.search_ancestral_branes(pattern,
        line_number)`. Terminates at root (no enclosing statement). UNANCHORED search.
        No scan to find the line — it's the stored `line_number`.
      - `line_of_child` collapses to `child.line_number()` (stored field). Keep
        `Rc::ptr_eq(&parent.statements[line_number], &child)` only as a debug-assert.
      - Rust subtlety: borrow up the chain via `upgrade()` then `borrow()`; do NOT hold
        a `borrow_mut` on a node while recursing upward.
- [ ] Write UNIT TESTS for the three methods (in `unit_tests.rs`), flat AND nested
      branes — these are load-bearing primitives, test them directly not only via
      snapshots:
      - `iterate_ib_statements`: backward/forward yields handles, skips anonymous
        (unnamed) statements, empty brane, `from_line` at 0 and len.
      - `search_immediate_brane`: hit/miss, nearest-match-wins (shadowing), regex
        pattern, `from_line` excludes later statements (forward-ref not found).
      - `search_ancestral_branes`: resolves in immediate; resolves in parent; does NOT
        resolve a parent name defined AFTER the nested brane; terminates at root
        (None, no panic); two-level nesting (grandparent).
      - `StatementFir.parent` + `line_number`: every statement's parent is the owning
        brane and `parent.statements[line_number]` is the statement itself, after
        construction AND after a nested-brane clone/recoordination. (`line_of_child` is
        just `line_number`.)
      - `get_parent` / `get_brane`: for `{b = 1 + c}`, `c`'s get_parent = `+`/statement,
        get_brane = the containing brane (skips operator + statement); get_brane on the
        root brane = None; nested case returns the immediate enclosing brane.
      - Build test branes via parser + root `.search(...)` per AGENTS.md "Unit Test
        Readability".
- [ ] Retire the flat `Scope`: remove `entries: Vec<(String, FirRef)>` (`ubc.rs:38`)
      and its `search()` over `entries.iter().rev()` (`ubc.rs:95-103`); remove the
      pre-push of ALL names (`ubc.rs:230-234`, the ROOT CAUSE of forward refs
      resolving — Bugs 1.x/2.3); remove the `current_brane`/`current_stmt_idx` stale
      snapshot (`ubc.rs:39-40,239-244`). Keep the alarm sink surface — `unit_tests.rs:193`
      uses `Scope::new().with_alarms().emit()`; preserve an equivalent (alarms can ride
      on a slim eval context or move onto the brane-step entry point).
- [ ] Rewrite `braning_step` (currently `re_step_brane_bodies`, `ubc.rs:216`) to kill
      the O(n²): the per-statement loop (`ubc.rs:237`) clones `brane.statements` AND
      `local_scope` EVERY iteration. New version steps statement N reading its own
      earlier statements + ancestors via the search methods — no per-statement clones.
- [ ] Add `is_search()` predicate on the FIR trait. Default `false`; override `true`
      for `SearchFir`, `IndexFir`, `HeadTailFir`.
      - Atlas decision: FIRST-CLASS concept, on the trait so EVERY FIR answers for
        itself (not FirKind-only, not a string compare).
      - Doc comment = the invariant: "A search consults the surrounding brane to find a
        value: name Search, positional Index/seek (`#-1`), HeadTail (`{}^`). These are
        the ONLY FIRs whose meaning stays indeterminate once the expression text is
        composed — everything else has singular invariant meaning. SFF marks exactly
        these ECONSTANIC at the start."
      - NOTE: in Phase 1 `is_search()` exists and is ROUTED THROUGH (below); its SFF
        *effect* (mark ECONSTANIC) is Phase 6. Phase 1 use stays behavior-preserving.
- [ ] Route `has_unresolved_forward_refs` (`ubc.rs:160-195`) and the WOCONSTANIC-chain
      follow (`ubc.rs:1060`, `short_circuit_self` `fir.rs:1046`) through `is_search()`
      instead of `fir_variant() == "Search"`.
      - LATENT BUG fixed for free: `has_unresolved_forward_refs` has arms for
        Search/Operator/Concatenation/StayFoolish but NOT Index/HeadTail → they fall to
        `_ => false`, so an unresolved seek is invisible. (Verify it doesn't move any of
        the 64 — if it does, that's a real pre-existing bug surfacing; flag it.)
- [ ] Replace `fir_variant() -> &'static str` (`fir.rs:370` + all 10 overrides) with
      `kind() -> FirKind` enum for DISPATCH. A local `Variant` enum exists
      (`ubc.rs:359`) — promote it to a shared `FirKind`.
      - Kills stringly-typed `== "Search"` / `== "NormalBrane"` / `== "StayFullyFoolish"`.
      - Keep `kind()` (dispatch) SEPARATE from `is_search()` (predicate) — different homes.
- [ ] Remove `reset_searches` (`ubc.rs:260-323`) entirely — with positional `from_line`
      search, forward refs are out of range and never resolve, so nothing to reset.
      (`constanic_clone` still does per-reuse resets, `ubc.rs:466-477`.)
- [ ] Rename `short_circuit` (`ubc.rs:498`) / `short_circuit_self` (`fir.rs:1046`)
      → `wo_short_circuit`, reframe as a QUERY on the Search FIR (encapsulation rule,
      Task 0). Follows the **WOCONSTANIC** target chain (verified: loop walks
      `target→target` while each link is WOCONSTANIC, `ubc.rs:506-512`) — `wo_` names
      what it short-circuits.
      - Shape: `fn wo_short_circuit(&self) -> &Fir`. If `self` is a WOCONSTANIC search,
        follow the chain through successive WOCONSTANIC searches, return the first
        node that is **not** WOCONSTANIC (terminus — typically ECONSTANIC, may be
        CONSTANT/NK). Otherwise return `self`. Stopping rule = "first non-WOCONSTANIC",
        preserving current behavior (NOT "ECONSTANIC-only"). Byte-identical on the 64.
      - Call site: `self.target = search_result_target.wo_short_circuit();` — called on
        the TARGET (found result), asking for its passthrough terminus.
- [ ] Update sequencer (`sequencer.rs`) / serialization (`serialization.rs`) read
      paths only as needed by the parent-pointer/field changes (e.g. don't serialize
      the `Weak` parent — it would cycle; serialize children only). Lower-volume than
      the rejected Box plan since `Rc<RefCell>` stays.
- [ ] **GATE: `cargo test -p foolish-core --lib` — all 64 approved `.snap` pass
      byte-identical. The 15 WIP files still fail (expected, no `.snap`).** Plain
      `cargo test` (not `cargo insta test`) so WIP absence shows as failure honestly.
- [ ] Verify all unit tests pass (existing `fir::builder_tests`, `sequencer_tests`,
      `unit_tests`, `signature::tests`) PLUS the new search-method unit tests.

### Phase 2: Backward search / source-order (Bugs 1.1–1.3, 2.3, 6.1–6.2, 15)

**Root cause (shared):** the old flat scope pre-pushed all names. With Phase 1's
positional backward search, forward refs are simply out of range → stay ECONSTANIC
with no special-casing. Bugs 6.x were the SAME root cause via a different symptom:
the old `current_brane` snapshot (`ubc.rs:239-244`) pointed at RESET EMBRYONIC bodies,
so `#-1` retrieved a NYE body and `constanic_clone` (`ubc.rs:459`) hit the
INVARIANT-VIOLATED path (`ubc.rs:478-492`). Phase 1's scope references stepped bodies →
seek never sees NYE → violation is genuinely unreachable (no `permit_nye` hack needed).

- [ ] Confirm positional backward search prevents forward-reference resolution
- [ ] Test Bug 1.1: `{y = x; x = 42;}` — `y` is Search/ECONSTANIC, not `Int(42)`
- [ ] Test Bug 1.2: `{outer = {val = x}; x = 100;}` — `val` is Search/ECONSTANIC
- [ ] Test Bug 1.3: `{nested = {inner = {val = x}}; x = 42;}` — `val` is Search/ECONSTANIC
- [ ] Test Bug 2.3 (shadowing/SSA): `{x = 10; x; x = 20; x;}` — second `x`=10, fourth=20.
      This is exactly "nearest-earlier wins" — good direct test of the backward scan.
- [ ] Test Bug 6.1: `{a=10, b=20, c=30, result=#-1+#-2, result2=#-1*#-2, result3=#-1-#-2;}`
      — all CONSTANT (result=50, result2=600, result3=10), NO invariant violations
- [ ] Test Bug 6.2: `{a=1; b={c=#-1; d=2; e=#-1}; f=#-1;}` — no violations; `c` seeks to
      `a`, `e` seeks to `d`(=2), `f` seeks to `b`'s brane value
- [ ] Bug 15 (folded here — it's a seek/boundary bug): fix `index_in_brane`
      (`search.rs:20-29`). Negative offset clamps with `.max(0)` (`search.rs:23`) →
      `b#-4` on a 3-element brane wrongly returns `Int(first)`. Should be NK when
      `|offset| > len`. Asymmetric clamp is the bug.
      - Test Bug 15: `{b={10;20;30}; last=b#-1; second=b#1; first=b#-3; oob=b#-4;}` —
        `oob` is NK, not `Int(10)`.
- [ ] Promote the 7 WIP files (6 above + `anchored_seek_negative_boundary`); the 64
      still pass byte-identical
- [ ] (@human note Bug 15 / `anchored_seek_negative_boundary` completion in FOOP-32 too)

### Phase 3: Boundary crossing (Bug 2.1)

**Note:** this is the OPPOSITE symptom of Bugs 1.x (too restrictive vs too permissive)
but the SAME mechanism — parent delegation. The parent's `stmt_idx` bounds its backward
scan, so a name defined BEFORE the nested brane is in range; one defined AFTER is not.
One mechanism fixes both. Tackle after Phase 2 since they share the search path.

- [ ] Confirm search crosses to parent brane for names defined BEFORE the nested brane
- [ ] Test Bug 2.1: `{a=10; b=20; sum=a+b; nested={inner=sum/2}; result=nested.inner;}`
      — `sum`→30 inside `nested`, `inner`→15, whole `nested` CONSTANIC, `result`→15
- [ ] Promote the WIP file; the 64 still pass

### Phase 4: Search-tree resolution (Bugs 2.2, 3.2, 4.1)

**Shared theme:** how resolved values substitute into expressions. Rule: COLLAPSE what's
resolved, PRESERVE what's not. Bug 4.1 may already improve once searches return inlined
owned values (Phase 1) rather than Rc-wrapped trees — check before writing new logic.

- [ ] When a search resolves to CONSTANT, collapse the search tree to the value (don't
      leak the inner search into the outer AST). Bug 2.2: the `a` reference resolved in
      `c` must not reappear in `d`'s AST.
- [ ] When a search is unresolved (ECONSTANIC/WOCONSTANIC), preserve it through
      concatenation (don't drop it). Bug 3.2: concatenation currently drops constanic
      entries — see ConcatenationFir merge (`fir.rs:1227-1248`), the `flat_map` that
      only keeps NormalBrane entries.
- [ ] Test Bug 2.2: `{a=1; b={c=a+1; d=c+1};}` — `d`'s AST has only a search for `c`, not `a`
- [ ] Test Bug 3.2: `{a={x=ref}; b={y=2}; c=a b;}` — `c` keeps `x=Search(ref, ECONSTANIC)`
- [ ] Test Bug 4.1: `{x=10; y=20; z=30; sum=x+y+z; avg=sum/3;}` — `avg`→20, not WOCONSTANIC
- [ ] Promote the 3 WIP files; the 64 still pass

### Phase 5: Concatenation precedence (Bug 3.1)

**May require PARSER changes**, not just evaluator — `a b` is being treated as `a`
searched by `b` rather than two operands concatenated. Check `compiler.rs` precedence
first. This is the most likely phase to touch the parser.

- [ ] Determine where `a b` becomes a search vs a concatenation (compiler/precedence)
- [ ] Fix so `a b` is concatenation of two operands
- [ ] Test Bug 3.1: `{target={...c={a=1,b=2,c=3}}; b1={x=10}; result=b1 target.c;}`
      — `result` = `{x=10; a=1; b=2; c=3}` (and `result_1 = b1(target.c)` still works)
- [ ] Promote the WIP file; the 64 still pass

### Phase 6: SF/SFF markers (Bugs 5.1–5.3)

**Spec is in FOOP-52.md §"SF/SFF Marker Specification" — read it; it was corrected
during review (Findings A–E resolved).** Key decisions:

- [ ] SFF (`<<...>>`): mark ALL `is_search()` FIRs ECONSTANIC at the START — Search,
      Index/`#-1`, HeadTail/`{}^`. This is the Finding A resolution: SFF is a
      Foolish-code copier; the only thing indeterminate after text composition is
      searches, and Index/HeadTail ARE searches (`is_search()` from Phase 1).
      - Current SFF handling: `StayFullyFoolish` step is a no-op (`fir.rs:1193`), and
        search init happens in `SearchFir::step_unanchored` (`fir.rs:968`). Need a way
        for the SFF context to force ECONSTANIC at search creation — likely a flag on
        the scope/parent, analogous to the existing `block_brane_searches`
        (`ubc.rs:105-107`, read at `fir.rs:982`). Reuse that mechanism's shape.
- [ ] SFFMark / SFMark transparent to constanic_clone: strip wrapper, clone inner
      (current `strip_sf_wrapper` `ubc.rs:326` and constanic_clone SFF strip
      `ubc.rs:460-465` already do this — preserve under owned bodies).
- [ ] SF (`<...>`) constanic_clone asymmetry (Finding D resolution — state it clearly):
      - **Concern 2 (ASSEMBLE, RHS is `<b>`):** the search for `b` runs, its result is
        constanic_cloned with **sfcc=True** into the SFMark result field → preserve
        ECONSTANIC/WOCONSTANIC.
      - **Concern 1 (CONSUME, later `c = a` finds an SFMark):** wrapper stripped, the
        consuming search uses a **NORMAL (sfcc=False)** clone → re-resolve.
      - constanic_clone (`ubc.rs:459`) needs an `sfcc` parameter (currently only
        `permit_nye`). sfcc=True: ECONSTANIC→ECONSTANIC, WOCONSTANIC→WOCONSTANIC,
        CONSTANT→CONSTANT (instead of the reset table at `ubc.rs:466-477`). Pass sfcc
        recursively.
- [ ] Test Bug 5.1: `{a=1, b=2; inner={c=<<a+b>>; c}; inner;}` — searches ECONSTANIC,
      operator WOCONSTANIC (not EMBRYONIC)
- [ ] Test Bug 5.2: `{x=5; y=10; inner={calc=<<x+y>>; doubled=calc*2};}` — same; note
      `doubled`→30 still works (calc's searches resolve when doubled clones it)
- [ ] Test Bug 5.3: `{x=10; y=<x>; z=y+5;}` — verify SF (this is formalization; current
      output is already correct per review — lock it in, don't "fix" it)
- [ ] Test the corrected SFF example (Atlas's, replaces the old vacuous one):
      `{a=1; f=<<a+b>>; g1=f; a=2; g2=f;}` — g1=`1+b`, g2=`2+b`, b stays ECONSTANIC in
      both. Demonstrates re-resolution at each reference site.
- [ ] Test Index-is-a-search inside SFF: `{...; b=<<#-1>>; ...}` — `b` stays ECONSTANIC
      until referenced/coordinated elsewhere (Finding A consequence)
- [ ] Promote the 3 WIP files; the 64 still pass

---

## Final Verification

- [ ] `cargo insta test -p foolish-core --lib` regenerates any remaining `.snap.new`
- [ ] All 64 existing `.snap` pass byte-identical (no semantic drift from the rewrite)
- [ ] All 15 WIP files produce correct output and are promoted to `.snap` (64 + 15 = 79)
- [ ] Remove `!!! WIP FOOP-52 !!!` markers from the 15 input files
      (`foolish-core/snapshot_tests/input/`)
- [ ] Re-sign promoted snapshots with the computer key if needed (see AGENTS.md
      `verify_signatures` — do NOT auto-accept; human reviews first per the CRITICAL
      snapshot rule)
- [ ] Update FOOP-52.md and FOOP-32.md to note completion
- [ ] Update FOOP-52.md status → Final

---

## Worktree Lifecycle (Major work — per AGENTS.md)

Not created yet — these boxes stay unchecked until the plan is approved and
implementation begins.

```
WORKTREE_BRANCH_NAME=scope-search-rework-foop-52
WORKTREE_FULL_FS_PATH=${HOME}/tmp/foolish-worktrees/scope-search-rework-foop-52
```

- [ ] Create worktree at ${HOME}/tmp/foolish-worktrees/scope-search-rework-foop-52 with
      branch `scope-search-rework-foop-52` (from `alpha`)
- [ ] (Task 0 + all phases happen in the worktree)
- [ ] Verify all work complete in the worktree and committed
- [ ] Merge `scope-search-rework-foop-52` to alpha
  - [ ] (Foolish uses git merge, not rebase — handle any alpha conflicts here)
- [ ] STOP! ASK HUMAN to check this box before continuing. Agent will NOT continue
      past this point automatically.
- [ ] Cleanup ${HOME}/tmp/foolish-worktrees/scope-search-rework-foop-52
  - [ ] Confirm all but cleanup checkboxes are complete
  - [ ] Remove the worktree
  - [ ] This is the last checkbox

---

## Notes

- Scope of FOOP-52 = the **15 WIP files only** for *bug* work; the other ~58 pending
  `.snap.new` files are out of scope — leave them alone (Atlas, explicit).
- The FIR is a parent-linked GRAPH (shared CONSTANT nodes by read-only ref + readable
  `Weak` parent pointers), NOT a uniquely-owned tree → keep `Rc<RefCell>`, add `Weak`
  parents. The Phase 1 byte-identical gate proves the search/scope rework preserved
  behavior; `wo_short_circuit` is the likeliest spot to move a snapshot.
- Phases 2–4 are search-behavior-related and may share a root cause; they can be
  implemented together if so, but each keeps its own WIP-promotion checkpoint.
- Encapsulation rule (Task 0, done) governs the rework: free functions in `ubc.rs`
  migrate onto the types; string dispatch → enum/method dispatch; the three search
  methods live ON the brane.

## Last Updated

**Date**: 2026-06-07 (later, correction)
**Updated By**: Claude Code 2.1.119 (Claude Code); Sonnet 4.6
**Changes**: CORRECTED architecture — the FIR is a parent-linked GRAPH, not a tree;
the owned-`Box<Fir>`/mechanical-substitution framing was wrong. Settled model:
`Rc<RefCell<Fir>>` children + `Weak<RefCell<Fir>>` parents, mutate-self. Rewrote
Framing and Phase 1: `Weak` parents made real+used; three recursive brane search
methods (`iterate_ib_statements` / `search_immediate_brane` /
`search_ancestral_branes` + `line_of_child`) replace the flat `Scope`; added REQUIRED
unit tests for them (flat + nested). Renamed worktree `fir-owned-bodies-foop-52` →
`scope-search-rework-foop-52`. Removed `split_at_mut`/Box references from active tasks.

**Date**: 2026-06-07
**Updated By**: Claude Code 2.1.119 (Claude Code); Sonnet 4.6
**Changes**: Full rewrite + per-checkbox rationale. Reframed as Major. Added Task 0
(AGENTS.md + encapsulation), `is_search()` predicate, `fir_variant()`→`FirKind`,
dead-field removal, `wo_short_circuit` query, worktree lifecycle, Bug 15 → Phase 2.
[Note: this entry's owned-`Box<Fir>` architecture was superseded by the correction
above.]
