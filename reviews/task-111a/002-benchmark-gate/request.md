# Review Request: Task 111a Benchmark Gate

## Scope

This packet is the benchmark gate for Task 111a after the scan-side dense
posting coalescing implementation and the suite runner `session_gucs` support.
It compares row postings, dense-old, and dense+A across real 50k and 100k
fixtures for TurboQuant and RaBitQ.

## Result

Approach A works. The TurboQuant dense regression was a scan-side batch-width
problem: dense-old scored almost every dense block as a tiny batch, while
dense+A coalesces across dense blocks and restores wide SIMD flushes.

At 100k / nprobe 32 / TurboQuant:

| Surface | p50 | p95 | p99 | Recall@10 | NDCG@10 | width_ge32 | width_8_15 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| row | 38.7 ms | 49.2 ms | 50.6 ms | 0.9370 | 0.9966 | 20363 | 5 |
| dense-old | 37.2 ms | 42.2 ms | 47.8 ms | 0.9370 | 0.9966 | 0 | 519350 |
| dense+A | 26.7 ms | 31.6 ms | 34.2 ms | 0.9370 | 0.9966 | 21443 | 142 |

At 50k / nprobe 32 / TurboQuant, dense+A is also faster than both baselines:
row p50 14.4 ms, dense-old p50 17.2 ms, dense+A p50 13.1 ms, with identical
recall@10 0.9420 and NDCG@10 0.9994.

RaBitQ still wins after Approach A. At 100k / nprobe 32 / RB1, row p50 is
14.5 ms, dense-old p50 is 11.6 ms, and dense+A p50 is 12.3 ms, with identical
recall@10 0.7630 and NDCG@10 0.9875. Dense+A is slightly slower than dense-old
for RaBitQ in this run, but it remains materially faster than row and avoids
the TurboQuant regression.

Storage reduction is preserved because Approach A does not change the on-disk
dense layout:

| Scale/format | Row index | Dense index |
| --- | ---: | ---: |
| 50k TurboQuant | 44.1 MiB | 39.8 MiB |
| 50k RaBitQ | 15.2 MiB | 11.6 MiB |
| 100k TurboQuant | 87.6 MiB | 78.9 MiB |
| 100k RaBitQ | 29.7 MiB | 22.5 MiB |

## Recommendation

Adopt Approach A for dense posting block scans. Reject/close Approach B for
Task 111a as dominated: the multi-page on-disk dense packing idea would change
the durable format to fix a batch-width issue already fixed at scan time.

Do not promote dense posting blocks as the default index layout in this packet.
The scan fix is good enough for the 50k/100k gate and preserves the RaBitQ win,
but dense posting blocks should remain explicitly opted in unless a separate
promotion packet runs the larger default-promotion lane.

## Evidence

- Full manifest and interpretation: `artifacts/manifest.md`
- Suite config: `artifacts/task111a-dense-coalescing-suite.json`
- Suite run log: `artifacts/suite-run.log`
- Suite status: `artifacts/suite-status.log`
- Suite report: `artifacts/suite-report.log`
- Structured results: `artifacts/suite/results.jsonl`
- Parsed report rows: `artifacts/suite/results-report.jsonl`
- Per-step latency/recall/storage/EXPLAIN logs: `artifacts/suite/*.log`

The suite status is 60 completed, 0 failed, 0 skipped, 0 missing artifacts, 0
stale.

Generated recall truth caches under `artifacts/suite/truth-*.json` are not part
of this review evidence and are intentionally left uncommitted.
