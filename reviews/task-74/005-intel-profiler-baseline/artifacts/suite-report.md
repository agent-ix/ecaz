# Suite Report: task74-intel-profiler-baseline

- config: `reviews/task-74/005-intel-profiler-baseline/artifacts/suite.json`
- config_sha256: `336cc795fb6e6f4c58752825f4e8e14c14cb5a8b3e554e9a7080b8fdb200334d`
- dry_run: `false`
- steps: completed 6, failed 0, skipped 0, dry-run 0, missing artifacts 0, stale 0

| Step | Kind | Status | Duration ms | Artifacts |
| --- | --- | --- | ---: | --- |
| precheck-host-and-inputs | raw | Succeeded | 26 | `reviews/task-74/005-intel-profiler-baseline/artifacts/precheck-host-and-inputs.log` |
| drop-prior-intel-profiler-fixtures | raw | Succeeded | 17 | `reviews/task-74/005-intel-profiler-baseline/artifacts/drop-prior-intel-profiler-fixtures.log` |
| load-100k-spire-highrecall-tg128-b0 | load | Succeeded | 301656 | `reviews/task-74/005-intel-profiler-baseline/artifacts/load-100k-spire-highrecall-tg128-b0.log` |
| latency-100k-spire-highrecall-tg128-b0 | latency | Succeeded | 29305 | `reviews/task-74/005-intel-profiler-baseline/artifacts/latency-100k-spire-highrecall-tg128-b0.log` |
| load-100k-ivf-control | load | Succeeded | 289035 | `reviews/task-74/005-intel-profiler-baseline/artifacts/load-100k-ivf-control.log` |
| latency-100k-ivf-control | latency | Succeeded | 9263 | `reviews/task-74/005-intel-profiler-baseline/artifacts/latency-100k-ivf-control.log` |

## Parsed Results

| Step | Kind | Metric | Values |
| --- | --- | --- | --- |
| load-100k-spire-highrecall-tg128-b0 | load | load_timing | `phase=copy_corpus`, `prefix=task74_intel_spire_highrecall_tg128_b0`, `profile=ec_spire`, `seconds=94.340000`, `subject=task74_intel_spire_highrecall_tg128_b0_corpus`, `suite_database=postgres`, `suite_host=/home/peter/.pgrx`, `suite_port=28818` |
| load-100k-spire-highrecall-tg128-b0 | load | load_timing | `phase=encode_corpus`, `prefix=task74_intel_spire_highrecall_tg128_b0`, `profile=ec_spire`, `seconds=38.230000`, `subject=task74_intel_spire_highrecall_tg128_b0_corpus`, `suite_database=postgres`, `suite_host=/home/peter/.pgrx`, `suite_port=28818` |
| load-100k-spire-highrecall-tg128-b0 | load | load_timing | `phase=copy_queries`, `prefix=task74_intel_spire_highrecall_tg128_b0`, `profile=ec_spire`, `seconds=0.930040`, `subject=task74_intel_spire_highrecall_tg128_b0_queries`, `suite_database=postgres`, `suite_host=/home/peter/.pgrx`, `suite_port=28818` |
| load-100k-spire-highrecall-tg128-b0 | load | load_timing | `phase=build_index`, `prefix=task74_intel_spire_highrecall_tg128_b0`, `profile=ec_spire`, `seconds=8.880000`, `subject=task74_intel_spire_highrecall_tg128_b0_idx`, `suite_database=postgres`, `suite_host=/home/peter/.pgrx`, `suite_port=28818` |
| load-100k-spire-highrecall-tg128-b0 | load | load_timing | `phase=total`, `prefix=task74_intel_spire_highrecall_tg128_b0`, `profile=ec_spire`, `seconds=301.630000`, `subject=task74_intel_spire_highrecall_tg128_b0`, `suite_database=postgres`, `suite_host=/home/peter/.pgrx`, `suite_port=28818` |
| latency-100k-spire-highrecall-tg128-b0 | latency | latency | `cache_state=post_load_warm`, `count=200`, `hwm_peak_kb=150716`, `max=183.7 ms`, `mean=138.8 ms`, `memory_samples=1050`, `min=125.9 ms`, `nprobe=96`, `p50=137.9 ms`, `p95=151.4 ms`, `p99=161.8 ms`, `prefix=task74_intel_spire_highrecall_tg128_b0`, `profile=ec_spire`, `rss_peak_kb=150716`, `stddev=7.87 ms`, `suite_database=postgres`, `suite_host=/home/peter/.pgrx`, `suite_port=28818` |
| load-100k-ivf-control | load | load_timing | `phase=copy_corpus`, `prefix=task74_intel_ivf_control`, `profile=ec_ivf`, `seconds=94.680000`, `storage_format=pq_fastscan`, `subject=task74_intel_ivf_control_corpus`, `suite_database=postgres`, `suite_host=/home/peter/.pgrx`, `suite_port=28818` |
| load-100k-ivf-control | load | load_timing | `phase=encode_corpus`, `prefix=task74_intel_ivf_control`, `profile=ec_ivf`, `seconds=35.160000`, `storage_format=pq_fastscan`, `subject=task74_intel_ivf_control_corpus`, `suite_database=postgres`, `suite_host=/home/peter/.pgrx`, `suite_port=28818` |
| load-100k-ivf-control | load | load_timing | `phase=copy_queries`, `prefix=task74_intel_ivf_control`, `profile=ec_ivf`, `seconds=0.950230`, `storage_format=pq_fastscan`, `subject=task74_intel_ivf_control_queries`, `suite_database=postgres`, `suite_host=/home/peter/.pgrx`, `suite_port=28818` |
| load-100k-ivf-control | load | load_timing | `phase=build_index`, `prefix=task74_intel_ivf_control`, `profile=ec_ivf`, `seconds=8.200000`, `storage_format=pq_fastscan`, `subject=task74_intel_ivf_control_pq_fastscan_idx`, `suite_database=postgres`, `suite_host=/home/peter/.pgrx`, `suite_port=28818` |
| load-100k-ivf-control | load | load_timing | `phase=total`, `prefix=task74_intel_ivf_control`, `profile=ec_ivf`, `seconds=289.020000`, `storage_format=pq_fastscan`, `subject=task74_intel_ivf_control`, `suite_database=postgres`, `suite_host=/home/peter/.pgrx`, `suite_port=28818` |
| latency-100k-ivf-control | latency | latency | `cache_state=post_load_warm`, `count=200`, `hwm_peak_kb=156508`, `max=52.8 ms`, `mean=38.6 ms`, `memory_samples=293`, `min=34.4 ms`, `nprobe=96`, `p50=37.8 ms`, `p95=44.9 ms`, `p99=49.8 ms`, `prefix=task74_intel_ivf_control`, `profile=ec_ivf`, `rss_peak_kb=156508`, `stddev=2.95 ms`, `suite_database=postgres`, `suite_host=/home/peter/.pgrx`, `suite_port=28818` |
