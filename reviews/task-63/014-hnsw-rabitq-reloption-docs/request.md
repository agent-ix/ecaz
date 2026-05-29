# Review Request: HNSW RaBitQ Reloption Docs

- task: `plan/tasks/63-hnsw-rabitq-storage-format.md`
- branch: `task/60-diskann-rabitq`
- packet: `reviews/task-63/014-hnsw-rabitq-reloption-docs/`

## Summary

This packet fixes the remaining local reloption/spec wording that still
described HNSW storage formats as only TurboQuant/PqFastScan after the RaBitQ
implementation landed.

## Touched Files

- `src/am/ec_hnsw/options.rs`
  - updates the PostgreSQL reloption help string for `storage_format` to list
    `rabitq`.
- `spec/tests.md`
  - updates the HNSW `storage_format` option permutation row to require
    `turboquant`, `pq_fastscan`, and `rabitq`.

## Validation

Packet-local logs:

- `artifacts/cargo-test-storage-format-reloption-no-run.log`
  - `cargo test -q --lib storage_format_reloption --no-run`
  - passed compile/no-run validation.
- `artifacts/cargo-test-storage-format-reloption.log`
  - `cargo test -q --lib storage_format_reloption`
  - blocked locally before the test body by the known pgrx-linked runtime
    symbol issue: `undefined symbol: LockBuffer`.

No benchmarks were run.
