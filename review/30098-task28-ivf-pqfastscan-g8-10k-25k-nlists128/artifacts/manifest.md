# Artifact Manifest

Head SHA: `363096abf5e2528c6c519b61af7095704fcca877`

Packet: `review/30098-task28-ivf-pqfastscan-g8-10k-25k-nlists128`

Timestamp: 2026-04-28 06:45 America/Los_Angeles

Fixture:

- Local PG18 database `postgres`.
- Existing PQ g8 10k/25k surfaces cloned into isolated n128 tables.
- Rows: 10k and 25k corpus rows, 100 query rows each.
- Dimensions: 1536.
- Storage format: `pq_fastscan`.
- PQ group size: 8.
- Rerank mode: `heap_f32`.
- Rerank width: 750.
- Runtime nprobe sweep: 48, 64, 96 for recall; 96 for latency.
- Cache state: warm local development run; no explicit cache drop.
- Memory high-water mark: not captured.

## build_pqg8_n128_surfaces.sql

Command:

`cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30098-task28-ivf-pqfastscan-g8-10k-25k-nlists128/artifacts/build_pqg8_n128_surfaces.sql --raw --log-output review/30098-task28-ivf-pqfastscan-g8-10k-25k-nlists128/artifacts/build_pqg8_n128_surfaces.log`

Key lines:

- 10k copy: `SELECT 10000` in `2504.774 ms`
- 10k build: `CREATE INDEX` in `40985.646 ms`
- 25k copy: `SELECT 25000` in `6569.149 ms`
- 25k build: `CREATE INDEX` in `69914.068 ms`
- `task28_ivf_pqg10k_g8_n128_idx | 2960 kB | {nlists=128,nprobe=128,training_sample_rows=2000,storage_format=pq_fastscan,pq_group_size=8,rerank=heap_f32,rerank_width=750}`
- `task28_ivf_pqg25k_g8_n128_idx | 5688 kB | {nlists=128,nprobe=128,training_sample_rows=2000,storage_format=pq_fastscan,pq_group_size=8,rerank=heap_f32,rerank_width=750}`

## recall10_pqg8_10k_n128_w750.log

Command:

`cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg10k_g8_n128 --profile ec_ivf --k 10 --queries-limit 100 --sweep 48,64,96 --force-index --log-output review/30098-task28-ivf-pqfastscan-g8-10k-25k-nlists128/artifacts/recall10_pqg8_10k_n128_w750.log`

Key lines:

- `48 | 0.9600 | 0.9969 | 76.56 ms`
- `64 | 0.9770 | 0.9989 | 83.10 ms`
- `96 | 0.9900 | 0.9997 | 95.62 ms`

## recall100_pqg8_10k_n128_w750.log

Command:

`cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg10k_g8_n128 --profile ec_ivf --k 100 --queries-limit 100 --sweep 48,64,96 --force-index --log-output review/30098-task28-ivf-pqfastscan-g8-10k-25k-nlists128/artifacts/recall100_pqg8_10k_n128_w750.log`

Key lines:

- `48 | 0.9082 | 0.9929 | 118.43 ms`
- `64 | 0.9229 | 0.9950 | 124.27 ms`
- `96 | 0.9370 | 0.9967 | 136.16 ms`

## recall10_pqg8_25k_n128_w750.log

Command:

`cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg25k_g8_n128 --profile ec_ivf --k 10 --queries-limit 100 --sweep 48,64,96 --force-index --log-output review/30098-task28-ivf-pqfastscan-g8-10k-25k-nlists128/artifacts/recall10_pqg8_25k_n128_w750.log`

Key lines:

- `48 | 0.9720 | 0.9987 | 111.86 ms`
- `64 | 0.9810 | 0.9992 | 130.19 ms`
- `96 | 0.9910 | 0.9999 | 156.86 ms`

## recall100_pqg8_25k_n128_w750.log

Command:

`cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg25k_g8_n128 --profile ec_ivf --k 100 --queries-limit 100 --sweep 48,64,96 --force-index --log-output review/30098-task28-ivf-pqfastscan-g8-10k-25k-nlists128/artifacts/recall100_pqg8_25k_n128_w750.log`

Key lines:

- `48 | 0.8921 | 0.9929 | 151.96 ms`
- `64 | 0.9139 | 0.9952 | 169.53 ms`
- `96 | 0.9285 | 0.9968 | 191.17 ms`

## latency_pqg8_10k_n128_w750_p96.log

Command:

`cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench latency --prefix task28_ivf_pqg10k_g8_n128 --profile ec_ivf --k 10 --iterations 100 --sweep 96 --force-index --log-output review/30098-task28-ivf-pqfastscan-g8-10k-25k-nlists128/artifacts/latency_pqg8_10k_n128_w750_p96.log`

Key line:

- `96 | 100 | 95.3 ms | 7.94 ms | 87.8 ms | 92.9 ms | 108.5 ms | 116.9 ms | 147.4 ms`

## latency_pqg8_25k_n128_w750_p96.log

Command:

`cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench latency --prefix task28_ivf_pqg25k_g8_n128 --profile ec_ivf --k 10 --iterations 100 --sweep 96 --force-index --log-output review/30098-task28-ivf-pqfastscan-g8-10k-25k-nlists128/artifacts/latency_pqg8_25k_n128_w750_p96.log`

Key line:

- `96 | 100 | 157.8 ms | 12.1 ms | 145.9 ms | 154.1 ms | 188.4 ms | 203.9 ms | 204.1 ms`
