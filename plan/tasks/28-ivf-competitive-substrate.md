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

1. **Heap-rerank heap-page prefetch**
   - PG18 path should prefetch candidate heap blocks before heap-f32 rerank.
   - Keep MVCC visibility and tuple fetch semantics unchanged.
   - Validate with focused heap-f32 and `ec_ivf` PG18 tests.

2. **Index-internal rerank scoring kernel**
   - Keep SQL `<#>` byte-stable semantics on the SQL function path.
   - Add a separate index-internal f32 IP helper that can use the faster
     unrolled/SIMD source-score kernel.
   - Wire IVF `heap_f32` rerank to the internal helper.

3. **Post-optimization sweep**
   - Re-run the 10k and 25k DBPedia slices after items 1 and 2.
   - Sweep `nlists`, `nprobe`, and `rerank_width` again because the cost ratio
     changes after prefetch/scoring work.
   - Keep packet-local raw logs and manifests.

4. **Quantizer dispatch seam**
   - Replace hardcoded `ProdQuantizer::cached(...)` build/scan paths with an
     enum-dispatched scoring/prepared-query surface selected by reloption.
   - Keep unsupported profiles rejected until their concrete implementations
     land, but route the default profile through the same dispatch seam.

5. **Probe-candidate aggregation pressure**
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

## Initial Slice

The first slice is item 1: heap-rerank heap-page prefetch. It is intentionally
small because it touches only scan-time I/O scheduling and should not change
scores, order, visibility, or index format.
