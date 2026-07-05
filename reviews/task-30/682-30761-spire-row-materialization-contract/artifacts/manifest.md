# 30761 Artifact Manifest

Head SHA: `bdc6349797a04090a0486fa664529fb2a9e4f23d`

Packet: `30761-spire-row-materialization-contract`

Scope: SPIRE Phase 11 Stage D remote-origin row materialization contract for
PostgreSQL index AM delivery.

Lane: local PG18 compile/static validation. Fixture: contract and SQL
diagnostic surface only. Storage format: not applicable. Rerank mode: not
applicable. Surface shape: no isolated or shared SQL/index fixture was started
for this packet.

## Artifacts

- `cargo-fmt-check.log`
  - Command: `script -q -e -c 'cargo fmt --check' ...`
  - Timestamp: 2026-05-10 13:01 PDT
  - Result: pass; only existing stable-rustfmt warnings for
    `imports_granularity` / `group_imports`.

- `git-diff-check-code.log`
  - Command: `script -q -e -c 'git diff --check -- <changed code/docs>' ...`
  - Timestamp: 2026-05-10 13:01 PDT
  - Result: pass, no whitespace errors.

- `cargo-check-pg18.log`
  - Command: `script -q -e -c 'cargo check --no-default-features --features pg18' ...`
  - Timestamp: 2026-05-10 13:01 PDT
  - Result line: `Finished dev profile ... target(s) in 9.39s`.

- `cargo-check-pg18-pg-test.log`
  - Command: `script -q -e -c 'cargo check --no-default-features --features "pg18 pg_test"' ...`
  - Timestamp: 2026-05-10 13:01 PDT
  - Result line: `Finished dev profile ... target(s) in 17.85s`.

- `cargo-test-row-materialization-contract.log`
  - Command: `script -q -e -c 'cargo test row_materialization_contract --no-default-features --features pg18' ...`
  - Timestamp: 2026-05-10 13:02 PDT
  - Result: compiled, then failed before running the test body with the known
    direct-test pgrx loader issue, `undefined symbol: SPI_finish`.
