# 30760 Artifact Manifest

Head SHA: `65c98d3eb5ce51b92a7310750895dc531011d571`

Packet: `30760-spire-remote-row-materialization-constant`

Scope: SPIRE Phase 11 Stage D reviewer P2 response for the AM remote-placement
gate's `remote_row_materialization` blocker string.

Lane: local PG18 compile/static validation. Fixture: constant reuse only.
Storage format: not applicable. Rerank mode: not applicable. Surface shape:
no SQL or index fixture was started for this packet.

## Artifacts

- `cargo-fmt-check.log`
  - Command: `script -q -e -c 'cargo fmt --check' ...`
  - Timestamp: 2026-05-10 12:49 PDT
  - Result: pass; only existing stable-rustfmt warnings for
    `imports_granularity` / `group_imports`.

- `cargo-check-pg18.log`
  - Command: `script -q -e -c 'cargo check --no-default-features --features pg18' ...`
  - Timestamp: 2026-05-10 12:49 PDT
  - Result line: `Finished dev profile ... target(s) in 0.18s`.

- `cargo-check-pg18-pg-test.log`
  - Command: `script -q -e -c 'cargo check --no-default-features --features "pg18 pg_test"' ...`
  - Timestamp: 2026-05-10 12:49 PDT
  - Result line: `Finished dev profile ... target(s) in 0.12s`.

- `git-diff-check-code.log`
  - Command: `script -q -e -c 'git diff --check -- <changed code/docs>' ...`
  - Timestamp: 2026-05-10 12:49 PDT
  - Result: pass, no whitespace errors.
