# Artifact Manifest: Task 86 SPIRE TurboQuant Suite

- head SHA: `e596db9b42b3ae1973486c901a17c5fdd06bbaf1`
- task bucket: `reviews/task-86/007-spire-suite`
- captured at: `2026-06-07T07:09:23Z`
- lane: SPIRE TurboQuant synthetic 1536-d PG18 suite
- fixture: 256 generated corpus rows, 32 generated query rows
- storage format: `ec_spire` with `storage_format=turboquant`, 4-bit no-QJL TurboQuant
- rerank mode: `rerank_width=10`
- index reloptions: `nlists=8`, `recursive_fanout=2`, `nprobe=4`, `top_graph_enabled=1`, `top_graph_degree=16`, `top_graph_build_list_size=32`, `top_graph_search_list_size=16`
- database: `task86_spire_tq`
- host / port: `/Users/peter/.pgrx` / `28818`
- isolated surface: yes, task-local database and prefix `task86_spire_synth256_tq`

## Artifacts

- `suite.json`: checked-in `ecaz bench suite` config for this lane.
- `task86_spire_synth256_corpus.tsv`: deterministic generated corpus, seed `8600`.
- `task86_spire_synth256_queries.tsv`: deterministic generated queries, seed `8601`, start id `100000`.
- `generate-corpus.log`: corpus generation log.
- `generate-queries.log`: query generation log.
- `install-ecaz-pg-test.log`: PG18 extension install log for this branch.
- `db-exists-check.log`: pre-create database existence check.
- `create-db.log`: `CREATE DATABASE task86_spire_tq` log.
- `suite-audit.log`: initial audit before the final `recursive_fanout=2` config adjustment.
- `suite-dry-run.log`: dry-run expansion log.
- `suite-dry-run-manifest.json`: dry-run manifest.
- `results-dry-run.jsonl`: dry-run results.
- `suite-run.log`: failed first real run, missing global host/database for load steps.
- `suite-manifest.json`: manifest for the failed first real run.
- `suite-run-host.log`: failed second real run, rejected by SPIRE top graph because `recursive_fanout < 2`.
- `suite-manifest-host.json`: manifest for the failed second real run.
- `suite-audit-after-recursive-fanout.log`: passing audit for the final config.
- `suite-run-host-rerun.log`: successful suite run log.
- `suite-manifest-host-rerun.json`: successful suite manifest.
- `results-host-rerun.jsonl`: successful structured suite results.
- `suite-report.md`: rendered report for the successful suite manifest/results.
- `suite-report.stderr.log`: report stderr log.
- `precheck-pg18-extension.log`: raw precheck step output.
- `load-synth256-spire-turboquant.log`: load/build step output.
- `storage-synth256-spire-turboquant.log`: storage step output.
- `pipeline-synth256-spire-turboquant.log`: SPIRE pipeline step output.
- `results-host-rerun-report.jsonl`: structured report output.

## Commands

```text
/Users/peter/.cargo/bin/ecaz corpus generate --output reviews/task-86/007-spire-suite/artifacts/task86_spire_synth256_corpus.tsv --n 256 --dim 1536 --seed 8600 --kind corpus --log-file reviews/task-86/007-spire-suite/artifacts/generate-corpus.log
```

```text
/Users/peter/.cargo/bin/ecaz corpus generate --output reviews/task-86/007-spire-suite/artifacts/task86_spire_synth256_queries.tsv --n 32 --dim 1536 --seed 8601 --kind queries --start-id 100000 --log-file reviews/task-86/007-spire-suite/artifacts/generate-queries.log
```

```text
./target/debug/ecaz --log-file reviews/task-86/007-spire-suite/artifacts/install-ecaz-pg-test.log dev install ecaz-pg-test --pg 18
```

```text
./target/debug/ecaz dev sql --pg 18 --db postgres --socket-dir /Users/peter/.pgrx --raw --sql "CREATE DATABASE task86_spire_tq;" --log-output reviews/task-86/007-spire-suite/artifacts/create-db.log
```

```text
./target/debug/ecaz bench suite audit --config reviews/task-86/007-spire-suite/suite.json > reviews/task-86/007-spire-suite/artifacts/suite-audit-after-recursive-fanout.log 2>&1
```

```text
./target/debug/ecaz bench suite run --config reviews/task-86/007-spire-suite/suite.json --dry-run --manifest-output reviews/task-86/007-spire-suite/artifacts/suite-dry-run-manifest.json --results-output reviews/task-86/007-spire-suite/artifacts/results-dry-run.jsonl --log-file reviews/task-86/007-spire-suite/artifacts/suite-dry-run.log
```

```text
./target/debug/ecaz --database task86_spire_tq --host /Users/peter/.pgrx --port 28818 bench suite run --config reviews/task-86/007-spire-suite/suite.json --manifest-output reviews/task-86/007-spire-suite/artifacts/suite-manifest-host-rerun.json --results-output reviews/task-86/007-spire-suite/artifacts/results-host-rerun.jsonl --log-file reviews/task-86/007-spire-suite/artifacts/suite-run-host-rerun.log
```

```text
./target/debug/ecaz bench suite report --manifest reviews/task-86/007-spire-suite/artifacts/suite-manifest-host-rerun.json --results-output reviews/task-86/007-spire-suite/artifacts/results-host-rerun-report.jsonl > reviews/task-86/007-spire-suite/artifacts/suite-report.md 2> reviews/task-86/007-spire-suite/artifacts/suite-report.stderr.log
```

## Key Result Lines

From `suite-audit-after-recursive-fanout.log`:

```text
[suite:task86-spire-turboquant-synth256] audit passed: 4 steps
```

From `suite-report.md`:

```text
steps: completed 4, failed 0, skipped 0, dry-run 0, missing artifacts 0, stale 0
load build_index: 0.009850 s
load total: 0.213550 s
ec_spire index task86_spire_synth256_tq_idx: 328.0 KiB, 1312.0 B/row
nprobe=4: latency p50 0.526 ms, p95 0.645 ms, recall@k 0.5813
nprobe=8: latency p50 0.625 ms, p95 0.634 ms, recall@k 0.9187
```

The SPIRE pipeline reports `status=requires_rabitq_storage_format` for remote-serving SPIRE endpoint export, which is expected for this local TurboQuant suite. The local scan and query metric steps still succeeded and returned `result_source=local_heap_candidates`.

## Interpretation

This packet records index-level evidence for the Task 86 SPIRE change that routes eligible TurboQuant assignment scoring through the no-QJL 4-bit prepared LUT path. It is not a before/after latency comparison; the before/after SIMD/kernel evidence remains in the earlier Task 86 micro packets. This suite verifies that the accepted no-format-change SPIRE path builds, stores, scans, and reports recall/latency through the canonical `ecaz bench suite` workflow on PG18.
