# Artifact manifest

- Head SHA: `a4d374c2f294dc209b1b0f499bd527e52b375b06`
- Implementation commit: `a4d374c2f294dc209b1b0f499bd527e52b375b06`
- Task bucket / packet: `reviews/task-179/058-inactive-gate-fast-path`
- Branch: `task-179-ec-distann-physical-shards`
- Lane: local focused PG18 durable build-gate cache correctness
- PostgreSQL: pgrx PG18 18.3, with `ecaz` installed and preloaded by pg_test
- Run: `2026-07-13T06:48:09-07:00` through
  `2026-07-13T06:54:39-07:00`
- Fixture: isolated distributed-control source/index with independent
  pre-gate cache, owner, contender, and post-gate backends
- Storage format: durable coordinator registration and control metadata
- Rerank mode: not applicable
- Isolation surface: one-index-per-source test fixture; no shared-table surface

This is correctness evidence, not the performance measurement. Packet 057
owns the immutable A/B suite and results.

## Commands

```text
cargo pgrx test pg18 test_distann_begin_build_competing_backend_busy \
  --no-default-features --features pg18

cargo check --no-default-features --features pg18
```

## Artifact index

- `pgrx-test-pg18.log`: complete focused build/install/live regression output,
  exit code 0.
- `cargo-check-pg18.log`: exact-SHA PG18 compile output, exit code 0.

## Key cited results

```text
test tests::pg_test_distann_begin_build_competing_backend_busy ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured;
  2507 filtered out; finished in 80.75s

Finished `dev` profile [unoptimized + debuginfo] target(s) in 14.06s
```
