# Task 189: ec_distann Hybrid Distance and Codec Evaluation

Status: **proposed, conditionally dormant** (2026-07-19). Priority: P3.

Program ledger: `plan/design/ec-distann-recall-latency-roadmap.md`, candidate
family `CODEC-01` through `CODEC-13`, excluding head-only `CODEC-08` when owned
by Task 186.

## Why

Task 183's byte-identical-seed experiment found that replacing RaBitQ neighbor
scoring with full exact-neighbor traversal reduced recall from 0.9625 to 0.9605
and raised p50 from 43.8 to 113.1 ms. A broad exact-vector or codec rewrite is
therefore not justified now. A later hybrid may still be useful if Tasks
185--188 isolate ambiguous frontier comparisons or a residual that tracks
neighbor-estimation error.

## Goal

Only after a same-seed trigger, compare a small number of hybrid distance or
codec candidates and advance at most one. Preserve graph, seeds, BW/H, final
ranking, topology, and failure behavior during attribution.

## Entry gate

Tasks 185--188 must produce query-level evidence that:

- correct candidates are present/reachable but approximate ordering loses
  them; or
- a bounded subset of frontier comparisons has an actionable RaBitQ error
  margin.

Absent that evidence, close with a conditional skip. Do not repeat the
unchanged full exact-neighbor arm.

## Candidate screen

Pre-register no more than three same-seed arms:

1. selective exact correction for ambiguous or final frontier comparisons
   (`CODEC-03`, `CODEC-04`);
2. one richer RaBitQ representation or residual (`CODEC-01`, `CODEC-02`, or
   `CODEC-10`); and
3. at most one structurally different codec (`CODEC-05`, `CODEC-07`,
   `CODEC-09`, or `CODEC-11`) only if the first two establish a quality signal.

Every arm must fail closed on seed-digest mismatch and report code bytes,
record/graph bytes, decode/scoring time, exact reads, build cost, and query work.
OPQ/PQ is an experiment, not presumed paper-parity or a default.

## Confirmation and decision

Screen at 100k through checked-in `ecaz bench suite`. Only a useful isolated
candidate proceeds to 10k/50k/100k. Advance at most one candidate with a clear
relative recall/latency/storage win and no unresolved bit width, transform,
fallback, format, or upgrade choice. Otherwise STOP.

A winner changing the node record or codec requires a separate production
task, clean versioning/rebuild contract, ADR/spec changes, DML/vacuum parity,
and full lifecycle evidence.

## Required review packets

1. `reviews/task-189/001-entry-trigger/`;
2. `reviews/task-189/002-same-seed-screen/`;
3. `reviews/task-189/003-isolated-candidate/`;
4. `reviews/task-189/004-full-scale-decision/`.

## Non-goals

- Unchanged full exact-neighbor traversal.
- Head-only compression/routing owned by Task 186.
- Graph/search changes owned by Task 188.
- Multiple codec changes stacked with graph/head changes.
- Production format/default changes.

## References

- Task 183 packet 002.
- FR-076, FR-078, ADR-085 D7.
- `plan/design/ec-distann-recall-latency-roadmap.md`.
