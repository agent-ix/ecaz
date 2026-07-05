# Task 75 Closeout

Reviewer: please review this Task 75 closeout marker.

## Summary

Task 75 closes as a measurement and decision task. The branch added the missing
candidate-funnel diagnostic surface, ran the Intel-local routing-envelope suite,
and recorded why no Phase 2 routing slice should land from this evidence.

The measured high-recall local point reaches recall@10 `0.9975`, but the
candidate envelope remains broad: tg96/tg128 b0 scans `2,784,952` leaf
candidates over the 200-query sample while only `5,000` rows survive to heap
rerank and `2,000` rows are returned. IVF nprobe96 reaches comparable recall
with a much smaller posting/candidate surface and lower latency.

## Evidence

- Diagnostic/code packet:
  `reviews/task-75/001-candidate-funnel-diagnostics/`
- Phase 2 decision packet:
  `reviews/task-75/002-phase2-decision/`
- Benchmark packet:
  `benchmarks/task75-intel-local-routing-envelope/`
- Benchmark manifest:
  `benchmarks/task75-intel-local-routing-envelope/manifest.md`
- Benchmark summary:
  `benchmarks/task75-intel-local-routing-envelope/artifacts/summary.md`

Key Intel-local 100k rows:

| Point | nprobe | recall@10 | p50 | p95 | leaf candidates | retained | returned |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| SPIRE tg16 b0 | 16 | 0.8525 | 26.814 ms | 33.414 ms | 2,087,914 | 5,000 | 2,000 |
| SPIRE tg32 b0 | 32 | 0.9310 | 48.199 ms | 54.407 ms | 2,769,013 | 5,000 | 2,000 |
| SPIRE tg64 b0 | 64 | 0.9825 | 90.643 ms | 100.316 ms | 2,784,952 | 5,000 | 2,000 |
| SPIRE tg96 b0 | 96 | 0.9975 | 131.292 ms | 143.238 ms | 2,784,952 | 5,000 | 2,000 |
| SPIRE tg128 b0 | 96 | 0.9975 | 134.271 ms | 145.134 ms | 2,784,952 | 5,000 | 2,000 |
| IVF nprobe96 | 96 | 0.9980 | 37.0 ms | 42.0 ms | 77,760 observed postings | 500 rerank rows | 2,000 top-k rows |

## Slice Decisions

- **Score-bound early termination** is shelved. The funnel shows only `0.18%`
  of SPIRE candidates survive to heap rerank, but pushing a heap bound into
  approximate per-leaf scoring needs a correctness proof that is outside this
  measurement task.
- **Adaptive nprobe collapse** is shelved. tg64/tg96/tg128 share the same
  aggregate candidate count, but recall improves from `0.9825` to `0.9975`, so
  collapsing nprobe from aggregate candidate count would regress the Task 73
  recall floor.
- **Tighter recursive draft** is shelved. The packet identifies discarded
  candidates after scoring, but it does not identify a routing-level predicate
  that rejects those leaves before scoring without reopening recursion
  semantics.

## Validation

- `cargo test -p ecaz-cli spire_pipeline --no-default-features` passed in
  `reviews/task-75/001-candidate-funnel-diagnostics/`.
- `cargo build -p ecaz-cli --no-default-features` passed in
  `reviews/task-75/001-candidate-funnel-diagnostics/` with the existing
  `LoadedDistributedPlacementConfig.path` warning.
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
  passed: `artifacts/cargo-clippy-pg18.log`.
- Source/planning/review-text `git diff --check` passed:
  `artifacts/git-diff-check.log`. A whole-branch check is intentionally not
  cited because committed benchmark transcript artifacts contain generated
  psql table whitespace.
- Source diff file list: `artifacts/code-diff-files.log`.
- No added source `unsafe { ... }` blocks were found:
  `artifacts/no-new-unsafe-scan.log`.

## Outcome

`plan/tasks/75-spire-latency-routing-envelope.md` is marked complete. No SPIRE
routing/runtime behavior change ships from Task 75; the useful output is the
diagnostic surface plus the documented no-slice decision.
