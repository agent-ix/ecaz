# Artifact Manifest

Head SHA: `1ea304d393161c473acb4e632577ee0d842cc719`

Packet: `review/30094-task28-ivf-pqfastscan-g8-100k-n128-nprobe-middle`

Timestamp: 2026-04-28 05:05 America/Los_Angeles

Fixture:

- Source prefix: `task28_ivf_pqg100k_g8_n128`
- Rows: 100000 corpus rows, 100 query rows
- Dimensions: 1536
- Isolated one-index-per-table surface: yes
- Storage format: `pq_fastscan`
- PQ group size: 8
- Rerank mode: `heap_f32`
- Rerank width: 500
- nlists: 128
- Runtime nprobe sweep: 40, 48, 56, 64
- Cache state: warm local development run; no explicit cache drop
- Memory high-water mark: not captured

## set_g8_100k_n128_width500.sql

Command:

`cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30094-task28-ivf-pqfastscan-g8-100k-n128-nprobe-middle/artifacts/set_g8_100k_n128_width500.sql --raw --log-output review/30094-task28-ivf-pqfastscan-g8-100k-n128-nprobe-middle/artifacts/set_g8_100k_n128_width500.log`

Key lines:

- `ALTER INDEX`
- `task28_ivf_pqg100k_g8_n128_idx | {nlists=128,nprobe=128,training_sample_rows=2000,storage_format=pq_fastscan,pq_group_size=8,rerank=heap_f32,rerank_width=500}`

## recall_g8_100k_n128_w500_p40_48_56_64.log

Command:

`cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg100k_g8_n128 --profile ec_ivf --k 10 --queries-limit 100 --sweep 40,48,56,64 --force-index --log-output review/30094-task28-ivf-pqfastscan-g8-100k-n128-nprobe-middle/artifacts/recall_g8_100k_n128_w500_p40_48_56_64.log`

Key lines:

- Ground truth: `100 queries vs 100000 corpus rows (dim=1536)`, `20.49s`
- `40 | 0.9810 | 0.9993 | 204.82 ms`
- `48 | 0.9920 | 0.9997 | 234.61 ms`
- `56 | 0.9930 | 0.9997 | 265.49 ms`
- `64 | 0.9940 | 0.9997 | 295.03 ms`

## latency_g8_100k_n128_w500_p40_48_56_64.log

Command:

`cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench latency --prefix task28_ivf_pqg100k_g8_n128 --profile ec_ivf --k 10 --iterations 100 --sweep 40,48,56,64 --force-index --log-output review/30094-task28-ivf-pqfastscan-g8-100k-n128-nprobe-middle/artifacts/latency_g8_100k_n128_w500_p40_48_56_64.log`

Key lines:

- `40 | 100 | 207.2 ms | 16.6 ms | 177.1 ms | 205.8 ms | 233.8 ms | 255.8 ms | 258.9 ms`
- `48 | 100 | 237.0 ms | 15.0 ms | 205.4 ms | 238.5 ms | 262.1 ms | 274.7 ms | 275.8 ms`
- `56 | 100 | 269.9 ms | 21.5 ms | 234.9 ms | 266.3 ms | 309.5 ms | 356.0 ms | 357.5 ms`
- `64 | 100 | 296.1 ms | 15.4 ms | 261.3 ms | 295.8 ms | 322.5 ms | 332.8 ms | 333.2 ms`
