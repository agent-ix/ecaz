# Review Request: SPIRE Remote Epoch Manifest Apply

## Scope

This packet reviews commit `1d8499b9 Apply SPIRE remote epoch manifests over libpq`.

The slice turns the manifest libpq executor from validation-only into an apply path:

- adds durable remote-side catalog tables:
  - `ec_spire_remote_epoch_manifest_applied`
  - `ec_spire_remote_epoch_manifest_applied_entry`
- adds `ec_spire_apply_remote_epoch_manifest_payload(remote_index_oid, active_epoch, manifest_payload)`
  to validate and persist the received manifest payload on the remote node.
- rewires `ec_spire_remote_epoch_manifest_libpq_request_plan` so the advertised request SQL now targets
  `ec_spire_apply_remote_epoch_manifest_payload(...)`.
- rewires `ec_spire_remote_epoch_manifest_libpq_executor_results` so the actual libpq call uses the
  same apply endpoint as the dispatch contract.
- addresses reviewer F2/F3 from 30648/30649:
  - replaces the dead executor `raw_conninfo_exposed` result column with `conninfo_lookup_kind`.
  - refactors manifest executor result construction through `SpireManifestExecutorResultRow` and converts
    to the SQL tuple in one place.

## Validation

Focused PG18 coverage:

```text
cargo pgrx test pg18 test_ec_spire_remote_epoch_manifest_libpq_executor_loopback
cargo pgrx test pg18 test_ec_spire_remote_search_libpq_executor_loopback_empty
git diff --check
```

Both PG18 tests passed. The manifest loopback fixture now asserts the remote applied manifest header and
entry rows exist after the libpq executor runs.

## Review Notes

- `validated_entry_count` remains the result column name for compatibility with the existing executor result
  contract, but it is now the count of entries accepted by the apply endpoint.
- `conninfo_lookup_kind` is intentionally diagnostic only. It reports `secret_provider` or `not_attempted`
  and never returns the resolved conninfo string.
- This is still loopback coverage, not multi-cluster coverage. Multi-cluster fixture work remains a later
  Phase 7 item.
