# Artifact Manifest: RaBitQ Suite Evidence

- head SHA: `c5b37ce0c38d0f23292dfa2595549c2c88a821c4`
- baseline SHA: `7a8388efdf9519801eb121017b51a082366d1359`
- task bucket: `reviews/task-78/`
- packet path: `reviews/task-78/002-rabitq-suite-evidence/`
- lane: Intel-local PG18, 100k real corpus, 200 query rows
- fixture: `/home/peter/dev/ecaz/target/real-corpus/staged-task50/`
- isolated surface: one benchmark database per lane
- lanes:
  - `baseline`: parent RaBitQ, `task78_spire_rabitq_baseline`
  - `current`: current RaBitQ cutoff slice, `task78_spire_rabitq_current`
  - `turboquant-current`: current TurboQuant comparison, `task78_spire_turboquant_current`

## Suite Configs

- `../suite-rabitq-baseline.json`
- `../suite-rabitq-current.json`
- `../suite-turboquant-current.json`

All three configs were audited with `ecaz bench suite audit`; logs:

- `audit-rabitq-baseline.log`
- `audit-rabitq-current.log`
- `audit-turboquant-current.log`

## Install / Restart Evidence

- `install-current-ecaz-pg18.log`: current extension installed before the current RaBitQ suite; installed backend SHA256 `d781b7af4e4a7734f7a2711f9583133fd5333ef60bde47b8cb14d35fb7a45817`.
- `install-baseline-ecaz-pg18.log`: parent extension installed before the baseline suite; installed backend SHA256 `5a5cea5122964390050b1d0384e95a17dbd093329453a39f920ef61218300c0b`.
- `reinstall-current-after-baseline-ecaz-pg18.log`: current extension reinstalled before the TurboQuant comparison.
- `restart-current-pg18.log`, `restart-baseline-pg18.log`, `restart-current-after-baseline-pg18.log`: PG18 restart command logs.

## Commands

Current RaBitQ:

```sh
target/debug/ecaz --log-file reviews/task-78/002-rabitq-suite-evidence/artifacts/current/suite-run.log --database task78_spire_rabitq_current --host /home/peter/.pgrx --port 28818 bench suite run --config reviews/task-78/002-rabitq-suite-evidence/suite-rabitq-current.json --manifest-output reviews/task-78/002-rabitq-suite-evidence/artifacts/current/suite-manifest.json --results-output reviews/task-78/002-rabitq-suite-evidence/artifacts/current/results.jsonl
target/debug/ecaz --log-file reviews/task-78/002-rabitq-suite-evidence/artifacts/current/suite-status.log bench suite status --manifest reviews/task-78/002-rabitq-suite-evidence/artifacts/current/suite-manifest.json
target/debug/ecaz --log-file reviews/task-78/002-rabitq-suite-evidence/artifacts/current/suite-report.log bench suite report --manifest reviews/task-78/002-rabitq-suite-evidence/artifacts/current/suite-manifest.json --results-output reviews/task-78/002-rabitq-suite-evidence/artifacts/current/report-results.jsonl
```

Parent RaBitQ baseline:

```sh
/tmp/ecaz-task78-baseline/target/debug/ecaz --log-file /home/peter/dev/ecaz/reviews/task-78/002-rabitq-suite-evidence/artifacts/baseline/suite-run.log --database task78_spire_rabitq_baseline --host /home/peter/.pgrx --port 28818 bench suite run --config /home/peter/dev/ecaz/reviews/task-78/002-rabitq-suite-evidence/suite-rabitq-baseline.json --manifest-output /home/peter/dev/ecaz/reviews/task-78/002-rabitq-suite-evidence/artifacts/baseline/suite-manifest.json --results-output /home/peter/dev/ecaz/reviews/task-78/002-rabitq-suite-evidence/artifacts/baseline/results.jsonl
/tmp/ecaz-task78-baseline/target/debug/ecaz --log-file /home/peter/dev/ecaz/reviews/task-78/002-rabitq-suite-evidence/artifacts/baseline/suite-status.log bench suite status --manifest /home/peter/dev/ecaz/reviews/task-78/002-rabitq-suite-evidence/artifacts/baseline/suite-manifest.json
/tmp/ecaz-task78-baseline/target/debug/ecaz --log-file /home/peter/dev/ecaz/reviews/task-78/002-rabitq-suite-evidence/artifacts/baseline/suite-report.log bench suite report --manifest /home/peter/dev/ecaz/reviews/task-78/002-rabitq-suite-evidence/artifacts/baseline/suite-manifest.json --results-output /home/peter/dev/ecaz/reviews/task-78/002-rabitq-suite-evidence/artifacts/baseline/report-results.jsonl
```

Current TurboQuant comparison:

```sh
target/debug/ecaz --log-file reviews/task-78/002-rabitq-suite-evidence/artifacts/turboquant-current/suite-run.log --database task78_spire_turboquant_current --host /home/peter/.pgrx --port 28818 bench suite run --config reviews/task-78/002-rabitq-suite-evidence/suite-turboquant-current.json --manifest-output reviews/task-78/002-rabitq-suite-evidence/artifacts/turboquant-current/suite-manifest.json --results-output reviews/task-78/002-rabitq-suite-evidence/artifacts/turboquant-current/results.jsonl
target/debug/ecaz --log-file reviews/task-78/002-rabitq-suite-evidence/artifacts/turboquant-current/suite-status.log bench suite status --manifest reviews/task-78/002-rabitq-suite-evidence/artifacts/turboquant-current/suite-manifest.json
target/debug/ecaz --log-file reviews/task-78/002-rabitq-suite-evidence/artifacts/turboquant-current/suite-report.log bench suite report --manifest reviews/task-78/002-rabitq-suite-evidence/artifacts/turboquant-current/suite-manifest.json --results-output reviews/task-78/002-rabitq-suite-evidence/artifacts/turboquant-current/report-results.jsonl
```

## Suite Status

- `current/suite-status.log`: completed `9`, failed `0`, skipped `0`, missing artifacts `0`, stale `0`.
- `baseline/suite-status.log`: completed `9`, failed `0`, skipped `0`, missing artifacts `0`, stale `0`.
- `turboquant-current/suite-status.log`: completed `9`, failed `0`, skipped `0`, missing artifacts `0`, stale `0`.

## Key Latency / Recall Rows

Source: `latency-recall-summary.json`.

| lane | storage | nprobe | recall@10 | p50 | p95 | p99 |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| baseline | rabitq | 64 | 0.9825 | 41.597 ms | 45.084 ms | 50.998 ms |
| current | rabitq | 64 | 0.9825 | 41.757 ms | 52.954 ms | 62.954 ms |
| turboquant-current | turboquant | 64 | 0.9825 | 89.144 ms | 96.880 ms | 102.124 ms |
| baseline | rabitq | 96 | 0.9975 | 60.881 ms | 70.157 ms | 74.160 ms |
| current | rabitq | 96 | 0.9975 | 60.256 ms | 73.437 ms | 95.535 ms |
| turboquant-current | turboquant | 96 | 0.9975 | 129.835 ms | 140.492 ms | 150.275 ms |
| baseline | rabitq | 128 | 1.0000 | 73.774 ms | 82.751 ms | 91.681 ms |
| current | rabitq | 128 | 1.0000 | 74.951 ms | 88.697 ms | 101.919 ms |
| turboquant-current | turboquant | 128 | 1.0000 | 167.193 ms | 176.838 ms | 188.629 ms |

## Funnel / Stage Attribution

Source: `funnel-attribution-summary.json`.

| lane | nprobe | candidates | retained | returned | score p50 | object-read p50 | materialize p50 | heap-append p50 | score share |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| baseline | 64 | 10,420,357 | 5,000 | 2,000 | 20.705 ms | 10.290 ms | 1.779 ms | 1.261 ms | 87.2% |
| current | 64 | 10,420,357 | 5,000 | 2,000 | 22.490 ms | 9.606 ms | 1.742 ms | 1.298 ms | 88.1% |
| turboquant-current | 64 | 10,420,357 | 5,000 | 2,000 | 67.963 ms | 9.961 ms | 1.751 ms | 1.314 ms | 95.7% |
| baseline | 96 | 15,506,227 | 5,000 | 2,000 | 31.384 ms | 15.707 ms | 2.658 ms | 1.845 ms | 87.5% |
| current | 96 | 15,506,227 | 5,000 | 2,000 | 33.768 ms | 14.877 ms | 2.609 ms | 1.959 ms | 88.1% |
| turboquant-current | 96 | 15,506,227 | 5,000 | 2,000 | 101.357 ms | 14.851 ms | 2.620 ms | 1.953 ms | 95.7% |
| baseline | 128 | 20,000,000 | 5,000 | 2,000 | 39.315 ms | 19.196 ms | 3.373 ms | 2.360 ms | 87.3% |
| current | 128 | 20,000,000 | 5,000 | 2,000 | 42.446 ms | 19.434 ms | 3.319 ms | 2.479 ms | 88.0% |
| turboquant-current | 128 | 20,000,000 | 5,000 | 2,000 | 130.979 ms | 21.495 ms | 3.348 ms | 2.494 ms | 95.7% |

## Decision

The bounded RaBitQ cutoff slice does not clear Task 78's `>=10%` matched-recall p50 gate. Candidate counts, retained counts, and returned counts are unchanged against the RaBitQ parent baseline at all measured points, and current RaBitQ p50 is within noise or slightly worse:

- nprobe64: `41.597 ms` baseline -> `41.757 ms` current (`-0.4%`, worse)
- nprobe96: `60.881 ms` baseline -> `60.256 ms` current (`+1.0%`, better)
- nprobe128: `73.774 ms` baseline -> `74.951 ms` current (`-1.6%`, worse)

RaBitQ remains the correct primary/default direction for the validated lane relative to TurboQuant in this matrix: current RaBitQ is `53.2%`, `53.6%`, and `55.2%` lower p50 than current TurboQuant at nprobe64/96/128 with identical recall rows.

Task 78 P0 is therefore shelved with evidence rather than accepted as a landed latency optimization. The next useful work is not another bounded-heap cutoff; it is a routing/candidate-selection slice that reduces the `10.4M` / `15.5M` / `20.0M` scored candidate surfaces while preserving the Task 73/75 recall floor.
