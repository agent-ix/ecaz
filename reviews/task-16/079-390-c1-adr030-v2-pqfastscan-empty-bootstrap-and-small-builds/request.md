# Review Request: C1 ADR-030 V2 PqFastScan Empty Bootstrap And Small Builds

## Context

The branch already had first-class `PqFastScan` build, scan, built-index insert,
and built-index vacuum support.

But two practical parity gaps remained:

- `PqFastScan` build rejected small source sets because grouped codebook training
  insisted on at least 16 training samples
- live insert into an empty `PqFastScan` index still failed because there were
  no persisted grouped codebooks yet

That left `storage_format = 'pq_fastscan'` awkwardly dependent on a non-empty
build path before the index became usable.

## Problem

Task 15 wants `PqFastScan` to behave like a first-class format on `main`.

Without this slice:

1. small tables could not build a `PqFastScan` index at all
2. an empty `PqFastScan` index could be created, but the first insert failed
3. runtime still assumed grouped codebooks had to come from a prior non-empty
   build, which is not a clean first-class lifecycle

## Planned Slice

One functional checkpoint:

1. teach grouped codebook training to handle low-cardinality source sets
2. reuse the grouped build output path to bootstrap an empty `PqFastScan` index
   on first insert
3. add regression coverage for both behaviors

## Implementation

Updated:

- `src/am/build.rs`
- `src/am/insert.rs`
- `src/lib.rs`

### 1. Small-cardinality grouped builds now train deterministically

In `src/am/build.rs`:

- removed the old top-level `source_vectors.len() < 16` hard error from
  `train_build_grouped_pq_model(...)`
- changed `train_group_codebook(...)` so:
  - zero samples still error
  - fewer than 16 samples now produce a deterministic seeded fallback codebook
    instead of failing
- added `seed_group_codebook_from_small_samples(...)` to repeat the available
  samples across the 16 centroid slots

This preserves the current fixed 4-bit grouped search layout while allowing
very small source tables to build usable grouped codebooks.

### 2. Empty `PqFastScan` insert now bootstraps the index instead of rejecting

In `src/am/insert.rs`:

- removed the old empty-index `PqFastScan` reject path
- on the first insert into an empty grouped index, `run_insert_with_adapter(...)`
  now:
  - acquires the metadata-page lock
  - confirms the index is still empty under the lock
  - builds a one-tuple grouped `BuildState`
  - reuses grouped flush generation through
    `build::default_pq_fastscan_flush_output(...)`
  - writes grouped data pages with `build::write_data_pages(...)`
  - installs the generated metadata directly under the same metadata lock
- if another inserter wins the race, the loser rereads metadata and re-enters
  the normal grouped insert path

The runtime-side grouped search-code derivation error was also narrowed to the
real invariant:

- `tqhnsw PqFastScan metadata is missing persisted grouped codebooks`

instead of the old blanket “prebuilt index required” error.

### 3. Added regression coverage for both lifecycle gaps

In `src/lib.rs`:

- added `test_tqhnsw_pq_fastscan_small_source_build_writes_grouped_pages`
  which builds a 4-row grouped index and asserts:
  - grouped metadata format
  - grouped hot / rerank / neighbor tuple counts
  - grouped codebook tuple count matches metadata
  - grouped codebook head is valid
  - entry point lands on a grouped hot tuple
- replaced the old empty-index reject test with
  `test_tqhnsw_insert_bootstraps_empty_pq_fastscan_index`, which:
  - creates an empty `PqFastScan` index
  - inserts one row successfully
  - verifies grouped hot / rerank / neighbor persistence
  - verifies search-code and binary-sidecar sizes
  - verifies ordered scan returns the inserted row

### 4. Shared build helpers are now available to runtime bootstrap code

Also in `src/am/build.rs`:

- exposed `default_pq_fastscan_flush_output(...)` as `pub(super)`
- exposed `write_data_pages(...)` as `pub(super)`

This is the narrow reuse seam needed for empty-index grouped bootstrap without
duplicating grouped build serialization logic in `aminsert`.

## Measurements

No benchmark or recall rerun in this slice.

## Validation

Passed:

- `cargo check --tests`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Required full-test commands still fail on this workstation at the same known
PostgreSQL linker layer:

- `cargo test`
- `/bin/bash -lc "PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17"`

Observed unresolved PostgreSQL symbols remain in the same family, including:

- `CurrentMemoryContext`
- `PG_exception_stack`
- `error_context_stack`
- `CopyErrorData`
- `errstart`

## Outcome

This closes two real first-class lifecycle gaps for `PqFastScan`:

1. grouped build now works on very small source tables
2. empty grouped indexes can bootstrap themselves on first insert
3. grouped runtime no longer requires a separate prebuilt-index-only lifecycle

What this slice intentionally does **not** do:

- change the fixed default `PqFastScan` layout parameters
- add recall or latency measurements for the low-cardinality fallback
- remove the need for end-to-end parity proof across build / insert / vacuum /
  scan on scratch and real-corpus harnesses

## Next Slice

The remaining work is mostly landing proof and cleanup:

1. close any remaining runtime parity edges that still distinguish
   `PqFastScan` from `TurboQuant`
2. run or tighten the explicit task-15 proof surface for both formats
3. keep removing remaining legacy grouped-v2 terminology where it still leaks
