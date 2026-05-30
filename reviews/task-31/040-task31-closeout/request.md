# Task 31 Closeout

Reviewer: please review this Task 31 closeout marker.

## Scope

This packet closes the current IVF M5 optimization pass. It does not add new
measurements or runtime changes.

## Evidence

- `reviews/task-31/038-30203-task31-current-m5-candidate-decision/` is the
  current Apple-Silicon Task 31 source of truth.
- Packet `038` selected:
  - balanced recommendation:
    `ec_ivf`, `pq_fastscan`, `pq_group_size=8`, `nlists=128`,
    `nprobe=96`, `rerank_width=500`
  - quality recommendation:
    `ec_ivf`, `pq_fastscan`, `pq_group_size=8`, `nlists=128`,
    `nprobe=96`, `rerank_width=1000`
- The selected quality point clears the packet's stated gates:
  - recall floor: `0.992 >= 0.99`
  - p50 budget: `12.1 <= 15.0`
- Packet `038` explicitly says no further generic IVF cleanup is justified by
  the measured results.

## Outcome

Task 31 is complete for the requested M5 pass. Future IVF work should start
from the packet `038` selected points and open a new packet only for a narrower
follow-up hypothesis.

## Validation

- `git diff --check`

No tests or benchmarks were run because this is a docs-only closeout marker.
