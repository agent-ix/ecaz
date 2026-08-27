# Task 234 packet 004 artifact manifest

Date: 2026-08-25 (America/Los_Angeles)

## Scope and head

- Task bucket: `reviews/task-234/`
- Packet: `004-current-tls-reintegration/`
- Current-TLS campaign base: `ed5ac814c05350ca695533fcd54d0df11faa876b`
- Validated candidate extension: `7c42dc818e80fe68246dc5c45255640b81c551b1`
- Fault-matrix harness fix included in the candidate: `eada215ae6836948c281fff263af199992c79dd3`
- PostgreSQL: PG18 release extension with `pg_test`, SSL toolchain at
  `/home/peter/.ecaz/toolchains/pg18-ssl/bin`
- Fixture: four-node fresh physical fixture, 2,000 rows, dimension 16, graph
  degree 32, verify-full mutual TLS. The matrix uses isolated owner indexes;
  it is not a shared-table surface.

## Commands and results

```text
cargo check --no-default-features --features pg18
cargo check --no-default-features --features pg18,pg_test
cargo check -p ecaz-cli
cargo test --lib --no-default-features --features pg18 remote_transport::tests

/home/peter/.cargo-target/debug/ecaz dev distann-multicluster local-multinode-pg18 \
  --pgbin /home/peter/.ecaz/toolchains/pg18-ssl/bin \
  --run-dir /home/peter/.ecaz/clusters/task234-current-tls-fault-matrix \
  --artifact-dir reviews/task-234/004-current-tls-reintegration/artifacts/runtime \
  --nodes 4 --rows 2000 --dim 16 --graph-degree 32 --base-port 44330 \
  --read-rpc-fault-matrix --secure-remote-transport \
  --allow-debug-extension --skip-fault-drills
```

- `cargo check` passed for PG18 production, PG18+`pg_test`, and `ecaz-cli`.
  The CLI check emitted only the pre-existing dead-code warning in
  `corpus/load.rs`.
- Focused `remote_transport::tests`: 15 passed, 0 failed.
- Secure fault matrix: 25 passed, 0 failed across five RPCs and five fault
  classes. Every cell returned no partial rows, drained matching remote work,
  and completed a positive clean retry. The slowest connection-reset cell was
  1.223 seconds under the 5-second bound; all non-reset cells were at most
  509 ms under the 2-second bound.
- The first secure attempt exposed a fixture-only restart defect: the harness
  restarted the owner without `ssl=on` and secure conninfo. The retained
  `failed-tls-restart-console.log` records that diagnostic. Commit
  `eada215ae` makes Task 234 restarts transport-aware; the final 25-cell run is
  the cited acceptance evidence.
- The external fault-fixture run directory was removed after the cited logs
  were captured.

## Artifacts

- `multicluster-console.log` — final 25-cell command output; SHA-256
  `16fccef08bae949cda80d40cf7c52d01e0798cfe227182cf1e3edcaa7e33116e`.
- `runtime/task234-read-rpc-fault-matrix.log` — compact per-cell source of
  truth; SHA-256
  `4d78f9900fbd60d12898d9c38fc2058c97f6047edaab5b6246e3743a43b41abb`.
- `runtime/node{1,2,3,4}-postgres.log` — final PG18 node diagnostics.
- `cargo-pgrx-install-pg18-pg-test.log` — validated extension install;
  SHA-256 `992c53dd4b60e8f47c4cba5a61dbc31fb5a07221d17580bddaa35b4f50adbe0f`.
- `cargo-build-ecaz-cli-debug.log` — validated CLI build; SHA-256
  `9457bf353e9e28cbad1dcc14d86c4c3abccb936de2bb97feb4be1cddb27083c8`.
- `failed-tls-restart-console.log` and
  `failed-tls-restart-runtime/node{1,2,3,4}-postgres.log` — retained diagnostic
  for the fixture restart defect; not used as passing evidence.

The required 10k/50k/100k recall, latency, and storage evidence is stored in
`benchmarks/task234-current-tls-read-rpc-cancellation-ab/manifest.md` and its
packet-local artifacts.
