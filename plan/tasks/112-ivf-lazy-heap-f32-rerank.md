# Task 112: IVF Lazy Heap-F32 Rerank

Status: **infrastructure merged to `main` 2026-06-19; latency win is conditional
and deferred (not realized).** Phases 1–4 (lazy-rerank driver + correctness
contract + `ec_ivf.lazy_heap_rerank` gate + considered/skipped counters) landed
and were reviewed in `reviews/task-112/001-lazy-rerank-contract-instrumentation/`;
Task 113 then supplied the calibrated bound seam (`RaBitQLazyBound`) and carried
both 112 seam fixes (monotonicity precondition; `worst_kept.is_finite()` stop
gate) — reviewed in `reviews/task-113/002-rabitq-lazy-bound/`.
**Outcome of the joint 112/113 investigation:** the lazy path is recall-safe and
byte-identical today, and a *recall-safe material skip is conditional, not a
pending certainty*. The sound exact-score bound is the quantization residual
`|⟨q,o−x_dec⟩| ≤ ||q||·||o−x_dec||`, which is **loose at 1-bit RaBitQ** (skips
rarely fire) and tighter at higher bit-depth; the only *tight* per-candidate
envelope is probabilistic (recall-unsafe, rejected). Realizing a skip also needs
a **k-cap / on-demand cross-`amgettuple` suffix emission** (the AM has no `k`
pushdown, so `min_kept == rerank_width` and the stop is never reached today).
**Remaining (deferred, do not pursue blind):** run the joint
`task-113-lazy-rerank-ab.intel-local.json` A/B on the Intel bench host; only if
it shows the residual bound fires at the target bit-depth/recall cells is the
k-cap + per-candidate residual carriage worth building. Acceptance criterion 4
(bench evidence) + the heap-fetch-reduction goal remain open and bench-gated.
Priority: P0 latency.

## Goal

Reduce `ec_ivf` query latency by fetching and exact-scoring fewer heap rows in
the `rerank = 'heap_f32'` path while preserving final result correctness.

This task targets the exact rerank stage only. It does not change candidate
generation, posting layout, quantizer math, or index storage.

## Why

The current heap-f32 rerank path takes the approximate candidate frontier,
sorts/fetches the chosen rerank slice, exact-scores those heap vectors, and
then sorts by exact score. This is correct and simple, but it can fetch and
detoast heap rows that cannot affect the final top-k once better exact scores
are known.

A lazy rerank strategy should exact-score candidates only while they can still
change the returned result set.

## Scope

- IVF only.
- `rerank = 'heap_f32'` only.
- Works with current row-shaped postings and with dense posting blocks if Task
  111 lands first.
- Adds counters for heap rows fetched, heap blocks fetched, exact rerank time,
  candidates skipped by lazy stop, and final rerank width.
- Preserves existing fixed-width behavior behind a fallback or diagnostic
  switch if useful.

## Non-Goals

- Do not store full vectors in the index.
- Do not change approximate posting scoring.
- Do not change `rerank_width` semantics unless the new behavior is explicitly
  gated.
- Do not add recall-risky heuristic early stops.

## Phases

### Phase 1 - Baseline Instrumentation

- Confirm counters for approximate scan time, exact rerank time, heap blocks
  fetched, heap rows fetched, and rerank rows.
- Add missing counters or diagnostics needed to attribute current rerank cost.
- Run a baseline over the current high-recall IVF cells.

Stop condition: if heap rerank is not a meaningful share of latency after the
current scan layout, close with evidence and no behavior change.

### Phase 2 - Correctness Contract

- Define the safe stop condition for lazy exact rerank.
- Document what score or bound information is required from the approximate
  frontier.
- Prove that skipped candidates cannot affect final top-k under the chosen
  contract.
- If the available approximate scores are insufficient for safe stopping,
  close or move the missing bound work to Task 113.

### Phase 3 - Lazy Rerank Implementation

- Replace or augment fixed-slice rerank with a lazy frontier.
- Exact-score candidates in an order that supports early stop and heap-block
  locality.
- Keep final results sorted by exact score.
- Preserve duplicate handling and snapshot semantics.

### Phase 4 - Diagnostics and Fallback

- Add debug counters for lazy stop decisions.
- Make EXPLAIN or packet-local diagnostics show how many candidates were
  considered, exact-reranked, skipped, and returned.
- Keep a deterministic way to compare against fixed-width rerank.

### Phase 5 - Benchmark Gate

Run fixed-width versus lazy rerank on the same index and query set.

Promotion criteria:

- Recall is unchanged.
- Returned top-k ordering matches exact-rerank expectations.
- Heap rows or heap blocks fetched decrease materially.
- p50 or p95 improves at high-recall cells without unacceptable tail
  regression.

## Acceptance Criteria

1. Lazy heap-f32 rerank is implemented behind a gate or as an explicitly
   justified replacement.
2. Focused tests cover early stop, ties, duplicate heap TIDs, empty frontier,
   and `rerank_width` boundaries.
3. Counters expose rerank rows, heap blocks, skipped candidates, and exact
   rerank elapsed time.
4. Benchmark evidence compares fixed-width and lazy behavior at matched recall.
5. The final packet recommends promote, iterate, or abandon.

## Evidence Requirements

Benchmark packets must include:

- suite config,
- reloptions,
- query count,
- recall@10 and NDCG@10,
- p50/p95/p99 and mean,
- heap rows fetched,
- heap blocks fetched,
- exact rerank elapsed time,
- approximate scan elapsed time,
- skipped candidate count.

## Dependencies and Coordination

- Can run before or after Task 111.
- Coordinates with Task 113 if lazy stopping requires stronger approximate
  bounds.
- Must not rely on page-layout changes for correctness.
