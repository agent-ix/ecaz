# Task 131 Phase 1 50k n1024/b2 Local Multi-Instance A/B

## Request

Review the Phase 1 local multi-instance A/B evidence for `50k / n1024 / b2 /
nprobe64`, comparing baseline production read against
`ec_spire.remote_search_global_pre_heap_merge=on`.

This is one required matrix cell for Phase 1. It is not a promotion or closeout
claim.

## Command

```sh
target/debug/ecaz bench suite run \
  --config reviews/task-131/004-phase1-local-mi-ab-suite/artifacts/task131-phase1-local-mi-ab-suite.json \
  --only mi-50k-n1024-b2-global-preheap-ab \
  --manifest-output reviews/task-131/004-phase1-local-mi-ab-suite/artifacts/50k-n1024-b2-suite-manifest.json \
  --results-output reviews/task-131/004-phase1-local-mi-ab-suite/artifacts/50k-n1024-b2-results.jsonl \
  --log-file reviews/task-131/004-phase1-local-mi-ab-suite/artifacts/50k-n1024-b2-suite-run.log
```

The source run was copied from packet 004 artifacts into this packet after
pruning generated distributed-correctness TSVs. Packet-local provenance and
artifact details are in `artifacts/manifest.md`.

## Result Summary

Structured source: `artifacts/50k-n1024-b2/bench-suite/results.jsonl`.

| Variant | recall@10 | query p50 | query p95 | query p99 | production-read total p50/p95/p99 | heap rows |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| baseline | 0.9980 | 663.809 ms | 795.704 ms | 904.363 ms | 393 / 447 / 549 ms | 6000 |
| global-preheap | 0.9980 | 663.340 ms | 718.746 ms | 859.830 ms | 317 / 345 / 411 ms | 2000 |

Safety counters were clean in both variants: strict failures, timeouts,
cancellations, degraded skips, and remote heap failed dispatches were all `0`.

## Interpretation

At `50k / n1024 / b2 / nprobe64`, coordinator query p50 is flat while p95/p99
improve modestly under global-preheap. The production-read profile shows the
expected aggregate heap-row reduction from `6000` to `2000`, and total
production-read profile p50/p95/p99 improves from `393 / 447 / 549 ms` to
`317 / 345 / 411 ms`.

The fixture cost was high: the coordinator index build took `2872.58s`, and the
three remote index builds took `2061.87s`, `1999.13s`, and `2016.32s`.

The profile uses `--production-read-timeline-no-payload`, so payload byte
avoidance is intentionally not measured in this lane (`payload_bytes_sum=0` in
both variants). As in prior Phase 1 packets, the per-node timeline rows still
show `payload_rows_sum=2000` per node in the global-preheap path; use the
aggregate production-read profile row for the heap-row A/B summary.
