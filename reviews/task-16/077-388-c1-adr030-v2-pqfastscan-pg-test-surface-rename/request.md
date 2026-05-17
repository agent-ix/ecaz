# Review Request: C1 ADR-030 V2 PqFastScan Pg-Test Surface Rename

## Context

Packet 386 renamed the shared `src/lib.rs` runtime-test helper surface from the
old feasibility names to `pq_fastscan_*`, but it intentionally left the wider
pg-test surface alone to keep that slice narrow.

After packet 387, the branch still had a large block of pg-test function names,
SQL fixture table names, and index names using:

- `grouped_v2`
- `experimental_grouped_v2`

That meant the helper layer was on the ADR-032 product name while much of the
visible pg-test surface was still on the feasibility-era name.

## Problem

This was naming drift, not a runtime bug:

1. production/runtime code now says `PqFastScan`
2. shared test helpers now say `pq_fastscan`
3. many pg-test entry points and SQL fixtures still said `grouped_v2`

That inconsistency makes the test surface harder to navigate and keeps old
feasibility naming alive longer than necessary.

## Planned Slice

One broad-but-mechanical cleanup checkpoint:

1. rename the remaining `grouped_v2` / `experimental_grouped_v2` pg-test
   function names in `src/lib.rs`
2. rename the matching SQL fixture table/index names and open-index labels in
   those tests
3. remove the final old-name reloption test input in `src/am/options.rs`

No runtime behavior change.

## Implementation

Updated:

- `src/lib.rs`
- `src/am/options.rs`

### 1. Renamed the remaining wide pg-test surface to `pq_fastscan`

This sweep replaced the remaining `grouped_v2` / `experimental_grouped_v2`
test-facing names with `pq_fastscan` across:

- pg-test function names
- SQL fixture table names
- SQL fixture index names
- `open_valid_tqhnsw_index(...)` caller labels

The affected area is the large `PqFastScan` runtime / scan diagnostic block in
`src/lib.rs`.

### 2. Fixed PostgreSQL identifier-length fallout from the rename

This sweep surfaced one real compile-time constraint:

- a few `#[pg_test]` function names became 64 characters long after the
  `pq_fastscan` rename

PostgreSQL truncates identifiers at 63 bytes, and pgrx rejects test names that
cross that boundary. Shortened the four affected test names to stay under the
limit while keeping the `pq_fastscan` naming visible.

### 3. Removed the last old-name reloption test input

In `src/am/options.rs`, changed the negative reloption parse test input from:

- `grouped-v2`

to:

- `legacy_format`

so the test no longer keeps the old public name alive.

## Measurements

No benchmark or recall rerun in this slice.

## Validation

Passed:

- `cargo check --tests`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Required full-test commands still fail on this workstation at the same known
PostgreSQL linker layer as prior checkpoints:

- `cargo test`
- `/bin/bash -lc "PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17"`

Observed unresolved PostgreSQL symbols remain in the same family, including:

- `CurrentMemoryContext`
- `PG_exception_stack`
- `error_context_stack`
- `CopyErrorData`
- `errstart`

## Outcome

This checkpoint reduces the remaining naming mismatch between the landed
product/runtime terminology and the pg-test surface:

1. the wider pg-test block now says `pq_fastscan`
2. SQL fixtures and debug/open labels now match that terminology
3. the last old-name reloption test input is gone

What this slice intentionally does **not** do:

- change any production AM behavior
- solve empty-index `PqFastScan` insert
- finish every remaining naming cleanup outside the touched test surface

## Next Slice

The next practical work remains functional rather than naming-oriented:

1. close the remaining task-15 parity gaps that still block a real `main`
   landing
2. decide whether the empty-index `PqFastScan` insert path needs a dedicated
   design slice before merge
