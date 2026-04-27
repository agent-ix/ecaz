# Artifact Manifest

Packet: `30051-task28-ivf-postopt-smoke`

Head SHA: `f7799a759c4ba56441dc3ff14a48770d55a6b103`

Timestamp: `2026-04-27T13:26:31-07:00`

Lane: Task 28 IVF post-optimization smoke

Fixture: local PG18 scratch, database `postgres`, DBPedia-derived 10k/25k x
1536 surfaces, 100 queries.

Storage format: `turboquant`

Rerank mode: `heap_f32`, `rerank_width=25`

Surface isolation: isolated one-index-per-table surfaces.

Cache state: warm local development run. Latency followed build and recall on
the same scratch cluster.

Memory high-water mark: not captured.

## Artifacts

- `inspect_source_tables.sql`
  - Command: `cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30051-task28-ivf-postopt-smoke/artifacts/inspect_source_tables.sql --raw --log-output review/30051-task28-ivf-postopt-smoke/artifacts/inspect_source_tables.log`
  - Purpose: inspect source corpus/query tables and existing indexes.

- `inspect_truth_tables.sql`
  - Command: `cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30051-task28-ivf-postopt-smoke/artifacts/inspect_truth_tables.sql --raw --log-output review/30051-task28-ivf-postopt-smoke/artifacts/inspect_truth_tables.log`
  - Purpose: inspect query/truth helper tables before creating isolated
    surfaces.

- `create_postopt_n32w25_surfaces.sql`
  - Command: `cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30051-task28-ivf-postopt-smoke/artifacts/create_postopt_n32w25_surfaces.sql --raw --log-output review/30051-task28-ivf-postopt-smoke/artifacts/create_postopt_n32w25_surfaces.log`
  - Key lines:
    - 10k `CREATE INDEX`: `Time: 25875.514 ms (00:25.876)`
    - 25k `CREATE INDEX`: `Time: 47589.480 ms (00:47.589)`
    - 10k index size: `9160 kB`
    - 25k index size: `22 MB`

- `recall_10k_n32w25.log`
  - Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database postgres bench recall --prefix task28_ivf_postopt10k_n32w25 --profile ec_ivf --k 10 --queries-limit 100 --sweep 16,24,32`
  - Key lines:
    - `nprobe=16 recall@10=0.9800 mean=108.17 ms`
    - `nprobe=24 recall@10=0.9980 mean=142.24 ms`
    - `nprobe=32 recall@10=1.0000 mean=187.02 ms`

- `recall_25k_n32w25.log`
  - Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database postgres bench recall --prefix task28_ivf_postopt25k_n32w25 --profile ec_ivf --k 10 --queries-limit 100 --sweep 16,24,32`
  - Key lines:
    - `nprobe=16 recall@10=0.9480 mean=242.21 ms`
    - `nprobe=24 recall@10=0.9760 mean=341.40 ms`
    - `nprobe=32 recall@10=1.0000 mean=457.31 ms`

- `latency_10k_n32w25.log`
  - Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database postgres bench latency --prefix task28_ivf_postopt10k_n32w25 --profile ec_ivf --k 10 --concurrency 1 --iterations 100 --sweep 16,24,32`
  - Key lines:
    - `nprobe=16 p50=103.5 ms p95=114.4 ms p99=118.8 ms`
    - `nprobe=24 p50=141.3 ms p95=181.4 ms p99=197.9 ms`
    - `nprobe=32 p50=186.4 ms p95=223.8 ms p99=248.3 ms`

- `latency_25k_n32w25.log`
  - Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database postgres bench latency --prefix task28_ivf_postopt25k_n32w25 --profile ec_ivf --k 10 --concurrency 1 --iterations 100 --sweep 16,24,32`
  - Key lines:
    - `nprobe=16 p50=231.7 ms p95=266.7 ms p99=278.8 ms`
    - `nprobe=24 p50=336.8 ms p95=385.9 ms p99=473.9 ms`
    - `nprobe=32 p50=445.7 ms p95=457.3 ms p99=465.4 ms`

## Validation

- `cargo test -p ecaz-cli build_knn_sql --no-default-features`
  - `3 passed; 0 failed`
- `cargo test -p ecaz-cli latency --no-default-features`
  - `13 passed; 0 failed`
- `cargo fmt --check`
- `git diff --check`
