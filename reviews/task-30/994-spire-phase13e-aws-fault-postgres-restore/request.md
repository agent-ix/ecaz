# Review Request: AWS Fault Restore PostgreSQL Restart

Task: `plan/tasks/task30-phase13e-spire-aws-production-gap-closure.md`

Code commit: `256f1e99bc37a6b1a3fea33e1a162ae8ff0d0881`

## Summary

The Graviton synthetic correctness run reached production-read and degraded-fault execution, then failed during fault restore because the stopped remote EC2 instance returned to `running`/SSM-tunnel-ready but PostgreSQL never accepted SQL again on the restored node.

This change makes `scripts/spire-aws/fault.sh` deterministic after EC2 stop/start:

- waits for the target remote to become SSM-online after `aws ec2 start-instances`;
- sends an `AWS-RunShellScript` command to start/restart the node PostgreSQL service;
- records the SSM invocation at `fault-<drill>-remote-<node>-postgres-restart.json`;
- restarts the local SSM port forward and then waits for SQL readiness as before.

## Evidence

- `artifacts/aws-fault-restore-failure-summary.md`: failure being addressed.
- `artifacts/preflight.log`: `make -C infra/spire-aws preflight` passed after the change.

## Notes

No new AWS provisioning run was started for this packet. The previous failed run was torn down first; post-teardown checks showed no active Phase 13 instances and empty Terraform state.
