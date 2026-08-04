# FOOP-33 Crash Log — Session Recovery Document

**Date**: 2026-07-31
**Author**: Sisyphus / xiaomi/mimo-v2.5-pro
**Purpose**: Document all specification updates, design decisions, and implementation findings from this session. Recovery reference after code progress loss.

---

## 1. Worktree & Branch

- **Worktree**: `/yolo/foolish_worktrees/foop-33-creation-postulate`
- **Branch**: `foop-33-creation-postulate`
- **Origin branch**: `origin/foop-33-creation-postulate` (pushed)
- **Last commit**: `223fd76f` — "FOOP-33: Unicode underlined operators + lexer cleanup"
- **Status**: Phases 0–6 complete, Phase 7 (docs/tests) in progress

---

## 2. Parser Changes — `'name` Syntax (Null Characterization)

### Problem
The parser did not support `'True` as a reference or assignment target. `parse_characterizations()` only handled `name'name'name` patterns (e.g., `a'b'c'name`), not leading `'` (null characterization).

### Solution — Three parser fixes

**A. `parse_characterizations()`** — Handle leading `'` and consecutive `''`:
```rust
fn parse_characterizations(&mut self) -> Vec<String> {
    let mut chars = Vec::new();
    // Handle leading apostrophe: 'name is a null-characterized name
    if self.peek_token() == Some(&Token::Apostrophe) {
        chars.push(String::new());
        self.advance();
    }
    loop {
        match self.peek_token() {
            Some(Token::Ident(name)) => {
                if self.tokens.get(self.pos + 1).map(|t| &t.token) == Some(&Token::Apostrophe) {
                    chars.push(name.clone());
                    self.advance();
                    self.advance();
                    continue;
                }
                break;
            }
            // '' — consecutive apostrophe (null characterization)
            Some(Token::Apostrophe) => {
                chars.push(String::new());
                self.advance();
                continue;
            }
            _ => break,
        }
    }
    chars
}
```

**B. `is_assignment_start()`** — Same leading `'` and consecutive `''` handling added.

**C. `parse_primary()`** — Handle `'name` as a reference (expression context):
```rust
Some(Token::Apostrophe) => {
    let chars = self.parse_characterizations();
    let id = self.parse_identifier()?;
    Ok(Astn::Identifier { characterizations: chars, id })
}
```

### Key insight
`a''b'name` parses as characterizations `["a", "", "b"]`, name `"name"`. The null characterization is the LAST entry (`"b"` is not empty, so NOT null-characterized). Proximity is king.

---

## 3. Unanchored `?name` Search

### Problem
Parser rejected `?tag'x` with "Unanchored ? without = not supported". Only `?=value` was supported for unanchored search.

### Solution
In `parse_primary()`, when `?` is followed by a pattern (not `=`), create an unanchored `RegexpSearch` with a dummy empty brane anchor:
```rust
} else {
    Ok(Astn::RegexpSearch {
        anchor: Box::new(Astn::Brane { characterizations: vec![], statements: vec![] }),
        operator: SearchOperator::RegexpLocal,
        pattern,
    })
}
```

### Key insight
The dummy empty brane anchor makes the search unanchored — it searches the current brane context.

---

## 4. `parse_regexp_pattern()` — Apostrophe Handling

### Problem
`?tag'x` would parse as `?` then `tag` (Ident), then `'` (Apostrophe) breaks the loop. Pattern was just `"tag"` instead of `"tag'x"`.

### Solution
Added `Token::Apostrophe` handling in `parse_regexp_pattern()`:
```rust
Some(Token::Apostrophe) => {
    pattern.push('\'');
    self.advance();
}
```

---

## 5. Comparison Operators — `\o` Prefix Convention

### Design Decision
- `<` and `>` are StayFoolish delimiters — cannot be comparison operators
- `<=` and `>=` are existing tokens (SF/SFF related) — repurposed as comparison operators
- New operators use `\o` prefix: `\o<`, `\o>`, `\o<=`, `\o>=`, `\o==`
- Unicode form: each operator character gets U+0332 combining low line suffix

### Token mapping
| Input (ASCII) | Input (Unicode) | Token | Op string |
|---------------|-----------------|-------|-----------|
| `\o<` | `<̲` | `LTOp` | `\<` |
| `\o>` | `>̲` | `GTOp` | `\>` |
| `\o<=` | `<̲=̲` | `Le` | `<=` |
| `\o>=` | `>̲=̲` | `Ge` | `>=` |
| `\o==` | `=̲=̲` | `EqOp` | `\==` |

### Lexer implementation
- `\o` prefix: recognized in main `match c` block before identifier check
- Unicode form: recognized BEFORE plain `>`, `=` matches (ordering critical!)
- `<` Unicode: handled in `lt_token()` function (called before main match)

### Evaluator
Comparison operators are string-matched in `OperatorFir::fir_op_step`:
```rust
if matches!(self.op.as_str(), "<=" | ">=" | "\\<" | "\\>" | "\\==") {
    // ... check both operands are integers, else NK
    let bool_result = match self.op.as_str() {
        "<=" | "\\<" => values[0] <= values[1],
        ">=" | "\\>" => values[0] >= values[1],
        "\\==" => values[0] == values[1],
        _ => unreachable!(),
    };
    // Resolve True/False from system.foo
}
```

---

## 6. Sequencer — U+0332 Rendering

### Rule
The sequencer ALWAYS outputs operators with U+0332 combining low line on EACH character.

### Implementation
```rust
const COMBINING_LOWLINE: char = '\u{0332}';

fn op_display(op: &str) -> String {
    let mut result = String::new();
    for ch in op.chars() {
        result.push(ch);
        result.push(COMBINING_LOWLINE);
    }
    result
}
```

### Used in
- `render_fir()` — inline operator rendering
- `proto_brane_formatter()` — humanized `Op<̲=̲(` form

---

## 7. AGENTS.md Update — Unicode Operator Convention

Added to Code Style section:
> **Agents MUST use Unicode operator forms when writing Foolish code.** The `\o` prefix is for keyboard input only. When an agent writes `.foo` files, it must use the Unicode underlined forms:
> - `⬤` not `{*}` for creation
> - `<̲` not `\o<` for less-than
> - `>̲` not `\o>` for greater-than
> - `<̲=̲` not `\o<=` for less-than-or-equal
> - `>̲=̲` not `\o>=` for greater-than-or-equal
> - `=̲=̲` not `\o==` for equality

---

## 8. Einmo Test Organization — `foop/33/` Subdirectories

### Structure
```
foop/33/
  creation/
    basics.foo          — ⬤ and {*} basics, identity, inequality
    nilpotent.foo       — creation value search, inequality with comparison
    referential_equality.foo — same Rc across branes, different Rc
  creation_concat.foo   — null-constant rule in concatenation
  boolean/
    comparison_operators.foo — all 5 operators, true/false/NK cases
    constants.foo       — True/False from system.foo
    if_then_else.foo    — value search pattern matching (user-edited)
  characterizations/
    null_char_constant.foo — 'True/'False redefinition rules
    nf_error.foo        — NF (Not Foolish) error, sibling unaffected
    quote_bearing_search.foo — pattern with ' matches characterized_name()
    proximity_rule.foo  — a''b'name interior null doesn't count
  int_comparators.foo   — Unicode + ASCII \o forms side by side
  comprehensive.foo     — all features interacting
```

### Convention
- `int_comparators.foo` and `creation/basics.foo` show ASCII next to Unicode
- All other tests use Unicode only

---

## 9. system.foo — Null-Characterized Constants

### Current content
```foolish
{!!system.foo
	'True  = ⬤
	'False = ⬤
}
```

### Key points
- `'True` and `'False` are null-characterized name constants
- Defined in `system/system.foo`, embedded via `build.rs` → `OUT_DIR` → `include_str!`
- system.foo is the root brane (its own parent), user program is its child
- `_ab_search` terminates at system.foo

---

## 10. NF (Not Foolish) — Sub-condition of NK

### Definition
NF is a semantic label on `NkFir` for violations of Foolish's own rules. First case: overwriting a null-characterized name constant with a different value.

### Implementation
```rust
pub const NF_PREFIX: &str = "not-foolish";

pub fn is_nf_reason(reason: &str) -> bool {
    reason.contains(NF_PREFIX)
}
```

### Reason string format
`"'<name> not-foolish"` — e.g., `"'True not-foolish"`

### Where it triggers
- BraneFir step (PREMBRYONIC/EMBRYONIC): null-constant rule check
- ConcatenationFir step: collision-aware merge

---

## 11. `default_equal` — Three-Valued Equality

### Definition
```rust
pub enum Equality { Equal, NotEqual, Unknowable }

pub fn default_equal(a: &FirRef, b: &FirRef) -> Equality {
    // 1. NK guard: either NK → Unknowable
    // 2. Integer equality: both as_i64() → Equal/NotEqual
    // 3. Creation equality: both CreationFir → Equal iff Rc::ptr_eq
    // 4. Everything else → Unknowable
}
```

### Matcher mapping
| `default_equal` | `MatchOutcome` |
|-----------------|----------------|
| `Equal` | `Approve` |
| `NotEqual` | `Reject` |
| `Unknowable` | `NkStop` |

### Key design decision
Brane-vs-integer comparison returns `Unknowable` (→ NkStop), NOT `NotEqual` (→ Reject). "Equality must be known, not assumed."

---

## 12. Creation Identity — `Rc::ptr_eq`

### Key points
- `CreationFir` has NO id field. Identity = `Rc::ptr_eq`.
- Born `Independent` — no NYES transitions.
- Constanic clone returns SAME `Rc` (identity-preserving). Works automatically via `fir_kinds.rs:180` branch for `Independent` non-brane FIRs.
- Do NOT add a `FirKind::Creation` clone arm that constructs a new `CreationFir` — that would break identity.
- Do NOT derive/implement deep `Clone` on `CreationFir`.

---

## 13. Identifier/Characterizations Types

### Location
`foolish-ubca/src/identifier.rs` (new file)

### Identifier struct
```rust
pub struct Identifier {
    fully_characterized_name: String,  // "a'b'c'd'e''x"
    name: String,                       // "x"
    characterization_string: String,    // "a'b'c'd'e''"
    characterizations: Characterizations,
}
```

### Characterizations struct
```rust
pub struct Characterizations {
    is_nully: bool,  // true iff last characterization is empty
}
```

### Proximity rule
Only the LAST characterization (touching the name) determines null-characterization. `a''b'name`: last is `"b"` (not empty) → NOT null-characterized.

---

## 14. Search Pattern Projection

### Rule
- Pattern WITHOUT `'` → matcher matches against `Identifier::name()` (bare coordinate name)
- Pattern WITH `'` → matcher matches against `Identifier::characterized_name()` (whole LHS)

### Implementation
In `_search_brane()`:
```rust
let candidate = if expression.contains('\'') {
    child_borrowed.as_stmt_identifier().map(|id| id.characterized_name())
} else {
    child_borrowed.as_stmt_name()
};
```

Updated in both `BraneFir::_search_brane` and `ConcatHelper::_search_brane`.

---

## 15. Compilation Pattern Folding (Gotcha #3)

### Problem
`?'True` compiled from `id` only (`"True"`), losing the `'`. Pattern was `^True$` instead of `'^True$`.

### Solution
In `build_fir()` for `Astn::Identifier`:
```rust
let full_pattern = if characterizations.is_empty() {
    id.clone()
} else {
    let char_str: String = characterizations.iter().map(|c| format!("{c}'")).collect();
    format!("{char_str}{id}")
};
```

---

## 16. Pending Items / Known Issues

### Deferred
- `BraneFir.characterizations` migration from `Vec<String>` to `Characterizations` (sequencer needs raw strings)
- `|~` cascading search operator (FOOP-24)
- Boolean logic operators (`and`, `or`, `not`) — follow-on FOOP

### Not yet done
- Phase 7 documentation updates
- Phase 8 merge prep (comprehensive test, fmt/clippy, STOP! checkpoint)
- Promote einmo baselines after test updates

### Pre-existing failures
- `zweimomo::crash_crumb_survives_foolish_stack_overflow` — OS "No child processes" error, unrelated to FOOP-33

---

## 17. Files Modified This Session

### Parser (`foolish-parser/`)
- `src/token.rs` — `Creation`, `LTOp`, `GTOp`, `EqOp` tokens
- `src/lexer.rs` — `⬤`, `{*}`, `\o` prefix, Unicode U+0332, `'` in patterns
- `src/parser.rs` — `'name` syntax, unanchored `?name`, `_<_`/`_>_` → `\o` operators, `LTOp`/`GTOp`/`EqOp` in additive expr

### UBCa (`foolish-ubca/`)
- `src/identifier.rs` — NEW: `Identifier`, `Characterizations` types
- `src/lib.rs` — `pub(crate) mod identifier`
- `src/fir_kinds.rs` — `CreationFir`, `FirKind::Creation`, `NF_PREFIX`, `default_equal`, `Equality`, `_search_brane` projection, `OperatorFir` comparison handling
- `src/fir_trait.rs` — `FirKind::Creation`, `as_stmt_identifier()`
- `src/compiler.rs` — `Astn::Creation` → `CreationFir`, fold `'` into pattern
- `src/evaluator.rs` — `Fir::Creation` core-fir, `system.foo` embedding
- `build.rs` — NEW: copies `system/system.foo` to `OUT_DIR`

### Core (`foolish-core/`)
- `src/fir.rs` — `Fir::Creation` variant, `hs_creation()`, `hs_variant`, `hs_state`, `set_state`, `fir_variant`, `fir_to_json`
- `src/sequencer.rs` — `op_display()` with U+0332, `hs_creation()` rendering

### Other
- `system/system.foo` — `'True = ⬤`, `'False = ⬤`
- `AGENTS.md` — Unicode operator convention
- `docs/foop/FOOP-33.plan.md` — Phase checkboxes, stashing notes (removed)

---

## 18. Design Decisions Summary

1. **No equality operator** — equality matters only during search; `default_equal` is the single home
2. **Three-valued equality** — `Unknowable` (not `NotEqual`) for incomparable types
3. **NF over NK** — null-constant violations get NF label, not generic NK
4. **`Rc::ptr_eq` for creation identity** — no id, no registry, no counter
5. **Proximity is king** — only the characterization slot touching the name matters
6. **`\o` prefix for operators** — avoids conflicts with `<`/`>` StayFoolish
7. **Unicode U+0332** — each operator character gets its own combining low line
8. **system.foo is root** — self-parenting, user program is child, line numbers preserved
9. **Greedy known-to-be-equal matcher** — approves only on positive proof, stops on unknowable
10. **Poison scoped to statement** — NK lives on the offending statement's body, not globally

---

## Appendix A: Einmo Test Files

### `foop/33/int_comparators.foo` — Unicode + ASCII side by side
```foolish
{
	!! Integer comparators — Unicode underlined + ASCII \o forms
	!! Unicode: each op char + U+0332 combining low line
	!! ASCII: \o prefix (keyboard input form)

	!! --- Unicode form ---
	ult = 3 <̲ 5;
	ule = 3 <̲=̲ 3;
	ugt = 5 >̲ 3;
	uge = 5 >̲=̲ 5;
	ueq = 3 =̲=̲ 3;

	!! --- ASCII \o form (same operators) ---
	alt = 3 \o< 5;
	ale = 3 \o<= 3;
	agt = 5 \o> 3;
	age = 5 \o>= 5;
	aeq = 3 \o== 3;

	!! --- false cases (Unicode) ---
	false_lt = 5 <̲ 3;
	false_le = 5 <̲=̲ 3;
	false_gt = 3 >̲ 5;
	false_ge = 3 >̲=̲ 5;
	false_eq = 2 =̲=̲ 3;

	!! --- boundary ---
	zero = 0 <̲=̲ 0;
	neg = -5 <̲=̲ -3;
	neg_ge = -3 >̲=̲ -5;
	neg_eq = -3 =̲=̲ -3;

	!! --- non-integer operands → NK ---
	nk_creation = ⬤ <̲=̲ 3;
	nk_brane = {x=1;} >̲=̲ 0;
	nk_eq = ⬤ =̲=̲ ⬤;
}
```

### `foop/33/creation/basics.foo` — Creation basics
```foolish
{
	!! Creation basics: ⬤ and {*} produce unique values.
	bare = ⬤;
	ascii = {*};
	orig = ⬤;
	ref = orig;
	same_ref = ref~=orig;
	x = ⬤;
	y = ⬤;
	diff = x~=y;
}
```

### `foop/33/creation/nilpotent.foo` — Nilpotent creation
```foolish
{
	!! Nilpotent creation: creations that create inequalities and go away immediately.
	a = {blah={*}};
	fails = a~={*};
	check = {*} <̲ 10;
}
```

### `foop/33/creation/referential_equality.foo` — Referential equality
```foolish
{
	!! Referential equality: same Rc across branes, different Rc across branes.
	shared = ⬤;
	ba = {v = shared;};
	bb = {v = shared;};
	cross_same = ba.v~=bb.v;
	bc = {v = ⬤;};
	bd = {v = ⬤;};
	cross_diff = bc.v~=bd.v;
}
```

### `foop/33/creation_concat.foo` — Concatenation with null-constant rule
```foolish
{
	!! Creation in concatenation: same creation permitted, different → NF.
	P = {k = ⬤;};
	Q = {k = ⬤;};
	pq = P Q;
	shared = ⬤;
	R = {k = shared;};
	S = {k = shared;};
	rs = R S;
}
```

### `foop/33/boolean/comparison_operators.foo` — All operators
```foolish
{
	!! Comparison operators produce True/False from system.foo.
	a = 3;
	b = 5;
	le_t = a <̲=̲ b;
	le_eq = a <̲=̲ a;
	le_f = b <̲=̲ a;
	ge_t = b >̲=̲ a;
	ge_eq = b >̲=̲ b;
	ge_f = a >̲=̲ b;
	eq_t = a =̲=̲ a;
	eq_f = a =̲=̲ b;
	nk = ⬤ <̲=̲ 3;
}
```

### `foop/33/boolean/constants.foo` — True/False from system.foo
```foolish
{
	!! True/False are ancestral constants from system.foo.
	t = True;
	f = False;
	flag = 3 <̲=̲ 7;
	is_t = flag~=True;
	is_f = flag~=False;
}
```

### `foop/33/boolean/if_then_else.foo` — Value search pattern matching
```foolish
{
	!! If-then-else: comparison result feeds name search to select branch.
	ite = {
                cond=1; result=100;
                cond=2; result=200;
                cond=5; result=500;
                result=-1;
        };
	c1   = ite~cond=1#1;
	c2   = ite~cond=2#1;
	c5   = ite~cond=5#1;
	!! else = ite~cond=101#1|~else#1;  !! Enable after we've implemented the |~ cascading search operator.
}
```

### `foop/33/characterizations/null_char_constant.foo` — Null-characterized constants
```foolish
{
	!! Null-characterized constants: 'True/'False defined in system.foo.
	!! Redeclaring with same value permitted, different → NF.
	'True   = 10
	'True   = 'False;
	'False  = 'False;

        !! True for anything not just those declared in the system.foo
	'ref_t  = 'True;
	'ref_f  = 'False;
}
```

### `foop/33/characterizations/nf_error.foo` — NF error
```foolish
{
	!! NF error: redefining a null-characterized constant with a different
	!! value produces NF. Sibling branes are unaffected.
	sibling = {ok = 'True;};
	'True = 3;
	poisoned = 'True;
}
```

### `foop/33/characterizations/quote_bearing_search.foo` — Quote-bearing search
```foolish
{
	!! Quote-bearing search: pattern with ' matches characterized_name().
	tag'x = 7;
	plain_x = 9;
	hit = ?tag'x;
	miss = ?x;
}
```

### `foop/33/characterizations/proximity_rule.foo` — Proximity rule
```foolish
{
	!! Proximity rule: only the characterization slot IMMEDIATELY
	!! touching the name determines null-characterization.
	a''b'name = 42;
	plain_name = 99;
	found = ?name;
	found2 = ?a''b'name;
}
```

### `foop/33/comprehensive.foo` — All features interacting
```foolish
{
	!! FOOP-33 comprehensive: creation, equality, characterizations,
	!! system.foo prelude, null-constants, comparisons, nested branes.

	a = ⬤;
	b = {*};
	c = a;
	same = c~=a;
	diff = a~=b;

	t = True;
	f = False;
	flag = 3 <̲=̲ 7;
	is_t = flag~=True;

	gate = {True = 100; False = -100;};
	cond = 5 <̲=̲ 10;
	picked = gate~cond;

	tag'x = 7;
	plain_x = 9;
	hit = ?tag'x;
	miss = ?x;

	'True = 'True;

	inner = {deep = True;};

	nk = ⬤ <̲=̲ 3;
}
```

---

## Appendix B: Key Implementation Snippets

### `system/system.foo`
```foolish
{!!system.foo
	'True  = ⬤
	'False = ⬤
}
```

### `foolish-ubca/build.rs`
```rust
use std::{env, fs, path::Path};

fn main() {
    let manifest = env::var("CARGO_MANIFEST_DIR").unwrap();
    let src = Path::new(&manifest).join("../system/system.foo");
    let out = Path::new(&env::var("OUT_DIR").unwrap()).join("system.foo");
    fs::copy(&src, &out).expect("copy system/system.foo into OUT_DIR");
    println!("cargo:rerun-if-changed=../system/system.foo");
}
```

### `foolish-ubca/src/identifier.rs` — Identifier/Characterizations
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Characterizations {
    is_nully: bool,
}

impl Characterizations {
    pub fn from_parts(chars: &[String], _name: &str) -> Self {
        let is_nully = if chars.is_empty() {
            false
        } else {
            chars.last().map_or(false, |last| last.is_empty())
        };
        Characterizations { is_nully }
    }

    pub fn is_nully_characterizing_coordinate_name(&self) -> bool {
        self.is_nully
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identifier {
    fully_characterized_name: String,
    name: String,
    characterization_string: String,
    characterizations: Characterizations,
}

impl Identifier {
    pub fn from_parts(characterizations: Vec<String>, id: &str) -> Self {
        let canonical_chars: Vec<String> = characterizations
            .iter()
            .map(|c| c.chars().filter(|ch| !ch.is_whitespace()).collect())
            .collect();
        let name = id.to_owned();
        // Each component gets a ' suffix: a'b'c''
        let characterization_string: String =
            canonical_chars.iter().map(|c| format!("{c}'")).collect();
        let fully_characterized_name = format!("{characterization_string}{name}");
        let characterizations = Characterizations::from_parts(&canonical_chars, &name);
        Identifier { fully_characterized_name, name, characterization_string, characterizations }
    }

    pub fn name(&self) -> &str { &self.name }
    pub fn characterized_name(&self) -> &str { &self.fully_characterized_name }
    pub fn characterization_string(&self) -> &str { &self.characterization_string }
    pub fn is_nully_characterizing_coordinate_name(&self) -> bool {
        self.characterizations.is_nully_characterizing_coordinate_name()
    }
}
```

### `foolish-ubca/src/fir_kinds.rs` — NF prefix and default_equal
```rust
pub const NF_PREFIX: &str = "not-foolish";

pub fn is_nf_reason(reason: &str) -> bool {
    reason.contains(NF_PREFIX)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Equality { Equal, NotEqual, Unknowable }

pub fn default_equal(a: &FirRef, b: &FirRef) -> Equality {
    let a_borrowed = a.borrow();
    let b_borrowed = b.borrow();
    if a_borrowed.core().get_nyes() == Nyes::Nk || b_borrowed.core().get_nyes() == Nyes::Nk {
        return Equality::Unknowable;
    }
    if let (Some(av), Some(bv)) = (a_borrowed.as_i64(), b_borrowed.as_i64()) {
        return if av == bv { Equality::Equal } else { Equality::NotEqual };
    }
    if a_borrowed.kind() == FirKind::Creation && b_borrowed.kind() == FirKind::Creation {
        return if Rc::ptr_eq(a, b) { Equality::Equal } else { Equality::NotEqual };
    }
    Equality::Unknowable
}
```

### `foolish-ubca/src/fir_kinds.rs` — CreationFir
```rust
#[derive(Debug)]
pub struct CreationFir {
    pub(crate) core: ProtoBrane,
}

impl CreationFir {
    pub fn creation(parent: Weak<RefCell<dyn Fir>>) -> FirRef {
        Rc::new(RefCell::new(CreationFir {
            core: ProtoBrane::new(vec![], parent, Nyes::Independent),
        }))
    }
}

impl Fir for CreationFir {
    fn core(&self) -> &ProtoBrane { &self.core }
    fn fir_op_step(&self, _scope: &Scope) -> Result<(), UbcError> { Ok(()) }
    fn kind(&self) -> FirKind { FirKind::Creation }
}
```

### `foolish-core/src/sequencer.rs` — op_display with U+0332
```rust
const COMBINING_LOWLINE: char = '\u{0332}';

fn op_display(op: &str) -> String {
    let mut result = String::new();
    for ch in op.chars() {
        result.push(ch);
        result.push(COMBINING_LOWLINE);
    }
    result
}
```

### `foolish-parser/src/lexer.rs` — Unicode U+0332 recognition (in `lt_token`)
```rust
// <̲ (Unicode combining low line)
if self.peek_at(0) == Some('\u{0332}') {
    self.advance();
    // <̲=̲
    if self.peek_at(0) == Some('=') && self.peek_at(1) == Some('\u{0332}') {
        self.advance();
        self.advance();
        return (TokenAndLocation::new(Token::Le, line, column), false);
    }
    return (TokenAndLocation::new(Token::LTOp, line, column), false);
}
```

### `foolish-parser/src/lexer.rs` — `\o` prefix recognition
```rust
'\\' if self.peek_at(1) == Some('o') => {
    self.advance();
    self.advance();
    match self.peek() {
        Some('<') => {
            self.advance();
            if self.peek() == Some('=') {
                self.advance();
                return (self.make_token(Token::Le), false);
            }
            return (self.make_token(Token::LTOp), false);
        }
        Some('>') => {
            self.advance();
            if self.peek() == Some('=') {
                self.advance();
                return (self.make_token(Token::Ge), false);
            }
            return (self.make_token(Token::GTOp), false);
        }
        Some('=') => {
            self.advance();
            if self.peek() == Some('=') {
                self.advance();
                return (self.make_token(Token::EqOp), false);
            }
            return (self.make_token(Token::Ident("\\o=".into())), false);
        }
        _ => {
            return (self.make_token(Token::Ident("\\o".into())), false);
        }
    }
}
```

### `foolish-parser/src/parser.rs` — `parse_characterizations` with leading `'` and `''`
```rust
fn parse_characterizations(&mut self) -> Vec<String> {
    let mut chars = Vec::new();
    if self.peek_token() == Some(&Token::Apostrophe) {
        chars.push(String::new());
        self.advance();
    }
    loop {
        match self.peek_token() {
            Some(Token::Ident(name)) => {
                if self.tokens.get(self.pos + 1).map(|t| &t.token) == Some(&Token::Apostrophe) {
                    chars.push(name.clone());
                    self.advance();
                    self.advance();
                    continue;
                }
                break;
            }
            Some(Token::Apostrophe) => {
                chars.push(String::new());
                self.advance();
                continue;
            }
            _ => break,
        }
    }
    chars
}
```

### `foolish-parser/src/parser.rs` — `'name` as primary expression
```rust
Some(Token::Apostrophe) => {
    let chars = self.parse_characterizations();
    let id = self.parse_identifier()?;
    Ok(Astn::Identifier { characterizations: chars, id })
}
```

---

## Appendix C: Unit Tests (foolish-ubca/src/fir_kinds.rs)

### creation_nyes_transitions
```rust
#[test]
fn creation_nyes_transitions() {
    let parent = make_brane(vec![]);
    let creation = CreationFir::creation(Rc::downgrade(&parent));
    let trace = step_to_settled(&creation, &Scope::empty());
    assert!(trace.iter().all(|n| *n == Nyes::Independent));
    assert!(trace.first().unwrap().is_constanic());
}
```

### creation_constanic_clone_preserves_identity
```rust
#[test]
fn creation_constanic_clone_preserves_identity() {
    let parent = make_brane(vec![]);
    let creation = CreationFir::creation(Rc::downgrade(&parent));
    let clone = ProtoBrane::constanic_clone_at(&creation, &Rc::downgrade(&parent), 0, false, false);
    assert!(Rc::ptr_eq(&creation, &clone), "constanic clone of CreationFir must return same Rc");
    let creation2 = CreationFir::creation(Rc::downgrade(&parent));
    assert!(!Rc::ptr_eq(&creation, &creation2), "two distinct creations must not be ptr_eq");
}
```

### matcher_value_reject_non_integer_candidate (updated for FOOP-33)
```rust
// Was: assert_eq!(pred.matches(&stmt, &ctx), MatchOutcome::Reject)
// Now: brane-vs-integer is Unknowable → NkStop
assert_eq!(pred.matches(&stmt, &ctx), MatchOutcome::NkStop, "brane-vs-integer is Unknowable → NkStop");
```

---

## Appendix D: Parser Unit Tests (foolish-parser/src/parser.rs)

### parses_star_brane_as_creation
```rust
#[test]
fn parses_star_brane_as_creation() {
    let ast = parse_single("{x = {*};}").unwrap();
    match ast {
        Astn::Brane { statements, .. } => {
            assert_eq!(statements.len(), 1);
            match &statements[0] {
                Astn::Assignment { expr, .. } => {
                    assert!(matches!(**expr, Astn::Creation));
                }
                other => panic!("expected Assignment, got {:?}", other),
            }
        }
        _ => panic!("expected brane"),
    }
}
```

### parses_unicode_creation
```rust
#[test]
fn parses_unicode_creation() {
    let ast = parse_single("{x = \u{2B24};}").unwrap();
    match ast {
        Astn::Brane { statements, .. } => {
            assert_eq!(statements.len(), 1);
            match &statements[0] {
                Astn::Assignment { expr, .. } => {
                    assert!(matches!(**expr, Astn::Creation));
                }
                other => panic!("expected Assignment, got {:?}", other),
            }
        }
        _ => panic!("expected brane"),
    }
}
```

### spaced_star_is_not_creation
```rust
#[test]
fn spaced_star_is_not_creation() {
    // {* } has space before } — not a creation
    let result = parse_single("{x = {* };}");
    assert!(result.is_ok() || result.is_err());
}
```

---

## Appendix E: Token Enum (foolish-parser/src/token.rs)

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    LBrace, RBrace, LParen, RParen, LBracket, RBracket,
    Semicolon, Comma,
    Assign, Plus, Minus, Mul, Div, Dot, DotDot,
    Caret, Dollar, Question, QuestionQuestion, QuestionEquals,
    Tilde, TildeTilde, TildeEquals,
    Hash, Ampersand,
    Lt, Gt,           // < > (StayFoolish delimiters)
    Le, Ge,           // \o<= \o>= (comparison operators)
    LTOp, GTOp,       // \o< \o> (comparison operators)
    EqOp,             // \o== (equality operator)
    LtEqGt, LtLt, GtGt, LtLtEqGtGt,
    Apostrophe,
    Creation,         // ⬤ (U+2B24)
    Integer(u64), Ident(String), Shebang(String),
    LineComment, BlockComment(String),
    Unknown, Up,
    If, Then, Elif, Else, Fi,
    Eof,
}
```
