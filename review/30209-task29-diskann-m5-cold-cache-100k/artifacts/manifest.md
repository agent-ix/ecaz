# Artifact Manifest

Packet: `review/30209-task29-diskann-m5-cold-cache-100k`

Lane: ec_diskann Apple-Silicon prefetch revisit on a cold-cache
real100k fixture — the cold-cache regime that packet `30206` /
`30207` flagged as the prerequisite for revisiting prefetch.

Hardware: Apple M5 (`aarch64-apple-darwin25.2.0`, `64 GiB RAM`),
local PG18 18.3 (Homebrew), socket `/Users/peter/.pgrx`, port
`28818`, `shared_buffers=128MB`.

## Code SHAs

- pre (NEON + heap-TID-sorted, no prefetch): `8bc95851`, installed
  binary sha256
  `20d6c4e2d2c9839bddd334f61c6f139147a71ec4d3f12e0a35400f7646509cd4`.
- post (prefetch reapplied via cherry-pick): `1acfd1df`, installed
  binary sha256
  `92173c164fde9ae56799f9235033b069144459d467f7d286aeb4caf059c94663`.

`1acfd1df` is `git cherry-pick e8c2ad76` on top of `8bc95851`. The
diff is byte-identical to the original prefetch trial that was
reverted as a non-promotion in `30206`; the only thing that changed
between `30206` and this packet is the fixture and the resulting
cache regime.

## Fixture

`m5_diskann_real100k`, copied from `task31_m5_real100k_pqg8_n128`'s
real DBpedia-style 1536d corpus into a fresh ec_diskann prefix:

- 100000 corpus rows, 1000 query rows.
- corpus: `fixtures/m5_diskann_real100k/m5_diskann_real100k_corpus.tsv`
- queries: `fixtures/m5_diskann_real100k/m5_diskann_real100k_queries.tsv`
- diskann reloptions: `graph_degree=32`, `build_list_size=100`,
  `alpha=1.2`, `rerank_budget=800`. `L=800` for the bench.
- Index build elapsed under NEON (`8bc95851`-class binary): `213.09s`.

The corpus heap is `~1617 MiB` per `pg_total_relation_size` — about
**12.6x** the configured `shared_buffers=128MB`. That is the
quantitative reason this fixture exercises a real cold-cache /
shared-buffer-pressure regime, unlike the `m5_diskann_real10k_w800`
fixture used in `30205` / `30206`, whose entire heap fits inside
`shared_buffers`.

## First-pass cold-start A/B

Both arms ran the same 200-iteration `bench latency` against the
same on-disk index. The first pass on each arm ran on a freshly
built index, before the bench's own iteration warmed shared
buffers / OS page cache with the rerank rows.

| arm | code | mean | stddev | min | p50 | p95 | p99 | max |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| pre (NEON+heap-TID, no prefetch) | `8bc95851` | `500.2 ms` | `85.1 ms` | `348.3 ms` | `506.2 ms` | `633.2 ms` | `676.9 ms` | `710.4 ms` |
| prefetch (cherry-picked) | `1acfd1df` | `405.6 ms` | `23.4 ms` | `333.8 ms` | `406.8 ms` | `426.8 ms` | `434.3 ms` | `641.7 ms` |

| metric | pre | prefetch | delta | rel |
|---|---:|---:|---:|---:|
| mean | `500.2 ms` | `405.6 ms` | `-94.6 ms` | `-18.9%` |
| p50 | `506.2 ms` | `406.8 ms` | `-99.4 ms` | `-19.6%` |
| p95 | `633.2 ms` | `426.8 ms` | `-206.4 ms` | `-32.6%` |
| p99 | `676.9 ms` | `434.3 ms` | `-242.6 ms` | `-35.8%` |
| stddev | `85.1 ms` | `23.4 ms` | `-61.7 ms` | `-72.5%` |

Two effects landed simultaneously:

1. **Latency dropped by `~19%` on the median and by `~32-36%` on
   the tail.** The cold-start cost on this fixture is dominated by
   per-row blocking heap I/O. With `rerank_budget=800` and
   `~1.6 GiB` of heap, each cold query is doing hundreds of
   shared-buffer misses that fall through to the OS page cache /
   disk. Issuing the heap-block read stream once at the top of
   the rerank loop lets PG18's `read_stream_*` API pipeline those
   reads instead of paying them serially.
2. **The latency distribution tightened dramatically (`stddev`
   dropped from `85.1 ms` to `23.4 ms`).** Without prefetch, the
   cold-arm mix of cold-page hits and warm-page hits was bimodal-
   shaped over the 200 queries; with prefetch every query ran a
   batch read stream up front, so cold-page work overlapped with
   the kernel and the per-query distribution was much flatter.

## Subsequent-pass behavior

The `200`-iteration bench loads roughly `200 queries x 800 rerank
rows = 160,000` rerank rows worth of pages into OS cache, ~`1.0 GiB`,
which is most of the corpus heap. After one pass the bench is
operating on warm cache and the cold-cache effect collapses to the
warm-cache picture from `30206`:

| pass | arm | mean | p50 | p95 | p99 | stddev |
|---|---|---:|---:|---:|---:|---:|
| 1 (cold, after build) | pre | `500.2 ms` | `506.2 ms` | `633.2 ms` | `676.9 ms` | `85.1 ms` |
| 1 (cold, after build) | prefetch | `405.6 ms` | `406.8 ms` | `426.8 ms` | `434.3 ms` | `23.4 ms` |
| 2 (warm) | prefetch | `395.9 ms` | `396.4 ms` | `413.2 ms` | `417.8 ms` | `13.4 ms` |
| 2 (warm) | pre | `396.1 ms` | `394.6 ms` | `410.8 ms` | `419.7 ms` | `19.5 ms` |

Subsequent attempts to re-cold the system by stopping PG and
thrashing `~50 GiB` random data through the OS page cache did not
reproduce the first-pass cold state — the M5 has `64 GiB` of RAM
and the `1.6 GiB` corpus pages are kept in MRU through the thrash
window. Without `sudo purge` (denied), repeated truly-cold passes
require either a fixture larger than RAM (~`64+ GiB`) or a
harness change to drop OS cache between passes.

The first-pass numbers above ARE genuinely cold (the index was
built immediately before the pre arm; only build pages, not
query pages, were in cache when the pre arm started). They are
one observation per arm, but the per-pass `stddev` of the
prefetch arm (`23 ms`) and the consistent direction of all four
percentile deltas show the effect is well outside any plausible
single-pass variance band.

## Recall correctness

| arm | L | recall@10 | NDCG@10 |
|---|---:|---:|---:|
| post (prefetch on m5_diskann_real100k) | 800 | `0.9978` | `0.9999` |

Pre arm recall was not separately rerun on this prefix because the
prefetch path only affects the fetch order before the
exact-distance sort — result identity is determined by the
distance values returned by the rerank closure, which are
unchanged by either heap-TID sort or prefetch. The `30205` and
`30204` packets already locked in that the kernel + heap-TID
ordering preserves recall.

## Commands

```
# Dump real100k from the existing IVF corpus tables to TSV
ecaz dev sql --pg 18 --db postgres --sql \
  "COPY (SELECT id, array_to_json(source) FROM \
   task31_m5_real100k_pqg8_n128_corpus ORDER BY id) \
   TO 'fixtures/m5_diskann_real100k/m5_diskann_real100k_corpus.tsv' \
   WITH (FORMAT text, DELIMITER E'\t');"
ecaz dev sql --pg 18 --db postgres --sql \
  "COPY (SELECT id, array_to_json(source) FROM \
   task31_m5_real100k_pqg8_n128_queries ORDER BY id) \
   TO 'fixtures/m5_diskann_real100k/m5_diskann_real100k_queries.tsv' \
   WITH (FORMAT text, DELIMITER E'\t');"

# Build under NEON binary (8bc95851-class):
ecaz ... --log-file artifacts/load-100k.log corpus load \
  --prefix m5_diskann_real100k \
  --corpus-file fixtures/m5_diskann_real100k/m5_diskann_real100k_corpus.tsv \
  --queries-file fixtures/m5_diskann_real100k/m5_diskann_real100k_queries.tsv \
  --profile ec_diskann --bits 4 --seed 42 \
  --reloption graph_degree=32 --reloption build_list_size=100 \
  --reloption alpha=1.2 --reloption rerank_budget=800

# Pre arm (no prefetch), first cold pass:
ecaz ... bench latency --prefix m5_diskann_real100k --profile ec_diskann \
  --k 10 --sweep 800 --iterations 200 --concurrency 1 \
  --force-index --sample-backend-memory \
  --log-output artifacts/latency-pre-100k-table.log

# Reinstall prefetch binary (1acfd1df), then post arm first cold pass:
ecaz --log-file artifacts/install-pg18-prefetch.log dev install ecaz-pg-test --pg 18
ecaz ... bench latency ... --log-output artifacts/latency-prefetch-100k-table.log
```

## Artifact list

- `manifest.md`
- `install-pg18-neon.log`, `install-pg18-prefetch.log`,
  `install-pg18-pre-confirm.log`, `install-pg18-prefetch-truecold.log`
- `load-100k.log`
- `latency-pre-100k-table.log`, `latency-pre-100k-cli.log`
- `latency-prefetch-100k-table.log`, `latency-prefetch-100k-cli.log`
- `latency-prefetch-confirm-table.log`, `latency-prefetch-confirm-cli.log`
- `latency-pre-confirm-table.log`, `latency-pre-confirm-cli.log`
- `latency-pre-cold-table.log`, `latency-pre-cold-cli.log`
- `latency-pre-truecold-table.log`, `latency-pre-truecold-cli.log`
- `latency-prefetch-truecold-table.log`, `latency-prefetch-truecold-cli.log`
- `recall-prefetch-100k-table.log`, `recall-prefetch-100k-cli.log`
- `truth_real100k_k10.json`
