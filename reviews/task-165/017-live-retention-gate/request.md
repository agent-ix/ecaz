# Review request — Task 165: live retention gate (FR-082-AC-3 production wiring)

**Branch:** `task-165-ec-distann-m3`. Makes AC-3's retention gate **live** — real
in-flight scans block retire — without a shared-memory subsystem.

## What landed

`ec_distann_retire_epoch` now gates on PostgreSQL's **lock manager**: a scan
holds `AccessShareLock` on the index for its lifetime, so retire conditionally
acquiring `AccessExclusiveLock` (`ConditionalLockRelationOid`) fails while any
scan is in flight. This is a real, **auto-clearing** count (a crashed scan's lock
releases on backend exit) — lower-risk and more correct than a manual
shared-memory counter, which a naive block-0 write would leak on error and which
would need `_PG_init` shmem infra. The persisted `in_flight_count` field + the
logged `force_retire` override remain the operator-visible / wedge path (AC-6).

## Evidence (`artifacts/distann-multinode-summary.log`, real 3× PG18)

```
live_retention_gate pass=true
concurrency_scan_insert_epochswap pass=true
RECALL_RESULT n_queries=50 identical=50 mismatched_ids=0
recovery ... recovered=true
```

`live_retention_gate`: a held-open index scan (AccessShareLock via a cursor)
blocks `retire` (gated), and retire succeeds once the scan drains. **110 distann
pg_tests pass**; clippy clean.

## Remaining tail

AC-5 cross-epoch frozen-vector tier (reverses ADR-085 D11; needs design +
storage measurement), suite-driven recall gate, and disjoint-shard
build-then-distribute — the deep storage/bench follow-ups tracked in packet 014.

## Ask

Review the lock-based gate design (`epoch_manifest.rs`) and the fixture drill.
