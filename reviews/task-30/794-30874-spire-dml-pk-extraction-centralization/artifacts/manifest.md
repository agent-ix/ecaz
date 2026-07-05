# Artifact Manifest: 30874 SPIRE DML PK Extraction Centralization

Head SHA: `bcb8de5f210c6c6c2e6c53f54bd388b7016dd652`

Packet/topic: `30874-spire-dml-pk-extraction-centralization`

Timestamp: `2026-05-11 18:10 PDT`

## Artifacts

### `cargo-test-dml-frontdoor-lib.log`

- Lane: Rust unit tests plus PG18 pgrx tests filtered by `dml_frontdoor`
- Fixture: DML frontdoor classifier, replacement decision, primitive planning,
  primitive invocation, and PK SELECT CustomScan coverage
- Storage format: N/A
- Rerank mode: N/A
- Surface: N/A
- Command: `cargo test dml_frontdoor --lib`
- Head SHA: `bcb8de5f210c6c6c2e6c53f54bd388b7016dd652`
- Timestamp: `2026-05-11 18:09 PDT`
- Key result:
  - `test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 1648 filtered out; finished in 17.47s`
  - `COMMAND_EXIT_CODE="0"`

### `cargo-fmt-check.log`

- Lane: Rust formatting check
- Fixture: repository formatting
- Storage format: N/A
- Rerank mode: N/A
- Surface: N/A
- Command: `cargo fmt --check`
- Head SHA: `bcb8de5f210c6c6c2e6c53f54bd388b7016dd652`
- Timestamp: `2026-05-11 18:10 PDT`
- Key result:
  - `COMMAND_EXIT_CODE="0"`
  - Known stable-rustfmt warnings are present for unstable
    `imports_granularity` and `group_imports` options.

### `git-diff-check.log`

- Lane: whitespace check
- Fixture: 30874 committed diff
- Storage format: N/A
- Rerank mode: N/A
- Surface: N/A
- Command: `git diff --check HEAD^ HEAD -- src/am/ec_spire/custom_scan.rs src/am/ec_spire/dml_frontdoor.rs src/am/ec_spire/mod.rs plan/tasks/task30-phase11-spire-distributed-production-parity.md`
- Head SHA: `bcb8de5f210c6c6c2e6c53f54bd388b7016dd652`
- Timestamp: `2026-05-11 18:10 PDT`
- Key result:
  - `COMMAND_EXIT_CODE="0"`
