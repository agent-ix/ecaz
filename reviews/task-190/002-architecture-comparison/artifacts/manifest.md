# Task 190 packet 002 architecture comparison

Date: 2026-07-23 (America/Los_Angeles)

Source evidence and lane identity are recorded in packet 001. No new
measurement is claimed here.

## Compared families

| Dimension | Coordinator traversal replica (`ARCH-02` / `TRAV-28`) | Dedicated binary traversal RPC (`ARCH-07`) |
|---|---|---|
| Measured component addressed | Removes serial owner-expansion remote/backend boundaries; local traversal replaces remote service | Serialization/service path around the same serial boundaries |
| Direct measured ceiling | 4.078--5.013 ms/scan wait; owner work is not counted as guaranteed saving | 0.071 ms/scan measured connection/encode/decode; unknown share of wait may be backend/service overhead |
| Boundary count | Can remove traversal RPCs; final lazy payload RPC remains | Ten traversal remote/backend boundaries remain unless it also becomes a new orchestration architecture |
| Logical bytes | Avoids about 24.4 KiB traversal traffic/scan | Bytes already small; binary packing is not bandwidth-motivated |
| 100k storage | Faithful upper bound +2,496,626,688 bytes per coordinator; compact lower-envelope estimate about 1.445 GB, unmeasured | No generation replica |
| Build/network | Full per-epoch copy and validation; multiplied by coordinator count | New service deployment but no epoch bulk copy |
| DML | Invalidate before visible mutation; remote fallback until coherence is separately designed | Owners remain authoritative; normal DML shape retained |
| Lifecycle | New Ready/active/stale/retired derived state, fingerprint binding, scan fencing, reclaim | New protocol/service version, authentication, cancellation, snapshot and epoch fencing |
| Failure | Missing/stale/corrupt replica falls back; no partial result | Service outage must fall back or fail under a new transport boundary |
| Compatibility | Existing on-owner generation and payload endpoint retained | New endpoint and deployment compatibility surface |
| Rollback | Invalidate/drop replica and use existing traversal immediately | Disable new service/client and return to SQL transport |
| Confidence | High that round trips are removed; uncertain local-copy cache cost | Low that codec alone matters; service-level saving is unattributed |

## Replica storage arithmetic

The current 100k owner graph relations total:

```text
274,563,072 + 276,299,776 + 276,062,208 = 826,925,056 bytes
```

The full physical generation, including frozen row tier, directory and control,
is 2,496,626,688 bytes. A faithful replica therefore has a known 1.0×
per-coordinator upper envelope.

For design comparison only, graph bytes plus one raw
100,000 × 1,536 × 4-byte vector tier plus the current 3,194,880-byte directory
is 1,444,519,936 bytes (57.9% of the generation). That is a lower-envelope
estimate: it excludes real tuple/index/control/alignment overhead and is not a
benchmark result or accepted storage target.

## Selection

Select the coordinator traversal replica.

It has a direct, measurable path to the dominant cost: remove the serial
remote traversal boundary while keeping final payloads hash-owned. Dedicated
binary RPC is rejected for this escalation because its directly attributed
serialization-adjacent ceiling is negligible and a service capable of
attacking the remaining wait would introduce a new failure/deployment model
without removing the sequential remote/backend boundaries.

`ARCH-08` shared-memory/Unix-domain transport is also not selected. It could
reduce same-host IPC/service overhead on this benchmark lane, but does not
serve genuinely remote owners or remove the sequential owner boundaries; it
would make the decision topology-specific.

The selection is conditional on Task 198 proving same-result semantics and a
material end-to-end win at an explicitly accepted measured storage/build cost.
Task 190 does not promote production behavior.
