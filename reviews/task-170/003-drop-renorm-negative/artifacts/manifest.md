# Task 170 / Packet 003 Drop Renorm Artifact Manifest

- task bucket: `reviews/task-170/003-drop-renorm-negative`
- head SHA at packet creation: `2de14b389`
- lane: code cleanup after measured-negative Slice 2
- storage format / rerank mode: no new measurement; see packet 002 for the A/B evidence
- timestamp: 2026-07-05 local session

## Artifacts

- `request.md`: review request and rationale.

## Evidence Source

The decision evidence is packet-local to `reviews/task-170/002-length-renorm-ab/`:

- `artifacts/summary.md`
- `artifacts/manifest.md`
- `artifacts/baseline/results.jsonl`
- `artifacts/renorm-fixed/results.jsonl`

Key cited result:

- pure TQ default 100k nprobe 40 latency regressed from `1.85 ms` to `11.80 ms`;
- pure TQ default 100k recall improved from `92.50%` to `93.13%` only at nprobe 64;
- stage2 recall was unchanged across all measured cells.
