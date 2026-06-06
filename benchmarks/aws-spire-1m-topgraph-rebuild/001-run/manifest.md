# AWS SPIRE 1M top-graph rebuild benchmark

- packet: `benchmarks/aws-spire-1m-topgraph-rebuild/001-run`
- head SHA: `73bbcb1573b1a29074208bad11eeba83302e8ace`
- date: `2026-06-04`
- lane: `aws-graviton`
- cloud profile: `1m`
- database: `postgres`
- S3 bucket: `ecaz-cloud-1m-b62eb804`
- corpus table: `corpus`
- corpus rows: `990000`
- query rows: `10000`
- benchmark query count: `500`
- truth cache: `benchmarks/task51-aws-ivf-rabitq-final-gate/artifacts/truth-aws-real-1m-q500-k10.json`

## Purpose

Measure whether rebuilding the 1M SPIRE rabitq index with a larger
`top_graph_search_list_size` allows high-recall scans above the old index's
`nprobe=96` ceiling, and whether the recovered recall is competitive.

## Commands

```bash
target/debug/ecaz bench suite audit \
  --config benchmarks/aws-spire-1m-topgraph-rebuild/001-run/suite-precheck.json

target/debug/ecaz cloud bench --profile 1m --database postgres \
  --config benchmarks/aws-spire-1m-topgraph-rebuild/001-run/suite-precheck.json \
  --suite aws-spire-1m-topgraph-rebuild-precheck \
  --ecaz-bin /usr/local/bin/ecaz \
  --log-file benchmarks/aws-spire-1m-topgraph-rebuild/001-run/artifacts/cloud-bench-precheck.log

target/debug/ecaz bench suite audit \
  --config benchmarks/aws-spire-1m-topgraph-rebuild/001-run/suite-build-tg256-recall-cache-500.json

target/debug/ecaz cloud bench --profile 1m --database postgres \
  --config benchmarks/aws-spire-1m-topgraph-rebuild/001-run/suite-build-tg256-recall-cache-500.json \
  --suite aws-spire-1m-rabitq-tg256-recall-cache-500 \
  --ecaz-bin /usr/local/bin/ecaz \
  --log-file benchmarks/aws-spire-1m-topgraph-rebuild/001-run/artifacts/cloud-bench-tg256-recall-cache-500.log

target/debug/ecaz bench suite audit \
  --config benchmarks/aws-spire-1m-topgraph-rebuild/001-run/suite-query-tg256-recall-cache-500.json

target/debug/ecaz cloud bench --profile 1m --database postgres \
  --config benchmarks/aws-spire-1m-topgraph-rebuild/001-run/suite-query-tg256-recall-cache-500.json \
  --suite aws-spire-1m-rabitq-tg256-query-recall-cache-500 \
  --ecaz-bin /usr/local/bin/ecaz \
  --log-file benchmarks/aws-spire-1m-topgraph-rebuild/001-run/artifacts/cloud-bench-query-tg256-recall-cache-500.log

target/debug/ecaz bench suite report \
  --manifest benchmarks/aws-spire-1m-topgraph-rebuild/001-run/artifacts/tg256-query-recall-cache-500/suite-manifest.json \
  --results-output benchmarks/aws-spire-1m-topgraph-rebuild/001-run/artifacts/tg256-query-recall-cache-500/results-report.jsonl \
  --log-file benchmarks/aws-spire-1m-topgraph-rebuild/001-run/artifacts/tg256-query-recall-cache-500/suite-report.md

target/debug/ecaz cloud pause --profile 1m --database postgres \
  --log-file benchmarks/aws-spire-1m-topgraph-rebuild/001-run/artifacts/cloud-pause-after-query-tg256.log

target/debug/ecaz cloud status --profile 1m --database postgres \
  --log-file benchmarks/aws-spire-1m-topgraph-rebuild/001-run/artifacts/cloud-status-final.log
```

## Indexes

Precheck showed the existing SPIRE index:

```text
aws_spire_1m_rabitq_global1152_idx | ec_spire | 872 MB |
{nlists=128,recursive_fanout=8,nprobe=24,rerank_width=25,storage_format=rabitq,boundary_replica_count=0,top_graph_enabled=1,top_graph_degree=32,top_graph_build_list_size=100,top_graph_search_list_size=96}
```

The build suite created the wider top-graph sibling index:

```text
aws_spire_1m_rabitq_tg256_idx | ec_spire | 775 MB |
{nlists=128,recursive_fanout=8,nprobe=24,rerank_width=25,storage_format=rabitq,boundary_replica_count=0,top_graph_enabled=1,top_graph_degree=32,top_graph_build_list_size=100,top_graph_search_list_size=256}
```

Build timing:

```text
ec_spire_ambuild_timing index=aws_spire_1m_rabitq_tg256_idx phase=complete heap_tuples=990000 scanned_tuples=990000 index_tuples=990000 recursive_fanout=8 setup_ms=2 heap_scan_ms=600539 sample_collect_ms=2 kmeans_ms=2117 kmeans_calls=1 assignment_ms=25955 recursive_kmeans_ms=1 recursive_kmeans_calls=1 recursive_kmeans_max_level=1 recursive_assignment_ms=0 recursive_routing_initial_children=128 recursive_routing_final_children=8 recursive_routing_iterations=1 draft_ms=4924 draft_total_ms=4925 draft_input_clone_ms=2021 draft_pid_alloc_ms=0 draft_recursive_routing_ms=6 draft_route_map_ms=0 draft_leaf_rows_ms=293 draft_leaf_inputs_ms=2043 draft_validation_ms=0 top_graph_ms=16371 pq4_training_ms=0 object_store_ms=0 object_store_total_ms=16371 publish_ms=2 total_ms=649926
```

The initial build+query suite failed after the successful build because v1
SPIRE relation-context loading does not tolerate two `ec_spire` indexes on the
same heap relation for this query path. The follow-up query suite explicitly
dropped `aws_spire_1m_rabitq_global1152_idx` before running, leaving
`aws_spire_1m_rabitq_tg256_idx` as the only active SPIRE index.

## q500 recall/latency curve

The successful query suite used `top_graph_search_list_size=256`,
`rerank_width=25`, the q500 truth cache, and the same leaf block pruning
settings as the prior 1M SPIRE run.

| nprobe | effective nprobe | recall@10 | p50 ms | p95 ms | p99 ms | candidate sum | route sum | heap rerank sum |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 64 | 64 | 0.9976 | 554.168 | 585.839 | 595.174 | 251510240 | 32000 | 12500 |
| 96 | 96 | 0.9994 | 779.315 | 805.380 | 814.695 | 373897385 | 48000 | 12500 |
| 128 | 128 | 1.0000 | 1038.917 | 1049.278 | 1052.191 | 495000000 | 64000 | 12500 |
| 256 | 128 | 1.0000 | 1029.300 | 1040.377 | 1043.784 | 495000000 | 64000 | 12500 |

`nprobe=256` caps at effective `nprobe=128` because this index has 128 leaves.

## Comparison

Prior accepted 1M SPIRE result on the old `top_graph_search_list_size=96` index:

| index | nprobe | recall@10 | p50 ms | p95 ms | candidate sum |
| --- | ---: | ---: | ---: | ---: | ---: |
| `aws_spire_1m_rabitq_global1152_idx` | 96 | 0.9832 | 268.824 | 331.460 | 9213846 |
| `aws_spire_1m_rabitq_tg256_idx` | 64 | 0.9976 | 554.168 | 585.839 | 251510240 |
| `aws_spire_1m_rabitq_tg256_idx` | 96 | 0.9994 | 779.315 | 805.380 | 373897385 |
| `aws_spire_1m_rabitq_tg256_idx` | 128 | 1.0000 | 1038.917 | 1049.278 | 495000000 |

The wider top graph recovers recall, including full q500 recall at effective
`nprobe=128`, but it does so by expanding the candidate surface by roughly
27x to 54x versus the prior optimized `nprobe=96` run. This is useful as a
recall ceiling measurement, but it is not a competitive read-path improvement.

## Artifact inventory

- `suite-precheck.json`: precheck suite config.
- `suite-build-tg256-recall-cache-500.json`: build plus query suite config; build succeeded, query failed after duplicate SPIRE index relation-context issue.
- `suite-query-tg256-recall-cache-500.json`: query-only suite config that activates the tg256 index as the only SPIRE index.
- `artifacts/precheck/existing-spire-1m-reloptions.log`: row counts and existing index reloptions.
- `artifacts/precheck/suite-manifest.json`: precheck suite manifest.
- `artifacts/precheck/suite-run.log`: precheck suite run log.
- `artifacts/tg256-recall-cache-500/build-spire-1m-rabitq-tg256-index.log`: successful tg256 index build output.
- `artifacts/tg256-recall-cache-500/suite-manifest-failed.json`: failed build+query suite manifest.
- `artifacts/tg256-recall-cache-500/ssm-error-summary.log`: SSM failure summary for the first query attempt.
- `artifacts/tg256-query-recall-cache-500/activate-spire-1m-rabitq-tg256-index.log`: query-only activation step, including drop of old SPIRE index.
- `artifacts/tg256-query-recall-cache-500/pipeline-spire-1m-rabitq-tg256-query-recall-cache-500.log`: raw q500 recall/latency output.
- `artifacts/tg256-query-recall-cache-500/results.jsonl`: structured results emitted by `ecaz bench suite`.
- `artifacts/tg256-query-recall-cache-500/results-report.jsonl`: structured results emitted by `ecaz bench suite report`.
- `artifacts/tg256-query-recall-cache-500/suite-manifest.json`: successful query suite manifest.
- `artifacts/tg256-query-recall-cache-500/suite-report.md`: parsed report for the successful query suite.
- `artifacts/tg256-query-recall-cache-500/suite-run.log`: query suite run log.
- `artifacts/cloud-bench-tg256-recall-cache-500.log`: local cloud-bench wrapper log for the first build+query attempt.
- `artifacts/cloud-status-final.log`: final AWS status; profile `1m` is paused.
