---
id: ADR-059
title: "SPIRE Remote Heap Resolution Contract"
status: ACCEPTED
impact: Affects Task 30 Phase 10 remote final row delivery
date: 2026-05-09
---
# ADR-059: SPIRE Remote Heap Resolution Contract

## Status

Accepted.

## Related

- ADR-064 defines the separate AM-side materialization lifecycle required after
  origin-node heap visibility succeeds. ADR-059 decides who resolves remote
  heap visibility; ADR-064 decides when the coordinator may turn a visible
  remote-origin row into a local `xs_heaptid`.

## Context

Remote SPIRE search returns compact candidates with `node_id`, `vec_id`,
`score`, assignment metadata, and an opaque `row_locator`. The coordinator can
validate, merge, and dedupe these candidates, but it cannot safely interpret a
remote node's heap locator as a local heap TID.

The current SQL-visible libpq heap-candidate surface is diagnostic under
ADR-058. It is useful for loopback proof and operator checks, but it is not the
production AM final-row path.

Cross-node boundary-replica dedupe also depends on stable global vector
identity. Existing local `0x01` vec IDs are node-scoped compatibility IDs; they
are not sufficient for an end-to-end production claim that replicas across
different nodes dedupe as the same source vector.

## Decision

Remote heap resolution is owned by the origin node. The coordinator treats
remote `row_locator` bytes as opaque and must not decode them directly.

Until a production origin-node heap resolver lands, remote heap rows remain an
explicit blocked/deferred result state. The coordinator may expose validated
candidate batches and diagnostics, but it must not present remote candidates as
fully ready final SQL rows.

Any production remote heap resolver must return only heap-visible rows from the
origin node under the requested epoch and consistency contract. Rows that are
not heap-visible on the origin node must be omitted or reported through an
explicit degraded/blocked result status; they must not be returned as partially
ready candidates.

Production cross-node boundary-replica dedupe requires writer-side global vec
ID allocation using the `0x02 || stable_global_payload_bytes` format from
ADR-055. The node-scoped local vec-id fallback remains valid for diagnostics
and single-node compatibility, but it is not enough for a production
cross-node replica-dedupe claim.

## Required Invariants

- `row_locator` is scoped to the origin node and opaque to the coordinator.
- Remote final row delivery must happen by asking the origin node, not by
  decoding the remote locator locally.
- Production remote heap-ready status requires origin-node heap visibility
  checks under the requested epoch and consistency mode.
- Without global `0x02` vec IDs, cross-node boundary-replica dedupe must be
  described as blocked or diagnostic-only.
- Blocked remote heap resolution must be visible in coordinator result
  summaries instead of surfacing partial final rows.

## Rationale

PostgreSQL heap TIDs, MVCC visibility, relation identity, and row contents are
owned by the origin node. Treating an opaque remote locator as local state would
couple the coordinator to remote physical layout and would bypass the node that
can actually evaluate heap visibility.

Keeping remote candidates blocked until origin-node heap resolution exists is
less convenient than returning partial rows, but it prevents the system from
appearing production-ready while final row delivery and replica identity are
still incomplete.

## Consequences

- Phase 10.6 chooses origin-node heap resolution as the production ownership
  model.
- Existing coordinator summaries that report
  `requires_remote_heap_resolution` are the correct production state until the
  origin-node resolver lands.
- The SQL-visible heap-candidate executor remains diagnostic/operator-only per
  ADR-058.
- Future implementation work must pair remote heap visibility checks with
  writer-side global vec-id allocation before claiming end-to-end distributed
  boundary-replica dedupe.
