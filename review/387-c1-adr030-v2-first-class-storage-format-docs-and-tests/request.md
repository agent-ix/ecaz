# Review Request: C1 ADR-030 V2 First-Class Storage Format Docs And Tests

## Context

Task 15 and ADR-032 both treat `TurboQuant` and `PqFastScan` as first-class
storage formats selected per index with:

- `WITH (storage_format = 'turboquant')`
- `WITH (storage_format = 'pq_fastscan')`

The branch already had the implementation plumbing for that reloption, but the
landing surface still had two obvious gaps:

1. README still described `tqhnsw` as a single-format story
2. pg coverage heavily exercised `pq_fastscan`, but did not yet include an
   explicit `turboquant` reloption build test

That meant the implementation was ahead of the docs and the storage-format test
story was still asymmetric.

## Problem

Without this slice:

1. the public README did not tell users how to choose between the two formats
2. there was no explicit regression proving `storage_format = 'turboquant'`
   creates the expected scalar on-disk layout
3. one of the main `pq_fastscan` build tests still used the old
   `experimental_grouped_v2` naming even though it was verifying the landed
   `storage_format = 'pq_fastscan'` path

This is not a runtime bug, but it is landing debt.

## Planned Slice

One narrow checkpoint:

1. add README guidance for choosing `turboquant` vs `pq_fastscan`
2. add the missing explicit `turboquant` reloption build/layout pg test
3. rename the primary `pq_fastscan` build test away from the old
   `experimental_grouped_v2` wording

No AM behavior changes. No new runtime architecture.

## Implementation

Updated:

- `README.md`
- `src/lib.rs`

### 1. README now documents the per-index format choice

Added a short `Choosing A Format` section that:

- introduces `storage_format`
- states that `turboquant` is the default
- positions `pq_fastscan` as the measured latency-oriented format
- documents that switching formats requires `REINDEX`

### 2. Added explicit `turboquant` build/layout pg coverage

New test:

- `test_tqhnsw_turboquant_storage_format_build_writes_scalar_pages`

It verifies that an index built with
`WITH (storage_format = 'turboquant')`:

- records `storage_format=turboquant` in reloptions
- persists scalar-format metadata
- writes only scalar element + neighbor tuples
- writes no grouped hot / rerank / grouped codebook tuples
- keeps the entry point on a scalar element tuple

### 3. Renamed the main `pq_fastscan` build test to landed terminology

Renamed:

- `test_experimental_grouped_v2_source_build_writes_grouped_pages`
  →
  `test_tqhnsw_pq_fastscan_source_build_writes_grouped_pages`

Also renamed the SQL fixture table/index names in that test from
`grouped_v2_*` to `pq_fastscan_*`, and updated the final assertion message so
it no longer references an experimental build gate.

## Measurements

No benchmark or recall rerun in this slice.

## Validation

Passed:

- `cargo check --tests`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Required full-test commands still fail on this workstation at the same known
PostgreSQL linker layer as previous checkpoints:

- `cargo test`
- `/bin/bash -lc "PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17"`

Observed unchanged unresolved PostgreSQL symbols include:

- `CurrentMemoryContext`
- `PG_exception_stack`
- `error_context_stack`
- `CopyErrorData`
- `errstart`

## Outcome

This slice moves the branch closer to the task-15 landing contract by making
the first-class format story visible and testable:

1. README now explains how to choose a format and how to migrate between them
2. pg coverage now explicitly proves the `turboquant` reloption path
3. the main `pq_fastscan` build coverage no longer uses the old experimental
   feasibility naming

What this slice intentionally does **not** do:

- change any AM runtime behavior
- solve empty-index `PqFastScan` insert
- finish the broader `grouped_v2` naming cleanup across the wider test surface

## Next Slice

The next practical follow-ups are:

1. continue removing the remaining `grouped_v2` / `experimental_grouped_v2`
   naming from the wider pg-test surface
2. keep closing the remaining task-15 parity gaps that still block a real
   `main` landing
