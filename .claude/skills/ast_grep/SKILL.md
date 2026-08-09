---
name: ast_grep
description: "MUST USE for structural code search and refactoring with the `ast_grep_engine` tool — finding call sites, signatures, and code shapes by AST pattern rather than text, and rewriting them safely across a crate. Covers metavariables ($X, $$$ARGS, $_), the three actions (search, rewrite, scan), relational YAML rules (inside, has, precedes, follows), language keys, and scoping a query to one crate. Prefer this over grep/ripgrep whenever the target is a code *structure* rather than a literal string. Triggers: 'ast-grep', 'ast grep', 'structural search', 'find all calls to', 'find every function that', 'refactor all', 'rewrite all occurrences', 'codemod', 'find call sites', 'match code pattern', 'lint rule', 'where is this pattern used'."
---

# Structural Search & Refactoring with ast_grep_engine

`ast_grep_engine` matches against the parsed syntax tree, not against text. A pattern
like `$X.unwrap()` finds every unwrap call regardless of receiver, whitespace, or line
breaks — things a regex either misses or over-matches.

---

## 0. Use a sub-agent for wide sweeps

A repo-wide search returns hundreds of lines of match output. **Delegate broad sweeps to
a sub-agent**, which absorbs the raw output and returns a compressed summary.

```
task(
  category="deep",
  load_skills=["ast_grep"],
  prompt="Find every `$X.unwrap()` in foolish-core and foolish-parser. Report the
         file, line, and enclosing function for each, grouped by file. Do not
         paste the raw match output."
)
```

For a single narrow query (one crate, one pattern), just call the tool directly — the
sub-agent overhead is not worth it.

---

## 1. Tool parameters

| Param | Required | Meaning |
|---|---|---|
| `action` | yes | `search` (read-only), `rewrite` (mutates files), `scan` (YAML rule) |
| `lang` | yes | tree-sitter language key — see §5 |
| `pattern` | for search/rewrite | the structural template |
| `rewrite` | for rewrite | replacement template, reusing bound metavariables |
| `yamlRule` | for scan | a complete inline ast-grep rule document |
| `scope` | no | sub-path to restrict the query to, e.g. `foolish-core` |

**Always pass `scope` when you know which crate you care about.** Without it the query
runs over the whole project and returns far more than you need. `scope` is confined to
the project directory; paths that climb out are rejected.

---

## 2. Metavariables

### `$NAME` — exactly one node

A single `$` plus UPPERCASE letters binds exactly one construct (identifier,
expression, literal, parameter).

Reusing the same name inside one pattern forces both positions to be **identical**:
`$A == $A` matches `x == x` but not `x == y`.

### `$$$NAME` — zero or more sibling nodes

`$$$`, `$$$ARGS`, `$$$BODY` match a run of siblings — argument lists, statement blocks,
struct fields.

### `$_` — one node, not captured

Matches a single node but binds nothing. Use it when you need a position filled but do
not intend to reference it in a `rewrite`.

> **Metavariables are never backslash-escaped.** Write `$X`, never `\$X`. Inside a
> `yamlRule`, a `\$` produces `Cannot parse rule` and the whole scan fails.

---

## 3. Action: `search`

Read-only. Verified against this repo:

```
action: search   lang: rust   scope: foolish-core
pattern: pub fn $NAME($$$ARGS) -> Result<$T, $E> { $$$ }
```

```
foolish-core/src/serialization.rs:42:pub fn fir_to_json(fir: &Fir) -> Result<String, SerdeError> {
foolish-core/src/serialization.rs:46:pub fn fir_from_json(text: &str) -> Result<Fir, SerdeError> {
```

Patterns may contain quotes and braces freely — arguments are passed as an argv array,
never through a shell, so `$` and backticks are not expanded:

```
action: search   lang: rust   scope: foolish-parser
pattern: panic!("$MSG")
```

Other language examples:

- Python route decorators — `@app.route($PATH, methods=[$$$])`
- TypeScript promises — `new Promise(($RESOLVE, $REJECT) => { $$$ })`
- Go error returns — `if $ERR != nil { return $$$ }`

---

## 4. Action: `rewrite`

**Mutates files in place.** There is no dry-run flag, so run the same pattern under
`search` first and read the match list before rewriting.

Every metavariable used in `rewrite` must be bound in `pattern`:

```
pattern:  fs.readFile($PATH, (err, $DATA) => { $$$BODY })
rewrite:  const $DATA = await fs.promises.readFile($PATH); $$$BODY
```

Migrating a removed API (`createCipher` was removed in Node 22 — no IV, MD5-based KDF):

```
pattern:  crypto.createCipher("aes-128-cbc", $KEY)
rewrite:  crypto.createCipheriv("aes-256-gcm", $KEY, $IV)
```

The second example introduces `$IV`, which is **not** bound in the pattern — it is
emitted literally as the text `$IV`. That is occasionally what you want (a deliberate
compile error marking every site a human must finish), but never by accident.

---

## 5. Action: `scan` — relational YAML rules

When a flat pattern cannot express the condition, pass a full rule document as
`yamlRule`. Verified rule:

```yaml
id: no-unwrap
language: rust
rule:
  pattern: $X.unwrap()
severity: warning
message: unwrap found
```

Scoped to `foolish-core`, this returns 7 warnings with file, line and a source excerpt.

### Relational selectors

| Selector | Direction | Meaning |
|---|---|---|
| `inside` | upward | node must sit within the given structure |
| `has` | downward | node must contain a matching child |
| `precedes` / `follows` | sideways | ordering among siblings |
| `not` | — | negate any sub-rule |
| `all` / `any` | — | combine sub-rules |

### Composite example

Async arrow functions that `await` but are not wrapped in a `try`:

```yaml
id: async-error-boundary
language: typescript
rule:
  all:
    - kind: arrow_function
    - has:
        pattern: await $A
        stopBy: end
    - not:
        inside:
          kind: try_statement
severity: warning
message: awaited call outside a try boundary
```

`stopBy: end` makes `has` descend the whole subtree instead of stopping at the first
level. Without it, deeply nested `await`s are missed.

### Language keys

`javascript`, `typescript`, `tsx`, `python`, `rust`, `go`, `c`, `cpp`, `java`, `ruby`,
`php`. The `lang` argument and the `language:` field in a `yamlRule` must agree.

---

## 6. When NOT to use this

- **Literal string hunting** — a config key, a log message, a URL. Use `grep`.
- **Non-parsing languages** — `.foo` and `.einmo` fixtures have no tree-sitter grammar.
- **Whole-file reading** — use `bundle_repo_ctx` with a `crate` argument instead.

---

## 7. Troubleshooting

| Symptom | Cause |
|---|---|
| `Cannot parse rule` | A `\$` in the YAML, or `language:` missing/misspelled |
| No matches on an obviously-present shape | Pattern is not a complete parseable construct — add the surrounding `{ $$$ }` |
| Match count far too high | Missing `scope`; the query ran repo-wide |
| Rewrite emitted a literal `$NAME` | That metavariable was never bound in `pattern` |
| Empty result reported as success | Correct — no matches is a success, not a failure |

---

## Last Updated

**Date**: 2026-08-09
**Changes**: Renamed from `ast_grep.md` to `SKILL.md` (the loader only reads `SKILL.md`,
so the previous file never loaded) and added the required `name`/`description`
frontmatter. Corrected the YAML blueprint, which used backslash-escaped `\$A`
metavariables that fail rule parsing. Documented the `scope` parameter. Replaced the
invented examples with ones verified against this workspace.
