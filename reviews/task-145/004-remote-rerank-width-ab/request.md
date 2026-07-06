# Task 145 Packet 004: Remote Rerank Width Release A/B

## Request

Please review the remote-path release A/B for `ec_spire.rerank_width=0` versus
`50` at 10k/50k/100k:

- `reviews/task-145/004-remote-rerank-width-ab/artifacts/manifest.md`
- `reviews/task-145/004-remote-rerank-width-ab/artifacts/task145-remote-rerank-width-ab-suite.json`
- nested per-scale `bench-suite/results.jsonl` and suite manifests under
  `remote-10k-n128-r3/`, `remote-50k-n1024-r3/`, and `remote-100k-n1024-r3/`

## Why This Packet Exists

Packet 003 approved the code checkpoint for remote `rerank_width`, but correctly
said the end-to-end evidence was still owed: release build, real remote path,
`remote_fanout_sum > 0`/remote heap counters, latency, and recall.

This packet runs that A/B through `ecaz bench suite` on `spire-local-multinode`.

## Result

This is a negative/diagnostic result, not a Task 145 closeout claim.

- Release install/build profiles are recorded for coordinator and all three
  remotes in every `local-multinode.log`.
- The remote path is engaged: profile rows have nonzero
  `remote_heap_candidate_sum` and `local_heap_candidate_sum=0`.
- Recall is equal between variants at every measured nprobe; the identity JSONL
  files are byte-identical between width 0 and width 50 at all three scales.
- Width 50 does **not** reduce `remote_heap_candidate_sum` versus width 0, and
  latency differences are small/noisy. At nprobe 96 each scale remains
  `remote_heap_candidate_sum=6000` for both variants.

The next implementation slice should inspect whether the production-read remote
profile counter is pre-truncation, whether the GUC is applied too late for the
measured path, or whether a remote SQL path still bypasses the packet 003
truncation.

## Key nprobe=96 Lines

| scale | width | distinct_recall@k | pipeline p50/p95 | profile total p50/p95 | remote_heap_candidate_sum |
| --- | ---: | ---: | --- | --- | ---: |
| 10k n128 | 0 | 0.9855 | 64.556 / 67.950 ms | 34.053 / 36.269 ms | 6000 |
| 10k n128 | 50 | 0.9855 | 63.256 / 67.006 ms | 33.227 / 36.068 ms | 6000 |
| 50k n1024 | 0 | 0.9560 | 68.213 / 76.032 ms | 38.478 / 43.431 ms | 6000 |
| 50k n1024 | 50 | 0.9560 | 70.605 / 72.780 ms | 40.212 / 42.411 ms | 6000 |
| 100k n1024 | 0 | 0.9480 | 70.048 / 72.402 ms | 40.732 / 42.511 ms | 6000 |
| 100k n1024 | 50 | 0.9480 | 69.947 / 72.280 ms | 40.470 / 42.463 ms | 6000 |

## Hygiene

I am staging the bounded suite config, manifests, JSONL result/identity rows,
and production-read logs only. Generated corpus TSVs, local distributed
correctness TSVs, and PostgreSQL server logs are intentionally not part of the
review evidence.
