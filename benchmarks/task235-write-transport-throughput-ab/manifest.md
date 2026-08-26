# Task 235 write-transport throughput A/B

Date: 2026-08-26 (America/Los_Angeles)

## Purpose and preregistration

This packet supplies the only missing measurement requested by
`reviews/task-235/003-2pc-lifecycle-fault-matrix/feedback/2026-08-25-01-reviewer.md`.
It measures the existing `physical_benchmark_insert_throughput_ab` surface at
10k/50k/100k with `skip_single_control: false`. It does not rerun the Task 235
fault matrix or add instrumentation.

- Control extension SHA: `387c2137f85a7950fb243d34bb0adbb7903b5c07`.
- Candidate extension SHA: `b871d5481376df87c60ae486d68bb78519944c21`.
- Host/toolchain: local Intel development host, PostgreSQL 18 release
  extension, `/home/peter/.ecaz/toolchains/pg18-ssl/bin`.
- Transport: fresh three-owner physical fixtures using verify-full mutual TLS.
- Corpus: staged `ec_real_10k`, `ec_real_50k`, and `ec_real_100k` under
  `data/staged-current/`; corpus data and PGDATA are not committed.
- Surface: isolated one-index-per-owner physical generation plus the normal
  monolithic control required by the insert-throughput A/B. The config sets
  `skip_single_control: false` explicitly and `skip_single_benchmark: true`, so
  the control is built and measured for writes without duplicating its read
  benchmark arm.
- Run order: all control scales, install the exact candidate, then all
  candidate scales. Each step uses a distinct run directory and port range.
- Decision metric: report `physical_rows_per_second`,
  `control_rows_per_second`, and `physical_over_control` for each arm and scale,
  plus candidate-vs-control deltas. No performance threshold is invented here;
  the outside reviewer owns the final Task 235 disposition.

The preregistered `suite.json` SHA-256 is
`86159b17b6646f587c4732397f9494795010791664224bc14ece835454025f97`.
`ecaz bench suite audit` passed all six steps before either arm ran.

## Commands

```text
/home/peter/.cargo-target/debug/ecaz bench suite audit \
  --config benchmarks/task235-write-transport-throughput-ab/suite.json \
  --log-file benchmarks/task235-write-transport-throughput-ab/artifacts/suite-audit.log

/home/peter/.cargo-target/debug/ecaz bench suite run \
  --config benchmarks/task235-write-transport-throughput-ab/suite.json \
  --only-tag control \
  --manifest-output benchmarks/task235-write-transport-throughput-ab/artifacts/suite-manifest-control.json \
  --results-output benchmarks/task235-write-transport-throughput-ab/artifacts/suite-results-control.jsonl \
  --log-file benchmarks/task235-write-transport-throughput-ab/artifacts/suite-control.log

/home/peter/.cargo-target/debug/ecaz bench suite run \
  --config benchmarks/task235-write-transport-throughput-ab/suite.json \
  --only-tag candidate \
  --manifest-output benchmarks/task235-write-transport-throughput-ab/artifacts/suite-manifest-candidate.json \
  --results-output benchmarks/task235-write-transport-throughput-ab/artifacts/suite-results-candidate.jsonl \
  --log-file benchmarks/task235-write-transport-throughput-ab/artifacts/suite-candidate.log
```

## Results

Pending at preregistration. This section will be filled from the committed
suite results and per-step summaries after both exact-SHA runs complete.
