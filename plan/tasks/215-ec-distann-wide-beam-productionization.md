# Task 215: ec_distann Wide-Beam Traversal Productionization

Status: **proposed — entry gated on Task 205 closeout** (2026-08-06).
Priority: P1 latency/default productionization.

Program ledger: `plan/design/ec-distann-recall-latency-roadmap.md`.

## Why

Task 206 is review-closed and measured the paper-shaped wide-beam/few-round
regime on the conforming sharded owner path. The strongest measured point is
`beam_width=64`, `hop_rounds=8`, with the production derivation selecting
`head_seed_count=128`; the seed-count requalification found no recall benefit
from 200 seeds at that width. The current shipped default remains
`beam_width=4`, `hop_rounds=100`, so the measured recommendation has not yet
been subjected to a normal-build default-change gate.

Task 206 was intentionally a measurement/recommendation task. A separate
productionization task is required before changing defaults, preserving the
rule that a benchmark or feature-build win is not itself a release decision.

## Entry gate

Do not change the shipped default until all of the following are available:

1. Task 205's corrected bounded-L packet has a final outside-review
   disposition, including the requested threshold-versus-limit attribution and
   benchmark/provenance cleanup. The Task 205 implementation may remain the
   current `candidate_heap_limit=32` path; this task does not reopen its
   algorithmic design.
2. Task 206's review-closed packets are cited, including the corrected
   effective-seed-count evidence and the scan-round attribution note.
3. The control is the current conforming sharded owner-traversal release:
   `beam_width=4`, `hop_rounds=100`, `candidate_heap_limit=32`, the current
   production head/materialization behavior, and no coordinator full-graph
   replica.
4. Task 208/210 conformance evidence is available for the exact topology used
   by the release matrix, including per-node storage and owner engagement.

If Task 205 does not close, this task may prepare configuration and release
checks but must not claim a production recommendation.

## Goal

Run a normal PG18 release A/B and, only if it passes, make the measured
wide-beam regime an operator-approved production default. The candidate arm is
the reviewed Task 206 point:

- `beam_width=64`;
- `hop_rounds=8`;
- effective `head_seed_count=128`;
- `candidate_heap_limit=32` and Task 205 pushdown enabled;
- the current sharded owner-traversal path;
- no traversal replica or benchmark-only selector.

The final implementation must retain a clear rollback path to BW4/H100 and
must not make the candidate dependent on attribution-only GUCs or feature
builds.

## Phases

1. **Release contract.** Record the exact default changes, compatibility
   behavior for existing sessions/indexes, bounded-work implications, and
   rollback/operator procedure. Amend an ADR only if the default changes a
   durable lifecycle or compatibility contract.
2. **Production wiring.** Move only the reviewed BW/H regime into the normal
   release configuration. Keep benchmark axes, seed-count overrides, scan
   notices, and feature-build selectors out of the production path.
3. **Release A/B.** Use `ecaz bench suite` on fresh, byte-identified physical
   generations at 10k/50k/100k. Compare unchanged production against the
   candidate with recall, CI, p50/p95/p99/max latency, hop rounds, expanded
   work, request/response bytes, storage, owner engagement, and conformance.
4. **Decision.** Promote only if the candidate preserves ordered results and
   recall, remains NFR-021/NFR-022 conforming, and provides a material
   end-to-end latency or Pareto improvement. Otherwise record STOP and leave
   BW4/H100 shipped.

## Benchmark gate

All matrices use a checked-in `ecaz bench suite` configuration and a normal
PG18 release binary. The candidate must report:

- exact or statistically equivalent recall at all three scales;
- mean, p50, p95, p99, and max latency;
- hop count, per-round transport/straggler counters where available, and
  expanded-node/frontier work;
- request and response bytes separately;
- per-node storage and normalized NFR-021 growth;
- physical owner engagement and no coordinator full-graph substitution; and
- unanimous release SHA/profile provenance.

Instrumentation-only rows may explain the mechanism but cannot substitute for
the uninstrumented release decision rows.

## Non-goals

- Reopening Task 206's regime sweep or Task 207's head-construction decision.
- Changing head selection; that belongs to Task 185.
- Adding degraded completion or hedging; that belongs to Task 209.
- Combining a new owner serialization optimization with the BW/H default A/B;
  Task 216 owns that latency family.
- Reviving the inadmissible coordinator full-graph replica.

## Required review packets

1. `reviews/task-215/001-release-contract/`;
2. `reviews/task-215/002-production-wiring/`;
3. `reviews/task-215/003-release-matrix-and-decision/`.

## References

- `plan/tasks/205-ec-distann-expansion-pushdown.md`;
- `plan/tasks/206-ec-distann-traversal-regime.md`;
- `reviews/task-206/006-re-review-corrections/`;
- `reviews/task-206/007-scan-round-capture/`;
- `plan/tasks/208-ec-distann-conformance-gates.md`;
- `plan/tasks/210-ec-distann-distribution-restoration.md`;
- `spec/non-functional/NFR-021-distann-distribution-invariant.md`; and
- `spec/non-functional/NFR-022-distann-control-validity.md`.
