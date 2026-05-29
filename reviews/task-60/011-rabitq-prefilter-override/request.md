# Review Request: RaBitQ Prefilter Override Guard

## Scope

This checkpoint tightens the DiskANN RaBitQ scan prefilter selection.

`ec_diskann.prefilter_kind = 'binary_sidecar'` now fails explicitly for RaBitQ
indexes because RaBitQ stores its search code directly and does not persist the
legacy binary sidecar. `auto` continues to use the RaBitQ estimator, and
`grouped_pq` remains rejected for RaBitQ indexes.

## Validation

Artifacts are under
`reviews/task-60/011-rabitq-prefilter-override/artifacts/`.

- `cargo-check-pg18.log`: `cargo check --no-default-features --features pg18`
  passed.
- `cargo-test-rabitq-prefilter-override.log`: the focused unit test compiled,
  then the local test binary failed before executing due to the existing local
  PostgreSQL symbol loader issue: `undefined symbol: BufferBlocks`.

## Remaining Task 60 Gate

The external benchmark host still needs to run the full 100k/1M Task 60 suite
and record recall, latency, storage, and the 1M shipping decision.
