# Artifact Manifest: 30879 SPIRE DML Baserel Mode Wrappers

Head SHA: `e23850136112c2b35049d2bb89a31d6a1ef8d336`

Packet/topic: `30879-spire-dml-baserel-mode-wrappers`

Timestamp: `2026-05-11 20:07 PDT`

## Artifacts

### `cargo-test-dml-frontdoor-lib.log`

- Lane: Rust unit tests plus PG18 pgrx tests filtered by `dml_frontdoor`
- Fixture: DML frontdoor classifier, baserel handoff, mode guard,
  replacement decision, primitive planning, primitive invocation, and PK SELECT
  CustomScan coverage
- Storage format: N/A
- Rerank mode: N/A
- Surface: N/A
- Command: `script -q -e -c "cargo test dml_frontdoor --lib" review/30879-spire-dml-baserel-mode-wrappers/artifacts/cargo-test-dml-frontdoor-lib.log`
- Head SHA: `e23850136112c2b35049d2bb89a31d6a1ef8d336`
- Timestamp: `2026-05-11 20:06 PDT`
- Key result:
  - `test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 1649 filtered out; finished in 16.52s`
  - `COMMAND_EXIT_CODE="0"`

### `cargo-fmt-check.log`

- Lane: Rust formatting check
- Fixture: repository formatting
- Storage format: N/A
- Rerank mode: N/A
- Surface: N/A
- Command: `cargo fmt --check`
- Head SHA: `e23850136112c2b35049d2bb89a31d6a1ef8d336`
- Timestamp: `2026-05-11 20:07 PDT`
- Key result:
  - `COMMAND_EXIT_CODE="0"`
  - Known stable-rustfmt warnings are present for unstable
    `imports_granularity` and `group_imports` options.

### `git-diff-check.log`

- Lane: whitespace check
- Fixture: 30879 committed diff
- Storage format: N/A
- Rerank mode: N/A
- Surface: N/A
- Command: `git diff --check HEAD^ HEAD -- src/am/ec_spire/dml_frontdoor.rs`
- Head SHA: `e23850136112c2b35049d2bb89a31d6a1ef8d336`
- Timestamp: `2026-05-11 20:07 PDT`
- Key result:
  - `COMMAND_EXIT_CODE="0"`
