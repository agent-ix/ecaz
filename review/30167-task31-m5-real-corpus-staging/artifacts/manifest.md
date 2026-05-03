# Task 31 M5 Real Corpus Staging Artifact Manifest

Head SHA: `d1d49552427e05c6d23faf7bd4dae7efb3878efe`

Packet/topic: `review/30167-task31-m5-real-corpus-staging`

Timestamp: `2026-05-03T02:21:13Z`

Machine: Task 31 M5 laptop from packet `30162`, Apple M5 Pro, macOS local PG18
pgrx environment.

Surface: local real-corpus staging only. No Postgres load, index build, recall,
latency, storage, or EXPLAIN/counter claim is made by this packet.

CLI path: `/Users/peter/.cargo/bin/ecaz`

Source dataset:

- Dataset key: `qdrant-dbpedia-openai3-large-1536-1m`
- Repo: `Qdrant/dbpedia-entities-openai3-text-embedding-3-large-1536-1M`
- Revision: `main`
- Local parquet directory: `data/task31_m5_dbpedia_fetch/data`
- Local fetch manifest: `data/task31_m5_dbpedia_fetch/ecaz_fetch_manifest.json`
- Shards: `26`

Staged output directory: `data/task31_m5_dbpedia_staged`

## Artifacts

### `fetch.log`

- Lane: Task 31 M5 real-corpus staging.
- Fixture: canonical DBPedia OpenAI 1536D parquet fetch.
- Command:
  `/Users/peter/.cargo/bin/ecaz --log-file review/30167-task31-m5-real-corpus-staging/artifacts/fetch.log corpus fetch --output-dir data/task31_m5_dbpedia_fetch`
- Key result:
  - Downloaded shards `train-00000-of-00026.parquet` through
    `train-00016-of-00026.parquet`.
  - Remote connection reset while reading `train-00017-of-00026.parquet`.
- Interpretation: first fetch attempt was partial and was resumed by
  `fetch-rerun.log`.

### `fetch-rerun.log`

- Lane: Task 31 M5 real-corpus staging.
- Fixture: canonical DBPedia OpenAI 1536D parquet fetch resume.
- Command:
  `/Users/peter/.cargo/bin/ecaz --log-file review/30167-task31-m5-real-corpus-staging/artifacts/fetch-rerun.log corpus fetch --output-dir data/task31_m5_dbpedia_fetch`
- Key result:
  - `downloaded=9 skipped=17 parquet_dir=data/task31_m5_dbpedia_fetch/data`
  - all `26` shards are now present locally.

### `prepare-10k.log`

- Lane: Task 31 M5 real-corpus staging.
- Fixture: `ec_hnsw_real_10k`
- Command:
  `/Users/peter/.cargo/bin/ecaz corpus prepare --profile ec_hnsw_real_10k --parquet data/task31_m5_dbpedia_fetch/data --output-dir data/task31_m5_dbpedia_staged --log-file review/30167-task31-m5-real-corpus-staging/artifacts/prepare-10k.log`
- Key result:
  - `corpus_rows=10000`
  - `query_rows=200`
  - corpus SHA256 `c67c5810b66d982d705974e48d4775479adfbd92a988f694091266e049a35e75`
  - queries SHA256 `a2c191bb742017d849e73f6e6866e8e0f0bac1579ba212f7fc76b8eb09904ae8`

### `prepare-25k.log`

- Lane: Task 31 M5 real-corpus staging.
- Fixture: `ec_hnsw_real_25k`
- Command:
  `/Users/peter/.cargo/bin/ecaz corpus prepare --profile ec_hnsw_real_25k --parquet data/task31_m5_dbpedia_fetch/data --output-dir data/task31_m5_dbpedia_staged --log-file review/30167-task31-m5-real-corpus-staging/artifacts/prepare-25k.log`
- Key result:
  - `corpus_rows=25000`
  - `query_rows=500`
  - corpus SHA256 `234739ca91125db9d68052fe65380c32b9f41b42aa339320d77915680197a405`
  - queries SHA256 `80548c67c965dc8f22e793d0ec7af78c96d23e60793fd5c41311a5543b64d2f8`

### `prepare-50k.log`

- Lane: Task 31 M5 real-corpus staging.
- Fixture: `ec_hnsw_real_50k`
- Command:
  `/Users/peter/.cargo/bin/ecaz corpus prepare --profile ec_hnsw_real_50k --parquet data/task31_m5_dbpedia_fetch/data --output-dir data/task31_m5_dbpedia_staged --log-file review/30167-task31-m5-real-corpus-staging/artifacts/prepare-50k.log`
- Key result:
  - `corpus_rows=50000`
  - `query_rows=1000`
  - corpus SHA256 `56023baaa7bc42f758272e8617603d538808e6290a8a70a3a84e057571240133`
  - queries SHA256 `95ac7992578aa80bb193657f10fbcbf1ea3867e559739244bf5a467f7a5a9fa3`

### `prepare-100k.log`

- Lane: Task 31 M5 real-corpus staging.
- Fixture: `ec_hnsw_real_100k`
- Command:
  `/Users/peter/.cargo/bin/ecaz corpus prepare --profile ec_hnsw_real_100k --parquet data/task31_m5_dbpedia_fetch/data --output-dir data/task31_m5_dbpedia_staged --log-file review/30167-task31-m5-real-corpus-staging/artifacts/prepare-100k.log`
- Key result:
  - `corpus_rows=100000`
  - `query_rows=1000`
  - corpus SHA256 `07275cfd5a7a4b415ddf5eacc086de98294ac978532df46ffae30f9202323a95`
  - queries SHA256 `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782`

## Local Data Not Committed

The parquet and TSV outputs are local ignored data under `data/`. They are not
committed in this packet.
