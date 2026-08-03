# Task 201 packet 003: MAT-40 release matrix

- Head SHA: `c830b184fe4c750936ab13eab2891f63f06ba3d0`.
- Task bucket: `reviews/task-201/003-release-matrix-and-decision/`.
- Suite config: `artifacts/task201-mat40-release-10-50-100k.json` (SHA256 `a88f792b9c789fec0a3fc97fbbf9309940f78bf599e10aa4043468ca111954b2`).
- Results: `artifacts/run-v2/results.jsonl` (SHA256 `92f7592ff518905255ca727878f8f0d5d1cefbc0ef2d91e2e732ca411943b5f0`).
- Suite manifest: `artifacts/run-v2/suite-manifest.json` (SHA256 `1580786e259cdc3f91a21df920a314ecc34b0a82b8553a3086196ca26ebcd62e`).
- Command: `/home/peter/dev/ecaz/.worktrees/task201/target/release/ecaz bench suite run --config reviews/task-201/003-release-matrix-and-decision/artifacts/task201-mat40-release-10-50-100k.json --artifact-dir reviews/task-201/003-release-matrix-and-decision/artifacts/run-v2`.
- Runner manifest reports all three steps succeeded with exit code 0. The initial 10k attempt was discarded before results because its 200-row evaluation file could not also supply the required 200 training landmarks; the corrected v2 arm uses the staged 50k query file for disjoint training rows 201–400 and the 10k file for evaluation rows 1–200. It is the only 10k result cited here.
- All arms used PG18, release extension build `c830b184fe4c750936ab13eab2891f63f06ba3d0`, three physical nodes, one shared physical table surface, 4096-head `training_landmarks_exact`, persisted head, width 32, seed count 32, RaBitQ neighbors, beam 4, hop rounds 100, materialization batch 10, 200 evaluation queries, 50 measured iterations, and 10 warmups. Only `owner_payload_plan_cache` differs: off control versus on MAT-40 candidate.
- Run directories were outside the repository under `/home/peter/.ecaz/clusters/` and are disposable fixture state, not evidence.

## Release matrix

| scale | variant | recall | mean / p50 / p95 / p99 / max ms | physical generation / coordinator source / control index bytes |
| --- | --- | ---: | --- | --- |
| 10k | control | 0.9990 | 15.60 / 15.20 / 18.80 / 19.20 / 19.30 | 242,745,344 / 166,715,392 / 24,576 |
| 10k | MAT-40 candidate | 0.9990 | 16.10 / 16.10 / 18.90 / 19.60 / 19.80 | 242,745,344 / 166,715,392 / 24,576 |
| 50k | control | 0.9685 | 16.40 / 16.10 / 18.60 / 19.40 / 19.90 | 1,242,750,976 / 833,224,704 / 24,576 |
| 50k | MAT-40 candidate | 0.9685 | 16.20 / 16.20 / 18.40 / 19.30 / 19.60 | 1,242,750,976 / 833,224,704 / 24,576 |
| 100k | control | 0.9625 | 16.00 / 16.00 / 18.80 / 19.30 / 19.50 | 2,496,651,264 / 1,666,342,912 / 24,576 |
| 100k | MAT-40 candidate | 0.9625 | 15.80 / 15.90 / 18.50 / 19.10 / 19.10 | 2,496,651,264 / 1,666,342,912 / 24,576 |

The candidate is 3.2% slower at 10k and 1.2% faster at both 50k and 100k. Recall and storage are identical at every scale. The scale-dependent, small latency movement does not meet a release-promotion bar, so MAT-40 remains off and no production default or follow-up implementation task is opened.

The packet-local `run-v2/mat40-release-*/distann-multinode-summary.log` files contain the cited compact result lines. `run-v2/results.jsonl` is the structured source of truth; no corpus TSV, raw node log, polling exhaust, or prediction capture is committed.
