# Task 76 Closeout

Reviewer: please review this Task 76 closeout marker.

## Summary

Task 76 closes as a measurement and defaults-policy decision task. The
Intel-local Pareto suite measured SPIRE, IVF, and HNSW controls at 10k and
100k. The canonical local 1M TSV fixture was unavailable, so the closeout does
not promote a 1M-informed default.

No SPIRE default change lands from this packet. The 10k SPIRE points are strong,
but the 100k high-recall points remain much slower than IVF at comparable recall
and tail latency.

## Evidence

- Pareto measurement packet:
  `reviews/task-76/001-pareto-measurement/`
- Benchmark packet:
  `benchmarks/task76-intel-local-spire-pareto/`
- Benchmark manifest:
  `benchmarks/task76-intel-local-spire-pareto/manifest.md`
- Benchmark summary:
  `benchmarks/task76-intel-local-spire-pareto/artifacts/summary.md`
- Defaults follow-up amendment:
  `plan/design/spire-quality-defaults-followup.md`

Key Intel-local 100k rows:

| Surface | Setting | recall@10 | p50 | p95 |
| --- | --- | ---: | ---: | ---: |
| SPIRE | tg16/nprobe16 | 0.8525 | 26.373 ms | 30.224 ms |
| SPIRE | tg32/nprobe32 | 0.9310 | 48.362 ms | 54.251 ms |
| SPIRE | tg64/nprobe64 | 0.9825 | 98.584 ms | 112.208 ms |
| SPIRE | tg96/nprobe96 | 0.9975 | 146.693 ms | 175.128 ms |
| SPIRE | tg128/nprobe128 | 1.0000 | 172.401 ms | 205.287 ms |
| IVF | nprobe96 | 0.9980 | 37.7 ms | 46.5 ms |
| HNSW | ef_search400 | 0.9385 | 15.6 ms | 22.1 ms |

The 100k SPIRE candidate surface also plateaus after nprobe64:

- nprobe64: recall@10 `0.9825`, leaf routes `3,556`, candidates `2,784,952`
- nprobe96: recall@10 `0.9975`, leaf routes `3,556`, candidates `2,784,952`
- nprobe128: recall@10 `1.0000`, leaf routes `3,556`, candidates `2,784,952`

## Decision

Keep the current SPIRE defaults unchanged.

The measured local Intel evidence does not support moving the default to
tg32/tg64/tg96. Those points improve recall at 100k, but the latency and tail
costs are not competitive with IVF at comparable recall. Because the 1M fixture
was unavailable, the packet also cannot justify a cross-size adaptive default
or quality preset as a completed Task 76 slice.

The follow-up design note is amended to point future defaults work at reducing
SPIRE candidate/materialization cost before raising default recall aggression.

## Validation

- Suite audit, dry-run, full run, and report completed in
  `reviews/task-76/001-pareto-measurement/` and
  `benchmarks/task76-intel-local-spire-pareto/`.
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
  passed: `artifacts/cargo-clippy-pg18.log`.
- Source/planning/review-text `git diff --check` passed:
  `artifacts/git-diff-check.log`. A whole-branch check is intentionally not
  cited because committed benchmark transcript artifacts contain generated
  psql table whitespace.
- Source diff file list: `artifacts/code-diff-files.log`.
- No added source `unsafe { ... }` blocks were found:
  `artifacts/no-new-unsafe-scan.log`.
- AWS status after local work is packet-local in
  `reviews/task-76/001-pareto-measurement/artifacts/`: profile `1m` was
  `paused`, profile `10k-medium` was `down`, and both reported `$0.00/hr`
  running cost.

## Outcome

`plan/tasks/76-spire-recall-default-pareto.md` is marked complete. Task 76
ships no default change; it closes the local Intel defaults investigation with
a documented no-change decision and an amended follow-up policy note.
