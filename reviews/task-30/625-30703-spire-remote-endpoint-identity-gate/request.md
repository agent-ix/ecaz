# Review Request: SPIRE Remote Endpoint Identity Gate

Status: open
Owner: coder1
Head SHA: `6e9df896a562a3e2429a7d7c89f71b8a34fadc63`

## Summary

This Phase 11.3 / Stage B slice adds a serving-identity gate for SPIRE remote
search endpoints. It gives the coordinator and reviewers a stable, SQL-visible
identity record to validate before we place those fields into the candidate row
wire shape.

Key changes:

- Adds `ec_spire_remote_search_endpoint_identity(index_oid)` with protocol
  version, extension version, opclass identity, storage format, assignment
  payload format, quantizer profile, scoring profile, deterministic profile
  fingerprint, status, and recommendation.
- Treats RaBitQ as the only ready Phase 11 production remote serving profile.
  Default `auto`/TurboQuant endpoint identities report
  `requires_rabitq_storage_format`.
- Adds the identity surface to the operator entrypoint contract.
- Records conservative Stage B progress in the Phase 11 task file.
- Adds a focused PG18 test proving a RaBitQ SPIRE index reports a ready
  identity while the default non-RaBitQ endpoint remains blocked.

## Deliberate Limits

- The identity row is not yet included in `ec_spire_remote_search` candidate
  rows or libpq decoding.
- The profile fingerprint is a deterministic serving-profile fingerprint over
  protocol, extension, opclass, storage/assignment format, quantizer profile,
  scoring profile, key build options, and active epoch. A deeper training-stat
  fingerprint can replace or extend it when training metadata is persisted in a
  more explicit format.
- PQ/PQFastScan remains unsupported for Phase 11 remote serving.

## Validation

- `cargo fmt`
  - passed; rustfmt still prints existing stable-toolchain warnings for
    unstable import-grouping settings
- `cargo test endpoint_identity --lib`
  - passed: 1 passed, 0 failed
- `git diff --check`
  - passed

## Review Focus

- Is the identity surface the right contract object to review before expanding
  remote candidate rows?
- Is `requires_rabitq_storage_format` the right fail-closed behavior for
  default/non-RaBitQ SPIRE indexes in Phase 11 remote serving?
- Is the current profile fingerprint scope acceptable as an initial served
  identity, given that explicit training-stat fingerprinting is still called
  out as a follow-up?
