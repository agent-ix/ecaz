# Artifact Manifest

Packet: `review/30071-task28-ivf-n128-cost-diagnostic`

Measured code SHA: `de05e740651b5fee508a8eabdd80d65c8b92e086`

Timestamp: `2026-04-27T17:16:22-07:00`

Lane: Task 28 IVF n128 planner-cost diagnostic, local PG18 only.

Fixture: existing isolated DBPedia-derived 10k n64/n128 surfaces in local PG18
database `postgres`; prepared SQL KNN query over
`task28_ivf_postopt10k_n128w25_corpus`.

Storage format: `turboquant`.

Rerank mode: `heap_f32`, `rerank_width=25`.

Surface isolation: one-index-per-table surfaces from earlier Task 28 packets.

Cache state: warm local development run; no explicit cache drop.

Memory high-water mark: not captured.

## Artifacts

### `n128_cost_diagnostic.sql`

- SQL script used for the planner diagnostic.
- Uses `LOAD 'ecaz'`, standard PG18 cost constants, `enable_seqscan=on`, and
  `EXPLAIN (ANALYZE, BUFFERS)` for `ec_ivf.nprobe` values `8,16,24,32`.

### `n128_cost_diagnostic.log`

- Command: `cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30071-task28-ivf-n128-cost-diagnostic/artifacts/n128_cost_diagnostic.sql --raw --log-output review/30071-task28-ivf-n128-cost-diagnostic/artifacts/n128_cost_diagnostic.log`
- Key result lines cited:
  - `normal planner explain n128 nprobe 8`
  - `Index Scan using task28_ivf_postopt10k_n128w25_idx`
  - `cost=43.00..594.25`
  - `Execution Time: 77.796 ms`
  - `normal planner explain n128 nprobe 16`
  - `Index Scan using task28_ivf_postopt10k_n128w25_idx`
  - `cost=43.00..644.51`
  - `Execution Time: 50.494 ms`
  - `normal planner explain n128 nprobe 24`
  - `Index Scan using task28_ivf_postopt10k_n128w25_idx`
  - `cost=43.00..694.77`
  - `Execution Time: 59.792 ms`
  - `normal planner explain n128 nprobe 32`
  - `Index Scan using task28_ivf_postopt10k_n128w25_idx`
  - `cost=43.00..745.03`
  - `Execution Time: 67.941 ms`
