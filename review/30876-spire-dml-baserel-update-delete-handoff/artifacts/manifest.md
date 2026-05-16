# Artifact Manifest: 30876 SPIRE DML Baserel UPDATE/DELETE Handoff

Head SHA: `1f1649bc0dd0725ad1e690d443ce5bc530a23e3d`

Packet/topic: `30876-spire-dml-baserel-update-delete-handoff`

Timestamp: `2026-05-11 19:45 PDT`

## Artifacts

### `cargo-test-dml-frontdoor-lib.rerun.log`

- Lane: Rust unit tests plus PG18 pgrx tests filtered by `dml_frontdoor`
- Fixture: DML frontdoor classifier, baserel handoff, replacement decision,
  primitive planning, primitive invocation, and PK SELECT CustomScan coverage
- Storage format: N/A
- Rerank mode: N/A
- Surface: N/A
- Command: `script -q -e -c "cargo test dml_frontdoor --lib" review/30876-spire-dml-baserel-update-delete-handoff/artifacts/cargo-test-dml-frontdoor-lib.rerun.log`
- Head SHA: `1f1649bc0dd0725ad1e690d443ce5bc530a23e3d`
- Timestamp: `2026-05-11 19:44 PDT`
- Key result:
  - `test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 1648 filtered out; finished in 16.94s`
  - `COMMAND_EXIT_CODE="0"`

### `cargo-test-dml-frontdoor-lib.log`

- Lane: Failed sandboxed attempt for the same focused Rust/PG18 pgrx filter
- Fixture: DML frontdoor tests
- Storage format: N/A
- Rerank mode: N/A
- Surface: N/A
- Command: `cargo test dml_frontdoor --lib`
- Head SHA: `1f1649bc0dd0725ad1e690d443ce5bc530a23e3d`
- Timestamp: `2026-05-11 19:43 PDT`
- Key result:
  - `Read-only file system (os error 30)`
  - `test result: FAILED. 11 passed; 14 failed; 0 ignored; 0 measured; 1648 filtered out; finished in 0.39s`
  - `COMMAND_EXIT_CODE="101"`
- Note: this is an environment failure from cargo-pgrx installing into the
  configured local PG18 tree. The rerun artifact above is the validation result
  cited by `request.md`.

### `cargo-fmt-check.log`

- Lane: Rust formatting check
- Fixture: repository formatting
- Storage format: N/A
- Rerank mode: N/A
- Surface: N/A
- Command: `cargo fmt --check`
- Head SHA: `1f1649bc0dd0725ad1e690d443ce5bc530a23e3d`
- Timestamp: `2026-05-11 19:45 PDT`
- Key result:
  - `COMMAND_EXIT_CODE="0"`
  - Known stable-rustfmt warnings are present for unstable
    `imports_granularity` and `group_imports` options.

### `git-diff-check.log`

- Lane: whitespace check
- Fixture: 30876 committed diff
- Storage format: N/A
- Rerank mode: N/A
- Surface: N/A
- Command: `git diff --check HEAD^ HEAD -- src/am/ec_spire/dml_frontdoor.rs`
- Head SHA: `1f1649bc0dd0725ad1e690d443ce5bc530a23e3d`
- Timestamp: `2026-05-11 19:45 PDT`
- Key result:
  - `COMMAND_EXIT_CODE="0"`
