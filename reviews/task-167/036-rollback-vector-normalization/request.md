---
agent: codex
role: coder
model: GPT-5
date: 2026-08-21
seq: 1
---

# Task 167 rollback-vector normalization

Status: review-open; final clean-head synthetic gate passed, real matrix
pending.

Please review harness checkpoint `caa8ad63f`.

Packet 035's clean-head synthetic suite passed every gate, but PostgreSQL
correctly warned that the separate `mi` mid-insert rollback subfixture still
built unnormalized vectors under `ecvector_distann_ip_ops`. The main synthetic
physical fixture and concurrency drill were already normalized.

This narrow follow-up routes the isolated rollback fixture's initial 500 rows,
injected-failure row, and replacement UPDATE through the same deterministic
unit-vector expression. Source arrays and encoded embeddings continue to use
the identical expression in each write.

The focused Task 167 CLI tests pass. The final clean-head synthetic suite also
passes at `5568aba17` with the production `pg18` feature set. The rollback,
replacement, concurrency, natural-retry, saturation, routed-delete, and
topology gates are all green, and no unit-normalization warning remains.

The 10k/50k/100k real-corpus matrix is still pending and no task closeout is
requested by this packet alone.
