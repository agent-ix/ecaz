# Review Request: Task 28 IVF PG18 ReadStream Posting Reads

Status: open
Owner: coder2
Date: 2026-04-25
Branch: `task28-ivf`
Code checkpoint: `e1a3e645dde31a666e4707a691eafe04d840ab6a`

## Scope

- Route PG18 IVF posting-list range reads through
  `read_stream_begin_relation` / `read_stream_next_buffer`.
- Reuse the shared sequential `LinearPrefetchState` and
  `linear_prefetch_cb` callback already used by the HNSW linear scan surface.
- Share the posting-tuple decode logic between ReadStream buffers and the
  non-PG18 `ReadBufferExtended` fallback.
- Mark Phase 7 PG18 hooks complete in `plan/tasks/28-ivf-access-method.md`.

## Files

- `src/am/ec_ivf/page.rs`
- `plan/tasks/28-ivf-access-method.md`

## Validation

- `cargo check --no-default-features --features pg18 --tests`
- `cargo pgrx test pg18 test_pg18_ec_ivf_concurrent_same_list_inserts_remain_reachable`
- `cargo pgrx test pg18 test_ec_ivf_vacuum_bulkdelete_removes_dead_heap_tid`
- `git diff --check`

No PG17 tests were run.

## Review Focus

- Whether posting-list range reads are the correct first IVF ReadStream surface
  because scan, duplicate-check, and vacuum paths all flow through it.
- Whether using `READ_STREAM_SEQUENTIAL` is appropriate for the directory
  head/tail block range scan.
- Whether the shared decode helper correctly handles buffer lock/release
  ownership for both ReadStream and fallback readers.

## Non-Goals

- New prefetch instrumentation counters.
- Direct ReadStream coverage for centroid pages.
- Performance measurement claims.
