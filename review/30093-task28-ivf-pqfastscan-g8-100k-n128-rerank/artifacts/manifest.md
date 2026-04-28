# Artifact Manifest

Head SHA: `b2108e72ab2227e0fe16b6aec9f50c28a89090bb`

Packet: `review/30093-task28-ivf-pqfastscan-g8-100k-n128-rerank`

Timestamp: 2026-04-28 04:45 America/Los_Angeles

Fixture:

- Source prefix: `task28_ivf_pqg100k_g8_n128`
- Rows: 100000 corpus rows, 100 query rows
- Dimensions: 1536
- Isolated one-index-per-table surface: yes
- Storage format: `pq_fastscan`
- PQ group size: 8
- Rerank mode: `heap_f32`
- nlists: 128
- Runtime nprobe sweep: 48, 64
- Cache state: warm local development run; no explicit cache drop
- Memory high-water mark: not captured

## set_g8_100k_n128_width500.sql

Command:

`cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30093-task28-ivf-pqfastscan-g8-100k-n128-rerank/artifacts/set_g8_100k_n128_width500.sql --raw --log-output review/30093-task28-ivf-pqfastscan-g8-100k-n128-rerank/artifacts/set_g8_100k_n128_width500.log`

Key lines:

- `ALTER INDEX`
- `task28_ivf_pqg100k_g8_n128_idx | {nlists=128,nprobe=128,training_sample_rows=2000,storage_format=pq_fastscan,pq_group_size=8,rerank=heap_f32,rerank_width=500}`

## recall_g8_100k_n128_w500.log

Command:

`cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg100k_g8_n128 --profile ec_ivf --k 10 --queries-limit 100 --sweep 48,64 --force-index --log-output review/30093-task28-ivf-pqfastscan-g8-100k-n128-rerank/artifacts/recall_g8_100k_n128_w500.log`

Key lines:

- `48 | 0.9920 | 0.9997 | 238.14 ms`
- `64 | 0.9940 | 0.9997 | 295.79 ms`

## set_g8_100k_n128_width625.sql

Command:

`cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30093-task28-ivf-pqfastscan-g8-100k-n128-rerank/artifacts/set_g8_100k_n128_width625.sql --raw --log-output review/30093-task28-ivf-pqfastscan-g8-100k-n128-rerank/artifacts/set_g8_100k_n128_width625.log`

Key lines:

- `ALTER INDEX`
- `task28_ivf_pqg100k_g8_n128_idx | {nlists=128,nprobe=128,training_sample_rows=2000,storage_format=pq_fastscan,pq_group_size=8,rerank=heap_f32,rerank_width=625}`

## recall_g8_100k_n128_w625.log

Command:

`cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg100k_g8_n128 --profile ec_ivf --k 10 --queries-limit 100 --sweep 48,64 --force-index --log-output review/30093-task28-ivf-pqfastscan-g8-100k-n128-rerank/artifacts/recall_g8_100k_n128_w625.log`

Key lines:

- `48 | 0.9920 | 0.9997 | 242.22 ms`
- `64 | 0.9940 | 0.9997 | 299.33 ms`

## set_g8_100k_n128_width500_for_latency.log

Command:

`cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30093-task28-ivf-pqfastscan-g8-100k-n128-rerank/artifacts/set_g8_100k_n128_width500.sql --raw --log-output review/30093-task28-ivf-pqfastscan-g8-100k-n128-rerank/artifacts/set_g8_100k_n128_width500_for_latency.log`

Key lines:

- `ALTER INDEX`
- `task28_ivf_pqg100k_g8_n128_idx | {nlists=128,nprobe=128,training_sample_rows=2000,storage_format=pq_fastscan,pq_group_size=8,rerank=heap_f32,rerank_width=500}`

## latency_g8_100k_n128_w500.log

Command:

`cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench latency --prefix task28_ivf_pqg100k_g8_n128 --profile ec_ivf --k 10 --iterations 100 --sweep 48,64 --force-index --log-output review/30093-task28-ivf-pqfastscan-g8-100k-n128-rerank/artifacts/latency_g8_100k_n128_w500.log`

Key lines:

- `48 | 100 | 237.7 ms | 18.3 ms | 207.6 ms | 236.0 ms | 273.8 ms | 284.9 ms | 307.8 ms`
- `64 | 100 | 295.9 ms | 14.1 ms | 265.0 ms | 295.9 ms | 323.7 ms | 330.3 ms | 337.6 ms`

## set_g8_100k_n128_width750_restore.log

Command:

`cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30093-task28-ivf-pqfastscan-g8-100k-n128-rerank/artifacts/set_g8_100k_n128_width750.sql --raw --log-output review/30093-task28-ivf-pqfastscan-g8-100k-n128-rerank/artifacts/set_g8_100k_n128_width750_restore.log`

Key lines:

- `ALTER INDEX`
- `task28_ivf_pqg100k_g8_n128_idx | {nlists=128,nprobe=128,training_sample_rows=2000,storage_format=pq_fastscan,pq_group_size=8,rerank=heap_f32,rerank_width=750}`
