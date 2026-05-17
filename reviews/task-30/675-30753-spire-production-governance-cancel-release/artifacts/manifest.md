# 30753 Artifact Manifest

Head SHA: `8f3c10cc8fb9f9b13cb1cc366c4eb840987d1ed0`

Packet: `30753-spire-production-governance-cancel-release`

Scope: SPIRE Phase 11 Stage C production async transport / compact-candidate
receive governance and local-cancel cleanup.

Storage format: `rabitq` where candidate-receive fixtures build a remote index.
Rerank mode: none. Surface shape: one packet-specific PG18 database
`ecaz_30753`; test wrappers use isolated fixture tables/indexes.

## Artifacts

- `cargo-fmt-check.log`
  - Command: `script -q -e -c 'cargo fmt --check' ...`
  - Timestamp: 2026-05-10 10:36 PDT
  - Result: pass; only existing stable-rustfmt warnings for
    `imports_granularity` / `group_imports`.

- `cargo-check-pg18.log`
  - Command: `script -q -e -c 'cargo check --no-default-features --features pg18' ...`
  - Timestamp: 2026-05-10 10:55 PDT
  - Result line: `Finished dev profile ... target(s) in 12.66s`

- `git-diff-check-code.log`
  - Command: `script -q -e -c 'git diff --check -- ...' ...`
  - Timestamp: 2026-05-10 10:57 PDT
  - Result: pass, no whitespace errors.

- `cargo-pgrx-test-pg18-production-fault-matrix-contract-escalated.log`
  - Command: `script -q -e -c 'cargo pgrx test pg18 production_fault_matrix_contract' ...`
  - Timestamp: 2026-05-10 10:50 PDT
  - Result: blocked before test execution by the standalone Rust test binary
    loader: `undefined symbol: SPI_finish`.
  - Follow-up: validation moved to the pgrx-installed PG18 SQL wrappers below.

- `cargo-pgrx-install-pg18-pg-test.log`
  - Command: `script -q -e -c 'cargo pgrx install --test -c /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --no-default-features --features "pg18 pg_test"' ...`
  - Timestamp: 2026-05-10 10:53 PDT
  - Result lines: `Finished installing ecaz`; `Discovered 688 SQL entities`.

- `cargo-pgrx-start-pg18.log`
  - Command: `script -q -e -c 'cargo pgrx start pg18' ...`
  - Timestamp: 2026-05-10 10:54 PDT
  - Result line: `Starting Postgres v18 on port 28818`

- `pg18-focused-db-drop.log`
  - Command: `psql -h /home/peter/.pgrx -p 28818 -d postgres ... DROP DATABASE IF EXISTS ecaz_30753 WITH (FORCE)`
  - Timestamp: 2026-05-10 10:55 PDT
  - Result line: `DROP DATABASE`

- `pg18-focused-db-create.log`
  - Command: `psql -h /home/peter/.pgrx -p 28818 -d postgres ... CREATE DATABASE ecaz_30753`
  - Timestamp: 2026-05-10 10:55 PDT
  - Result line: `CREATE DATABASE`

- `pg18-focused-tests.sql`
  - SQL wrapper file run by `pg18-focused-tests.log`.
  - Calls the five focused PG18 test wrappers in the `tests` schema.

- `pg18-focused-tests.log`
  - Command: `psql -h /home/peter/.pgrx -p 28818 -d ecaz_30753 -v ON_ERROR_STOP=1 -f pg18-focused-tests.sql`
  - Timestamp: 2026-05-10 10:55 PDT
  - Result lines:
    - `test_ec_spire_production_fault_matrix_contract` returned one row.
    - `test_ec_spire_prod_transport_governance_overload` returned one row.
    - `test_ec_spire_prod_receive_governance_overload` returned one row.
    - `test_ec_spire_prod_transport_local_cancel_remote_cancel` returned one row.
    - `test_ec_spire_prod_receive_local_cancel_remote_cancel` returned one row.

- `cargo-pgrx-stop-pg18.log`
  - Command: `script -q -e -c 'cargo pgrx stop pg18' ...`
  - Timestamp: 2026-05-10 10:55 PDT
  - Result line: `Stopping Postgres v18`
