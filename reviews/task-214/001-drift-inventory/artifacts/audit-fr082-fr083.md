# Audit: FR-082 (epoch lifecycle) + FR-083 (DML path) vs code

Task 214 P0 slice. Auditor: parallel subagent, 2026-08-01, worktree
`.worktrees/task-203` @ `baf81d498`.

Structural headline: **two implementation lanes exist** — the legacy local/v4
metadata-page lane (`epoch.rs`, `epoch_manifest.rs`, `dml.rs`, `insert.rs`,
`remote_endpoint.rs`) and the physical/v5 distributed-control lane
(`manifest_v2.rs`, `lifecycle_wire.rs`, `participant_lifecycle.rs`,
`coordinator_*`, `generation_*`, `build_coordinator/`, `scan_registry.rs`).
The spec describes only the v5 lane; several requirements are implemented only
in the legacy lane, and vice versa. The v5 wire formats conform closely.

## Findings

### F-01 — Two incompatible epoch-fingerprint schemes ship simultaneously (high, specified-but-changed)
Spec: 34-byte `u16_le(2) || manifest_digest` (CON-5). v5 conforms (`manifest_v2.rs:32,544-584`), but the entire legacy multi-node read/write path validates a 16-byte FNV-1a v1 fingerprint (`epoch.rs:24-136`; `remote_endpoint.rs:207-224,252-275`; `routine.rs:458-462`) that violates CON-5 and the `EC_EPOCH_FINGERPRINT_VERSION` contract. The spec has no notion of the second scheme.

### F-02 — Complete second (legacy v4) lifecycle surface unspecified, with SQL-name collisions (high, shipped-but-unspecified)
`epoch_manifest.rs:93-200` — overloads `ec_distann_publish_epoch(oid, bigint)`, `_retire_epoch(oid)`, `_force_retire_epoch(oid)`, `_epoch_status(oid)` on the v4 metadata page (`epoch_state`/`active_epoch`/`in_flight_count`). Retention gate is `ConditionalLockRelationOid(AccessExclusiveLock)` + persisted counter, not the spec's shared-memory fence; force-retire emits a warning instead of the required committed audit record. Rejects distributed-control indexes; nowhere in the spec.

### F-03 — Recovery endpoint takes a build id; spec says build-id-free (medium, specified-but-changed)
`t4a.rs:9` — `(index_regclass, build_id uuid)`; FR-082:223 declares `(index_regclass)` only. Same finding as FR-077/078 audit F5.

### F-04 — Advisory-lock single-flight + scan-triggered T4a do not exist (medium, specified-but-changed)
Spec mandates a transaction-scoped advisory lock keyed by logical-index UUID and non-blocking scan-triggered recovery. Code: `SourceSessionLockGuard` + `ShareRowExclusiveLock` + registry revision lock (`t4a.rs:25-46`); zero advisory locks in the module; scans only read+pin the active pointer (`generation_read.rs:3031-3072`). Functionally equivalent exclusion; mechanism and scan-triggered path stale.

### F-05 — Abandon replay ignores caller identity (medium, specified-but-changed)
`coordinator_abandonment.rs:192-205` — replay matches on build/ordinal/reason; stored `caller_name` never compared to `session_user`. Spec requires a different caller to get `EC_PREDECESSOR_ABANDON`.

### F-06 — `EC_PUBLISH_PENDING` never implemented (medium, specified-but-removed)
Zero occurrences in `src/`; unavailable successors surface as transport errors / `EC_EPOCH_STATE` (`t4a.rs:305-348`). Behavior holds (predecessor stays active) but the stable retriable error code is absent.

### F-07 — Parent-epoch ordering not validated until retirement (medium, specified-but-changed)
Spec: manifest `epoch` must exceed the parent's resolved epoch. Code checks only fingerprint well-formedness (`manifest_v2.rs:610-661`) and active-pointer equality at T3 (`t3.rs:198-208`); ordering checked only at `mark_epoch_retired` (`participant_lifecycle.rs:438-440`) — after the successor is Published and active.

### F-08 — Status endpoint extended to 11 columns + `CancelledReclaimed` state (low, shipped-but-unspecified)
`participant_lifecycle.rs:299-419` adds `cancellation_audit_digest` and a tombstone-derived state row; spec declares 10 columns and no such state value.

### F-09 — No `Aborted` generation state; abort = row deletion (low, specified-but-changed)
`lifecycle_state.rs:87-95` — `GenerationState` lacks Aborted; abort deletes the row (`generation_store.rs:894+`); Aborted exists only on `RegistrationState`. Observable status is `EC_GENERATION_MISSING`.

### F-10 — Routed tombstone delete does not exist; v5 deletes are silently dropped (high, specified-but-changed)
FR-083: ambulkdelete tombstones at the hash-owned node via the remote write endpoint; failed write errors. Code: `dml.rs:63-121` local-only flag flip; **distributed-control ambulkdelete returns noop vacuum stats** (`routine.rs:137-149`); `ec_distann_apply_record_writes` is legacy-storage-only (`remote_endpoint.rs:263-264`). A DELETE on a real multi-node index never tombstones anything — the exact "lost tombstone resurrects a deleted row" hazard FR-083 calls out.

### F-11 — Entire FR-083 update contract unimplemented (high, specified-but-removed)
No redirect machinery; `routine.rs:527-534` documents delete-then-insert with a *different* vec_id; `insert.rs:324-329` rejects duplicate vec_ids outright. Stable-vec_id preservation, replacement append, atomic directory redirect, old-version retention (FR-083-AC-8): none exist on any lane.

### F-12 — Both insert postures legacy-lane-only; no v5 insert at all (high, specified-but-changed)
Delta insert only for local-identity mode on a non-empty legacy index (`dml.rs:278-356`; include-mode errors :298-303); v5 aminsert errors `EC_GENERATION_MISSING` (`routine.rs:107-111`); incremental insert (`insert.rs:279-475`) is single-node, no co-placed row-tier write, no owner routing, rejects GroupedPQ. FR-083-AC-3/4/7 unsatisfiable on a real multi-node deployment.

### F-13 — Remote write endpoint implements 1 of 3 specified operations (high, specified-but-changed)
`remote_endpoint.rs:226-310` — tombstone-set only; docstring says append/back-edge "are later"; legacy-storage-only with the 16-byte fingerprint (F-01). Isolation rejection + hardening correctly applied.

### F-14 — Fold endpoint not in the protected endpoint class (medium, specified-but-changed)
`ec_distann_fold_delta_into_graph` (`insert.rs:482-488`) calls `require_read_committed` but has no SECURITY DEFINER / search_path pin / PUBLIC revoke in `src/lib.rs` — the only graph-mutating endpoint outside the hardened class.

### F-15 — Insert collision handling collapses both spec branches (medium, specified-but-changed)
`insert.rs:320-329` — any directory hit errors; no same-identity→update dispatch (consistent with F-11 internally; spec semantics unimplemented).

### F-16 — `ec_distann_reclaim_cancelled_generation` endpoint unnormalized (low, shipped-but-unspecified)
`participant_lifecycle.rs:868-1134` + tombstone table behave as prose describes, but the spec's exhaustive function list omits the endpoint signature.

### F-17 — FR-083 spec-internal defect: duplicate `FR-083-AC-5` (low)
Two rows labeled AC-5 shift numbering for everything after; test tags citing AC-5..AC-8 cannot resolve uniquely.

## Behaviors in NO distann spec (grep-confirmed)
- Legacy v4 metadata-page lifecycle endpoints + fields (F-02).
- 16-byte FNV fingerprint + `ec_distann_epoch_fingerprint(oid)` (F-01).
- Delta-buffer internals: `DISTANN_DELTA_BUFFER_CAP = 4096` (`dml.rs:51`), metadata fields, `ec_distann_fold_delta_into_graph` by name.
- `ec_distann_initialize_control_registry`, `ec_distann_prepare_control_rebuild`.
- Head-replica/gateway export surface in `generation_read.rs` (cross-ref FR-079/081 + FR-080 audits).
- Retirement head-state cleanup: `coordinator_retirement.rs:645-665` deletes `_generation_head_state` rows during retire recovery — coordinator-side FR-080 object reclaim FR-082 never mentions.

## Confirmed-conformant highlights
Manifest v2 field order/encoding/domains; 303-byte receipt; 34-byte fingerprint; candidate digest chain recompute at T3/T4; commit-only Pending decision with Ready→Decided CAS; CAS pointer swap + Activated/Applied split + per-ordinal dispositions; cancellation xmin guard and exact-reason replay; shared-memory scan registry with 65,536/4,096 defaults and all three EC_EPOCH_PIN/REGISTRY codes; retire fence before zero-count observation (`EC_RETENTION_ACTIVE`); participant tombstone-before-delete; `require_read_committed` everywhere; SECURITY DEFINER + search_path + revoke on all v5 lifecycle functions except the fold endpoint (F-14).
