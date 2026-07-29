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
(perspectives, MCP) attaches to. It also adds (§S.11) a **layered post-quantum section attestation**:
a conservative SPHINCS+/SLH-DSA signature over a whole stage section (manifest + byte-joined files),
recomputed when the section updates, on top of — never replacing — the existing per-file Ed25519 stamps.

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

**The goal state, stated plainly (once, in one place):** a healthy suite has **no flags, every artifact
signed, and every artifact matching** — output matches checked matches verified (at the suite's level),
and for every stamp the **public signature verifies against the key the passphrase derives** (the
signer is who they claim to be; no computer key masquerading as a human `verified` stamp). Flags are the
explicit exception to "healthy": a flag is a red mark that breaks the suite until a human resolves it.
"Green" therefore means zero flags + all-signed + all-matching + all-signatures-valid; anything less is
not done.

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

**A flag BREAKS THE TEST by default — this is the newly-designed behavior as of FOOP 25.** Previously a
flag only moved an artifact into the `flagged/` sink and it was a matter of interpretation whether that
should fail a run. FOOP 25 makes it definite: **the presence of any flagged artifact for a test fails
that test** (`einmo test` returns non-zero; the gate is red). A suite CAN be configured to not treat
flags as failures — `--flag-is-not-failure` (per-suite config or CLI flag) downgrades a flag from
"failure" to "advisory" — **but even then, a flag ALWAYS produces stderr output announcing its
existence** (`einmo: warning: <N> flagged artifact(s) present: …`). There is no configuration under
which a flag is silent; the most it can be made is non-fatal-but-loud. This keeps flags impossible to
lose: the default punishes them, and the opt-out still shouts.

**Flags break the test and do not diff (per FOOP 64 §Flagging).** A `Flag` is not a comparison against a
baseline; it is a deliberate "this is wrong, stop and look". The reviewer's note is kept **in full and
in context** (they annotate the rendered body right where the error is; the whole annotated text is the
note, not just an added line).

**`flagged/` is PLAINTEXT, UNSIGNED, and TRANSIENT — a development-process component, not a durable
signed record.** This FOOP settles a question the corpus had left open: **flagging writes a plaintext
message with no signature.** A flag is a short-lived "in progress, broken" marker meant to be resolved
and removed, not to persist or be cryptographically attributed. So `EinmoReview` executing a `Flag`
simply writes the note as plaintext into `flagged/<test>` — and re-flagging **concatenates**: the new
dated, annotated content goes ON TOP, the existing flagged content BELOW, in the same path. Because it
is plaintext by design, there is no envelope to corrupt and no verification to fail; `flagged/` remains
**exempt from the escalation** exactly as today. Its only job is to **break the test by default** (S.3,
above) until a human resolves it. A pending `Flag` still replaces on re-edit (normal rule); on execute
it concatenates newest-on-top; concurrent multi-verifier flags serialize under the `exec` mutex so both
dated blocks land, none lost. The journal records who flagged, when, and with what note.

**Durable, attributed observations go in a NEW signed `notes/` stage — not `flagged/`.** For an
observation meant to LAST — a design note, a reviewed finding, an attributable comment that should
survive past the bug it describes — `flagged/` is the wrong home (it is transient and unsigned). This
FOOP adds a `notes/` sibling stage that **is signed** (a proper `.einmo` envelope, verify-on-inspect,
stamped like any stage). The same concatenated annotated content that a flag holds as plaintext can be
promoted into `notes/` as the **signed body of a note** — so a throwaway flag can graduate into a
durable, attributed record. `notes/` participates in signature checks (its stamps must verify against
their passphrase-derived keys, per the goal state); `flagged/` never does. Rule of thumb: **`flagged/`
is for the development loop and should trend to empty; `notes/` is for what you want to keep.**

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

**Multi-stage promotion of one file, one passphrase (subsumes the CLI `::` idea).** Because pending
promotions live in the session's decision set, a reviewer deciding a file needs BOTH
`output->checked` and `checked->verified` (poor_einmo's `\Y`) is just two decisions on one file (or a
`Decision::Promote { to: Verified, through: true }` convenience meaning "carry it up from wherever it
is"). `execute`/`execute_one` then apply the stages **in lifecycle order** (output->checked before
checked->verified — the later hop reads the freshly written checked) under a **single derived
`Signer`**, so the human is prompted at most once for the whole batch, mixed stages included. This is
the durable home of "promote several stages in one go, one passphrase"; a bare-CLI `einmo promote …
:: …` chain (considered under FOOP 35 §S.2b and deferred) would, if ever added, be a thin
argument-parser over this same session primitive — the in-memory decision set is the mechanism, the
`::` syntax merely a shorthand. Ordered-apply-under-one-key lives in the library so every frontend
(bash, server, MCP) inherits it.

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

### S.11 Section-level post-quantum attestation (SPHINCS+)

**Layered, not a replacement.** The per-artifact Ed25519 stamps stay exactly as they are (fast,
per-file, the existing approval chain). This adds a SECOND, coarser signature over a whole **section**
(`output/`, `checked/`, or `verified/` as a unit) using **SPHINCS+ / SLH-DSA** at a **conservative
parameter set** (the large-signature, slow-signing variant — e.g. `slh_dsa_sha2_256s` via the
pure-Rust `fips205` crate; this attestation runs rarely, so size and speed do not matter and we buy the
biggest security margin). Because it is additive, **no existing `.snap`/`.einmo` signature is
invalidated** — the migration pain of a scheme swap is avoided entirely.

**Encapsulated in a `CorpusSigner` object — NOT mixed into `EinmoReview`.** The whole
section-attestation pipeline (build the manifest, read the section in parallel into one buffer, hash,
SLH-DSA sign/verify) is one cohesive responsibility and lives in its own object. `EinmoReview` *uses*
it; it does not contain the logic. This mirrors the S.4 discipline that keeps key custody (`Signer`)
out of the review object — `CorpusSigner` is the section-level analogue.

```rust
/// Owns section-level post-quantum attestation for one suite. Stateless w.r.t.
/// review; given a stage it (re)builds the manifest, reads the section, and
/// signs or verifies. Send + Sync so the server's single review can call it.
pub struct CorpusSigner {
    suite_root: PathBuf,
    params: SlhDsaParams,     // the conservative set, e.g. slh_dsa_sha2_256s
    read_workers: usize,      // bounded read-parallelism (S.11 read pass)
}

impl CorpusSigner {
    pub fn new(suite_root: &Path, params: SlhDsaParams, read_workers: usize) -> Self;
    /// Deterministic manifest for a stage (sorted mirror-paths + sizes/offsets).
    pub fn manifest(&self, stage: Stage) -> Result<SectionManifest>;
    /// Manifest + parallel read + hash → the message digest to sign/verify.
    pub fn digest(&self, stage: Stage) -> Result<SectionDigest>;
    /// (Re)sign a stage's section with the SLH-DSA key; writes `.section.sig`.
    pub fn sign(&self, stage: Stage, signer: &Signer) -> Result<SectionSig>;
    /// Recompute and check a stage's `.section.sig`; Ok(()) or a mismatch error.
    pub fn verify(&self, stage: Stage) -> Result<()>;
}
```

`EinmoReview::execute` holds an `Arc<CorpusSigner>` (or constructs one from the suite) and calls
`sign(stage, signer)` as the final step of promoting into that stage — the review object orchestrates,
`CorpusSigner` does the work. Verification (CLI `einmo verify`, the server, poor_einmo) calls
`verify(stage)` without any review session at all.

**What `CorpusSigner` signs.** For a section, the signed message is built deterministically:

1. A **manifest** header: the stage name, the parameter set id, and the ordered list of included
   mirror-paths. Order is einmo's existing sorted walk (`walk_input_tree` sorts; deterministic), so the
   manifest is reproducible.
2. Then, in manifest order, each file's **bytes byte-joined** onto the running message (the signed
   envelope bytes as they sit on disk — the whole artifact, not just its body).
3. The whole thing is **hashed**, and SPHINCS+ signs that digest. The section signature + its manifest
   live in one file per stage (e.g. `checked/.section.sig` — dot-named, so einmo's walkers skip it).

**Reading the section — parallel, one allocation (bandwidth-maximizing).** The byte-join is a
"load many files into one contiguous buffer" workload; a naïve sequential `read` per file, growing a
`Vec`, wastes both disk queue depth and memory bandwidth. Use the two-pass structure that makes the
read both fast AND deterministic:

1. **Metadata pass** — `fs::metadata(len)` over the manifest-ordered paths to compute each file's size
   and its **offset** in the final buffer; sum to the total. One `vec![0u8; total]` allocation, no
   reallocation or per-file heap churn.
2. **Parallel read pass** — because every file's destination is a **disjoint** `&mut` sub-slice
   (`buffer[offset..offset+len]`), N worker threads can `read_exact` into their slices with **no
   locking and no data races** (Rust's borrow checker witnesses the disjointness via
   `split_at_mut`/chunked slicing). This saturates disk queue depth on many small files and memory
   bandwidth on large ones. A sketch of the sequential core (the parallel version splits the
   `(path, slice)` pairs across a small thread pool / `rayon`):

   ```rust
   // sizes/offsets from the metadata pass; `buffer` is one allocation.
   let mut cur = 0;
   for (path, &size) in paths.iter().zip(&sizes) {
       File::open(path)?.read_exact(&mut buffer[cur..cur + size])?;
       cur += size;
   }
   ```

**Determinism is preserved regardless of read order.** Offsets are fixed by the *manifest* order in
the metadata pass, so which thread finishes first is irrelevant — the buffer's byte layout, and thus
the hash, is identical every run. The parallelism is purely an I/O-throughput optimization over a
layout the manifest already pinned.

**Concurrency caveat (why the metadata pass alone is not the integrity check).** Sizes read in pass 1
could disagree with bytes in pass 2 if the section changed underneath (a concurrent promotion). Guard
it: a `read_exact` short read (file shrank) or leftover bytes (file grew) is a hard error that aborts
the signature; and the section sign runs under `execute`'s write lock (S.2/S.4), which already excludes
concurrent mutation. Verification re-reads the same way and re-checks — a mid-flight change simply
fails verify, which is the correct outcome.

**Bounded, not unbounded, parallelism.** Cap the worker pool (e.g. a small multiple of CPU count, or a
config knob) so a giant section does not spawn thousands of threads; huge individual files can be split
into ranged reads across workers.

**Two read strategies — DEFAULT is fast parallel-buffer; a streaming alternative is also implemented
and tested.** `CorpusSigner` provides two `ReadStrategy` implementations behind one seam, so the same
manifest yields the same digest either way:

- **`ReadStrategy::ParallelBuffer` (default).** The two-pass, massively-parallel read above: one
  allocation, disjoint-slice parallel `read_exact`, then **hand the whole buffer to the signer at
  once**. Maximizes disk/memory bandwidth; the signer (or hasher) sees one contiguous message. This is
  what `sign`/`verify`/`digest` use unless told otherwise.
- **`ReadStrategy::Stream` (alternative).** Reads files sequentially in manifest order and feeds the
  hasher **incrementally** (`update(chunk)` per read block), never materializing the whole section in
  memory. Bounded memory for pathologically large sections, and a cross-check oracle. It is slower but
  must produce a **byte-identical digest** to `ParallelBuffer`.

Both are implemented and unit-tested; a test asserts the two strategies agree bit-for-bit on the same
fixtures (this also pins that a single-threaded path equals the parallel one). The default is
`ParallelBuffer`; `Stream` is selectable (config/flag) for constrained environments or as the
verification oracle. Semantically they are interchangeable — the manifest fixes the byte order, the
strategy only chooses how the bytes reach the signer.

**When it runs.** Whenever the section updates — `EinmoReview::execute`/`execute_one`, promoting into a
stage, calls `CorpusSigner::sign(stage, signer)` as its final step (execution already holds the write
lock and the `Signer`). Verification calls `CorpusSigner::verify(stage)`, which recomputes the
manifest+hash and checks the SLH-DSA signature; a mismatch means a file was added, removed, reordered,
or altered under the section as a whole — integrity above the per-file level.

**Keys.** By default the **same passphrase** derives BOTH the existing Ed25519 stamp key and the
section SPHINCS+ key (via the S.4 `Signer`, extended to expose both a per-file Ed25519 signer and a
section SLH-DSA signer from one derivation). SPHINCS+ keygen takes a seed; the Argon2id output is
expanded to the required seed length and fed to deterministic keygen, preserving einmo's
"same passphrase ⇒ same key" invariant. A future option may separate the two keys, but same-passphrase
is the default.

**Scope for THIS FOOP: crypto core + tests only.** Build and unit-test the section-signature primitive
(manifest builder, deterministic hash, SLH-DSA sign/verify, same-passphrase dual derivation) as a
self-contained module. Do NOT wire it into the live promotion flow or write `.section.sig` into the
real corpus yet — that corpus-touching integration is a later step (it interacts with the FOOP 64
gate and human re-sign discipline). The primitive is proven in isolation first.

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
- **Unit — section PQ attestation (S.11, crypto core only)**: manifest is deterministic for a fixed
  file set (reorder inputs on disk → same sorted manifest → same message); adding/removing/altering one
  file changes the signed digest; SLH-DSA sign→verify round-trips; a tampered signature or a changed
  file fails verify; **same passphrase derives both** the Ed25519 stamp key and the section SLH-DSA key
  (dual-derivation determinism: same passphrase ⇒ same section pubkey across runs); empty-section
  manifest is well-formed. NO real-corpus writes in this FOOP — pure module tests over fixtures.
- **Unit — CorpusSigner read strategies (S.11)**: `ParallelBuffer` (default) and `Stream` produce a
  **byte-identical digest** over the same fixture set, independent of worker count and read completion
  order; the parallel two-pass buffer has exactly `sum(len)` bytes with each file at its manifest
  offset; a file that shrinks between the metadata and read pass (short read) or grows (leftover bytes)
  is a hard error, not a silent mis-hash; `Stream` holds bounded memory (never materializes the whole
  section). Stress with a mix of many tiny files and a few large ones. `CorpusSigner` is exercised as a
  standalone object (no `EinmoReview`), proving the encapsulation.
- **Unit — execute**: plan/execute equivalence with CLI `einmo promote` byte-for-byte; skip-and-report
  on mid-plan drift; retract cascade; exclusive exec under concurrent decide traffic (no lost updates).
- **Unit — flag = plaintext, concatenating (S.3)**: `flagged/<test>` is PLAINTEXT, unsigned; executing a
  `Flag` on a fresh test writes the annotated note as plaintext; re-flagging CONCATENATES the new dated
  block ON TOP of the existing content (same path); two reviewers flagging the same test → both dated
  blocks present, ordered, none lost (serialized by the exec mutex); a pending `Flag` replaces on
  re-edit; `flagged/` stays exempt from verification (a plaintext/broken file there fails no gate); the
  journal has both flag events.
- **Unit — signed `notes/` stage (S.3)**: a note promoted into `notes/` is a valid SIGNED `.einmo`
  (verify-on-inspect passes; stamp verifies against the passphrase-derived key); the same concatenated
  content that was a plaintext flag can be signed as a note's body; `notes/` participates in signature
  checks while `flagged/` does not.
- **Unit — flag breaks tests (S.3)**: a flagged artifact makes the run FAIL by default (non-zero exit /
  red gate); `--flag-is-not-failure` downgrades it to non-fatal BUT stderr still announces the flag
  count; there is no config that makes a flag silent; the goal-state check (zero flags + all signed +
  all matching + all signatures valid against their passphrase-derived keys) is green only when no flags
  exist.
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
- Parallel section-read (§S.11): `rayon` (ergonomic, another dep) vs a small hand-rolled std
  thread-pool (keeps the crate leaner). Either way the single-threaded fallback must be byte-identical.
  Also: worker-count default / config knob, and whether to range-split individual very large files.

## References

- **FOOP-15** — Secured interactive einmo review: this FOOP supplies the session/state layer its
  R1–R4 phases attach to; R4 (MCP) and R3 (perspectives SPA) are explicitly *not* duplicated here.
- **FOOP-64** — the einmo suite migration and `poor_einmo.sh`, whose review protocol (verbs,
  replace-not-stack, `u`-revisit, PROMOTE gate, `-d` default, top info panel) is the behavioral
  prototype this FOOP libraries-ize.
- **FOOP-92** (Complete) — einmo itself; `einmo.README.md` (three-role keys, verify-on-inspect).
- Code: `einmo/src/{einmo_suite,transitions,signature,verify,format,compare}.rs`; `poor_einmo.sh` at
  the repository root (as of branch `foop-64-einmo-suite`).
