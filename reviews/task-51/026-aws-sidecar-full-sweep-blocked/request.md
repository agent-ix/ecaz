# Review Request: AWS Sidecar Full Sweep Blocked By Cloud State

## Scope

This packet records the attempted AWS 1M sidecar-only sweep for the new RaBitQ8 variants:

- `rabitq8`
- `rabitq8ls`
- `rabitq8c3`
- `rabitq8c4`

No vchord, pgvectorscale/DiskANN, or unchanged comparator benchmarks were run.

## Outcome

No AWS benchmark numbers were produced. The run was blocked before install/bench execution by stale Terraform/AWS EBS state.

Observed failure:

- `ecaz cloud up --profile 10k-medium --from-snapshot snap-0b72153293b0b749b ...`
- Terraform state still referenced `vol-0a8a848f89f637f25`.
- AWS reported `IncorrectState: vol-0a8a848f89f637f25 is not 'available'` when attaching to the new DB instance.
- Partial compute resources were destroyed after the failed attempts.
- Final status reported `state: down`, `$0.00/hr running`.

## Code/Config Change

The `10k-medium` profile was corrected from 50 GB to 100 GB so it matches the preserved 1M benchmark snapshot volume size. This prevents Terraform from trying an impossible EBS shrink from 100 GB to 50 GB.

Files:

- `infra/cloud/terraform/profiles/10k-medium.tfvars`
- `crates/ecaz-cloud/src/profiles.rs`

## Evidence

Benchmark packet:

- `benchmarks/task51-aws-rabitq8-sidecar-full-sweep/manifest.md`
- `benchmarks/task51-aws-rabitq8-sidecar-full-sweep/suite.json`
- `benchmarks/task51-aws-rabitq8-sidecar-full-sweep/artifacts/suite-audit-local.log`
- `benchmarks/task51-aws-rabitq8-sidecar-full-sweep/artifacts/suite-dry-run-local.log`
- `benchmarks/task51-aws-rabitq8-sidecar-full-sweep/artifacts/cloud-up-converge-100gb.log`
- `benchmarks/task51-aws-rabitq8-sidecar-full-sweep/artifacts/cloud-up-converge-attach-retry.log`
- `benchmarks/task51-aws-rabitq8-sidecar-full-sweep/artifacts/cloud-up-from-snapshot-after-volume-delete.log`
- `benchmarks/task51-aws-rabitq8-sidecar-full-sweep/artifacts/cloud-status-after-second-attach-down.log`
- `benchmarks/task51-aws-rabitq8-sidecar-full-sweep/artifacts/cloud-up-dry-run-after-stale-volume.log`

## Review Focus

Review the 100 GB profile correction and the blocker assessment. A successful AWS benchmark still requires state repair so the next startup creates a fresh restored volume from `snap-0b72153293b0b749b` instead of reusing stale `vol-0a8a848f89f637f25`.
