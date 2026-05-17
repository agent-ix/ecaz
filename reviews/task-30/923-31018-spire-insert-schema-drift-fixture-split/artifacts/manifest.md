# Artifact Manifest: 31018 SPIRE Insert Schema Drift Fixture Split

Head SHA: `efb2786c5acae68058539ed07fd4719a65186b56`
Packet/topic: `31018-spire-insert-schema-drift-fixture-split`
Timestamp: `2026-05-13T17:45:02-07:00`
Lane: Phase 12b cleanup, insert fixture relocation
Fixture: coordinator insert schema drift and remote schema fingerprint
pre-dispatch
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
Script done on 2026-05-13 17:40:00-07:00 [COMMAND_EXIT_CODE="0"]
```

Notes: stable rustfmt emitted the repository's existing unstable-option
warnings for `imports_granularity` and `group_imports`.

### `cargo-test-schema-drift.log`

Command:

```sh
cargo test --no-default-features --features pg18 test_ec_spire_schema_drift_fails_before_dispatch_sql -- --nocapture
```

Key result:

```text
test tests::pg_test_ec_spire_schema_drift_fails_before_dispatch_sql ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1711 filtered out; finished in 35.78s
```

### `cargo-test-remote-schema-fingerprint.log`

Command:

```sh
cargo test --no-default-features --features pg18 test_ec_spire_remote_schema_fingerprint_pre_dispatch_sql -- --nocapture
```

Key result:

```text
test tests::pg_test_ec_spire_remote_schema_fingerprint_pre_dispatch_sql ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1711 filtered out; finished in 31.65s
```

### `location-check.log`

Command:

```sh
rg -n 'fn test_ec_spire_schema_drift_fails_before_dispatch_sql|fn test_ec_spire_remote_schema_fingerprint_pre_dispatch_sql' src/tests/insert.rs src/tests/mod.rs
```

Key result:

```text
src/tests/insert.rs
1767:    fn test_ec_spire_schema_drift_fails_before_dispatch_sql() {
1908:    fn test_ec_spire_remote_schema_fingerprint_pre_dispatch_sql() {
```

### `line-counts.log`

Command:

```sh
wc -l src/tests/mod.rs src/tests/insert.rs src/lib.rs
```

Key result:

```text
  35897 src/tests/mod.rs
   2065 src/tests/insert.rs
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
Script done on 2026-05-13 17:44:33-07:00 [COMMAND_EXIT_CODE="0"]
```
