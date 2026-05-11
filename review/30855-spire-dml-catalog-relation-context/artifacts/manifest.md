# Artifacts Manifest

Packet: `30855-spire-dml-catalog-relation-context`
Head SHA: `88cba0d818bab0e5876c2137983fff95c8a0ce2d`
Timestamp: `2026-05-11 14:15 PDT`
Surface: ADR-069 DML front-door relation context
Storage format / rerank mode: n/a
Isolated one-index-per-table vs shared-table surfaces: n/a

## cargo-test-dml-frontdoor-lib.log

- Command:
  `script -q -c "cargo test dml_frontdoor --lib" review/30855-spire-dml-catalog-relation-context/artifacts/cargo-test-dml-frontdoor-lib.log`
- Lane / fixture:
  focused Rust unit + PG18 pgrx test filter for DML front-door surfaces.
- Key result lines:
  - `running 15 tests`
  - `test tests::pg_test_ec_spire_dml_frontdoor_relation_context_sql ... ok`
  - `test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 1648 filtered out; finished in 16.71s`

## cargo-fmt-check.log

- Command:
  `script -q -c "cargo fmt --check" review/30855-spire-dml-catalog-relation-context/artifacts/cargo-fmt-check.log`
- Lane / fixture:
  Rust formatting check.
- Key result lines:
  - Command exited with code `0`.
  - Existing stable-rustfmt warnings about unstable import options are present.

## git-diff-check.log

- Command:
  `script -q -c "git diff --check" review/30855-spire-dml-catalog-relation-context/artifacts/git-diff-check.log`
- Lane / fixture:
  whitespace/error check.
- Key result lines:
  - Command exited with code `0`.
