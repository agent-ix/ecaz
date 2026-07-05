# Task 30 Review Request: AWS Representative Rerun After Staging Fix

## Summary

This packet records the AWS representative rerun after the `/var/tmp` node-local staging fix. The run used the established Graviton/aarch64 lane:

- Region/AZ: `us-west-2` / `us-west-2a`
- Coordinator: `m7g.large`, `i-0816a4ff07f1d6216`
- Remotes: three `m7g.large` nodes, `i-0d5d91e976761d37c`, `i-0b44498ba4bf6c196`, `i-01df94dd720d0b078`
- Coordinator storage: 200 GB root volume
- Remote storage: 100 GB root volumes

The previous disk-space failure did not recur. The coordinator node-local SSM output showed `/dev/nvme0n1p1 200G ... 196G` available before downloading the 2.0 GiB corpus.

The run then failed in node-local reset SQL because `load.sh` used `ecaz dev sql` inside the EC2 node. That CLI path requires a `.pgrx` home that exists on the operator host but not on the AWS node:

```text
reading pgrx home .pgrx
No such file or directory (os error 2)
```

The harness teardown completed cleanly at `2026-05-28T03:43:46Z`, and direct EC2 verification after teardown returned no pending/running/stopping/stopped instances.

## Evidence

- `artifacts/run-representative-performance-pass-rerun.log`: full AWS pass transcript.
- `artifacts/coordinator-load-representative.ssm.json`: failed node-local coordinator load invocation.
- `artifacts/coordinator-load-representative-error.log`: extracted root error.
- `artifacts/ec2-post-teardown-verify.log`: direct post-teardown EC2 check.
- `artifacts/aws-pass-watchdog.log`: watchdog and teardown record.

## Follow-Up

The follow-up fix is packet `1042-spire-phase13e-node-local-psql-reset-fix`: use local PostgreSQL `psql` for node-local reset/drop SQL and keep `ecaz` for corpus load/inspect.
