# Review Request: SPIRE Test Layout Standardization

Branch: `task-30-spire`
Task row: Phase 12b.4
Checkpoint scope: test-layout standardization, no intended behavior change

## Summary

This checkpoint standardizes the remaining inline Rust unit-test modules named
in Phase 12b.4. Each migrated module now has a directory layout with production
code in `mod.rs` and unit tests in `tests.rs`.

Migrated modules:

- `assign`
- `cost`
- `diagnostics`
- `dml_frontdoor`
- `options`
- `quantizer`
- `vacuum`

`custom_scan` was already converted in packet `30992`. The convention is now
documented in `docs/SPIRE_CODE_LAYOUT.md`.

## Notes

The sanity scan still reports `src/am/ec_spire/meta/root_control.rs:29`, but
that is a field-level `#[cfg(test)]`, not an inline `mod tests` block. Existing
external test modules under `build/`, `meta/`, `root/`, `scan/`, `storage/`,
and `update/` are intentionally unchanged.

## Validation

Artifacts are in `review/30994-spire-test-layout-standardization/artifacts/`.

- `cargo check --no-default-features --features pg18`: pass.
- `cargo fmt --check`: pass, with existing stable-rustfmt config warnings.
- `git diff --check -- ...`: pass.
- Test-layout sanity confirms the migrated modules have `mod.rs` and `tests.rs`.
- Filtered test build/run: `cargo test --no-default-features --features pg18 cost_increases_with_effective_nprobe`
  passed, 1 passed / 0 failed / 1711 filtered out.

## Review Focus

- Confirm the directory-module conversions preserve module names and visibility.
- Confirm the new layout convention is adequate for future SPIRE work.
- Confirm the tracker accurately closes only Phase 12b.4, leaving the large
  `src/lib.rs` fixture split and other Phase 12b rows open.

