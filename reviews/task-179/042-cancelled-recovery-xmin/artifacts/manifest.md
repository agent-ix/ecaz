# Artifact manifest

- Head SHA: `13905ef146a9fc7bf20d3ae49a90041d66e34bbf`
- Implementation commit: `13905ef14` (`fix(distann): require committed cancellation recovery`)
- Task bucket / packet: `reviews/task-179/042-cancelled-recovery-xmin`
- Lane: local PG18, focused lint and real multi-transaction lifecycle validation
- Fixture: pgrx ephemeral PG18 plus a loopback `postgres::Client` using explicit
  BEGIN/ROLLBACK and autocommit transactions
- Storage format / rerank mode: not applicable
- Timestamp: `2026-07-12T18:44:29-07:00`

All cited commands run from the clean implementation head above. This is a
correctness packet, not benchmark evidence; no corpus or shared/isolated
benchmark storage surface is used.

## Validation commands and results

### PG18 clippy

```text
cargo clippy --lib --no-default-features --features pg18 -- -D warnings
```

Result: exit 0. See `clippy-pg18.log`.

### Multi-epoch cancellation/recovery lifecycle

```text
cargo pgrx test pg18 test_distann_multi_epoch_publish
```

Result: exit 0; 1 passed, 0 failed. The test rejects cleanup recovery from the
same explicit transaction that wrote cancellation, rolls that transaction
back, then proves committed cancellation, audited reclaim, exact replay, and a
subsequent epoch build all succeed. See `multi-epoch-cancel-recovery-pg18.log`.

The pgrx “running for over 60 seconds” line includes extension compilation,
installation, and SQL generation. It is not lifecycle endpoint execution time.

## Artifact index

- `clippy-pg18.log`: warnings-denied PG18 library lint.
- `multi-epoch-cancel-recovery-pg18.log`: same-transaction rejection followed
  by committed cancellation, audited reclaim, replay, and subsequent build.

No PostgreSQL server log or operational polling output is committed.
