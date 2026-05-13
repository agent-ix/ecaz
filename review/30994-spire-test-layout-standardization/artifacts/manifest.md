# Artifact Manifest: SPIRE Test Layout Standardization

Packet: `30994-spire-test-layout-standardization`
Head SHA at run time: `3793cd53` plus working-tree checkpoint changes
Timestamp: 2026-05-13 America/Los_Angeles

This packet is a layout/refactor checkpoint for Phase 12b.4. Lane, fixture,
storage format, and rerank mode are not applicable except where a command
explicitly names a test filter.

| Artifact | Command | Result |
|---|---|---|
| `cargo-check-pg18.log` | `cargo check --no-default-features --features pg18` | exit 0; one pre-existing unused-import warning in `src/am/mod.rs` |
| `cargo-fmt-check.log` | `cargo fmt --check` | exit 0; rustfmt emitted stable-toolchain warnings for unstable import-group config |
| `git-diff-check.log` | `git diff --check -- docs/SPIRE_CODE_LAYOUT.md plan/tasks/task30-phase12b-spire-cleanup.md ...` | exit 0 |
| `cfg-test-sanity.log` | `rg -n '#\\[cfg\\(test\\)\\]\\s*$' src/am/ec_spire --glob '!**/tests.rs' --glob '!**/tests/**'` | one remaining non-test-file match: field-level `#[cfg(test)]` in `meta/root_control.rs`, not an inline test module |
| `module-test-file-sanity.log` | `find src/am/ec_spire -maxdepth 2 -type f '(' -name 'mod.rs' -o -name 'tests.rs' ')' \| sort \| rg 'assign\|cost\|diagnostics\|dml_frontdoor\|options\|quantizer\|vacuum'` | confirms each migrated module has `mod.rs` and `tests.rs` |
| `cargo-test-cost-filter.log` | `cargo test --no-default-features --features pg18 cost_increases_with_effective_nprobe` | exit 0; 1 passed, 0 failed, 1711 filtered out |

