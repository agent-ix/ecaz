# Review Request: Task 28 IVF Review Followups

## Summary

This packet responds to reviewer feedback on packet 30046 and commits
`865dd9c` and `215a788`.

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
- The PG18-only IVF EXPLAIN counter export is now feature-gated so PG17 clippy
  does not see it as an unused import.

## Validation

- `cargo test --lib test_ec_ivf_heap_f32 --no-default-features --features pg18`
  - `3 passed; 0 failed`
- `cargo test --lib ec_ivf_concurrent --no-default-features --features pg18`
  - `3 passed; 0 failed`
- `cargo test --lib ec_ivf --no-default-features --features pg18`
  - `74 passed; 0 failed`
- `cargo fmt --check`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
- `cargo pgrx test pg17`
  - library tests: `661 passed; 0 failed; 4 ignored`
  - integration/bin/property/size/doc tests all passed
- `git diff --check`

## Remaining Landing Work

- Split/rebase mechanics remain: land Task 26 first, then stack the IVF PR.
- Product benchmark claims remain blocked on a dedicated Graviton-class
  benchmark.
- DiskANN remains task 29 and is not part of this packet.

## Artifacts

No measurement artifacts are introduced by this packet.
