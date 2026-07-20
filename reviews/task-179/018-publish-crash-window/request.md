---
agent: codex
role: coder
model: gpt-5
date: 2026-07-11
seq: 01
---

# Review request — Packet 018 publish transaction/crash boundary

Closes packet 012 P2-1's one-transaction evidence defect and adds an executable
FR-082 T3→T4 transaction boundary plus the post-ack/pre-pointer-swap crash drill.

## Commit

- `b87523e7` — enforce publish recovery transaction boundary and crash window.

## Behavior

- Recovery reads the durable decision row's PostgreSQL `xmin` and uses
  `TransactionIdIsCurrentTransactionId` to reject a decision inserted by the
  current top transaction with `EC_TRANSACTION_BOUNDARY`.
- `ec_distann.debug_fail_recover_after_publish_ack` injects an error after the
  participant returns the exact Published fingerprint but before active-pointer,
  decision-state, or registration-state mutation.
- Because participant publication and the coordinator swap are in the same local
  T4 transaction in this single-node slice, injection rolls participant state
  back to Ready while the separately committed decision remains Pending and the
  registration remains Decided.

## Evidence

- The former one-transaction positive fixture is now a negative fixture proving
  recovery rejects an uncommitted decision.
- The real-backend multi-epoch fixture runs begin, build, decide, and recover in
  separate autocommit transactions. It injects the post-ack failure and asserts
  `Pending / Decided / Ready` with no active pointer, disables injection, retries
  T4a successfully, then publishes a successor through predecessor CAS.

Validation and provenance are in `artifacts/manifest.md`.

## Still open

This packet does not claim Task 179 closeout. Physical Published-generation reads,
scan-token RAII, T4b retirement/reclaim, the real three-node owner-sharded fixture,
and required 10k/50k/100k A/B evidence remain open.

Leaving this request open for outside review.
