# Review Request: Task 28 IVF A10 RaBitQ Fill

## Summary

This packet fills the missing RaBitQ rows in the A10 quantizer comparison.

The existing 10k RaBitQ IVF surface was reused. A matching 25k RaBitQ surface
was built from the existing 25k corpus/query tables with:

- `storage_format = 'rabitq'`
- `nlists = 64`
- `nprobe = 64`
- `training_sample_rows = 2000`
- `rerank = 'heap_f32'`
- `rerank_width = 750`

The scan rows use session `nprobe=48` and `rerank_width=750` to match the
10k/25k A10 TurboQuant and PQ-FastScan comparison point.

## Results

RaBitQ has high recall on these bounded checks, but it remains far outside the
latency band for the current IVF substrate.

| corpus | recall@10 | recall@100 | mean q-time @10 | mean q-time @100 |
|---|---:|---:|---:|---:|
| 10k | 1.0000 | 0.9930 | 1940.08 ms | 1953.62 ms |
| 25k | 1.0000 | 0.9915 | 5015.51 ms | 5043.47 ms |

Latency/HWM runs used 10 iterations because the recall rows already showed
multi-second per-query latency:

| corpus | p50 | p95 | p99 | HWM |
|---|---:|---:|---:|---:|
| 10k | 1947.8 ms | 2096.9 ms | 2128.3 ms | 69980 kB |
| 25k | 4973.0 ms | 5257.7 ms | 5327.9 ms | 145012 kB |

Index size:

| corpus | index size |
|---|---:|
| 10k | 9,641,984 bytes |
| 25k | 23,519,232 bytes |

25k build time for `task28_ivf_qcmp25k_rabitq_idx`: `40699.103 ms`.

## Interpretation

This supports the current A10 recommendation:

- RaBitQ is selectable and can produce high recall.
- RaBitQ is not a current default candidate for IVF because scan latency is
  roughly seconds per query at the same `nprobe=48`, `rerank_width=750` point
  where TurboQuant and PQ-FastScan are in the sub-second band.
- PQ-FastScan g8 remains the best measured speed/size profile for the 100k IVF
  lane, while TurboQuant remains the safer smaller-corpus recall@100 default.

Cache state: warm local development runs; no explicit OS or PostgreSQL buffer
cache drop.

## Limits

The RaBitQ recall rows use `queries-limit=20`, and the latency rows use
`iterations=10`, because the measured per-query time is already high enough to
establish that RaBitQ is latency-uncompetitive in this IVF integration.

## Validation

- `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30144-task28-ivf-a10-rabitq-fill/artifacts/build_rabitq_25k.sql --raw --log-output review/30144-task28-ivf-a10-rabitq-fill/artifacts/build_rabitq_25k.log`
- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_qcmp10k_rabitq --profile ec_ivf --k 10 --queries-limit 20 --sweep 48 --rerank-width 750 --force-index --log-output review/30144-task28-ivf-a10-rabitq-fill/artifacts/recall10_rabitq_10k_n64_w750_p48_q20.log`
- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_qcmp10k_rabitq --profile ec_ivf --k 100 --queries-limit 20 --sweep 48 --rerank-width 750 --force-index --log-output review/30144-task28-ivf-a10-rabitq-fill/artifacts/recall100_rabitq_10k_n64_w750_p48_q20.log`
- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_qcmp25k_rabitq --profile ec_ivf --k 10 --queries-limit 20 --sweep 48 --rerank-width 750 --force-index --log-output review/30144-task28-ivf-a10-rabitq-fill/artifacts/recall10_rabitq_25k_n64_w750_p48_q20.log`
- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_qcmp25k_rabitq --profile ec_ivf --k 100 --queries-limit 20 --sweep 48 --rerank-width 750 --force-index --log-output review/30144-task28-ivf-a10-rabitq-fill/artifacts/recall100_rabitq_25k_n64_w750_p48_q20.log`
- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench latency --prefix task28_ivf_qcmp10k_rabitq --profile ec_ivf --k 10 --iterations 10 --sweep 48 --rerank-width 750 --force-index --sample-backend-memory --memory-sample-interval-ms 25 --log-output review/30144-task28-ivf-a10-rabitq-fill/artifacts/latency_rabitq_10k_n64_w750_p48_i10_hwm.log`
- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench latency --prefix task28_ivf_qcmp25k_rabitq --profile ec_ivf --k 10 --iterations 10 --sweep 48 --rerank-width 750 --force-index --sample-backend-memory --memory-sample-interval-ms 25 --log-output review/30144-task28-ivf-a10-rabitq-fill/artifacts/latency_rabitq_25k_n64_w750_p48_i10_hwm.log`
- `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30144-task28-ivf-a10-rabitq-fill/artifacts/rabitq_size.sql --raw --log-output review/30144-task28-ivf-a10-rabitq-fill/artifacts/rabitq_size.log`

## Artifacts

- `artifacts/build_rabitq_25k.sql`
- `artifacts/build_rabitq_25k.log`
- `artifacts/recall10_rabitq_10k_n64_w750_p48_q20.log`
- `artifacts/recall100_rabitq_10k_n64_w750_p48_q20.log`
- `artifacts/recall10_rabitq_25k_n64_w750_p48_q20.log`
- `artifacts/recall100_rabitq_25k_n64_w750_p48_q20.log`
- `artifacts/latency_rabitq_10k_n64_w750_p48_i10_hwm.log`
- `artifacts/latency_rabitq_25k_n64_w750_p48_i10_hwm.log`
- `artifacts/rabitq_size.sql`
- `artifacts/rabitq_size.log`
- `artifacts/manifest.md`
