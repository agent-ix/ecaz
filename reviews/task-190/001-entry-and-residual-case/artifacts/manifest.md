# Task 190 packet 001 artifact manifest

Date: 2026-07-23 (America/Los_Angeles)

This packet introduces no benchmark run. It derives the architecture entry
case from accepted immutable packets already on `main`.

## Retained production Pareto point

Source:
`reviews/task-195/002-release-matrix/artifacts/manifest.md` and its accepted
feedback.

| Scale | Distinct recall | Warm mean / p95 ms | Physical generation bytes |
|---|---:|---:|---:|
| 10k | 0.9990 | 20.90 / 25.50 | 242,745,344 |
| 50k | 0.9685 | 20.90 / 24.30 | 1,242,750,976 |
| 100k | 0.9625 | 19.90 / 23.30 | 2,496,626,688 |

Lane: local Intel, PG18, three exact/disjoint physical owners, one index per
table, trained exact cap-4,096 head, 32 seeds, degree 32, BW4/H100, RaBitQ
neighbor values, exact final ranking, production lazy10 and owner-schema cache.
The A/B used 200 recall queries / 2,000 trials and 50 warm samples after 10
warmups. Release provenance and all topology gates passed.

## Residual attribution

Sources:

- `reviews/task-194/008-nine-way-completion-audit/artifacts/manifest.md`;
- packet 008 `run/results.jsonl` and compact summary; and
- accepted feedback
  `reviews/task-194/008-nine-way-completion-audit/feedback/2026-07-22-01-reviewer.md`.

The fully instrumented 100k scan recorded:

- traversal total 9.065098 ms/scan;
- remote expansion 7.429284;
- owner service 2.258880, including graph read 1.200080 and scoring 0.894481;
- transport wait 5.012911;
- connection + request encode + receive/decode 0.070834;
- ten sequential rounds, 40 nodes requested and returned, zero repeats; and
- 13,871.92 logical request bytes and 10,530.32 logical response bytes.

Remote and traversal decomposition errors were 1.17% and 1.32%, below their
5% and 10% hard gates. The observer-heavy run's 27.70 ms wall mean is not used
as the production baseline. Packet 006's lighter telemetry recorded
4.078199 ms transport wait and 23.50 ms wall mean. The accepted reviewer
explicitly requires this observer distinction.

## Narrower negative result

Source: `reviews/task-194/007-fixed-work-candidate/`.

On one shared 100k generation, BW8/H50 versus BW4/H100:

- recall 0.9625 to 0.9675;
- hops 10.0 to 5.88;
- traversal 7.685 to 7.082 ms;
- transport wait 4.180 to 3.435 ms;
- nodes 40.0 to 47.04;
- warm mean 24.30 to 24.20 ms; and
- p95 27.80 to 28.30 ms.

It was correctly rejected: a local stage win did not improve end-to-end
latency usefully and regressed the tail.

## Scope boundary

Task 190 was operator-activated for this latency case after Tasks 194--197.
It does not claim Tasks 185/186/188/189 complete, select a recall policy, or
close their independent head/graph/codec branches. Those branches cannot
remove the measured ten-round transport wait while preserving the architecture
premise. Task 189's unchanged full exact-neighbor path is already a measured
slower/lower-recall negative from Task 183.

No 1m measurement was triggered because Task 194's isolated 100k candidate
failed the pre-registered usefulness gate. This is a conditional skip.

## Entry decision

The directly measured, architecture-addressable opportunity is roughly
4.1--5.0 ms/scan of serial transport wait, or 20--25% of the retained 19.90 ms
100k mean. That is material enough to compare at most two architecture
families, but not to implement either inside Task 190.
