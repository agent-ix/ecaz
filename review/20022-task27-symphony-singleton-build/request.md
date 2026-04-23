# Review Request: Task 27 Slice 6 — Symphony Singleton Build Path

Scope: wire the first real non-empty build path through the Symphony V5
codec. This slice makes `ambuild` persist one heap row as one element +
empty-neighbor tuple pair, while multi-row heaps remain explicitly
guarded until graph construction lands.

Task: `plan/tasks/27-symphony-access-method.md` Phase 1
("Build path" first tuple-write slice).

Branch: `task27-symphony-stage2-phase0-oracle` (slice 6 builds on
`d6aabe6`).

Files in scope:
- `src/am/symphony/build.rs`

Validation:
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `cargo test --lib symphony::build::tests --no-default-features --features pg18 -- --test-threads=1`
- `cargo pgrx install --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --no-default-features --features pg18`
- `cargo pgrx start pg18`
- fresh-db smoke:
  - `psql -h /home/peter/.pgrx -p 28818 -d postgres -Atc "DROP DATABASE IF EXISTS symphony_singleton_smoke"`
  - `psql -h /home/peter/.pgrx -p 28818 -d postgres -Atc "CREATE DATABASE symphony_singleton_smoke"`
  - `psql -h /home/peter/.pgrx -p 28818 -d symphony_singleton_smoke -Atc "CREATE EXTENSION ecaz; CREATE TABLE empty_fixture (embedding ecvector); CREATE INDEX symphony_empty_idx ON empty_fixture USING symphony (embedding ecvector_symphony_ip_ops); CREATE TABLE singleton_ec_fixture (embedding ecvector); INSERT INTO singleton_ec_fixture VALUES ('[1,2,3]'::ecvector); CREATE INDEX symphony_singleton_ec_idx ON singleton_ec_fixture USING symphony (embedding ecvector_symphony_ip_ops); CREATE TABLE singleton_tq_fixture (embedding tqvector); INSERT INTO singleton_tq_fixture VALUES ('[dim=3,bits=4,seed=42,gamma=1.0]:123456'::tqvector); CREATE INDEX symphony_singleton_tq_idx ON singleton_tq_fixture USING symphony (embedding tqvector_symphony_ip_ops); SELECT 'empty=' || pg_relation_size('symphony_empty_idx') || ',singleton_ec=' || pg_relation_size('symphony_singleton_ec_idx') || ',singleton_tq=' || pg_relation_size('symphony_singleton_tq_idx');"`
  - observed result: `empty=8192,singleton_ec=16384,singleton_tq=16384`
- multi-row guard smoke:
  - `psql -h /home/peter/.pgrx -p 28818 -d symphony_singleton_smoke -Atc "CREATE TABLE nonempty_fixture (embedding ecvector); INSERT INTO nonempty_fixture VALUES ('[1,2,3]'::ecvector), ('[4,5,6]'::ecvector); CREATE INDEX symphony_nonempty_idx ON nonempty_fixture USING symphony (embedding ecvector_symphony_ip_ops);"`
  - observed result: `ERROR:  symphony ambuild for populated relations is not implemented yet`

## What landed

### 1. Heap scan now distinguishes 0 / 1 / >1 visible rows

`symphony_ambuild` no longer treats every populated heap the same. It now
scans just far enough to classify the build input as:

- empty
- singleton
- multi-row

That is the minimum needed to land a correct non-empty build slice
without faking a graph for the general case.

### 2. First real tuple-write path through the V5 codec

For the singleton case, the builder now creates and persists:

- one `SymphonyNeighborTuple` with `count = 0`
- one `SymphonyElementTuple` pointing at that neighbor tuple

written through:

- `page::DataPageChain`
- `insert_symphony_neighbor`
- `insert_symphony_element`
- a Symphony-owned WAL page flush helper

This is the first time a non-empty `symphony` index writes data block 1
through the V5 tuple codec instead of only block-0 metadata.

### 3. Metadata rewrite after tuple persistence

The build now bootstraps metadata first so block 0 exists, writes the
singleton data page, then rewrites metadata with final singleton state:

- `entry_point` = the persisted element tuple TID
- `dimensions` = indexed vector dimension from the heap datum
- same per-index RaBitQ seed as the bootstrap metadata

That keeps the metadata truthful for the smallest valid graph instance.

### 4. Indexed-column dimension decode for both supported heap types

This slice adds minimal indexed-column shape resolution directly in
`src/am/symphony/build.rs`:

- resolve the single indexed heap attribute from `IndexInfo`
- reject expression / partial indexes
- classify the indexed column as `ecvector` or `tqvector`
- derive dimensions from the on-datum binary payload

Important detail: this works for both operator classes already exposed in
bootstrap SQL:

- `ecvector_symphony_ip_ops`
- `tqvector_symphony_ip_ops`

The fresh-db smoke confirms both singleton cases build successfully.

### 5. Guard remains explicit for real graph work

Multi-row heaps still fail fast with the same explicit guard:

`symphony ambuild for populated relations is not implemented yet`

That is still the right boundary for this slice: singleton is fully
correct with zero adjacency, but anything larger needs real graph
construction, padding, and quantization-aware pruning.

## Why the size numbers matter

The fresh-db smoke gives a cheap physical sanity check:

- empty Symphony index: `8192` bytes = metadata page only
- singleton Symphony index: `16384` bytes = metadata page + first data page

That is the simplest end-to-end proof that the tuple-write path is real.

## Unit coverage

Added / updated Rust tests for:

- metadata defaults
- singleton page-chain layout (`neighbor` then `element`)
- binary dimension extraction for both `ecvector` and `tqvector`

## What this slice intentionally does NOT do

- no build path for `>1` visible row
- no graph construction
- no centered RaBitQ neighbor-code persistence
- no out-degree padding
- no quantization-aware pruning
- no scan support for the singleton-built index yet

This slice is only about crossing the boundary from metadata-only build to
first real V5 tuple persistence.

## Review focus

Please focus on:

1. Whether singleton-only non-empty build is the right narrow checkpoint
   before general graph construction.
2. Whether the minimal indexed-column decode in `build.rs` is a sound
   temporary seam without reaching back into `ec_hnsw`.
3. Whether the metadata rewrite after tuple persistence is the right
   ordering for block 0 / block 1 ownership.

## Closing

Task 27 now has a real non-empty build seam:

- empty builds still work
- singleton `ecvector` builds work
- singleton `tqvector` builds work
- the first data page is physically written through the Symphony V5 codec
- multi-row builds still fail explicitly until graph construction lands

The next slice can expand this from one persisted vertex to a real
multi-vertex graph flush path.
