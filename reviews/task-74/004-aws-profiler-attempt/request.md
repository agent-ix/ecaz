# Task 74 AWS Profiler Attempt

Reviewer: please review this profiler-attempt packet for Task 74.

## Summary

This packet attempts to close Task 74's missing profiler evidence on the
retained AWS Graviton `1m` stack after the local M5 evidence and AWS benchmark
gate had already shown material SPIRE-vs-IVF overhead.

The attempt did not produce a usable `cargo flamegraph` SVG. The host accepted
`perf stat`, but `perf record` sampling failed for hardware, `cpu-clock`, and
`task-clock` events with the same virtualized-host PMU limitation:
`PMU Hardware doesn't support sampling/overflow-interrupts. Try 'perf stat'`.

Task 74 therefore remains pending profiler evidence and reviewer approval.

## What Ran

1. Restarted the retained `1m` DB/coordinator instances.
2. Verified the retained 100k SPIRE high-recall and IVF control tables exist:
   both `task7374_aws_spire_highrecall_tg128_b0_corpus` and
   `task7374_aws_ivf_control_corpus` reported `100000` rows.
3. Installed the missing profiler toolchain on the AWS DB host:
   `perf`, `cargo`, `rust`, and `cargo-flamegraph` / `flamegraph`.
4. Tried `flamegraph` against the high-recall SPIRE latency command using:
   - hardware sampling
   - software `cpu-clock`
   - software `task-clock`
5. Verified `perf stat -e task-clock` can run the SPIRE latency command, but
   this is not a flamegraph and does not satisfy the Task 74 profiler gate.
6. Stopped both AWS instances after the attempt.

## Key Evidence

- Tool install + table row counts:
  `artifacts/ssm-install-perf-cargo-dbcheck.json`
- `cargo-flamegraph` install:
  `artifacts/ssm-install-cargo-flamegraph.json`
- Hardware sampling failure:
  `artifacts/ssm-flamegraph-smoke-hardware.json`
- `cpu-clock` sampling failure:
  `artifacts/ssm-flamegraph-smoke-cpu-clock.json`
- `task-clock` sampling failure:
  `artifacts/ssm-flamegraph-smoke-task-clock.json`
- `perf stat` capability check:
  `artifacts/ssm-perf-capability-check.json`
- AWS instances stopped:
  `artifacts/ec2-status-after-profiler-attempt.json`

## Commands

The successful database/toolchain precheck used:

```console
sudo -u postgres /usr/local/bin/ecaz dev sql --pg 18 --db postgres \
  --socket-dir /var/run/postgresql --port 5432 --raw \
  --sql "SELECT count(*) FROM task7374_aws_spire_highrecall_tg128_b0_corpus; SELECT count(*) FROM task7374_aws_ivf_control_corpus;"
sudo dnf install -y perf cargo rust
cargo install flamegraph
```

The flamegraph attempts used this shape:

```console
/root/.cargo/bin/flamegraph --root \
  --cmd 'record -F 99 -e task-clock -a -g --call-graph dwarf' \
  --ignore-status \
  --output reviews/task-74/004-aws-flamegraph-profile/artifacts/smoke-spire-flamegraph.svg \
  -- /usr/local/bin/ecaz bench latency \
    --database postgres --host /var/run/postgresql --port 5432 --user postgres \
    --prefix task7374_aws_spire_highrecall_tg128_b0 \
    --profile ec_spire --k 10 --sweep 96 --iterations 10 \
    --concurrency 1 --force-index
```

## Outcome

This packet is evidence of a real profiling attempt, not completion evidence.
Task 74 still needs either:

- a profiler-capable host for `samply` / `cargo flamegraph` at the Task 73
  high-recall SPIRE point plus IVF control; or
- reviewer-approved amendment accepting the suite-visible pipeline counters as
  the substitute for the explicit profiler requirement.
