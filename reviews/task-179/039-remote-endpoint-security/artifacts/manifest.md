# Artifact manifest

- Head SHA: `1c1490107d87772456fcdf16269cc004d432bef7`
- Implementation commit: `1c1490107` (`fix(distann): close remote endpoint privilege class`)
- Task bucket / packet: `reviews/task-179/039-remote-endpoint-security`
- Lane: local PG18, focused lint and installed-extension pgrx validation
- Fixture: pgrx ephemeral PG18 instance plus one loopback backend for isolation
- Storage format / rerank mode: not applicable
- Timestamp: `2026-07-12T16:57:18-07:00`

All cited commands ran from the clean implementation head above. This is a
security/correctness regression packet, not a measurement packet; no corpus or
shared/isolated benchmark storage surface was used.

## Validation commands and results

### PG18 clippy

```text
cargo clippy --lib --no-default-features --features pg18 -- -D warnings
```

Result: exit 0. See `clippy-pg18.log`.

### Installed remote-endpoint ACL class

```text
cargo pgrx test pg18 test_distann_remote_endpoint_acl_class
```

Result: exit 0; 1 passed, 0 failed. The test queries installed `pg_proc`
metadata for every overload named by the remote expansion/materialization/write
class. It requires exactly eight current overloads, no EXECUTE privilege for a
fresh unprivileged role, SECURITY DEFINER on every overload, and the exact
`pg_catalog`, extension schema, `pg_temp` search path. See
`remote-endpoint-acl-pg18.log`.

### Remote write isolation and success

```text
cargo pgrx test pg18 apply_record_writes
```

Result: exit 0; 2 passed, 0 failed. The stronger-isolation case begins a real
loopback `REPEATABLE READ` transaction and calls the endpoint with invalid OID
0; `EC_TRANSACTION_ISOLATION` must therefore occur before relation access. The
existing tombstone test proves the ordinary READ COMMITTED success path still
applies two writes and preserves epoch mismatch handling. See
`apply-record-writes-pg18.log`.

The pgrx “running for over 60 seconds” lines include extension build,
installation, and SQL generation. They are not endpoint execution durations.

## Artifact index

- `clippy-pg18.log`: exact-SHA warnings-denied lint.
- `remote-endpoint-acl-pg18.log`: installed function privilege/class audit.
- `apply-record-writes-pg18.log`: isolation rejection plus normal write success.

No PostgreSQL server or operational polling log is committed.
