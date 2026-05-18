# Artifact Manifest: 30871 SPIRE DML PK Byte Boundary Feedback

Head SHA: `d8a00e18a9127a037252219e5fd5a1392e43d25a`

Packet/topic: `30871-spire-dml-pk-byte-boundary-feedback`

Timestamp: `2026-05-11 16:18 PDT`

## Artifacts

### `cargo-test-dml-frontdoor-lib.log`

- Lane: Rust unit tests plus PG18 pgrx tests filtered by `dml_frontdoor`
- Fixture: DML frontdoor classifier, planner hook diagnostics, PK byte encoding, typed primitive plan, runtime parameter bytea conversion, and primitive invocation coverage
- Storage format: N/A
- Rerank mode: N/A
- Surface: N/A
- Command: `cargo test dml_frontdoor --lib`
- Head SHA: `d8a00e18a9127a037252219e5fd5a1392e43d25a`
- Timestamp: `2026-05-11 16:18 PDT`
- Key result:
  - `test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 1648 filtered out; finished in 15.84s`

### `cargo-fmt-check.log`

- Lane: Rust formatting check
- Fixture: repository formatting
- Storage format: N/A
- Rerank mode: N/A
- Surface: N/A
- Command: `cargo fmt --check`
- Head SHA: `d8a00e18a9127a037252219e5fd5a1392e43d25a`
- Timestamp: `2026-05-11 16:18 PDT`
- Key result:
  - `COMMAND_EXIT_CODE="0"`
  - Known stable-rustfmt warnings are present for unstable `imports_granularity` and `group_imports` options.

### `git-diff-check.log`

- Lane: whitespace check
- Fixture: 30871 committed diff
- Storage format: N/A
- Rerank mode: N/A
- Surface: N/A
- Command: `git diff --check HEAD^ HEAD -- src/am/ec_spire/dml_frontdoor.rs src/lib.rs plan/tasks/task30-phase11-spire-distributed-production-parity.md`
- Head SHA: `d8a00e18a9127a037252219e5fd5a1392e43d25a`
- Timestamp: `2026-05-11 16:18 PDT`
- Key result:
  - `COMMAND_EXIT_CODE="0"`
