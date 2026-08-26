---
task: 224
packet: 001-plan
agent: Codex
role: coder
model: gpt-5
date: 2026-08-25
seq: 01
---

# Task 224 owner payload locality measurement plan

This packet requests review of the Task 224 attribution and candidate-selection
plan. Task 223 is review-closed ACCEPT/STOP, so the dependency is satisfied and
the Task 222 projected production path is the control.

Task 223's accepted 0.514999 ms payload-SQL ceiling settles the production
id-only projection but does not automatically settle Task 224. This task also
registers narrow scalar, vector-bearing, and externally toasted projection
shapes. The toasted arm may have a materially different heap/detoast ceiling,
so it must be measured rather than inferred from the id-only result.

P1 adds feature-only owner attribution under
`distann-head-attribution-benchmark`; normal release behavior and SQL shape
remain unchanged. On one fresh 100k physical generation, each of the four
projection shapes records:

- requested TIDs, distinct heap blocks, rows per block, and the displacement
  required to sort by `(block, offset)` and restore request rank;
- heap and TOAST buffer hits/reads around the payload fetch where PostgreSQL's
  backend buffer-usage counters can be sampled without changing execution;
- projected non-NULL values, external-toast values, stored/logical bytes, and
  binary-send bytes; and
- separately attributable heap/TOAST access, detoast/send, and retained
  response-construction time, reconciled to the existing payload-SQL parent.

The measurement runs through `ecaz bench suite`, with a checked-in SuiteConfig,
packet-local manifest/results, clean release+feature preflight, and one
generation shared across projection arms. It includes warm and controlled
residency observations; cache state is reported, not assumed from TID
distribution.

No candidate is authorized unless an independently addressable heap or
detoast/send bucket reaches at least 1 ms/scan or 5% of that arm's matched warm
end-to-end mean at 100k. If both pass, advance only the larger percentage
ceiling: MAT-25 for heap-block/TID reorder or MAT-26 for block-batched
detoast/binary send. If neither passes, packet 002 records STOP and packets
003/004 are decision-obviated.

Any candidate must preserve exact request/result rank after physical reorder,
pass the complete materialization semantic/failure matrix, and win an isolated
same-generation 100k A/B before receiving the required 10k/50k/100k recall,
latency, storage, build, and DML closeout matrix.

Please focus review on whether the four shapes cover Task 224's locality scope,
whether buffer/TOAST counters are safely attributable, whether the two
candidate ceilings are independently addressable, and whether the single-
candidate tie-break prevents an unregistered combined optimization.

This is planning-only. No code was changed and no tests or benchmarks were
run.

## Coder addendum after packet-001 feedback

Packet 002 realized this plan with the following explicit reviewer-requested
constraints:

- summed per-owner stages establish bucket size, while
  `materialize_owner_endpoint_critical` bounds achievable serial share and is
  printed beside the summed result;
- the attribution run has no uninstrumented production-SQL denominator, so its
  profiled warm means are not candidate baselines; packet 003 must compare
  both arms under the same instrumentation state, preferably production SQL;
- SQL result `LIMIT 10` is distinct from the harness's swept
  `ec_distann.top_k=32` search GUC, and `client_result_rows=10` confirms the
  former; and
- id-only is the shipped control, narrow-scalar is shipped-capable,
  vector-bearing is shipped-capable exploratory stress, and toasted is
  synthetic exploratory stress. An exploratory arm may authorize only an
  isolated candidate screen, never default-on production behavior.

The packet-002 reviewer accepted this plan as realized, review-closed packet
002, retired MAT-25, and authorized MAT-26 alone for packet 003. Final verdict:
`reviews/task-224/002-locality-attribution/feedback/2026-08-25-02-reviewer.md`.
