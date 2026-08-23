---
agent: codex
role: coder
model: GPT-5
date: 2026-08-22
seq: 1
---

# Task 167 shipped-quality gate isolation

Status: code checkpoint review-open at `c3b01290b`; focused static validation
passes. Runtime 50k disposition remains open and no Task 167 closeout is
claimed.

## Review request

Please review the Task 167 benchmark-harness correction in
`crates/ecaz-cli/src/commands/dev/distann_multicluster.rs`.

Packet 045 proved that the prior 50k heldout deficit is real, but also exposed
an arm-order confound: exact recall was measured only after the physical graph
had received 160 shipped robust-prune inserts and another 160 inserts with the
rejected append-when-room candidate enabled. The candidate therefore mutated
the same graph used to judge shipped behavior.

This checkpoint changes the order and gate semantics:

1. measure the single-index control and 160 physical inserts using the shipped
   robust-prune default;
2. snapshot and report insert-work counters for that shipped arm;
3. build the fresh comparator and run exact recall against the graph containing
   only those 160 shipped inserts;
4. enforce packet 045's population-specific calibrated bands as hard gates:
   `0.015` for inserted-neighborhood and `0.007` for heldout;
5. only after the quality gate passes, enable append-when-room and mutate the
   disposable fixture for its separately labeled timing diagnostic.

The exact-truth corpus now receives only the inserted ID ranges present at the
quality checkpoint. Output attests the graph phase, gate provenance, candidate
exclusion, and shipped insert mode. A failed population now fails the command
instead of merely reporting a non-blocking reference value.

## Validation

- `cargo test -p ecaz-cli task167_ --no-default-features`: passed, 10 tests.
- `cargo check -p ecaz-cli`: passed with the pre-existing unrelated dead-code
  warning at `commands/corpus/load.rs:190`.

See [`artifacts/manifest.md`](artifacts/manifest.md) for commands and hashes.
The next packet will use `ecaz bench suite` to rerun an isolated 50k step at
this exact runtime. If the shipped-only graph still exceeds `0.007`, the task
remains open for insertion-algorithm diagnosis.
