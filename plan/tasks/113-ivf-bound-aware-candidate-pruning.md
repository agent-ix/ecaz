# Task 113: IVF Bound-Aware Candidate Pruning

Status: **proposed**.
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
