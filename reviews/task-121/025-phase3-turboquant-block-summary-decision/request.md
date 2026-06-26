# Review Request: Task 121 TurboQuant Block-Summary Decision

## Scope

This packet closes the remaining Phase 3 TurboQuant/default block-summary item
as an explicit implementation-gap decision, not as new benchmark evidence.

No code was changed and no benchmark was run in this packet.

## Decision

Do not implement TurboQuant/default block pruning in Task 121. Treat it as a
separate future implementation task only if block pruning becomes
promotion-worthy.

## Basis

TurboQuant was already measured as a route-screen compatibility/control axis in
packet 001:

```text
nprobe: 8 16 24 32 48 64 96
baseline recall@10:   0.7250 0.8525 0.9045 0.9310 0.9645 0.9825 0.9975
turboquant recall@10: 0.7250 0.8525 0.9045 0.9310 0.9645 0.9825 0.9975
```

Packet 005 therefore excluded TurboQuant from the Phase 2 route-recovery grid:
it was not a routing lever, only a compatibility/Pareto follow-up.

The current code also does not actually run global/sample block pruning for
TurboQuant. The build path can emit a non-RaBitQ mean summary, but the scan
selectors return `None` unless the prepared scorer payload is RaBitQ:

```text
src/am/ec_spire/scan/candidates.rs:1830-1833
src/am/ec_spire/scan/candidates.rs:1893-1895
src/am/ec_spire/scan/candidates.rs:1927-1929
```

The generic payload scorer can score summary chunks, but the radius bonus and
the active pruning selectors are RaBitQ-oriented:

```text
src/am/ec_spire/scan/candidates.rs:2023-2028
src/am/ec_spire/scan/candidates.rs:2172-2179
```

## Read

The RaBitQ Phase 3 pruning evidence is not strong enough to justify adding a
new TurboQuant pruning implementation and rerunning the full 10k/50k/100k
matrix. Packet 024 shows the retuned policy is recall-neutral but only improves
the high-recall `nprobe=96` point; low operating points are flat and object
bytes are unchanged.

Given that, implementing TurboQuant/default block pruning inside Task 121 would
expand the work without improving the task's route-stage answer. This packet
therefore records the explicit implementation gap the reviewer requested before
Phase 4.

## Evidence

Packet manifest: `artifacts/manifest.md`
