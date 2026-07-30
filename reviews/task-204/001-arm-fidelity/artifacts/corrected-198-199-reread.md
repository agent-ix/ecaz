# Corrected reread of committed Task 198/199 artifacts

The committed artifacts show why the old “unchanged storage” conclusion was
not an arm-fidelity result:

| packet | scale | arm | old `physical_generation_bytes` | replica relation in `results.jsonl` |
| --- | --- | --- | ---: | --- |
| Task 198 `005-full-scale-decision` | 100k | owner-control | 2,496,659,456 | absent |
| Task 198 `005-full-scale-decision` | 100k | coordinator-replica | 2,496,659,456 | absent |
| Task 199 `003-release-matrix-and-decision` | 100k | owner-control | 3,188,056,064 | absent |
| Task 199 `003-release-matrix-and-decision` | 100k | coordinator-replica | 3,188,056,064 | absent |

Sources:

- `reviews/task-198/005-full-scale-decision/artifacts/lifecycle-100k/results.jsonl:158-161`
- `reviews/task-199/003-release-matrix-and-decision/artifacts/run/results.jsonl:53-56`
- `reviews/task-199/003-release-matrix-and-decision/artifacts/suite-report-r25.md:203-206`

The reviewer record for Task 199 reports the coordinator-side replica relation
as `relation_bytes=1,659,518,976` (and WAL bytes up to `1,937,700,656`), but that
measurement was log-only rather than a `results.jsonl` storage row:
`reviews/task-199/003-release-matrix-and-decision/feedback/2026-07-25-01-reviewer.md:186-205`.

Therefore the corrected accounting is not “the arms use equal storage.” It is:

1. the old storage emitter duplicated the same generation scalar across arms;
2. the replica relation was not included in the structured result rows; and
3. the old rows cannot establish the NFR-018 summed ratio or the NFR-021
   per-node maximum.

Task 204 fixes the measurement path. A fresh two-arm 100k run is still required
to populate this packet with the new per-arm relation, ratio, and per-node rows.
