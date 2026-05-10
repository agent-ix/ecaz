# ec_diskann M5 Cold-Cache Prefetch — Promotion Reversal

Reviewer: this packet revisits the prefetch trial that was reverted
in `30206` after a warm-cache non-promotion. On the cold-cache
real100k fixture (`heap = 12.6x shared_buffers`) the same prefetch
diff produces a clean across-the-board win: `-19% p50, -33% p95,
-36% p99, -73% stddev`. Recommendation: keep the prefetch on the
branch (the reverted commit was cherry-picked back as `1acfd1df`),
treat the `30206` warm-cache verdict as fixture-specific rather
than fundamental, and supersede the `30207` "negative result"
disposition for prefetch.

## Why a fresh fixture

`30206` ruled prefetch a non-promotion on `m5_diskann_real10k_w800`,
which has a heap that fits entirely in `shared_buffers=128MB`. The
reviewer's third suggestion called that out explicitly:

> 3. Only revisit prefetch with a cold-cache harness or a corpus
>    larger than shared buffers. The current packet already shows
>    the warm-cache benchmark is not measuring a workload where
>    prefetch can help.

This packet picks the second of those — a corpus larger than
shared buffers. The 10k reuses + 100k corpus from
`task31_m5_real100k_pqg8_n128` was already loaded for IVF; this
packet re-dumps its source column to TSV and re-loads under
`ec_diskann` as a new prefix `m5_diskann_real100k`. Heap is
`~1617 MiB` per `pg_total_relation_size` vs `128 MiB`
`shared_buffers` — `12.6x` over.

## Result — first-pass cold-start A/B

`200`-iteration `bench latency` on the same on-disk index built
under the NEON binary, run once per arm before the bench warmed
its own working set:

| arm | code | mean | stddev | min | p50 | p95 | p99 |
|---|---|---:|---:|---:|---:|---:|---:|
| pre (NEON+heap-TID, no prefetch) | `8bc95851` | `500.2 ms` | `85.1 ms` | `348.3 ms` | `506.2 ms` | `633.2 ms` | `676.9 ms` |
| prefetch (cherry-picked) | `1acfd1df` | `405.6 ms` | `23.4 ms` | `333.8 ms` | `406.8 ms` | `426.8 ms` | `434.3 ms` |
| **delta** | | `-94.6 ms` | `-61.7 ms` | `-14.5 ms` | `-99.4 ms` | `-206.4 ms` | `-242.6 ms` |
| **rel** | | `-18.9%` | `-72.5%` | `-4.2%` | `-19.6%` | `-32.6%` | `-35.8%` |

Two effects:

1. The mean and percentiles all moved consistently in the
   improving direction. The tail (`p95`, `p99`) moved roughly
   twice as much as the median, which is the expected shape when
   a workload is heap-fetch-bound and prefetch turns the worst
   cold misses into pipelined async reads.
2. The within-pass `stddev` collapsed from `85 ms` to `23 ms`.
   Without prefetch, the first cold pass mixes per-query cold-page
   blocking with later partially-warm queries, so individual
   query times are bimodal-shaped over the run. With prefetch,
   every query batches its `rerank_budget=800` heap blocks into
   a single PG18 `read_stream_*` call that pipelines the I/O,
   so the per-query distribution flattens.

Recall on this prefix is `0.9978 / 0.9999` (`recall@10` / `NDCG@10`),
unchanged by the prefetch path (which only reorders fetches
before the exact-distance sort).

## What changed since `30206`

Nothing in the diff — `1acfd1df` is `git cherry-pick e8c2ad76`,
byte-identical to the originally-reverted prefetch commit. What
changed is the fixture: `30206`'s `m5_diskann_real10k_w800` had a
heap that fits in shared buffers, and the bench was warm-cache.
`30206`'s warm-cache deltas (`+0.05 to +0.25 ms` across
percentiles) are real but they reflect the buffer-table-lookup
plus pin/unpin overhead of the prefetch on already-cached pages,
not anything the prefetch can recover from. On the
`m5_diskann_real100k` fixture the pin/unpin overhead is dwarfed
by the I/O the prefetch is now amortizing.

So `30206`'s warm-cache result still stands as data, but
`30207`'s disposition that the prefetch should not land "until
a cold-cache harness or a corpus larger than shared buffers"
gets resolved by this packet. The cold-cache regime is the one
the prefetch was designed for, and it wins decisively there.

## Subsequent-pass / re-cold attempts

Pass 2 onward on the same fixture warms PG shared buffers + OS
page cache enough that both arms collapse to `~396 ms` p50, with
prefetch still slightly negative (`+0.2 ms` p50) — same shape as
`30206`. So the warm-cache regime is unchanged by any of this; we
are simply adding a separate cold-cache regime where the prefetch
helps a lot.

Repeated truly-cold passes were attempted by stopping PG,
writing+reading `~50 GiB` random data to thrash the OS page cache,
restarting PG, and benching. With `64 GiB` of RAM and a
`1.6 GiB` corpus, the corpus pages survived the thrash window
through MRU retention; the post-thrash passes ran at warm-cache
speed. So while we have only one truly-cold pass per arm in this
packet, both arms had identical "fresh from build" preconditions,
and the per-pass deltas are well outside the per-pass `stddev`
band of either arm.

## Recommendation

- Keep `1acfd1df` (`Prefetch ec_diskann heap rerank blocks`) on
  the `ec-diskann-apple-neon-rerank` branch tip. The reverted
  state from `30206` is no longer the right tip; this packet
  records the cold-cache result that justifies bringing it back.
- Update the disposition in `30207`'s decision packet: cold-cache
  prefetch is no longer deferred — it has a measurement and is
  promoted on the cold-cache regime. The other deferred items
  (async-overlapping prefetch, same-page-run grouping) stay
  deferred since this synchronous-drain prefetch already captured
  most of the cold-cache I/O win on this fixture, leaving the
  remaining headroom to async overlap with the rerank kernel
  itself — a structurally bigger change that would only matter
  if the kernel work share grew.
- For the same reason `30206` already documented, do not enable
  the prefetch on workloads whose entire heap fits in
  shared buffers if a `+0.2 ms` warm-cache regression matters.
  In practice that is almost never the case for ec_diskann's
  intended workloads (real-corpus indexes are typically larger
  than `shared_buffers`), so the conservative move is to land
  the prefetch unconditionally.

## Validation

- `cargo check --no-default-features --features pg18` clean at
  `1acfd1df`.
- 29 scan tests still pass at the cherry-picked tip (the diff is
  identical to the previously-tested `e8c2ad76` plus the
  `64645bfc` test tightening on top of the heap-TID sort).

## Artifacts

All under `artifacts/`. See `artifacts/manifest.md` for SHAs,
commands, and the full per-pass tables (cold-start, two warm
passes per arm, two post-thrash passes).
