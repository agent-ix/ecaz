# Task 67 Review Request: Intel Cloud Profile

## Scope

This packet adds a first-class `10k-intel` cloud profile so Task 67 Slice J can
run the checked-in Intel measurement suite on an x86_64 Intel AWS host through
the existing `ecaz cloud bench` path.

Code commit:

- `4b14e27465310f4fd0c11b38513877796aa72f5d`

## Changes

- Added `Profile::P10kIntel` to `ecaz-cloud` with:
  - DB host: `m7i.2xlarge`
  - loader host: `c7i.large`
  - DB volume: 100 GB
- Added `infra/cloud/terraform/profiles/10k-intel.tfvars`.
- Updated Terraform and cloud README wording so the cloud harness is no longer
  documented as Graviton-only.
- Added unit tests covering profile parsing and the Intel host selection.

AWS metadata checked for `us-west-2` reports both selected instance types as
Intel `x86_64`. AWS public pages identify C7i/M7i as 4th Gen Intel Xeon
Scalable / Sapphire Rapids families. The actual AVX-512 feature set remains a
runtime validation requirement for the Slice J measurement packet.

## Validation

See `artifacts/manifest.md` and `artifacts/validation.log`.

- `cargo fmt` passed with the existing stable-rustfmt warnings.
- `terraform fmt` passed for the changed Terraform files.
- `cargo test -p ecaz-cloud profiles -- --nocapture` passed: 2 tests.
- `git diff --check` passed.
- `aws ec2 describe-instance-types ...` confirmed the selected instance
  metadata in `us-west-2`.
- `terraform -chdir=infra/cloud/terraform validate` was attempted but failed
  before evaluating this change because local provider plugins failed to start.

## Remaining Task 67 Work

This packet does not launch AWS resources. The next Slice J step is to provision
or reuse the `10k-intel` lane, run `reviews/task-67/010-intel-measurement-suite/artifacts/task67-intel-suite.json`
without `--dry-run`, and publish the final measurement packet with recall
deltas and throughput ratios.
