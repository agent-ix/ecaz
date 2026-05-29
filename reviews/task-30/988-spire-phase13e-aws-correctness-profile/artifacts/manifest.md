# Manifest: Phase 13e AWS Correctness Profile Cleanup Checkpoint

- Head SHA: `65e2f7528`
- Task bucket: `reviews/task-30/988-spire-phase13e-aws-correctness-profile`
- Lane: AWS Graviton Phase 13e correctness/profile setup reset
- Timestamp: 2026-05-26
- Topology target: `m7g.large` coordinator plus 3 x `m7g.large` remotes, `us-west-2a`
- Storage format / rerank mode: not applicable; no AWS correctness workload ran in this checkpoint
- Isolation: AWS account residue cleanup and preflight only; no EC2 instances were running after cleanup

## Artifacts

### `artifacts/provision.log`

- Command:
  `SPIRE_AWS_ALLOW_PREEXISTING_RESIDUE=1 make -C infra/spire-aws ARTIFACT_DIR=reviews/task-30/988-spire-phase13e-aws-correctness-profile/artifacts provision`
- Result: failed before instances were created.
- Key lines:
  - `VpcLimitExceeded: The maximum number of VPCs has been reached.`
  - `EntityAlreadyExists: Role with name ecaz-spire-aws-node already exists.`

### `artifacts/teardown-after-failed-provision.log`

- Command:
  `make -C infra/spire-aws ARTIFACT_DIR=reviews/task-30/988-spire-phase13e-aws-correctness-profile/artifacts teardown`
- Result: destroyed the partial failed-provision resources tracked in the active Terraform state.

### `artifacts/terraform-state-redacted-summary.md`

- Command:
  `make -C infra/spire-aws ARTIFACT_DIR=reviews/task-30/988-spire-phase13e-aws-correctness-profile/artifacts archive-local-state`
- Result: archived stale local Terraform state before the residue reset.
- Note: the full state file is intentionally ignored and not committed because it contains generated remote database passwords. This redacted summary records the managed resource IDs needed to review the cleanup.

### `artifacts/cleanup-residue-execute.log`

- Command:
  `scripts/spire-aws/cleanup-residue.sh --execute --allow-preexisting-residue`
- Result: force-deleted old active SPIRE secrets and started stale VPC endpoint deletion.
- Key lines:
  - `Force-deleted secret arn:aws:secretsmanager:us-west-2:932658697181:secret:ecaz-spire-aws-90599215-remote-1-20260525221947626500000001-ZoRM5b`
  - `Deleted VPC endpoints in VPC vpc-08e477285812abc44`
- Note: this first pass exposed the missing endpoint waiter and left the endpoint security group dependency-blocked.

### `artifacts/cleanup-residue-execute-2.log`

- Command:
  `scripts/spire-aws/cleanup-residue.sh --execute --allow-preexisting-residue`
- Result: completed stale VPC and IAM cleanup through the fixed waiter path.
- Key lines:
  - `No Secrets Manager secrets matched prefix ecaz-spire-aws`
  - `Deleted VPC vpc-08e477285812abc44`
  - `Deleted IAM role ecaz-spire-aws-node`
  - `Deleted IAM instance profile ecaz-spire-aws-node`

### `artifacts/preflight-after-residue-cleanup.log`

- Command:
  `SPIRE_AWS_ALLOW_PREEXISTING_RESIDUE=1 make -C infra/spire-aws ARTIFACT_DIR=reviews/task-30/988-spire-phase13e-aws-correctness-profile/artifacts preflight-operator preflight-state preflight-permissions`
- Result: passed.
- Key lines:
  - `SPIRE AWS operator preflight passed: region=us-west-2 az=us-west-2a ami=ami-04e0d7d889f694536 coordinator=m7g.large remote=m7g.large remote_count=3`
  - `SPIRE AWS state preflight passed: local Terraform state has no managed resources`
  - `SPIRE AWS permission preflight passed`

### `artifacts/preflight-static-after-cleanup-script.log`

- Command:
  `make -C infra/spire-aws preflight`
- Result: passed.
- Key lines:
  - `Success! The configuration is valid.`
  - `shellcheck not found; skipping shellcheck`
  - `jq empty scripts/spire-aws/suite-correctness.json scripts/spire-aws/suite-representative.json scripts/spire-aws/suite-stress.json`
