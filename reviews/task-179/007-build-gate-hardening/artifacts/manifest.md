# Packet 007 — build-gate hardening artifact manifest

Task bucket: `reviews/task-179/`
Packet path: `reviews/task-179/007-build-gate-hardening/`
Surface: durable coordinator build gate (`src/am/ec_distann/build_gate.rs`,
`build_coordinator.rs`), enforced via installed PostgreSQL hooks. Isolated
one-index-per-fixture pgrx test surfaces (no shared benchmark tables).

## Commits under review

- `965fa1dfd0d836aadfe66ec5fc48faadf59028c4` — feat(distann): fail closed on
  build gate preload, inheritance, and global utilities.
- `0ee8d49b309dd1e73d0f2ee49316e88b878bdf2c` — fix(distann): enforce build gate
  at ExecutorStart to close cached-plan bypass.

Both are on `task-179-ec-distann-physical-shards`, rebased onto outside-review
commit `7296ca106`.

## Artifacts

| File | Head SHA | Produced | Command | Key result |
|---|---|---|---|---|
| `pgrx-begin-build-gate.log` | `0ee8d49b3` | 2026-07-11 | `cargo pgrx test pg18 --no-default-features --features pg18 test_distann_begin_build` | `test result: ok. 3 passed; 0 failed` — competing-backend (incl. cached-plan P1 regression + positive control), inherited-source rejection, and lock-lifecycle |
| `pgrx-preloaded-hook-passthrough.log` | `965fa1dfd` | 2026-07-11 | `cargo pgrx test pg18 --no-default-features --features pg18 test_distann_preloaded_hook_passes_through_without_extension` | `test result: ok. 1 passed; 0 failed` — ordinary DML passes in a shared-preloaded database without CREATE EXTENSION |
| `cargo-clippy-pg18-gate-hardening.log` | `0ee8d49b3` | 2026-07-11 | `cargo clippy --no-default-features --features 'pg18 pg_test' --lib --tests -- -D warnings` | `Finished` exit 0, warnings denied |

Additional validation run but not separately logged here (identical source):
`cargo check --no-default-features --features 'pg18 pg_test' --tests` passes at
`0ee8d49b3`.

## Notes on live-test debugging (evidence hygiene)

The preloaded-passthrough test was iterated three times before it was correct;
the final design is deadlock-free and rerun-safe:

- `ALTER INDEX ... SET SCHEMA` is not valid PostgreSQL (indexes follow their
  table) — removed from the gate rejection matrix.
- `DROP DATABASE` — even without `FORCE` — emits a global
  `PROCSIGNAL_BARRIER_SMGRRELEASE`. The test-function backend blocks
  synchronously in libpq while driving the drop from a second connection, so it
  can never absorb that barrier and the drop deadlocks. The test now never
  drops a database: `CREATE DATABASE` (WAL_LOG strategy, no barrier) runs once,
  the probe is idempotent, and the ephemeral pgrx instance discards the
  database.
