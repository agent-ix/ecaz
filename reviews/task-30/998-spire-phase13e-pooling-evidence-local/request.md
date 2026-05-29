# Review Request: Local SPIRE Pooling Evidence

## Summary

This checkpoint addresses the reviewer 992 request for explicit with/without
pooling evidence.

`scripts/run_spire_phase13e_aws_harness_local_pg18.sh` now records a persistent
coordinator-backend pooling probe after the normal local AWS-shape smoke run:

- pool disabled for two reads;
- pool enabled warmup plus two follow-up reads;
- one remote stopped while the same coordinator backend keeps its pool;
- degraded read skips the stopped remote;
- remote restarted and strict read verifies only that failed remote connection
  is reopened.

## Code

- Commit: `ea6a2a6f8aead915ca78cb0aada63e340f58ff5b`
- `scripts/run_spire_phase13e_aws_harness_local_pg18.sh`
  - Adds `pooling-socket-open-comparison.tsv`.
  - Asserts:
    - disabled pooling: `disabled_socket_sum=6` across two reads;
    - pooled follow-ups: `pooled_followup_socket_sum=0`;
    - remote-down degraded read: `degraded_status=degraded_ready`, `degraded_skipped=1`;
    - post-restart read: `after_restart_socket=1`, proving the failed remote connection was dropped rather than returned to the idle pool.
- `plan/tasks/task30-phase13e-spire-aws-production-gap-closure.md`
  - Records packet 998 as the local proof for socket-open reduction and failed-connection drop.

## Evidence

- `artifacts/phase13e-aws-harness-local.log`
  - `HARNESS PASSED`
  - `pooling_socket_open_comparison=disabled_rows=2|disabled_socket_sum=6|pooled_warmup_socket=3|pooled_followup_rows=2|pooled_followup_socket_sum=0|degraded_status=degraded_ready|degraded_socket=0|degraded_skipped=1|after_restart_status=ready|after_restart_socket=1|bad=0`
- `artifacts/pooling-socket-open-summary.tsv`
  - Explicit per-read rows for disabled, warmup, pooled follow-up, remote-down degraded, and post-restart reads.
- `artifacts/bench-spire-pipeline-smoke.log`
  - Still reports p50/p95/p99 query latency and recall in the local AWS-shape harness.

No AWS provisioning was run for this packet. Post-run checks showed no Phase 13
EC2 instances and clean local Terraform state.
