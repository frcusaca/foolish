---
foop: 53
title: Ship einmo — own repository, crates.io registration, and a working `cargo einmo test`
author: Atlas <hc.busy@gmail.com>
status: Draft
type: Standards
created: 2026-07-19
phase: meta
supersedes: []
begun: [ ]
---

# FOOP-35: Ship einmo — own repository, crates.io registration, and a working `cargo einmo test`

FOOP numbering is little-endian; the full rules live in `foop.md` at the
repository root — **read it before creating or editing a FOOP.**

## Abstract

Take einmo from "standalone crate inside the foolish monorepo" to a **published dual product**: a
**library** other Rust projects can depend on (`einmo = "0.1"` — the suite, format, signing,
verification, and transition APIs, rustdoc'd and semver'd) and a **utility** anyone can install as a
cargo command (`cargo install einmo` → `einmo` + the `cargo-einmo` alias, so `cargo einmo test` runs a
suite and fails the build on drift). Covers: the new `test` subcommand; extraction into einmo's own
repository (history preserved); registration and publication on crates.io; and the testing/CI battery
expected of a published Rust tool (property tests, fuzzing of the untrusted-input parsers, CLI
snapshot tests, deny/audit, MSRV, coverage). The specification is written as the **sequential
walkthrough of decisions and registrations** the process requires — read §S.1 through §S.8 in order
and you have performed the process.

## The Aspirational Goal

**`cargo install einmo && cargo einmo test` on any machine, any project.**

Einmo's promise — directory-based, cryptographically signed snapshot testing with a staged promotion
pipeline — is not Foolish-specific, and FOOP 92 §1 built it dependency-free precisely so it could
leave home. When this FOOP is complete, a stranger with a Rust toolchain can install einmo from
crates.io, point it at a suite, and get a red/green answer that their CI can trust; the Foolish
monorepo becomes just einmo's first customer; and the crate carries the testing rigor that a tool
whose whole job is *verifying other people's work* owes its users.

## Motivation

Today einmo runs only from this monorepo (`cargo run -p einmo --` or `./target/debug/einmo`). The
`cargo-einmo` alias binary already exists and correctly strips cargo's injected argv — but there is no
`test` verb to point CI at, no LICENSE files backing the declared `MIT OR Apache-2.0`, no repository
metadata, no publish, and no external home. FOOP 64 is turning the UBCa corpus into einmo's flagship
suite and FOOP 25 will grow a review server on top; both deserve an einmo that is a real product with
its own release cadence rather than a subdirectory.

## Specification — the walkthrough, in order

Each stage below is a *decision or registration*, stated with its recommendation. Executing this FOOP
means resolving each in sequence; the plan mirrors these stages one-to-one.

### S.1 The decisions ledger (resolve before any mechanics)

| # | Decision | Recommendation |
|---|----------|----------------|
| 1 | **Crate/binary names** | `einmo` (library + `einmo` bin + `cargo-einmo` bin, as today — one crate, three targets). The library target is a first-class product: `einmo = "0.1"` as a dependency (FOOP 25's server, build scripts, other people's harnesses) with the binaries riding along; no separate `cargo-einmo` crate — `cargo install einmo` installs both binaries. |
| 2 | **Name availability** | Check `einmo` on crates.io *first* (`cargo search einmo`, and the web UI for squatting). If taken, the fallback naming decision gates everything downstream — resolve with human before proceeding. |
| 3 | **Own repository?** | Yes (this FOOP's position): `einmo` becomes its own repo. The monorepo keeps the *suite* (`foolish-ubca/einmo_suite/`) and consumes einmo as a dependency. |
| 4 | **Repo host + owner** | Human decision (github.com/<owner>/einmo assumed below). |
| 5 | **What moves** | `einmo/` (crate), `einmo.README.md` → `README.md`, new `LICENSE-MIT` + `LICENSE-APACHE`, CI workflows. `zweimomo` stays in the monorepo (comparison harness, not product). The FOOP docs stay (they are Foolish process history). |
| 6 | **Version + MSRV policy** | Publish as `0.1.0`, semver with 0.x minor-bump-on-break; pin MSRV (`rust-version` in Cargo.toml) to the oldest toolchain CI proves, and test it. |
| 7 | **Publish before or after FOOP 25?** | Before. FOOP 25 (review server) is additive API; shipping 0.1.0 first gives it a stable base and real users' feedback. |
| 8 | **`einmo test` pass policy** | See §S.4 — checked-correspondence is the default gate; `--verified` escalates. |

### S.2 The `test` subcommand — the reason `cargo einmo test` exists

New CLI verb (the one piece of real feature work in this FOOP):

```
einmo test [SUITE] [--filter <substr>] [--level checked|verified] [--update-output]
           [--evaluator <cmd>] [--jobs N] [--json]
```

- **Suite discovery**: explicit `SUITE` argument, else walk upward from CWD for an `einmo.toml`
  (the existing `config.rs` TestConfig); a workspace may list multiple suites.
- **Evaluator**: einmo stays evaluator-agnostic — the suite's `einmo.toml` names the command that
  turns an input into output (for Foolish: the UBCa runner). `--evaluator` overrides.
- **Semantics**: evaluate all inputs (respecting `--jobs`), write/refresh `output/`, then for each
  test: verify every stamp chain (verify-on-inspect — a tampered artifact is a FAILURE, not a skip)
  and compare output↔checked (`--level checked`, the default) or additionally checked↔verified
  (`--level verified`, the PR-merge tier per FOOP 64's two-tier gate).
- **Exit codes**: 0 all green; 1 drift/mismatch; 2 verification failure (tampering — always fatal);
  101 configuration/evaluator errors. `--json` emits a machine-readable report for CI annotation.
- **`cargo einmo test`** then needs **no registration with cargo at all** — that is the walkthrough's
  key teaching: cargo runs any `cargo-<name>` binary found on `$PATH` as `cargo <name>`. The alias
  binary already handles cargo's injected `einmo` argv token. Installation *is* the registration.

### S.2b Multi-stage promotion in one go — DEFERRED to FOOP 25's session model

A recurring want (poor_einmo's `\Y`: promote a file `output->checked` AND `checked->verified` in one
action, prompting for the passphrase once) is **deliberately NOT built here**. A CLI-only version
(`einmo promote … :: …` chaining segments in one invocation) was considered and rejected as premature:
it would build a workaround for the thing FOOP 25 makes real. In FOOP 25's `EinmoReview`, pending
promotions already live **in memory** as the reviewer's decision set (so a UI can update what the user
wants before anything is signed); executing that set with one derived `Signer` (FOOP 25 §S.4) IS
"promote several stages, one passphrase," done properly. Any `::` CLI syntax later becomes a thin
argument-parsing convenience over that session primitive, not the mechanism itself.

Until then, the workflow is two explicit steps — `\c` then `\v` in poor_einmo, or two `einmo promote`
commands — which is correct, just not one keystroke. (Note for whoever revisits: the CLI `::` parse,
if ever wanted, is `split_promote_chain(raw)` splitting on `"::"` argv elements, each segment through
the existing `split_promote_args`, executed left-to-right under one resolved key. `::` was chosen over
`--`/`---` because those are a shell end-of-options marker / a literal einmo filename argument and
would corrupt a pasted command.)

### S.3 Crate completeness (what crates.io will demand or docs.rs will shame)

- `LICENSE-MIT` and `LICENSE-APACHE` files (declared license currently has no backing texts).
- Cargo.toml additions: `repository`, `homepage`, `documentation` (docs.rs), `readme = "README.md"`,
  `keywords = ["snapshot", "testing", "signing", "approval-testing", "cargo-subcommand"]`,
  `categories = ["development-tools::testing", "command-line-utilities"]`, `rust-version`,
  `include`/`exclude` so the package tarball is lean (no test corpora).
- `#![forbid(unsafe_code)]` at the crate root (einmo has no unsafe; make it a guarantee).
- **Public library surface audit** — the crate ships as a dependency, so what is `pub` is a semver
  promise: walk `lib.rs` and every module, deliberately choosing the exported API (suite,
  format/envelope, stamps/signing, verify, transitions, compare); `#![deny(missing_docs)]` with a
  rustdoc example on every public item; an `examples/` directory with at least "drive a suite from
  Rust" and "verify an envelope programmatically".
- README rewrite for an external audience, covering BOTH products: library quickstart (embed a
  suite, verify programmatically) and CLI quickstart (install, suite setup, `cargo einmo test` in
  CI), the promotion pipeline, key roles, security model.

### S.4 Repository extraction (history preserved)

1. Fresh clone of the monorepo; `git filter-repo --subdirectory-filter einmo/` (fallback:
   `git subtree split -P einmo`) → einmo's commit history stands alone.
2. Graft `einmo.README.md` history as `README.md`; add LICENSE files, CI, `.gitignore`.
3. Create the remote (S.1 #4), push, tag `v0.1.0-rc0`.
4. Back in the monorepo: remove `einmo/` from the workspace, add the dependency back —
   `einmo = { git = "…" }` immediately, switched to `einmo = "0.1"` after S.6, with an optional
   `[patch]` path override for local co-development while FOOP 25 is being built.
5. The suite, `poor_einmo.sh`, and the gates keep working against the installed/`git`-dep binary;
   the monorepo's CI installs einmo rather than building it in-tree.

### S.5 Local install and dogfood (before any publish)

- `cargo install --path . --locked` from the new repo → verifies both binaries land in
  `~/.cargo/bin` and `cargo einmo test` runs the Foolish suite end-to-end from a clean shell.
- `einmo self-check` (existing verb) wired into the new repo's own CI as its smoke test.

### S.6 crates.io registration and publication

1. crates.io account with **verified email** (publish is refused without it); org/owner choice.
2. `cargo login` with a token scoped to publish (prefer a scoped token; store nowhere in-repo).
3. `cargo publish --dry-run --locked` — fixes the include list, missing metadata, path leaks.
4. `cargo package --list` — eyeball the tarball contents (no keys, no corpora, no scratch).
5. `cargo publish`; verify the docs.rs build; `cargo install einmo` from a machine that has never
   seen the repo; tag `v0.1.0`.
6. Add co-owners (`cargo owner --add`) so the bus factor is > 1.

### S.7 The testing battery (recommended additional Rust testing)

In rough order of value for *this* crate — a tool whose input is untrusted files and whose output is
trust:

| Technique | Target in einmo | Tooling |
|-----------|-----------------|---------|
| CLI integration tests | every verb's exit codes/stdout against fixture suites | `assert_cmd` + `predicates`, or `trycmd` for CLI-snapshot files |
| Property-based tests | envelope/stamps **roundtrip** (`parse(serialize(x)) == x`), body-extraction stability, promotion idempotence | `proptest` |
| **Fuzzing** | the envelope parser, STAMPS section parser, and signature-line decoders — the untrusted-input surface | `cargo-fuzz` (3 targets), corpus seeded from the suite |
| Mutation testing | does the test suite actually catch a broken verifier? | `cargo-mutants` (run on `signature.rs`/`verify.rs` at minimum) |
| Supply-chain gates | licenses, advisories, duplicate deps | `cargo-deny check` + `cargo audit` in CI |
| MSRV proof | `rust-version` honesty | `cargo-msrv verify` job |
| Coverage | floor on `format`/`signature`/`verify`/`transitions` | `cargo llvm-cov` with a ratchet, not a vanity % |
| Doc tests | every README/lib.rs example compiles and runs | `cargo test --doc` in CI |
| Benches (optional) | verify throughput on a 1k-file suite | `criterion`, informational only |

CI matrix (GitHub Actions assumed): {linux, macos, windows} × {stable, MSRV}; jobs: fmt, clippy
`-D warnings`, test, doc test, deny/audit, msrv, fuzz-smoke (short), coverage upload. Release
workflow: tag `vX.Y.Z` → dry-run → publish → GitHub release with prebuilt binaries
(`cargo-dist` or plain matrix builds; also enables `cargo binstall einmo`).

### S.8 Documentation and hand-back

- The new repo's README owns user documentation; `einmo.README.md` in the monorepo shrinks to a
  pointer plus the Foolish-specific suite conventions.
- AGENTS.md build-commands section updated: `cargo einmo test` replaces the bespoke invocations
  where appropriate; the FOOP 64 gate scripts point at the installed binary.
- CHANGELOG.md (keep-a-changelog) started at 0.1.0 in the new repo.

## FIR Impact

None. Tooling, packaging, and process only.

## UBC Step Impact

None.

## Test Plan

- Unit + integration tests for the new `test` verb: green suite → 0; drift → 1; tampered stamp → 2;
  missing evaluator → 101; `--level verified` escalation; `--filter`; `--json` schema.
- The §S.7 battery is itself the deliverable — each row lands as a working CI job in the new repo;
  the plan checks them off individually.
- End-to-end dogfood: from a clean container, `cargo install einmo`, clone the monorepo, run
  `cargo einmo test foolish-ubca/einmo_suite --level checked` and get the same verdict the in-tree
  binary gives.
- Comprehensive test, adapted (meta/tooling FOOP — no FVM surface, the reserved
  `foop_35_comprehensive.foo` does not apply): the clean-container dogfood run above, scripted, is
  the comprehensive test.

## Rejected Alternatives

### A. Publish from inside the monorepo (no repo extraction)

crates.io publishing works fine from a subdirectory, so extraction is not technically required.
Rejected because the repo *is* part of the product for a dev-tool: issues, releases, CI badges, and
docs live where users look, and monorepo CI noise (Foolish FVM churn) would gate einmo releases.
The crate was built dependency-free precisely to leave (FOOP 92 §1).

### B. A separate `cargo-einmo` wrapper crate

Some subcommands ship the alias as its own crate. Rejected: one crate installing both binaries is
simpler for users (`cargo install einmo` — done) and the alias already exists in-tree.

### C. Make `cargo test` run einmo suites via a custom test harness

A `#[test]`-harness shim would let plain `cargo test` drive suites. Rejected for now: it couples
einmo to each project's test binary and hides the staged-promotion semantics behind an interface that
expects pass/fail only; `cargo einmo test` in CI is one line. Revisit as a follow-up crate
(`einmo-harness`) if users ask.

### D. Do nothing

Einmo remains a monorepo-internal tool; FOOP 64's gates keep hand-rolled invocations; nobody outside
Foolish can adopt it. Rejected: the tool is finished enough that shipping it is cheaper than
maintaining its privacy.

## Open Questions

- Crates.io name availability for `einmo` (S.1 #2) — check at begun-time; fallback name needs human
  choice.
- Repo host/owner (S.1 #4) and whether the monorepo consumes einmo via git-dep or published version
  during the FOOP 25 co-development window (`[patch]` recommended).
- Whether prebuilt release binaries ship from day one (`cargo-dist`) or post-0.1.
- Evaluator contract in `einmo.toml` for non-Foolish users — is the current TestConfig command shape
  general enough, or does `einmo test` need a documented protocol (stdin/stdout vs argv/file)?

## References

- FOOP-92 (Complete) §1 — einmo built standalone "so it can be promoted to its own repository";
  `einmo.README.md` (the alias, key roles, format).
- FOOP-64 — the flagship suite and two-tier gate `cargo einmo test` will serve; FOOP-25 — the
  review server that will ride on the published crate.
- Code: `einmo/Cargo.toml` (bin targets already declared), `einmo/src/bin/cargo_einmo.rs` (argv
  strip already correct), `einmo/src/{cli,config,einmo_suite}.rs` (where `test` lands).
- External: crates.io publishing docs; cargo book §"Custom subcommands"; docs.rs metadata; tools
  named in §S.7 (`assert_cmd`, `trycmd`, `proptest`, `cargo-fuzz`, `cargo-mutants`, `cargo-deny`,
  `cargo-msrv`, `cargo-llvm-cov`, `cargo-dist`, `cargo-binstall`).
