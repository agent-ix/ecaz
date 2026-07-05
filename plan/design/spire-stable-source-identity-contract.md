# SPIRE Stable Source Identity Contract

Status: active Phase 11.2 writer contract
Task: Task 30 Phase 11
Provider ADR: `spec/adr/ADR-063-spire-source-identity-provider.md`

## Decision

Phase 11 distributed SPIRE uses a fixed-width stable source identity when a
writer emits a global `0x02` vector ID:

- The global source identity payload is exactly 16 bytes.
- The bytes are opaque to the storage layer, but they must be deterministic for
  the same logical source row on every node that can produce or replicate that
  row.
- Heap TID is explicitly not a stable source identity. It can move on VACUUM,
  rewrites, restore, replica rebuild, or cross-node ingest.
- The assignment layer exposes
  `SpireVecIdSourceIdentity::StableFixedGlobalPayload([u8; 16])` for the future
  live writer path.
- Variable-length `StableGlobalPayload(Vec<u8>)` remains available for
  row-encoded diagnostics and compatibility helpers, but it is not the Phase 11
  live writer contract for Leaf V2 base objects.

## Required Live Writer Provider

The current `ec_spire` AM build/insert callbacks only receive the indexed
vector datum and heap TID, and `resolve_indexed_vector_kind` still rejects
multi-column, expression, and partial indexes. Therefore the current live writer
cannot derive a production global source identity by itself.

A production provider must enter through one of these surfaces:

- A supported source-identity column that the AM reads during build and insert
  and canonicalizes into the 16-byte payload. ADR-063 selects one included UUID
  or exact-16-byte `bytea` column as the v1 provider.
- A higher-level distributed ingest path that already owns a stable source key
  and passes the 16-byte payload into the assignment builders.

Until one of those providers lands, live build/insert paths must continue to
use local `0x01` IDs and remote merge must scope them by `node_id`.

## Leaf V2 Requirement

Leaf V2 global base objects require a fixed vector-ID stride per object. The
16-byte source payload produces a 17-byte stored `SpireVecId` including the
`0x02` discriminator, which satisfies that fixed-width requirement and avoids a
new storage version.

## Open Follow-Up

The next Phase 11.2 code slice should implement ADR-063: enable the INCLUDE
provider shape, validate DDL, canonicalize UUID/bytea16 values, and thread the
payload into live build/insert writers.
