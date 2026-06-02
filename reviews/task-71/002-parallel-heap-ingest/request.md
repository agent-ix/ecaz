# Task 71 Review Request: Parallel Heap Ingest

## Scope

This packet reviews the first implementation slice for IVF parallel build:
parallel heap tuple ingestion. The code commit under review is
`012ece576 Add IVF parallel heap build ingestion`.

The slice:

- Enables `amcanbuildparallel` for `ec_ivf` while keeping parallel scan support
  (`amcanparallel`) disabled.
- Adds a PostgreSQL `ParallelContext`/shared table scan/shm_mq worker path for
  `ec_ivf_ambuild`.
- Has workers encode heap TID, quantized payload, gamma, dimensions, and source
  vector bits, then send those tuples to the leader.
- Sorts worker tuples by heap TID on the leader before feeding the existing
  training, assignment, staging, and page flush path.
- Falls back to the existing serial build path when PostgreSQL requests no
  workers or launches none.
- Adds pg_test-only timing/count introspection for validating that the parallel
  worker path ran and preserved tuple counts.

This does not yet claim the full Task 71 measurement/closeout work. The next
slice still needs benchmark-suite evidence across worker counts and a final
task audit.

## Validation

Packet-local artifacts are under
`reviews/task-71/002-parallel-heap-ingest/artifacts/`.

- `cargo check --no-default-features --features pg18`
  - `Finished dev profile [unoptimized + debuginfo] target(s) in 51.31s`
- `cargo test --no-default-features --features pg18 am::ec_ivf::build_parallel`
  - `test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 1936 filtered out`
- `cargo pgrx test pg18 test_ec_ivf_parallel_build_workers_and_counts`
  - Initial sandboxed run failed at extension install with `Operation not permitted`.
  - Escalated rerun passed:
    `test tests::pg_test_ec_ivf_parallel_build_workers_and_counts ... ok`
    and `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1939 filtered out`

## Review Focus

- PostgreSQL parallel build resource handling: DSM sizing, queue lifecycle,
  snapshot handling, worker launch/finish cleanup, WAL/buffer accounting.
- Worker tuple wire format and decode error handling.
- Deterministic merge ordering by heap TID before reusing the existing IVF
  training/flush path.
- Whether the pg_test-only timing/count hook is scoped tightly enough for this
  validation need.
