# Artifact Manifest: 31020 SPIRE Insert Delta Fixture Split

Head SHA: `a75c8479970788c55899ad8a5aae70b8f964a3ac`
Packet/topic: `31020-spire-insert-delta-fixture-split`
Timestamp: `2026-05-13T18:02:59-07:00`
Lane: Phase 12b cleanup, insert fixture relocation
Fixture: post-build insert deltas and PG18 concurrent same-leaf inserts
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
Script done on 2026-05-13 17:57:53-07:00 [COMMAND_EXIT_CODE="0"]
```

Notes: stable rustfmt emitted the repository's existing unstable-option
warnings for `imports_granularity` and `group_imports`.

### `cargo-test-insert-same-leaf-deltas.log`

Command:

```sh
cargo test --no-default-features --features pg18 test_ec_spire_insert_after_build_multiple_same_leaf_deltas -- --nocapture
```

Key result:

```text
test tests::pg_test_ec_spire_insert_after_build_multiple_same_leaf_deltas ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1711 filtered out; finished in 39.21s
```

### `cargo-test-concurrent-same-leaf-inserts.log`

Command:

```sh
cargo test --no-default-features --features pg18 test_pg18_ec_spire_concurrent_same_leaf_inserts -- --nocapture
```

Key result:

```text
test tests::pg_test_pg18_ec_spire_concurrent_same_leaf_inserts ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1711 filtered out; finished in 34.79s
```

### `location-check.log`

Command:

```sh
rg -n 'fn test_ec_spire_insert_after_build_delta_epoch|fn test_ec_spire_insert_after_build_multiple_same_leaf_deltas|fn test_pg18_ec_spire_concurrent_same_leaf_inserts|fn test_pg18_ec_spire_placement_write_contention_distinct_pk_dml' src/tests/insert.rs src/tests/mod.rs
```

Key result:

```text
src/tests/insert.rs
2229:    fn test_ec_spire_insert_after_build_delta_epoch() {
2271:    fn test_ec_spire_insert_after_build_multiple_same_leaf_deltas() {
2374:    fn test_pg18_ec_spire_concurrent_same_leaf_inserts() {

src/tests/mod.rs
12372:    fn test_pg18_ec_spire_placement_write_contention_distinct_pk_dml() {
```

### `line-counts.log`

Command:

```sh
wc -l src/tests/mod.rs src/tests/insert.rs src/lib.rs
```

Key result:

```text
  35492 src/tests/mod.rs
   2470 src/tests/insert.rs
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
Script done on 2026-05-13 17:57:52-07:00 [COMMAND_EXIT_CODE="0"]
```
