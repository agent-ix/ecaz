---
task: 184
packet: 001-materialization-plan
role: coder
status: open
date: 2026-07-19
head: eafcb6bae
---

# Review request: Task 184 materialization attribution plan

This packet freezes Task 183's retained 100k production-path baseline and the
Task 184 attribution/selection contract before counter implementation.

## Frozen baseline

- three exact/disjoint physical owners, graph degree 32;
- production `training_landmarks_exact`, cap 4,096, exact head scoring,
  32 seeds, BW4/H100, normal RaBitQ neighbors, exact final ranking;
- evaluation rows 1--200 and training rows 201--400 from the attested staged
  100k query file;
- 200 recall queries / 2,000 distinct top-10 trials;
- 50 timed latency queries after 10 same-connection warmups, concurrency 1;
- Task 183 installed release head `97cd5a76a5ea2d20ef94078566f66f85dacc97b2`;
- recall 0.9625 and warm mean/p50/p95/p99/max
  40.20/39.20/51.50/56.30/57.90 ms; and
- remote materialization 26.955 ms/query of 40.20 ms wall mean.

Task 184 builds and measures a fresh generation; the Task 183 numbers are an
immutable historical anchor, not a substitute for the new profile.

## Attribution contract

`artifacts/materialization-attribution-contract.md` defines the timer nesting,
work counters, derived residuals, feature gates, reset boundary, and required
result rows. The independent coordinator partition is:

1. candidate partition/request preparation;
2. connection and prepared-statement readiness;
3. concurrent owner request wait;
4. coordinator row-to-payload decode;
5. response validation/result-map insertion; and
6. ranked payload association/output installation.

Owner endpoint timers are nested inside request wait. A benchmark-only owner
telemetry response will make owner lookup/payload SQL visible; the per-owner
round-trip minus owner endpoint time is reported as the remaining PostgreSQL
row encoding, transport, and client protocol residual rather than falsely
claiming those components are individually separated.

Work counters report candidates ranked, remote rows requested/returned,
owners, payload columns/bytes, payloads installed, and total/local/remote rows
actually yielded by the CustomScan to the executor. The latter is the relevant
consumption boundary because executor quals run above the access callback.

## Candidate-selection contract

No behavior candidate is selected in this packet. Packet 002 may pre-register
exactly one isolated candidate only after the fresh 100k results identify its
target. Selection order is:

1. demonstrated end-to-end ceiling from the measured stage/work share;
2. preservation of projection/qual/identity/failure semantics;
3. bounded work with an explicit worst-case cap;
4. smallest durable format/protocol/lifecycle impact; and
5. no stacked candidate families.

If eager remote rows materially exceed executor-consumed rows, bounded lazy
materialization (`MAT-01`/`MAT-04`) is eligible. If owner setup/lookup dominates,
one owner-side reuse/lookup candidate is eligible. If coordinator copy/map work
dominates, one ownership/layout candidate is eligible. Otherwise STOP.

Please review timer non-overlap/nesting, counter sufficiency, fixture identity,
and the post-attribution selection discipline.

## Validation disposition

No tests or benchmark were run for this plan-only checkpoint. The canonical
staged corpora, release/debug CLI binaries, PG18 installation, and Task 183
artifacts were inspected and are available. Implementation and fresh evidence
belong to packet 002.
