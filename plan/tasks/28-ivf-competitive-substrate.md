# Task 28 Follow-On: IVF Competitive Substrate

Status: active follow-up on `task28-ivf`
Owner: coder1 / runtime-index track

## Goal

Raise `ec_ivf` from a correct local tuning substrate to a competitive IVF
foundation before any product benchmark claim is made.

The first Task 28 slice established correctness and a local DBPedia-derived
frontier. The next slice closes the latency and extension-seam gaps called out
in review packet 30047 feedback seq 02.

## Non-Goals

- No DiskANN implementation. DiskANN remains task 29.
- No Graviton-class product claim from local development measurements.
- No paper-specific quantizer landing until the base IVF quantizer seam is real.

## Required Work Before Product Benchmarking

1. **Heap-rerank heap-page prefetch** - done in `3ef4442`
   - PG18 path should prefetch candidate heap blocks before heap-f32 rerank.
   - Keep MVCC visibility and tuple fetch semantics unchanged.
   - Validate with focused heap-f32 and `ec_ivf` PG18 tests.

2. **Index-internal rerank scoring kernel** - done in `a1eda50`
   - Keep SQL `<#>` byte-stable semantics on the SQL function path.
   - Add a separate index-internal f32 IP helper that can use the faster
     unrolled/SIMD source-score kernel.
   - Wire IVF `heap_f32` rerank to the internal helper.

3. **Post-optimization sweep**
   - Re-run the 10k and 25k DBPedia slices after items 1 and 2.
   - Sweep `nlists`, `nprobe`, and `rerank_width` again because the cost ratio
     changes after prefetch/scoring work.
   - Keep packet-local raw logs and manifests.
   - First post-optimization smoke is recorded in packet 30051 for
     `nlists=32`, `rerank_width=25`, `nprobe in {16,24,32}`. It confirms
     correctness but shows latency still above the 10k target, so higher
     `nlists` and narrower rerank-width surfaces are still required.
   - Packet 30052 records the `nlists=64`, `rerank_width=25` continuation.
     It improves the curve, but 10k still only reaches sub-50 ms p50 at
     `recall@10=0.7800`; high-recall points remain above the target.
   - Packet 30053 records the `nlists=128`, `rerank_width=25` build and a
     planner blocker. Normal planning selected a sequential scan for the n128
     surface, while forcing the IVF index path returned one nprobe=8 query in
     67.987 ms. Do not quote n128 recall/latency until the benchmark force-index
     mode or cost model is fixed.
   - Packet 30054 reruns n128 through the new benchmark `--force-index` mode.
     It confirms n128 is not the better frontier on this fixture: 10k
     `nprobe=64` reaches only `recall@10=0.9860` at p50 104.7 ms, and lower
     probes only hit sub-50 ms p50 at low recall.
   - Packet 30055 tests rerank-width reduction on the n64 surface. Width 10
     materially reduces recall with almost no latency win, and a 10k width 5
     spot check collapses recall to 0.5000 while staying in the same latency
     band. Do not keep shaving rerank width as the next latency lever.

4. **Quantizer dispatch seam** - done in `0e9202d`
   - Replace hardcoded `ProdQuantizer::cached(...)` build/scan paths with an
     enum-dispatched scoring/prepared-query surface selected by reloption.
   - Keep unsupported profiles rejected until their concrete implementations
     land, but route the default profile through the same dispatch seam.

5. **Probe-candidate aggregation pressure** - done in `dc1f369`
   - Investigate pooling or bounded aggregation for per-query probe candidates.
   - Prefer a bounded/deduplicating structure if it preserves recall and avoids
     unbounded per-query `HashMap` churn.

6. **Build/training/vacuum deeper pass**
   - Confirm sampled training behavior and 1M-scale training limits.
   - Read `build.rs`, `training.rs`, and `vacuum.rs` for product-gate risks.
   - Add follow-up packets for any correctness or performance findings.
   - Packet 30056 records the deeper pass. Training is sampled, but build
     still retains/stages the full tuple set. Vacuum repairs counts and marks
     empty postings deleted, but does not compact/reclaim posting pages.
     Live insert remains the hottest product-gate concern because it reloads
     centroids, scans all postings for duplicate heap TIDs, and updates list
     plus metadata counters per row.

7. **Concurrency measurement**
   - Measure per-list insert serialization under concurrent insert load before
     making broad concurrency claims.
   - Packet 30056 adds `ecaz stress ivf-insert` and records a synthetic PG18
     run: 1 worker inserted 668 rows in 10s, while 4 workers inserted 1592
     rows in 10s. The run also found and fixed a large-build live-insert
     directory traversal bug in commit `43563e5`.
   - Packet 30057 narrows live-insert duplicate heap-TID checking to the
     assigned list. On the same synthetic PG18 harness, 1-worker throughput
     improved from 66.80 rows/s to 275.30 rows/s, and 4-worker throughput
     improved from 159.20 rows/s to 657.50 rows/s.

## Completed Slices

- Item 1: heap-rerank heap-page prefetch. It touches only scan-time I/O
  scheduling and should not change scores, order, visibility, or index format.
- Item 2: index-internal rerank scoring. It keeps SQL `<#>` on the
  byte-stable scalar helper and routes IVF heap-f32 rerank through the
  index-internal source scorer.
- Item 4: quantizer dispatch seam. It keeps v1 `auto`/`turboquant` behavior
  on the existing product quantizer while routing build encode, scan query
  preparation, and posting score through an IVF-local enum dispatch layer.
- Item 5: probe-candidate aggregation pressure. It moves the per-query
  heap-tid dedup `HashMap` onto the scan opaque so rescans clear and reuse the
  allocation instead of allocating a fresh map for every query.
- Item 6/7 first pass: build/training/vacuum review plus insert-concurrency
  measurement. Packet 30056 records the sampled-training confirmation, vacuum
  compaction gap, live-insert hot-path risks, and the fixed directory traversal
  bug discovered by the new insert stress harness.
- Item 7 hot-path follow-up: assigned-list duplicate checking. Packet 30057
  records the `bfbb40d` optimization and the post-change insert stress result.

## Next Slice

The next slice is the remaining live-insert fixed per-row work: centroid model
reload, one-posting-per-row append shape, and list-directory plus metadata
counter writes. Keep cost-model repair and posting-list scoring/layout work on
the active backlog: n128 normal planning still falls back to sequential scan,
and packet 30055 shows rerank-width reduction is not the missing high-recall
latency lever.
