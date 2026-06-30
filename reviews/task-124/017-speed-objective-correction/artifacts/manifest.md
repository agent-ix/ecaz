# Task 124 Speed Objective Correction Manifest

- head SHA before packet: `94a5a6a46c0d6d8ee9e341501b2f6bb7589b28d51`
- task bucket: `reviews/task-124`
- packet path: `reviews/task-124/017-speed-objective-correction`
- lane: objective correction / reopen
- date: 2026-06-29

## Correction

Packet `016-closeout-shelve` treated Task 124 completion as requiring a product
win over the current RaBitQ + f32 baseline. That over-constrained the user
objective. The active objective is to improve TurboQuant speed, while preserving
the TQ focus guardrails and measurement discipline.

## Current Evidence That Still Matters

- Packets 001-002: TQ stage-2 path and attribution counters exist.
- Packet 003: baseline TQ stage-2 A/B exists at 10k/50k/100k.
- Packet 005: final15 was the best measured final exact width; final10 broke recall.
- Packet 011: selected-payload slab produced a small measured TQ latency improvement and should remain part of the speed-improvement baseline.
- Packets 012-014: three structural attempts were measured and rejected, but they do not exhaust all TQ speed work.
- Packet 015: Phase 6 local macOS `F_NOCACHE` validation was attempted, but later
  reviewer feedback corrected that it is not controlled cold-cache evidence; it
  is not a reason to stop TQ speed optimization.

## Next Work

Continue Task 124 with a narrow speed-focused slice. The next packet must report
TQ-before/TQ-after latency and scorer/materialization counters, not just
TQ-versus-RaBitQ product comparison.
