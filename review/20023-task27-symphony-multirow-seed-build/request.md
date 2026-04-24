# Review Request: Task 27 Slice 7 — Symphony Multi-Row Seed Build Path

Scope: extend Symphony `ambuild` from the singleton-only checkpoint to a
real populated-table path that persists every visible heap row as a V5
element + empty-neighbor tuple pair. This slice still does not build the
real graph; it only removes the `>1 row` guard and lands deterministic
multi-vertex flush behavior.

Task: `plan/tasks/27-symphony-access-method.md` Phase 1
("Build path" first multi-vertex flush slice).

Branch: `task27-symphony` (slice 7 builds on `63ab592`).

Files in scope:
- `src/am/symphony/build.rs`

Validation:
- `cargo test --lib symphony::build::tests --no-default-features --features pg18 -- --test-threads=1`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `cargo pgrx test pg18 --no-default-features --features pg18`
- direct fresh-db smoke after `cargo pgrx start pg18`:
  - `psql -h /home/peter/.pgrx -p 28818 -d postgres -Atc "DROP DATABASE IF EXISTS symphony_multi_build_smoke;"`
  - `psql -h /home/peter/.pgrx -p 28818 -d postgres -Atc "CREATE DATABASE symphony_multi_build_smoke;"`
  - `psql -h /home/peter/.pgrx -p 28818 -d symphony_multi_build_smoke -Atc "CREATE EXTENSION ecaz;"`
  - `psql -h /home/peter/.pgrx -p 28818 -d symphony_multi_build_smoke -Atc "DROP TABLE IF EXISTS sym_ec; CREATE TABLE sym_ec (id int, v ecvector(3)); INSERT INTO sym_ec VALUES (1, '[1,0,0]'), (2, '[0,1,0]'), (3, '[0,0,1]'); CREATE INDEX sym_ec_idx ON sym_ec USING symphony (v); SELECT 'sym_ec=' || pg_relation_size('sym_ec_idx');"`
  - observed result: `sym_ec=16384`
  - `psql -h /home/peter/.pgrx -p 28818 -d symphony_multi_build_smoke -Atc "DROP TABLE IF EXISTS sym_tq; CREATE TABLE sym_tq (id int, v tqvector); INSERT INTO sym_tq VALUES (1, encode_to_tqvector(ARRAY[1.0,0.0,0.0], 4, 42)), (2, encode_to_tqvector(ARRAY[0.0,1.0,0.0], 4, 42)), (3, encode_to_tqvector(ARRAY[0.0,0.0,1.0], 4, 42)); CREATE INDEX sym_tq_idx ON sym_tq USING symphony (v); SELECT 'sym_tq=' || pg_relation_size('sym_tq_idx');"`
  - observed result: `sym_tq=16384`

## What landed

### 1. Build-time heap scan now supports arbitrary visible-row counts

`scan_heap_for_build_input` no longer stops at `0 / 1 / >1`. It now:

- walks all visible heap rows
- collects every heap TID that must become a Symphony vertex
- derives and validates indexed-column dimensions on every row
- errors if the heap mixes dimensions within one index build

That is the minimum real populated-build seam before graph construction
can start consuming the same scan result.

### 2. Non-empty builds persist a deterministic seed graph

Every visible heap row now materializes as:

- one empty `SymphonyNeighborTuple`
- one level-0 `SymphonyElementTuple` pointing at that neighbor tuple

The builder emits those pairs for the full heap into a
`page::DataPageChain`, then WAL-flushes the staged chain to the index
relation.

Important detail: this is still a **seed graph**, not the final Stage-2
graph. Adjacency remains empty until the real graph builder lands.

### 3. Entry-point and metadata rewrite now work for any non-empty build

The metadata update path is no longer singleton-specific. For any
non-empty build it now records:

- `entry_point` = the first persisted element tuple TID
- `dimensions` = the validated build-time dimension

That keeps block 0 truthful even when the build spans multiple pages.

### 4. Multi-page spill is covered in unit tests

The Rust tests now cover:

- multiple persisted vertices on one page
- overflow of the staged seed graph onto later data blocks
- dimension-mismatch rejection
- empty-input rejection for the seed-graph helper

This gives the next graph-construction slice a stable page-flush seam to
build on.

### 5. `pg_test` warning cleanup

The local unit-test module in `build.rs` is now `#[cfg(test)]` only.
That avoids pulling a dead imports block into the `pg_test` feature build
that `cargo pgrx test pg18` uses.

## Why the smoke result matters

The direct `pg18` smoke proves the behavior change this slice claims:

- populated `ecvector` Symphony builds no longer error
- populated `tqvector` Symphony builds no longer error
- both create a physical two-page relation (`16384` bytes) for the
  current three-row seed graph

The exact size is expected here because the build is still writing one
metadata page plus one first data page with empty adjacency.

## What this slice intentionally does NOT do

- no real graph edges yet
- no centered per-neighbor RaBitQ codes yet
- no quantization-aware pruning
- no out-degree padding
- no scan-path use of the populated build output yet

This slice only lands the first truthful multi-row flush path.

## Review focus

Please focus on:

1. Whether a deterministic empty-adjacency seed graph is the right
   interim populated-build seam before the real Stage-2 graph builder.
2. Whether first-element entry-point selection is the right temporary
   policy for this checkpoint.
3. Whether the dimension-consistency check belongs in the heap scan at
   this stage.

## Closing

Task 27 now has a real populated-table build seam:

- singleton and multi-row builds both persist V5 tuples
- populated `ecvector` and `tqvector` `CREATE INDEX ... USING symphony`
  succeed on `pg18`
- staged page emission spans multiple blocks when needed
- metadata stays consistent for any non-empty seed build

The next slice can replace the empty adjacency with real graph
construction instead of still fighting the tuple-write path.
