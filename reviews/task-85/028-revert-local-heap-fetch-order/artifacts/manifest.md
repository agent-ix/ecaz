# Task 85 Packet 028 Artifact Manifest

- head SHA: `7302c8369`
- task bucket: `reviews/task-85/028-revert-local-heap-fetch-order/`
- lane: Task 85 rejected-code cleanup
- fixture: not applicable
- storage format: not applicable
- rerank mode: restores pre-packet-026 local heap fetch order
- timestamp: 2026-06-07
- isolation: no benchmark run; packet 027 is the AWS evidence source

## Command

```bash
git revert --no-edit 4f92108eda6903fb524b6feb068b886622ff0122
```

## Result

- revert commit: `7302c8369`
- files changed: `src/am/ec_spire/coordinator/hierarchy_snapshots.rs`,
  `src/am/ec_spire/coordinator/tests.rs`
- validation: tests not run; see packet 027 for AWS rejection evidence
