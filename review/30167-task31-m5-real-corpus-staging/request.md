# Task 31 M5 Real Corpus Staging

Reviewer: please review this staging packet before the first real M5 IVF load
and baseline packet.

## Scope

This packet fetches the canonical DBPedia OpenAI 1536D parquet release and
prepares local TSV subsets for the Task 31 M5 IVF baseline surfaces. It does
not load those TSVs into PostgreSQL, build indexes, run recall, run latency, or
make storage/build-time claims.

All command logs are under `artifacts/`; `artifacts/manifest.md` is the
packet-local source of truth for commands, paths, row counts, and hashes.

## Fetch

Source:

- Dataset key: `qdrant-dbpedia-openai3-large-1536-1m`
- Repo: `Qdrant/dbpedia-entities-openai3-text-embedding-3-large-1536-1M`
- Revision: `main`
- Local parquet directory: `data/task31_m5_dbpedia_fetch/data`
- Shards: `26`

The first fetch attempt downloaded shards through `train-00016-of-00026.parquet`
and then hit a remote connection reset while reading shard 17. The rerun skipped
the existing 17 shards and downloaded the remaining 9:

- `fetch.log`: partial first attempt, stopped by remote reset
- `fetch-rerun.log`: completed with `downloaded=9 skipped=17`

The local fetch manifest is written at:

- `data/task31_m5_dbpedia_fetch/ecaz_fetch_manifest.json`

## Prepared Subsets

Prepared output directory:

- `data/task31_m5_dbpedia_staged`

| profile | corpus rows | query rows | corpus SHA256 | query SHA256 |
|---|---:|---:|---|---|
| `ec_hnsw_real_10k` | 10000 | 200 | `c67c5810b66d982d705974e48d4775479adfbd92a988f694091266e049a35e75` | `a2c191bb742017d849e73f6e6866e8e0f0bac1579ba212f7fc76b8eb09904ae8` |
| `ec_hnsw_real_25k` | 25000 | 500 | `234739ca91125db9d68052fe65380c32b9f41b42aa339320d77915680197a405` | `80548c67c965dc8f22e793d0ec7af78c96d23e60793fd5c41311a5543b64d2f8` |
| `ec_hnsw_real_50k` | 50000 | 1000 | `56023baaa7bc42f758272e8617603d538808e6290a8a70a3a84e057571240133` | `95ac7992578aa80bb193657f10fbcbf1ea3867e559739244bf5a467f7a5a9fa3` |
| `ec_hnsw_real_100k` | 100000 | 1000 | `07275cfd5a7a4b415ddf5eacc086de98294ac978532df46ffae30f9202323a95` | `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782` |

The 25k and 100k subset recipes depend on code checkpoint `bfee0e29`, reviewed
in packet `30168-task31-corpus-prepare-profiles`.

## Commands

Fetch:

```sh
/Users/peter/.cargo/bin/ecaz --log-file review/30167-task31-m5-real-corpus-staging/artifacts/fetch.log \
  corpus fetch --output-dir data/task31_m5_dbpedia_fetch

/Users/peter/.cargo/bin/ecaz --log-file review/30167-task31-m5-real-corpus-staging/artifacts/fetch-rerun.log \
  corpus fetch --output-dir data/task31_m5_dbpedia_fetch
```

Prepare:

```sh
/Users/peter/.cargo/bin/ecaz corpus prepare --profile ec_hnsw_real_10k \
  --parquet data/task31_m5_dbpedia_fetch/data \
  --output-dir data/task31_m5_dbpedia_staged \
  --log-file review/30167-task31-m5-real-corpus-staging/artifacts/prepare-10k.log

/Users/peter/.cargo/bin/ecaz corpus prepare --profile ec_hnsw_real_25k \
  --parquet data/task31_m5_dbpedia_fetch/data \
  --output-dir data/task31_m5_dbpedia_staged \
  --log-file review/30167-task31-m5-real-corpus-staging/artifacts/prepare-25k.log

/Users/peter/.cargo/bin/ecaz corpus prepare --profile ec_hnsw_real_50k \
  --parquet data/task31_m5_dbpedia_fetch/data \
  --output-dir data/task31_m5_dbpedia_staged \
  --log-file review/30167-task31-m5-real-corpus-staging/artifacts/prepare-50k.log

/Users/peter/.cargo/bin/ecaz corpus prepare --profile ec_hnsw_real_100k \
  --parquet data/task31_m5_dbpedia_fetch/data \
  --output-dir data/task31_m5_dbpedia_staged \
  --log-file review/30167-task31-m5-real-corpus-staging/artifacts/prepare-100k.log
```

## Next Checkpoint

Create `30169-task31-m5-pqg8-10k-load-baseline` and load only the 10k real
surface first:

```sh
/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 \
  --log-file review/30169-task31-m5-pqg8-10k-load-baseline/artifacts/load_real10k_pqg8_n64_w750.log \
  corpus load --prefix task31_m5_real10k_pqg8_n64 \
  --profile ec_ivf \
  --corpus-file data/task31_m5_dbpedia_staged/ec_hnsw_real_10k_corpus.tsv \
  --queries-file data/task31_m5_dbpedia_staged/ec_hnsw_real_10k_queries.tsv \
  --manifest-file data/task31_m5_dbpedia_staged/ec_hnsw_real_10k_manifest.json \
  --reloption storage_format=pq_fastscan \
  --reloption pq_group_size=8 \
  --reloption nlists=64 \
  --reloption nprobe=48 \
  --reloption rerank=heap_f32 \
  --reloption rerank_width=750
```

After load/build succeeds, run the 10k recall, latency/HWM, storage, and
EXPLAIN/counter captures from packet `30165`. Do not start 25k/100k loads until
the 10k real baseline path is verified end to end.

## Validation

For the CLI profile support used by this packet:

```sh
cargo test -p ecaz-cli corpus::prepare
```

Result: passed, `24` prepare tests. The test run is recorded in packet
`30168-task31-corpus-prepare-profiles`.

No PostgreSQL load, recall, latency, storage, or EXPLAIN validation was run in
this staging packet.

## Artifacts

- `artifacts/fetch.log`
- `artifacts/fetch-rerun.log`
- `artifacts/prepare-10k.log`
- `artifacts/prepare-25k.log`
- `artifacts/prepare-50k.log`
- `artifacts/prepare-100k.log`
- `artifacts/manifest.md`
