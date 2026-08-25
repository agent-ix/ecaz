# Task 235 PG18 2PC/lifecycle fault-matrix artifact manifest

Date: 2026-08-25 (America/Los_Angeles)

## Scope and head

- Task bucket: `reviews/task-235/`
- Packet: `003-2pc-lifecycle-fault-matrix/`
- Current-TLS campaign base: `387c2137f85a7950fb243d34bb0adbb7903b5c07`
- Validated candidate: `b871d5481376df87c60ae486d68bb78519944c21`
- PostgreSQL: PG18 release extension with `pg_test`, built and installed with
  `/home/peter/.ecaz/toolchains/pg18-ssl/bin/pg_config` (`--with-openssl`).
- Fixture: three fresh physical PostgreSQL nodes, 2,000 rows, dimension 16,
  graph degree 32, one build shard, verify-full mutual TLS, client certificate
  authentication, and plaintext rejection. Each node has its own table/index
  surface; this is not a shared-table benchmark fixture.
- Runtime: 2026-08-25 15:26:00--15:26:24 PDT.
- External run directory:
  `/home/peter/.ecaz/clusters/task235-secure-release-final`. The fixture stopped
  all nodes after capturing the cited packet artifacts; the cluster directory
  is not review evidence.

## Commands

```text
cargo build -p ecaz-cli

cargo pgrx install --release \
  --pg-config /home/peter/.ecaz/toolchains/pg18-ssl/bin/pg_config \
  --no-default-features \
  --features 'pg18 pg_test distann-head-attribution-benchmark'

cargo test --lib remote_transport::tests \
  --no-default-features --features pg18,pg_test

/home/peter/.cargo-target/debug/ecaz dev distann-multicluster \
  local-multinode-pg18 \
  --pgbin /home/peter/.ecaz/toolchains/pg18-ssl/bin \
  --run-dir /home/peter/.ecaz/clusters/task235-secure-release-final \
  --artifact-dir reviews/task-235/003-2pc-lifecycle-fault-matrix/artifacts \
  --nodes 3 --rows 2000 --dim 16 --graph-degree 32 --base-port 46200 \
  --secure-remote-transport --write-lifecycle-fault-matrix \
  --allow-debug-extension --skip-fault-drills
```

## Results

- Release preflight passed unanimously on all three nodes with extension git
  SHA `b871d5481376df87c60ae486d68bb78519944c21`, build profile `release`, and
  features `distann-head-attribution-benchmark,pg-test,pg18`.
- Secure-transport preflight reported `tls=verify-full`, client-certificate
  authentication, and plaintext rejection.
- The matrix passed exactly 23 scenario records and 107 total records:
  eight lifecycle lost-ack/replay cells, one operator status-unavailable STOP
  cell, and fourteen write/recovery cells.
- Lifecycle coverage: handoff begin, stage, seal, and abort; epoch publish;
  predecessor retirement; retire application; and cancelled-generation
  reclaim. Every cell observed an injected mixed participant state, converged
  through the production recovery API, accepted duplicate recovery, and left
  the required final state with zero reclaimed-relation residue where
  applicable.
- Write coverage: clean commit; before/during/after endpoint mutation;
  coordinator death after prepare plus immediate owner crash/restart; lost
  precommit, commit-prepared, and rollback-prepared acknowledgements; one-owner
  partial commit; missing intent; prepared-slot saturation; and routed
  tombstone owner-backend death. Every cell reached the asserted source/owner,
  prepared-xact, intent, source-map, graph/row, directory, and tombstone state;
  duplicate recovery emitted zero actions.
- Focused transport tests passed 19/19 (2,588 filtered), including coordinator
  commit-status authority, stable fault names, nonzero connect timeout,
  interrupt handling, outcome classification, redaction, and the prepared-slot
  operator hint.

## Artifacts

- `task235-write-lifecycle-fault-matrix.log` — compact scenario and recovery
  source of truth; SHA-256
  `c55844a4728ce15048ad49fca91e239036c3d6ca2be5d950c3bd7e3abad3b5de`.
- `secure-release-matrix-console.log` — full clean-SHA fixture transcript and
  preflight; SHA-256
  `6f2201f6e5fc533c14697cceff18478dc63318a523a0b941ec713668490ff410`.
- `cargo-pgrx-install-pg18-ssl-pg-test-release.log` — SSL PG18 release install;
  SHA-256
  `a09e09da93cee97d2a791356fedb14268ec5916801ef265e56638abf11f2d414`.
- `cargo-build-ecaz-cli.log` — clean-head CLI build; SHA-256
  `a54a63f478122527149440790c25985407de76281f5e75d75bf9fdad497096b7`.
- `cargo-test-remote-transport.log` — focused current-head unit results;
  SHA-256
  `6511133ecfa57c3be930ea10cf1285d491f796798e0f8950745253cab0ff077c`.

The final packet deliberately excludes accumulated PostgreSQL logs from failed
fixture-development attempts. The compact matrix and final console above are
the only runtime sources cited for acceptance.
