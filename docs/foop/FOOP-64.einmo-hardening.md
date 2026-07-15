# FOOP-64 — three einmo hardening features (plan)

Consolidated from Atlas's inputs, 2026-07-15. **Written before further coding**;
features 1 and 3 are already committed, feature 2 is specified here and is the
one still to build.

---

## Feature 1 — Parallel execution with configurable parallelism ✅ DONE (296fce26)

**Directive:** *"Is parallel execution configurable yet?"* → *"default to the
normal default of number of cpu's"*.

**Was:** the mechanism existed (`evaluate_raw_parallel`, `std::thread::scope` +
work-stealing, pinned by `parallel_and_serial_agree`), but `parallel` was the
only tunable with **no env tier** — every other limit had one
(`EINMO_WALK_DEPTH_LIMIT`, `EINMO_DURATION_LIMIT`, …). The UBCa suite ran serial.

**Now:**

```
EINMO_PARALLEL > with_parallel() > einmo.toml [suite] parallel
               > default: std::thread::available_parallelism()
```

- `available_parallelism` — std, no new dep (einmo stays standalone /
  repo-promotable); respects cgroup+affinity so containers get their quota.
- `0` or `1` ⇒ serial. CI can pin a cap test code cannot override.
- Unavailable ⇒ serial rather than a guess.

**Known gap this exposed (documented at the branch, not papered over):**
`suite_duration_limit` is enforced **only on the serial path** — it is checked
*before starting* each test, so parallel workers all launch before a short
budget expires and nothing aborts. einmo's own
`suite_duration_limit_aborts_early` caught it immediately; that test now pins
serial explicitly. Proper fix = a shared deadline the workers poll between
tests. **Deferred — needs its own decision.**

---

## Feature 2 — Einmo is extra critical of its own file paths ✅ DONE (0e71339d), ⬅ NOW SUPERSEDED IN SHAPE

**Superseded 2026-07-15 by Atlas's escalating-levels directive** — see FOOP-64.md
§"The escalating validation levels: Output → Checked → Verified". The rules
below were right but applied *unconditionally* across all three stages, so an
unpopulated `verified/` made a dev suite red. They now sit at escalating levels:
each level performs everything the level below it requires, plus its own. The
API has **no default level** (the configuring test states which level it
produces); the CLI defaults to the Checked level with `--level verified` to
escalate. `SuiteIntegrity`/`IntegrityViolation` become the `Problem` enum.

### (original, for the record)

**Directive:** *"the Einmo library should be extra critical of it's file paths.
If it ever finds extraneous file, it should be a test error. If it ever finds a
missing input for some checked or verified file, that's a deal breaker and test
should error out."* Plus: *"the API should return a record describing each
error, including one for each unmatched extraneous file"* and *"the CLI/API
should be complementary in informing caller/user of the offending files."*

**Rationale:** silent skipping is how a corpus rots. The insta corpus this
replaces hid `assignment_anchor_search.foo` — a test that had **never
compiled** — for months, because the harness `eprintln!`d the error and moved
on. Einmo must never quietly ignore a file in its own tree.

### The two rules

| # | Rule | Rationale |
|---|------|-----------|
| R1 | **No extraneous files in `input/`** — anything dot-prefixed (editor swap/backup, `.DS_Store`) is *skipped for discovery* (so it can never become a phantom test) but **reported and fails the suite**. | A test input is a file someone deliberately named. |
| R2 | **No orphaned artifacts** — a `.einmo` in `output/`, `checked/`, or `verified/` whose `input/` file is gone is a signed baseline for a test that cannot be re-run or reviewed. **Hard error, no exceptions.** | Deleting a test must mean deleting (or flagging) its artifacts. |

`flagged/` is **exempt from R2**: flagging *is* retirement, so a flagged
artifact without an input is a completed retirement, not an orphan.

### The record (API)

One record per violation — not loose path lists:

```rust
pub struct IntegrityViolation {
    pub path: PathBuf,          // relative to the work dir
    pub fault: IntegrityFault,
}

pub enum IntegrityFault {
    ExtraneousInput,             // R1
    OrphanedArtifact(Stage),     // R2 — which stage holds the orphan
}

impl IntegrityViolation {
    pub fn reason(&self) -> String;      // what is wrong
    pub fn remedy(&self) -> &'static str; // what to do about it
}
// + Display: "path: reason — remedy"

pub struct SuiteIntegrity { pub violations: Vec<IntegrityViolation> }
impl SuiteIntegrity {
    pub fn is_clean(&self) -> bool;
    pub fn report(&self) -> String;
}
```

`TestResults` gains `integrity: SuiteIntegrity`; `all_output_written_and_verified()`
returns false when it is not clean, so **every existing gate inherits the check
without touching call sites**.

### Plumbing

- `stage::walk_input_tree_reporting(dir, depth) -> (inputs, extraneous)` — the
  existing `walk_input_tree` delegates to it, so no caller changes. Discovery
  still skips dotfiles; this variant hands them back to be reported.
- `EinmoSuite::evaluate_all` runs the integrity check and fills
  `results.integrity`.
- **CLI (complementary, per the directive):** `einmo verify` reports the same
  violations — it is the natural "is my suite sound?" command — with `--json`
  emitting one object per violation. `einmo list` already surfaces
  presence/status per stage; verify is where shape is judged.

### Consequence to accept

While a `.swp` exists in `input/`, the gate is **red**. That is intended (a
dirty tree is not a clean baseline) — and feature 3 is what makes it livable.

### Tests

`walk_skips_hidden_entries` (exists) + new: extraneous input → violation with
`ExtraneousInput`; orphaned checked/verified artifact → violation naming the
stage; `flagged/` orphan → **no** violation; a clean suite → `is_clean()`;
`all_output_written_and_verified()` false when integrity is dirty.

---

## Feature 3 — poor_einmo.sh must not create the files feature 2 rejects ✅ DONE (296fce26)

**Directive:** *"the script is still not skipping .swp files"* → *"set a
temporary and backup folder for the call to vimdiff … mktemp those directories
for each run"* → *"update script to also put the temp/backup files somewhere
else so as to not trigger these errors."*

**Root cause:** vim writes swap/backup/undo **next to the file being edited**,
so reviewing an input dropped `.name.foo.swp` *into* the suite — and opening a
stale one bred `.comprehensive.foo.swp.swp`. The script was manufacturing the
cruft that feature 2 now fails on.

**Now:** per-run `mktemp -d` scratch dirs handed to vim
(`directory=`/`backupdir=`/`undodir=`, trailing `//` so same-named tests in
different FOOP dirs cannot collide), removed on exit; the loop also refuses to
open any dot-prefixed path. `-n` dry-run echoes the full call (Atlas's trick for
debugging scripts that would otherwise lock the terminal).

---

## Feature 4 — `produced_by` returns a `&'static str` ⬅ TO BUILD (small)

**Directive:** *"And plus that static string"* — the loose end I flagged when
reporting the 31× fix.

The dominant cost is gone (`produced_by` no longer SHA-256s the 24 MB binary per
stamp; it is a `LazyLock`), but it still `.clone()`s a fresh `String` for every
stamp. The value is immutable for the life of the process, so the honest type is
`&'static str`:

```rust
pub(crate) fn produced_by() -> &'static str {
    static PRODUCED_BY: LazyLock<String> = LazyLock::new(|| { … });
    PRODUCED_BY.as_str()
}
```

Trivial next to what was removed — but it makes the *type* state the fact that
the value never changes, and drops 161 allocations per promotion. Callers that
need an owned copy call `.to_string()` at the one site that stores it.

## Order

3 → 1 → **2 last**, deliberately: feature 2 makes stray files fatal, so the
script must stop creating them *first*. That ordering already happened. Feature
4 rides with 2 (same file family, no interaction).

## Open question for the human

The `suite_duration_limit` gap (feature 1): fix now with a shared polled
deadline, or leave documented and deferred?
