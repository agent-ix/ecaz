# Artifact Manifest: `11094-task29-diskann-grouped-frontier-probe`

Packet: `review/11094-task29-diskann-grouped-frontier-probe`
Head SHA: `eb826920197a226a0aa6085661020e190eeb2bed`
Timestamp: `2026-04-29T21:28:35-07:00`

## frontier-q10001-table.log

- head SHA: `eb826920197a226a0aa6085661020e190eeb2bed`
- packet/topic: `11094-task29-diskann-grouped-frontier-probe`
- lane: Task 29 DiskANN initial tuning
- fixture: real-10k local PG18 corpus
- database: `task29_diskann_baseline`
- prefix: `task29_diskann_real10k`
- storage format: default `ec_diskann` search-code model shape simulated in `ecaz-cli`
- rerank mode: simulated exact source-vector rerank over grouped-PQ frontier
- isolated/shared surface: isolated one-index-per-table prefix
- command:

```text
cargo run -p ecaz-cli --release -- --host /home/peter/.pgrx --port 28818 --database task29_diskann_baseline --log-file review/11094-task29-diskann-grouped-frontier-probe/artifacts/frontier-q10001-cli.log bench diskann-build-probe --prefix task29_diskann_real10k --graph-degree 32 --build-list-size 100 --alpha 1.2 --seed 42 --medoid-sample-cap 1024 --scan-list-size 200 --recall-k 10 --frontier-query-id 10001 --frontier-rerank-budget 200 --frontier-top 40 --log-output review/11094-task29-diskann-grouped-frontier-probe/artifacts/frontier-q10001-table.log
```

- key result lines:
  - rows `10000`, dimensions `1536`, queries `200`
  - in-memory source-vector graph recall@10 `1.0000`
  - scan_list_size `200`, rerank_budget `200`
  - reranked IDs: `8885,9785,9957,9826,9926,9944,9855,9915,9976,9999`
  - exact ID `9717`: exact rank `5`, frontier rank `missing`, in_rerank_budget `false`
  - exact ID `7782`: exact rank `10`, frontier rank `missing`, in_rerank_budget `false`
  - exact ID `9785`: frontier rank `31`, in_rerank_budget `true`
  - exact ID `9926`: frontier rank `48`, in_rerank_budget `true`

## frontier-q10001-cli.log

- head SHA: `eb826920197a226a0aa6085661020e190eeb2bed`
- packet/topic: `11094-task29-diskann-grouped-frontier-probe`
- contents: raw mirrored CLI output for `frontier-q10001-table.log`
