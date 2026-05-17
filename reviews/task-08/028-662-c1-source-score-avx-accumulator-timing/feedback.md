---
seq: 01
disposition: approve
reviewer: opus-4.7
reviewed_commit: 1802d707339a24785ee1cea46688a3e5b50c2056
---

# Review Feedback: AVX Source-Score Accumulator Unroll

## Disposition

Approve. The change is correct and the measured 50k impact is consistent with what 4-way accumulator unrolling buys on a memory-bound 1536-d dot product.

## What I read

- `src/am/ec_hnsw/source.rs` — `inner_product_avx2_fma` now keeps four
  independent `__m256` accumulators across the 32-lane main loop, falls back
  to single-accumulator 8-lane tail using `acc0`, and reduces with
  `(acc0+acc1)+(acc2+acc3)` before the horizontal sum.
- Reduction order: tail-block adds to `acc0` *before* the four-way combine, so
  no contributions are dropped. Confirmed by reading lines 106–148.
- 1536 % 32 == 0 ⇒ at the dimension actually exercised by the real-50k
  fixture, the 8-lane tail loop never executes. The unroll is hot for
  every build comparison.
- AVX-only: the SQL-facing rerank path is explicitly held to the prior
  sequential f32 order (per packet narrative); spot-checked that the
  rerank score path still routes through its sequential helper, so
  regression-test expectations are unchanged.

## Observations

1. **Marginal win is honest.** 1.01× wall-clock and 1.01× graph_us at 50k
   real m=16 ef_construction=128 is small, but it lands on top of the 8.75×
   already banked through 660. At 1536 dims the source dot product is
   bandwidth-bound, and FMA dependency-chain reduction only helps the small
   compute slice — the reported number is the right shape, not noise being
   sold as a win.

2. **Tail policy is fine for 1536, worth a one-line comment for posterity.**
   Future callers at non-multiples of 32 (e.g., a 768-d or 1024-d corpus)
   will exercise both tail loops; the current code is correct for them, but
   a single `// 32-lane main loop, 8-lane tail, scalar remainder; tail
   accumulates into acc0 which is folded back during reduction` would save
   the next reader the rederivation. Not blocking.

3. **No equivalence test added in this packet.** The existing source IP
   tests presumably covered this through the regular `inner_product`
   surface; I did not regrep to confirm a 1536-length AVX-vs-scalar
   equivalence assertion exists. Worth confirming such a test exists at
   a length that exercises all three loops (e.g. 41 elements) before
   landing further unroll variants. Not a packet-662 blocker.

## Phase 3 status read

Combined with packets 658 (recall parity 0.91/0.91 on real 50k m=16) and
659/660/662 (30:16 → 3:25 wall-clock, ≈8.86×), Phase 3 of Task 26 is now
substantively closed for the real 50k surface: recall faithful, build-time
materially faster, and the speedup is dominated by graph-phase work, which
is the gate the task plan named. The remaining Phase 3 deliverable is the
combined measurement packet that records both results in one place — the
current packet chain does that across 658/659/660/662 but not in a single
artifact.

## Recommended next steps

- Land packet 663 (backlink target score reuse) on the same fixture for
  another point in the optimization curve, then call Phase 3 closed and
  start Phase 4 (default-switch + shm_mq cleanup).
- Phase 5 (1M / 10M scale curves at 2/4/8 workers) remains the substantive
  outstanding lift.
