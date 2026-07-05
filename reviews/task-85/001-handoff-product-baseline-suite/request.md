# Task 85 Review Request: Handoff and Product Baseline Suite

## Summary

This packet starts Task 85 as the larger product-scale Pareto program requested
after Task 84. It does not claim a product profile yet. It defines the first
AWS 1M/q500 baseline suite that future latency work must beat at matched or
improved recall.

The Task 84 handoff is explicit:

- Task 84 packet 006 closed recall recovery with no bounded selected-block
  recovery policy.
- Task 84 packet 007 corrected the user-goal framing to latency at retained
  recall and found no Task 84 mechanism beats the warmed retained k2 surface.
- Reviewer feedback accepted packet 007 and set the operational floor at about
  `recall@10=0.9832`, `candidate_sum=9.21M`, warmed p50 about `255 ms`,
  p95 about `315 ms`, p99 about `332 ms`.

## Baseline Method

The checked-in suite:

`reviews/task-85/001-handoff-product-baseline-suite/suite-aws-1m-product-baseline-q500.json`

will run through `ecaz bench suite` and captures:

- precheck rows for host/database/input/index surface and SPIRE GUCs;
- retained Task 79/81 surface twice in order, so Task 85 uses the warmed repeat
  row as the latency floor rather than a cold artifact;
- Task 83 global-cap controls at `1280` and `1536`;
- storage for the retained SPIRE 1M surface.

The suite keeps q500, `nprobe=96`, `rerank_width=25`, the retained k2 block
summary index, and the existing q500 truth cache constant.

## Comparator Policy

This packet cites the current IVF comparator from
`benchmarks/task51-aws-ivf-rabitq-final-gate/`. HNSW and DiskANN 1M comparator
evidence is not complete enough in the current checkout to close Task 85; later
Task 85 packets must either run those rows or explicitly close with that gap
listed.

## Product Baseline Result

The AWS 1M/q500 product-baseline suite completed after the Task 85 cloud
wrapper fixes in packet 002. All 6 suite steps succeeded and the final AWS
status was captured as `paused`.

The warmed retained Task 79/81 row is the Task 85 latency floor:

| Step | Recall@10 | p50 | p95 | p99 | Candidate Sum | Heap Rerank Sum |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| retained global1152 first | 0.9832 | 264.946 ms | 328.183 ms | 338.347 ms | 9,213,846 | 12,500 |
| retained global1152 warm repeat | 0.9832 | 246.397 ms | 304.476 ms | 321.342 ms | 9,213,846 | 12,500 |
| Task83 global1280 control | 0.9846 | 255.151 ms | 309.259 ms | 325.029 ms | 10,237,554 | 12,500 |
| Task83 global1536 control | 0.9876 | 272.482 ms | 327.933 ms | 337.447 ms | 12,284,852 | 12,500 |

Interpretation: the Task83 controls improve recall by spending more candidate
work, but neither is a latency win at the retained recall point. Task 85 should
therefore optimize from the retained global1152 warm row and reject any option
that lowers recall below `0.9832`.

Storage for the retained 1M surface is `18.4 GiB` total, with the retained
SPIRE k2 index `872.1 MiB` / `923.7 B` per row and the Task84 k3 control index
`936.4 MiB` / `991.9 B` per row.

## Validation

- `ecaz bench suite audit`: passed for the 6-step Task 85 AWS 1M/q500
  product-baseline suite.
- `ecaz cloud bench`: completed
  `task85-aws-1m-product-baseline-q500`; synced artifacts from
  `s3://ecaz-cloud-1m-b62eb804/bench-artifacts/task85-aws-1m-product-baseline-q500/20260607T060021Z/`.
- `ecaz bench suite report`: completed with 6 succeeded, 0 failed, 0 skipped.
- AWS lifecycle: profile `1m` was paused before the successful run, resumed for
  the suite, paused immediately after artifact sync, and final status was
  captured as `paused`.

## Requested Review

Please review whether this measured retained warm row is the correct Task 85
floor: future work must beat p50 `246.397 ms` / p95 `304.476 ms` / p99
`321.342 ms` at `recall@10 >= 0.9832`, not compare against cold rows,
pre-Task-79 surfaces, or lower-recall configurations.
