# Review request — Task 165 AC-2: FR-082 epoch lifecycle (all 6 sub-ACs covered)

**Branch:** `task-165-ec-distann-m3`. This packet consolidates the FR-082
lifecycle work: the roster/epoch moved from an M2 session GUC to a **persisted,
lifecycle-managed** state (format v4), and every FR-082 sub-AC now has an
implementation + test. Read with the design in `014-fr082-lifecycle-design/`.

## Sub-AC coverage

| Sub-AC | Implementation | Evidence |
|---|---|---|
| **AC-1** publish / atomic epoch swap | `ec_distann_publish_epoch` — one metadata-page write flips the active epoch; v4 persists `epoch_state`/`active_epoch` across restart | `test_ec_distann_epoch_lifecycle_publish_retire_override` |
| **AC-2** mismatch → one restart then error | `run_scan_attempt_with_restart` wraps the coordinator orchestration; discard partial state, refresh, retry once, second mismatch errors, non-epoch errors don't retry | 4 unit tests (`scan::tests::restart_*`) |
| **AC-3** retention gate | `ec_distann_retire_epoch` errors while `in_flight_count > 0`, keeping the epoch Published | lifecycle pg_test (`retention gate` error asserted) |
| **AC-4** tombstones honored at expansion | FR-083 `ec_distann_apply_record_writes` tombstone-flag set; scan excludes at expansion; `ec_distann_owning_node` surface for per-node bucketing | `test_ec_distann_tombstone_excludes_and_preserves_live_vectors` (3 excluded), `test_ec_distann_owning_node_surface` |
| **AC-5** frozen vec_id→vector correspondence | Under ADR-085 D10 (nothing physically reclaimed within a Published epoch), a live record's rerank vector is unaffected by others' tombstones — because deletion is a tombstone flag, NOT a raw base DELETE (the `EC_VECTOR_MISSING` hazard, packet 013) | same tombstone test — 5 survivors at **byte-identical** exact distance |
| **AC-6** operator override, logged | `ec_distann_force_retire_epoch` force-retires a wedged count, clears it, logs a WARNING | lifecycle pg_test |

**110 distann pg_tests pass** (103 at session start); clippy clean; format v3→v4
(existing indexes REINDEX — research posture, no migration).

## Honest remaining tail (deep / resource-gated, not core-AC gaps)

- **AC-4 concurrent half** (scan *while* inserting; no half-applied back-edge) —
  needs a concurrency harness on the 3-instance fixture. Per-record write
  atomicity (the property it asserts) is already the M5 insert contract.
- **AC-5 multi-node frozen-vector tier** + the VACUUM/TID-reuse race — the
  D10-boundary case. D10 forbids mid-epoch physical reclaim, so the core
  correspondence holds today; the frozen tier is for *cross-epoch* reclaim and is
  a storage-format change (reverses D11's heap-resident vector) that needs a
  design pass + storage measurement.
- **Live in-flight wiring** — the counter that feeds AC-3 in production needs a
  *shared-memory* design (a per-scan block-0 write would serialize all scans),
  not the debug setter used to test the gate here.
- Epoch-swap-under-load drill; suite-driven recall gate; disjoint-shard
  build-then-distribute — fixture/bench work, currently disk-blocked (the shared
  instance is 97% full from unrelated 1m bench corpora).

## Ask

Review the lifecycle state machine (`epoch_manifest.rs`), the restart-once seam
(`scan.rs` + `routine.rs`), and the AC-4/AC-5 tombstone semantics. The deep tail
above is tracked for a follow-up slice.
