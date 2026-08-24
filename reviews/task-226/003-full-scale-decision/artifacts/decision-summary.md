# Task 226 BW8 full-scale decision summary

All rows use release extension
`a1f1584966011ca7c16175fe91f8efc302c8cf25` on three PG18 owners, one
immutable generation per scale, 200 held-out queries, top-k 10, L32, H100,
RaBitQ neighbor scoring, lazy-10 materialization, one build shard, and a fixed
4,096-entry persisted sharded head. Only beam width changes from BW4 to BW8.

| Scale | BW4 recall | BW8 recall | Paired delta (95% CI) | BW4 mean / p95 ms | BW8 mean / p95 ms | Gate |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| 10k | 0.9990 | 0.9990 | 0.000000 `[0.000000, 0.000000]` | 14.80 / 17.90 | 14.20 / 16.80 | ADVANCE (a) |
| 50k | 0.9540 | 0.9690 | +0.015000 `[+0.006500, +0.026000]` | 16.90 / 19.40 | 16.80 / 20.20 | ADVANCE (b) |
| 100k | 0.9285 | 0.9450 | +0.016500 `[+0.008000, +0.026500]` | 16.40 / 19.00 | 16.20 / 19.80 | ADVANCE (b) |

At 10k, all 200 paired queries tie on recall while mean improves 4.05% and p95
improves 6.15%. Storage is arm-invariant at 242,958,336 physical generation
bytes (76,095,488 graph-side and 166,862,848 owner row-tier bytes). Published
owner rows are 3,323 + 3,391 + 3,286 = 10,000, with zero non-owned rows and
zero orphans. Both variants share generation identity
`0200052037ced3d8eeefde4b8a72d9cea98e2f167faef441f09928b6328d7f44d639`.

The immutable 100k source is packet 002. It satisfies branch (b), has
arm-invariant 2,498,281,472-byte physical storage, and has conforming exact
100,000-row topology. See
`reviews/task-226/002-current-head-100k/artifacts/decision-summary.md`.

At 50k, paired recall improves on 16 queries and declines on 1, with 183 ties.
Mean latency improves 0.59%; p95 regresses 4.12%, inside branch (b)'s 5%
envelope. Storage is arm-invariant at 1,243,561,984 physical generation bytes
(410,214,400 graph-side and 833,347,584 owner row-tier bytes). Published
owner rows are 16,637 + 16,756 + 16,607 = 50,000, with zero non-owned rows
and zero orphans. Both variants share generation identity
`02009db2e9614fad3e1f49dd0db38b8126c11046dea65817768b44071d0ae800b983`.

The registered rule passes at 10k, 50k, and 100k, so the measured disposition
is `USEFUL NON-DEFAULT CONFIGURATION — EVIDENCE REVIEW-OPEN`. The candidate is recall-neutral with
better latency at 10k and improves recall within the registered mean/p95
envelope at 50k and 100k. It is not an unqualified Pareto win: p99 regresses
7.14% at 50k (19.60 to 21.00 ms) and 5.08% at 100k (19.70 to 20.70 ms).
Task 219's review-closed policy retains recall-equivalence for the shipped
interactive default, so this recall-changing point cannot become the default
through Task 226. An explicit product-policy ruling would be required to
reopen that decision; packet review here validates the evidence and supported
non-default disposition, including the tail tradeoff.
