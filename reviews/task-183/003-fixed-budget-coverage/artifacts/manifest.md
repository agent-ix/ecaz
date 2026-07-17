# Task 183 fixed-budget coverage manifest

- Pre-registration head: `de3b54f82`
- Task bucket / packet: `reviews/task-183/003-fixed-budget-coverage/`
- Lane: algorithm pre-registration; no Phase 2 code or measurement yet
- Frozen baseline: Task 183 packet 002 trained RaBitQ at 100k, recall 0.9625
  and warm p50 43.8 ms
- Frozen upper reference: same-generation owner-scan RaBitQ recall 0.9970;
  never selectable
- Candidate input: only disjoint training rows 201--400
- Evaluation input: held-out rows 1--200; unavailable to builders
- Fixed query work: cap 4,096, exact head scoring, 32 seeds, BW4/H100
- Fixed graph/codec: degree 32, RaBitQ neighbor codes/traversal, exact rerank
- Isolation: future policy arms use fresh one-index-per-table physical
  generations through a checked-in `ecaz bench suite` config
- Timestamp: 2026-07-17 America/Los_Angeles

## Frozen policy names

- `training_landmarks`: Task 182 frequency/coverage control
- `training_region_balanced`: deterministic geometry-region round-robin
- `training_query_facility`: deterministic rotated query-neighborhood
  round-robin

No Phase 2 result is claimed. Corpus/query TSVs, truth caches, node logs, and
live run directories will not be committed.
