---
foop: 51
title: Secured interactive einmo review — cryptographically attested inspection of einmos and their perspectives
author: Atlas <hc.busy@gmail.com>
status: Draft
type: Standards
created: 2026-07-14
phase: meta
supersedes: []
begun: [ ]
---

# FOOP-15: Secured interactive einmo review — cryptographically attested inspection of einmos and their perspectives

FOOP numbering is little-endian; the full rules live in `foop.md` at the
repository root — **read it before creating or editing a FOOP.**

> **Roadmap note:** this FOOP is the deliberate home of FOOP-92's deferred interactive tooling
> (serve/SPA, MCP server, rich perspective rendering). It is an important feature and the
> project's stated goal is to **build up to it** — it runs any time after Track 0 (it touches
> only the einmo crate family, not the FVM), and its phases are sized so each lands value alone.

## Abstract

A **cryptographically secured interactive system for reviewing einmos and their various
perspectives**. Every artifact shown is verified-on-inspect before rendering; every action a
reviewer takes (promote, flag, re-inspect) is a signed, attributable stamp; the interface
renders not just INPUT/OUTPUT but the einmo's **perspectives** (derived signed sections — the
Charmer aspects, the brane-name perspective, future language-specific views) side by side with
the primary output. Three frontends over one backend library: the CLI (`console-review`,
delivered by FOOP-64), an HTTP/WebSocket service with an SPA, and an MCP server for agent
reviewers — all calling the same einmo library so the invariants hold identically for humans
and machines.

## Motivation

Signed snapshots are only as trustworthy as the review that promotes them. Today review is
file-level diffing; the reviewer mentally reconstructs what changed and why. The einmo format
already carries the ingredients for much better review — verified stamp chains (who produced
this, who promoted it), perspectives (views of the same output for different reader needs), and
staged correspondence (what drifted between output/checked/verified). This FOOP turns those
ingredients into an interactive system where inspecting is safe by construction (tampered files
are refused, never rendered), acting is attested (a promotion from the UI appends the same
signed stamp the CLI would), and perspectives make review legible (a reviewer approves what
they actually understood, and their signature says so).

## Specification (phased — build up to it)

### Phase R1 — read-only inspection service
`einmo serve <work_dir>` (axum, loopback): `/api/tree` (suite overview with per-file stage
badges + signature status), `/api/show` (verified envelope + stamp-chain summary), `/api/diff`
(per-section diff between any two stages, signature lines hidden). **Verify-on-inspect
server-side on every read**; a tampered file returns an alert object, never content. No actions
yet; no key material in the process beyond verification (pure Ed25519 checks).

### Phase R2 — attested actions
`POST /api/promote`, `POST /api/flag`: the passphrase arrives in the request body, is derived
to a key, used to sign, and dropped — the server never stores key material. Non-human
attestation detection surfaces in the UI exactly as in the CLI (empty-passphrase key on a
`verified` stamp → visible warning). Randomized re-inspection (`--reexamine-rate`) drives a
review queue.

### Phase R3 — perspectives-rich review
Render each einmo's perspective sections beside the primary OUTPUT (Charmer aspects; the
brane-name perspective; the einmo-in-HTML metadata convention for HTML-bearing outputs, per
FOOP-92 §C.4). The **byte-steadiness invariant** governs: the signed bytes are immutable; all
interactivity (folding, search, highlighting) is a non-mutating view layer over them. Approve /
flag buttons sit on the rendered artifact and call Phase R2's endpoints.

### Phase R4 — MCP server for agent reviewers
`einmo-mcp` (or `einmo serve --mcp`): `einmo_list`, `einmo_show`, `einmo_diff`,
`einmo_promote`, `einmo_flag`, `einmo_verify`, `einmo_confirm_signatures`, `einmo_root_cause`
as structured tools over the same library. Ships with the AGENTS.md review-flow skill templates
(reconcile output-vs-checked; burden-of-correction; flag-vs-escalate) so agent review
discipline is loaded, not improvised.

### Security invariants (all phases)
1. Verify-on-inspect before any render or action — no fast path.
2. Every state-changing action appends a signed stamp through the einmo library — the UI/MCP
   never write `.einmo` bytes directly.
3. Key material: derived, used, dropped; never persisted, never logged.
4. The signed bytes are never mutated by presentation (byte-steadiness).

## FIR Impact
None — einmo crate family only.

## UBC Step Impact
None.

## Test Plan
Per phase: endpoint tests against a tempdir suite (tampered file → refused; promote via API ==
promote via CLI byte-for-byte); UI/MCP action stamps verified by `einmo verify --all`;
R3 renders golden perspective fixtures; the security invariants each get a negative test
(no-store of passphrase, no render of tampered content, no direct byte writes).

## Rejected Alternatives

### A. Keep review CLI-only forever
The corpus is growing past what vimdiff review scales to (162 inputs and climbing; FOOP-64's
migration is itself a mass-review event), and perspectives are unreviewable as raw text blocks.

### B. Third-party review UI (generic snapshot viewers)
None verify stamp chains or produce signed promotions; bolting signing onto a foreign UI breaks
invariant 2.

## Open Questions
- SPA stack (the FOOP-92 draft suggested Vite+React or SvelteKit via `rust-embed`) — decide at
  R3 time.
- Whether R4's MCP server is a binary in the einmo crate or a sibling crate.
- Auth story beyond loopback binding for R2 (per-reviewer identity is a future extension; the
  stamp passphrase is the attestation today).

## References
- FOOP-92 (Complete) §Use Case C and its plan Phases 12/13 — the deferred material this FOOP
  re-homes; `einmo.README.md` (perspectives, Charmer, three-role keys, verify-on-inspect).
- FOOP-64 (Track 0) — delivers `console-review`, the CLI predecessor of this system.
