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

7. **Concurrency measurement**
   - Measure per-list insert serialization under concurrent insert load before
     making broad concurrency claims.

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

## Next Slice

The next slice is to continue item 3 with `nlists=64/128` isolated surfaces
and a lower `nprobe` sweep, then use items 6 and 7 to document deeper
build/training/vacuum risks before product benchmarking.
