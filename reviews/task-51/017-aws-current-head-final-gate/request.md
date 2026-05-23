# Review Request: AWS Current-HEAD IVF/RaBitQ Final Gate

Please review the Task 51 AWS current-head measurement packet:

- benchmark packet: `benchmarks/task51-aws-ivf-rabitq-current-head-final-gate/`
- checked-in suite config: `benchmarks/task51-aws-ivf-rabitq-current-head-final-gate/suite.json`
- review artifacts: `reviews/task-51/017-aws-current-head-final-gate/artifacts/`

## Scope

This is the AWS final-gate run for the current IVF/RaBitQ branch head. It uses
the preserved AWS DB/index snapshot, exercises the current binary on the DB
host, runs q=500 recall, q=200 latency, EXPLAIN counters, and the sidecar
real-I/O harness.

No vchord or pgvectorscale steps were run.

## Host / Snapshot / Index Verification

```text
AWS host head SHA: 902e8e066944d4cabfb26ee5cc9039b466856891
restored snapshot: snap-0e0632400184fadd4
db instance: m8g.2xlarge
loader instance: c8g.medium
database: tqvector_bench
server_version: 18.3
corpus_rows: 990000
query_rows: 10000
index: real_1m_ivf_rabitq1_rerank_rabitq_idx
reloptions: {quant_bits=1,rerank=heap_f32,rerank_width=50,storage_format=rabitq}
```

## Suite Status

```text
command_id: 70df8076-1c85-4481-b1c9-a3e8bdbd7f88
status: Success
response_code: 0
elapsed: PT31M14.927S
```

The stack is paused after the run:

```text
state: paused
cost: ~$0.00/hr running, ~$4.00/mo retained storage
```

## Results Available Locally

Recall, q=500:

| nprobe | recall@10 | CI95 | NDCG@10 |
| ---: | ---: | --- | ---: |
| 256 | 0.9936 | 0.9910-0.9955 | 0.9998 |

Latency, q=200, concurrency=1:

| nprobe | p50 | p95 | p99 | max |
| ---: | ---: | ---: | ---: | ---: |
| 256 | 69.1 ms | 75.7 ms | 80.2 ms | 109.5 ms |

EXPLAIN counters:

```text
index_size=298 MB
postings_scored=293022
posting_pages_read=10975
rerank_rows=50
heap_blocks_fetched=48
approximate_scan_elapsed_us=79706
exact_rerank_elapsed_us=944
execution_time=84.427 ms
```

Sidecar row visible before SSM stdout truncation:

| variant | read mode | concurrency | recall@10 | sidecar p50 | sidecar p95 | sidecar size |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| f16 | random-id | 1 | 0.9815 | 18.761 ms | 324.069 ms | 2.83 GiB |

## Sidecar Spectrum Caveat

The checked-in suite executed the intended sidecar spectrum:

- `f16`, random-id, concurrency 1
- `f16`, TID-sorted, concurrency 1
- `rabitq8`, random-id, concurrency 1
- `rabitq8`, TID-sorted, concurrency 1
- `rabitq8`, TID-sorted, concurrency 4

The SSM response truncated stdout during the sidecar table, so only the first
row is locally visible right now. The complete remote artifacts were written on
the DB host under the benchmark packet path, including `results.jsonl`,
`results-report.jsonl`, `suite-manifest.json`, and the sidecar logs.

A non-escalated SSM artifact sync failed with an SSM endpoint error. I did not
request escalation because the operator instruction was to avoid approval gates.
The stack was paused, not destroyed, so those remote artifacts are preserved for
a later artifact-copy step without rerunning the suite.

## Notes

- The sidecar steps use `allow_unsafe_index_shape=true` because the preserved
  AWS snapshot contains a `rerank=heap_f32` index, not a `rerank=off`
  sidecar-only index. Treat sidecar values as real sidecar I/O measurements on
  the preserved candidate frontier, not as product in-index sidecar storage.
- Adaptive nprobe and scratch SoA GUCs are not claimed by this packet. This
  packet exercises the current branch sanity path plus sidecar real-I/O harness.
