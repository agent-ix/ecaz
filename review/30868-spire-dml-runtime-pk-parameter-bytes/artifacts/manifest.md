# Artifacts Manifest

Packet: `30868-spire-dml-runtime-pk-parameter-bytes`
Head SHA: `aebdf0c5dd0648fa4be0f85a36c5e2ca9a1d6f11`
Timestamp: `2026-05-11 16:01 PDT`
Surface: ADR-069 DML front-door runtime PK parameter byte conversion
Storage format / rerank mode: n/a
Isolated one-index-per-table vs shared-table surfaces: n/a

## cargo-test-dml-frontdoor-lib.log

- Command:
  `script -q -c "cargo test dml_frontdoor --lib" review/30868-spire-dml-runtime-pk-parameter-bytes/artifacts/cargo-test-dml-frontdoor-lib.log`
- Lane / fixture:
  focused Rust unit + PG18 pgrx test filter for DML front-door surfaces.
- Key result lines:
  - `running 22 tests`
  - `test tests::pg_test_ec_spire_dml_frontdoor_primitive_plan_from_decision ... ok`
  - `test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 1648 filtered out; finished in 16.03s`

## cargo-fmt-check.log

- Command:
  `script -q -c "cargo fmt --check" review/30868-spire-dml-runtime-pk-parameter-bytes/artifacts/cargo-fmt-check.log`
- Lane / fixture:
  Rust formatting check.
- Key result lines:
  - Command exited with code `0`.
  - Existing stable-rustfmt warnings about unstable import options are present.

## git-diff-check.log

- Command:
  `script -q -c "git diff --check" review/30868-spire-dml-runtime-pk-parameter-bytes/artifacts/git-diff-check.log`
- Lane / fixture:
  whitespace/error check.
- Key result lines:
  - Command exited with code `0`.
