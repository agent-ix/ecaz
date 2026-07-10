# Task 163: ec_distann M1 — Sharded Build + Stitch Correctness

Status: partial / open (2026-07-10). The stitched-graph quality, determinism,
and property work landed, but ADR-085 D8 / FR-077-CON-4 remains open: the
implementation retains every shard output in memory instead of spilling sorted
streams and merging from bounded cursors. Depends on: Task 161; Task 162
(record format). The D8 closeout is a hard prerequisite of Task 179's physical
owner handoff.
Owner: coder (to be assigned). One coder, one branch.
Priority: P0 — the paper's least-proven step and the program's highest
technical risk (spec-review SR-006 FND-001).

## Why

FR-077: one coherent global graph from parallel shard builds. If stitch
quality fails, the fallback is the monolithic build (degrades build
parallelism, not the program) — but that must be proven, not assumed.

## Goal

Stitched 100k build within 0.001 distinct_recall of a monolithic build at
equal search parameters, with all structural invariants property-tested.

## Corrective closeout note (2026-07-10)

Packets 001–002 establish graph quality and honestly report retained shard
output memory. They do not close this task. The remaining implementation must
write each sorted shard output to PostgreSQL-managed temporary spill storage,
k-way merge through bounded cursors, and prove peak stitch memory is one vec_id
group plus prune scratch. That work stays in Task 163 and its next packet; Task
179 consumes the resulting stream but does not redefine D8.

## Scope

- Closure-overlap shard assignment: repurpose the distance-ratio machinery
  (`src/am/ec_spire/build/routing_plan.rs:132-250`, `closure_epsilon`).
- Per-shard Vamana builds (parallel where cheap; sequential acceptable v1).
- Stitch pass: group by vec_id (shard outputs sorted; stream per ADR-085
  D8), union edge lists, `robust_prune` re-prune, emit one record each.
- Seed determinism end-to-end (FR-077; enables FR-081-AC-1 later).
- Build-time vec_id collision detection (D6: fail the build).
- proptest suite in `src/am/ec_distann/` (TC-038): degree ≤ R, uniqueness,
  medoid reachability, idempotence, α-prune invariant.
- Epoch-manifest rows: duplication factor, stitch stats, peak stitch memory.

## Required Evidence

TC-038 green; TC-039 bench A/B (stitched vs monolithic, 100k, equal params,
release build); build-time + duplication-factor + peak-memory rows.

## Non-Goals

Placement/remote anything (164). Head-index changes beyond consuming
multi-shard entry samples.

## Acceptance Criteria

1. All FR-077 ACs/CONs verifiable green (property tests + bench A/B).
2. Determinism: identical corpus+seed+options → identical stitched graph.
3. Fallback documented: monolithic path still selectable by option.

## References

- FR-077; ADR-085 D6/D8; `plan/design/distann-global-graph-architecture.md` (M1)
