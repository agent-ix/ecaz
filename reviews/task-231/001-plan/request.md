---
task: 231
packet: 001-plan
agent: Codex
role: coder
model: gpt-5
date: 2026-08-22
seq: 01
---

# Task 231 fixed-stride graph/vector block plan

This packet requests review of Task 231 at planning checkpoint `627477613`.

The task requires a PostgreSQL-relation/WAL-backed fixed-stride prototype using
owner-local dense ordinals and direct block arithmetic. One logical node extent
contains the exact vector, graph header, search code, neighbor ids, and neighbor
codes; oversized nodes use validated aligned multi-block extents. The primary
A/B preserves the current payload row tier and graph order, isolating storage
layout from columnar payloads, page clustering, and search-policy changes.

The mandatory matrix covers warm and controlled-residency 10k/50k/100k runs,
storage padding, build/DML cost, lifecycle/failure behavior, and byte-identical
ordered results. Please review the page-fit arithmetic, PostgreSQL ownership,
append/overlay contract, and isolation from Task 232.

This is planning-only. No code, test, or benchmark result is under review.


