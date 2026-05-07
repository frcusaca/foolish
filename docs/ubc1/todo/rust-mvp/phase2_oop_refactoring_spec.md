# UBC OOP Refactoring — Spec

> Goal: Transform the FIR enum from a big match dispatch into a trait-based
> OOP system where each variant is a struct with its own `step_one()` method.
> This prepares for UBCb (breadth-first) and makes the codebase maintainable.

---

## Current Problem

The `Fir` enum has 12 variants. Every operation (step, state, children,
format) is a match dispatch in the `impl Fir` block. Adding a new variant
means editing 5+ match statements across 3 files.

```rust
// Current: big enum, big matches
pub enum Fir {
    ConstantInt { value: i64, state: Nyes },
    Nk { reason: String, state: Nyes },
    NormalBrane { characterizations: Vec<String>, statements: Vec<StatementFir>, state: Nyes },
    BinaryOp { op: String, left: Box<Fir>, right: Box<Fir>, state: Nyes },
    // ... 8 more variants
}

impl Fir {
    pub fn state(&self) -> Nyes {
        match self {
            Fir::ConstantInt { state, .. } => *state,
            Fir::Nk { state, .. } => *state,
            Fir::NormalBrane { state, .. } => *state,
            // ... 12 arms
        }
    }
}
```

## Target Design: Trait-Based OOP

### Core Concept

Each FIR variant becomes a **struct** that implements the **`Steppable`** trait.
The `Fir` enum becomes a **type tag** for serialization and exhaustiveness.
Runtime dispatch uses the trait (virtual dispatch), not match.

### The Trait

```rust
pub trait Steppable {
    /// Perform one evaluation step. Returns what happened.
    fn step_one(&mut self, scope: &Scope) -> Result<StepResult, UbcError>;

    /// Current Nyes state
    fn state(&self) -> Nyes;

    /// Set Nyes state
    fn set_state(&mut self, state: Nyes);

    /// Mutable references to child FIRs for stepping
    fn children_mut(&mut self) -> Vec<&mut FirRef>;

    /// Shared, inherited method: step all children
    fn step_members(&mut self, scope: &Scope) -> Result<Vec<StepResult>, UbcError> {
        self.children_mut().iter_mut()
            .map(|child| child.borrow_mut().step_one(scope))
            .collect()
    }
}
```

### The Hierarchy

```
Steppable (trait)
├── ConstantIntFir ──── No children, terminal
├── NkFir ───────────── No children, terminal
├── BinaryOpFir ─────── Children: [left, right]
├── UnaryOpFir ──────── Child: [expr]
├── SearchFir ───────── No children (anchor/target resolved separately)
├── IndexFir ────────── No children (anchor resolved separately)
├── HeadTailFir ─────── No children (anchor resolved separately)
├── StayFoolishFir ──── Child: [expr], special stepping
├── StayFullyFoolishFir ── Child: [expr], no stepping
├── ConcatenationFir ─ Children: [elements...]
└── NormalBraneFir ──── Children: [stmt.body for stmt in statements]
```

### Struct Definitions

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct ConstantIntFir {
    pub value: i64,
    pub state: Nyes,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NkFir {
    pub reason: String,
    pub state: Nyes,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BinaryOpFir {
    pub op: String,
    pub left: FirRef,
    pub right: FirRef,
    pub state: Nyes,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnaryOpFir {
    pub op: String,
    pub expr: FirRef,
    pub state: Nyes,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SearchFir {
    pub pattern: String,
    pub direction: SearchDirection,
    pub anchored: bool,
    pub anchor: Option<FirRef>,
    pub target: Option<FirRef>,
    pub state: Nyes,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IndexFir {
    pub offset: i32,
    pub anchored: bool,
    pub anchor: Option<FirRef>,
    pub state: Nyes,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HeadTailFir {
    pub is_head: bool,
    pub anchored: bool,
    pub anchor: Option<FirRef>,
    pub state: Nyes,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StayFoolishFir {
    pub expr: FirRef,
    pub state: Nyes,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StayFullyFoolishFir {
    pub expr: FirRef,
    pub state: Nyes,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConcatenationFir {
    pub elements: Vec<FirRef>,
    pub merged: Option<FirRef>,
    pub state: Nyes,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NormalBraneFir {
    pub characterizations: Vec<String>,
    pub statements: Vec<StatementFir>,
    pub state: Nyes,
}
```

### FirRef Becomes Trait Object

```rust
// Current: enum inside RefCell
pub type FirRef = Rc<RefCell<Fir>>;

// New: trait object inside RefCell
pub trait Steppable: std::fmt::Debug {
    // ... trait methods
}

pub type FirRef = Rc<RefCell<dyn Steppable>>;
```

### StepResult Enum

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum StepResult {
    /// No change (terminal state)
    NoOp,
    /// State changed, may need re-stepping
    MadeProgress,
    /// New children exposed (enqueue them)
    NewChildren,
    /// Blocked waiting on dependency
    Blocked(FirRef),
}

impl StepResult {
    pub fn is_constanic(&self) -> bool {
        matches!(self, StepResult::NoOp)
    }
}
```

### Steppable Trait Implementations

```rust
impl Steppable for ConstantIntFir {
    fn step_one(&mut self, _scope: &Scope) -> Result<StepResult, UbcError> {
        Ok(StepResult::NoOp)
    }
    fn state(&self) -> Nyes { self.state }
    fn set_state(&mut self, s: Nyes) { self.state = s; }
    fn children_mut(&mut self) -> Vec<&mut FirRef> { vec![] }
}

impl Steppable for BinaryOpFir {
    fn step_one(&mut self, scope: &Scope) -> Result<StepResult, UbcError> {
        let results = self.step_members(scope)?;
        if results.iter().all(|r| r.is_constanic()) {
            // Compute result, replace self
            let result = compute_binary(&self.op, &self.left, &self.right)?;
            *self = result.into(); // Need FirRef -> Self conversion
        }
        Ok(StepResult::MadeProgress)
    }
    fn state(&self) -> Nyes { self.state }
    fn set_state(&mut self, s: Nyes) { self.state = s; }
    fn children_mut(&mut self) -> Vec<&mut FirRef> {
        vec![&mut self.left, &mut self.right]
    }
}

impl Steppable for NormalBraneFir {
    fn step_one(&mut self, scope: &Scope) -> Result<StepResult, UbcError> {
        match self.state {
            Nyes::Prembrionic => { self.state = Nyes::Embryonic; return Ok(StepResult::NoOp); }
            Nyes::Embryonic => { self.state = Nyes::Braning; return Ok(StepResult::NewChildren); }
            Nyes::Braning => {
                let results = self.step_members(scope)?;
                self.state = compute_brane_state(&self.statements);
                return Ok(StepResult::MadeProgress);
            }
            _ => return Ok(StepResult::NoOp), // Terminal
        }
    }
    fn state(&self) -> Nyes { self.state }
    fn set_state(&mut self, s: Nyes) { self.state = s; }
    fn children_mut(&mut self) -> Vec<&mut FirRef> {
        self.statements.iter_mut().map(|s| &mut s.body).collect()
    }
}
```

### StatementFir Uses FirRef

```rust
pub struct StatementFir {
    pub name: Option<String>,
    pub body: FirRef,  // Changed from Fir to FirRef
    pub state: Nyes,
}
```

---

## Implementation Plan

### Phase 1: Struct Extraction (no behavior change)
1. Create struct types for each variant
2. Keep the enum, but each variant wraps a struct
3. Implement `Steppable` trait for each struct
4. Enum dispatches to trait via match

### Phase 2: Trait Objects
1. Change `FirRef` to `Rc<RefCell<dyn Steppable>>`
2. Replace enum with trait objects
3. Remove match dispatch, use virtual dispatch
4. Update compiler to create structs directly

### Phase 3: Cleanup
1. Remove enum-based serialization, use trait serialization
2. Update sequencer to use trait methods
3. Remove `impl Fir` match blocks
4. All tests pass

---

## Risks and Mitigations

| Risk | Mitigation |
|------|-----------|
| Trait objects lose exhaustiveness checking | Keep enum for serialization, derive structs from enum definition |
| `Box<dyn Steppable>` performance | Use `Rc<RefCell<dyn Steppable>>` — same as current `Rc<RefCell<Fir>>` |
| Breaking serde serialization | Keep enum as serde interface, convert enum ↔ struct |
| UBCb parallelism needs Arc | Add `Arc<dyn Steppable>` alias for parallel path |

---

## Success Criteria

- All 16 approval tests pass
- Each variant is a struct with its own `step_one()` method
- `step_members()` is shared via trait default implementation
- Compiler creates structs directly (no enum match)
- Sequencer uses trait methods (no enum match)
- `FirRef` is `Rc<RefCell<dyn Steppable>>`
- Adding a new variant only requires one new struct + one trait impl

---

## Last Updated

**Date**: 2026-05-06
**Updated By**: Claude Code; Qwen3.6-27B-AWQ-BF16-INT4
**Changes**: Initial creation — OOP refactoring spec. Documents trait-based design,
struct hierarchy, Steppable trait with step_members() base class method,
implementation phases, and risk mitigation.
