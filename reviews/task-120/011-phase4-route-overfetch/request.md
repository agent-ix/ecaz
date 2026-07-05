# Task 120 Phase 4 Route-Overfetch Curves

Please review the local topology route-set refinement evidence for Task 120
Phase 4.

## Scope

This packet reuses the Phase 2 recursive RabitQ indexes and measures route
overfetch plus routed-row budgets at 10k / 50k / 100k. It tests whether a
larger topology route set can recover recall, and whether route-time row
budgets can retain that recall with less local object/candidate work.

No source code changed for this packet.

## Evidence

- Artifact manifest:
  `reviews/task-120/011-phase4-route-overfetch/artifacts/manifest.md`
- Compact summary:
  `reviews/task-120/011-phase4-route-overfetch/artifacts/phase4-route-overfetch-summary.txt`
- Suite config:
  `reviews/task-120/011-phase4-route-overfetch/artifacts/suite.json`
- Suite status:
  `reviews/task-120/011-phase4-route-overfetch/artifacts/suite-status.log`
  reports `completed=16 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`
- Structured results:
  - `reviews/task-120/011-phase4-route-overfetch/artifacts/suite-results.jsonl`
  - `reviews/task-120/011-phase4-route-overfetch/artifacts/suite-report-results.jsonl`
- Host precheck:
  `reviews/task-120/011-phase4-route-overfetch/artifacts/precheck-host.log`
  reports PostgreSQL `18.3` and `ecaz_build_profile=release`

## Matrix

- Scales: 10k, 50k, 100k
- Queries: 200 per scale
- Overfetch sweep: `nprobe=32,48,64,96`
- Route-row caps at `nprobe=96`: `max_routed_candidate_rows=25000,50000,75000`
- Index ceiling: staged reloptions have `top_graph_search_list_size=96`, so
  `nprobe=128` was not part of this packet

## Result

Plain route overfetch is recall-positive, especially at 50k and 100k, but it
gets expensive. The 100k point improves from `0.9310` recall at `nprobe=32` to
`0.9975` at `nprobe=96`, while p50/p95 rises from `26.121/32.602 ms` to
`66.596/96.757 ms` and object reads rise to `13,043,852,590` bytes across 200
queries.

The 25k routed-row cap preserves the high-recall `nprobe=96` result while
cutting routed/object volume:

| Scale | Variant | recall@10 | p50 | p95 | routes | candidate_sum | object_bytes_sum |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 50k | nprobe96 | 1.0000 | 33.494 ms | 35.887 ms | 19,200 | 7,008,867 | 5,906,826,386 |
| 50k | nprobe96 rowcap25k | 1.0000 | 32.172 ms | 34.423 ms | 13,799 | 5,043,969 | 4,250,896,754 |
| 100k | nprobe96 | 0.9975 | 66.596 ms | 96.757 ms | 19,200 | 15,506,227 | 13,043,852,590 |
| 100k | nprobe96 rowcap25k | 0.9975 | 63.595 ms | 93.190 ms | 6,315 | 5,109,734 | 4,298,195,094 |

The looser 50k and 75k routed-row caps do not improve recall at 100k and add
work relative to the 25k cap. The final heap rerank volume remains capped at
5,000 rows total for 200 queries in all measured variants.

## Recommendation

Carry `nprobe=96` route overfetch with a tight routed-row budget forward as a
candidate Phase 5/AWS hypothesis. Do not promote a product default from this
packet: this is local-only evidence, 100k latency is still high, and Task 120
still needs distributed near-data rerank/shipping evidence plus AWS 1M evidence.

## Notes

- This is not Task 120 closeout. Phases 5-6 and AWS/distributed evidence remain
  open.
- Corpus/query TSVs and truth-cache data are not committed.
- No raw per-query pipeline JSONL families are committed; the packet keeps the
  suite runner's structured result JSONL plus per-step logs.
