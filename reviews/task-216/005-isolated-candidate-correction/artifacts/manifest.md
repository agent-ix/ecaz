# Task 216 packet 005 correction manifest

- Packet: `reviews/task-216/005-isolated-candidate-correction/`
- Status: decision-only correction; no new measurement
- Diagnostic audit: `artifacts/diagnostic.md` records the reproducible suite
  arm, source GUC semantics, stage-gated instrumentation, and cited counter
  lines
- Source measurement: accepted Task 216 isolated 100k packet on local ref
  `task-216-mat15-isolated`, control/candidate source commits
  `e8f15ab0c68887c176a260107fe826c402c2f827` /
  `6662b302f8370695320dcb36edda3cd291c8c1bc`
- Arm control: `materialization_batch_size=0` (explicit eager control), not
  production lazy-10
- Build provenance: cargo `release` profile with
  `distann-head-attribution-benchmark`; the `distann-stage-counters` lines
  are feature instrumentation and make the absolute latency feature-build
  diagnostic rather than a featureless normal-release point
- Control physical 100k latency: mean/p50/p95/p99
  `40.60/39.30/54.30/57.20 ms`
- MAT-15 candidate physical 100k latency: mean/p50/p95/p99
  `86.10/85.00/113.70/127.00 ms`
- Coordinator decode: `0.076 -> 0.096 ms/scan`; control ceiling
  `0.076/40.60 = 0.19%`
- Owner payload SQL: `40.376 -> 118.422 ms` owner-summed; this is the
  candidate implementation's secondary regression, not the MAT-15 addressable
  coordinator stage
- Returned payload bytes: `576,576 -> 576,945 B`, effectively flat
- Ordered identity: physical predictions differed in `2/200` rows because the
  arms rebuilt different generations; no ordered-identity claim is made
- Fault coverage: explicitly skipped in the registered diagnostic
- Decision: `STOP`; no packet-003 full-scale matrix and no MAT-21 A/B until the
  same-generation lane is corrected
