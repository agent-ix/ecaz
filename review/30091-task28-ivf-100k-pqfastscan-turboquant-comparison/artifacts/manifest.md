# Artifact Manifest

Head SHA: `d7ea2f7`

Packet: `review/30091-task28-ivf-100k-pqfastscan-turboquant-comparison`

Timestamp: 2026-04-28 03:25 America/Los_Angeles

Fixture:

- Source for both surfaces: `task28_ivf_pqg100k_g8_corpus` and
  `task28_ivf_pqg100k_g8_queries`, derived in packet 30090 from the local
  990k DBPedia anchor.
- Derived TurboQuant prefix: `task28_ivf_tq100k_n64w25`
- Rows: 100000 corpus rows, 100 query rows
- Dimensions: 1536
- Isolated one-index-per-table surfaces: yes
- nlists: 64
- Runtime nprobe sweep: 32, 48
- Cache state: warm local development run; no explicit cache drop
- Memory high-water mark: not captured

## build_turboquant_100k_surface.sql

Command:

`cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30091-task28-ivf-100k-pqfastscan-turboquant-comparison/artifacts/build_turboquant_100k_surface.sql --raw --log-output review/30091-task28-ivf-100k-pqfastscan-turboquant-comparison/artifacts/build_turboquant_100k_surface.log`

Key lines:

- `SELECT 100000` in `25969.616 ms`
- `CREATE INDEX` in `136602.323 ms`
- `task28_ivf_tq100k_n64w25_idx | 87 MB | {nlists=64,nprobe=64,training_sample_rows=2000,storage_format=turboquant,rerank=heap_f32,rerank_width=25}`

## recall_turboquant_100k_n64w25.log

Command:

`cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_tq100k_n64w25 --profile ec_ivf --k 10 --queries-limit 100 --sweep 32,48 --force-index --log-output review/30091-task28-ivf-100k-pqfastscan-turboquant-comparison/artifacts/recall_turboquant_100k_n64w25.log`

Key lines:

- Ground truth: `100 queries vs 100000 corpus rows (dim=1536)`, `20.40s`
- `32 | 0.9930 | 0.9998 | 466.29 ms`
- `48 | 1.0000 | 1.0000 | 712.49 ms`

## latency_turboquant_100k_n64w25.log

Command:

`cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench latency --prefix task28_ivf_tq100k_n64w25 --profile ec_ivf --k 10 --iterations 100 --sweep 32,48 --force-index --log-output review/30091-task28-ivf-100k-pqfastscan-turboquant-comparison/artifacts/latency_turboquant_100k_n64w25.log`

Key lines:

- `32 | 100 | 469.7 ms | 43.5 ms | 389.1 ms | 464.8 ms | 538.0 ms | 556.8 ms | 585.1 ms`
- `48 | 100 | 707.5 ms | 32.0 ms | 629.1 ms | 705.7 ms | 760.6 ms | 782.7 ms | 854.8 ms`

## Packet 30090 PQ-FastScan Reference

Referenced packet-local logs:

- `review/30090-task28-ivf-pqfastscan-g8-100k-smoke/artifacts/build_g8_100k_surface.log`
- `review/30090-task28-ivf-pqfastscan-g8-100k-smoke/artifacts/recall_g8_100k_w750.log`
- `review/30090-task28-ivf-pqfastscan-g8-100k-smoke/artifacts/latency_g8_100k_w750.log`

Key lines:

- Build: `CREATE INDEX` in `156088.030 ms`
- Size: `task28_ivf_pqg100k_g8_idx | 18 MB`
- Recall: `32 | 0.9930 | 0.9998 | 286.73 ms`
- Recall: `48 | 1.0000 | 1.0000 | 410.19 ms`
- Latency: `32 | 100 | 280.5 ms | 21.2 ms | 240.7 ms | 279.5 ms | 312.5 ms | 323.1 ms | 335.4 ms`
- Latency: `48 | 100 | 409.2 ms | 20.3 ms | 373.3 ms | 407.6 ms | 439.6 ms | 496.1 ms | 500.0 ms`
