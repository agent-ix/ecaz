# Task 239 packet 001 reproduction decision

## Disposition

**REPRODUCED — EAGER-PATH COUNTER SHAPE ON EXACT CURRENT MAIN.**

The single authorized attribution run failed at the preregistered rule-1 gate:

```text
materialization correctness scenario exactly_one_window failed:
rows=10/10 identity=true null_ok=true external_toast_ok=true
remote_requested=8 local_consumed=4 payload_reads=12/10
duplicate_requested=0
```

This is not a production lazy-10 regression finding. Exact main's semantic
harness sets its control to batch size 0 and fails to restore the candidate's
production default, so this failure is an eager-path observation.

## Lane 1 — featureless production gate

- Suite status: `succeeded`, exit 0.
- Runner/extension SHA: exact
  `41392c011106cb040095fd6004c4d5c0f136f1a0`.
- Extension profile/features: `release`, `pg18`; three nodes unanimous;
  `pg-test` absent.
- Seven core scenarios: exactly once each, all correct and byte-identical,
  `attribution_available=false`.
- Feature isolation: `normal_release=true`,
  `attribution_hooks_absent=true`, `semantic_scenarios=7`.
- Recall: 0.9990 for both labels over 200 queries / 2,000 trials; as
  preregistered, these are two executions of the same featureless production
  configuration.
- Prediction files are byte-identical, SHA-256
  `801f6a0b83237047fea6ebd92cb1b85f07aa8dd80ee6dbd5c7877153e724fb6e`.
- No Task 167 quality-gate skip. Routed DELETE+VACUUM passed.

## Lane 2 — attribution reproduction

- Suite status: `failed`, exit 1 at the single expected rule-1 gate; no resume
  or replacement run was attempted.
- Runner/extension SHA: exact `41392c011...`.
- Extension profile/features: `release`,
  `distann-head-attribution-benchmark,pg18`; three nodes unanimous;
  `pg-test` absent.
- The nominal semantic candidate reproduced 8 remote + 4 local = 12/10 with
  correct/identical ten rows and zero duplicate requests.
- Both independently spawned recall arms completed at 0.9990 over 200 queries /
  2,000 trials, and their prediction files are byte-identical to each other and
  to lane 1 (same SHA-256 above).

The independently spawned latency arms do not share the semantic harness's
session leak. Their counters provide the packet's direct production-path
diagnostic context:

| Child arm | Remote requested | Local consumed | Total requested reads | Duplicates |
| --- | ---: | ---: | ---: | ---: |
| eager control (`batch_size=0`) | 27 | 4 | 31 | 0 |
| production lazy-10 (default `-1`) | 6 | 4 | 10 | 0 |

The production lazy-10 child therefore records exactly ten requested/consumed
rows on the same attribution fixture and returns ten client rows. It is direct
diagnostic context, not the corrected semantic gate. Its one-iteration latency
and the eager/lazy timing delta have no decision weight.

## Packet 002 consequence

Packet 002 must fix the semantic harness by explicitly restoring/observing the
candidate batch-size state, prove the corrected nine-scenario matrix on exact
main, and reconcile the accepted Task 191/198 10/10 evidence. Because exact
main reproduced the eager-path 12/10 shape, preregistered rule 2's direct
Task-224-delta comparison is not the first branch; source comparison can remain
diagnostic after the harness correction.

No bound is widened, no production runtime is changed, and packet 001 does not
close Task 239. The 10k/50k/100k A/B rule remains untriggered unless packet 002
changes production scan, rerank, posting, payload, or storage behavior.
