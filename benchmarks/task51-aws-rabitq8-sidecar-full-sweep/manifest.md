# Task 51 AWS RaBitQ8 Sidecar Full Sweep Attempt

- Timestamp: 2026-05-24T00:18:45Z
- Branch: `aws-optimization-ivf-rabitq-spire`
- Task bucket: `reviews/task-51`
- Benchmark packet: `benchmarks/task51-aws-rabitq8-sidecar-full-sweep`
- Intended scope: AWS 1M IVF/RaBitQ sidecar-only sweep
- Intended variants: `rabitq8`, `rabitq8ls`, `rabitq8c3`, `rabitq8c4`
- Excluded: vchord, pgvectorscale/DiskANN, unchanged comparator reruns
- Restored snapshot target: `snap-0b72153293b0b749b`
- AWS profile: `10k-medium`
- AWS shape: DB `m8g.xlarge`, loader `c8g.medium`

## Outcome

No AWS benchmark rows were produced in this attempt.

The suite was prepared and locally audited, but AWS did not reach install or benchmark execution. `ecaz cloud up` repeatedly tried to reuse stale Terraform state for EBS volume `vol-0a8a848f89f637f25`; AWS reported that volume was not `available` during attach. The partial compute resources were destroyed after each failed attach attempt.

Final observed status:

```text
profile:  10k-medium
state:    down
snapshot: snap-0b72153293b0b749b
cost:     ~$0.00/hr running, ~$4.00/mo retained storage
```

## Evidence

- `suite.json`: Intended sidecar-only SuiteConfig.
- `artifacts/suite-audit-local.log`: suite audit passed.
- `artifacts/suite-dry-run-local.log`: suite dry-run expanded to the intended precheck and `sidecar-rerank` command.
- `artifacts/suite-dry-run-manifest.json`: dry-run manifest for the intended run.
- `artifacts/cloud-up-from-snapshot.log`: first restore attempt; failed on profile volume size mismatch before attach.
- `artifacts/cloud-up-converge-100gb.log`: retried after correcting profile volume size; failed because stale volume was not attachable.
- `artifacts/cloud-up-converge-attach-retry.log`: attach retry failed with same stale-volume state.
- `artifacts/cloud-down-after-attach-failure.log`: partial stack teardown after first attach failure.
- `artifacts/cloud-up-from-snapshot-after-volume-delete.log`: later retry still planned to reuse the stale volume and failed attaching it.
- `artifacts/cloud-down-after-second-attach-failure.log`: partial stack teardown after second attach failure.
- `artifacts/cloud-status-after-second-attach-down.log`: status after teardown reported profile down and zero running compute spend.
- `artifacts/cloud-up-dry-run-after-stale-volume.log`: dry-run after teardown still planned to reuse `vol-0a8a848f89f637f25`; no compute was started by this dry-run.

## Required Follow-Up

Before another AWS benchmark attempt, repair the `10k-medium` Terraform state so `aws_ebs_volume.db` no longer points at stale `vol-0a8a848f89f637f25`. The next real startup should create a fresh DB data volume from `snap-0b72153293b0b749b` and then run install + sidecar suite in one continuous session.

Do not interpret this packet as a performance result.
