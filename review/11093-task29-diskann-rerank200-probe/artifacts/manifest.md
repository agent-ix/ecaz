# Artifact Manifest: `11093-task29-diskann-rerank200-probe`

Packet: `review/11093-task29-diskann-rerank200-probe`
Head SHA: `de1ad349bae852c82b6edcabf91d6f29e5214e43`
Timestamp: `2026-04-29T21:15:14-07:00`

## alter-rerank-budget-200.log

- head SHA: `de1ad349bae852c82b6edcabf91d6f29e5214e43`
- packet/topic: `11093-task29-diskann-rerank200-probe`
- lane: Task 29 DiskANN initial tuning
- fixture: real-10k local PG18 corpus
- database: `task29_diskann_baseline`
- prefix: `task29_diskann_real10k`
- isolated/shared surface: isolated one-index-per-table prefix
- command:

```text
cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db task29_diskann_baseline --raw --sql "ALTER INDEX task29_diskann_real10k_idx SET (rerank_budget = 200); SELECT reloptions FROM pg_class WHERE relname = 'task29_diskann_real10k_idx';" --log-output review/11093-task29-diskann-rerank200-probe/artifacts/alter-rerank-budget-200.log
```

- key result line: `{graph_degree=32,build_list_size=100,alpha=1.2,rerank_budget=200}`

## recall-rerank200-table.log

- head SHA: `de1ad349bae852c82b6edcabf91d6f29e5214e43`
- packet/topic: `11093-task29-diskann-rerank200-probe`
- lane: Task 29 DiskANN initial tuning
- fixture: real-10k local PG18 corpus
- database: `task29_diskann_baseline`
- prefix: `task29_diskann_real10k`
- storage format: default `ec_diskann` storage
- rerank mode: persisted SQL exact heap rerank with index reloption `rerank_budget=200`
- isolated/shared surface: isolated one-index-per-table prefix
- command:

```text
cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database task29_diskann_baseline bench recall --prefix task29_diskann_real10k --profile ec_diskann --k 10 --sweep 200,400,800 --force-index --truth-cache-file review/11088-task29-diskann-seeded-build-probe/artifacts/real10k-truth-k10.json --log-output review/11093-task29-diskann-rerank200-probe/artifacts/recall-rerank200-table.log
```

- key result lines:
  - list_size `200`: recall `0.9845`, NDCG `0.9990`, mean `143.63 ms`
  - list_size `400`: recall `0.9845`, NDCG `0.9990`, mean `187.27 ms`
  - list_size `800`: recall `0.9845`, NDCG `0.9990`, mean `327.79 ms`

## compare-rerank200-q5-table.log

- head SHA: `de1ad349bae852c82b6edcabf91d6f29e5214e43`
- packet/topic: `11093-task29-diskann-rerank200-probe`
- lane: Task 29 DiskANN initial tuning
- fixture: real-10k local PG18 corpus
- database: `task29_diskann_baseline`
- prefix: `task29_diskann_real10k`
- storage format: default `ec_diskann` storage
- rerank mode: persisted SQL exact heap rerank with index reloption `rerank_budget=200`
- isolated/shared surface: isolated one-index-per-table prefix
- command:

```text
cargo run -p ecaz-cli --release -- --host /home/peter/.pgrx --port 28818 --database task29_diskann_baseline --log-file review/11093-task29-diskann-rerank200-probe/artifacts/compare-rerank200-q5-cli.log bench diskann-build-probe --prefix task29_diskann_real10k --graph-degree 32 --build-list-size 100 --alpha 1.2 --seed 42 --medoid-sample-cap 1024 --scan-list-size 200 --recall-k 10 --compare-queries 5 --log-output review/11093-task29-diskann-rerank200-probe/artifacts/compare-rerank200-q5-table.log
```

- key result lines:
  - in-memory recall@10 `1.0000`
  - query `10001`: exact/memory `10/10`, exact/sql `8/10`, memory/sql `8/10`
  - query `10001` exact/memory IDs: `8885,9785,9957,9826,9717,9926,9944,9855,9915,7782`
  - query `10001` SQL IDs: `8885,9785,9957,9826,9926,9944,9855,9915,9976,9999`

## reset-rerank-budget.log

- head SHA: `de1ad349bae852c82b6edcabf91d6f29e5214e43`
- packet/topic: `11093-task29-diskann-rerank200-probe`
- command:

```text
cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 --db task29_diskann_baseline --raw --sql "ALTER INDEX task29_diskann_real10k_idx RESET (rerank_budget); SELECT reloptions FROM pg_class WHERE relname = 'task29_diskann_real10k_idx';" --log-output review/11093-task29-diskann-rerank200-probe/artifacts/reset-rerank-budget.log
```

- key result line: `{graph_degree=32,build_list_size=100,alpha=1.2}`

## Mirrored CLI Logs

- `compare-rerank200-q5-cli.log`: raw mirrored CLI output for `compare-rerank200-q5-table.log`
