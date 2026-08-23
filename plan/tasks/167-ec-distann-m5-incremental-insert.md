# Task 167: ec_distann M5 — Incremental Distributed Insert (Committed Scope)

Status: implementation open; packet 043 reviewer findings are addressed by
packets 044–049, but all four measured isolated 50k backlink candidates fail
the fixed heldout quality gate. Packet 047's retained robust-prune deficit was
`0.008611`, missing the `0.007000` band by `0.001611`; packet 051's
append-when-room deficit was `0.010611` (miss `0.003611`); and packets 054 and
057 each measured `0.009611` (miss `0.002611`) for conservative admission and
the full-target pruned-backlink no-op respectively. Packet 057 recorded 702
full-target prune rejections, while its inserted-neighborhood deficit passed
at `0.008970`; the dominant heldout result nevertheless rejects the candidate.
Packet 058 restores the retained robust-prune product/harness state and is
review-open. The threshold is unchanged; no merge, final scale matrix, or
closeout is claimed. Required follow-ups are an outside-review verdict and
diagnosis of a materially different isolated candidate; only a clean 50k pass
permits isolated 10k/50k/100k recall, latency, and storage confirmation.
Evidence: packets 047, 051, 054–058; reviewer source:
`reviews/task-167/043-exact-recall-disposition/feedback/2026-08-22-01-reviewer.md`.
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
