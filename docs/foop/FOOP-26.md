---
foop: D62
title: Carrying FOOP-55's semantics onto foolish-ubca2 — marks, concatenation-as-operator, and the three-beat step
author: Claude Code / claude-opus-5 (directed by Atlas)
status: Draft
phase: phase-4
supersedes: []
begun: [ ]
---

# FOOP-26: Carrying FOOP-55's semantics onto `foolish-ubca2`

FOOP numbering is little-endian; the full rules live in `foop.md` at the repository root —
**read it before creating or editing a FOOP.** The `foop:` front-matter field here is the
big-endian sort key preceded by `D` (`foop: D62`, file `FOOP-26.md`, following FOOP-16's 61).

## Abstract

`foolish-ubca2` is green today — 134/134, all three einmo gates passing. **This FOOP
deliberately changes what programs mean on it**, because three things in the current language
are wrong in ways that make macros impossible to write. Project Euler 1 is the program that
exposes them; it fails because a macro like `'cmod` settles **NK at its own definition site**,
and NK is terminal.

The design was discovered by FOOP-55 across 113 commits against `foolish-ubca` and never
landed. That branch is unmergeable (§Motivation), so this FOOP states the *result* — the
finished language — rather than the repairs that led to it.

**Three changes, and a fourth that holds them together:**

### Change 1 — SF/SFF mark handling becomes one explicit status

Every constanic copy runs under exactly one **mark status**, which decides one thing: whether
each copied node's NYES is **reset** so the copy re-evaluates in its new brane, or **preserved
verbatim** so it does not. `normal` resets; `under-sfm` preserves; `under-ufm` resets even when
enclosed by an SF that would otherwise have preserved. They are mutually exclusive, `under-ufm`
is absorbing, and the status is passed down a copy recursively so one status governs a whole
subtree. This replaces a two-valued `bool` that could not name the third case.

**The FOOP-55 branch's strip *budget* — a per-path counter that let `<< <<X>> >>` defer one
coordination — is deliberately NOT brought over**, because a budget is hard to reason about:
its depth is not derivable by reading the source. The preferred replacement, a **brane-depth
mark remover**, is left to its own FOOP. Consequence: nested marks do not defer, and no
existing baseline moves. → **§2**

### Change 2 — A new mark and operator, UFM `<<<…>>>`, clears all marks

The Unstay Foolishness Mark is a mark to the Foolisher and an **operator** to the evaluator: it
copies its single content FIR with **every** SF/SFF mark on every path below stripped, then
lets it step again. Where `<<X>>` removes one layer of deferral, `<<< X >>>` removes all of
them. It is consumed producing its result, like any operator. → **§3**

### Change 3 — Concatenation becomes an operator, with defined ergonomics

`BraneConcatOp` is a process, exactly as `Op+` is: `Op+(a,b)` is not an integer, and a
concatenation is not a brane — its settled *value* is. It answers no brane-like question about
itself, says "don't know" rather than zero while unsettled, and waits for an element whose
search chain is merely ECONSTANIC instead of proceeding past it. This is what makes
concatenation work when its dependencies are searches that resolve to branes. Alongside it, the
**ergonomics**: the compiler marks each written constituent by its syntactic form, so the
Foolisher does not hand-mark them. → **§4**

### The frame — every FIR steps in three beats

**Step `foolish_children` to constanic → constanic-copy under a mark status → step
`ubc_children`.** Beat 2 is the only place the mark status matters: it decides whether the
copied nodes' NYES is reset so they re-evaluate here, or preserved so they do not. Beats 1 and
3 are two phased queues because "done
providing input" and "done doing my own further work" are different questions —
`BraneConcatOp` genuinely has both. → **§1**

One supporting rule runs underneath all of it: a query requires **its own** context to be
constanic, and that constancy is established by ordinary stepping — never assumed from a NYES
computed in another context, never approximated by a partial answer. → **§5**

### A second purpose: this FOOP is the arena model's first real test

**After this FOOP the two evaluator crates deliberately disagree.** `foolish-ubca` is frozen
and does not receive these semantics, so for every behavior changed here it stops being a valid
cross-check — it remains the oracle only for the pre-FOOP-26 language, and new einmo cases go
into `foolish-ubca2`'s suite alone (§Test Plan 3).

That divergence is not merely a cost to absorb. **It is, in part, the point.** FOOP-16 argued
the arena model on construction-boilerplate grounds — 68 `Rc::new_cyclic` sites collapsing to
one `create_child` call each — but a boilerplate argument is not the real question. The real
question is whether the arena makes the language **easier to develop, and easier to debug**.

This FOOP is the experiment that answers it, because the comparison is controlled: the FOOP-55
branch spent **113 commits** attempting exactly this work against `foolish-ubca` and did not
land it (§Motivation). The same problems — mark handling, concatenation-as-operator, the
recoordination defects — are now posed to `foolish-ubca2`. Whatever it costs here is directly
comparable, and the things the arena should make easier are precisely the things that went
wrong there:

- **Copy-and-recoordinate is one recursive function** (`revive_constanic`) over `Copy` indices,
  not a recursive re-wiring of `Rc`/`Weak` pairs. §2's mark state threads down one call chain.
- **"Which kinds behave differently" is a closed-enum question the compiler answers**, not an
  open set of trait overrides found by grepping.
- **There is no interior mutability to reason about** — one `&mut FVMStorage` borrow is the
  whole exclusivity story, so a defect like the branch's D9 (an ambient decision reused across
  unrelated clones) has fewer places to hide.

So the honest framing is: this FOOP buys the language three semantic changes **and** a real
answer about the arena. If it turns out to be no easier here than it was there, that is a
finding worth having, and it should be recorded plainly rather than explained away.

### What this FOOP is not

Not a port — nothing from the FOOP-55 branch is merged or cherry-picked, and `foolish-ubca`
stays frozen and untouched. Not FOOP-55 resumed: **Euler 1 is measured, not gated**
(§"Items needing a human decision"). Success is that the semantics above are right and the 179-case einmo suite is green
under them.

**Dependencies:** FOOP-16 (complete). **Explicit non-dependencies:** FOOP-55's event-based
child-readiness refactor (§8 — separate document, may not be needed) and UBCb/UBCc/UBCd.

# Part I — Motivation

## What actually happened on the FOOP-55 branch

Measured on `worktree-foop-55-event-handlers` @ `a9e4d60b`, 2026-09-01 — not inferred from
the plan's checkboxes:

| Signal | State |
|---|---|
| Build | clean |
| Unit tests | **387 pass, 0 fail** |
| `einmo_gate_checked` | **FAILS** |
| Pre-existing baselines regressed | **4** — `foop/42` hfs, `foop/62/sf_sff_nested_combined`, `misc/concat_sf_f_more`, `misc/sf_of_sff` |
| Plan checkboxes | ~102 of ~300 |
| `exercises/fibonacci/1.foo` | `NK(ITERATION-EXCEEDED)`, most of the tree still PREMBRIONIC |
| `foop/55/euler_small.foo` | `NK(ITERATION-EXCEEDED)` |
| Final commit message | `buggy` |

Those four regressed baselines are a violation of the project's own non-regression hard rule
(`AGENTS.md`: "A FOOP under development **must not change the OUTPUT** of any einmo test
belonging to a different, already-shipped FOOP"). The branch cannot be merged as it stands,
and repairing it in place means finishing ~200 checkboxes against a crate that has since been
superseded.

In `euler_small.foo` the
macro `'cmod` settles **NK at its own definition site**:

```
'cmod={NK
    INTERNALˍnumerator=#(offset=-3, UNANCHORED, NK);
    INTERNALˍdivisor=#(offset=-3, UNANCHORED, NK);
    INTERNALˍeq=#(offset=-3, UNANCHORED, NK);
    ...
}
```

Its `#-3` operands search for neighbours that only a *caller* supplies, find nothing where
they are written, and settle NK — which is terminal. Every call site then inherits a dead
macro, and no amount of stepping recovers it. Premature stripping does not merely resolve
early — it resolves against the **wrong neighbours**, and an early miss settles NK, which is
terminal.

**This FOOP does not fix that, and says so up front.** The FOOP-55 branch's answer was a strip
budget; §2 explains why that design is not carried over and what replaces it later (a
brane-depth mark remover, its own FOOP). Euler 1 therefore remains blocked after this FOOP on
the mark question specifically. What this FOOP delivers is the rest — an explicit mark **state**
in place of an unnameable `bool`, concatenation as an operator, UFM, and the ergonomics — which
is the ground the depth-based remover has to stand on. The exercise is measured, not gated
(§"Items needing a human decision"), and this is the concrete reason to expect it still not to run.

## The defects are live in `foolish-ubca2` — measured, not assumed

Three of FOOP-55's findings were re-probed directly against `foolish-ubca2`'s own evaluator on
2026-09-01 (temporary test, since removed; `foolish-cli` is wired to `foolish-ubca` only, so
the oracle comparison was run separately).

**1. Premature mark-stripping — `foolish-ubca` and `foolish-ubca2` agree byte-for-byte.** For
`{blah = 7; A = <{abcd = 1; deep = << <<#-2>> >>}>; B = A; C = B;}` both crates produce the
identical rendering, in which `deep` settles

```
B=?(result={NK  abcd=1;  deep=#(offset=-2, UNANCHORED, NK) }, pattern='^A$', ..., NK)
```

— **NK at `B`**, where `#-2` finds nothing, and therefore already dead by the time the value
reaches `C`, where the search would have succeeded. This is §2's canonical damage case,
reproduced on the target crate. The byte-identity is itself the good news: ubca2 is a faithful
UBCa, so the semantics stated here apply to both, and ubca remains a valid oracle.

**2. D10 is live, and its ubca2 symptom is worse than ubca's.** For
`{f = {c = {1,2} <<#-2>>; n = c$;}; tbl={x=8;y=9;}; r =$ {tbl,0}f}`:

| Crate | `c` | `n = c$` |
|---|---|---|
| `foolish-ubca` | flattens to 2 | `2` — silently wrong, no alarm |
| `foolish-ubca2` | flattens **correctly** to 4 (`1;2;x=8;y=9`) | **NK** |

So ubca2 already builds the right concatenation and then **fails to read the tail out of it**.
That is a different failure than ubca's frozen `2`, and it means §4's fix must be verified
against ubca2's actual behavior rather than transplanted from the branch's diagnosis.

**3. D9's shape is present** — in `{a = {1,2}, b=<<#-2>>, c= a b}` the concatenation's second
element remains a search whose result is an ECONSTANIC index, so `c` does not reach the target
`{1;2;1;2}`.

These measurements are why this FOOP exists as a refactor of `foolish-ubca2` rather than an
archive of FOOP-55: the problems are not historical.

## Why re-specify instead of port

`foolish-ubca2` is not a refactor of `foolish-ubca`; it is a different program with the same
meaning. The translation table is total:

| `foolish-ubca` | `foolish-ubca2` |
|---|---|
| `FirRef = Rc<RefCell<dyn Fir>>` | `FirPointer` (`Copy`) + `&FVMStorage` / `&mut FVMStorage` |
| `Weak<RefCell<dyn Fir>>` parent | `FirPointer` on `Slot` |
| 14 structs implementing `trait Fir` | one `FirSpec` enum, 14 variants, exhaustive `match` |
| `impl Fir for XFir { fn fir_op_step }` | a new arm in the `fir_op_step` match |
| a new `as_*` trait method | an inherent method on `FirCursor` |
| `Rc::new_cyclic(...)` | `create_child` / `make_orphan_child` / `make_root` |
| `ProtoBrane::constanic_clone_at` | `FVMStorage::revive_constanic` |
| `Result<(), UbcError>` | `()` + `panic!` / `unreachable!` |
| `Scope` (pub, Clone) | `ArenaScope` (private, `Copy`) |
| `fir_kinds.rs` / `fir_trait.rs` / `proto_brane.rs` / `compiler.rs` | all inside `fvm_storage.rs` |

Every one of FOOP-55's ~4,700 changed lines in `fir_kinds.rs` alone would have to be rewritten
by hand against a file that does not exist. There is no merge base to build on. Re-specifying
from the `.md` is strictly cheaper and produces a better result, because it lets us drop what
the branch itself retracted (its D8, its §5.6 handler, its `ExtremumFir`) instead of porting it and
then deleting it.

**And `foolish-ubca2` on `jia` is green** — 134/134 tests, all three einmo gates passing,
179 cases in each of `input/`/`output/`/`checked/`/`verified/`. That is a clean base to change
deliberately, which the FOOP-55 branch has not been for some time.

# Part II — Specification

What `foolish-ubca2` shall do. Sections §1-§5 state the finished behavior of each construct;
the *Today, and how to get there* subsection at the end of each names what differs now.


These sections describe `foolish-ubca2` **as it will behave when this FOOP is done**. They are
written to be read as the language's rules; where today's behavior differs, that is confined to
a *Today* subsection at the end of each, so the rule can be read without the diff.

Reading order follows the Abstract: §1 is the frame, §2 is change 1 (marks), §3 is change 3
(UFM, which the mark rule creates the need for), §4 is change 2 (the concatenation operator
and its ergonomics), and §5 is the supporting rule.

## §1. How a FIR steps — the three beats

Every FIR with children steps in three beats:

| Beat | What happens | Store |
|---|---|---|
| **1 — ready the input** | Step every child to constanic. The kind's syntactic operands or statements must be usable before it can act. | `foolish_children` |
| **2 — copy under a mark status** | Constanic-copy what is needed into the new context. The status decides whether copied nodes re-evaluate here (§2). | — |
| **3 — do my own work** | Step the children this kind created for itself. | `ubc_children` |

**NYES vocabulary used throughout** (`AGENTS.md` §Foolish Terminology). *Constanic* — any
terminal state, so "step to constanic" means "step until no more stepping is appropriate".
*Constantew* — CONSTANT, INDEPENDENT or NK: will not change no matter what. *Conclusive* —
CONSTANT or INDEPENDENT: reached a value. The last two differ **exactly on NK**, which is
constantew (nothing will change it) yet inconclusive (it never produced a value). This FOOP
gates on constantew where a settled *failure* must be caught, and on conclusive where a *value*
is required; the two are not interchangeable and each use says which it means.

Beat 2's copy is made under a **mark status** — `normal`, `under-sfm` or `under-ufm` — carried
by the stepping path rather than chosen by the copying kind. §2 states what each status does.

**Beat 3 is where kinds differ most.** For `Operator`'s `+` there is nothing to do in beat 3
but receive an already-computed result. For `BraneConcatOp` (§4) and UFM (§3), the operator's
own work — building and stepping `ConcatHelper`s, re-stepping a demarked copy — *happens* in
beat 3. That is why the two stores must stay distinguishable: "done providing input" and "done
doing my own further work" are different questions, and a single flat "are my children done?"
gate cannot express both.

The arena makes the phasing structural rather than conventional:

- `parent` and `foolish_children: Vec<FirPointer>` live on **`Slot`** (`fvm_storage.rs:70,72`)
  — the arena owns topology.
- `ubc_children: Vec<FirPointer>` lives on the **payload, `ProtoBrane`** (`fvm_storage.rs:89`).

**No new machinery.** `foolish-ubca2` already steps this way — no kind calls `push_ubc_child`
before its `foolish_children`-driven decision has run. §1 only gives the discipline a name so
§§2–5 have one place to attach. Where a kind needs a policy other than the default, it is
expressed as a **free function matched on `FirSpec`** or written into that kind's `fir_op_step`
arm; a closed enum makes "which kinds differ" a question the compiler answers. Whether the
beats should instead be driven by an event/handler protocol is **out of scope** — see
Part IV, §"Out of scope".

## §2. (Change 1 — marks) The mark status — `normal` / `under-sfm` / `under-ufm`

`foolish-ubca2` shall carry a three-valued **mark status** on the stepping scope, replacing
`ArenaScope::has_ancestral_sfm: bool` (`fvm_storage.rs:745`). The three values are mutually
exclusive:

| Status | Effect on a constanic copy made under it |
|---|---|
| `normal` | markers met are dropped; each copied node's NYES is reset — anything **not constantew** becomes `Embryonic`, so the copy re-evaluates in its new brane |
| `under-sfm` | markers met are dropped; each copied node's NYES is **preserved verbatim**, so the copy does not re-evaluate |
| `under-ufm` | markers met are dropped; NYES is reset as under `normal` — **and this holds even when the copy is made beneath an enclosing SF**, where `under-sfm` would otherwise have preserved |

**All three drop markers.** A constanic copy never reproduces an SF, SFF or UFM node: on
meeting one it descends into the content and copies that instead (`revive_constanic:1977-1992`).
This applies at every depth, not only at the copied root — measured 2026-09-02 against
`foolish-ubca2`: for both `{b=2; t=<1 + <<b>>>; r=t; r;}` and `{b=2; t=<<1 + <<b>>>>; r=t; r;}`
the clone at `r` resolves the inner `<<b>>` and settles `r=3`, so the nested marker did not
survive into the copy.

What distinguishes the statuses is therefore **not** whether markers are removed — that is
unconditional — but what NYES the copied nodes are given.

**The status is a property of the path a FIR is being stepped under**, not a parameter of a
copy. `step_inner` sets it while descending (`:769-772`). When a search under that path resolves
and clones what it found, `revive_constanic` receives the status the stepping FIR is carrying.

**Inside the copy, the status is passed down recursively, unchanged**, to every child
(`:2031`) and every `ubc_child` (`:2045`). One status governs the whole copied subtree.

**Where the status is actually consumed.** It has exactly one use: it is the argument to
`Nyes::transform_for_clone` (`foolish-core/src/fir.rs:186`), called once per copied node at
`fvm_storage.rs:2024`. That function is the whole mechanism:

```rust
pub fn transform_for_clone(self, descendent_of_sfm_and_foolishly_ignorant: bool) -> Nyes {
    if descendent_of_sfm_and_foolishly_ignorant {
        return self;                      // under-sfm: preserve verbatim
    }
    match self {
        Nyes::Constant | Nyes::Independent | Nyes::Nk => self,   // constantew: no re-evaluation
        _ => Nyes::Embryonic,                                    // re-evaluate in the new brane
    }
}
```

So "deferral" is not the copy refusing to descend, and not a mark being retained. It is the
copy **declining to reset NYES**. A search cloned under `under-sfm` keeps whatever state it had
— an `Econstanic` search stays `Econstanic` and does not run again; under `normal` the same
search would be re-born `Embryonic` and would run in the new brane.

#### The copy-side flag is two-valued, not three

The **stepping scope** carries three statuses. The flag `revive_constanic` receives needs only
**two** values, because `normal` and `under-ufm` ask the copy for the same thing:

| Copy-side flag | Behavior |
|---|---|
| `UnderSfm` | drop markers; preserve each copied node's NYES verbatim |
| `UnderUfm` | drop markers; reset NYES normally |

`normal` maps to the copy-side `UnderUfm` — the "reset normally" behavior — which is why the
existing parameter is a `bool` and why widening it to a three-valued type at the copy boundary
would add a distinction the copy cannot act on.

**Where the three-valued stepping status earns its keep** is the mapping, not the copy: a copy
made while the scope reads `normal` or `under-ufm` passes the reset-normally flag, and one made
while the scope reads `under-sfm` passes the preserve flag. Without the third stepping value
there is no way to say "reset normally **even though** an SF encloses this" — an SF's
`under-sfm` would win, and a UFM inside an SF would not clear anything. That case is the whole
reason the stepping status is three-valued and the copy-side flag is two-valued.

Both types are defined once, in **Part III**. This section states only the rule they encode.

Transitions: `normal` and `under-sfm` may each flip to `under-ufm`. `under-ufm` is absorbing.
There is no other transition — `normal` versus `under-sfm` is decided by the stepping path
before any copy begins.

Which construct sets which status is stated per mark in §2.1, §2.2 and §3.1.

In Rust this is an enum `MarkState { Normal, UnderSfm, UnderUfm }`; this document writes the
values as `normal` / `under-sfm` / `under-ufm` in prose. It replaces the `bool` parameter named
`descendent_of_sfm_and_foolishly_ignorant` / `sfm` that carries this today.

#### §2.1 SF — the Stay Foolish mark, `<…>`

Syntax: a run of one `<`, its content, a run of one `>`.

SF acts at stepping time. Its content is compiled as ordinary source: descendant searches are
born `Prembrionic`, and `build_fir`'s `Astn::StayFoolish` arm (`fvm_storage.rs:4452`) passes
`under_sff` through unchanged.

SF steps its content the ordinary way — enqueue it, step it to constanic, adopt its
result (`fir_op_step`'s `FirSpec::StayFoolish` arm, `:907-940`).

While stepping an SF's children, they are passed the `under-sfm` status
(`step_inner:769-772`).

A constanic copy that meets an SF node unwraps it: the mark is not copied. The copy descends
into the SF's settled `ubc_children[0]` if it has one, otherwise into its written body
(`revive_constanic:1977-1992`). This is the same under every status — the status does not decide
whether to unwrap, only what NYES the copied nodes get (§2).

The consequence of a search resolving while the scope reads `under-sfm`: `revive_constanic`
receives that status and therefore preserves each copied node's NYES verbatim rather than
resetting it to `Embryonic`. The copied subtree does not re-evaluate in its new brane. The flag
is passed down the copy recursively, so this holds for the whole subtree, not just its root.

An SF resolves against the neighbours of the position it is written at.

#### §2.2 SFF — the Stay Fully Foolish mark, `<<…>>`

Syntax: a run of two `<`, its content, a run of two `>`.

SFF acts at compile time. Every search built beneath an SFF is born `Nyes::Econstanic` instead
of `Nyes::Prembrionic`. `build_fir` carries an `under_sff: bool`; the `Astn::StayFullyFoolish`
arm (`:4457`) passes `true` for its content, and `search_nyes` (`:4249`) turns that into the
birth NYES. `check_sff_marked_child` (`:1811`) then walks the constructed subtree and asserts no
descendant search escaped the rule.

A search born `Econstanic` has already settled "found nothing here", so it does not resolve in
the brane the SFF is written in.

While stepping an SFF's children, they are passed the same status the SFF itself is being
stepped under. SFF does not set `under-sfm`. Its deferral is achieved at compile time, and
setting a stepping status as well would defer a second time.

When an SFF's content is recoordinated into a new brane, the copy's `transform_for_clone`
(`foolish-core/src/fir.rs:186`) maps `Econstanic → Embryonic`. That mapping ends the
detachment: the search is re-born ready to run, in the new brane.

A constanic copy that meets an SFF node unwraps it: the mark is not copied, and the copy
descends into the written body (`foolish_children[0]`). Unlike SF, an SFF has no settled result
to prefer, because its content has not run.

An SFF resolves against the neighbours of the position it is recoordinated to.

#### §2.3 Four changes that would break SFF

The FOOP-55 branch's SFF work failed because the implementation altered SFF's behavior while
intending only to change how it was carried. These four are the specific ways that happens.

1. **Setting `under-sfm` when stepping into an SFF's children**, for symmetry with SF. `under-sfm`
   makes a copy preserve NYES verbatim (§2). An SFF's descendant searches are already
   `Econstanic` from compile time, so preserving that state means the copy at the *use* site
   never resets them to `Embryonic` either — the searches never run at all, in any brane. SFF's
   deferral is undone by the reset; keeping the state instead makes it permanent.
2. **Changing `transform_for_clone`'s `Econstanic → Embryonic` mapping.** That mapping is what
   ends SFF's detachment. It is also in `foolish-core`, shared with the frozen `foolish-ubca`,
   so a change there changes both crates.
3. **Weakening or skipping `check_sff_marked_child`.** It is the only assertion that
   `under_sff` reached every descendant. A search that escapes it is born `Prembrionic`,
   resolves in the defining brane, and on a miss settles NK, which is terminal.
4. **Making an SFF copy prefer a settled result, as SF does.** SFF content is unresolved by
   construction. Where a settled result exists, it is evidence of change 3.

A single-mark SFF program must produce byte-identical einmo OUTPUT before and after any change
in this area.

#### Not carried over: the mark budget

The FOOP-55 branch's answer to nested marks was a **strip budget** — a counter, `Copy` and
passed by value, decremented once per root-to-leaf path, so that `<<X>>` resolved as today
while `<< <<X>> >>` sat out one coordination and resolved at the next. Deferral depth was
written by the Foolisher as mark depth.

**That design is deliberately not brought over** (human, 2026-09-02). It works, and it is not
worth its cost in comprehensibility: **a budget is difficult to reason about.** Reading a
program, a Foolisher cannot see how many strips a term will meet, because the count depends on
how many constanic copies the evaluation performs — not on anything written in the source. §5's
own record states the required depth "is not derivable by reading the source — a single
source-level step can perform more than one constanic copy", and `fibonacci/1.foo`'s comment
records that its `2` was measured, not reasoned. A rule whose correct usage must be found
experimentally is not a rule a language should ship.

**What replaces it, later: a brane-depth mark remover.** The preferred future design counts
something a developer *can* see — **brane depth**, the nesting level at which marks come off —
rather than an invisible tally of copy operations. That is a separate FOOP; it is named here so
the intent is on record and so nobody re-derives the budget by default when the need next
arises.

*Requirements that FOOP inherits.* Whatever it is, it has to answer four program shapes. The
first is the one the target exercises need; the other three were never established even for the
budget, and are recorded here so its successor is not designed against the easy case alone.

1. **A table row that is not selected.** The working shape: a recursive term sits in a row a
   value search may or may not select, and the unselected row is never coordinated a second
   time, so its mark never comes off and the recursion does not fire. This is what
   `fibonacci/1.foo` and `euler_small` rely on, and it is the acceptance bar.
2. **Recursion in the SELECTED branch, deeper than one level.** The selected branch *is*
   coordinated, once per level. Does the deferral a term needs stay constant, or grow with
   recursion depth? If it grows, no fixed spelling in the source can be right — which is
   precisely the objection to the budget, and the successor must not inherit it.
3. **Two branes that reach each other** — `A`'s body coordinates `B`, `B`'s coordinates `A`,
   neither reaching itself directly. Each hop is a separate copy, so a per-copy rule sees a
   fresh start each time. No program in either suite does this today; there is no evidence
   either way.
4. **The recursive term is also the accumulated value** — the shape `euler_small`'s
   `sum35 = sum35 + (…)` has. The term must be *read* (so the mark must come off) and *carried
   forward* (so it must stay on), possibly within one coordination. Whether one rule can serve
   both roles at once is untested.

**The consequence, stated plainly.** Without a budget and without the depth-based remover,
**nested marks do not defer.** `<< <<X>> >>` behaves as `<<X>>` does — the copy's recursion
meets the outer mark, strips it, recurses into the content, meets the inner mark, and strips
that too, all in one pass. This is unchanged from today's behavior, so **no existing baseline
moves** and this section becomes a *naming and structuring* change rather than a semantic one.

It also means the specific damage the budget was designed to prevent **remains present**:

```foolish
{
	blah = 7;
	A = <{abcd = 1; deep = << <<#-2>> >>}>;
	B = A;
	C = B;
}
```

`deep` resolves at `B`, where `#-2` finds nothing, settles **NK** — terminal — and is dead by
the time the value reaches `C`, where the search would have succeeded. The same shape kills
`euler_small`'s `'cmod` at its definition site (§Motivation). **This FOOP does not fix that**,
and the exercise programs are correspondingly not expected to run on the strength of §2 alone.
That is the accepted trade: a comprehensible mark model now, and the depth-based remover as its
own piece of work, rather than shipping a counter nobody can reason about.

#### Today, and how to get there

The status exists today as `ArenaScope::has_ancestral_sfm: bool` (`fvm_storage.rs:745`),
propagated in `step_inner` (`:769-772`), read at six copy call sites, and consumed at exactly one
place — `transform_for_clone(sfm)` (`:2024`). The change widens that one field from two values to
three:

1. **Add the two enums and their mapping**, exactly as defined in Part III.
2. **Change `ArenaScope`'s field** from `has_ancestral_sfm: bool` to `mark_state: MarkState`, and
   `revive_constanic`'s parameter (`:1962`) from `sfm: bool` to `CloneMarks`. The six sites
   reading `scope.has_ancestral_sfm` (`:1163`, `:1181`, `:1235`, `:1291`, `:1349`, and
   `clone_stmt_result` `:2535`) apply the mapping. The flag continues to pass down all four of
   `revive_constanic`'s recursive calls (`:1981`, `:1990`, `:2031`, `:2045`) unchanged — that
   recursion is what makes one flag govern a whole copied subtree.
3. **Set the status in `step_inner`, beside the line that sets it today.** At `:769-772` the child
   scope gains `UnderSfm` when `ptr` is a `StayFoolish`. Add the UFM case in the same place: when
   descending into a UFM's `ubc_children`, the child scope becomes `UnderUfm`. Because `UnderUfm`
   is absorbing, the SF arm must not overwrite it — guard so a `StayFoolish` met below a UFM
   leaves the status alone. These two lines are the entire transition system.
4. **Feed `transform_for_clone` from the copy-side flag.** It takes
   `descendent_of_sfm_and_foolishly_ignorant: bool` and preserves NYES when true
   (`foolish-core/src/fir.rs:186`); pass `matches!(flag, CloneMarks::UnderSfm)` at the call site
   (`:2024`). **Do not change `foolish-core`** — it is shared with the frozen `foolish-ubca`
   (§2.3 hazard 2), and `CloneMarks` is two-valued precisely so the existing `bool` still
   suffices.

The SF/SFF unwrap block (`:1977-1992`) is **unchanged**: it drops markers under every status,
at every depth. It gains one arm for the UFM kind (§3), so a UFM node met inside a copy is
dropped the same way. The status decides NYES, not whether markers are removed.

**§3's UFM operator does not itself set the status.** It pushes its constanic copy to
`ubc_children`, and the ordinary stepping descent in step 3 does the rest. That is what makes UFM
an operator like any other rather than a special case in the copy path.

**Acceptance bar: no einmo baseline moves.** `Normal` and `UnderSfm` reproduce exactly what
`sfm: false` and `sfm: true` do today, and `UnderUfm` is unreachable until §3 lands. A
divergence here is a regression, not a discovery.

**One constraint worth knowing before starting:** `Nyes::transform_for_clone`
(`foolish-core/src/fir.rs:186`) lives in `foolish-core`, shared with the frozen `foolish-ubca`,
so changing its behavior changes both crates. §2 does not require changing it — step 4 maps the
three-valued status onto its existing `bool` at the call site instead. If an implementation finds
itself wanting to change that function's behavior, stop and raise it: that is §2.3 hazard 2.

## §3. (Change 2 — UFM) UFM `<<<…>>>`, a new mark and operator

#### §3.1 UFM — the Unstay Foolishness Mark, `<<<…>>>`

`foolish-ubca2` shall implement UFM.

Syntax: a run of three `<`, its content, a run of three `>`. The run length selects the mark:
1 is SF, 2 is SFF, 3 is UFM. An opener run of 4 or more is rejected at the lexer. A closer run
of 4 or more is legal and splits by nesting depth.

UFM is an operator. It holds its constituent FIR in `foolish_children`. It steps that
constituent to constanic, then constanic-copies it into `ubc_children`, then steps the copy.

While stepping its `foolish_children`, they are **not** passed `under-ufm` — they carry
whatever status the UFM itself is being stepped under, `normal` or `under-sfm`. The constituent
therefore settles under its own enclosure, unchanged.

**The copy into `ubc_children` is made under `under-ufm`.** Two things follow, and both are
what "removes all markers" means:

- **Every marker in the copied subtree is dropped** — SF, SFF and UFM alike, at every depth.
  A constanic copy already does this under any status (§2), so UFM inherits it rather than
  adding it.
- **Every copied node's NYES is reset**: an `Econstanic` search — including one born
  `Econstanic` by an enclosing SFF's compile-time rule (§2.2) — is re-born `Embryonic`. The
  reset is what ends the deferral, and it reaches the whole subtree because the flag is passed
  down the copy recursively (§2).

The reset is the same one `normal` performs. What `under-ufm` adds is that it performs it **even
when the UFM itself is enclosed by an SF**, where `under-sfm` would otherwise have preserved
NYES verbatim. A UFM inside an SF therefore still clears; without the third stepping value it
could not.

**While stepping its `ubc_children`, they are passed `under-ufm`.** Any further copy made below
resets NYES too, so nothing under a UFM re-enters deferral.

`under-ufm` is absorbing. An SF met below a UFM does not restore `under-sfm` — this is the one
place the two statuses could conflict, and UFM wins.

UFM acts at stepping time. Its content is compiled as ordinary source, and `under_sff` passes
through unchanged: a UFM does not clear an enclosing SFF's compile-time detachment during
construction. It ends that detachment at step time, by the NYES reset above.

A UFM does not survive its own copy. It is consumed producing its result, as any operator is.

The three marks side by side:

| | SF `<…>` | SFF `<<…>>` | UFM `<<<…>>>` |
|---|---|---|---|
| Acts at | stepping | compile | stepping |
| Mechanism | steps content, adopts result | descendant searches born `Econstanic` | copies content to `ubc_children`, steps it |
| Status passed to `foolish_children` | `under-sfm` | inherited unchanged | inherited unchanged |
| Status passed to `ubc_children` | inherited | inherited | `under-ufm` |
| Effect on a copy made below it | NYES preserved verbatim | — (no copy is initiated; the search never runs) | NYES reset to `Embryonic` |
| Effect on deferral | adds | adds | removes |
| Survives its own copy | no — unwrapped | no — unwrapped | no — consumed |

#### §3.2 The angle-run rule

A run of `<` or `>` is terminated by any character that is not `<` or `>`. The run length
selects the mark: 1 is SF, 2 is SFF, 3 is UFM.

Openers are strict: a run of 4 or more `<` is rejected at the lexer. The lexer rejects it
rather than the parser resolving it, because a recursive-descent parser could disambiguate
`<<<<` but a reader cannot (human, 2026-08-27: *"`<<<<` 4 angle should be illegal, there's no
way to determine what is meant by it conveniently"*).

Closers are greedy: a run of 4 or more `>` is legal and splits by nesting depth. `<<a+<<b>>>>`
parses as `SFF(a + SFF(b))` with its 4 closers split 2+2; `<<< <<b>>>>>` splits its 5 as 2+3.
A fixed maximal split does not work — only nesting depth decides. Runs of exactly 2 or 3 keep
their own token, so `<<x>>` and `<<<x>>>` lex unchanged.

The asymmetry is deliberate (human correction, 2026-08-27: *"we insist the starting `<<` cannot
be confusing `<<<<<` precisely so we can greedily consume terminal `>>`, that extra space should
not be needed"*). Because every opener is unambiguous, the recursive-descent stack always knows
how many `>` the current frame needs, so each frame takes its 1, 2 or 3 and leaves the
remainder to its parent.

Implementation, proven on the FOOP-55 branch (`e18c31dd`): the lexer emits the first `>` of an
over-long run as a plain `Gt` and lets the parser pull the rest; the parser gains
`expect_closer(want)`, which takes `want` closers from the cursor, splitting a longer token in
place, and errors if the run is too short.

#### Today, and how to get there

UFM does not exist. It is the largest piece of new construction in this FOOP and spans three
crates:

1. **`foolish-parser` (SHARED with the frozen `foolish-ubca` — read the warning below).**
   - Lexer: `Token::LtLtLt` / `Token::GtGtGt`, both arms ordered **before** their 2-char forms
     (`lexer.rs:424` is where `LtLt` is produced today) so a 3-run is not mis-lexed as 2+1.
     Reject an opener run of 4+. For closers, emit the first `>` of an over-long run as a plain
     `Gt` and let the parser pull the rest.
   - Parser: `expect_closer(want)`, replacing the direct `expect(&Token::Gt…)` calls, taking
     `want` closers and splitting a longer token in place.
   - AST: a new `Astn::UnstayFoolish { expr }` beside `StayFoolish` (`ast.rs:113`) and
     `StayFullyFoolish` (`:118`).
2. **`foolish-core`:** a builder for the new kind, beside `StayFoolishFirBuilder` /
   `StayFullyFoolishFirBuilder`. **Needed** — see the rendering rule below.
3. **`foolish-ubca2`:** a `FirSpec::UnstayFoolish` variant; a `fir_op_step` arm implementing the
   three beats (its `ubc_children` step under `under-ufm`); `validate_astn` +
   `build_fir` arms; a
   `classify_concat_element` arm (§4.3's Marker row); and a `core_fir_conversion` arm.

**Rendering** (human, 2026-09-02). A pre-constanic UFM renders as `<<< … >>>` around its
content. A constanic UFM does not render: it has been consumed producing its result, so the
adapter unwraps to that result. This is the rule every operator already follows — `.value()`
returns the node while pre-constanic and its result once settled — so no special case is needed
beyond the arm itself. The `foolish-core` builder is required rather than optional because
einmo renders mid-evaluation trees, so an unsettled UFM reaches the renderer.

**`foolish-ubca` and the shared parser.** `foolish-parser` is a dependency of both evaluator
crates, so the new token and AST variant reach `foolish-ubca` whether or not it implements
them. Left alone it still compiles — its `build_fir` ends in
`_ => unreachable!("validate_astn should have rejected this")` (`compiler.rs:437`) — but a UFM
program would panic there.

`foolish-ubca` shall compile a UFM node into an NK FIR and discard its children (human,
2026-09-02). In its `build_fir`, add an `Astn::UnstayFoolish` arm building

```rust
Rc::new(RefCell::new(NkFir {
    core: ProtoBrane::new(vec![], child_parent!(), Nyes::Nk),
    reason: "Syntax Not Implemented".to_string(),
}))
```

— the same shape `Astn::UnknownLit` already uses at `compiler.rs:231`, and the same
empty-`vec![]` children discard `ConcatElemKind::Error` uses at `:113`. This is deliberately
**not** a compile error: an unanswerable question yields NK, exactly as the rest of Foolish
does, rather than refusing to run the program. `foolish-ubca` then renders such a program
honestly — `??? (Syntax Not Implemented)` — instead of panicking or pretending to evaluate it.

This is a change to the frozen crate, and it is authorized in this one narrow form: an arm that
only ever produces NK, adds no evaluation behavior, and cannot alter any existing baseline (no
current input contains `<<<`).

**Why not `<@ … @>`:** the opener `<@` is free, but the closer `@>` is **not** — `<a@>`
already parses today as SF-of-`@`-projection and evaluates to `0`. A greedy `@>` closer would
silently change the meaning of existing programs, and whitespace does not disambiguate (both
spacings already parse). The angle-run spelling also keeps all three marks in one visual family.

**No baseline is disturbed — and this is exactly why the asymmetry exists.** Swept
2026-09-01: the only program in either corpus with a 4-run is
`einmo_suite/input/misc/sff_nested.foo` (present in both crates), `{a=1,b=2; c=<<a+<<b>>>>; c;
c;}`. Its run is on the **closer** side, so greedy splitting parses it unchanged and **no
respelling is required**. Both crates' `verified/` twins stay valid.

This is recorded because an earlier reading of the rule applied the illegal-run restriction to
both sides, which would have forced an edit to a frozen, verified-twinned baseline. That
reading was wrong: the strict-opener rule is what *buys* the greedy closer. Any future change
to mark syntax must preserve the asymmetry for the same reason.

## §4. (Change 3 — concatenation) `BraneConcatOp` and its ergonomics

`foolish-ubca2` shall treat a concatenation as an operator that produces a brane.

> **Scope note.** This section specifies the operator's **contract** (§4.1, §4.5) and its
> **ergonomics** (§4.3) — what a concatenation *is*, what it answers about itself, and how the
> compiler marks its constituents. The **implementation is rewritten separately**, by
> **FOOP-46**, against a phased search behavior this FOOP does not specify: during Gathering an
> IB search demanded by a constituent finds nothing in the concatenation and falls through to
> its parent; once Joined, IB searches resolve normally within `ubc_children`. FOOP-46 also
> carries the open question of whether `ubc_children` should be a hidden brane. What this FOOP
> lands is the contract that FOOP-46 then implements underneath.

#### §4.1 What a concatenation is

`BraneConcatOp` is an operator, as `Op+` is. `Op+(a, b)` is not an integer; the integer is its
settled value. A concatenation is not a brane; the brane is its settled value.

Three things are distinct and must not be conflated:

| | |
|---|---|
| a **Foolish Brane** | a `{...}` literal written in the program |
| a **Search Result Brane** | a name or search form that resolves to a brane |
| **`BraneConcatOp`** | an operator that produces a brane |

`ConcatProvenance` and `ConcatHelper` keep their names: provenance records how the operator was
spelled, and `ConcatHelper` is the brane the operator produces.

#### §4.2 Parsing

A written concatenation is a juxtaposition of two or more constituents, or the backtick
tail-concatenator form (FOOP-65). The parser produces `Astn::Concatenation` or
`Astn::TailConcatenation`; the tail form's constituents are reversed relative to source order.

A single `{…}` is a brane literal, not a concatenation.

#### §4.3 Compiling into FIR — the ergonomics

**The problem the ergonomics solve.** A concatenation joins the *statements* of its
constituent branes. To do that it must reach each constituent brane's content — but a constituent brane must not
resolve against the neighbours it is written beside, because after the join its statements live
somewhere else, beside the other constituents' statements. Written by hand, every constituent
would need a mark saying so. The ergonomics supply that mark from the constituent's syntactic
form, so the Foolisher writes the join and nothing else:

```foolish
{
	a = {x=1;} {y=2;}    !! written plainly
	b = <<{x=1;}>> <<{y=2;}>>    !! what it would take by hand
}
```

Both give `a` and `b` the two statements `x=1; y=2`. The first is what a Foolisher writes; the
compiler produces the second.

**The classification.** Each constituent is classified by its syntactic form, and the
classification decides the mark:

| Constituent | Compiled as | Why |
|---|---|---|
| Marker (`<…>`, `<<…>>`, `<<<…>>>`) | no added mark | the written mark stands; the Foolisher has already said what they want |
| Brane | SFF detachment | its statements must not resolve against where the brane is written |
| Search — any form | wrapped in an SF node | the search must resolve to a brane, but not step past that |
| Concatenation | the ambient status, as a brane-like constituent | it is itself an operator producing a brane; its own constituents are already marked |
| Constantew (integer, `???`, `⬤`) | no mark | it will not change and has no statements to contribute; rejected when the operator flattens |
| Operator | rejected at classify time | an NK node naming the kind |

**The nested-concatenation row, expanded.** A concatenation constituent is written with
parentheses — without them the juxtaposition is flat, since a bare run of constituents is one
concatenation, not nested ones:

```foolish
{ a = {a=1}{b=2}({c=3}{d=4}){e=5}; a;}
!! a = {a=1; b=2; c=3; d=4; e=5}
```

`a`'s third constituent is itself a concatenation. Compiling `a` therefore **descends into
building a second `BraneConcatOp`**, and that inner operator classifies and marks *its own*
constituents by the same rules — `{c=3}` and `{d=4}` each get SFF detachment from the inner
operator, not from `a`. Nothing about the outer operator's classification reaches inside.

Stepping follows the same nesting. `a` steps its constituents to constanic (§4.4 beat 1); the
inner concatenation is one of them, and becomes constanic by doing its own three beats — joining
`c=3; d=4` into its own helper. Only then is it brane-like, so `a` can flatten it in turn. The
result is fully flat: nesting affects construction and stepping order, not the shape of the
answer.

A constituent of the inner concatenation may itself be a search, and it is SF-wrapped by the
inner operator exactly as it would be by the outer one:

```foolish
{ huh={z=9}; a = {a=1}{b=2}({c=3}huh{d=4}){e=5}; a;}
!! a = {a=1; b=2; c=3; z=9; d=4; e=5}
```

`huh` resolves to a brane, contributes its `z=9` to the inner join, and that join's statements
reach the outer one. Both of these evaluate as shown on `foolish-ubca2` today (measured
2026-09-02).

> **How `huh` reaches its binding — and why FOOP-46 owns the rule.** `huh` is a bare name
> demanded by a constituent while the inner concatenation is still gathering. It finds nothing
> in the inner concatenation, forwards to the outer one — which is also still gathering, because
> the inner one is one of *its* constituents and is not yet constanic — finds nothing there
> either, and resolves at the outermost brane. The fall-through **chains**, and it can only chain
> this way: a concatenation cannot be finished while a constituent is still pre-constanic.
> FOOP-46 §2-§3 specify that rule and own it. The remaining question it carries is what should
> happen when a name is visible in **both** a sibling constituent and the parent, where the rule
> says the parent wins and the sibling is never consulted.

Substituting a non-brane for `huh` shows the constantew defect propagating through the nesting —
the whole expression collapses rather than reporting the offending constituent:

```foolish
{ huh=12345; a = {a=1}{b=2}({c=3}huh{d=4}){e=5}; a;}
!! shall be:   a = ??? (cannot concatenate number)
!! today:      a = {}      -- all five branes lost, no alarm
```

**Worked through.** Each of these evaluates as shown on `foolish-ubca2` today (measured
2026-09-02) except the last, which is the defect §4.6 fixes.

*A brane constituent is SFF-detached, so it contributes statements rather than resolving where
it sits:*

```foolish
{a = {x=1;} {y=2;}; a;}          !! a = {x=1; y=2}
```

*A search constituent is SF-wrapped. It resolves to a brane, and that brane's statements are
contributed — in either position:*

```foolish
{t = {y=2;}; a = t {z=3;};  a;}  !! a = {y=2; z=3}
{t = {y=2;}; a = {x=1;} t;  a;}  !! a = {x=1; y=2}
```

*Statements from different constituents become neighbours, so a search into the result crosses
between them:*

```foolish
{t = {y=2;}; a = {x=1;} t; b = a?y; b;}   !! b = 2
```

*A nested concatenation is brane-like and flattens completely:*

```foolish
{a = ({x=1;} {y=2;}) {z=3;}; a;}          !! a = {x=1; y=2; z=3}
```

*An explicit mark is left alone — it means the same as the mark the compiler would have added:*

```foolish
{a = {x=1;} <<{y=2;}>>; a;}               !! a = {x=1; y=2}
```

*A constantew constituent has no statements to contribute, so the concatenation is NK:*

```foolish
{a = {x=1;} `99; a;}
!! shall be:   a = ??? (cannot concatenate number)
!! today:      a = {}      -- silently empty, no alarm
```

**Three things the rule deliberately does not say.**

- **"Search — any form" is flat.** Uncontexted unanchored, uncontexted anchored, and contexted
  chains are treated alike: if the top FIR of a constituent is a search, it is SF-wrapped. The
  variety of search forms does not multiply the rule.
- **Marking and acceptance are separate.** The classifier decides only the mark. Whether a
  constituent may be concatenated at all is decided later, when the operator flattens — which
  is why the constantew row is marked "no mark" rather than "rejected". An operator is rejected
  at classify time because it can never be brane-like; a constantew is passed through and
  rejected at flatten time. Both end in NK naming the kind.
- **It says nothing about mark depth.** How many coordinations a term survives is §2's
  question. "No added mark" means no marker node is inserted, not that the constituent is
  compiled in a vacuum — the ambient `under_sff` still threads through as normal.

#### §4.4 Stepping — `foolish_children`, then `ubc_children`

A concatenation's constituents are its `foolish_children`. The `ConcatHelper` it builds is its
`ubc_children`. It steps in §1's three beats.

**Beat 1 — the constituents.** On `Prembrionic`/`Embryonic`, every constituent is enqueued and
the node goes `Braning` (`fvm_storage.rs:1004-1015`). Each constituent is stepped to constanic
before the operator acts.

A constituent that is a search whose result chain terminates `Econstanic` is **not** ready. The
operator keeps waiting rather than proceeding past it. Readiness walks the result chain
(`ubc_children[0]`, per FOOP-23's two-child invariant), answering "not ready" at the first
`Econstanic` hop and "ready" when the chain ends **conclusive** — a hop that reached a value.
Note this is the conclusive cut, not constantew: an NK hop is constantew but never produced a
value, and it is caught by the type-error check below rather than by waiting.

**Between the beats — the type check.** Once the constituents have drained, each is resolved
through `.value()` and asked whether it is brane-like:

- any constituent that is **constantew** but not brane-like → the whole concatenation settles
  NK, naming the offending indexes. The gate is constantew rather than **conclusive** because
  the two differ exactly on NK, and an NK constituent is a settled type error: it will never
  become a brane, so there is nothing to wait for. A conclusive-only gate would let it through
  to the wait branch and hang;
- any constituent not yet brane-like and not a type error → the concatenation settles
  `Woconstanic` and waits;
- all constituents brane-like → proceed to beat 2.

**Beat 2 — build the helper.** `populate_concat_helpers` (`:2720`) creates one `ConcatHelper`
as an orphan child, constanic-copies every constituent's statements into it in order with
renumbered line numbers, and pushes it to `ubc_children`. The helper is built first so its
pointer is the parent of every copied line, which is what lets cross-constituent searches
resolve. This runs exactly once, gated by `helpers_populated`.

**Beat 3 — step the helper.** The helper is pushed as a task and drained. The concatenation
then settles from the drained helper, not from the raw constituents.

#### §4.5 What a concatenation answers about itself

A concatenation answers no brane-like question about itself. `stmt_count`, `stmt_at` and
`is_brane_like` are questions for its settled value; a caller unwraps through `.value()` to the
`ConcatHelper`, which answers all three.

While its helper is unpopulated, a concatenation's statement count is unknown, not zero. An
unknown count makes an enclosing search's index lookup answer "not found", which the existing
IB-then-AB fallback handles. No concatenation-specific search handler exists or is needed.

#### §4.6 Today, and how to get there

**Naming.** `FirSpec::Concatenation` becomes `FirSpec::BraneConcatOp`.

**§4.3, the classifier.** `arena_compiler::classify_concat_element` (`fvm_storage.rs:4148`)
returns `ConcatElemKind` (`:4072`) with variants `BareBrane`, `BareConcat`, `BareSearch`,
`SfSearch`, `SfBrane`, `Error`. `build_concat_element` (`:4206`) applies them. Three deltas:

- `Astn::StayFullyFoolish` wrapping a brane currently maps to `SfBrane` (`:4187-4198`), the
  same variant as `Astn::StayFoolish` wrapping a brane. A written mark must stand as written,
  so SF and SFF must stop collapsing to one kind here.
- There is no constantew arm; an integer, `???` or `⬤` currently falls to `Error`, and the
  resulting NK node contributes no statements, so the concatenation silently flattens to an
  **empty brane** rather than reporting anything. Measured 2026-09-02: `{a = {x=1;} `99; a;}`
  gives `a={}`, losing `x=1` with no alarm. A constantew must pass through unmarked and be
  rejected at flatten time with a reason naming the kind.
- UFM needs an arm under the Marker row (§3).

Note `BareBrane` achieves its detachment by passing `under_sff = true` into `build_fir`
(`:4213`) rather than by inserting an SFF node, while `SfBrane` inserts one (`:4222`). Both
produce SFF detachment; only the second leaves a marker node in the tree.

This classifier is a deliberate byte-for-byte duplicate of `foolish-ubca`'s private one. After
this FOOP the two intentionally differ — do not restore parity as a cleanup.

**§4.5, the brane-like answers.** `FirCursor::stmt_count` (`:1653`), `stmt_at` (`:1675`) and
`is_brane_like` (`:1706`, defined as `stmt_count().is_some()`) currently answer for a
`Concatenation` by summing and walking its own `ubc_children`. Remove those answers.

`stmt_count().unwrap_or(0)` appears at nine sites (`:1214`, `:1271`, `:1663`, `:1684`, `:2394`,
`:2727`, `:2743`, `:2891`, `:3952`), each turning "unknown" into "zero". Triage all nine — some
legitimately want a default, others are the defect. The two inside `stmt_count`/`stmt_at`
themselves (`:1663`, `:1684`) are the core.

**A nested concatenation is where this bites hardest.** An inner concatenation that is not yet
joined reports zero statements, so an outer one reads it as an empty-but-valid brane and joins
anyway — silently dropping everything the inner one was going to contribute. FOOP-46 §3.1
measures the case and treats it as its own failing test; fixing the count here may resolve it,
which is worth checking before FOOP-46 changes anything.

**§4.4, constituent readiness.** The `Braning` arm (`:1016-1050`) currently treats any constanic
constituent as ready. Add the `Econstanic`-chain walk described above.

**Acceptance targets**, both measured failing against `foolish-ubca2` (§Motivation):

- `{a = {1,2}, b=<<#-2>>, c= a b}` → `c={1;2;1;2}` (§4.4 readiness).
- `{f = {c = {1,2} <<#-2>>; n = c$;}; tbl={x=8;y=9;}; r =$ {tbl,0}f}` → `n` reads `9`, the tail
  of the correctly-flattened four-statement `c`. `foolish-ubca2` already flattens `c` correctly
  and fails at the read, so this target exercises §4.5, not §4.4.

**Blast radius**, measured on the FOOP-55 branch. ~21 unit tests call brane-like methods on a
concatenation without unwrapping first. A block of ~9 (`concat_equals_big_brane`,
`concat_ib_search_crosses_segments`, and siblings) is built on "a concatenation behaves exactly
like an equivalent big brane" — true of the result, not of the operator. Each needs reading
individually. The analogous `settled_result()` correction on the branch produced two
test-behavior changes and zero einmo OUTPUT regressions.

**Two open items**, both parser-shaped, neither blocking:

- `{1} 2+3` splits into a concatenation plus a separate statement `5`, because
  concatenation-continuation does not start on a bare integer. A silent split turns a malformed
  program into a different valid one.
- `{x=1;} 99` splits and never reaches the classifier, while ``99`{x=1;}`` reaches it and NKs.
  FOOP-65's Equivalence Law says the two spellings must agree; they do not.

## §5. (Supporting rule) A query requires its own context to be constanic

**The rule.**

> A content or search query requires its **search context** to be constanic, and that constancy
> is produced by **ordinary stepping of dependencies** — never assumed by reading a NYES that
> was computed in a different context, and never approximated by a partial answer.

Clause by clause:

- **In-brane context** (`?name`, `~name`, `#`, `^`, `$` over my own brane) is constanic by
  **FIFO drain order** — "everything before me" is an entitlement, not something to check. No
  new mechanism.
- **An anchored search's anchor** is a `foolish_children` dependency and must be **stepped** to
  constanic before its resolved brane is scanned. A stepping obligation, not a fact to assume.
- **A concatenation's own content** must drive its elements — including a nested
  concatenation's own flattening — to constanic before anything is locked in.
- **A recoordinated copy is a NEW dependency in a NEW context.** Its pre-copy NYES — especially
  ECONSTANIC, which means "no value *here*, may gain one via recoordination" — is evidence
  about the **old** context and is not evidence about the new one. It must be re-enqueued and
  re-stepped like any other not-yet-constanic child.

**Why this rule is cheap to keep.** Because §2's mark already carries the deferral, nothing
downstream ever needs to peek at a container's partial content to decide whether it is "ready
enough" — the mark already decided. A survey of every brane-like call site found **no caller
that needs or benefits from a partial answer**. So this section adds **no new NYES state and no
new readiness type**: `is_constanic()` is the only gate a content query ever needs, and the fix
at each violating site is to call it on the right thing at the right time before trusting or
memoizing an answer.

An earlier draft proposed a graduated readiness ladder for a future breadth-first evaluator's
benefit. It was solving a problem UBCa does not have, and is rejected here. If UBCc is ever
built, it gets its own baselines (§7) and can revisit this on its own terms.

# Part III — Design

The types and mechanisms the specification resolves to, defined once.

## The two enums and the mapping

The specification sections above state each construct's behavior. This section is the single
place the resulting **types** are defined, so an implementation has one thing to build against
and the sections above do not each carry their own version.

### Definitions

Two distinct types, because they answer two different questions.

```rust
/// The stepping scope's mark status. A property of the path a FIR is being
/// stepped under. Set by `step_inner` while descending; read at every site
/// that initiates a constanic copy.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum MarkState {
    /// Ordinary stepping.
    Normal,
    /// Stepping below a StayFoolish node.
    UnderSfm,
    /// Stepping a UFM's `ubc_children`. Absorbing: once set, descending
    /// through a StayFoolish does NOT restore `UnderSfm`.
    UnderUfm,
}

/// What a constanic copy does with the NYES of the nodes it copies.
/// Markers are dropped under BOTH variants, at every depth — that is
/// unconditional, and is not what this type selects.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum CloneMarks {
    /// Preserve each copied node's NYES verbatim. The copy does not
    /// re-evaluate in its new brane.
    UnderSfm,
    /// Reset each copied node's NYES normally: constantew states (CONSTANT,
    /// INDEPENDENT, NK) keep theirs, every other state becomes `Embryonic`,
    /// so the copy re-evaluates here.
    UnderUfm,
}

impl MarkState {
    /// The only place the two types meet. `Normal` and `UnderUfm` ask the
    /// copy for the same thing; only `UnderSfm` differs.
    pub(crate) fn for_clone(self) -> CloneMarks {
        match self {
            MarkState::UnderSfm => CloneMarks::UnderSfm,
            MarkState::Normal | MarkState::UnderUfm => CloneMarks::UnderUfm,
        }
    }
}
```

**Why two types and not one.** A copy cannot act on a distinction between `Normal` and
`UnderUfm` — both reset NYES. Collapsing them at the copy boundary makes that explicit in the
type, so nobody later has to ask what `Normal` means to a copy and invent an answer. The
three-valued status still earns its keep on the stepping side: without `UnderUfm` there is no
way to express "reset normally **even though** an SF encloses this", and a UFM inside an SF
would not clear.

### Where each is set and read

| | `MarkState` | `CloneMarks` |
|---|---|---|
| Lives on | `ArenaScope` (`fvm_storage.rs:745`), replacing `has_ancestral_sfm: bool` | `revive_constanic`'s parameter (`:1962`), replacing `sfm: bool` |
| Set by | `step_inner` while descending (`:769-772`) | `MarkState::for_clone()` at each copy call site |
| Set to `UnderSfm` when | descending into a `StayFoolish`'s children | mapped from `MarkState::UnderSfm` |
| Set to `UnderUfm` when | descending into a UFM's `ubc_children` | mapped from `MarkState::Normal` or `UnderUfm` |
| Propagates by | scope passed to `step_inner`'s recursion | passed down `revive_constanic`'s four recursive calls (`:1981`, `:1990`, `:2031`, `:2045`) unchanged |
| Consumed at | the six copy call sites (`:1163`, `:1181`, `:1235`, `:1291`, `:1349`, `:2535`) | exactly one place — `transform_for_clone(matches!(flag, CloneMarks::UnderSfm))` at `:2024` |

### What is NOT changed

- **`Nyes::transform_for_clone`** (`foolish-core/src/fir.rs:186`) keeps its `bool` parameter and
  its behavior. `CloneMarks` is two-valued precisely so this stays true, and `foolish-core` is
  shared with the frozen `foolish-ubca`.
- **The marker-drop block** (`revive_constanic:1977-1992`) keeps dropping SF and SFF under every
  flag, at every depth. It gains one arm for the UFM kind. Marker removal is unconditional; the
  flag selects NYES treatment only.
- **The three-beat step.** No kind's stepping order changes.

### What this settles

- Deferral is a copy **declining to reset NYES**, not a marker being retained and not a copy
  refusing to descend.
- Marker removal is unconditional in a constanic copy, so "UFM removes all markers" describes
  what UFM *inherits*, not a behavior unique to it. What is unique to UFM is forcing the NYES
  reset through an enclosing SF.
- There is no budget and no counter anywhere in the design (§2, "Not carried over").

## FIR Impact

Grouped by crate, because this FOOP is not confined to one.

**`foolish-ubca2`**
- `FirSpec::Concatenation` → `FirSpec::BraneConcatOp` (§4), losing its brane-like answers.
- A new `FirSpec::UnstayFoolish` variant (§3), with `fir_op_step`, `validate_astn`, `build_fir`,
  `classify_concat_element` and `core_fir_conversion` arms.
- Two new types, `MarkState` and `CloneMarks`, with the mapping between them — defined in
  Part III. Neither exists today.
- `ArenaScope::has_ancestral_sfm: bool` becomes `mark_state: MarkState`, and `step_inner`'s
  scope propagation gains the UFM case beside the existing SF one.
- `revive_constanic`'s `sfm: bool` parameter becomes `CloneMarks`. Its marker-drop block is
  unchanged except for one new arm recognizing the UFM kind.
- **No new NYES state.** §5 is explicit that `is_constanic()` is the only gate needed.

**`foolish-parser` (shared)**
- `Token::LtLtLt`/`GtGtGt`; strict 4+ opener rejection; greedy closer splitting;
  `expect_closer(want)`; `Astn::UnstayFoolish` (§3).

**`foolish-core` (shared)**
- A builder for the UFM kind (required — a pre-constanic UFM reaches the renderer, §3).
  **`Nyes::transform_for_clone` is deliberately NOT changed** (§2).

**`foolish-ubca` (frozen — one authorized exception)**
- A single `build_fir` arm compiling `Astn::UnstayFoolish` to `NkFir { reason: "Syntax Not
  Implemented" }` with children discarded (§3). It produces only NK, adds no evaluation
  behavior, and cannot change any existing baseline — no current input contains `<<<`.

## UBC Step Impact

§2 changes **only the type naming what a copy already decides** — `UnderSfm` keeps a mark
exactly where `sfm: true` keeps it today, `Normal` strips exactly where `sfm: false` strips.
No step order changes and no FIR kind's state machine changes. §3 adds one operator that steps
like any other. §4 removes answers the operator should never have given. §5 adds calls to an
existing predicate at sites that skipped it.

**Step counts:** unchanged by §2 (nothing moves — see Test Plan 2) and by §3 for existing
programs (none contain `<<<`). §4 *will* move step counts for concatenation cases with deferred
constituents — that is its point, and each moved baseline goes through the Promotion Review
Gate.

# Part IV — Plan of Implementation

Scope, ordering, verification, and the decisions still outstanding.

## Implementation order

The contents, so the choice can be made on evidence.



| § | Change | Crates touched | Files | Moves baselines? | Depends on |
|---|---|---|---|---|---|
| **§2** | mark status enum | ubca2 only | `fvm_storage.rs` (`ArenaScope`, `step_inner`, `revive_constanic`) | **no** — reproduces today's behavior exactly | — |
| **§3** | UFM | **parser + core + ubca2 + ubca** | `lexer.rs`, `parser.rs`, `ast.rs`, `fir.rs` builder, `fvm_storage.rs` (5 arms), `compiler.rs` (1 NK arm) | only new cases; nothing existing uses `<<<` | §2 (it sets `under-ufm`) |
| **§4** | `BraneConcatOp` + ergonomics | ubca2 only | `fvm_storage.rs` (3 accessors, 9 `unwrap_or(0)` sites, `Braning` arm, `classify_concat_element`, rename) | yes — concatenation cases | §3, for §4.3's Marker row only |

**Observations.** §2 moves no baseline at all, so it can land first with a purely mechanical
acceptance test. §3 has the widest blast radius (four crates) but the lowest risk to existing
behavior, since no current program contains `<<<`; it needs §2's status enum to have a value
to set. §4 carries the only real baseline risk — it is the one that changes what existing
concatenation programs evaluate to — and it needs §3 only for §4.3's Marker row.

**Suggested: §2 → §3 → §4**, each landing green before the next. The order is
least-risk-first: a change that moves nothing, then a change that only adds, then the one
change that moves existing baselines.

**The separate question is FOOP-36** (the Foolish-rendering sequencer), which argues in its
own §"Why this should land before FOOP-26" that it should go first. Its case is strong and
the two do overlap in files (`fvm_storage.rs`'s `core_fir_conversion`, and
`ubca_snapshot_tester.rs`): it rewrites all 179 ubca2 baselines **once, for rendering only,
with semantics fixed**, so that FOOP-26's baseline diffs afterwards show *Foolish source
changing* rather than one `?(pattern=…, ECONSTANIC)` becoming another. Since this FOOP's
Promotion Review Gate requires justifying every changed OUTPUT line, and §4 moves
baselines, doing FOOP-36 first makes that review materially cheaper — and it costs FOOP-26
nothing, because FOOP-36 changes no FIR, no step rule, and no step count. **Recommendation:
FOOP-36 first, then §2 → §3 → §4.** Human confirms.

## Test Plan

1. **Regression floor first.** `foolish-ubca2` is green today (134/134, three gates). Every
   change is measured against that, one at a time. Any pre-existing baseline that diverges is a
   regression **introduced by this FOOP**, never a stale baseline to promote over.
2. **§2 moves NO baseline at all.** `Normal`/`UnderSfm` reproduce exactly what `sfm: false`/
   `sfm: true` do today and `UnderUfm` is unreachable until §3, so the entire 179-case suite
   must stay byte-identical, step counts included. This makes §2 the safest possible first
   step — a pure type-and-naming change with a mechanical acceptance test.
3. **New cases live under `foop/26/` in `foolish-ubca2`'s suite ONLY.** Do **not** add them to
   `foolish-ubca`'s suite: an input with no matching `checked/` baseline fails that crate's gate
   as an *extraneous input* (`einmo_suite.rs`'s suite-shape check), and a UFM program evaluates
   there to `??? (Syntax Not Implemented)` (§3), which is honest but not a useful cross-check.
   Cross-implementation validation therefore stops being available for anything this FOOP
   changes — an accepted cost: after this FOOP the two crates deliberately disagree, and
   `foolish-ubca` is an oracle only for the pre-FOOP-26 language. (`input/` trees currently
   differ by one case, `foop/16/comprehensive`; after this FOOP they diverge further, by
   design.)
   **Read `foolish-ubca2/einmo_suite/einmo.toml` before writing any case** — that suite
   separates sections on `①` (U+2460) + LF, not `foolish-ubca`'s `!!` + LF, and content
   containing the separator is a hard write-time error.
4. **The three mark statuses (§2) get one unit test each, asserting the copied NYES** — under
   `Normal` a non-constantew node comes back `Embryonic`; under `UnderSfm` it comes back with its
   source NYES unchanged (an `Econstanic` search stays `Econstanic`); under `UnderUfm` it comes
   back `Embryonic` **even when the copy is made beneath an enclosing SF**, which is the case
   that distinguishes `UnderUfm` from `Normal`. Plus one asserting the status reaches a
   grandchild, not just the copied root — that is what the recursive pass-down buys. Plus one
   asserting `UnderUfm` is absorbing: an SF met below a UFM must not restore `UnderSfm`.
5. **§4 gets one case per constituent kind** in §4.3's table — marker (each of SF/SFF/UFM),
   brane, search, nested concatenation, constantew, operator — asserting the compiled mark and
   the flatten outcome separately, since marking and acceptance are different questions.
6. **`*_nyes_transitions` for every kind touched** (§9), per `AGENTS.md`.
7. **Euler and fib are measured, not gated** (§"Items needing a human decision"). Record
   where they get to, in the plan,
   each time the semantics change.

## Items needing a human decision (blocking)

1. ~~`misc/sff_nested.foo` must be respelled.~~ **RESOLVED — not a blocker.** §3's angle-run
   rule is asymmetric (strict openers, greedy closers), so `<<a+<<b>>>>` parses unchanged and
   no frozen baseline is touched. Retained as a record of a resolved false alarm.
2. **`foolish-ubca2`'s `einmo_gate_verified` doc comment is stale** — it says `verified/` "is
   still empty here", but the directory holds 179 signed files and the test passes. The comment
   should be corrected; flagging rather than editing, since it documents a human decision
   (commit `5c65a84c`).
3. **Whether Euler 1 is a gate for this FOOP or a measurement.** This FOOP proposes
   **measurement**: land the semantics, keep the suite green, then report where `fibonacci/1`
   and `euler_small` actually get to. Making the exercise a gate is what turned FOOP-55 into a
   113-commit branch that regressed four shipped baselines.

## Open Questions

1. **When does the brane-depth mark remover get written, and by whom?** §2 declines the strip
   budget and names its replacement without specifying it. Until that FOOP exists, nested marks
   do not defer and the exercise programs stay blocked on this specific point. §2 records the
   four program shapes its successor must answer. This is the single largest piece of
   still-unwritten design in this area.
2. **What exactly freezes a concatenation's shape?** "Indexable" must mean "the shape can no
   longer change", not "the shape is currently readable". For a concatenation whose operands
   are themselves searches returning branes, the answer is *not yet decidable* until those
   searches settle — and the condition must be computable without walking the whole tree every
   step.
3. **Does a contexted search need §4's treatment?** A contexted search navigates from a carried
   *position* rather than doing a fresh IB/AB walk. No case has exercised it against a
   not-yet-populated concatenation; if one does, trace it before assuming the fallback's
   reasoning transfers.
4. **Should `MAX_DEPTH` be loud?** ubca2 returns silently at depth 100. A silent no-progress at
   the ceiling is indistinguishable from a settled program to anything watching from outside.
5. **Is the FOOP-65 Equivalence Law divergence (§4.6) in scope here or its own FOOP?**
6. ~~What should `foolish-ubca` do with a UFM program?~~ **ANSWERED (human, 2026-09-02):**
   compile it to an `NkFir` with reason `"Syntax Not Implemented"` and discard its children —
   not a compile error. **§3.**
7. ~~Does the rendering adapter need a UFM arm?~~ **ANSWERED (human, 2026-09-02):** yes when the
   UFM is not constant, no once it is. A settled UFM unwraps to its result like any operator; a
   pre-constanic one renders as `<<< … >>>`. The `foolish-core` builder is therefore required.
   **§3.**

## What is deliberately NOT carried forward

Recording these prevents their rediscovery.

| Item | Disposition |
|---|---|
| **`ExtremumFir` (§7)** | **Built, worked, deleted.** Superseded by position-projection. Not retired for being wrong — do not build against it. Its durable residue is the observation that *every computing postfix operator should be declared `'name = {NameFir}`* — the brane wrapper is what makes an operator concatenable, and it turns misuse into a type error rather than a runtime check. |
| **D8 (SFF self-reference spins BRANING)** | **RETRACTED — not a defect.** The reproduction was malformed (`n = <<#-1>>` at index 0 reaches before the brane); BRANING forever is the honest answer to a self-referential question. |
| **§5.6's bespoke concatenation search handler** | **Designed, implemented, proven dead code, removed.** The general IB-then-AB fallback already implements it. |
| **Pure-Foolish `'or` truth table** | **Implemented and failed** — a value search inside `system.foo` cannot resolve when its operand is ECONSTANIC there. |
| **Breadth-first stepping inside UBCa** | **Rejected, and renamed UBCc.** FIFO sequential draining is Foolish **semantics**, not strategy: ~185 NYES reads and ~83 value reads rest on the entitlement that predecessors have settled, and FOOP-23 *defines* the immediate brane as "lines before the current expression." Breadth-first does not reorder the meaning of "so far" — it dissolves it. Therefore it warrants a **separate implementation with its own baselines**, never a mutation of this one. |
| **The 113 branch commits** | **Not merged, not cherry-picked.** `foolish-ubca` stays frozen as the oracle. |

**The UBC lineage** (authoritative code names — a letter names an *implementation of the
evaluator*, not a version of the language; any two that disagree about what a program means
have a bug in at least one): **UBCa** in use; **UBCb** dependency-tracking with priority
stepping, attempted for months and not adopted, machinery expected to be reusable in UBCd;
**UBCc** breadth-first, proposed; **UBCd** message-passing, proposed, worth its own Major.
`foolish-ubca2` is **not** a new letter — it is UBCa's semantics on arena-backed storage.

## Out of scope — FOOP-55's event-based child-readiness refactor

FOOP-55's §11 proposed re-expressing child-readiness as an **event/handler** mechanism — four
new trait methods, whole-set gates, and `on_*_op_ready` handlers that *report* a NYES for a
single central caller to commit. **That refactor is out of scope here and gets its own
document. It may not be needed at all.**

Two reasons, and the second is decisive:

1. §1 captures everything this FOOP actually depends on — the three beats and the mark status —
   **without** introducing a handler protocol.
2. **`foolish-ubca2`'s `ProtoBrane` is not the type the refactor is named for.** FOOP-55's §11
   proposal hangs its handlers off `foolish-ubca`'s `ProtoBrane` — a `RefCell`/`Cell`-wrapped
   `dyn Fir` trait object. `foolish-ubca2` has a `ProtoBrane` too (§9 — the payload struct
   formerly named `ArenaFir`), but it is a plain evaluation-state struct sitting beside `Slot`
   (topology) in the arena, with no `RefCell`/`Cell` wrapper and no vtable: its `&mut
   FVMStorage` borrow replaces both. The refactor's central premise — hanging overridable
   handlers off a `dyn Fir` vtable, explicitly "no `match self.kind()` anywhere" — is the
   *opposite* of ubca2's closed-enum dispatch, and ubca2's `ProtoBrane` gives it nothing to
   hang them on.

If the event mechanism is later wanted, it should be specified against ubca2's actual shape,
not carried over as a vtable design.

## Misc — `foolish-ubca2` cleanup tracked here

Small items found while surveying, not worth their own FOOP:

- **`ArenaFir` renamed to `ProtoBrane` (done — this FOOP's first landed change).**
  `foolish-ubca2`'s per-node payload struct — FOOP-16's `ArenaFir` — is now named
  `ProtoBrane`, matching the name `foolish-ubca` uses for the type filling the same role: the
  shared, mutable per-node evaluation state that sits beside the kind-specific data (see §1's
  `Slot`/payload split, and the table entry translating `ProtoBrane::constanic_clone_at` to
  `FVMStorage::revive_constanic`). The rationale is that both types carry the same role in
  their respective crates, so they take the same name; the two crates now use one vocabulary
  for one concept, and every comparative statement in this spec — and in the FOOP-16 addendum
  — reads directly instead of through a translation step. **Caution:** `foolish-ubca` and
  `foolish-ubca2` now both have a `ProtoBrane`, and they are different Rust types in different
  crates (ubca's is a `RefCell`/`Cell`-wrapped `dyn Fir` trait object; ubca2's is a plain
  arena-backed struct — see §"Out of scope", point 2), so anyone reading a bare `ProtoBrane`
  must know
  which crate they are in. Note `foolish-ubca2` never calls into `foolish-ubca`: the two are
  independent implementations, and ubca2's few doc-comments that once compared the two were
  reworded to describe ubca2's own contract rather than to name ubca's type.
  **`FOOP-16.addendum.md` has been updated** to the new name (it is a post-completion note, not
  spec), but **`FOOP-16.md` and `FOOP-16.plan.md` are deliberately left saying `ArenaFir`** —
  completed FOOPs are a historical record and this project does not rewrite them. A reader of
  those two should map `ArenaFir` → `ProtoBrane`.
- **Rendering: there is no ubca2 sequencer — the sequencer is shared, reached by an adapter.**
  `FirSequencer`/`HumanizingFirSequencerRef` live once, in `foolish-core/src/sequencer.rs`, and
  render `foolish_core::fir::Fir`. Neither evaluator crate has its own. Each instead owns a
  **conversion adapter** that materializes its private tree into that shared core FIR:
  `foolish-ubca`'s free functions in `evaluator.rs` (`proto_to_core_fir`, ~line 187) and
  `foolish-ubca2`'s `mod core_fir_conversion` (`fvm_storage.rs:3251-4055`, `proto_to_core_fir`
  at `:3368`). This is *why* the two implementations can be compared byte-for-byte at all —
  they converge on one representation before anything is printed, so an einmo diff is a
  difference in evaluation, never in rendering. **Consequence for this FOOP, and it is not theoretical:** the
  adapter is *not* a thin structural copy — its `StayFoolish`/`StayFullyFoolish` arms
  (`fvm_storage.rs:3840-3950`, plus `proto_to_core_fir_sff_body`/`_sff_operand` at `:3378`/
  `:3425`) carry ~100 lines of mark-specific logic about what to show under a mark. §2 and §3
  change which marks survive a copy, so **the adapter is a second place every mark change must
  land** — a correct evaluation can still render wrong. A UFM kind (§3) also needs an arm here,
  or it renders as nothing. Conversely a pure *rendering* fix — such as the settled-wrapper
  ambiguity below — belongs in `foolish-core`, where it lands for both implementations at once.

- **Sequencer: a settled wrapper hides the value it holds** (human observation, 2026-08-28,
  reading `misc/sf_of_sff`). Statement #3 renders as
  `sf=?(result=<<WOCONSTANIC Op+(...)>>, ...)`, which reads as though the `Op+` were rendered
  poorly. Traced: the wrapper is genuinely in the FIR, not a rendering artifact — but the
  rendering makes **two different situations look identical on screen**: a value merely
  *displayed* under a mark versus one *blocked* by it. Worth deciding separately: should the
  sequencer show the value inside a settled wrapper, so a reader can tell "wrapped but
  readable" from "wrapped and therefore unresolvable"? This matters more once §2 lands, since
  retained marks become common. Explicitly **not** the same question as §2's mark state.

- **Documentation.** `foolish-ubca2` has no crate-level architecture document. The FOOP-16
  addendum covers container structure; nothing covers the `fvm_storage.rs` module layout
  (`arena_compiler`, `core_fir_conversion`, `search_engine`, `search_fir_dispatch`) or the
  three allocation primitives (`make_my_child` / `make_orphan_child` / `make_root`) whose
  misuse is a real silent bug.
- **`*_nyes_transitions` coverage is thin** — 6 mentions in ubca2 against 23 in ubca's
  `fir_kinds.rs`. `AGENTS.md` requires these per kind, and this FOOP changes NYES-adjacent
  behavior, so the gap must be closed for every kind it touches. They cannot be copied from
  ubca — those are written against `dyn Fir`.
- **`Index` dispatch is asymmetric** — inline in `fir_op_step` (`:1181-1361`) while `Search`
  dispatch lives in `search_fir_dispatch`. A change touching index searches lands in a
  different place than one touching name searches.
- **Dead search types** — `CursorSource` (`:2138`) and `SearchPredicate::Head`/`Tail` carry
  `#[expect(dead_code)]`; `^`/`$` compile down to `Index`.
- **`sf_inner_pattern` was dropped** in the migration. If any of this work needs it, the field
  must be re-added to `ProtoBrane` first.
- **`FirCursorMut::check_sff_marked_child`** has exactly one production call site, where ubca
  had two — ubca2's `system_foo` path no longer calls it. Confirm that is intended before §2
  changes the mark-state threading.

## Rejected Alternatives

**A. Merge or cherry-pick the FOOP-55 branch.** Rejected: zero textual overlap with ubca2 (no
`fir_kinds.rs`, no `fir_trait.rs`, no `proto_brane.rs`, no `compiler.rs`), four regressed
shipped baselines, ~200 open checkboxes, and a final commit named `buggy`. The branch's `.md`
is the asset; its `.rs` is not.

**B. Finish FOOP-55 on `foolish-ubca` first, then port.** Rejected: doubles the work and ports
a result that is not green. It also further entrenches the frozen oracle, which FOOP-16
deliberately stopped changing.

**C. Keep concatenation brane-like and fix the symptoms individually.** Rejected: the symptoms
(D9, D10, the classifier bug) are one category error, and fixing them separately is what
produced three successive wrong root-cause diagnoses on the branch.

**D. Carry FOOP-55 §11's event/handler mechanism across.** Rejected — see §"Out of scope".
Its premise (vtable
overrides, "no `match self.kind()`") is the opposite of ubca2's dispatch, and the type it is
named for no longer exists.

**E. Do nothing.** Rejected: `foolish-ubca2` today has the *same* premature-strip defect, so
the first macro anyone writes on it dies the same way `'cmod` does — NK at its definition site,
unrecoverable.

## References

- `docs/foop/FOOP-16.md`, `FOOP-16.plan.md`, `FOOP-16.addendum.md` — `foolish-ubca2`.
- `docs/foop/FOOP-46.md` — the `BraneConcatOp` rewrite. Implements §4's contract against a
  phased search behavior, and carries the unresolved bare-name-constituent question (§4.3's
  second nested example). Ordered after this FOOP.
- `docs/foop/FOOP-36.md` — a `foolish-ubca2`-owned Foolish-rendering sequencer. **Not disjoint
  from this FOOP**: it overlaps in `fvm_storage.rs`'s `core_fir_conversion` and in
  `ubca_snapshot_tester.rs`, and it rewrites all 179 ubca2 baselines once for rendering
  reasons. See Open Question 8 for the ordering recommendation.
- **FOOP-55, on branch `worktree-foop-55-event-handlers`** — `docs/foop/FOOP-55.md` (3222
  lines), `FOOP-55.plan.md`, **`FOOP-55.addendum.step_cc_marker_table.md`** (the 3×3 table
  §2 reduces; human-confirmed, empirically probed — **read before changing mark behavior**),
  `docs/foop/UFM-scoping-study.md`.
- FOOP-23 (searches, the two-child invariant), FOOP-33 (named creations, system operators),
  FOOP-65 (tail concatenator, the Equivalence Law), FOOP-73 (`'or` design).
- `AGENTS.md` §"Development Rules" — the non-regression invariant and the Promotion Review Gate.

## Last Updated

**Date**: 2026-09-02
**Updated By**: Claude Code / claude-opus-5
**Changes**: Draft, organized in four parts — **I Motivation** (what happened on the FOOP-55
branch, and the defects re-measured live against `foolish-ubca2`), **II Specification** (what
`foolish-ubca2` shall do), **III Design** (the types that resolves to, defined once), **IV Plan
of Implementation** (order, tests, open decisions).

Three changes: (1) SF/SFF mark handling becomes an explicit mark status, (2) UFM `<<<…>>>` is a
new mark and operator, (3) concatenation becomes an operator with compiler-supplied constituent
ergonomics — held together by the three-beat step (§1) and the context-constancy rule (§5). Each
construct is specified along one frame: what it is, how it parses, how it compiles into FIR, how
it steps its `foolish_children` to constanic and then its `ubc_children`, and what a constanic
copy made under it does. §2.3 names four changes that would break SFF, because the FOOP-55
branch's SFF work failed by altering SFF's behavior while intending only to change how it was
carried.

Part III defines `MarkState { Normal, UnderSfm, UnderUfm }` on the stepping scope and
`CloneMarks { UnderSfm, UnderUfm }` at the copy boundary, with the one mapping between them, a
table of where each is set/propagated/consumed, and what is deliberately not changed.

**Terminology aligned to `AGENTS.md` §Foolish Terminology** (2026-09-02): §1 carries a note
defining *constanic*, *constantew* and *conclusive*, and every gate in this FOOP now says which
cut it means. The two that differ exactly on NK are used deliberately — the flatten-time type
check gates on **constantew** (an NK constituent is a settled type error, so there is nothing to
wait for; a conclusive-only gate would send it to the wait branch and hang), while the
constituent-readiness chain walk ends on **conclusive** (a hop that reached a value; an NK hop
never did). `transform_for_clone`'s preserved states are named *constantew* rather than the
ambiguous "terminal", since constanic states are terminal too but are reset.

Measured 2026-09-02 and recorded: a constanic copy drops SF/SFF markers unconditionally at every
depth, so marker removal is not what the copy-side flag selects — NYES treatment is. That is why
the copy-side type is two-valued while the stepping status is three-valued. §4.3 is worked
through with Foolish examples, each evaluated against `foolish-ubca2`: brane and search
constituents, cross-constituent search, nested concatenation, an explicit mark left alone, and
the one that fails — `{a = {x=1;} \`99; a;}` silently yields `a={}` instead of NK naming the
kind.

Human decisions recorded 2026-09-02: **`BraneConcatOp`'s implementation is split out to
FOOP-46**, to be rewritten rather than patched, against a phased search behavior (during
Gathering an IB search demanded by a constituent finds nothing here and falls through to the
parent; once Joined, IB searches resolve normally within `ubc_children`); §4 keeps the operator's
contract and ergonomics, and the unresolved bare-name-constituent case in §4.3 is handed to it.
The strip **budget** is not carried over; the preferred
replacement (a brane-depth mark remover) is deferred to its own FOOP with the four program shapes
it must answer recorded in §2. UFM is an operator: it steps its constituent to constanic, copies
it to `ubc_children` under `under-ufm`, and that copy steps `under-ufm` while its
`foolish_children` do not. `foolish-ubca` compiles a UFM into
`NkFir { reason: "Syntax Not Implemented" }` with children discarded. The deliberate divergence
between the two evaluators is partly the point: this FOOP is the controlled experiment for
whether the arena model makes the language easier to develop and debug. Records this FOOP's first
landed changes: `ArenaFir` → `ProtoBrane` and `lib.rs` restated so the two crates read as
independent implementations.
