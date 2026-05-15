# Artifact Manifest: SPIRE Phase 13c AWS Readiness

Head SHA: `5a7b8308`
Packet/topic: `765-c1-spire-phase13c-aws-readiness`
Lane: Phase 13c local AWS-readiness blockers
Fixture: static PG18 build/check; no live TLS PostgreSQL fixture
Storage format: n/a
Rerank mode: n/a
Surface: shared remote libpq TLS helper; PK SELECT schema-drift guard
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
