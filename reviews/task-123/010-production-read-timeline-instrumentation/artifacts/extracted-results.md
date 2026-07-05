# Extracted Results

- Head SHA: `a91955a274fe4ec987f4067ed096f3700131d331`
- Task bucket: `reviews/task-123/010-production-read-timeline-instrumentation/`
- Lane: contained local multi-instance, one coordinator plus three worker PG18 instances on one host
- Fixture: correctness synthetic 10k corpus, `ec_spire_aws_synth_10k`
- Query sample: 4 queries, `top_k=10`, nprobe 8
- Storage format: `rabitq`
- Transport status: `pg_binary_attr_v1` ready

## Unit Tests

| Log | Result |
| --- | --- |
| `unit-tests/ecaz-cli-production-read-renderers.log` | 2 passed, 0 failed |
| `unit-tests/ecaz-cli-sql-contracts.log` | 1 passed, 0 failed |
| `unit-tests/ecaz-production-profile-rollup.log` | 1 passed, 0 failed |

## Contained Multi-Instance Smoke

`correctness-smoke/local-multinode-command.log` ends with:

```text
SPIRE local multinode fixture passed
HARNESS PASSED
```

The production-read suite ran through the real distributed executor:

| Step | nprobe | Queries | Recall@10 | Latency p50 | Latency p95 |
| --- | ---: | ---: | ---: | ---: | ---: |
| default | 8 | 4 | 0.1750 | 106.562 ms | 134.743 ms |
| rowcap25k | 8 | 4 | 0.1750 | 106.722 ms | 129.344 ms |

## Production-Read Profile

| Step | Result source | Selected pids | Remote pids | Dispatches | Remote heap candidates | Candidate p50/p95 | Heap p50/p95 | Total p50/p95 |
| --- | --- | ---: | ---: | ---: | ---: | --- | --- | --- |
| default | remote_heap_candidates | 32 | 32 | 12 | 120 | 22/22 ms | 26/27 ms | 72/78 ms |
| rowcap25k | remote_heap_candidates | 32 | 32 | 12 | 120 | 21/22 ms | 26/26 ms | 71/73 ms |

The aggregate profile's `payload_bytes_sum` is still `0` because that diagnostic
profile function does not request tuple payload columns. The new timeline table
does request the query projection and reports the per-worker payload bytes.

## Per-Worker Timeline

Default:

| Node | Phase | Candidate sum | Elapsed p50/p95 | Payload rows | Payload bytes |
| ---: | --- | ---: | --- | ---: | ---: |
| 2 | candidate_receive | 40 | 11/11 ms | 0 | 0 |
| 2 | heap_receive | 40 | 25/26 ms | 40 | 320 |
| 3 | candidate_receive | 40 | 10/10 ms | 0 | 0 |
| 3 | heap_receive | 40 | 24/24 ms | 40 | 320 |
| 4 | candidate_receive | 40 | 11/11 ms | 0 | 0 |
| 4 | heap_receive | 40 | 26/27 ms | 40 | 320 |

Rowcap25k:

| Node | Phase | Candidate sum | Elapsed p50/p95 | Payload rows | Payload bytes |
| ---: | --- | ---: | --- | ---: | ---: |
| 2 | candidate_receive | 40 | 11/11 ms | 0 | 0 |
| 2 | heap_receive | 40 | 24/25 ms | 40 | 320 |
| 3 | candidate_receive | 40 | 10/10 ms | 0 | 0 |
| 3 | heap_receive | 40 | 24/25 ms | 40 | 320 |
| 4 | candidate_receive | 40 | 12/12 ms | 0 | 0 |
| 4 | heap_receive | 40 | 26/26 ms | 40 | 320 |

## Read

The new instrumentation answers the missing per-worker object-byte part of the
review request for the contained local multi-instance path. Candidate receive
rows have no tuple payload, while heap receive rows report the projected payload
returned from each worker. In this smoke the projection is `id`, so each worker
returns 40 payload rows / 320 bytes over 4 queries.

This is instrumentation validation, not a replacement for packet 009's 100k
baseline. Packet 009 remains the cost/recall baseline for `n128 b4/tr50/f8` and
`n1024 b2/tr50/f8`.
