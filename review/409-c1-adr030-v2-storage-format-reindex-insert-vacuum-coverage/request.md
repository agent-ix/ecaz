# Review Request: C1 ADR-030 V2 Storage-Format REINDEX Insert/Vacuum Coverage

Current head: `e8fc673`

## Context

Packet `403` added the storage-format REINDEX guardrail and proved it on the
ordered-scan path:

- if reloptions say `pq_fastscan`
- but on-disk metadata says `turboquant`
- runtime now errors with:
  - `REINDEX after switching formats`

Reviewer feedback on `403` pointed out one remaining proof gap:

- scan had explicit pg coverage
- insert and vacuum used the same `from_index_relation(...)` guardrail
- but there was no pg smoke proving those paths also reject a reloption-only
  format switch

This slice closes that test gap without changing runtime behavior.

## Problem

After `403`, the code already routed all three runtime opens through the same
guardrail seam:

1. ordered scan
2. insert
3. vacuum

But the pg coverage only exercised ordered scan. A future refactor could
accidentally bypass the seam in `aminsert` or vacuum while leaving the scan
test green.

## Planned Slice

No AM logic change. Add two explicit pg tests on top of the existing scan
coverage:

1. reloption-only format switch rejects live insert
2. reloption-only format switch rejects vacuum cleanup

Both use the same mismatch:

- build `turboquant`
- `ALTER INDEX ... SET (storage_format = 'pq_fastscan')`
- hit the runtime path without `REINDEX`

## Implementation

Updated:

- `src/lib.rs`

Concrete changes:

1. kept the existing scan mismatch test unchanged
2. added `test_tqhnsw_storage_format_switch_rejects_insert_until_reindex`
   - builds a `turboquant` runtime fixture
   - flips only the reloption to `pq_fastscan`
   - attempts a normal heap insert
   - asserts the same explicit REINDEX panic text
3. added `test_tqhnsw_storage_format_switch_rejects_vacuum_until_reindex`
   - builds a `turboquant` runtime fixture
   - deletes one row
   - flips only the reloption to `pq_fastscan`
   - calls `am::debug_vacuum_remove_heap_tids(...)`
   - asserts the same explicit REINDEX panic text

## Validation

Passed:

- `cargo check --tests`
- `cargo check --tests --no-default-features --features 'pg17 pg_test'`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Required full-test commands were run and hit the same known workstation linker
boundary as the rest of this branch:

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`

Observed unresolved PostgreSQL symbols remain in the same family:

- `CurrentMemoryContext`
- `PG_exception_stack`
- `error_context_stack`
- `CopyErrorData`
- `errstart`

## Outcome

Packet `403` is now covered across all three load-bearing runtime entry paths:

1. ordered scan rejects reloption/metadata drift
2. insert rejects reloption/metadata drift
3. vacuum rejects reloption/metadata drift

That makes the REINDEX contract materially harder to regress during the final
merge prep.

## Next Slice

Unless new outside feedback lands, the remaining work is mostly merge evidence
and final closeout, not more runtime scaffolding.
