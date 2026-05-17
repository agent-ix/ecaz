# 30755 Artifact Manifest

Head SHA: `cc8fc351f7bdd3bc3e5fb5ddd74d46b0b4eb3848`

Packet: `30755-spire-production-heap-resolution`

Scope: SPIRE Phase 11 Stage D production heap-resolution proof surface.

Lane: local PG18. Fixture: packet-specific PG18 SQL wrapper with one
coordinator fixture table/index and one loopback remote fixture table/index.
Storage format: `rabitq`. Rerank mode: exact heap-vector rerank after
origin-node heap visibility resolution. Surface shape: isolated
one-index-per-table fixtures with one coordinator leaf placement rewritten to a
remote node descriptor.

## Artifacts

- `cargo-fmt-check.log`
  - Command: `script -q -e -c 'cargo fmt --check' ...`
  - Timestamp: 2026-05-10 11:42 PDT
  - Result: pass; only existing stable-rustfmt warnings for
    `imports_granularity` / `group_imports`.

- `cargo-check-pg18.log`
  - Command: `script -q -e -c 'cargo check --no-default-features --features pg18' ...`
  - Timestamp: 2026-05-10 11:42 PDT
  - Result line: `Finished dev profile ... target(s) in 0.27s`.

- `cargo-check-pg18-pg-test.log`
  - Command: `script -q -e -c 'cargo check --no-default-features --features "pg18 pg_test"' ...`
  - Timestamp: 2026-05-10 11:42 PDT
  - Result line: `Finished dev profile ... target(s) in 0.12s`.

- `git-diff-check-code.log`
  - Command: `script -q -e -c 'git diff --check -- <changed code/docs>' ...`
  - Timestamp: 2026-05-10 11:43 PDT
  - Result: pass, no whitespace errors.

- `cargo-pgrx-install-pg18-pg-test.log`
  - Command: `script -q -e -c 'cargo pgrx install --test -c /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --no-default-features --features "pg18 pg_test"' ...`
  - Timestamp: 2026-05-10 11:43 PDT
  - Result lines: `Discovered 692 SQL entities`; `Finished installing ecaz`.

- `cargo-pgrx-start-pg18.log`
  - Command: `script -q -e -c 'cargo pgrx start pg18' ...`
  - Timestamp: 2026-05-10 11:44 PDT
  - Result: pass; command exit code 0.

- `pg18-focused-db-reset.log`
  - Command: `psql -h /home/peter/.pgrx -p 28818 -d postgres ... DROP DATABASE IF EXISTS ecaz_30755 WITH (FORCE); CREATE DATABASE ecaz_30755`
  - Timestamp: 2026-05-10 11:44 PDT
  - Result lines: `DROP DATABASE`; `CREATE DATABASE`.

- `pg18-focused-production-scan-heap-resolution.log`
  - Command: `psql -h /home/peter/.pgrx -p 28818 -d ecaz_30755 -v ON_ERROR_STOP=1 ... CREATE EXTENSION ecaz CASCADE; SELECT tests.test_ec_spire_prod_scan_heap_resolution()`
  - Timestamp: 2026-05-10 11:44 PDT
  - Result lines: `CREATE EXTENSION`; `test_ec_spire_prod_scan_heap_resolution`; `(1 row)`.
