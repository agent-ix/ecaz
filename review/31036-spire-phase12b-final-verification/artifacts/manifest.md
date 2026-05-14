---
head_sha: b11ae95a58c9959a222f26042ee82076c78267c2
packet: 31036-spire-phase12b-final-verification
timestamp: 2026-05-14T09:23:46-07:00
---

# Artifact Manifest

## cargo-test-ecaz-rerun5.log

- head SHA: `b11ae95a58c9959a222f26042ee82076c78267c2`
- packet/topic: `31036-spire-phase12b-final-verification`
- lane / fixture / storage format / rerank mode: full crate test suite, PG18 pg_test plus Rust tests, mixed storage formats, default rerank modes
- command used: `cargo test -p ecaz`
- timestamp: 2026-05-14T09:17:00-07:00
- isolated one-index-per-table or shared-table surfaces: mixed suite coverage
- key result lines:
  - `test result: ok. 1714 passed; 0 failed; 4 ignored; 0 measured; 0 filtered out`
  - non-pg test targets and doc-tests all ended with `0 failed`

## cargo-pgrx-test-pg18.log

- head SHA: `b11ae95a58c9959a222f26042ee82076c78267c2`
- packet/topic: `31036-spire-phase12b-final-verification`
- lane / fixture / storage format / rerank mode: explicit PG18 pgrx suite, mixed storage formats, default rerank modes
- command used: `cargo pgrx test pg18`
- timestamp: 2026-05-14T09:23:00-07:00
- isolated one-index-per-table or shared-table surfaces: mixed suite coverage
- key result lines:
  - `test result: ok. 1714 passed; 0 failed; 4 ignored; 0 measured; 0 filtered out`
  - non-pg test targets and doc-tests all ended with `0 failed`

## src-am-ec-spire-line-counts.log

- head SHA: `b11ae95a58c9959a222f26042ee82076c78267c2`
- packet/topic: `31036-spire-phase12b-final-verification`
- lane / fixture / storage format / rerank mode: line-count audit for production SPIRE Rust files
- command used: `find src/am/ec_spire -name '*.rs' -printf '%p\n' | xargs wc -l | sort -nr`
- timestamp: 2026-05-14T09:23:00-07:00
- isolated one-index-per-table or shared-table surfaces: not applicable
- key result lines:
  - largest production file: `src/am/ec_spire/dml_frontdoor/mod.rs` = 2,498 lines
  - no `src/am/ec_spire/*.rs` file exceeds the 2,500-line cap

## src-tests-line-counts.log

- head SHA: `b11ae95a58c9959a222f26042ee82076c78267c2`
- packet/topic: `31036-spire-phase12b-final-verification`
- lane / fixture / storage format / rerank mode: line-count audit for evacuated PG18 fixture files
- command used: `find src/tests -name '*.rs' -printf '%p\n' | xargs wc -l | sort -nr`
- timestamp: 2026-05-14T09:23:00-07:00
- isolated one-index-per-table or shared-table surfaces: not applicable
- key result lines:
  - largest fixture file: `src/tests/remote_search/contracts.rs` = 2,864 lines
  - `src/tests/mod.rs` = 2,799 lines
  - no `src/tests/*.rs` file exceeds 3,000 lines

## lib-mod-spire-fixture-grep.log

- head SHA: `b11ae95a58c9959a222f26042ee82076c78267c2`
- packet/topic: `31036-spire-phase12b-final-verification`
- lane / fixture / storage format / rerank mode: fixture-sink absence check
- command used: `rg -n "test_ec_spire_" src/lib.rs src/tests/mod.rs`
- timestamp: 2026-05-14T09:23:00-07:00
- isolated one-index-per-table or shared-table surfaces: not applicable
- key result lines:
  - no matches; the artifact is intentionally empty

## Earlier Diagnostic Logs

The packet also retains `cargo-test-ecaz.log`, `cargo-test-ecaz-rerun1.log`,
`cargo-test-ecaz-rerun2.log`, `cargo-test-ecaz-rerun3.log`, and
`cargo-test-ecaz-rerun4.log` as raw diagnostic history from the final
verification loop. `rerun3` documents the sandboxed read-only filesystem
failure during pgrx install; `rerun4` documents the single mixed-delta
assertion that was corrected before `rerun5`.
