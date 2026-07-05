# Manifest

- Head SHA: `cdcfa3e211ef4676e9147279af71c4e31a95346a`
- Branch: `diskann-aws-optimization`
- Task bucket: `reviews/task-30/1063-spire-phase13e-aws-pooling-comparison-q20`
- Timestamp: `2026-05-28T21:18:55-07:00`
- Lane: Phase 13e AWS representative read profile, preserved packet 1062 cluster
- Fixture: `ec_spire_aws_repr_1m`, q=20, `top_k=10`, sweep `8,16,24,32`
- Storage format / rerank mode: existing packet 1062 SPIRE representative corpus and index; production read profile, production-read-only
- Surface: distributed remote placements on preserved 1 coordinator + 3 remote Graviton EC2 nodes
- Isolated/shared: existing packet 1062 representative SPIRE surface, one prefix/index under `ec_spire_aws_repr_1m`

## Cluster

| role | instance_id | private_ip | instance_type | az |
| --- | --- | --- | --- | --- |
| coordinator | `i-0bb09286bba26190f` | `10.42.1.75` | `m7g.large` | `us-west-2a` |
| remote-2 | `i-0b0e5ae9daea017d3` | `10.42.1.159` | `m7g.large` | `us-west-2a` |
| remote-3 | `i-0a051a40c355ef8bc` | `10.42.1.248` | `m7g.large` | `us-west-2a` |
| remote-4 | `i-07a6a4778337f20df` | `10.42.1.99` | `m7g.large` | `us-west-2a` |

## Artifacts

| artifact | description |
| --- | --- |
| `debug-production-read-profile-q20-pool-off.log` | Pool disabled baseline with `PGOPTIONS="-c ec_spire.remote_search_connection_pool_size=0"` |
| `debug-production-read-profile-q20-pool-on.log` | Pool enabled/default probe, pool size default 16 |
| `pooling-q20-delta-summary.tsv` | Three-row TSV of pool-off minus pool-on deltas for `socket_open_sum`, `connect_p50`, and `total_p50` |
| `aws-topology.tunneled.pool-off.json` | Operator tunnel topology for the pool-off run |
| `aws-topology.tunneled.pool-on.json` | Operator tunnel topology for the pool-on run |
| `tunnel-*.log` | SSM port-forward setup logs from the latest probe |

## Commands

Pool off:

```bash
scripts/spire-aws/with-ssm-port-forwards.sh \
  reviews/task-30/1062-spire-phase13e-aws-representative-after-preserve-harness/artifacts/aws-topology.json \
  reviews/task-30/1063-spire-phase13e-aws-pooling-comparison-q20/artifacts \
  reviews/task-30/1063-spire-phase13e-aws-pooling-comparison-q20/artifacts/aws-topology.tunneled.pool-off.json \
  -- env PGOPTIONS="-c ec_spire.remote_search_connection_pool_size=0" \
  /home/peter/dev/ecaz/target/release/ecaz \
  --database postgres --host 127.0.0.1 --port 15432 --user ecaz_coord \
  bench spire-pipeline \
  --prefix ec_spire_aws_repr_1m \
  --queries-limit 20 \
  --sweep 8,16,24,32 \
  --include-remote --require-remote-placements \
  --top-k 10 \
  --include-query-metrics --include-recall \
  --truth-corpus-file reviews/task-30/1062-spire-phase13e-aws-representative-after-preserve-harness/artifacts/work/qdrant-dbpedia/prepared/ec_real_100k_corpus.tsv \
  --include-production-read-profile --production-read-only \
  --query-metric-k 10 \
  --query-metric-projection-columns id \
  --log-output reviews/task-30/1063-spire-phase13e-aws-pooling-comparison-q20/artifacts/debug-production-read-profile-q20-pool-off.log
```

Pool on:

```bash
scripts/spire-aws/with-ssm-port-forwards.sh \
  reviews/task-30/1062-spire-phase13e-aws-representative-after-preserve-harness/artifacts/aws-topology.json \
  reviews/task-30/1063-spire-phase13e-aws-pooling-comparison-q20/artifacts \
  reviews/task-30/1063-spire-phase13e-aws-pooling-comparison-q20/artifacts/aws-topology.tunneled.pool-on.json \
  -- /home/peter/dev/ecaz/target/release/ecaz \
  --database postgres --host 127.0.0.1 --port 15432 --user ecaz_coord \
  bench spire-pipeline \
  --prefix ec_spire_aws_repr_1m \
  --queries-limit 20 \
  --sweep 8,16,24,32 \
  --include-remote --require-remote-placements \
  --top-k 10 \
  --include-query-metrics --include-recall \
  --truth-corpus-file reviews/task-30/1062-spire-phase13e-aws-representative-after-preserve-harness/artifacts/work/qdrant-dbpedia/prepared/ec_real_100k_corpus.tsv \
  --include-production-read-profile --production-read-only \
  --query-metric-k 10 \
  --query-metric-projection-columns id \
  --log-output reviews/task-30/1063-spire-phase13e-aws-pooling-comparison-q20/artifacts/debug-production-read-profile-q20-pool-on.log
```

## Key Lines

Pool off production read profile:

| nprobe | dispatch_sum | socket_open_sum | connect_p50 | total_p50 | recall@10 |
| --- | ---: | ---: | ---: | ---: | ---: |
| 8 | 53 | 53 | 19.000 ms | 48.000 ms | 0.8150 |
| 16 | 60 | 60 | 20.000 ms | 50.000 ms | 0.8600 |
| 24 | 60 | 60 | 20.000 ms | 52.000 ms | 0.9000 |
| 32 | 60 | 60 | 20.000 ms | 55.000 ms | 0.9250 |

Pool on production read profile:

| nprobe | dispatch_sum | socket_open_sum | connect_p50 | total_p50 | recall@10 |
| --- | ---: | ---: | ---: | ---: | ---: |
| 8 | 53 | 0 | 0.000 ms | 38.000 ms | 0.8150 |
| 16 | 60 | 0 | 0.000 ms | 41.000 ms | 0.8600 |
| 24 | 60 | 0 | 0.000 ms | 43.000 ms | 0.9000 |
| 32 | 60 | 0 | 0.000 ms | 44.000 ms | 0.9250 |

Delta TSV, defined as pool-off minus pool-on:

```tsv
metric	nprobe_8	nprobe_16	nprobe_24	nprobe_32
socket_open_sum_delta	53	60	60	60
connect_p50_delta_ms	19	20	20	20
total_p50_delta_ms	10	9	9	11
```

Cleanup check after the probes found no `with-ssm-port-forwards`, `session-manager-plugin`, `aws ssm start-session`, or `bench spire-pipeline` child process still running.
