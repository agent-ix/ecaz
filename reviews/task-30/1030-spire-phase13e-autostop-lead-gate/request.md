# Review Request: SPIRE AWS Auto-Stop Lead Gate

Task: `plan/tasks/task30-phase13e-spire-aws-production-gap-closure.md`

Code commit: `2a9435ccb`

## Summary

This checkpoint closes a remaining pre-provision safety gap for the Phase 13e
representative AWS run. `preflight-operator.sh` already rejected expired
`auto_stop_at` values, but it still allowed a run to start with only a few
minutes of auto-stop lead time. That is too weak for the representative pass,
whose watchdog budget is four hours.

The operator preflight now requires `auto_stop_at` to be at least `18000`
seconds after preflight time. This is the four-hour representative watchdog
budget plus buffer. The current Graviton `terraform.tfvars` passes this guard.

No AWS provisioning was started.

## Validation

- `bash -n scripts/spire-aws/preflight-operator.sh`
- good lead fixture with packet-local fake AWS image lookup
- short lead fixture with packet-local fake AWS image lookup
  - expected exit 2
  - rejected before the AWS image lookup could matter
- current `infra/spire-aws/terraform.tfvars`
  - passed against real read-only AMI architecture lookup
- `git diff --check HEAD^ HEAD -- scripts/spire-aws/preflight-operator.sh plan/tasks/task30-phase13e-spire-aws-production-gap-closure.md`

## Artifacts

- `artifacts/manifest.md`
- `artifacts/bash-n-preflight-operator.log`
- `artifacts/preflight-good-lead.log`
- `artifacts/preflight-short-lead.log`
- `artifacts/preflight-current-tfvars.log`
- `artifacts/git-show-stat.log`
- `artifacts/git-diff-check.log`
- `artifacts/fake-bin/aws`
- `artifacts/tfvars/good-lead.tfvars`
- `artifacts/tfvars/short-lead.tfvars`

The existing untracked SPIRE artifact directories were left untouched and were
not staged.
