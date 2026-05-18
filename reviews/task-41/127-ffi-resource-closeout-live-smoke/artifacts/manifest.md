# Task 41 packet 127 artifact manifest

- Head SHA: `0de684c46607f137c70edf14a1bfdcace313dc09`
- Task bucket: `reviews/task-41/127-ffi-resource-closeout-live-smoke`
- Timestamp: `2026-05-18T14:39:54Z`
- Scope: Task 41 invariants #1 and #3 only. Invariant #2 is owned by another
  coder branch.

## Artifacts

### `cargo-check-ecaz-cli.log`

- Command:
  `script -q reviews/task-41/127-ffi-resource-closeout-live-smoke/artifacts/cargo-check-ecaz-cli.log cargo check -p ecaz-cli`
- Purpose: compile-check the CLI closeout change that skips Linux-only
  RLIMIT_AS smoke probes on macOS.
- Head SHA: `0de684c46607f137c70edf14a1bfdcace313dc09`
- Key result:
  - `Finished dev profile [unoptimized + debuginfo] target(s) in 0.13s`
- Notes:
  - Warnings are existing PG18 C-header warnings and existing unused imports in
    `src/am/mod.rs`.

### `make-ffi-lint.log`

- Command:
  `script -q reviews/task-41/127-ffi-resource-closeout-live-smoke/artifacts/make-ffi-lint.log make ffi-lint`
- Purpose: rerun the all-source Task 41 invariant #1/#3 static gates at the
  closeout commit.
- Head SHA: `0de684c46607f137c70edf14a1bfdcace313dc09`
- Key result lines:
  - `ffi audit passed: 101 direct C ABI functions, 288 pgrx-managed SQL entrypoints`
  - `ffi audit self-test passed`
  - `ffi lint self-test passed`
  - `ffi lint passed: raw PostgreSQL resource APIs are confined to guard modules`
  - `dylint self-test passed`
  - `Finished dev profile [unoptimized + debuginfo] target(s) in 0.10s`
- Notes:
  - The `ffi_lint.py` self-test now includes a raw `SPI_freetuptable` negative
    fixture and an allowed `src/storage/spi_guard.rs` fixture.
  - The `ffi_lint.py` self-test now also includes a raw tuple-slot API negative
    fixture and an allowed `src/storage/slot_guard.rs` fixture.

### `install-ecaz-pg-test.log`

- Command:
  `./target/debug/ecaz dev install --log-file reviews/task-41/126-dylint-ffi-guard-lint/artifacts/install-ecaz-pg-test.log ecaz-pg-test --pg 18`
- Purpose: install the current PG18 `ecaz-pg-test` build before live smoke.
- Head SHA: `ab9dc31524956e782efe6f573b2e023b9b61f8b6`
- Key result lines:
  - `[install] backend artifact assertion passed`
  - `[install] installed_backend=/opt/homebrew/lib/postgresql@18/ecaz.dylib`
  - `[install] sha256=137254eee8e131139e278fc0003ed8a83c9b684debf3a651f1913cad9c6f8588`
- Notes:
  - The log was first captured while preparing packet 126 and then moved into
    this closeout packet before commit.

### `live-ffi-leak-smoke-hnsw.log`

- Command:
  `script -q reviews/task-41/126-dylint-ffi-guard-lint/artifacts/live-ffi-leak-smoke-hnsw.log env PGDATABASE=ecaz_task41_ffi_896ecf27 PGHOST=/Users/peter/.pgrx PGPORT=28818 make ffi-leak-smoke FAULT_SMOKE_FLAGS=--rows\ 16\ --am\ hnsw`
- Purpose: run the PG18 live leak-smoke aggregate against an isolated database
  with one HNSW index surface.
- Head SHA: `ab9dc31524956e782efe6f573b2e023b9b61f8b6`
- Fixture:
  - Database: `ecaz_task41_ffi_896ecf27`
  - Storage format / AM: HNSW
  - Rerank mode: default
  - Surface isolation: one-index-per-table smoke surface in an isolated database
- Key result lines:
  - `[fault] memory_palloc_sweep_completed am=ec_hnsw lane=build first_success_nth=2`
  - `[fault] memory_palloc_sweep_completed am=ec_hnsw lane=scan first_success_nth=5`
  - `[fault] memory_palloc_sweep_completed am=ec_hnsw lane=insert first_success_nth=2`
  - `[fault] memory_palloc_sweep_completed am=ec_hnsw lane=vacuum first_success_nth=2`
  - `[fault] memory_rlimit_oom_skipped target_os=macos ams=ec_hnsw reason=linux-only`
  - `[fault] memory oom-kill ec_hnsw build postmaster_recovered=true`
  - `[fault] memory oom-kill ec_hnsw scan postmaster_recovered=true`
  - `[fault] memory oom-kill ec_hnsw insert postmaster_recovered=true`
  - `[fault] pg_buffercache_fixture_pins_ok=true pins=0`
  - `[fault] resource_accumulator_pressure am=ec_hnsw rows=4096 limit=1000 returned=1000 work_mem=64kB effective_cache_size=1MB`
  - `[fault] resource_temp_spill_accounting am=ec_hnsw mode=temp_file_limit temp_bytes_before=0 after=0 delta=0`
  - `[fault] wal_rotation_accounting am=ec_hnsw`
- Notes:
  - Expected backend-disconnect log lines appear during SIGKILL/OOM proxy
    cases; each is followed by `postmaster_recovered=true` or successful
    postcondition probes.
  - The log was first captured while preparing packet 126 and then moved into
    this closeout packet before commit.
