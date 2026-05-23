# Task 50 Postchange Benchmark Smoke Artifacts

- head SHA: `34e7d88c13fca8bc94d57bebc64f4cb495ecbf72`
- task bucket: `reviews/task-50/028-postchange-benchmark-smoke/`
- timestamp: `2026-05-20T08:22:36-07:00`
- host: local WSL2 PG18 pgrx instance, socket `/home/peter/.pgrx`, port `28818`
- database: `tqvector_bench`
- fixture: `ec_real_10k`
- scope: narrow postchange smoke for major issues only; not a full regression matrix

## Configs

- `suite.json` copies the Task 50 baseline suite with this packet's artifact directory.
- `suite-tight.json` is the authoritative smoke config. It uses the same selected 10k load/recall/latency/storage steps, with `defaults.iterations = 50` so latency remains a smoke run.

## Commands

Install current branch extension into PG18:

```text
cargo pgrx install --release --features bench --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config
```

Audit:

```text
target/release/ecaz bench suite audit --config reviews/task-50/028-postchange-benchmark-smoke/suite-tight.json --host /home/peter/.pgrx --port 28818 --log-file reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/suite-tight-audit.log
```

Run:

```text
target/release/ecaz bench suite run --config reviews/task-50/028-postchange-benchmark-smoke/suite-tight.json --host /home/peter/.pgrx --port 28818 --only load-10k-ivfrabitq --only recall-10k-ivfrabitq --only latency-10k-ivfrabitq --only storage-10k-ivfrabitq --only load-10k-spirerabitq --only recall-10k-spirerabitq --only latency-10k-spirerabitq --only storage-10k-spirerabitq --only load-10k-hnsw --only recall-10k-hnsw --only latency-10k-hnsw --only storage-10k-hnsw --only load-10k-diskann --only recall-10k-diskann --only latency-10k-diskann --only storage-10k-diskann --continue-on-error --log-file reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/suite-tight-smoke.driver.log
```

Status and report:

```text
target/release/ecaz bench suite status --manifest reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/suite-manifest.json --log-file reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/suite-tight-status.log
target/release/ecaz bench suite report --manifest reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/suite-manifest.json --results-output reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/results-report.jsonl --log-file reviews/task-50/028-postchange-benchmark-smoke/artifacts/tight/suite-tight-report.md
```

## Result Summary

`suite-tight-status.log`:

```text
[suite:task-50-postchange-smoke] completed=16 failed=0 skipped=57 dry_run=0 missing_artifacts=0 stale=0
```

Selected cells:

| Surface | Load | Recall | Latency | Storage |
| --- | --- | --- | --- | --- |
| `10k-ivfrabitq` | succeeded | succeeded | succeeded | succeeded |
| `10k-spirerabitq` | succeeded | succeeded | succeeded | succeeded |
| `10k-hnsw` | succeeded | succeeded | succeeded | succeeded |
| `10k-diskann` | succeeded | succeeded | succeeded | succeeded |

Key terminal points from `suite-tight-report.md` / `results-report.jsonl`:

| Surface | Recall terminal point | Latency terminal point | Index storage |
| --- | --- | --- | --- |
| `ivfrabitq` | nprobe 64: recall@k `0.9790`, mean q-time `25.54 ms` | nprobe 64: mean `26.0 ms`, p95 `35.1 ms`, count `50` | `9.8 MiB` |
| `spirerabitq` | nprobe 64: recall@k `1.0000`, mean q-time `240.15 ms` | nprobe 64: mean `241.7 ms`, p95 `258.4 ms`, count `50` | `9.0 MiB` |
| `hnsw` | ef_search 400: recall@k `0.9720`, mean q-time `4.54 ms` | ef_search 400: mean `4.32 ms`, p95 `4.87 ms`, count `50` | `25.3 MiB` total indexes |
| `diskann` | list_size 800: recall@k `0.9975`, mean q-time `9.91 ms` | list_size 800: mean `10.1 ms`, p95 `12.4 ms`, count `50` | `4.7 MiB` |

## Diagnostic Attempts

- `suite-smoke.missing-host.driver.log`, `suite-manifest-missing-host.json`, and `results-missing-host.jsonl` capture the first run without explicit `--host /home/peter/.pgrx --port 28818`; every selected step failed at connection setup with `both host and hostaddr are missing`.
- `suite-smoke.driver.log` and `suite-manifest.json` capture the rerun with the correct socket but the original 1000-iteration latency defaults. It completed IVF/RabitQ and SPIRE/RabitQ load/recall, then was stopped during SPIRE/RabitQ latency because that was too wide for the requested narrow smoke.

## Artifact Index

- `artifacts/tight/suite-manifest.json` - authoritative suite manifest.
- `artifacts/tight/results.jsonl` - normalized result rows written by suite run.
- `artifacts/tight/results-report.jsonl` - normalized result rows parsed during report generation.
- `artifacts/tight/suite-tight-status.log` - status summary.
- `artifacts/tight/suite-tight-report.md` - markdown report.
- `artifacts/tight/*10k*.log` - packet-local load, recall, latency, and storage logs for the selected smoke cells.
