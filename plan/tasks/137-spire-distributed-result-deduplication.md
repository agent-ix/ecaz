# Task 137: SPIRE Distributed Result Deduplication

Status: review requested (2026-07-02; fix + full 10k/50k/100k A/B evidence in
`reviews/task-137/001-source-identity-dedupe-ab/`, decision per ADR-083;
originally filed from Task 131 packet 027 identity
artifacts and reviewer feedback).
Numbering note: originally filed as task 132 on 2026-07-02; renumbered to 137
the same day because the TQ optimization lane had already claimed 132-136
(`132-tq-scorer-lut-dimension-tiling` and successors, filed 2026-07-01).
Owner: coder (to be assigned). One coder, one branch.
Priority: P0 correctness/result-quality follow-up for SPIRE distributed reads.

## Why

Task 131 added per-query returned-ID identity artifacts for local multi-instance
SPIRE production reads. Those artifacts exposed a separate result-quality defect:
the distributed read surface can return the same corpus row multiple times
inside one top-k result.

This is not a Task 131 optimization problem. It is a correctness and metric
quality problem for distributed SPIRE reads:

- users requesting `k=10` can receive fewer than 10 distinct corpus rows;
- recall@k can be overstated if duplicate returned IDs are counted repeatedly;
- inter-arm identity comparisons remain useful for A/B no-op detection, but
  "matched recall" on affected packets must be read as matched under duplicate-
  tolerant current metrics.

## Evidence

Primary evidence is packet-local under
`reviews/task-131/027-phase3-increment-a-ab/`:

- 10k identity file:
  `artifacts/10k-n128-b4/bench-suite/production-read-k10-threshold-off-default-identity.jsonl`
  - 183 of 200 queries contain duplicate IDs in returned top-10.
  - Worst case: 6 duplicate positions, leaving only 4 distinct IDs.
  - Example query 1:
    `[9897,9897,9897,9786,9786,9786,9580,9580,9580,9477]`.
- 50k identity file:
  `artifacts/50k-n128-b4/bench-suite/production-read-k10-threshold-off-default-identity.jsonl`
  - 1000 of 1000 queries contain duplicate IDs in returned top-10.
  - Worst case: 6 duplicate positions, leaving only 4 distinct IDs.
  - Example query 1:
    `[38468,38468,38468,43507,43507,43507,49535,49535,49537,49537]`.
- The off/on identity files are byte-identical in packet 027, so this defect is
  pre-existing/shared by both A/B arms and does not change Task 131's
  threshold-gate rejection.

Reviewer feedback:

- `reviews/task-131/027-phase3-increment-a-ab/feedback/2026-07-02-01-reviewer.md`

## Suspected Mechanism To Confirm

The Task 131 local multi-instance fixture uses `boundary_replica_count=4` across
three remote nodes. If replica copies of the same corpus row carry
node-local/list-local `vec_id` values, a merge that deduplicates by `vec_id`
rather than global corpus identity can emit multiple copies of the same heap row.

Confirm this before fixing it. The task should inspect the final distributed
merge keys, remote identity mapping, and recall metric implementation rather
than assuming boundary replication is the only source.

## Scope

1. Reproduce the duplicate returned-ID behavior on a narrow local multi-instance
   SPIRE fixture with packet-local identity artifacts.
2. Identify the correct global row identity for final-result deduplication
   across replicas and remote nodes.
3. Fix the distributed final top-k merge so one corpus row can appear at most
   once in a returned result, unless a query explicitly asks for replica-level
   internals.
4. Fix or harden recall/result-identity metrics so duplicate returned IDs cannot
   flatter recall@k.
5. Preserve strict/degraded read semantics, tie behavior, and matched-recall
   latency/storage evidence for the corrected surface.

## Required Evidence

- A failing pre-fix identity/metric artifact showing duplicate returned IDs.
- Unit or integration coverage for final merge deduplication by global corpus
  identity, including boundary-replica cases.
- A post-fix local multi-instance packet showing:
  - no duplicate returned IDs within each top-k result;
  - recall@10 recomputed with distinct returned IDs;
  - latency p50/p95/p99 and storage unchanged or explicitly accounted for;
  - strict/degraded/fault behavior preserved for the touched path.
- A note in affected benchmark readouts explaining that older Task 123/131
  distributed recall numbers predate this fix and may be duplicate-tolerant.

## Non-Goals

- Do not reopen Task 131 streaming-threshold optimization inside this task.
- Do not change the placement or boundary-replication policy unless required to
  produce correct distinct final results.
- Do not treat lower duplicate-tolerant recall as a regression if it simply
  exposes the prior metric bug.

## Acceptance Criteria

1. Distributed SPIRE top-k results are distinct by corpus row identity.
2. Recall/result-identity metrics count distinct returned corpus rows and expose
   duplicate-result regressions.
3. Boundary-replica duplicates are covered by tests or packet-local fixtures.
4. A local multi-instance benchmark packet proves corrected behavior and reports
   latency/recall/storage at the touched scale.
