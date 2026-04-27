# Artifact Manifest

Packet: `30055-task28-ivf-rerank-width-postopt`

Head SHA: `d464780d99e0c16d2066d260b5812cdc725c9770`

Timestamp: `2026-04-27T14:25:38-07:00`

Lane: Task 28 IVF post-optimization rerank-width sweep

Fixture: local PG18 scratch, database `postgres`, DBPedia-derived 10k/25k x
1536 surfaces, 100 queries.

Storage format: `turboquant`

Rerank mode: `heap_f32`, swept from `rerank_width=25` baseline to
`rerank_width=10` and a 10k-only `rerank_width=5` spot check.

Surface isolation: isolated one-index-per-table n64 surfaces created in
packet 30052.

Benchmark mode: `--force-index`

Cache state: warm local development run.

Memory high-water mark: not captured.

## Artifacts

- `alter_n64_width10.sql`, `alter_n64_width10.log`
  - Command: `cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30055-task28-ivf-rerank-width-postopt/artifacts/alter_n64_width10.sql --raw --log-output review/30055-task28-ivf-rerank-width-postopt/artifacts/alter_n64_width10.log`
  - Key line: both n64 indexes reported `rerank_width=10`.

- `recall_10k_n64w10_force_index.log`
  - Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database postgres bench recall --prefix task28_ivf_postopt10k_n64w25 --profile ec_ivf --k 10 --queries-limit 100 --sweep 32,48 --force-index`
  - Key lines:
    - `nprobe=32 recall@10=0.9260 mean=99.72 ms`
    - `nprobe=48 recall@10=0.9370 mean=139.37 ms`

- `latency_10k_n64w10_force_index.log`
  - Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database postgres bench latency --prefix task28_ivf_postopt10k_n64w25 --profile ec_ivf --k 10 --concurrency 1 --iterations 100 --sweep 32,48 --force-index`
  - Key lines:
    - `nprobe=32 p50=96.1 ms p95=105.0 ms p99=107.0 ms`
    - `nprobe=48 p50=138.9 ms p95=149.3 ms p99=152.0 ms`

- `recall_25k_n64w10_force_index.log`
  - Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database postgres bench recall --prefix task28_ivf_postopt25k_n64w25 --profile ec_ivf --k 10 --queries-limit 100 --sweep 32,48 --force-index`
  - Key lines:
    - `nprobe=32 recall@10=0.9280 mean=246.19 ms`
    - `nprobe=48 recall@10=0.9390 mean=344.30 ms`

- `latency_25k_n64w10_force_index.log`
  - Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database postgres bench latency --prefix task28_ivf_postopt25k_n64w25 --profile ec_ivf --k 10 --concurrency 1 --iterations 100 --sweep 32,48 --force-index`
  - Key lines:
    - `nprobe=32 p50=244.6 ms p95=268.2 ms p99=296.1 ms`
    - `nprobe=48 p50=348.5 ms p95=362.5 ms p99=369.0 ms`

- `alter_n64_width5.sql`, `alter_n64_width5.log`
  - Command: `cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30055-task28-ivf-rerank-width-postopt/artifacts/alter_n64_width5.sql --raw --log-output review/30055-task28-ivf-rerank-width-postopt/artifacts/alter_n64_width5.log`
  - Key line: both n64 indexes reported `rerank_width=5`.

- `recall_10k_n64w5_force_index.log`
  - Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database postgres bench recall --prefix task28_ivf_postopt10k_n64w25 --profile ec_ivf --k 10 --queries-limit 100 --sweep 32,48 --force-index`
  - Key lines:
    - `nprobe=32 recall@10=0.5000 mean=95.34 ms`
    - `nprobe=48 recall@10=0.5000 mean=140.55 ms`

- `latency_10k_n64w5_force_index.log`
  - Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database postgres bench latency --prefix task28_ivf_postopt10k_n64w25 --profile ec_ivf --k 10 --concurrency 1 --iterations 100 --sweep 32,48 --force-index`
  - Key lines:
    - `nprobe=32 p50=93.7 ms p95=102.4 ms p99=108.4 ms`
    - `nprobe=48 p50=135.5 ms p95=143.0 ms p99=148.0 ms`

- `restore_n64_width25.sql`, `restore_n64_width25.log`
  - Command: `cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30055-task28-ivf-rerank-width-postopt/artifacts/restore_n64_width25.sql --raw --log-output review/30055-task28-ivf-rerank-width-postopt/artifacts/restore_n64_width25.log`
  - Key line: both n64 indexes restored to `rerank_width=25`.

## Validation

- `git diff --check`
