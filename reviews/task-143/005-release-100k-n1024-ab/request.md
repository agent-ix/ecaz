# Review Request: Task 143 Packet 005 - Release 100k/n1024 A/B Evidence

## Summary

This packet adds the 100k/n1024 release `ecaz bench suite` evidence slice for Task 143.

It runs the 100k/n1024/b0 anchor shape against five variants:

1. baseline accumulated-score routing: `leaf_score_only_routing=off`, `route_overfetch_multiplier=1.0`
2. leaf-score-only final routing: `leaf_score_only_routing=on`, `route_overfetch_multiplier=1.0`
3. accumulated-score routing with overfetch alpha `1.25`
4. accumulated-score routing with overfetch alpha `1.5`
5. accumulated-score routing with overfetch alpha `2.0`

The suite precheck records `ecaz_build_profile() = release`; the suite manifest records `build_profile: release` and `coordinator:28818:release`; every `spire-pipeline` row in `suite-results.jsonl` carries the release backend profile.

## Evidence

- `artifacts/suite-task143-100k-n1024-ab.json`: checked-in suite config.
- `artifacts/suite-manifest.json`: all nine suite steps succeeded.
- `artifacts/suite-results.jsonl`: structured load, storage, recall, and pipeline metrics.
- `artifacts/precheck-host.log`: release backend profile and default GUC values.
- `artifacts/manifest.md`: artifact manifest and compact A/B route-containment summary table.

The generated truth-cache JSON and raw per-query pipeline diagnostic JSONL files are intentionally not committed because they are gitignored regenerable data. The recall log, suite result rows, pipeline logs, and compact route-containment table are committed.

## Key Result

At 100k/n1024, leaf-score-only routing is the best recall variant at every tested nprobe:

| Variant | nprobe 32 distinct recall | nprobe 32 p50 | nprobe 64 distinct recall | nprobe 64 p50 | nprobe 96 distinct recall | nprobe 96 p50 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| baseline | 0.8585 | 123.136 ms | 0.9120 | 241.140 ms | 0.9300 | 371.433 ms |
| leaf-only | 0.8895 | 118.427 ms | 0.9375 | 246.891 ms | 0.9570 | 362.912 ms |
| overfetch-1.25 | 0.8680 | 119.511 ms | 0.9195 | 241.390 ms | 0.9405 | 378.804 ms |
| overfetch-1.5 | 0.8750 | 119.418 ms | 0.9225 | 236.499 ms | 0.9465 | 369.890 ms |
| overfetch-2.0 | 0.8840 | 117.992 ms | 0.9315 | 239.017 ms | 0.9505 | 365.220 ms |

Route-containment matches final distinct recall in the stage-containment rows. Overfetch alpha `2.0` is a useful improvement over baseline, but it does not recover as much recall as leaf-only at 100k. Across packets 003-005, the current evidence favors promoting leaf-only routing first and treating overfetch as non-default follow-up unless a later task needs its latency/recall tradeoff.

## Feedback Handling

Before this slice I re-scanned `reviews/task-141` through `reviews/task-146` feedback. The only feedback present remains Task 141 packet 001: `2026-07-04-01-agent-ix.md` and the superseding approval `2026-07-05-02-agent-ix.md`. No Task 142-146 feedback files were present.

## Review Focus

1. Confirm the packet satisfies the Task 143 100k/n1024 release evidence shape: release backend, suite-driven, pipeline plus containment artifacts.
2. Confirm the interpretation that leaf-only dominates recall across this 100k slice, while overfetch improves baseline but does not catch leaf-only.
3. Confirm Task 143 can proceed to a final decision packet using release evidence from packets 003, 004, and 005.
