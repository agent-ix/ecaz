# Artifact Manifest: `675-c1-task29-diskann-real10k-baseline`

Head SHA: `9291ec008b978f13da593821ac5a92d28d634373`
Packet: `review/675-c1-task29-diskann-real10k-baseline`
Lane: `task29 / DiskANN initial tuning / pg18 local`
Fixture: `qdrant-dbpedia-openai3-1m -> target/real-corpus/ec_hnsw_real_10k`
Storage format: default
Rerank mode: none
Surface: shared-table `ecaz-cli` corpus/bench path
Database: local PG18 pgrx, `task29_diskann_baseline`
Cache state: warm/local WSL2 page cache; no cache drop attempted
Machine: WSL2 Linux `6.6.87.2-microsoft-standard-WSL2`, Intel Core i9-10900K, 20 logical CPUs, 62 GiB RAM

## Fixture Setup

- `fetch-real-corpus.log`
  - command: `cargo run -p ecaz-cli -- --log-file review/675-c1-task29-diskann-real10k-baseline/artifacts/fetch-real-corpus.log corpus fetch --dataset qdrant-dbpedia-openai3-large-1536-1m --output-dir target/real-corpus`
  - key result: downloaded 26 parquet shards to `target/real-corpus/data`
- prepare command:

```text
cargo run -p ecaz-cli -- \
  --log-file review/675-c1-task29-diskann-real10k-baseline/artifacts/prepare-real10k.log \
  corpus prepare \
  --profile ec_hnsw_real_10k \
  --parquet target/real-corpus/data \
  --output-dir target/real-corpus/ec_hnsw_real_10k
```

  - note: the empty `prepare-real10k.log` was removed because the current prepare path uses ordinary stderr instead of the CLI mirror; request.md cites the generated manifest/TSV hashes reported by later loader output.
- `create-db.log`, `drop-db.log`, `terminate-db-sessions.log`
  - command family: `cargo run -p ecaz-cli -- --database postgres dev sql --pg 18 ...`
  - purpose: fresh local benchmark DB setup through checked-in CLI surface.

## DiskANN Load / Index

- command:

```text
cargo run -p ecaz-cli -- \
  --host /home/peter/.pgrx \
  --port 28818 \
  --database task29_diskann_baseline \
  --log-file review/675-c1-task29-diskann-real10k-baseline/artifacts/load-diskann.log \
  corpus load \
  --prefix ec_hnsw_real_10k \
  --corpus-file target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_corpus.tsv \
  --queries-file target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_queries.tsv \
  --profile ec_diskann \
  --reloption graph_degree=32 \
  --reloption build_list_size=100 \
  --reloption alpha=1.2
```

- Note: no build/load timing claim is made in this packet. The empty `load-diskann.log` was removed because the current loader's `--log-file` does not capture its ordinary stdout/stderr, and this packet avoids shell redirection/wrappers per Task 29 constraints.

## DiskANN Recall Probe

- `recall-diskann-q1-table.log`
  - command:

```text
cargo run -p ecaz-cli -- \
  --host /home/peter/.pgrx \
  --port 28818 \
  --database task29_diskann_baseline \
  --log-file review/675-c1-task29-diskann-real10k-baseline/artifacts/recall-diskann-q1-cli.log \
  bench recall \
  --prefix ec_hnsw_real_10k \
  --profile ec_diskann \
  --k 10 \
  --sweep 64 \
  --queries-limit 1 \
  --truth-cache-file review/675-c1-task29-diskann-real10k-baseline/artifacts/truth-k10-q1.json \
  --log-output review/675-c1-task29-diskann-real10k-baseline/artifacts/recall-diskann-q1-table.log
```

- key result:

```text
│ 64        ┆ 1.0000   ┆ 1.0000 ┆ 3993.69 ms  │
```

- `recall-activity.log`, `recall-cancel.log`
  - full 200-query, five-point sweep command was started with `--sweep 64,128,200,400,800`.
  - after one active KNN query ran far beyond expected local real-10k latency, it was cancelled through `ecaz-cli dev sql`.

## Planner / Harness Evidence

- `explain-prepared-diskann-q1.sql`
  - packet-local SQL file for `ecaz-cli dev sql`.
- `explain-prepared-diskann-q1.log`
  - command:

```text
cargo run -p ecaz-cli -- \
  --database postgres \
  dev sql \
  --pg 18 \
  --db task29_diskann_baseline \
  --raw \
  --file review/675-c1-task29-diskann-real10k-baseline/artifacts/explain-prepared-diskann-q1.sql \
  --log-output review/675-c1-task29-diskann-real10k-baseline/artifacts/explain-prepared-diskann-q1.log
```

- key result:

```text
Limit (actual time=4002.280..4002.283 rows=10.00 loops=1)
  ->  Sort (actual time=4002.278..4002.279 rows=10.00 loops=1)
        Disabled: true
        ->  Seq Scan on ec_hnsw_real_10k_corpus (actual time=0.773..3994.989 rows=10000.00 loops=1)
Execution Time: 4002.321 ms
```

## HNSW Reference Probe

- HNSW load command: `cargo run -p ecaz-cli -- ... corpus load --prefix ec_hnsw_real_10k --profile ec_hnsw --m 16`
  - note: the empty `load-hnsw-m16.log` was removed because loader mirror logging is currently incomplete.
- `recall-hnsw-q1-table.log`
  - key result:

```text
│ 64        ┆ 1.0000   ┆ 1.0000 ┆ 4044.26 ms  │
```

## Storage

- `storage-diskann.log`
  - command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database task29_diskann_baseline --log-file review/675-c1-task29-diskann-real10k-baseline/artifacts/storage-diskann.log bench storage --prefix ec_hnsw_real_10k`
  - key result:

```text
ec_hnsw_real_10k_idx         ec_diskann  {graph_degree=32,build_list_size=100,alpha=1.2}  4.7 MiB  494.0 B/row
```
