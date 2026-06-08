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

This packet tested whether the retained block16 index could avoid global block
allocation overhead by replacing the global cap with equal per-leaf caps. The
headline `perleaf12` row has the same nominal block budget as global1152:
`96` leaf routes times `12` block16 blocks equals `1152` blocks.

## Commands

Audit:

`target/debug/ecaz bench suite audit --config reviews/task-85/005-aws-1m-per-leaf-block-cap/suite-aws-1m-per-leaf-block-cap-q500.json --log-file reviews/task-85/005-aws-1m-per-leaf-block-cap/artifacts/suite-audit.log`

AWS run:

`target/debug/ecaz cloud bench --profile 1m --database postgres --config reviews/task-85/005-aws-1m-per-leaf-block-cap/suite-aws-1m-per-leaf-block-cap-q500.json --suite task85-aws-1m-per-leaf-block-cap-q500 --ecaz-bin /usr/local/bin/ecaz --log-file reviews/task-85/005-aws-1m-per-leaf-block-cap/artifacts/cloud-bench-per-leaf-block-cap.log`

Report generation:

`target/debug/ecaz bench suite report --manifest reviews/task-85/005-aws-1m-per-leaf-block-cap/artifacts/aws-1m-per-leaf-block-cap-q500/suite-manifest.json --results-output reviews/task-85/005-aws-1m-per-leaf-block-cap/artifacts/aws-1m-per-leaf-block-cap-q500/results-report.jsonl --log-file reviews/task-85/005-aws-1m-per-leaf-block-cap/artifacts/aws-1m-per-leaf-block-cap-q500/suite-report.md`

## Cloud Run

- SSM command: `ac72e32f-c89e-46a1-ab7a-3024bd21e7b2`
- S3 artifact prefix:
  `s3://ecaz-cloud-1m-b62eb804/bench-artifacts/task85-aws-1m-per-leaf-block-cap-q500/20260607T091531Z/`
- Suite result: completed `7`, failed `0`, skipped `0`.
- Final AWS status: `profile=1m state=paused db=10.42.1.131 bucket=ecaz-cloud-1m-b62eb804`

## Results

| Row | Recall@10 | p50 | p95 | p99 | Candidate Sum |
| --- | ---: | ---: | ---: | ---: | ---: |
| global1152 control first | 0.9832 | 271.674 ms | 337.657 ms | 349.454 ms | 9,213,846 |
| perleaf8 | 0.7108 | 479.673 ms | 498.422 ms | 505.507 ms | 6,142,746 |
| perleaf10 | 0.7454 | 474.696 ms | 491.903 ms | 500.470 ms | 7,678,346 |
| perleaf12 | 0.7714 | 477.209 ms | 494.663 ms | 500.243 ms | 9,213,924 |
| perleaf14 | 0.7980 | 486.028 ms | 503.404 ms | 510.191 ms | 10,749,465 |
| global1152 control repeat | 0.9832 | 249.160 ms | 306.730 ms | 318.763 ms | 9,213,846 |

## Funnel Timing

| Row | p50 Object Read | p50 Candidate/summary Score | Candidate Sum |
| --- | ---: | ---: | ---: |
| global1152 control repeat | 170.822 ms | 55.787 ms | 9,213,846 |
| perleaf8 | 353.752 ms | 53.205 ms | 6,142,746 |
| perleaf10 | 351.422 ms | 54.812 ms | 7,678,346 |
| perleaf12 | 350.475 ms | 56.665 ms | 9,213,924 |
| perleaf14 | 356.014 ms | 58.361 ms | 10,749,465 |

## Verdict

Per-leaf caps are rejected as a Task85 product-scale latency improvement. They
fail the core requirement twice: they lose recall badly and they are much
slower than the same-suite global warm control. The evidence implies that
global block allocation is essential for retained recall, and the next latency
work must target object read/layout cost or a deeper format/scoring path rather
than equal per-leaf block budgets.
