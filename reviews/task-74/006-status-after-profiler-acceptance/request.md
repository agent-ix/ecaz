# Task 74 Status After Profiler Acceptance

Reviewer: this packet records the Task 74 status update after reviewer
acceptance of the Intel-local profiler baseline in
`reviews/task-74/005-intel-profiler-baseline/feedback/2026-05-31-01-reviewer.md`.

## Summary

Task 74 is now marked complete in `plan/tasks/74-spire-leaf-scan-overhead.md`
and `plan/tasks/README.md`.

The accepted profiler packet establishes:

- Intel-local `perf`/flamegraph evidence satisfies the profiler gate.
- Identifiable SPIRE-specific orchestration above scoring is about `4.9%`,
  below the task's `~10%` stop-condition floor.
- No Phase 2 SPIRE scan slices are warranted for this task.

## Fresh AWS Refresh

Per operator instruction to continue to AWS testing, I also ran a fresh AWS
Graviton `1m` refresh packet after the Intel profile was accepted:

`benchmarks/task73-74-aws-spire-quality-overhead-refresh-20260531/`

The suite completed `8` steps with `0` failures and `0` missing artifacts.
Key refreshed results:

| Surface | nprobe | recall@10 | p50 | p95 | p99 |
| --- | ---: | ---: | ---: | ---: | ---: |
| SPIRE default `tg16/b0` | 16 | 0.8525 | 33.879 ms | 122.728 ms | 246.520 ms |
| SPIRE high-recall `tg128/b0` | 96 | 0.9975 | 134.458 ms | 149.487 ms | 587.682 ms |
| SPIRE ceiling `tg128/b0` | 128 | 1.0000 | 159.866 ms | 160.505 ms | 161.241 ms |
| IVF control | 96 | 0.9980 | 28.7 ms | 30.4 ms | 30.9 ms |
| IVF control | 128 | 1.0000 | 35.2 ms | 36.7 ms | 37.2 ms |

The refreshed AWS p50 SPIRE/IVF ratio is `4.69x` at nprobe `96` and `4.54x`
at nprobe `128`, matching the previously documented AWS shape.

The `1m` instances were stopped after the run. The packet records the final
cloud status as `state=paused`, DB instance `stopped`, running compute
`$0.00/hr`.

## Files

- Task status: `plan/tasks/74-spire-leaf-scan-overhead.md`
- Task index: `plan/tasks/README.md`
- AWS refresh manifest:
  `benchmarks/task73-74-aws-spire-quality-overhead-refresh-20260531/manifest.md`
- AWS suite config:
  `benchmarks/task73-74-aws-spire-quality-overhead-refresh-20260531/suite.json`
- AWS raw artifacts:
  `benchmarks/task73-74-aws-spire-quality-overhead-refresh-20260531/artifacts/`

## Validation

- `ecaz bench suite run --dry-run` passed for the AWS refresh config.
- `ecaz cloud bench --profile 1m ... --ecaz-bin /usr/local/bin/ecaz` completed
  and synced packet-local artifacts.
- `ecaz bench suite report` completed and wrote `results-report.jsonl`.
- AWS `1m` stack was stopped after the refresh.
