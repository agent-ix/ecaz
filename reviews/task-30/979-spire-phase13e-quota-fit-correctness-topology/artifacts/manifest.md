# Artifact Manifest: SPIRE Phase 13e Quota-Fit Correctness Topology

Head SHA: `9f097000b5036d1b472844bb56ea3690864e02f4`
Task bucket: `reviews/task-30/979-spire-phase13e-quota-fit-correctness-topology`
Timestamp: `2026-05-25T21:44:10Z`
Lane: local operator-surface validation only
AWS resources provisioned: no
Storage format / rerank mode: not applicable
Surface isolation: not applicable; no benchmark tables were created

## Artifacts

| Artifact | Command | Result |
| --- | --- | --- |
| `bash-syntax.log` | `bash -n scripts/spire-aws/preflight-operator.sh scripts/spire-aws/bootstrap-node.sh` | exit 0 |
| `terraform-fmt-check.log` | `terraform -chdir=infra/spire-aws fmt -check` | exit 0 |
| `terraform-validate.log` | `terraform -chdir=infra/spire-aws validate` | exit 0; `Success! The configuration is valid.` |
| `preflight-default-graviton-pass.log` | `scripts/spire-aws/preflight-operator.sh` with mocked AWS returning `arm64` and default instance types | exit 0 |
| `preflight-r6i-reject.log` | `scripts/spire-aws/preflight-operator.sh` with mocked AWS returning `arm64` and `r6i.4xlarge` coordinator | expected exit 2; r6i rejected |
| `git-diff-check.log` | `git diff --check` | exit 0 |

## Key Result Lines

- Default Graviton pass: `SPIRE AWS operator preflight passed: region=us-west-2 az=us-west-2a ami=ami-0123456789abcdef0 coordinator=m7g.large remote=m7g.large remote_count=3`
- r6i rejection: `ERROR: coordinator_instance_type must use the established Graviton/aarch64 lane, got: r6i.4xlarge`
- Terraform validate: `Success! The configuration is valid.`
