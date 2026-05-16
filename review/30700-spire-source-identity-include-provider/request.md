# Review Request: SPIRE Include Source Identity Provider

Status: open
Owner: coder1
Head SHA: `fcdd8938a20c2eba5e362a055f5acbe72586a40c`

## Summary

This Phase 11.2 Stage A slice implements the first live writer-side global
vector identity provider selected by ADR-063:
`source_identity = 'include'`.

Key changes:

- Enables AM `INCLUDE` support for SPIRE indexes.
- Adds the `source_identity` reloption with the only accepted v1 provider:
  `include`.
- Validates the v1 DDL shape strictly: one vector key column, exactly one
  included source-identity column when the provider is enabled, no expression
  identity, no partial index.
- Accepts included `uuid` and exact-16-byte `bytea`, canonicalized to
  `StableFixedGlobalPayload([u8; 16])`; rejects NULL, unsupported types, and
  malformed bytea widths.
- Threads source identity through populated build, empty-index insert
  bootstrap, live insert deltas, and boundary assignment paths without
  advancing the local vec-id sequence for global rows.
- Adds `ec_spire_index_writer_identity_snapshot(...)` to classify local-only,
  global-capable-not-yet-published, and global-writer-active indexes.
- Updates the remote candidate receive contract so selected leaf batches can
  validly return leaf-derived delta object PIDs.
- Updates the Phase 11 task file with landed Stage A evidence and remaining
  gaps.

## Deliberate Limits

- ADR-063 is still open for reviewer acceptance; this code follows its selected
  design and should be revised if review changes the provider contract.
- Scheduled replacement paths still need explicit source-identity threading
  before Stage A fully closes.
- Replica-specific proof that boundary replicas share one global ID remains
  open, as does a fresh local-only namespace proof in this Stage A packet.
- PQ/PQFastScan remain out of scope; RaBitQ is still the first supported
  quantized scoring path.

## Validation

- `cargo fmt`
  - passed; rustfmt still prints existing stable-toolchain warnings for
    unstable import-grouping settings
- `cargo test source_identity --lib`
  - 5 passed, 0 failed
- `cargo test remote_candidate_batch_validation --lib`
  - 3 passed, 0 failed
- `cargo pgrx test pg18 test_ec_spire_srcid`
  - 6 passed, 0 failed
- `cargo pgrx test pg18 test_ec_spire_include_requires_srcid_reloption`
  - 1 passed, 0 failed
- `git diff --check`
  - passed

## Review Focus

- Is the `source_identity = 'include'` AM contract strict enough for the first
  production writer identity path?
- Are UUID raw bytes and exact-16-byte bytea canonicalization handled safely in
  build and insert callbacks?
- Does the delta candidate receive-contract adjustment correctly preserve the
  selected-leaf endpoint model while allowing live insert delta rows?
- Are the remaining Stage A limits correctly captured before moving to remote
  endpoint and libpq coordinator production work?
