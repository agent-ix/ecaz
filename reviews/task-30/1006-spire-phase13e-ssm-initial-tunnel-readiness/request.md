# Review Request: SPIRE AWS Initial SSM Tunnel Readiness

## Scope

This packet covers commit `1f243bd59b15df4f5aeae35afa09bb463947ae66`, which
uses Session Manager's own opened-port log line as the readiness signal for
initial SPIRE AWS SSM PostgreSQL tunnels.

This closes a mismatch between the Phase 13e task note and checked-in code:
`restart-ssm-port-forward.sh` already waited for
`Port <port> opened for sessionId`, but `with-ssm-port-forwards.sh` still used a
raw `/dev/tcp` probe.

## Change Summary

- Added `scripts/spire-aws/wait-for-ssm-port-forward-ready.sh`.
- Changed `with-ssm-port-forwards.sh` to wait for the shared opened-port log
  readiness helper for coordinator and remote tunnels.
- Changed `restart-ssm-port-forward.sh` to use the same helper.

## Validation

No AWS provisioning, Terraform apply, Terraform destroy, EC2 start, real SSM
session, or PostgreSQL cluster was used for this packet.

- `bash -n scripts/spire-aws/wait-for-ssm-port-forward-ready.sh scripts/spire-aws/with-ssm-port-forwards.sh scripts/spire-aws/restart-ssm-port-forward.sh`
  - artifact: `artifacts/bash-n-ssm-tunnel.log`
  - result: exit 0
- fake opened-port helper success
  - artifact: `artifacts/wait-helper-fake-ready.log`
  - result: exit 0 after the fake log writes `Port 15432 opened for sessionId`
- fake opened-port helper timeout
  - artifact: `artifacts/wait-helper-fake-timeout.log`
  - result: exit 1 when no opened-port log line appears
- full wrapper with fake `aws ssm start-session`
  - artifact: `artifacts/with-ssm-fake-wrapper-success.log`
  - result: exit 0 after coordinator and two remotes report opened-port readiness
  - fake tunnel logs: `artifacts/fake-wrapper-run-success/tunnel-*.log`
- `make -C infra/spire-aws preflight`
  - artifact: `artifacts/preflight.log`
  - result: exit 0

## Remaining Phase 13e Work

The next required proof is still the explicitly approved Graviton representative
performance pass: p50/p95/p99 latency, recall, production read profile, and
pooling A/B deltas.
