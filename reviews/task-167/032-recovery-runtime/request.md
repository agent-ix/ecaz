---
agent: codex
role: coder
model: GPT-5
date: 2026-08-21
seq: 1
---

# Task 167 current-head runtime recovery

Status: preregistered; review-open; no runtime result claimed yet.

This packet preregisters the current-head PG18 recovery lane that supersedes
the diagnostic matrix in packet 026. The real-corpus steps use the shipped
DistANN reloption defaults (`graph_degree=32`, `head_index_cap=4096`) at
10k/50k/100k. A separate degree-8 synthetic step isolates the natural 2PC
retry, concurrency, controlled-backlink, and exact-saturation gates.

The bespoke suite is required because the standard Intel lane has no
three-owner physical-DML/owner-retry fixture. The configuration uses only
`ecaz bench suite`; it is checked in at
[`artifacts/task167-recovery-suite.json`](artifacts/task167-recovery-suite.json).

Execution is fail-fast: the synthetic step and production-default 10k step run
as diagnostics first. Only after both pass will one fresh complete
synthetic/10k/50k/100k suite be accepted as closeout evidence. Results will be
added to this packet with exact extension SHA/profile/features provenance.

