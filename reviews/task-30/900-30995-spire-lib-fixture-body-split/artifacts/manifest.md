# Artifact Manifest: SPIRE Lib Fixture Body Split

Packet: `30995-spire-lib-fixture-body-split`
Head SHA at run time: `aa59b5eb` plus working-tree checkpoint changes
Timestamp: 2026-05-13 America/Los_Angeles

This packet is a mechanical fixture-sink checkpoint for Phase 12b.2. Lane,
fixture, storage format, and rerank mode are not applicable except where a
command explicitly names a test filter.

| Artifact | Command | Result |
|---|---|---|
| `cargo-check-pg18.log` | `cargo check --no-default-features --features pg18` | exit 0; one pre-existing unused-import warning in `src/am/mod.rs` |
| `cargo-fmt-check.log` | `cargo fmt --check` | exit 0; rustfmt emitted stable-toolchain warnings for unstable import-group config |
| `git-diff-check.log` | `git diff --check -- src/lib.rs src/tests/mod.rs plan/tasks/task30-phase12b-spire-cleanup.md` | exit 0 |
| `lib-tests-line-count.log` | `wc -l src/lib.rs src/tests/mod.rs` | `src/lib.rs` 17,812 lines; `src/tests/mod.rs` 48,693 lines |
| `fixture-location-sanity.log` | `rg -n 'fn test_ec_spire_\|#\\[pg_test\\]' src/lib.rs src/tests/mod.rs \| head -40` | `test_ec_spire_*` and `#[pg_test]` matches are in `src/tests/mod.rs`, not `src/lib.rs` |
| `cargo-test-custom-scan-status-pg.log` | `cargo test --no-default-features --features pg18 test_ec_spire_custom_scan_status_registered_fail_closed` | exit 0; 1 passed, 0 failed, 1711 filtered out |

