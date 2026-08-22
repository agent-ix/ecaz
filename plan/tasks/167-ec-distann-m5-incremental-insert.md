# Task 167: ec_distann M5 — Incremental Distributed Insert (Committed Scope)

Status: implementation/evidence checkpoints through `6968f0a3d`; packets 031
through 039 review-open on PR 77; outside reviewer disposition pending. The
clean production PG18 synthetic gate passed at packet 036, including rollback,
replacement, saturation, natural-retry, backlink, routed-delete, and topology
coverage. Packet 037's corrected production 10k measurement completed recall,
latency, storage, insert A/B, insert-work, concurrency, and delete gates, but
failed required post-insert physical-vs-fresh distinct-recall parity:
append-disabled `0.541667`, append-enabled `0.541667`, required `0.80`.
Checkpoint `6968f0a3d` makes that existing threshold fail the suite process;
50k/100k were stopped at the failed 10k gate. No merge or closeout until the
outside reviewer dispositions packets 037 through 039. PR:
`https://github.com/agent-ix/ecaz/pull/77`.
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
