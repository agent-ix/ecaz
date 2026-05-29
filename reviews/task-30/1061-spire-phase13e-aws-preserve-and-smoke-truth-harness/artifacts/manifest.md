# Artifact Manifest

- head SHA before code commit: `a25a0b8b62b8a07cac318900ee9be4aa311d3c94`
- task bucket: `reviews/task-30/1061-spire-phase13e-aws-preserve-and-smoke-truth-harness`
- timestamp: `2026-05-28T23:38:00Z`
- lane: Phase 13e representative AWS harness hardening
- fixture: local harness/preflight checks only; no AWS resources touched by this packet
- storage format: rabitq
- rerank mode: default
- surface: representative performance runner, smoke harness, local operator CLI

## Artifacts

- `check-watchdog-local.log`
  - command: `scripts/spire-aws/check-watchdog-local.sh`
  - key result: success exits still run teardown under explicit `SPIRE_AWS_TEARDOWN_ON_EXIT=always`; failing passes now log `preserving AWS resources for in-place diagnosis` and do not run teardown by default.

- `preflight-representative-performance.log`
  - command: `scripts/spire-aws/preflight-representative-performance.sh`
  - key result: `SPIRE representative performance preflight passed`

- `cargo-build-release-ecaz-cli.log`
  - command: `cargo build --release --bin ecaz --package ecaz-cli`
  - key result: `Finished release profile`; one pre-existing dead-code warning in `crates/ecaz-cli/src/commands/corpus/load.rs`.

- `help-spire-pipeline-truth-corpus.log`
  - command: `target/release/ecaz bench spire-pipeline --help | grep -F -- --truth-corpus-file`
  - key result: current local benchmark binary exposes `--truth-corpus-file`.

- `help-recall-truth-corpus.log`
  - command: `target/release/ecaz bench recall --help | grep -F -- --truth-corpus-file`
  - key result: current local recall binary exposes `--truth-corpus-file`.
