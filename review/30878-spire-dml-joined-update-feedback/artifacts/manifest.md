# Artifact Manifest: 30878 SPIRE DML Joined UPDATE Feedback

Head SHA: `18663ced63a2429203b247c912dd70e58696960b`

Packet/topic: `30878-spire-dml-joined-update-feedback`

Timestamp: `2026-05-11 19:59 PDT`

## Artifacts

### `cargo-test-dml-frontdoor-lib.log`

- Lane: Rust unit tests plus PG18 pgrx tests filtered by `dml_frontdoor`
- Fixture: DML frontdoor classifier, replacement decision, primitive planning,
  primitive invocation, PK SELECT CustomScan coverage, and joined UPDATE
  rejection
- Storage format: N/A
- Rerank mode: N/A
- Surface: N/A
- Command: `script -q -e -c "cargo test dml_frontdoor --lib" review/30878-spire-dml-joined-update-feedback/artifacts/cargo-test-dml-frontdoor-lib.log`
- Head SHA: `18663ced63a2429203b247c912dd70e58696960b`
- Timestamp: `2026-05-11 19:58 PDT`
- Key result:
  - `test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 1649 filtered out; finished in 16.87s`
  - `COMMAND_EXIT_CODE="0"`

### `cargo-fmt-check.log`

- Lane: Rust formatting check
- Fixture: repository formatting
- Storage format: N/A
- Rerank mode: N/A
- Surface: N/A
- Command: `cargo fmt --check`
- Head SHA: `18663ced63a2429203b247c912dd70e58696960b`
- Timestamp: `2026-05-11 19:59 PDT`
- Key result:
  - `COMMAND_EXIT_CODE="0"`
  - Known stable-rustfmt warnings are present for unstable
    `imports_granularity` and `group_imports` options.

### `git-diff-check.log`

- Lane: whitespace check
- Fixture: 30878 committed diff
- Storage format: N/A
- Rerank mode: N/A
- Surface: N/A
- Command: `git diff --check HEAD^ HEAD -- src/am/ec_spire/dml_frontdoor.rs src/lib.rs`
- Head SHA: `18663ced63a2429203b247c912dd70e58696960b`
- Timestamp: `2026-05-11 19:59 PDT`
- Key result:
  - `COMMAND_EXIT_CODE="0"`
