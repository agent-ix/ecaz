# Artifact Manifest

Original head SHA: `a919bb21d459954069ca372dc47e49eb09f0ef26`
Provider increment SHA: `e751fb89a30b5d0c825a1056f520c6b6b70fc142`
Latest code SHA: `89570bb60fb32626b24b05ab4cf4cb11a0e12099`
Packet: `review/31145-task36-38-hardening-validation`
Timestamp: `2026-05-17T03:57:31Z`

## task36-38-cargo-fmt-check.log

- Lane: formatting after Task 36/38 review fixes
- Fixture: repository worktree
- Storage format: n/a
- Rerank mode: n/a
- Command: `script -q -c "cargo fmt --all --check" review/31145-task36-38-hardening-validation/artifacts/task36-38-cargo-fmt-check.log`
- Isolated one-index-per-table: n/a
- Key result: command exited 0; rustfmt printed existing stable-toolchain warnings for unstable import-formatting options.

## task36-simd-diff.log

- Lane: Task 36 SIMD/scalar differential
- Fixture: Rust integration test `tests/simd_diff.rs`
- Storage format: `turboquant`/packed PQ fixtures
- Rerank mode: scalar reference vs dispatched and forced host-backend score paths
- Command: `script -q -c "cargo test --features bench --test simd_diff -- --test-threads=1" review/31145-task36-38-hardening-validation/artifacts/task36-simd-diff.log`
- Isolated one-index-per-table: n/a
- Key result: `8 passed; 0 failed`, including forced AVX2/FMA score and FWHT checks on this host, 2..=8 pack/unpack roundtrips, and `production_1536_4bit_score_path_matches_scalar_reference`.

## task38-fault-injection-crate.log

- Lane: Task 38 matrix/provider unit tests
- Fixture: pure Rust fault matrix and Linux LD_PRELOAD provider self-tests
- Storage format: n/a
- Rerank mode: n/a
- Command: `script -q -c "cargo test -p ecaz-fault-injection" review/31145-task36-38-hardening-validation/artifacts/task38-fault-injection-crate.log`
- Isolated one-index-per-table: n/a
- Key result: `7 passed; 0 failed`, including matched `EIO` read and matched `ENOSPC` create checks.

## task38-ecaz-cli-fault-parse-tests.log

- Lane: CLI parse tests for Task 38 fault commands
- Fixture: `ecaz-cli` clap parser
- Storage format: n/a
- Rerank mode: n/a
- Command: `script -q -c "cargo test -p ecaz-cli cli_parses_fault" review/31145-task36-38-hardening-validation/artifacts/task38-ecaz-cli-fault-parse-tests.log`
- Isolated one-index-per-table: n/a
- Key result: `4 passed; 0 failed`, including `provider-env`, `provider-restart`, `provider-restore`, and smoke dry-run parsing.

## task38-pg18-cancel-smoke.log

- Lane: Task 38 live PG18 cancel smoke
- Fixture: local pgrx PG18 database `ecaz_fault_probe_36_38`
- Storage format: AM-specific fixtures for `ec_hnsw`, `ec_ivf`, `ec_diskann`, and `ec_spire`
- Rerank mode: repeated AM KNN scan cancellation
- Command: `script -q -c "cargo run -p ecaz-cli -- --database ecaz_fault_probe_36_38 --host /home/peter/.pgrx --port 28818 dev fault smoke --lane cancel --rows 64" review/31145-task36-38-hardening-validation/artifacts/task38-pg18-cancel-smoke.log`
- Isolated one-index-per-table: yes, one fixture table and index per AM
- Key result: cancel lane completed for all four AMs with postcondition probes emitted and asserted.

## task38-pg18-timeout-smoke.log

- Lane: Task 38 live PG18 statement-timeout smoke
- Fixture: local pgrx PG18 database `ecaz_fault_probe_36_38`
- Storage format: AM-specific fixtures for `ec_hnsw`, `ec_ivf`, `ec_diskann`, and `ec_spire`
- Rerank mode: repeated AM KNN scan statement timeout
- Command: `script -q -c "cargo run -p ecaz-cli -- --database ecaz_fault_probe_36_38 --host /home/peter/.pgrx --port 28818 dev fault smoke --lane timeout --rows 64" review/31145-task36-38-hardening-validation/artifacts/task38-pg18-timeout-smoke.log`
- Isolated one-index-per-table: yes, one fixture table and index per AM
- Key result: statement-timeout lane completed for all four AMs with postcondition probes emitted and asserted.

## task38-pg18-lock-timeout-smoke.log

- Lane: Task 38 live PG18 lock-timeout smoke
- Fixture: local pgrx PG18 database `ecaz_fault_probe_36_38`
- Storage format: AM-specific fixtures for `ec_hnsw`, `ec_ivf`, `ec_diskann`, and `ec_spire`
- Rerank mode: `REINDEX INDEX CONCURRENTLY` interrupted by `lock_timeout`
- Command: `script -q -c "cargo run -p ecaz-cli -- --database ecaz_fault_probe_36_38 --host /home/peter/.pgrx --port 28818 dev fault smoke --lane lock-timeout --rows 64" review/31145-task36-38-hardening-validation/artifacts/task38-pg18-lock-timeout-smoke.log`
- Isolated one-index-per-table: yes, one fixture table and index per AM
- Key result: lock-timeout lane completed for all four AMs with postcondition probes emitted and asserted.

## task38-provider-restart.log

- Lane: Task 38 provider-backed postmaster startup
- Fixture: local pgrx PG18 postmaster
- Storage format: n/a
- Rerank mode: n/a
- Command: `script -q -c "cargo run -p ecaz-cli -- dev fault provider-restart --mode slow-disk --path-match base/ --after 1 --latency-ms 1 --marker /tmp/ecaz-fault-provider-task38-20260517.marker" review/31145-task36-38-hardening-validation/artifacts/task38-provider-restart.log`
- Isolated one-index-per-table: n/a
- Key result: postmaster restarted and printed marker `/tmp/ecaz-fault-provider-task38-20260517.marker`.

## task38-pg18-slow-disk-smoke.log

- Lane: Task 38 provider-backed slow-disk smoke
- Fixture: local pgrx PG18 database `ecaz_fault_probe_36_38`; postmaster restarted with the LD_PRELOAD provider in `slow-disk` mode and marker `/tmp/ecaz-fault-provider-task38-20260517.marker`
- Storage format: AM-specific fixtures for `ec_hnsw`, `ec_ivf`, `ec_diskann`, and `ec_spire`
- Rerank mode: AM default scan/insert/vacuum smoke under provider latency
- Command: `script -q -c "cargo run -p ecaz-cli -- --database ecaz_fault_probe_36_38 --host /home/peter/.pgrx --port 28818 dev fault smoke --lane slow-disk --rows 64 --provider-marker /tmp/ecaz-fault-provider-task38-20260517.marker" review/31145-task36-38-hardening-validation/artifacts/task38-pg18-slow-disk-smoke.log`
- Isolated one-index-per-table: yes, one fixture table and index per AM
- Key result: provider-backed slow-disk lane completed for all four AMs with postcondition probes emitted and asserted.

## task38-provider-restore.log

- Lane: Task 38 provider cleanup
- Fixture: local pgrx PG18 postmaster
- Storage format: n/a
- Rerank mode: n/a
- Command: `script -q -c "cargo run -p ecaz-cli -- dev fault provider-restore" review/31145-task36-38-hardening-validation/artifacts/task38-provider-restore.log`
- Isolated one-index-per-table: n/a
- Key result: postmaster restarted without provider environment.
