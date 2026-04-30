# Artifact Manifest

Head SHA: `95fef9acca515c1dee61d6195085c62f5362779f`

Packet: `review/11100-task29b-diskann-vacuum-prefilter-consistency`

Lane: Task 29b DiskANN cleanup and vacuum consistency.

Fixture: local PG18, real-10k 1536-d corpus copied into isolated prefix
`task29b_vacuum_real10k`, 200 query rows.

Storage format: `ec_diskann` `pq_fastscan` tuple format with persisted binary
sidecar payload.

Rerank mode: heap-f32 exact rerank, default `rerank_budget=64`.

Table model: isolated one-index-per-table prefix
`task29b_vacuum_real10k`.

Cache state: warm local run on the existing PG18 scratch instance.

Timestamp: 2026-04-30T11:55:16-07:00

## Artifacts

### `drop-task29b-vacuum-prefix.log`

Command:

`cargo run -p ecaz-cli -- --database task29_diskann_baseline dev sql --pg 18 --socket-dir /home/peter/.pgrx --port 28818 --log-output review/11100-task29b-diskann-vacuum-prefilter-consistency/artifacts/drop-task29b-vacuum-prefix.log --sql "DROP INDEX IF EXISTS task29b_vacuum_real10k_idx; DROP TABLE IF EXISTS task29b_vacuum_real10k_corpus; DROP TABLE IF EXISTS task29b_vacuum_real10k_queries;"`

Key result: previous isolated objects did not exist.

### `load-task29b-vacuum-real10k.log`

Command:

`cargo run --release -p ecaz-cli -- --database task29_diskann_baseline --host /home/peter/.pgrx --port 28818 --log-file review/11100-task29b-diskann-vacuum-prefilter-consistency/artifacts/load-task29b-vacuum-real10k.log corpus load --prefix task29b_vacuum_real10k --corpus-file target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_corpus.tsv --queries-file target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_queries.tsv --profile ec_diskann --reloption graph_degree=32 --reloption build_list_size=100 --reloption alpha=1.2 --allow-manifest-mismatch`

Key result rows:

- copied corpus table in `4.62s`
- encoded corpus table in `4.43s`
- copied queries table in `100.71ms`
- built `task29b_vacuum_real10k_idx` in `494.39s`
- completed prefix in `505.57s`

### `schema-task29b-vacuum-real10k.log`

Command:

`cargo run -p ecaz-cli -- --database task29_diskann_baseline dev sql --pg 18 --socket-dir /home/peter/.pgrx --port 28818 --log-output review/11100-task29b-diskann-vacuum-prefilter-consistency/artifacts/schema-task29b-vacuum-real10k.log --sql "SELECT column_name, data_type FROM information_schema.columns WHERE table_name = 'task29b_vacuum_real10k_corpus' ORDER BY ordinal_position;"`

Key result rows: `id bigint`, `source ARRAY`, `embedding USER-DEFINED`.

### `recall-task29b-prevacuum-table.log`

Command:

`cargo run --release -p ecaz-cli -- --database task29_diskann_baseline --host /home/peter/.pgrx --port 28818 --log-file review/11100-task29b-diskann-vacuum-prefilter-consistency/artifacts/recall-task29b-prevacuum-cli.log bench recall --prefix task29b_vacuum_real10k --profile ec_diskann --k 10 --sweep 200 --force-index --truth-cache-file review/11100-task29b-diskann-vacuum-prefilter-consistency/artifacts/task29b-prevacuum-truth-k10.json --log-output review/11100-task29b-diskann-vacuum-prefilter-consistency/artifacts/recall-task29b-prevacuum-table.log`

Key result row:

- L=200: recall@10 `0.9970`, NDCG `0.9999`, mean query time `52.52 ms`

### `delete-task29b-vacuum-real10k.log`

Command:

`cargo run -p ecaz-cli -- --database task29_diskann_baseline dev sql --pg 18 --socket-dir /home/peter/.pgrx --port 28818 --log-output review/11100-task29b-diskann-vacuum-prefilter-consistency/artifacts/delete-task29b-vacuum-real10k.log --sql "DELETE FROM task29b_vacuum_real10k_corpus WHERE id % 20 = 0; SELECT count(*) AS remaining_rows FROM task29b_vacuum_real10k_corpus;"`

Key result rows: `DELETE 500`, remaining rows `9500`.

### `vacuum-task29b-vacuum-real10k.log`

Command:

`cargo run -p ecaz-cli -- --database task29_diskann_baseline dev sql --pg 18 --socket-dir /home/peter/.pgrx --port 28818 --log-output review/11100-task29b-diskann-vacuum-prefilter-consistency/artifacts/vacuum-task29b-vacuum-real10k.log --sql "VACUUM (ANALYZE) task29b_vacuum_real10k_corpus;"`

Key result row: `VACUUM`.

### `recall-task29b-postvacuum-table.log`

Command:

`cargo run --release -p ecaz-cli -- --database task29_diskann_baseline --host /home/peter/.pgrx --port 28818 --log-file review/11100-task29b-diskann-vacuum-prefilter-consistency/artifacts/recall-task29b-postvacuum-cli.log bench recall --prefix task29b_vacuum_real10k --profile ec_diskann --k 10 --sweep 200 --force-index --truth-cache-file review/11100-task29b-diskann-vacuum-prefilter-consistency/artifacts/task29b-postvacuum-truth-k10.json --log-output review/11100-task29b-diskann-vacuum-prefilter-consistency/artifacts/recall-task29b-postvacuum-table.log`

Key result row:

- L=200: recall@10 `0.9975`, NDCG `0.9999`, mean query time `52.33 ms`

### `cargo-asm-unavailable.log`

Command:

`script -q -e -c "cargo asm --no-default-features --features pg18 --release ecaz::am::ec_diskann::scan_query::hamming_xor_popcount" review/11100-task29b-diskann-vacuum-prefilter-consistency/artifacts/cargo-asm-unavailable.log`

Key result: local cargo install does not have the `asm` subcommand.

### `hamming-xor-popcount-asm.log`

Command:

`script -q -e -c "sed -n '441110,441205p' target/release/deps/ecaz.s" review/11100-task29b-diskann-vacuum-prefilter-consistency/artifacts/hamming-xor-popcount-asm.log`

The source assembly was generated by:

`cargo rustc --lib --no-default-features --features pg18 --release -- --emit=asm -C target-cpu=native -C link-dead-code`

Key result rows:

- bulk path includes `vpxor`, `vpshufb`, `vpsadbw`, and vector accumulation
- scalar tail/closure includes `popcntq`

### Truth caches

- `task29b-prevacuum-truth-k10.json`: generated from 10,000 live rows.
- `task29b-postvacuum-truth-k10.json`: generated from 9,500 live rows after
  delete and vacuum.
