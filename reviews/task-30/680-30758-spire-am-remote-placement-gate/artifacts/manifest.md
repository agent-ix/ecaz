# 30758 Artifact Manifest

Head SHA: `6d76971c5dbc227ef9763f72113447ff03bfd7bb`

Packet: `30758-spire-am-remote-placement-gate`

Scope: SPIRE Phase 11 Stage D AM-side local heap delivery gate for active
remote placements.

Lane: local PG18 compile/static validation. Fixture: Rust-side placement
directory gate only. Storage format: `rabitq`. Rerank mode: unchanged from
packet `30756`; this packet blocks the legacy local heap cursor before it can
consume remote placements. Surface shape: no SQL or index fixture was started
for this packet; the active placement directory contract is independent of
shared-table versus isolated one-index-per-table surfaces.

## Artifacts

- `cargo-fmt-check.log`
  - Command: `script -q -e -c 'cargo fmt --check' ...`
  - Timestamp: 2026-05-10 12:23 PDT
  - Result: pass; only existing stable-rustfmt warnings for
    `imports_granularity` / `group_imports`.

- `cargo-check-pg18.log`
  - Command: `script -q -e -c 'cargo check --no-default-features --features pg18' ...`
  - Timestamp: 2026-05-10 12:23 PDT
  - Result line: `Finished dev profile ... target(s) in 0.18s`.

- `cargo-check-pg18-pg-test.log`
  - Command: `script -q -e -c 'cargo check --no-default-features --features "pg18 pg_test"' ...`
  - Timestamp: 2026-05-10 12:23 PDT
  - Result line: `Finished dev profile ... target(s) in 0.12s`.

- `git-diff-check-code.log`
  - Command: `script -q -e -c 'git diff --check -- <changed code/docs>' ...`
  - Timestamp: 2026-05-10 12:23 PDT
  - Result: pass, no whitespace errors.

- `cargo-test-local-heap-delivery-gate-loader-blocked.log`
  - Command: `script -q -e -c 'cargo test local_heap_delivery_gate --no-default-features --features pg18' ...`
  - Timestamp: 2026-05-10 12:23 PDT
  - Result: blocked after compile by known pgrx standalone test-binary loader
    issue, `undefined symbol: SPI_finish`; exit code 127.
