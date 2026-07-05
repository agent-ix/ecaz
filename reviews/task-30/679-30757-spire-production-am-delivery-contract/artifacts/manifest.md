# 30757 Artifact Manifest

Head SHA: `ff3bf705882102ced95ee42e5ac45f1e480ae8fa`

Packet: `30757-spire-production-am-delivery-contract`

Scope: SPIRE Phase 11 Stage D AM-delivery classification for the production
scan result stream.

Lane: local PG18 compile/static validation. Fixture: Rust-side production scan
stream classification only. Storage format: `rabitq`. Rerank mode: exact
heap-vector rerank is unchanged from packet `30756`; this packet classifies the
post-rerank stream for AM delivery. Surface shape: no SQL or index fixture was
started for this packet; the stream contract applies after the isolated
one-index-per-table loopback fixture validated in packet `30756`.

## Artifacts

- `cargo-fmt-check.log`
  - Command: `script -q -e -c 'cargo fmt --check' ...`
  - Timestamp: 2026-05-10 12:15 PDT
  - Result: pass; only existing stable-rustfmt warnings for
    `imports_granularity` / `group_imports`.

- `cargo-check-pg18.log`
  - Command: `script -q -e -c 'cargo check --no-default-features --features pg18' ...`
  - Timestamp: 2026-05-10 12:15 PDT
  - Result line: `Finished dev profile ... target(s) in 0.28s`.

- `cargo-check-pg18-pg-test.log`
  - Command: `script -q -e -c 'cargo check --no-default-features --features "pg18 pg_test"' ...`
  - Timestamp: 2026-05-10 12:15 PDT
  - Result line: `Finished dev profile ... target(s) in 0.12s`.

- `git-diff-check-code.log`
  - Command: `script -q -e -c 'git diff --check -- <changed code/docs>' ...`
  - Timestamp: 2026-05-10 12:15 PDT
  - Result: pass, no whitespace errors.

- `cargo-test-production-scan-am-delivery-loader-blocked.log`
  - Command: `script -q -e -c 'cargo test production_scan_am_delivery --no-default-features --features pg18' ...`
  - Timestamp: 2026-05-10 12:15 PDT
  - Result: blocked after compile by known pgrx standalone test-binary loader
    issue, `undefined symbol: SPI_finish`; exit code 127.
