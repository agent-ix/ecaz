# Artifact Manifest: `11073-task17-diskann-real-10k-recall`

Head SHA: `dca0f69`
Packet: `review/11073-task17-diskann-real-10k-recall`
Lane: `task17 / DiskANN UX / pg18 real-corpus gate`
Fixture: `qdrant-dbpedia-openai3-1m -> target/real-corpus/ec_hnsw_real_10k`
Storage format: `default (no storage_format reloption)`
Rerank mode: `none`
Surface: `shared-table ecaz corpus/bench path`

## `artifacts/load-prefix-mismatch.log`

- Timestamp: `2026-04-21 11:11:55 -0700`
- Command:

```text
cargo run -p ecaz-cli -- \
  --host /home/peter/.pgrx \
  --port 28818 \
  --database postgres \
  corpus load \
  --prefix ec_diskann_real_10k \
  --corpus-file /home/peter/dev/tqvector/target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_corpus.tsv \
  --queries-file /home/peter/dev/tqvector/target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_queries.tsv \
  --profile ec_diskann \
  --reloption graph_degree=32 \
  --reloption build_list_size=100 \
  --reloption alpha=1.2
```

- Key result lines:

```text
manifest verification failed for ... ec_hnsw_real_10k_manifest.json: prefix="ec_hnsw_real_10k" (expected "ec_diskann_real_10k")
```

- Notes:
  This artifact is historical context only. It records the rejected prefix
  rename and is not cited for the final DiskANN measurement result.

## `artifacts/load.log`

- Timestamp: `2026-04-21 11:47:55 -0700`
- Command:

```text
cargo run -p ecaz-cli -- \
  --host /home/peter/.pgrx \
  --port 28818 \
  --database postgres \
  --log-file review/11073-task17-diskann-real-10k-recall/artifacts/load.log \
  corpus load \
  --prefix ec_hnsw_real_10k \
  --corpus-file /home/peter/dev/tqvector/target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_corpus.tsv \
  --queries-file /home/peter/dev/tqvector/target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_queries.tsv \
  --profile ec_diskann \
  --reloption graph_degree=32 \
  --reloption build_list_size=100 \
  --reloption alpha=1.2
```

- Key result lines:

```text
[loader] verified manifest ... ec_hnsw_real_10k_manifest.json for prefix ec_hnsw_real_10k
[loader] building ec_hnsw_real_10k_idx using ec_diskann (reloptions=[graph_degree=32, build_list_size=100, alpha=1.2]) ...
│ profile ┆ ec_diskann                                                             │
│ corpus  ┆ ec_hnsw_real_10k_corpus (10000 rows)                                   │
│ queries ┆ ec_hnsw_real_10k_queries (200 rows)                                    │
│ indexes ┆ ec_hnsw_real_10k_idx [graph_degree=32, build_list_size=100, alpha=1.2] │
```

## `artifacts/recall.log`

- Timestamp: `2026-04-21 12:38:50 -0700`
- Command:

```text
cargo run -p ecaz-cli -- \
  --host /home/peter/.pgrx \
  --port 28818 \
  --database postgres \
  --log-file review/11073-task17-diskann-real-10k-recall/artifacts/recall.log \
  bench recall \
  --prefix ec_hnsw_real_10k \
  --profile ec_diskann \
  --k 10 \
  --sweep 128
```

- Key result lines cited in `request.md`:

```text
[recall] ground truth in 4.52s
│ 128       ┆ 0.0075   ┆ 0.4833 ┆ 38.23 ms    │
```
