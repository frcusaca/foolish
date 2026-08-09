---
name: repo-context
description: "MUST USE before broad code exploration in this workspace — loads ONE cargo crate's full source as a single payload instead of opening files one at a time, and runs structural (AST) search across a crate. Covers the crate-at-a-time discipline, listing workspace crates, the verified-einmo fixture skip, and scoping structural queries. Prefer this over repeated Read/Glob/Grep when you need to understand a whole crate, and over grep when matching a code *shape* rather than a literal string. Triggers: 'understand the codebase', 'how does X work', 'read the crate', 'explore the repo', 'what crates are there', 'load context', 'bundle the repo', 'structural search', 'find all call sites', 'refactor across the crate'."
---

# Repo context and structural search

Two tools live in `.opencode/tools/`. Each is **both** an opencode tool and a
runnable script, so the same file serves either agent:

| | opencode | Claude Code / any shell |
|---|---|---|
| load a crate | `bundle_repo_ctx` tool | `bun .opencode/tools/bundle_repo_ctx.ts` |
| structural search | `ast_grep_engine` tool | `bun .opencode/tools/ast_grep_engine.js` |

Run them from the workspace root — both resolve paths against the current
working directory.

---

## 1. One crate at a time

This is a six-directory cargo workspace. Bundling all of it produces roughly
1.6M tokens, so the tool refuses to: it works on exactly one crate per call.

**Step 1 — see what's there.** With no arguments it lists crates and what each
would actually cost, without bundling anything (~125 tokens):

```bash
bun .opencode/tools/bundle_repo_ctx.ts
```

```
## AVAILABLE CRATES
- einmo                  13 files    321K  (einmo/)
- foolish-cli             1 files      4K  (foolish-cli/)
- foolish-core            9 files    155K  (foolish-core/)
- foolish-parser          5 files    100K  (foolish-parser/)
- foolish-ubca          534 files    1.2M  (foolish-ubca/)
```

The list comes from `cargo metadata`, so it reflects `[workspace] members` — a
directory that exists on disk but was removed from the manifest will not appear.

**Step 2 — bundle the one you need.**

```bash
bun .opencode/tools/bundle_repo_ctx.ts --crate foolish-core
```

You get the workspace `Cargo.toml` (for the inherited dependency and lint
context), then the crate's file tree with permissions, timestamps and git
status, then every source file inline.

### Budget before you call

| crate | ~tokens |
|---|---|
| `foolish-cli` | 1.5K |
| `foolish-parser` | 26K |
| `foolish-core` | 40K |
| `einmo` | 83K |
| `foolish-ubca` | 288K |

`foolish-ubca` is large enough to be worth a targeted read instead. If you do
bundle it, note that `foolish-ubca/src/fir_kinds.rs` is 320KB and exceeds the
default body limit — it appears in the tree and under `## SKIPPED`, not inline.

### Other flags

```bash
--max-depth N                   # traversal depth below the crate root (default 8)
--max-file-bytes N              # skip bodies over this size (default 200000)
--do-not-skill-einmo-verified   # include **/verified/**/*.einmo
```

Settled `verified/` einmo fixtures — about a third of all `.einmo` files — are
skipped by default. The payload always reports how many were withheld. Only
pass the flag when the fixtures themselves are the subject.

`docs/`, `.omo/`, `tmp/` and `target/` are always excluded, as are patterns from
`.claudeignore`.

---

## 2. Structural search

Use this when the target is a code **shape**, not a literal string. `grep` is
still the right tool for a config key or a log message.

```bash
bun .opencode/tools/ast_grep_engine.js --action search --lang rust \
    --pattern '$X.unwrap()' --scope foolish-core
```

**Always pass `--scope`.** Without it the query runs over the whole workspace.

Metavariables are `$X` (one node), `$$$ARGS` (zero or more siblings), `$_` (one
node, uncaptured). Never backslash-escape them — `\$X` is a parse error.

Multi-line YAML rules are awkward as a shell argument, so write the rule to a
file:

```bash
bun .opencode/tools/ast_grep_engine.js --action scan --lang rust \
    --yaml-rule-file /tmp/rule.yaml --scope foolish-core
```

`--action rewrite` **mutates files in place with no dry run.** Run the same
pattern under `--action search` first and read the match list.

For the full pattern and rule syntax, load the `ast_grep` skill.

---

## 3. When not to use these

- You already know the file — just read it.
- You need a literal string — use `grep`.
- You want the whole workspace at once — that is the case this tool exists to
  prevent. Pick a crate.

---

## Last Updated

**Date**: 2026-08-09
**Changes**: Created. Documents the dual tool/CLI entrypoints so the same files
serve opencode and any shell-capable agent.
