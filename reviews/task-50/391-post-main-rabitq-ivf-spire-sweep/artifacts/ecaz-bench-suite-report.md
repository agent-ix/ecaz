Script started on 2026-05-21 20:57:18-07:00 [<not executed on terminal>]
# Suite Report: task50-post-main-rabitq-ivf-spire-local

- config: `reviews/task-50/391-post-main-rabitq-ivf-spire-sweep/artifacts/rabitq-ivf-spire-local-suite.json`
- config_sha256: `d6636965582aedcb4f4b6b2cdc126f06d9964a68b46c850ee16d392a052a04eb`
- dry_run: `false`
- steps: completed 4, failed 0, skipped 0, dry-run 0, missing artifacts 0, stale 0

| Step | Kind | Status | Duration ms | Artifacts |
| --- | --- | --- | ---: | --- |
| ivf-rabitq-10k-recall-k10 | recall | Succeeded | 32913 | `reviews/task-50/391-post-main-rabitq-ivf-spire-sweep/artifacts/ivf-rabitq-10k-recall-k10.log` |
| ivf-rabitq-10k-latency-k10-c1 | latency | Succeeded | 8114 | `reviews/task-50/391-post-main-rabitq-ivf-spire-sweep/artifacts/ivf-rabitq-10k-latency-k10-c1.log` |
| spire-rabitq-10k-recall-k10 | recall | Succeeded | 62231 | `reviews/task-50/391-post-main-rabitq-ivf-spire-sweep/artifacts/spire-rabitq-10k-recall-k10.log` |
| spire-rabitq-10k-latency-k10-c1 | latency | Succeeded | 32395 | `reviews/task-50/391-post-main-rabitq-ivf-spire-sweep/artifacts/spire-rabitq-10k-latency-k10-c1.log` |

## Parsed Results

| Step | Kind | Metric | Values |
| --- | --- | --- | --- |
| ivf-rabitq-10k-recall-k10 | recall | recall | `mean q-time=59.36 ms`, `ndcg@k=0.9995`, `nprobe=8`, `recall@k=0.9720` |
| ivf-rabitq-10k-recall-k10 | recall | recall | `mean q-time=97.70 ms`, `ndcg@k=0.9998`, `nprobe=16`, `recall@k=0.9780` |
| ivf-rabitq-10k-latency-k10-c1 | latency | latency | `count=50`, `max=88.9 ms`, `mean=62.1 ms`, `min=37.1 ms`, `nprobe=8`, `p50=63.3 ms`, `p95=82.0 ms`, `p99=86.5 ms`, `stddev=12.4 ms` |
| ivf-rabitq-10k-latency-k10-c1 | latency | latency | `count=50`, `max=128.7 ms`, `mean=92.2 ms`, `min=61.0 ms`, `nprobe=16`, `p50=90.1 ms`, `p95=119.6 ms`, `p99=124.9 ms`, `stddev=16.1 ms` |
| spire-rabitq-10k-recall-k10 | recall | recall | `mean q-time=330.34 ms`, `ndcg@k=0.9996`, `nprobe=8`, `recall@k=0.9880` |
| spire-rabitq-10k-recall-k10 | recall | recall | `mean q-time=416.22 ms`, `ndcg@k=0.9999`, `nprobe=16`, `recall@k=0.9960` |
| spire-rabitq-10k-latency-k10-c1 | latency | latency | `count=50`, `max=302.8 ms`, `mean=229.7 ms`, `min=143.2 ms`, `nprobe=8`, `p50=229.1 ms`, `p95=286.8 ms`, `p99=302.5 ms`, `stddev=41.2 ms` |
| spire-rabitq-10k-latency-k10-c1 | latency | latency | `count=50`, `max=529.6 ms`, `mean=411.1 ms`, `min=246.3 ms`, `nprobe=16`, `p50=427.5 ms`, `p95=509.0 ms`, `p99=528.7 ms`, `stddev=70.4 ms` |
wrote reviews/task-50/391-post-main-rabitq-ivf-spire-sweep/artifacts/results-report.jsonl

Script done on 2026-05-21 20:57:18-07:00 [COMMAND_EXIT_CODE="0"]
