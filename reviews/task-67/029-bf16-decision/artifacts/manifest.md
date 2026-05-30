# Task 67 Packet 029 Artifact Manifest

- head SHA: c4ebcbca2d7ab9289332922290424f03e960e7dd
- task bucket: `reviews/task-67/029-bf16-decision/`
- timestamp: 2026-05-30T15:31:00Z
- lane: Slice I bf16 on/off SQL decision
- fixture: `ec_real_10k`, 200 queries, PG18, AWS Intel `10k-intel`
- storage format / rerank mode: `rabitq`, `quant_bits=4`, `rerank=heap_f32`, `rerank_width=100`
- surface: isolated one-index-per-table prefixes:
  - `task67_bf16off_10k_rabitq4`
  - `task67_bf16on_10k_rabitq4`

## Suite Configs

### `artifacts/task67-bf16-off-suite.json`

- command: `target/debug/ecaz bench suite audit --config reviews/task-67/029-bf16-decision/artifacts/task67-bf16-off-suite.json`
- audit log: `artifacts/local/suite-audit-bf16-off.log`
- result: passed

### `artifacts/task67-bf16-on-suite.json`

- command: `target/debug/ecaz bench suite audit --config reviews/task-67/029-bf16-decision/artifacts/task67-bf16-on-suite.json`
- audit log: `artifacts/local/suite-audit-bf16-on.log`
- result: passed

## Successful Measurement Runs

### bf16 off

- install command: `target/debug/ecaz cloud install --profile 10k-intel --git-ref ad499dbf3 --skip-extension-recreate --database postgres --timeout 3600`
- install log: `artifacts/preflight/cloud-install-bf16-off-ad499dbf3.log`
- bench command: `target/debug/ecaz cloud bench --profile 10k-intel --simd-mode auto --config reviews/task-67/029-bf16-decision/artifacts/task67-bf16-off-suite.json --suite task67-bf16-off --database postgres --ecaz-bin /usr/local/bin/ecaz`
- bench log: `artifacts/bf16-off/cloud-bench-bf16-off.log`
- S3 source: `s3://ecaz-cloud-10k-intel-f85c5b34/bench-artifacts/task67-bf16-off/20260530T145755Z/`
- key results:
  - nprobe 16 latency p50/mean/recall: `2.02 ms` / `2.08 ms` / `0.9985`
  - nprobe 32 latency p50/mean/recall: `3.32 ms` / `3.34 ms` / `1.0000`
  - nprobe 64 latency p50/mean/recall: `5.52 ms` / `5.56 ms` / `1.0000`

### bf16 on

- successful install command: `target/debug/ecaz cloud install --profile 10k-intel --git-ref 650fb11c8 --extension-feature rabitq-bf16 --skip-cli-build --clean-cargo-target --skip-extension-recreate --database postgres --timeout 3600`
- successful install log: `artifacts/preflight/cloud-install-bf16-on-650fb11c8-pre-git-clean.log`
- bench command: `target/debug/ecaz cloud bench --profile 10k-intel --simd-mode auto --config reviews/task-67/029-bf16-decision/artifacts/task67-bf16-on-suite.json --suite task67-bf16-on --database postgres --ecaz-bin /usr/local/bin/ecaz`
- bench log: `artifacts/bf16-on/cloud-bench-bf16-on-rerun-after-corpus-restore.log`
- S3 source: `s3://ecaz-cloud-10k-intel-f85c5b34/bench-artifacts/task67-bf16-on/20260530T153009Z/`
- key results:
  - nprobe 16 latency p50/mean/recall: `2.25 ms` / `2.31 ms` / `0.9985`
  - nprobe 32 latency p50/mean/recall: `3.58 ms` / `3.62 ms` / `1.0000`
  - nprobe 64 latency p50/mean/recall: `6.45 ms` / `6.45 ms` / `1.0000`

## Recovery / Preflight Logs

- `artifacts/preflight/cloud-install-bf16-on-ad499dbf3.log`: initial bf16-on install failed after extension install during CLI rebuild with no space left.
- `artifacts/preflight/cloud-install-bf16-on-ad1aec9f2-skip-cli-build.log`: retry skipped CLI rebuild but failed during extension compile with no space left.
- `artifacts/preflight/cloud-install-bf16-on-25f2b4754-clean-target.log`: retry attempted target cleanup after git reset; failed because the host was too full to write `.git/index.lock`.
- `artifacts/preflight/cloud-install-bf16-on-650fb11c8-pre-git-clean.log`: successful install after cleaning Cargo target before git reset.
- `artifacts/preflight/remote-restore-ec-real-10k-invocation.log`: restored the `ec_real_10k` corpus files after `cargo clean` removed `target/real-corpus`.
- `artifacts/preflight/cloud-pause-after-bf16-on-success.log`: AWS stop request after successful measurement.
- `artifacts/preflight/cloud-status-after-bf16-on-success.log`: post-pause status; state was `stopping`, cost reported `$0.00/hr running`.

## Comparison

See `artifacts/comparison.md`.
