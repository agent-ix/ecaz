# Design proposal — FR-082 ec_distann epoch lifecycle (Task 165 AC-2)

**Status:** proposed design for the remaining M3 acceptance criteria. Read
alongside `spec/functional/index/distann/FR-082-distann-epoch-lifecycle.md`
(6 sub-ACs) and `/tmp/distann-build-4.md`. This is the design pass that must
precede coding the FR-082 subsystem; it is not yet implemented.

## Why a design pass

The read-path core (CustomScan + real 3-instance gate + 6-drill fault matrix,
packets 010–013) is done and proven. The remaining AC-2 work is the epoch
**lifecycle** (Building → Published → Retired) which ec_distann does not yet
have — the roster/epoch are session GUCs. The spec mandates "reusing the SPIRE
epoch-manifest machinery," so this is an *integration + adaptation*, not a
greenfield build, but it is design-sensitive (on-disk manifest, atomic publish,
retention semantics) and worth pinning before code.

## What already exists (reuse targets)

- **SPIRE**: `src/am/ec_spire/meta/{epoch.rs,root_control.rs,snapshot.rs}` — a
  root/control page that names the active epoch's manifest TID; publish = atomic
  root/control swap; `coordinator/maintenance.rs` carries
  `active_epoch_before/after`; retention/reclaim on epoch build. On-disk fixture:
  `fixtures/on-disk/spire_epoch_manifest_v1.hex`.
- **ec_distann**: content-based epoch fingerprint (`epoch.rs`,
  `compute_epoch_fingerprint`) already validated on every `expand_nodes` /
  `materialize_row_payloads` call; the metadata page (`page.rs`, format v3);
  FR-083 tombstone/delta/insert primitives (packets 001/002, M5).

## Proposed implementation, mapped to each sub-AC

### Persisted manifest (the linchpin — unblocks AC-1/3/6 and the AC-2 restart)
Add a **distann epoch manifest** page (adapt `ec_spire/meta/epoch.rs`) storing:
roster (node_id → conninfo/identity), placement hash version, format version,
build-time record-set digest, head-sample identity, and the frozen vector-tier
handle. A **root/control** page (adapt `root_control.rs`) names the active
epoch's manifest TID. `roster.rs` GUCs become a *fallback/override* for the
single-node degenerate case; multi-node sources roster+epoch from the manifest
(the `roster.rs` TODO comment already anticipates this).

### AC-1 — epoch swap returns results wholly from one epoch
**Publish** = write the new manifest, then one atomic root/control TID swap
(mirrors SPIRE). A scan reads the active epoch's fingerprint once at
`ambeginscan`/CustomScan `BeginCustomScan` and pins it for the whole attempt;
every remote call carries that pinned fingerprint. Drill: publish a new epoch
while a concurrent load runs; assert every query's result set maps to exactly
one epoch (already structurally enforced by the pinned fingerprint + AC-2
restart).

### AC-2 — mismatch → ONE restart under refreshed epoch, then error
Wrap the coordinator scan (`collect_distann_hits`) in a **restart-once** loop:
on `DistannExpandError::EpochMismatch` from any hop, discard partial hits,
re-read the active epoch from the manifest, and re-run from the head index once;
a second mismatch raises. (Today the mismatch→error half is built + proven by
`remote_content_divergence`; only the single refresh-and-retry is missing.)
Resets NFR-019 accounting per attempt (≤2).

### AC-3 / AC-6 — retirement gated on in-flight; operator override, logged
**Retire** an epoch only when its in-flight query count hits zero (reuse SPIRE's
retention gate). Maintain the in-flight counter as scans enter/leave an epoch.
Add `ec_distann_epoch_retire(index)` (gated) and
`ec_distann_epoch_force_retire(index, epoch)` (operator override) that logs a
`WARNING` and reclaims a wedged epoch. Physical record/vector reclaim + FR-077
edge repair run at the next build (per D10), never mid-epoch.

### AC-4 — concurrency: only expanded records; no half-applied back-edge
Already upheld by the M5 insert path's per-record write atomicity + tombstone-
at-expansion. Remaining = a **concurrency drill** on the fixture: run scans while
applying tombstones/inserts via `ec_distann_apply_record_writes`; assert results
draw only from expanded records and never a torn adjacency.

### AC-5 — frozen vector snapshot; no base-table TID-reuse race
This is the sharpest gap and was demonstrated live: a base-table `DELETE`
strips the co-placed rerank vector → `[EC_VECTOR_MISSING]` (see packet 013).
Within a Published epoch the vector tier must be an **epoch-owned frozen
snapshot** (multi-node) so a concurrent delete+VACUUM+TID-reuse cannot re-point
a vec_id's rerank at a different tuple. Design: the manifest pins the vector-tier
handle; multi-node rerank reads the frozen tier, not the live base table.
Single-node degenerate case keeps serving the base table under the AM's existing
tombstone/vacuum consistency (spec-permitted).

## Fixture extensions (once the above lands)
`ecaz dev distann-multicluster` gains: `--publish-epoch` / `--retire` steps, an
epoch-swap-under-load drill, a tombstone-based `mid_delete` drill (needs an
`owning_node` SQL helper for per-node bucketing), and a force-retire drill.

## Recommendation
Land this as its own scoped slice/task (it is milestone-scale even reusing
SPIRE). The read-path M3 core (010–013) stands on its own as the proven
deliverable; AC-2's lifecycle is the tracked remainder.
