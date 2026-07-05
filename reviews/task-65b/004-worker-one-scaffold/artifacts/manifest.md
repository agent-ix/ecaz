# Task 65b Worker-One Scaffold Artifact Manifest

- code commits under review:
  - `a816c2ccf` — align DiskANN worker scaffold with PG worker controls
  - `e979a7d80` — address Slice C locking design feedback
  - `156c9af2b45f56a14053d69ebf32981b7426ddb0` — add worker=0 parity test
- task bucket: `reviews/task-65b/004-worker-one-scaffold/`
- timestamp: `2026-06-05T00:37:51Z`
- host lane: local M5, PG18 via Homebrew PostgreSQL 18
- surface: `ec_diskann` build scaffold, no corpus benchmark run

## Code Change

`src/am/ec_diskann/routine.rs`

- sets `amcanbuildparallel = true`
- wires `amestimateparallelscan`, `aminitparallelscan`, and
  `amparallelrescan` to the common AM parallel callbacks

`src/am/ec_diskann/ambuild.rs`

- reads PostgreSQL `IndexInfo::ii_ParallelWorkers`
- passes that count into `BuildParallelConfig`
- records parallel scaffold fields in `ec_diskann_ambuild_timing`

`src/am/ec_diskann/options.rs`, `src/am/ec_diskann/mod.rs`

- removes the custom `parallel_build_workers` reloption
- keeps `parallel_build_batch_size` and `parallel_build_flush_rate`

`src/am/ec_diskann/build.rs`

- adds `BuildParallelConfig` and `BuildParallelStats`
- supports `requested_workers = 0` serial fallback
- supports `requested_workers = 1` rayon scaffold
- rejects `requested_workers > 1`
- rejects non-default `parallel_build_batch_size` / `parallel_build_flush_rate`
- adds worker=0 and worker=1 parity tests

`spec/adr/ADR-075-diskann-graph-build-worker-stepping-stone.md`

- documents rayon as a graph-core stepping stone only
- keeps PostgreSQL as worker-count authority
- states Gate #6 disposition and migration trigger

## Validation

`cargo-fmt-check.log`

- command: `cargo fmt --check`
- result: passed
- note: stable-rustfmt emitted existing warnings about nightly-only
  `imports_granularity` and `group_imports`

`cargo-check-pg18-lib.log`

- command: `cargo check -p ecaz --lib --no-default-features --features pg18`
- result: passed, finished in `10.67s`

`cargo-test-ec-diskann-single-thread-escalated.log`

- command: `cargo test -p ecaz --lib --no-default-features --features pg18 am::ec_diskann -- --test-threads=1`
- run mode: escalated because pgrx test install writes to the Homebrew
  PostgreSQL extension directory
- result: passed, `190 passed; 0 failed; 1777 filtered out`
- duration: `83.09s`

## Sandbox Notes

`cargo-test-ec-diskann.log` and `cargo-test-ec-diskann-single-thread.log`
document failed non-escalated attempts. Both failed at pgrx extension install:

- failed writing `ecaz.control` to
  `/opt/homebrew/share/postgresql@18/extension/ecaz.control`
- `Operation not permitted (os error 1)`

The escalated single-threaded log is the authoritative passing test artifact.
