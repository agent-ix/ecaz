# Review Request: Task 28 IVF A10 TurboQuant Memory Fill

## Summary

This packet fills the A10 warm-cache memory-HWM gap for the smaller-corpus
TurboQuant matched-width rows.

It reuses the existing 10k and 25k TurboQuant IVF surfaces from packet 30097,
and measures `nprobe=48`, `rerank_width=750`, forced IVF path, with backend
memory sampling enabled.

## Result

| corpus | profile | p50 | p95 | p99 | HWM |
|---|---|---:|---:|---:|---:|
| 10k | TurboQuant w750 | 130.6 ms | 231.6 ms | 267.9 ms | 109600 kB |
| 25k | TurboQuant w750 | 284.5 ms | 402.4 ms | 441.5 ms | 155540 kB |

For A10 context, packet 30137 already recorded the matched PQ-FastScan g8
post-A7 rows:

| corpus | profile | p50 | p95 | p99 | HWM |
|---|---|---:|---:|---:|---:|
| 10k | PQ-FastScan g8 w750 | 77.3 ms | 80.4 ms | 82.2 ms | 137244 kB |
| 25k | PQ-FastScan g8 w750 | 116.8 ms | 123.7 ms | 125.7 ms | 156112 kB |

## Interpretation

This does not change the A10 recommendation. PQ-FastScan g8 remains much faster
on the 10k/25k matched-width surfaces after A7. TurboQuant still has the
smaller-corpus recall@100 advantage from packet 30097, so the current `auto`
default should still remain unchanged in Task 28.

Memory does not decide the smaller-corpus comparison: TurboQuant is lower HWM
on the 10k run, and effectively tied with PQ-FastScan g8 on the 25k run.

Cache state: warm local development runs; no explicit OS or PostgreSQL buffer
cache drop.

## Validation

- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench latency --prefix task28_ivf_qcmp10k_turboquant --profile ec_ivf --k 10 --iterations 100 --sweep 48 --rerank-width 750 --force-index --sample-backend-memory --memory-sample-interval-ms 25 --log-output review/30143-task28-ivf-a10-turboquant-memory/artifacts/latency_turboquant_10k_n64_w750_p48_hwm.log`
- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench latency --prefix task28_ivf_postopt25k_n64w25 --profile ec_ivf --k 10 --iterations 100 --sweep 48 --rerank-width 750 --force-index --sample-backend-memory --memory-sample-interval-ms 25 --log-output review/30143-task28-ivf-a10-turboquant-memory/artifacts/latency_turboquant_25k_n64_w750_p48_hwm.log`

## Artifacts

- `artifacts/latency_turboquant_10k_n64_w750_p48_hwm.log`
- `artifacts/latency_turboquant_25k_n64_w750_p48_hwm.log`
- `artifacts/manifest.md`
