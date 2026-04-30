# Artifact Manifest: `11092-task29-diskann-rerank-budget-probe`

Packet: `review/11092-task29-diskann-rerank-budget-probe`
Head SHA: `365aec8d5b45cfc52c0176ad48112f05850c0159`
Timestamp: `2026-04-29T21:06:32-07:00`

## alter-rerank-budget-100.log

- head SHA: `365aec8d5b45cfc52c0176ad48112f05850c0159`
- packet/topic: `11092-task29-diskann-rerank-budget-probe`
- lane: Task 29 DiskANN initial tuning
- fixture: real-10k local PG18 corpus
- database: `task29_diskann_baseline`
- prefix: `task29_diskann_real10k`
- isolated/shared surface: isolated one-index-per-table prefix
- command:

```text
cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db task29_diskann_baseline --raw --sql "ALTER INDEX task29_diskann_real10k_idx SET (rerank_budget = 100); SELECT reloptions FROM pg_class WHERE relname = 'task29_diskann_real10k_idx';" --log-output review/11092-task29-diskann-rerank-budget-probe/artifacts/alter-rerank-budget-100.log
```

- key result line: `{graph_degree=32,build_list_size=100,alpha=1.2,rerank_budget=100}`

## recall-rerank100-valid-table.log

- head SHA: `365aec8d5b45cfc52c0176ad48112f05850c0159`
- packet/topic: `11092-task29-diskann-rerank-budget-probe`
- lane: Task 29 DiskANN initial tuning
- fixture: real-10k local PG18 corpus
- database: `task29_diskann_baseline`
- prefix: `task29_diskann_real10k`
- storage format: default `ec_diskann` storage
- rerank mode: persisted SQL exact heap rerank with index reloption `rerank_budget=100`
- isolated/shared surface: isolated one-index-per-table prefix
- command:

```text
cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database task29_diskann_baseline bench recall --prefix task29_diskann_real10k --profile ec_diskann --k 10 --sweep 100,128,200,400,800 --force-index --truth-cache-file review/11088-task29-diskann-seeded-build-probe/artifacts/real10k-truth-k10.json --log-output review/11092-task29-diskann-rerank-budget-probe/artifacts/recall-rerank100-valid-table.log
```

- key result lines:
  - list_size `100`: recall `0.9550`, NDCG `0.9976`, mean `81.96 ms`
  - list_size `128`: recall `0.9550`, NDCG `0.9976`, mean `85.36 ms`
  - list_size `200`: recall `0.9555`, NDCG `0.9976`, mean `99.95 ms`
  - list_size `400`: recall `0.9555`, NDCG `0.9977`, mean `139.34 ms`
  - list_size `800`: recall `0.9555`, NDCG `0.9977`, mean `281.22 ms`

## compare-rerank100-q5-table.log

- head SHA: `365aec8d5b45cfc52c0176ad48112f05850c0159`
- packet/topic: `11092-task29-diskann-rerank-budget-probe`
- lane: Task 29 DiskANN initial tuning
- fixture: real-10k local PG18 corpus
- database: `task29_diskann_baseline`
- prefix: `task29_diskann_real10k`
- storage format: default `ec_diskann` storage
- rerank mode: persisted SQL exact heap rerank with index reloption `rerank_budget=100`
- isolated/shared surface: isolated one-index-per-table prefix
- command:

```text
cargo run -p ecaz-cli --release -- --host /home/peter/.pgrx --port 28818 --database task29_diskann_baseline --log-file review/11092-task29-diskann-rerank-budget-probe/artifacts/compare-rerank100-q5-cli.log bench diskann-build-probe --prefix task29_diskann_real10k --graph-degree 32 --build-list-size 100 --alpha 1.2 --seed 42 --medoid-sample-cap 1024 --scan-list-size 100 --recall-k 10 --compare-queries 5 --log-output review/11092-task29-diskann-rerank-budget-probe/artifacts/compare-rerank100-q5-table.log
```

- key result lines:
  - in-memory recall@10 `0.9995`
  - query `10001`: exact/memory `10/10`, exact/sql `8/10`, memory/sql `8/10`
  - query `10001` exact/memory IDs: `8885,9785,9957,9826,9717,9926,9944,9855,9915,7782`
  - query `10001` SQL IDs: `8885,9785,9957,9826,9926,9944,9855,9915,9976,9999`

## reset-rerank-budget.log

- command:

```text
cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db task29_diskann_baseline --raw --sql "ALTER INDEX task29_diskann_real10k_idx RESET (rerank_budget); SELECT reloptions FROM pg_class WHERE relname = 'task29_diskann_real10k_idx';" --log-output review/11092-task29-diskann-rerank-budget-probe/artifacts/reset-rerank-budget.log
```

- key result line: `{graph_degree=32,build_list_size=100,alpha=1.2}`

## Mirrored CLI Logs

- `compare-rerank100-q5-cli.log`: raw mirrored CLI output for `compare-rerank100-q5-table.log`
