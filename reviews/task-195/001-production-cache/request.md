---
task: 195
packet: 001-production-cache
role: coder
status: review_requested
date: 2026-07-22
seq: 1
---

# Task 195 production owner-schema cache checkpoint

This checkpoint makes Task 192's measured immutable row-schema entry the one
normal physical-owner materialization path. It removes the benchmark-only
schema-cache selector from extension options, the physical profile endpoint,
remote transport parameters, CLI variant parsing, suite JSON, result lines,
and provenance strings. The task remains in progress until packet 002 records
the release-profile 10k/50k/100k before/after suite matrix.

## Production behavior

`RetainedGenerationScan` now resolves the immutable row schema when it installs
the retained generation entry and always uses that resolved entry during
payload materialization. A request still opens the exact retained row-tier,
graph-store, and directory relations before returning a cache hit. It still
checks the requested generation fingerprint, descriptor schema fingerprint,
caller-expected schema fingerprint, projection attnums, and relation
availability. No result, projection, payload, placement, storage, traversal,
or materialization-window format changed.

The retained-generation LRU remains backend-local and bounded to four indexes
with at most one fingerprint per index. Its existing relcache callback evicts
on the control index, row tier, graph store, directory, or global invalidation.
The pre-existing `ec_distann.physical_epoch_cache` diagnostic controls the
whole physical epoch cache; there is no schema-cache-specific production GUC,
reloption, endpoint argument, or suite selector.

## Removed selector surface

- deleted `ec_distann.benchmark_owner_validation_cache` and its getter;
- deleted `use_cached_schema` from the profile endpoint and transport request;
- deleted `owner_validation_cache` from suite JSON and variant encoding;
- made suite deserialization reject the removed JSON field rather than ignore
  a stale A/B configuration;
- retained the independent Task 193 payload-plan and Task 194 fixed-work
  benchmark controls with their fields shifted into the shorter encoding.

## Validation

- normal PG18 strict Clippy: pass;
- PG18 attribution-feature strict Clippy: pass, proving feature isolation;
- removed-selector and shifted-neighbor-control CLI tests: 4 passed total;
- PG18 `test_distann_multi_epoch_publish`: 1 passed, 0 failed, 2,519
  filtered. The drill exercises the production cached path across successor
  publication, retained predecessor access, reclaim, and stale-fingerprint
  rejection.

The pgrx lifecycle test installed a debug extension. Packet 002 will reinstall
and byte-verify release binaries before any measurement, specifically avoiding
the debug-binary incident diagnosed during Task 194.

## Review focus

Please verify that the selector is gone from every public/profile/transport and
suite surface, the immutable schema entry preserves exact epoch fencing, the
cache bound and invalidation coverage remain intact, and normal builds do not
gain attribution instrumentation.
