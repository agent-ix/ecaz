# 30754 Artifact Manifest

Head SHA: `38beeb6fbc33dec4e84295a82451d67960040f1b`

Packet: `30754-spire-production-scan-handoff`

Scope: SPIRE Phase 11 Stage C / C5 production AM-scan candidate handoff.

Lane: local PG18. Fixture: packet-specific PG18 SQL wrapper with one
coordinator fixture table/index and one loopback remote fixture table/index.
Storage format: `rabitq`. Rerank mode: none. Surface shape: isolated
one-index-per-table fixtures with one coordinator leaf placement rewritten to a
remote node descriptor.

## Artifacts

- `cargo-fmt-check.log`
  - Command: `script -q -e -c 'cargo fmt --check' ...`
  - Timestamp: 2026-05-10 11:21 PDT
  - Result: pass; only existing stable-rustfmt warnings for
    `imports_granularity` / `group_imports`.

- `cargo-check-pg18.log`
  - Command: `script -q -e -c 'cargo check --no-default-features --features pg18' ...`
  - Timestamp: 2026-05-10 11:21 PDT
  - Result line: `Finished dev profile ... target(s) in 0.17s`

- `git-diff-check.log`
  - Command: `script -q -e -c 'git diff --check -- <changed code/docs>' ...`
  - Timestamp: 2026-05-10 11:21 PDT
  - Result: pass, no whitespace errors.

- `cargo-pgrx-install-pg18-pg-test.log`
  - Command: `script -q -e -c 'cargo pgrx install --test -c /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --no-default-features --features "pg18 pg_test"' ...`
  - Timestamp: 2026-05-10 11:21 PDT
  - Result lines: `Discovered 690 SQL entities`; `Finished installing ecaz`.

- `pg18-focused-db-reset.log`
  - Command: `psql -h /home/peter/.pgrx -p 28818 -d postgres ... DROP DATABASE IF EXISTS ecaz_30754 WITH (FORCE); CREATE DATABASE ecaz_30754`
  - Timestamp: 2026-05-10 11:21 PDT
  - Result lines: `DROP DATABASE`; `CREATE DATABASE`.

- `pg18-focused-production-scan-handoff.log`
  - Command: `psql -h /home/peter/.pgrx -p 28818 -d ecaz_30754 -v ON_ERROR_STOP=1 ... CREATE EXTENSION ecaz CASCADE; SELECT tests.test_ec_spire_prod_scan_handoff_receive()`
  - Timestamp: 2026-05-10 11:22 PDT
  - Result lines: `CREATE EXTENSION`; `test_ec_spire_prod_scan_handoff_receive`; `(1 row)`.
