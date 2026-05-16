# 30716 — SPIRE libpq degraded identity mismatch cache flow

Code commit: `2610837d3898d9652faf977d8db3a99bcda74cf0`

This packet closes the 30714 P2 follow-up before broader Stage C identity-cache
reuse: degraded-mode live endpoint fingerprint mismatch now has direct cache-flow
coverage.

## What Changed

- Added PG18 loopback coverage for a descriptor `remote_index_identity` that
  disagrees with the live remote endpoint `profile_fingerprint` while the
  coordinator epoch is in degraded mode.
- The test asserts the receive-attempt policy shape required by 30714:
  `endpoint_identity_mismatch`, `next_blocker = remote_endpoint_identity`,
  `failure_action = skip_node`, and zero candidates.
- The test also asserts the identity-cache summary stays empty on this mismatch:
  one dispatch row, zero compact candidates, zero heap candidates, one live
  identity query, one miss, zero hits, and zero cached entries.
- Updated the identity-cache summary helper so endpoint-identity mismatch is
  reported as a summary status with zero compact/heap candidates instead of
  raising through the diagnostic summary surface.
- Updated the Phase 11 Stage C task file to mark degraded live-fingerprint
  mismatch coverage complete.

## Validation

See `artifacts/manifest.md` for command metadata and key result lines.

- `cargo check --no-default-features --features pg18`
- `cargo pgrx test pg18 test_ec_spire_libpq_degraded_identity_mismatch_skips`
- `cargo pgrx test pg18 test_ec_spire_remote_search_libpq_executor_loopback_empty`
- `cargo pgrx test pg18 test_ec_spire_libpq_rejects_identity_mismatch`
- `git diff --check`

## Review Focus

- Confirm the degraded mismatch assertions satisfy the 30714 P2 requirement:
  skip the node, report `remote_endpoint_identity`, and keep compact/heap
  candidates out of the result path.
- Confirm the identity-cache summary behavior is the right operator surface:
  report `endpoint_identity_mismatch` with zero compact/heap candidates instead
  of throwing, while strict executor candidate receive still fails closed.
- Confirm this is enough to unblock the next Stage C slice on broader
  coordinator resource governance and cancellation.
