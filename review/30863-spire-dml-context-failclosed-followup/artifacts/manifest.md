# Artifacts Manifest

Packet: `30863-spire-dml-context-failclosed-followup`
Head SHA: `e78724fc5a18d6bd4f4af826fb491b20cf265c21`
Timestamp: `2026-05-11 15:26 PDT`
Surface: ADR-069 DML front-door fail-closed context-error follow-up
Storage format / rerank mode: n/a
Isolated one-index-per-table vs shared-table surfaces: n/a

## cargo-test-dml-frontdoor-lib.log

- Command:
  `script -q -c "cargo test dml_frontdoor --lib" review/30863-spire-dml-context-failclosed-followup/artifacts/cargo-test-dml-frontdoor-lib.log`
- Lane / fixture:
  focused Rust unit + PG18 pgrx test filter for DML front-door surfaces.
- Key result lines:
  - `running 19 tests`
  - `test tests::pg_test_ec_spire_dml_frontdoor_hook_fail_closed_context_error ... ok`
  - `test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 1648 filtered out; finished in 17.07s`

## cargo-fmt-check.log

- Command:
  `script -q -c "cargo fmt --check" review/30863-spire-dml-context-failclosed-followup/artifacts/cargo-fmt-check.log`
- Lane / fixture:
  Rust formatting check.
- Key result lines:
  - Command exited with code `0`.
  - Existing stable-rustfmt warnings about unstable import options are present.

## git-diff-check.log

- Command:
  `script -q -c "git diff --check" review/30863-spire-dml-context-failclosed-followup/artifacts/git-diff-check.log`
- Lane / fixture:
  whitespace/error check.
- Key result lines:
  - Command exited with code `0`.
