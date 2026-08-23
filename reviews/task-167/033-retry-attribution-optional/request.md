---
agent: codex
role: coder
model: GPT-5
date: 2026-08-21
seq: 1
---

# Task 167 optional retry attribution

Status: review-open; attribution remediation verified, synthetic gate exposed a
separate snapshot-lifetime defect.

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
[`artifacts/manifest.md`](artifacts/manifest.md).

Runtime was then rerun from a fresh cluster with the release CLI and PG18
extension built at exact head `1c70b57d2`. The original undefined-table error
did not recur, release provenance and three-owner topology passed, and the
first distributed serving query passed with 10 rows. The later full-owner
materialization query aborted the coordinator backend on PostgreSQL's
`TransactionIdFollowsOrEquals(xid, TransactionXmin)` assertion. Symbolication
places the extension frame in `generation_read::lookup_graph_nodes`.

That assertion is a separate blocker: the 10k/50k/100k steps were not selected
and no concurrency, retry, saturation, recall, latency, or storage result is
claimed. See
[`artifacts/smoke-synthetic-remediated/concurrency-synthetic/node1-postgres.log`](artifacts/smoke-synthetic-remediated/concurrency-synthetic/node1-postgres.log)
and the packet manifest for the exact boundary.
