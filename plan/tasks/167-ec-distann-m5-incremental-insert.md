# Task 167: ec_distann M5 — Incremental Distributed Insert (Committed Scope)

Status: implementation checkpoint `79afb0d82`; packets 026 and 027 review-open;
outside reviewer disposition pending; both owner retry paths now reopen graph
and directory guards per attempt, but current-head production benchmark
closeout is not complete. Existing 10k diagnostic evidence fails
inserted-neighborhood parity and the real saturated-target gate; the 50k/100k
rerun is blocked by the host read-only filesystem. No merge or closeout until
current-head production 10k/50k/100k evidence and reviewer ACCEPT.
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
