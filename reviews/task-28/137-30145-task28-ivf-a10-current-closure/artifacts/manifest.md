# Artifact Manifest: 30145 Task 28 IVF A10 Current Closure

## `request.md`

- head SHA: `69150e78`
- packet/topic: `30145-task28-ivf-a10-current-closure`
- lane / fixture / storage format / rerank mode: A10 synthesis over TurboQuant, PQ-FastScan g8, and RaBitQ
- command: synthesis only; no new benchmark command
- timestamp: 2026-04-29 local
- isolated/shared surface: synthesis over packet-local isolated surfaces from 30097, 30126, 30137, 30143, 30144, and 30152
- key result lines:
  - Recommendation: keep `quantizer = 'auto'` unchanged in Task 28.
  - Recommend explicit `quantizer = 'pq_fastscan', pq_group_size = 8` for the 100k high-dimensional local IVF lane.
  - Keep TurboQuant as the safer smaller-corpus recall@100 profile.
  - RaBitQ remains selectable but is not a current IVF default candidate because corrected 10k/25k p50 latency remains slower than TurboQuant and PQ-FastScan at the matched A10 point.
