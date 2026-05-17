# Artifact Manifest: 30870 SPIRE DML Primitive Invocation Builder

Head SHA: `ef87b8b075be871c210ab110cfd91344e16f30f6`

Packet/topic: `30870-spire-dml-primitive-invocation-builder`

Timestamp: `2026-05-11 16:14 PDT`

## Artifacts

### `cargo-test-dml-frontdoor-lib.log`

- Lane: Rust unit tests plus PG18 pgrx tests filtered by `dml_frontdoor`
- Fixture: DML frontdoor classifier, planner hook diagnostics, PK byte encoding, typed primitive plan, and primitive invocation coverage
- Storage format: N/A
- Rerank mode: N/A
- Surface: N/A
- Command: `cargo test dml_frontdoor --lib`
- Head SHA: `ef87b8b075be871c210ab110cfd91344e16f30f6`
- Timestamp: `2026-05-11 16:12 PDT`
- Key result:
  - `test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 1648 filtered out; finished in 17.38s`

### `cargo-fmt-check.log`

- Lane: Rust formatting check
- Fixture: repository formatting
- Storage format: N/A
- Rerank mode: N/A
- Surface: N/A
- Command: `cargo fmt --check`
- Head SHA: `ef87b8b075be871c210ab110cfd91344e16f30f6`
- Timestamp: `2026-05-11 16:12 PDT`
- Key result:
  - `COMMAND_EXIT_CODE="0"`
  - Known stable-rustfmt warnings are present for unstable `imports_granularity` and `group_imports` options.

### `git-diff-check.log`

- Lane: whitespace check
- Fixture: current working tree diff
- Storage format: N/A
- Rerank mode: N/A
- Surface: N/A
- Command: `git diff --check`
- Head SHA: `ef87b8b075be871c210ab110cfd91344e16f30f6`
- Timestamp: `2026-05-11 16:12 PDT`
- Key result:
  - `COMMAND_EXIT_CODE="0"`
