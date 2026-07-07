# Task 167: ec_distann M5 — Incremental Distributed Insert (Committed Scope)

Status: proposed (2026-07-06). Depends on: Task 166 (gate verdict; committed
scope regardless of verdict unless the operator explicitly descopes — ADR-085
D5).
Owner: coder (to be assigned). One coder, one branch.
Priority: P1 — the program's committed write path (operator decision
2026-07-06: in scope, not conditional).

## Why

FR-083 full: new vectors join the published global graph without a rebuild.
This is the hardest consistency work in the program and is deliberately
last, where its failure cannot invalidate the read path.

## Goal

`aminsert` performs distributed self-insertion with the graph left
consistent under any single fault, at insert cost bounded by the traversal
cap + `graph_degree` back-edge amendments.

## Scope

- FR-083 final slice: beam-search placement (FR-081 loop), `robust_prune`
  edge selection, new-record append to the hash-owned node, batched
  back-edge amendments with on-node degree re-pruning via
  `ec_distann_apply_record_writes` (per-record atomicity).
- Insert-time vec_id collision → error (live-path D6); UPDATE =
  tombstone-then-insert under the same vec_id.
- Visibility per FR-082 D10 concurrent-mutation rules; failed insert leaves
  no dangling forward edge.
- Fixture: mid-insert fault drills + concurrent insert/query drills
  (TC-043); reachability-over-churn check at the next epoch build.

## Required Evidence

TC-043 green; FR-083-AC-4 bench cell (insert-then-query distinct_recall
parity vs fresh rebuild, `ecaz bench suite`, release, on the 166 protocol);
insert throughput A/B recorded; NFR-020 mid-insert drill logs.

## Non-Goals

Baton passing; roster changes mid-epoch; background insert queues.

## Acceptance Criteria

1. All FR-083 ACs green including AC-4 (bench) and AC-5/6 (fault +
   concurrency).
2. Per-insert work bound demonstrated by counter evidence.
3. Program closeout note: FR-083 committed scope delivered (or explicit
   operator descope recorded in ADR-085).

## References

- FR-083, FR-082 (D10), NFR-020; ADR-085 D5/D6
- `plan/design/distann-global-graph-architecture.md` (M5)
