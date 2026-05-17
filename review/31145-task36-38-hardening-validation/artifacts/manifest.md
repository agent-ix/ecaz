# Artifact Manifest

Head SHA: `a919bb21d459954069ca372dc47e49eb09f0ef26`
Packet: `review/31145-task36-38-hardening-validation`
Timestamp: `2026-05-17T03:22:13Z`

## cargo-fmt-check.log

- Lane: formatting
- Fixture: repository worktree
- Storage format: n/a
- Rerank mode: n/a
- Command: `cargo fmt --all --check`
- Isolated one-index-per-table: n/a
- Key result: command exited 0; rustfmt printed existing stable-toolchain warnings for unstable import-formatting options.

## fault-injection-crate.log

- Lane: Task 38 matrix unit tests
- Fixture: pure Rust fault matrix and workload SQL tests
- Storage format: n/a
- Rerank mode: n/a
- Command: `cargo test -p ecaz-fault-injection`
- Isolated one-index-per-table: n/a
- Key result: `4 passed; 0 failed`.

## ecaz-cli-tests.log

- Lane: CLI unit tests
- Fixture: ecaz-cli parser and unit tests
- Storage format: n/a
- Rerank mode: n/a
- Command: `cargo test -p ecaz-cli`
- Isolated one-index-per-table: n/a
- Key result: `333 passed; 0 failed`.

## simd-diff.log

- Lane: Task 36 SIMD/scalar differential
- Fixture: Rust integration test `tests/simd_diff.rs`
- Storage format: `turboquant`/packed PQ fixtures
- Rerank mode: scalar reference vs dispatched score paths
- Command: `cargo test --features bench --test simd_diff -- --test-threads=1`
- Isolated one-index-per-table: n/a
- Key result: `5 passed; 0 failed`, including `production_1536_4bit_score_path_matches_scalar_reference`.

## pg18-live-fault-smoke.log

- Lane: Task 38 live PG18 provider-free fault smoke
- Fixture: local pgrx PG18 database `ecaz_fault_probe_36_38`
- Storage format: AM-specific fixtures for `ec_hnsw`, `ec_ivf`, `ec_diskann`, and `ec_spire`
- Rerank mode: AM default smoke workloads
- Command: `cargo run -p ecaz-cli -- --database ecaz_fault_probe_36_38 --host /home/peter/.pgrx --port 28818 dev fault smoke --lane {cancel,timeout,lock-timeout,resource} --rows 16`
- Isolated one-index-per-table: yes, one fixture table and index per AM
- Key result: cancel, statement-timeout, lock-timeout, and resource lanes completed for all four AMs with postcondition probes emitted and asserted.
