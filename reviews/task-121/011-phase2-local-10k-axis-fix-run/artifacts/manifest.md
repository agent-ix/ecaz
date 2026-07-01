# Manifest: Task 121 Phase 2 Local 10k Axis-Fix Run

- Head SHA: `507c781fc92d815ed44693b190548a4730daadc2`
- Task bucket: `reviews/task-121/`
- Packet path: `reviews/task-121/011-phase2-local-10k-axis-fix-run/`
- Timestamp: `2026-06-23T12:02:18-07:00`
- Host/lane: local PG18 on `/home/peter/.pgrx`, port `28818`
- Database: `tqvector_bench_task121`
- Runner: `target/debug/ecaz bench suite run`
- Suite: `task121-phase2-local-10k-axis-fix-run`
- Corpus/query scale: staged real corpus 10k, 200 queries, k=10
- Storage format: `rabitq`
- PQ: not included
- AWS: not used
- Table/index isolation: one table/index per cell

## Command

```bash
script -q -c "target/debug/ecaz --database tqvector_bench_task121 --host /home/peter/.pgrx --port 28818 bench suite run --config reviews/task-121/011-phase2-local-10k-axis-fix-run/artifacts/suite-phase2-local-10k-axis-fix-run.json --manifest-output reviews/task-121/011-phase2-local-10k-axis-fix-run/artifacts/suite-phase2-local-10k-axis-fix-run-manifest.json --results-output reviews/task-121/011-phase2-local-10k-axis-fix-run/artifacts/suite-phase2-local-10k-axis-fix-run-results.jsonl --log-file reviews/task-121/011-phase2-local-10k-axis-fix-run/artifacts/suite-phase2-local-10k-axis-fix-run.log" reviews/task-121/011-phase2-local-10k-axis-fix-run/artifacts/suite-phase2-local-10k-axis-fix-run.script.log
```

## Inputs And Axes

- Config: `suite-phase2-local-10k-axis-fix-run.json`
- Pre-run prefix audit: `pre-run-existing-10k-prefixes.log`
- Boundary replica count: `0`, `1`, `2`, `4`
- Training sample rows: `10000`, `50000`
- Recursive fanout: `8`, `16`
- `nlists=128`
- Top graph enabled with degree `32`, build list `100`, search list `96`
- Pipeline nprobe sweep: `4,8,12,16,24,32,48,64,96`

## Artifacts

- `suite-phase2-local-10k-axis-fix-run.json`: checked suite config.
- `suite-phase2-local-10k-axis-fix-run-audit.log`: pre-run audit output.
- `suite-phase2-local-10k-axis-fix-run.log`: suite runner log.
- `suite-phase2-local-10k-axis-fix-run.script.log`: command transcript.
- `suite-phase2-local-10k-axis-fix-run-manifest.json`: suite runner manifest.
- `suite-phase2-local-10k-axis-fix-run-results.jsonl`: structured result rows.
- `load-10k_*.log`: load/build logs for each cell.
- `storage-10k_*.log`: storage reports for each cell.
- `pipeline-10k_*.log`: pipeline recall/latency summaries for each cell.
- `truth-cache-10k-q200-k10.log`: truth-cache generation log.
- `truth-cache-10k-q200-k10.json`: generated truth cache, intentionally not committed.
- `pipeline-10k_*-funnel.jsonl`: generated per-query funnel diagnostics, intentionally not committed.
- `pipeline-10k_*-stage-containment.jsonl`: generated per-stage containment diagnostics, intentionally not committed.

## Key Result Lines

At `nprobe=4`:

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

| Cell family | Index size |
| --- | ---: |
| b0 | 9.4 MiB |
| b1 | 17.2-17.3 MiB |
| b2 | 25.0-25.1 MiB |
| b4 | 40.6-40.7 MiB |

Load/build totals were 31.08-36.65 seconds across the 16 10k cells.

## Conclusion

`boundary_replica_count=1` is the only candidate worth scaling first. It improves low-probe recall by roughly 0.008-0.0115 absolute over b0, with a p50 increase from about 18-21 ms to about 26 ms and index growth from 9.4 MiB to 17.2-17.3 MiB. `b2` and `b4` add too much latency/storage for little additional recall at 10k. Training sample rows and fanout did not show enough 10k signal to justify broad scaling.
