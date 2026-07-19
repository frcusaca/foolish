---
foop: 52
title: EinmoReview — a thread-safe review-session object; thin bash, server, and dhtml frontends
author: Atlas <hc.busy@gmail.com>
status: Draft
type: Standards
created: 2026-07-19
phase: meta
supersedes: []
begun: [ ]
---

# FOOP-25: EinmoReview — a thread-safe review-session object; thin bash, server, and dhtml frontends

FOOP numbering is little-endian; the full rules live in `foop.md` at the
repository root — **read it before creating or editing a FOOP.**

## Abstract

Extract the einmo review *session* — the worklist, the reviewer's evolving decisions, verified-body
access, and deliberate signed execution — out of `poor_einmo.sh` and into a thread-safe Rust object,
`EinmoReview`, in the einmo crate. Key custody (passphrase → key → sign) is deliberately **not** part of
the review object: it lives in a separate `Signer` object that the review *uses* at execution time,
supporting both promote-one-at-a-time and accumulate-then-sign-at-the-end from a single passphrase
entry. One running review is exposed through a small server API; `poor_einmo.sh` shrinks to a thin,
fast client (vim stays the editor), and a dhtml page talking to the same server replaces vimdiff as the
first browser frontend. This FOOP is the session layer that FOOP 15's secured interactive review
(perspectives, MCP) attaches to.

## The Aspirational Goal

**One review, one object, every surface a thin view.**

A review session should be a first-class thing: it knows which tests need attention, what every stage's
verified body says, what the reviewer has decided so far, and what will happen when those decisions are
executed. Humans and agents should meet the *same* object through whatever surface is at hand — a bash
loop driving vim, a browser page, an MCP tool — and the invariants (verify-on-inspect, replace-not-stack
decisions, deliberate attested execution) should hold identically everywhere because they are enforced
in exactly one place. Verification is paid once and remembered, so review runs at the speed of reading,
not the speed of re-verifying. Signing stays a human act with its own object, its own lifetime, and its
own confirmation — never a side effect. And the corpus's story ("who decided what, when, and with which
key") is written down as it happens.

When this FOOP is complete, `poor_einmo.sh` is a dumb terminal loop an evening's read long, the browser
page is a courtesy view over the same API, and the next frontend — FOOP 15's perspective-rich SPA or an
agent reviewer — costs an afternoon, not a rewrite.

## Motivation

FOOP 64's review stopgap (`poor_einmo.sh`) proved the review *protocol* — panes, verbs,
replace-not-stack decisions, revisits, the PROMOTE gate — and every lesson was learned the hard way, in
bash arrays:

- The revisit/undo machinery (`undo_last_decision`, `answer_of`, `drop_from`) is ~80 lines of shell
  reimplementing "a map from test to current decision".
- Loop-control bugs (a `continue` that re-opened the same test forever) and ordering bugs (`show_cmd`
  called before its definition) shipped and were found by users — state machines in bash have no tests.
- Every run re-verifies every stamp chain via 3 `einmo body` spawns per test (~500 process spawns per
  full pass of the 161-test suite). The review is slower than it has any right to be.
- The pane-verb protocol ("type `promote` as the whole pane") exists only because vim has no action
  channel back to bash. It works, but it is a workaround, not a design.

The session logic wants to be a library with unit tests; the speed wants a resident process that
verifies once; the UIs want a real action channel. All three are the same refactor.

## Supported Use Cases

1. **The solo loop, faster** — a Foolisher runs `poor_einmo.sh`; between tests the bodies arrive from
   the server's verified cache in milliseconds instead of hundreds of ms of spawn+verify.
2. **Accumulate, then sign once** — review 40 tests, decide on each, and at the end type the PROMOTE
   confirmation and one passphrase; every checked→verified promotion is signed from that single entry.
3. **Promote as you go** — for a long session, execute each decision immediately after it is made; the
   session-scoped `Signer` (derived once) makes per-item signing as cheap as batch.
4. **Browser review** — the same 4-pane inspection (input│output│checked│verified) as a dhtml page:
   server-computed diffs, verbs as buttons, the gate as a typed confirmation. Replaces vimdiff for
   reviewers who prefer a browser; vim remains fully supported.
5. **Multiple verifiers, concurrently** — two humans (or a human and an agent) review the same suite at
   once; decisions are per-reviewer, stamps accumulate (two `verified` stamps are *stronger*, not a
   conflict), and soft claims prevent duplicated effort.
6. **Agent reviewers** — an AI agent lists, inspects, decides, and (with its own key) executes through
   the same API; FOOP 15 R4 wraps this as MCP tools.
7. **Resume after a crash** — the journal replays a session's decisions; nothing a reviewer decided is
   lost to a dropped ssh connection.
8. **Audit** — "who decided what, when, with which key" is answerable from the journal plus the stamp
   chains, per file.

## Specification

### S.1 The three layers

```
einmo core (exists)   format · signature · verify · stage · transitions · compare · EinmoSuite
einmo::review (NEW)   EinmoReview — session state, decisions, cache, plan/execute, journal
frontends (thin)      CLI verbs · `einmo review serve` · poor_einmo.sh (thin) · dhtml · (FOOP-15: MCP/SPA)
```

All frontends call the same `EinmoReview`; no frontend writes `.einmo` bytes or touches key material.

### S.2 The `EinmoReview` object

Thread-safe by construction (`Send + Sync`); the server holds one `Arc<EinmoReview>`. Interior
mutability is partitioned by contention:

```rust
pub struct EinmoReview {
    suite: EinmoSuite,               // immutable after open()
    worklist: RwLock<Worklist>,      // read-mostly; refresh() takes the write lock
    cache: VerifiedCache,            // fingerprint -> verified body; single-flight verification
    decisions: RwLock<DecisionBook>, // per-item, per-reviewer, versioned
    journal: Journal,                // Mutex<append-only writer>
    exec: Mutex<()>,                 // execution (disk mutation + signing) is exclusive
}

impl EinmoReview {
    pub fn open(suite: &Path, opts: ReviewOpts) -> Result<Self>;   // opts: differing_only, filter
    pub fn items(&self) -> Vec<ReviewItem>;                        // worklist rows + current decisions
    pub fn body(&self, m: &MirrorPath, s: Stage) -> Result<Arc<VerifiedBody>>;
    pub fn diff(&self, m: &MirrorPath, l: Stage, r: Stage) -> Result<DiffHunks>;
    pub fn decide(&self, r: ReviewerId, m: &MirrorPath, d: Decision) -> Result<Option<Decision>>;
    pub fn undecide(&self, r: ReviewerId, m: &MirrorPath) -> Option<Decision>;
    pub fn decision(&self, r: ReviewerId, m: &MirrorPath) -> Option<Decision>;  // "answer so far"
    pub fn plan(&self, r: ReviewerId) -> ExecutionPlan;            // pure preview
    pub fn execute(&self, plan: &ExecutionPlan, keys: &SignerSet) -> ExecutionReport;   // batch
    pub fn execute_one(&self, r: ReviewerId, m: &MirrorPath, keys: &SignerSet) -> Result<Executed>;
    pub fn refresh(&self) -> Vec<MirrorPath>;                      // rescan; stale decisions flagged
}
```

**Single-flight verification**: `VerifiedCache` maps `Fingerprint → Arc<OnceLock<VerifiedBody>>`. The
map lock is held only to fetch/insert the entry; verification runs inside `get_or_init` outside the map
lock — concurrent readers of the same artifact trigger exactly one stamp-chain verification and never
block readers of other files. Verify-on-inspect is preserved (nothing renders unverified); it is paid
once per byte-content, not once per look.

### S.3 Decisions — replace, never stack

```rust
pub enum Decision {
    Promote { to: Stage },              // output->checked | checked->verified
    Retract { from: Stage },            // checked cascades to verified (library enforces)
    Flag    { stage: Stage, reason: String },
    Skip,                               // looked, deliberately chose not to rule
}
// DecisionBook: MirrorPath -> { ReviewerId -> (Decision, version) }; absence = untouched.
```

`decide` replaces that reviewer's previous decision and returns it; `undecide` clears it; absence means
untouched. This map-shaped invariant replaces poor_einmo's entire `drop_from`/`undo_last_decision`/
`answer_of` machinery. Every item carries a `version` bumped on decision change or byte change;
frontends send it back (If-Match) so a stale view cannot silently decide about changed content.

### S.4 Signing is a separate object — the design answer

**Question posed**: should signing-from-passphrase (individually or in batch) be part of the review
process, or a separate object? **Answer: a separate object.** The review object holds *decisions*; a
`Signer` holds *key custody*. They meet only at execution:

```rust
pub struct Signer { /* Argon2id-derived Ed25519 key; zeroized on drop */ }
impl Signer {
    pub fn from_passphrase(pass: Passphrase) -> Signer;   // derive once; pass is consumed & wiped
    pub fn computer() -> Signer;                          // the empty-passphrase computer/agent key
}
pub struct SignerSet { pub checked: Signer, pub verified: Option<Signer> }
```

Rationale for the separation:

- **Different lifetimes.** Decisions live for the whole session and survive crashes (journal); key
  material should live as briefly as possible and never be persisted. One object cannot honor both.
- **Different owners.** A server can hold the review for many verifiers, but a key belongs to one
  human. With a separate `Signer`, the server stages decisions all day without ever touching key
  material; the passphrase enters only inside an execute call, is derived, used, dropped (FOOP 15
  security invariant 3).
- **Individual vs batch collapses into one design.** `execute_one` and batch `execute` take the same
  `&SignerSet`. A session-scoped signer derived once makes per-item signing as cheap as batch — the
  reviewer chooses cadence, not cost. Deriving per-call remains possible (highest caution mode).
- **Attestation stays honest.** stage promotions to `checked` may use the computer key; promotions to
  `verified` require a human signer — the `SignerSet` shape makes that rule visible in the types.

Execution is always deliberate: `plan()` renders exactly what will run (today's results block, kept),
and the frontend must present it and pass an explicit confirmation (the typed `PROMOTE` word survives as
the API's `confirm` token). Retractions carry their own confirmation and are never batched silently.

### S.5 Concurrency semantics for multiple verifiers

- Per-reviewer decisions coexist; replace-not-stack holds *within* a reviewer. Executing appends that
  reviewer's stamps; a second verifier executing later appends theirs. Multiple `verified` stamps are
  accumulated attestation, surfaced via `Stamps::stamped_by`.
- Soft claims (`claim(m, ttl)`) advertise "I'm on this one" in listings; advisory only, cannot wedge.
- The `exec` mutex serializes disk mutation; each write re-checks the file fingerprint first —
  anything drifted since planning is skipped-and-reported, never clobbered.
- An advisory lockfile makes a second *server* on the same suite refuse to start; external CLI
  mutations are caught by `refresh()`.

### S.6 The journal

Append-only JSONL per session (dot-named inside the suite or under a scratch dir — decided at
implementation; einmo's walkers skip dot entries): session id, reviewer, timestamp, produced_by, every
decide/undecide/claim/execute with outcomes. Reopen = replay. This is the audit and crash-recovery
substrate, and what a later quorum policy ("verified needs 2 distinct human stamps") reads.

### S.7 The server — one running review

`einmo review serve <suite>`: binds a **unix-domain socket by default** (`curl --unix-socket`; inherits
directory permissions — the mode-700 discipline poor_einmo already established), TCP on 127.0.0.1 with a
bearer token only when a browser needs it. Handlers are thin translations onto `Arc<EinmoReview>`.

| Method | Path | Meaning |
|--------|------|---------|
| GET    | `/api/review`                        | session summary: counts, cursor, dirty, verifiers |
| GET    | `/api/review/items?differing&filter=`| worklist rows incl. per-reviewer decisions |
| GET    | `/api/review/items/{m}`              | item detail; `version` for If-Match |
| GET    | `/api/review/items/{m}/body/{stage}` | verified body (ETag: fingerprint) |
| GET    | `/api/review/items/{m}/diff?l=&r=`   | hunks between stages, stamp lines excluded |
| PUT    | `/api/review/items/{m}/decision`     | decide (If-Match: version → 409 when stale) |
| DELETE | `/api/review/items/{m}/decision`     | undecide |
| POST   | `/api/review/items/{m}/claim`        | soft lease (TTL) |
| GET    | `/api/review/plan`                   | structured plan + rendered results-block text |
| POST   | `/api/review/execute`                | `{confirm:"PROMOTE", scope: all\|[m…], passphrase?}` |
| POST   | `/api/review/refresh`                | rescan; returns changed items |
| GET    | `/api/review/events`                 | SSE: decision-made / item-changed / executed |

The passphrase appears only in the execute request body (or is typed at the server's own terminal when
executing via CLI), is derived into a `Signer`, used under the `exec` mutex, and dropped.

### S.8 The reduced `poor_einmo.sh`

The script keeps exactly what bash+vim are good at and sheds all session state:

- **Keeps**: the per-test loop, the vim invocation (top info panel + 4 tiles, `\d`/`\D`/`\i`/`\I`,
  statusline), reading the reviewer's pane intent, the between-tests prompt.
- **Sheds**: decision arrays and all undo/answer bookkeeping (→ PUT/DELETE/GET decision), body
  rendering and verification (→ GET body from cache), the differing computation (→ server), results
  rendering (→ GET plan), the gate execution (→ POST execute), stats-table computation (→ item detail).
- Server discovery via the suite's socket file; **no server → the current direct-`einmo` path remains
  as a degraded fallback** (one `fetch_body`-style switch), so the script never hard-depends on the
  server.
- Success measure: script size roughly halves (~700 → ≤350 lines), and a full no-decision pass over
  the 161-test suite performs zero stamp verifications after the server's first pass (spawn count per
  test drops from 3 einmo processes to 3 socket reads).

### S.9 The dhtml frontend

A single self-contained page embedded in the binary (`include_str!`), served by the same server: the
4-pane layout with server-computed diff hunks (one diff implementation — `compare.rs` — everywhere),
verb buttons, a notes box (→ `Flag`), the plan view with the typed-PROMOTE gate, SSE-driven refresh so
concurrent verifiers see each other's decisions and claims live. Byte-steadiness per FOOP 15: signed
bytes are never mutated by presentation. No framework required at this phase; FOOP 15 R3's SPA decision
is unaffected.

### S.10 Drift tolerance

Einmo will likely evolve before this FOOP is undertaken (FOOP 64 is still landing; formats and CLI
surfaces may shift). This specification therefore binds to einmo's *behaviors* — stages, stamp chains,
verify-on-inspect, body extraction, promotion/retraction/flag transitions — not to exact function
signatures. The plan's first implementation task is a re-survey of `einmo/src` (`einmo_suite.rs`,
`transitions.rs`, `signature.rs`, `verify.rs`, `format.rs`, `compare.rs`) with spec touch-ups before any
code. The Rust sketches above are shape, not letter.

## FIR Impact

None. Einmo crate family and one shell script only.

## UBC Step Impact

None.

## Test Plan

Tests are written first, per project rules.

- **Unit — decisions**: replace-not-stack (second `decide` returns the first); `undecide` then
  unchanged pass = untouched; per-reviewer isolation; version bump on decide and on byte change;
  If-Match/409 on stale version.
- **Unit — cache**: N threads requesting one artifact → exactly one verification (test hook counter);
  tampered file → refused object, never content; fingerprint change invalidates.
- **Unit — signer**: derive-once reuse across N signings; zeroize on drop (best-effort assertion);
  computer vs human key selection per stage; passphrase never reachable after construction.
- **Unit — execute**: plan/execute equivalence with CLI `einmo promote` byte-for-byte; skip-and-report
  on mid-plan drift; retract cascade; exclusive exec under concurrent decide traffic (no lost updates).
- **Journal**: replay reconstructs the DecisionBook exactly; a truncated tail (crash) replays cleanly.
- **Server**: endpoint tests against a tempdir suite (list/body/diff/decide/plan/execute/SSE); UDS
  permission inheritance; 409 flows; token required on TCP.
- **Thin client**: a pty-driven end-to-end run of the reduced `poor_einmo.sh` against a live server
  (the stub-vim technique from FOOP 64's review work): promote, note→flag, `u`-revisit keeps answer,
  gate skip/confirm; plus the no-server fallback path.
- **Comprehensive test, adapted**: this is a meta/tooling FOOP with no FVM surface, so the reserved
  `foop_25_comprehensive.foo` language test does not apply; its obligation is discharged by a
  comprehensive end-to-end integration test — a scripted multi-verifier session (two reviewers, mixed
  individual/batch execution, one crash-resume, one drift) over a fixture suite, asserted against the
  resulting stamp chains via `einmo verify`.

## Rejected Alternatives

### A. Signing inside `EinmoReview`

Fold `Signer` into the session (review holds the derived key after first passphrase entry). Rejected:
the lifetimes and owners differ (S.4) — a server-held review would hold human key material for the
whole session, violating derive-use-drop; testing key custody would entangle with decision logic; and
per-reviewer keys in one shared object invite cross-signing bugs. The separation costs one extra
parameter at the two execute calls.

### B. Stateless server (re-verify per request)

Simplest server: every request re-runs `einmo` logic like the CLI does. Rejected: it re-imports the
exact cost this FOOP exists to remove — poor_einmo's slowness *is* repeated verification; a stateless
server makes the browser UI equally slow and makes multi-verifier coordination (versions, claims,
events) impossible.

### C. Keep growing `poor_einmo.sh`

Continue enriching the bash script (it works today). Rejected on this session's own evidence: the
revisit machinery, the infinite-loop `continue` bug, and the `show_cmd` ordering bug are all
state-machine defects bash cannot unit-test. The script's proper role is a thin terminal frontend.

### D. Do nothing

Review remains vimdiff-over-temp-files with per-run re-verification. Rejected: the corpus is growing
(161 and climbing per FOOP 64), FOOP 15 needs a session substrate anyway, and every future frontend
would re-implement review semantics.

## Open Questions

- HTTP stack for `serve`: `tiny_http`-class minimal (fits UDS-first, dependency-light) vs committing to
  axum now (FOOP 15 R1 names axum). Decide jointly with FOOP 15 at begun-time.
- Journal location: dot-file inside the suite (travels with the corpus, git-visible) vs scratch/state
  dir (ephemeral). Leaning suite-dot-file for auditability; confirm with human.
- Claim lease TTL default and whether claims appear in `plan()` output.
- Quorum policies (N-of-M human stamps for `verified`) — in scope here or a follow-up FOOP?
- Whether `ReviewOpts.differing_only` defaults on (matching poor_einmo's new `-d` default).

## References

- **FOOP-15** — Secured interactive einmo review: this FOOP supplies the session/state layer its
  R1–R4 phases attach to; R4 (MCP) and R3 (perspectives SPA) are explicitly *not* duplicated here.
- **FOOP-64** — the einmo suite migration and `poor_einmo.sh`, whose review protocol (verbs,
  replace-not-stack, `u`-revisit, PROMOTE gate, `-d` default, top info panel) is the behavioral
  prototype this FOOP libraries-ize.
- **FOOP-92** (Complete) — einmo itself; `einmo.README.md` (three-role keys, verify-on-inspect).
- Code: `einmo/src/{einmo_suite,transitions,signature,verify,format,compare}.rs`; `poor_einmo.sh` at
  the repository root (as of branch `foop-64-einmo-suite`).
