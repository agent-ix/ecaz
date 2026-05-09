# 30649 — SPIRE Manifest Libpq Executor Results

## Summary

This packet starts the actual manifest publication executor after the prior
manifest surfaces stopped at pre-I/O readiness.

Code commit:

- `38fae3e92f26d3c56b0db2707d7cd5ee6309ebbd` — Execute SPIRE manifest publication over libpq

## What Changed

- Added `ec_spire_remote_epoch_manifest_libpq_executor_results(...)`.
- The executor consumes `ec_spire_remote_epoch_manifest_libpq_dispatch_plan(...)`.
- For ready dispatch rows, it:
  - resolves the executor-owned conninfo secret internally,
  - opens a PostgreSQL client connection,
  - resolves `remote_index_regclass` on the remote connection with
    `to_regclass($1)::oid`,
  - sends the manifest payload to
    `ec_spire_validate_remote_epoch_manifest_payload(...)`,
  - returns sanitized validation status, validated entry count, next executor
    step, and recommendation.
- It does not expose raw conninfo or raw remote connection error text.

## Validation

Focused PG18 validation passed:

```text
cargo pgrx test pg18 test_ec_spire_remote_epoch_manifest_libpq_executor_loopback
```

Key result:

```text
test tests::pg_test_ec_spire_remote_epoch_manifest_libpq_executor_loopback ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1464 filtered out
```

The fixture creates a committed loopback remote SPIRE index through a separate
client connection, creates a transactional coordinator SPIRE index, registers a
remote descriptor pointing at the loopback index, persists the remote epoch
manifest, and verifies executor results:

- `connection_attempted = true`
- `connection_status = libpq_connection_opened`
- `validated_entry_count = 1`
- `validation_result_status = ready`
- `raw_conninfo_exposed = false`
- `status = ready`

Also passed:

```text
git diff --check
```

## Review Notes

This is still a validation-result executor, not durable remote application of
the manifest. It closes the actual manifest libpq send/receive gap and leaves
remote-applied catalog persistence/rename resolution as the next manifest
follow-up.
