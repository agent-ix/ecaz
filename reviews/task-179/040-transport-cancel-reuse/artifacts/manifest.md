# Artifact manifest

- Head SHA: `f5720977eb1a3540ee082838b08e2df725da1db1`
- Implementation commit: `f5720977e` (`fix(distann): preserve transport state across cancel`)
- Task bucket / packet: `reviews/task-179/040-transport-cancel-reuse`
- Lane: local PG18, focused static and live pgrx validation
- Fixture: pgrx loopback pooled remote session plus a second loopback backend
  issuing `pg_cancel_backend`
- Storage format: not applicable; this packet changes transport interrupt
  boundaries only
- Rerank mode: not applicable
- Timestamp: `2026-07-12T17:25:22-07:00`

All commands ran from the clean implementation head named above. This is a
correctness/regression packet, not benchmark evidence; no corpus, shared-table,
or isolated one-index-per-table measurement was used.

## Validation commands and results

### PG18 clippy

```text
cargo clippy --lib --no-default-features --features pg18 -- -D warnings
```

Result: exit 0. See `clippy-pg18.log`.

### Live cancellation and timeout regressions

```text
cargo pgrx test pg18 remote_transport_
```

Result: exit 0; 2 passed, 0 failed. This focused filter covers:

- `test_ec_distann_remote_transport_cancel_then_reuse`; and
- `test_ec_distann_remote_transport_statement_timeout`.

See `remote-transport-live-pg18.log`.

The cancellation case establishes the pooled session before a second backend
cancels the test backend during a 500 ms remote sleep. PostgreSQL reports
`canceling statement due to user request`; after the internal subtransaction
rolls back, an immediate zero-duration probe succeeds through the same
thread-local transport state. The companion case retains packet 036's 10 ms
remote statement-timeout assertion.

## Artifact index

- `clippy-pg18.log`: warnings-denied PG18 library lint.
- `remote-transport-live-pg18.log`: live cancellation/reuse and remote
  statement-timeout regressions.

No PostgreSQL server log or operational polling output is committed.
