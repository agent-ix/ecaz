# Artifact Manifest: `11091-task29-diskann-sql-vs-memory-compare`

Packet: `review/11091-task29-diskann-sql-vs-memory-compare`
Head SHA: `5a7893236165e99891efb218675b42abc7088b55`
Timestamp: `2026-04-29T20:26:23-07:00`

## compare-q5-table.log

- head SHA: `5a7893236165e99891efb218675b42abc7088b55`
- packet/topic: `11091-task29-diskann-sql-vs-memory-compare`
- lane: Task 29 DiskANN initial tuning
- fixture: real-10k local PG18 corpus
- database: `task29_diskann_baseline`
- prefix: `task29_diskann_real10k`
- storage format: default `ec_diskann` storage
- rerank mode: default profile behavior; command compares in-memory source-vector graph search with persisted SQL DiskANN
- isolated/shared surface: isolated one-index-per-table prefix
- command:

```text
cargo run -p ecaz-cli --release -- --host /home/peter/.pgrx --port 28818 --database task29_diskann_baseline --log-file review/11091-task29-diskann-sql-vs-memory-compare/artifacts/compare-q5-cli.log bench diskann-build-probe --prefix task29_diskann_real10k --graph-degree 32 --build-list-size 100 --alpha 1.2 --seed 42 --medoid-sample-cap 1024 --scan-list-size 100 --recall-k 10 --compare-queries 5 --log-output review/11091-task29-diskann-sql-vs-memory-compare/artifacts/compare-q5-table.log
```

- key result lines:
  - rows `10000`, queries `200`, dimensions `1536`
  - reachable `10000`, reachable_fraction `1.000000`
  - build_seconds `109.674`, recall_seconds `0.396`, recall@10 `0.9995`
  - query `10000`: exact/memory `10/10`, exact/sql `10/10`, memory/sql `10/10`
  - query `10001`: exact/memory `10/10`, exact/sql `8/10`, memory/sql `8/10`
  - query `10002`: exact/memory `10/10`, exact/sql `10/10`, memory/sql `10/10`
  - query `10003`: exact/memory `10/10`, exact/sql `10/10`, memory/sql `10/10`
  - query `10004`: exact/memory `10/10`, exact/sql `10/10`, memory/sql `10/10`
  - query `10001` exact/memory IDs: `8885,9785,9957,9826,9717,9926,9944,9855,9915,7782`
  - query `10001` SQL IDs: `8885,9785,9957,9826,9926,9944,9855,9915,9976,9999`

## compare-q5-cli.log

- same command and metadata as `compare-q5-table.log`
- raw mirrored CLI output
