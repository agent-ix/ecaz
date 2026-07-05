# Artifact Manifest: Task 43 Coordinator And Serialization Miri Prefixes

- Head SHA: `e811f3093b8b752ec53c29f802b031366723958b`
- Task bucket: `reviews/task-43/`
- Packet: `reviews/task-43/005-coordinator-serialization-miri-prefixes/`
- Timestamp: `2026-05-18T11:39:08-07:00`
- Storage surface: N/A, pure-Rust unit and hardening lanes
- Rerank mode: N/A
- Table isolation: N/A, no PostgreSQL tables or indexes were created

## Code Checkpoints

- `507fd4f2` - promoted coordinator, remote payload, routing, DiskANN tuple,
  and SPIRE top-graph tests into the `miri_` prefix; expanded the
  `hardening/careful` harness to 67 lifted pure tests.
- `e811f309` - corrected the many-seeds lane and docs to use Miri's range
  syntax, `-Zmiri-many-seeds=0..128`.

## Validation Artifacts

| Artifact | Command | Key result |
| --- | --- | --- |
| `miri-diskann-node-tuple.log` | `cargo +nightly miri test --lib miri_la_011_filled_node_roundtrip` | passed |
| `miri-diskann-codebook-tuple.log` | `cargo +nightly miri test --lib miri_la_030_codebook_tuple_roundtrip` | passed |
| `miri-spire-top-graph.log` | `cargo +nightly miri test --lib miri_top_graph_partition_object_round_trips_nodes` | passed |
| `miri-spire-root-routing.log` | `cargo +nightly miri test --lib miri_route_root_object_to_leaf_pids_keeps_bounded_best_routes` | passed |
| `miri-spire-adaptive-nprobe.log` | `cargo +nightly miri test --lib miri_adaptive_nprobe_reduces_routing_width_when_boundary_gap_is_large` | passed |
| `miri-spire-coordinator-state.log` | `cargo +nightly miri test --lib miri_production_executor_state_moves_ready_transport_to_candidate_receive` | passed |
| `miri-spire-remote-payload-caps.log` | `cargo +nightly miri test --lib miri_remote_payload_caps_reject_oversized_rows_and_batches` | passed |
| `miri-spire-prepared-xact-state.log` | `cargo +nightly miri test --lib miri_prepared_transaction_intent_transitions_cannot_bypass_prepare_ack` | passed |
| `cargo-test-careful-harness.log` | `cargo test --manifest-path hardening/careful/Cargo.toml --lib` | 67 passed; 0 failed |
| `make-careful.log` | `bash scripts/hardening.sh cargo-careful` | 67 passed; 0 failed; doc-tests 0 passed |
| `make-miri-tree.log` | `bash scripts/hardening.sh miri-tree` | 35 passed; 0 failed; 1756 filtered out |
| `make-miri-many-seeds.log` | `bash scripts/hardening.sh miri-many-seeds` | 128 seed attempts; final status 0; each visible batch reports 35 passed, 0 failed |
| `completion-audit.md` | Manual audit against `plan/tasks/43-miri-careful-depth.md` | all exit criteria mapped to packet-local evidence |

## Notes

- `make miri-many-seeds` uses concurrent seed execution, so the log interleaves
  output from adjacent seeds. The run records 128 `Trying seed:` entries and
  ends with `COMMAND_EXIT_CODE="0"`.
- No PostgreSQL callback, SPI, libpq, or memory-context behavior is claimed by
  these artifacts. Those paths remain covered by PG18 and live-cluster lanes.
