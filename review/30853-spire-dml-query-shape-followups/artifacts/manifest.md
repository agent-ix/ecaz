# Artifacts Manifest

Packet: `30853-spire-dml-query-shape-followups`
Head SHA: `af8dc5663e90c66ce58691d644b3c50f51830425`
Timestamp: `2026-05-11 13:59 PDT`
Surface: ADR-069 DML front-door query extraction
Storage format / rerank mode: n/a
Isolated one-index-per-table vs shared-table surfaces: n/a

## cargo-test-dml-frontdoor-lib.log

- Command:
  `script -q -c "cargo test dml_frontdoor --lib" review/30853-spire-dml-query-shape-followups/artifacts/cargo-test-dml-frontdoor-lib.log`
- Lane / fixture:
  focused Rust unit + PG18 pgrx test filter for DML front-door surfaces.
- Key result lines:
  - `running 15 tests`
  - `test am::ec_spire::dml_frontdoor::tests::query_layer_recognizes_bigint_integer_equality_variants ... ok`
  - `test tests::pg_test_ec_spire_dml_frontdoor_const_coercion_and_cte ... ok`
  - `test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 1648 filtered out; finished in 15.58s`

## cargo-fmt-check.log

- Command:
  `script -q -c "cargo fmt --check" review/30853-spire-dml-query-shape-followups/artifacts/cargo-fmt-check.log`
- Lane / fixture:
  Rust formatting check.
- Key result lines:
  - Command exited with code `0`.
  - Existing stable-rustfmt warnings about unstable import options are present.

## git-diff-check.log

- Command:
  `script -q -c "git diff --check" review/30853-spire-dml-query-shape-followups/artifacts/git-diff-check.log`
- Lane / fixture:
  whitespace/error check.
- Key result lines:
  - Command exited with code `0`.
