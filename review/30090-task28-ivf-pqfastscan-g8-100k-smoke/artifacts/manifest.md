# Artifact Manifest

Head SHA: `243121f`

Packet: `review/30090-task28-ivf-pqfastscan-g8-100k-smoke`

Timestamp: 2026-04-28 02:45 America/Los_Angeles

Fixture:

- Source corpus: `ec_hnsw_real_ann_benchmarks_anchor_corpus`
- Source queries: `ec_hnsw_real_ann_benchmarks_anchor_queries`
- Derived prefix: `task28_ivf_pqg100k_g8`
- Rows: 100000 corpus rows, 100 query rows
- Dimensions: 1536
- Isolated one-index-per-table surface: yes
- Storage format: `pq_fastscan`
- PQ group size: 8
- Rerank mode: `heap_f32`
- Rerank width: 750
- nlists: 64
- Runtime nprobe sweep: 32, 48
- Cache state: warm local development run; no explicit cache drop
- Memory high-water mark: not captured

## inspect_100k_tables.sql

Command:

`cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30090-task28-ivf-pqfastscan-g8-100k-smoke/artifacts/inspect_100k_tables.sql --raw --log-output review/30090-task28-ivf-pqfastscan-g8-100k-smoke/artifacts/inspect_100k_tables.log`

Key lines:

- No existing `task28...100k` tables were present.
- `ec_hnsw_real_ann_benchmarks_anchor_corpus | 990000`

## inspect_anchor_schema.sql

Command:

`cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30090-task28-ivf-pqfastscan-g8-100k-smoke/artifacts/inspect_anchor_schema.sql --raw --log-output review/30090-task28-ivf-pqfastscan-g8-100k-smoke/artifacts/inspect_anchor_schema.log`

Key lines:

- `ec_hnsw_real_ann_benchmarks_anchor_corpus | 990000`
- `ec_hnsw_real_ann_benchmarks_anchor_queries | 10000`
- Corpus columns: `id bigint`, `source real[]`, `embedding ecvector`
- Query columns: `id bigint`, `source real[]`

## build_g8_100k_surface.sql

Command:

`cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30090-task28-ivf-pqfastscan-g8-100k-smoke/artifacts/build_g8_100k_surface.sql --raw --log-output review/30090-task28-ivf-pqfastscan-g8-100k-smoke/artifacts/build_g8_100k_surface.log`

Key lines:

- `SELECT 100000` in `36328.304 ms`
- `CREATE INDEX` in `156088.030 ms`
- `task28_ivf_pqg100k_g8_idx | 18 MB | {nlists=64,nprobe=64,training_sample_rows=2000,storage_format=pq_fastscan,pq_group_size=8,rerank=heap_f32,rerank_width=750}`

## recall_g8_100k_w750.log

Command:

`cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg100k_g8 --profile ec_ivf --k 10 --queries-limit 100 --sweep 32,48 --force-index --log-output review/30090-task28-ivf-pqfastscan-g8-100k-smoke/artifacts/recall_g8_100k_w750.log`

Key lines:

- Ground truth: `100 queries vs 100000 corpus rows (dim=1536)`, `20.53s`
- `32 | 0.9930 | 0.9998 | 286.73 ms`
- `48 | 1.0000 | 1.0000 | 410.19 ms`

## latency_g8_100k_w750.log

Command:

`cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench latency --prefix task28_ivf_pqg100k_g8 --profile ec_ivf --k 10 --iterations 100 --sweep 32,48 --force-index --log-output review/30090-task28-ivf-pqfastscan-g8-100k-smoke/artifacts/latency_g8_100k_w750.log`

Key lines:

- `32 | 100 | 280.5 ms | 21.2 ms | 240.7 ms | 279.5 ms | 312.5 ms | 323.1 ms | 335.4 ms`
- `48 | 100 | 409.2 ms | 20.3 ms | 373.3 ms | 407.6 ms | 439.6 ms | 496.1 ms | 500.0 ms`
