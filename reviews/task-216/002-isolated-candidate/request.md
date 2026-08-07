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
measurable, and build/corpus/query provenance. Fault drills are explicitly
skipped in this first diagnostic suite (`skip_fault_drills: true`); the packet
must not imply outage-drill coverage. A malformed packed payload is a hard
error; there is no fallback to the old representation.

## Decision rule

MAT-15 can advance only if the comparison is attributable to this isolated
representation change and all hard invariants pass. A latency win without
recall/result identity and protocol proof is a rejection, not a promotion.

## Isolated result

The control and candidate completed the registered physical 100k suite on
release PG18 builds. The measured implementation regressed physical mean
latency from 40.60 ms to 86.10 ms, p95 from 54.30 ms to 113.70 ms, and p99
from 57.20 ms to 127.00 ms, but that is secondary evidence about this SQL
implementation, not the durable family-closing reason. The stage counters show
the regression is in owner payload SQL work (40.376 ms to 118.422 ms summed
over owners), while coordinator decode is only 0.076 ms to 0.096 ms per scan
and returned payload bytes are flat (576,576 to 576,945 bytes). MAT-15's
addressable ceiling on this profile is therefore 0.076 / 40.60 = 0.19% of
the control scan, with no wire-byte win to bank. The family is STOPped on that
ceiling; the slower SQL implementation is recorded only as a secondary
diagnostic.

The physical prediction files differed in 2 of 200 ordered query rows, while
the single prediction files matched. This is explained by the lane defect:
each arm built a fresh generation, producing different seed digests and
non-identical indexes. It is not attributed to MAT-15. Future MAT-21 or
successor A/B work must build once and swap only the extension binary, or pin
the drifting generation input, before asserting ordered identity. Candidate
screening must also include a maximum-addressable-win calculation before
advancing a stage-local hypothesis. No packet-003 full-scale run is
authorized under the preregistered usefulness rule.

The malformed-payload path is covered by packed-range unit tests: a middle
NULL preserves positional ranges, and negative, descending, out-of-bounds, or
length-mismatched offsets error. The three SQL-string tests remain separate
from that decode coverage.
