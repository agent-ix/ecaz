# Artifact Manifest

- Head SHA at request creation: `19b410928c131530063c5391147fb0bc387c8b4a`
- Task bucket: `reviews/task-51`
- Packet path: `reviews/task-51/025-cloud-bench-remote-sidecar-gate`
- Benchmark packet: `benchmarks/task51-aws-rabitq8c4-sidecar-final-gate`
- Timestamp: 2026-05-23T23:34:49Z
- Lane: AWS final gate attempt
- Fixture: preserved `real_1m_ivf_rabitq1_rerank`
- Storage/index format: IVF/RaBitQ
- Sidecar variants intended: `rabitq8`, `rabitq8ls`, `rabitq8c3`, `rabitq8c4`
- Read mode: `tid-sorted`
- Candidate frontier: `candidate_k=50`, `k=10`, `nprobe=128`, `queries_limit=200`
- Instance shape: DB `m8g.xlarge`, loader `c8g.medium`
- Surface isolation: preserved shared AWS fixture; no new index build

## Packet-Local Artifacts

This review packet intentionally points to the owning benchmark packet for raw logs, per the benchmark provenance rule:

- `benchmarks/task51-aws-rabitq8c4-sidecar-final-gate/manifest.md`
- `benchmarks/task51-aws-rabitq8c4-sidecar-final-gate/suite.json`
- `benchmarks/task51-aws-rabitq8c4-sidecar-final-gate/artifacts/suite-audit-full-sidecar-local.log`
- `benchmarks/task51-aws-rabitq8c4-sidecar-final-gate/artifacts/suite-dry-run-full-sidecar-local.log`
- `benchmarks/task51-aws-rabitq8c4-sidecar-final-gate/artifacts/cloud-bench-full-sidecar.log`
- `benchmarks/task51-aws-rabitq8c4-sidecar-final-gate/artifacts/precheck-preserved-1m-ivf-rabitq.log`
- `benchmarks/task51-aws-rabitq8c4-sidecar-final-gate/artifacts/suite-manifest.json`
- `benchmarks/task51-aws-rabitq8c4-sidecar-final-gate/artifacts/cloud-bench-remote-full-sidecar.log`
- `benchmarks/task51-aws-rabitq8c4-sidecar-final-gate/artifacts/cloud-down-after-stalled-sidecar.log`

## Result

No AWS benchmark result rows were produced. The suite did not reach the sidecar-rerank step.

Key lines cited by `request.md`:

```text
[suite:task51-aws-rabitq8-sidecar-final-gate] audit passed: 2 steps
psql: error: connection to server at "10.42.1.122", port 5432 failed: Connection timed out
ssm command 897e691d-cd68-4837-bcf1-b0d9cea44ccd on i-0b3375453f169ab75 ended in Failed (rc=-1)
profile:  10k-medium
state:    down
snapshot: snap-0b72153293b0b749b
cost:     ~$0.00/hr running, ~$4.00/mo retained storage
```
