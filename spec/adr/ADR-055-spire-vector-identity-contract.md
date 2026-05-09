---
id: ADR-055
title: "SPIRE Vector Identity Contract"
status: ACCEPTED
impact: Affects ADR-049, Task 30 Phase 9 graph architecture, remote merge
date: 2026-05-09
---
# ADR-055: SPIRE Vector Identity Contract

## Status

Accepted.

## Context

SPIRE scans deduplicate candidate rows by `SpireVecId`. That is correct for
boundary replicas inside one index because primary and replica rows share the
same vector identity. It is not automatically correct across serving nodes:
existing local writers allocate `0x01 || local_u64` IDs, and two nodes can
legitimately emit the same local sequence for unrelated vectors.

Task 30 Phase 9 needs a remote merge contract that prevents false cross-node
dedupe while preserving existing local-only indexes.

## Decision

`SpireVecId` has two durable identity forms:

- local: `0x01 || little_endian_u64`;
- global: `0x02 || stable_global_payload_bytes`.

The global form is the only identity that deduplicates across nodes. Existing
local IDs remain valid, but remote merge treats them as node-scoped by using
`node_id || local_vec_id_bytes` as the compatibility dedupe key.

Boundary replica rows must continue to store identical `SpireVecId` bytes for
the same original vector. Cross-node replicas need the global `0x02` form to
dedupe as one vector across nodes; local `0x01` IDs dedupe only within their
origin node.

## Required Invariants

- Candidate receive validation rejects malformed `vec_id` bytes before merge.
- Remote merge dedupes global IDs by raw global identity bytes.
- Remote merge dedupes local IDs only after scoping by origin `node_id`.
- Merge tie-break order remains score, assignment role, served epoch, node ID,
  PID, object version, row index, and row locator.
- Existing local-only indexes do not need an immediate rewrite.

## Rationale

Failing closed on all local IDs would block current local and loopback remote
diagnostics. Dedupe by raw local bytes would silently merge unrelated vectors
from different nodes. Node-scoped local fallback avoids both problems and makes
the remaining limitation explicit: cross-node replica dedupe requires global
IDs.

## Consequences

- Remote merge diagnostics should describe the dedupe key as
  `global_vec_id_or_node_scoped_local_vec_id`, not plain `vec_id`.
- Product-scale multi-node deployments that replicate the same original vector
  across nodes need a global ID allocation or source-ID encoding path.
- A future migration can rewrite local IDs into global IDs without changing the
  candidate merge comparator.
