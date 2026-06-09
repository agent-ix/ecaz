# Task 87 Packet 024 Artifact Manifest

- head SHA: `fc8d08286fa4f90fecb433f4ca1750dd7adbaafd`
- task bucket: `reviews/task-87/`
- packet path: `reviews/task-87/024-phase7-spire-real10k-p99-rerun/`
- timestamp: `2026-06-08T16:40:23-07:00`
- runner: `ecaz bench suite`
- suite config: `reviews/task-87/024-phase7-spire-real10k-p99-rerun/phase7-spire-real10k-p99-rerun-suite.json`
- suite status: `completed=3 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`
- database: `postgres`
- socket dir: `/home/peter/.pgrx`
- port: `28818`
- storage surface: existing packet 021 real10k SPIRE TurboQuant index

## Commands

Audit:

```sh
target/debug/ecaz bench suite audit --config reviews/task-87/024-phase7-spire-real10k-p99-rerun/phase7-spire-real10k-p99-rerun-suite.json --log-file reviews/task-87/024-phase7-spire-real10k-p99-rerun/artifacts/suite-audit.log
```

Run:

```sh
target/debug/ecaz bench suite run --config reviews/task-87/024-phase7-spire-real10k-p99-rerun/phase7-spire-real10k-p99-rerun-suite.json --database postgres --host /home/peter/.pgrx --port 28818 --manifest-output reviews/task-87/024-phase7-spire-real10k-p99-rerun/artifacts/run-manifest.json --results-output reviews/task-87/024-phase7-spire-real10k-p99-rerun/artifacts/results.jsonl --log-file reviews/task-87/024-phase7-spire-real10k-p99-rerun/artifacts/run.log
```

Status:

```sh
target/debug/ecaz bench suite status --manifest reviews/task-87/024-phase7-spire-real10k-p99-rerun/artifacts/run-manifest.json
```

## Key Results

- suite audit: `[suite:task87-phase7-spire-real10k-p99-rerun-suite] audit passed: 3 steps`
- suite status: `[suite:task87-phase7-spire-real10k-p99-rerun-suite] completed=3 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`
- SPIRE real10k off: `latency_p50=18.809 ms`, `latency_p95=21.837 ms`, `latency_p99=23.966 ms`, `recall@k=1.0000`
- SPIRE real10k on: `latency_p50=15.400 ms`, `latency_p95=16.354 ms`, `latency_p99=17.879 ms`, `recall@k=1.0000`
- deltas: p50 `-18.1%`, p95 `-25.1%`, p99 `-25.4%`
- on counters: `surface=spire flushes=4800 candidates=1551640 elapsed_ms=1783.173537 lut32_flushes=4800 lut32_candidates=1551640`

## Artifact Index

- `suite-audit.log`: suite audit output.
- `run.log`: full suite run log.
- `run-manifest.json`: structured suite manifest emitted by `ecaz bench suite run`.
- `results.jsonl`: structured parsed suite results.
- `status.log`: status output.
- `run/precheck-host.log`: host/database precheck.
- `run/pipeline-real10k-spire-candidate-batch-off.log`: SPIRE off pipeline result.
- `run/pipeline-real10k-spire-candidate-batch-on.log`: SPIRE on pipeline result.
