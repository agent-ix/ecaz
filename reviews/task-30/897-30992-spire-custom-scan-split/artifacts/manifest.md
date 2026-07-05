# Artifact Manifest: SPIRE CustomScan Split

Packet: `30992-spire-custom-scan-split`
Head SHA at run time: `8b4e793e` plus working-tree checkpoint changes
Timestamp: 2026-05-13 America/Los_Angeles

This packet is a layout/refactor checkpoint for Phase 12b.3 structural split.
Lane, fixture, storage format, and rerank mode are not applicable except where
a command explicitly names a test filter.

| Artifact | Command | Result |
|---|---|---|
| `cargo-check-pg18.log` | `cargo check --no-default-features --features pg18` | exit 0; `Finished dev profile`; one pre-existing unused-import warning in `src/am/mod.rs` |
| `cargo-fmt-check.log` | `cargo fmt --check` | exit 0; rustfmt emitted stable-toolchain warnings for unstable import-group config |
| `git-diff-check.log` | `git diff --check -- src/am/ec_spire/custom_scan.rs src/am/ec_spire/custom_scan plan/tasks/task30-phase12b-spire-cleanup.md` | exit 0 |
| `custom-scan-line-count.log` | `find src/am/ec_spire/custom_scan -type f -name '*.rs' -print0 \| xargs -0 wc -l \| sort -n` | largest split file is `dml.rs` at 746 lines; total split surface 2,953 lines |
| `cargo-test-custom-scan.log` | `cargo test --no-default-features --features pg18 custom_scan` | exit 0; 13 passed, 0 failed, 1699 filtered out |

