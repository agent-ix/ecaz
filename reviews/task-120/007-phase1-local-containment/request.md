# Review Request: Task 120 Phase 1 Local Containment

- task: Task 120
- packet: `reviews/task-120/007-phase1-local-containment`
- measured head SHA: `e33ac2ae928425231d2ec4907452d613e005ebbd`
- code change under review: none in this packet; this is Phase 1 measurement evidence

## Summary

This packet runs the local Phase 1 containment matrix for SPIRE on staged real
corpora at `10k`, `50k`, and `100k` with `nprobe=8,16,24,32`.

The suite used `ecaz bench suite` and captured load, recall, latency, storage,
candidate-funnel, stage-containment, target-block-rank, and target-candidate-rank
artifacts under `artifacts/`.

Final suite status:

```text
completed=16 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

## Result

Recall improves with `nprobe`, but brute route fanout is not close to enough at
100k:

```text
100k nprobe=8   recall@10=0.7695  p95=139.9 ms  avg_candidates=2484.15
100k nprobe=16  recall@10=0.8495  p95=266.2 ms  avg_candidates=5076.27
100k nprobe=24  recall@10=0.8940  p95=378.6 ms  avg_candidates=7620.48
100k nprobe=32  recall@10=0.9205  p95=495.4 ms  avg_candidates=10237
```

The local candidate frontier containment exactly matches final recall for every
scale and `nprobe`. The exact/source rerank frontier does not introduce another
loss stage, and it does not recover rows absent from the candidate frontier.

Target-candidate rank confirms the same pattern. For retained truth rows,
`selected_by_prefix=100%` and p95 approximate rank is at most `10`; missed rows
are `candidate_not_retained`.

The target-block-rank snapshot is not decision-grade in this local packet:
every scale/`nprobe` reports all 2,000 truth rows as
`not_found_in_routed_leaves` with zero scored blocks. That conflicts with the
candidate-rank snapshot proving retained truth rows do enter the candidate
frontier, so this packet should not be used for route-vs-block attribution.

## Recommendation

Proceed to Phase 2, but frame it as a candidate-frontier experiment: either
recover missing truth rows before the final frontier or build the same frontier
more cheaply. A post-frontier exact/source rerank policy alone is not supported
by this evidence.

Before Phase 4 route-set refinement, the target-block-rank attribution needs a
fix or replacement diagnostic and a rerun, because this local packet does not
produce reliable route/block containment attribution.

## Evidence

See `artifacts/manifest.md` for commands, provenance, and key result lines.

Primary summary artifacts:

- `artifacts/measurement-summary.txt`
- `artifacts/pipeline-stage-containment-summary.txt`
- `artifacts/pipeline-target-candidate-rank-summary.txt`
- `artifacts/pipeline-target-block-rank-summary.txt`
- `artifacts/pipeline-funnel-summary.txt`
- `artifacts/suite-report.md`
- `artifacts/suite-results.jsonl`
- `artifacts/suite-manifest.json`

## Closeout Status

This satisfies the local Phase 1 diagnostic packet only. Task 120 remains open
for Phases 2 through 6, and AWS 1M evidence is still required before any SPIRE
product-default or product-claim decision.
