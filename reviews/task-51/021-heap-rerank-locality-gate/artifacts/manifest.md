# Artifact Manifest: Heap Rerank Locality Gate

- task bucket: `reviews/task-51/`
- packet path: `reviews/task-51/021-heap-rerank-locality-gate/`
- timestamp: `2026-05-23T17:25:00Z`
- lane: local/AWS counter evidence review
- code change: none
- AWS: no new AWS run for this packet; cites existing final-gate artifacts
- competitors: none; IVF/RaBitQ only

## Artifacts

| artifact | notes |
| --- | --- |
| `benchmarks/task51-local-ivf-scratch-soa-chunked/artifacts/explain-static-990k-rabitq1-n1024-w50-p128.log` | Local 990k static scan counters at nprobe 128. |
| `benchmarks/task51-aws-ivf-rabitq-final-gate/artifacts/explain-1m-rabitq1-rerank-p128.log` | Existing AWS 1M counters at nprobe 128. |
| `benchmarks/task51-aws-ivf-rabitq-final-gate/artifacts/explain-1m-rabitq1-rerank-p256.log` | Existing AWS 1M counters at nprobe 256. |

## Key Lines Cited

Local 990k:

```text
Rerank Rows: 50
Heap Blocks Fetched: 48
Approximate Scan Elapsed Us: 610717
Exact Rerank Elapsed Us: 3742
Execution Time: 622.097
```

AWS 1M nprobe 128:

```text
Rerank Rows: 50
Heap Blocks Fetched: 48
Approximate Scan Elapsed Us: 36774
Exact Rerank Elapsed Us: 945
Execution Time: 40.877
```

AWS 1M nprobe 256:

```text
Rerank Rows: 50
Heap Blocks Fetched: 48
Approximate Scan Elapsed Us: 73649
Exact Rerank Elapsed Us: 454
Execution Time: 77.837
```

## Decision

Experiment 4 does not meet its start condition. Exact rerank is 0.6-2.3% of observed execution time in the cited cells, so further heap-rerank locality work cannot produce the required 15% p50 improvement. No additional benchmark run was needed for this gate decision.
