# Task 131 Phase 1 50k n128/b4 Local Multi-Instance A/B

## Request

Review the Phase 1 local multi-instance A/B evidence for `50k / n128 / b4 /
nprobe96`, comparing baseline production read against
`ec_spire.remote_search_global_pre_heap_merge=on`.

This is one required matrix cell for Phase 1. It is not a promotion or closeout
claim.

## Command

```sh
target/debug/ecaz bench suite run \
  --config reviews/task-131/004-phase1-local-mi-ab-suite/artifacts/task131-phase1-local-mi-ab-suite.json \
  --only mi-50k-n128-b4-global-preheap-ab \
  --manifest-output reviews/task-131/004-phase1-local-mi-ab-suite/artifacts/50k-n128-b4-suite-manifest.json \
  --results-output reviews/task-131/004-phase1-local-mi-ab-suite/artifacts/50k-n128-b4-results.jsonl \
  --log-file reviews/task-131/004-phase1-local-mi-ab-suite/artifacts/50k-n128-b4-suite-run.log
```

The source run was copied from packet 004 artifacts into this packet after
pruning generated distributed-correctness TSVs. Packet-local provenance and
artifact details are in `artifacts/manifest.md`.

## Result Summary

Structured source: `artifacts/50k-n128-b4/bench-suite/results.jsonl`.

| Variant | recall@10 | query p50 | query p95 | query p99 | production-read total p50/p95/p99 | heap p50/p95/p99 | heap rows |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| baseline | 1.0000 | 2582.977 ms | 2953.783 ms | 3514.706 ms | 2537 / 3261 / 3522 ms | 3337 / 4157 / 4880 ms | 6000 |
| global-preheap | 1.0000 | 2593.483 ms | 2981.787 ms | 3492.307 ms | 1303 / 1758 / 1904 ms | 7 / 11 / 12 ms | 2000 |

Safety counters were clean in both variants: strict failures, timeouts,
cancellations, degraded skips, and remote heap failed dispatches were all `0`.

## Interpretation

The coordinator query latency is effectively flat at this scale and shape:
global-preheap is slightly worse at p50/p95 and slightly better at p99. The
production-read profile still shows the intended heap-materialization reduction:
aggregate heap rows fall from `6000` to `2000`, and aggregate heap p50 falls
from `3337 ms` to `7 ms`.

The profile uses `--production-read-timeline-no-payload`, so payload byte
avoidance is intentionally not measured in this lane (`payload_bytes_sum=0` in
both variants). As in the 10k packets, the per-node timeline rows still show
`payload_rows_sum=2000` per node in the global-preheap path; use the aggregate
production-read profile row for the heap-row A/B summary.
