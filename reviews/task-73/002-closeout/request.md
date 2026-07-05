# Task 73 Closeout

Reviewer: please review this Task 73 closeout marker.

## Summary

Task 73 is closed as a measurement and decision packet. The local M5 suite
reproduced the Task 68 default recall floor and showed the floor is tunable:
SPIRE reaches recall@10 `0.9975` at `top_graph_search_list_size=128`,
`boundary_replica_count=0`, `nprobe=96`, and reaches `1.0000` at the same
top-graph setting with `nprobe=128`. AWS Graviton reproduced the same quality
shape.

No default or routing-quality code change is being shipped in this branch. The
high-recall point is real, but it has a large latency cost versus the IVF
control, so changing defaults belongs in a follow-on product/defaults decision
rather than as an automatic Task 73 slice.

## Evidence

- Local M5 quality gate: `reviews/task-73/001-spire-m5-quality-gate/`
- Shared AWS quality/overhead packet:
  `benchmarks/task73-74-aws-spire-quality-overhead/`
- AWS closeout commit with results and cost cleanup: `f10a9bfd3`

Key local M5 results:

| surface | setting | recall@10 | p50 | p95 | p99 |
| --- | --- | ---: | ---: | ---: | ---: |
| SPIRE 10k default reproduction | tg16 b0 nprobe=16 | 0.9995 | 5.939 ms | 6.246 ms | 6.344 ms |
| SPIRE 100k default reproduction | tg16 b0 nprobe=16 | 0.8525 | 13.505 ms | 15.410 ms | 15.868 ms |
| SPIRE 100k high-recall | tg128 b0 nprobe=96 | 0.9975 | 75.790 ms | 79.387 ms | 82.456 ms |
| SPIRE 100k ceiling | tg128 b0 nprobe=128 | 1.0000 | 95.960 ms | 96.476 ms | 99.049 ms |
| IVF 100k control | nprobe=96 | 0.9980 | 10.6 ms | 11.9 ms | 14.0 ms |
| IVF 100k control | nprobe=128 | 1.0000 | 12.7 ms | 13.8 ms | 14.3 ms |

Key AWS Graviton results:

| surface | setting | recall@10 | p50 | p95 | p99 |
| --- | --- | ---: | ---: | ---: | ---: |
| SPIRE default | tg16 b0 nprobe=16 | 0.8525 | 25.287 ms | 28.274 ms | 29.222 ms |
| SPIRE high-recall | tg128 b0 nprobe=96 | 0.9975 | 127.618 ms | 132.383 ms | 133.764 ms |
| SPIRE ceiling | tg128 b0 nprobe=128 | 1.0000 | 162.482 ms | 162.930 ms | 163.331 ms |
| IVF control | nprobe=96 | 0.9980 | 28.6 ms | 30.2 ms | 30.9 ms |
| IVF control | nprobe=128 | 1.0000 | 35.0 ms | 36.6 ms | 37.0 ms |

## Slice Decisions

- Default `top_graph_search_list_size` and `nprobe` changes are shelved. They
  can restore recall, but the measured high-recall point is 4.5x-7.6x slower
  than the IVF control depending on host and setting.
- Boundary-replica-aware routing is shelved. The local sweep showed replicas
  improve lower-nprobe recall but are slower than the `b0` Pareto points.
- Adaptive routing aggression is shelved. The current evidence covers 10k and
  100k fixtures; a row-count-aware default needs a broader size curve.
- Additional diagnostic surface is shelved. The suite diagnostics were enough
  to classify this as a tunable recall/defaults issue, not a hard recall
  ceiling.

## Validation

- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
  passed: `reviews/task-73/002-closeout/artifacts/cargo-clippy-pg18.log`
- `git diff --check` passed:
  `reviews/task-73/002-closeout/artifacts/git-diff-check.log`
- `reviews/task-73/002-closeout/artifacts/code-diff-files.log` is empty for
  `src/` and `crates/` paths in the branch diff against `origin/main`; this
  closeout introduces no source changes and no new `unsafe { ... }` blocks.

## Outcome

`plan/tasks/73-spire-recall-characterization.md` is marked complete. The
follow-on decision is whether SPIRE should expose or adopt a quality-oriented
default despite the measured overhead.
