# Task 43 Miri / Cargo-Careful Safety Campaign Tracker

Canonical task: `plan/tasks/43-miri-careful-depth.md`

Tracking status: **in progress, not complete**

Last updated: `2026-05-18`

This tracker is the source of truth for Task 43 completion. Review packets are
evidence, not completion by themselves. A surface is complete only when this
file records the required coverage, validation, and mutation/sensitivity
evidence or a concrete blocker with the extraction work needed.

## Campaign Rules

- Treat Task 43 as a safety campaign, not a representative smoke pass.
- Do not declare completion from vacuous criteria. If `miri-many-seeds` runs
  only deterministic single-threaded tests, it is not a concurrency-depth win.
- Prefer real production pure helpers over synthetic test-only code.
- Do not pull pgrx, SPI, libpq, PostgreSQL memory contexts, or live relation
  callbacks into Miri/cargo-careful. Extract narrow pure helpers when needed.
- Promote existing bounded tests when they already exercise the target path.
  Add new tests only where existing tests do not cover a named safety contract.
- For every skipped candidate, record a specific blocker. "Too tangled" is not
  precise enough.
- Keep default Miri, Tree Borrows, many-seeds, cargo-careful, normal Rust
  tests, and mutation probes in separate packet-local artifacts.
- Mutation probes are temporary. Store the diff and failure log in the packet;
  revert the probe before committing production code.
- Future packets must update this tracker before they request review.

## Completion Gates

| Gate | Requirement | Status | Evidence / next action |
| --- | --- | --- | --- |
| G1 | Default Miri and Tree Borrows are first-class hardening lanes. | Done | Packets 002 and 005; `miri-full` is in `hardening-nightly-local`; `make-miri-tree.log` passed 35 tests. |
| G2 | `miri-many-seeds` includes at least one real threaded/atomic `miri_` test. | Done | Packet 007 adds `miri_parallel_worker_slots_are_unique_under_threaded_contention`; targeted `0..128` many-seeds passed. |
| G3 | All strategy-named pure subsystem candidates are covered or precisely blocked. | Done | Packets 008-011 close the strategy-named pure subsystem breadth rows. Remaining work is tracked under G6 careful mirroring/blockers, G7 mutation probes, and G8 final audit. |
| G4 | Remote parser coverage includes Row-independent typed payload validation, not only byte caps. | Done | Packet 009 extracts field-level typed payload validation and covers valid/adversarial fields under Miri. |
| G5 | SPIRE vacuum/delete-delta visibility has Miri coverage or a precise extraction blocker. | Done | Packet 010 covers delete-delta visibility, active delta folding, and invalid delete-target rejection under Miri. |
| G6 | cargo-careful mirrors every path-liftable Miri surface. | Done | Packet 012 proves the current path-lifted careful harness under normal Rust and `make careful` at 69 tests. SPIRE mirrors are blocked by pgrx/SPIRE include boundaries and need extraction or a SPIRE-specific careful micro-harness. |
| G7 | Mutation/sensitivity probes exist for each major subsystem. | Done | Packet 013 records temporary diffs and failing Miri logs for common parallel, DiskANN graph, HNSW graph, DiskANN vacuum, SPIRE top-k, SPIRE routing, SPIRE vacuum/delete-delta, remote typed payload, and SPIRE serialization. |
| G8 | Final audit maps task-file requirements and reviewer findings to evidence. | Not done | Produce final packet only after G2-G7 are complete. |

## Existing Evidence Baseline

| Packet | What it proves | What it does not prove |
| --- | --- | --- |
| `001-coverage-survey-strategy` | Initial inventory and this tracker. | Completion; any code behavior. |
| `002-miri-depth-lanes` | Lane wiring: `miri-tree`, `miri-many-seeds`, `miri-full`, hardening docs. | Real many-seeds interleaving. |
| `003-pure-graph-miri-prefixes` | First DiskANN/HNSW graph tests pass under Miri. | Full graph-helper breadth. |
| `004-spire-vacuum-miri-prefixes` | First DiskANN vacuum and SPIRE top-k tests pass under Miri. | Full vacuum/top-k breadth; SPIRE vacuum. |
| `005-coordinator-serialization-miri-prefixes` | 35-test aggregate, Tree Borrows, many-seeds execution, careful 67-test harness, coordinator/serialization additions. | Real concurrent many-seeds coverage; mutation probes; full strategy breadth. |
| `007-real-many-seeds-parallel-state` | Real threaded common-parallel shared-state test passes under default Miri, Tree Borrows, and `0..128` many-seeds. | Full campaign breadth; mutation probe for common parallel state. |
| `008-breadth-closure-existing-tests` | 32 targeted Miri tests across DiskANN graph, DiskANN vacuum, HNSW graph, SPIRE top-k, and SPIRE routing; careful harness passes 69 tests. | Remote typed payload validation; SPIRE delete-delta/vacuum visibility; SPIRE serialization delta helpers; mutation probes; final aggregate campaign. |
| `009-remote-parser-extraction` | Row-independent remote typed payload validation passes targeted Miri, including valid fields, invalid hex, byte caps, width mismatches, OID/collation parsing, and transport/format constraints. | cargo-careful mirror; mutation probe; final aggregate campaign. |
| `010-spire-vacuum-delete-delta` | SPIRE vacuum/delete-delta visibility and rejection coverage passes targeted Miri: 1 vacuum visibility test, 6 delta snapshot tests, and 1 replacement fold test. | cargo-careful mirror; mutation probe; final aggregate campaign. |
| `011-spire-serialization-layout` | SPIRE assignment row, delta object, and vec-id serialization/layout helpers pass targeted Miri. | cargo-careful mirror; mutation probe; final aggregate campaign. |
| `012-careful-mirroring` | The path-lifted cargo-careful harness passes 69 tests under both normal Rust and cargo-careful; non-lifted SPIRE mirrors have explicit blockers and extraction plans. | Mutation probe; final aggregate campaign. |
| `013-mutation-probes` | Nine temporary mutations failed the targeted Miri tests that protect the campaign's major safety contracts. Each mutation diff was saved and reverted before commit. | Final aggregate campaign. |

## Subsystem Coverage Matrix

Status values:

- **Done**: covered with packet-local validation and no open breadth gap.
- **Partial**: some coverage landed, but named targets remain.
- **Not done**: no adequate coverage yet.
- **Blocked**: not covered, with a concrete blocker and extraction plan.

### Hardening Lanes

| Target | Status | Required evidence |
| --- | --- | --- |
| Default `miri_` prefix | Done | Existing `make miri` / `miri-expanded` lane. |
| Tree Borrows `miri-tree` | Done | Packet 005 `make-miri-tree.log`: 35 passed. |
| Many-seeds range syntax | Done | Packet 005 `make-miri-many-seeds.log`: 128 seed attempts; `0..128` syntax. |
| Real many-seeds interleaving | Done | Packet 007 targeted threaded many-seeds log records 128 seed attempts and exit 0. Final campaign still needs aggregate `make miri-many-seeds`. |
| SB/TB disagreement triage runbook | Partial | Docs exist; no real disagreement to triage. Future disagreement must produce paired logs and classification. |

### Common Parallel Shared State

| Target | Current status | Required action |
| --- | --- | --- |
| Worker slot claim/release atomics | Done | `miri_parallel_worker_slots_are_unique_under_threaded_contention` claims, publishes, and releases real worker slots. |
| Concurrent slot contention | Done | The threaded test spawns more workers than slots, holds live claims until all threads have attempted a claim, and asserts unique slot ownership. |
| Rescan/epoch stale-publish rejection under contention | Done | `miri_publish_parallel_scan_worker_slot_runtime_snapshot_rejects_stale_epoch` covers stale publish rejection after rescan. |
| Many-seeds coverage | Done | `miri-parallel-threaded-many-seeds.log` records 128 seed attempts for the threaded test with exit 0. |
| Mutation probe | Done | Packet 013 `common-parallel-claim-expected.patch` changes the slot claim compare-exchange expected value; `mutation-common-parallel.log` fails `miri_parallel_worker_slots_are_unique_under_threaded_contention` with 0 live claims instead of 3. |

### DiskANN Graph

| Target | Current status | Required action |
| --- | --- | --- |
| `robust_prune` alpha dominance | Done | `miri_robust_prune_excludes_alpha_dominated`. |
| `greedy_search` convergence | Done | `miri_greedy_search_finds_nearest`. |
| `build_vamana_graph_with_stats` | Done | Packet 008 adds `miri_build_small_graph_is_connected`, a bounded 16-node production-helper build test. |
| `build_vamana_graph_with_pass1_extra_candidates` | Done | Packet 008 adds `miri_build_stats_include_pass1_extra_candidates`, proving pass-1 extras enlarge candidate pools. |
| cargo-careful mirror | Done | Packet 008 `careful-harness-cargo-test.log`: 69 passed, including both new bounded Vamana Miri tests. |
| Mutation probe | Done | Packet 013 `diskann-robust-prune-alpha.patch` inverts the alpha-dominance pruning condition; `mutation-diskann-robust-prune.log` fails `miri_robust_prune_excludes_alpha_dominated`. |

### HNSW Graph

| Target | Current status | Required action |
| --- | --- | --- |
| `BeamSearch` dedupe / best-first path | Done | `miri_beam_search_deduplicates_self_loops_and_parallel_edges`. |
| `VisibleFrontier` live scheduler preference | Done | `miri_visible_frontier_best_candidate_prefers_live_scheduler_node`. |
| Stale-candidate removal | Done | Packet 008 promotes `miri_beam_search_peek_best_matching_skips_stale_leaders` and `miri_beam_search_peek_best_matching_returns_none_after_dropping_fully_stale_frontier`. |
| `select_next_with_refill` | Done | Packet 008 promotes `miri_visible_frontier_select_next_with_refill_skips_until_selected_then_advances`. |
| Deterministic frontier ordering | Done | Packet 008 promotes `miri_beam_search_forget_queued_removes_frontier_node_and_allows_reseed`, covering frontier removal and reseed order. |
| cargo-careful mirror | Done | Packet 008 `careful-harness-cargo-test.log`: 69 passed, including promoted HNSW `miri_` tests. |
| Mutation probe | Done | Packet 013 `hnsw-stale-filter-inverted.patch` inverts stale-candidate matching; `mutation-hnsw-stale-filter.log` fails `miri_beam_search_peek_best_matching_skips_stale_leaders`. |

### DiskANN Vacuum

| Target | Current status | Required action |
| --- | --- | --- |
| `repair_neighbors` compaction and padding | Done | `miri_vc_006_repair_neighbors_compacts_and_pads`. |
| `repair_neighbors` encoded length | Done | `miri_vc_009_repair_preserves_encoded_length`. |
| `mark_deleted` | Done | Packet 008 promotes `miri_vc_001_mark_deleted_is_idempotent` and `miri_vc_002_mark_deleted_preserves_payload`. |
| `strip_dead_primary_heaptid` | Done | Packet 008 promotes `miri_vc_003_strip_dead_primary_heaptid_predicate` and `miri_vc_004_strip_skips_already_invalid`. |
| `is_fully_dead` | Done | Packet 008 promotes `miri_vc_005_is_fully_dead_semantics`. |
| Deletion state-machine composition | Done | Packet 008 promotes `miri_vc_010_full_deletion_state_machine`. |
| cargo-careful mirror | Done | Packet 008 `careful-harness-cargo-test.log`: 69 passed, including the promoted DiskANN vacuum tests. |
| Mutation probe | Done | Packet 013 `diskann-vacuum-fully-dead-overflow.patch` weakens the overflow-chain guard; `mutation-diskann-vacuum-fully-dead.log` fails `miri_vc_005_is_fully_dead_semantics`. |

### SPIRE Top-K / Candidate Merge

| Target | Current status | Required action |
| --- | --- | --- |
| Bounded vec-id dedupe | Done | `miri_rank_routed_leaf_rows_by_ip_keeps_bounded_best_deduped_candidates`. |
| Candidate cursor emission | Done | `miri_scan_candidate_cursor_emits_ranked_candidates_once`. |
| `scored_candidate_cmp` tie order | Done | Packet 008 promotes `miri_scored_candidate_tie_break_prefers_newer_epoch_then_primary_role`. |
| Primary-vs-boundary tie under bounded dedupe | Done | Packet 008 promotes `miri_rank_routed_leaf_rows_by_ip_keeps_primary_tie_break_under_bounded_dedupe`. |
| `rerank_scored_candidates_by_ip` prefix replacement | Done | Packet 008 promotes `miri_rerank_scored_candidates_by_ip_rescores_prefix_and_truncates` and invisible-candidate behavior. |
| Non-finite rerank rejection | Done | Packet 008 promotes `miri_rerank_scored_candidates_by_ip_rejects_non_finite_scores`. |
| cargo-careful mirror | Blocked | Packet 012 records the blocker: the covered scan/top-k tests live in the SPIRE scan test include tree and depend on SPIRE `meta`, `storage`, `quantizer`, `options`, `SpirePublishedEpochSnapshot`, `SpireObjectReader`, and pgrx-facing `ItemPointer`/OID context. Mirroring requires either extracting the comparator/rerank/bounded-merge helpers behind pgrx-free DTOs or adding a SPIRE careful micro-harness with narrow shims for those boundaries. |
| Mutation probe | Done | Packet 013 `spire-topk-epoch-tie-order.patch` reverses epoch tie ordering; `mutation-spire-topk-comparator.log` fails `miri_scored_candidate_tie_break_prefers_newer_epoch_then_primary_role`. |

### SPIRE Routing

| Target | Current status | Required action |
| --- | --- | --- |
| Root-to-leaf routing | Done | `miri_route_root_object_to_leaf_pids_keeps_bounded_best_routes`. |
| Adaptive nprobe reduction | Done | `miri_adaptive_nprobe_reduces_routing_width_when_boundary_gap_is_large`. |
| Internal routing | Done | Packet 008 promotes `miri_route_routing_object_to_child_pids_routes_internal_level`. |
| Top-graph deterministic routing | Done | Packet 008 promotes `miri_route_top_graph_to_child_pids_uses_graph_frontier_with_deterministic_routes`. |
| Recursive routing to leaf level | Done | Packet 008 promotes `miri_route_recursive_routing_objects_to_leaf_pids_descends_to_leaf_level` and conservative upper-level nprobe coverage. |
| Route rejection paths | Done | Packet 008 promotes root mismatch, internal-parent, missing-child, and wrong-child-level rejection tests. |
| cargo-careful mirror | Blocked | Packet 012 records the blocker: routing helpers depend on the SPIRE scan/meta/storage/top-graph object reader contracts and pgrx-facing tuple identity types. Mirroring requires extraction of the route-ranking/adaptive-nprobe core behind pgrx-free graph/object DTOs or a SPIRE careful micro-harness that shims only the object-reader and `ItemPointer` boundary. |
| Mutation probe | Done | Packet 013 `spire-routing-adaptive-nprobe.patch` keeps the requested nprobe despite a large score gap; `mutation-spire-routing-adaptive-nprobe.log` fails `miri_adaptive_nprobe_reduces_routing_width_when_boundary_gap_is_large`. |

### SPIRE Vacuum / Delete-Delta Visibility

| Target | Current status | Required action |
| --- | --- | --- |
| Delete-delta grouping | Done | Packet 010 promotes `miri_delta_epoch_draft_from_snapshot_carries_base_entries` and `miri_replacement_leaf_rows_fold_active_deltas_into_base_leaf_rows`. |
| Visible assignments exclude delete-delta targets | Done | Packet 010 adds `miri_collect_visible_assignments_excludes_delete_delta_targets`. |
| Duplicate/stale delete target rejection | Done | Packet 010 promotes unknown, mismatched, stale, duplicate, and already-deleted delete-target rejection tests. |
| cargo-careful mirror | Blocked | Packet 012 confirms the blocker: the covered SPIRE vacuum/update paths depend on the SPIRE storage/meta/update modules and pgrx-facing crate context. Mirroring requires a SPIRE-focused careful harness with pgrx-free shims for the local object store, manifest types, and `ItemPointer` boundary. |
| Mutation probe | Done | Packet 013 `spire-vacuum-delete-filter.patch` disables delete-delta filtering for leaf and delta assignments; `mutation-spire-vacuum-delete-filter.log` fails `miri_collect_visible_assignments_excludes_delete_delta_targets`. |

### Remote Parser / Typed Payload

| Target | Current status | Required action |
| --- | --- | --- |
| Row byte cap | Done | `miri_remote_payload_caps_reject_oversized_rows_and_batches`. |
| Batch row cap | Done | Same test. |
| Valid typed payload hex byte count | Done | Packet 009 `miri_remote_typed_payload_fields_accept_valid_row_independent_payload`. |
| Odd-length hex rejection | Done | Packet 009 `miri_remote_typed_payload_fields_reject_adversarial_shapes`. |
| Invalid hex rejection | Done | Packet 009 `miri_remote_typed_payload_fields_reject_adversarial_shapes`. |
| Payload width mismatch | Done | Packet 009 covers payload vector and collation width mismatches before Row decoding. |
| OID / collation / format / transport constraints | Done | Packet 009 covers invalid type OID, invalid collation OID, non-ready transport status, unsupported transport, and bad per-column format. |
| cargo-careful mirror | Blocked | Packet 012 confirms the blocker: the extracted helper still lives inside the pgrx SPIRE coordinator include module and returns `pg_sys::Oid` values in `SpireRemoteTypedTuplePayload`. Careful mirroring requires either a pgrx-free OID newtype/adapter or a remote-payload micro-harness that shims only this type boundary. |
| Mutation probe | Done | Packet 013 `remote-typed-payload-cap.patch` bypasses the typed-payload row byte limit; `mutation-remote-typed-payload-cap.log` fails `miri_remote_typed_payload_fields_reject_adversarial_shapes` after an over-cap tuple is accepted. |

### Serialization / Layout

| Target | Current status | Required action |
| --- | --- | --- |
| Storage page / `ItemPointer` | Done | Existing `miri_` storage tests. |
| DiskANN metadata page | Done | Existing `miri_vamana_metadata_roundtrip_with_codebook_head`. |
| DiskANN node tuple | Done | `miri_la_011_filled_node_roundtrip`. |
| DiskANN codebook tuple | Done | `miri_la_030_codebook_tuple_roundtrip`. |
| HNSW page tuples | Done | Existing HNSW page `miri_` tests. |
| SPIRE leaf V2 | Done | Existing leaf V2 `miri_` tests. |
| SPIRE top graph | Done | `miri_top_graph_partition_object_round_trips_nodes`. |
| SPIRE delta / assignment / vec-id helpers | Done | Packet 011 promotes assignment row, delta partition object, and vec-id helper tests under Miri. |
| SPIRE serialization cargo-careful mirror | Blocked | Packet 012 records the blocker: the SPIRE serialization tests live behind the SPIRE storage/meta/update include graph and share pgrx-facing tuple identity and object-manifest types. Mirroring requires a SPIRE careful micro-harness or extraction of the assignment/delta/vec-id codecs behind pgrx-free DTOs. |
| Mutation probe | Done | Packet 013 `spire-serialization-delta-duplicate-vec-id.patch` disables duplicate vec-id rejection for delta objects; `mutation-spire-serialization-delta-duplicate.log` fails `miri_delta_partition_object_rejects_duplicate_vec_ids`. |

## Validation Matrix

Each implementation packet must choose the narrowest commands that prove its
claims, then the final campaign packet must run the aggregate matrix.

| Lane | Required for final campaign | Artifact expectation |
| --- | --- | --- |
| `cargo fmt --all -- --check` | Yes | Formatting log or explicit clean output. |
| `git diff --check` | Yes | Clean whitespace log. |
| Targeted default Miri per new subsystem | Yes | One log per subsystem/test group. |
| Targeted Tree Borrows per new subsystem | Yes for risky aliasing/concurrency additions | Separate log from default Miri. |
| Targeted many-seeds threaded test | Yes | `MIRI_MANY_SEEDS=0..128` log for real threaded/atomic test. |
| `make miri-tree` | Yes | Aggregate Tree Borrows log. |
| `make miri-many-seeds` | Yes | Aggregate many-seeds log after threaded test is included. |
| `cargo test --manifest-path hardening/careful/Cargo.toml --lib` | Yes | Careful harness normal Rust count. |
| `make careful` | Yes | cargo-careful count. |
| Mutation probes | Yes | Patch, failing command, failure excerpt, and restore note. |

## Planned Packets

| Packet | Purpose | Status |
| --- | --- | --- |
| `006-safety-campaign-tracker` | Install this tracker and commit reviewer feedback. | Done |
| `007-real-many-seeds-parallel-state` | Add threaded/atomic Miri coverage for real common parallel shared state. | Done |
| `008-breadth-closure-existing-tests` | Promote existing small pure tests named in packet 001 across DiskANN, HNSW, SPIRE top-k, routing, and vacuum. | Done |
| `009-remote-parser-extraction` | Extract/test Row-independent typed payload parser validation. | Done |
| `010-spire-vacuum-delete-delta` | Cover SPIRE delete-delta/vacuum visibility or produce precise blocker/extraction plan. | Done |
| `011-spire-serialization-layout` | Close the remaining SPIRE delta / assignment / vec-id serialization/layout breadth row. | Done |
| `012-careful-mirroring` | Mirror path-liftable new Miri surfaces in `hardening/careful`; document blockers. | Done |
| `013-mutation-probes` | Run mutation/sensitivity probes for each major subsystem. | Done |
| `014-final-campaign-audit` | Run aggregate lanes and map every gate/finding to evidence. | Not started |

## Reviewer Feedback Disposition

| Feedback | Tracker disposition |
| --- | --- |
| Many-seeds is structurally empty. | G2 and packet 007 require real threaded/atomic Miri coverage. |
| Strategy had no numeric/breadth gate. | G3 and subsystem matrix require every named pure candidate to be covered or blocked. |
| HNSW graph coverage too narrow. | HNSW matrix requires stale removal, refill, and deterministic ordering. |
| DiskANN graph build helpers skipped. | DiskANN graph matrix requires both build helper tests. |
| DiskANN vacuum short helpers skipped. | DiskANN vacuum matrix requires mark/strip/fully-dead/state-machine tests. |
| SPIRE top-k comparator/rerank skipped. | SPIRE top-k matrix requires comparator, tie-break, rerank, rejection tests. |
| SPIRE vacuum has zero Miri coverage. | G5 and packet 010 require coverage or a precise extraction blocker. |
| Remote parser only covers caps. | G4 and packet 009 require Row-independent parser extraction and adversarial tests. |
| No mutation/bug-injection verification. | G7 and packet 012 require mutation probes per major subsystem. |
| Careful coverage claim too broad. | G6 requires mirroring path-liftable surfaces and explicit blockers for the rest. |

## Completion Rule

Task 43 may be called complete only when:

- every completion gate is **Done** or **Blocked** with a concrete extraction
  task,
- every subsystem matrix row is **Done** or **Blocked**,
- every reviewer finding has evidence or a blocker,
- packet-local artifacts support all claims,
- the final campaign audit is committed and pushed.
