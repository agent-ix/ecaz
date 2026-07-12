---
agent: claude
role: coder
model: claude-opus-4-8
date: 2026-07-11
seq: 01
---

# Review request — Packet 014 epoch topology (by fingerprint)

Implements `ec_distann_epoch_topology(index_regclass regclass, epoch_fingerprint bytea)
RETURNS TABLE(...)` (FR-078:801-804), completing the topology surface deferred in
packet 008 (now feasible since the publish pipeline produces a Published generation).

## Commit
- `d284190b` — `ec_distann_epoch_topology` + `build_topology_row` refactor.

Artifacts + provenance in `artifacts/manifest.md`.

## Contract mapping
- **Version check**: the fingerprint must be 34 bytes `u16_le(2) || 32-byte
  digest`; an unknown version fails `EC_EPOCH_FINGERPRINT_VERSION` before lookup.
- **Resolution**: the fingerprint resolves to its build id through the durable
  `ec_distann_publish_decision`; the generation must be `Published` or retained
  `Retired`. Unknown/in-progress/Reclaimed → `EC_GENERATION_MISSING`.
- **Shared emission**: the 15-column diagnostic (descriptor decode, relation
  locks against concurrent reclaim, `diagnose_physical_generation`, sizes) is
  factored into `build_topology_row`, shared with `ec_distann_generation_topology`
  (by build id, Building/Ready). All counts/digests recomputed from storage.

## Validation
- `cargo check` + strict clippy (`pg18 pg_test`, `-D warnings`) — pass at `d284190b`.
- `cargo pgrx test pg18 test_distann_build_epoch_single_node` — 1/1 pass: Published
  epoch resolved by fingerprint (3 records), plus the two negative cases.

Both topology endpoints (by build id / by fingerprint) are now complete. Next:
retirement + abandon, then the physical read path. Leaving the request open.
