# Task 28 IVF Latency Memory HWM Flag

## Scope

This packet covers `306b31b5`, which adds optional backend memory sampling to `ecaz bench latency`.

New flags:

- `--sample-backend-memory`
- `--memory-sample-interval-ms <ms>`

When enabled, each latency worker samples its own PostgreSQL backend `/proc/{pid}/status` while the sweep runs. The output table adds:

- `rss_peak_kb`
- `hwm_peak_kb`
- `memory_samples`

The feature is off by default, so existing latency packet output remains unchanged unless the new flag is passed.

## Smoke Result

PG18 smoke on the existing 100k IVF PQ-FastScan surface completed with memory columns:

- prefix: `task28_ivf_pqg100k_g8_n128`
- nprobe: `48`
- iterations: `2`
- p50: `244.6 ms`
- p95: `260.2 ms`
- p99: `261.6 ms`
- RSS peak: `89108 kB`
- HWM peak: `89108 kB`
- memory samples: `19`

Raw output is in `artifacts/latency_memory_smoke.log`.

## Validation

- `cargo test -p ecaz-cli latency`
- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench latency --prefix task28_ivf_pqg100k_g8_n128 --profile ec_ivf --k 10 --iterations 2 --sweep 48 --rerank-width 500 --force-index --sample-backend-memory --memory-sample-interval-ms 25 --log-output review/30112-task28-ivf-latency-memory-hwm/artifacts/latency_memory_smoke.log`
- `git diff --check`

## Next

Use `--sample-backend-memory` on A10/A9 latency packets so scan memory HWM is captured by the benchmark surface instead of by ad hoc side monitoring.
