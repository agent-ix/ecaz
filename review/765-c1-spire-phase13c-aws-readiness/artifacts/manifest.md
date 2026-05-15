# Artifact Manifest: SPIRE Phase 13c AWS Readiness

Head SHA: `e544410be6ca`
Packet/topic: `765-c1-spire-phase13c-aws-readiness`
Lane: Phase 13c local AWS-readiness blockers
Fixture: static PG18 build/check; Docker PostgreSQL 18 TLS-only remote plus
pgrx PG18 coordinator for local TLS probes
Storage format: n/a
Rerank mode: n/a
Surface: shared remote libpq TLS helper; async production transport probe; PK
SELECT schema-drift guard
Timestamp: 2026-05-15
Isolated one-index-per-table or shared-table surfaces: n/a

## Commands

1. `cargo check --no-default-features --features pg18`
   - Result: passed.
   - Key lines: `Finished dev profile`; warning only for pre-existing unused
     imports in `src/am/mod.rs`.

2. `rg -n "NoTls" src/am/ec_spire/coordinator/remote_candidates -g '!tls.rs'`
   - Result: no matches.

3. `git diff --check`
   - Result: passed.

4. `cargo test spire_remote_tls_tests --lib --no-default-features --features pg18`
   - Result: build completed, test binary execution failed before assertions.
   - Key lines: `undefined symbol: pg_re_throw`; `process didn't exit
     successfully`.

5. `cargo pgrx install --test --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --features "pg18 pg_test" --no-default-features`
   - Result: passed.
   - Key lines: `Finished installing ecaz`; discovered `883 SQL entities`.

6. `/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --configure`
   - Result: recorded in `pg18-pgrx-configure.log`.
   - Key lines: local pgrx PG18 configure flags do not include
     `--with-openssl`, so the live TLS fixture uses a Docker PostgreSQL 18
     remote and the pgrx PG18 coordinator.

7. `bash scripts/run_spire_remote_tls_docker_pg18.sh --skip-install --artifact-dir review/765-c1-spire-phase13c-aws-readiness/artifacts --run-id 20260515Tlocaltls06Z`
   - Result: passed.
   - Artifacts:
     - `remote-tls-docker-success.log`
     - `remote-postgres.log`
     - `coord-postgres.log`
   - Key lines:
     - `require_probe=connected,true,TLSv1.3`
     - `verify_full_probe=connected,true,TLSv1.3`
     - `disable_probe=connect_failed,false`
     - `bad_host_probe=connect_failed,false`
     - `require_transport=2,ready,none,3`
     - `verify_full_transport=3,ready,none,3`
     - `SPIRE remote TLS Docker PG18 probe passed`
   - TLS-negative evidence: `remote-postgres.log` records
     `pg_hba.conf rejects connection ... no encryption` for the
     `sslmode=disable` attempt.

8. `bash -n scripts/run_spire_remote_tls_docker_pg18.sh`
   - Result: passed.

9. `git diff --cached --check`
   - Result: passed for code commit `e544410be6ca`.
