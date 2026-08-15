---
agent: codex
role: coder
model: GPT-5
date: 2026-08-15
seq: 1
---

# Task 167 natural retry attribution follow-up

Status: review-open; not merge-ready.

Checkpoint `04657f7f0` tightens the owner-retry concurrency drill after the
reviewer identified ambiguous evidence and a mid-drill counter reset. The
unlogged attribution relation is still created and truncated only in the
pre-wave setup (and reset after the wave); the extra reset immediately before
the wave is removed. The diagnostic now labels the measured count as
`retry_source=natural_2pc_wave` and explicitly reports
`forced_retry_probe=false`, so a non-zero count cannot be mistaken for the
pg_test forced fault probe.

The same checkpoint updates the extension-preflight unit-test fixtures for the
feature provenance field and adds coverage that rejects a release-profile
`pg-test` extension without the explicit diagnostic override.

Validation:

- `timeout 120s env CARGO_TARGET_DIR=/home/peter/dev/ecaz/target cargo check -p ecaz-cli --tests --no-default-features --quiet` passed.
- The production extension checks at the preceding head passed for both `pg18` and `pg18,pg_test`.
- Runtime preflight and the 10k/50k/100k matrix remain outstanding because
  `/home/peter/.ecaz/clusters` is on a read-only filesystem; fixture startup
  fails before PostgreSQL starts.

No merge or task closeout is requested. The remaining gates are still the
production-feature install, actual-head runtime matrix, unforced retry count,
50k/100k liveness, exact-degree saturation, recall interpretation, latency
sample size, storage, and packet-local evidence.
