# Artifacts Manifest

Packet: `30857-spire-dml-feedback-hardening`
Code commit: `500876c0c602c8ef13a01286f489ed8f3ba1c735`
Artifact head SHA: `52d9cca2d05c63c34c98925d9960feeb4fb7ebfd`
Timestamp: `2026-05-11 14:30 PDT`
Surface: ADR-069 DML front-door classifier feedback follow-up
Storage format / rerank mode: n/a
Isolated one-index-per-table vs shared-table surfaces: n/a

## cargo-test-dml-frontdoor-lib.log

- Command:
  `script -q -c "cargo test dml_frontdoor --lib" review/30857-spire-dml-feedback-hardening/artifacts/cargo-test-dml-frontdoor-lib.log`
- Lane / fixture:
  focused Rust unit + PG18 pgrx test filter for DML front-door surfaces.
- Key result lines:
  - `running 16 tests`
  - `test am::ec_spire::dml_frontdoor::tests::query_layer_walks_nested_integer_coercion_wrappers ... ok`
  - `test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 1648 filtered out; finished in 28.73s`

## cargo-fmt-check.log

- Command:
  `script -q -c "cargo fmt --check" review/30857-spire-dml-feedback-hardening/artifacts/cargo-fmt-check.log`
- Lane / fixture:
  Rust formatting check.
- Key result lines:
  - Command exited with code `0`.
  - Existing stable-rustfmt warnings about unstable import options are present.

## git-diff-check.log

- Command:
  `script -q -c "git diff --check" review/30857-spire-dml-feedback-hardening/artifacts/git-diff-check.log`
- Lane / fixture:
  whitespace/error check.
- Key result lines:
  - Command exited with code `0`.
