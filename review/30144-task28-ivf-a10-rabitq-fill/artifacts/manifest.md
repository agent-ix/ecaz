# Artifact Manifest: 30144 Task 28 IVF A10 RaBitQ Fill

## `build_rabitq_25k.sql`

- head SHA: `b5fe7b30`
- packet/topic: `30144-task28-ivf-a10-rabitq-fill`
- lane / fixture / storage format / rerank mode: A10 25k RaBitQ build, `nlists=64`, `nprobe=64`, `rerank=heap_f32`, `rerank_width=750`
- command: packet-local SQL input for `build_rabitq_25k.log`
- timestamp: 2026-04-29 local
- isolated/shared surface: isolated 25k RaBitQ surface cloned from existing 25k corpus/query tables
- key result lines: source SQL only

## `build_rabitq_25k.log`

- head SHA: `b5fe7b30`
- packet/topic: `30144-task28-ivf-a10-rabitq-fill`
- lane / fixture / storage format / rerank mode: A10 25k RaBitQ build, `nlists=64`, `nprobe=64`, `rerank=heap_f32`, `rerank_width=750`
- command: `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30144-task28-ivf-a10-rabitq-fill/artifacts/build_rabitq_25k.sql --raw --log-output review/30144-task28-ivf-a10-rabitq-fill/artifacts/build_rabitq_25k.log`
- timestamp: 2026-04-29 local
- isolated/shared surface: isolated 25k RaBitQ surface cloned from existing 25k corpus/query tables
- key result lines:
  - `task28_ivf_qcmp25k_rabitq_idx build_ms=40699.103`
  - `index_bytes=23519232`, `index_size=22 MB`

## `recall10_rabitq_10k_n64_w750_p48_q20.log`

- head SHA: `b5fe7b30`
- packet/topic: `30144-task28-ivf-a10-rabitq-fill`
- lane / fixture / storage format / rerank mode: A10 10k RaBitQ recall@10, `nprobe=48`, `rerank_width=750`
- command: `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_qcmp10k_rabitq --profile ec_ivf --k 10 --queries-limit 20 --sweep 48 --rerank-width 750 --force-index --log-output review/30144-task28-ivf-a10-rabitq-fill/artifacts/recall10_rabitq_10k_n64_w750_p48_q20.log`
- timestamp: 2026-04-29 local
- isolated/shared surface: isolated existing 10k RaBitQ surface
- key result lines: `recall@k=1.0000`, `ndcg@k=1.0000`, `mean q-time=1940.08 ms`

## `recall100_rabitq_10k_n64_w750_p48_q20.log`

- head SHA: `b5fe7b30`
- packet/topic: `30144-task28-ivf-a10-rabitq-fill`
- lane / fixture / storage format / rerank mode: A10 10k RaBitQ recall@100, `nprobe=48`, `rerank_width=750`
- command: `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_qcmp10k_rabitq --profile ec_ivf --k 100 --queries-limit 20 --sweep 48 --rerank-width 750 --force-index --log-output review/30144-task28-ivf-a10-rabitq-fill/artifacts/recall100_rabitq_10k_n64_w750_p48_q20.log`
- timestamp: 2026-04-29 local
- isolated/shared surface: isolated existing 10k RaBitQ surface
- key result lines: `recall@k=0.9930`, `ndcg@k=0.9995`, `mean q-time=1953.62 ms`

## `recall10_rabitq_25k_n64_w750_p48_q20.log`

- head SHA: `b5fe7b30`
- packet/topic: `30144-task28-ivf-a10-rabitq-fill`
- lane / fixture / storage format / rerank mode: A10 25k RaBitQ recall@10, `nprobe=48`, `rerank_width=750`
- command: `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_qcmp25k_rabitq --profile ec_ivf --k 10 --queries-limit 20 --sweep 48 --rerank-width 750 --force-index --log-output review/30144-task28-ivf-a10-rabitq-fill/artifacts/recall10_rabitq_25k_n64_w750_p48_q20.log`
- timestamp: 2026-04-29 local
- isolated/shared surface: isolated 25k RaBitQ surface
- key result lines: `recall@k=1.0000`, `ndcg@k=1.0000`, `mean q-time=5015.51 ms`

## `recall100_rabitq_25k_n64_w750_p48_q20.log`

- head SHA: `b5fe7b30`
- packet/topic: `30144-task28-ivf-a10-rabitq-fill`
- lane / fixture / storage format / rerank mode: A10 25k RaBitQ recall@100, `nprobe=48`, `rerank_width=750`
- command: `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_qcmp25k_rabitq --profile ec_ivf --k 100 --queries-limit 20 --sweep 48 --rerank-width 750 --force-index --log-output review/30144-task28-ivf-a10-rabitq-fill/artifacts/recall100_rabitq_25k_n64_w750_p48_q20.log`
- timestamp: 2026-04-29 local
- isolated/shared surface: isolated 25k RaBitQ surface
- key result lines: `recall@k=0.9915`, `ndcg@k=0.9997`, `mean q-time=5043.47 ms`

## `latency_rabitq_10k_n64_w750_p48_i10_hwm.log`

- head SHA: `b5fe7b30`
- packet/topic: `30144-task28-ivf-a10-rabitq-fill`
- lane / fixture / storage format / rerank mode: A10 10k RaBitQ latency/HWM, `nprobe=48`, `rerank_width=750`
- command: `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench latency --prefix task28_ivf_qcmp10k_rabitq --profile ec_ivf --k 10 --iterations 10 --sweep 48 --rerank-width 750 --force-index --sample-backend-memory --memory-sample-interval-ms 25 --log-output review/30144-task28-ivf-a10-rabitq-fill/artifacts/latency_rabitq_10k_n64_w750_p48_i10_hwm.log`
- timestamp: 2026-04-29 local
- isolated/shared surface: isolated existing 10k RaBitQ surface
- key result lines: `p50=1947.8 ms`, `p95=2096.9 ms`, `p99=2128.3 ms`, `hwm_peak_kb=69980`

## `latency_rabitq_25k_n64_w750_p48_i10_hwm.log`

- head SHA: `b5fe7b30`
- packet/topic: `30144-task28-ivf-a10-rabitq-fill`
- lane / fixture / storage format / rerank mode: A10 25k RaBitQ latency/HWM, `nprobe=48`, `rerank_width=750`
- command: `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench latency --prefix task28_ivf_qcmp25k_rabitq --profile ec_ivf --k 10 --iterations 10 --sweep 48 --rerank-width 750 --force-index --sample-backend-memory --memory-sample-interval-ms 25 --log-output review/30144-task28-ivf-a10-rabitq-fill/artifacts/latency_rabitq_25k_n64_w750_p48_i10_hwm.log`
- timestamp: 2026-04-29 local
- isolated/shared surface: isolated 25k RaBitQ surface
- key result lines: `p50=4973.0 ms`, `p95=5257.7 ms`, `p99=5327.9 ms`, `hwm_peak_kb=145012`

## `rabitq_size.sql`

- head SHA: `b5fe7b30`
- packet/topic: `30144-task28-ivf-a10-rabitq-fill`
- lane / fixture / storage format / rerank mode: A10 RaBitQ size SQL
- command: packet-local SQL input for `rabitq_size.log`
- timestamp: 2026-04-29 local
- isolated/shared surface: isolated 10k and 25k RaBitQ surfaces
- key result lines: source SQL only

## `rabitq_size.log`

- head SHA: `b5fe7b30`
- packet/topic: `30144-task28-ivf-a10-rabitq-fill`
- lane / fixture / storage format / rerank mode: A10 RaBitQ size rows
- command: `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30144-task28-ivf-a10-rabitq-fill/artifacts/rabitq_size.sql --raw --log-output review/30144-task28-ivf-a10-rabitq-fill/artifacts/rabitq_size.log`
- timestamp: 2026-04-29 local
- isolated/shared surface: isolated 10k and 25k RaBitQ surfaces
- key result lines:
  - 10k `index_bytes=9641984`, `index_size=9416 kB`
  - 25k `index_bytes=23519232`, `index_size=22 MB`
