# 30710 — SPIRE Remote Descriptor Identity Binding

## Summary

This packet asks for review of `e5bdf0803ef834849b2a997fd8d914dc76b4a565`
(`Bind SPIRE remote descriptor identity before receive`).

Packet 30709 made the libpq executor preflight live endpoint readiness before
compact or heap receive. This slice binds that live endpoint to the
coordinator's remote-node descriptor: the executor now compares descriptor
`remote_index_identity` bytes with the live endpoint `profile_fingerprint`
bytes before accepting any remote candidate batch.

## Changes Under Review

- Threaded descriptor `remote_index_identity` bytes through internal libpq
  connection and dispatch rows. SQL-visible connection diagnostics still expose
  only the byte count.
- Extended the endpoint preflight to decode the live endpoint
  `profile_fingerprint` hex and compare it against the descriptor identity.
- Classified descriptor/endpoint identity mismatch as
  `endpoint_identity_mismatch` with next blocker `remote_endpoint_identity` in
  receive-attempt diagnostics.
- Updated the ready RaBitQ loopback fixture so it registers the descriptor
  identity from the live remote endpoint fingerprint.
- Added strict PG18 coverage proving a ready RaBitQ remote with the wrong
  descriptor identity reports `endpoint_identity_mismatch` and fails closed
  before compact candidate merge.
- Updated the Phase 11 task file without marking the wider Stage B verification
  line complete.

## Validation

Raw logs are in `artifacts/`; see `artifacts/manifest.md` for metadata and key
lines.

- `cargo pgrx test pg18 test_ec_spire_libpq`
- `cargo pgrx test pg18 test_ec_spire_remote_search_libpq_executor_loopback_empty`
- `cargo fmt`
- `git diff --check`

## Review Questions

- Is comparing descriptor `remote_index_identity` to live endpoint
  `profile_fingerprint` the right first production binding for Stage B?
- Is it acceptable that SQL-visible connection-plan diagnostics continue to
  expose only identity byte count while the internal dispatch row carries the
  bytes?
- Does the strict mismatch fixture cover the right boundary before merge?
