# Artifact Manifest: SPIRE Phase 13c AWS Readiness

Head SHA: `cdf4dbd5f565`
Packet/topic: `765-c1-spire-phase13c-aws-readiness`
Lane: Phase 13c local AWS-readiness blockers
Fixture: static PG18 build/check; Docker PostgreSQL 18 TLS-only remote plus
pgrx PG18 coordinator for local TLS probes; local PG18 PK SELECT drift smoke;
two-cluster PG18 CustomScan read smoke
Storage format: n/a
Rerank mode: n/a
Surface: shared remote libpq TLS helper; async production transport probe; PK
SELECT schema-drift guard; CustomScan read path
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
   - Key lines: `Finished installing ecaz`; discovered `884 SQL entities`
     after the PK SELECT drift pg_test was added.

6. `/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --configure`
   - Result: recorded in `pg18-pgrx-configure.log`.
   - Key lines: local pgrx PG18 configure flags do not include
     `--with-openssl`, so the live TLS fixture uses a Docker PostgreSQL 18
     remote and the pgrx PG18 coordinator.

7. `bash scripts/run_spire_remote_tls_docker_pg18.sh --skip-install --artifact-dir review/765-c1-spire-phase13c-aws-readiness/artifacts --run-id 20260515Tlocaltls08Z`
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
   - Result: passed for code commit `cdf4dbd5f565`.

10. `cargo fmt --check`
    - Result: passed.
    - Key lines: only stable-rustfmt warnings for the repository's unstable
      `imports_granularity` and `group_imports` settings.

11. `bash -n scripts/run_spire_phase13c_drift_pg18.sh`
    - Result: passed.

12. `cargo pgrx test pg18 test_ec_spire_select_schema_drift_variants_sql`
    - Result: local test harness failed before assertions.
    - Artifact: `cargo-pgrx-test-select-schema-drift.log`.
    - Key lines: `undefined symbol: BufferBlocks`; `test exited abnormally`.

13. `bash scripts/run_spire_phase13c_drift_pg18.sh --skip-install --artifact-dir review/765-c1-spire-phase13c-aws-readiness/artifacts --run-id 20260515Tpkselect02Z`
    - Result: passed.
    - Artifacts:
      - `phase13c-drift-success.log`
      - `pk-select-schema-drift-coord_only.log`
      - `pk-select-schema-drift-remote_only.log`
      - `pk-select-schema-drift-both_sides.log`
      - `postgres.log`
    - Key lines:
      - `pk_select_schema_drift_variant=coord_only,schema_drift,coordinator side drifted`
      - `pk_select_schema_drift_variant=remote_only,schema_drift,remote side drifted`
      - `pk_select_schema_drift_variant=both_sides,schema_drift,coordinator and remote schema fingerprints differ`
      - `SPIRE Phase 13c PG18 PK SELECT drift smoke passed`

14. `bash scripts/run_spire_multicluster_customscan_read_pg18.sh --skip-install --artifact-dir review/765-c1-spire-phase13c-aws-readiness/artifacts/customscan-read --run-id phase13ccscan02`
    - Result: passed.
    - Artifacts:
      - `customscan-read/multicluster-customscan-read.log`
      - `customscan-read/remote-postgres.log`
      - `customscan-read/coord-postgres.log`
    - Key lines:
      - `Custom Scan (EcSpireDistributedScan)`
      - `read_row=10|remote alpha|{red,blue}|domain alpha|(7,left)`
      - `typed_payload_probe=ready,pg_binary_attr_v1,t,t`
      - `SPIRE multicluster CustomScan read passed`
