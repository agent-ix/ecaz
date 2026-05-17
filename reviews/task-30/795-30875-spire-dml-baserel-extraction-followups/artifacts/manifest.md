# Artifact Manifest: 30875 SPIRE DML Baserel Extraction Followups

Head SHA: `6efe8f84c1c6286b69e336e9536211880a971d03`

Packet/topic: `30875-spire-dml-baserel-extraction-followups`

Timestamp: `2026-05-11 19:33 PDT`

## Artifacts

### `cargo-test-dml-frontdoor-lib.log`

- Lane: Rust unit tests plus PG18 pgrx tests filtered by `dml_frontdoor`
- Fixture: DML frontdoor classifier, replacement decision, primitive planning,
  primitive invocation, and PK SELECT CustomScan coverage
- Storage format: N/A
- Rerank mode: N/A
- Surface: N/A
- Command: `cargo test dml_frontdoor --lib`
- Head SHA: `6efe8f84c1c6286b69e336e9536211880a971d03`
- Timestamp: `2026-05-11 19:32 PDT`
- Key result:
  - `test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 1648 filtered out; finished in 17.29s`
  - `COMMAND_EXIT_CODE="0"`

### `cargo-fmt-check.log`

- Lane: Rust formatting check
- Fixture: repository formatting
- Storage format: N/A
- Rerank mode: N/A
- Surface: N/A
- Command: `cargo fmt --check`
- Head SHA: `6efe8f84c1c6286b69e336e9536211880a971d03`
- Timestamp: `2026-05-11 19:33 PDT`
- Key result:
  - `COMMAND_EXIT_CODE="0"`
  - Known stable-rustfmt warnings are present for unstable
    `imports_granularity` and `group_imports` options.

### `git-diff-check.log`

- Lane: whitespace check
- Fixture: 30875 committed diff
- Storage format: N/A
- Rerank mode: N/A
- Surface: N/A
- Command: `git diff --check HEAD^ HEAD -- src/am/ec_spire/custom_scan.rs src/am/ec_spire/dml_frontdoor.rs plan/tasks/task30-phase11-spire-distributed-production-parity.md`
- Head SHA: `6efe8f84c1c6286b69e336e9536211880a971d03`
- Timestamp: `2026-05-11 19:33 PDT`
- Key result:
  - `COMMAND_EXIT_CODE="0"`
