# Artifact Manifest: `11083-task17-diskann-post-vacuum-smoke`

Head SHA: `f3f5cb0`
Packet: `review/11083-task17-diskann-post-vacuum-smoke`
Lane: `task17 / DiskANN post-vacuum smoke / pg18`
Fixture: `qdrant-dbpedia-openai3-1m -> target/real-corpus/ec_hnsw_real_10k`
Storage format: `default (no storage_format reloption)`
Rerank mode: `none`
Surface: `shared-table ecaz corpus/bench path on the slower local smoke box`

## `artifacts/load.log`

- Timestamp: `2026-04-21 16:21:08 -0700`
- Command:

```text
cargo run -p ecaz-cli -- \
  --host /home/peter/.pgrx \
  --port 28818 \
  --database diskann_vacuum_smoke_c \
  --log-file review/11083-task17-diskann-post-vacuum-smoke/artifacts/load.log \
  corpus load \
  --prefix ec_hnsw_real_10k \
  --corpus-file /home/peter/dev/tqvector/target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_corpus.tsv \
  --queries-file /home/peter/dev/tqvector/target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_queries.tsv \
  --profile ec_diskann \
  --reloption graph_degree=32 \
  --reloption build_list_size=100 \
  --reloption alpha=1.2
```

- Shared or isolated surface: `shared-table`
- Key result lines:

```text
[loader] building ec_hnsw_real_10k_idx using ec_diskann (reloptions=[graph_degree=32, build_list_size=100, alpha=1.2]) ...
│ corpus  ┆ ec_hnsw_real_10k_corpus (10000 rows)                                   │
│ indexes ┆ ec_hnsw_real_10k_idx [graph_degree=32, build_list_size=100, alpha=1.2] │
```

## `artifacts/pre-vacuum-recall.log`

- Timestamp: `2026-04-21 16:21:58 -0700`
- Command:

```text
cargo run -p ecaz-cli -- \
  --host /home/peter/.pgrx \
  --port 28818 \
  --database diskann_vacuum_smoke_c \
  --log-file review/11083-task17-diskann-post-vacuum-smoke/artifacts/pre-vacuum-recall.log \
  bench recall \
  --prefix ec_hnsw_real_10k \
  --profile ec_diskann \
  --k 10 \
  --sweep 128
```

- Shared or isolated surface: `shared-table`
- Key result lines:

```text
[recall] ground truth in 4.46s
│ 128       ┆ 0.9310   ┆ 0.9965 ┆ 82.34 ms    │
```

## `artifacts/delete.log`

- Timestamp: `2026-04-21 16:22:20 -0700`
- Command:

```text
psql -h /home/peter/.pgrx -p 28818 -d diskann_vacuum_smoke_c -Atc "with deleted as (delete from ec_hnsw_real_10k_corpus where id % 10 = 0 returning 1) select 'deleted_rows=' || count(*) from deleted" -o review/11083-task17-diskann-post-vacuum-smoke/artifacts/delete.log
```

- Shared or isolated surface: `shared-table`
- Key result lines:

```text
deleted_rows=1000
```

## `artifacts/vacuum.log`

- Timestamp: `2026-04-21 16:27:31 -0700`
- Command:

```text
psql -h /home/peter/.pgrx -p 28818 -d diskann_vacuum_smoke_c -v ON_ERROR_STOP=1 -c "\timing on" -c "vacuum (analyze) ec_hnsw_real_10k_corpus" -o review/11083-task17-diskann-post-vacuum-smoke/artifacts/vacuum.log
```

- Shared or isolated surface: `shared-table`
- Key result lines:

```text
VACUUM
```

## `artifacts/vacuum-timing.log`

- Timestamp: `2026-04-21 16:27:31 -0700`
- Command:

```text
Captured stdout timing line from the same `psql -c "\timing on" -c "vacuum (analyze) ..."` session as `artifacts/vacuum.log`.
```

- Shared or isolated surface: `shared-table`
- Key result lines:

```text
Time: 305180.393 ms (05:05.180)
```

## `artifacts/post-vacuum-recall.log`

- Timestamp: `2026-04-21 16:28:23 -0700`
- Command:

```text
cargo run -p ecaz-cli -- \
  --host /home/peter/.pgrx \
  --port 28818 \
  --database diskann_vacuum_smoke_c \
  --log-file review/11083-task17-diskann-post-vacuum-smoke/artifacts/post-vacuum-recall.log \
  bench recall \
  --prefix ec_hnsw_real_10k \
  --profile ec_diskann \
  --k 10 \
  --sweep 128
```

- Shared or isolated surface: `shared-table`
- Key result lines:

```text
[recall] ground truth in 3.97s
│ 128       ┆ 0.9285   ┆ 0.9966 ┆ 81.17 ms    │
```
