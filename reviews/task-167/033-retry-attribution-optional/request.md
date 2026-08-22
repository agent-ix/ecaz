---
agent: codex
role: coder
model: GPT-5
date: 2026-08-21
seq: 1
---

# Task 167 optional retry attribution

Status: review-open; runtime remediation pending.

Please review product/harness checkpoint `c9c9628eb`, which addresses the
first packet-032 synthetic failure.

The owner visibility retry path wrote to the Task 167 fixture-only unlogged
attribution relation unconditionally. `Spi::run` on an absent relation raised
an undefined-table error and aborted an otherwise recoverable scan before the
fixture reached its concurrency setup.

The retry path now checks the fully-qualified diagnostic relation with
`pg_catalog.to_regclass` and skips attribution when it is absent. The Task 167
fixture creates, truncates, reads, and resets the same fully-qualified public
relation before its measured 2PC wave, so diagnostic attribution remains
available without becoming a production correctness dependency.

The production PG18 compile check passed; see
[`artifacts/validation-check.log`](artifacts/validation-check.log) and
[`artifacts/manifest.md`](artifacts/manifest.md). Runtime remediation will be
added only after rebuilding/installing this exact checkpoint and rerunning the
synthetic gate from a fresh cluster directory.

