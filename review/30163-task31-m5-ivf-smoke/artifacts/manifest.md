# Task 31 M5 IVF Smoke Artifact Manifest

Head SHA: `6429a9e4b9de5a36ab982670bd8151644c6a2af0`

Packet/topic: `review/30163-task31-m5-ivf-smoke`

Timestamp: `2026-05-03T00:12:38Z`

Surface: local M5 PG18 pgrx smoke, synthetic 10k corpus, one-index-per-table corpus prefix `task31_m5_smoke_pqg8`, access method/profile `ec_ivf`.

Corpus shape:

- Path: `data/task31_m5_smoke/`
- Corpus file: `task31_m5_smoke_corpus.tsv`
- Query file: `task31_m5_smoke_queries.tsv`
- Corpus rows: `10000`
- Query rows: `20`
- Dimensions: `1536`
- Corpus seed: `31`
- Query seed: `3100`
- Corpus SHA256 from loader: `38dde7700ef3d60357035833aa7eb101a834264044f1ba24506c52d494fa3a89`
- Query SHA256 from loader: `0c381a769a984698e9bc3863f74b0ffefab2bbbc2ce9974d2969d7abf544b180`

Index/load surface:

- Lane: Task 31 local M5 IVF smoke
- Fixture: synthetic unit-sphere TSV
- Storage format: `pq_fastscan`
- PQ group size: `8`
- Rerank mode: `heap_f32`
- Rerank width: `500`
- IVF reloptions: `nlists=128`, `nprobe=8`
- Surface isolation: one corpus prefix/table for this smoke, with one `ec_ivf` index plus the corpus pkey.

## Artifacts

### `pg18-ecaz-status.log`

- Command: `/Users/peter/.cargo/bin/ecaz dev sql --pg 18 --db postgres --socket-dir /Users/peter/.pgrx --raw --sql "select version(); select extname, extversion from pg_extension where extname = 'ecaz'; select 1 as pg18_reachable;" --log-output review/30163-task31-m5-ivf-smoke/artifacts/pg18-ecaz-status.log`
- Purpose: verify PG18 is reachable and the installed extension is visible.
- Key result lines:
  - `PostgreSQL 18.3 (Homebrew) on aarch64-apple-darwin25.2.0`
  - `ecaz    | 0.1.1`
  - `pg18_reachable = 1`

### `corpus-list-before.log`

- Command: `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 --log-file review/30163-task31-m5-ivf-smoke/artifacts/corpus-list-before.log corpus list`
- Purpose: verify starting corpus state.
- Key result: `(no corpora loaded in postgres)`.

### `generate-corpus.log`

- Command: `/Users/peter/.cargo/bin/ecaz --log-file review/30163-task31-m5-ivf-smoke/artifacts/generate-corpus.log corpus generate --output data/task31_m5_smoke/task31_m5_smoke_corpus.tsv --n 10000 --dim 1536 --seed 31 --kind corpus`
- Purpose: generate deterministic synthetic corpus TSV.
- Key result: `wrote 10000 x dim 1536 rows`.

### `generate-queries.log`

- Command: `/Users/peter/.cargo/bin/ecaz --log-file review/30163-task31-m5-ivf-smoke/artifacts/generate-queries.log corpus generate --output data/task31_m5_smoke/task31_m5_smoke_queries.tsv --n 20 --dim 1536 --seed 3100 --kind queries`
- Purpose: generate deterministic synthetic query TSV.
- Key result: `wrote 20 x dim 1536 rows`.

### `load.log`

- Command: `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 --log-file review/30163-task31-m5-ivf-smoke/artifacts/load.log corpus load --prefix task31_m5_smoke_pqg8 --profile ec_ivf --corpus-file data/task31_m5_smoke/task31_m5_smoke_corpus.tsv --queries-file data/task31_m5_smoke/task31_m5_smoke_queries.tsv --reloption storage_format=pq_fastscan --reloption pq_group_size=8 --reloption nlists=128 --reloption nprobe=8 --reloption rerank=heap_f32 --reloption rerank_width=500`
- Purpose: load the corpus and build the requested IVF PQ-FastScan index.
- Key result lines:
  - `corpus: 10000 rows`
  - `queries: 20 rows`
  - `built task31_m5_smoke_pqg8_idx in 4.76s`
  - `completed prefix task31_m5_smoke_pqg8 in 7.21s`

### `corpus-inspect.log`

- Command: `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 --log-file review/30163-task31-m5-ivf-smoke/artifacts/corpus-inspect.log corpus inspect --prefix task31_m5_smoke_pqg8`
- Purpose: inspect loaded corpus and index metadata.
- Key result lines:
  - `task31_m5_smoke_pqg8_corpus (10000 rows)`
  - `task31_m5_smoke_pqg8_queries (20 rows)`
  - `task31_m5_smoke_pqg8_idx [storage_format=pq_fastscan, pq_group_size=8, nlists=128, nprobe=8, rerank=heap_f32, rerank_width=500]`

### `corpus-list-after.log`

- Command: `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 --log-file review/30163-task31-m5-ivf-smoke/artifacts/corpus-list-after.log corpus list`
- Purpose: verify corpus appears in the database.
- Key result: `task31_m5_smoke_pqg8`, `10000`, `yes`, `btree, ec_ivf`, `ec_ivf`.

### `recall_q3-table.log`, `truth_k10_q3.json`

- Command: `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 bench recall --prefix task31_m5_smoke_pqg8 --profile ec_ivf --k 10 --queries-limit 3 --sweep 8 --rerank-width 500 --force-index --truth-cache-file review/30163-task31-m5-ivf-smoke/artifacts/truth_k10_q3.json --log-output review/30163-task31-m5-ivf-smoke/artifacts/recall_q3-table.log`
- Purpose: tiny recall smoke only, no latency sweep or longer baseline.
- Key result: `nprobe=8`, `recall@k=0.1667`, `ndcg@k=0.8428`, `mean q-time=3.49 ms`.

### `storage.log`

- Command: `/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 --log-file review/30163-task31-m5-ivf-smoke/artifacts/storage.log bench storage --prefix task31_m5_smoke_pqg8`
- Purpose: optional tiny storage smoke.
- Key result lines:
  - `rows = 10000`
  - `table (heap + toast + fsm/vm) = 159.4 MiB`
  - `indexes = 3.7 MiB`
  - `task31_m5_smoke_pqg8_idx ... 3.2 MiB ... 338.3 B`
