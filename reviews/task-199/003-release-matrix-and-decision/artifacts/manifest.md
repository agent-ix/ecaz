# Task 199 packet manifest

- Head SHA: `2a4a70b23161f556c44d6d1d2c960541fbcb1bdb`
- Packet: `reviews/task-199/003-release-matrix-and-decision/`
- Runner: `ecaz bench suite`; config: `task199-normal-release-10k-50k-100k.json`
- Exact release matrix suite manifest: `artifacts/run/suite-manifest.json`
- Result rows: `artifacts/run/results.jsonl`
- Command: `ecaz bench suite run --config .../task199-normal-release-10k-50k-100k.json --artifact-dir .../artifacts/run`; 50k and 100k resumed with `--resume-from` and `--only`.
- Timestamp: 2026-07-25; isolated three-node local-multinode PG18, shared-table physical surface, release profile.

## Results

| scale | owner recall | replica recall | owner latency ms | replica latency ms | no-replica rows/s | physical generation bytes |
|---|---:|---:|---:|---:|---:|---:|
| 10k | .9990 | .9990 | 18.30 | 16.00 | 2292.764 | 311,910,400 |
| 50k | .9685 | .9685 | 19.40 | 16.40 | 2564.925 | 1,588,461,568 |
| 100k | .9625 | .9625 | 19.90 | 16.20 | 2524.220 | 3,188,056,064 |

Storage lines report identical owner and coordinator-replica generation,
coordinator-source, and single-index bytes at each scale. The packet-local
`distann-multinode-summary.log` files are the cited raw result artifacts.

## Additional evidence

- Lifecycle and ENOSPC regression closure: `../002-operations-lifecycle-and-isolation/`.
- Historical no-replica baseline: `artifacts/no-replica-before/pre-task199-no-replica-10k/distann-multinode-summary.log` (2315.234 rows/s, pre-extension SHA `ebf9950c1e8a3a6cbbf66a19e8117f9c64b17436`).
- Graviton ordered-identity and teardown evidence: `artifacts/graviton-run/` and `artifacts/cloud-teardown-verification-r25.log`; the cloud runner predates the guard-only follow-up, while read-path code is unchanged.
- All cited suite summaries carry extension SHA `2a4a70b23` and `release` profile.
