# Review Request: C1 Warm-Cache Verified Surface

## Context

Packet `260` changed the C1 interpretation materially:

- representative warm-cache SQL startup on the real `10k` corpus is now around
  `4.1ms` at `m=8, ef_search=40`
- repeated plain scans of that same representative query are around
  `1.1ms/query`
- the large remaining gap on the current verified surface is cold-cache I/O,
  not hidden warm-path CPU overhead

That means the current C1 reporting is incomplete relative to `NFR-001`, which
already requires warm-cache and cold-cache results to be reported separately
when feasible.

## Problem

The current verified launcher and durable C1 artifacts still center the cold
`EXPLAIN`-timed surface. That makes the C1 read misleading now that the warm
surface appears to be at or below the NFR target on the representative `10k`
lane.

## Planned work

1. Add a verified warm-cache measurement seam that preserves the existing
   planner/index guard.
2. Capture a representative warm-cache result on the real `10k` `m=8` lane.
3. Report warm and cold separately in the C1 packet and status/docs.

## Exit criteria

- warm-cache measurement is reproducible through a committed repo-local seam
- the warm run still refuses to measure the wrong planner/index path
- C1 reporting clearly distinguishes warm vs cold instead of treating the cold
  surface as the only headline
