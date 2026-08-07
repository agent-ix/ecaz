# Task 216 packet 003 manifest

- Packet: `reviews/task-216/003-full-scale-decision/`
- Decision: STOP; no full-scale matrix authorized after the isolated gate.
- Source evidence: `reviews/task-216/002-isolated-candidate/`.
- Control source SHA: `e8f15ab0c68887c176a260107fe826c402c2f827`.
- Candidate source SHA: `6662b302f8370695320dcb36edda3cd291c8c1bc`.
- Suite config: `reviews/task-216/002-isolated-candidate/artifacts/task216-mat15-100k.json`.
- Control result artifact:
  `reviews/task-216/002-isolated-candidate/artifacts/control/results.jsonl`
  SHA256 `0833af02189bf11a38b99cbf2e53748bc043ee7f5e7194a5602d549359a32352`.
- Candidate result artifact:
  `reviews/task-216/002-isolated-candidate/artifacts/candidate/results.jsonl`
  SHA256 `ac97d9b15cc0626988189cdf01f962323c0d5426eb1f7ed61376c5097f187482`.
- Control physical 100k: recall `0.9275`; latency mean/p50/p95/p99/max
  `40.60/39.30/54.30/57.20/58.50 ms`; storage
  `2496659456 B`, amplification `1.351173`.
- Candidate physical 100k: recall `0.9295`; latency mean/p50/p95/p99/max
  `86.10/85.00/113.70/127.00/127.70 ms`; storage
  `2496634880 B`, amplification `1.351160`.
- Topology gates: 3 owners, 100,000 source rows, zero orphans, two remote
  owner probes passed in both runs. Candidate release preflight SHA was
  `6662b302...` on all nodes.
- Identity: physical ordered prediction arrays differ in 2/200 rows; single
  arrays match. Physical seed digests differ, so this gate is a hard stop and
  the cause is left unresolved rather than attributed.
- NFR-021: actual unavailable at this diagnostic scale in both suite runs;
  no conformance claim is made.
