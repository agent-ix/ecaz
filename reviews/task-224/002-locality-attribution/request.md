---
task: 224
packet: 002-locality-attribution
agent: Codex
role: coder
model: gpt-5
date: 2026-08-25
seq: 01
---

# Task 224 locality attribution and MAT-26 GO request

This packet requests outside review of the Task 224 attribution implementation,
100k measurement, and single-candidate decision. Packet 001's plan also still
awaits outside feedback; this request asks the reviewer to rule on that plan as
realized by packet 002 rather than leave the planning checkpoint dangling.

The implementation adds default-off, feature-only owner locality telemetry. It
measures TID/block dispersion, buffer observations, external TOAST and byte
counts, exact PostgreSQL binary-send time, the enclosing SPI payload span, and
post-SPI response construction. It also adds four suite-addressable projection
shapes: id-only, narrow scalar, vector-bearing, and a forced external-TOAST
payload with a matching predicate. The normal extension excludes the profiler
and retains the production payload SQL; focused featureless and feature tests
pin both sides.

The suite runner gained an explicit `skip_routed_delete_vacuum_drill` option so
later projection-only steps can reuse an exact generation without the standard
destructive DML drill changing its row count. This is opt-in, requires physical
benchmark mode, and is set on all four attribution steps. The final live suite
created one 100k fixture outside the repository, reused its exact epoch
fingerprint for all four shapes, and succeeded at the final release SHA. The
fixture was removed after evidence capture.

The preregistered result is a **MAT-26 GO, MAT-25 no-advance**:

- vector-bearing binary send is 6.967996 ms/scan against 28.20 ms warm mean,
  a 24.709206% ceiling; it passes both the 1 ms and 5% gates;
- toasted SPI-minus-send is a conservative MAT-25 ceiling of 1.617951 ms/scan
  against 46.10 ms, so it passes only the absolute gate at 3.509655%; and
- the plan requires advancing exactly one candidate when both pass, so MAT-26
  wins the percentage tie-break. The result remains the same if toasted work is
  normalized by end-user query rather than qualified custom-scan execution.

The binary-send timer sums owner work that can overlap, so 24.71% is an upper
bound, not a latency-win claim. The measurement authorizes only packet 003: one
isolated block-batched detoast/binary-send candidate and same-generation 100k
A/B. It does not authorize production behavior or the full-scale matrix. If
that candidate is not useful, Task 224 STOPs without packet 004; if useful, it
must then receive independent 10k/50k/100k recall, latency, storage, build, and
DML evidence.

Please verify specifically:

1. the feature gate and production SQL exclusion;
2. timer nesting and the `payload_spi - binary_send` conservative MAT-25 bound;
3. the exact-generation/projection-shape provenance and counter parsing;
4. the `skip_routed_delete_vacuum_drill` reuse invariant; and
5. the gate arithmetic and single-candidate tie-break.

Coder recommendation: **ACCEPT packet 001 as realized, ACCEPT packet 002,
authorize MAT-26 only for packet 003, and do not authorize MAT-25.**
