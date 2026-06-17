# Task 111a RaBitQ Bit-Width Sweep Summary

Head: `5cdddc04f`

This packet extends packet `004-all-dense-options-benchmark` from TQ and rb1 to
RaBitQ quant bits 2, 4, and 8. The suite covered both 50k and 100k fixtures and
all six dense layout surfaces:

- `row`: row posting tuples, no dense posting blocks.
- `dense-old`: original dense posting blocks, no coalescing, typed views off.
- `dense-a`: original dense posting blocks with dense coalescing on.
- `dense-typed`: original dense posting blocks, typed views on.
- `dense-b`: page-spanning packed dense posting groups.
- `dense-b-typed`: page-spanning packed dense posting groups with typed views on.

The run completed 180/180 suite steps with no failures, stale outputs, or
missing artifacts.

## nprobe=32 Recall

Recall is identical within each bit-width/scale across all storage surfaces.

| bits | scale | recall@10 |
| ---: | --- | ---: |
| 2 | 50k | 0.8840 |
| 2 | 100k | 0.8670 |
| 4 | 50k | 0.9410 |
| 4 | 100k | 0.9290 |
| 8 | 50k | 0.9460 |
| 8 | 100k | 0.9390 |

## nprobe=32 Latency Mean

| bits | scale | row | dense-old | dense-a | dense-typed | dense-b | dense-b-typed |
| ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 2 | 50k | 77.1 ms | 58.3 ms | 60.4 ms | 58.0 ms | 59.9 ms | 59.9 ms |
| 2 | 100k | 140.7 ms | 124.5 ms | 138.4 ms | 130.6 ms | 141.5 ms | 135.4 ms |
| 4 | 50k | 18.0 ms | 15.3 ms | 15.7 ms | 15.4 ms | 17.4 ms | 17.3 ms |
| 4 | 100k | 38.6 ms | 36.7 ms | 35.0 ms | 32.4 ms | 40.7 ms | 35.6 ms |
| 8 | 50k | 21.5 ms | 13.4 ms | 14.2 ms | 13.9 ms | 13.9 ms | 13.6 ms |
| 8 | 100k | 43.7 ms | 32.3 ms | 32.6 ms | 32.6 ms | 35.4 ms | 35.1 ms |

Interpretation:

- The simpler dense layouts are the best durable candidates for rb2/rb4/rb8.
  They preserve recall and usually beat row latency.
- Page-spanning packed dense works functionally, but is not the storage or
  latency winner in this shape. It often gives up the original dense storage
  win, and at 100k rb4/rb8 it is slower than original dense.
- Coalescing matters most when the physical layout fragments scoring batches.
  For rb2 in this run, original dense has many small physical flushes but still
  wins some latency rows because its index/page footprint is smaller enough to
  offset the batch fragmentation.

## EC IVF Index Size

| bits | scale | row | dense-old | dense-a | dense-typed | dense-b | dense-b-typed |
| ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 2 | 50k | 25.2 MiB | 21.3 MiB | 21.3 MiB | 21.3 MiB | 25.0 MiB | 25.0 MiB |
| 2 | 100k | 49.6 MiB | 41.8 MiB | 41.8 MiB | 41.8 MiB | 49.4 MiB | 49.4 MiB |
| 4 | 50k | 44.1 MiB | 39.8 MiB | 39.8 MiB | 39.8 MiB | 49.2 MiB | 49.2 MiB |
| 4 | 100k | 87.6 MiB | 78.9 MiB | 78.9 MiB | 78.9 MiB | 98.1 MiB | 98.1 MiB |
| 8 | 50k | 98.4 MiB | 78.9 MiB | 78.9 MiB | 78.9 MiB | 86.0 MiB | 86.0 MiB |
| 8 | 100k | 196.0 MiB | 157.0 MiB | 157.0 MiB | 157.0 MiB | 171.4 MiB | 171.4 MiB |

The current page-spanning format is storage-neutral for rb2, worse than row for
rb4, and better than row but worse than original dense for rb8.

## rb2 Batch Counters at nprobe=32

The available block-kernel counters were emitted for rb2. They show that row and
coalesced/page-spanning surfaces get wide batches, while original dense
fragments into many width 16-31 flushes.

| scale | variant | flushes | width >=32 | width 16-31 | width 8-15 | width <8 | kernel elapsed |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 50k | row | 9,147 | 9,134 | 9 | 3 | 1 | 6,563.5 ms |
| 50k | dense-old | 124,191 | 0 | 121,779 | 942 | 1,470 | 5,189.8 ms |
| 50k | dense-a | 10,699 | 10,389 | 45 | 88 | 177 | 5,318.1 ms |
| 50k | dense-b | 74,498 | 71,298 | 1,571 | 712 | 917 | 5,225.0 ms |
| 100k | row | 20,380 | 20,363 | 7 | 5 | 5 | 12,237.0 ms |
| 100k | dense-old | 275,136 | 0 | 272,838 | 1,348 | 950 | 11,319.5 ms |
| 100k | dense-a | 21,778 | 21,443 | 140 | 142 | 53 | 12,266.3 ms |
| 100k | dense-b | 164,473 | 161,307 | 1,262 | 875 | 1,029 | 12,287.8 ms |

This answers the earlier batch-size question: both row and coalesced dense get
wide RaBitQ batches. Original dense does not, but the rb2 kernel is still fast
enough, and original dense reads enough fewer pages, that it can beat row and
some coalesced/page-spanning variants.

## Page-Spanning Packed Counters

Selected `EXPLAIN (ANALYZE, FORMAT JSON)` logs show the page-spanning packed
path assembling logical groups from multiple physical segment tuples.

| bits | variant | posting pages read | dense blocks visited | groups assembled | segments assembled | payload bytes copied | groups borrowed | payload bytes borrowed | scan elapsed |
| ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 2 | dense-b | 2,641 | 2,650 | 1,314 | 2,628 | 16,622,496 | 22 | 77,220 | 113,305 us |
| 2 | dense-b-typed | 2,641 | 2,650 | 1,314 | 2,628 | 16,622,496 | 22 | 77,220 | 111,833 us |
| 4 | dense-b | 5,266 | 5,277 | 1,323 | 5,264 | 32,848,140 | 13 | 45,240 | 43,027 us |
| 4 | dense-b-typed | 5,266 | 5,277 | 1,323 | 5,264 | 32,848,140 | 13 | 45,240 | 47,131 us |
| 8 | dense-b | 9,223 | 9,231 | 1,327 | 9,222 | 65,243,556 | 9 | 37,152 | 50,003 us |
| 8 | dense-b-typed | 9,223 | 9,231 | 1,327 | 9,222 | 65,243,556 | 9 | 37,152 | 49,155 us |

That validates option 3 behavior and also explains why the current physical
fragment format is not yet the best durable format: it multiplies page reads and
payload copies. The reviewer-suggested shape, with metadata once per logical
group and payload-only continuation segments, is the right next storage design
direction if we keep page-spanning dense.
