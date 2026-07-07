# Task 146 Packet 003 Artifact Manifest

- Head SHA: `c0fd9aae4c118d48f803ce1bcf07360a95f00802`
- Task bucket: `reviews/task-146`
- Packet path: `reviews/task-146/003-single-instance-suite-config`
- Timestamp: `2026-07-07T00:38:30Z`
- Packet type: suite-config review request
- Isolated one-index-per-table or shared-table surface: separate index/table
  prefixes per scale/shape

## Artifacts

- `suite-task146-pareto-single-instance.json`
  - Command used: generated mechanically from packet 001's frozen shape table
  - Key result: 57 suite steps: 18 load, 3 recall truth-cache, 18 storage,
    18 spire-pipeline
- `dry-run-suite-manifest.json`
  - Command used:
    `target/release/ecaz bench suite run --config reviews/task-146/003-single-instance-suite-config/artifacts/suite-task146-pareto-single-instance.json --dry-run --manifest-output reviews/task-146/003-single-instance-suite-config/artifacts/dry-run-suite-manifest.json --log-file reviews/task-146/003-single-instance-suite-config/artifacts/dry-run.log`
  - Key result: 57 selected steps
- `dry-run.log`
  - Command used: same dry-run command as above
  - Key result: expanded suite commands without execution
- `audit.log`
  - Command used:
    `target/release/ecaz bench suite audit --config reviews/task-146/003-single-instance-suite-config/artifacts/suite-task146-pareto-single-instance.json`
  - Key result:
    `[suite:task146-pareto-single-instance] audit passed: 57 steps`

## Non-Claims

- No matrix cells were executed in this packet.
- No recall, latency, storage, or scan-fraction conclusion is made here.
- Packet 001 feedback obligations remain open for the matrix/results packet:
  HNSW anchors, matched 10k/50k anchors, epoch-cache engagement proof,
  viability-vs-dominance wording, and 15% scan gate justification.

