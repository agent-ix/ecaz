# Artifacts Manifest

Packet: `30858-spire-dml-relation-context-docs`
Head SHA: `52d9cca2d05c63c34c98925d9960feeb4fb7ebfd`
Timestamp: `2026-05-11 14:30 PDT`
Surface: ADR-069 DML front-door relation-context loader documentation
Storage format / rerank mode: n/a
Isolated one-index-per-table vs shared-table surfaces: n/a

## cargo-fmt-check.log

- Command:
  `script -q -c "cargo fmt --check" review/30858-spire-dml-relation-context-docs/artifacts/cargo-fmt-check.log`
- Lane / fixture:
  Rust formatting check.
- Key result lines:
  - Command exited with code `0`.
  - Existing stable-rustfmt warnings about unstable import options are present.

## git-diff-check.log

- Command:
  `script -q -c "git diff --check" review/30858-spire-dml-relation-context-docs/artifacts/git-diff-check.log`
- Lane / fixture:
  whitespace/error check.
- Key result lines:
  - Command exited with code `0`.
