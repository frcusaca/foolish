---
foop: D75
title: Assignment Attached Searches — `LHS =SEARCH_SPEC RHS` as sugar for `LHS = RHS SEARCH_SPEC`
author: Sisyphus / claude-opus-5 (directed by Atlas)
status: Draft
type: Standards
created: 2026-08-07
phase: phase-2
supersedes: []
begun: [ ]
---

# FOOP-75: Assignment Attached Searches — `LHS =SEARCH_SPEC RHS` as sugar for `LHS = RHS SEARCH_SPEC`

FOOP numbering is little-endian; the full rules live in `foop.md` at the
repository root — **read it before creating or editing a FOOP.** The one
template-specific note: the `foop:` front-matter field may either match the
filename digits directly, or give the big-endian decimal value preceded by
`D` (this file: `foop: D75` — digits `75` reversed = sort key 57). In all
cases, the `FOOP-75.md` file name is ultimately the right numbering.

## Abstract

This FOOP generalizes the statement form `LHS = RHS` to **`LHS =SEARCH_SPEC
RHS`**, which is defined to mean exactly `LHS = RHS SEARCH_SPEC`. The
suffix is triggered when the character **immediately after** the `=` of a
statement is the start of a search operator — one of `^ $ ~ ? # .` — and it
means: parse that search specification, then anchor it on the **end of the
RHS**. `=SEARCH_SPEC` admits **no spaces**, matching the existing rule that
spaces are never allowed inside a search specification. The **reverse
direction** is a sequencer obligation: given a settled statement whose body
is a search (and whose anchor is itself a search, transitively, until the
anchor is not a search), the sequencer lifts that entire run of searches out
of the body and renders it immediately after the `=`. So `A = B~=5#-2`
sequences to `A =~=5#-2 B`.

This subsumes and repairs the two existing ad-hoc sugars, `=$` and `=^`,
which today are separately hand-coded, mutually inconsistent, and — as
this FOOP documents — **both defective**.

## Motivation

### What exists today

Two special cases are hard-coded in `parse_assignment`
(`foolish-parser/src/parser.rs:326-354`): if the token after `=` is `Dollar`
or `Caret`, the parser rewrites the statement into a `BinaryOp` whose left
operand is a synthetic `UnanchoredSeek { offset: -1 }`. FOOP-54 §D.5
(status `Complete`) specifies the intended meaning:

> **Bind-tail (`=$`)**: `a =$ b` ≡ `a = b$` — "bind the value of the last
> statement of `b` to the name `a`."

That is precisely the generalization this FOOP makes uniform. But the
existing implementation does not deliver it. Verified live on `jia` at
`dc6db093` (each row run through `UbcaEvaluator` + `FirSequencer::format`):

| Source | Actual output | Expected per FOOP-54 §D.5 |
|---|---|---|
| `{b = {1,2,3}; z = b$}` | `z=3` | `3` ✅ |
| `{b = {1,2,3}; y =$ b}` | `y =$ { 1; 2; 3 }` (WOCONSTANIC) | `3` ❌ |
| `{b = {1,2,3}; z = b^}` | `z=1` | `1` ✅ |
| `{b = {1,2,3}; y =^ b}` | `y=Op^({1;2;3}, {1;2;3}, WOCONSTANIC)` | `1` ❌ |

Three distinct defects are visible here:

1. **`=$` computes the wrong value.** It yields the whole brane, not its
   tail. `fir_kinds.rs:713`'s `"$"` arm validates that the RHS is a brane
   and then `return Ok(())` without extracting the tail element.
2. **`=^` does not evaluate at all.** There is **no `"^"` arm** in
   `fir_kinds.rs` to match `"$"` at line 713, so the `OperatorFir` never
   settles to a value and leaks into rendered output as `Op^(...)`.
3. **The reverse direction is missing for the postfix forms.** `b$` compiles
   to an `IndexFir` (`compiler.rs:354`), not an `OperatorFir`, so the
   sequencer's sugar branch — gated on `hs_operator()` returning `Some(("$",
   ..))` at `sequencer.rs:650` — never fires for it. `z = b$` renders as
   `z=3`, with the `$` gone entirely.

### The corpus already assumes this rule

The plan's Phase 1 survey found that §2's rewrite is **already documented in
the source corpus**, independently of FOOP-54 §D.5. Two files state it
verbatim in comments:

```foolish
h =$ #-1 ;   !! Syntactic sugar for h = #-1$ (tail of #-1)
j =^ #-3 ;   !! Syntactic sugar for j = #-3^ (head of #-3)
                        !! — test-resources/.../unanchoredSeekBasic.foo:26,28
e =$ #-2;    !! Syntactic sugar for e = #-2$ - #-2 is b, tail of b is 30
                        !! — test-resources/.../test_unanchored_oneshot.foo:6
```

An attached **chain** already appears too — `c =$#-1;`
(`test-resources/.../test_syntax.foo:4`), i.e. `c = #-1$` under §2/§3.

The same is true of the space and parenthesis rules of §5/§6. In
`test-resources/.../regexSearchShadowy.foo`, the author annotates each
chained search with its expected result and calls out the delimiter
explicitly:

```foolish
result0 = brn2.hard_to_typo_name_for_a_brane .bobulous;  !! ... NOTE the space
result3 = brn2?(.*brane)?a.*;                            !! 5 !! advent
result4 = brn2?(.*brane) .bobulous;                      !! still 7: NOTE the space
```

**None of these work today.** Measured on `jia@dc6db093`,
`brn2?(.*brane)?a.*` yields a single search with pattern `"(.*brane)?a.*"`,
and `brn2?(.*brane) .bobulous` yields pattern `"(.*brane).bobulous"` — the
space is absorbed, the parens are absorbed, and no chain forms. These files
are evidence that Foolishers already expect §5 and §6.2, and write code as
though they hold.

Additionally the spelling is inconsistent across documents: FOOP-23 §942/946
asserts that `a$=b` / `a^=b` are "already implemented" and elects to keep
them. **That is a transposition error** — the parser implements `a=$b` /
`a=^b` (dispatching on the token *after* `Assign`). FOOP-55 §D6
independently rediscovered that `$=` does not parse, and §E5 rewrote all six
of its `$=` lines as explicit `(X)$` to work around it.

### What the world looks like after

One rule replaces two special cases, and it covers **every** search
operator rather than just `$` and `^`:

```foolish
call_result =$ {a=10, b=-3} fn      !! bind fn's tail  (FOOP-54 §D.5)
first       =^ some_brane           !! bind the head
found       =?name some_brane       !! bind a backward name search
nth         =#-2 some_brane         !! bind a positional index
matched     =~=5 some_brane         !! bind a forward value search
```

Each is defined by a single mechanical rewrite, so there is no per-operator
semantics to specify, implement, or get wrong. The function-application
idiom FOOP-54 §D.5 documents becomes correct rather than aspirational, and
the round-trip property (parse → evaluate → sequence → same text) holds for
the whole family instead of one member.

## Terminology

These terms are authoritative for this FOOP and are intended to enter
general project use (AGENTS.md §Foolish Terminology).

- **Attached search** — a search specification written **immediately after
  a statement's `=`**, with no intervening space, and terminated by a
  space. `A =$ B` has an attached search `$`. `A = B$` does not (its `$`
  is an ordinary postfix search). An attached search is *attached to the
  assignment*, and is defined to apply to the RHS (§2).
- **Attached search sequence** — the full run of attached searches on one
  `=`, when more than one is chained: in `A =#-2$ B` the sequence is
  `#-2$`, two searches deep (§3).
- **Postfix search** — the ordinary, already-existing spelling in which the
  search follows its anchor expression (`B$`). Every attached search is
  defined by rewriting to a postfix search (§2); the two spellings are the
  same program.
- **Suffixed statement** — a statement that has an attached search.

## Specification

### §1. The trigger

In a statement, when the character **immediately following** the `=` is the
first character of a search operator, the statement is a **suffixed
statement**. The trigger set is:

| Char | Operator |
|---|---|
| `^` | head |
| `$` | tail |
| `~` | forward name search (and `~=`, forward value search) |
| `?` | backward name search (and `?=`, backward value search) |
| `#` | positional index |
| `.` | deepening name search |

**Immediately** is literal: `=$` triggers, `= $` does **not**. This
restates the existing project-wide rule that a search specification never
contains spaces (AGENTS.md §Searches); the suffix position is no exception.
`A = B` with a following space is an ordinary assignment whose RHS happens
to begin with something else.

### §2. The rewrite (parse direction)

```
LHS =SEARCH_SPEC RHS     ≡     LHS = RHS SEARCH_SPEC
```

The rewrite is **purely syntactic and total**. The parser parses
`SEARCH_SPEC` exactly as it would in postfix position, then anchors it on
the **end of the RHS** — that is, the fully-parsed RHS expression becomes
the search's `anchor`. There is no new AST node, no new FIR variant, and no
new evaluation rule: the resulting tree is **identical**, node for node, to
the tree produced by the postfix spelling.

Grammar fragment (`parse_assignment`):

```
assignment  ::= characterizations? IDENT '=' search_spec? expr
search_spec ::= <the postfix suffix loop of parse_postfix, with no leading space>
```

Implementation shape: after consuming `Token::Assign`, if the next token
begins a search operator **and is adjacent** (see §5), record the suffix
token position, parse the RHS expression, then replay the recorded suffix
against the parsed RHS using the *existing* postfix-suffix construction —
the same code path that builds `Astn::HeadTail`, `Astn::Seek`,
`Astn::RegexpSearch`, and `Astn::ValueSearch` at `parser.rs:640-760`.
Reusing that path is what guarantees the trees are identical.

### §3. Chains

A `SEARCH_SPEC` may be a **chain** of searches. Chained postfix suffixes
already build a **left-nested spine** through each node's `anchor` field,
each wrapping the previous expression. Verified live:

```
B~=5#-2   parses to   Seek { offset: -2, anchor: ValueSearch { anchor: B, forward: true,
                                                               value_pattern: 5 } }
```

The suffixed form must produce that same spine. So:

```
A =~=5#-2 B     ≡     A = B~=5#-2
```

Note the order: the chain is written in the suffix **in the same left-to-right
order it would appear postfix**. The leftmost search in the spec is the
innermost (closest to the RHS) in the resulting spine.

### §4. The reverse direction (sequencer obligation)

> **CORRECTED 2026-08-08 during Phase 5 implementation.** As first written,
> this section claimed the normalization applies to *every* statement whose
> body is a search. That is **wrong**, and the implementation found the
> boundary the spec missed. The corrected rule is in §4.0; the original text
> below stands for the cases it does cover (unsettled and NK).

#### §4.0 The rule applies only where the search is still visible

The sequencer's governing principle — long predating this FOOP — is
**transparency when settled**: a search that found its target renders as
**the value it found**, not as the search that found it. This is enforced by
`should_show_nyes()` / `should_show_search_nyes()` in `render_fir`.

So `z = b$` renders `z=3`. The `$` is **not "lost"** — the search is
*resolved*, exactly as `x = 1+2` renders `x=3` rather than `x=Op+(1,2)`.
§Motivation's original framing of this as defect (3) was mistaken about the
cause, though the `=$`/`=^` defects (1) and (2) were real and are fixed.

The attached form therefore applies **only to statements that still show
their search structure** — those that are unsettled, or that settled NK:

```
e = {}^        renders   e =^ {} (???)          !! NK — structure shown
d =$ 4         renders   d =$ 4 (???)           !! NK — structure shown
z = b$         renders   z=3                    !! settled — value shown
y =$ b         renders   y=3                    !! settled — same value
```

Normalizing the settled cases would mean **un-resolving** the output —
contradicting the rendering model and making every snapshot noisier for no
gain. And the round-trip property survives: for settled statements both
spellings converge on the same *value*, which is a stronger agreement than
converging on the same *text*.

**Consequence for §8**: the six frozen baselines still move, but every
changed line is an NK rendering. No settled value changes. Verified line by
line in Phase 5.

#### §4.1 The original text (applies to unsettled and NK statements)

Given a settled statement, the sequencer walks the body's **anchor spine**:
while the node is a search, follow its anchor; stop at the first node that
is not a search. That entire run of searches is lifted out and rendered
immediately after the `=`, in spine order (innermost first), and the
non-search node at the bottom of the spine is rendered as the RHS.

```
A = B~=5#-2      sequences to      A =~=5#-2 B
```

The FIR kinds that count as "a search" for spine-walking purposes are
those reachable by the trigger set of §1: `IndexFir` (covering `^`, `$`,
and `#`), `SearchFir` / the `ContextfulSearch` family (covering `~`, `?`,
`.`, and the value forms). A statement body that is not a search renders
exactly as it does today — the fallback is total.

**This obligation applies to the postfix spelling too.** `A = B$` and
`A =$ B` produce the same tree (§2), so they necessarily sequence to the
same text. Per §4 that text is `A =$ B`. This is a deliberate
**normalizing** choice: the suffixed form is canonical in output.

### §5. Space is the terminator — and the lexer must first be taught to see it

#### §5.1 The rule — an attached search MUST be space-terminated

**If a statement's `=` has an attached search, that attached search MUST be
terminated by a space character.** The space is a **required delimiter**,
not merely a boundary that happens to fall there.

Three parts, all mandatory:

1. **Attachment requires adjacency.** `=$` has an attached search; `= $`
   does not. The suffix exists only when the search operator immediately
   follows the `=` with no intervening space.
2. **The run continues while no space intervenes.** `=#-2$` is one attached
   search sequence, two searches deep. Chains continue by adjacency.
3. **The run MUST end with a space.** A statement whose attached search is
   terminated by anything else — `;`, `,`, `}`, `)`, or EOF — is a **parse
   error**, not a statement with an empty RHS.

Point 3 is what makes the form self-checking. An attached search is a
promise that an RHS follows, so:

```foolish
A =$ B          !! valid — attached `$`, space, then RHS `B`
A =$;           !! PARSE ERROR — attached search not space-terminated
A =$}           !! PARSE ERROR — same
A = B$          !! valid — no attached search; ordinary postfix (§2)
```

The error must name the rule: something of the form *"attached search must
be terminated by a space"*, citing the offending statement's line and
column. Diagnosing this at parse time is the point — the alternative is a
confusing downstream failure about a missing operand.

This restates, at statement level, the project-wide rule that a search
specification never contains spaces (AGENTS.md §Searches), and adds the
converse: the specification's *end* is marked by exactly one thing, a
space.

Note the interaction with §6: a space also ends a **pattern**, so
`=~(.*asdf$) B` needs its parens for the pattern's internal `$`, and the
space before `B` is simultaneously the pattern's terminator (§6) and the
attached search's required terminator (this section). One rule, applied at
two nesting levels.

#### §5.2 The lexer does not currently preserve this information

**This is a prerequisite, not a detail.** Verified live on `jia` at
`dc6db093` — these three sources lex to **byte-identical token streams**:

```
"{a =$ b}"      →  LBrace@2 Ident(a)@2 Assign@4 Dollar@5 Ident(b)@5 RBrace@7 Eof@7
"{a = $ b}"     →  LBrace@2 Ident(a)@2 Assign@4 Dollar@5 Ident(b)@5 RBrace@7 Eof@7
"{a =   $ b}"   →  LBrace@2 Ident(a)@2 Assign@4 Dollar@5 Ident(b)@5 RBrace@7 Eof@7
```

The cause: `Lexer::skip_whitespace` (`lexer.rs:38-58`) advances `pos` past
spaces and tabs **without incrementing `column`** (`column` is only bumped
by `advance()` at `lexer.rs:72`, and reset to 1 on newline). `column` is
therefore a count of *consumed non-whitespace characters since the line
started*, not a character offset — adequate for error messages, but it
**cannot** answer "were these two tokens adjacent?"

Consequently §5.1 is **not implementable** against the current token stream.
The parser has no way to distinguish `=$` from `= $`.

#### §5.3 Required lexer change

The lexer must record, per token, whether whitespace preceded it. The
scaffolding is already present but unused: `Lexer::next_token` returns
`(TokenAndLocation, bool)` and the caller at `lexer.rs:32` discards the
second element as `_skip_leading_space`. The change is to compute that
flag in `skip_whitespace` (did it consume anything?) and carry it onto the
token:

```rust
pub struct TokenAndLocation {
    pub token: Token,
    pub line: u32,
    pub column: u32,
    /// True when whitespace (space, tab, or newline) immediately preceded
    /// this token. FOOP-75 §5: a space terminates a search-specification
    /// sequence, so the parser needs adjacency, which `column` cannot
    /// supply (it does not count skipped whitespace).
    pub preceded_by_space: bool,
}
```

The parser's suffix trigger (§2) then reads: the token after `Assign`
begins a search operator **and** `!preceded_by_space`. Chain continuation
(§3) reads the same way for each subsequent suffix token.

**Compatibility**: adding a field to `TokenAndLocation` touches its
constructor and any exhaustive struct-literal construction. It is purely
additive to behavior — nothing consults the new field except the §2
trigger — so no existing parse changes. `TokenAndLocation::new` gains the
argument; the plan updates all call sites.

Two consequences worth stating plainly:

- Fixing `column` to count whitespace was considered and **rejected**: it
  would change every existing parse-error message's reported column, moving
  einmo baselines for error cases for no gain, and it conflates "where is
  this for a human" with "was this adjacent" — two different questions.
- Because the flag is per-token rather than a re-lex, there is no
  performance cost beyond one bool per token.

#### §5.4 `&` is excluded from the trigger set

`&` (contexted search) is deliberately **excluded** from the §1 trigger set.
`&` is not a search operator on its own — it is a cursor-source modifier
that prefixes one (AGENTS.md §Searches, group 2). Admitting `=&?x` would
raise the question of what position the contexted search reads from when its
anchor is a whole RHS expression rather than a statement position, and that
question has no obviously correct answer. It is left to a future FOOP; see
Open Questions.

### §6. Pattern-boundary ambiguity after `?` and `~`, and the parenthetical form

#### §6.1 The ambiguity is genuine, not merely a greedy parser

`$` and `^` are **both** Foolish search operators (§1) **and** regexp
anchors (end-of-string, start-of-string). After a `~` or `?`, a bare `$`
is therefore truly ambiguous:

```foolish
A = B~.*asdf$      !! is the trailing $ a regexp end-anchor, or a tail search
                   !! chained onto the name search?
```

This is **not** resolvable by making the pattern scanner stop at `$` — that
would break every legitimate end-anchored pattern. It requires an explicit
delimiter.

Current behavior, verified live on `jia` at `dc6db093`:

| Source | Resulting pattern | Chain parsed? |
|---|---|---|
| `B~.*asdf$` | `".*asdf$"` | no — `$` absorbed as anchor |
| `B~(.*asdf$)` | `"(.*asdf$)"` | no — parens absorbed *into* pattern |
| `B~(x)#1` | `"(x)#1"` | no — `)` does not terminate |
| `B~(x)$` | `"(x)$"` | no |
| `B?x#1` | `"x#1"` | no |

`parse_regexp_pattern` (`parser.rs:800-891`) breaks only on `;`, `,`, `}`,
`)`, `]`, EOF, line comment, `=`, and `&`. Its `LParen` arm (823-842)
copies the parenthesized run **including both parens** into the pattern and
continues scanning. So today, **no** chain after `?`/`~` parses as a chain,
and parentheses do not delimit.

#### §6.2 The parenthetical form (specified here)

A search pattern **may** be written parenthesized, and when it is, the
**matching close paren terminates the pattern**:

```
~(PATTERN)     the pattern is exactly PATTERN; scanning resumes after `)`
~PATTERN       the pattern runs to the existing break set (unchanged)
```

This makes chains through `?`/`~` expressible, and makes an end-anchored
pattern unambiguous:

```foolish
A =~(.*asdf$) B      !! forward name search, pattern `.*asdf$` (end-anchored)
A =~(.*asdf)$ B      !! forward name search on `.*asdf`, THEN tail
A =?(x)#-2 B         !! backward name search on `x`, then index -2
```

The outer parens are **delimiters, not pattern text** — `~(x)` and `~x`
produce the identical pattern `x`. Nested parens inside `PATTERN` are
regexp grouping and are preserved, so the terminator is the paren that
**matches** the opening one, tracked by depth.

Rationale for parens over a new sigil: parentheses already read as grouping
everywhere else in the language, they are already lexed
(`Token::LParen`/`RParen`), and the existing `LParen` arm already walks a
paren run — the change is to make that walk depth-tracking and terminating
rather than absorbing.

#### §6.3 Compatibility and its limit

`~(x)` currently yields pattern `"(x)"`; under §6.2 it yields `"x"`. For a
**name** search these match the same names (an unanchored group around the
whole pattern is semantically inert), so ordinary programs are unaffected.
Two cases are **not** inert and must be checked, not assumed:

- A pattern whose parens are *not* the whole pattern, e.g. `~(a|b)c` —
  under §6.2 the pattern terminates at `)` and `c` begins a chain, which is
  a **meaning change**.
- Any rendered baseline that contains a parenthesized pattern, since the
  stored pattern text changes.

The plan therefore requires a repo-wide survey of existing `~(`/`?(`
occurrences **before** implementing §6.2, and treats any hit as a
review item rather than a mechanical update. If the survey shows the
meaning-change case occurs in practice, §6.2 is **split into its own FOOP**
and this FOOP proceeds with §6.4 alone.

#### §6.4 If §6.2 is deferred

Without §6.2, chains are reliable only where every element is `^`, `$`, or
`#` (e.g. `=#-2$`, `=$#1`), and `A =?x#1 B` yields one `RegexpSearch` with
pattern `x#1` — matching postfix `A = B?x#1` exactly, per §2. That
inheritance is correct behavior for this FOOP even though the inherited
behavior is undesirable; the test plan pins it either way, with a comment
citing this section.

### §7. Disposition of `$=` and `=^`

- **`$=` is not adopted.** FOOP-54 §D.5 canonicalizes `=$`, the parser
  implements `=$`, and the verified baseline pins `=$`. FOOP-23 §942/946's
  `a$=b`/`a^=b` is a transposition error and is **corrected** by this FOOP
  (a documentation change to FOOP-23, not a code change).
- **`=^` and `=$` become instances of §2** rather than hand-coded special
  cases. The bespoke branches at `parser.rs:326-354` are **deleted**, and
  with them the synthetic `UnanchoredSeek { offset: -1 }` left operand.
- Because `=$` will then compile to an `IndexFir` (via `Astn::HeadTail`)
  exactly as `b$` does, defects (1) and (2) from §Motivation **dissolve**:
  there is no `OperatorFir` `"$"`/`"^"` path left to be wrong. The
  now-unreachable `"$"` arm at `fir_kinds.rs:713` is removed.

### §8. Behavior change to verified baselines

> **Revised 2026-08-07 after the plan's Phase 1 survey.** This section
> originally named one frozen baseline; the survey found **six**. The
> difference matters: §4's normalization affects **postfix** inputs too
> (that is its purpose), so any baseline rendering a `$`/`^` statement
> moves, not only those whose *input* uses the `=$` sugar.

Six `verified/` (human-signed, frozen) baselines are affected, each with a
`checked/` twin that also moves:

| `verified/` baseline | example input | renders today |
|---|---|---|
| `misc/head_tail_empty_brane` | `{e = {}^; f = {}$;}` | `e=^(NK); f=$(NK)` |
| `misc/anchored_search_on_constanic` | — | `chained=^(NK)` |
| `misc/offset_access_empty_brane` | — | `result=^(NK)` |
| `foop/33/boolean/comparison_non_integer` | — | `braneˍoperand=$(…, NK)` ×4 |
| `foop/42/…hfs` | — | (examine during Phase 7) |
| `regression/disappearing_brane_statements` | `d =$ 4` | `d =$ ??? (…)` |

Note there are **two distinct existing renderings**, and §4 unifies them:

- `e=^(NK)` — `name=` followed by the search FIR's *own* rendering. This is
  the ordinary path; the `^`/`$` here is the search's self-description, not
  the sugar.
- `d =$ ???` — the hand-coded sugar branch at `sequencer.rs:650`.

Under §4, `{e = {}^}` renders `e =^ {}` and the first shape disappears.

The `disappearing_brane_statements` case additionally changes *evaluation
path*: `d =$ 4` ≡ `d = 4$` — a tail search anchored on the non-brane `4`.
The anchored-miss rule (AGENTS.md §Searches: "Anchored miss → NK") says
this settles NK, so it remains an error line, but its exact text may change
(the alarm reason comes from the `OperatorFir` arm being deleted in §7; the
`IndexFir` path produces its own). It pins:

```
  d =$ ??? (4 is not a brane);
```

Under this FOOP, `d =$ 4` ≡ `d = 4$` — a tail search anchored on the
non-brane `4`. The anchored-miss rule (AGENTS.md §Searches: "Anchored miss →
NK") says this settles NK, so the line remains an error line, but its exact
rendered text may change (the alarm reason is produced by the `OperatorFir`
arm being deleted in §7; the `IndexFir` path produces its own).

**These baselines are frozen and MUST NOT be promoted over by an agent**
(AGENTS.md §Non-regression invariant). The plan gates on presenting every
diff to the human reviewer for a signing decision. If the human declines a
change, this FOOP must preserve the exact existing text — for the non-brane
case that is achievable by giving the `IndexFir` miss path the same alarm
reason — and that becomes a requirement rather than an option.

If the human declines §4's normalization *wholesale*, the fallback is to
keep §4's spine walk for statements whose **input** used an attached search
(preserving today's `e=^(NK)` for postfix inputs). That sacrifices the
round-trip property (§2 tree identity would no longer imply render
identity) and is not recommended, but it is available and would confine the
signing surface back to one baseline.

## FIR Impact

**None.** This is the central design property of this FOOP.

The rewrite of §2 is purely syntactic: `LHS =SEARCH_SPEC RHS` builds the
*same* AST as `LHS = RHS SEARCH_SPEC`, which builds the *same* FIR as it
does today. No new FIR variant, no new NYES state, no new transition.

Two **removals** fall out of §7, both of previously-reachable-but-defective
paths:

- The `"$"` arm of `OperatorFir` (`fir_kinds.rs:713`) becomes unreachable
  and is deleted.
- The synthetic `UnanchoredSeek { offset: -1 }` operand construction in
  `parse_assignment` is deleted.

Because no FIR kind is added and no NYES transition is added or changed, the
`*_nyes_transitions` test mandate (AGENTS.md §"NYES transition tests") adds
no *new* required test here. The existing `IndexFir` NYES test now covers
the `=$`/`=^` forms too, since they compile to `IndexFir`; the plan extends
its documentation accordingly.

## UBC Step Impact

**None.**

No new step rule. `=$`/`=^`/`=#N`/`=?x` all step exactly as the
corresponding postfix search steps today, because they *are* the same FIR.

The one observable evaluation change is a **bug fix that falls out of
deletion**, not a new rule: `=$` currently yields the whole brane and `=^`
currently does not settle; after §7 both route through `IndexFir` and settle
to the tail and head element respectively — which is what FOOP-54 §D.5
already specifies.

## Test Plan

### Parser unit tests (`foolish-parser/src/parser.rs` tests module)

- **Tree identity** — the core property. For each of `^ $ ~ ? # .` and for
  the chains of §3, assert that `parse("{...; A =SPEC B;}")` produces a tree
  **structurally equal** to `parse("{...; A = B SPEC;}")`. This is the
  strongest available statement of §2 and makes most per-operator tests
  unnecessary.
- **Adjacency (§5)** — `A =$ B` triggers; `A = $ B` does not. Assert the
  latter is either an ordinary assignment or a parse error, and pin which.
- **`&` is not a trigger (§5)** — `A =&?x B` does not take the suffix path.
- **§6 limitation, pinned** — `A =?x#1 B` yields one `RegexpSearch` with
  `pattern: "x#1"`, matching postfix `A = B?x#1`. A test comment must state
  that this pins a **known defect**, cite §6, and say what the fixed
  behavior would be, so a future FOOP finds it rather than "fixing" the test.

### Sequencer unit tests (`foolish-core/src/sequencer_tests.rs`)

- **Round-trip** — for each operator and chain: parse → evaluate → sequence,
  and assert the output re-parses to the same tree.
- **Normalization (§4)** — `A = B$` sequences to `A =$ B`.
- **Spine walking (§4)** — `A = B~=5#-2` sequences to `A =~=5#-2 B`.
- **Non-search fallback** — an ordinary `A = B` statement is unchanged.

### Value-correctness tests (`foolish-ubca`)

- `{b = {1,2,3}; y =$ b}` settles to `3` (FOOP-54 §D.5; currently `{1;2;3}`).
- `{b = {1,2,3}; y =^ b}` settles to `1` (currently `Op^(...)`).
- Non-brane anchor: `{d =$ 4}` settles NK per the anchored-miss rule (§8).

### einmo approval tests

- New: `foolish-ubca/einmo_suite/input/foop/75/` covering each operator in
  suffixed form, the §3 chains, and the §5 adjacency and `&` cases.

- **New, documentary:
  `foolish-ubca/einmo_suite/input/foop/75/search_operator_inside_patterns_howto.foo`**
  — a dedicated, heavily-commented test whose purpose is to **clarify §6**:
  what happens when a character that is both a search operator and a regexp
  metacharacter appears inside a pattern. This file is written as a
  teaching document first and a test second. It must cover, each with a
  comment stating the expected result *and why*:

  - `~.*asdf$` — bare, `$` absorbed as a regexp end-anchor (§6.1).
  - `~(.*asdf$)` — parenthesized, pattern is `.*asdf$`, no chain (§6.2).
  - `~(.*asdf)$` — parenthesized, then a **tail search chained** (§6.2).
  - `~(x)#-2` and `?(x)#1` — pattern then positional index (§6.2).
  - `~^abc` — `^` as a regexp start-anchor, contrasted with `^` as head.
  - `~(a|b)c` — the §6.3 meaning-change case, pinned explicitly.
  - The same set in **suffixed form** (`A =~(.*asdf$) B`), demonstrating §2
    tree identity: suffixed and postfix spellings produce identical output.

  Because this file's whole value is its explanatory comments, it is
  exempt from the usual preference for terse test inputs. It is the artifact
  a future Foolisher (or agent) is pointed at when they ask "can I put a `$`
  in a pattern?" — the answer must be readable from the file alone, without
  reading this FOOP.

  If §6.2 is deferred per §6.3, this file is still written, covering §6.1
  and §6.4 only, and its comments state plainly that the parenthetical
  terminator is **not yet available** and name the FOOP that would add it.
- Comprehensive: `foop_75_comprehensive.foo` (§Plan), mixing suffixed
  statements with existing features — nested branes, concatenation-based
  function application per FOOP-54 §D.5, creations, and searches inside
  suffixed RHS expressions.
- **Existing, must be re-reviewed, not promoted blindly**:
  `verified/regression/disappearing_brane_statements.foo.einmo` (§8) —
  frozen; requires a human signing decision.
  `input/foop/13/comprehensive.foo:40` contains `#-1=$=s` inside a **comment**
  and is therefore unaffected; the plan verifies this rather than assuming it.

### Cross-checks required before promotion

Per AGENTS.md §"The einmo review workflow" step 4, every changed OUTPUT line
must be justified in the agent's own words against the in-force spec. For
this FOOP the in-force authorities are FOOP-54 §D.5 (what `=$` means),
AGENTS.md §Searches (NK vs ECONSTANIC miss outcomes), and this FOOP §2/§4.

## §9. Relationship to FOOP-65 (tail concatenator) — shared structure

Atlas observed (2026-08-07) that FOOP-65 and FOOP-75 both "orient and attach
FIR components in different places than normal," and asked whether the two
have synergetic design needs. They do. This section records the analysis so
both FOOPs can be implemented without colliding, and so the shared
machinery is built once.

### §9.1 The shared shape

Both FOOPs are **source-order-to-tree-order permutations**: the surface
syntax writes a component in one position and the tree attaches it in
another. Neither introduces new evaluation semantics.

| | FOOP-65 (tail concatenator) | FOOP-75 (attached searches) |
|---|---|---|
| Surface | `A = fn`{a,b}` | `A =$ B` |
| Means | `A = {a,b} fn` | `A = B$` |
| Moved component | the method, to the tail | the search, to the anchor position |
| Compile-direction op | **reverse** the operand list (§5) | **replay** the suffix onto the RHS (§2) |
| Sequence-direction op | render through the inner concatenation | **walk the anchor spine** and lift (§4) |
| New FIR? | yes — `TailConcatenationFir` (a deliberate hook) | **no** — reuses `IndexFir`/`SearchFir` |

The deep commonality is the **sequencer obligation**: both need to
recognize a settled sub-tree, decide it came from a permuted surface form,
and render it back in that surface form. That is FOOP-75 §4's spine walk
and FOOP-65's "render through the inner concatenation."

### §9.2 Where they genuinely share machinery

**The spine/chain walk.** FOOP-75 §4 walks a left-nested `anchor` spine
(searches). FOOP-65's chain is *flat* by construction (`Astn::TailConcatenation
{ elements }`, n-ary, §4 of that FOOP) precisely so it needs no walk. So the
walk itself is **not** shared — FOOP-65 avoided needing it.

What *is* shared is the **structural-recognition predicate**: "is this FIR
node one that a surface form permuted?" FOOP-75 needs `is_search(node)`
(over `IndexFir` + the search family); FOOP-65 needs
`is_tail_concatenation(node)`. Both are one-node classifiers consulted by
the sequencer before rendering. FOOP-75 §4 already proposes `is_head()` /
`is_tail()` predicates over `hs_index()` to replace the repeated
`offset == -1 && anchored` magic-number pair. **Recommendation**: site
these as a small, named family of `FirQueryable` classifiers rather than
ad-hoc matches in `sequencer.rs`, so FOOP-65's arm joins the same family
instead of adding a fourth bespoke branch.

### §9.3 The real interaction: both edit `parse_expr`'s precedence tower

This is the concrete collision risk, and it is a **merge-order** problem,
not a design conflict.

- FOOP-65 §4 inserts a **new weakest level above** `parse_expr`
  (`parser.rs:371-388`): the current body becomes `parse_concat_level`, and
  a new `parse_expr` collects the backtick chain.
- FOOP-75 §2 modifies **`parse_assignment`** (`parser.rs:296-368`), which
  *calls* `parse_expr` for the RHS, and reuses the postfix suffix loop
  (`parser.rs:640-760`).

These touch adjacent but distinct functions. The interaction is that FOOP-75
must anchor its attached search on **the whole RHS**, and after FOOP-65 the
"whole RHS" includes a backtick chain. Both readings are defensible:

```foolish
A =$ fn`{a,b}      !! (i) $ applies to the whole application: (fn`{a,b})$
                   !! (ii) $ applies to the chain's last operand only
```

**§9.3 specifies reading (i)**: the attached search applies to the entire
RHS expression, whatever its internal structure. This follows directly from
§2 (`A =$ RHS` ≡ `A = RHS$`) and from FOOP-65 §2 making the backtick the
*weakest* operator — so the backtick chain *is* the whole RHS. Reading (ii)
would require the attached search to reach inside the chain, which §2 never
licenses.

Usefully, this makes the attached form the **ergonomic answer to FOOP-65's
own Open Question**. FOOP-65 notes that extracting an application result
needs parentheses today — `` (fn`X)$ `` — and defers whether a convenience
exists. Under §9.3 it does:

```foolish
A = (fn`{a,b})$        !! FOOP-65 today: parentheses required
A =$ fn`{a,b}          !! FOOP-75 §9.3: same tree, no parentheses
```

FOOP-65's Open Question says "nothing in this FOOP depends on it," so this
is an unplanned benefit, not a dependency either way.

### §9.4 Merge order and non-interference

**Neither FOOP blocks the other, and either may land first.**

- If **FOOP-65 lands first**: FOOP-75's §2 needs no change (it anchors on
  whatever `parse_expr` returns). The §9.3 case becomes testable and must
  be covered.
- If **FOOP-75 lands first**: FOOP-65's `parse_expr` refactor must preserve
  the attached-search replay, since `parse_assignment` calls `parse_expr`
  for the RHS *after* recording the suffix. The replay happens on the
  returned tree, so the refactor is transparent to it.

**Verified non-interference of the token sets**: FOOP-65 adds
`Token::Backtick`; FOOP-75's trigger set is `^ $ ~ ? # .` plus a
`preceded_by_space` flag on `TokenAndLocation`. No overlap. FOOP-75's
lexer change (§5.3) is a *field addition* to `TokenAndLocation`, which
FOOP-65's new token arm must populate — a one-line consideration, noted
here so whichever lands second does not miss it.

**Recommendation**: land **FOOP-75's §5.3 lexer change first regardless**,
since it is small, purely additive, and both FOOPs' parser work sits on top
of it. Beyond that, the order is free.

## Rejected Alternatives

### A. Do nothing

Leave `=$` yielding the whole brane, `=^` leaking `Op^(...)`, and the
reverse direction missing. Rejected: FOOP-54 §D.5 is a `Complete` FOOP that
specifies `a =$ b ≡ a = b$`, and the implementation does not do that. This
is a plain contradiction between a completed specification and the code, and
it is load-bearing — FOOP-54 §D.5 presents `=$` as *the* function-application
idiom in a language with no call syntax.

### B. Fix `=$` and `=^` in place, as two special cases

Add the missing `"^"` arm to `fir_kinds.rs`, correct the `"$"` arm to
extract the tail, and add a matching `^` branch to the sequencer. Rejected:
it leaves two hand-written operator paths that duplicate what `IndexFir`
already does correctly, keeps the two spellings structurally divergent (so
`B$` still never re-sugars), and does nothing for `~ ? # .`. It is strictly
more code than §2's rewrite, for strictly less coverage. It also preserves
the "two `$` constructs with different FIR kinds" trap that produced these
defects in the first place.

### C. Adopt `$=` (prefix spelling) instead of `=SEARCH_SPEC`

Match FOOP-23 §942/946's prose. Rejected: `$=` reads as "the `$` belongs to
the LHS", which is backwards — the search applies to the RHS. It also
generalizes badly: `~=5#-2=` is unreadable, and `?name=` collides
outright with the existing combined name-and-value form `?name=value`
(AGENTS.md §Searches: an atomic conjunctive operator). FOOP-54, the parser,
and the verified baseline all already use `=$`.

### D. Allow spaces in the suffix (`A = $ B`)

Rejected: it is ambiguous with an ordinary assignment whose RHS begins with
a search operator, and it contradicts the project-wide rule that search
specifications never contain spaces. The no-space rule is what makes the
trigger decidable with one token of lookahead.

### E. Make the sequencer emit the postfix form (`A = B$`) instead

The reverse of §4's normalization choice. Rejected: it would make `=$`
input render as `= B$`, so the very idiom FOOP-54 §D.5 teaches would never
appear in any snapshot, and the existing verified baseline
(`d =$ ???`) already renders the suffixed form. Normalizing toward the
suffix preserves that baseline's shape and keeps the taught idiom visible.

## Open Questions

- **Should `&` ever be admissible in a suffix (§5)?** Deferred. It needs a
  decision on what statement position a contexted search reads when its
  anchor is an expression rather than a found statement. Nothing in this
  FOOP depends on the answer.
- **Does §8's verified baseline text change?** To be determined
  empirically during implementation, then put to the human reviewer. The
  plan gates on this; the FOOP does not presume the outcome.
- **Should §6 (regexp pattern greediness) get its own FOOP?** Recommended,
  but out of scope here. This FOOP only pins the current behavior.

## References

- Prior FOOPs: **FOOP-54 §D.5** (`Complete` — defines bind-tail `a =$ b ≡ a
  = b$` and the emergent function-application idiom; the in-force authority
  for what `=$` means); FOOP-23 §942/946 (the `a$=b`/`a^=b` transposition
  corrected by §7) and §Terminology (the three search operator groups);
  FOOP-55 §D6/§E5 (independent discovery that `$=` does not parse, and the
  `(X)$` workaround); FOOP-65 (postfix `$` after backtick application —
  uses only the postfix form, unaffected by this FOOP).
- Docs: `AGENTS.md` §Searches (operator tables, NK vs ECONSTANIC miss
  outcomes, the one-engine model); `docs/vintage_legacy/NAMES_SEARCHES_N_BOUNDS.md`;
  `rust_instructions.md` §"Phase-by-phase testing discipline".
- Code anchors: `foolish-parser/src/parser.rs` — `parse_assignment` 296-368
  (the two bespoke branches at 326-354 deleted by §7), postfix suffix loop
  640-760 (reused by §2), `parse_regexp_pattern` 800-814 (§6);
  `foolish-parser/src/ast.rs` — `Astn::HeadTail` 93, `Seek`, `RegexpSearch`,
  `ValueSearch`; `foolish-ubca/src/compiler.rs:354` (`HeadTail` → `IndexFir`);
  `foolish-ubca/src/fir_kinds.rs:713` (the `"$"` arm deleted by §7),
  `IndexFir` 1709-1714; `foolish-core/src/sequencer.rs:650-700` (the `=$`
  sugar branch, generalized by §4); `foolish-core/src/fir.rs:561`
  (`IndexQuery`), 576-578 (`hs_operator` / `hs_index`).

## Last Updated

**Date**: 2026-08-07 (2)
**Updated By**: Claude Code / claude-opus-5
**Changes**: Revised after the plan's Phase 1 survey. **§8 widened from one
frozen baseline to six** — §4's normalization affects postfix inputs too, so
any baseline rendering a `$`/`^` statement moves (`misc/head_tail_empty_brane`,
`misc/anchored_search_on_constanic`, `misc/offset_access_empty_brane`,
`foop/33/boolean/comparison_non_integer`, `foop/42/…hfs`, plus the originally
identified `regression/disappearing_brane_statements`); documented the two
distinct existing renderings (`e=^(NK)` vs `d =$ ???`) that §4 unifies, and
added a declinable fallback. **§Motivation gained "The corpus already assumes
this rule"** — three `test-resources/` files state §2's rewrite verbatim in
comments, one already contains an attached chain (`c =$#-1;`), and
`regexSearchShadowy.foo` annotates chained searches with expected results that
require both §5's space rule and §6.2's parens, none of which work today.
Earlier: initial draft. Generalizes `LHS = RHS` to `LHS =SEARCH_SPEC RHS`
≡ `LHS = RHS SEARCH_SPEC` over the trigger set `^ $ ~ ? # .`, with the
sequencer obligation to lift a statement body's whole search spine back to
the suffix position. Documents three verified defects in the existing
`=$`/`=^` special cases and dissolves them by routing both through the
existing `IndexFir` path (§7). Records the `parse_regexp_pattern`
greediness limitation (§6) as pinned-not-fixed, and the frozen verified
baseline at §8 as requiring a human signing decision.
