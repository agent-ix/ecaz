# 30709 — SPIRE Remote Heap Endpoint Gate

## Summary

This packet asks for review of `d4e84e5ccecc17bf5a84cfe39288c951d4994930`
(`Gate SPIRE remote heap candidates by endpoint identity`).

The slice closes a Stage B/D receive-boundary gap: compact remote candidate
receive already rejected non-ready endpoint identities, but the origin-node
remote heap candidate path could reach final-row delivery through a narrower
row shape. The coordinator now preflights
`ec_spire_remote_search_endpoint_identity(...)` after resolving the remote
index and before either compact candidate receive or remote heap candidate
receive.

## Changes Under Review

- Added `validate_remote_search_libpq_endpoint_identity_for_dispatch(...)`,
  shared by both libpq compact candidate dispatch and libpq remote heap
  candidate dispatch.
- Refactored endpoint validation so the same protocol, extension version,
  opclass, storage format, assignment payload, quantizer profile, scoring
  profile, fingerprint, and ready-status checks apply to endpoint-identity rows
  and candidate row envelopes.
- Added an endpoint contract row:
  `remote_heap_candidate_endpoint_identity_preflight`.
- Added PG18 coverage proving
  `ec_spire_remote_search_coordinator_result_summary(...)` rejects a non-RaBitQ
  remote-serving endpoint before final remote heap rows are accepted.
- Updated the Phase 11 task file without marking the broader Stage B/D work
  complete.

## Validation

Raw logs are in `artifacts/`; see `artifacts/manifest.md` for metadata and key
lines.

- `cargo pgrx test pg18 test_ec_spire_remote_search_receive_contract`
- `cargo pgrx test pg18 test_ec_spire_remote_search_libpq_executor_loopback_empty`
- `cargo pgrx test pg18 test_ec_spire_heap_endpoint_rejects_non_ready`
- `cargo pgrx test pg18 test_ec_spire_libpq`
- `cargo fmt`
- `git diff --check`

## Review Questions

- Is the endpoint identity preflight placed at the right receive boundary for
  both compact candidates and remote heap candidates?
- Is the extra per-dispatch endpoint identity query acceptable as a correctness
  gate for Stage B/D, with caching/pipelining still owned by Stage C?
- Does the new coordinator-result negative fixture prove the remote heap/final
  row path cannot bypass the non-ready endpoint gate?
