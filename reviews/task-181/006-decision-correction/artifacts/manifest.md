# Task 181 decision-correction manifest

- Head at correction start: `71690b6c4c06e4457c97a3254c6b6b53bea916d4`
- Task bucket / packet: `reviews/task-181/006-decision-correction/`
- Lane: documentation-only correction; no benchmark rerun and no result change
- Source evidence: `reviews/task-181/005-full-scale-decision/artifacts/full-scale/results.jsonl`
- Retained 10k control evidence:
  `reviews/task-181/005-full-scale-decision/artifacts/full-scale/current-10k-results.jsonl`
- Retained 10k control evidence SHA-256:
  `2181e3ecd1e0966921ab627ba4e64be240c09062c96389c7f57eeafdf58fd536`
- Source manifest: `reviews/task-181/005-full-scale-decision/artifacts/manifest.md`
- Fixture: three exact/disjoint physical owners, release build, 200 held-out
  queries / 2,000 distinct top-10 trials, 50 warm latency samples after 10
  warmups, concurrency 1
- Storage format / rerank: unchanged Task 181 physical ec_distann generation;
  RaBitQ neighbor scoring; exact scoring only for the bounded candidate head
- Command: none; this packet re-evaluates already-recorded A/B results
- Timestamp: 2026-07-15 America/Los_Angeles
- Isolation: one physical generation per scale as recorded by packet 005

## Unchanged result lines

| Scale | Production recall | Candidate recall | Recall delta | Production p50 | Candidate p50 | p50 delta |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k | 0.9990 | 0.9990 | 0.0000 | 33.0 ms | 34.1 ms | +1.1 ms |
| 50k | 0.9545 | 0.9685 | +0.0140 | 42.6 ms | 35.8 ms | -6.8 ms |
| 100k | 0.9275 | 0.9625 | +0.0350 | 42.2 ms | 39.8 ms | -2.4 ms |

Physical storage remained effectively unchanged: 50k candidate was 16,384
bytes smaller than production, and the 10k/100k recorded totals did not expose
a material candidate storage penalty. All topology, remote-engagement, and
release-provenance checks passed.

## Corrected decision

GO to Task 182. The candidate provides a material, scale-increasing recall gain
and is simultaneously faster at 50k and 100k. Its flat 10k recall with a 1.1 ms
p50 increase must remain visible in the production A/B, but does not negate the
larger-scale result.

The proposed 0.9990 recall target and 37.6 ms IVF anchor remain useful context.
They are not stakeholder-approved hard gates and cannot support the superseded
NO-GO by themselves.

Outside review ACCEPT is recorded at
`feedback/2026-07-17-01-reviewer.md`. F181-1 is resolved by the normalized
retained-control JSONL above; no benchmark was rerun and no result changed.
