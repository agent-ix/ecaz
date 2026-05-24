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

The stack was snapshotted and torn down after artifact retrieval:

```text
snapshot: snap-0758119609e81ab7f
state: down
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

Sidecar real-I/O, q=200, nprobe=128, candidate_k=50:

| variant | read mode | concurrency | recall@10 | sidecar p50 | sidecar p95 | sidecar p99 | total bound p50 | sidecar size |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| f16 | random-id | 1 | 0.9815 | 18.761 ms | 324.069 ms | 529.692 ms | 63.026 ms | 2.83 GiB |
| f16 | TID-sorted | 1 | 0.9815 | 0.523 ms | 0.787 ms | 1.920 ms | 43.619 ms | 2.83 GiB |
| rabitq8 | random-id | 1 | 0.9455 | 1.918 ms | 4.819 ms | 11.585 ms | 45.166 ms | 1.43 GiB |
| rabitq8 | TID-sorted | 1 | 0.9455 | 0.413 ms | 0.437 ms | 0.535 ms | 43.499 ms | 1.43 GiB |
| rabitq8 | TID-sorted | 4 | 0.9455 | 1.121 ms | 1.723 ms | 334.866 ms | 41.615 ms | 1.43 GiB |

## Artifact Status

Remote artifacts were copied into the benchmark packet after the run. The suite
manifest now reports:

```text
[suite:task51-aws-ivf-rabitq-current-head-final-gate] completed=6 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

## Notes

- The sidecar steps use `allow_unsafe_index_shape=true` because the preserved
  AWS snapshot contains a `rerank=heap_f32` index, not a `rerank=off`
  sidecar-only index. Treat sidecar values as real sidecar I/O measurements on
  the preserved candidate frontier, not as product in-index sidecar storage.
- Adaptive nprobe and scratch SoA GUCs are not claimed by this packet. This
  packet exercises the current branch sanity path plus sidecar real-I/O harness.
