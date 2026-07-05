# Artifact Manifest: 30872 SPIRE DML CustomScan Expression Handoff

Head SHA: `e68fa9667c8816b55044a91f2e4548a590c2c2ce`

Packet/topic: `30872-spire-dml-customscan-expression-handoff`

Timestamp: `2026-05-11 16:28 PDT`

## Artifacts

### `cargo-test-dml-frontdoor-lib.log`

- Lane: Rust unit tests plus PG18 pgrx tests filtered by `dml_frontdoor`
- Fixture: DML frontdoor classifier, replacement decision, typed primitive plan, primitive invocation, runtime PK bytes, and CustomScan expression handoff coverage
- Storage format: N/A
- Rerank mode: N/A
- Surface: N/A
- Command: `cargo test dml_frontdoor --lib`
- Head SHA: `e68fa9667c8816b55044a91f2e4548a590c2c2ce`
- Timestamp: `2026-05-11 16:27 PDT`
- Key result:
  - `test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 1648 filtered out; finished in 16.58s`

### `cargo-fmt-check.log`

- Lane: Rust formatting check
- Fixture: repository formatting
- Storage format: N/A
- Rerank mode: N/A
- Surface: N/A
- Command: `cargo fmt --check`
- Head SHA: `e68fa9667c8816b55044a91f2e4548a590c2c2ce`
- Timestamp: `2026-05-11 16:28 PDT`
- Key result:
  - `COMMAND_EXIT_CODE="0"`
  - Known stable-rustfmt warnings are present for unstable `imports_granularity` and `group_imports` options.

### `git-diff-check.log`

- Lane: whitespace check
- Fixture: 30872 committed diff
- Storage format: N/A
- Rerank mode: N/A
- Surface: N/A
- Command: `git diff --check HEAD^ HEAD -- src/am/ec_spire/dml_frontdoor.rs src/am/ec_spire/mod.rs src/am/mod.rs src/lib.rs plan/tasks/task30-phase11-spire-distributed-production-parity.md`
- Head SHA: `e68fa9667c8816b55044a91f2e4548a590c2c2ce`
- Timestamp: `2026-05-11 16:28 PDT`
- Key result:
  - `COMMAND_EXIT_CODE="0"`
