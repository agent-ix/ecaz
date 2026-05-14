# Artifact Manifest: 31021 SPIRE Source Identity Insert Fixture Split

Head SHA: `f8825e31ef8dac5da610de02b04965cf23bd79d5`
Packet/topic: `31021-spire-source-identity-insert-fixture-split`
Timestamp: `2026-05-13T18:13:37-07:00`
Lane: Phase 12b cleanup, insert fixture relocation
Fixture: SPIRE source-identity include/global-id fixtures
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
Script done on 2026-05-13 18:06:21-07:00 [COMMAND_EXIT_CODE="0"]
```

Notes: stable rustfmt emitted the repository's existing unstable-option
warnings for `imports_granularity` and `group_imports`.

### `cargo-test-srcid-uuid-global.log`

Command:

```sh
cargo test --no-default-features --features pg18 test_ec_spire_srcid_uuid_global_ids -- --nocapture
```

Key result:

```text
test tests::pg_test_ec_spire_srcid_uuid_global_ids ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1711 filtered out; finished in 40.47s
```

### `cargo-test-srcid-bytea-bootstrap.log`

Command:

```sh
cargo test --no-default-features --features pg18 test_ec_spire_srcid_bytea_bootstrap_global -- --nocapture
```

Key result:

```text
test tests::pg_test_ec_spire_srcid_bytea_bootstrap_global ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1711 filtered out; finished in 32.63s
```

### `cargo-test-srcid-bad-bytea.log`

Command:

```sh
cargo test --no-default-features --features pg18 test_ec_spire_srcid_rejects_bad_bytea_width -- --nocapture
```

Key result:

```text
test tests::pg_test_ec_spire_srcid_rejects_bad_bytea_width - should panic ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1711 filtered out; finished in 33.46s
```

### `location-check.log`

Command:

```sh
rg -n 'fn test_ec_spire_srcid_uuid_global_ids|fn test_ec_spire_srcid_bytea_bootstrap_global|fn test_ec_spire_srcid_rejects_bad_bytea_width|single-key indexes only' src/tests/insert.rs src/tests/mod.rs src/am/ec_hnsw/source.rs
```

Key result:

```text
src/am/ec_hnsw/source.rs
285:        pgrx::error!("ec_hnsw {label} currently supports single-key indexes only");

src/tests/insert.rs
2473:    fn test_ec_spire_srcid_uuid_global_ids() {
2648:    fn test_ec_spire_srcid_bytea_bootstrap_global() {
2793:    fn test_ec_spire_srcid_rejects_bad_bytea_width() {
```

### `line-counts.log`

Command:

```sh
wc -l src/tests/mod.rs src/tests/insert.rs src/lib.rs src/am/ec_hnsw/source.rs
```

Key result:

```text
  35148 src/tests/mod.rs
   2814 src/tests/insert.rs
  17812 src/lib.rs
    765 src/am/ec_hnsw/source.rs
  56539 total
```

### `git-diff-check.log`

Command:

```sh
git diff --check
```

Key result:

```text
Script done on 2026-05-13 18:06:20-07:00 [COMMAND_EXIT_CODE="0"]
```
