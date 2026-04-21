# Artifact Manifest: `11078-task17-diskann-real-recall-recovery`

Head SHA: `0a23340`
Packet: `review/11078-task17-diskann-real-recall-recovery`
Lane: `task17 / DiskANN real-corpus recovery / pg18`
Fixture: `qdrant-dbpedia-openai3-1m -> target/real-corpus/ec_hnsw_real_10k`
Storage format: `default (no storage_format reloption)`
Rerank mode: `none`
Surface: `shared-table ecaz corpus/bench path`

## `artifacts/pre-distance-sweep.log`

- Timestamp: `2026-04-21 13:24:37 -0700`
- Command:

```text
cargo run -p ecaz-cli -- \
  --host /home/peter/.pgrx \
  --port 28818 \
  --database postgres \
  --log-file target/real-corpus/diskann-sweep-after-guc.log \
  bench recall \
  --prefix ec_hnsw_real_10k \
  --profile ec_diskann \
  --k 10 \
  --sweep 64,128,200,400,800
```

- Key result lines:

```text
[recall] ground truth in 4.38s
│ 64        ┆ 0.0055   ┆ 0.4577 ┆ 35.42 ms    │
│ 128       ┆ 0.0090   ┆ 0.4919 ┆ 38.93 ms    │
│ 200       ┆ 0.0095   ┆ 0.4935 ┆ 39.29 ms    │
│ 400       ┆ 0.0095   ┆ 0.4935 ┆ 39.26 ms    │
│ 800       ┆ 0.0095   ┆ 0.4935 ┆ 39.29 ms    │
```

- Notes:
  This is the diagnostic midpoint after the session-override fix proved the
  sweep was live, but before the index was rebuilt with the corrected build
  distance.

## `artifacts/rebuild.log`

- Timestamp: `2026-04-21 13:24:37 -0700`
- Command:

```text
cargo run -p ecaz-cli -- \
  --host /home/peter/.pgrx \
  --port 28818 \
  --database postgres \
  --log-file target/real-corpus/diskann-rebuild.log \
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
[loader] ec_hnsw_real_10k_corpus already has 10000 rows; skipping reload
[loader] ec_hnsw_real_10k_queries already has 200 rows; skipping reload
[loader] building ec_hnsw_real_10k_idx using ec_diskann (reloptions=[graph_degree=32, build_list_size=100, alpha=1.2]) ...
│ profile ┆ ec_diskann                                                             │
│ corpus  ┆ ec_hnsw_real_10k_corpus (10000 rows)                                   │
│ queries ┆ ec_hnsw_real_10k_queries (200 rows)                                    │
│ indexes ┆ ec_hnsw_real_10k_idx [graph_degree=32, build_list_size=100, alpha=1.2] │
```

## `artifacts/post-distance-sweep.log`

- Timestamp: `2026-04-21 13:24:37 -0700`
- Command:

```text
cargo run -p ecaz-cli -- \
  --host /home/peter/.pgrx \
  --port 28818 \
  --database postgres \
  --log-file target/real-corpus/diskann-sweep-after-distance-fix.log \
  bench recall \
  --prefix ec_hnsw_real_10k \
  --profile ec_diskann \
  --k 10 \
  --sweep 64,128,200,400,800
```

- Key result lines cited in `request.md`:

```text
[recall] ground truth in 4.42s
│ 64        ┆ 0.9280   ┆ 0.9959 ┆ 43.69 ms    │
│ 128       ┆ 0.9310   ┆ 0.9966 ┆ 55.46 ms    │
│ 200       ┆ 0.9315   ┆ 0.9966 ┆ 69.62 ms    │
│ 400       ┆ 0.9315   ┆ 0.9966 ┆ 122.57 ms   │
│ 800       ┆ 0.9315   ┆ 0.9966 ┆ 299.29 ms   │
```
