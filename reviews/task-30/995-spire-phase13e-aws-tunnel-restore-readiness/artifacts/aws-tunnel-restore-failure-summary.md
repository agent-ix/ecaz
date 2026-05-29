# AWS Tunnel Restore Failure Summary

Run artifact root:

`reviews/task-30/994-spire-phase13e-aws-fault-postgres-restore/artifacts/aws-correctness-after-postgres-restore-fix`

Command:

`SPIRE_AWS_ALLOW_PREEXISTING_RESIDUE=1 make -C infra/spire-aws pass-correctness ARTIFACT_DIR=reviews/task-30/994-spire-phase13e-aws-fault-postgres-restore/artifacts/aws-correctness-after-postgres-restore-fix`

## Passed Before Failure

- Topology: `us-west-2a`, Graviton `m7g.large`, 1 coordinator, 3 remotes.
- Remote materialization completed: node 2 had 3295 rows, node 3 had 3317 rows, node 4 had 3388 rows.
- Published static placements: local node 0 had 1 leaf; remote nodes 2/3/4 had 34/33/33 leaves.
- CustomScan smoke used `EcSpireDistributedScan`, `remote_fanout: 3`, `tuple_transport_status: ready`.
- Production read smoke returned remote heap rows: `remote_pid_count=10`, `dispatch_count=3`, `returned_candidate_count=10`, `result_source=remote_heap_candidates`, `status=ready`.
- Warm production profile showed pooled connections: `socket_open_sum=0`.
- Recall@10 over 100 queries completed: nprobe 8 `0.2400`, 16 `0.3530`, 24 `0.4480`, 32 `0.5180`.
- Latency sweep completed over 200 iterations per nprobe: p50/p95 were approximately 73.3/80.9ms, 73.9/81.7ms, 76.5/86.0ms, 75.8/81.9ms for nprobe 8/16/24/32.
- Production profile over 100 profiles per nprobe completed: all rows `status=ready`, `result_source=remote_heap_candidates`, `returned_sum=1000`, `socket_open_sum=0`.
- Degraded fault passed while remote node 2 was stopped: `status=degraded_ready`, `remote_heap_ready_dispatch_count=2`, `degraded_skipped_dispatch_count=1`, `returned_candidate_count=10`.

## Failure

The restored remote node reached EC2 running state and SSM online state. The PostgreSQL restart command on the remote returned success with stdout:

```text
active
1
```

That means the remote PostgreSQL service was active and `SELECT 1` succeeded on the node itself.

The operator-side local SSM tunnel restore then failed:

- `restart-ssm-port-forward.sh` reported `tunnel remote-2 restarted on 127.0.0.1:15433 after 1 attempt(s)`.
- First SQL readiness probe failed with `server closed the connection unexpectedly`.
- Later SQL readiness probes failed with `Connection refused`.
- The harness timed out with `remote node 2 SQL did not become ready within 300s`.
- The subsequent strict-mode restoration failed against `127.0.0.1:15433`.

## Cleanup

The run's trap destroyed all 33 Terraform-managed resources. Follow-up checks after teardown:

- `aws ec2 describe-instances ...` returned `[]` for active Phase 13 instances.
- `make -C infra/spire-aws preflight-state` passed with no managed resources in local Terraform state.
