# Manifest: Task 30 / 1045 SPIRE Phase 13e Single AWS Runtime Build

- head SHA: `170b1e4b3e0a41e0851498fee02d894b7da8386c`
- task bucket: `reviews/task-30`
- packet path: `reviews/task-30/1045-spire-phase13e-single-aws-runtime-build`
- timestamp: `2026-05-28T05:07:27Z`
- lane: local static/mock validation for AWS representative install harness
- fixture: mocked AWS/openssl, 1 coordinator + 3 remotes
- storage format: n/a
- rerank mode: n/a
- isolated one-index-per-table or shared-table surface: n/a

## Artifacts

### `artifacts/bash-n-spire-aws.log`

- command: `script -q -e -c "bash -n scripts/spire-aws/*.sh" reviews/task-30/1045-spire-phase13e-single-aws-runtime-build/artifacts/bash-n-spire-aws.log`
- result: passed
- key result: command exited successfully.

### `artifacts/install-runtime-selfcheck.log`

- command: `script -q -e -c "scripts/spire-aws/check-install-parallel-local.sh" reviews/task-30/1045-spire-phase13e-single-aws-runtime-build/artifacts/install-runtime-selfcheck.log`
- result: passed
- key result lines:
  - `install-mode i-coord source=ecaz-source.tar.gz runtime=ecaz-runtime-linux-aarch64.tar.gz build=1 wait=0`
  - `install-mode i-remote-1 source= runtime=ecaz-runtime-linux-aarch64.tar.gz build=0 wait=1`
  - `install-mode i-remote-2 source= runtime=ecaz-runtime-linux-aarch64.tar.gz build=0 wait=1`
  - `install-mode i-remote-3 source= runtime=ecaz-runtime-linux-aarch64.tar.gz build=0 wait=1`
  - `SPIRE AWS install parallel self-check passed: install_sends=4 install_waits=4`

### `artifacts/aws-running-check.log`

- command: `script -q -e -c "aws ec2 describe-instances --region us-west-2 --filters Name=instance-state-name,Values=pending,running,stopping --query 'Reservations[].Instances[].[InstanceId,State.Name,InstanceType,PrivateIpAddress,Tags[?Key==\`Name\`].Value|[0]]' --output text" reviews/task-30/1045-spire-phase13e-single-aws-runtime-build/artifacts/aws-running-check.log`
- result: passed
- key result: no pending/running/stopping instances were returned.
