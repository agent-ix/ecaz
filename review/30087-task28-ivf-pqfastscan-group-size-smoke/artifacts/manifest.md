# Artifact Manifest

Head SHA: `810fe30`

Packet: `review/30087-task28-ivf-pqfastscan-group-size-smoke`

Timestamp: 2026-04-28 01:10 America/Los_Angeles

Fixture:

- Source prefix: `task28_ivf_postopt10k_n64w25`
- Derived prefixes:
  - `task28_ivf_pqg10k_g8`
  - `task28_ivf_pqg10k_g16`
  - `task28_ivf_pqg10k_g32`
- Rows: 10000 corpus rows, 100 query rows
- Dimensions: 1536
- Isolated one-index-per-table surfaces: yes
- Storage format: `pq_fastscan`
- Rerank mode: `heap_f32`
- nlists: 64
- Runtime nprobe sweep: 32, 48
- Cache state: warm local development run; no explicit cache drop
- Memory high-water mark: not captured

## build_group_size_surfaces.sql

Command:

`cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30087-task28-ivf-pqfastscan-group-size-smoke/artifacts/build_group_size_surfaces.sql --raw --log-output review/30087-task28-ivf-pqfastscan-group-size-smoke/artifacts/build_group_size_surfaces.log`

Key lines:

- `pq_group_size=8`: `CREATE INDEX` in `27697.866 ms`; index size `2448 kB`
- `pq_group_size=16`: `CREATE INDEX` in `24881.716 ms`; index size `1968 kB`
- `pq_group_size=32`: `CREATE INDEX` in `23914.444 ms`; index size `1768 kB`

## recall_g8_w25.log

Command:

`cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg10k_g8 --profile ec_ivf --k 10 --queries-limit 100 --sweep 32,48 --force-index --log-output review/30087-task28-ivf-pqfastscan-group-size-smoke/artifacts/recall_g8_w25.log`

Key lines:

- `32 | 0.6470 | 0.9697 | 42.89 ms`
- `48 | 0.6570 | 0.9711 | 53.82 ms`

## recall_g16_w25.log

Command:

`cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg10k_g16 --profile ec_ivf --k 10 --queries-limit 100 --sweep 32,48 --force-index --log-output review/30087-task28-ivf-pqfastscan-group-size-smoke/artifacts/recall_g16_w25.log`

Key lines:

- `32 | 0.3880 | 0.9079 | 33.99 ms`
- `48 | 0.3890 | 0.9081 | 41.21 ms`

## recall_g32_w25.log

Command:

`cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg10k_g32 --profile ec_ivf --k 10 --queries-limit 100 --sweep 32,48 --force-index --log-output review/30087-task28-ivf-pqfastscan-group-size-smoke/artifacts/recall_g32_w25.log`

Key lines:

- `32 | 0.1790 | 0.8019 | 30.20 ms`
- `48 | 0.1780 | 0.8012 | 34.36 ms`

## set_g8_width250.sql

Command:

`cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30087-task28-ivf-pqfastscan-group-size-smoke/artifacts/set_g8_width250.sql --raw --log-output review/30087-task28-ivf-pqfastscan-group-size-smoke/artifacts/set_g8_width250.log`

Key line:

- `task28_ivf_pqg10k_g8_idx | {nlists=64,nprobe=64,training_sample_rows=2000,storage_format=pq_fastscan,pq_group_size=8,rerank=heap_f32,rerank_width=250}`

## recall_g8_w250.log

Command:

`cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg10k_g8 --profile ec_ivf --k 10 --queries-limit 100 --sweep 32,48 --force-index --log-output review/30087-task28-ivf-pqfastscan-group-size-smoke/artifacts/recall_g8_w250.log`

Key lines:

- `32 | 0.9170 | 0.9945 | 51.17 ms`
- `48 | 0.9330 | 0.9964 | 61.57 ms`

## set_g8_width1000.sql

Command:

`cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30087-task28-ivf-pqfastscan-group-size-smoke/artifacts/set_g8_width1000.sql --raw --log-output review/30087-task28-ivf-pqfastscan-group-size-smoke/artifacts/set_g8_width1000.log`

Key line:

- `task28_ivf_pqg10k_g8_idx | {nlists=64,nprobe=64,training_sample_rows=2000,storage_format=pq_fastscan,pq_group_size=8,rerank=heap_f32,rerank_width=1000}`

## recall_g8_w1000.log

Command:

`cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg10k_g8 --profile ec_ivf --k 10 --queries-limit 100 --sweep 32,48 --force-index --log-output review/30087-task28-ivf-pqfastscan-group-size-smoke/artifacts/recall_g8_w1000.log`

Key lines:

- `32 | 0.9780 | 0.9980 | 81.38 ms`
- `48 | 0.9970 | 0.9998 | 93.73 ms`

## set_g8_width25.sql

Command:

`cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30087-task28-ivf-pqfastscan-group-size-smoke/artifacts/set_g8_width25.sql --raw --log-output review/30087-task28-ivf-pqfastscan-group-size-smoke/artifacts/set_g8_width25.log`

Key line:

- `task28_ivf_pqg10k_g8_idx | {nlists=64,nprobe=64,training_sample_rows=2000,storage_format=pq_fastscan,pq_group_size=8,rerank=heap_f32,rerank_width=25}`
