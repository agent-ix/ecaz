# Task 85 AWS 1M Per-Leaf Block Cap Manifest

- Packet: `reviews/task-85/005-aws-1m-per-leaf-block-cap/`
- Branch: `task-85-spire-product-scale-pareto`
- Head SHA: `b6b9306a901817ca2e5a0e7d1cd582e13534b75b`
- Suite config:
  `reviews/task-85/005-aws-1m-per-leaf-block-cap/suite-aws-1m-per-leaf-block-cap-q500.json`
- Lane / fixture: AWS profile `1m`, q500, prefix
  `task67_1m_hnsw_m7g2xlarge`, database `postgres`
- Storage format: `rabitq`
- Rerank mode: `rerank_width=25`
- Surface: retained block16 SPIRE index
  `aws_spire_1m_rabitq_t80_block16_tg256_idx`; no index build.

## Rationale

Packet 004 showed block8 reduced candidates but increased the expensive parts
of the query path:

| Row | Candidate Sum | p50 Object Read | p50 Candidate/summary Score |
| --- | ---: | ---: | ---: |
| retained block16 global1152 | 9,213,846 | ~171.6 ms | ~56.8 ms |
| block8 global1152 | 4,607,442 | ~228.7 ms | ~97.8 ms |

That rules out row-count reduction by smaller geometry as a complete latency
mechanism. The next low-risk query-only hypothesis is that the retained global
block selector may spend avoidable time sorting/allocating a global block
frontier. A per-leaf cap of `12` blocks should produce the same nominal block
budget as global1152 while using per-leaf selection.

## Commands

Audit:

`target/debug/ecaz bench suite audit --config reviews/task-85/005-aws-1m-per-leaf-block-cap/suite-aws-1m-per-leaf-block-cap-q500.json --log-file reviews/task-85/005-aws-1m-per-leaf-block-cap/artifacts/suite-audit.log`

AWS run:

`target/debug/ecaz cloud bench --profile 1m --database postgres --config reviews/task-85/005-aws-1m-per-leaf-block-cap/suite-aws-1m-per-leaf-block-cap-q500.json --suite task85-aws-1m-per-leaf-block-cap-q500 --ecaz-bin /usr/local/bin/ecaz --log-file reviews/task-85/005-aws-1m-per-leaf-block-cap/artifacts/cloud-bench-per-leaf-block-cap.log`

## Pending Artifacts

Expected successful run artifacts under:

`reviews/task-85/005-aws-1m-per-leaf-block-cap/artifacts/aws-1m-per-leaf-block-cap-q500/`

Final packet update must include:

- `suite-manifest.json`, `results.jsonl`, and `suite-report.md`;
- pipeline logs and funnel JSONL for all rows;
- cloud resume/bench/pause logs;
- final AWS paused status.
