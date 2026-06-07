# Task 84 Latency-Retention k2/k3 Control Manifest

- Task: `plan/tasks/84-spire-1m-recall-recovery-without-candidate-inflation.md`
- Packet: `reviews/task-84/007-latency-retention-aws-k2-k3-control/`
- Branch: `task-84-spire-recall-recovery`
- Head SHA at run: `0e3effe04`
- Lane: AWS Graviton / PG18 / retained `1m` profile
- Corpus prefix: `task67_1m_hnsw_m7g2xlarge`
- Query rows: q500 from `task67_1m_hnsw_m7g2xlarge_queries`
- Truth cache:
  `benchmarks/task51-aws-ivf-rabitq-final-gate/artifacts/truth-aws-real-1m-q500-k10.json`
- Runner: `ecaz bench suite`

## Purpose

This packet reinterprets Tasks 80-84 under the correct user goal:

- preserve the retained AWS 1M/q500 recall point `0.9832`;
- preserve the retained candidate surface around `9.21M`;
- improve p50/p95/p99 latency.

The packet specifically tests whether Task 84's apparent k3 latency win was a
real k3 effect or run-order/warm-state variance by running paired rows in one
suite:

1. retained k2 `global1152`;
2. retained k3 `global1152`;
3. k2 route-prior `0.10`;
4. retained k2 `global1152` repeat.

## Commands

- Suite audit:
  `target/debug/ecaz bench suite audit --config reviews/task-84/007-latency-retention-aws-k2-k3-control/suite-aws-1m-latency-retention-k2-k3-q500.json --log-file reviews/task-84/007-latency-retention-aws-k2-k3-control/artifacts/suite-audit.log`
- Resume:
  `target/debug/ecaz cloud resume --profile 1m --database postgres --log-file reviews/task-84/007-latency-retention-aws-k2-k3-control/artifacts/cloud-resume-latency-retention.log`
- Install:
  `target/debug/ecaz cloud install --profile 1m --database postgres --git-ref task-84-spire-recall-recovery --skip-extension-recreate --log-file reviews/task-84/007-latency-retention-aws-k2-k3-control/artifacts/cloud-install-latency-retention.log`
- Bench:
  `target/debug/ecaz cloud bench --profile 1m --database postgres --config reviews/task-84/007-latency-retention-aws-k2-k3-control/suite-aws-1m-latency-retention-k2-k3-q500.json --suite task84-aws-1m-latency-retention-k2-k3-q500 --ecaz-bin /usr/local/bin/ecaz --log-file reviews/task-84/007-latency-retention-aws-k2-k3-control/artifacts/cloud-bench-latency-retention-k2-k3.log`
- Pause:
  `target/debug/ecaz cloud pause --profile 1m --database postgres --log-file reviews/task-84/007-latency-retention-aws-k2-k3-control/artifacts/cloud-pause-after-latency-retention.log`
- Final status:
  `target/debug/ecaz cloud status --profile 1m --database postgres --log-file reviews/task-84/007-latency-retention-aws-k2-k3-control/artifacts/cloud-status-after-latency-retention-pause.log`

## Artifacts

- Suite config:
  `reviews/task-84/007-latency-retention-aws-k2-k3-control/suite-aws-1m-latency-retention-k2-k3-q500.json`
- Synced suite artifact directory:
  `reviews/task-84/007-latency-retention-aws-k2-k3-control/artifacts/aws-1m-latency-retention-k2-k3-q500/`
- Suite manifest:
  `reviews/task-84/007-latency-retention-aws-k2-k3-control/artifacts/aws-1m-latency-retention-k2-k3-q500/suite-manifest.json`
- Results:
  `reviews/task-84/007-latency-retention-aws-k2-k3-control/artifacts/aws-1m-latency-retention-k2-k3-q500/results.jsonl`
- Suite report:
  `reviews/task-84/007-latency-retention-aws-k2-k3-control/artifacts/aws-1m-latency-retention-k2-k3-q500/suite-report.md`
- Compact table:
  `reviews/task-84/007-latency-retention-aws-k2-k3-control/artifacts/latency-retention-summary.tsv`
- Final AWS status:
  `reviews/task-84/007-latency-retention-aws-k2-k3-control/artifacts/cloud-status-after-latency-retention-pause.log`

## Result

| row | recall@10 | candidate_sum | heap_rerank_sum | p50 | p95 | p99 | miss split |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| k2 first | 0.9832 | 9,213,846 | 12,500 | 274.607 ms | 340.723 ms | 354.243 ms | 4916/3/81 |
| k3 | 0.9832 | 9,213,742 | 12,500 | 257.047 ms | 319.000 ms | 336.771 ms | 4916/3/81 |
| k2 route-prior 0.10 | 0.9832 | 9,213,619 | 12,500 | 254.764 ms | 317.188 ms | 332.755 ms | 4916/3/81 |
| k2 repeat | 0.9832 | 9,213,846 | 12,500 | 255.571 ms | 314.469 ms | 331.985 ms | 4916/3/81 |

## Interpretation

The apparent Task 84 k3 latency win is not a durable k3 win. In the paired
suite, k3 is much faster than the first k2 row, but not faster than the warmed
k2 repeat:

- k3 p50: `257.047 ms`;
- k2 repeat p50: `255.571 ms`.

Route-prior `0.10` also does not materially beat warmed k2:

- route-prior p50: `254.764 ms`;
- k2 repeat p50: `255.571 ms`.

All rows preserve recall `0.9832`, the `9.21M` candidate surface, and the same
miss split `4916/3/81`.

Corrected conclusion: Tasks 82-84 were wrongly framed around recall recovery
for the user's latency goal, but the tested k3 and route-prior rows still do
not establish a configuration-level latency win once paired with a same-run k2
repeat. The real lesson is that AWS 1M SPIRE latency claims need a standardized
warmup/order-controlled suite before accepting or rejecting small latency
deltas.

AWS `1m` was paused at closeout: `state: paused`, running cost `~$0.00/hr`.
