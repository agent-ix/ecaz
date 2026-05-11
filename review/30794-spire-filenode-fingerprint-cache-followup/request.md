# Review Request: SPIRE Filenode Fingerprint Cache Follow-up

## Summary

This packet processes reviewer follow-up from 30791 and 30792.

Code checkpoint: `bd9341f5ea51326f4de8539ed5ff8c5e5668a212`

The docs now state that the v1 endpoint `profile_fingerprint` includes
`pg_relation_filenode(index_oid)` as `generation_identity`. This makes
`REINDEX INDEX CONCURRENTLY` visible through the existing descriptor
`remote_index_identity` / live `profile_fingerprint` mismatch rule, without a
second generation token beside the fingerprint.

The focused PG18 identity-cache matrix still passes after the filenode
fingerprint change, confirming that cache invalidation continues to flow
through the already-covered live fingerprint mismatch path.

## Evidence

Artifacts are stored under `artifacts/`:

- `identity-cache-pg18.log`
- `manifest.md`

Key result lines:

```text
test tests::pg_test_ec_spire_remote_search_libpq_executor_loopback_empty ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1589 filtered out
COMMAND_EXIT_CODE="0"
```

## Validation

- `git diff --check -- plan/design/spire-remote-node-model.md plan/design/spire-libpq-identity-cache.md plan/design/spire-production-coordinator-executor.md plan/tasks/task30-phase11-spire-distributed-production-parity.md`
- `cargo pgrx test pg18 test_ec_spire_remote_search_libpq_executor_loopback_empty`
- `script -q -c "cargo pgrx test pg18 test_ec_spire_remote_search_libpq_executor_loopback_empty" review/30794-spire-filenode-fingerprint-cache-followup/artifacts/identity-cache-pg18.log`

## Review Focus

- Whether documenting `pg_relation_filenode(index_oid)` as
  `generation_identity` closes the 30791 P3 design-doc gap.
- Whether the identity-cache matrix is sufficient to close the 30791 P2
  verification request, given filenode changes are surfaced as ordinary
  fingerprint changes.
- Whether the coordinator design note makes candidate/heap receive identity
  cache behavior clear enough before AM cursor integration.
