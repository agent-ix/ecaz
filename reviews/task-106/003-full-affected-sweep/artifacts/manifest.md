# Task 106 Full Affected Sweep Artifact Manifest

- head SHA: `f782eec330f10ea6c5998bad3afd1074a50eb3cd`
- task bucket: `reviews/task-106/003-full-affected-sweep`
- captured: `2026-06-13T15:21:55-07:00`
- host lane: local Intel PG18, database `postgres`, socket dir `/home/peter/.pgrx`, port `28818`
- runner: `target/release/ecaz bench suite`
- surface model: one prefix/table per quant/index/options cell; SPIRE and HNSW batch-on/off are shared-table toggles via session GUCs
- fixture sizes: 10k, 50k, 100k, 1m
- affected cells: IVF RaBitQ b1/b2/b4/b8 scratch on/off; IVF auto TurboQuant scratch on/off; IVF explicit TurboQuant scratch on/off; SPIRE RaBitQ candidate-batch on/off plus pipeline; HNSW PQ-FastScan grouped-PQ candidate-batch on/off; SPIRE PQ-FastScan negative gap

## Suite Configs

- `task106-full-affected-sweep.json`
  - command: `target/release/ecaz bench suite run --continue-on-error --config reviews/task-106/003-full-affected-sweep/task106-full-affected-sweep.json --database postgres --host /home/peter/.pgrx --port 28818 --log-file reviews/task-106/003-full-affected-sweep/artifacts/full-sweep-run-fixed.log`
  - manifest: `artifacts/suite/suite-manifest.json`
  - status/report: `artifacts/main-suite-status.log`, `artifacts/main-suite-report.md`, `artifacts/main-suite-report-results.jsonl`
  - result: `completed=125 failed=9 stale=40`
  - notes: main suite completed 10k, 50k, 100k, and 1m B1 scratch-on before being stopped/resumed into `task106-full-affected-sweep-1m-continuation.json`.

- `task106-full-affected-sweep-1m-continuation.json`
  - command: `target/release/ecaz bench suite run --continue-on-error --config reviews/task-106/003-full-affected-sweep/task106-full-affected-sweep-1m-continuation.json --database postgres --host /home/peter/.pgrx --port 28818 --log-file reviews/task-106/003-full-affected-sweep/artifacts/full-sweep-1m-continuation.log`
  - manifest: `artifacts/suite-1m-continuation/suite-manifest.json`
  - status/report: `artifacts/continuation-suite-status.log`, `artifacts/continuation-suite-report.md`, `artifacts/continuation-suite-report-results.jsonl`
  - result: `completed=37 failed=3 stale=0`
  - notes: the 1m continuation covers the main suite's stale 1m cells. The two SPIRE recall failures are replaced by the SPIRE recall supplemental. The 1m SPIRE pipeline command wrote literal `${artifact_dir}` funnel outputs in this continuation; clean packet-local pipeline JSONL evidence is replaced by the SPIRE pipeline supplemental.

- `task106-spire-recall-supplemental.json`
  - command: `target/release/ecaz bench suite run --continue-on-error --config reviews/task-106/003-full-affected-sweep/task106-spire-recall-supplemental.json --database postgres --host /home/peter/.pgrx --port 28818 --log-file reviews/task-106/003-full-affected-sweep/artifacts/spire-recall-supplemental.log`
  - manifest: `artifacts/suite-spire-recall-supplemental/suite-manifest.json`
  - status/report: `artifacts/spire-recall-supplemental-status.log`, `artifacts/spire-recall-supplemental-report.md`, `artifacts/spire-recall-supplemental-report-results.jsonl`
  - result: `completed=2 failed=0 stale=0`
  - replaces: `recall-1m-spire-rabitq-batch-on`, `recall-1m-spire-rabitq-batch-off`

- `task106-spire-pipeline-supplemental.json`
  - command: `target/release/ecaz bench suite run --continue-on-error --config reviews/task-106/003-full-affected-sweep/task106-spire-pipeline-supplemental.json --database postgres --host /home/peter/.pgrx --port 28818 --log-file reviews/task-106/003-full-affected-sweep/artifacts/spire-pipeline-supplemental.log`
  - manifest: `artifacts/suite-spire-pipeline-supplemental/suite-manifest.json`
  - status/report: `artifacts/spire-pipeline-supplemental-status.log`, `artifacts/spire-pipeline-supplemental-report.md`, `artifacts/spire-pipeline-supplemental-report-results.jsonl`
  - result: `completed=8 failed=0 stale=0`
  - replaces: SPIRE pipeline cells for 10k, 50k, 100k, and 1m, candidate-batch on/off
  - JSONL: eight nonempty per-cell funnel files plus `results.jsonl` under `artifacts/suite-spire-pipeline-supplemental/`

## Failure Accounting

- SPIRE pipeline config failures in the main suite:
  - `spire-pipeline-10k-spire-rabitq-batch-on`
  - `spire-pipeline-10k-spire-rabitq-batch-off`
  - `spire-pipeline-50k-spire-rabitq-batch-on`
  - `spire-pipeline-50k-spire-rabitq-batch-off`
  - `spire-pipeline-100k-spire-rabitq-batch-on`
  - `spire-pipeline-100k-spire-rabitq-batch-off`
  - cause: bad `--truth-cache-file` use in the original suite config.
  - replacement evidence: `task106-spire-pipeline-supplemental.json`, `completed=8 failed=0`.

- SPIRE recall failures in the 1m continuation:
  - `recall-1m-spire-rabitq-batch-on`
  - `recall-1m-spire-rabitq-batch-off`
  - cause: CLI predicted-source fetch used bulk `WHERE id = ANY($1::bigint[])`, which SPIRE distributed reads reject for this path.
  - code fix: `crates/ecaz-cli/src/commands/bench/recall.rs` now falls back to per-id `WHERE id = $1::bigint` fetches if the bulk lookup fails.
  - replacement evidence: `task106-spire-recall-supplemental.json`, `completed=2 failed=0`.

- Expected SPIRE PQ-FastScan negative gap:
  - `load-10k-spire-pqfastscan-gap`
  - `load-50k-spire-pqfastscan-gap`
  - `load-100k-spire-pqfastscan-gap`
  - `load-1m-spire-pqfastscan-gap`
  - expected diagnostic: `ec_spire PQ-FastScan encoding requires a persisted grouped-PQ model`
  - evidence logs: `artifacts/suite/load-10k-spire-pqfastscan-gap.log`, `artifacts/suite/load-50k-spire-pqfastscan-gap.log`, `artifacts/suite/load-100k-spire-pqfastscan-gap.log`, `artifacts/suite-1m-continuation/load-1m-spire-pqfastscan-gap.log`

## Key Result Lines

- Main suite status: `completed=125 failed=9 skipped=0 dry_run=0 missing_artifacts=0 stale=40`
- 1m continuation status: `completed=37 failed=3 skipped=0 dry_run=0 missing_artifacts=2 stale=0`
- SPIRE recall supplemental status: `completed=2 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`
- SPIRE pipeline supplemental status: `completed=8 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`
- 1m SPIRE recall supplemental, batch-on recall: `0.9540/0.9700/0.9760/0.9800` at nprobe `16/24/32/48`
- 1m SPIRE recall supplemental, batch-off recall: `0.9540/0.9700/0.9760/0.9800` at nprobe `16/24/32/48`
- 1m HNSW PQ-FastScan grouped-PQ load: copy `407.88s`, encode `298.84s`, build index `3762.28s`, total `4581.31s`
- 1m HNSW PQ-FastScan grouped-PQ batch-on recall: `0.8260/0.8370/0.8580/0.8640` at ef_search `80/120/200/400`
- 1m HNSW PQ-FastScan grouped-PQ batch-off recall: `0.8260/0.8370/0.8580/0.8640` at ef_search `80/120/200/400`
- 1m SPIRE pipeline supplemental batch-on p50: `52.838/64.479/73.647/97.896 ms`, recall `0.9540/0.9700/0.9760/0.9800`
- 1m SPIRE pipeline supplemental batch-off p50: `52.243/66.979/87.149/104.560 ms`, recall `0.9540/0.9700/0.9760/0.9800`

## Provenance Notes

- The suite intentionally used `--allow-manifest-mismatch` for reusable corpora whose manifest prefix differs from the task-specific table prefix; those warnings are captured in load logs.
- The final authoritative clean SPIRE pipeline evidence is the supplemental suite, not the earlier continuation pipeline JSONL path with literal `${artifact_dir}`.
- The final authoritative clean SPIRE 1m recall evidence is the supplemental suite run after the CLI fallback fix.
