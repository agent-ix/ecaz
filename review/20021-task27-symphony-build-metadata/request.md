# Review Request: Task 27 Slice 5 — Symphony Build Metadata Bootstrap

Scope: land the first real Symphony build-time behavior: its own AM
reloptions callback, metadata-page bootstrap writer, and an honest
`ambuild` / `ambuildempty` seam for empty indexes on `pg18`.

Task: `plan/tasks/27-symphony-access-method.md` Phase 1
("Wire format" + first "Build path" scaffold).

Branch: `task27-symphony-stage2-phase0-oracle` (slice 5 builds on
`d253c46`).

Files in scope:
- `src/am/common/metadata.rs`
- `src/am/common/mod.rs`
- `src/am/symphony/build.rs`
- `src/am/symphony/mod.rs`
- `src/am/symphony/options.rs`
- `src/am/symphony/routine.rs`

Validation:
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `cargo pgrx install --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --no-default-features --features pg18`
- `cargo pgrx start pg18`
- empty-build smoke in fresh database:
  - `psql -h /home/peter/.pgrx -p 28818 -d postgres -Atc "DROP DATABASE IF EXISTS symphony_build_smoke"`
  - `psql -h /home/peter/.pgrx -p 28818 -d postgres -Atc "CREATE DATABASE symphony_build_smoke"`
  - `psql -h /home/peter/.pgrx -p 28818 -d symphony_build_smoke -Atc "CREATE EXTENSION ecaz; CREATE TABLE empty_fixture (embedding ecvector); CREATE INDEX symphony_empty_idx ON empty_fixture USING symphony (embedding ecvector_symphony_ip_ops); SELECT 'ok:' || indexrelid::regclass::text FROM pg_index WHERE indexrelid = 'symphony_empty_idx'::regclass;"`
  - observed result: `ok:symphony_empty_idx`
- populated-build guard smoke in same database:
  - `psql -h /home/peter/.pgrx -p 28818 -d symphony_build_smoke -Atc "CREATE TABLE nonempty_fixture (embedding ecvector); INSERT INTO nonempty_fixture VALUES ('[1,2,3]'::ecvector); CREATE INDEX symphony_nonempty_idx ON nonempty_fixture USING symphony (embedding ecvector_symphony_ip_ops);"`
  - observed result: `ERROR:  symphony ambuild for populated relations is not implemented yet`

Notes on validation:
- The long-lived local `postgres` database had an older `ecaz`
  installation whose catalog still referenced `ec_diskann_handler`.
  Dropping or reusing that database was misleading, so the smoke checks
  above use a fresh `symphony_build_smoke` database after reinstalling
  the current `pg18` extension artifacts.

## What landed

### 1. Shared metadata-page bootstrap helper

`src/am/common/metadata.rs` now owns a tiny generic helper that writes a
metadata payload into block 0 with WAL, without reaching back into
`ec_hnsw::shared`.

This is intentionally narrow:

- pick block 0 or `P_NEW`
- initialize the page special space for the metadata payload
- copy bytes and finish the generic WAL record

That gives Symphony a legal metadata-page bootstrap without violating the
ADR-041 module boundary.

### 2. Symphony-owned reloptions callback

`symphony` now has its own `amoptions` callback and reloptions reader in
`src/am/symphony/options.rs`.

This fixes the pg18 `CREATE INDEX` abort seen before the slice:

```text
Assert("amoptions != NULL"), File: "reloptions.c", Line: 2092
```

The first Symphony reloptions surface is deliberately small:

- `m`
- `ef_construction`
- `padding_factor`

That is enough to persist the Stage-2 metadata fields already frozen in
packet 20018 without pulling scan or insert concerns into this slice.

### 3. Real metadata initialization in `ambuildempty`

`symphony_ambuildempty` now writes a V5 Symphony metadata page instead of
erroring out.

The metadata it writes includes:

- reloption-backed `m`
- reloption-backed `ef_construction`
- reloption-backed `padding_factor`
- `rabitq_bits = 1`
- `entry_point = INVALID`
- `dimensions = 0`
- `max_level = 0`
- `inserted_since_rebuild = 0`
- a freshly generated per-index rotation seed

The seed point matters: task 25’s handoff requires Symphony rotations to
live per-index in metadata rather than falling back to a process-global
default.

### 4. Honest `ambuild` behavior for this stage

`symphony_ambuild` now does one real thing and one explicit non-thing:

- if the heap is empty, it bootstraps the metadata page and returns an
  empty `IndexBuildResult`
- if the heap has visible tuples, it errors with
  `symphony ambuild for populated relations is not implemented yet`

This is the right seam for the next slices:

- empty-index creation works on pg18
- callers do not get a fake partial graph build
- the failure mode for non-empty relations is explicit and testable

### 5. Narrow unit coverage

`src/am/symphony/build.rs` retains a Rust unit test that checks the
bootstrap metadata defaults and V5 field mapping.

## What this slice intentionally does NOT do

- no heap scan + encode path for populated builds
- no graph construction
- no quantization-aware pruning
- no out-degree padding pass
- no scan-time use of the metadata yet
- no insert/vacuum metadata maintenance yet

This slice is only the build/bootstrap seam needed before the graph work
can start.

## Review focus

Please focus on:

1. Whether the shared metadata writer in `src/am/common/metadata.rs` is
   the right minimum shared primitive.
2. Whether the initial Symphony reloptions surface is the right Phase-1
   minimum for metadata bootstrap.
3. Whether the `ambuild` populated-table guard is the right interim
   behavior before Stage-2 graph construction lands.

## Closing

Task 27 now has a real pg18-safe build bootstrap:

- `CREATE INDEX ... USING symphony` works on empty tables
- metadata page V5 initialization is real
- per-index RaBitQ seed storage is real
- populated-table builds fail fast with an explicit guard instead of
  pretending the graph exists

The next slice can start writing real element / neighbor tuples through
the V5 codec without first solving AM registration or metadata bootstrap.
