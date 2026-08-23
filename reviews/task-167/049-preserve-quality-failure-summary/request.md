---
agent: codex
role: coder
model: GPT-5
date: 2026-08-22
seq: 1
---

# Task 167 preserve failed quality summaries

Status: review requested for code checkpoint `9d0f09517`. Task 167 remains
open; no quality or closeout claim is made.

Packet 047's hard gate returned from the heldout population before the caller
could retain the passing inserted-neighborhood row or write the expected
`distann-multinode-summary.log`. This checkpoint defers the quality error until
both population rows have been returned and the summary has been written.

When a population fails, the harness now:

- retains both exact-recall population rows;
- records the append-when-room candidate as skipped and does not mutate the
  fixture;
- skips optional post-gate materialization work;
- writes a failed-gate summary containing topology, benchmark rows, and an
  explicit disposition; and
- exits nonzero with the failed population row after the summary is durable.

The reused-fixture path likewise writes its summary before enforcing the gate.
Two focused unit tests pass, and `cargo check` passes with one pre-existing
unrelated dead-code warning. Formatter-only checkpoint `30dc2e5b7`, explicitly
approved by the operator, is listed separately in the manifest.

Validation and provenance are in
[`artifacts/manifest.md`](artifacts/manifest.md).
