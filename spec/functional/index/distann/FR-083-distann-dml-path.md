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

The AM SHALL support deletes via tombstones reclaimed by the next epoch build, an interim
insert posture during the read-path milestones, and — as committed program
scope — full incremental distributed insert: new vectors join the published
global graph via distributed self-insertion without a rebuild.

## Behavior

Milestone slicing: the delete/tombstone and interim-insert behaviors below
are early-milestone prerequisites (they land with the read-path milestones);
only incremental distributed insert is the final milestone.

- **Delete**: `ambulkdelete` SHALL set the tombstone flag (FR-076 flag bit)
  on the record at its hash-owned node via the remote write endpoint below;
  tombstoned records remain traversable but are never returned. A failed
  remote tombstone write SHALL error (a lost tombstone would silently
  resurrect a deleted row —
  [NFR-020](../../../non-functional/NFR-020-distann-fault-behavior.md)
  covers mid-delete faults). Delete SHALL NOT reclaim the record's co-placed
  source-row-tier tuple within the epoch: under FR-082 immutability the tuple is
  retained (so a still-traversable tombstoned record's `exact_dist` read, if
  attempted, never faults) and is reclaimed only at the next epoch build. The
  origin node issuing the DML need not own the vec_id; the tombstone write is
  routed to the hash-owning node exactly like a record write.
- **Physical reclaim**: records are never physically reclaimed within a
  Published epoch (FR-082 mutation model). The next epoch build SHALL drop
  tombstoned records, repair all adjacency referencing them, and
  re-establish the FR-077 structural invariants (degree bound, uniqueness,
  reachability).
- **Update**: an UPDATE of an indexed row SHALL preserve its source-derived
  vec_id.
- The update path SHALL append a complete replacement row-tier tuple and graph
  record without overwriting the old tuple or record.
- The update path SHALL atomically redirect the owner-local vec_id directory to
  the complete replacement only after all replacement writes and required
  back-edge amendments are durable.
- The update path SHALL retain the old record and row-tier tuple until the next
  epoch build.
- An in-flight scan MAY observe the old or replacement directory target under
  the FR-082 visibility rule.
- An in-flight scan SHALL NOT observe both versions as separate result rows.
- **Interim insert (early milestones)**: per ADR-085 decision D5,
  `aminsert` SHALL spool to a bounded exact-scan delta buffer whose rows are
  merged into results with same-statement visibility; the buffer SHALL be
  drained by the next epoch build. The interim posture is not a terminal
  state: the program closes only with incremental insert landed or an
  explicit operator descope.
- **Remote write endpoint**: data nodes SHALL expose a write counterpart to
  FR-079 (`ec_distann_apply_record_writes`: new-record append **with its
  co-placed source-row payload in the epoch row tier**, tombstone set, back-edge
  amendment with per-record `robust_prune` re-pruning executed on the owning
  node), epoch-fingerprint-validated like FR-079, with per-record atomicity.
  A new-record append SHALL co-place the record and its frozen source row on the same
  hash-owned node atomically ([FR-078](./FR-078-distann-hash-placement.md)),
  so FR-079 exact rerank of a freshly inserted vec_id always has a
  node-local vector and final tuple payload. `aminsert`/`ambulkdelete` run on the
  coordinator and drive this endpoint; degree re-pruning executes on the
  data node that owns the amended record.
- **Incremental distributed insert (committed scope, final milestone)**:
  `aminsert` SHALL run the [FR-081](./FR-081-distann-query-orchestration.md)
  beam search for the new vector, select its edges with `robust_prune`,
  write the new record **and its co-placed frozen source-row payload**
  to its hash-owned node, and apply back-edges to
  affected neighbor records via the write endpoint; a failed insert SHALL
  leave the graph in its prior consistent state (no dangling forward edge
  without its record). Per-insert work SHALL be bounded by the FR-081
  traversal cap plus at most `graph_degree` back-edge amendments (this is the
  insert-path counterpart of the [NFR-019](../../../non-functional/NFR-019-distann-per-query-touch-bound.md)
  scan touch bound). Back-edge re-pruning SHALL NOT drop an edge that would
  disconnect a node from the medoid: reachability (FR-077-CON-3) is preserved
  across incremental inserts, and any residual degradation is repaired at the
  next epoch build.
- **Insert-time identity collision**: when the computed vec_id already
  exists with a different `source_identity`/heap identity, `aminsert` SHALL
  error (the live-path counterpart of ADR-085 D6's build-time
  fail-on-collision); an existing record with the SAME source identity is
  the update path above.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-083-AC-1 | Deleted rows disappear from results immediately; recall on remaining rows is unaffected | Test |
| FR-083-AC-2 | Epoch build drops tombstoned records, repairs all referencing adjacency, and re-establishes FR-077 invariants; expansion never encounters a reclaimed record within an epoch | Test |
| FR-083-AC-3 | Interim delta-buffer insert gives same-statement visibility and drains at the next epoch build | Test |
| FR-083-AC-4 | After incremental insert, distinct_recall@10 on queries targeting the inserted rows' neighborhoods matches a fresh rebuild containing the same rows | Test (bench A/B via `ecaz bench suite`) |
| FR-083-AC-5 | A mid-insert fault leaves no dangling forward edge (graph consistent) | Test (fault drill) |
| FR-083-AC-6 | Concurrent inserts and queries interleave without wrong results | Test (concurrency drill) |
| FR-083-AC-7 | After incremental insert, a query expanding and materializing the inserted vec_id reads its co-placed source row node-locally, returns a valid exact_dist, and reconstructs every requested payload column | Test (TC-043) |
| FR-083-AC-8 | An UPDATE atomically redirects the stable vec_id to one complete replacement record/row, never exposes two result rows, and retains the old physical version until the next epoch build | Test (TC-043) |

## Dependencies

- **Upstream**: [FR-077](./FR-077-distann-sharded-build-and-stitch.md),
  [FR-082](./FR-082-distann-epoch-lifecycle.md); ADR-085 decision D5
- **Downstream**: program milestone M5 (incremental insert task)
