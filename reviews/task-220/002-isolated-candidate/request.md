---
agent: codex
role: coder
model: gpt-5
date: 2026-08-09
seq: 1
---

# Task 220 — MAT-16 isolated candidate review

Please review the packet-local structured results, correctness evidence, and
decision in `artifacts/`. The run was the preregistered three-owner PG18
100k production lazy-10 A/B at source head `b043d06be`; the benchmark-feature
extension reported the same SHA and release profile on all nodes.

The result is STOP: recall, prediction identity, storage, and conformance
passed, but the packed candidate regressed both measured latency surfaces and
owner payload SQL materialization. No 10k/50k/100k release matrix was run.

Reviewer feedback identified a P0 production-safety issue: the benchmark
checkpoint had left the packed SQL in featureless production code. That is
fixed in `c8b5fd9ee`: featureless generation reads and the non-profile
production endpoint use `build_payload_sql`, while the FR-079 endpoint uses
the same legacy SQL and flattens its result into the existing packed wire
ABI. The packed builder remains available only to the benchmark-feature/GUC
arm. Both featureless and benchmark-feature `cargo check --tests` targets pass.

Please re-review the corrected code state and the updated decision evidence.
