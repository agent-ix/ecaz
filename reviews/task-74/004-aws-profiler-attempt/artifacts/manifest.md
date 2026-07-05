# Task 74 AWS Profiler Attempt Manifest

- head SHA: `512524cd3b588f79094ca7fa6d875b8ce2a425a8`
- task bucket: `reviews/task-74/004-aws-profiler-attempt/`
- timestamp: `2026-05-31T15:35Z` - `2026-05-31T15:40Z`
- lane: AWS Graviton `1m` retained stack (`m7g.2xlarge` DB host)
- storage format: SPIRE `turboquant`; IVF control `pq_fastscan`
- rerank mode: SPIRE rerank width 25; IVF heap rerank width 500
- surface: shared-table AWS benchmark prefixes from
  `benchmarks/task73-74-aws-spire-quality-overhead/`

## Artifact Index

| artifact | command / purpose | key result |
| --- | --- | --- |
| `ssm-precheck-tools.json` | initial tool/db precheck | failed because `ecaz dev sql` defaulted to pgrx port `28818` |
| `ssm-install-perf-cargo-dbcheck.json` | DB row-count check and `dnf install -y perf cargo rust` | both AWS 100k corpus tables reported `100000` rows; `perf` and `cargo` installed |
| `ssm-install-cargo-flamegraph.json` | `cargo install flamegraph` | installed `cargo-flamegraph` and `flamegraph` v0.6.12 |
| `ssm-flamegraph-smoke-hardware.json` | `flamegraph --cmd 'record -F 99 -a -g --call-graph dwarf' ...` | failed before producing SVG |
| `ssm-flamegraph-smoke-cpu-clock.json` | `flamegraph --cmd 'record -F 99 -e cpu-clock -a -g --call-graph dwarf' ...` | failed: `cpu-clock: PMU Hardware doesn't support sampling/overflow-interrupts` |
| `ssm-flamegraph-smoke-task-clock.json` | `flamegraph --cmd 'record -F 99 -e task-clock -a -g --call-graph dwarf' ...` | failed: `task-clock: PMU Hardware doesn't support sampling/overflow-interrupts` |
| `ssm-perf-capability-check.json` | `perf stat -e task-clock -- ecaz bench latency ...` | `perf stat` works, but it is counter evidence only and not a flamegraph |
| `cloud-status-1m-after-profiler-attempt.json` | `ecaz cloud status --profile 1m --json` | returned unknown after direct EC2 stop |
| `ec2-status-after-profiler-attempt.json` | direct EC2 describe-instances | both `1m` instances are `stopped` |

## Key Lines Cited

- `ssm-install-perf-cargo-dbcheck.json`: both corpus row-count queries returned
  `100000`.
- `ssm-install-cargo-flamegraph.json`: installed package `flamegraph v0.6.12`.
- `ssm-flamegraph-smoke-cpu-clock.json`: `PMU Hardware doesn't support
  sampling/overflow-interrupts`.
- `ssm-flamegraph-smoke-task-clock.json`: `PMU Hardware doesn't support
  sampling/overflow-interrupts`.
- `ec2-status-after-profiler-attempt.json`: both instance states are `stopped`.
