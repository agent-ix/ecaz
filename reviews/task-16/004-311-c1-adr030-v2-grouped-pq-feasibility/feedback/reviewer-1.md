## Feedback: ADR-030 v2 Grouped PQ Feasibility

This is the load-bearing measurement for the whole v2 lane. The 311 numbers
(`spearman_rho = 0.8859` at `group_size = 16`, `~15.5x` over exact on SRHT-transformed
1536-dim) justify continuing to invest in the composed pipeline instead of collapsing
back to the scalar path.

### What's solid

- The feasibility harness is in `src/bin/approx_score_study.rs` (`StudyMode::GroupedPqF32`
  and `GroupedPqU8`), which means the result is reproducible on new corpora without
  plumbing a new build lane. Keeping this accessible is important as the rerank codec is
  still unchosen.
- Using SRHT-transformed data for this study matches what the build path in packet 315+
  actually produces. Apples-to-apples.

### Caveats to keep in mind as the scorer lands

1. The `15.5x` is in-process, exact-over-approx arithmetic only. The runtime win will be
   smaller because (a) binary prefilter already removes most candidates, and (b) the
   inner loop is likely IR-bound on real workloads. Plan to re-measure end-to-end.
2. Spearman at 0.8859 is a ranking score, not a recall score. Whether approximate
   ranking errors get caught by rerank is a downstream question and no packet in the
   310-328 sequence answers it yet.
3. The current study uses u8-packed LUTs (`GroupedPqU8`). The real runtime will want
   vpshufb 4-bit LUT lookups (packet 313 tuple contract uses 4-bit packed nibbles).
   Confirm the spearman translates to the actual 4-bit representation before the scorer
   packet.

### Duplication to watch

Encode path is going to end up in three places if we're not careful: `approx_score_study`,
`build.rs` (`encode_grouped_pq`), and any insert path that follows. Collapse to one
shared encoder before the scorer lands; otherwise a subtle packing bug in one copy will
be very hard to see.
