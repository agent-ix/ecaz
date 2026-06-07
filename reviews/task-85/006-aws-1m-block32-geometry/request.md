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

## Status

Prepared for AWS execution. No benchmark results yet.
