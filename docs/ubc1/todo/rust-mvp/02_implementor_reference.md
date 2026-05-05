# Foolish Rust MVP — Implementor Reference

> Everything a new implementor needs to understand the build infrastructure,
> parser module, Java AST, and interop strategy before writing a single line
> of Rust code.
>
> Read this before touching any source file.

---

## Repository Layout

```
foolish/                          ← root (Maven multi-module for Java/Scala)
├── foolish-parser-java/          ← ANTLR grammar + Java AST records  ← READ THIS
├── foolish-core-java/            ← Java UBC1 evaluator  ← reference only
├── foolish-core-scala/           ← Scala UBC1 evaluator  ← Scala MVP target
├── foolish/                      ← Rust crates (NEW)
│   ├── Cargo.toml                ← workspace root
│   ├── foolish-parser/           ← antlr4rust parser crate  ← Phase 1
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs            ← re-exports, FoolishAstn types
│   │   │   ├── ast.rs            ← Rust AST enum (mirrors Java AST records)
│   │   │   └── parser.rs         ← antlr4rust visitor wrapper
│   │   └── build.rs              ← runs antlr4 tool to generate Rust parser from .g4
│   ├── foolish-core/             ← Rust UBC1 evaluator  ← Phase 2+
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── fir.rs            ← FIR sealed enum with variants (derives serde)
│   │   │   ├── serialization.rs  ← FirSerializer trait + JsonSerializer impl
│   │   │   ├── compiler.rs       ← AST → FIR translation
│   │   │   ├── ubc.rs            ← UBC step evaluation
│   │   │   ├── search.rs         ← search resolution
│   │   │   ├── sequencer.rs      ← human-readable output formatter
│   │   │   └── test_helper.rs    ← approval test infrastructure (uses insta)
│   │   └── tests/
│   │       ├── ast.rs            ← parser correctness tests
│   │       ├── compiler.rs       ← AST → FIR tests
│   │       └── roundtrip.rs      ← FIR JSON roundtrip tests
│   ├── foolish-cli/              ← CLI binary  ← Phase 4
│   │   ├── Cargo.toml
│   │   └── src/main.rs
│   └── foolish-web/              ← Web browser  ← Phase 6
│       ├── Cargo.toml
│       └── src/main.rs
├── test-resources/               ← shared .foo approval test inputs
└── ...
```

**The Rust implementation lives in a Cargo workspace** under `foolish/` (a
subdirectory of the Maven root). The workspace is independent of Maven —
it uses its own `build.rs` to generate the parser from the shared grammar file.

---

## Build System

### Key versions

| Thing | Version | Crate/Tool |
|-------|---------|------------|
| Rust edition | 2024 | `rustc 1.85+` |
| ANTLR runtime | 4.13.2 | `antlr4rust` (Rust target) |
| Serialization (trait) | — | `serde` (derive on FIR types) |
| Serialization (default) | — | `serde_json` (cross-language JSON format) |
| Serialization (future) | — | `bincode` or `postcard` (binary, optional feature) |
| CLI | — | `clap` |
| Web server | — | `axum` + `tower-http` |
| Error handling | — | `anyhow` + `thiserror` |
| Regex | — | `regex` crate |
| Testing | — | `cargo test` (built-in) + `pretty_assertions` |

### Essential build commands

```bash
# From the foolish/ Rust workspace root:

# Full clean build with all tests
cargo clean && cargo test

# Build only, skip tests (fast iteration on compilation errors)
cargo build

# Run only a specific test
cargo test --package foolish-core parser::ast::parses_bare_identifier

# Run with verbose output (shows which crates are being compiled)
cargo test -v

# Check for compilation errors without full build (faster feedback)
cargo check

# Clippy lints
cargo clippy -- -D warnings

# Release build
cargo build --release
```

### Workspace structure

The workspace root `Cargo.toml`:

```toml
[workspace]
members = [
    "foolish-parser",
    "foolish-core",
    "foolish-cli",
    "foolish-web",
]
resolver = "2"

[workspace.dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1"
bincode = { version = "2", optional = true }      # binary format (optional)
postcard = { version = "1", optional = true }     # CBOR binary (optional)
antlr4rust = "0.7"
clap = { version = "4", features = ["derive"] }
regex = "1"
anyhow = "1"
thiserror = "2"

# foolish-core features
# foolish-core/Cargo.toml:
# [features]
# default = ["json"]
# json = ["dep:serde_json"]          # cross-language JSON (default)
# bincode = ["dep:bincode"]          # fast binary format (optional)
```

---

## Parser: ANTLR4 with antlr4rust

### Grammar file

The shared grammar lives at the Maven root:
```
foolish-parser-java/src/main/antlr4/Foolish.g4
```

The Rust parser crate's `build.rs` copies this grammar file and uses the
antlr4rust code generator to produce Rust parser/lexer sources:

```rust
// foolish-parser/build.rs
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    // Copy the grammar file from the Maven project
    let grammar_src = PathBuf::from("../foolish-parser-java/src/main/antlr4/Foolish.g4");
    let grammar_dst = PathBuf::from("src/main/antlr4/Foolish.g4");
    fs::copy(&grammar_src, &grammar_dst).expect("copy Foolish.g4");

    // Generate Rust parser using antlr4rust tool
    // The antlr4rust build tool reads the .g4 file and emits Rust sources
    // into OUT_DIR
    println!("cargo:rerun-if-changed=src/main/antlr4/Foolish.g4");
    // See antlr4rust documentation for build integration
}
```

Generated sources appear in `target/` — never checked in.

### How antlr4rust works

The `antlr4rust` crate provides a Rust runtime for ANTLR4-generated parsers.
Unlike the Java target which generates Java classes, antlr4rust generates
Rust structs and traits (lexer, parser, visitor, listener).

The generated visitor trait `FoolishVisitor` has methods for each grammar rule.
We implement a custom visitor (like Java's `ASTBuilder`) that walks the parse
tree and produces our Rust AST types:

```rust
use antlr4rust::tree::ParseTreeVisitor;
use crate::generated::foolish_visitor::FoolishVisitor;
use crate::ast::FoolishAstn;

pub struct AstBuilder;

impl<'a> FoolishVisitor<'a, FoolishAstn> for AstBuilder {
    fn visit_program(&self, ctx: &ProgramContext<'a>) -> FoolishAstn {
        // Walk children and construct FoolishAstn tree
        let branes = self.visit_children(ctx);
        branes
    }

    fn visit_integer_literal(&self, ctx: &IntegerLiteralContext<'a>) -> FoolishAstn {
        // Parse the token text into a Long
        let value = ctx.integer_literal_literal().get_text().parse::<i64>().unwrap();
        FoolishAstn::IntLit(value)
    }

    // ... implement for each grammar rule
}

pub fn parse(source: &str) -> Result<FoolishAstn, anyhow::Error> {
    let input = antlr4rust::CharStreams::from_string(source);
    let lexer = FoolishLexer::new(input);
    let tokens = antlr4rust::CommonTokenStream::new(lexer);
    let parser = FoolishParser::new(tokens);
    let tree = parser.program()?;
    let visitor = AstBuilder;
    Ok(tree.accept(&visitor))
}
```

### Identifier canonicalization

All three separator forms (`_`, `ˍ` modifier letter low line, ` ` narrow no-break
space) are normalized to `ˍ` by `AstBuilder`:

```rust
fn canonicalize_id(s: &str) -> String {
    s.replace(['_', 'ˍ', ' '], "ˍ")
}
```

When matching identifiers in the evaluator, always compare the canonicalized form.

---

## Rust AST: Node Types

The Rust AST mirrors the Java AST records but uses Rust enums and structs:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum FoolishAstn {
    IntLit(i64),
    Identifier {
        characterizations: Vec<String>,
        id: String,
    },
    Brane {
        characterizations: Vec<String>,
        statements: Vec<FoolishAstn>,
    },
    BinaryOp {
        op: String,
        left: Box<FoolishAstn>,
        right: Box<FoolishAstn>,
    },
    UnaryOp {
        op: String,
        expr: Box<FoolishAstn>,
    },
    Assignment {
        identifier: String,
        expr: Box<FoolishAstn>,
        operator: AssignmentOperator,
    },
    Concatenation {
        elements: Vec<FoolishAstn>,
    },
    UnknownLit,  // ???
    DotSearch {
        anchor: Box<FoolishAstn>,
        coordinate: String,
    },
    RegexSearch {
        anchor: Box<FoolishAstn>,
        operator: SearchOperator,
        pattern: String,
    },
    Seek {
        anchor: Box<FoolishAstn>,
        offset: i32,
    },
    OneShotSearch {
        anchor: Box<FoolishAstn>,
        operator: SearchOperator,
    },
    UnanchoredSeek {
        offset: i32,  // negative
    },
    NotImplemented(String),  // deferred features
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SearchOperator {
    Head,    // ^
    Tail,    // $
    RegexpLocal,       // ?
    RegexpForward,     // ~
    Seek,              // #
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AssignmentOperator {
    Assign,   // =
    SF,       // <=>
    SFF,      // <<=>>
}
```

Key Rust conventions applied:
- `enum` variants with associated data (not nested structs)
- `Box<T>` for recursive references (avoids infinite size)
- `#[derive(Debug, Clone, PartialEq)]` on all AST types
- No nullable types — use `Option<T>` or `Vec<T>` (empty) instead

---

## FIR Design: Rust Patterns

### Sealed FIR enum

Rust's enum system provides exhaustive pattern matching — no `@unchecked` needed
(unlike Scala matching Java sealed types):

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Fir {
    ConstantInt(i64),
    NKFir { reason: String },
    NormalBrane {
        characterizations: Vec<String>,
        statements: Vec<StatementFir>,
        state: Nyes,
    },
    Statement {
        name: Option<String>,
        body: Rc<RefCell<Fir>>,
        state: Nyes,
    },
    BinaryOp {
        op: String,
        left: Rc<RefCell<Fir>>,
        right: Rc<RefCell<Fir>>,
        state: Nyes,
    },
    UnaryOp {
        op: String,
        expr: Rc<RefCell<Fir>>,
        state: Nyes,
    },
    Search {
        pattern: String,
        direction: SearchDirection,
        anchored: bool,
        anchor: Option<Rc<RefCell<Fir>>>,
        target: Option<Rc<RefCell<Fir>>>,
        state: Nyes,
    },
    Index {
        offset: i32,
        anchored: bool,
        anchor: Option<Rc<RefCell<Fir>>>,
        state: Nyes,
    },
    HeadTail {
        is_head: bool,
        anchored: bool,
        anchor: Option<Rc<RefCell<Fir>>>,
        state: Nyes,
    },
    Concatenation {
        elements: Vec<Rc<RefCell<Fir>>>,
        merged: Option<Rc<RefCell<Fir>>>,
        state: Nyes,
    },
}
```

### Interior mutability with `Rc<RefCell<T>>`

FIR objects need two properties:
1. **Shared ownership** — multiple parents can reference the same FIR (search results, cloned values)
2. **Mutable state** — `state` transitions through the Nyes lifecycle

Rust's type system doesn't allow `&mut` and `&` references to coexist. The
standard pattern for graph-structured data with mutable nodes is
`Rc<RefCell<T>>`:

```rust
use std::rc::Rc;
use std::cell::RefCell;

let fir = Rc::new(RefCell::new(Fir::ConstantInt(42)));
let child = Rc::clone(&fir);  // shared ownership

// Mutable access with runtime borrow checking
let mut borrow = fir.borrow_mut();
borrow.state = Nyes.EMBRYONIC;
drop(borrow);  // borrow is released

// Immutable access
let value = fir.borrow().state;
```

This is the idiomatic Rust approach for the FIR graph. Every FIR that can be
referenced by multiple parents is wrapped in `Rc<RefCell<Fir>>`.

**Alternative considered:** `Arc<RwLock<Fir>>` if multi-threaded evaluation is
needed in Phase 5. For Phase 2 (single-threaded), `Rc<RefCell<T>>` is faster
(no atomic operations, no locking overhead).

### Nyes lifecycle enum

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Nyes {
    Prembrionic,
    Embryonic,
    Braning,
    Econstanic,
    Woconstanic,
    Constant,
    Independent,
    Nk,
}

impl Nyes {
    pub fn is_constanic(&self) -> bool {
        matches!(self, Nyes::Econstanic | Nyes::Woconstanic | Nyes::Constant | Nyes::Independent)
    }

    pub fn is_nigh(&self) -> bool {
        matches!(self, Nyes::Prembrionic | Nyes::Embryonic | Nyes::Braning)
    }
}
```

### Serde derives on FIR types

Every FIR variant derives `Serialize` and `Deserialize`:

```rust
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Fir { ... }
```

The FIR types themselves depend on `serde` (derive only), **not** on `serde_json`.

---

### Serialization Abstraction Layer

The serialization format is abstracted behind a trait in `foolish-core/src/serialization.rs`.
This keeps the FIR types and evaluator independent of any particular encoding:

```rust
// foolish-core/src/serialization.rs

use crate::fir::Fir;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SerdeError {
    #[error("encode failed: {0}")]
    Encode(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("decode failed: {0}")]
    Decode(#[source] Box<dyn std::error::Error + Send + Sync>),
}

/// Trait for encoding/decoding FIR trees to/from bytes or strings.
/// Default implementation uses serde_json.
/// Alternative implementations (bincode, postcard, rmp-serde)
/// can be enabled behind Cargo feature flags.
pub trait FirSerializer: Send + Sync {
    fn encode(&self, fir: &Fir) -> Result<Vec<u8>, SerdeError>;
    fn decode(&self, data: &[u8]) -> Result<Fir, SerdeError>;
    fn encode_to_string(&self, fir: &Fir) -> Result<String, SerdeError>;
    fn decode_from_str(&self, text: &str) -> Result<Fir, SerdeError>;
}
```

The default implementation uses `serde_json` and is the **canonical FIR wire format**
that all three language implementations (Rust, Java, Scala) must honor:

```rust
#[cfg(feature = "json")]  // enabled by default
pub struct JsonSerializer;

#[cfg(feature = "json")]
impl FirSerializer for JsonSerializer {
    fn encode(&self, fir: &Fir) -> Result<Vec<u8>, SerdeError> {
        Ok(serde_json::to_vec(fir).map_err(|e| SerdeError::Encode(Box::new(e)))?)
    }
    fn decode(&self, data: &[u8]) -> Result<Fir, SerdeError> {
        Ok(serde_json::from_slice(data).map_err(|e| SerdeError::Decode(Box::new(e)))?)
    }
    fn encode_to_string(&self, fir: &Fir) -> Result<String, SerdeError> {
        Ok(serde_json::to_string(fir).map_err(|e| SerdeError::Encode(Box::new(e)))?)
    }
    fn decode_from_str(&self, text: &str) -> Result<Fir, SerdeError> {
        Ok(serde_json::from_str(text).map_err(|e| SerdeError::Decode(Box::new(e)))?)
    }
}
```

A binary implementation can be added behind a feature flag without touching FIR types:

```rust
#[cfg(feature = "bincode")]
pub struct BincodeSerializer;

#[cfg(feature = "bincode")]
impl FirSerializer for BincodeSerializer {
    fn encode(&self, fir: &Fir) -> Result<Vec<u8>, SerdeError> {
        Ok(bincode::serialize(fir).map_err(|e| SerdeError::Encode(Box::new(e)))?)
    }
    fn decode(&self, data: &[u8]) -> Result<Fir, SerdeError> {
        Ok(bincode::deserialize(data).map_err(|e| SerdeError::Decode(Box::new(e)))?)
    }
    // ...
}
```

Callers use a type parameter or a boxed trait object:

```rust
// foolish-core/src/lib.rs
pub type DefaultSerializer = serialization::JsonSerializer;

pub fn fir_to_json(fir: &Fir) -> Result<String, SerdeError> {
    DefaultSerializer.encode_to_string(fir)
}

pub fn fir_from_json(text: &str) -> Result<Fir, SerdeError> {
    DefaultSerializer.decode_from_str(text)
}
```

**Why this separation:**
- FIR types and evaluator logic never import `serde_json`
- Switching to `bincode`, `postcard`, or `rmp-serde` is a feature flag — not a refactor
- The JSON format serves as the cross-language contract (Java/Scala can read it)
- Binary formats can be used for inter-process or persistent storage when speed matters

### Cross-language JSON compatibility

The serde-derived JSON format from Rust is the canonical FIR wire format. Java
(using Jackson/Gson) and Scala (using Circe/upickle) must produce and consume
the exact same JSON schema. The roundtrip tests enforce this:

```rust
// Roundtrip test — Rust writes, Rust reads
let fir = Fir::ConstantInt(42);
let json = DefaultSerializer.encode_to_string(&fir).unwrap();
let decoded: Fir = DefaultSerializer.decode_from_str(&json).unwrap();
assert_eq!(fir, decoded);
```

When testing cross-language compatibility, the Rust-produced JSON for a given
`.foo` input must be byte-identical to the JSON that Java and Scala implementations
produce for the same input.

---

## Test Resource Layout

The Rust crates point at the same shared `.foo` input directory:
```
test-resources/org/foolish/fvm/inputs/
```

Rust approval output lives in:
```
foolish/foolish-core/src/test/resources/org/foolish/fvm/rubc/
```

(The `rubc` = Rust UBC subdirectory, parallel to `ubc` for Java and `scubc` for Scala.)

---

## Approval Testing with `insta`

The [`insta`](https://insta.rs/) crate provides snapshot (approval) testing for Rust.
It is the de facto standard — explicitly described as "snapshot testing (also
sometimes called approval tests)."

```toml
[dev-dependencies]
insta = { version = "1", features = ["yaml"] }
```

### How it maps to our approval test workflow

| Our protocol | insta equivalent |
|--------------|-----------------|
| Run test → produces `.received.foo` | Run test → insta writes `.snap.new` |
| `diff -y received approved` | `cargo insta review` (TUI with side-by-side colored diff) |
| `mv received approved` | Accept in review UI |
| Commit mentions approval update | Snapshot files go in commit |

### Usage pattern

For the Foolish approval tests, use **external snapshot** mode (not inline),
so the approval output lives in `.foo` files alongside Java and Scala:

```rust
#[test]
fn simple_addition_is_approved() {
    let source = std::fs::read_to_string("test-resources/org/foolish/fvm/inputs/simpleAddition.foo").unwrap();
    let fir = Compiler::compile(&source).unwrap();
    let result = Ubc::run_to_completion(&fir).unwrap();
    let output = Sequencer::format(&fir);

    insta::with_settings!({
        snapshot_suffix => ".approved.foo",
        snapshots_path => "./test-resources/org/foolish/fvm/rubc/",
    }, {
        insta::assert_snapshot!("simpleAddition", output);
    });
}
```

When a snapshot changes:
```bash
cargo test          # produces .snap.new files
cargo insta review  # interactive TUI to accept/reject each change
```

For manual diff review (matching our existing protocol):
```bash
diff -y --color test.received.foo test.approved.foo
```

**Never edit `.approved.foo` / `.snap` files directly.** The protocol is:

1. Source code or `.foo` input changes
2. Run the test → produces new snapshot
3. Review: `cargo insta review` or `diff -y --color`
4. Human approves → accept in review UI
5. Commit message must mention "approval test updated"

---

## Rust-Specific Conventions

### Error handling

- Use `Result<T, E>` with `thiserror`-defined error types at crate boundaries
- Use `anyhow::Result<T>` internally for convenient error propagation
- Never use `unwrap()` in library code — use `?` operator
- Parse errors produce descriptive messages with line/column info from ANTLR

### No unsafe code

The entire implementation should be safe Rust. No `unsafe` blocks needed.
The FIR graph with `Rc<RefCell<T>>` provides all the mutability semantics
required.

### Pattern matching

Rust's exhaustive match eliminates the need for `@unchecked` casts:

```rust
match fir.borrow().state {
    Nyes::Embryonic => { /* ... */ }
    Nyes::Braning => { /* ... */ }
    Nyes::Constant => { /* ... */ }
    // compiler warns if cases are missing
}
```

### String handling

- Use `&str` for borrowed string references (parameters, patterns)
- Use `String` for owned data (AST node fields, FIR fields)
- Identifier canonicalization returns `String`

---

## Module Dependencies

```
foolish-parser (antlr4rust, serde)
    ↑
foolish-core (foolish-parser, serde, regex)
    ├── serde_json  [default feature "json"]
    ├── bincode     [optional feature "bincode"]
    ├── postcard    [optional feature "postcard"]
    │
    ↑
foolish-cli (foolish-core, clap)
    ↑
foolish-web (foolish-core, axum, tower-http)
```

Note: `serde_json` is a dependency of `foolish-core` under the default feature,
not of the FIR types themselves. The FIR enum derives `serde::Serialize` and
`serde::Deserialize`, but the serialization trait implementation (`FirSerializer`)
is what depends on `serde_json`. This keeps the FIR module encoding-agnostic.

---

## Last Updated

**Date**: 2026-05-05
**Updated By**: Claude Code; Qwen3.6-27B-AWQ-BF16-INT4
**Changes**: Initial creation — comprehensive Rust implementor reference. Covers
Cargo workspace layout, antlr4rust parser generation, Rust AST types, FIR design
with Rc<RefCell<T>> interior mutability, serde for JSON, error handling conventions,
and module dependency graph.

**Date**: 2026-05-05
**Updated By**: Claude Code; Qwen3.6-27B-AWQ-BF16-INT4
**Changes**: Added serialization abstraction layer (FirSerializer trait) separating
serde derives on FIR types from serde_json implementation. Added feature flags for
swapping to bincode/postcard. Added cross-language JSON compatibility section.
Added insta crate for approval/snapshot testing with cargo insta review workflow.
