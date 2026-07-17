# Task 183 residual-plan manifest

- Preliminary plan head: `07a16b86e235a380d539d55be0a26fbfbc2e6e8c`
- Frozen baseline head: `973f4dc3db57650c3a6f8d41818880f146e87896`
- Task bucket / packet: `reviews/task-183/001-residual-plan/`
- Lane: planning only; no code, fixture, or benchmark
- Frozen baseline: Task 182 production-path 10k/50k/100k A/B
- Source evidence: `reviews/task-182/006-production-ab/artifacts/run/results.jsonl`
- Source manifest: `reviews/task-182/006-production-ab/artifacts/manifest.md`
- Source decision: `reviews/task-182/006-production-ab/request.md`
- Command: none
- Timestamp: 2026-07-17 America/Los_Angeles
- Isolation: not applicable; no measurement

## Frozen planning facts

- Task 182 productionized `training_landmarks_exact` with cap 4,096, exact
  scoring, at most 32 returned seeds, BW4/H100, graph degree 32, RaBitQ
  neighbor traversal, and exact final rerank.
- Training uses exactly rows 201--400 from each declared staged query file;
  evaluation uses held-out rows 1--200. The persisted policy, training digest,
  head sample digest, count, and cap were attested in every trained cell.
- Trained production distinct recall was 0.9990 / 0.9685 / 0.9625 and warm p50
  was 38.5 / 39.3 / 41.4 ms at 10k/50k/100k.
- Relative to unchanged production, recall changed by 0.0000 / +0.0140 /
  +0.0350 and warm p50 changed by +4.3 / -4.8 / +0.7 ms.
- Physical generation bytes were 242,761,728 / 1,242,742,784 /
  2,496,626,688; cached-head estimates were 25,826,119 / 25,900,434 /
  25,892,203; physical build time was 78,176 / 426,094 / 912,404 ms.
- The same-generation owner oracle reached 0.9995 / 0.9970 / 0.9970 recall
  with normal RaBitQ traversal but remained O(N), approximately 7x / 31x / 62x
  trained p50, and non-selectable.
- All nine Task 182 cells used fresh three-owner physical generations, passed
  topology and two-remote-owner engagement gates, and unanimously attested
  release SHA `f02cf58a0224dc8a420dbb4964425fe31338e1e2`.

No new measurement result is claimed by this packet. Future evidence must be
produced through checked-in `ecaz bench suite` configs in the owning Task 183
packets. Task 182 artifacts are cited in place and are not copied.
