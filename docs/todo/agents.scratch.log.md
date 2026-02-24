# AI Scratch Pad — agents.scratch.log.md

Append-only. Each entry is headed with `---` then `[YYYY-MM-DD] agent-identity / model-version`.
Never edit prior entries. Reference by line number from other documents.

---
[2026-02-24] Claude Code v1.0.0 / claude-sonnet-4-6

## WOCONSTANIC Dereferencing — Why It's Transparent, Not a Barrier

This took significant iteration to get right. The original ubc2_design.md text claimed
WOCONSTANIC searches are barriers "because they haven't performed their search yet." That is
false. By definition, WOCONSTANIC means the search *did* find its target — the target is
constanic. The search is structurally complete; it is only waiting.

The correct protocol: dereference *through* WOCONSTANIC searches to reach their constanic
target (which is a CONSTANIC barrier). This means a chain `c → b(WOCONSTANIC) → a(CONSTANIC)`
correctly collapses so that `c` becomes WOCONSTANIC targeting `a`, not `b`. This was
established in `todo/0.a.concatenation.woes.md` and is now canonical in `how/ubc2_design.md`
§ Key Dereferencing Rules and § Example: Search Chain.

The resolve() pseudocode pivot point:
```python
if cur_res.at_woconstanic():
    cur_res = cur_res.target   # follow through, re-enter loop
    continue
if cur_res.at_constanic():
    self.state = WOCONSTANIC   # stop here — CONSTANIC is the barrier
    self.target = cur_res
    return
```

Do not re-open this debate without re-reading `todo/0.a.concatenation.woes.md` first.

---
[2026-02-24] Claude Code v1.0.0 / claude-sonnet-4-6

## Output Symbol Ordering Rationale — More ?s = Bigger Unknown

When assigning `?` / `??` / `???` to the three constanic states, the intuition is:
more question marks = larger unknown. NK (`🧠???`) is definitively unknowable — largest.
CONSTANIC (`🧠??`) is unknown in this context but potentially findable — medium. WOCONSTANIC
(`🧠?`) found everything, just waiting — smallest unknown.

This is the *opposite* of the assignment inherited from UBC1. The swap was deliberate. The
mnemonic: if you see more question marks in the output, the situation is worse.

Note also: none of these three symbols are valid Foolish source syntax. All carry the 🧠
prefix to signal they are sequencer output. A human can type them in approval tests but not
in `.foo` program source files.

---
[2026-02-24] Claude Code v1.0.0 / claude-sonnet-4-6

## Terminology Anchoring — Why achievedConstanic() not isConstanic()

The old `isConstanic()` method meant "at or beyond CONSTANIC" — a compound predicate that
confused "is in state CONSTANIC" with "has reached the constanic family." This caused bugs
and was a source of the 20-day CONSTANIC confusion documented in git history (see
`agents.status.md` § Git History: Design Iteration Patterns).

The new convention:
- `atConstanic()` = exactly CONSTANIC state (and `atWoconstanic()`, etc.)
- `isConstanic()` = same as `at`, exact state only (no overloading)
- `achievedConstanic()` = at CONSTANIC or any post-constanic state
  (`at_constanic || at_woconstanic || at_constant || at_independent`)
- `isNigh()` = before and not including CONSTANIC (replaces `isNye()`)

The word *nigh* was chosen because it means "near but not yet reached." The old `NYE` label
came from a different etymology and was less transparent. Both Java (`isNigh()`) and Python
pseudocode (`is_nigh()`) use this spelling.

---
[2026-02-24] Claude Code v1.0.0 / claude-sonnet-4-6

## 🧠 Is the Denotational Marker — Not a "Machine-Generated" Tag

The 🧠 prefix was previously described as signaling "machine-generated output" or "not typable
by the programmer." That explanation is technically correct but misses the point. The real
meaning is deeper:

**🧠 is the denotational marker for Foolish semantics.** It means: we are now talking about
the *meaning* of this expression, not its surface syntax.

This unifies all three uses coherently:
- `🧠1` — the denotation of the literal `1` (a single-element brane)
- `🧠+` — the denotation of the addition operator `+` (a system operator brane)
- `🧠??` — the denotation of a CONSTANIC state (what the sequencer renders for meaning)

In all three cases, the programmer writes syntax (`1`, `+`, nothing), and the 🧠 form is what
that syntax *means* at the semantic level. This is a standard denotational semantics convention:
you write ⟦e⟧ for "the meaning of expression e." In Foolish, 🧠 plays the role of ⟦⟧.

The practical consequences are unchanged (programmers don't type 🧠 symbols in source), but the
*reason* is now principled rather than incidental. Future designers should treat 🧠 as a
semantic bracket, not an implementation tag.

---
[2026-02-24] Claude Code v1.0.0 / claude-sonnet-4-6

## Refinement: 🧠 as Single-Token Semantic Bracket Prefix

The prior entry described 🧠 as "analogous to ⟦⟧." More precisely:

🧠 is a **single-token prefix for the Foolish semantic bracket**. In standard notation,
⟦1 + 2⟧ is the meaning of the whole expression. Foolish writes this as `🧠⟦1 + 2⟧`, and
when working token by token, each token gets its own prefix: `🧠1 🧠2 🧠+`.

So `🧠1 🧠2 🧠+` and `🧠⟦1 + 2⟧` are two ways of writing the same thing. The 🧠 on the
open bracket is part of the Foolish bracket notation — `🧠⟦` opens it. The single-token
prefix form drops the close bracket because each token is self-contained.

This makes the prefix notation a strict shorthand, not just an analogy. A document can use
either form; the token-by-token form is preferred in implementation contexts where individual
FIR instances need to be named.
