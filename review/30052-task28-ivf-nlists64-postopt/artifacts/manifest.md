# Artifact Manifest

Packet: `30052-task28-ivf-nlists64-postopt`

Head SHA: `bd9f785d2b08f2989433b77a24486067b74148e2`

Timestamp: `2026-04-27T13:28:54-07:00`

Lane: Task 28 IVF post-optimization nlists sweep

Fixture: local PG18 scratch, database `postgres`, DBPedia-derived 10k/25k x
1536 surfaces, 100 queries.

Storage format: `turboquant`

Rerank mode: `heap_f32`, `rerank_width=25`

Surface isolation: isolated one-index-per-table surfaces.

Cache state: warm local development run. Latency followed build and recall on
the same scratch cluster.

Memory high-water mark: not captured.

## Artifacts

- `create_postopt_n64w25_surfaces.sql`
  - Command: `cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30052-task28-ivf-nlists64-postopt/artifacts/create_postopt_n64w25_surfaces.sql --raw --log-output review/30052-task28-ivf-nlists64-postopt/artifacts/create_postopt_n64w25_surfaces.log`
  - Key lines:
    - 10k `CREATE INDEX`: `Time: 44356.259 ms (00:44.356)`
    - 25k `CREATE INDEX`: `Time: 76271.757 ms (01:16.272)`
    - 10k index size: `9416 kB`
    - 25k index size: `22 MB`

- `recall_10k_n64w25.log`
  - Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database postgres bench recall --prefix task28_ivf_postopt10k_n64w25 --profile ec_ivf --k 10 --queries-limit 100 --sweep 8,16,24,32,48`
  - Key lines:
    - `nprobe=8 recall@10=0.7800 mean=35.45 ms`
    - `nprobe=32 recall@10=0.9800 mean=99.52 ms`
    - `nprobe=48 recall@10=1.0000 mean=139.94 ms`

- `recall_25k_n64w25.log`
  - Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database postgres bench recall --prefix task28_ivf_postopt25k_n64w25 --profile ec_ivf --k 10 --queries-limit 100 --sweep 8,16,24,32,48`
  - Key lines:
    - `nprobe=8 recall@10=0.8130 mean=79.40 ms`
    - `nprobe=32 recall@10=0.9840 mean=248.14 ms`
    - `nprobe=48 recall@10=0.9990 mean=353.31 ms`

- `latency_10k_n64w25.log`
  - Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database postgres bench latency --prefix task28_ivf_postopt10k_n64w25 --profile ec_ivf --k 10 --concurrency 1 --iterations 100 --sweep 8,16,24,32,48`
  - Key lines:
    - `nprobe=8 p50=34.9 ms p95=41.1 ms p99=47.0 ms`
    - `nprobe=32 p50=98.1 ms p95=105.9 ms p99=111.7 ms`
    - `nprobe=48 p50=140.2 ms p95=148.2 ms p99=158.4 ms`

- `latency_25k_n64w25.log`
  - Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database postgres bench latency --prefix task28_ivf_postopt25k_n64w25 --profile ec_ivf --k 10 --concurrency 1 --iterations 100 --sweep 8,16,24,32,48`
  - Key lines:
    - `nprobe=8 p50=79.9 ms p95=92.3 ms p99=97.3 ms`
    - `nprobe=32 p50=246.2 ms p95=261.0 ms p99=264.5 ms`
    - `nprobe=48 p50=351.4 ms p95=383.2 ms p99=584.5 ms`
