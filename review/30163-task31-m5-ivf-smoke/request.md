# Task 31 M5 IVF Smoke

Reviewer: please review this smoke packet before the first real Task 31 IVF baseline pass.

## Scope

This packet sets up a small local M5 smoke corpus through `ecaz-cli` and proves the PG18 IVF operator path end to end. It intentionally avoids latency sweeps, long baseline passes, and real-corpus benchmark claims.

All raw command logs are under `artifacts/`; `artifacts/manifest.md` is the packet-local source of truth for commands, metadata, and cited result lines.

## Environment Status

- Repo head at measurement time: `6429a9e4b9de5a36ab982670bd8151644c6a2af0`.
- Working tree was clean before this packet was created.
- CLI path used: `/Users/peter/.cargo/bin/ecaz`.
- PG18 status: reachable through the pgrx socket at `/Users/peter/.pgrx`, port `28818`.
- PostgreSQL: `18.3 (Homebrew)` on `aarch64-apple-darwin25.2.0`.
- Extension: `ecaz 0.1.1`.
- Starting database state: `ecaz corpus list` showed no corpora loaded in `postgres`.

One local setup note: `ecaz` was not initially on shell `PATH`, so it was installed from `crates/ecaz-cli` and then invoked by absolute path. The later socket operations all used `ecaz`, matching the intended approval-minimizing operator surface.

## Corpus

Path: `data/task31_m5_smoke/`

- Corpus TSV: `task31_m5_smoke_corpus.tsv`
- Query TSV: `task31_m5_smoke_queries.tsv`
- Corpus rows: `10000`
- Query rows: `20`
- Dimensions: `1536`
- Corpus seed: `31`
- Query seed: `3100`
- Loader corpus SHA256: `38dde7700ef3d60357035833aa7eb101a834264044f1ba24506c52d494fa3a89`
- Loader query SHA256: `0c381a769a984698e9bc3863f74b0ffefab2bbbc2ce9974d2969d7abf544b180`

The generated corpus files are local ignored data, not committed in this review-packet commit.

## IVF Load

Loaded prefix: `task31_m5_smoke_pqg8`

Profile and reloptions:

- `profile=ec_ivf`
- `storage_format=pq_fastscan`
- `pq_group_size=8`
- `nlists=128`
- `nprobe=8`
- `rerank=heap_f32`
- `rerank_width=500`

Load result from `artifacts/load.log`:

- Copied `10000` corpus rows.
- Copied `20` query rows.
- Built `task31_m5_smoke_pqg8_idx` in `4.76s`.
- Completed load/build for the prefix in `7.21s`.

Inspect/list result:

- `artifacts/corpus-inspect.log` shows `task31_m5_smoke_pqg8_corpus (10000 rows)`, `task31_m5_smoke_pqg8_queries (20 rows)`, and the requested IVF reloptions.
- `artifacts/corpus-list-after.log` shows the loaded prefix with `btree, ec_ivf` indexes and `ec_ivf` profile.

## Smoke Results

Tiny recall smoke only:

- Command used `k=10`, `queries-limit=3`, `sweep=8`, `rerank-width=500`, and `--force-index`.
- Result from `artifacts/recall_q3-table.log`: `nprobe=8`, `recall@k=0.1667`, `ndcg@k=0.8428`, `mean q-time=3.49 ms`.
- This is a plumbing smoke, not a quality or latency claim.

Tiny storage smoke:

- Result from `artifacts/storage.log`: table total `159.4 MiB`, indexes `3.7 MiB`, IVF index `3.2 MiB`, IVF index per row `338.3 B`.

## Next Command

First real Task 31 IVF baseline should move from this synthetic smoke to the landed PQ-FastScan group-size-8 baseline surface, using the release-installed extension and packet-local logs. Start with the smallest real surface before 25k/100k:

```sh
/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 bench recall \
  --prefix <real_10k_pqg8_prefix> \
  --profile ec_ivf \
  --k 10 \
  --sweep 8 \
  --rerank-width 500 \
  --force-index \
  --truth-cache-file review/<next-packet>/artifacts/truth_k10.json \
  --log-output review/<next-packet>/artifacts/recall_10k_table.log
```

After recall is confirmed on the real 10k prefix, run the matching `ecaz bench latency` and `ecaz bench storage` commands in the next packet; those were intentionally not run here.
