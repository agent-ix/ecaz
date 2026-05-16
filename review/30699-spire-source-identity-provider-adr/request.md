# Review Request: SPIRE Source Identity Provider ADR

Status: open
Owner: coder1
Head SHA: `28a4d7ef2547c2f6434fa024b2bd223fdd138901`

## Summary

This planning slice responds to the Phase 11 review request to go deeper before
landing broad production code. It adds the provider-selection ADR needed before
the next Phase 11.2 writer implementation slice and expands the production
landing sequence through local multi-instance readiness.

Key changes:

- Added `spec/adr/ADR-063-spire-source-identity-provider.md`.
- ADR-063 selects `source_identity = 'include'` as the v1 production source
  identity provider.
- The selected DDL shape is one SPIRE vector key column plus one included
  identity column when the reloption is configured.
- Supported v1 canonical forms are `uuid` raw 16 bytes and exact-16-byte
  `bytea`.
- NULL, unsupported types, malformed bytea values, and missing identity columns
  under `source_identity = 'include'` must reject rather than falling back to
  local IDs.
- Expanded Phase 11 into staged production gates: writer identity provider,
  production remote endpoint, production libpq coordinator, remote heap
  resolution, local multi-instance epoch/lifecycle/fault matrix, multi-store
  hardening, and the final AWS gate.
- Updated the stable source identity and paper-parity planning docs to point at
  ADR-063.

## Deliberate Limit

This packet does not implement the access-method changes. The next code slice
should only proceed after ADR-063 is accepted or revised, because it changes the
AM DDL contract, build/insert callback handling, and global-ID failure policy.

## Validation

- `git diff --check`
  - passed

No Rust or pgrx tests were run because this is a planning/ADR-only checkpoint.

## Review Focus

- Is the included-column provider the right v1 contract for stable cross-node
  vector identity?
- Are `uuid` and exact-16-byte `bytea` sufficient and strict enough for the
  first production path?
- Does the expanded Stage A-G sequence cover the production-readiness gaps
  raised in review without over-constraining later implementation?
