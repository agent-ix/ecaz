# Artifact Manifest

Head SHA: `60aa27755ae260bfa512cd687f487ce478f62d30`

Packet: `review/30085-task28-ivf-pqfastscan-rerank-diagnostic`

Timestamp: 2026-04-28 00:05 America/Los_Angeles

Fixture:

- Source prefix: `task28_ivf_qcmp10k_pqfastscan`
- Rows: 10000 corpus rows, 100 query rows
- Dimensions: 1536
- Isolated one-index-per-table surface: yes
- Storage format: `pq_fastscan`
- Rerank mode: `heap_f32`
- Rerank width sweep: 100, 250, 1000, restored to 25
- nlists: 64
- Runtime nprobe sweep: 32, 48
- Cache state: warm local development run; no explicit cache drop
- Memory high-water mark: not captured

## set_pqfastscan_width100.sql

Command:

`cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30085-task28-ivf-pqfastscan-rerank-diagnostic/artifacts/set_pqfastscan_width100.sql --raw --log-output review/30085-task28-ivf-pqfastscan-rerank-diagnostic/artifacts/set_pqfastscan_width100.log`

Key line:

- `task28_ivf_qcmp10k_pqfastscan_idx | {nlists=64,nprobe=64,training_sample_rows=2000,storage_format=pq_fastscan,rerank=heap_f32,rerank_width=100}`

## recall_pqfastscan_width100.log

Command:

`cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_qcmp10k_pqfastscan --profile ec_ivf --k 10 --queries-limit 100 --sweep 32,48 --force-index --log-output review/30085-task28-ivf-pqfastscan-rerank-diagnostic/artifacts/recall_pqfastscan_width100.log`

Key lines:

- `32 | 0.6160 | 0.9574 | 36.79 ms`
- `48 | 0.6210 | 0.9583 | 43.50 ms`

## set_pqfastscan_width250.sql

Command:

`cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30085-task28-ivf-pqfastscan-rerank-diagnostic/artifacts/set_pqfastscan_width250.sql --raw --log-output review/30085-task28-ivf-pqfastscan-rerank-diagnostic/artifacts/set_pqfastscan_width250.log`

Key line:

- `task28_ivf_qcmp10k_pqfastscan_idx | {nlists=64,nprobe=64,training_sample_rows=2000,storage_format=pq_fastscan,rerank=heap_f32,rerank_width=250}`

## recall_pqfastscan_width250.log

Command:

`cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_qcmp10k_pqfastscan --profile ec_ivf --k 10 --queries-limit 100 --sweep 32,48 --force-index --log-output review/30085-task28-ivf-pqfastscan-rerank-diagnostic/artifacts/recall_pqfastscan_width250.log`

Key lines:

- `32 | 0.7530 | 0.9730 | 42.52 ms`
- `48 | 0.7620 | 0.9740 | 49.23 ms`

## set_pqfastscan_width1000.sql

Command:

`cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30085-task28-ivf-pqfastscan-rerank-diagnostic/artifacts/set_pqfastscan_width1000.sql --raw --log-output review/30085-task28-ivf-pqfastscan-rerank-diagnostic/artifacts/set_pqfastscan_width1000.log`

Key line:

- `task28_ivf_qcmp10k_pqfastscan_idx | {nlists=64,nprobe=64,training_sample_rows=2000,storage_format=pq_fastscan,rerank=heap_f32,rerank_width=1000}`

## recall_pqfastscan_width1000.log

Command:

`cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_qcmp10k_pqfastscan --profile ec_ivf --k 10 --queries-limit 100 --sweep 32,48 --force-index --log-output review/30085-task28-ivf-pqfastscan-rerank-diagnostic/artifacts/recall_pqfastscan_width1000.log`

Key lines:

- `32 | 0.9090 | 0.9916 | 74.07 ms`
- `48 | 0.9200 | 0.9928 | 81.96 ms`

## set_pqfastscan_width25.sql

Command:

`cargo run -p ecaz-cli -- dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --file review/30085-task28-ivf-pqfastscan-rerank-diagnostic/artifacts/set_pqfastscan_width25.sql --raw --log-output review/30085-task28-ivf-pqfastscan-rerank-diagnostic/artifacts/set_pqfastscan_width25.log`

Key line:

- `task28_ivf_qcmp10k_pqfastscan_idx | {nlists=64,nprobe=64,training_sample_rows=2000,storage_format=pq_fastscan,rerank=heap_f32,rerank_width=25}`
