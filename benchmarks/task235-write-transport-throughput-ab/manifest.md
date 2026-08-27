# Task 235 write-transport throughput A/B

Date: 2026-08-26 (America/Los_Angeles)

## Purpose and preregistration

This packet supplies the only missing measurement requested by
`reviews/task-235/003-2pc-lifecycle-fault-matrix/feedback/2026-08-25-01-reviewer.md`.
It measures the existing `physical_benchmark_insert_throughput_ab` surface at
10k/50k/100k with `skip_single_control: false`. It does not rerun the Task 235
fault matrix or add instrumentation.

- Control extension SHA: `387c2137f85a7950fb243d34bb0adbb7903b5c07`.
- Candidate runtime extension SHA:
  `b802fe3690beb53f9b2695332a163a9d1a8fb56f`. The extension source tree at
  that head is identical to reviewed Task 235 candidate
  `b871d5481376df87c60ae486d68bb78519944c21`; intervening commits contain
  review evidence and suite-runner changes only.
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

cargo clean -p ecaz --release

cargo pgrx install --release \
  --pg-config /home/peter/.ecaz/toolchains/pg18-ssl/bin/pg_config \
  --no-default-features --features pg18

/home/peter/.cargo-target/debug/ecaz bench suite run \
  --config benchmarks/task235-write-transport-throughput-ab/suite.json \
  --only-tag candidate \
  --manifest-output benchmarks/task235-write-transport-throughput-ab/artifacts/suite-manifest-candidate.json \
  --results-output benchmarks/task235-write-transport-throughput-ab/artifacts/suite-results-candidate.jsonl \
  --log-file benchmarks/task235-write-transport-throughput-ab/artifacts/suite-candidate.log
```

## Results

The fixed-harness control ran at `387c2137f85a7950fb243d34bb0adbb7903b5c07`.
The final candidate run used release extension
`b802fe3690beb53f9b2695332a163a9d1a8fb56f` with feature `pg18`; all three
fixture preflights were unanimous. Runtime was 2026-08-26 22:12:27--23:35:31
PDT. All six suite steps succeeded across the two arms.

### Distributed write throughput

Each mean is over five 32-row trials. CV is the sample coefficient of
variation. The 95% confidence interval uses Student's t with four degrees of
freedom. Candidate delta is relative to the matching control mean.

| Scale | Control physical rows/s (CV; 95% CI) | Candidate physical rows/s (CV; 95% CI) | Candidate delta |
|---|---:|---:|---:|
| 10k | 0.868135 (8.89%; 0.772338--0.963933) | 1.011184 (13.09%; 0.846871--1.175498) | +16.48% |
| 50k | 0.507188 (1.70%; 0.496489--0.517887) | 0.580209 (5.68%; 0.539275--0.621142) | +14.40% |
| 100k | 0.353847 (4.04%; 0.336102--0.371593) | 0.386153 (2.35%; 0.374878--0.397427) | +9.13% |

The preregistered decision scale is 50k, corroborated by 100k. Both observed
directions are faster, so this experiment finds no write-throughput regression
from the bounded Task 235 transport. It does not claim that the safety wrapper
improves throughput: the arms are sequential fresh fixtures and the wrapper
has no mechanism expected to create that gain. The 10k intervals overlap and
cannot resolve the effect.

### Required recall, read-latency, storage, and post-insert gates

| Scale | Arm | Distinct recall | Warm mean / p95 ms | Physical generation bytes |
|---|---|---:|---:|---:|
| 10k | control | 0.9990 | 7.70 / 8.60 | 242,868,224 |
| 10k | candidate | 0.9990 | 7.34 / 8.13 | 242,860,032 |
| 50k | control | 0.9540 | 9.13 / 11.50 | 1,243,480,064 |
| 50k | candidate | 0.9545 | 10.30 / 11.80 | 1,243,488,256 |
| 100k | control | 0.9290 | 8.81 / 10.80 | 2,498,207,744 |
| 100k | candidate | 0.9285 | 8.73 / 10.60 | 2,498,215,936 |

Recall differs by at most 0.0005 and storage by one 8 KiB page, both within
fresh-fixture measurement resolution. Read latency is mixed across scales and
is disclosed rather than attributed to the write-only change. The candidate
post-insert exact-recall deficit was -0.003762 / -0.011946 / -0.005663 at
10k/50k/100k; every scale passed the pre-existing 0.015 absolute gate.

## Provenance correction and cleanup

An initial candidate invocation was rejected from evidence because fixture
preflight identified the installed extension as control SHA `387c2137f`; a
second invocation was stopped at the same preflight after Cargo reused that
stale release artifact. No measurements from either invocation are cited. The
interrupted 1.4 GB run directory was removed after confirming that no fixture
process remained. Only package `ecaz` release artifacts were then refreshed in
the shared `CARGO_TARGET_DIR` (590.9 MiB); no alternate target or worktree was
created. The final installed backend SHA-256 was
`de0a17e94d68d069caa0f710dbe015477dffa6b4bb14537907074ee7c20aa49f`.

The accepted suite cleanup removed the final candidate 10k, 50k, and 100k run
directories after durable artifact capture. PostgreSQL clusters, truth caches,
node logs, and fixture console exhaust are not evidence and are not committed.

## Artifacts

- `artifacts/suite-manifest-control.json` and
  `artifacts/suite-results-control.jsonl`: committed fixed-harness control
  source of truth.
- `artifacts/suite-manifest-candidate.json` and
  `artifacts/suite-results-candidate.jsonl`: final candidate commands,
  timestamps, success states, parsed metrics, and cleanup tags.
- `artifacts/suite-control.log` and `artifacts/suite-candidate.log`: suite
  runner summaries.
- `artifacts/run/{control,candidate}-{10k,50k,100k}/distann-multinode-summary.log`:
  per-trial write values and the cited recall, latency, storage, work, and
  post-insert gate records.
- The matching `physical-head-membership.json`,
  `physical-production-recall.log`, `physical-production-latency.log`, and
  `physical-production-predictions.json` files are the compact result evidence
  retained for each arm and scale.
