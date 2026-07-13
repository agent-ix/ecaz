# Artifact manifest

- Head SHA: `4aa8817ce6bef68de54e8039972d2e10d0815b6a`
- Implementation commit: `4aa8817ce6bef68de54e8039972d2e10d0815b6a`
- Task bucket / packet: `reviews/task-179/055-build-gate-correctness`
- Branch: `task-179-ec-distann-physical-shards`
- Lane: local focused PG18 durable build-gate correctness
- PostgreSQL: pgrx PG18 installation at
  `/home/peter/.pgrx/18.3/pgrx-install`, with `ecaz` installed and preloaded by
  the pg_test harness
- Run: `2026-07-13T05:56:24-07:00` through
  `2026-07-13T06:02:56-07:00`
- Fixture: one isolated distributed-control source/index and local helper
  tables, plus independent loopback owner/contender/gate backends
- Storage format: distributed-control metadata and durable build registration;
  no corpus or benchmark storage surface
- Rerank mode: not applicable
- Isolation surface: one-index-per-source test fixture; no shared-table surface

This is correctness evidence, not a benchmark or measurement packet. It uses
no corpus/query data and makes no latency, recall, storage, or promotion claim.

## Commands

PG18 compile validation:

```text
cargo check --no-default-features --features pg18
```

Focused live PG18 callback/lifecycle regression:

```text
cargo pgrx test pg18 test_distann_begin_build_competing_backend_busy \
  --no-default-features --features pg18
```

## Artifact index

- `cargo-check-pg18.log`: exact-SHA PG18 compile output, exit code 0.
- `pgrx-test-pg18.log`: complete focused pg_test build, install, and execution
  output, exit code 0.

## Key cited results

```text
Finished `dev` profile [unoptimized + debuginfo] target(s) in 14.77s

test tests::pg_test_distann_begin_build_competing_backend_busy ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured;
  2507 filtered out; finished in 84.99s
```
