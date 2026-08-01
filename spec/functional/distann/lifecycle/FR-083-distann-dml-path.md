---
id: FR-083
title: Distann DML Path
type: FR
status: PROPOSED
relationships:
  - target: "ix://agent-ix/ecaz/FR-077"
    type: "depends_on"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-081"
    type: "depends_on"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-082"
    type: "depends_on"
    cardinality: "N:1"
---
# FR-083: Distann DML Path

## Description

The AM SHALL support deletes via tombstones reclaimed by the next epoch build,
an interim insert posture during the read-path milestones, and — as committed
program scope — full incremental distributed insert: new vectors join the
published global graph via distributed self-insertion without a rebuild.

This requirement is explicitly scoped into two tiers, and every clause below
names its tier:

- **Tier 1 — Shipped now.** DML on the legacy single-node lane (the
  [FR-085](../FR-085-distann-domain-model.md) fixture/bootstrap substrate):
  local in-place tombstone delete, the bounded delta-buffer interim insert
  with same-statement visibility, and the fold maintenance endpoint. On the
  v5 physical/distributed-control lane, insert fails closed
  (`EC_GENERATION_MISSING`) and — as a flagged data-integrity gap —
  `ambulkdelete` currently returns noop vacuum stats, silently dropping
  deletes.
- **Tier 2 — Committed final-milestone scope (not implemented).** Routed
  tombstone writes to the hash-owning node, the stable-vec_id update path with
  atomic directory redirect, incremental distributed insert with co-placed row
  write and back-edge amendment, and the three-operation remote write
  endpoint. No Tier 2 clause is satisfied by the current implementation; they
  remain committed program scope (milestone M5), closing only with the
  implementation landed or an explicit operator descope.

## Behavior

### Tier 1 — Shipped Now

- **Local tombstone delete (legacy lane)**: on a non-distributed-control
  index, `ambulkdelete` SHALL set the tombstone flag (FR-076 flag bit) in
  place on the local graph record of every heap-dead row reported by the
  vacuum callback; tombstoned records remain traversable but are never
  returned. Tombstoning is monotone: an already-tombstoned record is never
  re-flipped. A failed tombstone flag write SHALL error (a lost tombstone
  would silently resurrect a deleted row —
  [NFR-020](../../../non-functional/NFR-020-distann-fault-behavior.md)
  covers mid-delete faults).
- Delete SHALL NOT reclaim the record's co-placed source-row-tier tuple within
  the epoch: under FR-082 immutability the tuple is retained (so a
  still-traversable tombstoned record's `exact_dist` read, if attempted, never
  faults) and is reclaimed only at the next epoch build.
- **Physical reclaim (next epoch build)**: records are never physically
  reclaimed within a Published epoch (FR-082 mutation model). The next epoch
  build SHALL drop tombstoned records, repair all adjacency referencing them,
  and re-establish the FR-077 structural invariants (degree bound, uniqueness,
  reachability).
- **v5 delete posture — flagged gap**: on a distributed-control index,
  `ambulkdelete` currently returns noop vacuum stats and performs no tombstone
  write of any kind: a DELETE on a real multi-node index silently drops the
  tombstone, the exact resurrection hazard this requirement names.

  > **Implementation gap (2026-08-01, Task 214):** this silent noop is a
  > data-integrity hazard, not accepted behavior. The normative v5 delete
  > contract is the Tier 2 routed tombstone write below; until it lands, the
  > v5 lane has no conforming delete path. Candidate code fix independent of
  > the full Tier 2 endpoint (e.g. failing closed instead of silently
  > dropping).
- **Interim insert (legacy lane)**: per ADR-085 decision D5, `aminsert` on a
  non-distributed-control index SHALL spool to a bounded exact-scan delta
  buffer whose rows are merged into results with same-statement visibility;
  the buffer SHALL be drained by the next epoch build. The shipped bound is
  4,096 buffered rows; a full buffer errors until the operator drains it with
  a rebuild. The interim posture is not a terminal state: the program closes
  only with incremental insert landed or an explicit operator descope.
- **v5 insert posture**: on a distributed-control index, `aminsert` SHALL fail
  closed with `EC_GENERATION_MISSING`; there is no distributed insert path
  until Tier 2 lands. It SHALL NOT silently drop the row.
- **Fold endpoint (legacy lane)**: the SQL maintenance endpoint
  `ec_distann_fold_delta_into_graph(index_regclass)` folds the interim delta
  buffer into the graph. It SHALL reject an isolation level other than READ
  COMMITTED before relation access or mutation and SHALL belong to the
  protected FR-079 endpoint class (`SECURITY DEFINER`, fixed trusted search
  path, no `PUBLIC` EXECUTE).

  > **Implementation gap (2026-08-01, Task 214):** the shipped endpoint
  > enforces READ COMMITTED but is not in the protected class — it lacks
  > `SECURITY DEFINER`, the pinned search path, and the `PUBLIC` revoke,
  > making it the only graph-mutating endpoint outside the hardened class.
  > The requirement stands as written. Candidate code fix.

### Tier 2 — Committed Final-Milestone Scope (Not Implemented)

None of the clauses in this subsection are implemented today. They are the
committed contract for the incremental-insert milestone (program milestone
M5), retained verbatim as obligations rather than descriptions of shipped
behavior.

- **Routed tombstone delete (v5 lane)**: `ambulkdelete` SHALL set the
  tombstone flag on the record at its hash-owned node via the remote write
  endpoint below; a failed remote tombstone write SHALL error. The origin
  node issuing the DML need not own the vec_id; the tombstone write is routed
  to the hash-owning node exactly like a record write. The Tier 1 retention
  rule (no within-epoch row-tier reclaim) applies unchanged.
- **Update**: an UPDATE of an indexed row SHALL preserve its source-derived
  vec_id.
- The update path SHALL append a complete replacement row-tier tuple and graph
  record without overwriting the old tuple or record.
- The update path SHALL atomically redirect the owner-local vec_id directory
  to the complete replacement only after all replacement writes and required
  back-edge amendments are durable.
- The update path SHALL retain the old record and row-tier tuple until the
  next epoch build.
- An in-flight scan MAY observe the old or replacement directory target under
  the FR-082 visibility rule.
- An in-flight scan SHALL NOT observe both versions as separate result rows.
- **Remote write endpoint**: data nodes SHALL expose a write counterpart to
  FR-079 (`ec_distann_apply_record_writes`: new-record append **with its
  co-placed source-row payload in the epoch row tier**, tombstone set,
  back-edge amendment with per-record `robust_prune` re-pruning executed on
  the owning node), epoch-fingerprint-validated like FR-079, with per-record
  atomicity. A new-record append SHALL co-place the record and its frozen
  source row on the same hash-owned node atomically
  ([FR-078](../build/FR-078-distann-hash-placement.md)), so FR-079 exact
  rerank of a freshly inserted vec_id always has a node-local vector and
  final tuple payload. `aminsert`/`ambulkdelete` run on the coordinator and
  drive this endpoint; degree re-pruning executes on the data node that owns
  the amended record. The endpoint SHALL reject any transaction isolation
  level other than READ COMMITTED before relation access or mutation, SHALL
  revoke `PUBLIC` execute, and SHALL use the same fixed SECURITY DEFINER
  search path as the FR-079 remote endpoint class. (A same-named endpoint
  exists today but implements only the tombstone-set operation, on legacy
  storage with the legacy 16-byte fingerprint; it does not satisfy this
  clause.)
- **Incremental distributed insert**: `aminsert` SHALL run the
  [FR-081](../read/FR-081-distann-query-orchestration.md) beam search for the
  new vector, select its edges with `robust_prune`, write the new record
  **and its co-placed frozen source-row payload** to its hash-owned node, and
  apply back-edges to affected neighbor records via the write endpoint; a
  failed insert SHALL leave the graph in its prior consistent state (no
  dangling forward edge without its record). Per-insert work SHALL be bounded
  by the FR-081 traversal cap plus at most `graph_degree` back-edge
  amendments (this is the insert-path counterpart of the
  [NFR-019](../../../non-functional/NFR-019-distann-per-query-touch-bound.md)
  scan touch bound). Back-edge re-pruning SHALL NOT drop an edge that would
  disconnect a node from the medoid: reachability (FR-077-CON-3) is preserved
  across incremental inserts, and any residual degradation is repaired at the
  next epoch build.
- **Insert-time identity collision**: when the computed vec_id already exists
  with a different `source_identity`/heap identity, `aminsert` SHALL error
  (the live-path counterpart of ADR-085 D6's build-time fail-on-collision);
  an existing record with the SAME source identity is the update path above.
  (The shipped interim insert collapses both branches — any directory hit
  errors — consistent with the update path not existing yet; the dispatch is
  a Tier 2 obligation.)

## Acceptance Criteria

Note (2026-08-01, Task 214): an earlier revision of this file carried two rows
labeled `FR-083-AC-5`. The second AC-5 (mid-insert fault) is renumbered to
AC-6, and the former AC-6/AC-7/AC-8 are now AC-7/AC-8/AC-9. Test tags citing
the old identifiers map accordingly.

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-083-AC-1 | (Tier 1, legacy lane) Deleted rows disappear from results immediately; recall on remaining rows is unaffected | Test |
| FR-083-AC-2 | (Tier 1) Epoch build drops tombstoned records, repairs all referencing adjacency, and re-establishes FR-077 invariants; expansion never encounters a reclaimed record within an epoch | Test |
| FR-083-AC-3 | (Tier 1, legacy lane) Interim delta-buffer insert gives same-statement visibility and drains at the next epoch build; v5-lane insert fails closed with `EC_GENERATION_MISSING` | Test |
| FR-083-AC-4 | (Tier 2) After incremental insert, distinct_recall@10 on queries targeting the inserted rows' neighborhoods matches a fresh rebuild containing the same rows | Test (bench A/B via `ecaz bench suite`) |
| FR-083-AC-5 | (Tier 2) The remote write endpoint rejects stronger isolation before relation access and is absent from every unprivileged remote-endpoint EXECUTE surface | Test (TC-040) |
| FR-083-AC-6 | (Tier 2) A mid-insert fault leaves no dangling forward edge (graph consistent) | Test (fault drill) |
| FR-083-AC-7 | (Tier 2) Concurrent inserts and queries interleave without wrong results | Test (concurrency drill) |
| FR-083-AC-8 | (Tier 2) After incremental insert, a query expanding and materializing the inserted vec_id reads its co-placed source row node-locally, returns a valid exact_dist, and reconstructs every requested payload column | Test (TC-043) |
| FR-083-AC-9 | (Tier 2) An UPDATE atomically redirects the stable vec_id to one complete replacement record/row, never exposes two result rows, and retains the old physical version until the next epoch build | Test (TC-043) |

## Dependencies

- **Upstream**: [FR-077](../build/FR-077-distann-sharded-build-and-stitch.md),
  [FR-082](./FR-082-distann-epoch-lifecycle.md); ADR-085 decision D5
- **Downstream**: program milestone M5 (incremental insert task)
