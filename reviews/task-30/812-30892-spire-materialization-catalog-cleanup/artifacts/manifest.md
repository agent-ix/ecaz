# Artifacts Manifest

Packet: `30892-spire-materialization-catalog-cleanup`

Head SHA: `e716c07a`

## Artifacts

- `cargo-test-custom-scan-lib.log`
  - head SHA: `e716c07a`
  - lane: Rust + PG18 pg_test filtered custom scan lane
  - command: `cargo test custom_scan --lib`
  - timestamp: `2026-05-11T22:38:44-07:00`
  - isolated/shared surface: shared extension catalog; focused CustomScan unit
    and PG18 fixtures
  - key result: `test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 1669 filtered out`

- `cargo-test-remote-catalog-lib.log`
  - head SHA: `e716c07a`
  - lane: Rust + PG18 pg_test filtered remote catalog cleanup lane
  - command: `cargo test remote_catalog --lib`
  - timestamp: `2026-05-11T22:38:44-07:00`
  - isolated/shared surface: shared extension catalog; remote catalog cleanup
    diagnostics
  - key result: `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 1680 filtered out`

- `cargo-fmt-check.log`
  - head SHA: `e716c07a`
  - lane: formatting
  - command: `cargo fmt --check`
  - timestamp: `2026-05-11T22:38:44-07:00`
  - isolated/shared surface: not applicable
  - key result: exited 0; stable rustfmt emitted the existing warnings about
    unstable `imports_granularity` and `group_imports` settings.

- `git-diff-check.log`
  - head SHA: `e716c07a`
  - lane: whitespace check
  - command: `git diff --check e716c07a^ e716c07a`
  - timestamp: `2026-05-11T22:38:44-07:00`
  - isolated/shared surface: not applicable
  - key result: exited 0 with no whitespace errors.
