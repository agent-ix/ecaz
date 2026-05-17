# Artifact Manifest

Code checkpoint SHA: `63be6e2d6f4a3a1de2e611193ecdeac454ab8324`
Packet: `review/31145-task36-38-hardening-validation`
Timestamp: `2026-05-17T04:55:03Z`

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

## task38-pg18-lock-rollback-guard.log

- Lane: Task 38 live lock-timeout cleanup guard
- Command: `script -q -e -c "cargo run -p ecaz-cli -- --database ecaz_fault_probe_36_38 --host /home/peter/.pgrx --port 28818 dev fault smoke --lane lock-timeout --rows 64" review/31145-task36-38-hardening-validation/artifacts/task38-pg18-lock-rollback-guard.log`
- Key result: all four AMs completed lock-timeout smoke after changing the harness to attempt holder rollback before propagating waiter reset errors.

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
