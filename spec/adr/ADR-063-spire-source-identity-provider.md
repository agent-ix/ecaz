---
id: ADR-063
title: "SPIRE Source Identity Provider"
status: PROPOSED
impact: Affects ADR-055, Task 30 Phase 11.2 writer-side global vector identity
date: 2026-05-09
---
# ADR-063: SPIRE Source Identity Provider

## Status

Proposed.

## Context

ADR-055 defines local `0x01` and global `0x02` SPIRE vector identities. Phase
11.2 has landed the allocation hook, fixed-width Leaf V2 global storage, and a
16-byte stable source identity contract. The remaining gap is a live writer
provider: build and insert callbacks must receive stable source-identity bytes
from real table data before `ec_spire` can emit global IDs end to end.

The current access method only supports one key column, sets `amcaninclude =
false`, and rejects multi-column, expression, and partial indexes. The callback
can reliably read the indexed vector datum and heap TID today. Heap TID is not
a stable source identity because it can change across VACUUM, rewrite, restore,
replica rebuild, and cross-node ingest.

## Decision

The v1 production provider is an explicit included identity column:

```sql
CREATE INDEX ... USING ec_spire (embedding)
INCLUDE (source_identity)
WITH (source_identity = 'include');
```

The index has exactly one key attribute, the vector column, and zero or one
included source-identity attribute. When the `source_identity = 'include'`
reloption is set, exactly one included attribute is required and live writers
must canonicalize that included value into the 16-byte Phase 11 source payload.

Supported v1 canonicalization:

- `uuid`: use the 16 raw UUID bytes as stored by PostgreSQL.
- `bytea`: require exactly 16 detoasted bytes and use them directly.

Expression-derived identities are represented in v1 by a stored generated
column or other ordinary heap column included in the index. Native expression
index identities are deferred until the AM has a broader expression-index
contract.

## Required Invariants

- If `source_identity = 'include'` is configured, build and insert reject NULL
  identity values.
- If the included identity type is unsupported or a `bytea` value is not
  exactly 16 bytes, build and insert reject the row. They must not fall back to
  local `0x01` for that row because mixed identity namespaces inside a global
  writer index would break cross-node dedupe.
- If `source_identity` is not configured, existing single-column indexes keep
  local `0x01` IDs and remote merge scopes them by `node_id`.
- A source identity is considered stable only if it remains the same for the
  same logical source row across origin-node UPDATE-as-delete-plus-insert,
  logical replication, backup/restore, and remote replica rebuild.
- Operators must treat changes to the source identity value for the same
  logical row as a new vector identity.

## Rationale

An included column is visible in DDL, carried through PostgreSQL build/insert
callback value arrays, and does not make the identity a search key. That keeps
the vector operator class single-column while providing deterministic bytes for
global dedupe.

Using UUID and exact 16-byte `bytea` avoids hashing ambiguity in the first
production path. Hash-based canonicalization of text or composite keys can land
later as an explicit extension once collision policy, collation/encoding
normalization, and cross-version stability are specified.

## Consequences

- The next Phase 11.2 code slice must enable `amcaninclude`, add the
  `source_identity` reloption, validate the exact DDL shape, and thread the
  included identity bytes into `SpireVecIdSourceIdentity::StableFixedGlobalPayload`.
- Indexes without `source_identity = 'include'` remain local-ID compatible and
  need diagnostics that say they cannot make cross-node replica dedupe claims.
- A future ADR may extend this provider with expression indexes or hash-based
  canonicalization, but that must not alter the UUID/bytea16 semantics above.
