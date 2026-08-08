# Task 165: ec_distann M3 — Multinode Lifecycle + Fault Drills

Status: **complete — superseded by the accepted physical Task 179 lifecycle
lane** (2026-08-08). The 12-drill transport/fanout matrix and single-relation
prototype remain replicated-serving controls in `reviews/task-165/024–028`;
Task 179 packets 059–060 accept the physical generations, owner handoff,
publication/recovery, scan fencing, and three-process fault evidence. The
physical DML adaptation remains explicitly carried by Task 167. Depends on:
Task 164.
Owner: coder (to be assigned). One coder, one branch.
Priority: P0 — NFR-020's never-silently-wrong bar is proven here.

## Why

A hop-round architecture's new hazard class (partial beams, placement
drift, epoch swaps mid-scan) must be drilled before any gate number is
trusted. FR-082 full lifecycle + FR-083's early DML slices land here.

## Goal

Full fault matrix green on a 3-worker fixture; epoch swap under concurrent
load provably consistent.

## Corrective boundary (2026-07-10)

The existing replicated fixture remains control evidence and does not close
FR-082 or NFR-020's physical publication/recovery rows. Task 179 reran the
relevant drills against distinct Building/Ready/Published/Retired physical
generations and accepted the physical lifecycle. Existing fault names and
result-or-classified-error assertions remain useful controls. Task 167 carries
the remaining physical FR-083 DML adaptation.

## Scope

- 3-worker build/publish/retire pipeline (FR-082 full: D10 mutation model,
  retirement gating + operator override for wedged in-flight counts).
- FR-083 early slices distributed: remote tombstone writes via the write
  endpoint (`ec_distann_apply_record_writes`), delta-buffer drain at epoch
  build, epoch-build reclaim + edge repair re-establishing FR-077
  invariants.
- distann multinode fixture (sibling of `spire_multicluster.rs`): lifecycle
  cases + fault cases — reused set (connection_reset_mid_batch,
  epoch_mismatch, remote_statement_timeout, remote_backend_termination,
  missing_or_reindexed_remote_index, simulated_network_partition) + NEW
  hop_round_failure_mid_beam, missing_node_record, placement_drift,
  mid-delete failure.
- Tests: TC-042 drill matrix; FR-082-AC-1/2/3/4; 50k multinode recall
  integration.

## Required Evidence

Drill logs packet-local; every drill asserts error-or-identical-to-baseline
(NFR-020); epoch-swap-under-load run; 50k multinode distinct_recall ≥
single-node − 0.001 via `ecaz bench suite`, release build.

## Non-Goals

Gate matrix (166); incremental insert (167).

## Acceptance Criteria

1. 100% drill pass across the taxonomy, zero wrong-result occurrences.
2. FR-082 all ACs green incl. restart semantics and retirement override.
3. Tombstone/delta DML behavior verified distributed (FR-083-AC-1/2/3).

## References

- FR-082, FR-083 (early slices), NFR-020
- `plan/design/distann-global-graph-architecture.md` (M3)
