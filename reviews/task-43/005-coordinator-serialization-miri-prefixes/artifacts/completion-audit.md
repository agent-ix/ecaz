# Completion Audit: Task 43 Miri / Cargo-Careful Depth

Audit target: `plan/tasks/43-miri-careful-depth.md`

Audited head: `e811f3093b8b752ec53c29f802b031366723958b`

## Exit Criteria

### Stacked Borrows And Tree Borrows Are In The Nightly Local Lane

Satisfied.

- `Makefile` has `miri-full`, `miri-tree`, `miri-many-seeds`, and `careful`
  targets.
- `hardening-nightly-local` depends on `miri-full careful`.
- `scripts/hardening.sh miri-full` runs the default Miri prefix, then
  `miri-tree`, then `miri-many-seeds`.
- `make-miri-tree.log` records the Tree Borrows lane passing:
  `35 passed; 0 failed; 1756 filtered out`.

### Many-Seeds Run Is Included

Satisfied.

- `scripts/hardening.sh miri-many-seeds` now exports
  `-Zmiri-many-seeds=0..128`.
- `docs/hardening.md` documents the same range syntax and override format.
- `make-miri-many-seeds.log` records 128 `Trying seed:` entries, ends with
  `COMMAND_EXIT_CODE="0"`, and its completed batches report
  `35 passed; 0 failed; 1756 filtered out`.

No current `miri_` test was identified that creates real threads or atomics
inside Miri. The lane still runs the full current `miri_` prefix across the
seed range, so future threaded `miri_` tests are automatically included.

### Pure-Rust Miri Coverage Spans Major Subsystems

Satisfied.

- SPIRE coordinator: `miri_production_executor_state_moves_ready_transport_to_candidate_receive`
  and `miri_prepared_transaction_intent_transitions_cannot_bypass_prepare_ack`.
- DiskANN graph: `miri_greedy_search_finds_nearest` and
  `miri_robust_prune_excludes_alpha_dominated`.
- HNSW graph: `miri_beam_search_deduplicates_self_loops_and_parallel_edges`
  and `miri_visible_frontier_best_candidate_prefers_live_scheduler_node`.
- Top-k merge / candidate priority: `miri_rank_routed_leaf_rows_by_ip_keeps_bounded_best_deduped_candidates`
  and `miri_scan_candidate_cursor_emits_ranked_candidates_once`.
- Remote parser / payload caps: `miri_remote_payload_caps_reject_oversized_rows_and_batches`.
- Vacuum: `miri_vc_006_repair_neighbors_compacts_and_pads` and
  `miri_vc_009_repair_preserves_encoded_length`.
- Serialization / layout helpers: storage page, DiskANN metadata, DiskANN
  tuple/codebook, HNSW page tuples, SPIRE leaf V2, and SPIRE top graph.
- Quant pure helpers remain in the Miri prefix and are included in the 35-test
  aggregate lane.

### Cargo-Careful Expansion

Satisfied for the path-liftable pure harness.

- `hardening/careful` now path-lifts storage page, DiskANN tuple, DiskANN
  vacuum, DiskANN Vamana graph, and HNSW search modules.
- `cargo-test-careful-harness.log` records `67 passed; 0 failed`.
- `make-careful.log` records the same 67-test set under `cargo-careful`.

The harness intentionally excludes pgrx callback, PostgreSQL memory-context,
SPI, and libpq paths. Those are outside cargo-careful's pure-Rust scope and
remain in PG18 or live-cluster validation.

### Documentation

Satisfied.

- `docs/hardening.md` documents `miri-expanded`, `miri-tree`,
  `miri-many-seeds`, `miri-full`, and `make careful`.
- The docs describe the Tree Borrows/default comparison and how to triage
  model disagreements.
- The docs list the current seeded Miri subsystem coverage.

## Residual Risk

- The task file says many-seeds explores concurrent schedules. The repo now has
  the lane wired and validated, but no current `miri_` test appears to exercise
  an actual threaded/atomic schedule under Miri.
- The careful harness demonstrates 67 lifted pure tests. It is not a claim that
  cargo-careful can run every test in the pgrx-heavy main crate.
