# Task 67 Review Request: Intel AMI Selection

## Scope

This packet fixes the `10k-intel` cloud lane added in packet 015. The first
dry-run plan selected the existing AL2023 arm64 AMI for Intel `m7i` / `c7i`
instances, which would fail at launch. Terraform now selects the AL2023 AMI
architecture from profile tfvars.

Code commit:

- `4485f58be6ccf3db50c85b00ed726031d5f7c8f3`

## Changes

- Replaced the hard-coded `data.aws_ami.al2023_arm64` lookup with an
  architecture-parameterized `data.aws_ami.al2023` lookup.
- Added `instance_architecture` Terraform variable with validation for
  `arm64` / `x86_64`.
- Set `instance_architecture = "x86_64"` in `10k-intel.tfvars`.
- Existing Graviton profiles keep the default `arm64` behavior.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.log`.

- `terraform fmt` passed for changed Terraform files.
- `git diff --check` passed.
- `target/debug/ecaz cloud up --profile 10k-intel --git-ref 67f59264b --dry-run`
  passed and selected x86_64 AL2023 AMI `ami-029a761f237195c2c` for
  `m7i.2xlarge` and `c7i.large`.
- `target/debug/ecaz cloud up --profile 10k --git-ref 67f59264b --dry-run`
  passed and still selected arm64 AL2023 AMI `ami-0a2a049c945b84826` for
  Graviton instances.

## Remaining Task 67 Work

This packet still does not launch AWS resources. With the Intel lane now
planning a compatible AMI, the next step is to provision `10k-intel`, verify
runtime CPU flags on the DB host, and run the Slice J suite.
