# Task 51 Local IVF Sidecar Concurrency Smoke

- head SHA: `4235b7ba12965359453c8229c0bdfa2b651ddf40`
- timestamp: `2026-05-23T17:03:36Z`
- benchmark packet: `benchmarks/task51-local-ivf-sidecar-concurrency-smoke/`
- runner: `ecaz bench suite`
- SuiteConfig: `benchmarks/task51-local-ivf-sidecar-concurrency-smoke/suite.json`
- suite manifest: `benchmarks/task51-local-ivf-sidecar-concurrency-smoke/artifacts/suite-manifest.json`
- results: `benchmarks/task51-local-ivf-sidecar-concurrency-smoke/artifacts/results.jsonl`
- parsed report results: `benchmarks/task51-local-ivf-sidecar-concurrency-smoke/artifacts/results-report.jsonl`
- lane: local PG18 / WSL2 only
- AWS: not used
- competitors: none; IVF/RaBitQ only
- fixture: preserved isolated local 50k prefix `task51_local_50k_ivf_rabitq1_n128_sidecar_off`
- profile: `ec_ivf`
- storage format: `rabitq`
- index reloptions: `nlists=128`, `nprobe=128`, `training_sample_rows=10000`, `quant_bits=1`, `rerank=off`
- candidate frontier: IVF approximate `LIMIT 50`, then sidecar rerank to top 10
- sidecar variants: `f16`, `rabitq8`
- sidecar read modes: `random-id`, `tid-sorted`
- concurrency: `4` sidecar DB fetch/score tasks per variant/read-mode
- query limit: 20 local smoke
- isolated one-index-per-table surface: yes, inherited from `benchmarks/task51-local-ivf-rabitq-sidecar/`

## Commands

Validation:

```text
script -q -c "cargo test -p ecaz-cli --no-default-features sidecar" benchmarks/task51-local-ivf-sidecar-concurrency-smoke/artifacts/cargo-test-sidecar.log
script -q -c "cargo build -p ecaz-cli --no-default-features" benchmarks/task51-local-ivf-sidecar-concurrency-smoke/artifacts/cargo-build-ecaz-cli.log
```

Suite execution:

```text
target/debug/ecaz --log-file benchmarks/task51-local-ivf-sidecar-concurrency-smoke/artifacts/suite-audit.log bench suite audit --config benchmarks/task51-local-ivf-sidecar-concurrency-smoke/suite.json
target/debug/ecaz --host /home/peter/.pgrx --port 28818 --log-file benchmarks/task51-local-ivf-sidecar-concurrency-smoke/artifacts/suite-dry-run.log bench suite run --config benchmarks/task51-local-ivf-sidecar-concurrency-smoke/suite.json --dry-run --manifest-output benchmarks/task51-local-ivf-sidecar-concurrency-smoke/artifacts/suite-manifest.json
target/debug/ecaz --host /home/peter/.pgrx --port 28818 --log-file benchmarks/task51-local-ivf-sidecar-concurrency-smoke/artifacts/suite-run.log bench suite run --config benchmarks/task51-local-ivf-sidecar-concurrency-smoke/suite.json --manifest-output benchmarks/task51-local-ivf-sidecar-concurrency-smoke/artifacts/suite-manifest.json
target/debug/ecaz --log-file benchmarks/task51-local-ivf-sidecar-concurrency-smoke/artifacts/suite-status.log bench suite status --manifest benchmarks/task51-local-ivf-sidecar-concurrency-smoke/artifacts/suite-manifest.json
target/debug/ecaz --log-file benchmarks/task51-local-ivf-sidecar-concurrency-smoke/artifacts/suite-report.log bench suite report --manifest benchmarks/task51-local-ivf-sidecar-concurrency-smoke/artifacts/suite-manifest.json --results-output benchmarks/task51-local-ivf-sidecar-concurrency-smoke/artifacts/results-report.jsonl
```

## Status

`suite-status.log`:

```text
[suite:task51-local-ivf-sidecar-concurrency-smoke] completed=1 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

`cargo-test-sidecar.log`:

```text
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 354 filtered out; finished in 0.00s
```

## Key Results

q=20, k=10, candidate_k=50, nprobe=96, concurrency=4:

| variant | read mode | recall@10 | sidecar I/O p50 | sidecar score p50 | sidecar total p50 | total p50 |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| f16 | random-id | 1.0000 | 34.653 ms | 4.969 ms | 39.615 ms | 180.480 ms |
| f16 | tid-sorted | 1.0000 | 18.743 ms | 4.962 ms | 23.733 ms | 164.994 ms |
| rabitq8 | random-id | 0.9450 | 26.470 ms | 1.092 ms | 27.516 ms | 168.099 ms |
| rabitq8 | tid-sorted | 0.9450 | 4.419 ms | 1.063 ms | 5.552 ms | 145.633 ms |

## Interpretation

- The new `--concurrency` option works through both the CLI and `ecaz bench suite`.
- The local DB path completed successfully at `concurrency=4` for both sidecar variants and both reviewer-requested read modes.
- This is a local functional and concurrency smoke only. It is not a replacement for the requested Graviton 1m sidecar cell.
- As in packet 020, the `sidecar_io_*` columns include DB fetch and `ORDER BY ctid` work for `tid-sorted`; there is no separate product frontier TID-sort metric yet.

## Artifacts

- `suite.json`: checked-in suite config.
- `artifacts/cargo-test-sidecar.log`: focused Rust sidecar/suite tests.
- `artifacts/cargo-build-ecaz-cli.log`: focused CLI build.
- `artifacts/suite-audit.log`: suite audit output.
- `artifacts/suite-dry-run.log`: dry-run showing `--concurrency 4` expansion.
- `artifacts/suite-run.log`: authoritative suite run.
- `artifacts/suite-status.log`: final suite status.
- `artifacts/suite-report.log`: parsed report.
- `artifacts/suite-manifest.json`: structured suite manifest.
- `artifacts/results.jsonl`: structured run results.
- `artifacts/results-report.jsonl`: structured report results.
- `artifacts/sidecar-concurrency-c4-50k-rabitq1-n128-k50.log`: measurement table.
