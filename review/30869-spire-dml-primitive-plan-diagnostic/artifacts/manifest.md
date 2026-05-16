# Artifacts Manifest

Packet: `30869-spire-dml-primitive-plan-diagnostic`
Head SHA: `a9d8df2c944303589117f72a38f7c512eebd37d0`
Timestamp: `2026-05-11 16:07 PDT`
Surface: ADR-069 DML front-door primitive plan SQL diagnostic
Storage format / rerank mode: n/a
Isolated one-index-per-table vs shared-table surfaces: n/a

## cargo-test-dml-frontdoor-lib.log

- Command:
  `script -q -c "cargo test dml_frontdoor --lib" review/30869-spire-dml-primitive-plan-diagnostic/artifacts/cargo-test-dml-frontdoor-lib.log`
- Lane / fixture:
  focused Rust unit + PG18 pgrx test filter for DML front-door surfaces.
- Key result lines:
  - `running 23 tests`
  - `test tests::pg_test_ec_spire_dml_frontdoor_primitive_plan_sql ... ok`
  - `test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 1648 filtered out; finished in 16.49s`

## cargo-fmt-check.log

- Command:
  `script -q -c "cargo fmt --check" review/30869-spire-dml-primitive-plan-diagnostic/artifacts/cargo-fmt-check.log`
- Lane / fixture:
  Rust formatting check.
- Key result lines:
  - Command exited with code `0`.
  - Existing stable-rustfmt warnings about unstable import options are present.

## git-diff-check.log

- Command:
  `script -q -c "git diff --check" review/30869-spire-dml-primitive-plan-diagnostic/artifacts/git-diff-check.log`
- Lane / fixture:
  whitespace/error check.
- Key result lines:
  - Command exited with code `0`.
