# Task 195: ec_distann Owner Schema Cache Productionization

> **MULTI-NODE MEASUREMENT RULE (NON-NEGOTIABLE).** Any decision about
> distributed behavior — latency, recall, storage, or overhead — MUST be measured
> on a multi-node configuration. A single-node / single-instance arm is NEVER
> acceptable as the basis for a decision about a distributed algorithm; its only
> permitted use is a clearly labeled baseline that quantifies distribution
> overhead. Label every reported number with its arm's node count. See
> AGENTS.md → "Distributed Measurement: Multi-Node Arms Only".

Status: **complete — outside-reviewed ACCEPT; PROMOTE** (2026-07-22).
Priority: P1. Promotes Task 192's measured `MAT-37`/`MAT-38` winner without
carrying benchmark controls into production.

Program ledger: `plan/design/ec-distann-recall-latency-roadmap.md`.

## Why

Task 192's feature-gated, backend-local retained-generation row-schema cache
preserved exact recall and storage at 10k/50k/100k while reducing warm mean
latency by 5.4 / 3.7 / 4.0 ms (21.9% / 15.7% / 16.9%). Owner open/validate
fell from 7.818 / 6.708 / 6.889 ms to 0.026 / 0.023 / 0.024 ms, while payload
SQL and all request/result work remained flat. Packet 006 separately proved
the epoch transition, retained predecessor, reclaim, and stale-fingerprint
failure contract on PG18.

## Goal

Make the validated immutable row-schema entry the normal physical-owner path,
then remove the Task 192 benchmark GUC and its extra wire/profile selector.
Normal requests must continue opening the exact retained row/graph/directory
relations and validating the generation fingerprint, descriptor schema
fingerprint, caller-expected fingerprint, projection attnums, and relation
availability. Only the repeated live catalog reconstruction is amortized.

## Hard constraints

- Preserve exact epoch fencing and FR-079 retained-generation semantics.
- Keep the backend-local cache bounded to four indexes and at most one
  fingerprint per index; relcache invalidation must cover the control index,
  row tier, graph store, directory, and global invalidation.
- No result, projection, wire payload, placement, storage-format, traversal,
  or materialization-window change.
- No production GUC or reloption for the cache; this is one normal behavior.
- Preserve normal builds without attribution instrumentation.

## Phases

1. Productionize the cached schema path and delete the benchmark selector from
   extension options, physical endpoint arguments, transport parameters, and
   suite variant encoding.
2. Keep and extend the PG18 multi-epoch lifecycle test as needed to cover the
   exact production call path, including successor publication, retained
   predecessor access, reclaim, and stale failure.
3. Run release-profile before/after evidence at 10k/50k/100k through
   `ecaz bench suite` (recall, latency, storage, 50/10 protocol). The promoted
   release must reproduce Task 192's causal stage movement and must not rely on
   the removed GUC.
4. Request outside review and merge only after the production binary, suite
   provenance, and feature-isolation audit agree.

## Measured outcome

Packet 002's release-profile, one-index-per-table A/B preserved exact recall at
0.9990 / 0.9685 / 0.9625 for 10k / 50k / 100k and matched all 78 compared
production materialization work metrics. Warm mean latency improved from
22.80 / 24.10 / 24.30 ms to 20.90 / 20.90 / 19.90 ms (8.33% / 13.28% /
18.11%), while owner open/validate fell from 7.030 / 6.792 / 7.122 ms to
0.028 / 0.024 / 0.024 ms. Storage varied by at most three PostgreSQL pages
(under 0.007%) across independent builds; the task changes no storage format.
The final installed normal PG18 release binary has no attribution endpoint,
removed selector, or neighboring benchmark controls. Outside review accepted
packets 001 and 002 with no blockers and marked the task merge-ready.

## Required review packets

1. `reviews/task-195/001-production-cache/`;
2. `reviews/task-195/002-release-matrix/`.

## References

- `reviews/task-192/005-paired-cache-ab/`;
- `reviews/task-192/006-epoch-safety/`;
- `reviews/task-192/007-full-scale-decision/`;
- ADR-085 D10/D12; FR-079; FR-082; NFR-019/NFR-020.
