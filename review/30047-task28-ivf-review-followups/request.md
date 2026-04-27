# Review Request: Task 28 IVF Review Followups

## Summary

This packet responds to reviewer feedback on packet 30046 and commit
`865dd9c`.

Resolved in code:

- `heap_f32` scans now truncate output to `rerank_width` when
  `rerank_width > 0`, so callers cannot receive a mixed exact-score head and
  approximate-score tail.
- `heap_f32` heap fetches no longer fall back to `GetLatestSnapshot()`. The
  scan uses the executor snapshot or active snapshot and errors if neither is
  available.
- The `pg_test` `psql` helper now resolves the matching pgrx `psql` binary
  when `psql` is not on `PATH`, clearing the three IVF concurrent-insert tests
  cited by the reviewer.
- The pre-existing `cargo fmt --check` drift called out by the reviewer is
  fixed.
- `plan/status.md` now records the Task 26 local scale conclusion and the Task
  28 IVF initial tuning checkpoint.

## Validation

- `cargo test --lib test_ec_ivf_heap_f32 --no-default-features --features pg18`
  - `3 passed; 0 failed`
- `cargo test --lib ec_ivf_concurrent --no-default-features --features pg18`
  - `3 passed; 0 failed`
- `cargo test --lib ec_ivf --no-default-features --features pg18`
  - `74 passed; 0 failed`
- `cargo fmt --check`
- `git diff --check`

PG17 validation was not run in this slice. Task 28 remains a PG18-default local
tuning lane per the task instructions; PG17 coverage should be handled as a
separate landing gate if maintainers require it before merge.

## Remaining Landing Work

- Split/rebase mechanics remain: land Task 26 first, then stack the IVF PR.
- Product benchmark claims remain blocked on a dedicated Graviton-class
  benchmark.
- DiskANN remains task 29 and is not part of this packet.

## Artifacts

No measurement artifacts are introduced by this packet.
