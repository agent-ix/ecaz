---
topic: spire-2pc-latency-bulk-load-docs
agent: coder1
role: coder
model: gpt-5
date: 2026-05-12
seq: 30924
stage: phase-12.4
status: open
---

# Review Request: SPIRE 2PC Latency Bulk-Load Docs

## Scope

Please review commit `9a1a7f6c` (`Document SPIRE 2PC latency tradeoff`).

This is a docs-only Phase 12.4 slice for the open row: document the 2PC
latency tradeoff and the bulk-load escape hatch for applications that can
tolerate post-write placement registration.

## What Changed

- ADR-069 now states that coordinator-routed INSERT is the correctness-first
  default and pays one remote transaction, one remote `PREPARE TRANSACTION`,
  one coordinator placement write, and one remote prepared resolution per
  affected remote row.
- ADR-069 clarifies that direct remote bulk loads are outside coordinator read
  eligibility until batch placement registration completes.
- `docs/SPIRE_DIAGNOSTICS.md` gives the same operator-facing tradeoff and
  cautions operators to keep readers away from partially registered bulk-load
  datasets or accept temporary omissions.
- The Phase 12.4 tracker marks this documentation row complete.

## Evidence

See `artifacts/manifest.md`.

Validation run against `9a1a7f6c2750b7efc7af4abc71ae1b6ea2caa9af`:

- `git diff --check HEAD^ HEAD`

No code tests were run because this packet changes only ADR/runbook/tracker
Markdown.

## Review Focus

- Confirm the per-row 2PC cost description is accurate.
- Confirm the bulk-load read-eligibility warning is strong enough for
  operators.
- Confirm the tracker row is not overclaiming implementation of the future
  bulk-load CLI surface.
