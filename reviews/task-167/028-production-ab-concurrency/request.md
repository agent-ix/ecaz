---
agent: codex
role: coder
model: GPT-5
date: 2026-08-15
seq: 1
---

# Task 167 production A/B and concurrency follow-up

Status: review-open; not merge-ready.

This checkpoint addresses a remaining production-path gap from reviewer
feedback `reviews/task-167/026-owner-retry/feedback/2026-08-15-01-reviewer.md`.
The append-when-room userset GUC was already registered for production and
honored by `physical_dml`, but the two remote owner-session propagation blocks
were still compiled only for `pg_test`. Removing those gates makes the A/B
toggle reach the owner transaction in a production-feature build.

The suite config in packet 026 now also enables the 50k and 100k concurrency
drills, with fresh external run-directory names. The config remains bespoke
because the standard current Intel lane has no three-owner physical-DML and
owner-retry fixture; this lane isolates the Task 167 physical 10k/50k/100k
corpora, production owner routing, retry attribution, saturation, and append
A/B behavior.

Validation:

- `CARGO_TARGET_DIR=/home/peter/dev/ecaz/target cargo check --no-default-features --features pg18` passed.
- `CARGO_TARGET_DIR=/home/peter/dev/ecaz/target cargo check --no-default-features --features pg18,pg_test` passed.
- Runtime preflight and the matrix remain outstanding: the required external
  cluster root is mounted read-only on this host, so fixture initialization
  fails before PostgreSQL starts.

No merge or task closeout is requested. The production-feature install and
the complete 10k/50k/100k suite must still run on a writable external cluster
root, and the unforced retry, liveness, exact-degree saturation, recall, and
latency gates must pass before closeout.
