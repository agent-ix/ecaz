# 30756 Artifact Manifest

Head SHA: `dc724acf6dedab0f982306c7e7cc13024634da16`

Packet: `30756-spire-production-scan-result-stream`

Scope: SPIRE Phase 11 Stage D Rust-side production scan result-stream state.

Lane: local PG18. Fixture: packet-specific PG18 SQL wrapper reusing
`tests.test_ec_spire_prod_scan_heap_resolution()`. Storage format: `rabitq`.
Rerank mode: exact heap-vector rerank after origin-node heap visibility
resolution. Surface shape: isolated one-index-per-table loopback fixture with
one coordinator leaf placement rewritten to a remote node descriptor.

## Artifacts

- `cargo-fmt-check.log`
  - Command: `script -q -e -c 'cargo fmt --check' ...`
  - Timestamp: 2026-05-10 11:59 PDT
  - Result: pass; only existing stable-rustfmt warnings for
    `imports_granularity` / `group_imports`.

- `cargo-check-pg18.log`
  - Command: `script -q -e -c 'cargo check --no-default-features --features pg18' ...`
  - Timestamp: 2026-05-10 11:59 PDT
  - Result line: `Finished dev profile ... target(s) in 0.12s`.

- `cargo-check-pg18-pg-test.log`
  - Command: `script -q -e -c 'cargo check --no-default-features --features "pg18 pg_test"' ...`
  - Timestamp: 2026-05-10 11:59 PDT
  - Result line: `Finished dev profile ... target(s) in 0.12s`.

- `cargo-test-focused-loader-blocked.log`
  - Command: `script -q -e -c 'cargo test production_scan_result_outputs_preserve_heap_resolution_origin --no-default-features --features pg18' ...`
  - Timestamp: 2026-05-10 11:59 PDT
  - Result: blocked after compile by known pgrx standalone test-binary loader
    issue, `undefined symbol: SPI_finish`; exit code 127.

- `git-diff-check-code.log`
  - Command: `script -q -e -c 'git diff --check -- <changed code/docs>' ...`
  - Timestamp: 2026-05-10 11:59 PDT
  - Result: pass, no whitespace errors.

- `ecaz-dev-install-pg18-pg-test.log`
  - Command: `target/release/ecaz dev install ecaz-pg-test --pg 18 --log-file ...`
  - Timestamp: 2026-05-10 12:00 PDT
  - Result: blocked before install by stale operator CLI repo-root resolution
    from `crates/ecaz-cli`.

- `cargo-pgrx-install-pg18-pg-test.log`
  - Command: `script -q -e -c 'cargo pgrx install --test -c /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --no-default-features --features "pg18 pg_test"' ...`
  - Timestamp: 2026-05-10 12:00 PDT
  - Result lines: `Discovered 692 SQL entities`; `Finished installing ecaz`.

- `cargo-pgrx-start-pg18.log`
  - Command: `script -q -e -c 'cargo pgrx start pg18' ...`
  - Timestamp: 2026-05-10 12:01 PDT
  - Result: pass; command exit code 0.

- `pg18-focused-db-reset.log`
  - Command: `target/release/ecaz dev sql --pg 18 --db postgres --raw --sql "DROP DATABASE IF EXISTS ecaz_30756 WITH (FORCE)" --log-output ...`
  - Timestamp: 2026-05-10 12:02 PDT
  - Result line: `DROP DATABASE`.

- `pg18-focused-db-create.log`
  - Command: `target/release/ecaz dev sql --pg 18 --db postgres --raw --sql "CREATE DATABASE ecaz_30756" --log-output ...`
  - Timestamp: 2026-05-10 12:02 PDT
  - Result line: `CREATE DATABASE`.

- `pg18-focused-create-extension.log`
  - Command: `target/release/ecaz dev sql --pg 18 --db ecaz_30756 --raw --sql "CREATE EXTENSION ecaz CASCADE" --log-output ...`
  - Timestamp: 2026-05-10 12:02 PDT
  - Result line: `CREATE EXTENSION`.

- `pg18-focused-production-scan-heap-resolution.log`
  - Command: `target/release/ecaz dev sql --pg 18 --db ecaz_30756 --raw --sql "SELECT tests.test_ec_spire_prod_scan_heap_resolution()" --log-output ...`
  - Timestamp: 2026-05-10 12:02 PDT
  - Result lines: `test_ec_spire_prod_scan_heap_resolution`; `(1 row)`.
