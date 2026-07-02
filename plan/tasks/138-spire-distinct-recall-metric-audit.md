# Task 138: SPIRE Distinct-Recall Metric And Historical Evidence Audit

Status: review requested (2026-07-02; metric + full audit evidence in
`reviews/task-138/001-distinct-recall-rescore/`; originally filed from the
Task 131 closeout research
synthesis and the packet 027 duplicate-ID finding).
Owner: coder (to be assigned). One coder, one branch.
Priority: P0 evidence-integrity follow-up; gates Task 139 interpretation.

## Why

Task 131 packet 027's identity artifacts proved the distributed SPIRE read
surface returns duplicate corpus IDs inside a single top-k result (183/200
queries at 10k, 1000/1000 at 50k; worst case 4 distinct IDs in a k=10 result),
while the current recall@k metric still reports 0.9985-1.0000. The metric is
therefore duplicate-tolerant: it counts repeated hits of the same truth ID.

This contaminates conclusions, not just numbers. Boundary replication
(`boundary_replica_count`) — which Task 121's DOE crowned the primary
route-recovery lever — is precisely the mechanism that produces duplicate
results. Some unknown fraction of b4's measured recall lift may be duplicate
inflation rather than real neighbors recovered. Every "matched recall" claim
on the multi-instance surface (Tasks 121, 123, 131) is qualified until
re-scored.

Task 137 owns fixing the dedupe defect in the read path. This task owns the
metric and the retroactive evidence audit.

## Goal

Define and implement a distinct-neighbor recall metric, re-score existing
artifacts where returned-ID evidence exists, and publish a corrected
evidence table stating explicitly which prior conclusions survive, weaken, or
flip.

## Scope

### Phase 0 - Metric Definition And Runner Support

- Define `distinct_recall@k`: |distinct(returned) ∩ truth top-k| / k. Duplicate
  returned IDs count once. Document the definition next to the existing metric.
- Extend the `ecaz bench suite` runner / spire-pipeline query metrics (FR-038
  rule: extend the runner, never fork a script) to emit `distinct_recall@k`
  and `distinct_returned_count` alongside the current recall on every
  recall-bearing step.
- Keep the current metric emitted unchanged for comparability; never edit
  historical artifacts.

### Phase 1 - Re-Scorable Artifact Inventory

- Inventory which historical packets contain per-query returned-ID evidence
  (identity JSONL exists only from Task 131 packet 027 onward; earlier packets
  may be re-scorable only by re-running).
- Publish the inventory in the packet manifest: re-scorable as-is, re-runnable
  cheaply, or unrecoverable.

### Phase 2 - Retroactive Re-Score And Corrected Table

- Re-score packet 027's 10k/50k identity artifacts with `distinct_recall@k`.
- Re-run the cheapest representative cells needed to compare shapes where no
  ID artifacts exist: minimum 10k/50k for `n128/b4` and `n1024/b2` on the
  multi-instance lane, single-instance spot checks if needed for attribution.
- Publish a corrected recall table: current metric vs distinct metric, per
  shape, per scale.

### Phase 3 - Conclusion Re-Ranking

For each prior conclusion, state survive / weaken / flip with numbers:

- Task 121: "boundary_replica_count is the primary route-recovery lever."
- Task 123/131 baselines: "recall 1.0 at nprobe=96 n128/b4" and
  "recall ~1.0 at nprobe=64 n1024/b2".
- Task 131 packet 027: inter-arm "matched recall" (expected to survive —
  identity was byte-identical — but state it).

## Required Evidence

- `ecaz bench suite` for any re-runs; per-query ID JSONL artifacts packet-local.
- The corrected table and the survive/weaken/flip readout in the packet.
- No fabricated numbers; unrecoverable history stays labeled unrecoverable.

## Non-Goals

- Do not fix the dedupe defect (Task 137 owns the read path).
- Do not tune routing configs (Task 139 owns the Pareto).
- Do not edit or re-write historical packet artifacts.

## Acceptance Criteria

1. `distinct_recall@k` emitted by the standard runner alongside current recall.
2. Re-scorable inventory published.
3. Corrected recall table for at least 10k/50k × {n128/b4, n1024/b2}.
4. Explicit survive/weaken/flip statement for the Task 121 boundary-replica
   conclusion and the Task 123/131 matched-recall baselines.

## References

- `plan/tasks/137-spire-distributed-result-deduplication.md`
- `reviews/task-131/027-phase3-increment-a-ab/` (identity JSONL evidence)
- `reviews/task-131/027-phase3-increment-a-ab/feedback/2026-07-02-01-reviewer.md`
- `plan/tasks/121-spire-coarse-routing-recall-doe.md`
- `spec/non-functional/NFR-007-benchmark-provenance.md`
