# Einmo

> Gold-standard snapshot testing with discrete, cryptographically signed promotion.

## What is Einmo?

Einmo is a standalone Rust crate for snapshot-style testing where every
generated output is a **signed artifact** and promotion between review stages
is a **deliberate, attributable act** -- never an automated accept.

The genre goes by several names (see below). Whatever you call it, the
universal lifecycle is the same: a machine generates output, a human reviews
the diff, and the reviewed output becomes the baseline that future runs are
checked against. Einmo's contribution is to make each step of that lifecycle
**cryptographically attested** and **separately inspectable**, so that an
auditor can tell who produced a baseline, who reviewed it, and whether the
reviewer was a human or a machine.

### What einmo adds

- **Four-stage promotion pipeline**: `output` -> `checked` -> `flagged` /
  `verified`. Each stage is a directory; each promotion appends a signed stamp.
- **Ed25519 + Argon2id signing**: every `.einmo` file carries a tamper-evident
  stamp chain. No surveyed framework signs its snapshots.
- **Verify-on-inspect**: every read verifies all signatures first. A tampered
  file is refused, never operated on.
- **Directory-based hierarchical storage**: stage directories mirror the
  `input/` tree at any depth, decoupling test organisation from test code.
- **No automated accept**: there is no `--accept`, no `--update`, no
  `INSTA_UPDATE` equivalent. Promotion is always a CLI command.
- **Catastrophe crumb defense**: a signed "test in progress" file is written
  before evaluation. If the process crashes, the crumb survives as forensic
  evidence.
- **Duration and depth limits**: per-test timeouts, per-suite timeouts, and
  configurable recursion depth -- all env-var accessible for CI.

Einmo does not aim to replace any existing tool. It targets projects that need
stronger attestation than an unsigned `.snap` file provides -- VM reference
implementations, security-sensitive approval suites, and any codebase where
distinguishing "a machine generated this" from "a human signed off on this"
matters.

## The testing style

The genre has four names, one mechanism:

| Name | Origin | Core idea |
|---|---|---|
| **Characterization test** | Michael Feathers, *Working Effectively with Legacy Code* (2004) | Document actual behaviour, not desired behaviour. |
| **Approval test** | Llewellyn Falco, ApprovalTests (2008) | A machine generates output; a human approves or rejects it. |
| **Golden master** | Record industry metaphor | Capture the known-good output of a system you may not fully understand. |
| **Snapshot test** | Jest / web community | The most common name today; the "snapshot" is the approved baseline file. |

The philosophical core: a machine can only verify "the received output equals
the approved file." It can never verify "this output is the behaviour we
want." The genre trades the oracle problem for a review problem.

## Other implementations

Einmo is not the first tool in this space. The following table (adapted from
the FOOP-92 research appendices) surveys the landscape:

| Ecosystem | Tool | Storage | Review model |
|---|---|---|---|
| JS/TS | [Jest](https://jestjs.io/docs/snapshot-testing) `toMatchSnapshot` | `__snapshots__/*.snap` | `-u` overwrites; 1 state, no review gate |
| JS/TS | [Vitest](https://vitest.dev/guide/snapshot.html) | `__snapshots__/*.snap` | `-u`; refuses to write in CI |
| Python | [Syrupy](https://github.com/tophat/syrupy) | `__snapshots__/*.ambr` | `--snapshot-update` |
| Rust | [insta](https://crates.io/crates/insta) | `.snap` / `.snap.new` | `cargo insta review` TUI; 2 states, review == promotion |
| Rust | [expect-test](https://crates.io/crates/expect-test) | inline `expect!["..."]` | `UPDATE_EXPECT=1` |
| Rust | [trycmd](https://crates.io/crates/trycmd) | `.stdout` / `.stderr` | `TRYCMD=overwrite`; elision for volatile values |
| Ruby | [ApprovalTests.Ruby](https://github.com/approvals/ApprovalTests.Ruby) | `.received` / `.approved` | `approvals verify --ask` |
| Go | [cupaloy](https://github.com/bradleyjkemp/cupaloy) | `testdata/*.golden` | `-update`; updating *fails* the test to block CI auto-update |
| JVM | [ApprovalTests.Java](https://github.com/approvals/ApprovalTests.Java) | `.received` / `.approved` | Pluggable reporters; scrubbers |
| Swift | [SnapshotTesting](https://github.com/pointfreeco/swift-snapshot-testing) | `.snap` / images | `record` mode |

**None of these cryptographically signs snapshot files.** That is the gap
einmo fills. insta comes closest with its two-state `.snap` / `.snap.new`
model and `cargo insta review` TUI, and has even added [non-interactive review
for LLMs/CI](https://github.com/mitsuhiko/insta/pull/815) -- but there is no
signing, no human-vs-machine key distinction, and no separation between "I
reviewed this" and "I promoted this."

For projects where a signed, auditable promotion chain is not needed, insta
and expect-test remain excellent choices. Einmo is for when attestation
matters.

## The Four Stages

| Stage | Directory | What it holds | Who writes it |
|---|---|---|---|
| **Output** | `output/` | Generated test results (signed by test runner) | `EinmoSuite::evaluate` |
| **Checked** | `checked/` | Reviewed outputs (AI or human promoted) | `einmo promote output->checked` |
| **Flagged** | `flagged/` | Set aside (terminal sink) | `einmo flag <stage>` |
| **Verified** | `verified/` | Human-signed (passphrase required) | `einmo promote checked->verified` |

Each stage directory mirrors the `input/` tree at any depth. An input file
like `stage1/section3/specific.foo` produces
`output/stage1/section3/specific.foo.einmo`, and the same relative path is
used in every other stage directory.

## The `.einmo` File Format

A `.einmo` file is a header line, followed by sections separated by a
configurable separator, ending with the JSON STAMPS section and an optional
unsigned advisory line:

```
#einmo 1 encoding=utf-8 separator=①\n
test: <test-name>
suite: <suite-name>
producer: <git-sha>
producer-diff: <sha256-of-git-diff-or-omitted>
generated: <ISO8601>
status: normal
status-detail: 
sections: INPUT, OUTPUT, COMMENTS, STAMPS
①
<input content>
①
<output content>
①
<comments content>
①
<stamps JSON lines>
# flagged: <reason> <timestamp>   (optional, unsigned advisory)
```

**Header line**: declares the format version (`1`), the encoding (`utf-8`),
and the escaped separator string.

**Metadata section**: fixed key/value lines in a byte-stable order (`test`,
`suite`, `producer`, optional `producer-diff`, `generated`, `status`,
`status-detail`, optional `reference`, `sections`). The order never changes,
so signatures cover a stable byte sequence.

**Body sections**: separated by the configurable separator. Default is `①\n`
(U+2460 followed by LF). Foolish suites use `!!\n` (a Foolish line comment
followed by LF). The `sections:` metadata field declares the ordered list of
section names, and the number of declared body sections must match what
appears on disk.

**STAMPS section**: JSON-lines, one stamp object per line. Each stamp records
its role key, hex pubkey, what it signs, the base64 signature, producer
provenance, and a timestamp.

**Advisory line**: an optional `# flagged: <reason> <timestamp>` line may
appear after STAMPS. It is stripped before verification and is not part of
the signed bytes, so flagging a file does not invalidate its stamp chain.

**Separator collision rule**: serialization refuses if any section body or
the metadata contains the configured separator. This keeps parsing byte-exact
and unambiguous. When it happens, configure a different separator via
`TestConfig::with_separator`.

## The Three-Role Key Model

Every `.einmo` file carries a chain of Ed25519 stamps built from three key
roles:

- **Compiled Key**: embedded in the binary at compile time. In the stock
  open-source build it is a deterministic, publicly-known keypair derived from
  a fixed seed passphrase. Its stamp *certifies* the Configured Key's public
  key.
- **Configured Key**: set at configuration time (defaults to the
  empty-passphrase key). Its stamp *certifies* the Stage Key's public key.
- **Stage Keys**: one per stage, resolved through the passphrase cascade.
  Each `stage:<name>` stamp signs **all file bytes before its own line**,
  forming an append-only integrity chain.

The two certification stamps sign only small, constant-size public keys.
Content integrity comes solely from the stage-key stamps, which cover every
prior byte. A promotion appends exactly one destination-stage stamp and never
touches existing stamps.

**Emergent human-attestation**: the `verified` stage is deliberately
unconfigured by default (its passphrase is `None`), so the cascade falls
through to an interactive prompt. An AI that pipes `--passphrase ""` instead
gets the well-known computer key. Einmo detects this post-hoc: the promotion
report flags `non_human: true` whenever a `stage:verified` stamp is produced
by the empty-passphrase key, and the CLI prints a warning.

Keys are derived deterministically from a passphrase via Argon2id. The
Argon2id parameters are pinned constants (m=19456 KiB, t=2, p=1, matching the
OWASP Password Storage Cheat Sheet minimum baseline), not crate defaults. A
dependency bump cannot silently change key derivation. The salt is
domain-separated (`einmo:stamp-key:v1`) from any other derivation. Changing
these parameters invalidates every previously-derived keypair, so they must
not change without a corpus re-sign.

## Verify-on-Inspect

Every operation that reads a `.einmo` file verifies ALL stamps first. A
tampered file is refused, never operated on. The single filesystem-touching
entry point is `EinmoFile::from_file`, which reads bytes and passes them
through `verify_bytes`.

The verification path checks the **actual raw file bytes**, not a recomputed
canonical form. This matters because `parse()` normalizes metadata (it trims
whitespace after `key:`). If someone adds an extra space to a metadata value
on disk, the parsed form looks identical, but the raw bytes differ. The stage
stamps cover the raw bytes, so `verify_bytes` reconstructs the exact byte
range from the file and checks against that. Metadata whitespace tampering is
caught.

The pure verification functions (`verify_bytes`, `verify_all`) touch no
filesystem, no tty, and no Argon2. They only re-check Ed25519 signatures
already present in a parsed file. This makes them WASM-targetable.

## Flagging

Flagging moves a file from any stage (`output`, `checked`, `verified`) into
`flagged/`. The move appends an unsigned advisory line
`# flagged: <reason> <timestamp>` after the STAMPS section. No stamp is added
-- flagging is not a promotion. The original file is removed from its source
stage (move semantics, not copy).

The advisory line is outside the signed bytes, so flagging does not
invalidate the existing stamp chain. A flagged file still verifies normally.

**Flagging the same path twice:** if `flagged/<rel>` already exists when a new
flag operation targets the same relative path, the new file gets a timestamp
suffix: `flagged/<base>.<timestamp>.einmo`. The previously flagged file is
not overwritten -- both coexist, each with its own advisory line recording
when and why it was flagged. This preserves the history of set-aside files.

## Quick Start

```rust
use einmo::{EinmoSuite, Evaluator, TestConfig, Stage};

struct MyEvaluator;
impl Evaluator for MyEvaluator {
    fn evaluate(&self, source: &str) -> Result<Vec<String>, String> {
        Ok(vec![format!("result: {source}")])
    }
}

fn main() {
    let config = TestConfig::new("my-suite")
        .require_correspondence(Stage::Output, Stage::Checked);
    let suite = EinmoSuite::new(config);
    let results = suite.evaluate_all(&MyEvaluator).unwrap();
    assert!(results.all_output_written_and_verified());
}
```

The suite discovers every file under `input/`, evaluates each one, writes a
signed `.einmo` to `output/`, and re-verifies what it just wrote. If you
configured `require_correspondence(Output, Checked)`, it also compares the two
stages and reports any files that exist only on one side or differ in their
INPUT/OUTPUT sections.

## The `Evaluator` Trait

```rust
pub trait Evaluator: Sync {
    fn evaluate(&self, source: &str) -> Result<Vec<String>, String>;
}
```

The trait is language-agnostic. Source text goes in, formatted output chunks
come out. Returning `Err(String)` signals the input could not be parsed or
accepted (recorded as `status: input-error`). A panic during `evaluate` is
caught by the suite and recorded as `status: output-error`. An expected error
*value* (a division-by-zero alarm, "infinite loop detected") is a normal `Ok`
output, marked `status: normal`.

The `Sync` bound lets `evaluate_all` share one evaluator across threads.
Adapters that wrap a `!Send` interpreter construct it *inside* `evaluate`,
per call. The reference implementations in zweimomo do exactly this:

- `UbcaEvaluatorAdapter` wraps the Foolish UBCa evaluator and formats each FIR
  via the humanizing sequencer.
- `RustPythonEvaluator` spins up a fresh sandboxed `rustpython-vm` interpreter
  (no stdlib) per call.
- `BoaEvaluator` spins up a fresh `boa_engine` context (no fs/network/Node
  APIs) per call.

Einmo never parses body content. It treats every section as opaque text, so
any language that can format its results as strings works as an evaluator.

## CLI

Einmo ships a single CLI app with two binary targets sharing the same parser:
`einmo` (canonical) and `cargo-einmo` (so `cargo einmo ...` also works).

| Subcommand | What it does |
|---|---|
| `einmo promote <from>-><to> <work_dir>` | Append the destination stage's stamp to every matching file. `*->flagged` delegates to `flag`. |
| `einmo flag <work_dir> <stage>` | Move matching files into `flagged/` with an unsigned advisory line. No stamp. |
| `einmo compare <a> <b> <work_dir>` | Per-section comparison of two stages over the mirrored tree. |
| `einmo verify <work_dir>` | Verify signature integrity across one stage (`--stage`) or all stages (`--all`). |
| `einmo confirm-signatures <path> <prefix>` | Report which files carry a stamp whose pubkey starts with the prefix. |
| `einmo show <file>` | Print an envelope's metadata and stamp chain summary. |
| `einmo self-check` | Compute the SHA-256 of the running binary (self-attestation). |

Every subcommand accepts `--json` for machine-readable output. `promote` and
`flag` accept `--filter <glob>` to restrict the operation to matching input
paths (the glob supports `*` as a wildcard). `compare` accepts `--root-cause`
to descend the differing subtree and report only the deepest differing
descendants, and `--require-match` to exit non-zero when anything differs, is
one-sided, or is tampered.

### Targeting specific files

Most of the time you want to act on the whole suite or a glob-filtered
subset. But sometimes you need to target one or two files precisely.
`promote`, `flag`, `compare`, and `verify` all accept **positional file
arguments** after `work_dir`:

```bash
# Promote just one file:
einmo promote output->checked suite_dir alarm_division_by_zero.foo.einmo

# Promote two files:
einmo promote output->checked suite_dir a.foo.einmo b.foo.einmo

# Use -- to separate flags from file names (when a file name starts with -):
einmo promote output->checked suite_dir -- -weird-name.einmo

# Flag a single file with a reason:
einmo flag suite_dir output broken_test.einmo --reason "produces wrong output"

# Compare just one file between two stages:
einmo compare output checked suite_dir important.einmo

# Verify just one file:
einmo verify --all suite_dir critical.einmo
```

**Reading file paths from stdin** with `-`:

```bash
# Pipe a list of files to promote:
echo "a.foo.einmo\nb.foo.einmo" | einmo promote output->checked suite_dir -

# Promote everything that changed (find + einmo):
find suite_dir/output -name '*.einmo' -newer suite_dir/checked | \
  einmo promote output->checked suite_dir -

# Read from a file list:
cat changed-files.txt | einmo promote output->checked suite_dir -
```

**Path normalization:** the CLI accepts file paths in any of these forms and
normalizes them internally to mirror-relative paths:

| Input form | Example | Normalizes to |
|---|---|---|
| Mirror-relative | `test.einmo` | `test.einmo` |
| Nested mirror-relative | `subdir/test.einmo` | `subdir/test.einmo` |
| Stage-relative | `output/test.einmo` | `test.einmo` |
| Stage-relative (nested) | `checked/sub/test.einmo` | `sub/test.einmo` |
| Absolute path | `/home/user/suite/output/test.einmo` | `test.einmo` |
| Input name (no `.einmo`) | `test.foo` | `test.foo.einmo` |

When file arguments are provided, `--filter` is ignored. When no file
arguments are given, `--filter` (or all files if no filter) is used.

Promotion key resolution follows a cascade: `--passphrase` >
`--stdin-passphrase` > `EINMO_PASSPHRASE` env var > `einmo.toml
[signing.<stage>]` > interactive prompt. The `--interactive` flag forces the
prompt, skipping all other tiers. An explicit empty string is "set to empty"
(the computer key), never "unset".

## Catastrophe Crumb Defense

Before each evaluator call, einmo writes a **signed** `.einmo` "catastrophe
crumb" to the output path with `status: output-error` and `status_detail:
"TEST IN PROGRESS -- if you see this file, the test harness crashed during
evaluation. Escalate to human or other agents for support."`.

If the process crashes during evaluation (stack overflow, OOM, abort, kill
signal), this signed file remains as forensic evidence. `catch_unwind` catches
ordinary panics and records them as `status: output-error` with the panic
message, but it cannot catch `abort()` or SIGSEGV. The catastrophe crumb
covers those cases.

On success, `write_output` overwrites the crumb with the real output. The
crumb itself is a valid signed `.einmo` file: it can be verified, compared,
and even promoted. The test suite proves this by spawning a child process
that calls `std::process::abort()`, then checking that the crumb exists,
verifies, and promotes normally.

### Detecting stale crumbs

If a previous run crashed and left a catastrophe crumb on disk, the next run
will **not** silently overwrite it. Einmo detects stale crumbs and gates on
three flags (see Configuration Precedence below):

1. If the crumb's path is in `ignore_catastrophe_crumbs`, the test is skipped
   and marked `ignored`. The suite passes if all other tests pass. The crumb
   stays on disk and can be promoted as the accepted output.
2. If `rerun_catastrophes` is enabled, the crumb is overwritten and the test
   re-runs normally.
3. If neither applies, the suite **fails** with a message naming the crumb
   path and suggesting both flags.

## Configuration Precedence

Einmo resolves each configurable parameter from the first available source,
in decreasing priority:

1. **CLI flag** (e.g. `--walk-depth-limit 32`)
2. **Environment variable** (e.g. `EINMO_WALK_DEPTH_LIMIT=32`)
3. **Code configuration** (`TestConfig::with_walk_depth_limit(32)`)
4. **Per-suite `einmo.toml`** (in `work_dir/einmo.toml`)
5. **Crate-wise `einmo.toml`** (found by walking up from `work_dir`'s parent)
6. **Default**

Environment variables override code configuration. This is intentional: a CI
environment or operator can enforce a limit that test code cannot accidentally
disable.

### Parameters

| Parameter | Env var | Type | Default |
|---|---|---|---|
| Walk depth limit | `EINMO_WALK_DEPTH_LIMIT` | integer | 64 |
| Per-test duration limit | `EINMO_DURATION_LIMIT` | seconds | none |
| Per-suite duration limit | `EINMO_SUITE_DURATION_LIMIT` | seconds | none |
| Rerun catastrophes | `EINMO_RERUN_CATASTROPHES` | `1` / `true` / `yes` | false |
| Ignore catastrophe crumbs | `EINMO_IGNORE_CATASTROPHE_CRUMBS` | colon-separated paths | empty |

### `einmo.toml` `[suite]` section

```toml
[suite]
walk_depth_limit = 32
duration_limit = 30
suite_duration_limit = 300
rerun_catastrophes = false
ignore_catastrophe_crumbs = ["crash.foo.einmo", "overflow.foo.einmo"]
```

A per-suite `einmo.toml` in the work directory beats a crate-wise `einmo.toml`
found by walking up the directory tree. Per-suite values override crate-wise
values; unset keys fall through to the crate-wise file, then to defaults.

## Duration Limits

Two parameters control timeouts:

- `EINMO_DURATION_LIMIT` (seconds): per-test timeout. Tests exceeding this
  are marked `OutputError` with a detail line reporting the limit and actual
  elapsed time.
- `EINMO_SUITE_DURATION_LIMIT` (seconds): per-suite timeout. Aborts early,
  skipping remaining tests, and records a correspondence failure explaining
  how many were skipped.

You can also set these programmatically via `TestConfig::with_duration_limit`
and `TestConfig::with_suite_duration_limit`. The per-test limit is checked
after evaluation completes (it does not interrupt a running evaluator), while
the per-suite limit is checked before starting each new test.

## Configurable Depth Limit

`TestConfig::with_walk_depth_limit(n)` controls the maximum recursion depth
for directory walks (default 64). This prevents stack overflow on
pathologically deep trees and catches symlink cycles. When the walk exceeds
the limit, einmo returns an I/O error explaining the situation. Symlinks are
followed: a broken symlink (target missing) is skipped silently, but an
ELOOP from a symlink cycle propagates as an error.

## The `TestConfig` API

`TestConfig` is constructed via `new(work_dir)` or `default_for(work_dir)` and
refined with builder-style methods:

| Method | Effect |
|---|---|
| `new(work_dir)` / `default_for(work_dir)` | Create a config with all defaults for the given work directory. |
| `require_correspondence(a, b)` | Add a required stage pair checked after `evaluate_all`. |
| `with_separator(sep)` | Set a custom section separator. |
| `foolish_separator()` | Use `!!\n` (a Foolish line comment) as the separator. |
| `with_perspectives(vec)` | Register statically configured perspectives. |
| `with_parallel(threads)` | Run with `n` threads (`None` for serial). |
| `with_walk_depth_limit(n)` | Set the recursion depth limit for input-tree walks. |
| `with_duration_limit(duration)` | Set the per-test timeout. |
| `with_suite_duration_limit(duration)` | Set the per-suite timeout. |
| `with_rerun_catastrophes(bool)` | Allow overwriting stale catastrophe crumbs. |
| `with_ignore_catastrophe_crumbs(vec)` | Accept specific crumbs as expected (skip + pass). |
| `with_diff_limit(chars)` | Set the DIFF size limit for dependents (default 2000). |
| `with_dependent_separator(sep)` | Set the dependent-name separator (default `++`). |
| `with_match_sections(policy)` | Set which sections must match in `compare`. |
| `with_suite_name(name)` | Set the human-readable suite name for metadata. |

Accessors (`work_dir()`, `input_path()`, `stage_dir(stage)`, `separator()`,
`encoding()`, `suite_name()`, etc.) are all `#[must_use]`.

Defaults: `input/` input dir, standard stage dirs
(`output`/`checked`/`flagged`/`verified`), `①\n` separator, `InputOutput`
matching, empty-passphrase for `output` and `checked` stages, `verified`
unset (prompts interactively), `++` dependent separator, 2000-char diff
limit, 64-deep walk limit.

## Perspectives

A `Perspective` is a pure `fn(&str) -> String` transform applied to the INPUT
or a specific OUTPUT chunk, producing a derived section stored alongside the
body. Einmo stays language-agnostic: it never parses body content, it just
applies the function and stores the result.

```rust
use einmo::{Perspective, PerspectiveOf};

let shout = Perspective {
    name: "shout",
    of: PerspectiveOf::Input,
    extract: |s| s.to_uppercase(),
};
```

`PerspectiveOf::Input` derives from the INPUT body. `PerspectiveOf::Output(i)`
derives from the `i`-th OUTPUT chunk. Perspectives are registered via
`TestConfig::with_perspectives(vec![...])` and emitted automatically during
`evaluate` and `evaluate_all`.

The **Charmer** plugin (in zweimomo) is a reference perspective.
`zweimomo::aspects::aspects_perspective()` wraps `compute_aspects`, which
reports four metrics from the primary output chunk:

```
encoding: ascii
lines: 1
chars: 1
alnum: 1
```

The core `compute_aspects` function has zero einmo dependency. You can copy
it into any project and call it directly.

## Error Handling

Einmo uses a single `EinmoError` enum, marked `#[non_exhaustive]` and deriving
`thiserror::Error`. All fallible functions return `Result<T, EinmoError>`.
The variants:

- `Io { path, source }` carries the offending path.
- `Parse(String)` for malformed envelopes.
- `SeparatorCollision { section }` when a body contains the separator.
- `Verification(String)` when a stamp fails verify-on-inspect.
- `Stamp(String)` for invalid stamp JSON.
- `InvalidStageName(String)` for names failing `[A-Za-z0-9_-]+`.
- `IllegalTransition { from, to }` for disallowed stage pairs.
- `Config(String)` for invalid configuration values.
- `NoKey(String)` when no key material could be resolved.

## Standalone Scope

Einmo has **zero dependency on any Foolish crate**. It reimplements the
signing and format machinery from scratch: Ed25519 via `ed25519-dalek`,
Argon2id via `argon2`, SHA-256 via `sha2`. The crate is structured to be
promoted to its own repository as-is. The `Cargo.toml` explicitly documents
this: "Standalone crate: NO dependency on any workspace crate."

## Dependencies

| Crate | Version | Purpose |
|---|---|---|
| `ed25519-dalek` | 2 | Ed25519 signing and verification |
| `argon2` | 0.5 | Argon2id passphrase-to-key derivation |
| `base64` | 0.22 | Base64 encoding of signatures |
| `hex` | 0.4 | Hex encoding of pubkeys and hashes |
| `clap` | 4 | CLI argument parsing (with `derive` and `env` features) |
| `serde` | 1 | Serialization (with `derive` feature) |
| `serde_json` | 1 | JSON stamp serialization |
| `toml` | 0.8 | Configuration file parsing |
| `time` | 0.3 | ISO-8601 timestamps (with `macros`, `formatting`, `parsing` features) |
| `thiserror` | 2 | Error enum derivation |
| `similar` | 2 | Unified diff generation for dependent DIFF sections |
| `sha2` | 0.10 | SHA-256 for git diff hashing and self-check |
| `rpassword` | 7 | Interactive passphrase prompt on the tty |

Dev dependency: `tempfile` 3 (for tests).

## License

`MIT OR Apache-2.0`

---

## Appendix: Migrating an insta test to einmo

This appendix demonstrates, step by step, how to refactor an existing insta
snapshot test into an einmo suite. The example is based on a real test in the
Foolish project (`foolish-ubca/src/ubca_snapshot_tester.rs`).

### Before: the insta test

The original test uses insta's `Settings` API to bind a snapshot path, then
calls `assert_snapshot!` for each evaluated input:

```rust
// foolish-ubca/src/ubca_snapshot_tester.rs (BEFORE)

use std::path::PathBuf;
use crate::evaluator::UbcaEvaluator;

fn suite() -> foolish_core::SnapshotSuite {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    foolish_core::SnapshotSuite::new(
        base.join("snapshot_tests").join("input"),
        base.join("snapshot_tests").join("approved"),
    )
}

#[cfg(test)]
mod approval_tests {
    use super::*;

    #[test]
    fn approval_all() {
        let eval = UbcaEvaluator;
        let suite = suite();
        let evaluations = suite.evaluate_all(num_cpus::get(), &eval);
        let approved = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("snapshot_tests")
            .join("approved");
        let mut settings = insta::Settings::clone_current();
        settings.set_snapshot_path(&approved);
        settings.set_prepend_module_to_snapshot(false);
        settings.set_omit_expression(true);
        settings.bind(|| {
            for (name, result) in evaluations {
                eprintln!("Evaluating: {}", name);
                match result {
                    Ok(output) => {
                        insta::assert_snapshot!(format!("{}.foo", name), output);
                    }
                    Err(msg) => {
                        eprintln!("  ERROR: {}", msg);
                    }
                }
            }
        });
    }
}
```

What this does:
- Evaluates every `.foo` file under `snapshot_tests/input/`.
- For each result, calls `insta::assert_snapshot!` which compares against
  `.snap` files in `snapshot_tests/approved/`.
- If the output differs, insta writes a `.snap.new` file. A human runs
  `cargo insta review` to accept or reject.
- There is no signing. The `.snap` files are unsigned text. Anyone can edit
  them, and there is no audit trail of who approved what.

The `.snap` file format is YAML frontmatter + content:

```
---
source: foolish-ubca/src/ubca_snapshot_tester.rs
assertion_line: 34
---
INPUT:
```foolish
{a = 10 / 2; b = 10 / 0; c = 20 / 4;}
```
[0] RESULT:
```hfssnap
{NK
...
```
```

### After: the einmo test

The refactored test creates an einmo `EinmoSuite` over the same input
directory, uses an `Evaluator` adapter around `UbcaEvaluator`, and enforces
the `output == checked` correspondence gate:

```rust
// foolish-ubca/src/ubca_snapshot_tester.rs (AFTER)

use std::path::PathBuf;
use einmo::{EinmoSuite, Evaluator, Stage, TestConfig};

fn suite_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("snapshot_tests")
}

#[cfg(test)]
mod approval_tests {
    use super::*;

    struct UbcaEinmoAdapter;
    impl Evaluator for UbcaEinmoAdapter {
        fn evaluate(&self, source: &str) -> Result<Vec<String>, String> {
            let inner = crate::evaluator::UbcaEvaluator;
            let firs = inner.evaluate(source)?;
            Ok(firs.iter().map(|fir| {
                let fir = foolish_core::clone_steppable(fir);
                foolish_core::FirSequencer::format(&fir)
            }).collect())
        }
    }

    #[test]
    fn approval_all() {
        let config = TestConfig::new(suite_dir())
            .require_correspondence(Stage::Output, Stage::Checked);
        let suite = EinmoSuite::new(config);
        let results = suite.evaluate_all(&UbcaEinmoAdapter)
            .expect("evaluate_all should not fail at the fs level");

        // Every input must have produced a written, self-verifying .einmo.
        assert!(!results.files.is_empty());
        for file in &results.files {
            assert!(
                file.written_and_verified,
                "{} was not written+verified: {:?}",
                file.rel_path.display(),
                file.detail
            );
        }

        // If a checked/ baseline exists, output must match it.
        assert!(
            results.all_output_written_and_verified(),
            "correspondence failures: {:?}",
            results.correspondence_failures
        );
    }
}
```

### What changed

| Aspect | insta (before) | einmo (after) |
|---|---|---|
| **Storage** | `approved/*.snap` (flat, module-coupled) | `output/*.einmo` + `checked/*.einmo` (hierarchical, mirrors `input/`) |
| **Format** | YAML frontmatter + content | Header + metadata + sections + signed JSON stamps |
| **Signing** | None | Ed25519 append chain (compiled + configured + stage stamps) |
| **Review** | `cargo insta review` (TUI, review == promotion) | `einmo promote output->checked` (CLI, review and promotion are separate acts) |
| **CI gate** | `INSTA_UPDATE=no` (env var, easy to miss) | `einmo compare output checked --require-match` (explicit, signed) |
| **Human attestation** | None | `einmo promote checked->verified --interactive` (passphrase, detects computer key) |
| **Crash safety** | None (crash leaves no trace) | Catastrophe crumb (signed "TEST IN PROGRESS" file survives) |
| **Error capture** | `eprintln!` (swallowed) | `status: input-error` / `status: output-error` in the signed `.einmo` |

### Step-by-step migration guide

**Step 1: Create the einmo suite directory structure.**

If your insta snapshots live in `snapshot_tests/input/` and
`snapshot_tests/approved/`, create the einmo stage directories alongside
`input/`:

```
snapshot_tests/
├── input/          # existing .foo inputs (unchanged)
├── output/         # einmo generates here (new)
├── checked/        # promoted baselines (new, replaces approved/)
├── flagged/        # set-aside files (new)
└── verified/       # human-signed (new)
```

**Step 2: Write the `Evaluator` adapter.**

Wrap your existing evaluator in an `impl Evaluator for ...` that returns
`Vec<String>` (one string per top-level result). The adapter formats the
internal type to a human-readable string. For Foolish, this is
`FirSequencer::format`.

**Step 3: Replace the test function.**

Replace the `insta::Settings` + `assert_snapshot!` loop with
`EinmoSuite::evaluate_all`. The suite handles discovery, evaluation, signing,
writing, and re-verification.

**Step 4: Run the test to generate `output/`.**

```bash
cargo test -p foolish-ubca -- approval_all
```

Every input produces `output/<name>.einmo` -- a signed file with the input,
the evaluator's output, and the stamp chain.

**Step 5: Review and promote to `checked/`.**

```bash
einmo compare output checked snapshot_tests/          # see what's new
einmo promote output->checked snapshot_tests/          # promote all

# Or promote just one file (no --filter needed):
einmo promote output->checked snapshot_tests/ alarm_division_by_zero_in_brane.foo.einmo

# Promote a few specific files:
einmo promote output->checked snapshot_tests/ a.foo.einmo b.foo.einmo c.foo.einmo

# Promote everything that changed (pipe from find):
find snapshot_tests/output -name '*.einmo' -newer snapshot_tests/checked | \
  einmo promote output->checked snapshot_tests/ -
```

The promoted files in `checked/` carry an additional `stage:checked` stamp.
The `output == checked` correspondence gate now passes on the next test run.

**Step 6: (Optional) Human-verify for release.**

```bash
einmo promote checked->verified snapshot_tests/ --interactive
# Enter a human passphrase (not empty) to produce a stage:verified stamp.
# An AI piping --passphrase "" gets the computer key -- post-hoc detectable.
einmo confirm-signatures snapshot_tests/verified <release-key-prefix> --require-all

# Verify just one critical file before signing off:
einmo verify --all snapshot_tests/ critical_path_test.einmo

# Show the stamp chain on one file to confirm the officer's key:
einmo show snapshot_tests/verified/critical_path_test.einmo

# Compare one specific file between checked and verified:
einmo compare checked verified snapshot_tests/ critical_path_test.einmo --require-match

# If a file needs to be pulled from the release:
einmo flag snapshot_tests/ verified broken_release_test.einmo --reason "regression found post-verify"
```

**Step 7: Delete the old `.snap` files.**

Once the einmo suite is green and the `checked/` baselines are committed,
remove the old `approved/*.snap` files and the `insta` dev-dependency. The
migration is complete.

### Notes for the migrating agent

- The `Evaluator` adapter must be `Sync`. If your interpreter is `!Send`
  (RustPython, Boa), construct it *inside* `evaluate` per call -- do not
  store it in the adapter struct.
- If your output contains the separator character (`①` by default), use
  `TestConfig::with_separator("!!\n")` or another string that does not appear
  in your output.
- The `checked/` directory replaces `approved/`. The `.snap` format and the
  `.einmo` format are not compatible -- migration is a one-way conversion
  (generate fresh `.einmo` from the evaluator, then promote).
- Catastrophe crumbs from crashed runs will block re-runs. Use
  `EINMO_RERUN_CATASTROPHES=1` to overwrite them, or
  `EINMO_IGNORE_CATASTROPHE_CRUMBS=crash.foo.einmo` to accept a known crash
  as expected.

---

## Appendix: Draconian Release Gate

This example shows a deployment test that enforces the strictest possible
attestation chain. It is appropriate for a release gate where every test
output must be human-verified under a specific release officer's key before
the release may ship.

The test is named `iron_grid_release_attestation` to convey its severity.

### What it enforces

1. Every input file produces a written, self-verifying `.einmo` in `output/`.
2. Every `output/` file has a corresponding, non-empty file in `verified/`.
3. Every `verified/` file carries a `stage:verified` stamp whose public key
   matches the release officer's key embedded in the test code.
4. No file in `verified/` was signed by the well-known computer (empty
   passphrase) key -- an AI bypass is detected and fails the gate.
5. The `output` and `verified` stages are byte-identical in their INPUT and
   OUTPUT sections (no unreviewed drift).

If any of these conditions fail, the test fails and the release is blocked.

### The code

```rust
use einmo::{
    confirm_signatures, compare, EinmoFile, EinmoSuite, Evaluator,
    MatchSections, Stage, TestConfig,
};
use std::path::Path;

/// The release officer's Ed25519 public key (hex).
///
/// This is NOT the empty-passphrase computer key. It is derived from the
/// release officer's private passphrase, which is never stored in code.
/// Only someone who knows the passphrase can produce a `stage:verified`
/// stamp whose pubkey matches this constant.
const RELEASE_OFFICER_PUBKEY: &str = "dc5f586c3a1b2e4f8a0c7d9e1f3a5b7c9d1e3f5a7b9c1d3e5f7a9b1c3d5e7f9a";

/// The well-known computer key prefix (first 8 hex chars of the
/// empty-passphrase key). Used to detect AI bypass.
const COMPUTER_KEY_PREFIX: &str = "a4c2e6b0";

#[test]
fn iron_grid_release_attestation() {
    let suite_dir = Path::new("release_suite");

    // --- Phase 1: generate output/ ---
    //
    // Every input is evaluated. The suite writes signed .einmo files to
    // output/. If any file fails to write or self-verify, the gate fails
    // immediately.
    let config = TestConfig::new(suite_dir);
    let suite = EinmoSuite::new(config);
    let results = suite
        .evaluate_all(&ProductionEvaluator)
        .expect("evaluation must not fail at the filesystem level");

    assert!(
        !results.files.is_empty(),
        "release suite must contain at least one test"
    );
    for file in &results.files {
        assert!(
            file.written_and_verified,
            "RELEASE BLOCKED: {} was not written and verified: {:?}",
            file.rel_path.display(),
            file.detail
        );
    }

    // --- Phase 2: every output must have a non-empty verified counterpart ---
    //
    // The verified/ directory must contain a file for every input. The file
    // must be non-empty (a zero-byte file means someone created the path
    // but never actually promoted and signed it).
    let inputs = einmo::verify(&TestConfig::new(suite_dir), Some(Stage::Output))
        .expect("verify output stage");
    for file_verif in &inputs.files {
        let verified_path = suite_dir
            .join("verified")
            .join(&file_verif.rel_path);

        assert!(
            verified_path.exists(),
            "RELEASE BLOCKED: no verified file for {}",
            file_verif.rel_path.display()
        );

        let metadata = std::fs::metadata(&verified_path)
            .expect("verified file must be stat-able");
        assert!(
            metadata.len() > 0,
            "RELEASE BLOCKED: verified file for {} is empty (was it actually promoted?)",
            file_verif.rel_path.display()
        );
    }

    // --- Phase 3: every verified file must carry the release officer's key ---
    //
    // confirm_signatures walks verified/ and reports which files carry a
    // stamp whose pubkey starts with RELEASE_OFFICER_PUBKEY. If any file
    // lacks the officer's signature, the gate fails.
    let sig_report = confirm_signatures(
        &suite_dir.join("verified"),
        RELEASE_OFFICER_PUBKEY,
    )
    .expect("confirm-signatures must not fail at the filesystem level");

    assert!(
        sig_report.all_matched(),
        "RELEASE BLOCKED: {} verified file(s) lack the release officer's signature ({:#x}): {:?}",
        sig_report.unmatched.len(),
        RELEASE_OFFICER_PUBKEY,
        sig_report.unmatched,
    );

    // --- Phase 4: no verified file may carry the computer key ---
    //
    // An AI that ran `einmo promote checked->verified --passphrase ""` would
    // produce a stage:verified stamp under the well-known empty-passphrase
    // key. This is post-hoc detectable: confirm_signatures with the computer
    // key prefix should match ZERO files.
    let computer_report = confirm_signatures(
        &suite_dir.join("verified"),
        COMPUTER_KEY_PREFIX,
    )
    .expect("computer-key scan must not fail");

    assert!(
        computer_report.matched.is_empty(),
        "RELEASE BLOCKED: {} verified file(s) were signed by the computer key (AI bypass detected): {:?}",
        computer_report.matched.len(),
        computer_report.matched,
    );

    // --- Phase 5: output and verified must be byte-identical in content ---
    //
    // The INPUT and OUTPUT sections of every file in output/ must match its
    // counterpart in verified/. This catches unreviewed drift: if someone
    // changed the code after the last verification, output/ will differ from
    // verified/ and the gate fails.
    let cmp = compare(
        &TestConfig::new(suite_dir),
        Stage::Output,
        Stage::Verified,
        MatchSections::InputOutput,
    )
    .expect("compare must not fail at the filesystem level");

    assert!(
        cmp.is_clean(),
        "RELEASE BLOCKED: output does not match verified \
         ({} differing, {} only-in-output, {} only-in-verified, {} tampered)",
        cmp.differing.len(),
        cmp.only_in_a.len(),
        cmp.only_in_b.len(),
        cmp.tampered.len(),
    );
}

// --- The evaluator (application-specific) ---

struct ProductionEvaluator;
impl Evaluator for ProductionEvaluator {
    fn evaluate(&self, source: &str) -> Result<Vec<String>, String> {
        // ... your production evaluator logic ...
        Ok(vec![source.to_uppercase()])
    }
}
```

### How to set up the release officer's key

The `RELEASE_OFFICER_PUBKEY` constant is the hex-encoded Ed25519 verifying
key derived from the officer's passphrase. To obtain it:

```bash
# The officer runs this once and pastes the output into the test code:
echo -n "my-secret-release-passphrase" | einmo derive-pubkey
# (hypothetical subcommand; or use a small Rust script calling
#  einmo::signature::derive_keypair -- note: derive_keypair is pub(crate),
#  so in practice you'd derive it via the einmo CLI or a helper crate.)
```

The passphrase itself is never stored in the repository. Only the public key
is embedded in the test. An attacker who steals the repository cannot produce
a valid `stage:verified` stamp without the passphrase.

### Why this is draconian

| Gate | What it catches |
|---|---|
| Phase 1 (written + verified) | Evaluator crash, signing failure, filesystem error |
| Phase 2 (non-empty verified) | Someone forgot to promote a test to verified/ |
| Phase 3 (officer key present) | Verified by the wrong person, or not verified at all |
| Phase 4 (no computer key) | An AI bypassed the human gate with `--passphrase ""` |
| Phase 5 (output == verified) | Code changed after the last verification (drift) |

If all five phases pass, every test output has been generated, human-reviewed,
signed by the release officer, and has not drifted since. The release may
ship.

---

## Appendix: Development Compliance Test

This is the everyday test a coding agent runs during development. It is
lightweight: it generates `output/`, then checks that `output` matches the
committed `checked/` baseline. No human passphrase, no verified stage, no
release officer key -- just "does my code still produce the same output as
the last reviewed baseline?"

The default signing key (empty passphrase, the well-known computer key) is
configured directly in the code. This is intentional for development: the
test runner signs with the computer key, and the `checked/` baseline was
promoted with the same key. A human reviews the diff before promoting, but
the signature is the computer key -- not a human attestation. That is fine
for development; the draconian gate (above) is what enforces human
attestation for releases.

### The code

```rust
use einmo::{EinmoSuite, Evaluator, Stage, TestConfig};

struct DevEvaluator;
impl Evaluator for DevEvaluator {
    fn evaluate(&self, source: &str) -> Result<Vec<String>, String> {
        // ... your evaluator ...
        Ok(vec![format!("processed: {source}")])
    }
}

#[test]
fn dev_compliance_output_matches_checked() {
    let config = TestConfig::new("dev_suite")
        // The default key is the empty-passphrase computer key.
        // TestConfig::new already sets this (output/checked passphrases
        // default to ""). This line makes it explicit for readability.
        .require_correspondence(Stage::Output, Stage::Checked);

    let suite = EinmoSuite::new(config);
    let results = suite.evaluate_all(&DevEvaluator)
        .expect("evaluate_all should not fail at the filesystem level");

    // Every test must have been written and self-verified.
    for file in &results.files {
        assert!(
            file.written_and_verified,
            "{} failed to write/verify: {:?}",
            file.rel_path.display(),
            file.detail
        );
    }

    // output must match checked. If checked/ is empty (first run, or
    // no baseline committed yet), this will report only-in-output for
    // every file -- which is the signal to review and promote.
    assert!(
        results.all_output_written_and_verified(),
        "output does not match checked baseline:\n  {}",
        results.correspondence_failures.join("\n  ")
    );
}
```

### What the coding agent does during development

The agent writes code, runs the test, and inspects the diff. If `output`
diverged from `checked/`, the agent decides: fix the code (so output matches
checked), or promote the new output (if the change is intentional).

```bash
# 1. Run the test (generates output/ and compares to checked/)
cargo test -- dev_compliance_output_matches_checked

# 2. If the test failed, see what changed:
einmo compare output checked dev_suite/

# Or check just the files that failed:
einmo compare output checked dev_suite/ failing_test.einmo

# 3a. If the change is a bug -- fix the code, re-run step 1.

# 3b. If the change is intentional -- review the diff, then promote:
einmo promote output->checked dev_suite/                         # promote all
einmo promote output->checked dev_suite/ fixed_test.einmo        # or just one
echo "a.einmo\nb.einmo" | einmo promote output->checked dev_suite/ -  # or from stdin

# 4. Verify signature integrity of the promoted files:
einmo verify dev_suite/ --stage checked                           # all files
einmo verify --all dev_suite/ important_test.einmo                # just one

# 5. Confirm the computer key signed everything (no human key leaked in):
einmo confirm-signatures dev_suite/checked a4c2e6b0 --require-all

# 6. Show a specific file's stamp chain to inspect provenance:
einmo show dev_suite/checked/important_test.einmo

# 7. If a test is wrong and needs to be set aside:
einmo flag dev_suite checked --filter broken_test.einmo --reason "needs rework"
# Or target the file directly:
einmo flag dev_suite checked broken_test.einmo --reason "needs rework"

# 8. Re-run the test to confirm the suite is clean:
cargo test -- dev_compliance_output_matches_checked
```

### Setting up a new suite from scratch

When starting a new einmo suite with no `checked/` baseline, every input
will appear as `only-in-output` on the first run. The agent reviews the
generated output, then promotes:

```bash
# First run -- generates output/, checked/ is empty:
cargo test -- dev_compliance_output_matches_checked
# Expected: failures for every file (only-in-output).

# Review the generated outputs:
einmo show dev_suite/output/first_test.einmo
einmo show dev_suite/output/second_test.einmo

# Promote everything to checked/ (the computer key is used by default):
einmo promote output->checked dev_suite/

# Commit the checked/ baseline:
git add dev_suite/checked/
git commit -m "Add einmo checked baseline for dev_suite"

# Now the test passes:
cargo test -- dev_compliance_output_matches_checked
```

### The default key in configuration

`TestConfig::new(work_dir)` sets the `output` and `checked` stage passphrases
to `""` (the empty string). This means the test runner signs with the
well-known computer key. An `einmo.toml` in the work directory can override
this:

```toml
# dev_suite/einmo.toml
[signing.output]
passphrase = ""

[signing.checked]
passphrase = ""
```

Or, to use a shared team key for development (so team members can promote
without prompting):

```toml
[signing.checked]
passphrase = "team-dev-shared-passphrase"
```

The passphrase is resolved through the cascade: CLI `--passphrase` > env
`EINMO_PASSPHRASE` > `einmo.toml [signing.<stage>]` > interactive prompt.
For development, the empty passphrase is the default and requires no
configuration.
