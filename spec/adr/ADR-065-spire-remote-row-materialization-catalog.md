---
id: ADR-065
title: "SPIRE Remote Row Materialization Catalog"
status: PROPOSED
impact: Affects Task 30 Phase 11 Stage D catalog-backed remote row delivery
date: 2026-05-10
---
# ADR-065: SPIRE Remote Row Materialization Catalog

## Status

Proposed.

## Related

- ADR-059 defines origin-node heap visibility resolution.
- ADR-064 defines the v1 lifecycle rule: the SPIRE index AM may only return a
  remote-origin row when a coordinator-visible row already exists in the same
  heap relation being scanned. The scan path must not create proxy rows.

## Context

Packet 30797 added the AM-side provider seam. The seam accepts a
materialized-row mapping only when it preserves the remote identity and proves
the returned TID belongs to the scanned coordinator heap relation and is visible
to the scan snapshot.

The remaining Stage D question is where the provider reads those mappings from.
The storage must support exact remote identity lookup, work across scans, clean
up with normal remote catalog lifecycle, and avoid per-row or per-query heap
writes in `amrescan` / `amgettuple`.

## Decision

SPIRE v1 stores coordinator materialized-row mappings in an extension-owned
catalog table, `ec_spire_remote_row_materialization`.

The catalog identity is:

- `coordinator_index_oid`
- `requested_epoch`
- `served_epoch`
- `origin_node_id`
- `vec_id`
- opaque `row_locator`

The mapping value is:

- `scan_heap_relation_oid`
- `materialized_heap_block`
- `materialized_heap_offset`
- `status`
- `materialized_at_micros`

The AM provider reads catalog rows in a batch for the current result stream,
then validates each exact mapping through the provider seam from packet 30797.
Only `status = 'ready'` rows may be considered for delivery. The provider must
also fetch the materialized heap TID under the current scan snapshot before
marking a mapping visible. If the row is absent or invisible, the remote-origin
output fails closed before AM delivery with `remote_row_materialization`.

## Required Invariants

- The catalog never stores or exposes decoded origin heap tuple identity. The
  origin `row_locator` remains opaque coordinator-side bytes.
- `coordinator_index_oid` scopes mappings to the SPIRE index that produced the
  result stream.
- `scan_heap_relation_oid` must match the heap relation owning that index.
- The AM provider must validate visibility with PostgreSQL heap fetch APIs
  under the executor scan snapshot; catalog presence alone is not enough.
- The scan path may read the catalog and heap row. It must not insert, update,
  or delete materialization rows or user heap rows.
- Cleanup is owned by operator or epoch lifecycle surfaces and by remote
  catalog cleanup, not by scan state.

## Cleanup Ownership

`ec_spire_remote_catalog_index_cleanup` and the DROP INDEX event trigger remove
catalog rows for the dropped `coordinator_index_oid`.
`ec_spire_remote_catalog_orphan_cleanup` removes rows whose coordinator SPIRE
index no longer exists.

Epoch-retirement cleanup of still-live indexes is intentionally separate from
this ADR. It should mark or remove obsolete mappings outside the AM scan path
and after the epoch retention window is safe.

## Rationale

Keeping the mapping in an extension-owned catalog makes the scan path a pure
lookup and validation operation. It also gives operators a SQL-visible
inspection surface for materialization readiness without storing raw libpq
conninfo or decoded remote row locators.

Batch lookup avoids a per-row SPI query during AM finalization. Snapshot
validation still happens per candidate TID, because PostgreSQL visibility is a
heap/MVCC property, not a durable catalog fact.

## Consequences

- Catalog rows can make remote-origin outputs AM-deliverable once the
  coordinator heap row already exists and is visible.
- Missing, stale, wrong-relation, or invisible mappings remain fail-closed at
  `remote_row_materialization`.
- The catalog is not the mirror lifecycle itself; it records mappings to rows
  created by an operator-owned materialization or replication process.
- A future cleanup slice must add epoch-retirement handling for obsolete
  materialization rows on live indexes.
