# Review Request: C1 ADR-032 Node-Centric Redesign Index

## Purpose

This packet is now the ADR-032 umbrella/index packet.

On 2026-04-13, the original oversized packet `293` was retrospectively split so that each
significant ADR-032 attempt has its own packet with its own full measurement log.

## Context

ADR-031 remains the best kept current-format warm-path win on the real-`50k` seam.

ADR-032 is the larger runtime-redesign lane that tries to compound that win without taking on
ADR-030's index-v2 encoding/layout work yet.

Packets already split before this cleanup:

- `291`: [review/291-c1-adr032-element-cache-arena/request.md](../291-c1-adr032-element-cache-arena/request.md)
- `292`: [review/292-c1-adr032-neighbor-cache-arena/request.md](../292-c1-adr032-neighbor-cache-arena/request.md)

Packets split retrospectively out of the original `293`:

- `294`: [review/294-c1-adr032-fused-node-cache-cut/request.md](../294-c1-adr032-fused-node-cache-cut/request.md)
- `295`: [review/295-c1-adr032-slot-frontier-scheduler-cut/request.md](../295-c1-adr032-slot-frontier-scheduler-cut/request.md)
- `296`: [review/296-c1-adr032-score-pressure-diagnostic/request.md](../296-c1-adr032-score-pressure-diagnostic/request.md)
- `297`: [review/297-c1-adr032-exact-on-head-frontier-promotion/request.md](../297-c1-adr032-exact-on-head-frontier-promotion/request.md)
- `298`: [review/298-c1-adr032-full-layer0-source-promotion/request.md](../298-c1-adr032-full-layer0-source-promotion/request.md)
- `299`: [review/299-c1-adr032-bounded-low-ef-promotion-budget/request.md](../299-c1-adr032-bounded-low-ef-promotion-budget/request.md)
- `300`: [review/300-c1-adr032-low-ef-head-window/request.md](../300-c1-adr032-low-ef-head-window/request.md)
- `301`: [review/301-c1-adr032-binary-score-calibration/request.md](../301-c1-adr032-binary-score-calibration/request.md)
- `302`: [review/302-c1-adr032-low-ef-exact-score-floor/request.md](../302-c1-adr032-low-ef-exact-score-floor/request.md)

## Current Read

- `297` is the first real ADR-032 keep candidate and the current kept ADR-032 runtime cut on this
  branch.
- `298` through `302` show that low-`ef_search` quality recovery is not solved by local-per-source
  or score-shape heuristics.
- The next credible ADR-032 slice should be a global frontier-level exact-work policy rather than
  another local-per-source tweak.
- Future ADR-032 attempts should start at packet `303+` instead of extending this packet.

## Feedback

The existing reviewer feedback in
`feedback/2026-04-13-01-reviewer.md` predates the retrospective split. It should be read as
cross-cutting ADR-032 feedback spanning packets `294` through `302`, especially `297` through
`302`.
