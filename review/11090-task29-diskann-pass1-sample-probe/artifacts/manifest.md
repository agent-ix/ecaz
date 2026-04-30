# Artifact Manifest: `11090-task29-diskann-pass1-sample-probe`

Packet: `review/11090-task29-diskann-pass1-sample-probe`
Head SHA: `29d72b1edbf37fb6426167a9a46b05c0ec912748`
Timestamp: `2026-04-29T20:18:24-07:00`

## baseline-table.log

- head SHA: `29d72b1edbf37fb6426167a9a46b05c0ec912748`
- packet/topic: `11090-task29-diskann-pass1-sample-probe`
- lane: Task 29 DiskANN initial tuning
- fixture: real-10k local PG18 corpus
- database: `task29_diskann_baseline`
- prefix: `task29_diskann_real10k`
- storage format: default `ec_diskann` storage
- rerank mode: default profile behavior; this probe replays in-memory build and graph search
- isolated/shared surface: isolated one-index-per-table prefix
- command:

```text
cargo run -p ecaz-cli --release -- --host /home/peter/.pgrx --port 28818 --database task29_diskann_baseline --log-file review/11090-task29-diskann-pass1-sample-probe/artifacts/baseline-cli.log bench diskann-build-probe --prefix task29_diskann_real10k --graph-degree 32 --build-list-size 100 --alpha 1.2 --seed 42 --medoid-sample-cap 1024 --scan-list-size 100 --recall-k 10 --log-output review/11090-task29-diskann-pass1-sample-probe/artifacts/baseline-table.log
```

- key result lines:
  - rows `10000`, queries `200`, dimensions `1536`
  - pass1_sample_candidates `0`
  - reachable `10000`, reachable_fraction `1.000000`
  - build_seconds `110.290`, recall_seconds `0.404`, recall@10 `0.9995`
  - pass 1 pool mean/p95 `101.93/106`, selected mean/p95 `8.22/12`
  - pass 2 pool mean/p95 `105.42/113`, selected mean/p95 `21.59/32`
  - final out degree min/mean/p50/p95/p99/max `1/24.50/25/32/32/32`
  - final in degree min/mean/p50/p95/p99/max `1/24.50/22/43/61/3250`

## pass1-sample32-table.log

- head SHA: `29d72b1edbf37fb6426167a9a46b05c0ec912748`
- packet/topic: `11090-task29-diskann-pass1-sample-probe`
- lane: Task 29 DiskANN initial tuning
- fixture: real-10k local PG18 corpus
- database: `task29_diskann_baseline`
- prefix: `task29_diskann_real10k`
- storage format: default `ec_diskann` storage
- rerank mode: default profile behavior; this probe replays in-memory build and graph search
- isolated/shared surface: isolated one-index-per-table prefix
- command:

```text
cargo run -p ecaz-cli --release -- --host /home/peter/.pgrx --port 28818 --database task29_diskann_baseline --log-file review/11090-task29-diskann-pass1-sample-probe/artifacts/pass1-sample32-cli.log bench diskann-build-probe --prefix task29_diskann_real10k --graph-degree 32 --build-list-size 100 --alpha 1.2 --seed 42 --medoid-sample-cap 1024 --pass1-sample-candidates 32 --pass1-sample-pool-size 1024 --scan-list-size 100 --recall-k 10 --log-output review/11090-task29-diskann-pass1-sample-probe/artifacts/pass1-sample32-table.log
```

- key result lines:
  - rows `10000`, queries `200`, dimensions `1536`
  - pass1_sample_candidates `32`, pass1_sample_pool_size `1024`
  - reachable `10000`, reachable_fraction `1.000000`
  - augmentation_seconds `1.401`, build_seconds `108.383`, recall_seconds `0.394`, recall@10 `1.0000`
  - pass 1 pool mean/p95 `114.77/128`, selected mean/p95 `8.67/13`
  - pass 2 pool mean/p95 `105.61/113`, selected mean/p95 `21.73/32`
  - final out degree min/mean/p50/p95/p99/max `5/24.64/25/32/32/32`
  - final in degree min/mean/p50/p95/p99/max `4/24.64/22/43/61/3249`

## Mirrored CLI Logs

- `baseline-cli.log`: raw mirrored CLI output for `baseline-table.log`
- `pass1-sample32-cli.log`: raw mirrored CLI output for `pass1-sample32-table.log`
