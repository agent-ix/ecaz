# Review Request: SPIRE Production Receive Identity Guard

## Summary

Code checkpoint: `1e97e67ee0f1f30bda5b500abe108a23e8a23ee8`

This slice tightens the production compact-candidate receive contract before
C5 AM scan integration:

- `SpireRemoteProductionCandidateReceiveRequest` now carries the expected
  descriptor `remote_index_identity`.
- Production receive rejects returned candidate rows whose endpoint
  `profile_fingerprint` bytes do not match that expected identity.
- The diagnostic libpq receive path now uses the same candidate-row identity
  match when decoding remote rows.
- The receive request-state test verifies descriptor identity is preserved into
  production receive requests.
- A PG18 loopback fixture proves a mismatched candidate-row fingerprint fails
  as `endpoint_identity_mismatch`.

This is not the full C3 identity cache yet. It is the first production receive
guard that prevents mismatched remote scores from entering merge state.

## Key Files

- `src/am/ec_spire/root/remote_candidates.rs`
  - `SpireRemoteProductionCandidateReceiveRequest::remote_index_identity`
  - candidate-row endpoint identity matching in
    `decode_remote_search_candidate_pg_row`
  - production decode failure category mapping to
    `endpoint_identity_mismatch`
- `src/lib.rs`
  - loopback identity-byte helper
  - `test_ec_spire_prod_receive_identity_mismatch`
  - existing production receive fixtures updated with expected identity
- `plan/tasks/task30-phase11-spire-distributed-production-parity.md`

## Validation

Packet-local logs are in `artifacts/` and indexed in
`artifacts/manifest.md`.

- `cargo fmt --check`
- `cargo check --no-default-features --features pg18`
- `cargo test --no-default-features --features pg18 production_executor_compact_receive_requests_use_dispatch_state --lib`
- `cargo pgrx test pg18 prod_receive`
- `git diff --check -- src/am/ec_spire/root/remote_candidates.rs src/lib.rs plan/tasks/task30-phase11-spire-distributed-production-parity.md`

## Review Questions

- Is matching candidate-row `profile_fingerprint` bytes to descriptor
  `remote_index_identity` the right production receive guard before the full
  identity cache lands?
- Should protocol/extension endpoint mismatches continue to collapse to the
  sanitized production category `endpoint_identity_mismatch` at this layer?
- Is the request-struct identity handoff sufficient for C5 to consume
  `CandidateReceiveReady` batches without re-resolving descriptor identity?

