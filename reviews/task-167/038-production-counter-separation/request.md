---
agent: codex
role: coder
model: GPT-5
date: 2026-08-21
seq: 1
---

# Task 167 production counter separation

Status: review-open; clean production runtime built and corrected real-corpus
matrix pending.

Please review CLI checkpoint `a49ffd92a`.

Packet 037's first 10k attempt showed that the fixture's
`distann_stage_counters` switch coupled two different surfaces: optional
query-stage timing instrumentation and Task 167's production insert-work
counters. The former requires the
`distann-head-attribution-benchmark` extension feature; the latter is part of
the production `pg18` surface and is required for the append A/B evidence.

This checkpoint makes Task 167 insert-work reset, snapshot, sample-count, and
graph-degree-bound validation unconditional in the physical benchmark. The
query-stage switch now controls only query-stage instrumentation and receives
an early extension-feature preflight before any corpus build begins.

The focused Task 167 CLI tests pass. A clean release CLI and production-only
PG18 extension were built at exact runtime head `cce839647`; the corrected
three-step suite audit passes. No extension product code changed. The
10k/50k/100k matrix remains required before task closeout.
