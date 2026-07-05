# Packet Manifest: Task 30 Packet 1066

## Metadata

- Packet: `reviews/task-30/1066-spire-phase13e-aws-operations-fault-restore`
- Head SHA: `6f11d0c8a0434e775403ff14120240e8c448e74d`
- Date: 2026-05-29
- Lane: AWS Graviton representative SPIRE operations fault restore
- Region/AZ: `us-west-2` / `us-west-2a`
- Instance shape: `m7g.large`
- Topology source: `reviews/task-30/1062-spire-phase13e-aws-representative-after-preserve-harness/artifacts/aws-topology.json`
- Coordinator: `i-0bb09286bba26190f`
- Remotes: `node_id=2 i-0b0e5ae9daea017d3`, `node_id=3 i-0a051a40c355ef8bc`, `node_id=4 i-07a6a4778337f20df`
- Prefix: `ec_spire_aws_repr_1m`
- Coordinator index: `ec_spire_aws_repr_1m_idx`
- Corpus/data setup: reused preserved packet 1062 real representative corpus placement; no provision, install, reload, or rebuild.
- Query fixture: packet 1062 real representative prepared queries, first query selected by production fault harness with `ORDER BY id LIMIT 1`.
- Storage format: representative AWS SPIRE remote tuple transport, `pg_binary_attr_v1`
- Rerank mode: default read-profile and smoke settings
- Isolated one-index-per-table surface: yes, `ec_spire_aws_repr_1m_idx`

## Code Commits

- `83c59a293 Add SPIRE AWS topology start helper`
- `6f11d0c8a Use first available SPIRE AWS fault query`

## Commands

### Start Preserved Topology

```bash
SPIRE_AWS_EXPECT_INSTANCE_TYPE=m7g.large \
SPIRE_AWS_EXPECT_AVAILABILITY_ZONE=us-west-2a \
scripts/spire-aws/start-topology-instances.sh \
  reviews/task-30/1062-spire-phase13e-aws-representative-after-preserve-harness/artifacts/aws-topology.json \
  reviews/task-30/1066-spire-phase13e-aws-operations-fault-restore/artifacts/rerun-after-query-vector-fix
```

### Fault Rerun

```bash
scripts/spire-aws/with-ssm-port-forwards.sh \
  reviews/task-30/1062-spire-phase13e-aws-representative-after-preserve-harness/artifacts/aws-topology.json \
  reviews/task-30/1066-spire-phase13e-aws-operations-fault-restore/artifacts/rerun-after-query-vector-fix \
  reviews/task-30/1066-spire-phase13e-aws-operations-fault-restore/artifacts/rerun-after-query-vector-fix/aws-topology.tunneled.json \
  -- make -C infra/spire-aws \
    ARTIFACT_DIR=/home/peter/dev/ecaz/reviews/task-30/1066-spire-phase13e-aws-operations-fault-restore/artifacts/rerun-after-query-vector-fix \
    TOPOLOGY=/home/peter/dev/ecaz/reviews/task-30/1066-spire-phase13e-aws-operations-fault-restore/artifacts/rerun-after-query-vector-fix/aws-topology.tunneled.json \
    WORK_DIR=/home/peter/dev/ecaz/reviews/task-30/1062-spire-phase13e-aws-representative-after-preserve-harness/artifacts/work \
    ECAZ_BIN=/home/peter/dev/ecaz/target/release/ecaz \
    PREFIX=ec_spire_aws_repr_1m \
    COORD_INDEX=ec_spire_aws_repr_1m_idx \
    smoke-representative fault-degraded fault-strict smoke-representative
```

The final `smoke-representative` target did not emit a fresh post-restore
smoke in the same Make invocation, so the post-restore smoke was captured
directly with the checked-in smoke helper:

```bash
scripts/spire-aws/with-ssm-port-forwards.sh \
  reviews/task-30/1062-spire-phase13e-aws-representative-after-preserve-harness/artifacts/aws-topology.json \
  reviews/task-30/1066-spire-phase13e-aws-operations-fault-restore/artifacts/rerun-after-query-vector-fix/post-restore-smoke \
  reviews/task-30/1066-spire-phase13e-aws-operations-fault-restore/artifacts/rerun-after-query-vector-fix/post-restore-smoke/aws-topology.tunneled.json \
  -- env \
    ECAZ_BIN=/home/peter/dev/ecaz/target/release/ecaz \
    PREFIX=ec_spire_aws_repr_1m \
    WORK_DIR=/home/peter/dev/ecaz/reviews/task-30/1062-spire-phase13e-aws-representative-after-preserve-harness/artifacts/work \
    scripts/spire-aws/smoke.sh \
    reviews/task-30/1066-spire-phase13e-aws-operations-fault-restore/artifacts/rerun-after-query-vector-fix/post-restore-smoke/aws-topology.tunneled.json \
    reviews/task-30/1066-spire-phase13e-aws-operations-fault-restore/artifacts/rerun-after-query-vector-fix/post-restore-smoke
```

### Stop And Verify

```bash
aws ec2 stop-instances --region us-west-2 \
  --instance-ids i-07a6a4778337f20df i-0b0e5ae9daea017d3 i-0bb09286bba26190f i-0a051a40c355ef8bc

aws ec2 wait instance-stopped --region us-west-2 \
  --instance-ids i-07a6a4778337f20df i-0b0e5ae9daea017d3 i-0bb09286bba26190f i-0a051a40c355ef8bc

aws ec2 describe-instances --region us-west-2 \
  --filters Name=instance-state-name,Values=pending,running,stopping \
  --query 'Reservations[].Instances[].[InstanceId,State.Name,InstanceType,Placement.AvailabilityZone,Tags[?Key==`Name`]|[0].Value]' \
  --output text
```

## Artifact Index

Root artifacts preserve the first failed attempt. That attempt failed before
the actual fault semantics because `scripts/spire-aws/fault.sh` assumed
`WHERE id = 0` for the representative query vector. The fixed and successful
attempt is under `artifacts/rerun-after-query-vector-fix/`.

- `aws-pre-start-active-instances.log`: no active instance rows before start.
- `start-topology-instance-state.log.before`: all four topology instances stopped before the successful rerun.
- `start-topology-instance-state.log.after`: all four topology instances running and status-ok after helper start.
- `fault-degraded-session-summary.log`: degraded production read profile.
- `fault-degraded-assertion.log`: contains `degraded_ok`.
- `fault-degraded.log`: degraded stop/restore transcript, including SQL readiness.
- `fault-strict-knn-strict.log`: strict failure output.
- `fault-strict.log`: strict stop/restore transcript, including fail-closed confirmation and SQL readiness.
- `post-restore-smoke/smoke-customscan-read.log`: final CustomScan smoke after both restores.
- `post-restore-smoke/production-read-profile-smoke.log`: final production read profile after both restores.
- `post-restore-smoke/bench-spire-pipeline-smoke.log`: final q=5 smoke latency/recall/profile run.
- `aws-stop-instances-after-success.log`: stop request output.
- `aws-wait-stopped-after-success.log`: AWS wait command transcript.
- `aws-stop-verify-after-success.log`: no active instance rows after stopping all `us-west-2` instances.

## Key Result Lines

### Degraded Fault

From `rerun-after-query-vector-fix/fault-degraded-session-summary.log`:

```text
consistency_mode	degraded
remote_heap_ready_dispatch_count	2
returned_candidate_count	10
result_source	remote_heap_candidates
status	degraded_ready
remote_timeout_count	0
remote_cancel_count	0
degraded_skipped_dispatch_count	1
```

From `rerun-after-query-vector-fix/fault-degraded-assertion.log`:

```text
degraded_ok
```

From `rerun-after-query-vector-fix/fault-degraded.log`:

```text
remote node 2 PostgreSQL restart ssm command id: f137e744-7174-441a-be72-b4e493c38b1d
remote node 2 SQL ready after 1 attempt(s)
```

### Strict Fault

From `rerun-after-query-vector-fix/fault-strict-knn-strict.log`:

```text
ERROR:  ec_spire remote write shape fingerprint failed to open connection for node_id 2
```

From `rerun-after-query-vector-fix/fault-strict.log`:

```text
strict fault drill failed closed as expected for node_id=2
remote node 2 PostgreSQL restart ssm command id: ba1e9e09-259f-43e9-8ce2-634008df2a7d
remote node 2 SQL ready after 1 attempt(s)
```

### Post-Restore Strict Smoke

From `rerun-after-query-vector-fix/post-restore-smoke/smoke-customscan-read.log`:

```text
Custom Scan (EcSpireDistributedScan)
remote_fanout: 3
consistency_mode	strict
remote_heap_ready_dispatch_count	3
returned_candidate_count	10
result_source	remote_heap_candidates
status	ready
remote_timeout_count	0
remote_cancel_count	0
degraded_skipped_dispatch_count	0
```

From `rerun-after-query-vector-fix/post-restore-smoke/bench-spire-pipeline-smoke.log`:

```text
nprobe  queries  latency_p50  recall@k
8       5        100.588 ms   0.7600
16      5        110.329 ms   0.8200
32      5        149.189 ms   0.9200
```

Production profile rows in the same smoke report `status=ready`,
`result_source=remote_heap_candidates`, `socket_open_sum=0`, and
`degraded_skip_sum=0` for nprobe `8`, `16`, and `32`.

### AWS Shutdown

`rerun-after-query-vector-fix/aws-stop-verify-after-success.log` contains only
the `script` wrapper start/done lines and no instance rows, which records no
`pending`, `running`, or `stopping` instances in `us-west-2` after the stop and
wait commands.
