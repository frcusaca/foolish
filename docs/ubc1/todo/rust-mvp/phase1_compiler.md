# Phase 1 — Compiler: Source to FIR JSON

> Goal: Parse `.foo` source, translate AST to Foolish Internal Representation (FIR),
> serialize as JSON via serde. **No evaluation.** Every FIR comes out in
> `EMBRYONIC` state (the Nyes lifecycle name for "freshly initialized, not yet
> stepped"), except integer literals (compile to `INDEPENDENT`) and `???`
> (compiles to `NK`).

---

## Phase 1 Deliverable

A `Compiler::compile(source: &str) -> anyhow::Result<Fir>` function that:

1. Parses Foolish source via antlr4rust (generated from `Foolish.g4`)
2. Walks the parse tree with a custom `FoolishVisitor`, converts to Rust AST (`FoolishAstn`)
3. Translates Rust AST to FIR (the work of this phase)
4. Serializes the FIR tree via `FirSerializer` trait (default: `JsonSerializer`)

The FIR serialization is the contract between Phase 1 and Phase 2. Phase 2's evaluator
reads this serialized data, deserializes it back to FIR, and steps the states forward.

The default `JsonSerializer` produces JSON that Java and Scala implementations
must also be able to read — the serde-derived JSON schema is the canonical wire format.

---

## Why This Approach

**The compiler is the parser plus a structural translator.** No arithmetic is
performed. `1 + 2 * 3` compiles to:

```rust
BinaryOp {
    op: "+".to_string(),
    left: Box::new(ConstantInt(1)),
    right: Box::new(BinaryOp {
        op: "*".to_string(),
        left: Box::new(ConstantInt(2)),
        right: Box::new(ConstantInt(3)),
    }),
    state: Nyes::Embryonic,
}
```

All three integer literals are already `INDEPENDENT` (no work needed to know their
value), but the binary expression itself is `EMBRYONIC` until Phase 2's evaluator
visits it.

**Bare identifiers compile to fully-configured search FIRs.** `a_config` becomes:

```rust
Search {
    pattern: "^aˍconfig$".to_string(),
    direction: SearchDirection::Backward,
    anchored: false,
    anchor: None,
    target: None,
    state: Nyes::Embryonic,
}
```

---

## antlr4rust Integration

The `foolish-parser` crate uses `antlr4rust` for parsing. The `build.rs` script
generates Rust parser/lexer sources from the shared `Foolish.g4` grammar file.

```toml
# foolish-parser/Cargo.toml
[package]
name = "foolish-parser"
version = "0.1.0"
edition = "2024"

[dependencies]
antlr4rust = "0.7"
serde = { workspace = true }
anyhow = { workspace = true }
```

The visitor implementation follows the antlr4rust visitor pattern:

```rust
impl<'a> FoolishVisitor<'a, FoolishAstn> for AstBuilder {
    fn visit_standard_brane(&self, ctx: &StandardBraneContext<'a>) -> FoolishAstn {
        let mut statements = Vec::new();
        for stmt_ctx in ctx.statement() {
            statements.push(self.visit(&stmt_ctx));
        }
        for body_ctx in &ctx.stmt_body() {
            statements.push(self.visit(body_ctx));
        }
        FoolishAstn::Brane {
            characterizations: Vec::new(),
            statements,
        }
    }
}
```

---

## Three Test Layers

Phase 1 has three test files, one per failure mode:

### Layer 1: `tests/ast.rs` — parser correctness

Per-construct unit tests. Each test:
- inputs a small Foolish source string
- parses + converts via `AstBuilder` visitor
- asserts the resulting `FoolishAstn` tree equals an inline-constructed expected value

```rust
#[test]
fn parses_bare_identifier() {
    let ast = parse("{aˍconfig}").unwrap();
    assert_eq!(ast, FoolishAstn::Brane {
        characterizations: vec![],
        statements: vec![
            FoolishAstn::Identifier {
                characterizations: vec![],
                id: "aˍconfig".to_string(),
            }
        ],
    });
}
```

### Layer 2: `tests/compiler.rs` — AST → FIR translation correctness

Per-construct unit tests for the AST → FIR step. Each test:
- inputs a `FoolishAstn` value (Rust literal, no parsing)
- runs the AST → FIR compiler
- asserts the resulting FIR equals an inline-constructed expected value

```rust
#[test]
fn compiles_bare_identifier_to_backward_search() {
    let ast = FoolishAstn::Identifier {
        characterizations: vec![],
        id: "aˍconfig".to_string(),
    };
    let fir = compile_astn(ast);
    assert_eq!(fir, Fir::Search {
        pattern: "^aˍconfig$".to_string(),
        direction: SearchDirection::Backward,
        anchored: false,
        anchor: None,
        target: None,
        state: Nyes::Embryonic,
    });
}
```

### Layer 3: `tests/roundtrip.rs` — serialization roundtrip

For every FIR variant, assert encode→decode preserves structure. These tests
use the `FirSerializer` trait (default: `JsonSerializer`), not `serde_json` directly —
keeping them format-agnostic.

```rust
#[test]
fn roundtrip_search_fir() {
    let fir = Fir::Search {
        pattern: "^x$".to_string(),
        direction: SearchDirection::Backward,
        anchored: false,
        anchor: None,
        target: None,
        state: Nyes::Embryonic,
    };
    let serializer = DefaultSerializer;  // JsonSerializer by default
    let encoded = serializer.encode_to_string(&fir).unwrap();
    let decoded: Fir = serializer.decode_from_str(&encoded).unwrap();
    assert_eq!(fir, decoded);
}
```

When a binary serializer is enabled (e.g., `cargo test --features bincode`),
the same roundtrip tests run against `BincodeSerializer` instead.

---

## Phase 1 Implementation Steps

### P1.1 — Skeleton

Land the empty stub Compiler that parses, converts to AST, and emits `NKFir` for
every input. Verify `cargo test` runs.

### P1.2 — Integer literals

| Add to `compile_astn` | Test in |
|---|---|
| `IntLit(v) -> ConstantInt(v)` | `compiler.rs`, `roundtrip.rs` (already), `ast.rs` |

### P1.3 — Empty brane and brane with anonymous statements

| Add to `compile_astn` | Test in |
|---|---|
| `Brane { chars, stmts } -> NormalBrane { ... }` | all 3 layers |
| Anonymous statement: `Statement { name: None, body }` | |

### P1.4 — Identification

| Add to `compile_astn` | Test in |
|---|---|
| `Assignment { id, operator: Assign, expr } -> Statement { name: Some(id), body }` | all 3 layers |

### P1.5 — Arithmetic (tree only, no compute)

| Add to `compile_astn` | Test in |
|---|---|
| `BinaryOp { op, left, right } -> BinaryOp { ... }` | all 3 layers |
| `UnaryOp { op, expr } -> UnaryOp { ... }` | |

### P1.6 — Bare identifiers (unanchored search)

| Add to `compile_astn` | Test in |
|---|---|
| `Identifier { id, .. } -> Search { "^"+id+"$", Backward, anchored=false }` | all 3 layers |

### P1.7 — `#-N` unanchored seek

| Add to `compile_astn` | Test in |
|---|---|
| `UnanchoredSeek { offset } -> Index { offset, anchored=false }` | all 3 layers |

### P1.8 — Anchored search operators

| Add to `compile_astn` | Test in |
|---|---|
| `DotSearch { anchor, coordinate } -> Search { "^"+coord+"$", anchored=true }` | all 3 layers |
| `RegexSearch { anchor, pattern, .. } -> Search { pattern, anchored=true }` | |
| `Seek { anchor, offset } -> Index { offset, anchored=true }` | |
| `OneShotSearch { anchor, Head/Tail } -> HeadTail { is_head, anchored=true }` | |

### P1.9 — Assignment sugar

`=$ expr` and `=^ expr` are already desugared by `AstBuilder` to `OneShotSearchExpr`.

### P1.10 — `???` literal

| Add to `compile_astn` | Test in |
|---|---|
| `UnknownLit -> NKFir { reason: "??? literal" }` | all 3 layers |

### P1.11 — Reject deferred constructs

For these AST node types, `compile_astn` must return a clear error:

```rust
match ast {
    FoolishAstn::NotImplemented(reason) => Err(anyhow::anyhow!("Not yet implemented: {}", reason)),
    FoolishAstn::Concatenation { .. } => Err(anyhow::anyhow!("Concatenation: deferred to Phase 3")),
    _ => Ok(...),
}
```

---

## What's Out of Phase 1

- Any actual evaluation
- Concatenation, forward search, detachment, SF/SFF
- The 60 `.foo` approval tests live in Phase 2

---

## Phase 1 Exit Criteria

- All three test layers pass for every language construct in scope.
- Every FIR variant has a roundtrip test.
- `Compiler::compile(source)` runs end-to-end on at least 5 representative
  `.foo` source files without panicking — output is hand-inspected, no automated approval yet.
- `phase2_ubc.md` has been read and any open questions about the FIR schema have
  been resolved.

---

## Last Updated

**Date**: 2026-05-05
**Updated By**: Claude Code; Qwen3.6-27B-AWQ-BF16-INT4
**Changes**: Initial creation — Rust Phase 1 compiler plan. Adapted from Scala version:
Circe → serde, ScalaTest → cargo test, Scala case classes → Rust enums,
ANTLR Java visitor → antlr4rust visitor.
