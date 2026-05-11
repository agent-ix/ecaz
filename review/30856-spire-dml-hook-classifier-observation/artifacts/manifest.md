# Artifacts Manifest

Packet: `30856-spire-dml-hook-classifier-observation`
Head SHA: `c27faed93e25a32b52d7df4075f1a6a73da4748f`
Timestamp: `2026-05-11 14:23 PDT`
Surface: ADR-069 DML front-door planner-hook classifier observation
Storage format / rerank mode: n/a
Isolated one-index-per-table vs shared-table surfaces: n/a

## cargo-test-dml-frontdoor-lib.log

- Command:
  `script -q -c "cargo test dml_frontdoor --lib" review/30856-spire-dml-hook-classifier-observation/artifacts/cargo-test-dml-frontdoor-lib.log`
- Lane / fixture:
  focused Rust unit + PG18 pgrx test filter for DML front-door surfaces.
- Key result lines:
  - `running 15 tests`
  - `test tests::pg_test_ec_spire_dml_frontdoor_const_coercion_and_cte ... ok`
  - `test tests::pg_test_ec_spire_dml_frontdoor_hook_status_installed_pass_through ... ok`
  - `test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 1648 filtered out; finished in 17.12s`

## cargo-fmt-check.log

- Command:
  `script -q -c "cargo fmt --check" review/30856-spire-dml-hook-classifier-observation/artifacts/cargo-fmt-check.log`
- Lane / fixture:
  Rust formatting check.
- Key result lines:
  - Command exited with code `0`.
  - Existing stable-rustfmt warnings about unstable import options are present.

## git-diff-check.log

- Command:
  `script -q -c "git diff --check" review/30856-spire-dml-hook-classifier-observation/artifacts/git-diff-check.log`
- Lane / fixture:
  whitespace/error check.
- Key result lines:
  - Command exited with code `0`.
