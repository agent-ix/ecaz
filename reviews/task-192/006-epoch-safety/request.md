---
task: 192
packet: 006-epoch-safety
role: coder
status: review_requested
date: 2026-07-21
seq: 1
---

# Task 192 epoch-fencing safety slice

This checkpoint closes the correctness gate that packet 005 deliberately left
open. It does not change production behavior: the schema-cache arm and its two
inspection helpers remain absent unless the attribution or `pg_test` feature is
enabled.

## Change

The backend-local retained-generation cache remains bounded to four entries,
but now admits at most one fingerprint per index. When a request observes a new
fingerprint for an index, it drops the older same-index entry before installing
the new immutable generation/schema snapshot. Multiple independent indexes may
still occupy the four-entry LRU.

The PG18 test build now compiles the same cached row-schema selection used by
the attribution candidate. Two `pg_test`-only helpers let the existing real
multi-epoch lifecycle test validate and inspect that path without adding any
production SQL surface.

## Safety argument

- Cache identity is exact `(index_oid, 34-byte epoch_fingerprint)`; a request
  cannot hit an entry from another generation.
- A newly observed same-index fingerprint explicitly evicts its predecessor.
- The existing relcache callback evicts an entry when its index, row-tier heap,
  graph heap, directory index, or all relations are invalidated.
- Every request still opens the exact generation's row/graph/directory
  relations and checks the descriptor schema fingerprint, caller-expected
  schema fingerprint, descriptor equality, and requested attributes.
- ADR-085 D10 / FR-082 make the Published build-time row tier immutable. A
  Retired predecessor remains addressable only while retained, per FR-079
  AC-10; physical reclaim broadcasts relcache invalidation and removes the
  catalog identity.

## Validation

`test_distann_multi_epoch_publish` passed on PG18 (1 passed, 0 failed, 2,519
filtered). The augmented real-backend drill proves:

1. epoch 7 Published warms cached schema validation;
2. epoch 8 publication retires epoch 7;
3. selecting epoch 8 replaces the same-index epoch-7 cache entry;
4. the retained epoch-7 fingerprint still validates and symmetrically replaces
   epoch 8 while it remains retained;
5. reclaim evicts the entry; the same epoch-7 call then fails with classified
   `EC_GENERATION_MISSING`.

Both the ordinary PG18 production feature set and the PG18 attribution feature
set pass `cargo check`. The pgrx test intentionally installed a 330,795,520-byte
debug binary; it will be replaced and byte-verified against a fresh release
attribution build before the full-scale benchmark.

