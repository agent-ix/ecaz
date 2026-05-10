# Artifact Manifest

Packet: `30708-spire-libpq-receive-attempt-diagnostics`
Head SHA: `29b130f3f751e567dd9156d45fca76ba0b685fe0`
Timestamp: `2026-05-10T04:42:31Z`

## cargo-pgrx-pg18-libpq-receive-attempts.log

- Command: `script -q -c "cargo pgrx test pg18 test_ec_spire_libpq" /home/peter/dev/ecaz/review/30708-spire-libpq-receive-attempt-diagnostics/artifacts/cargo-pgrx-pg18-libpq-receive-attempts.log`
- Lane: Phase 11 Stage B/E receive-attempt diagnostics.
- Fixture: PG18 loopback coordinator plus one loopback remote PostgreSQL SPIRE index.
- Storage format: default non-RaBitQ remote endpoint for mismatch reporting.
- Rerank mode: `ec_spire_remote_search` candidate scoring via current SQL endpoint.
- Surface shape: isolated one-index-per-table coordinator and remote loopback surfaces.
- Key result line: `test tests::pg_test_ec_spire_libpq_receive_attempts_degraded_skip ... ok`
- Key result line: `test tests::pg_test_ec_spire_libpq_executor_rejects_non_ready_endpoint - should panic ... ok`
- Key result line: `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1516 filtered out`

## cargo-pgrx-pg18-operator-policy-contract.log

- Command: `script -q -c "cargo pgrx test pg18 test_ec_spire_remote_phase7_policy_contracts" /home/peter/dev/ecaz/review/30708-spire-libpq-receive-attempt-diagnostics/artifacts/cargo-pgrx-pg18-operator-policy-contract.log`
- Lane: operator entrypoint contract reachability.
- Fixture: PG18 SQL-visible policy contract surface.
- Storage format: contract-only; no index storage read path.
- Rerank mode: contract-only; no rerank execution.
- Surface shape: operator entrypoint contract surface.
- Key result line: `test tests::pg_test_ec_spire_remote_phase7_policy_contracts ... ok`
- Key result line: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1517 filtered out`
