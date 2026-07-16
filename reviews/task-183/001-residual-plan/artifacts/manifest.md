# Task 183 residual-plan manifest

- Head at plan start: `07a16b86e235a380d539d55be0a26fbfbc2e6e8c`
- Task bucket / packet: `reviews/task-183/001-residual-plan/`
- Lane: planning only; no code, fixture, or benchmark
- Required future baseline: Task 182 production-path 10k/50k/100k A/B
- Source evidence: `reviews/task-181/005-full-scale-decision/artifacts/full-scale/results.jsonl`
- Corrected source decision: `reviews/task-181/006-decision-correction/`
- Command: none
- Timestamp: 2026-07-16 America/Los_Angeles
- Isolation: not applicable; no measurement

## Frozen planning facts

- Task 181 selected a bounded cap-4,096, exact-scoring, 32-seed trained landmark
  head for Task 182 production validation.
- Relative to unchanged production, the benchmark-only candidate changed recall
  by 0.0000 / +0.0140 / +0.0350 and warm p50 by +1.1 / -6.8 / -2.4 ms at
  10k/50k/100k.
- The owner oracle reached 0.9970 at 50k/100k with RaBitQ neighbor traversal but
  remained O(N) and non-selectable.
- Task 183 inherits only Task 182's measured production outcome; these Task 181
  facts motivate the plan but cannot substitute for its baseline.

No result is claimed by this packet. Future evidence must be produced through
checked-in `ecaz bench suite` configs in the owning Task 183 packets.
