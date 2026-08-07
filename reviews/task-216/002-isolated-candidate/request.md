---
task: 216
packet: 002-isolated-candidate
agent: coder
role: coder
model: gpt-5
date: 2026-08-06
seq: 01
---

# Task 216 — MAT-15 isolated candidate

This packet pre-registers the one candidate advanced by packet 001: MAT-15,
an owner-side packed payload representation consisting of a null bitmap, a
monotone cumulative-offset array, and one flat byte buffer. The current
representation is the control: one `bytea` value per projected payload column.

The implementation will change only the remote row-payload transport and its
decode path. It will not change graph traversal, quantization, beam width,
head policy, L semantics, reranking, projection selection, or Task 215
defaults. MAT-21 is not stacked with MAT-15 and is not measured in this
packet.

## Measurement plan

Run the checked-in SuiteConfig at fresh physical 100k first, once at the
control commit and once at the MAT-15 commit. Both runs use the same normal
PG18 release configuration, corpus/query identity, three-owner topology,
persisted-head seed policy, 50 timed iterations, and 10 warmups. Each run gets
a distinct packet-local artifact directory and an external cluster directory;
the cluster is removed after its cited artifacts are captured.

The candidate stops here if it does not improve end-to-end latency or the
owner materialization stage/tails, or if any recall/result-identity,
topology, failure-semantics, storage, or protocol invariant changes. A useful
100k result authorizes packet 003's full 10k/50k/100k decision matrix; it does
not itself promote the representation.

The evidence must include recall and confidence interval, ordered result
identity, latency mean/p50/p95/p99/max, stage counters, allocations/copies,
request/response bytes, storage/topology gates, failure drills where
measurable, and build/corpus/query provenance. A malformed packed payload is
a hard error; there is no fallback to the old representation.

## Decision rule

MAT-15 can advance only if the comparison is attributable to this isolated
representation change and all hard invariants pass. A latency win without
recall/result identity and protocol proof is a rejection, not a promotion.

## Isolated result

The control and candidate completed the registered physical 100k suite on
release PG18 builds. MAT-15 was rejected: physical mean latency increased
from 40.60 ms to 86.10 ms, p95 from 54.30 ms to 113.70 ms, and p99 from
57.20 ms to 127.00 ms. Recall was 0.9275 versus 0.9295 and storage was
effectively unchanged (1.351173 versus 1.351160 amplification). The physical
prediction files differed in 2 of 200 ordered query rows, while the single
prediction files matched. The physical seed digests differed between runs,
so that identity discrepancy is recorded as a hard reproducibility gate and
not claimed as a candidate-caused regression. No packet-003 full-scale run is
authorized under the preregistered usefulness rule.
