# 30647 — SPIRE Remote Search Libpq Executor Send

## Summary

This packet extends the remote-search executor from connection probing to an
actual libpq request path.

Code commit:

- `0c0480be61c46dc866409caf71602d7522bb5404` — Send SPIRE remote search over libpq executor

## What Changed

- Added `ec_spire_remote_search_libpq_executor_candidates(...)`.
- The executor now:
  - consumes existing dispatch rows,
  - resolves the executor-owned conninfo secret internally,
  - opens a PostgreSQL client connection,
  - resolves `remote_index_regclass` to an OID on the remote connection with
    `to_regclass($1)::oid`,
  - sends the existing six-parameter
    `ec_spire_remote_search($1::oid, $2::bigint, $3::real[], $4::bigint[], $5::integer, $6::text)`
    request,
  - decodes the result contract into candidate rows,
  - validates the received batch with `validate_remote_search_candidate_batch`.
- Local-node result rows from a storage endpoint are enveloped under the
  coordinator descriptor `node_id` before validation.
- The SQL surface returns the same candidate shape as the remote search result
  contract.

## Validation

Focused PG18 validation passed:

```text
cargo pgrx test pg18 test_ec_spire_remote_search_libpq_executor_loopback_empty
```

Key result:

```text
test tests::pg_test_ec_spire_remote_search_libpq_executor_loopback_empty ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1463 filtered out
```

The fixture creates a committed loopback "remote" SPIRE index through a separate
client connection, registers a coordinator descriptor pointing at that remote
index, verifies `ec_spire_remote_search_libpq_executor_connection_check` reports
`libpq_connection_opened`, and then verifies the executor candidate function
successfully sends a `top_k = 0` remote search and returns an empty candidate
set.

Also passed:

```text
git diff --check
```

## Review Notes

This is the first send/receive slice. It intentionally validates the empty
candidate path first because it proves remote connection, remote OID resolution,
parameter binding, endpoint invocation, result decoding, and batch validation
without taking on nonempty remote heap identity semantics in the same packet.
