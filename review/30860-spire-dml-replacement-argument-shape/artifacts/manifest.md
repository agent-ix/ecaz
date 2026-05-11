# Artifacts Manifest

Packet: `30860-spire-dml-replacement-argument-shape`
Head SHA: `9c73a87b6284a239c4d871f941c704b4d5f4d389`
Timestamp: `2026-05-11 15:01 PDT`
Surface: ADR-069 DML front-door replacement argument shape
Storage format / rerank mode: n/a
Isolated one-index-per-table vs shared-table surfaces: n/a

## cargo-test-dml-frontdoor-lib.log

- Command:
  `script -q -c "cargo test dml_frontdoor --lib" review/30860-spire-dml-replacement-argument-shape/artifacts/cargo-test-dml-frontdoor-lib.log`
- Lane / fixture:
  focused Rust unit + PG18 pgrx test filter for DML front-door surfaces.
- Key result lines:
  - `running 17 tests`
  - `test tests::pg_test_ec_spire_dml_frontdoor_replacement_decision_sql ... ok`
  - `test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 1648 filtered out; finished in 17.35s`

## cargo-fmt-check.log

- Command:
  `script -q -c "cargo fmt --check" review/30860-spire-dml-replacement-argument-shape/artifacts/cargo-fmt-check.log`
- Lane / fixture:
  Rust formatting check.
- Key result lines:
  - Command exited with code `0`.
  - Existing stable-rustfmt warnings about unstable import options are present.

## git-diff-check.log

- Command:
  `script -q -c "git diff --check" review/30860-spire-dml-replacement-argument-shape/artifacts/git-diff-check.log`
- Lane / fixture:
  whitespace/error check.
- Key result lines:
  - Command exited with code `0`.
