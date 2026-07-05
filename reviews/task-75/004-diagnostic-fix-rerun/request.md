---
task: 75
topic: diagnostic-fix-rerun
agent: codex
role: coder
model: GPT-5
date: 2026-05-31
---

# Task 75 Diagnostic Fix Rerun

## Request

Please review the diagnostic fix and the rerun evidence for the reviewer finding in `reviews/task-75/001-candidate-funnel-diagnostics/feedback/2026-05-31-01-reviewer.md`.

The root cause was that `scan::collect_single_level_scan_placement_diagnostics` always used recursive routing. Production scan uses top-graph routing when `top_graph_search_list_size` is enabled, so the diagnostic was counting a different route set. The fix threads `options.top_graph_plan()` through the diagnostic path and, when enabled, routes through `route_top_graph_object_to_leaf_routes` before collecting the same quantized leaf candidates with the diagnostic observer.

Code commit: `f5dea05fcd32e0b871e4b5815371109fb3123883`

## Evidence

Benchmark packet: `benchmarks/task75-intel-local-routing-envelope-diagnostic-fix-rerun/`

The rerun used `ecaz bench suite` with the checked-in packet config at `benchmarks/task75-intel-local-routing-envelope-diagnostic-fix-rerun/suite.json`.

Key corrected results:

| setting | suite candidate_sum | funnel candidate_sum | per-query min | per-query max | recall@10 | p50 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| tg16 / nprobe16 | 2,514,557 | 2,514,557 | 9,146 | 15,406 | 0.8525 | 27.536 ms |
| tg32 / nprobe32 | 5,165,224 | 5,165,224 | 22,757 | 29,825 | 0.9310 | 50.550 ms |
| tg64 / nprobe64 | 10,420,357 | 10,420,357 | 47,310 | 56,751 | 0.9825 | 92.957 ms |
| tg96 / nprobe96 | 15,506,227 | 15,506,227 | 72,629 | 81,736 | 0.9975 | 132.203 ms |
| tg128 / effective nprobe96 | 15,506,227 | 15,506,227 | 72,629 | 81,736 | 0.9975 | 132.196 ms |

The diagnostic is no longer query-invariant at tg32+; route count remains fixed by `nprobe`, but the selected leaves and their candidate cardinalities vary by query. The diagnostic candidate sums now match the suite runner counters exactly for every SPIRE setting.

## Validation

- `cargo test -p ecaz-cli spire_pipeline --no-default-features`
  - log: `artifacts/cargo-test-spire-pipeline.log`
- `target/debug/ecaz bench suite audit ...`
  - log: `benchmarks/task75-intel-local-routing-envelope-diagnostic-fix-rerun/artifacts/suite-audit.log`
- `target/debug/ecaz bench suite run --dry-run ...`
  - log: `benchmarks/task75-intel-local-routing-envelope-diagnostic-fix-rerun/artifacts/suite-dry-run.log`
- `target/debug/ecaz bench suite run ...`
  - log: `benchmarks/task75-intel-local-routing-envelope-diagnostic-fix-rerun/artifacts/suite-run.log`
- `target/debug/ecaz bench suite report ...`
  - report: `benchmarks/task75-intel-local-routing-envelope-diagnostic-fix-rerun/artifacts/suite-report.md`

## Notes

This packet supersedes the candidate counts cited by `reviews/task-75/001-candidate-funnel-diagnostics/request.md`. The old 2.78M high-recall plateau was a diagnostic artifact. The corrected high-recall candidate envelope is 15.5M candidates over 200 queries at tg96/tg128.
