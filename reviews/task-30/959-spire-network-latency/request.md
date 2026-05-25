# Review Request — SPIRE AWS Tier-1 Network Latency

Packet: `reviews/task-30/959-spire-network-latency/`
Branch: `task-30-phase13-spire-aws-prep`
Head SHA: `9fc846707`
Status: OPEN

## Summary

Captured inter-node network latency for a 1+1 SPIRE topology on AWS
Graviton (r8g.large, same AZ). This is **Tier-1 only**: raw network +
libpq, no SPIRE dispatch/fanout (see scope limits below).

- **TCP-connect RTT coord→remote:** p50 **0.065 ms**, p95 0.108 ms,
  max 0.146 ms (n=100). Same-AZ ENA; network is not a bottleneck.
- **libpq connect + SELECT 1:** p50 **10.2 ms**, mean 11.6 ms (n=100),
  dominated by per-connection backend fork/auth, not the ~65 µs hop.

Evidence: `artifacts/network-latency.log`, reproduced by
`artifacts/spire_net_probe.sh` (SSM command `2c3be242`).

## Why no fanout / dispatch numbers

Traced during this run (full detail in `tier2-scope.md`):

1. Real remote fanout is only produced by
   `tests.ec_spire_test_rewrite_placement_node`, gated
   `#[cfg(any(test, feature = "pg_test"))]` (`src/lib.rs:18096`). The AWS
   bootstrap builds plain `--release`, so the helper is absent.
2. The only working multi-cluster fixture
   (`scripts/run_spire_multicluster_pg18_smoke.sh`) is loopback-only
   (shared Unix socket dir), not TCP cross-node.
3. **No production multi-node data-distribution path exists** — fanout is
   only ever exercised on hand-placed 2-row tables. This is why packet
   958's bench fell through to `not_applicable_local_scan`
   (`remote_fanout_sum=0`), and why loading a 50k/100k corpus here would
   reproduce that null result rather than fan out.

## Asks for reviewer

1. Sanity-check the network interpretation (65 µs RTT credible same-AZ ENA?).
2. Confirm the Tier-2 prerequisites in `tier2-scope.md` before any
   pg_test-build AWS spend.
3. Prioritization steer: the production data-distribution gap (item 3
   above) blocks all real multi-cluster scale testing — worth a
   dedicated task ahead of further AWS latency work?

## Topology lifecycle

Provisioned 1+1 r8g.large (4 vCPU, exact fit under the 16-vCPU quota
while IVF held 12). Torn down after capture — see teardown log.
