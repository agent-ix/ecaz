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
  `f4ed3968395a55b9263723ae9e34ab2467993994de178d5c3f8e3e0b5eb60521`.
- `cargo-pgrx-install-pg18-ssl-pg-test-release.log` — SSL PG18 release install;
  SHA-256
  `a9e8a8782c2f08de2a7700f654322c5af37cefe7bb33e23c18d894dcd844867e`.
- `cargo-build-ecaz-cli.log` — clean-head CLI build; SHA-256
  `d1a7616aba764213d0e121820e08f8171e0283c586421868d279fd487f677056`.
- `cargo-test-remote-transport.log` — focused current-head unit results;
  SHA-256
  `978e05a880be4defa0619d198bd4a0ab177398445cc1c722e3c29b1b97e326ca`.

The final packet deliberately excludes accumulated PostgreSQL logs from failed
fixture-development attempts. The compact matrix and final console above are
the only runtime sources cited for acceptance.
