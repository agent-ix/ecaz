# Artifacts Manifest

Packet: `30864-spire-dml-pk-byte-encoding`
Head SHA: `8d447bf6f15012edb41692eaf5d8425987dee9a5`
Timestamp: `2026-05-11 15:34 PDT`
Surface: ADR-069 DML front-door bigint PK byte encoding
Storage format / rerank mode: n/a
Isolated one-index-per-table vs shared-table surfaces: n/a

## cargo-test-dml-frontdoor-lib.log

- Command:
  `script -q -c "cargo test dml_frontdoor --lib" review/30864-spire-dml-pk-byte-encoding/artifacts/cargo-test-dml-frontdoor-lib.log`
- Lane / fixture:
  focused Rust unit + PG18 pgrx test filter for DML front-door surfaces.
- Key result lines:
  - `running 20 tests`
  - `test tests::pg_test_ec_spire_dml_frontdoor_pk_value_bytes_match_int8send ... ok`
  - `test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 1648 filtered out; finished in 16.14s`

## cargo-fmt-check.log

- Command:
  `script -q -c "cargo fmt --check" review/30864-spire-dml-pk-byte-encoding/artifacts/cargo-fmt-check.log`
- Lane / fixture:
  Rust formatting check.
- Key result lines:
  - Command exited with code `0`.
  - Existing stable-rustfmt warnings about unstable import options are present.

## git-diff-check.log

- Command:
  `script -q -c "git diff --check" review/30864-spire-dml-pk-byte-encoding/artifacts/git-diff-check.log`
- Lane / fixture:
  whitespace/error check.
- Key result lines:
  - Command exited with code `0`.
