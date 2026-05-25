Script started on 2026-05-25 12:05:50-07:00 [<not executed on terminal>]
# Suite Report: pooling-gate-synthetic

- config: `reviews/task-30/974-spire-phase13e-pooling-gate-report/artifacts/pooling-gate-suite.json`
- config_sha256: `synthetic`
- dry_run: `false`
- steps: completed 1, failed 0, skipped 0, dry-run 0, missing artifacts 0, stale 0

| Step | Kind | Status | Duration ms | Artifacts |
| --- | --- | --- | ---: | --- |
| synthetic-profile | spire-pipeline | Succeeded | 1000 | `reviews/task-30/974-spire-phase13e-pooling-gate-report/artifacts/pooling-gate-spire-pipeline.log` |

## Parsed Results

| Step | Kind | Metric | Values |
| --- | --- | --- | --- |
| synthetic-profile | spire-pipeline | spire-pipeline | `latency_max=10.000 ms`, `latency_min=7.000 ms`, `latency_p50=8.000 ms`, `latency_p95=10.000 ms`, `latency_p99=10.000 ms`, `nprobe=8`, `queries=2`, `recall@k=1.0000` |
| synthetic-profile | spire-pipeline | spire-pipeline | `latency_max=10.000 ms`, `latency_min=8.000 ms`, `latency_p50=9.000 ms`, `latency_p95=10.000 ms`, `latency_p99=10.000 ms`, `nprobe=16`, `queries=2`, `recall@k=1.0000` |
| synthetic-profile | spire-pipeline | spire-pipeline | `cancel_sum=0`, `candidate_p50=2.000 ms`, `candidate_p95=3.000 ms`, `candidate_query_sum=3`, `connect_p50=0.500 ms`, `connect_p95=1.000 ms`, `degraded_skip_sum=0`, `dispatch_sum=3`, `heap_p50=2.000 ms`, `heap_p95=3.000 ms`, `heap_query_sum=3`, `merge_p50=0.100 ms`, `merge_p95=0.200 ms`, `nprobe=8`, `payload_bytes_sum=1024`, `profiles=2`, `remote_pid_sum=6`, `result_source=remote_heap_candidates`, `returned_sum=10`, `selected_pid_sum=6`, `socket_open_sum=3`, `status=ready`, `timeout_sum=0`, `total_p50=8.000 ms`, `total_p95=10.000 ms` |
| synthetic-profile | spire-pipeline | spire-pipeline | `cancel_sum=0`, `candidate_p50=2.000 ms`, `candidate_p95=3.000 ms`, `candidate_query_sum=3`, `connect_p50=1.200 ms`, `connect_p95=1.600 ms`, `degraded_skip_sum=0`, `dispatch_sum=3`, `heap_p50=2.000 ms`, `heap_p95=3.000 ms`, `heap_query_sum=3`, `merge_p50=0.100 ms`, `merge_p95=0.200 ms`, `nprobe=16`, `payload_bytes_sum=1024`, `profiles=2`, `remote_pid_sum=6`, `result_source=remote_heap_candidates`, `returned_sum=10`, `selected_pid_sum=6`, `socket_open_sum=3`, `status=ready`, `timeout_sum=0`, `total_p50=8.000 ms`, `total_p95=10.000 ms` |

## SPIRE Connection Pooling Gate

| Step | nprobe | connect_p50_ms | connect_p95_ms | latency_p95_ms | connect_p95/read_p95 | decision |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| synthetic-profile | 8 | 0.500 | 1.000 | 10.000 | 0.1000 | pooling_not_justified |
| synthetic-profile | 16 | 1.200 | 1.600 | 10.000 | 0.1600 | pooling_candidate |
wrote reviews/task-30/974-spire-phase13e-pooling-gate-report/artifacts/pooling-gate-results.jsonl

Script done on 2026-05-25 12:05:51-07:00 [COMMAND_EXIT_CODE="0"]
