# Review Request: C1 ADR-030 V2 PqFastScan Runtime Test Helper Names

## Context

Packets 380 and 384 pushed the core AM/runtime naming toward the ADR-032
contract:

- `TurboQuant`
- `PqFastScan`

But the shared runtime-test helper surface in `src/lib.rs` still exported the
older feasibility-era helper names:

- `create_grouped_v2_runtime_fixture*`
- `grouped_v2_runtime_query(...)`
- `grouped_v2_runtime_source(...)`
- `grouped_v2_*_runtime_observed_scores(...)`

Those helpers are heavily reused by the `PqFastScan` scan/runtime tests, so
they were still dragging the old naming into new code even after the core
runtime surface had moved on.

## Problem

This was not a behavioral bug. It was naming drift:

1. the core runtime and metadata enums now speak in `TurboQuant` /
   `PqFastScan`
2. the shared test-helper layer still advertised `grouped_v2`
3. new tests therefore had to choose between the product name and the
   feasibility helper name every time they reused the fixture/query helpers

That inconsistency makes test code harder to read and keeps feasibility jargon
alive longer than necessary.

## Planned Slice

One test-helper naming checkpoint:

1. rename the shared `src/lib.rs` runtime fixture helpers to `pq_fastscan_*`
2. update the helper call sites that consume those shared helpers
3. leave SQL table/index names and existing pg-test function names alone for
   now to keep the slice narrow

No behavior change. No production-code change.

## Implementation

Updated:

- `src/lib.rs`

### 1. Shared runtime fixtures now use `pq_fastscan_*`

Renamed helper families include:

- `create_grouped_v2_runtime_fixture_internal(...)` →
  `create_pq_fastscan_runtime_fixture_internal(...)`
- `create_grouped_v2_runtime_fixture(...)` →
  `create_pq_fastscan_runtime_fixture(...)`
- `create_grouped_v2_runtime_fixture_with_source_raw(...)` →
  `create_pq_fastscan_runtime_fixture_with_source_raw(...)`
- `create_grouped_v2_runtime_fixture_with_m(...)` →
  `create_pq_fastscan_runtime_fixture_with_m(...)`

### 2. Shared query/source helpers now match the format name

Also renamed:

- `grouped_v2_runtime_query(...)` → `pq_fastscan_runtime_query(...)`
- `grouped_v2_runtime_source(...)` → `pq_fastscan_runtime_source(...)`
- `grouped_v2_exact_traversal_runtime_observed_scores(...)` →
  `pq_fastscan_exact_traversal_runtime_observed_scores(...)`
- `grouped_v2_heap_rerank_runtime_observed_scores(...)` →
  `pq_fastscan_heap_rerank_runtime_observed_scores(...)`
- `assert_grouped_v2_runtime_live_window_matches_windowed_simulation(...)` →
  `assert_pq_fastscan_runtime_live_window_matches_windowed_simulation(...)`

### 3. Call sites were updated, but wider test names were left alone

This packet intentionally does **not** rename:

- the many existing pg-test function names in `src/lib.rs`
- SQL table/index names that already contain `grouped_v2`

Those are broader churn and can be handled in a separate sweep if we still want
them gone. This slice only normalizes the shared helper layer.

## Measurements

No benchmark or recall work in this slice. Naming-only cleanup.

## Validation

Passed:

- `cargo check --tests`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Required full-test commands still fail on this workstation at the same known
PostgreSQL linker layer as prior checkpoints:

- `cargo test`
- `/bin/bash -lc "PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17"`

Observed failure mode is unchanged:

- unresolved PostgreSQL symbols during link, including
  `CurrentMemoryContext`, `PG_exception_stack`, `error_context_stack`,
  `CopyErrorData`, and `errstart`

## Outcome

This checkpoint makes the shared runtime-test helper surface match the product
name:

1. shared `src/lib.rs` helper APIs now say `pq_fastscan`
2. helper call sites no longer have to mix product naming with feasibility
   helper names
3. no behavior changed

What it intentionally does **not** do:

- rename the wider pg-test surface yet
- change SQL fixture names
- touch production runtime behavior

## Next Slice

The next practical cleanup / landing slices are:

1. continue reducing remaining `grouped_v2` / experimental naming in the wider
   test surface if that churn is still worth it
2. keep closing the remaining ADR-032 / task-15 parity gaps outside naming
3. decide whether the wider pg-test rename is actually worth the noise relative
   to the remaining functional work
