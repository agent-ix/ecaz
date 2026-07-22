---
task: 192
packet: 001-preregistration-baseline
role: coder
status: review_requested
date: 2026-07-21
seq: 1
---

# Task 192 pre-registration: validated row-schema state cache

The single candidate is to retain the resolved frozen row-tier schema in the
existing bounded per-backend retained-epoch cache. It is keyed by index OID
and canonical generation fingerprint, inherits the four-entry LRU cap, and is
discarded with the generation entry. The payload endpoint still checks the
caller-provided schema fingerprint and the cached descriptor identity before
fetching rows; epoch fencing and failure behavior are unchanged.

This removes repeated catalog schema resolution from hot payload requests. The
candidate is isolated from payload SQL and transport changes. Validation:
`cargo check --offline -p ecaz --lib --no-default-features --features pg18`
passed after the change. A 100k paired suite is required before any promotion.
