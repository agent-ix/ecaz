# Artifact Manifest

Packet: `30054-task28-ivf-nlists128-forced-index`

Head SHA: `210e4e8919f6311bf03a9740a7c87ddc1ce9b4d0`

Timestamp: `2026-04-27T14:16:44-07:00`

Lane: Task 28 IVF post-optimization nlists sweep

Fixture: local PG18 scratch, database `postgres`, DBPedia-derived 10k/25k x
1536 surfaces, 100 queries.

Storage format: `turboquant`

Rerank mode: `heap_f32`, `rerank_width=25`

Surface isolation: isolated one-index-per-table surfaces created in packet
30053.

Benchmark mode: `--force-index`

Cache state: warm local development run.

Memory high-water mark: not captured.

## Artifacts

- `recall_10k_n128w25_force_index.log`
  - Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database postgres bench recall --prefix task28_ivf_postopt10k_n128w25 --profile ec_ivf --k 10 --queries-limit 100 --sweep 8,16,24,32,48,64 --force-index`
  - Key lines:
    - `nprobe=16 recall@10=0.8540 mean=49.25 ms`
    - `nprobe=48 recall@10=0.9660 mean=86.31 ms`
    - `nprobe=64 recall@10=0.9860 mean=113.56 ms`

- `latency_10k_n128w25_force_index.log`
  - Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database postgres bench latency --prefix task28_ivf_postopt10k_n128w25 --profile ec_ivf --k 10 --concurrency 1 --iterations 100 --sweep 8,16,24,32,48,64 --force-index`
  - Key lines:
    - `nprobe=16 p50=42.5 ms p95=48.9 ms p99=50.9 ms`
    - `nprobe=48 p50=84.2 ms p95=90.6 ms p99=97.4 ms`
    - `nprobe=64 p50=104.7 ms p95=112.7 ms p99=116.1 ms`

- `recall_25k_n128w25_force_index.log`
  - Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database postgres bench recall --prefix task28_ivf_postopt25k_n128w25 --profile ec_ivf --k 10 --queries-limit 100 --sweep 8,16,24,32,48,64 --force-index`
  - Key lines:
    - `nprobe=16 recall@10=0.8090 mean=89.01 ms`
    - `nprobe=48 recall@10=0.9770 mean=202.66 ms`
    - `nprobe=64 recall@10=0.9860 mean=264.68 ms`

- `latency_25k_n128w25_force_index.log`
  - Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database postgres bench latency --prefix task28_ivf_postopt25k_n128w25 --profile ec_ivf --k 10 --concurrency 1 --iterations 100 --sweep 8,16,24,32,48,64 --force-index`
  - Key lines:
    - `nprobe=16 p50=88.3 ms p95=102.3 ms p99=118.2 ms`
    - `nprobe=48 p50=202.6 ms p95=233.6 ms p99=247.5 ms`
    - `nprobe=64 p50=256.2 ms p95=275.1 ms p99=287.0 ms`

## Validation

- `cargo test -p ecaz-cli build_knn_sql --no-default-features`
  - `3 passed; 0 failed`
- `cargo test -p ecaz-cli latency --no-default-features`
  - `13 passed; 0 failed`
- `cargo fmt --check`
- `git diff --check`
