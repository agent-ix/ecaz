# Review Request: Task 143 Packet 004 - Release 50k/n1024 A/B Evidence

## Summary

This packet adds the second release `ecaz bench suite` evidence slice for Task 143.

It runs the 50k/n1024/b0 anchor shape against five variants:

1. baseline accumulated-score routing: `leaf_score_only_routing=off`, `route_overfetch_multiplier=1.0`
2. leaf-score-only final routing: `leaf_score_only_routing=on`, `route_overfetch_multiplier=1.0`
3. accumulated-score routing with overfetch alpha `1.25`
4. accumulated-score routing with overfetch alpha `1.5`
5. accumulated-score routing with overfetch alpha `2.0`

The suite precheck records `ecaz_build_profile() = release`; the suite manifest records `build_profile: release` and `coordinator:28818:release`; every `spire-pipeline` row in `suite-results.jsonl` carries the release backend profile.

## Evidence

- `artifacts/suite-task143-50k-n1024-ab.json`: checked-in suite config.
- `artifacts/suite-manifest.json`: all nine suite steps succeeded.
- `artifacts/suite-results.jsonl`: structured load, storage, recall, and pipeline metrics.
- `artifacts/precheck-host.log`: release backend profile and default GUC values.
- `artifacts/manifest.md`: artifact manifest and compact A/B route-containment summary table.

The generated truth-cache JSON and raw per-query pipeline diagnostic JSONL files are intentionally not committed because they are gitignored regenerable data. The recall log, suite result rows, pipeline logs, and compact route-containment table are committed.

## Key Result

At 50k/n1024, leaf-score-only routing improves baseline recall across the ladder and is faster at the highest tested nprobe values:

| Variant | nprobe 32 distinct recall | nprobe 32 p50 | nprobe 64 distinct recall | nprobe 64 p50 | nprobe 96 distinct recall | nprobe 96 p50 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| baseline | 0.8965 | 65.688 ms | 0.9390 | 128.159 ms | 0.9590 | 187.015 ms |
| leaf-only | 0.9105 | 66.356 ms | 0.9475 | 122.661 ms | 0.9595 | 182.717 ms |
| overfetch-1.25 | 0.9070 | 67.732 ms | 0.9440 | 126.423 ms | 0.9590 | 183.245 ms |
| overfetch-1.5 | 0.9090 | 65.906 ms | 0.9475 | 123.916 ms | 0.9600 | 184.283 ms |
| overfetch-2.0 | 0.9110 | 67.585 ms | 0.9485 | 124.577 ms | 0.9605 | 183.365 ms |

Route-containment matches final distinct recall in the stage-containment rows, so the 50k/n1024 evidence still supports the routing-defect hypothesis. Overfetch alpha `2.0` gives the highest recall in this slice, but it is not a clean latency win; leaf-only remains the simpler candidate for promotion unless 100k changes the tradeoff.

## Feedback Handling

Before writing this packet I re-scanned `reviews/task-141`, `reviews/task-142`, and `reviews/task-143` feedback. New Task 141 feedback `2026-07-05-02-agent-ix.md` approves Task 141 and explicitly says the P0 substrate is complete and unblocks 142-146. No Task 142 or Task 143 feedback files were present.

## Review Focus

1. Confirm the packet satisfies the Task 143 50k/n1024 release evidence shape: release backend, suite-driven, pipeline plus containment artifacts.
2. Confirm the interpretation that leaf-only is still a strong default candidate, while overfetch alpha `2.0` is a recall-only improvement in this slice.
3. Confirm the remaining Task 143 work should proceed to 100k release A/B and then a final decision packet.
