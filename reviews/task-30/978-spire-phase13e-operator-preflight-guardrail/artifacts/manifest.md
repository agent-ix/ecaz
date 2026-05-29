# Artifact Manifest: SPIRE Phase 13e Operator Preflight Guardrail

Head SHA: `eb87ae23ed1693d1816acb2ba892d1c609592e2b`
Task bucket: `reviews/task-30/978-spire-phase13e-operator-preflight-guardrail`
Timestamp: `2026-05-25T21:41:14Z`
Lane: local operator-surface validation only
AWS resources provisioned: no
Storage format / rerank mode: not applicable
Surface isolation: not applicable; no benchmark tables were created

## Artifacts

| Artifact | Command | Result |
| --- | --- | --- |
| `bash-syntax.log` | `bash -n scripts/spire-aws/preflight-operator.sh` | exit 0 |
| `preflight-missing-tfvars.log` | `make -C infra/spire-aws preflight-operator` with no local `terraform.tfvars` | expected exit 2; missing tfvars error |
| `preflight-graviton-pass.log` | `scripts/spire-aws/preflight-operator.sh` with mocked AWS returning `arm64` and Graviton tfvars | exit 0 |
| `preflight-r6i-reject.log` | `scripts/spire-aws/preflight-operator.sh` with mocked AWS returning `arm64` and `r6i.4xlarge` coordinator | expected exit 2; r6i rejected |
| `git-diff-check.log` | `git diff --check` | exit 0 |

## Key Result Lines

- Missing tfvars: `ERROR: missing /home/peter/dev/ecaz/infra/spire-aws/terraform.tfvars; create it from infra/spire-aws/terraform.tfvars.example before provisioning`
- Graviton pass: `SPIRE AWS operator preflight passed: region=us-west-2 az=us-west-2a ami=ami-0123456789abcdef0 coordinator=r7g.4xlarge remote=r7g.2xlarge remote_count=3`
- r6i rejection: `ERROR: coordinator_instance_type must use the established Graviton/aarch64 lane, got: r6i.4xlarge`
