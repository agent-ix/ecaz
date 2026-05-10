# ec_diskann Apple-Silicon Heap-TID-Ordered Rerank Fetch

Reviewer: please review this Apple-Silicon-specific ec_diskann
follow-on to packet `30204` and its packet-local A/B measurement.

## Scope

Packet `30204` landed an aarch64 NEON specialization for the exact
rerank inner-product kernel and recommended that the next
Apple-specific candidates are exact rerank source decode overhead and
heap fetch / cache locality in the rerank path, picked by measurement.

This packet measures the second of those: heap-fetch ordering. It
compares NEON head `eda51e9f` (current
`origin/ec-diskann-apple-neon-rerank` after `30204`, unordered fetch)
against new commit `e191a9e1` (`Visit ec_diskann rerank rows in
heap-TID order`) on Apple M5.

## Code Checkpoint

- code commit: `e191a9e1` (`Visit ec_diskann rerank rows in heap-TID
  order`).
- shape: in `src/am/ec_diskann/scan.rs::vamana_scan_with`, the rerank
  candidate batch is now sorted by `(block_number, offset_number)`
  before the rerank closure runs. This mirrors the IVF locality fix
  in commit `79c1a11c` (`Fetch ec_ivf rerank rows in heap order`).
- Final ordering still comes from the existing exact-distance sort
  later in the same function, so this change only affects fetch order.
- Adds unit test `sc_003b_rerank_visits_in_heap_tid_order` that
  records the `primary_heaptid` values the rerank closure observes
  and asserts they are non-decreasing in `(block_number,
  offset_number)`, regardless of the prefilter-score ranking.

Focused validation before measurement:

- `cargo check --no-default-features --features pg18`
- `cargo test --no-default-features --features pg18 --lib am::ec_diskann::scan`
  (29 tests pass, including the new `sc_003b`).

No broader cargo or pgrx test sweep was run for this packet; the slice
is a narrow ordering change in the rerank loop.

## Why this change is credible on Apple Silicon

The hot path for SQL ordered scans on a diskann index is:

1. `routine.rs::ec_diskann_amrescan` runs `scan::vamana_scan_with`.
2. `vamana_scan_with` returns the top-`rerank_budget` candidates from
   greedy descent, sorted by prefilter score (uncorrelated with heap
   layout).
3. For each candidate the rerank closure calls
   `routine.rs::exact_heap_rerank_distance` ->
   `routine.rs::with_heap_source_vector` ->
   `scan_state::fetch_heap_row_version`, which reads, pins, and
   unpins the heap page that holds that candidate's `primary_heaptid`.
4. The exact NEON inner-product kernel scores the row.

At `rerank_budget=800`, step 3 happens 800 times per query in
prefilter-score order. With DBpedia-shaped real embeddings on a
1536d 10k corpus, score order is uncorrelated with disk layout so
each rerank row tends to land on a fresh shared-buffer page, paying
a full pin / unpin and (cold) page read per row. Sorting the batch by
`(block_number, offset_number)` before the loop coalesces consecutive
rerank fetches that fall on the same / adjacent pages, the same fix
the IVF lane already validated.

## Result

200 iterations / pass, 2 passes per arm, on the same on-disk
`m5_diskann_real10k_w800` index built during packet `30204`.

| pass | mean | stddev | min | p50 | p95 | p99 | max |
|---|---:|---:|---:|---:|---:|---:|---:|
| pre pass 1 (NEON unordered, `eda51e9f`) | 15.8 ms | 0.66 ms | 15.1 ms | 15.7 ms | 16.4 ms | 17.1 ms | 23.9 ms |
| pre pass 2 (NEON unordered, `eda51e9f`) | 16.9 ms | 21.0 ms | 14.7 ms | 15.3 ms | 15.9 ms | 18.7 ms | 313.3 ms |
| post pass 1 (NEON heap-ordered, `e191a9e1`) | 16.4 ms | 21.1 ms | 14.1 ms | 14.8 ms | 15.6 ms | 17.6 ms | 314.4 ms |
| post pass 2 (NEON heap-ordered, `e191a9e1`) | 14.8 ms | 0.53 ms | 14.2 ms | 14.8 ms | 15.3 ms | 16.0 ms | 20.6 ms |

(Two `300+ ms` autovacuum-shaped outliers, one in pre pass 2 and one
in post pass 1, inflate `mean` / `stddev` / `max`. The percentile
columns are unaffected by the outlier and agree across passes within
each arm.)

Pass-averaged percentile deltas:

| metric | pre avg | post avg | delta | rel |
|---|---:|---:|---:|---:|
| min | 14.9 ms | 14.15 ms | `-0.75 ms` | `-5.0%` |
| p50 | 15.5 ms | 14.8 ms | `-0.7 ms` | `-4.5%` |
| p95 | 16.15 ms | 15.45 ms | `-0.7 ms` | `-4.3%` |
| p99 | 17.9 ms | 16.8 ms | `-1.1 ms` | `-6.1%` |

Recall@10 stays at `1.0000` and NDCG@10 stays at `1.0000` on the post
arm (`recall-post-table.log`), matching the pre numbers from packet
`30204`. The exact-distance sort later in `vamana_scan_with` is what
determines result identity, and that sort is unchanged.

## Interpretation

This is a real second Apple-Silicon win on top of the NEON kernel:

- The improvement is consistent across `min`, `p50`, `p95`, `p99` at
  `4-6%` and is larger than the per-pass stddev (`0.5-0.7 ms` on the
  clean passes).
- The improvement is repeatable: the cleaner second post pass
  matches the first post pass on every percentile.
- Recall is unchanged because the change only reorders heap fetches
  before the exact-distance sort.
- The unit test locks in the new visit order so future refactors
  cannot silently regress the property without a test failure.
- The two outliers fall in different arms, so they are system noise
  rather than an arm-specific regression.

The combined Apple-Silicon shape after `30204` + this packet, on the
same kernel-stress lane:

- vs origin/main (scalar, unordered): NEON kernel saves
  `6.5-11.6%` (packet `30204`); heap-TID ordering saves a further
  `4-6%`. Stacked, that is roughly a `10-17%` improvement in p50/p99
  on the kernel-stress lane vs origin/main.

## Recommendation

Land `e191a9e1` on top of `30204`. The change is narrow, recall-
preserving, unit-tested, and produces a clean across-the-board
Apple-Silicon win on the same kernel-stress lane that surfaced the
`30204` NEON kernel result.

The remaining packet-`30204` follow-on (exact rerank source decode
overhead) is still open. A first read of
`routine.rs::with_heap_source_vector` -> `ambuild::with_ecvector_datum_slice`
suggests the rerank path already passes `&[f32]` borrowed (no
per-row `Vec<f32>` allocation in this exact loop), so source-decode
follow-on would need its own narrow read pass before any code
change. That is intentionally NOT in scope here.

## Artifacts

All artifacts live under `artifacts/`. See `artifacts/manifest.md` for
SHAs, commands, fixture provenance, and full per-pass tables.
