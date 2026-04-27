# Artifact Manifest

Packet: `30053-task28-ivf-nlists128-postopt`

Head SHA: `fc7809ba5bf00195d50be21801147385379018ed`

Timestamp: `2026-04-27T13:40:04-07:00`

Lane: Task 28 IVF post-optimization nlists sweep / planner blocker

Fixture: local PG18 scratch, database `postgres`, DBPedia-derived 10k/25k x
1536 surfaces, 100-query fixture.

Storage format: `turboquant`

Rerank mode: `heap_f32`, `rerank_width=25`

Surface isolation: isolated one-index-per-table surfaces.

Cache state: warm local development run.

Memory high-water mark: not captured.

## Artifacts

- `create_postopt_n128w25_surfaces.sql`
  - Command: `cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30053-task28-ivf-nlists128-postopt/artifacts/create_postopt_n128w25_surfaces.sql --raw --log-output review/30053-task28-ivf-nlists128-postopt/artifacts/create_postopt_n128w25_surfaces.log`
  - Key lines:
    - 10k `CREATE INDEX`: `Time: 80349.361 ms (01:20.349)`
    - 25k `CREATE INDEX`: `Time: 134007.631 ms (02:14.008)`
    - 10k index size: `9928 kB`
    - 25k index size: `23 MB`

- `recall_10k_n128w25_aborted.log`
  - Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database postgres bench recall --prefix task28_ivf_postopt10k_n128w25 --profile ec_ivf --k 10 --queries-limit 100 --sweep 8,16,24,32,48,64`
  - Key lines:
    - Ground truth completed in `2.04s`.
    - No result table finished after several minutes.
    - The process was killed locally.

- `explain_n128_single_query.sql`
  - Command: `cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30053-task28-ivf-nlists128-postopt/artifacts/explain_n128_single_query.sql --raw --log-output review/30053-task28-ivf-nlists128-postopt/artifacts/explain_n128_single_query.log`
  - Key lines:
    - Plan: `Seq Scan on task28_ivf_postopt10k_n128w25_corpus`
    - Execution time: `4426.320 ms`
    - Buffers: `shared hit=50145 read=6`

- `explain_n128_single_query_index_forced.sql`
  - Command: `cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30053-task28-ivf-nlists128-postopt/artifacts/explain_n128_single_query_index_forced.sql --raw --log-output review/30053-task28-ivf-nlists128-postopt/artifacts/explain_n128_single_query_index_forced.log`
  - Key lines:
    - Plan: `Index Scan using task28_ivf_postopt10k_n128w25_idx`
    - Execution time: `67.987 ms`
    - Buffers: `shared hit=370 read=192`
