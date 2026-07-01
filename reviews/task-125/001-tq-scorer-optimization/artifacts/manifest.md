# Task 125-129 TurboQuant Scorer Optimization Artifacts

- task bucket: `reviews/task-125/001-tq-scorer-optimization`
- code commit: `da1c79a0c Optimize TurboQuant scoring batch path`
- base commit: `6799686af9e9adf13332bd4ec6e19b60e7ceb80e`
- lane: local PG18, aarch64/NEON, `tqvector_bench`
- fixture: staged real corpus, `ec_ivf`, `storage_format=turboquant`, `bits=4`, `seed=42`, `nprobe=32`
- runner: `target/release/ecaz bench suite`
- timestamp: 2026-07-01T06:57:24Z
- isolation: existing one-index-per-prefix tables were reused; load steps skipped reload/rebuild when reloptions matched.

## Suite Config

- `tq-ivf-suite.json`
- sha256: `b258f49b2e712dcfd60e2d991656e6d338e37dec26b112158cf4075fbdc7e0ad`

## Commands

Baseline:

```sh
target/release/ecaz --database tqvector_bench --host /Users/peter/.pgrx --port 28818 bench suite run --config reviews/task-125/001-tq-scorer-optimization/artifacts/tq-ivf-suite.json --artifact-dir reviews/task-125/001-tq-scorer-optimization/artifacts/baseline --manifest-output reviews/task-125/001-tq-scorer-optimization/artifacts/baseline-suite-manifest.json --results-output reviews/task-125/001-tq-scorer-optimization/artifacts/baseline-results.jsonl
target/release/ecaz bench suite report --manifest reviews/task-125/001-tq-scorer-optimization/artifacts/baseline-suite-manifest.json --results-output reviews/task-125/001-tq-scorer-optimization/artifacts/baseline-results-report.jsonl
```

Final candidate:

```sh
target/release/ecaz --database tqvector_bench --host /Users/peter/.pgrx --port 28818 bench suite run --config reviews/task-125/001-tq-scorer-optimization/artifacts/tq-ivf-suite.json --artifact-dir reviews/task-125/001-tq-scorer-optimization/artifacts/candidate-final --manifest-output reviews/task-125/001-tq-scorer-optimization/artifacts/candidate-final-suite-manifest.json --results-output reviews/task-125/001-tq-scorer-optimization/artifacts/candidate-final-results.jsonl
target/release/ecaz bench suite report --manifest reviews/task-125/001-tq-scorer-optimization/artifacts/candidate-final-suite-manifest.json --results-output reviews/task-125/001-tq-scorer-optimization/artifacts/candidate-final-results-report.jsonl
```

## Key Results

Baseline -> final candidate:

- 10k recall: `0.9734 -> 0.9734`; latency mean `1.26 ms -> 1.20 ms`; p50 `1.22 ms -> 1.20 ms`; p95 `1.47 ms -> 1.27 ms`; TurboQuant NEON kernel `45.450584 ms -> 45.274061 ms`; index storage `1028.1 B/row -> 1028.1 B/row`; total storage `17703.7 B/row -> 17705.4 B/row`.
- 50k recall: `0.9521 -> 0.9521`; latency mean `2.55 ms -> 2.55 ms`; p50 `2.51 ms -> 2.51 ms`; p95 `2.83 ms -> 2.81 ms`; TurboQuant NEON kernel `75.372179 ms -> 75.735667 ms`; index storage `964.7 B/row -> 964.7 B/row`; total storage `17634.1 B/row -> 17634.9 B/row`.
- 100k recall: `0.8969 -> 0.8969`; latency mean `3.91 ms -> 3.85 ms`; p50 `3.88 ms -> 3.75 ms`; p95 `4.68 ms -> 4.73 ms`; TurboQuant NEON kernel `75.170506 ms -> 74.262683 ms`; index storage `948.2 B/row -> 948.2 B/row`; total storage `17616.8 B/row -> 17617.5 B/row`.

## Rejected Pruning Activation

Task 127 bounded TurboQuant scoring primitives were implemented and unit-tested, but production scan activation was not kept enabled. A final activation attempt regressed the 10k latency run to `7.42 ms` mean and `284.974750 ms` TurboQuant kernel time before the suite was stopped. The code commit therefore keeps TurboQuant out of `uses_score_bound_pruning()` while preserving the bounded primitive for future gated work.

## Artifact Index

- Baseline suite: `baseline-suite-manifest.json`, `baseline-results.jsonl`, `baseline-results-report.jsonl`, `baseline/*.log`
- Final suite: `candidate-final-suite-manifest.json`, `candidate-final-results.jsonl`, `candidate-final-results-report.jsonl`, `candidate-final/*.log`
- Ignored and not committed: `*/truth-cache/`
