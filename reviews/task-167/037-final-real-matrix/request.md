---
agent: codex
role: coder
model: GPT-5
date: 2026-08-21
seq: 1
---

# Task 167 final real-corpus matrix

Status: review-open; first 10k attempt exposed a production/config
incompatibility, remediation pending.

This packet owns the final Task 167 production 10k/50k/100k evidence at code
checkpoint `5568aba17`. The release CLI and PG18 extension were built from a
clean worktree at that checkpoint; packet 036 records their build logs, hashes,
and passing warning-free synthetic gate.

The immutable suite configuration is
[`../032-recovery-runtime/artifacts/task167-recovery-suite.json`](../032-recovery-runtime/artifacts/task167-recovery-suite.json).
The 10k step runs first as a gate. The 50k and 100k steps may run only after
10k succeeds. Every selected real step includes physical concurrency,
insert-then-query parity, append A/B, recall, latency, and storage measurements.

The first 10k attempt passed clean production provenance, three-owner build,
published topology, serving, and both remote-owner proofs. Its recall child
also completed. The following latency child failed because the preregistered
`distann_stage_counters=true` switch calls the benchmark-feature-only
`ec_distann_stage_scoring_snapshot()` function, which is intentionally absent
from the required production `pg18` build.

This is not a 10k result: concurrency, parity, append A/B, latency, and storage
did not complete, and 50k/100k remain blocked. The remediation will decouple
production insert-work counters (required by Task 167) from query-stage timing
instrumentation (benchmark-feature-only), preregister a corrected suite config,
and rerun 10k from a fresh cluster.
