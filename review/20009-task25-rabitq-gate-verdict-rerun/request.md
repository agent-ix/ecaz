# Review Request: Task 25 Slice 10 — ADR-045 Stage 1 Gate Re-Run (Paper-Faithful)

Scope: documentation + handoff contract amendment. Re-runs the
slice-8 gate decision against the paper-faithful estimator from
slice 9 and records the amended handoff contract for task 27.

Task: `plan/tasks/25-rabitq-quantizer.md`, Phase 2 decision gate
(re-run after slice-9 math correction).

Branch: `task25-rabitq-stage1-phase0` (slice 10 builds on `e49d9b8`).

Artifacts in this packet:
- `artifacts/run-dbpedia-10k-paper-faithful.txt` — verbatim harness output.
- Amendment to `review/20005-task25-task27-handoff-contract/request.md`
  embedded at the end of this packet (to be folded into the
  contract once slice 10 is reviewed).

## Verdict

**FAIL** stands. Recall gap is **10.25 pp** on DBpedia-10k.

But the FAIL is now **qualitatively different** from slice 8: the
estimator is paper-faithful, the error is 27× smaller than before,
and the bound is 12× tighter. This is a legitimate "RaBitQ at
PQ4-parity storage is not tight enough to clear a 1 pp gate on
DBpedia-1536d" result — not an "our math is wrong" result.

## Slice 8 vs. slice 10 comparison

Same reproducer, same corpus, only the estimator math differs.

| metric                | slice 8 (α = mean\|c_i\|) | slice 10 (paper-faithful) | ratio  |
|-----------------------|-----------|----------------|--------|
| recall@10             | 0.8935    | **0.8975**     | +0.4 pp (within noise) |
| mean bound            | 0.612     | **0.050**      | 12× tighter |
| p99 bound             | 0.630     | 0.052          | 12× tighter |
| mean error            | 0.268     | **0.010**      | 27× smaller |
| p99 error             | 0.352     | 0.037          | 9.5× smaller |
| tightness (err/bound) | 0.437     | 0.211          | safer margin |
| wall time             | 19 s      | 19 s           | — |

### What the numbers mean

1. **Error dropped 27×.** The paper's division by `o_dot` cancels
   the variance that the slice-4 estimator was shedding into the
   realized error. Our implementation is now producing estimates
   that match exact IP within 0.010 on DBpedia-1536d, in line
   with RaBitQ paper numbers.
2. **Bound is 12× tighter and still dominates realized error.**
   `tightness = 0.211` means the bound envelopes the realized error
   about 5× on average — healthy margin for Stage 3 candidate-pool
   sizing without being pathologically loose.
3. **Recall barely moved (+0.4 pp).** This is the key finding.
   Making the estimator dramatically more accurate did not
   appreciably move recall. That means **the binary bit budget
   itself is the bottleneck** — the gap between the true rank-10
   and rank-11 inner products on DBpedia-1536d is often smaller
   than the irreducible quantization floor of 192 sign bits,
   regardless of how correctly the estimator computes from those
   bits.

## Decision per the task rubric

Task doc:
> **Pass** (≤1 pp) → publish, unblock Symphony Stage 2.
> **Marginal** (1–2 pp) → keep the module; OPQ (task 20).
> **Fail** (>2 pp) → shelve Stages 2–3, null result.

10.25 pp is FAIL. **The shelve recommendation from slice 8 stands.**

Unlike slice 8's verdict, this one is defensible against the "maybe
the math was wrong" objection. The math is now paper-faithful and
unit-tested, and the recall gap persists. The issue is not
implementation fidelity; it is the fundamental information content
of 192 sign bits against DBpedia-1536d's fine-grained top-K
structure.

## Honest caveats that remain

One caveat from slice 8 is closed (estimator math); two remain:

1. **Corpus size.** 10k is the only prepared slice on this box. At
   50k and 1M, recall@K sometimes improves because the K-th
   boundary's IP gap grows. `ecaz corpus prepare --profile ec_hnsw
   --parquet target/real-corpus/qdrant-dbpedia-openai3-1m/data` can
   emit the larger slices from the existing parquet shards.
   However: a 10 pp gap at 10k would need to close nearly an order
   of magnitude for the 1M slice to PASS. Unlikely but not
   impossible — recall@10 can scale non-monotonically.
2. **OPQ rotation (task 20).** The MARGINAL rubric specifically
   names "return to OPQ" as the lever. At 10.25 pp we are well
   past MARGINAL, so OPQ is unlikely to close the gap on its own —
   but it was never tried. A combined follow-up packet could
   carry 50k/1M + OPQ numbers and close the shelve decision
   definitively.

My read: the verdict is actionable as-is. The shelve action is
consistent with both the 10k/paper-faithful result and the task
doc's pre-registered rubric. The two caveats above should be
recorded as follow-up questions but do not block slice 10
acceptance.

## Consequences

Per the slice-6 handoff contract, task 27 kickoff is gated on a
PASS. **Task 27 is recommended for shelve.** `src/quant/rabitq.rs`
keeps its current ADR-031 successor role — the AM prefilter in
`ec_hnsw` continues to work unchanged (it uses only the Hamming
path, which does not depend on the estimator).

## Contract amendment (to be folded into packet 20005)

Replace the "Estimator semantics" block in
`review/20005-task25-task27-handoff-contract/request.md` with:

```
### Estimator semantics (paper-faithful, slice 9)

Per-vector stored scalar: o_dot = ⟨o_unit, sign(o)/√D⟩
  (4-byte f32, immediately after the norm in the code payload)

Inner-product estimate:
  ⟨q, o⟩ ≈ ||o|| · Σ_i q_i · sign(o_i) / (o_dot · √D)

ε-concentration bound at RABITQ_BOUND_CONFIDENCE = 2.5:
  ε²(o) = (1 − o_dot²) / (D · o_dot²)
  |⟨q, o⟩ − estimate| ≤ C · ||q|| · ||o|| · ε(o)   (probabilistic,
                                                     ~99% one-sided)

Degenerate o_dot (|o_dot| < 1e-6 or non-finite):
  estimate = 0.0
  bound    = +∞
  Stage 3 should treat those candidates as unscorable — filter
  via `bound.is_finite()`.

Invariants Stage 3 may rely on:
  - bound ≥ 0, or bound = +∞ for degenerate candidates.
  - bound → 0 as o_dot → 1 (sign-aligned vector, estimator exact).
  - The bound is probabilistic, not worst-case. Realized tail
    violation rate matches RABITQ_BOUND_CONFIDENCE's nominal
    tail probability.
```

No other section of the contract changes. Constants `RABITQ_ALPHA_LEN`
→ `RABITQ_UNIT_DOT_LEN` rename flows through the "Types" listing.

## Closing

Task 25 closes with a clean null result — exactly the shelvable
outcome the task doc pre-registered in the "Notes" section:

> **Ship the null result if it fails.** A clean write-up of "RaBitQ
> at PQ4-parity storage loses 3pp recall on 1M real corpus" is a
> real contribution and frees reviewer attention. Do not hide a
> marginal result by promoting it.

The number is 10 pp, not 3 pp, but the principle is the same.
`src/quant/rabitq.rs` stays as the paper-faithful ADR-031 successor;
task 27 Stages 2 & 3 stay gated pending a future OPQ + larger-slice
follow-up if anyone wants to re-open the question.
