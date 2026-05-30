# Task 67 015 Intel Cloud Profile Artifacts

Head SHA: `4b14e27465310f4fd0c11b38513877796aa72f5d`
Task bucket: `reviews/task-67/015-intel-cloud-profile`
Timestamp: `2026-05-30T02:38:39Z`

## Artifact: validation.log

- Lane: Task 67 Intel cloud profile prep.
- Fixture: none; infrastructure/profile registry only.
- Storage format: none.
- Rerank mode: none.
- Isolated one-index-per-table or shared-table surfaces: not applicable.
- Commands:
  - `cargo fmt`
  - `terraform fmt infra/cloud/terraform/profiles/10k-intel.tfvars infra/cloud/terraform/variables.tf`
  - `cargo test -p ecaz-cloud profiles -- --nocapture`
  - `cargo build -p ecaz-cli`
  - `git diff --check`
  - `aws ec2 describe-instance-types --region us-west-2 --instance-types m7i.2xlarge c7i.large --query 'InstanceTypes[].{Type:InstanceType,VCPU:VCpuInfo.DefaultVCpus,MemoryMiB:MemoryInfo.SizeInMiB,Arch:ProcessorInfo.SupportedArchitectures,Clock:ProcessorInfo.SustainedClockSpeedInGhz,Manufacturer:ProcessorInfo.Manufacturer}' --output json`
  - `terraform -chdir=infra/cloud/terraform validate`
  - `target/debug/ecaz cloud up --profile 10k-intel --git-ref 4b14e27465310f4fd0c11b38513877796aa72f5d --dry-run`
- Key result lines:
  - `cargo test -p ecaz-cloud profiles -- --nocapture`: `2 passed; 0 failed`.
  - `cargo build -p ecaz-cli`: passed with one existing dead-code warning.
  - `git diff --check`: passed.
  - AWS metadata: `c7i.large` and `m7i.2xlarge` are Intel `x86_64` in `us-west-2`.
  - `terraform validate`: blocked by local provider plugin schema startup failure before module validation.
  - `ecaz cloud up --profile 10k-intel --dry-run`: Terraform initialized successfully and planned `26 to add, 0 to change, 0 to destroy` with `m7i.2xlarge` DB and `c7i.large` loader.

## External Metadata Sources

- AWS CLI `describe-instance-types` output is captured in `validation.log`.
- AWS public instance pages checked during profile selection:
  - `https://aws.amazon.com/ec2/instance-types/c7i/`
  - `https://aws.amazon.com/ec2/instance-types/m7i/`
