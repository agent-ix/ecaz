# Review Request: SPIRE Remote Endpoint Ready Gate

Status: open
Owner: coder1
Head SHA: `c2e894ab67cc0c67e1944fb4e14b54a348ebc56b`

## Summary

This Phase 11.3 / Stage B slice makes the libpq candidate receive path fail
closed on non-ready endpoint identities before candidates can enter the merge
path.

Key changes:

- `validate_remote_search_candidate_endpoint_identity()` now rejects candidate
  rows whose `endpoint_status` is not `ready`.
- The libpq decode path already validates protocol version, extension version,
  and nonempty endpoint identity fields; this slice adds the readiness gate.
- The loopback executor PG18 fixture now builds its remote-serving SPIRE index
  with `storage_format = 'rabitq'`, matching the Phase 11 ready remote-serving
  profile.
- The Phase 11 task file records this as the first merge-entry gate for RaBitQ
  profile / extension / opclass identity binding.

## Deliberate Limits

- Degraded-mode skip reporting for non-ready endpoint identities is still open.
  This slice enforces the fail-closed libpq receive path.
- Direct calls to `ec_spire_remote_search` may still show non-ready endpoint
  rows for diagnostics; production libpq receive rejects them before merge.

## Validation

- `cargo fmt`
  - passed; rustfmt still prints existing stable-toolchain warnings for
    unstable import-grouping settings
- `cargo test remote_search_libpq_executor_loopback_empty --lib`
  - passed: 1 passed, 0 failed
- `git diff --check`
  - passed

## Review Focus

- Is libpq receive the right first enforcement point for non-ready endpoint
  status before merge?
- Is it acceptable that degraded skip behavior remains a follow-up, while
  strict/libpq receive fails closed now?
- Does switching the loopback remote-serving fixture to RaBitQ match the Phase
  11 remote serving contract?
