# Manifest

- Head SHA: `8b18e2ee0defa6d04ebc41e7a385c1178d9b2026`
- Branch: `diskann-aws-optimization`
- Task bucket: `reviews/task-30/1064-spire-phase13e-aws-representative-performance-suite`
- Timestamp: `2026-05-28T21:32:59-07:00`
- Lane: Phase 13e AWS representative performance suite, interrupted for nightly shutdown
- Fixture: `ec_spire_aws_repr_1m`, preserved packet 1062 representative corpus/index
- Topology: preserved packet 1062 Graviton cluster, 1 coordinator + 3 remotes, `m7g.large`, `us-west-2a`
- Storage format / rerank mode: existing representative SPIRE surface, production read profile smoke only in this packet
- Isolated/shared: existing packet 1062 representative SPIRE prefix/index under `ec_spire_aws_repr_1m`

## Cluster State

At shutdown verification time all four instances were stopped:

| role | instance_id | private_ip | instance_type | az | state |
| --- | --- | --- | --- | --- | --- |
| coordinator | `i-0bb09286bba26190f` | `10.42.1.75` | `m7g.large` | `us-west-2a` | `stopped` |
| remote-2 | `i-0b0e5ae9daea017d3` | `10.42.1.159` | `m7g.large` | `us-west-2a` | `stopped` |
| remote-3 | `i-0a051a40c355ef8bc` | `10.42.1.248` | `m7g.large` | `us-west-2a` | `stopped` |
| remote-4 | `i-07a6a4778337f20df` | `10.42.1.99` | `m7g.large` | `us-west-2a` | `stopped` |

Process verification after shutdown found no `with-ssm-port-forwards`, `session-manager-plugin`, `aws ssm start-session`, `bench suite`, `bench recall`, `bench spire-pipeline`, or `ecaz cloud down` process remaining.

## Attempt Boundary

This is not the final representative performance proof. The run started through the established tunnel wrapper and Make targets:

```bash
scripts/spire-aws/with-ssm-port-forwards.sh \
  reviews/task-30/1062-spire-phase13e-aws-representative-after-preserve-harness/artifacts/aws-topology.json \
  reviews/task-30/1064-spire-phase13e-aws-representative-performance-suite/artifacts \
  reviews/task-30/1064-spire-phase13e-aws-representative-performance-suite/artifacts/aws-topology.tunneled.json \
  -- make -C infra/spire-aws \
    ARTIFACT_DIR=/home/peter/dev/ecaz/reviews/task-30/1064-spire-phase13e-aws-representative-performance-suite/artifacts \
    TOPOLOGY=/home/peter/dev/ecaz/reviews/task-30/1064-spire-phase13e-aws-representative-performance-suite/artifacts/aws-topology.tunneled.json \
    WORK_DIR=/home/peter/dev/ecaz/reviews/task-30/1062-spire-phase13e-aws-representative-after-preserve-harness/artifacts/work \
    ECAZ_BIN=/home/peter/dev/ecaz/target/release/ecaz \
    PREFIX=ec_spire_aws_repr_1m \
    COORD_INDEX=ec_spire_aws_repr_1m_idx \
    smoke-representative bench-representative-priority bench-representative-pooling
```

The operator requested shutdown shortly after the suite entered `13a3a-recall-k10`. The process group was terminated and all running `us-west-2` EC2 instances were stopped.

The suite manifest shows all selected representative-priority steps still `pending`; no representative latency/recall/pooling acceptance row completed in this packet.

## Artifacts

| artifact | description |
| --- | --- |
| `aws-topology.tunneled.json` | Tunneling topology generated before shutdown |
| `smoke-customscan-read.log` | Representative smoke EXPLAIN and production read profile evidence |
| `production-read-profile-smoke.log` | Smoke production profile output |
| `bench-spire-pipeline-smoke.log` | q=5 smoke `ecaz bench spire-pipeline` output |
| `suite-representative-priority.json` | Rendered representative-priority suite using the packet-local artifact directory and packet 1062 truth corpus |
| `suite-manifest-representative-priority.json` | Suite manifest; all selected steps remained pending after interruption |
| `truth-cache/13a3a-recall-k10.json` | Truth cache generated before the interruption |
| `tunnel-*.log` | SSM port-forward setup logs |

## Key Lines

- `smoke-customscan-read.log` contains `Custom Scan (EcSpireDistributedScan)`.
- `smoke-customscan-read.log` and `production-read-profile-smoke.log` report `result_source remote_heap_candidates`.
- `bench-spire-pipeline-smoke.log` q=5 rows report `socket_open_sum=0`, `connect_p50=0.000 ms`, `timeout_sum=0`, `cancel_sum=0`, and `degraded_skip_sum=0`.
- `suite-manifest-representative-priority.json` contains the expected representative suite commands, but no completed suite status rows.

## Next Resume Point

The next representative proof should start a fresh packet or rerun this packet deliberately after starting the stopped Graviton instances. The required acceptance evidence remains:

- `bench-representative-priority`
- `bench-representative-pooling`
- `summarize-representative-performance`
- `verify-representative-performance-summary`
