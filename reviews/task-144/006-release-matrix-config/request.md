# Review Request: Task 144 Packet 006 - Release Matrix Suite Config

## Summary

This packet pre-registers the Task 144 release matrix as an `ecaz bench suite` config.

The suite covers:

- Scales: `10k`, `50k`, `100k`
- Assignment variants:
  - `single`: `boundary_replica_count=0`, `closure_epsilon=0`
  - `fixed_b2`: `boundary_replica_count=2`, `closure_epsilon=0`
  - `closure_e010_b8`: `boundary_replica_count=8`, `closure_epsilon=0.10`
- Query modes:
  - fixed nprobe sweep
  - `ec_spire.probe_distance_ratio=1.25`
  - `--adaptive-nprobe`
- Per loaded shape:
  - storage step
  - `spire-pipeline` recall/latency/profile cells
  - stage containment JSONL for per-query route/probed-list and recall-tail analysis
  - result identity JSONL for returned-id auditing

This is a config/checkpoint packet only. The release matrix has not been run yet.

## Artifacts

- `artifacts/suite-task144-release-matrix.json`
- `artifacts/suite-audit.log`
- `artifacts/suite-dry-run.log`
- `artifacts/suite-dry-run-manifest.json`

## Validation

Audit:

```text
target/release/ecaz bench suite audit --config reviews/task-144/006-release-matrix-config/artifacts/suite-task144-release-matrix.json
```

Result:

```text
[suite:task144-release-matrix] audit passed: 46 steps
```

Dry-run:

```text
target/release/ecaz bench suite run --config reviews/task-144/006-release-matrix-config/artifacts/suite-task144-release-matrix.json --dry-run --manifest-output reviews/task-144/006-release-matrix-config/artifacts/suite-dry-run-manifest.json --results-output reviews/task-144/006-release-matrix-config/artifacts/results-dry-run.jsonl
```

Dry-run manifest:

- 46 total steps
- 1 raw precheck
- 9 load
- 9 storage
- 27 `spire-pipeline`
- 9 ratio-pruning cells
- 9 adaptive cells

## Review Focus

Please review whether this suite shape is sufficient for Task 144 closeout execution before the long release run:

- Are the assignment variants enough for isolated closure on/off and fixed-count control?
- Is `probe_distance_ratio=1.25` the right first ratio-pruning value, or should the suite include another ratio before spending the run?
- Are `stage_containment_output` and `result_identity_output` sufficient for the requested per-query probed-list and recall-tail distributions?
