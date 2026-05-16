# Artifacts Manifest

Packet: `30867-spire-dml-primitive-pk-bytes`
Head SHA: `c7baa829e4129979f042ca01d0d3ee632832ce22`
Timestamp: `2026-05-11 15:55 PDT`
Surface: ADR-069 DML front-door primitive PK byte conversion
Storage format / rerank mode: n/a
Isolated one-index-per-table vs shared-table surfaces: n/a

## cargo-test-dml-frontdoor-lib.log

- Command:
  `script -q -c "cargo test dml_frontdoor --lib" review/30867-spire-dml-primitive-pk-bytes/artifacts/cargo-test-dml-frontdoor-lib.log`
- Lane / fixture:
  focused Rust unit + PG18 pgrx test filter for DML front-door surfaces.
- Key result lines:
  - `running 22 tests`
  - `test tests::pg_test_ec_spire_dml_frontdoor_primitive_plan_from_decision ... ok`
  - `test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 1648 filtered out; finished in 17.64s`

## cargo-fmt-check.log

- Command:
  `script -q -c "cargo fmt --check" review/30867-spire-dml-primitive-pk-bytes/artifacts/cargo-fmt-check.log`
- Lane / fixture:
  Rust formatting check.
- Key result lines:
  - Command exited with code `0`.
  - Existing stable-rustfmt warnings about unstable import options are present.

## git-diff-check.log

- Command:
  `script -q -c "git diff --check" review/30867-spire-dml-primitive-pk-bytes/artifacts/git-diff-check.log`
- Lane / fixture:
  whitespace/error check.
- Key result lines:
  - Command exited with code `0`.
