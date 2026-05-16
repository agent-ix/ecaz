---
id: ADR-067
title: "SPIRE Distributed Scan via CustomScan"
status: PROPOSED
impact: Replaces ADR-064 / ADR-065 / ADR-066 for the SPIRE distributed read
  path. Affects Task 30 Phase 11 Stage D and downstream stages. Phase 11.5
  mirror-sync work is superseded.
date: 2026-05-10
---
# ADR-067: SPIRE Distributed Scan via CustomScan

## Status

Proposed.

## Related

- ADR-049 governs SPIRE staging and partition-object storage.
- ADR-059 owns origin-node heap visibility resolution and keeps `row_locator`
  bytes opaque at the coordinator.
- ADR-063 defines the source identity contract for global vector IDs.
- **Supersedes ADR-064** (materialization lifecycle), **ADR-065** (catalog),
  and **ADR-066** (operator-owned mirror sync). Those ADRs solved a problem
  that this ADR removes by changing the integration point.
- Related to ADR-068 (distributed table topology) and ADR-069 (write path
  scope), which build on this ADR.

## Context

Phase 11 Stages A–E have built a complete distributed SPIRE read path inside
the production executor (Stage C): transport, identity, cancellation,
strict/degraded matrix, fault matrix, lifecycle matrix. The remaining v1
gap was tuple delivery from `amrescan` / `amgettuple`.

PostgreSQL's index AM contract requires `xs_heaptid` to point at a heap row
in the relation being scanned. Origin-node heap coordinates from a remote
SPIRE shard do not satisfy that constraint. ADR-064 chose to make remote-
origin AM delivery work inside the AM contract by requiring a pre-existing
same-relation coordinator heap row for every deliverable remote candidate;
ADR-065 added a catalog mapping remote identity to local TID; ADR-066
selected an operator-owned mirror sync mechanism to populate that catalog.

This composition works for correctness inside the AM but has two unacceptable
properties for a distributed vector search system:

1. **Storage does not scale out.** The coordinator's local heap must hold a
   mirror of every row whose remote-shard candidate may ever be returned.
   Aggregate dataset size is bounded by the coordinator's single-machine
   storage capacity. The "distributed" property is limited to compute
   parallelism on a shared dataset, not storage scale-out.
2. **Every searchable write costs a coordinator-side heap insert.** The
   mirror sync replicates remote-origin rows into the coordinator's heap.
   Write throughput is bounded by the coordinator's single-node WAL,
   autovacuum, and buffer pool capacity. Bulk loads quadratically penalize
   the coordinator.

These properties are inherited from the AM integration point, not from the
SPIRE algorithm. The SPIRE algorithm itself returns scored tuples; the
forced TID round-trip and mirror only exist because the AM cursor cannot
return tuples directly.

PostgreSQL has two integration points that can return tuples directly:
**Foreign Data Wrapper** (`IterateForeignScan`) and **CustomScan**
(`Begin/Exec/End` custom node). Both bypass the `xs_heaptid` constraint.
Both can handle the runtime routing property of ANN search (the planner
selects the scan path; the node does the centroid traversal at execute time
without per-leaf plan-time knowledge).

CustomScan was previously not evaluated because the project assumed runtime
routing ruled out FDW-style integrations. That assumption applied to
**static-partition FDW** (planner picks which foreign table to scan from
quals), which does not work for ANN. It does not apply to
**single-virtual-table FDW** or to **CustomScan**, both of which model
distribution inside the executor node and only require plan-time path
selection.

## Decision

SPIRE v1 distributed scan SHALL use a PostgreSQL **CustomScan** node, not
the index AM, to deliver remote-origin candidates.

A new CustomScan provider (`EcSpireDistributedScan`) is registered. The
provider:

- Hooks into the planner via `set_rel_pathlist_hook` (or equivalent
  set_join_pathlist hook for joined cases later).
- Registers a CustomPath for tables that have an `ec_spire` index whose
  active placement directory contains remote placements, when the query
  has an `ORDER BY <vector-distance-operator> LIMIT k` shape.
- At execute time, performs the full SPIRE distributed scan using the
  existing production executor state machine: routing, fanout, cancellation,
  identity validation, candidate receive, heap resolution at origin nodes,
  ordered merge with deterministic tie-breaks.
- Returns full row tuples directly to the PostgreSQL executor through the
  CustomScan tuple interface. No `xs_heaptid` indirection. No coordinator
  heap fetch. No local mirror.

The existing `ec_spire` index AM is retained unchanged for local-only
scans. When a SPIRE index has no remote placements, the planner picks the
existing AM index path. When remote placements exist, the planner picks
the CustomScan path. The two paths coexist.

## Required Invariants

- `EcSpireDistributedScan` SHALL be the only path the planner uses for
  vector-distance ordered scans against an `ec_spire` index with active
  remote placements.
- The CustomScan SHALL preserve every existing executor-state invariant:
  ADR-059 locator opacity, ADR-063 source identity, identity preflight,
  strict/degraded matrix, fault matrix vocabulary.
- The CustomScan SHALL return ordered tuples consistent with the existing
  deterministic tie-break ordering from the executor merge.
- No coordinator-side mirror rows, no `ec_spire_remote_row_materialization`
  catalog reads, and no `ec_spire_register_remote_row_materialization`
  calls SHALL exist on the v1 CustomScan delivery path.
- The CustomScan SHALL emit cancellation, timeout, identity-mismatch,
  governance-overload, and degraded-skip diagnostics through the same
  surfaces operator runbooks already consume (operator diagnostics rollup,
  fault matrix).
- Local-only `ec_spire` index scans SHALL continue to use the existing
  index AM unchanged.

## What is Preserved from Existing Work

The bulk of Stages A–E remains intact and reusable:

- **Stage A** writer identity, Leaf V2 storage, ADR-063 source identity.
- **Stage B** remote endpoint contract, 18-column candidate envelope,
  FNV-1a fingerprint, descriptor↔endpoint binding. The endpoint may need
  a tuple-return mode if it currently returns only candidate metadata; if
  full row columns are not already present, the endpoint contract gains
  a tuple-payload field.
- **Stage C** production executor: state machine, tokio-postgres transport,
  PG interrupt cancel bridge, governance, identity guards, strict/degraded
  matrix, scan handoff. The CustomScan `Exec` body invokes this executor
  directly.
- **Stage E** fault matrix (11/11) and lifecycle matrix (6/6) assertions.
  These assert against the executor state machine and the diagnostic SQL
  surfaces, not the AM cursor. Reusable as-is.
- Diagnostic SQL surfaces from packets 30702–30774 remain useful for
  operator inspection alongside the CustomScan production path.

## What is Discarded

The following work, all of which solved the AM-mirror integration, is
superseded:

- ADR-064 (materialization lifecycle).
- ADR-065 (catalog).
- ADR-066 (operator-owned mirror sync).
- Packet 30761 (row materialization contract surface).
- Packet 30762 (AM cursor wired to result stream).
- Packet 30765 (5-element mapping contract surface).
- Packet 30796 (AM tuple-path dedupe blocker coverage — Shape-A specific).
- Packet 30797 (provider seam).
- Packet 30798 (catalog table, register function, catalog diagnostic).
- Packet 30799 (AM materialized remote row end-to-end fixture).
- Packet 30801 (ADR-066 sync mechanism choice).
- In-flight mirror-sync contract surface and refresh primitive work.

The catalog table (`ec_spire_remote_row_materialization`) and register
function become vestigial. They MAY be removed in a cleanup packet after
the CustomScan path is in place, or retained briefly to support migration
from a Shape-A deployment.

The four-layer enforcement (executor classifier, AM gate, SQL contract,
AM cursor) collapses to a single rule under CustomScan: remote-origin
tuples are returned directly through the CustomScan tuple interface.
`requires_remote_row_materialization` ceases to be a possible status in
the CustomScan path; the AM-mirror status remains only on the legacy AM
path for local-only deployments.

## Consequences

- **Storage scales linearly with remote count.** Coordinator stores
  routing metadata and (optionally) its own shard; remote nodes store
  their own shards.
- **Coordinator write pressure is eliminated for remote-shard rows.**
  Writes that land on a remote shard do not propagate to the coordinator.
  See ADR-068 for placement and ADR-069 for the deferred write-path scope.
- **The AM is no longer the production distributed integration point.**
  It remains a local-only convenience: `CREATE INDEX ... USING ec_spire`
  on a non-distributed table still works.
- **The CustomScan provider is the production integration point** for
  distributed reads.
- **`EXPLAIN` output changes** for distributed queries. Distributed scans
  show `Custom Scan (EcSpireDistributedScan)` instead of `Index Scan using
  ...`. Operators and tools that depend on `Index Scan` in EXPLAIN need
  updated runbook entries.

## Rejected Alternatives

### Keep the index AM with materialization (current Shape A)

Rejected because storage does not scale out and every searchable write
costs a coordinator heap insert. Solved correctness within the AM but at
the cost of the most important architectural property of a distributed
vector search system.

### Single-virtual-table FDW

Rejected because:
- The user-visible relation must be a `FOREIGN TABLE`, which changes
  INSERT/UPDATE/DELETE semantics (write callbacks instead of normal heap
  paths). Operationally awkward for application developers used to regular
  tables.
- `EXPLAIN` shows `Foreign Scan`, which is even further from `Index Scan`
  in operator mental model than `Custom Scan` is.
- Mixing with regular B-tree or GIN indexes for non-vector quals on the
  same logical entity is harder when the relation itself is foreign.
- Vector-distance pathkey pushdown to FDW is bespoke per FDW with no
  standard protocol; no actual saving vs CustomScan path generation.

The FDW shape may still be appropriate for a future "SPIRE-as-external-
service" deployment where the coordinator role is a separate cluster.
That is out of v1 scope and a separate future ADR.

### Modify PostgreSQL to extend the index AM with remote TID support

Rejected. Out of scope for an extension.

## Open Questions

- The CustomScan implementation needs to declare path keys to enable
  ordered output without an explicit sort. Standard pattern; concrete
  declaration in the implementation packet.
- Cost estimation for the CustomScan path needs realistic numbers so the
  planner picks it over alternatives where appropriate.
- Parallel-worker integration (PostgreSQL parallel query) is left as a
  follow-up; v1 ships single-process Exec.
- Backwards compatibility for any user who has already created an
  `ec_spire` index with `boundary_replica_count > 0` and started a Shape-A
  deployment: probably a migration step that drops the catalog and
  recreates the index for CustomScan. Documented in the implementation
  packet, not this ADR.
