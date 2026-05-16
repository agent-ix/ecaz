# Review Request: Task 34 Comprehensive Hardening Surface

Head: `17c26f6db6401b96c29db3ccf2288552fa5c5833`

Scope:
- `Makefile`
- `scripts/hardening.sh`
- `docs/hardening.md`
- `fuzz/`
- `hardening/loom/`
- `hardening/shuttle/`
- `tests/kani_item_pointer.rs`
- `supply-chain/`
- Seed Miri tests in storage, DiskANN metadata, and SPIRE leaf V2 storage

What changed:
- Added local-first hardening targets for supply-chain checks, unsafe/static
  hygiene, expanded Miri, cargo-careful, libFuzzer/AFL, Kani, Loom, Shuttle,
  sanitizers, SQLsmith, and aggregate `hardening-local` /
  `hardening-nightly-local` lanes.
- Added `scripts/hardening.sh` so optional tools fail with install/setup text
  instead of obscure cargo-subcommand errors.
- Added hardening documentation covering prerequisites, local cadence, CI
  promotion tiers, pgrx/Miri/Kani boundaries, and manual deferrals.
- Added fuzz targets for DiskANN metadata decode, shared `ItemPointer` decode,
  and bounded vector encoding; made `fuzz/Cargo.toml` standalone so direct
  cargo checks work.
- Added pilot Kani, Loom, and Shuttle harnesses.
- Added additional `miri_` tests for shared data-page chains, DiskANN metadata,
  and SPIRE leaf V2 metadata/segment behavior.
- Initialized cargo-vet report-mode scaffolding under `supply-chain/`.
- Marked task 34 implemented in `plan/tasks/34-comprehensive-hardening.md`.

Review focus:
- Whether the Makefile surface covers every tool lane from task 34 with either
  a runnable local command or an explicit documented/manual deferral.
- Whether `scripts/hardening.sh` missing-tool behavior is actionable and does
  not hide real failures once tools exist.
- Whether the seed harnesses are meaningful but sufficiently isolated from
  normal builds.
- Whether exposing DiskANN metadata through `bench_api` is an acceptable narrow
  public test/fuzz surface.

Validation:
- `git diff --check` passed.
- `cargo check --manifest-path fuzz/Cargo.toml --bins` passed.
- `cargo test --manifest-path hardening/loom/Cargo.toml` passed.
- `cargo test --manifest-path hardening/shuttle/Cargo.toml` passed.
- `make cargo-vet` failed with the expected actionable missing-tool message
  because `cargo-vet` is not installed locally.
- `cargo test --lib miri_` compiled the test binary, then aborted before
  running tests on this macOS environment with unresolved PostgreSQL symbol
  `_BufferBlocks`; this is the existing pgrx-linked unit-test runner issue, not
  a task 34 assertion failure.

Tests intentionally not run:
- PG18 pgrx, sanitizer, SQLsmith, cargo-careful, Kani, cargo-geiger, Rudra,
  MIRAI, Flux, AFL, and full hardening aggregates remain optional/manual lanes
  until their tools and live-cluster prerequisites are installed.
