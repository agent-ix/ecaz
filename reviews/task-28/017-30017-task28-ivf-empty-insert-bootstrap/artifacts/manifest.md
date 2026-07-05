# Artifact Manifest: Task 28 IVF Empty Bootstrap Serialization

Head SHA: `cb3c75ae71f6d897c43fd9781d24be647a2fae66`

Packet/topic: `30017-task28-ivf-empty-insert-bootstrap`

Lane: PG18 correctness validation

Timestamp: `2026-04-25T23:58:31Z`

Measurement claim: none. These artifacts are validation logs for the reviewed
empty-index live-insert bootstrap race fix.

## Artifacts

- `pg18-cargo-check-tests.log`
  - Command: `cargo check --no-default-features --features pg18 --tests`
  - Fixture: compile all PG18 test targets
  - Storage format / rerank mode: not applicable
  - Table surface: not applicable
  - Key result: `Finished dev profile [unoptimized + debuginfo] target(s) in 0.16s`

- `pg18-unit-skip-pgtest.log`
  - Command: `cargo test --no-default-features --features pg18 --lib -- --skip pg_test`
  - Fixture: PG18 Rust unit coverage with pgrx SQL tests filtered out
  - Storage format / rerank mode: mixed unit fixtures / not applicable
  - Table surface: not applicable
  - Key result: `test result: ok. 372 passed; 0 failed; 0 ignored; 0 measured; 251 filtered out; finished in 31.43s`

- `pg18-clippy.log`
  - Command: `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
  - Fixture: PG18 lint gate
  - Storage format / rerank mode: not applicable
  - Table surface: not applicable
  - Key result: `Finished dev profile [unoptimized + debuginfo] target(s) in 10.16s`

- `pg18-empty-bootstrap-regression.log`
  - Command: `cargo pgrx test pg18 test_pg18_ec_ivf_concurrent_empty_bootstrap_reachable`
  - Fixture: two worker sessions insert concurrently into an empty heap/index
  - Storage format / rerank mode: default IVF ecvector storage / `off`
  - Table surface: isolated empty table and one IVF index
  - Key result: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 622 filtered out; finished in 16.23s`

- `pg18-same-list-regression.log`
  - Command: `cargo pgrx test pg18 test_pg18_ec_ivf_concurrent_same_list_inserts_remain_reachable`
  - Fixture: two worker sessions insert concurrently into the same IVF list
  - Storage format / rerank mode: default IVF ecvector storage / `off`
  - Table surface: isolated table and one IVF index
  - Key result: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 622 filtered out; finished in 16.08s`

- `pg18-pgrx-full.log`
  - Command: `cargo pgrx test pg18`
  - Fixture: full PG18 pgrx extension suite
  - Storage format / rerank mode: mixed PG fixtures
  - Table surface: mixed isolated PG test tables
  - Key results:
    - `test result: ok. 619 passed; 0 failed; 4 ignored; 0 measured; 0 filtered out; finished in 84.28s`
    - `test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 220.99s`
    - `test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`
