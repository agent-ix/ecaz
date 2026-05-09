# 30646 — SPIRE Remote Search Executor Connection Check

## Summary

This packet starts the actual remote-search executor path after the 30645
review recommendation to stop adding pre-I/O plan steps.

Code commit:

- `09e1eba3e23f133cee2103183277684cb4665026` — Add SPIRE remote search executor connection check

## What Changed

- Added `ec_spire_remote_search_libpq_executor_connection_check(...)`.
- The function consumes the existing descriptor -> secret -> connection-open
  plan chain.
- For rows ready to open a connection, it resolves the executor-owned
  conninfo secret internally, attempts a PostgreSQL client connection, and
  immediately drops the client.
- It reports only sanitized executor status:
  - no raw conninfo
  - no raw connection error text
  - explicit `connection_attempted`
  - explicit `connection_status`
  - next executor step/status/recommendation
- Added a direct `postgres = "0.19"` dependency. The crate was already present
  in `Cargo.lock`; this makes it a direct extension dependency for the first
  executor-owned client path.

## Validation

Focused PG18 validation passed:

```text
cargo pgrx test pg18 test_ec_spire_remote_node_descriptor_catalog_active
```

Key result:

```text
test tests::pg_test_ec_spire_remote_node_descriptor_catalog_active ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1462 filtered out
```

The fixture points the secret at a nonexistent local socket with
`connect_timeout=1`, verifies the executor attempts the connection, receives
`libpq_connection_open_failed`, keeps `raw_conninfo_exposed = false`, and keeps
the next step at `open_libpq_connection`.

Also passed:

```text
git diff --check
```

## Review Notes

This is intentionally the narrow first executor slice. It proves the executor
can cross from plan/secret readiness into real connection I/O while preserving
the no-raw-conninfo contract. Follow-up slices should build on this by sending
the remote search request and validating/receiving candidate rows.
