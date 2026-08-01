# ec_distann catalog/DDL surface inventory

Task 214 P0 slice (input to P3 per-table docs). Auditor: parallel subagent,
2026-08-01, worktree `.worktrees/task-203` @ `baf81d498`.

## Sources and mechanics
- All DDL in `sql/bootstrap.sql`, loaded via `extension_sql_file!` at `src/lib.rs:519`.
- Privileges/finalize SQL (SECURITY DEFINER closure, triggers) in `extension_sql!(... "distann_internal_privileges", finalize)` at `src/lib.rs:521-945`.
- Rust resolves table names via `CatalogRelations::resolve()` (`generation_catalog.rs:67-90`, 16 tables) plus ad-hoc `extension_relation_name(...)` for the four head-replica/traversal tables not in the struct.
- **20 tables total.**

## Table roster (name — DDL — role — spec mention)

| # | Table | DDL (bootstrap.sql) | Role / lifecycle | Spec mention |
|---|-------|--------------------|------------------|--------------|
| 1 | `ec_distann_participant_identity` | 272-280 | one row per v5 control index; written by `ec_distann_configure_participant_identity` (`node_registry.rs:395-413,451`); deleted by index cleanup (`generation_catalog.rs:918-921`) | none (FR-078 names the function only) |
| 2 | `ec_distann_registry_state` | 282-288 | roster-change revision fence; init `generation_catalog.rs:97-99`; bump `node_registry.rs:148-150`; probed by drop event trigger (bootstrap 1512-1515) | none |
| 3 | `ec_distann_node_descriptor` | 290-316 | operator-managed roster; register/unregister `node_registry.rs:469,594`; unregister refuses when referenced (:610-612); partial unique one-local | **FR-078:161** |
| 4 | `ec_distann_generation` | 318-398 | per-shard generation on owning participant; Building→Ready→Published→Retired→deleted; heavy state-machine CHECKs (377-389); fingerprint partial unique; writers: begin/stage/seal/abort handoff, publish, retire, reclaim (`generation_store.rs`, `handoff.rs`, `participant_lifecycle.rs`) | none |
| 5 | `ec_distann_generation_batch` | 400-425 | append-only batch acknowledgement ledger; FK CASCADE to generation; insert `generation_catalog.rs:500-516`; replay validation `generation_store.rs:630` | none |
| 6 | `ec_distann_build_registration` | 427-462 | coordinator gate row, one active per logical index (partial unique); states Registered/Building/Ready/Aborted/Decided/Published/Cancelled; T1-T4a + cancel writers; statement trigger `ec_distann_build_gate_registration_changed` (lib.rs:923-930 → `build_gate.rs:201`) | wire domain only (FR-078:257) |
| 7 | `ec_distann_build_participant_binding` | 464-501 | immutable per-build roster snapshot; T1 fan-out insert (`t1.rs:412`); CASCADE with registration | none |
| 8 | `ec_distann_build_candidate` | 503-540 | immutable T2 candidate (spec/descriptor/snapshot/receipts/manifest/fingerprint digests); insert `t2.rs:709`; RESTRICT-held by publish_decision | **FR-078:324,340** |
| 9 | `ec_distann_generation_head_state` | 546-575 | per-build head state; `membership bytea CHECK (octet_length = 4 + 8*sample_count)` (562) = NFR-021 clause 3 / Task 210 membership-only head; policy CHECKs (565-571); insert `head_sample.rs:804-809`; **NOT in REVOKE block** | none |
| 10 | `ec_distann_generation_head_sample` | 577-594 | per-landmark rows; `vector` nullable (NULL in membership-only mode → zero rows); CASCADE from head_state; **NOT in REVOKE block** | none |
| 11 | `ec_distann_head_shard_replica` | 602-612 | §4.1 replica copy table (Task 210 P2b), epoch_fingerprint-scoped; upsert `generation_read.rs:1405-1418`; read `replica_head_vectors` (:1444-1460); **no DELETE path anywhere; not in index cleanup — leaks rows**; **NOT in REVOKE block** | none |
| 12 | `ec_distann_head_replica_state` | 616-621 | population attestation row (written only after every (shard,replica) pair imported, `generation_read.rs:2160-2170`); read by routing gate (:3502-3520); **no deletion path; NOT in REVOKE block** | none |
| 13 | `ec_distann_publish_decision` | 623-745 | coordinator decision ledger Pending→Activated→Applied/Cancelled; predecessor quadruple + self-referential predecessor FK (726-733); one-recovery-active partial unique; T3/T4a/cancel/abandonment writers | none (FR-082 names endpoints only) |
| 14 | `ec_distann_predecessor_disposition` | 747-847 | per (successor build × predecessor ordinal) Pending/Retired/Abandoned; settled before successor Applied; T4a fan-out insert; wide RESTRICT FKs | endpoint + domain only (FR-082:234,403,423) |
| 15 | `ec_distann_retire_decision` | 849-913 | coordinator retire ledger Pending→Applied; covering-successor FK pinned to an Applied decision (902-912); forced/reason CHECK | domain only (FR-082:483) |
| 16 | `ec_distann_generation_reclaim` | 915-940 | participant retire tombstone (no FKs — survives its generation); insert `participant_lifecycle.rs:632-633,779` | none |
| 17 | `ec_distann_cancelled_generation_reclaim` | 942-966 | cancel-path tombstone; insert `participant_lifecycle.rs:868-869,1061` | none |
| 18 | `ec_distann_active_epoch` | 968-988 | **single active pointer per logical index** (PK `(index_oid, logical_index_uuid)`); FK NO ACTION → publish_decision; flipped only by T4a; the read path's epoch authority (`generation_read.rs:2705,2974`) | none |
| 19 | `ec_distann_traversal_replica` | 990-1079 | FR-084 coordinator-local derived copy; Building/Ready/Stale/Retiring + four-way state-shape CHECK; one-active + one-local-authority partial uniques; statement trigger (lib.rs:933-940 → `traversal_replica.rs:138`); own retire/reclaim path (not in index cleanup) | domain + status endpoint only (FR-084:63,166) |
| 20 | `ec_distann_traversal_replica_owner` | 1081-1098 | per-owner completion receipts (`copied = expected` CHECK); CASCADE with replica | none |

## SQL-visible control-plane API (pg_extern)
All SECURITY DEFINER + pinned search_path + REVOKE PUBLIC via lib.rs:523-866
explicit blocks or the class-wide closure at lib.rs:872-921 (exemptions:
`ec_distann_handler`, `ec_distann_owning_node`, `ec_distann_epoch_status`).

- **Registry/identity**: `initialize_control_registry`, `configure_participant_identity`, `register/unregister_node_descriptor`, `control_identity`, `test_set_conninfo_secret`.
- **Coordinator T1-T4**: `begin/abort_epoch_build` (t1), `build_epoch(_with_training)` (t2), `decide_epoch_publish` (t3), `recover_epoch_publish` (t4a), `cancel_epoch_publish` / `recover_cancelled_publish`, `epoch_build_status`, `build_gate_relation_mask`.
- **Participant handoff**: `begin/abort_epoch_handoff`, `list_unpublished_generations`, `prepare_control_rebuild`, `stage_epoch_batch`, `seal_epoch_handoff`, `generation_topology`, `epoch_topology`.
- **Participant lifecycle**: `publish_epoch` (4-arg), `epoch_generation_status`, `mark_epoch_retired`, `apply_epoch_retire`, `reclaim_cancelled_generation`.
- **Coordinator retire/abandon**: `retire_epoch`, `force_retire_epoch`, `recover_epoch_retire`, `abandon_predecessor_binding`. **Naming trap**: legacy v1 same-named overloads (`epoch_manifest.rs:92-192`) distinguished only by signature.
- **Cleanup**: `catalog_index_cleanup` via event trigger `ec_distann_catalog_drop_index_cleanup` (bootstrap 1495-1526).
- **Read/query plane**: `list_directory`, `owning_node`, `expand_nodes`, `epoch_fingerprint`, `apply_record_writes`, `materialize_rows/_row_payloads` (`remote_endpoint.rs:94-463`); physical forms `expand_physical_nodes(_profile)`, `materialize_physical_row_payloads(_profile)`, `head_search_physical`, `head_shard_export/import`, `populate_head_replicas`, `gateway_routing_export`, `stream_traversal_replica_chunk`, `active_head_policy` (`generation_read.rs:346-2864`); SQL wrapper overloads lib.rs:781-866.
- **Traversal replica**: build/mark-stale/guard-mutation/control-preflight/recover-invalidation/retire/reclaim/status (`traversal_replica.rs:1003-1834`).
- **DML/misc**: `fold_delta_into_graph` (`insert.rs:482`), `debug_tombstone` (`dml.rs:128`), `debug_expand_search` (`remote_transport.rs:2326`), `gateway_copy_stats` (`gateway_copy.rs:154`), stage counters (lib.rs:1881-1936), trigger fns.

## Cross-cutting findings for the spec docs
1. **Four tables missing from the REVOKE block** (bootstrap 1100-1115 covers 16/20): `_generation_head_state`, `_generation_head_sample`, `_head_shard_replica`, `_head_replica_state`.
2. **No deletion/reclaim path** for `_head_shard_replica` and `_head_replica_state`; absent from `delete_index_catalog_rows` (`generation_catalog.rs:851-931`) — stale epochs and dropped indexes leak rows.
3. **Spec coverage nearly zero at table level**: 17 of 20 tables — including `_generation`, `_active_epoch`, `_publish_decision`, and the entire head-state/replica group — appear nowhere in `spec/`.
4. The `membership` CHECK (562), nullable head-sample `vector` (583-587), and the two head-replica tables encode NFR-021 clause 3 / Task 210 P2a-b; none is in a spec file.
