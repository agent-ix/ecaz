# Review Request: Task 143 Packet 003 - Release 10k A/B Evidence

## Summary

This packet adds the first release `ecaz bench suite` evidence for Task 143.

It runs the 10k n128/b0 anchor shape against three variants:

1. baseline accumulated-score routing: `leaf_score_only_routing=off`, `route_overfetch_multiplier=1.0`
2. leaf-score-only final routing: `leaf_score_only_routing=on`, `route_overfetch_multiplier=1.0`
3. accumulated-score routing with overfetch: `leaf_score_only_routing=off`, `route_overfetch_multiplier=1.5`

The suite precheck records `ecaz_build_profile() = release` and the suite manifest records `build_profile: release` / `coordinator:28818:release`.

## Evidence

- `artifacts/suite-task143-10k-ab.json`: checked-in suite config.
- `artifacts/suite-manifest.json`: all seven suite steps succeeded.
- `artifacts/suite-results.jsonl`: structured load, storage, recall, and pipeline metrics.
- `artifacts/precheck-host.log`: release backend profile and default GUC values.
- `artifacts/manifest.md`: artifact manifest and compact A/B route-containment summary table.

The generated truth-cache JSON and raw per-query pipeline diagnostic JSONL files are intentionally not committed because they are gitignored regenerable data; the recall log, suite result rows, pipeline logs, and compact route-containment table are committed.

## Key Result

At 10k, leaf-score-only routing is the best tested variant:

| Variant | nprobe 32 distinct recall | nprobe 32 p50 | First perfect recall nprobe |
| --- | ---: | ---: | ---: |
| baseline | 0.9965 | 92.721 ms | 96 |
| leaf-only | 1.0000 | 89.706 ms | 32 |
| overfetch-1.5 | 0.9985 | 89.657 ms | 64 |

Route-containment matches final distinct recall in the stage-containment rows, so this packet supports Task 143's routing-defect hypothesis at 10k: leaf-only ranking recovers the missing truth leaves without adding latency.

## Review Focus

1. Confirm the packet satisfies the Task 143 10k release evidence requirement shape: release backend, suite-driven, pipeline plus containment artifacts.
2. Confirm the interpretation that leaf-only wins this 10k slice and overfetch `alpha=1.5` is not better than leaf-only here.
3. Confirm the remaining work should proceed to 50k/100k release A/B and the rest of the alpha sweep.
