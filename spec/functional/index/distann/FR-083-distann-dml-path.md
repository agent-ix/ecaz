---
id: FR-083
title: Distann DML Path
type: FR
status: PROPOSED
relationships:
  - target: "ix://agent-ix/ecaz/FR-077"
    type: "depends_on"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-082"
    type: "depends_on"
    cardinality: "N:1"
---
# FR-083: Distann DML Path

## Description

The AM SHALL support deletes via tombstones reclaimed by vacuum, an interim
insert posture during the read-path milestones, and — as committed program
scope — full incremental distributed insert: new vectors join the published
global graph via distributed self-insertion without a rebuild.

## Behavior

- **Delete**: `ambulkdelete` SHALL tombstone the record (FR-076 flag bit);
  tombstoned records remain traversable but are never returned; vacuum SHALL
  reclaim tombstones and repair adjacency referencing them (edges to
  reclaimed records are dropped at expansion time until repaired).
- **Interim insert (read-path milestones only)**: per ADR-085 decision D5,
  `aminsert` SHALL either error with a documented rebuild instruction or
  spool to a bounded exact-scan delta buffer merged into results; the chosen
  posture SHALL be a documented reloption default, and the buffer (if
  chosen) SHALL be drained by the next epoch build.
- **Incremental distributed insert (committed scope, final milestone)**:
  `aminsert` SHALL run the FR-081 beam search for the new vector, select its
  edges with `robust_prune`, write the new record to its hash-owned node,
  and apply back-edges to affected neighbor records via batched remote
  read-modify-write with per-record degree re-pruning; a failed insert SHALL
  leave the graph in its prior consistent state (no dangling forward edge
  without its record).
- Inserted vectors SHALL be visible to queries under the epoch/visibility
  semantics fixed in ADR-085 decision D5 (same-epoch delta visibility vs
  next-epoch visibility).

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-083-AC-1 | Deleted rows disappear from results immediately; recall on remaining rows is unaffected | Test |
| FR-083-AC-2 | Vacuum reclaims tombstones and no expansion ever errors on a reclaimed neighbor | Test |
| FR-083-AC-3 | Interim posture behaves exactly as documented (error, or delta rows present in results) | Test |
| FR-083-AC-4 | After incremental insert, querying the inserted vector's neighborhood reaches recall parity with a fresh rebuild containing the same rows | Test (bench A/B) |
| FR-083-AC-5 | A mid-insert fault leaves no dangling forward edge (graph consistent) | Test (fault drill) |
| FR-083-AC-6 | Concurrent inserts and queries interleave without wrong results | Test (concurrency drill) |

## Dependencies

- **Upstream**: [FR-077](./FR-077-distann-sharded-build-and-stitch.md),
  [FR-082](./FR-082-distann-epoch-lifecycle.md); ADR-085 decision D5
- **Downstream**: program milestone M5 (incremental insert task)
