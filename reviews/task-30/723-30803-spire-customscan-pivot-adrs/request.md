# Review Request: SPIRE CustomScan Pivot ADRs

Reviewer-initiated planning packet proposing the architectural pivot
from index-AM-with-mirror to CustomScan-with-direct-tuple-return for
SPIRE's distributed read path, and declaring the v1 write-path scope.

This packet **supersedes ADR-064 / ADR-065 / ADR-066** and stops further
mirror-sync work pending the pivot decision.

## Scope

Three new ADRs:

- **ADR-067 SPIRE Distributed Scan via CustomScan**: replaces the
  index-AM + materialization-mirror integration with a CustomScan node
  that returns tuples directly. The four-layer enforcement (executor
  classifier, AM gate, SQL contract, AM cursor) collapses to one rule:
  remote-origin tuples are returned through the CustomScan tuple
  interface. The existing `ec_spire` index AM is retained unchanged for
  local-only deployments.
- **ADR-068 SPIRE Distributed Table Topology**: coordinator hosts
  routing centroids and placement metadata; remote nodes host shard
  rows and a local SPIRE index. Endpoint contract extends with a
  tuple-column payload so the coordinator does not need to fetch rows
  locally. Operator-facing setup sketch included.
- **ADR-069 SPIRE Distributed Write Path Scope**: coordinator-routed
  INSERT with two-phase commit atomicity is the v1 write contract.
  UPDATE/DELETE/PK-read use a new placement directory for `id →
  node_id` lookup. UPDATE of the embedding column is rejected with a
  clear error pointing applications at DELETE + INSERT. Bulk load and
  cross-shard non-vector scatter-gather are explicitly deferred to
  separate ADRs.

## Why pivot

The current Stage A path (index AM + mirror sync, ADR-064/065/066)
solves correctness inside the index AM `xs_heaptid` constraint at two
unacceptable costs:

1. **Storage does not scale out.** The coordinator must hold a mirror
   of every searchable row across all shards.
2. **Every searchable write goes through the coordinator.** Mirror
   sync replicates remote-origin rows into the coordinator's heap.

These costs are imposed by the AM integration point, not the SPIRE
algorithm. SPIRE itself returns scored tuples; the AM forces a TID
round-trip through a local heap.

CustomScan is the PostgreSQL extension point that bypasses
`xs_heaptid` and accepts tuple-direct return from a custom executor
node. The runtime-routing property of ANN search (which leaves to
visit is discovered during traversal) is compatible with CustomScan
because the routing happens inside the node's `Exec`, not at plan
time.

## What is preserved

The bulk of Stages A–E remains intact:

- **Stage A** writer identity, Leaf V2 storage, ADR-063 source identity.
- **Stage B** remote endpoint contract, fingerprint, descriptor↔endpoint
  binding. The endpoint gains a tuple-payload mode.
- **Stage C** production executor: state machine, transport,
  cancellation, governance, identity guards, strict/degraded matrix,
  scan handoff. **Reused directly** as the CustomScan `Exec` body.
- **Stage E** fault matrix (11/11) and lifecycle matrix (6/6). All
  assert against the executor state machine, not the AM cursor. Reusable.

## What is superseded

The following work is superseded:

- ADR-064, ADR-065, ADR-066.
- Packets 30761, 30762, 30765, 30796, 30797, 30798, 30799, 30801.
- The mirror-sync contract surface and refresh primitive WIP.
- The `ec_spire_remote_row_materialization` catalog table and
  `ec_spire_register_remote_row_materialization` function (may be
  retained briefly for migration, then removed).

## Operator preferences captured

The write-path ADR (ADR-069) reflects these explicit operator
directions:

- INSERT must be **coordinator-routed** with transparent distribution,
  not application-routed.
- UPDATE/DELETE must be **clean and easy** — coordinator-routed via
  placement-directory lookup.
- Embedding-UPDATE may reject in v1 (clean rejection > complex
  cross-shard move) with future ADR for atomic moves.
- Bulk load is a **separate task** with its own packets, using the
  application-routed escape hatch.

## Files

- `spec/adr/ADR-067-spire-customscan-distributed-scan.md` (new)
- `spec/adr/ADR-068-spire-distributed-table-topology.md` (new)
- `spec/adr/ADR-069-spire-distributed-write-path-scope.md` (new)
- `spec/adr/index.md` (registers new ADRs; marks 064/065/066 as
  SUPERSEDED)

## Validation

- `git diff --check`

Docs-only checkpoint. No code or SQL behavior changed.

## Reviewer Focus

- Confirm CustomScan is the right integration-point pivot vs continuing
  the AM + mirror sync path.
- Confirm the read-path topology (coordinator-as-router + remote-shard)
  matches operator intent.
- Confirm the v1 write-path scope (coordinator-routed
  INSERT/UPDATE/DELETE/PK-read with two-phase commit, embedding-UPDATE
  rejected, bulk separate) is the right shape.
- Confirm the deferred items list in ADR-069 (bulk tooling, cross-shard
  moves, scatter-gather, DDL propagation, foreign keys, sequences,
  rebalance, multi-coordinator) covers the right Phase 12+ surface.

## Next steps after acceptance

If accepted:

1. Stop further mirror-sync work; cleanup packet removes catalog table
   and register function (or retains briefly for migration).
2. Open implementation packets for:
   - CustomScan node registration and planner path generation.
   - Stage B endpoint tuple-payload extension.
   - Coordinator-routed INSERT/UPDATE/DELETE with placement directory.
   - PK-keyed read forwarding.
   - Stage E fixture updates to assert against the CustomScan path
     (most fixtures preserved; a small subset that exercised the AM
     cursor need replacement).
3. Phase 11 Stage D scope rewrites: "deliver tuples through CustomScan,
   wire executor state into CustomScan Exec" replaces "build the
   materialization mechanism."
4. Stage F and Stage G remain on the same shape.
5. The bulk-load task gets its own task number and is scheduled
   separately.
