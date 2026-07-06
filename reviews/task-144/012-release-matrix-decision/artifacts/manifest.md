# Task 144 Packet 012 Artifact Manifest

- head SHA: `4204ffbda`
- task bucket: `reviews/task-144/012-release-matrix-decision/`
- packet type: decision checkpoint
- timestamp: 2026-07-05
- source evidence:
  - `reviews/task-144/009-release-matrix-10k-r2/`
  - `reviews/task-144/010-release-matrix-50k-r2/`
  - `reviews/task-144/011-release-matrix-100k-r2/`

## Evidence Chain

This packet does not introduce new measurements. It records the Task 144 promote / iterate / escalate decision against the approved r2 matrix evidence:

- 10k release matrix r2: packet 009 request and artifacts.
- 50k release matrix r2: packet 010 request, artifacts, and reviewer approval feedback.
- 100k release matrix r2: packet 011 request and artifacts.

All cited measurements trace to packet-local `ecaz bench suite` artifacts in those source packets.

## Cross-Scale Summary

| scale | best candidate for recall >= 0.99 | candidate rows | production p50 | result |
| --- | --- | ---: | ---: | --- |
| 10k | closure_e050_b8-ratio400 @ np16 | 2.57% | 7.670 ms | clears original AC |
| 10k | closure_e025_b8-adaptive @ np32 | 4.36% | 7.332 ms | clears original AC |
| 50k | fixed_b2-adaptive @ np96 | 35.6834% | 20.434 ms | misses <=5% scan budget |
| 100k | closure_e050_b8-ratio200 @ np96 | 78.6594% | 73.132 ms | misses <=5% scan budget |

## Decision

Do not promote Task 144 closure/ratio pruning as an accepted operating point.

Decision: **iterate / escalate**, not promote.

Reasons:

- The 10k AC does not reproduce at 50k or 100k.
- Ratio pruning has no stable useful operating point: tight ratio values lose recall; loose values converge to fixed/no-pruning behavior.
- At 50k, the least-bad 0.99 row is plain `fixed_b2`, not closure.
- At 100k, only `closure_e050_b8` reaches 0.99 recall, but at about 79% candidate row scan and 568.8 MiB / 7.1315 mean replicas per vector.
- The evidence points to a scaling problem in the candidate-generation strategy, not a threshold tuning problem.
