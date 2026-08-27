# Task 235 DistANN suite cleanup artifact manifest

Date: 2026-08-26 (America/Los_Angeles)

## Scope and head

- Task bucket: `reviews/task-235/`
- Packet: `005-suite-run-dir-cleanup/`
- Head: `d491fd37e3f06dfc68655c43fb478084da8c5b92` (cleanup implementation
  `dc3ddbae5e4b4f6f49c12299670da076f20d4b6b`).
- Build output: inherited shared `CARGO_TARGET_DIR=/home/peter/.cargo-target`.
- Validation runtime: 2026-08-26 15:33:11--15:34:27 PDT.
- Runtime fixture: none. No PostgreSQL process, PGDATA, corpus, truth cache, or
  benchmark run directory was created.

## Commands

```text
cargo check -p ecaz-cli
cargo test -p ecaz-cli runtime_paths_
cargo fmt --all --check
git diff --check -- crates/ecaz-cli/src/commands/bench/suite.rs \
  crates/ecaz-cli/src/commands/dev/mod.rs
```

## Results

- `cargo check -p ecaz-cli`: passed; one inherited dead-code warning in
  `crates/ecaz-cli/src/commands/corpus/load.rs`.
- `cargo test -p ecaz-cli runtime_paths_`: 2 passed, 0 failed, 536 filtered.
- `cargo fmt --all --check`: passed with the repository's inherited stable-
  rustfmt warnings about nightly-only import settings.
- `git diff --check`: passed.

## Artifacts

- `cargo-check-ecaz-cli.log` — shared-target compile check; SHA-256
  `6c4034a56dad1a7b49a07473ef3b9fa177fc0d7843bb899afc2e01f48db57457`.
- `cargo-test-runtime-paths.log` — two pure path-policy unit tests; SHA-256
  `0e937d589c26f5a6230f81f54bf2a780eaadf654e76a1436d0b7a5d1f720d0a3`.

The packet contains no operational logs or cluster state.

## Reviewer seq-01 follow-up

- Checkpoint: `d491fd37e3f06dfc68655c43fb478084da8c5b92`.
- Validation runtime: 2026-08-26 17:17:39--17:18:06 PDT.
- `cargo fmt --all --check`: passed with the inherited stable-rustfmt warnings
  about nightly-only import settings.
- `git diff --check -- crates/ecaz-cli/src/commands/bench/suite.rs`: passed.
- `cargo check -p ecaz-cli`: passed against inherited shared
  `CARGO_TARGET_DIR=/home/peter/.cargo-target`; the existing dead-code warning
  in `crates/ecaz-cli/src/commands/corpus/load.rs` remains.
- `cargo-check-error-precedence.log` — follow-up shared-target compile check;
  SHA-256
  `d3a5d050d11e3c7c76d7097111d1341ca6f4d4f448315cacb93b577600453869`.
- Tests were not run for this narrow failure-precedence correction.
- No PostgreSQL node, PGDATA directory, corpus, truth cache, fixture, or
  benchmark run directory was created.
