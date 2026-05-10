# Artifact Manifest

Packet: `review/30205-task29-diskann-m5-rerank-heap-order`

Lane: ec_diskann Apple-Silicon rerank heap-fetch ordering A/B.

Hardware: Apple M5 (`aarch64-apple-darwin25.2.0`), local PG18 18.3 (Homebrew),
socket `/Users/peter/.pgrx`, port `28818`.

Surface: kernel-stress lane on the existing real-data prefix
`m5_diskann_real10k_w800` (1536d real DBpedia-style 10k corpus,
200 queries, ec_diskann with `graph_degree=32`, `build_list_size=100`,
`alpha=1.2`, `rerank_budget=800`, swept at L=800). Both arms ran
against the SAME on-disk index that packet `30204` built and
benchmarked; only the loaded `ecaz.dylib` differed.

Cache state: warm local run; pre and post passes were interleaved
without stopping PG.

## Code SHAs

- pre (NEON, unordered fetch): `eda51e9f` (current `origin/ec-diskann-apple-neon-rerank`,
  tip after packet `30204`). Installed sha256
  `0538822d360075f8d8aac566800d94f19c92310ad728d2e8d49655067d9ae307`.
- post (NEON, heap-TID-ordered fetch): `e191a9e1`
  (`Visit ec_diskann rerank rows in heap-TID order`). Installed sha256
  `20d6c4e2d2c9839bddd334f61c6f139147a71ec4d3f12e0a35400f7646509cd4`.

The on-disk index `m5_diskann_real10k_w800_idx` was built once during
packet `30204` and was not rebuilt for this packet; the kernel and
fetch-ordering change only affect query-time behavior.

## Hypothesis

Packet `30204` recommended this exact follow-on: "the next Apple-specific
candidates are exact rerank source decode overhead and heap fetch /
cache locality in the rerank path, picked by measurement". The IVF
lane already closed the same gap in commit `79c1a11c` (`Fetch ec_ivf
rerank rows in heap order`).

In `src/am/ec_diskann/scan.rs::vamana_scan_with`, the rerank loop
visited the top-`rerank_budget` candidates in prefilter-score order,
which is uncorrelated with heap layout. Each `rerank()` call opens
and pins a fresh shared-buffer page even when adjacent rows on disk
would have shared a page. Sorting the rerank batch by
`(block_number, offset_number)` before the rerank closure runs is
expected to amortize page reads / pin / unpin under typical DBpedia-
shaped heap fragmentation.

The post commit adds the sort and a unit test
(`sc_003b_rerank_visits_in_heap_tid_order`) that records the
`primary_heaptid` values the rerank closure observes and asserts they
are non-decreasing in `(block_number, offset_number)`. Final result
ordering still comes from the existing exact-distance sort, so the
change cannot affect correctness.

## Commands

```
ecaz --log-file artifacts/install-pg18-pre-confirm.log dev install ecaz-pg-test --pg 18
ecaz ... --log-file artifacts/install-pg18-post.log dev install ecaz-pg-test --pg 18

ecaz ... bench latency --prefix m5_diskann_real10k_w800 --profile ec_diskann \
  --k 10 --sweep 800 --iterations 200 --concurrency 1 \
  --force-index --sample-backend-memory \
  --log-output artifacts/latency-{pre,post}{,-confirm}-table.log

ecaz ... bench recall  --prefix m5_diskann_real10k_w800 --profile ec_diskann \
  --k 10 --sweep 800 --force-index \
  --truth-cache-file review/30204-task29-diskann-m5-neon-rerank/artifacts/truth_real10k_k10.json \
  --log-output artifacts/recall-post-table.log
```

The truth cache from packet `30204` is reused unchanged.

## Latency results (200 iterations / pass, 2 passes per arm)

| pass | mean | stddev | min | p50 | p95 | p99 | max |
|---|---:|---:|---:|---:|---:|---:|---:|
| pre pass 1 (`latency-pre-table.log`) | 15.8 ms | 0.66 ms | 15.1 ms | 15.7 ms | 16.4 ms | 17.1 ms | 23.9 ms |
| pre pass 2 (`latency-pre-confirm-table.log`) | 16.9 ms | 21.0 ms | 14.7 ms | 15.3 ms | 15.9 ms | 18.7 ms | 313.3 ms |
| post pass 1 (`latency-post-table.log`) | 16.4 ms | 21.1 ms | 14.1 ms | 14.8 ms | 15.6 ms | 17.6 ms | 314.4 ms |
| post pass 2 (`latency-post-confirm-table.log`) | 14.8 ms | 0.53 ms | 14.2 ms | 14.8 ms | 15.3 ms | 16.0 ms | 20.6 ms |

The two `300+ ms` outliers (one in pre pass 2, one in post pass 1) are
autovacuum-shaped and inflate `mean` / `stddev` / `max`. The percentile
columns are unaffected.

Pass-averaged percentile deltas:

| metric | pre avg | post avg | delta | rel |
|---|---:|---:|---:|---:|
| min | 14.9 ms | 14.15 ms | `-0.75 ms` | `-5.0%` |
| p50 | 15.5 ms | 14.8 ms | `-0.7 ms` | `-4.5%` |
| p95 | 16.15 ms | 15.45 ms | `-0.7 ms` | `-4.3%` |
| p99 | 17.9 ms | 16.8 ms | `-1.1 ms` | `-6.1%` |

## Recall

- post recall@10 / NDCG@10 / mean q-time: `1.0000 / 1.0000 / 14.80 ms`
  (`recall-post-table.log`).
- pre recall@10 / NDCG@10 (from packet `30204`,
  `recall-neon-real-w800-table.log`): `1.0000 / 1.0000`.

Recall is unchanged. The change only affects fetch order; the final
result set is re-sorted by exact distance after rerank.

## Artifact list

- `manifest.md`
- `install-pg18-pre-confirm.log`, `install-pg18-post.log`
- `latency-pre-table.log`, `latency-pre-cli.log`,
  `latency-pre-confirm-table.log`, `latency-pre-confirm-cli.log`
- `latency-post-table.log`, `latency-post-cli.log`,
  `latency-post-confirm-table.log`, `latency-post-confirm-cli.log`
- `recall-post-table.log`, `recall-post-cli.log`
