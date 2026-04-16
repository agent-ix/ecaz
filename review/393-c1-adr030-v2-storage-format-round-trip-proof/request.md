# Review Request: C1 ADR-030 V2 Storage Format Round-Trip Proof

## Context

The branch already had many focused insert and vacuum checks:

- generic TurboQuant insert/vacuum lifecycle tests
- explicit `PqFastScan` insert checkpoints
- explicit `PqFastScan` vacuum checkpoints
- explicit `storage_format = 'turboquant'` build coverage
- explicit `storage_format = 'pq_fastscan'` build coverage

But task 15 wants something more direct at the landing boundary:

- an explicit insert + vacuum round-trip on **both** storage formats selected
  through the reloption

## Problem

Without this slice, the branch could prove:

1. TurboQuant works through the default path
2. PqFastScan works through format-specific insert and vacuum slices
3. both formats build through the reloption

But it still did not have one concise proof surface that said:

- `storage_format='turboquant'` survives build, live insert, ordered scan,
  delete, vacuum, and ordered scan again
- `storage_format='pq_fastscan'` survives that same round-trip

That was a proof gap, not a missing runtime implementation gap.

## Planned Slice

One pg-test checkpoint:

1. add tiny shared runtime-fixture/query helpers
2. add one explicit reloption round-trip test for TurboQuant
3. add one explicit reloption round-trip test for PqFastScan

No AM behavior change.

## Implementation

Updated:

- `src/lib.rs`

### 1. Added small shared helpers for round-trip proof

Added:

- `create_turboquant_runtime_fixture(...)`
- `runtime_fixture_embedding(id)`
- `observed_heap_tids_for_query(...)`
- `observed_ids_for_query(...)`

These are test-only helpers that mirror the existing `PqFastScan` fixture style
and let the round-trip tests stay short and readable.

### 2. Added explicit TurboQuant reloption round-trip proof

Added:

- `test_tqhnsw_turboquant_reloption_round_trip`

The test:

1. builds a runtime fixture with `storage_format = 'turboquant'`
2. verifies a fixture row ranks first for its own embedding
3. live-inserts row `17`
4. verifies ordered scan ranks the inserted row first for its own embedding
5. deletes row `1`
6. runs `debug_vacuum_remove_heap_tids(...)`
7. verifies ordered scan no longer emits the deleted row

That gives TurboQuant an explicit reloption-driven build/insert/vacuum/scan
round-trip check instead of relying on the default-format assumption.

### 3. Added explicit PqFastScan reloption round-trip proof

Added:

- `test_tqhnsw_pq_fastscan_reloption_round_trip`

The test follows the same structure, but uses the grouped source-backed runtime
fixture:

1. builds with `storage_format = 'pq_fastscan'`
2. verifies a fixture row ranks first for its own embedding
3. live-inserts row `17` with a matching `source real[]`
4. verifies ordered scan ranks the inserted row first
5. deletes row `1`
6. runs grouped vacuum repair/finalization
7. verifies ordered scan no longer emits the deleted row

That pulls the existing grouped insert/vacuum work into one explicit
first-class-format proof surface.

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

This slice strengthens task-15 landing proof without changing runtime code:

1. TurboQuant now has explicit reloption-driven insert/vacuum round-trip proof
2. PqFastScan now has matching reloption-driven round-trip proof
3. both first-class formats are now covered by one concise end-to-end lifecycle
   surface

What this slice intentionally does **not** do:

- add new AM logic
- replace the real-corpus harness requirement in task 15
- remove the remaining debug/runtime naming debt elsewhere in the tree

## Next Slice

The remaining work is now mostly landing polish:

1. continue removing old ADR030/grouped naming from the remaining debug/test
   surface where it still leaks
2. decide which runtime tuning/debug knobs should remain env-driven at merge
3. run or tighten the remaining task-15 proof surfaces beyond pg tests
