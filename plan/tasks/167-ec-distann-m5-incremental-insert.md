# Task 167: ec_distann M5 — Incremental Distributed Insert (Committed Scope)

Status: implementation and required 10k/50k/100k evidence complete; packets
040–043 review-open on PR 77; outside reviewer disposition pending, with no
merge or closeout claim. Packet 041 replaced the defective pairwise ANN-overlap
gate; packet 042 fixed incremental pruning to use the batch path's nonnegative
distance; packet 043 completed exact fp32 physical-vs-fresh measurements with
distinct-key denominators, a 48/152 inserted/heldout split, matched fresh
reloptions, and no unsupported hard process gate. Inserted deltas are
`-0.014385 / +0.011285 / -0.003472` and heldout deltas are
`-0.003289 / -0.025987 / -0.005921` at 10k/50k/100k. The real same-fixture
append-enabled/disabled throughput ratios are
`0.975741 / 0.997529 / 0.993053` (`pass=false` at every scale). Concurrency,
routed delete/vacuum, UPDATE, rollback, owner placement, storage, and topology
evidence landed. Packets 027–030 are explicitly superseded by this round.
PR: `https://github.com/agent-ix/ecaz/pull/77`.
REQUESTED on 2026-08-12; packet 025 claims were superseded by reviewer feedback
and are not acceptance evidence.
Depends on: Task 166 and Task 179's Published-generation storage/read contract
(gate verdict remains committed scope unless the operator explicitly descopes
— ADR-085 D5). The landed delta/fold experiments apply to the legacy local or
replicated surface; they do not yet implement FR-083 against disjoint owner row
tiers and retained replacement records.
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

## Corrective boundary (2026-07-10)

After Task 179 lands, reopen this task for physical owner routing: inserts must
append a complete row-tier tuple and record on one owner, UPDATE must atomically
redirect the stable vec_id while retaining the old physical pair, and all
materialization must use the generation schema descriptor. Task 179 must fail
closed on distributed-control DML until that adaptation is reviewed; it must
not silently invoke the legacy local-heap path.

## Scope

- FR-083 final slice: beam-search placement (FR-081 loop), `robust_prune`
  edge selection, new-record append to the hash-owned node, batched
  back-edge amendments with on-node degree re-pruning via the physical remote
  write endpoint family (`ec_distann_apply_physical_insert`,
  `ec_distann_apply_physical_backlink`, and
  `ec_distann_apply_physical_tombstone`) (per-record atomicity).
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
