# Task 74 Closeout

Reviewer: please review this Task 74 closeout marker.

## Summary

Task 74 is closed as an overhead audit and decision packet. The local M5 and
AWS Graviton evidence both show material SPIRE-specific overhead at matched
recall versus the IVF control. This branch does not ship optimization code:
without a local external flamegraph/samply profile, there is not enough
function-level attribution to justify one of the Phase 2 optimization slices.

The closeout records that Task 74's next useful step is a profiler-backed
optimization follow-on, not a speculative change in this branch.

## Evidence

- Local M5 overhead gate: `reviews/task-74/001-spire-m5-overhead-gate/`
- Shared Task 73 M5 suite artifacts:
  `reviews/task-73/001-spire-m5-quality-gate/artifacts/`
- Shared AWS quality/overhead packet:
  `benchmarks/task73-74-aws-spire-quality-overhead/`
- AWS closeout commit with results and cost cleanup: `f10a9bfd3`

At matched local M5 recall:

| surface | setting | recall@10 | p50 | p95 | p99 |
| --- | --- | ---: | ---: | ---: | ---: |
| SPIRE high-recall | tg128 b0 nprobe=96 | 0.9975 | 75.790 ms | 79.387 ms | 82.456 ms |
| IVF control | nprobe=96 | 0.9980 | 10.6 ms | 11.9 ms | 14.0 ms |
| SPIRE ceiling | tg128 b0 nprobe=128 | 1.0000 | 95.960 ms | 96.476 ms | 99.049 ms |
| IVF control | nprobe=128 | 1.0000 | 12.7 ms | 13.8 ms | 14.3 ms |

At matched AWS Graviton recall:

| surface | setting | recall@10 | p50 | p95 | p99 |
| --- | --- | ---: | ---: | ---: | ---: |
| SPIRE high-recall | tg128 b0 nprobe=96 | 0.9975 | 127.618 ms | 132.383 ms | 133.764 ms |
| IVF control | nprobe=96 | 0.9980 | 28.6 ms | 30.2 ms | 30.9 ms |
| SPIRE ceiling | tg128 b0 nprobe=128 | 1.0000 | 162.482 ms | 162.930 ms | 163.331 ms |
| IVF control | nprobe=128 | 1.0000 | 35.0 ms | 36.6 ms | 37.0 ms |

The overhead concern therefore reproduces on both hosts: roughly 7.1x-7.6x on
local M5 and roughly 4.5x-4.6x on AWS Graviton for the measured matched-recall
points.

## Slice Decisions

- Per-rescan setup reuse is shelved pending function-level profile evidence.
- Routing-draft caching is shelved because changing routing reuse semantics
  needs Task 30 coordination and a hot-path profile.
- Candidate buffer reuse is shelved pending allocation attribution.
- Leaf row decode pipeline work is shelved pending attribution that the hot
  loop is SPIRE row materialization rather than scoring or routing.
- Snapshot management changes are shelved pending attribution and correctness
  review against PostgreSQL snapshot semantics.

The Task 74 Phase 1 packet did not include an external `samply` or
`cargo flamegraph` artifact. It used suite-visible query metrics, SPIRE
pipeline counters, production-read totals, and the IVF same-host control. This
closeout accepts that evidence as enough to classify the overhead as material,
but not enough to land a safe optimization slice.

## Validation

- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
  passed: `reviews/task-74/002-closeout/artifacts/cargo-clippy-pg18.log`
- `git diff --check` passed:
  `reviews/task-74/002-closeout/artifacts/git-diff-check.log`
- `reviews/task-74/002-closeout/artifacts/code-diff-files.log` is empty for
  `src/` and `crates/` paths in the branch diff against `origin/main`; this
  closeout introduces no source changes and no new `unsafe { ... }` blocks.

## Outcome

`plan/tasks/74-spire-leaf-scan-overhead.md` is marked complete. The follow-on
should start with a profiler-backed M5 local run at the Task 73 high-recall
points before changing SPIRE scan code.
