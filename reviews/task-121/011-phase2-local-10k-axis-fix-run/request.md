# Task 121 Phase 2 Local 10k Axis-Fix Run

## Scope

This packet records a completed local-only `ecaz bench suite` run for the Task 121 Phase 2 boundary-replica axis screen at 10k scale.

- No AWS resources were used.
- Storage format: `rabitq`.
- PQ was not included.
- The run used isolated one table/index per cell.
- This is not task closeout evidence yet; 50k and 100k A/B evidence is still required before closeout.

## Evidence

- Suite config: `artifacts/suite-phase2-local-10k-axis-fix-run.json`
- Suite manifest: `artifacts/suite-phase2-local-10k-axis-fix-run-manifest.json`
- Structured results: `artifacts/suite-phase2-local-10k-axis-fix-run-results.jsonl`
- Suite log: `artifacts/suite-phase2-local-10k-axis-fix-run.log`
- Script transcript: `artifacts/suite-phase2-local-10k-axis-fix-run.script.log`
- Per-cell load, storage, and pipeline logs are under `artifacts/`.

The generated truth cache `artifacts/truth-cache-10k-q200-k10.json` and the large per-query funnel/stage-containment JSONLs are intentionally not committed. Their aggregate measurements are captured in `suite-phase2-local-10k-axis-fix-run-results.jsonl` and the per-cell pipeline logs.

## Result Summary

At `nprobe=4`, the meaningful 10k signal is:

| Cell | p50 | p95 | recall@10 |
| --- | ---: | ---: | ---: |
| b0/tr10/f8 | 18.657 ms | 24.666 ms | 0.9810 |
| b0/tr10/f16 | 18.697 ms | 23.721 ms | 0.9785 |
| b0/tr50/f8 | 18.355 ms | 23.265 ms | 0.9810 |
| b0/tr50/f16 | 20.569 ms | 29.632 ms | 0.9785 |
| b1/tr10/f8 | 26.575 ms | 40.647 ms | 0.9900 |
| b1/tr10/f16 | 25.847 ms | 44.583 ms | 0.9885 |
| b1/tr50/f8 | 25.964 ms | 42.147 ms | 0.9900 |
| b1/tr50/f16 | 26.002 ms | 42.490 ms | 0.9885 |
| b2/tr10/f8 | 33.256 ms | 49.278 ms | 0.9910 |
| b2/tr10/f16 | 34.103 ms | 49.558 ms | 0.9885 |
| b2/tr50/f8 | 32.318 ms | 49.255 ms | 0.9910 |
| b2/tr50/f16 | 31.979 ms | 47.579 ms | 0.9885 |
| b4/tr10/f8 | 45.302 ms | 66.701 ms | 0.9935 |
| b4/tr10/f16 | 43.014 ms | 72.356 ms | 0.9900 |
| b4/tr50/f8 | 44.714 ms | 69.889 ms | 0.9935 |
| b4/tr50/f16 | 44.206 ms | 74.354 ms | 0.9900 |

Representative ec_spire index sizes:

| Boundary replicas | Index size |
| ---: | ---: |
| 0 | 9.4 MiB |
| 1 | 17.2-17.3 MiB |
| 2 | 25.0-25.1 MiB |
| 4 | 40.6-40.7 MiB |

## Interpretation

`boundary_replica_count=1` is the only promising next-scale candidate from this 10k screen. It lifts low-probe recall from roughly `0.9785-0.9810` to `0.9885-0.9900`, but costs about 7-8 ms p50 and roughly doubles the ec_spire index from 9.4 MiB to 17.2-17.3 MiB.

`boundary_replica_count=2` adds little or no recall over b1 in this screen while adding more latency and storage. `boundary_replica_count=4` can improve some low-probe recall points, but the latency/storage cost is too high for broad 50k/100k scaling unless a reviewer asks for it.

`training_sample_rows=10000` vs `50000` and `recursive_fanout=8` vs `16` do not show a useful 10k effect in this matrix. The next practical run should scale a narrow b0-vs-b1 A/B first at 50k/100k.
