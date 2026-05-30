# Task 67 016 Intel AMI Selection Artifacts

Head SHA: `4485f58be6ccf3db50c85b00ed726031d5f7c8f3`
Task bucket: `reviews/task-67/016-intel-ami-selection`
Timestamp: `2026-05-30T02:45:27Z`

## Artifact: validation.log

- Lane: Task 67 Intel cloud profile prep.
- Fixture: none; infrastructure/profile registry only.
- Storage format: none.
- Rerank mode: none.
- Isolated one-index-per-table or shared-table surfaces: not applicable.
- Commands:
  - `terraform fmt infra/cloud/terraform/main.tf infra/cloud/terraform/variables.tf infra/cloud/terraform/profiles/10k-intel.tfvars`
  - `git diff --check`
  - `target/debug/ecaz cloud up --profile 10k-intel --git-ref 67f59264b --dry-run`
  - `target/debug/ecaz cloud up --profile 10k --git-ref 67f59264b --dry-run`
- Key result lines:
  - `git diff --check`: passed.
  - Intel dry-run: `data.aws_ami.al2023: Read complete after 1s [id=ami-029a761f237195c2c]`; DB `instance_type = "m7i.2xlarge"`; loader `instance_type = "c7i.large"`.
  - Graviton dry-run: `data.aws_ami.al2023: Read complete after 0s [id=ami-0a2a049c945b84826]`; DB `instance_type = "m8g.large"`; loader `instance_type = "c8g.medium"`.
