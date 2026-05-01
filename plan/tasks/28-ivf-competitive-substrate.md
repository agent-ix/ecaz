# Task 28 Follow-On: IVF Competitive Substrate

Status: **landed on `main` for the local IVF competitive-substrate lane**.
The former A1-A10 merge gate is closed for local v1 landing; larger product
benchmarks and fresh 990k exact fills are deferred to dedicated benchmark
hardware.
Owner: coder1 / runtime-index track

## Post-Merge Summary

Task 28's local landing status is consolidated in
`review/30151-task28-ivf-local-landing-status/`, with the remaining H/I cleanup
closed in `review/30153-task28-ivf-h-i-cleanups/`.

- A1-A8 and A10 are done for the local lane.
- A9 has local 100k/990k IVF evidence, with exact 990k fills and long HNSW
  reference rebuilds deferred out of the desktop gate by
  `review/30150-task28-ivf-local-990k-deferral/`.
- The measured local recommendation keeps `quantizer = 'auto'` unchanged and
  recommends explicit `storage_format = 'pq_fastscan', pq_group_size = 8` for
  larger high-dimensional IVF surfaces where speed and index size dominate.
- Further IVF work should be opened as a new, explicitly scoped follow-up, not
  treated as unfinished Task 28 merge-gate work.

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
  - Packet 30058 repairs the n128 normal-planner blocker. The prepared nprobe=8
    query now plans as an IVF index scan, and the 20-query normal benchmark
    smoke reports `recall@10=0.7000` at mean query time 40.98 ms without
    `--force-index`. This fixes the planner path; it does not change the n128
    frontier conclusion from packet 30054.

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
  - Packet 30059 tried caching the insert centroid model in commit `f2314bb`.
    The same nlists=16 harness fell to 249.60 rows/s at 1 worker and
    635.50 rows/s at 4 workers, so the change was backed out in `ce7a2b0`.
    Do not treat centroid reload as the next measured live-insert lever unless
    a larger-list-specific benchmark shows otherwise.
  - Packet 30060 removes duplicate insert source-vector normalization in
    commit `647abd1`. The change is a correctness-preserving cleanup, not a
    measured throughput win: the same nlists=16 harness reported 261.00 rows/s
    at 1 worker and 649.70 rows/s at 4 workers, below the packet 30057
    reference. A fresh PG18 database created from current extension SQL reported
    273.20 rows/s at 1 worker and 656.20 rows/s at 4 workers with
    `ec_ivf_index_admin_snapshot` metrics available.
  - Packet 30062 tried combining the per-insert directory and metadata counter
    rewrites into one generic WAL transaction. Tests passed, but the fresh
    nlists=16 harness fell to 265.20 rows/s at 1 worker and 645.10 rows/s at 4
    workers, so the change was backed out and should not be treated as the next
    live-insert lever.
  - Packet 30063 tried a live-insert-specific one-TID posting encoder to avoid
    constructing an `IvfPostingTuple` with a one-element heap-TID vector. Tests
    passed, but the fresh nlists=16 harness reported 267.80 rows/s at 1 worker
    and 650.20 rows/s at 4 workers, so the change was backed out.
  - Packet 30064 adds a `--dimensions` flag to `ecaz stress ivf-insert`.
    The existing 4D fixture remains the default, and a 1536D PG18 smoke passed
    with admin snapshot metrics. Use this before drawing more live-insert
    conclusions from the 4D fixture alone.
  - Packet 30065 records the first 1536D insert baseline on the fresh PG18
    database: 124.30 rows/s at 1 worker and 393.60 rows/s at 4 workers. Compared
    with packet 30060's fresh 4D runs (273.20 and 656.20 rows/s), dimensional
    source-vector/assignment/encoding work is a major live-insert cost.

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
- Item 7 negative follow-up: insert centroid-model caching. Packet 30059
  records the `f2314bb` trial and `ce7a2b0` backout after the nlists=16 stress
  harness failed to improve.
- Item 7 cleanup follow-up: duplicate source-vector normalization. Packet
  30060 records commit `647abd1`; keep it as a small hot-path quality cleanup,
  but do not count it as a live-insert throughput improvement.
- Item 7 negative follow-up: combined directory/metadata insert stats WAL.
  Packet 30062 records the trial and backout after the fresh PG18 insert stress
  harness failed to improve.
- Item 7 negative follow-up: single-posting encode helper. Packet 30063
  records the trial and backout after the fresh PG18 insert stress harness
  failed to improve.
- Item 7 harness follow-up: configurable insert-stress dimensions. Packet
  30064 records `656b2dc` and a 1536D PG18 smoke, enabling a more
  production-like live-insert split between vector-dimension work and per-row
  write-path work.
- Item 7 dimension split: 1536D insert baseline. Packet 30065 records c1/c4
  measurements and shifts the next useful optimization target from 4D
  write-path micro-costs to dimension-dependent source-vector and assignment
  work.
- Planner cost repair for n128 smoke measurements. Packet 30058 records commit
  `077aae1`, where quantized posting scans are modeled below full f32 random
  I/O cost so the normal planner can choose IVF for prepared benchmark queries.

## Historical Next Slice

This was the pre-landing next-slice note. The work was superseded by the later
A1-A10 packets and the landing-status packet above.

## Historical Merge Gate — From Reviewer Code Read 2026-04-27

**This section is retained as historical merge-gate context.** The branch has
landed on `main`; current status is summarized at the top of this file. These
items came out of the reviewer
code-read recorded in
`review/30070-task28-ivf-borrowed-scan-recall/feedback/2026-04-27-02-reviewer.md`.

Each item is independent of the others in implementation but ordered
in the sequencing section below for impact and risk.

Read every requirement literally. Where the action is "add," "wire up," or
"introduce," it is **purely additive** — no existing surface is removed or
deprecated unless the requirement explicitly says so.

### A1. Audit `cost.rs` constants against measured runtime cost

**Action:** verify the IVF planner-cost constants in
`src/am/ec_ivf/cost.rs` reflect actual runtime cost, not planner-selection
tuning. Specifically:

- `IVF_CENTROID_SCORING_DIMENSION_SCALE = 0.03` (was `0.75` before
  commit `077aae1`). A 25× reduction landed without a corresponding
  implementation change in centroid scoring. Either centroid scoring was
  quantized somewhere I did not read — in which case document the constant
  with a code reference — or the constant is hand-tuned to make the planner
  choose IVF and must be replaced with a faithful model.
- `IVF_POSTING_SCORING_DIMENSION_SCALE = 0.01`. Justify against the actual
  TurboQuant posting-scoring kernel cost. Cite a microbenchmark.
- `IVF_INDEX_PAGE_COST_SCALE = 0.25` applied to `seq_page_cost`. Justify
  against actual page-access pattern (warm-cache vs cold-cache).

**Done when:** each constant has either a microbenchmark-backed cost basis
documented in code comments, or has been adjusted to match measured cost.

**Cross-test:** confirm planner picks the correct path on a workload that
should *not* select IVF (e.g., low-selectivity scan, small table, no
ORDER BY on the indexed column). Add a test packet with the cross-test
matrix.

### A2. Replace full-list-into-memory vacuum with streaming bulkdelete

**Action:** rewrite `bulkdelete_list_postings` in
`src/am/ec_ivf/vacuum.rs` to walk posting pages incrementally instead of
calling `read_ivf_postings_for_list_blocks_with_tids` to materialize the
whole list as a `Vec<(tid, posting)>` before filtering.

**Why:** at `nlists=8` on a 10M-row corpus, a single list contains ≈1.25M
postings. Loading all of them into memory before filtering is a hard
scaling cliff. At product scale `ambulkdelete` will OOM or thrash.

**Done when:** vacuum walks one page at a time, applies the bulkdelete
callback per posting, rewrites the page in place, and updates running
totals. No data structure proportional to list size held in memory at
once. Add a packet measuring vacuum wall time and peak memory at
nlists ∈ {8, 32, 64} on at least 1M rows.

**Acceptance:** vacuum peak memory bounded by O(page_size), not
O(list_size).

### A3. Add physical-compaction support to vacuum

**Action:** extend vacuum so empty postings (those marked
`deleted = true` after all heap-TIDs are removed) reclaim their page
slots instead of remaining as on-disk tombstones forever.

**Why:** the current `vacuum.rs:210–212` path sets `posting.deleted =
true` and rewrites the tuple in place. The slot is never reclaimed. Over
a long-lived index with churn, the index never shrinks — same shape as
pgvector ivfflat. This is acceptable for v1 correctness, blocking for
any "vacuum reclaims space" claim.

**Done when:** vacuum either truncates trailing empty pages from a
posting list's block range, or rewrites the list to compact away
tombstones. Index size on a churn workload should track live tuple count,
not lifetime tuple count. Add a packet showing index-size convergence
under sustained insert+delete load.

### A4. Replace `exact_score_mode_name()` String comparison with typed enum match

**Action:** in `src/am/ec_ivf/quantizer.rs:81`, the dispatch into the
4-bit-LUT fast path uses
`quantizer.exact_score_mode_name() == "mse_no_qjl_4bit"`. Replace the
string comparison with a typed enum match on whatever the underlying
mode discriminator is in `crate::quant::prod`.

**Why:** string comparison on a mode name is fragile under refactor and
silently breaks if the underlying name changes. A typed match is
compile-time checked.

**Done when:** the dispatch arm in `prepare_ip_query` and
`score_ip_from_parts` matches a typed value, not a string. No literal
mode-name string appears in `quantizer.rs`.

### A5. Audit `ProdQuantizer::cached` cache key correctness across scans

**Action:** confirm that `ProdQuantizer::cached(dimensions, bits, seed)`
called from `IvfQuantizer::resolve` (and the build/insert call sites)
returns a cached instance keyed in a way that survives across scans and
across query executions on the same index. If the cache misses on every
call, the prepare cost is hitting every query.

**Why:** flagged in packet 30047 reviewer feedback as item 5; not
addressed by the dispatch refactor in commit `0e9202d`. The dispatch
refactor moved the call site but did not audit cache behavior.

**Done when:** there is either a microbenchmark showing the cache hits
across consecutive scans on the same index, or — if it doesn't — a fix
that makes it hit, plus a regression test asserting cache identity for a
fixed `(dimensions, bits, seed)` triple.

### A6. Revisit planner cost behavior at higher `nlists`

**Action:** packet 30053 noted that the planner did not select the IVF
index at `nlists=128`; packet 30058 patched the cost model so the
prepared-query path now does. Confirm the patch holds across:

- non-prepared queries (typical user workload),
- workloads with small `LIMIT`,
- workloads with large `LIMIT`,
- mixed `WHERE` predicates that combine the IVF column with other
  filters.

This is downstream of A1: if the constants in `cost.rs` are tuned to make
the planner pick IVF, they may make it pick IVF in cases where a
sequential scan or alternative index would be faster. A1 fixes the
constants; A6 verifies the planner behaves on real query shapes.

**Done when:** there is a packet matrix of planner-choice tests on at
least four query shapes against an `ec_ivf` index alongside an
`ec_hnsw` index on the same column, with EXPLAIN output and timing.

### A7. Add posting-scan early-stop with score-bound pruning

**Action:** during posting-list scoring, maintain a running top-`k ×
oversample` heap of the best scores seen so far. For each candidate
posting, compute a cheap lower bound on its possible score (using
`gamma` and a coarse summary of the payload — no full IP). If the lower
bound cannot enter the heap, skip the full IP score. This is the
standard FAISS-style pruning optimization for IVF posting scan.

**Why:** at the current operating point (`nlists=64, nprobe=48`), the
scan visits ≈75% of corpus postings per query. Score-volume reduction
is the structural latency lever the current arc has not exploited.
Items 1–4 from the prior list closed individual hot-path costs but did
not change how many postings are scored.

**Done when:** the scan path has an early-stop branch governed by the
top-`k` heap; a packet measures latency reduction on the existing
10k/25k DBPedia frontier; recall is verified to remain within tolerance
of the baseline top-`k` (the bound must be a true lower bound, not a
heuristic that can prune correct answers).

**Reference:** FAISS `IVF` `polysemous_ht` and `IndexIVFFastScan`
early-termination logic for the algorithm shape, but do not require the
exact polysemous-hash technique here — the running-top-`k` bound is the
mandatory part.

### A8. Wire PQ-FastScan and RaBitQ into ec_ivf alongside TurboQuant

**Action:** wire the existing PQ-FastScan and RaBitQ quantizer
implementations from `crate::quant` (see
`plan/tasks/15-pqfastscan-first-class.md` and
`plan/tasks/25-rabitq-quantizer.md`) into `ec_ivf` as additional
variants of `IvfQuantizerProfile`. After this work:

- `IvfQuantizerProfile` enum has three variants: `TurboQuant`,
  `PqFastScan`, `RaBitQ`. All three are real, supported variants.
- `IvfPreparedQuery` enum has matching `PqFastScan(...)` and
  `RaBitQ(...)` arms.
- `options.rs:63–64` no longer rejects `quantizer = 'pq_fastscan'` or
  `quantizer = 'rabitq'`; validation accepts both.
- `IvfQuantizer::encode_source`, `prepare_ip_query`, and
  `score_ip_from_parts` have `PqFastScan` and `RaBitQ` arms that call
  into the respective existing kernels.

**This work is purely additive.** TurboQuant remains a supported
variant. The existing TurboQuant code paths are not changed.
PQ-FastScan and RaBitQ as quantizers outside of `ec_ivf` (in
`crate::quant`) are not changed, removed, deprecated, or replaced. The
work is to make all three *selectable* on an `ec_ivf` index via the
`quantizer` reloption.

**Default selection:** keep the current default (`auto` → TurboQuant)
unchanged in this task. Whether to change the default after the
comparative measurement in A10 is a separate decision and out of scope
for A8.

**Done when:** for each of `quantizer = 'turboquant'`,
`quantizer = 'pq_fastscan'`, `quantizer = 'rabitq'`, an IVF index
builds, scans, inserts, and vacuums correctly; per-quantizer unit and
integration tests pass on PG17 and PG18; clippy is clean.

**Why this matters:** the dispatch seam from item 4 of the prior list is
currently exercised by exactly one variant. A seam with one variant is
a stub. Landing PQ-FastScan and RaBitQ as real second and third
variants validates the seam and prevents single-variant assumptions
from leaking into the dispatch interface.

### A10. Honest head-to-head quantizer assessment

**Action:** measure all three IVF quantizer variants from A8 against
each other on the same fixtures, with the same `nlists`, `nprobe`,
and `rerank_width` values. Report the comparison without a default-
favoring bias. Specifically:

- Build identical-shape `ec_ivf` indexes on the **same** corpus rows
  with three reloption settings: `quantizer = 'turboquant'`,
  `quantizer = 'pq_fastscan'`, `quantizer = 'rabitq'`. All other
  reloptions (`nlists`, `ef_construction`-equivalent, `rerank`,
  `rerank_width`, `training_sample_rows`) must be identical.
- For each variant, capture: build wall time, peak build memory, index
  size on disk, recall@10, recall@100, NDCG@10, p50/p95/p99 latency at
  matched recall points (sweep `nprobe` until each variant reaches
  recall@10 ≥ 0.99 and recall@10 ≥ 0.95, report both).
- Run on the existing 10k and 25k DBPedia slices for parity with the
  current frontier. Then extend to 100k once A9 lands.
- Capture cache state (cold and warm) for each measurement.
- Capture per-variant memory high-water mark during scan.

**Honest write-up requirements:**

- The packet must report the comparison **without preferring TurboQuant
  by default**. The user has flagged that TurboQuant may be worse than
  PQ-FastScan or RaBitQ on size and speed at comparable recall. The
  measurement is the source of truth, not the historical default.
- If TurboQuant loses on size, speed, or recall at a given recall
  target, the packet must say so plainly.
- If PQ-FastScan or RaBitQ wins on a Pareto axis (smaller index,
  faster scan, better recall at fixed compute), the packet must say
  so plainly.
- The packet must include a recommendation on which variant should be
  the default for `quantizer = 'auto'`. The recommendation must be
  grounded in the measured numbers, not in which variant is currently
  the default.
- Include a note on which variant best fits which workload (e.g.,
  "PQ-FastScan dominates at high recall on >512-d corpora;
  TurboQuant is preferable when …" — only if the data supports it).

**Done when:**

1. The comparison packet exists with the data above for 10k and 25k.
2. The packet is also extended to 100k once A9 lands (a follow-up
   packet is acceptable).
3. The packet's recommendation on the `auto` default is recorded. If
   the recommendation is to change the default away from TurboQuant,
   that change is opened as a separate task — A10 records the
   recommendation but does not itself change the default.
4. The recommendation does not handwave. If TurboQuant is kept as the
   default, the reason must be a property the measurement supports
   (e.g., "TurboQuant has lower variance across `nprobe` settings"),
   not "TurboQuant is the existing default."

**Sequencing:** A10 runs after A8 lands all three variants. A10 should
be re-run after A7 (early-stop pruning) lands, since the cost ratio
between variants may shift once score-volume reduction is in place.

### A9. Re-measure ec_ivf at 100k+ scale

**Action:** the current frontier is recorded only at 10k and 25k DBPedia
slices. Build a 100k slice and a 1M slice from the same DBPedia source,
and re-run the recall/latency sweep at the post-arc operating point
(currently `nlists=64, nprobe∈{32,48}, rerank_width=25` per packet
30070).

**Why:** IVF cost scales differently from HNSW with corpus size. The
10k/25k curve is positive but does not predict the 1M shape. Without a
100k+ measurement, "competitive substrate" cannot be claimed.

**Done when:** a packet records build time, index size, recall@10,
recall@100, p50/p95/p99 latency, cache state, and memory high-water
mark at 100k and 1M corpus sizes, alongside the same metrics on
`ec_hnsw` for comparison. Use the existing chunked corpus loader for
the 1M load.

**Sequencing:** A9 should be run *after* at least A1, A2, A4, and A7
land, so the measurement reflects the post-optimization substrate, not
the current snapshot.

## Sequencing of A1–A10 Before Merge

The 10 items above are independent in implementation but have natural
ordering for impact and risk. All 10 must land before merge.

1. **A4** (typed enum match) and **A5** (cache key audit) — small,
   localized, low-risk. Do these first to clear small debt before
   bigger work.
2. **A1** (cost-model audit) — required before A6. Honest cost
   constants are foundational.
3. **A6** (planner cross-test matrix) — runs immediately after A1.
4. **A7** (posting-scan early-stop with score-bound pruning) —
   biggest latency lever in the list.
5. **A8** (wire PQ-FastScan and RaBitQ as IVF quantizer variants) —
   substrate validation. Sequence after A7 so the early-stop
   optimization applies to all three quantizer variants when they are
   measured against each other.
6. **A10** (head-to-head quantizer assessment) — runs immediately
   after A8 lands all three variants. Includes the recommendation on
   which variant should back `quantizer = 'auto'`. Re-run once A7 is
   in place if A7 lands after A8.
7. **A2** (streaming vacuum) — can run in parallel with A1–A8;
   independent code surface.
8. **A3** (vacuum compaction) — sequence after A2.
9. **A9** (100k+ measurement) — runs after A1, A2, A4, A7, A8 land.
   A10 should also be re-run on the 100k+ slice as part of A9.

None of A1–A10 deprecate, remove, or replace existing functionality
unless the item explicitly calls that out. Default to additive
implementations.
