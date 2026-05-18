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
| G2 | `miri-many-seeds` includes at least one real threaded/atomic `miri_` test. | Not done | Add actual concurrent coverage, preferably in `src/am/common/parallel.rs`. |
| G3 | All strategy-named pure subsystem candidates are covered or precisely blocked. | Partial | See subsystem matrix below. |
| G4 | Remote parser coverage includes Row-independent typed payload validation, not only byte caps. | Not done | Extract/test parser helpers from remote candidate payload handling. |
| G5 | SPIRE vacuum/delete-delta visibility has Miri coverage or a precise extraction blocker. | Not done | Cover pure object-store/delete-delta tests or document exact blocker. |
| G6 | cargo-careful mirrors every path-liftable Miri surface. | Partial | Current careful harness covers 67 storage/DiskANN/HNSW tests; new surfaces need mirroring or blockers. |
| G7 | Mutation/sensitivity probes exist for each major subsystem. | Not done | Add temporary diffs and failure logs by subsystem. |
| G8 | Final audit maps task-file requirements and reviewer findings to evidence. | Not done | Produce final packet only after G2-G7 are complete. |

## Existing Evidence Baseline

| Packet | What it proves | What it does not prove |
| --- | --- | --- |
| `001-coverage-survey-strategy` | Initial inventory and this tracker. | Completion; any code behavior. |
| `002-miri-depth-lanes` | Lane wiring: `miri-tree`, `miri-many-seeds`, `miri-full`, hardening docs. | Real many-seeds interleaving. |
| `003-pure-graph-miri-prefixes` | First DiskANN/HNSW graph tests pass under Miri. | Full graph-helper breadth. |
| `004-spire-vacuum-miri-prefixes` | First DiskANN vacuum and SPIRE top-k tests pass under Miri. | Full vacuum/top-k breadth; SPIRE vacuum. |
| `005-coordinator-serialization-miri-prefixes` | 35-test aggregate, Tree Borrows, many-seeds execution, careful 67-test harness, coordinator/serialization additions. | Real concurrent many-seeds coverage; mutation probes; full strategy breadth. |

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
| Real many-seeds interleaving | Not done | Add threaded/atomic `miri_` test and run targeted many-seeds plus aggregate many-seeds. |
| SB/TB disagreement triage runbook | Partial | Docs exist; no real disagreement to triage. Future disagreement must produce paired logs and classification. |

### Common Parallel Shared State

| Target | Current status | Required action |
| --- | --- | --- |
| Worker slot claim/release atomics | Not done | Add `miri_` test around real `EcParallelCoordinatorState` / `EcParallelWorkerSlot` claim and release paths. |
| Concurrent slot contention | Not done | Spawn multiple Rust threads contending for real slots; assert unique claims and correct claim counts. |
| Rescan/epoch stale-publish rejection under contention | Not done | Add concurrent or sequenced Miri coverage where stale publishes fail benignly. |
| Many-seeds coverage | Not done | Run `MIRI_MANY_SEEDS=0..128` against the threaded test and aggregate lane. |
| Mutation probe | Not done | Temporarily weaken compare-exchange or claim counter update and prove the test fails. |

### DiskANN Graph

| Target | Current status | Required action |
| --- | --- | --- |
| `robust_prune` alpha dominance | Done | `miri_robust_prune_excludes_alpha_dominated`. |
| `greedy_search` convergence | Done | `miri_greedy_search_finds_nearest`. |
| `build_vamana_graph_with_stats` | Not done | Promote/add bounded `miri_` build test, likely smaller than the existing 100-node test if needed. |
| `build_vamana_graph_with_pass1_extra_candidates` | Not done | Promote/add bounded `miri_` test proving pass-1 extras enlarge candidate pools. |
| cargo-careful mirror | Partial | Current careful harness path-lifts Vamana and runs existing build tests; mirror any new Miri additions explicitly. |
| Mutation probe | Not done | Break alpha dominance or pass-1 candidate injection and record failure. |

### HNSW Graph

| Target | Current status | Required action |
| --- | --- | --- |
| `BeamSearch` dedupe / best-first path | Done | `miri_beam_search_deduplicates_self_loops_and_parallel_edges`. |
| `VisibleFrontier` live scheduler preference | Done | `miri_visible_frontier_best_candidate_prefers_live_scheduler_node`. |
| Stale-candidate removal | Not done | Promote stale-frontier tests, e.g. `beam_search_peek_best_matching_skips_stale_leaders` and fully stale frontier case. |
| `select_next_with_refill` | Not done | Promote `visible_frontier_select_next_with_refill_skips_until_selected_then_advances`. |
| Deterministic frontier ordering | Not done | Promote or add bounded order test covering score/tie/sequence behavior. |
| cargo-careful mirror | Partial | HNSW search is already path-lifted; ensure new promoted tests are in careful count. |
| Mutation probe | Not done | Break stale filtering/refill advancement and record failure. |

### DiskANN Vacuum

| Target | Current status | Required action |
| --- | --- | --- |
| `repair_neighbors` compaction and padding | Done | `miri_vc_006_repair_neighbors_compacts_and_pads`. |
| `repair_neighbors` encoded length | Done | `miri_vc_009_repair_preserves_encoded_length`. |
| `mark_deleted` | Not done | Promote `vc_001_mark_deleted_is_idempotent` and/or payload preservation. |
| `strip_dead_primary_heaptid` | Not done | Promote predicate and already-invalid behavior. |
| `is_fully_dead` | Not done | Promote overflow/live primary semantics. |
| Deletion state-machine composition | Not done | Promote `vc_010_full_deletion_state_machine`. |
| cargo-careful mirror | Partial | Vacuum is already path-lifted; ensure promoted tests are in careful count. |
| Mutation probe | Not done | Break fully-dead overflow guard or encoded-length repair and record failure. |

### SPIRE Top-K / Candidate Merge

| Target | Current status | Required action |
| --- | --- | --- |
| Bounded vec-id dedupe | Done | `miri_rank_routed_leaf_rows_by_ip_keeps_bounded_best_deduped_candidates`. |
| Candidate cursor emission | Done | `miri_scan_candidate_cursor_emits_ranked_candidates_once`. |
| `scored_candidate_cmp` tie order | Not done | Promote `scored_candidate_tie_break_prefers_newer_epoch_then_primary_role`. |
| Primary-vs-boundary tie under bounded dedupe | Not done | Promote `rank_routed_leaf_rows_by_ip_keeps_primary_tie_break_under_bounded_dedupe`. |
| `rerank_scored_candidates_by_ip` prefix replacement | Not done | Promote prefix/truncate and invisible-candidate tests. |
| Non-finite rerank rejection | Not done | Promote rejection test. |
| cargo-careful mirror | Not done | Determine if scan candidate helpers can be path-lifted; otherwise document blocker and extraction plan. |
| Mutation probe | Not done | Invert comparator or skip rerank score replacement and record failure. |

### SPIRE Routing

| Target | Current status | Required action |
| --- | --- | --- |
| Root-to-leaf routing | Done | `miri_route_root_object_to_leaf_pids_keeps_bounded_best_routes`. |
| Adaptive nprobe reduction | Done | `miri_adaptive_nprobe_reduces_routing_width_when_boundary_gap_is_large`. |
| Internal routing | Not done | Promote bounded internal routing test. |
| Top-graph deterministic routing | Not done | Promote deterministic top-graph route test. |
| Recursive routing to leaf level | Not done | Promote bounded recursive descent test. |
| Route rejection paths | Not done | Promote at least one missing/wrong-child rejection if Miri cost allows. |
| cargo-careful mirror | Not done | Determine path-lift feasibility or extract pure routing helpers. |
| Mutation probe | Not done | Corrupt route ordering or budget dedupe and record failure. |

### SPIRE Vacuum / Delete-Delta Visibility

| Target | Current status | Required action |
| --- | --- | --- |
| Delete-delta grouping | Not done | Promote pure test if it avoids pgrx; otherwise record exact object-store dependency. |
| Visible assignments exclude delete-delta targets | Not done | Promote pure visible-row filtering test or extract helper. |
| Duplicate/stale delete target rejection | Not done | Promote pure delta validation test or record exact blocker. |
| cargo-careful mirror | Not done | Mirror only if helper extraction keeps dependency closure pure. |
| Mutation probe | Not done | Break delete filtering or duplicate-target rejection and record failure. |

### Remote Parser / Typed Payload

| Target | Current status | Required action |
| --- | --- | --- |
| Row byte cap | Done | `miri_remote_payload_caps_reject_oversized_rows_and_batches`. |
| Batch row cap | Done | Same test. |
| Valid typed payload hex byte count | Partial | Existing test covers valid count only as part of cap test. |
| Odd-length hex rejection | Not done | Add/promote dedicated Miri test. |
| Invalid hex rejection | Not done | Extract validation beyond length if production parser currently relies on Row decode. |
| Payload width mismatch | Not done | Extract Row-independent typed-payload schema validation. |
| OID / collation / format / transport constraints | Not done | Extract pure descriptor validation where possible. |
| cargo-careful mirror | Not done | Mirror extracted pure helper if dependency closure permits. |
| Mutation probe | Not done | Accept invalid hex or skip cap check temporarily and record failure. |

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
| SPIRE delta / assignment / vec-id helpers | Not done | Promote pure storage tests if bounded; otherwise record blocker. |
| Mutation probe | Not done | Corrupt an encode/decode invariant and record failure. |

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
| `007-real-many-seeds-parallel-state` | Add threaded/atomic Miri coverage for real common parallel shared state. | Not started |
| `008-breadth-closure-existing-tests` | Promote existing small pure tests named in packet 001 across DiskANN, HNSW, SPIRE top-k, routing, vacuum, serialization. | Not started |
| `009-remote-parser-extraction` | Extract/test Row-independent typed payload parser validation. | Not started |
| `010-spire-vacuum-delete-delta` | Cover SPIRE delete-delta/vacuum visibility or produce precise blocker/extraction plan. | Not started |
| `011-careful-mirroring` | Mirror path-liftable new Miri surfaces in `hardening/careful`; document blockers. | Not started |
| `012-mutation-probes` | Run mutation/sensitivity probes for each major subsystem. | Not started |
| `013-final-campaign-audit` | Run aggregate lanes and map every gate/finding to evidence. | Not started |

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
