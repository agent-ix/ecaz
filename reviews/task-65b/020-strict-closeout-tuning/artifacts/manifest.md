# Artifact Manifest

- head SHA: `d5df80c889675b870c13371a754a1827cd036caa`
- task bucket: `reviews/task-65b`
- packet: `reviews/task-65b/020-strict-closeout-tuning`
- timestamp UTC: `2026-06-05T23:40:02Z`
- lane: local PG18, `ec_diskann`, `pq_fastscan`
- storage format: `pq_fastscan`
- isolation: one index per table prefix

## Code Change

Artifact source:

- commit: `d5df80c88 Tune DiskANN parallel build batch default`
- file: `src/am/ec_diskann/mod.rs`
- change: `ECDISKANN_DEFAULT_PARALLEL_BUILD_BATCH_SIZE = 704`

Interpretation:

- For 10k-row builds, the existing `TASK65B_SMALL_BUILD_NODE_CAP=10000` and `TASK65B_SMALL_BUILD_MAX_BATCH_SIZE=64` cap requested/default `704` to effective `64`.
- For real100k, the default remains effective `704`, matching the passing real100k closeout cell.

## Validation Artifacts

### `install-after-default.log`

Command:

`./target/debug/ecaz --log-file reviews/task-65b/020-strict-closeout-tuning/artifacts/install-after-default.log dev install ecaz-pg-test --pg 18`

Result:

- passed
- installed backend: `/opt/homebrew/lib/postgresql@18/ecaz.dylib`
- installed backend SHA256: `b206d0568414b689d5546103fa19d07ec533023f4b6c69b2e88a0af95452d097`

### `cargo-fmt-check.log`

Command:

`script -q reviews/task-65b/020-strict-closeout-tuning/artifacts/cargo-fmt-check.log cargo fmt --check`

Result:

- passed
- emitted only the existing stable-channel rustfmt warnings for `imports_granularity` and `group_imports`.

### `cargo-test-options-default.log`

Command:

`script -q reviews/task-65b/020-strict-closeout-tuning/artifacts/cargo-test-options-default.log cargo test -p ecaz --lib --no-default-features --features pg18 am::ec_diskann::options::tests::diskann_default_options_include_scan_runtime_defaults`

Result:

- passed: `1 passed; 0 failed`

### `cargo-test-build-task65b.log`

Command:

`script -q reviews/task-65b/020-strict-closeout-tuning/artifacts/cargo-test-build-task65b.log cargo test -p ecaz --lib --no-default-features --features pg18 am::ec_diskann::build::tests::task65b`

Result:

- passed: `6 passed; 0 failed`

## Synth10k Strict Recall Closeout

Suite config:

- `synth10k-b64-l240-suite.json`

Command:

`./target/debug/ecaz bench suite run --config reviews/task-65b/020-strict-closeout-tuning/synth10k-b64-l240-suite.json --host /Users/peter/.pgrx --port 28818 --manifest-output reviews/task-65b/020-strict-closeout-tuning/artifacts/synth10k-b64-l240-manifest.json --results-output reviews/task-65b/020-strict-closeout-tuning/artifacts/synth10k-b64-l240-results.jsonl --log-file reviews/task-65b/020-strict-closeout-tuning/artifacts/synth10k-b64-l240-run.log`

Artifacts:

- `synth10k-b64-l240-manifest.json`
- `synth10k-b64-l240-results.jsonl`
- `load-synth10k-w8-b64-r32-l240.log`
- `recall-synth10k-w8-b64-r32-l240.log`
- `storage-synth10k-w8-b64-r32-l240.log`
- `truth-synth10k-stream-k10.json`

Key result lines:

- corpus SHA256: `ccd9a13cdf99eded145fe92ba65d135a57495b55513444caf35c54d5bdcc6f2f`
- query SHA256: `155086601cd7b0487dab8cd6d4418faf0d2bfd4e0a8b7410d3adbdd31bd81b71`
- `parallel_effective_workers=8`
- `parallel_batch_size=64`
- `build_index=4.23s`
- backend `total_ms=4232`
- Recall@10 L64/L200/L800: `0.1610 / 0.2585 / 0.3295`

Gate interpretation:

- Task 65 accepted synth L200 reference: `0.2625`.
- Strict 0.5pp floor: `0.2575`.
- Packet 020 L200: `0.2585`, passes by `0.0010`.

## Real100k Strict Time + Recall Closeout

Suite config:

- `real100k-b704-suite.json`

Command:

`./target/debug/ecaz bench suite run --config reviews/task-65b/020-strict-closeout-tuning/real100k-b704-suite.json --host /Users/peter/.pgrx --port 28818 --manifest-output reviews/task-65b/020-strict-closeout-tuning/artifacts/real100k-b704-manifest.json --results-output reviews/task-65b/020-strict-closeout-tuning/artifacts/real100k-b704-results.jsonl --log-file reviews/task-65b/020-strict-closeout-tuning/artifacts/real100k-b704-run.log`

Artifacts:

- `real100k-b704-manifest.json`
- `real100k-b704-results.jsonl`
- `load-real100k-w8-b704-r32-l100.log`
- `recall-real100k-w8-b704-r32-l100.log`
- `truth-real100k-b704-k10.json`

Key result lines:

- corpus SHA256: `07275cfd5a7a4b415ddf5eacc086de98294ac978532df46ffae30f9202323a95`
- query SHA256: `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782`
- `parallel_effective_workers=8`
- `parallel_batch_size=704`
- `build_index=28.47s`
- backend `total_ms=28453`
- Recall@10 L64/L128/L200: `0.9225 / 0.9645 / 0.9720`

Gate interpretation:

- Task 65b real100k time gate: `<= 30s`; packet 020 passes at `28.47s`.
- Packet 001 real100k L200 reference: `0.9755`.
- Strict 0.5pp floor: `0.9705`.
- Packet 020 L200: `0.9720`, passes by `0.0015`.
