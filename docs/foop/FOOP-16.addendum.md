# FOOP-16 Addendum: container structure, `foolish-ubca` vs `foolish-ubca2`

This addendum is a post-completion note, not a spec amendment — FOOP-16 itself
is `status: complete` and its `.md`/`.plan.md` are the frozen historical
record of what was designed and executed. This file exists to answer a
recurring question about the finished code's shape: the design phase (see
`FOOP-16.md` §Specification, "The `FirCursor`/`FirCursorMut` wrapper" and
"Resolved: two cursor types, not one") discussed at length where mutable
tree-state lives and how it's accessed; this addendum restates that
discussion against the actual, as-built code, and names the one categorical
shift precisely: kind-polymorphism moved from the type system into data.

## `foolish-ubca`'s shape

Kind-identity lives **in the type system** — 13 distinct Rust struct types
(`IndepIntFir`, `OperatorFir`, `StatementFir`, ...), unified only through
`trait Fir`'s vtable. The shared/mutable tree-state was never inherited
(Rust has no struct inheritance); it was always **composition**:

```
Rc<RefCell<dyn Fir>>                    <- shared handle; RefCell = runtime-checked exclusivity
        │
        │  IS-A Fir  (13 concrete structs implement the trait; dispatch via vtable)
        ▼
┌───────────────────────────────┐
│ IndepIntFir  (or any of 13)     │
│                                  │
│  core: ProtoBrane  ──HAS-A──────┼──►┌────────────────────────────────┐
│  value: i64  (kind-specific)     │   │ ProtoBrane                      │
└───────────────────────────────┘   │  foolish_children: Vec<FirRef>   │ tree topology
                                      │  ubc_children: RefCell<Vec<_>>   │ tree topology (mutable)
                                      │  nyes: Cell<Nyes>                 │ eval state (mutable)
                                      │  tasks: RefCell<VecDeque<_>>     │ eval state (mutable)
                                      │  parent: Weak<RefCell<dyn Fir>>   │ tree topology (back-link)
                                      │  alarm_reason: RefCell<_>          │
                                      └────────────────────────────────┘
```

`trait Fir`'s one bridge method, `fn core(&self) -> &ProtoBrane`, is what
lets generic tree-walking code reach the shared state regardless of which of
the 13 concrete types it holds. Each kind-specific field (`value: i64`, etc.)
sits *beside* `core`, not inside it.

## `foolish-ubca2`'s shape

```
FirPointer  (Copy: arena stamp + index + generation)   <- validated handle, never dereferenced directly
        │
        │  looked up via FVMStorage::get/get_mut(ptr)
        ▼
┌──────────────────────────────────────┐
│ Slot                                    │  <- one arena slot = one node, owned by FVMStorage
│                                          │
│  parent: FirPointer         ────────────┼── tree topology (moved OUT of the payload)
│  foolish_children: Vec<FirPointer>  ────┼── tree topology (moved OUT of the payload)
│  generation: u32                          │
│                                            │
│  payload: ArenaFir  ──HAS-A───────────────┼──►┌─────────────────────────────────┐
└──────────────────────────────────────┘      │ ArenaFir                           │
                                                 │  spec: FirSpec ──HAS-A───────────┼──►FirSpec::IndepInt{value}
                                                 │                                    │     | FirSpec::Operator{op}
                                                 │                                    │     | ...14 variants
                                                 │  ubc_children: Vec<FirPointer>     │ eval state (plain)
                                                 │  nyes: Nyes                          │ eval state (plain)
                                                 │  tasks: VecDeque<FirPointer>         │ eval state (plain)
                                                 │  alarm_reason: Option<String>          │
                                                 │  nf_reason: Option<String>               │
                                                 │  helpers_populated: bool                  │
                                                 └─────────────────────────────────┘

FVMStorage { arena_id, slots: Vec<Slot> }   <- owns every node; &mut FVMStorage is the ONE exclusivity mechanism
```

`FirCursor<'s> { ptr: FirPointer, storage: &'s FVMStorage }` and
`FirCursorMut<'s> { ptr: FirPointer, storage: &'s mut FVMStorage }` are a
transient identity-plus-borrowed-arena pairing, built fresh for one call or
one short run of calls and dropped immediately after — a convenience
wrapper, never itself a stored container, and never the receiver `step`
recurses through (see `FOOP-16.md`'s "Stepping does not go through
`FirCursorMut` as its receiver").

## The actual shift: IS-A became HAS-A

- **`foolish-ubca`**: "which kind is this?" is answered by *which Rust type
  you're holding* — `IndepIntFir` **is-a** `Fir`, dispatched through a
  vtable. Kind-polymorphism lives in the type system.
- **`foolish-ubca2`**: every node is the *same* Rust type (`ArenaFir`, in a
  `Slot`, behind a `FirPointer`). "Which kind" is answered by *a value* —
  `ArenaFir` **has-a** `FirSpec`, and code `match`es on its discriminant.
  Kind-polymorphism moved from types into data — a closed sum type instead
  of an open trait-object hierarchy (matching `rust_instructions.md`'s
  "types over generics, generics over `dyn`" preference).

`FirSpec` is worth calling **is-more-than-a**: its variants aren't bare tags
— `FirSpec::IndepInt { value: i64 }` carries that kind's own data inline. It
is simultaneously the identity-selector and the data-holder: a tagged
union, not a marker enum.

The has-a container for shared mutable state never went away, because it
was never the is-a part to begin with — `ProtoBrane` was always
composition. `ArenaFir` is its direct structural descendant, stripped of
`Cell`/`RefCell` wrappers, because one global `&mut FVMStorage` borrow now
provides the exclusivity that used to need per-field interior mutability.

## One refinement beyond the literal spec sketch

`FOOP-16.md`'s own draft proposed:

```rust
struct Slot {
    fir: Fir,
    generation: u32,
}
```

— payload only. The as-built implementation pulls `parent` and
`foolish_children` *out* of the payload and onto `Slot` itself. That is what
makes `create_child` a true one-shot atomic primitive: the arena — not each
node — owns topology, so allocating a slot and wiring both its parent-link
and the parent's child-list happen as one write to arena-owned state, not
two writes to two different owners that could drift out of sync (exactly
the boilerplate/consistency risk FOOP-16's Motivation names as the reason
to migrate away from `Rc::new_cyclic` in the first place).
