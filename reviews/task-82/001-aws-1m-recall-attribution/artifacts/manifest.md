# Task 82 AWS 1M Recall Attribution Manifest

- Head SHA for measured code: `26173f7d6`
- Packet: `reviews/task-82/001-aws-1m-recall-attribution/`
- Suite config: `reviews/task-82/001-aws-1m-recall-attribution/suite-aws-1m-miss-attribution-q500.json`
- AWS run id: `20260605T174022Z`
- Truth cache: `benchmarks/task51-aws-ivf-rabitq-final-gate/artifacts/truth-aws-real-1m-q500-k10.json`
- Surface: retained Task 79/81 AWS 1M q500 local SPIRE scan, `task67_1m_hnsw_m7g2xlarge`, index `aws_spire_1m_rabitq_t80_block16_tg256_idx`, `nprobe=96`, `rerank_width=25`, global block budget `1152`, isolated one-index-per-table surface.

## Commands

Local validation:

```text
cargo test -p ecaz-cli spire_pipeline --no-default-features
cargo build -p ecaz-cli --no-default-features
target/debug/ecaz bench suite --config reviews/task-82/001-aws-1m-recall-attribution/suite-aws-1m-miss-attribution-q500.json --suite task82-aws-1m-miss-attribution-q500 --audit --log-file reviews/task-82/001-aws-1m-recall-attribution/artifacts/suite-audit-target-assignment.log
```

AWS benchmark:

```text
target/debug/ecaz cloud bench --profile 1m --database postgres --config reviews/task-82/001-aws-1m-recall-attribution/suite-aws-1m-miss-attribution-q500.json --suite task82-aws-1m-miss-attribution-q500 --ecaz-bin /usr/local/bin/ecaz --log-file reviews/task-82/001-aws-1m-recall-attribution/artifacts/cloud-bench-task82-q500-bounded.log
```

AWS shutdown/status:

```text
target/debug/ecaz cloud pause --profile 1m --database postgres --log-file reviews/task-82/001-aws-1m-recall-attribution/artifacts/cloud-pause-after-task82-bounded.log
script -q -c "target/debug/ecaz cloud status --profile 1m --database postgres" reviews/task-82/001-aws-1m-recall-attribution/artifacts/cloud-status-final-paused.log
```

## Artifacts

- `suite-audit-target-assignment.log`: suite audit for the bounded target-assignment diagnostic config. Key line: `[suite:task82-aws-1m-miss-attribution-q500] audit passed: 2 steps`.
- `cloud-bench-task82-q500-bounded.log`: cloud bench wrapper log. Key line: synced artifacts from `s3://ecaz-cloud-1m-b62eb804/bench-artifacts/task82-aws-1m-miss-attribution-q500/20260605T174022Z/`.
- `aws-1m-miss-attribution-q500/suite-run.log`: remote `ecaz bench suite` run log.
- `aws-1m-miss-attribution-q500/suite-manifest.json`: structured suite manifest.
- `aws-1m-miss-attribution-q500/results.jsonl`: structured metric rows.
- `aws-1m-miss-attribution-q500/pipeline-spire-1m-rabitq-block-summary-global1152-miss-attribution-q500.log`: human-readable pipeline tables.
- `aws-1m-miss-attribution-q500/miss-attribution-spire-1m-global1152-q500.jsonl`: one row per q500 truth neighbor with hit/miss stage.
- `miss-attribution-summary.txt`: parsed key results and recommendation.
- `cloud-pause-after-task82-bounded.log`: shutdown log after successful run.
- `cloud-status-final-paused.log`: final status artifact. Key line: `state:    paused`.

The earlier full block-rank attempt is retained for negative evidence:

- `cloud-bench-task82-q500.log`
- `cloud-bench-task82-q500-rerun.log`
- `cloud-pause-after-slow-attribution-cancel.log`

That path was cancelled after a single full block-rank backend ran longer than 11 minutes, so the final packet uses the bounded target-assignment diagnostic instead of retrying full rank attribution.

## Key Results

Retained Task 79/81 comparison point, rerun at Task 81 HEAD:

- `benchmarks/aws-spire-1m-task81-head-rerun/001-run/`
- `recall@10=0.9832`
- `candidate_sum=9,213,846`
- `latency_p50=250.251 ms`, `latency_p95=307.810 ms`, `latency_p99=322.053 ms`

Task 82 bounded attribution run:

- `recall@10=0.9832`
- `candidate_sum=9,213,846`
- `heap_rerank_sum=12,500`
- `latency_p50=281.345 ms`, `latency_p95=351.396 ms`, `latency_p99=370.986 ms`
- q500 truth rows: `5,000`
- hits: `4,916`
- misses: `84`

Miss-stage table:

| Stage | Missed truth rows |
| --- | ---: |
| `routing_miss` | 3 |
| `selected_leaf_block_pruning_or_candidate_cap` | 81 |
| `assignment_missing` | 0 |
| `candidate_or_rerank_cap` | 0 |

Interpretation: the 1M/q500 recall gap is dominated by rows whose leaf was selected but whose selected-leaf candidate surface was still pruned/capped. The next narrow SPIRE slice should therefore add a target-only selected-block containment diagnostic and then tune selected-leaf block scoring/pruning recovery. Wider top-graph routing is not justified by this packet because only `3/84` missed truth rows were pure routing misses, while known recall-ceiling top-graph runs required `251M-495M` q500 candidates.
