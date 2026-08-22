---
task: 233
packet: 001-plan
agent: Codex
role: coder
model: gpt-5
date: 2026-08-22
seq: 01
---

# Task 233 hybrid node/columnar generation plan

This packet requests review of Task 233 at planning checkpoint `75eae179f`.

Task 233 is the mandatory integration experiment after the four isolated
layout prototypes. It combines a Task-231 fixed-stride graph/vector extent with
Task-232 packed non-vector payload columns over one owner-local dense ordinal.
The exact vector is authoritative only in the node extent; vector projection
reads that surface, while Task 222's mask selects only required payload
segments. The public table remains a normal PostgreSQL heap and all private
generation storage remains relation/WAL managed.

The evidence gate is a same-surface four-arm graph-layout × payload-layout
factorial at 10k, 50k, and 100k, including warm and controlled-residency
profiles and traversal, narrow, vector-bearing, mixed, cold/wide, and whole-row
queries. The task runs even if either constituent closes STOP and reports main
effects separately from the interaction. It also reconciles Tasks 229 and 230
before selecting a final storage disposition; any default flip is a separate
productionization task.

Please review the single-authority format, common ordinal/directory contract,
base-plus-delta lifecycle, PostgreSQL storage boundary, factorial validity,
mandatory-negative policy, and promotion rule.

This is planning-only. No code, test, or benchmark result is under review.
