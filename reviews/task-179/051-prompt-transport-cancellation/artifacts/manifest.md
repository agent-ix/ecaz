# Artifact manifest

- Head SHA: `a94e5e9be83b523a907ca3590dc62cafeca3cb3a`
- Implementation commit: `a94e5e9be` (`fix(distann): observe remote transport cancel promptly`)
- Task bucket / packet: `reviews/task-179/051-prompt-transport-cancellation`
- Branch: `task-179-ec-distann-physical-shards`
- Lane: local PG18 focused static, unit, and live pgrx validation
- Fixture: pgrx loopback PostgreSQL plus a second backend issuing
  `pg_cancel_backend`; a loopback blackhole listener for the mid-connect case
- Storage format: not applicable; this packet changes transport cancellation
  observation only
- Rerank mode: not applicable
- Isolation surface: no corpus, benchmark table, or shared-table measurement
- Timestamp: `2026-07-13T02:53:20-07:00`

All commands ran from the exact implementation head above. This is a
correctness/regression packet, not benchmark evidence.

## Validation commands and results

### PG18 clippy

```text
cargo clippy --lib --no-default-features --features pg18 -- -D warnings
```

Result: exit 0. See `clippy-pg18.log`.

### Focused transport unit tests

```text
cargo test --lib --no-default-features --features pg18 remote_transport::tests::
```

Result: exit 0; 8 passed, 0 failed. This covers client deadline and remote
error preservation, connection timeout configuration, redaction, concurrent
owner futures, response validation, and reassembly. See
`unit-remote-transport.log`.

### Live PG18 cancellation and timeout tests

```text
cargo pgrx test pg18 remote_transport_
```

Result: exit 0; 3 passed, 0 failed. See `live-pg18.log`.

The cases are:

- `test_ec_distann_remote_transport_statement_timeout` — retains packet 036's
  live bounded remote-timeout proof;
- `test_ec_distann_remote_transport_cancel_then_reuse` — establishes a pooled
  session, starts a 5-second remote sleep with a 10-second remote timeout, has a
  second backend cancel after 100 ms, asserts completion in under one second,
  and immediately reconnects/reuses transport state in the same backend; and
- `test_ec_distann_remote_transport_cancel_mid_connect_then_reuse` — connects
  to a loopback listener that accepts but never performs a PostgreSQL
  handshake, cancels after 100 ms under a 10-second connect timeout, asserts
  completion in under one second, and then successfully connects to the real
  loopback PostgreSQL instance from the same backend.

## Artifact index

- `clippy-pg18.log`: exact-SHA warnings-denied PG18 library lint.
- `unit-remote-transport.log`: exact-SHA focused unit suite.
- `live-pg18.log`: exact-SHA live timeout, mid-await cancel, mid-connect
  cancel, and post-cancel reuse regressions.

The exploratory precommit run is not part of the review packet. PostgreSQL
server logs and operational output are not committed.
