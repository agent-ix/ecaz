# Task 145 packet 001 — dense-pool dedup + lazy-heap topk_collect (review request)

Status: **measured — awaiting review**. Coder: Codex. 2026-07-03.
Branch: `task-145-topk-collect` (off main `689528007`).
Code commits under review: `0679ac536` (lever 1), `22411e3dd` (lever 2).

## Summary

Task 145 targets the `topk_collect` stage — 17% of the 1m approximate
scan under the new dense+int8 defaults (Task 143 stage budget). Two
levers, one commit each, A/B'd separately per the closeout rules:

1. **Lever 1 — `CandidateDedupPool`** (`0679ac536`): the scan dedup map
   `HashMap<ItemPointer, EcIvfScoredCandidate>` becomes a
   heap_tid→u32-slot map plus a dense candidate vec; the collect walks
   the contiguous vec instead of iterating the hash map. Dedup
   semantics, explain counters, and the Task 113 running-top prune hint
   unchanged (single `Entry` site rewritten 1:1).
2. **Lever 2 — `RankedProbeCandidates::LazyHeap`** (`22411e3dd`): the
   profile revealed the pre-change stage cost is dominated by the FULL
   SORT of every deduped candidate (~45k at 1m/n32) in the unbounded
   collect path — the shipping default has no pre-rerank limit (rerank
   mode is not heap_f32; `exact_rerank` samples = 0 in every measured
   cell), and the executor pulls k=10 rows. The unbounded path now
   returns a min-heap popped lazily by `next_posting_candidate`: O(n)
   heapify + O(log n) per pulled row, and the palloc copy of the full
   candidate array is gone. The bounded pre-rerank (heap_f32) path
   keeps its sorted vec unchanged.

## Byte-identity (the task's hard gate)

- `candidate_cmp` is a strict total order over distinct heap tids, so
  bounded top-k selection is iteration-order independent and lazy-heap
  pop order equals the former full-sort order.
- Unit tests: pool-vs-reference-map equality on a duplicate-heavy
  10k-candidate stream at limits None/1/50/20k, plus strict-ascending
  pop-order assertion (`artifacts/unit-dedup-pool*.log`).
- Measured: **all 24 recall cells (10k/50k/100k/1m × nprobe 8–64)
  byte-identical across all three binaries**, against shared ground
  truth. Zero mismatches. Storage identical at 100k/1m (query-side-only
  change; the tiny 10k/50k heap drift is autovacuum settling on the
  fresh tables, stable across the code swap — see manifest).

## Headline numbers (before → after, baseline → levers 1+2)

Latency mean (warm, k=10, nprobe 32 / 40):

| scale | before | after | delta |
|---|---|---|---|
| 10k  | 0.63 / 0.72 ms | 0.58 / 0.64 ms | −7.9% / −11.1% |
| 50k  | 1.14 / 1.34 ms | 1.15 / 1.34 ms | noise (stage −72%, see manifest) |
| 100k | 1.81 / 2.12 ms | 1.63 / 1.83 ms | **−9.9% / −13.7%** |
| 1m   | 7.37 / 8.50 ms | 6.76 / 7.81 ms | **−8.3% / −8.1%** |

topk_collect stage (per-sweep, nprobe 32): 10k −77%, 50k −72%,
100k −73%, **1m 17.13 → 4.17 ms (−76%)** — from ~17% of the 1m
approximate scan to ~4.5%. Recall unchanged digit-for-digit.

Per-lever attribution: lever 1 alone moved the stage −4..−7% (real but
small — the microbench in `artifacts/profile-map-vs-pool-collect.log`
profiled the bounded shape; the unbounded path's sort dominates in
production). Lever 2 is the decisive win. Both levers are kept: lever 1
also shrinks the dedup map entry (24-byte value → u32 slot) and feeds
lever 2's heapify from a contiguous vec.

## Evidence

- `artifacts/manifest.md` — cells, shas, install logs, full tables.
- `artifacts/{baseline,pool,lazyheap}/` — suite runs (17/17 succeeded
  each): `results.jsonl`, recall/latency/storage logs with
  `ivf_stage_counters` + task87 counters, in-suite sha prechecks.
- Suite config: `task145-dedup-pool-ab-suite.json` (bespoke-config
  reason stated in manifest: before/after-binary A/B on fixed tables;
  registered ec_ivf default recall grid kept verbatim; latency at
  [32,40] for Task 143 comparability).
- Tests/validation: unit tests + clippy pg18 clean; pgrx runtime tests
  deferred per the known macOS `_BufferBlocks` dyld blocker (compile
  gates + the e2e suite A/B above stand in).

## Review asks

1. Lever-2 design: `RankedProbeCandidates` enum + lazy-heap consumption
   in `next_posting_candidate` — confirm no consumer of the former
   fully-sorted array semantics was missed (rerank is gated to the
   Sorted arm; `rerank_probe_candidates` is a no-op for non-heap_f32
   modes).
2. The `u32` slot bound in `CandidateDedupPool::record_best`
   (fail-closed `expect` at >4.29e9 candidates/scan) — acceptable?
3. Close criteria: the task's gate is "measurable topk_collect
   reduction at unchanged recall" — met at all four scales. Graviton
   remains the standing cross-lane follow-up (same as Tasks 136/141/143).
