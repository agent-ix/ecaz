# Artifact Manifest

- head SHA: `bde4a9d9b799500f29dd95d3f1a4b6897412885a`
- code SHA under measurement: `40c36f73982459f6fec39590482878445b5b187a`
- task bucket: `reviews/task-87/`
- packet path: `reviews/task-87/002-synthetic-latency/`
- timestamp: `2026-06-08T05:43:57Z`
- lane: Task 87 TurboQuant candidate batching
- fixture: deterministic synthetic 2,000-row corpus, 50 queries, dim 1536
- storage format: `turboquant`
- AM/profile: `ec_spire`
- rerank mode: `rerank_width=25`
- isolated one-index-per-table vs shared-table surface: one synthetic prefix,
  `task87_synth2k`, with one SPIRE index `task87_synth2k_idx`

## Suite Config

- file: `reviews/task-87/002-synthetic-latency/suite-synthetic-spire.json`
- runner: `target/debug/ecaz bench suite run`
- note: the first full run generated the TSVs and completed setup, then failed
  at load because suite `${artifact_dir}` templating is not applied to load
  input paths. The suite config was corrected to explicit packet-local paths
  and rerun for the load and latency steps.

## Commands

### Initial audit

- command:
  `target/debug/ecaz bench suite audit --config reviews/task-87/002-synthetic-latency/suite-synthetic-spire.json --database postgres --host /home/peter/.pgrx --port 28818 --log-file reviews/task-87/002-synthetic-latency/artifacts/suite-audit.log`
- result: expected pre-generation failure; audit reported the generated TSV
  inputs missing before raw generation steps ran

### Full suite setup attempt

- command:
  `target/debug/ecaz bench suite run --config reviews/task-87/002-synthetic-latency/suite-synthetic-spire.json --database postgres --host /home/peter/.pgrx --port 28818 --log-file reviews/task-87/002-synthetic-latency/artifacts/suite-run.log`
- result: setup/generation succeeded; load failed because load input paths used
  an unexpanded `${artifact_dir}` template

### Corrected load and latency rerun

- command:
  `target/debug/ecaz bench suite run --config reviews/task-87/002-synthetic-latency/suite-synthetic-spire.json --database postgres --host /home/peter/.pgrx --port 28818 --only load-synth2k-spire-turboquant --only latency-synth2k-spire-turboquant --log-file reviews/task-87/002-synthetic-latency/artifacts/suite-run-load-latency-rerun.log`
- result: pass
- status:
  `completed=2 failed=0 skipped=4 dry_run=0 missing_artifacts=0 stale=0`

## Durable Artifacts

- `precheck-pg18-extension.log`: PG18 extension and AM precheck from the full
  setup attempt.
- `cleanup-existing-fixture.log`: cleanup SQL log from the full setup attempt.
- `suite-run.log`: full setup attempt; includes successful generation and the
  load path-template failure.
- `suite-run-load-latency-rerun.log`: corrected load and latency rerun.
- `load-synth2k-spire-turboquant.log`: load timings, corpus/query hashes, and
  index reloptions.
- `latency-synth2k-spire-turboquant.log`: latency table.
- `suite-manifest.json`: final selected-rerun suite manifest.
- `suite-status.log`: final suite status.
- `suite-report.md`: parsed suite report.
- `results.jsonl` and `results-report.jsonl`: normalized result rows.

Generated TSV intermediates were intentionally not committed:

- `task87_synth2k_corpus.tsv` was 28 MB.
- `task87_synth2k_queries.tsv` was 716 KB.

They are reproducible from the suite config. The load log records the generated
fixture hashes:

- corpus SHA-256:
  `82ce5809a55fc5e2167becd55bc42f97c78d1f0c1abd5148f4317331e346e2a0`
- queries SHA-256:
  `ae3adb391a30fd2a52bfcd8e6dc53b1bd8f918b697e09b4a82023a2b4a611d0c`

## Key Results

- load total: `8.93s`
- build index: `3.40s`
- latency `nprobe=4`: count `50`, p50 `27.5 ms`, p95 `32.1 ms`,
  p99 `59.5 ms`
- latency `nprobe=8`: count `50`, p50 `36.4 ms`, p95 `41.8 ms`,
  p99 `62.3 ms`
