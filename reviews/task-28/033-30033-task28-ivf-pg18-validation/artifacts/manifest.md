# Artifact Manifest: Task 28 IVF PG18 Validation Gates

Head SHA: `33bfff75f2e3df304886345e5ca92f51b7dcf573`
Packet/topic: `30033-task28-ivf-pg18-validation`
Timestamp: 2026-04-25T15:08:35-07:00

## Passing Artifacts

### `unit-gate-cargo-test-pg18-lib-skip-pgtest-rerun.log`

- Lane: PG18 unit validation
- Fixture / storage format / rerank mode: pure Rust unit and lib tests; mixed repository coverage; IVF-specific trainer, codec, directory, scan, and insert helpers included
- Command: `cargo test --no-default-features --features pg18 --lib -- --skip pg_test`
- Surface isolation: not applicable; PostgreSQL `pg_test` cases were filtered out
- Key result lines:
  - `test result: ok. 372 passed; 0 failed; 0 ignored; 0 measured; 250 filtered out; finished in 31.19s`

### `extension-gate-cargo-pgrx-test-pg18-final.log`

- Lane: PG18 extension validation
- Fixture / storage format / rerank mode: full repository PG18 pgrx suite; mixed SQL fixtures and index surfaces, including `ec_ivf`
- Command: `cargo pgrx test pg18`
- Surface isolation: mixed suite; tests create their own fixtures as defined in the suite
- Key result lines:
  - `test result: ok. 618 passed; 0 failed; 4 ignored; 0 measured; 0 filtered out; finished in 90.67s`
  - `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s`
  - `test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 160.62s`
  - `test result: ok. 2 passed; 0 failed; 21 ignored; 0 measured; 0 filtered out; finished in 0.09s`
  - `test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`

### `lint-gate-cargo-clippy-pg18-rerun.log`

- Lane: PG18 lint validation
- Fixture / storage format / rerank mode: static analysis across all targets
- Command: `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- Surface isolation: not applicable
- Key result lines:
  - `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 12.62s`

## Diagnostic Artifacts

These logs are kept for traceability but are not cited as passing gates.

### `unit-gate-cargo-test-pg18-lib.log`

- Command: `cargo test --no-default-features --features pg18 --lib`
- Result: failed because plain `cargo test --lib` ran pgrx `pg_test` cases and attempted extension installation through the pgrx test framework.
- Key result lines:
  - `test result: FAILED. 372 passed; 246 failed; 4 ignored; 0 measured; 0 filtered out; finished in 31.53s`

### `extension-gate-cargo-pgrx-test-pg18.log`

- Command: `cargo pgrx test pg18`
- Result: failed inside the sandbox because pgrx could not install the extension into the normal test environment.
- Key result lines:
  - `Could not initialize test framework: Failure installing extension using command`

### `extension-gate-cargo-pgrx-test-pg18-rerun.log`

- Command: `cargo pgrx test pg18`
- Result: passing rerun before final clippy cleanup.
- Key result lines:
  - `test result: ok. 618 passed; 0 failed; 4 ignored; 0 measured; 0 filtered out; finished in 83.82s`

### `lint-gate-cargo-clippy-pg18.log`

- Command: `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- Result: failed on three PG18 lint findings that were fixed in checkpoint `33bfff75f2e3df304886345e5ca92f51b7dcf573`.
- Key result lines:
  - `error: redundant closure`
  - `error: unnecessary \`>= y + 1\` or \`x - 1 >=\``
  - `error: very complex type used. Consider factoring parts into \`type\` definitions`

### `unit-gate-cargo-test-pg18-lib-skip-pgtest.log`

- Command: `cargo test --no-default-features --features pg18 --lib -- --skip pg_test`
- Result: passing rerun before final clippy cleanup.
- Key result lines:
  - `test result: ok. 372 passed; 0 failed; 0 ignored; 0 measured; 250 filtered out; finished in 31.20s`
