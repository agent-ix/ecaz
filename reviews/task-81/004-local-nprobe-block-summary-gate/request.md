# Task 81 Local Nprobe Block-Summary Gate

## Summary

This packet tests whether broader routing can recover the AWS recall miss from `reviews/task-81/003-aws-1m-block-summary-gate/` without reopening the candidate surface. The accepted rerun builds an isolated local 100k tg256 block-summary surface, then runs the standard `ecaz bench suite` pipeline with global cap `1152`, block16 RabitQ summaries, q200, k10, and rerank width `25`.

The first attempt reused the Task 79 tg96 index and failed for `nprobe > 96` because the index's `top_graph_search_list_size` was `96`. That failed attempt is retained as provenance, but the accepted evidence is the rerun using `task81_nprobe_100k_idx` with `top_graph_search_list_size=256`.

## Result

| nprobe | effective_nprobe | route_sum | candidates | p50 ms | p95 ms | p99 ms | recall@10 |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 96 | 96 | 19,200 | 3,672,619 | 32.212 | 36.028 | 37.564 | 0.9945 |
| 128 | 128 | 25,600 | 3,672,641 | 34.351 | 41.397 | 47.028 | 0.9965 |
| 160 | 128 | 25,600 | 3,672,641 | 34.720 | 44.833 | 53.428 | 0.9965 |
| 192 | 128 | 25,600 | 3,672,641 | 34.994 | 39.457 | 46.414 | 0.9965 |

`nprobe=128` is the useful point: recall improves from `0.9945` to `0.9965`, candidates are effectively flat (`+22` over q200), and p50 remains under the Task 81 local gate at `34.351 ms`. Requested values above 128 clamp to `effective_nprobe=128`, so they do not add more route breadth.

## Decision

Use `nprobe=128` as the next AWS 1M gate candidate against the existing tg256 block-summary surface. The AWS run still has to prove the Task 81 scale gate: improve recall over the old `0.9832` q500 row while keeping candidates at or below `9,213,846`.

## Evidence

- `artifacts/manifest.md`
- `suite-local-nprobe-block-summary-gate.json`
- `artifacts/suite-audit-rerun.log`
- `artifacts/suite-manifest-rerun.json`
- `artifacts/results-rerun.jsonl`
- `artifacts/suite-run-rerun.log`
- `artifacts/suite-status-rerun.log`
- `artifacts/suite-report-rerun.log`
- `artifacts/suite-report-results-rerun.jsonl`
- `artifacts/prepare-task81-local-nprobe-tg256-surface.log`
- `artifacts/precheck-task81-local-nprobe-surface.log`
- `artifacts/pipeline-100k-rabitq-block-summary-global1152-nprobe-sweep.log`
- `artifacts/funnel-100k-rabitq-block-summary-global1152-nprobe-sweep.jsonl`
