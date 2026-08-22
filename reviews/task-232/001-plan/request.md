---
task: 232
packet: 001-plan
agent: Codex
role: coder
model: gpt-5
date: 2026-08-22
seq: 01
---

# Task 232 packed columnar row-tier plan

This packet requests review of Task 232 at planning checkpoint `627477613`.

The task is deliberately last and requires an opt-in immutable per-attnum
columnar prototype: fixed-width arrays, null maps, bounded offset/value segments
for variable-width attributes, a dedicated exact-vector segment, and canonical
build-time binary encoding. Published-base DML uses a transactional row-heap
overlay that is compacted by the next epoch rebuild. Segment publication and
recovery are atomic at the generation boundary.

The primary 10k/50k/100k A/B remains unstacked against the row-heap control and
covers id-only, scalar, vector-bearing, mixed, and whole-row workloads. Prior
layout winners appear only as secondary comparisons. Please review the segment
format, type-I/O/schema identity, overflow/corruption envelope, overlay
semantics, and promotion rule.

This is planning-only. No code, test, or benchmark result is under review.
