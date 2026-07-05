# Artifact Manifest: 31019 SPIRE Post-Build Insert Fixture Split

Head SHA: `107a6d3af3edc49f9d9a07ba78bbc7035eea37f9`
Packet/topic: `31019-spire-post-build-insert-fixture-split`
Timestamp: `2026-05-13T17:55:37-07:00`
Lane: Phase 12b cleanup, insert fixture relocation
Fixture: post-build insert multi-row, validation, and empty-index
bootstrap
Storage format: unchanged existing SPIRE test fixtures
Rerank mode: not applicable
Surface isolation: not a measurement run; existing unit-test fixtures only

## Artifacts

### `cargo-fmt-check.log`

Command:

```sh
cargo fmt --check
```

Key result:

```text
Script done on 2026-05-13 17:52:36-07:00 [COMMAND_EXIT_CODE="0"]
```

Notes: stable rustfmt emitted the repository's existing unstable-option
warnings for `imports_granularity` and `group_imports`.

### `cargo-test-insert-multi-row-epoch.log`

Command:

```sh
cargo test --no-default-features --features pg18 test_ec_spire_insert_after_build_multi_row_epoch_progression -- --nocapture
```

Key result:

```text
test tests::pg_test_ec_spire_insert_after_build_multi_row_epoch_progression ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1711 filtered out; finished in 38.24s
```

### `cargo-test-insert-bad-dimension.log`

Command:

```sh
cargo test --no-default-features --features pg18 test_ec_spire_insert_after_build_rejects_dimension_mismatch -- --nocapture
```

Key result:

```text
test tests::pg_test_ec_spire_insert_after_build_rejects_dimension_mismatch - should panic ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1711 filtered out; finished in 39.29s
```

### `location-check.log`

Command:

```sh
rg -n 'fn test_ec_spire_insert_after_build_multi_row_epoch_progression|fn test_ec_spire_insert_after_build_rejects_dimension_mismatch|fn test_ec_spire_insert_bootstraps_empty_index_epoch|fn test_ec_spire_srcid_uuid_global_ids' src/tests/insert.rs src/tests/mod.rs
```

Key result:

```text
src/tests/insert.rs
2068:    fn test_ec_spire_insert_after_build_multi_row_epoch_progression() {
2131:    fn test_ec_spire_insert_after_build_rejects_dimension_mismatch() {
2179:    fn test_ec_spire_insert_bootstraps_empty_index_epoch() {

src/tests/mod.rs
12782:    fn test_ec_spire_srcid_uuid_global_ids() {
```

### `line-counts.log`

Command:

```sh
wc -l src/tests/mod.rs src/tests/insert.rs src/lib.rs
```

Key result:

```text
  35736 src/tests/mod.rs
   2226 src/tests/insert.rs
  17812 src/lib.rs
  55774 total
```

### `git-diff-check.log`

Command:

```sh
git diff --check
```

Key result:

```text
Script done on 2026-05-13 17:52:35-07:00 [COMMAND_EXIT_CODE="0"]
```
