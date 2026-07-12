# Artifact manifest

- Head SHA: `ceb15f73ac69fcd98896457c9578fadae2ff0c09`
- Implementation commit: `ceb15f73a` (`fix(distann): bound remote transport waits`)
- Task bucket / packet: `reviews/task-179/036-bounded-remote-transport`
- Lane: local PG18, focused static/unit/pgrx validation
- Fixture: pgrx loopback pooled remote session and the existing three-owner
  physical-handoff integration fixture
- Storage format: not applicable; this packet changes transport waiting only
- Rerank mode: not applicable
- Timestamp: `2026-07-12T13:17:48-07:00`

All commands ran from the clean implementation head named above. This is a
correctness/regression packet, not benchmark evidence; no corpus, shared-table,
or isolated one-index-per-table measurement was used.

## Validation commands and results

### PG18 clippy

```text
cargo clippy --lib --no-default-features --features pg18 -- -D warnings
```

Result: exit 0. See `clippy-pg18.log`.

### Focused transport unit tests

```text
cargo test --lib --no-default-features --features pg18 \
  'am::ec_distann::remote_transport::tests::' -- --nocapture
```

Result: exit 0; 7 passed, 0 failed. The cases cover the client deadline,
preservation of a ready remote error, nonzero configured connect timeout,
conninfo redaction, interleaved multi-owner reassembly, missing coverage, and
wrong response counts. See `remote-transport-unit-pg18.log`.

### Live pooled-session statement timeout

```text
cargo pgrx test pg18 test_ec_distann_remote_transport_statement_timeout
```

Result: exit 0; the focused PG18 test passed. It first establishes the pooled
session with a 10-second budget, changes the same backend's Userset budget to
10 milliseconds, invokes `pg_sleep(1)`, and requires timeout completion within
two seconds. See `remote-statement-timeout-pg18.log`.

The pgrx harness's “running for over 60 seconds” line includes extension build,
installation, and SQL generation time. It is not the remote sleep duration;
the in-test elapsed-time assertion passed.

### Physical-handoff regression

```text
cargo pgrx test pg18 test_distann_three_owner_physical_handoff
```

Result: exit 0; the focused PG18 test passed. This exercises the existing
three-owner begin/stage/seal/publish/retire/abort behavior through the newly
bounded transport wrappers. See `physical-handoff-regression-pg18.log`.

## Artifact index

- `clippy-pg18.log`: warnings-denied PG18 library lint.
- `remote-transport-unit-pg18.log`: seven focused unit tests.
- `remote-statement-timeout-pg18.log`: live pooled-session timeout refresh.
- `physical-handoff-regression-pg18.log`: three-owner lifecycle regression.

No PostgreSQL server log or operational polling output is committed.
