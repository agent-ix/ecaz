# Artifact Manifest

- Task bucket: `reviews/task-51`
- Packet path: `reviews/task-51/026-aws-sidecar-full-sweep-blocked`
- Benchmark packet: `benchmarks/task51-aws-rabitq8-sidecar-full-sweep`
- Timestamp: 2026-05-24T00:18:45Z
- Lane: AWS 1M sidecar full sweep attempt
- Fixture: intended preserved `real_1m_ivf_rabitq1_rerank`
- Variants intended: `rabitq8`, `rabitq8ls`, `rabitq8c3`, `rabitq8c4`
- Read mode intended: `tid-sorted`
- Candidate frontier intended: `candidate_k=50`, `k=10`, `nprobe=128`, `queries_limit=200`
- Result: blocked before install/bench; no performance result rows

## Artifact References

- `benchmarks/task51-aws-rabitq8-sidecar-full-sweep/manifest.md`
- `benchmarks/task51-aws-rabitq8-sidecar-full-sweep/suite.json`
- `benchmarks/task51-aws-rabitq8-sidecar-full-sweep/artifacts/suite-audit-local.log`
- `benchmarks/task51-aws-rabitq8-sidecar-full-sweep/artifacts/suite-dry-run-local.log`
- `benchmarks/task51-aws-rabitq8-sidecar-full-sweep/artifacts/suite-dry-run-manifest.json`
- `benchmarks/task51-aws-rabitq8-sidecar-full-sweep/artifacts/cloud-up-from-snapshot.log`
- `benchmarks/task51-aws-rabitq8-sidecar-full-sweep/artifacts/cloud-up-converge-100gb.log`
- `benchmarks/task51-aws-rabitq8-sidecar-full-sweep/artifacts/cloud-up-converge-attach-retry.log`
- `benchmarks/task51-aws-rabitq8-sidecar-full-sweep/artifacts/cloud-down-after-attach-failure.log`
- `benchmarks/task51-aws-rabitq8-sidecar-full-sweep/artifacts/cloud-up-from-snapshot-after-volume-delete.log`
- `benchmarks/task51-aws-rabitq8-sidecar-full-sweep/artifacts/cloud-down-after-second-attach-failure.log`
- `benchmarks/task51-aws-rabitq8-sidecar-full-sweep/artifacts/cloud-status-after-second-attach-down.log`
- `benchmarks/task51-aws-rabitq8-sidecar-full-sweep/artifacts/cloud-up-dry-run-after-stale-volume.log`

## Key Lines

```text
[suite:task51-aws-rabitq8-sidecar-full-sweep] audit passed: 2 steps
api error InvalidParameterValue: New size cannot be smaller than existing size
api error IncorrectState: vol-0a8a848f89f637f25 is not 'available'
profile:  10k-medium
state:    down
snapshot: snap-0b72153293b0b749b
cost:     ~$0.00/hr running, ~$4.00/mo retained storage
```
