# Task 85 Review Request: AWS Retained Funnel Breakdown

## Scope

This packet records the AWS 1M/q500 retained block16/global1152 funnel run
using the packet-009 benchmark metrics extension. It is measurement evidence,
not a product closeout.

The purpose is to choose the next same-recall latency workstream from measured
bottlenecks and to remove the previous loophole where "future research" could
be named without being worked as part of Task 85.

## Result

Suite: `task85-aws-1m-retained-funnel-breakdown-q500`

| Step | Recall@10 | Candidate Sum | Heap Rerank Sum | p50 | p95 | p99 | Max |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| first | 0.9876 | 9,213,846 | 12,500 | 243.656 ms | 312.391 ms | 2557.207 ms | 27313.263 ms |
| warm repeat | 0.9876 | 9,213,846 | 12,500 | 224.787 ms | 281.079 ms | 292.543 ms | 299.931 ms |

Warm repeat funnel p50/p95:

| Metric | p50 | p95 |
| --- | ---: | ---: |
| object read | 181.330 ms | 242.776 ms |
| summary score | 47.541 ms | 49.267 ms |
| row score | 10.121 ms | 10.206 ms |
| candidate score total | 57.648 ms | 59.375 ms |
| leaf object bytes/query | 684,831,192 | 708,813,432 |
| summary bytes/query | 74,357,224 | 76,959,928 |
| row bytes/query | 610,463,408 | 631,842,944 |
| selected blocks/query | 1,152 | 1,152 |
| candidates/query | 18,431 | 18,432 |

## Interpretation

The retained-recall surface is not primarily blocked on heap rerank or row
score CPU. At the current same-recall candidate point, the warm run spends
about 181 ms p50 in object reads and reads about 685 MB/query of leaf object
payload, of which about 610 MB/query is row payload.

That makes Task 85's next required workstream read-path/layout reduction:
reduce row payload bytes and object-read latency while preserving the retained
candidate set, recall, and heap rerank width. Summary-score CPU remains a real
workstream, but it is secondary until the read-path hypothesis is attempted or
rejected with packet-local evidence.

## Evidence

- Suite config:
  `reviews/task-85/010-aws-retained-funnel-breakdown/suite-aws-1m-retained-funnel-breakdown-q500.json`
- Suite report:
  `reviews/task-85/010-aws-retained-funnel-breakdown/artifacts/aws-1m-retained-funnel-breakdown-q500/suite-report.md`
- Raw suite manifest/results:
  `reviews/task-85/010-aws-retained-funnel-breakdown/artifacts/aws-1m-retained-funnel-breakdown-q500/suite-manifest.json`
  and
  `reviews/task-85/010-aws-retained-funnel-breakdown/artifacts/aws-1m-retained-funnel-breakdown-q500/results.jsonl`
- Funnel summaries:
  `reviews/task-85/010-aws-retained-funnel-breakdown/artifacts/funnel-first-summary.json`
  and
  `reviews/task-85/010-aws-retained-funnel-breakdown/artifacts/funnel-repeat-summary.json`
- AWS final state:
  `reviews/task-85/010-aws-retained-funnel-breakdown/artifacts/aws-ec2-status-final-after-wait.log`

## AWS State

After the run, `ecaz-cloud-1m-db` and `ecaz-cloud-1m-loader` were both
confirmed `stopped`.
