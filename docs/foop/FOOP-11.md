---
foop: 11
title: Search stops at NK; search result becomes NK
author: hc <hc.busy@gmail.com>
status: Brewing
type: Standards
created: 2026-05-04
phase: phase-2
supersedes: []
---

# FOOP-11: Search stops at NK; search result becomes NK

## Abstract

When a search (anchored or unanchored) encounters an NK at any point
during resolution — in the anchor, in the dereferencing chain, in a
matched statement's body — the search itself becomes NK and stops.
There is no skip-NK-and-keep-searching mode in the MVP. NK propagates
strictly.

## Motivation

Earlier design exploration considered a search parameter
`nkPolicy: { StopAtNK | IgnoreNK }` that would let some searches skip
NK results and continue looking. This was overengineering for an MVP:

- We have no current language feature that requires IgnoreNK semantics.
- Implementing both modes doubles the search step rule's branching.
- NK is meant to be a hard error signal ("definitively unknown,
  recoordination cannot rescue") — letting it be silently skipped
  weakens that meaning.

The simpler rule wins: NK is contagious. A search that finds an NK
(anywhere along its resolution path) becomes NK. Done.

If a future language feature needs IgnoreNK semantics, a new FOOP can
introduce it as an explicit per-search flag. Until then, do not
implement it.

## Specification

### The rule

For every search FIR (`SearchFir`, `IndexFir`, `HeadTailFir`,
`CharacterizedRefFir`):

1. If the search's *anchor* is NK → search becomes NK.
2. If the search's *dereferenced target* is NK (chain ends in NK) →
   search becomes NK.
3. If the search *finds* a statement whose body is NK → search becomes
   NK.

There is no fallback, no continuation, no alternate-match-search.

### Compile-time signal

The `SearchFir` case class does NOT carry an `nkPolicy` field. The rule
is implicit: stop at NK.

This is intentional. Adding a field for a behavior that has only one
implemented value would be cosmetic clutter.

### Future extension

If/when an `IgnoreNK` mode is desired:

1. Write a new FOOP introducing the field.
2. Default existing searches to `StopAtNK` (this FOOP's behavior).
3. The few new searches that want `IgnoreNK` set the flag explicitly.
4. The field is added to the SearchFir variants and the JSON contract.

The field's absence today does not preclude its addition later — the
case class can grow a defaulted parameter without breaking JSON
roundtrip (Circe handles defaults).

## FIR Impact

None. No new fields, no new variants.

## UBC Step Impact

The Phase 2 search step rules (in `phase2_ubc.md`) follow the rule
above naturally. No special "NK skip" code path. No queue of "tried
that, NK, try the next match" — there is no next match to try.

## Test Plan

Phase 2 approval tests demonstrating the propagation:

```
{
  a = 5 / 0,        !! a is NK (div-by-zero)
  b = a             !! search for a finds NK; b is NK
}
```

```
{
  brane = {x = 5 / 0, y = 7},
  result = brane.x   !! brane.x finds x = NK; result is NK
}
```

```
{
  brane = {x = 5 / 0, y = 7},
  result = brane.y   !! brane.y finds y = 7 (CONSTANT); result is 7. The fact
                     !! that x is NK doesn't affect a search for y.
}
```

## Rejected Alternatives

### A. Implement both StopAtNK and IgnoreNK

**Rejected**: no current requirement; doubles search step branching;
weakens NK's meaning as a hard error signal. If/when needed, defer to a
future FOOP.

### B. Make NK soft — searches skip NK and continue

**Rejected**: NK is meant to be terminal. A search that ignores NK and
keeps looking would mask real errors (div-by-zero, depth-exceeded,
genuine permanent failures) by silently producing alternative results.
Foolish's diagnostic story works only if NK propagates.

### C. Make NK propagate but allow a per-search "rescue" expression

Like Java's try/catch but per search. **Rejected**: out of scope for
MVP; introduces a control-flow concept Foolish doesn't otherwise have.

## Open Questions

None.

## References

- `scala-mvp/foolish-scala/docs/phase2_ubc.md`: search step rules
  follow this rule by default — no separate code path needed.
- FOOP-10: anchored search through constanic anchors propagates NK
  consistently with this FOOP.
- FOOP-7: `constanicClone(NK) = NK` (NK is shared, not cloned, and
  remains NK).
