# Artifact Manifest: 30873 SPIRE DML PK SELECT CustomScan

Head SHA: `adcca43be8a86da8ed0be137a073e859cb1425aa`

Packet/topic: `30873-spire-dml-pk-select-customscan`

Timestamp: `2026-05-11 17:19 PDT`

## Artifacts

### `cargo-test-dml-frontdoor-lib.log`

- Lane: Rust unit tests plus PG18 pgrx tests filtered by `dml_frontdoor`
- Fixture: DML frontdoor classifier, replacement decision, primitive planning,
  primitive invocation, placement-gated PK SELECT CustomScan planning/execution
- Storage format: N/A
- Rerank mode: N/A
- Surface: N/A
- Command: `cargo test dml_frontdoor --lib`
- Head SHA: `adcca43be8a86da8ed0be137a073e859cb1425aa`
- Timestamp: `2026-05-11 17:19 PDT`
- Key result:
  - `test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 1648 filtered out; finished in 16.07s`
  - `COMMAND_EXIT_CODE="0"`

### `cargo-fmt-check.log`

- Lane: Rust formatting check
- Fixture: repository formatting
- Storage format: N/A
- Rerank mode: N/A
- Surface: N/A
- Command: `cargo fmt --check`
- Head SHA: `adcca43be8a86da8ed0be137a073e859cb1425aa`
- Timestamp: `2026-05-11 17:19 PDT`
- Key result:
  - `COMMAND_EXIT_CODE="0"`
  - Known stable-rustfmt warnings are present for unstable
    `imports_granularity` and `group_imports` options.

### `git-diff-check.log`

- Lane: whitespace check
- Fixture: 30873 committed diff
- Storage format: N/A
- Rerank mode: N/A
- Surface: N/A
- Command: `git diff --check ca4fa198fd34509f34bc6d2b8a73fda28ae5c907^ HEAD -- src/am/ec_spire/custom_scan.rs src/lib.rs plan/tasks/task30-phase11-spire-distributed-production-parity.md`
- Head SHA: `adcca43be8a86da8ed0be137a073e859cb1425aa`
- Timestamp: `2026-05-11 17:19 PDT`
- Key result:
  - `COMMAND_EXIT_CODE="0"`
