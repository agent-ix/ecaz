# Task 81 Packet 007: AWS 100k Task 79 Comparison

## Summary

This packet applies the corrected Task 81 acceptance comparison requested by
the user: compare against Task 79's accepted optimized candidate surface, not
the old full-leaf `15.5M` row.

The accepted run reuses the retained Task 79 AWS 100k/q200 surface and reruns
the `global1152` row with the current Task 81 branch code after a `global1024`
warmup row, matching the Task 79 measurement order closely enough for a
latency comparison.

## Result

Task 81 beats Task 79 at the same recall/candidate point on AWS:

| Row | Candidates | Recall@10 | p50 | p95 | p99 |
| --- | ---: | ---: | ---: | ---: | ---: |
| Task 79 accepted AWS `global1152` | `3,672,619` | `0.9945` | `35.199 ms` | `36.203 ms` | `36.591 ms` |
| Task 81 retained-surface warm `global1152` | `3,672,619` | `0.9945` | `32.023 ms` | `32.940 ms` | `33.315 ms` |

That is a `3.176 ms` p50 improvement (`9.02%`) with unchanged recall and
candidate count.

The local comparison had already cleared the same corrected bar:

- Task 79 local accepted `global1152`: `3,673,383` candidates, p50 `35.293 ms`,
  recall@10 `0.9940`.
- Task 81 local tg256/nprobe96 `global1152`: `3,672,619` candidates, p50
  `32.212 ms`, recall@10 `0.9945`.

## Notes

- The earlier Task 81 AWS q500 1M rows remain negative scale evidence; they do
  not pass the old q500 recall goal.
- The user clarified that the relevant completion bar is reduced latency at the
  Task 79 optimized recall/candidate point. This packet is the accepted AWS
  evidence for that corrected bar.
- A fresh-copy AWS suite failed because the retained host lacked enough free
  disk space to copy and encode another 100k table. The accepted run avoids
  this by reusing Task 79's retained AWS surface.

## Evidence

- Manifest: `artifacts/manifest.md`
- Accepted suite config:
  `suite-aws-100k-task79-retained-surface-warm.json`
- Accepted suite manifest:
  `artifacts/task79-retained-surface-warm/suite-manifest.json`
- Accepted results:
  `artifacts/task79-retained-surface-warm/results.jsonl`
- Accepted parsed report:
  `artifacts/task79-retained-surface-warm/suite-report-results.jsonl`
- Accepted status/report logs:
  - `artifacts/task79-retained-surface-warm/suite-status.log`
  - `artifacts/task79-retained-surface-warm/suite-report.log`
- Accepted cloud logs:
  - `artifacts/cloud-resume-before-warm-retained-surface.log`
  - `artifacts/cloud-bench-retained-task79-surface-warm.log`
  - `artifacts/cloud-pause-after-warm-retained-surface.log`

## Reviewer Focus

1. Confirm Task 79 is the correct baseline for Task 81 closeout.
2. Confirm the retained-surface AWS comparison is valid because it uses the
   exact Task 79 table/index/prefix and only reruns the pipeline with current
   branch code.
3. Confirm Task 81 can close on the corrected latency-at-same-recall objective,
   while preserving the q500 1M failures as non-accepted scale evidence.
