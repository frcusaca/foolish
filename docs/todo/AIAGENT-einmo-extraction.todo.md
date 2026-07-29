# einmo extraction — TODO

> **Note:** This file was hand-written because the `/todo` skill referenced by
> `CLAUDE.md`/`AGENTS.md` is not installed in this session. Reconcile with the
> skill's expected format once it is available.

## Log

- 2026-07-29 06:40:03 — Session started. Plan: (1) extract the `einmo` crate
  from this workspace into a new standalone repo at `~/yolo/einmo` (git init,
  publishable `Cargo.toml`, version `0.0.5`, MIT license, bring over
  `einmo.README.md` as the new repo's `README.md` and `poor_einmo.sh` renamed
  to `experimental_reviewer.sh`); (2) rewire `foolish-ubca` and `zweimomo` to
  depend on `einmo` via a path dependency pointing at `~/yolo/einmo` instead of
  the in-tree `einmo/` directory, then remove the in-tree copy; (3) record a
  follow-up item (below) rather than act on it now.

- 2026-07-29 06:44:43 — User decided to defer the workspace rewiring: keep
  `foolish-ubca`/`zweimomo` on the in-tree `einmo/` path dependency for now.
  Reverted the `Cargo.toml` edits (root workspace members, `foolish-ubca`,
  `zweimomo`) and restored the in-tree `einmo/` directory that had been
  removed — `cargo check --workspace` confirmed back to the original
  passing state. `~/yolo/einmo` (the extracted standalone copy) is kept as
  the future published home; it stays a second, currently-unwired copy until
  the items below are done.

## Open items

- [ ] Add tests and documentation to the standalone `~/yolo/einmo` repo (it
  currently has no dedicated test suite of its own — its only exercise today
  is via `foolish-ubca`/`zweimomo` in the foolish-rust workspace).
- [ ] Publish `einmo` to crates.io once tests/docs are in place.
- [ ] Switch `foolish-ubca` and `zweimomo` from the in-tree `einmo/` path
  dependency to the published crates.io `einmo` dependency, then remove the
  in-tree `einmo/` directory and drop it from the root `Cargo.toml` workspace
  members.
- [ ] Specify and implement a complex demo test in `zweimomo`, restructured as
  its own separately-compiling crate. Currently `zweimomo` is heavy —
  `rustpython-vm` and `boa_engine` pull in FFI-ish/apt-install build
  dependencies — and this weight is a blocker the user wants addressed
  distinctly from the einmo move. Scope as its own Major-sized item per
  AGENTS.md segmentation (likely warrants its own FOOP) — not done as part of
  the einmo extraction.
