---
id: ADR-064
title: "SPIRE Remote Row Materialization Lifecycle"
status: PROPOSED
impact: Affects Task 30 Phase 11 Stage D remote-origin final row delivery
date: 2026-05-10
---
# ADR-064: SPIRE Remote Row Materialization Lifecycle

## Status

Proposed.

## Context

ADR-059 assigns remote heap visibility to the origin node and keeps remote row
locators opaque at the coordinator. Packet 30761 adds the next AM delivery
contract: a PostgreSQL index AM may return only `xs_heaptid` values that point
into the heap relation being scanned. Origin-node heap coordinates are not
coordinator heap TIDs.

That leaves one production-safe AM path for remote-origin rows: before the AM
returns a remote-origin candidate, there must already be a coordinator-visible
heap row in the same scanned heap relation. The open question is the lifecycle
of that shadow/proxy row.

## Decision

The v1 SPIRE index AM does **not** create per-query or per-cursor proxy rows.
Remote row materialization for the index AM is epoch-scoped coordinator heap
materialization:

- A remote-origin row is AM-deliverable only when the coordinator already has a
  materialized row in the same heap relation being scanned.
- The materialized row's TID, not the origin-node heap coordinate, is the only
  value that may be assigned to `xs_heaptid`.
- Materialized rows outlive one cursor. Their lifetime is tied to the published
  SPIRE epoch, the coordinator heap relation's normal MVCC rules, and the epoch
  retention window.
- Cleanup is not a scan-time release operation. Cleanup happens through epoch
  retirement plus normal PostgreSQL delete/vacuum or an explicit mirror-table
  lifecycle step that runs outside `amgettuple`.

The coordinator may implement this as an operator-managed mirror table or as a
replicated subset of the user's relation, but the row must be an ordinary heap
tuple in the relation that owns the SPIRE index. Temp tables, unlogged scratch
relations, tuplestores, and in-memory proxy tuples are not valid v1 AM
materialization targets because their tuple identity does not belong to the
scanned heap relation.

The SPIRE AM must treat missing materialization as a blocker, not as permission
to synthesize a row during scan. A future FDW/custom-scan executor may deliver
remote tuples through a different tuple path, but that is outside the v1 index
AM contract.

## Required Invariants

- `amrescan` / `amgettuple` must never write user heap rows or proxy rows.
- Every returned `xs_heaptid` must belong to the heap relation that PostgreSQL
  is scanning for the index scan.
- The materialized coordinator heap row must be visible under the scan snapshot
  before it can be returned.
- Materialization identity must preserve the remote candidate's stable
  `vec_id`, origin `node_id`, requested epoch, and opaque origin `row_locator`
  so stale mappings can be rejected.
- If a remote-origin candidate has no visible coordinator materialized TID, the
  strict path fails with `remote_row_materialization`; degraded mode may skip it
  only with explicit degraded diagnostics.
- Per-cursor cleanup is limited to scan opaque memory. It must not delete heap
  tuples that downstream PostgreSQL nodes may still fetch.

## Rationale

PostgreSQL's index AM API returns a heap TID, not a tuple. Once `amgettuple`
sets `xs_heaptid`, the executor fetches that tuple from the scan's heap
relation. A temp table row, scratch relation row, tuplestore slot, or in-memory
tuple cannot satisfy that relation identity. Writing proxy heap rows during a
read-only scan would also violate operator expectations, create MVCC cleanup
hazards, and make cancellation/error cleanup hard to reason about.

Epoch-scoped coordinator materialization keeps the AM contract simple:
origin-node heap resolution proves the remote row is visible at the origin, and
coordinator materialization proves there is a visible local heap TID that
PostgreSQL can fetch. The scan itself only validates and cursors over that
state.

## Consequences

- The next implementation slice should add a materialized-row mapping contract
  and gate remote-origin AM delivery on a visible same-relation coordinator TID.
- Production remote AM delivery requires a coordinator heap mirror/replication
  lifecycle before it can return remote-origin rows. Pure remote-only data still
  remains blocked for the index AM until that mirror lifecycle exists.
- Per-query proxy rows, temp scratch tables, and tuple-store based delivery are
  explicitly rejected for the v1 AM path.
- FDW/custom-scan tuple delivery remains a future integration for deployments
  that do not want coordinator heap mirror rows.
