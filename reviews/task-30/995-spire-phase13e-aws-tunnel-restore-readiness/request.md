# Review Request: AWS Tunnel Restore Readiness

Task: `plan/tasks/task30-phase13e-spire-aws-production-gap-closure.md`

Code commit: `de3d909dfccf423334444fed707ac49d4198e91d`

## Summary

The Graviton synthetic correctness rerun proved the SPIRE data path through remote placement, distributed CustomScan, remote heap tuple return, pooled TLS connections, recall/latency sweeps, and degraded fault behavior. It still failed while restoring the stopped remote because the operator-side SSM port forward was declared ready on a weak raw TCP probe, then the first SQL probe closed the session and later probes saw `Connection refused`.

This change hardens the AWS fault harness:

- `restart-ssm-port-forward.sh` now waits for Session Manager's own `Port <port> opened for sessionId` log line instead of opening a raw TCP connection as the readiness probe.
- `fault.sh` now restarts the operator tunnel inside each restored-node SQL readiness attempt, so a dead local port-forward is replaced before the next `SELECT 1` probe.

No SPIRE core code changed.

## Evidence

- `artifacts/aws-tunnel-restore-failure-summary.md`: summarizes the failed restore and the SPIRE functionality that had already passed before the harness failure.
- `artifacts/preflight.log`: `make -C infra/spire-aws preflight` passed after the change.

## Notes

AWS was torn down before this change. Post-run cleanup checks returned no active Phase 13 instances and Terraform state had no managed resources.
