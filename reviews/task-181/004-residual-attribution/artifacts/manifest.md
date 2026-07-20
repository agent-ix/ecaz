# Task 181 packet 004 artifact manifest

This packet records the Phase 4 exact-neighbor residual-attribution
non-trigger. It creates no new benchmark run.

## Inputs

- Head / installed extension SHA: `e75dfc14bbf0c1ff406a7dc1795f7e1c2f4514d8`.
- Best bounded candidate: 4,096 disjoint-training landmarks, exact bounded
  scoring, 32 returned seeds, BW4/H100, RaBitQ traversal.
- Candidate result: 0.9625 distinct recall@10 (95% CI 0.9532-0.9700), from
  `reviews/task-181/003-fixed-cap-policy-screen/artifacts/fixed-cap-100k/results.jsonl`.
- Reproduced owner-oracle result: 0.9970 (95% CI 0.9935-0.9986), from
  `reviews/task-181/002-existing-head-coverage/artifacts/phase1-current-100k/results.jsonl`.
- Timestamp: 2026-07-15 PDT.

## Trigger decision

The candidate trails the owner oracle by 0.0345 absolute recall, seven times
the Phase 4 trigger limit of 0.0050. Even the candidate's upper Wilson bound
of 0.9700 is below the owner's lower bound of 0.9935. The result is therefore
not close enough to infer or measure a residual neighbor-code contribution.

Per the preregistered task, same-seed RaBitQ-versus-exact-neighbor traversal is
not run. Introducing exact-neighbor traversal, OPQ, a new quantizer, or a graph
change here would confound the still-dominant entry-membership loss.

No corpus, fixture, suite output, or operational log belongs to this packet.
The cited packet-local Task 181 results are the durable evidence.
