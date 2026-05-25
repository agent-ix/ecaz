# Artifact Manifest — SPIRE AWS Tier-1 Network Latency

Packet: `reviews/task-30/959-spire-network-latency/`
Owner: coder B (SPIRE AWS lane)
Branch: `task-30-phase13-spire-aws-prep`
Head SHA: `9fc846707` (merge of `origin/main` into the branch)

## Cluster Identity

| Field | Value |
| ----- | ----- |
| Role | SPIRE multi-cluster, Tier-1 network probe only (no fanout) |
| Topology | 1 coord `r8g.large` + 1 remote `r8g.large`, `remote_count=1` |
| vCPU fit | 2 + 2 = 4 vCPU; IVF cloud-bench held 12, quota 16, exact fit |
| Region / AZ | us-west-2 / us-west-2a (same subnet `10.42.1.0/24`) |
| Coordinator | `i-091b9082bdb774123` @ `10.42.1.162` |
| Remote | `i-0b9367e808370713d` @ `10.42.1.58`, node_id 2 |
| PG | 18, pgrx `--release` (no `pg_test`), `ecaz_coord` superuser role |
| Network | ENA, private IPs; SG permits only tcp/5432 between nodes (ICMP blocked) |

## Artifacts

| # | Artifact | What | Command | Timestamp | Key result |
|---|----------|------|---------|-----------|-----------|
| 1 | `network-latency.log` | TCP-connect RTT + libpq SELECT 1, coord→remote, ×100 each | `spire_net_probe.sh` via SSM cmd `2c3be242` | 2026-05-25 UTC | TCP RTT p50 **0.065 ms**; libpq SELECT 1 p50 **10.2 ms** |
| 2 | `spire_net_probe.sh` | The probe script (TCP connect timing + libpq timing in python3) | n/a (source) | 2026-05-25 | reproducible probe |
| 3 | `install.log` | `make install-extension` build+PG-start on both nodes via SSM | SSM cmd `cf448abd` | 2026-05-25 | both nodes built, PG up |
| 4 | `aws-topology.json` | terraform topology output | `terraform output -json topology` | 2026-05-25 | coord+remote IPs/IDs |

## Key Results

- **Raw inter-node network RTT (same-AZ, ENA, TCP connect):**
  p50 0.065 ms, p95 0.108 ms, max 0.146 ms, mean 0.068 ms (n=100).
  Network is not a dispatch bottleneck.
- **libpq connect + SELECT 1 (fresh connection per iteration):**
  p50 10.2 ms, p95 11.4 ms, mean 11.6 ms, one 145 ms cold outlier (n=100).
  Dominated by per-connection backend fork + auth, not the network hop.

## Scope Limits (see `../tier2-scope.md`)

- This packet does **not** measure SPIRE coord→remote dispatch latency.
  That path needs a `--features pg_test` build (test placement helper)
  and a TCP port of the loopback-only smoke fixture — neither present
  on this release-build topology.
- No corpus fanout was attempted: SPIRE has no production multi-node
  data-distribution path (fanout only via the pg_test test helper on
  hand-placed 2-row tables). Loading 50k/100k would reproduce packet
  958's `not_applicable_local_scan` null result.
