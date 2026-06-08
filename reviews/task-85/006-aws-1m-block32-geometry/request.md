# Task 85 Packet 006: AWS 1M Block32 Geometry

## Request

Evaluate larger SPIRE leaf block geometry against the retained Task 79/80
latency/recall point: improve latency while retaining the current recall level.

Packets 004 and 005 rejected block8 and per-leaf caps:

- block8 reduced candidate count but worsened read and scoring latency.
- per-leaf caps lost recall and doubled object read time at matched candidate
  count.

Packet 005 also established that global block allocation is doing real recall
work and implicitly skips leaf row reads. This packet keeps global allocation
and changes only the physical block geometry to test whether fewer summaries
and fewer row-segment reads can reduce latency without sacrificing recall.

## Suite

`reviews/task-85/006-aws-1m-block32-geometry/suite-aws-1m-block32-geometry-q500.json`

The suite is q500 on the AWS 1M profile. It builds a block32 index:

`aws_spire_1m_rabitq_t85_block32_tg256_idx`

and compares it with the retained block16 index:

`aws_spire_1m_rabitq_t80_block16_tg256_idx`

## Comparison Points

| row | intent |
|---|---|
| block16 global1152 first/repeat | same-suite retained control |
| block32 global384 | lower candidate budget, read-cost sensitivity |
| block32 global576 | matched candidate budget: `576 * 32 ~= 1152 * 16` |
| block32 global768 | recall recovery with fewer summaries than block16/global1152 |
| block32 global1152 | recall ceiling and cost ceiling |

The headline success criterion is stricter than earlier Tasks 80-84: beat the
retained block16/global1152 latency while preserving the same recall point.

## Results

Verdict: partial latency win, not a matched-candidate win.

| row | recall@10 | p50 ms | p95 ms | p99 ms | candidates |
|---|---:|---:|---:|---:|---:|
| block16 global1152 first | 0.9876 | 257.664 | 331.715 | 2353.880 | 9,213,846 |
| block32 global384 | 0.9636 | 149.910 | 199.878 | 218.125 | 6,137,953 |
| block32 global576 | 0.9730 | 178.624 | 235.037 | 250.139 | 9,206,722 |
| block32 global768 | 0.9800 | 199.480 | 259.216 | 275.563 | 12,275,644 |
| block32 global1152 | 0.9876 | 235.691 | 295.157 | 308.841 | 18,413,851 |
| block16 global1152 repeat | 0.9876 | 237.482 | 297.192 | 310.792 | 9,213,846 |

The matched-candidate block32/global576 row is not acceptable because recall
drops from 0.9876 to 0.9730. The same-recall block32/global1152 row is a small
latency improvement versus the same-suite warm retained control:

- p50: 237.482 ms -> 235.691 ms
- p95: 297.192 ms -> 295.157 ms
- p99: 310.792 ms -> 308.841 ms

That is only a ~1 ms p50 win, but it is the first Task 85 AWS 1M mechanism that
keeps the same recall point and does not regress latency.

Funnel medians explain the trade:

| row | p50 object read ms | p50 score ms |
|---|---:|---:|
| block16 global1152 repeat | 183.712 | 56.872 |
| block32 global576 | 123.267 | 33.717 |
| block32 global1152 | 167.198 | 44.204 |

Block32 reduces object-read and score time, but lower caps do not keep recall.
At same recall, the lower per-block overhead is mostly consumed by doubling row
coverage from 9.2M to 18.4M candidates.

AWS was paused after the run and confirmed paused.
