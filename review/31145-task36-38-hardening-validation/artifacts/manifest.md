# Artifact Manifest

Code checkpoint SHA: `12b2d0091a9146fd8af70976a2b2e67c190f7866`
Packet: `review/31145-task36-38-hardening-validation`
Timestamp: `2026-05-17T05:48:26Z`

All live PG18 artifacts use database `ecaz_fault_probe_36_38`, socket
directory `/home/peter/.pgrx`, port `28818`, and isolated one-index-per-table
fixtures for `ec_hnsw`, `ec_ivf`, `ec_diskann`, and `ec_spire` unless noted.

## task36-38-final-cargo-fmt-check.log

- Lane: final formatting check
- Command: `script -q -e -c "cargo fmt --all --check" review/31145-task36-38-hardening-validation/artifacts/task36-38-final-cargo-fmt-check.log`
- Key result: exited 0; rustfmt printed existing stable-toolchain warnings for unstable import-formatting options.

## task36-final-simd-diff.log

- Lane: Task 36 SIMD/scalar differential
- Fixture: `tests/simd_diff.rs`
- Command: `script -q -e -c "cargo test --features bench --test simd_diff -- --test-threads=1" review/31145-task36-38-hardening-validation/artifacts/task36-final-simd-diff.log`
- Key result: 9 passed, 0 failed. Coverage includes product-quantizer scoring, forced AVX2/FMA score/FWHT on this host, pack/unpack roundtrips, HNSW/DiskANN source inner-product SIMD, and the 1536/4 production score path.

## task36-ci-matrix-simd-diff-local.log

- Lane: Task 36 SIMD/scalar differential after adding the CI matrix
- Fixture: `.github/workflows/ci.yml` now runs `cargo test --features bench --test simd_diff -- --test-threads=1` on `ubuntu-24.04` x64 and `ubuntu-24.04-arm` arm64 runners.
- Command: `script -q -e -c "cargo test --features bench --test simd_diff -- --test-threads=1" review/31145-task36-38-hardening-validation/artifacts/task36-ci-matrix-simd-diff-local.log`
- Key result: local x64 run passed 9/9; the PR CI matrix is the remote verifier for the arm64/NEON hosted runner.

## task36-miri-scalar-reference.log

- Lane: Task 36 Miri scalar-reference coverage
- Command: `script -q -e -c "cargo +nightly miri test --lib -- miri_" review/31145-task36-38-hardening-validation/artifacts/task36-miri-scalar-reference.log`
- Key result: 19 passed, 0 failed.

## task36-simd-diff-mutation-control.log

- Lane: Task 36 mutation control
- Command: temporarily perturbed the 1536/4 production score assertion by `+0.01`, then ran `script -q -c "cargo test --features bench --test simd_diff production_1536_4bit_score_path_matches_scalar_reference -- --exact --nocapture" review/31145-task36-38-hardening-validation/artifacts/task36-simd-diff-mutation-control.log`
- Key result: failed as expected with absolute diff `0.010000000` above tolerance `0.000010000`; the source was restored before the final passing SIMD run.

## task38-final-fault-injection-crate.log

- Lane: Task 38 matrix/provider unit tests
- Command: `script -q -e -c "cargo test -p ecaz-fault-injection" review/31145-task36-38-hardening-validation/artifacts/task38-final-fault-injection-crate.log`
- Key result: 7 passed, 0 failed, including matched `EIO` read and matched `ENOSPC` create provider self-tests.

## task38-final-ecaz-cli-fault-parse-tests.log

- Lane: Task 38 CLI parser coverage
- Command: `script -q -e -c "cargo test -p ecaz-cli cli_parses_fault" review/31145-task36-38-hardening-validation/artifacts/task38-final-ecaz-cli-fault-parse-tests.log`
- Key result: 6 passed, 0 failed, including `prepare` and prepared-fixture I/O smoke parsing.

## task38-pg18-install-memory-fault.log

- Lane: PG18 extension install for live memory-fault GUC validation
- Command: `script -q -e -c "cargo pgrx install --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --no-default-features --features pg18" review/31145-task36-38-hardening-validation/artifacts/task38-pg18-install-memory-fault.log`
- Key result: installed `ecaz` into the PG18 pgrx tree.

## task38-provider-restore-after-install.log

- Lane: provider cleanup / normal postmaster restart after install
- Command: `script -q -e -c "cargo run -p ecaz-cli -- dev fault provider-restore" review/31145-task36-38-hardening-validation/artifacts/task38-provider-restore-after-install.log`
- Key result: restarted PG18 without provider environment.

## task38-reset-test-extension-local.log

- Lane: refresh test database extension SQL after install
- Command: `script -q -e -c "cargo run -p ecaz-cli -- dev sql --db ecaz_fault_probe_36_38 --socket-dir /home/peter/.pgrx --sql 'DROP EXTENSION IF EXISTS ecaz CASCADE; CREATE EXTENSION ecaz;'" review/31145-task36-38-hardening-validation/artifacts/task38-reset-test-extension-local.log`
- Key result: recreated the extension in `ecaz_fault_probe_36_38`.

## task38-pg18-memory-smoke.log

- Lane: Task 38 live memory/palloc smoke
- Command: `script -q -e -c "cargo run -p ecaz-cli -- --database ecaz_fault_probe_36_38 --host /home/peter/.pgrx --port 28818 dev fault smoke --lane memory --rows 64" review/31145-task36-38-hardening-validation/artifacts/task38-pg18-memory-smoke.log`
- Key result: all four AMs completed palloc-failure smoke with postcondition probes asserted.

## task38-pg18-install-palloc-sweep-sites.log

- Lane: PG18 extension install for final `ecaz.fault_palloc_nth` scan-site sweep
- Command: `script -q -e -c "cargo pgrx install --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --no-default-features --features pg18" review/31145-task36-38-hardening-validation/artifacts/task38-pg18-install-palloc-sweep-sites.log`
- Key result: installed the extension after adding scan query/order-by palloc instrumentation and renaming the live fault GUC to `ecaz.fault_palloc_nth`.

## task38-pg18-memory-palloc-sweep-sites.log

- Lane: Task 38 live memory/palloc scan-site sweep
- Command: `script -q -e -c "cargo run -p ecaz-cli -- --database ecaz_fault_probe_36_38 --host /home/peter/.pgrx --port 28818 dev fault smoke --lane memory --rows 64" review/31145-task36-38-hardening-validation/artifacts/task38-pg18-memory-palloc-sweep-sites.log`
- Key result: all four AMs completed the live palloc smoke. HNSW sweeps Nth failures 1..=4, IVF 1..=4, DiskANN 1, and SPIRE 1..=3, with postcondition probes asserted after each run.

## task38-pg18-install-memory-major-workloads.log

- Lane: PG18 extension install for build/scan/insert/vacuum memory smoke
- Command: `script -q -e -c "cargo pgrx install --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --no-default-features --features pg18" review/31145-task36-38-hardening-validation/artifacts/task38-pg18-install-memory-major-workloads.log`
- Key result: installed the extension after adding memory-fault checkpoints to each AM's build result, insert entry, and vacuum stats boundaries.

## task38-pg18-memory-major-workloads.log

- Lane: Task 38 live memory/palloc major-workload smoke
- Command: `script -q -e -c "cargo run -p ecaz-cli -- --database ecaz_fault_probe_36_38 --host /home/peter/.pgrx --port 28818 dev fault smoke --lane memory --rows 64" review/31145-task36-38-hardening-validation/artifacts/task38-pg18-memory-major-workloads.log`
- Key result: all four AMs completed memory smoke across build, scan, insert, and vacuum probes, with shared postcondition probes asserted.

## task38-pg18-lock-rollback-guard.log

- Lane: Task 38 live lock-timeout cleanup guard
- Command: `script -q -e -c "cargo run -p ecaz-cli -- --database ecaz_fault_probe_36_38 --host /home/peter/.pgrx --port 28818 dev fault smoke --lane lock-timeout --rows 64" review/31145-task36-38-hardening-validation/artifacts/task38-pg18-lock-rollback-guard.log`
- Key result: all four AMs completed lock-timeout smoke after changing the harness to attempt holder rollback before propagating waiter reset errors.

## task38-pg18-lock-ddl-matrix.log

- Lane: Task 38 live lock-timeout DDL matrix
- Command: `script -q -e -c "cargo run -p ecaz-cli -- --database ecaz_fault_probe_36_38 --host /home/peter/.pgrx --port 28818 dev fault smoke --lane lock-timeout --rows 64" review/31145-task36-38-hardening-validation/artifacts/task38-pg18-lock-ddl-matrix.log`
- Key result: all four AMs completed lock-timeout smoke across blocked `REINDEX INDEX CONCURRENTLY`, `CREATE INDEX`, and `VACUUM (FULL)` cases, with shared postcondition probes asserted.

## task38-pg18-cancel-terminate-matrix.log

- Lane: Task 38 live cancellation/termination matrix
- Command: `script -q -e -c "cargo run -p ecaz-cli -- --database ecaz_fault_probe_36_38 --host /home/peter/.pgrx --port 28818 dev fault smoke --lane cancel --rows 64" review/31145-task36-38-hardening-validation/artifacts/task38-pg18-cancel-terminate-matrix.log`
- Key result: all four AMs completed both `pg_cancel_backend` and `pg_terminate_backend` smoke cases, with shared postcondition probes asserted.

## task38-pg18-postcondition-pgstat-probes.log

- Lane: Task 38 live postcondition probe expansion
- Command: `script -q -e -c "cargo run -p ecaz-cli -- --database ecaz_fault_probe_36_38 --host /home/peter/.pgrx --port 28818 dev fault smoke --lane lock-timeout --rows 64" review/31145-task36-38-hardening-validation/artifacts/task38-pg18-postcondition-pgstat-probes.log`
- Key result: live lock-timeout smoke passed with optional probes enabled; `pg_buffercache_fixture_pins=0` and `pg_stat_io_ops_before=731 after=762`.

## task38-pg18-timeout-idle-tx.log

- Lane: Task 38 live timeout matrix expansion
- Command: `script -q -e -c "cargo run -p ecaz-cli -- --database ecaz_fault_probe_36_38 --host /home/peter/.pgrx --port 28818 dev fault smoke --lane timeout --rows 64" review/31145-task36-38-hardening-validation/artifacts/task38-pg18-timeout-idle-tx.log`
- Key result: all four AMs completed timeout smoke with both `statement-timeout` and `idle-in-transaction-timeout` cases listed in the matrix. The expected idle-timeout backend terminations were followed by shared postcondition probes; `pg_buffercache_fixture_pins=0` and `pg_stat_io_ops_before=764 after=795`.

## task38-pg18-resource-temp-spill.log

- Lane: Task 38 live resource/temp-spill expansion
- Command: `script -q -e -c "cargo run -p ecaz-cli -- --database ecaz_fault_probe_36_38 --host /home/peter/.pgrx --port 28818 dev fault smoke --lane resource --rows 64" review/31145-task36-38-hardening-validation/artifacts/task38-pg18-resource-temp-spill.log`
- Key result: all four AMs completed resource smoke with both `tiny-work-mem` and `temp-file-limit` cases listed in the matrix. The temp-spill subcase forced a `temp_file_limit = '64kB'` ERROR and verified backend usability before shared postcondition probes; `pg_buffercache_fixture_pins=0` and `pg_stat_io_ops_before=799 after=843`.

## task38-pg18-resource-provider-temp-spill.log

- Lane: Task 38 provider-backed temp-spill ENOSPC
- Command: `script -q -e -c "cargo run -p ecaz-cli -- --database ecaz_fault_probe_36_38 --host /home/peter/.pgrx --port 28818 dev fault smoke --lane resource --rows 64 --provider-marker review/31145-task36-38-hardening-validation/artifacts/task38-provider-temp-spill.marker" review/31145-task36-38-hardening-validation/artifacts/task38-pg18-resource-provider-temp-spill.log`
- Provider setup: `task38-provider-temp-spill-restart.log` restarted PG18 with `--mode enospc-write --path-match pgsql_tmp --after 1`; `task38-provider-temp-spill.marker` recorded `mode=enospc-write match=pgsql_tmp`; `task38-provider-temp-spill-restore.log` restored PG18 without provider environment.
- Key result: all four AMs completed resource smoke under the provider, the resource lane printed `resource_temp_spill_provider=enospc-write match=pgsql_tmp`, and shared postconditions passed with `pg_buffercache_fixture_pins=0` and `pg_stat_io_ops_before=1392 after=1712`.

## task38-spire-remote-oom.log

- Lane: Task 38 SPIRE Stage E remote transport fault
- Command: `script -q -e -c "cargo run -p ecaz-cli -- dev spire-multicluster fault-pg18 --case remote_oom --artifact-dir review/31145-task36-38-hardening-validation/artifacts/task38-spire-remote-oom --run-id task38-spire-remote-oom-20260517T0538Z --coord-port 39425 --remote-ready-port 39424 --skip-install" review/31145-task36-38-hardening-validation/artifacts/task38-spire-remote-oom.log`
- Key result: `SPIRE Stage E remote_oom PG18 fixture passed`. Strict mode observed `remote_transport_failed` with first failure category `remote_query_failed`; degraded mode observed `requires_compact_candidate_receive` with first degraded skip category `remote_query_failed`. The run also verified the Stage E fixture socket-path fix: the remote log shows the socket under `/home/peter/dev/ecaz/target/s-1656884002/.s.PGSQL.39424`.
- Related logs: `task38-spire-remote-oom/stage_e_fault_remote_oom.log`, `stage_e_fault_remote_oom_strict.log`, `stage_e_fault_remote_oom_degraded.log`, `remote-ready-postgres.log`, and `coord-postgres.log`.

## Provider-Backed I/O Smoke

- HNSW EIO: `task38-pg18-hnsw-eio-smoke.log`, path match `base/8052051/8054466`, provider marker `/tmp/ecaz-fault-provider-eio-hnsw-task38-final.marker`.
- HNSW ENOSPC: `task38-pg18-hnsw-enospc-smoke.log`, path match `base/8052051/8054456`, provider marker `/tmp/ecaz-fault-provider-enospc-hnsw-task38-final.marker`.
- IVF EIO: `task38-pg18-ivf-eio-smoke.log`, path match `base/8052051/8054478`, provider marker `/tmp/ecaz-fault-provider-eio-ivf-task38-final.marker`.
- IVF ENOSPC: `task38-pg18-ivf-enospc-smoke.log`, path match `base/8052051/8054468`, provider marker `/tmp/ecaz-fault-provider-enospc-ivf-task38-final.marker`.
- DiskANN EIO: `task38-pg18-diskann-eio-smoke.log`, path match `base/8052051/8054490`, provider marker `/tmp/ecaz-fault-provider-eio-diskann-task38-final.marker`.
- DiskANN ENOSPC: `task38-pg18-diskann-enospc-smoke.log`, path match `base/8052051/8054480`, provider marker `/tmp/ecaz-fault-provider-enospc-diskann-task38-final.marker`.
- SPIRE EIO: `task38-pg18-spire-eio-smoke.log`, path match `base/8052051/8054502`, provider marker `/tmp/ecaz-fault-provider-eio-spire-task38-20260517.marker`.
- SPIRE ENOSPC: `task38-pg18-spire-enospc-smoke.log`, path match `base/8052051/8054492`, provider marker `/tmp/ecaz-fault-provider-enospc-spire-task38-20260517.marker`.
- Key result: all eight runs exited 0. Each run restarted PG18 with the provider, ran `ecaz dev fault smoke --lane io --am <am> --assume-prepared`, asserted the shared postcondition probes, and restored the postmaster.

## Prior Live Smoke Artifacts Retained

- `task38-pg18-cancel-smoke.log`: cancel smoke for all four AMs passed.
- `task38-pg18-timeout-smoke.log`: statement-timeout smoke for all four AMs passed.
- `task38-pg18-lock-timeout-smoke.log`: lock-timeout smoke for all four AMs passed.
- `task38-pg18-slow-disk-smoke.log`: provider-backed slow-disk smoke for all four AMs passed.
- `task38-provider-restart.log` and `task38-provider-restore.log`: slow-disk provider startup/cleanup.

## task38-final-pg18-status.log

- Lane: final postmaster state check
- Command: `script -q -e -c "/home/peter/.pgrx/18.3/pgrx-install/bin/pg_ctl -D /home/peter/.pgrx/data-18 status" review/31145-task36-38-hardening-validation/artifacts/task38-final-pg18-status.log`
- Key result: PG18 postmaster is running without provider environment in the command line.
