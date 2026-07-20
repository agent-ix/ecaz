# Task 191 closeout artifact manifest

- Task bucket: `reviews/task-191/`
- Packet: `reviews/task-191/004-closeout/`
- Closeout HEAD before packet commit:
  `c8e77022e0939735431d4362babe759ff07aebc9`
- Timestamp: 2026-07-20 PDT
- Host/lane: Intel local, PG18 release, three loopback PostgreSQL owners
- Fixture: staged `ec_real_10k`
- Quant/index: normal-feature physical `ec_distann`, RaBitQ neighbors
- Rerank mode: production payload materialization
- Isolation: normal extension built with `--no-default-features --features pg18`;
  no benchmark feature or benchmark materialization override

## Production build artifacts

### `production-release-install.log`

- Command: `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/home/peter/dev/ecaz/target
  cargo pgrx install --release --pg-config <PG18 pg_config>
  --no-default-features --features pg18`
- Result: release build and install succeeded; cargo records only `pg18` in the
  extension feature list.

### `production-release-surface.log`

- Command: record HEAD and installed `.so` SHA-256; scan the shared library and
  generated SQL for `ec_distann.benchmark_materialization_batch_size`; record
  the source call chain for the fixed production driver.
- Installed normal extension SHA-256:
  `21b0998963e4982fbe521f034ed3bab3b1eb835bfb3e433239042352a3eec74c`.
- Shared-library benchmark materialization GUC strings: `0`.
- Generated-SQL benchmark materialization GUC strings: `0`.
- Source attestation: `PRODUCTION_MATERIALIZATION_BATCH_SIZE = 10`; normal
  `materialization_batch_size()` returns that constant; the unconditional
  custom scan calls that function for pending physical windows.

## Normal-release serving suite

### `production-isolation-suite.json`

- Checked-in `ecaz bench suite` config for the installed normal release.
- Shape: staged 10k real corpus, three owners, no `physical_benchmark`, no
  benchmark variants/GUCs, production head policy, 20 queries.

### `production-isolation-audit.log`

- Command: `ecaz bench suite audit --config <production-isolation-suite.json>`
- Result: audit passed, one step.

### `production-isolation-dry-run.log`

- Command: `ecaz bench suite run --config <production-isolation-suite.json>
  --dry-run`
- Result: expansion contains no benchmark variant, stage counter, or
  materialization override argument.

### `production-isolation-run/suite-manifest.json`

- Command: `ecaz bench suite run --config <production-isolation-suite.json>`
- Result: one succeeded step, exit code zero.
- Clean runner descriptor:
  `3ceaba9615bef4dcd9c4ca7c49c2dd07256b9e51`.

### `production-isolation-run/results.jsonl`

- Normalized source of truth for ready/published topology and serving gates.

### `production-isolation-run/normal-release-lazy10-smoke/distann-multinode-summary.log`

- Compact accepted summary.
- Results: physical serving returns 10 rows across three owners; node 2 and
  node 3 remote-owner materialization both use `CustomScan` and pass; topology
  gate passes with 10,000 source rows.

### `production-isolation-status.log`

- Command: `ecaz bench suite status --manifest <suite-manifest.json>`
- Result: completed 1; failed/skipped/dry-run/missing/stale all zero.

### `production-isolation-report.md`

- Command: `ecaz bench suite report --manifest <suite-manifest.json>`
- Result: generated normalized Markdown report over the accepted summary.

## Retained baseline and prior evidence

- Contract: `reviews/task-191/001-production-contract/`.
- Implementation and semantic/failure evidence:
  `reviews/task-191/002-production-implementation/`.
- Full 10k/50k/100k A/B and PROMOTE decision:
  `reviews/task-191/003-production-full-scale/`.
- Task 187 baseline: production trained head cap 4,096, 32 seeds, BW4/H100,
  graph degree 32, RaBitQ neighbor scoring, lazy10 payload windows; 100k recall
  `0.9625`, mean/p50/p95/p99/max
  `23.70/23.50/27.20/28.00/28.10 ms`, traversal `7.849 ms` (33.1%).

## Retention note

Raw node/fixture logs were pruned after compact summary and normalized results
generation. Corpus/query/truth data are not committed; the suite config records
the staged corpus paths.
