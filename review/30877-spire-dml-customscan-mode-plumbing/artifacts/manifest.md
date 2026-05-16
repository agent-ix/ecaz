# Artifact Manifest: 30877 SPIRE DML CustomScan Mode Plumbing

Head SHA: `cc89b889d10b9c613af165ef41d0575439a2db26`

Packet/topic: `30877-spire-dml-customscan-mode-plumbing`

Timestamp: `2026-05-11 19:54 PDT`

## Artifacts

### `cargo-test-custom-scan-lib.log`

- Lane: Rust unit tests plus PG18 pgrx tests filtered by `custom_scan`
- Fixture: CustomScan status, eligibility, cost, DML mode mapping, and PG18
  CustomScan registration/eligibility tests
- Storage format: N/A
- Rerank mode: N/A
- Surface: N/A
- Command: `script -q -e -c "cargo test custom_scan --lib" review/30877-spire-dml-customscan-mode-plumbing/artifacts/cargo-test-custom-scan-lib.log`
- Head SHA: `cc89b889d10b9c613af165ef41d0575439a2db26`
- Timestamp: `2026-05-11 19:53 PDT`
- Key result:
  - `test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 1667 filtered out; finished in 15.84s`
  - `COMMAND_EXIT_CODE="0"`

### `cargo-fmt-check.log`

- Lane: Rust formatting check
- Fixture: repository formatting
- Storage format: N/A
- Rerank mode: N/A
- Surface: N/A
- Command: `cargo fmt --check`
- Head SHA: `cc89b889d10b9c613af165ef41d0575439a2db26`
- Timestamp: `2026-05-11 19:54 PDT`
- Key result:
  - `COMMAND_EXIT_CODE="0"`
  - Known stable-rustfmt warnings are present for unstable
    `imports_granularity` and `group_imports` options.

### `git-diff-check.log`

- Lane: whitespace check
- Fixture: 30877 committed diff
- Storage format: N/A
- Rerank mode: N/A
- Surface: N/A
- Command: `git diff --check HEAD^ HEAD -- src/am/ec_spire/custom_scan.rs src/am/ec_spire/mod.rs`
- Head SHA: `cc89b889d10b9c613af165ef41d0575439a2db26`
- Timestamp: `2026-05-11 19:54 PDT`
- Key result:
  - `COMMAND_EXIT_CODE="0"`
