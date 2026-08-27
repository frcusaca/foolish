# UFM (Unstay Foolish Marker) — scoping study

**Status: STUDY ONLY. Nothing here is implemented. No code was changed** (one throwaway
experiment was run and reverted; see §D).

**Scope**: assess how big a change the proposed UFM feature is, find in git history what the
"old default" it is meant to restore actually was, and rigorously test the hypothesis that
restoring UFM-type behavior recovers three currently-diverged einmo baselines.

**Verdict up front:**

- **Size: MEDIUM.** Not small — it is a new FIR kind, which means a lexer change with a real
  collision, a new token, a new AST node, a new `Astn`→FIR compile arm, a new `foolish-core`
  `Fir` variant with ~15 satellite sites, a sequencer arm, plus the budget plumbing. Not large
  — none of it changes the step loop, no NYES state is added, and the budget hook (`unlimited()`)
  already exists and is exactly right.
- **Hypothesis (§D): NO, it does not hold, on two independent grounds.** Neither ground is a
  matter of degree; both are structural. See §D — this is the section that matters.

> **SUPERSEDING DESIGN (human, 2026-08-27): UFM is IMPLEMENTED as an operator.**
> It is still called the **Unstay Foolishness Mark**, still written `<@ … @>`, and to a
> Foolisher it is still a mark. What changes is only its internal shape: rather than a
> deferral wrapper structurally parallel to `StayFoolishFir` (which is what sections C
> and E below assumed), it behaves mechanically like an operator — it owns its content
> in `foolish_children`, waits for it to go constanic, then strips and re-steps it
> through `ubc_children`. See **§F**. That shape is simpler and resolves the
> compile-time/step-time tension in §E4–E5.

---

## A. The budget and stepping logic as it stands

All line numbers are against the current worktree
(`/yolo/foolish/.claude/worktrees/foop-55-event-handlers`, branch
`worktree-foop-55-event-handlers`, tip `7e60bee6` plus the uncommitted `StripBudget`
`Option` refactor).

### A.1 `StripBudget` — `foolish-ubca/src/fir_kinds.rs:126-197`

```rust
pub(crate) struct StripBudget {
    /// `None` = unlimited: every strip is permitted and nothing is ever
    /// decremented. `Some(n)` = `n` strips remain on this path.
    remaining: Option<u32>,
}

impl StripBudget {
    fn new(n: u32) -> Self { StripBudget { remaining: Some(n) } }

    #[expect(dead_code, reason = "constructor for the unlimited case, no caller yet")]
    fn unlimited() -> Self { StripBudget { remaining: None } }

    fn fresh() -> Self { StripBudget::new(1) }

    fn spend(self) -> (bool, Self) {
        match self.remaining {
            None => (true, self),
            Some(0) => (false, self),
            Some(n) => (true, StripBudget::new(n.saturating_sub(1))),
        }
    }
}
```

The type is `Copy` and passed **by value**, which is the mechanism that makes the budget
per-root-to-leaf-path rather than per-tree: a sibling recursion gets its own copy of whatever
the parent handed down, so spending in one subtree cannot starve a sibling. FOOP-55 §5 records
that this distinction was found the hard way — a per-tree budget broke twelve tests, because
`'mod`/comparison operators take `<<#-2>>` and `<<#-1>>` as *siblings*.

**Confirming the reading of `unlimited()`**: yes, it is the intended UFM hook, and it is
correct for that purpose. `spend()` on `None` returns `(true, self)` — permit the strip, hand
the same unlimited budget onward — which is exactly "removes the effects of `<>`/`<<>>` in ALL
descendants, for the whole path below." It is currently dead code with an `#[expect]` and a
`reason` naming precisely this situation. **Nothing else in the crate constructs it** (verified
by grep: one hit, the definition itself).

### A.2 `constanic_clone` — `fir_kinds.rs:293-317`

```rust
pub(crate) fn constanic_clone(
    fir_ref: &FirRef,
    new_parent: &Weak<RefCell<dyn Fir>>,
    index: usize,
    skip_foolish_children: bool,
    inside_sf_mark: bool,
) -> FirRef {
    let (_, stay_budget) = if inside_sf_mark {
        StripBudget::fresh().spend()      // => Some(0): exhausted before the clone starts
    } else {
        (true, StripBudget::fresh())      // => Some(1)
    };
    Self::_inner_constanic_clone(fir_ref, new_parent, index, false, skip_foolish_children, stay_budget)
}
```

This is the **only public entry point**. Every clone is one *operation* with one budget. Two
starting states exist today and only two: `Some(1)` (ordinary) and `Some(0)` (already exhausted,
because the stepper is inside an SF mark). `disable_nyes_reset` always starts `false` here.

### A.3 `_inner_constanic_clone` — `fir_kinds.rs:327-648`

The recursive worker. Its first act is the mark-encounter arm (`:335-374`):

```rust
if matches!(fir_ref.borrow().kind(), FirKind::StayFoolish | FirKind::StayFullyFoolish) {
    let (may_strip, stay_budget) = stay_budget.spend();
    if !may_strip {
        return Rc::clone(fir_ref);           // MARK RETAINED — shared, not deep-copied
    }
    let source = fir_ref.borrow();
    if source.kind() == FirKind::StayFoolish
        && let Some(constanic_result) = source.core().ubc_children().into_iter().next() {
        return Self::_inner_constanic_clone(&constanic_result, new_parent, index,
                                            disable_nyes_reset, skip_foolish_children, stay_budget);
    }
    if let Some(inner) = source.core().foolish_children().first().cloned() {
        return Self::_inner_constanic_clone(&inner, new_parent, index,
                                            disable_nyes_reset, skip_foolish_children, stay_budget);
    }
    eprintln!("ALARM: SF/SFF node has no children — cloning wrapper as-is");
}
```

Three things to note:

1. **Strip = descend past the wrapper.** A stripped mark does not produce a wrapper node at
   all; the clone becomes a clone of what the mark contained. The *spent* budget is what is
   passed down that same path (`stay_budget` shadowed by `spend()`'s second return).
2. **Retain = share the original `Rc`.** Sound per FOOP-55 §5 because an unstripped mark has
   not searched, so it holds no resolved brane reference and no per-site state.
3. **SF prefers its `ubc_children[0]`** (its already-resolved value) over its
   `foolish_children[0]` (the unrun expression). SFF has no such preference at the strip site —
   it always takes `foolish_children[0]`. This asymmetry matters in §D.

Below the mark arm, `:375-387` short-circuits shared-by-reference nodes (`IndepInt`/`Nk`
always; anything `Constant`/`Independent` that is not a `Brane`), then `:390-647` is the big
`match kind` — 17 arms, one per `FirKind`. Every structural arm calls
`clone_children_budgeted(..., stay_budget)`, threading the same budget value into all children.
`FirKind::StayFoolish | FirKind::StayFullyFoolish` at `:560-562` is
`unreachable!("SF/SFF stripped at fn top")`.

### A.4 `clone_children_budgeted` / `clone_children_for_constanic_clone` — `fir_kinds.rs:202-258`

```rust
pub(crate) fn clone_children_for_constanic_clone(...) -> ProtoBrane {
    // Children of one clone share that clone's budget (FOOP-55 §5).
    let budget = StripBudget::fresh();
    Self::clone_children_budgeted(source, self_weak, new_parent, nyes, sfm,
                                  skip_foolish_children, budget)
}

pub(crate) fn clone_children_budgeted(..., budget: StripBudget) -> ProtoBrane {
    let cloned_children: Vec<FirRef> = source.foolish_children().iter().enumerate()
        .map(|(i, c)| ProtoBrane::_inner_constanic_clone(c, self_weak, i, sfm, false, budget))
        .collect();
    let core = ProtoBrane::new(cloned_children, new_parent.clone(), nyes.transform_for_clone(sfm));
    for ubc in source.ubc_children() {
        core.push_ubc_child(ProtoBrane::_inner_constanic_clone(&ubc, self_weak, 0, sfm, false, budget));
    }
    core
}
```

`budget` is `Copy`, so each `.map()` iteration hands each child an *independent copy* — the
sibling-independence property. Note `clone_children_for_constanic_clone` (used only by
`system_foo::ComparisonFir::constanic_clone` and friends, `fir_kinds.rs:200-201`) starts a
**fresh** budget rather than inheriting — a small inconsistency, called out in §E.

### A.5 `SearchFir::clone_stmt_result` / `handle_found` — `fir_kinds.rs:1705-1738`

```rust
fn clone_stmt_result(stmt: &FirRef, new_parent: &Weak<RefCell<dyn Fir>>,
                     inside_sf_mark: bool) -> FirRef {
    let body = statement_value_for_comparison(stmt).expect("statement must have a body");
    let index = stmt.borrow().as_stmt_line_number().unwrap_or(0);
    ProtoBrane::constanic_clone(&body, new_parent, index, false, inside_sf_mark)
}

fn handle_found(&self, stmt: FirRef, _nyes: Nyes, scope: &Scope) {
    ...
    let clone = Self::clone_stmt_result(&stmt, &self_weak, scope.has_ancestral_sfm);
    push_search_result_pair(&self.core, clone, stmt);
    self.core.set_nyes(Nyes::Braning);
}
```

`fir_kinds.rs:1735` is **the only site in the crate that reads `scope.has_ancestral_sfm`**
(verified by grep; the two other hits are unit-test assertions at `:5314`/`:5321`). It is also
the newest — see §B.5.

### A.6 `step_inner` — `foolish-ubca/src/fir_trait.rs:637-677`

```rust
let this_kind = this.borrow().kind();
let mut child_scope = if this_kind == FirKind::StayFoolish {
    scope.with_ancestral_sfm(true)
} else {
    scope.clone()
};
```

**Two facts to flag, both load-bearing for §D and §E:**

- The flag is set for `FirKind::StayFoolish` **only** — `StayFullyFoolish` does *not* set it,
  despite SFF being the *stronger* mark. There is no comment explaining the omission.
- The flag is never *cleared* on the way down. `scope.clone()` carries a `true` set by an
  ancestor SF through every subsequent non-SF frame. So `has_ancestral_sfm` means "somewhere at
  or above me on this step recursion there is an SF wrapper," which is what its name says, but
  it is a *stepping-context* property, not a property of the node being cloned.

`Scope` (`fir_trait.rs:73-93`) is three fields: `current_brane`, `current_statement`,
`has_ancestral_sfm: bool`.

### A.7 `StayFoolishFir` / `StayFullyFoolishFir` `fir_op_step` — `fir_kinds.rs:3111-3222`

Both are the "external-trigger" shape FOOP-55 §11 names: PREMBRIONIC/EMBRYONIC pushes the
single child as a task; BRANING waits for it and then pushes its resolved value into
`ubc_children`. SF (`:3129-3149`) gates on `expr_nyes.is_constanic()`; SFF (`:3186-3213`) gates
on `_decide_nyes_due_to_children(&children).is_some()` and applies
`SearchFir::nyes_from_found` to the result's NYES. **Neither reads `scope`** (both take
`_scope`). The mark's *deferral* semantics live entirely in `_inner_constanic_clone`'s
mark arm, not in these two `fir_op_step`s.

---

## B. Git archaeology — what WAS the old default?

**Answer: it was found, it is unambiguous, and the human's recollection is correct.** The old
default was *unlimited, unconditional* stripping, and it held for roughly seven weeks.

### B.1 Before any stripping (through `0bce9dd0`, 2026-06-19)

The SF/SFF clone arm did not strip at all — it **rebuilt a fresh wrapper** at
`Nyes::Prembrionic`. A TODO in the code named this a spec violation:

> ```
> // TODO(FOOP-62, 2026-06-19): SPEC VIOLATION — "THE BIG BUT" (rev 14, §9.x/§10.1).
> // When a SEARCH clones an SF-mark, the clone must STRIP the mark — clone the inner
> // expression in NORMAL mode (descendent_of_sfm_and_foolishly_ignorant = false), so an
> // ECONSTANIC inner re-resolves. This arm instead rebuilds a fresh StayFoolish wrapper
> // (at PREMBRYONIC), which is wrong. Fix: strip + clone inner.
> ```

### B.2 The UFM-type default — commit `94ed10d2`, 2026-06-19

**`94ed10d2` "FOOP-62 Phase -1: strip SF/SFF mark on constanic-clone (THE BIG BUT)"**
(Fri Jun 19 17:06:53 2026 -0700) introduced stripping, with **no cap of any kind**:

```rust
FirKind::StayFoolish => {
    match borrowed.core().foolish_children().first() {
        Some(inner) => {
            constanic_clone_at(inner, new_parent, index, descendent_of_sfm_and_foolishly_ignorant)
        }
        None => Rc::clone(fir_ref),
    }
}
```

(identical shape for `StayFullyFoolish`). The recursive call is the plain `constanic_clone_at`,
which strips the next mark it meets, and the next, indefinitely. **This is the UFM behavior**:
one clone removed every mark on every path below it.

Commit message:

> ```
> - StayFoolish/StayFullyFoolish clone arms now STRIP the mark: clone the inner
>   expression directly, no wrapper rebuilt. Per Atlas, the incoming
>   descendent_of_sfm_and_foolishly_ignorant flag is PASSED ON to the inner clone
>   (not forced false), so a nested SF inside an outer SF's RHS stays foolish.
> - Snapshots unchanged (85/86); the search path already pre-strips via
>   unwrap_sf_sff, so this makes the clone itself spec-correct.
> ```

### B.3 The change away from it — commit `779b63f5`, 2026-08-11

**`779b63f5` "FOOP-55 §5 Phase 3A: implement the SFF strip budget (per PATH, not per tree)"**
is the commit that ended the UFM default:

> ```
> Implements §5. `constanic_clone_at` becomes a thin entry point that starts a
> fresh StripBudget and delegates to `constanic_clone_at_budgeted`, which carries
> it down; `clone_children_for_constanic_clone` gains the same split. At an SF/SFF
> node the budget is spent if available (strip as before) or the mark is RETAINED
> and shared via `Rc::clone` if not — sound because an unstripped mark has not
> searched, so it holds no resolved brane reference and no per-site state.
>
> CORRECTION TO THE SPEC, found by the tests. §5 says the budget is "per clone
> TREE". That is wrong, and 12 existing tests proved it: the comparison and
> modulo operators take TWO SFF operands (<<#-2>> and <<#-1>>) as SIBLINGS, and a
> per-tree budget let only the first resolve — modulo_basic_semantics went from
> Some(1) to None, and every comparison operator broke with it.
>
> The budget is per ROOT-TO-LEAF PATH. StripBudget is Copy and passed BY VALUE...
> ```

**The reason given was not "unlimited stripping is wrong in principle."** It was Euler 1's
`'ite`: FOOP-55 §5 needed `<< <<X>> >>` to sit out one coordination so the unselected branch of
a lookup table would not recurse. The budget of 1 is the *minimum* mechanism that produces that
one-level deferral; unlimited stripping cannot produce it at all. Follow-up `920e6d7d` (same
day) is doc-only, amending FOOP-55 §5's prose from "per tree" to "per path". `bbff6b0d`
(2026-08-14) found the only budget test was vacuous and replaced it.

### B.4 `Nyes::transform_for_clone` — `foolish-core/src/fir.rs`

Introduced by **`d48a987c` "new round of review"** (2026-07-01), moving an existing UBCa-local
free function `clone_nyes` into `foolish-core`:

```rust
/// Transform NYES for constanic-clone: the clone's initial state.
/// - SFM-descendant: preserve the source NYES verbatim (foolishly ignorant).
/// - CONSTANT / INDEPENDENT / NK: keep as-is (terminal, no re-evaluation).
/// - Everything else → EMBRYONIC (clone must re-evaluate in new context).
pub fn transform_for_clone(self, descendent_of_sfm_and_foolishly_ignorant: bool) -> Nyes {
    if descendent_of_sfm_and_foolishly_ignorant { return self; }
    match self {
        Nyes::Constant | Nyes::Independent | Nyes::Nk => self,
        Nyes::Econstanic | Nyes::Woconstanic | Nyes::Prembrionic
        | Nyes::Embryonic | Nyes::Braning => Nyes::Embryonic,
    }
}
```

`clone_nyes` itself dates to `0bce9dd0` (2026-06-19). The move to `foolish-core` also changed
the pre-constanic target from `Prembrionic` to `Embryonic`.

### B.5 `inside_sf_mark` on `clone_stmt_result` — commit `0eb0bf29`, **2026-08-26 (yesterday)**

This is the *most recent* change in the area and the direct cause of the divergences in §D:

> ```
> FOOP-55: preserve the mark when a search inside an SF mark copies its find
>
> A search that resolves while running inside an SF mark now constanic-
> copies the found body into its ubc_children WITHOUT stripping the mark:
> clone_stmt_result takes `inside_sf_mark` and passes it to
> constanic_clone, which spends the strip budget down to zero before the
> copy begins. SF enforcement happens during constanic cloning.
>
> Traced on `{a={1,2}, b=<<#-2>>, c= a b}`. c's foolish_children are
> [SFMark(search('a')), SFMark(search('b'))]. The search for b runs, finds
> b, and copies b's body into its OWN ubc_children. Before this change
> that copy arrived as a bare Index/Constant -- the <<#-2>> stripped and
> already run -- so the element settled Constant and the join fired. Now
> it arrives as StayFullyFoolish/Woconstanic, mark intact and unrun, the
> element settles Woconstanic, and c does not join.
> ```

Before `0eb0bf29`, `handle_found` took `_scope` (unused) and the call was hardcoded
`ProtoBrane::constanic_clone(&body, new_parent, index, false, false)` — a search's found-body
clone **always started with a fresh budget of 1**, regardless of ambient scope. That commit's
own notes record it left `misc/sf_of_sff` and `foop/62/sf_sff_nested_combined` diverged, marked
*"Justified mechanism, unjustified outcome — needs review."*

### B.6 `descendent_of_sfm_and_foolishly_ignorant` and `Scope::has_ancestral_sfm`

- Terminology coined in spec-only `d997960d` "FOOP-62 rev 14: ignorance terminology + foolish-flag
  clone model" (2026-06-19): "normally ignorant" (`false`) vs "foolishly ignorant" (`true`).
- `Scope::has_ancestral_sfm: bool` added in `0e57636c` (2026-06-19), nine minutes before the
  parameter was wired in by `0bce9dd0`. It has been a plain `bool` ever since; only its
  *readership* changed.
- `descendent_of_sfm_and_foolishly_ignorant` was renamed to `disable_nyes_reset` and demoted
  to `_inner_constanic_clone`'s parameter per the design in `65d28f80` "FOOP-55 D9: correct
  root-cause design (constanic_clone, not step_inner reset)" (2026-08-26):

  > ```
  > New design (FOOP-55.md D9 item 3, FOOP-55.plan.md Phase 3I): rename
  > constanic_clone_at to an internal _inner_constanic_clone(node,
  > stay_budget, disable_nyes_reset). New public constanic_clone(node) is
  > the entry point every Fir kind calls, always starting stay_budget=1,
  > disable_nyes_reset=false. The mark-encounter arm decides
  > disable_nyes_reset locally, from its own remaining budget, never from
  > an ambient Scope.
  > ```

  Note the tension: `65d28f80`'s stated goal is that the ambient `Scope` flag stop feeding
  clone decisions, and `0eb0bf29` — one day later — reintroduces exactly that coupling at
  `handle_found`. Flagged in §E.

### B.7 Timeline

| Date | Commit | Event |
|---|---|---|
| 2026-06-19 | `d997960d` | Spec-only: "foolishly ignorant" terminology coined |
| 2026-06-19 | `0e57636c` | `Scope::has_ancestral_sfm` added |
| 2026-06-19 | `0bce9dd0` | `descendent_of_sfm_and_foolishly_ignorant` + `clone_nyes`; SF/SFF arm still rebuilds the wrapper |
| 2026-06-19 | `94ed10d2` | **Stripping becomes the default — UNLIMITED ("THE BIG BUT")** |
| 2026-07-01 | `d48a987c` | `clone_nyes` → `Nyes::transform_for_clone` in `foolish-core` |
| 2026-08-11 | `779b63f5` | **`StripBudget` caps stripping at 1 per path** — for FOOP-55 §5's `'ite` |
| 2026-08-11 | `920e6d7d` | Doc-only: spec prose corrected per-tree → per-path |
| 2026-08-14 | `bbff6b0d` | Vacuous budget test replaced with real ones |
| 2026-08-26 | `65d28f80` | Design-only: `constanic_clone`/`_inner_constanic_clone` split, budget per call not inherited |
| 2026-08-26 | `0eb0bf29` | `inside_sf_mark` wired from `scope.has_ancestral_sfm`; **causes the §D divergences** |
| uncommitted | — | `StripBudget.remaining: Option<u32>` + dormant `unlimited()` — the UFM hook |

**The name "UFM"/"unstay" appears nowhere** in code, docs, or history (verified by
`grep -rni "unstay|\bufm\b"` across the tree). The name is new; the behavior is not.

---

## C. Blast radius

### C.1 Lexer — `foolish-parser/src/lexer.rs`

`<` dispatches to `lt_token()` (`:146-149`, body at `:397-441`), which tries in order:
`<<=>>>`, `<=>`, `<<`, else `Lt`. `>` handles `>>` at `:151-156`, else falls to the single-char
match. `*` is a plain single-char `Token::Mul` at `:246-249`.

**`<*` / `*>` COLLIDES with existing syntax, and not merely at the lexer.** Measured on the
current tree:

| source | result today |
|---|---|
| `{x=5; a = <* x *>;}` | **parse error** — `expected primary expression, found Gt at line 3, column 8` |
| `{x=5; a = <* x>;}` | **parses and evaluates** to `a=<NK ??? (unknown operator: * (1 operands))>` |
| `{x=5; a = <*x>;}` | same as above |
| `{x=5; a = < x *>;}` | parse error at the `>` |

The cause is that `Token::Mul` is a legal **prefix unary operator** (`parser.rs:747-754`,
`parse_unary_expr`'s `Some(Token::Mul)` arm producing `Astn::UnaryOp{op:"*"}`), so `<* x>`
already parses today as `StayFoolish { UnaryOp { "*", Identifier(x) } }`. Choosing `<* … *>`
would therefore **silently change the meaning of an existing, currently-parseable program
form**. It is not a free slot. `*>` is unambiguous (`*` is never postfix), but `<*` is not.

Practical consequences if `<* … *>` is kept:

- `lt_token()` needs a `<*` branch **before** the `<<` branch (it must also not shadow
  `<<=>>>`, which starts `<<`, so `<*` is safe in that ordering); a new `>` branch for `*>`
  must precede the single-char `*` match, which lives in a *different* function — the `*` arm
  at `:246` would have to peek at `*` + `>` , or `*>` be handled where `>>` is at `:151`.
  Handling `*>` at the `*` site is the cleaner of the two.
- The unary-`*` production would need to be re-examined: is `*x` meaningful? It currently
  compiles to an operator that evaluates to `NK ??? (unknown operator: *)` — i.e. it is
  already dead syntax that only produces an NK. If it is genuinely dead, deleting it removes
  the collision entirely, but that is its own decision with its own test fallout.

**Recommendation for the human**: `<* … *>` is workable but requires deciding what happens to
unary `*`. Alternatives worth a look, with the specific conflict noted for each: `<! … !>`
(`!` currently starts comments `!!`/`!!!` — likely worse); `<@ … @>` (`@` is `Token::At`, the
FOOP-55 §8 search-position operator, postfix — `<@` is not currently reachable, so this may be
the cleanest ASCII pair); or a Unicode pair in the house style (cf. `⬤`, `<̲`), which collides
with nothing by construction and matches the project's stated preference for Unicode operator
forms.

### C.2 Token enum, AST node, parser

- `foolish-parser/src/token.rs:31-36` — add `LtStar` / `StarGt` (or one `Ufm`-delimited pair)
  to `Token`; `token.rs` is a plain enum, so this is 2 lines plus 2 `Display` arms at
  `parser.rs:1347-1367`.
- `foolish-parser/src/ast.rs:153-160` — add `Astn::Ufm { expr: Box<Astn> }` beside
  `StayFoolish`/`StayFullyFoolish`.
- `foolish-parser/src/parser.rs:1164-1180` — a third arm in `parse_primary` exactly mirroring
  the two existing ones (advance, `parse_expr`, `expect(close)`, wrap).
- `parser.rs:550-570` — `Token::Lt`/`Token::LtLt` are in the *continues-a-concatenation* token
  set; the UFM open token almost certainly belongs there too, or `{a} <*b*>` will silently
  become two statements the way `{1,2} 'or` used to.
- Total in `foolish-parser`: SF/SFF currently occupy **8 sites** across `ast.rs` + `parser.rs`.
  Expect a comparable count for UFM.

### C.3 New `FirKind` variant + FIR struct, and every exhaustive `match`

- `foolish-ubca/src/fir_trait.rs:38-39` — add `FirKind::Ufm`.
- `foolish-ubca/src/fir_kinds.rs` — a new `UfmFir { core: ProtoBrane }` plus `impl Fir`,
  structurally a copy of `StayFoolishFir` (`:3099-3157`) or `StayFullyFoolishFir`
  (`:3160-3222`), ~50 lines.

**Exhaustive `match FirKind` sites that would fail to compile (this is the useful list):**

| file:line | what it is | UFM arm needed? |
|---|---|---|
| `fir_kinds.rs:390` | `_inner_constanic_clone`'s `match kind` — **fully exhaustive, all 17 arms, no `_`** | **YES — compile error without it.** This is *the* arm to write (see C.4) |
| `evaluator.rs:873` | `proto_to_core_fir_inner`'s tail — has `FirKind::Unknown \| FirKind::FoolRef =>` and named arms | Needs an arm or falls into whatever catch-all exists (see C.6) |
| `evaluator.rs:229`, `:274`, `:315` | `proto_to_core_fir_sff_body` / `_sff_operand` / `anchor_to_core_fir` — each ends `_ => proto_to_core_fir_inner(...)` | No compile error; behavior inherited. Verify it is right |
| `fir_trait.rs:270` | `_get_my_statement` — `FirKind::Statement` + `_` | No |

Non-exhaustive but semantically UFM-relevant `matches!` sites that must each be **audited**,
because they enumerate `StayFoolish | StayFullyFoolish` and a UFM is a third mark-like kind:

- `fir_kinds.rs:335` — the mark-encounter arm's own guard (must include UFM).
- `fir_kinds.rs:560` — the `unreachable!("SF/SFF stripped at fn top")` arm (must include UFM,
  or UFM falls through to a `match` arm that does not exist).
- `fir_kinds.rs:6403` — `peel_marks` test helper.
- `fir_trait.rs:656` — `step_inner`'s `has_ancestral_sfm` set (see §E).
- `evaluator.rs:526-529` — the `is_complex_type` set in the search-rendering path.
- `compiler.rs:70-75` — `classify_concat_element` rule 1 (see C.5).
- `fir_trait.rs:434` — `is_search_kind` (UFM is not a search; no change, but confirm).

### C.4 `constanic_clone`'s mark-handling arm — the semantic core

Assuming decisions 1 and 2 (UFM removes SF/SFF effects in **all** descendants; budget
unlimited for the **whole** path below), the change is small and localized:

```rust
// fir_kinds.rs, top of _inner_constanic_clone
if fir_ref.borrow().kind() == FirKind::Ufm {
    // Entering a UFM makes the remainder of THIS PATH unlimited.
    let inner = fir_ref.borrow().core().foolish_children().first().cloned();
    return match inner {
        Some(inner) => Self::_inner_constanic_clone(&inner, new_parent, index,
                                                    disable_nyes_reset, skip_foolish_children,
                                                    StripBudget::unlimited()),
        None => Rc::clone(fir_ref),
    };
}
```

This is `unlimited()`'s first caller and removes the `#[expect(dead_code)]`. The
sibling-independence property still holds, because `budget` remains `Copy`-by-value; an
unlimited budget simply propagates unlimited down every path *below the UFM*, which is exactly
the specified reach.

**Design questions this arm raises** (see §E): does the UFM wrapper survive into the clone
(rendered) or vanish like a stripped mark? The sketch above makes it vanish. If it must
survive, the arm is not a `return` but a rebuild, and then a UFM *inside* a clone re-arms
unlimited on every subsequent clone through it — a different and much stickier semantics.

### C.5 `build_fir` and `classify_concat_element` — `foolish-ubca/src/compiler.rs`

`classify_concat_element` (`:68-113`) is a five-rule ordered table. **Rule 1** (`:70-75`):

```rust
if matches!(ast, Astn::StayFoolish { .. } | Astn::StayFullyFoolish { .. }) {
    return ConcatElemKind::AsWritten;
}
```

Its doc comment says the ordering exists *by specification*: "a constituent the Foolisher has
already marked is compiled **as written**, adding nothing." A UFM is a mark the Foolisher wrote,
so **rule 1 is the natural home** — `Astn::Ufm` joins that `matches!` and the element compiles
as written, no auto-SF/auto-SFF wrapper added. This is one line.

But it is not obviously right, and the human should decide: rule 1's *rationale* is "the
Foolisher already deferred this, don't double-defer." A UFM is the **opposite** of a deferral —
it un-defers. Auto-SF-wrapping a UFM element (`<<* … *>>`… i.e. SF-of-UFM) would be
self-contradictory, so `AsWritten` is probably correct, but the reasoning differs from
SF/SFF's and should be written into the comment rather than silently inherited.

`build_fir` (`compiler.rs:557-575` for the SF/SFF `Astn` arms; the `under_sff` flag threaded
throughout) needs a third arm. Note `build_fir`'s `under_sff: bool` parameter causes searches
built under an SFF to start ECONSTANIC so they never run (`compiler.rs:98-101` comment) — a
UFM inside an SFF plausibly needs to **reset `under_sff` to `false`**, since un-staying is
precisely its job. That is a real semantic decision, not plumbing (§E).

Also: `validate_astn` (`compiler.rs:264-265`) and `compiler.rs:799-803`
(`AssignmentOperator::SF`/`SFF`) each need a UFM decision — is there a `=<*` assignment form?

### C.6 Rendering — `foolish-ubca/src/evaluator.rs` and `foolish-core`

Two layers:

1. **UBCa→core conversion**, `evaluator.rs:663-765`. `FirKind::StayFoolish` (`:663-751`) is
   ~90 lines with special cases for SF-of-search and SF-of-SF; `FirKind::StayFullyFoolish`
   (`:752-765`) is ~14 lines. A UFM arm would follow the SFF shape (simple wrap) unless the
   UFM must render its inner value transparently, in which case it may be a pass-through.
2. **`foolish-core` `Fir` enum**, `foolish-core/src/fir.rs`. `StayFoolish`/`StayFullyFoolish`
   occupy **61 sites across 3 files** (`fir.rs`, `sequencer.rs`, `sequencer_tests.rs`). A new
   variant means touching: the struct (`:344-352`), `children()` (`:411-412`), the `Fir` enum
   (`:524-525`), `kind_name()` ×2 (`:701`, `:911`), `state()`/`set_state()` ×2 pairs
   (`:715-716`, `:877-878`, `:892-893`), `as_stay_*` accessors (`:1375-1387`), `Steppable`
   impls (`:1262-1289`), JSON serialize (`:1477-1489`) and deserialize (`:1716-1740`), plus a
   `UfmFirBuilder` (`:2061+`). Call it **~15 mechanical sites** in `fir.rs`.
3. **The humanizing sequencer**, `foolish-core/src/sequencer.rs:462-493`. SF renders as
   `<STATE … >`, SFF as `<<STATE … >>`, both with a `should_show_nyes()` pass-through when the
   state is not shown. UFM would render as `<*STATE … *>` by the same 15-line pattern — but
   **every einmo baseline that contains a UFM would then carry that rendering**, and the
   `should_show_nyes()` transparency rule needs deciding (does a settled UFM render at all, or
   vanish like a settled SF does?).

`foolish-ubca/src/system_foo.rs` has no SF/SFF-kind matches that need an arm (its hits are
`FirKind::Creation`/`Nk`/`Comparison`), so it is likely untouched.

### C.7 Tests that must be added

Per AGENTS.md's stated requirement: a new FIR kind **must** get a `ufm_nyes_transitions` unit
test in `fir_kinds.rs`'s tests module, asserting the progression via `assert_progression`. Plus
budget-interaction unit tests mirroring the existing per-path/sibling ones, and einmo cases.

### C.8 Size verdict: **MEDIUM**

| Argument for SMALL | Argument for LARGE |
|---|---|
| The budget hook (`unlimited()`) exists and is exactly right | A **new FIR kind** is the single most cross-cutting change shape in this codebase — 4 crates |
| The semantic change is ~10 lines in one function (C.4) | The `foolish-core` `Fir` enum alone is ~15 mechanical sites |
| No new NYES state; step loop untouched | The lexer has a **real, measured collision** (C.1) that forces a syntax decision first |
| `StayFullyFoolishFir` is a 60-line template to copy | Sequencer rendering appears in every baseline containing a UFM |
| No search semantics change | Open design questions (§E) block the mark-arm's exact shape |

**MEDIUM**, and the mechanical parts dominate the risky parts. The genuinely hard work is
(a) choosing a non-colliding syntax and (b) settling §E's questions; after that it is a
well-trodden add-a-kind refactor. Roughly a Phase, not a Major, in the project's own
segmentation vocabulary — **provided §E is settled first**.

---

## D. The hypothesis — **it does not hold**

**Claim under test**: restoring UFM-type behavior will recover the three currently-diverged
`checked` values.

**The three divergences, measured on the current tree** (`cargo test -p foolish-ubca --lib --
einmo_gate_checked`, 2026-08-26). Confirmed present, and confirmed to be exactly the three the
task names (there are also five *new* cases missing from `checked/` and one INPUT-diff on
`misc/concat_sf_f_more`, all pre-existing and out of scope here):

**(i) `misc/sf_of_sff`** — input:
```foolish
{a = 1; b = 2; sff = <<a + b>>; sf = <sff>; a = 10; sf; sff;}
```
`checked` has the bare `sf;` statement settle to `12`; `output` has it stay an unresolved
search whose `result=` is a still-wrapped `<<WOCONSTANIC Op+(...) >>`.

**(ii) `foop/62/sf_sff_nested_combined`** — `sfsff = <sff>;` then `rsfsff = sfsff;`.
`checked` has `rsfsff=0` (`x` recoordinates to `-2`, so `-2 + 2 = 0`); `output` has `rsfsff`
as an unresolved search over a still-wrapped `<<WOCONSTANIC Op+(...) >>`.

**(iii) `foop/42/humanizing_sequencer_formatting_exhaustive_aka_hfs`** — `sf_chain`'s result
gains an extra search layer: `result={a=1;b=2}` becomes
`result=?(result={a=1;b=2}, pattern='^sfˍtarget$', UNANCHORED)`.

### D.1 Ground one — the marker must be WRITTEN, and these inputs do not contain it

This is decisive on its own and needs no experiment. Design decision 3 fixes UFM as **a
wrapper, like SF/SFF** — `<* expr *>` produces a FIR node only where a Foolisher typed it.
`grep -c` over the three inputs: **zero** occurrences of any candidate UFM syntax; the files
are the ones quoted above and contain only `<>`, `<<>>`, and ordinary statements.

An opt-in marker changes the behavior of programs that *use* it. `sf_of_sff.foo`,
`sf_sff_nested_combined.foo`, and the HFS input do not use it. Therefore **adding UFM cannot
change their OUTPUT by a single byte**, unless the same change also alters the DEFAULT — and
altering the default is not what "an explicit, opt-in marker" means. The hypothesis, taken
literally, is self-defeating: the feature as specified is precisely the feature that cannot
touch these three cases.

The only ways out are all changes of a different kind, and each should be named as such rather
than folded into "add UFM":
- **(a)** Also change the default (revert `0eb0bf29`'s coupling) — a *separate* change, and one
  that does not need UFM to exist at all.
- **(b)** Edit the three inputs to contain UFM markers — which changes what the tests test, and
  is a re-specification of three baselines (two of which belong to FOOP-62 and FOOP-42, i.e.
  **foreign FOOPs**, which the non-regression invariant forbids touching to fix your own work).
- **(c)** Make UFM implicit somewhere (e.g. auto-applied by the concatenation classifier) — no
  longer opt-in, and contradicts decision 3.

### D.2 Ground two — the *behavior* would not restore the values either. Measured.

Even granting a hypothetical implicit/default UFM, the recovery does not happen. I ran the
experiment rather than reasoning about it.

**Experiment** (throwaway, reverted; `git status` verified clean afterward): in
`constanic_clone` (`fir_kinds.rs:304-308`), replace the `inside_sf_mark` branch with the UFM
behavior — unlimited stripping down the whole path:

```rust
let (_, stay_budget) = if inside_sf_mark {
    (true, StripBudget::unlimited())     // EXPERIMENT (was: StripBudget::fresh().spend())
} else {
    (true, StripBudget::fresh())
};
```

This is the most generous possible reading of the hypothesis: it makes *every* clone reached
while inside an SF mark behave the UFM way. Result:

| case | `checked` | before experiment | **after experiment** |
|---|---|---|---|
| `foop/42/hfs` | `result={a=1;b=2}` | diverged (extra search layer) | **RECOVERED** |
| `misc/sf_of_sff` | `sf;` → `12` | diverged (mark preserved) | **STILL DIVERGED — now `3`** |
| `foop/62/sf_sff_nested_combined` | `rsfsff=0` | diverged (mark preserved) | **STILL DIVERGED** |
| `misc/seek_in_nested_result_after_concatenation` | passing | passing | **NEWLY BROKEN** — `OB={NK …}`, `whichˍ1` became `Op+(#(offset=-1, UNANCHORED, NK), 1, NK)` |

The `sf_of_sff` result is the informative one. The new output is:

```
  sf=?(result=3, pattern='^sff$', UNANCHORED);
  ...
  3;
```

against `checked`'s `12`. **Unlimited stripping overshoots.** In
`{a = 1; b = 2; sff = <<a + b>>; sf = <sff>; a = 10; sf; sff;}` the `<<a + b>>` is supposed to
stay deferred until `sf` is used *after* the `a = 10` restatement, so `a` recoordinates to `10`
and the sum is `10 + 2 = 12`. Stripping the SFF the moment the search for `sff` copies the body
lets `a` and `b` resolve immediately at the definition site, where `a` is still `1` — giving
`1 + 2 = 3`. That is not the old value; it is a *third* value, and it is wrong by the test's own
construction (the input deliberately restates `a` to prove the deferral).

So the two SF/SFF divergences are **not** an over-preservation that unlimited stripping undoes.
They sit between two failure modes: the current budget-0 preserves one mark too many, and
unlimited strips at least one mark too many. Neither endpoint is `12`.

And the new breakage in `misc/seek_in_nested_result_after_concatenation` shows unlimited
stripping is not a safe operation to apply broadly — it strips marks that a concatenation was
relying on to hold a `#-1` index unrun, producing NK.

### D.3 What the divergences actually are

Per §B.5, `0eb0bf29` (2026-08-26) introduced them, one day ago, deliberately, to fix a
*different* problem: `{a = {1,2}, b = <<#-2>>, c = a b}` joining too early. That commit's own
notes call the outcome *"Justified mechanism, unjustified outcome — needs review."* These are
**a known, self-inflicted, one-day-old regression on two foreign-FOOP baselines
(FOOP-62 and FOOP-42) with a review note already attached** — not a longstanding drift that a
new language feature would heal.

Under the project's own non-regression invariant (AGENTS.md, `rust_instructions.md` §"Phase-by-
phase testing discipline"), the remedy for a diverged foreign baseline is to **fix the code so
the baseline passes again** — here, to make `handle_found`'s copy stop preserving the mark in
the SF-of-search case while still preserving it in the concatenation-element case that
`0eb0bf29` was fixing. That is a targeted repair of `0eb0bf29`'s over-broad trigger, and it
requires no new syntax, no new FIR kind, and no new marker.

### D.4 Verdict, stated plainly

**The hypothesis does not hold.** UFM as specified — an explicit, opt-in, source-level wrapper —
**cannot** recover these three `checked` values, because none of the three inputs contains one
and an opt-in marker only affects programs that use it. And separately, when the UFM *behavior*
is applied as the default (the strongest form of the hypothesis, measured above), it recovers
only one of the three, leaves the other two at a **different wrong value** (`3` instead of the
expected `12`), and breaks a fourth previously-passing baseline.

Building UFM in the hope of fixing these baselines would spend a Phase's worth of work on four
crates and still leave the regression in place. **The three divergences are a `0eb0bf29` repair
task and should be sequenced separately from, and ahead of, any UFM work.** UFM should be
justified on its own merits — as a language feature Foolishers can use to un-defer a marked
term — not as a fix for these baselines.

---

## E. Open design questions for the human

Ordered roughly by how much they block implementation.

1. **Syntax — `<*` collides with existing, currently-parseable syntax.** `<* x >` parses today
   as `StayFoolish{UnaryOp{"*", x}}` and evaluates to
   `<NK ??? (unknown operator: * (1 operands))>`; only the closing `*>` errors. Adopting
   `<* … *>` therefore requires deciding what happens to the unary-`*` production
   (`parser.rs:747-754`), which appears to be dead syntax that only ever produces NK. Delete it,
   or pick a non-colliding pair? `<@ … @>` is the cleanest ASCII candidate found (`@` is
   postfix-only today, so `<@` is unreachable); a Unicode pair matches the house style and
   collides with nothing.

2. **Does the UFM wrapper survive the clone, or vanish like a stripped mark?** The C.4 sketch
   makes it vanish (return the inner clone). If instead it survives, then a UFM inside a
   cloned tree re-arms unlimited stripping on *every subsequent clone that passes through it* —
   a permanently-unlimited region rather than a one-shot un-defer. These are materially
   different features and the choice determines the mark-arm's whole shape. Note this is the
   same question `94ed10d2`'s "rebuild the wrapper vs strip it" decision faced for SF.

3. **Does UFM nest inside/outside SF/SFF, and what does each nesting mean?** `< <* X *> >` —
   the UFM is *below* an SF, so the SF's own deferral has not yet finished when the UFM is
   reached: does the UFM un-defer the SF that contains it (impossible — it is below it) or only
   marks below itself? `<* <<X>> *>` is the intended shape. `<* <* X *> *>` — idempotent, or an
   error? The current mark arm has no notion of "an outer mark I am inside," so nesting
   semantics must be stated before the arm can be written.

4. **Interaction with `has_ancestral_sfm`, which today has exactly one reader.**
   `fir_kinds.rs:1735` is the only production site reading it (`SearchFir::handle_found`), and
   `step_inner` (`fir_trait.rs:656`) sets it for `FirKind::StayFoolish` only — not for
   `StayFullyFoolish`.

   **ANSWERED (human, 2026-08-27) — the SFF omission is DELIBERATE, not a bug.** The governing
   principle:

   > **SF and UFM marks affect how STEPPING works; SFF achieves detachment from the
   > environment during COMPILATION.**

   SFF does its work in `build_fir`, which passes `under_sff = true` down its whole subtree so
   every descendant search is *born* `ECONSTANIC` ("SFF marker: from here down, searches are
   built ECONSTANIC", `compiler.rs`). A search with no environment to read needs no runtime flag
   to stop it reading one — the detachment is structural before stepping begins. SF passes
   `under_sff` through unchanged ("SF does NOT make descendants econstanic"), so its deferral
   must be expressed at step time, which is exactly what this flag does. Adding
   `StayFullyFoolish` to the condition would double-count a deferral SFF has already achieved.
   This principle is now recorded as a comment at `fir_trait.rs:656`.

   The remaining open part: should a UFM *clear* the flag on the way down
   (`scope.with_ancestral_sfm(false)`)? Since UFM is a stepping-side mark by the principle
   above, that is the natural reading — but it needs deciding explicitly.

5. **`compiler.rs`'s `under_sff` flag.** Searches built under an SFF start ECONSTANIC so they
   never run (`compiler.rs:98-101`). Should a UFM reset `under_sff` to `false` for everything it
   contains? That would be the compile-time analogue of the runtime un-defer, and without it a
   UFM inside an SFF would un-defer at clone time but its searches would still have been *built*
   frozen — the two halves would disagree.

6. **`classify_concat_element` rule 1.** A UFM element almost certainly classifies `AsWritten`
   (C.5), but rule 1's stated rationale ("the Foolisher already deferred this, don't
   double-defer") is the *inverse* of what a UFM does. Confirm `AsWritten` and write the real
   reason into the comment rather than inheriting SF/SFF's.

7. **Rendering.** SF renders `<STATE … >`, SFF `<<STATE … >>`, and both become transparent when
   `should_show_nyes()` is false (`sequencer.rs:462-493`). Does a settled UFM render at all? If
   it is transparent-when-settled like SF, a UFM will be invisible in most baselines; if it
   always renders, every baseline containing one gains two lines. Both are defensible; pick one
   before writing baselines.

8. **`clone_children_for_constanic_clone` starts a FRESH budget rather than inheriting**
   (`fir_kinds.rs:211`), while `clone_children_budgeted` inherits. Its only callers are
   `system_foo`'s `ComparisonFir`/`ModuloFir`/`OrFir` `constanic_clone`s (`fir_kinds.rs:420`,
   `:439`, `:453`). Under UFM this becomes a leak: a UFM's unlimited budget would be **reset to
   `Some(1)`** the moment the path descends through a comparison/modulo/or operand, silently
   truncating the "whole path below" reach that decision 2 mandates. Is that intended, or
   should these inherit? Pre-existing, but UFM makes it observable.

9. **Is there an assignment form?** `compiler.rs:799-803` has `AssignmentOperator::SF`/`SFF`
   (`=<`/`=<<`). Does UFM get `=<*`, or is it wrapper-only?

10. **Naming.** "Unstay Foolish Marker" is a good name for the concept, but it does not appear
    anywhere yet, and `UfmFir` sits oddly beside `StayFoolishFir`/`StayFullyFoolishFir`. Worth
    confirming the FIR-kind spelling (`UfmFir`? `UnstayFoolishFir`?) before it propagates
    to ~60 sites.

---

## F. Implementing the mark AS an operator (human, 2026-08-27)

The **Unstay Foolishness Mark**, written `<@ … @>`, keeps its name and its surface syntax.
Only the **implementation** is an operator: it owns a FIR in its `foolish_children`, waits for
it to go constanic, constanic-clones it while stripping **every** SF/SFF mark into
`ubc_children`, and lets it step again. In the human's words, "this seems like the most linear
description."

So: a mark to the Foolisher, an operator to the evaluator. Sections C and E above assumed the
other shape — a deferral wrapper parallel to `StayFoolishFir` — and are superseded wherever
they turn on that assumption.

### F.1 Why the operator shape resolves what the wrapper shape could not

§E5 raised a tension the mark framing could not settle: SFF achieves detachment at COMPILE
time (`build_fir` passes `under_sff = true` down, so descendant searches are *born*
`ECONSTANIC`), so a step-time mark cannot reach back and undo it. The human's requirement is
that it *should*: `{a=x}` removes one layer of SFF detachment, but `{a=<@ x @>}` must remove
**all** layers.

The operator shape achieves this without touching compilation, because of a fact already in
the code:

```rust
// foolish-core/src/fir.rs — transform_for_clone
Nyes::Econstanic | Nyes::Woconstanic | Nyes::Prembrionic
| Nyes::Embryonic | Nyes::Braning => Nyes::Embryonic,
```

A stripped clone taken with `disable_nyes_reset = false` has its `ECONSTANIC` **reset to
`EMBRYONIC`**. That reset *is* the removal of the compile-time effect: SFF's detachment lives
entirely in "this search was born ECONSTANIC", and re-birthing it EMBRYONIC un-does it. UFM
therefore needs no compile-time half at all — it is purely a stepping-time operator, which
keeps the governing principle intact:

> SF and UFM marks affect how STEPPING works; SFF achieves detachment from the environment
> during COMPILATION.

### F.2 The lifecycle, in the codebase's existing two-phase operator idiom

This maps directly onto the FOOP-55 §11 event-driven shape that `BraneConcatOpFir` already
uses — `fir_op_step` is pure orchestration; the two phases live in the two handlers.

| phase | handler | what UFM does |
|---|---|---|
| 1 | `fir_op_step` PREMBRIONIC/EMBRYONIC arm | push `foolish_children[0]` as a task; → BRANING |
| 2 | `on_foolish_op_ready` | not constanic yet → report a waiting NYES. Constanic → `constanic_clone` with **`StripBudget::unlimited()`**, push into `ubc_children`, report `None` |
| 3 | *(automatic)* | `push_ubc_child` auto-enqueues it, because the stripped clone is EMBRYONIC ⇒ not constanic. It steps again, now with no marks to defer it |
| 4 | `on_ubc_op_ready` | gate on `are_ubc_children_ready_for_op()`; once drained, settle from the result |

Step 3 is free — no explicit task push is needed:

```rust
// proto_brane.rs — push_ubc_child
self.ubc_children.borrow_mut().push(Rc::clone(&child));
if !child.borrow().core().get_nyes().is_constanic() {
    self.tasks.borrow_mut().push_back(child);   // auto-enqueue
}
```

### F.3 What this changes about the blast radius (§C)

**Smaller than §C estimated.** No `has_ancestral_sfm` interaction (§E4 moot), no `under_sff`
compile-time question (§E5 moot), no "does the wrapper survive the clone" question (§E2 moot —
an operator is consumed by producing its result, like every other operator). No new NYES state,
no step-loop change. It is a new FIR kind built to an idiom the codebase already has three
working examples of.

Still required: lexer + token + AST + parse arm; `FirKind::Ufm` + `UfmFir`; the `foolish-core`
`Fir` variant and its satellite sites; a sequencer arm; a `classify_concat_element` decision
(a UFM is an operator, so §9.2 rule 5 would currently make it an NK concatenation element —
that likely needs its own rule).

### F.4 The one real blocker: `clone_children_for_constanic_clone` resets the budget

```rust
// fir_kinds.rs:210-212
// Children of one clone share that clone's budget (FOOP-55 §5).
let budget = StripBudget::fresh();
```

`ComparisonFir`, `ModuloFir` and `OrFir` clone their children through this entry, which
**discards an inherited budget and installs `Some(1)`**. Under UFM that silently truncates
`unlimited()` back to one strip the moment a path descends through any of those three
operands — defeating "remove ALL layers" exactly where nested marks are most common
(`system.foo`'s operators are built from `<<#-2>>`/`<<#-1>>` operands).

This must be fixed for UFM to mean what it says. Two options: thread the budget through this
entry (add a parameter), or delete it in favour of the inheriting sibling
`clone_children_budgeted`. The second is tidier and folds into the Phase 4B "contract/combine
the constanic_clone family" TODO already recorded in FOOP-55.plan.md.

Note the exhausted-budget arm is *not* a blocker: with `None`, `spend()` returns `(true, None)`
forever, so the `Rc::clone(fir_ref)` share-path at `fir_kinds.rs:~340` is simply never taken.

### F.5 Open questions the operator shape does NOT resolve

1. **Syntax.** `<@ … @>` is the human's choice, replacing `<* … *>` (measured: `<* 5 >` parses
   *today* as `StayFoolish{UnaryOp{"*"}}` and evaluates to NK). `<@`/`@>` still needs a lexer
   check for collisions.
2. **What does UFM settle to?** Presumably the drained result's NYES, as `BraneConcatOpFir`
   does — but a UFM whose content settles NK, or ECONSTANIC-after-unfreezing, needs a stated
   answer.
3. **UFM inside a still-marked region.** `< <@ x @> >`: the outer SF defers, the inner UFM
   un-defers what it owns. Since UFM now behaves mechanically like an operator rather than
   like a competing deferral wrapper, this is
   probably unremarkable — the SF defers *the UFM*, and once the UFM runs it strips everything
   below it. Worth confirming against a written example.

---

## Last Updated

**Date**: 2026-08-26
**Updated By**: Claude Code / claude-opus-5
**Changes**: New file. Scoping study for the proposed UFM (Unstay Foolish Marker): §A reads the
current `StripBudget`/`constanic_clone` machinery; §B locates the old unlimited-stripping
default (`94ed10d2`, 2026-06-19) and the commit that ended it (`779b63f5`, 2026-08-11); §C
enumerates the blast radius across four crates and rates it MEDIUM, including a **measured
lexer/parser collision** on `<* … *>`; §D tests and **rejects** the hypothesis that UFM
recovers the three diverged einmo baselines, on two independent grounds, one of them measured;
§E lists ten open design questions.
