# Artifact manifest

- Head SHA: `7c3bc1485a28d9553a7775450aa0ae3c62af44ae`
- Implementation commit: `7c3bc1485` (`fix(distann): close endpoint privilege set`)
- Task bucket / packet: `reviews/task-179/041-endpoint-privilege-set`
- Lane: local PG18, installed-extension security and production-package checks
- Fixture: pgrx ephemeral PG18 plus a real loopback backend and fresh NOLOGIN
  role for function-level ACL calls
- Storage format / rerank mode: not applicable
- Timestamp: `2026-07-12T18:26:53-07:00`

All cited commands run from the clean implementation head above. This is a
security/correctness packet, not benchmark evidence; no corpus or shared/
isolated benchmark storage surface is used.

## Validation commands and results

### Release PG18 clippy

```text
cargo clippy --lib --no-default-features --features pg18 -- -D warnings
```

Result: exit 0. See `clippy-pg18.log`.

### Installed class-wide ACL calls

```text
cargo pgrx test pg18 test_distann_remote_endpoint_acl_class
```

Result: exit 0; 1 passed, 0 failed. The installed test enumerates the
extension-owned protected set from `pg_proc`/`pg_depend`, requires at least 40
overloads and supported non-NULL fixtures for every input type, grants schema
usage, assumes a fresh role, and requires function-level SQLSTATE `42501` from
every real call. See `endpoint-acl-calls-pg18.log`.

### Fold maintenance isolation

```text
cargo pgrx test pg18 test_ec_distann_fold_delta_requires_read_committed
```

Result: exit 0; 1 passed, 0 failed. A real loopback Repeatable Read transaction
calls the fold endpoint with invalid OID 0 and receives
`EC_TRANSACTION_ISOLATION`, proving the guard precedes relation access. See
`fold-isolation-pg18.log`.

### Production package surface

```text
cargo pgrx package \
  -c /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config \
  --no-default-features -F pg18 \
  --out-dir target/task179-packet041-release-package
```

The packaged SQL was then audited with:

```text
if rg -n 'ec_distann_debug_(tombstone|set_in_flight|expand_search)' \
  target/task179-packet041-release-package/home/peter/.pgrx/18.3/pgrx-install/share/postgresql/extension/ecaz--0.1.1.sql
then
  exit 1
fi
```

Result: both commands exit 0. The package contains 295 SQL entities under
production `pg18` features, zero occurrences of the three debug endpoints, and
does contain `ec_distann_list_directory`, `ec_distann_epoch_fingerprint`,
`ec_distann_fold_delta_into_graph`, and the class-wide privilege finalizer. See
`release-package-pg18.log` and `release-debug-endpoint-audit.log`.

The pgrx “running for over 60 seconds” lines include extension compilation,
installation, and SQL generation. They are not endpoint execution durations.

## Artifact index

- `clippy-pg18.log`: production-feature warnings-denied lint.
- `endpoint-acl-calls-pg18.log`: installed dependency-derived function class,
  real SET ROLE calls, and function-level permission denials.
- `fold-isolation-pg18.log`: real Repeatable Read rejection before relation
  access.
- `release-package-pg18.log`: production PG18 extension package generation.
- `release-debug-endpoint-audit.log`: packaged-SQL zero-occurrence check for
  all three debug endpoint names.

No packaged `.so`, PostgreSQL server log, corpus, or operational polling output
is committed.
