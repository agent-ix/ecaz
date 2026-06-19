# Task 113: IVF Bound-Aware Candidate Pruning

Status: **merged to `main` 2026-06-19** (recall-safe code complete; latency
promotion bench deferred to the Intel bench host). Reviewed in
`reviews/task-113/001-bound-contract-audit/` (Phase 1) and
`reviews/task-113/002-rabitq-lazy-bound/` (Phases 2/3/4/5).
- **Sound bound surface confirmed:** the Cauchy-Schwarz prune cutoff
  (`||o||·||q||/o_dot`) is a deterministic upper bound (recall-safe); the
  ε-concentration envelope is probabilistic and correctly **not** used for hard
  pruning (Non-Goal honored).
- **Posting-prune (Phases 2/3) landed + made recall-safe-provable:** the sound
  cutoff was already threaded into row / SoA-scalar / dense-block direct-scan
  paths; this task added the `ec_ivf.posting_bound_prune` A/B GUC (default on =
  pre-existing behavior), the `postings_pruned_by_bound` counter, and a
  **pruned==unpruned byte-identical** pg18 proof. Batch kernels deliberately do
  not pre-prune (preserve the 111 contiguous SIMD pass) — evidence-backed
  per acceptance criterion 3.
- **Phase 4 (lazy-rerank bound for Task 112):** `RaBitQLazyBound` seam is live
  with the correct **exact-score** residual bound `|⟨q,o−x_dec⟩| ≤ ||q||·||o−x_dec||`
  (NOT the estimate-space cutoff), plus the two Task-112 seam fixes (monotonicity
  precondition; `worst_kept.is_finite()` stop gate). Default slack `+inf` →
  byte-identical to `NoBound`.
- **Outstanding / deferred:** (a) the Phase-5 promotion benches (posting-prune
  A/B and joint 112+113 lazy A/B) are env-blocked here and must run on the Intel
  bench desktop; (b) the lazy material skip is a **conditional** win — the
  residual bound is loose at 1-bit (rare skips), tighter at higher RaBitQ
  bit-depth, and needs a k-cap / on-demand-suffix emission to fire; pursue only
  if the deferred lazy A/B shows it fires. See the 002 reviewer feedback.
Priority: P1 latency.

## Goal

Reduce IVF scan latency by using quantizer-provided score bounds to avoid
scoring, retaining, or reranking candidates that cannot beat the current
frontier.

RaBitQ is the first target because it already carries a bound-capable scoring
surface. Other quantizers are out of scope unless they can provide a sound
bound with equivalent correctness guarantees.

## Why

IVF scan latency is driven by the number of postings scored and the number of
candidates carried into dedup and exact rerank. When a bounded top-k frontier
already has a strong enough candidate, a later candidate with a provable upper
bound below the current threshold can be skipped without recall loss.

The repository already has RaBitQ scoring APIs that can use a
`min_ip_to_keep` threshold. This task makes that pruning systematic, measured,
and safe across the relevant IVF scan paths.

## Scope

- IVF only.
- RaBitQ first.
- Recall-safe pruning only.
- Current row-shaped posting path.
- Dense posting block path if Task 111 has landed or is developed in parallel.
- Counters for pruned-by-bound, scored, retained, deduplicated, and reranked.

## Non-Goals

- Do not add heuristic pruning that can drop recall.
- Do not change centroid selection or nprobe.
- Do not implement residual quantization.
- Do not change heap-f32 exact rerank except to expose the current frontier
  threshold needed for safe pruning.
- Do not force TurboQuant into this task without a sound bound contract.

## Phases

### Phase 1 - Bound API Audit

- Audit current RaBitQ candidate scoring APIs.
- Document which APIs produce exact approximate scores, conservative upper
  bounds, or both.
- Identify row-path and batch-path call sites that do not currently pass a
  useful frontier threshold.
- Add tests for bound monotonicity and cutoff behavior where missing.

Stop condition: if the available bound is too loose or not sound for the IVF
candidate frontier, close with evidence and no scan behavior change.

### Phase 2 - Row Posting Integration

- Thread the current bounded frontier threshold into row posting scoring.
- Prune candidates before full candidate retention when the bound proves they
  cannot survive.
- Count pruned candidates separately from scored and retained candidates.
- Preserve byte-equivalent recall on deterministic fixtures.

### Phase 3 - Batch and Dense-Block Integration

- Extend pruning to batch scoring where practical.
- If dense blocks exist, support block-level or lane-level pruning without
  degrading the direct-scan advantage.
- Record whether pruning happened before full score, after full score, or at
  retention.

### Phase 4 - Rerank Frontier Integration

- Share the strongest safe frontier threshold with heap-f32 rerank when useful.
- Coordinate with Task 112 so lazy exact rerank can use the same bound contract
  if applicable.

### Phase 5 - Benchmark Gate

Run matched before/after benchmarks with counters:

- postings visited,
- postings scored,
- candidates pruned by bound,
- candidates retained,
- heap rerank rows,
- latency and recall.

Promotion criteria:

- Recall is unchanged.
- Candidate scoring or rerank work drops materially.
- Latency improves on the target high-recall cells.
- No p95/p99 regression from extra bound bookkeeping.

## Acceptance Criteria

1. The RaBitQ bound contract is documented in code or task packet context.
2. Row posting path uses safe bound pruning.
3. Batch/dense-block path either uses safe bound pruning or has an explicit
   evidence-backed reason not to.
4. Scan counters expose bound pruning impact.
5. Bench evidence shows whether the change should be promoted, iterated, or
   abandoned.

## Evidence Requirements

Every benchmark packet must include:

- suite config,
- reloptions and quantizer bits,
- recall@10 and NDCG@10,
- p50/p95/p99 and mean,
- postings visited,
- postings scored,
- pruned-by-bound count,
- retained candidate count,
- heap rerank rows if applicable.

## Dependencies and Coordination

- Can follow Task 111 or run independently on the row posting path.
- Coordinates with Task 112 for lazy exact rerank thresholds.
- Coordinates with Task 115 if residual quantization changes the bound shape.
