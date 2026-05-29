# Review Request: Phase 13e AWS SSM Tunnel Fault Restore

## Scope

Commit under review: `d4c46b3574ee013ce42b29891bb9918eca1fa53d`

This slice fixes the AWS fault-drill harness after the synthetic correctness run exposed an operator lifecycle gap: stopping a remote EC2 instance kills the local SSM port forward, but the previous wrapper kept waiting on the dead local port after the instance restarted.

Changes:

- `scripts/spire-aws/with-ssm-port-forwards.sh`
  - records tunnel PIDs in a packet-local tunnel state directory.
  - exports tunnel restart metadata for child fault drills.
  - cleans up tunnels started by both the original wrapper and child restarts.
- `scripts/spire-aws/restart-ssm-port-forward.sh`
  - new repeatable operator helper for restarting one SSM PostgreSQL port forward.
  - retries session creation until the tunnel readiness timeout, covering the post-EC2-start window before SSM is ready.
- `scripts/spire-aws/fault.sh`
  - restarts the affected remote tunnel after `aws ec2 wait instance-running` and before SQL readiness.
  - marks the remote as no longer stopped before the readiness phase so the exit trap does not repeat the EC2 restart loop after a readiness failure.

## Validation

Artifacts are in `reviews/task-30/993-spire-phase13e-aws-correctness-after-pooling/artifacts/`.

- `preflight-after-tunnel-restart-fix.log`
  - `make -C infra/spire-aws preflight` passed.
  - Terraform fmt/init/validate passed.
  - `bash -n scripts/spire-aws/*.sh` passed.
  - Suite JSON validation passed.
- `aws-harness-local-after-tunnel-restart-fix/phase13e-aws-harness-local.log`
  - local PG18 AWS-harness fixture passed, no AWS used.
  - synthetic coordinator + 3 remotes loaded and registered.
  - CustomScan showed `remote_fanout: 3` and `tuple_transport_status: ready`.
  - production read returned `result_source = remote_heap_candidates`, `status = ready`.
  - local degraded and strict fault drills completed.

## Notes

No AWS run was started after this fix. The next AWS run should remain gated on this committed harness change plus the existing clean-state/preflight checks.
