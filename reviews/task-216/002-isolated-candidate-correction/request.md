---
task: 216
packet: 002-isolated-candidate-correction
agent: Codex
role: coder
model: gpt-5
date: 2026-08-07
seq: 01
---

# Task 216 MAT-15 isolated-candidate correction

This is a decision-only correction to the isolated MAT-15 record. It adds no
benchmark run and does not authorize the 10k/50k/100k matrix or MAT-21. The
source measurement is the accepted isolated 100k evidence on the local
`task-216-mat15-isolated` ref; this packet makes its arm mode and decision
rationale explicit on the current branch.

## Corrected interpretation

The captured arm used `materialization_batch_size=0`, which is the explicit
eager control. It did not measure the production lazy-10 path. The stage
counters also prove that the cargo release-profile binary carried the
`distann-head-attribution-benchmark` instrumentation feature; therefore its
absolute latency is a feature-build diagnostic, not a normal featureless
release-latency point.

The MAT-15 STOP remains correct, but for a different reason than the SQL
regression. The control measured coordinator decode at 0.076 ms against a
40.60 ms scan, a maximum addressable share of 0.19%, while returned payload
bytes were effectively flat (576,576 B versus 576,945 B). The candidate's
owner-SQL regression is secondary evidence about that implementation; it is
not evidence that packed coordinator buffers can move the dominant stage.

The two arms also rebuilt separate generations, so their physical prediction
difference cannot support ordered-identity attribution. Future MAT-21 work is
blocked on a build-once/swap-extension or equivalent same-generation lane.
Fault drills were skipped by preregistration, and the malformed-payload claim
is not treated as complete protocol coverage by this correction packet.

## Decision

**STOP MAT-15.** No full-scale matrix or productionization is authorized.
MAT-16 and MAT-21 remain owner-side stage candidates and are not retired by
the coordinator ceiling; MAT-22 remains an owner expansion/wire candidate.
