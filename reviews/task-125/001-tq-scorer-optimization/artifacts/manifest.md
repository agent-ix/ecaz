# Task 125-129 TurboQuant Scorer Optimization Artifacts

- task bucket: `reviews/task-125/001-tq-scorer-optimization`
- code commits:
  - `da1c79a0c Optimize TurboQuant scoring batch path`
  - `9d8ce1da12 Enable sparse TurboQuant bound pruning on NEON`
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

Sparse Task 127 candidate:

```sh
target/release/ecaz --database tqvector_bench --host /Users/peter/.pgrx --port 28818 bench suite run --config reviews/task-125/001-tq-scorer-optimization/artifacts/tq-ivf-suite.json --artifact-dir reviews/task-125/001-tq-scorer-optimization/artifacts/candidate-t127-sparse --manifest-output reviews/task-125/001-tq-scorer-optimization/artifacts/candidate-t127-sparse-suite-manifest.json --results-output reviews/task-125/001-tq-scorer-optimization/artifacts/candidate-t127-sparse-results.jsonl
target/release/ecaz bench suite report --manifest reviews/task-125/001-tq-scorer-optimization/artifacts/candidate-t127-sparse-suite-manifest.json --results-output reviews/task-125/001-tq-scorer-optimization/artifacts/candidate-t127-sparse-results-report.jsonl
```

## Key Results

Baseline -> final candidate:

- 10k recall: `0.9734 -> 0.9734`; latency mean `1.26 ms -> 1.20 ms`; p50 `1.22 ms -> 1.20 ms`; p95 `1.47 ms -> 1.27 ms`; TurboQuant NEON kernel `45.450584 ms -> 45.274061 ms`; index storage `1028.1 B/row -> 1028.1 B/row`; total storage `17703.7 B/row -> 17705.4 B/row`.
- 50k recall: `0.9521 -> 0.9521`; latency mean `2.55 ms -> 2.55 ms`; p50 `2.51 ms -> 2.51 ms`; p95 `2.83 ms -> 2.81 ms`; TurboQuant NEON kernel `75.372179 ms -> 75.735667 ms`; index storage `964.7 B/row -> 964.7 B/row`; total storage `17634.1 B/row -> 17634.9 B/row`.
- 100k recall: `0.8969 -> 0.8969`; latency mean `3.91 ms -> 3.85 ms`; p50 `3.88 ms -> 3.75 ms`; p95 `4.68 ms -> 4.73 ms`; TurboQuant NEON kernel `75.170506 ms -> 74.262683 ms`; index storage `948.2 B/row -> 948.2 B/row`; total storage `17616.8 B/row -> 17617.5 B/row`.

Baseline -> sparse Task 127 candidate:

- 10k recall: `0.9734 -> 0.9734`; latency mean `1.26 ms -> 1.22 ms`; p50 `1.22 ms -> 1.21 ms`; p95 `1.47 ms -> 1.33 ms`; TurboQuant NEON kernel `45.450584 ms -> 46.240597 ms`; index storage `1028.1 B/row -> 1028.1 B/row`; total storage `17703.7 B/row -> 17705.4 B/row`.
- 50k recall: `0.9521 -> 0.9521`; latency mean `2.55 ms -> 2.58 ms`; p50 `2.51 ms -> 2.55 ms`; p95 `2.83 ms -> 2.84 ms`; TurboQuant NEON kernel `75.372179 ms -> 76.955763 ms`; index storage `964.7 B/row -> 964.7 B/row`; total storage `17634.1 B/row -> 17634.9 B/row`.
- 100k recall: `0.8969 -> 0.8969`; latency mean `3.91 ms -> 3.86 ms`; p50 `3.88 ms -> 3.78 ms`; p95 `4.68 ms -> 4.68 ms`; TurboQuant NEON kernel `75.170506 ms -> 74.883089 ms`; index storage `948.2 B/row -> 948.2 B/row`; total storage `17616.8 B/row -> 17617.5 B/row`.

## Task 127 Activation

Task 127 is enabled for TurboQuant when the active ISA is NEON. The bounded scorer checks suffix bounds at 512-dimension checkpoints and at the final dimension so the common no-prune case does not pay a per-32-dimension bound-check cost. Non-NEON sessions return `false` from the bounded TurboQuant batch attempt and continue through the existing unbounded batch scorer.

An earlier all-chunk activation attempt is intentionally not part of the review evidence. It regressed the 10k latency run to `7.42 ms` mean and `284.974750 ms` TurboQuant kernel time before the suite was stopped; the sparse NEON-only candidate above replaces that approach.

## Artifact Index

- Baseline suite: `baseline-suite-manifest.json`, `baseline-results.jsonl`, `baseline-results-report.jsonl`, `baseline/*.log`
- Final suite: `candidate-final-suite-manifest.json`, `candidate-final-results.jsonl`, `candidate-final-results-report.jsonl`, `candidate-final/*.log`
- Sparse Task 127 suite: `candidate-t127-sparse-suite-manifest.json`, `candidate-t127-sparse-results.jsonl`, `candidate-t127-sparse-results-report.jsonl`, `candidate-t127-sparse/*.log`
- Ignored and not committed: `*/truth-cache/`
