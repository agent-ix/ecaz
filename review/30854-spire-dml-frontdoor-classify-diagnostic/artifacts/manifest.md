# Artifacts Manifest

Packet: `30854-spire-dml-frontdoor-classify-diagnostic`
Head SHA: `0788cc0154ba59927780d23ac28ac908f2778f29`
Timestamp: `2026-05-11 14:05 PDT`
Surface: ADR-069 DML front-door classifier diagnostic
Storage format / rerank mode: n/a
Isolated one-index-per-table vs shared-table surfaces: n/a

## cargo-test-dml-frontdoor-lib.log

- Command:
  `script -q -c "cargo test dml_frontdoor --lib" review/30854-spire-dml-frontdoor-classify-diagnostic/artifacts/cargo-test-dml-frontdoor-lib.log`
- Lane / fixture:
  focused Rust unit + PG18 pgrx test filter for DML front-door surfaces.
- Key result lines:
  - `running 15 tests`
  - `test tests::pg_test_ec_spire_dml_frontdoor_const_coercion_and_cte ... ok`
  - `test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 1648 filtered out; finished in 16.11s`

## cargo-fmt-check.log

- Command:
  `script -q -c "cargo fmt --check" review/30854-spire-dml-frontdoor-classify-diagnostic/artifacts/cargo-fmt-check.log`
- Lane / fixture:
  Rust formatting check.
- Key result lines:
  - Command exited with code `0`.
  - Existing stable-rustfmt warnings about unstable import options are present.

## git-diff-check.log

- Command:
  `script -q -c "git diff --check" review/30854-spire-dml-frontdoor-classify-diagnostic/artifacts/git-diff-check.log`
- Lane / fixture:
  whitespace/error check.
- Key result lines:
  - Command exited with code `0`.
