# Tier-2 Scope — Cross-node SPIRE dispatch latency (follow-up)

Tier-1 (this packet) measures raw inter-node network: TCP connect RTT +
libpq `SELECT 1`. It does **not** measure the SPIRE coordinator→remote
dispatch round-trip, because that path is not reachable on the current
AWS build. This note scopes what Tier-2 needs.

## Why Tier-2 isn't reachable today

1. **Test helper is feature-gated.** Real remote fanout is only produced
   by `tests.ec_spire_test_rewrite_placement_node`, defined in
   `src/tests/mod.rs` under `#[cfg(any(test, feature = "pg_test"))]`
   (`src/lib.rs:18096`). The AWS bootstrap builds plain
   `cargo pgrx install --release` (`scripts/spire-aws/bootstrap-node.sh:143`)
   with no `pg_test` feature, so the `tests.` schema does not exist on
   the nodes.

2. **The working fixture is loopback-only.** `scripts/run_spire_multicluster_pg18_smoke.sh`
   drives both coord and remote PG over a *shared Unix socket dir*
   (`host=$SOCKET_DIR`, lines 131/138/139) — same host. There is no TCP
   cross-node variant.

3. **Bigger gap — no production data-distribution path.** Even setting
   the above aside, SPIRE has no mechanism that takes a loaded corpus,
   partitions it, and ships leaf data to a remote node. Fanout is only
   ever exercised on hand-built 2-row tables whose leaf placement is
   force-rewritten by the test helper. This is why packet 958's bench
   fell through to `not_applicable_local_scan` with `remote_fanout_sum=0`.

## What Tier-2 requires

- **Build:** add a `--features pg_test` build path to the AWS bootstrap
  (or a separate node profile). **Risk:** the test feature compiles
  substantially more code; on a 16 GB r8g.large this may OOM even with
  `CARGO_PROFILE_RELEASE_LTO=thin`. Prefer r8g.2xlarge (64 GB) — needs
  the pending 16→32 vCPU quota raise to coexist with the IVF lane.
- **Fixture port:** adapt the loopback smoke to TCP — set the remote
  conninfo to the remote private IP, ensure `listen_addresses='*'`
  (already set by bootstrap:64) and pg_hba trusts `10.42.0.0/16`
  (already set bootstrap:90), and rewire the registered descriptor
  secret to the cross-node conninfo.
- **Measurement:** time `ec_spire_remote_search_libpq_executor_candidates`
  (and the `connection_check` / `heap_candidate_summary` variants) from
  the coord against a remote-hosted leaf. Subtract the Tier-1 libpq
  baseline to isolate SPIRE dispatch+merge overhead from raw network.

## Recommended sequencing

1. Land the production multi-node data-distribution path as its own task
   (largest item; unblocks all real multi-cluster scale testing).
2. Until then, Tier-2 can still produce a *synthetic* cross-node dispatch
   number via the pg_test fixture port above — useful for network-tax
   characterization, not for recall/latency-at-scale.
