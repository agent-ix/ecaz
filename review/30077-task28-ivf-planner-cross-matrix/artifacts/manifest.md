# Artifacts Manifest

Packet: `30077-task28-ivf-planner-cross-matrix`

Head SHA: `727a3fb2795c13be63b2f8c0a2fb89cb50da72bc`

Timestamp: `2026-04-27T19:06:14-07:00`

Lane: Task 28 A6 planner cross-test matrix.

Fixture: local PG18, existing 10k DBPedia-derived `task28_ivf_postopt10k_n128w25_corpus` table.

Storage format: IVF TurboQuant.

Rerank mode: existing n128/w25 surface.

Isolation: existing isolated one-index-per-table IVF surface, with an additional same-column HNSW comparison index created by this packet.

## `planner_cross_matrix.sql`

- SQL script used for planner matrix.
- Creates `task28_ivf_postopt10k_n128w25_hnsw_idx` if missing.
- Uses `LOAD 'ecaz'`, PG18 default cost constants, `enable_seqscan=on`, `ec_ivf.nprobe=32`, and `ec_hnsw.ef_search=64`.

## `planner_cross_matrix.log`

- Command: `cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30077-task28-ivf-planner-cross-matrix/artifacts/planner_cross_matrix.sql --raw --log-output review/30077-task28-ivf-planner-cross-matrix/artifacts/planner_cross_matrix.log`
- Key setup lines:
  - `task28_ivf_postopt10k_n128w25_hnsw_idx | ec_hnsw | 13664256`
  - `task28_ivf_postopt10k_n128w25_idx | ec_ivf | 10166272`
- Key result lines:
  - Shape 1: `Index Scan using task28_ivf_postopt10k_n128w25_idx ... (cost=105.92..1022.27 rows=10000 width=12)` and `Execution Time: 50.003 ms`
  - Shape 2: `Index Scan using task28_ivf_postopt10k_n128w25_idx ... (cost=105.92..1022.27 rows=10000 width=12)` and `Execution Time: 54.043 ms`
  - Shape 3: `Index Scan using task28_ivf_postopt10k_n128w25_corpus_pkey ... (cost=0.29..49.30 rows=1001 width=12)` and `Execution Time: 467.404 ms`
  - Shape 4: `Index Only Scan using task28_ivf_postopt10k_n128w25_corpus_pkey ... (cost=0.29..6.05 rows=101 width=0)` and `Execution Time: 0.096 ms`
  - Shape 5: `Seq Scan on task28_ivf_postopt10k_n128w25_corpus ... (cost=0.00..219.00 rows=9999 width=0)` and `Execution Time: 1.779 ms`

## Validation Command

- `cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30077-task28-ivf-planner-cross-matrix/artifacts/planner_cross_matrix.sql --raw --log-output review/30077-task28-ivf-planner-cross-matrix/artifacts/planner_cross_matrix.log`
