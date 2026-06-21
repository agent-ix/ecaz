# Task 120 Phase 3 Budget/Policy Curves

Please review the local candidate-budget and rerank-width policy evidence for
Task 120 Phase 3.

## Scope

This packet reuses the Phase 2 recursive RabitQ indexes and runs a local
`bench spire-pipeline` policy matrix at 10k / 50k / 100k. It tests whether
candidate caps or wider exact rerank widths close the remaining local SPIRE
recall gap before moving to topology refinement.

No source code changed for this packet.

## Evidence

- Artifact manifest:
  `reviews/task-120/010-phase3-budget-policy/artifacts/manifest.md`
- Compact summary:
  `reviews/task-120/010-phase3-budget-policy/artifacts/phase3-budget-policy-summary.txt`
- Suite config:
  `reviews/task-120/010-phase3-budget-policy/artifacts/suite.json`
- Suite status:
  `reviews/task-120/010-phase3-budget-policy/artifacts/suite-status.log`
  reports `completed=22 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`
- Structured results:
  - `reviews/task-120/010-phase3-budget-policy/artifacts/suite-results.jsonl`
  - `reviews/task-120/010-phase3-budget-policy/artifacts/suite-report-results.jsonl`
- Host precheck:
  `reviews/task-120/010-phase3-budget-policy/artifacts/precheck-host.log`
  reports PostgreSQL `18.3` and `ecaz_build_profile=release`

## Matrix

- Scales: 10k, 50k, 100k
- Queries: 200 per scale
- Sweep: `nprobe=8,16,24,32`
- Variants per scale:
  - default
  - `max_candidate_rows=10000`
  - `max_candidate_rows=25000`
  - `max_candidate_rows=0, rerank_width=25`
  - `max_candidate_rows=0, rerank_width=100`
  - `max_candidate_rows=0, rerank_width=500`

## Result

Cap-only variants are recall-neutral because the default `rerank_width=25`
still limits exact heap rerank to 5,000 rows total for 200 queries. Explicit
width variants raise exact rerank volume to 20,000 or 100,000 rows, but recall
does not improve at any measured scale or nprobe.

At `nprobe=32`:

| Scale | Variant | recall@10 | p50 | p95 | heap_rerank_sum |
| --- | --- | ---: | ---: | ---: | ---: |
| 10k | default | 0.9965 | 8.668 ms | 9.756 ms | 5,000 |
| 10k | width 500 | 0.9965 | 24.260 ms | 26.335 ms | 100,000 |
| 50k | default | 0.9725 | 14.869 ms | 17.703 ms | 5,000 |
| 50k | width 500 | 0.9725 | 31.612 ms | 34.350 ms | 100,000 |
| 100k | default | 0.9310 | 25.396 ms | 27.574 ms | 5,000 |
| 100k | width 500 | 0.9310 | 44.393 ms | 62.181 ms | 100,000 |

The decision from this packet is **no product default** for wider local leaf
rerank or candidate-cap policy. The remaining gap is not fixed by exact rerank
over the routed local candidate frontier; Phase 4 should focus on route-set /
topology refinement.

## Notes

- This is not Task 120 closeout. Phases 4-6 and AWS/distributed evidence remain
  open.
- Corpus/query TSVs and truth-cache data are not committed.
- No raw per-query pipeline JSONL families are committed; the packet keeps the
  suite runner's structured result JSONL plus per-step logs.
