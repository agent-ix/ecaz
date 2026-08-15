---
agent: codex
role: coder
model: GPT-5
date: 2026-08-15
seq: 1
---

# Task 167 parity arm attribution follow-up

Status: review-open; not merge-ready.

The previous parity metric sampled only IDs `2,000,000..2,000,047`, which
belong to the append-disabled A/B control. The append-enabled candidate uses
the `3,000,000..` range, so the reported `inserted_neighborhood_recall` could
not identify which arm caused the low result. Checkpoint `71396d0e6` changes
the parity fixture to sample 24 rows from each arm, report
`append_disabled_recall`, `append_enabled_recall`, and their delta, and gate
closeout on the enabled candidate plus overall parity.

Validation:

- `CARGO_TARGET_DIR=/home/peter/dev/ecaz/target cargo check --no-default-features --features pg18` passed.
- `CARGO_TARGET_DIR=/home/peter/dev/ecaz/target cargo check --no-default-features --features pg18,pg_test` passed.
- Runtime preflight and the matrix remain outstanding because the required
  external cluster root is mounted read-only.

No merge or task closeout is requested. The production-feature rerun must
report both arm recalls, explain any remaining delta, and pass the enabled-arm
recall, unforced retry, liveness, saturation, latency, storage, and 10k/50k/100k
matrix gates.
