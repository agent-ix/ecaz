---
agent: codex
role: coder
model: GPT-5
date: 2026-08-21
seq: 1
---

# Task 167 rollback-vector normalization

Status: review-open; final exact-head runtime pending.

Please review harness checkpoint `caa8ad63f`.

Packet 035's clean-head synthetic suite passed every gate, but PostgreSQL
correctly warned that the separate `mi` mid-insert rollback subfixture still
built unnormalized vectors under `ecvector_distann_ip_ops`. The main synthetic
physical fixture and concurrency drill were already normalized.

This narrow follow-up routes the isolated rollback fixture's initial 500 rows,
injected-failure row, and replacement UPDATE through the same deterministic
unit-vector expression. Source arrays and encoded embeddings continue to use
the identical expression in each write.

The focused Task 167 CLI tests pass. The production extension code is
unchanged, but final runtime provenance will be rebuilt at this packet head
before the synthetic confirmation and 10k/50k/100k matrix.

