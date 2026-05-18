# Task 43 Packet 013 Artifact Manifest

- Head SHA: `c389a5cdec4ecd3ab02239107ad27d804ce35591`
- Task bucket: `reviews/task-43/`
- Packet: `reviews/task-43/013-mutation-probes/`
- Timestamp: `2026-05-18T20:50:12Z`
- Lane: mutation / sensitivity probes
- Storage format / rerank mode:
  Pure Rust Miri unit tests only; no PostgreSQL table, index, storage fixture,
  or rerank runtime table was created.
- Shared-table surface:
  Not applicable.

## Mutation Matrix

| Subsystem | Patch | Command | Expected failure |
| --- | --- | --- | --- |
| Common parallel shared state | `patches/common-parallel-claim-expected.patch` | `script -q -c 'cargo +nightly miri test --lib miri_parallel_worker_slots_are_unique_under_threaded_contention' reviews/task-43/013-mutation-probes/artifacts/mutation-common-parallel.log` | `0 passed; 1 failed`; assertion saw 0 live claims instead of 3. |
| DiskANN graph | `patches/diskann-robust-prune-alpha.patch` | `script -q -c 'cargo +nightly miri test --lib miri_robust_prune_excludes_alpha_dominated' reviews/task-43/013-mutation-probes/artifacts/mutation-diskann-robust-prune.log` | `0 passed; 1 failed`; alpha-dominated candidates were retained. |
| HNSW graph | `patches/hnsw-stale-filter-inverted.patch` | `script -q -c 'cargo +nightly miri test --lib miri_beam_search_peek_best_matching_skips_stale_leaders' reviews/task-43/013-mutation-probes/artifacts/mutation-hnsw-stale-filter.log` | `0 passed; 1 failed`; stale leader was returned instead of the live candidate. |
| DiskANN vacuum | `patches/diskann-vacuum-fully-dead-overflow.patch` | `script -q -c 'cargo +nightly miri test --lib miri_vc_005_is_fully_dead_semantics' reviews/task-43/013-mutation-probes/artifacts/mutation-diskann-vacuum-fully-dead.log` | `0 passed; 1 failed`; overflow-chain tuple was incorrectly treated as fully dead. |
| SPIRE top-k / candidate merge | `patches/spire-topk-epoch-tie-order.patch` | `script -q -c 'cargo +nightly miri test --lib miri_scored_candidate_tie_break_prefers_newer_epoch_then_primary_role' reviews/task-43/013-mutation-probes/artifacts/mutation-spire-topk-comparator.log` | `0 passed; 1 failed`; older epoch won the tie. |
| SPIRE routing | `patches/spire-routing-adaptive-nprobe.patch` | `script -q -c 'cargo +nightly miri test --lib miri_adaptive_nprobe_reduces_routing_width_when_boundary_gap_is_large' reviews/task-43/013-mutation-probes/artifacts/mutation-spire-routing-adaptive-nprobe.log` | `0 passed; 1 failed`; effective nprobe remained 4 instead of reducing to 2. |
| SPIRE vacuum / delete-delta | `patches/spire-vacuum-delete-filter.patch` | `script -q -c 'cargo +nightly miri test --lib miri_collect_visible_assignments_excludes_delete_delta_targets' reviews/task-43/013-mutation-probes/artifacts/mutation-spire-vacuum-delete-filter.log` | `0 passed; 1 failed`; 3 visible assignments were returned instead of 2. |
| Remote typed payload | `patches/remote-typed-payload-cap.patch` | `script -q -c 'cargo +nightly miri test --lib miri_remote_typed_payload_fields_reject_adversarial_shapes' reviews/task-43/013-mutation-probes/artifacts/mutation-remote-typed-payload-cap.log` | `0 passed; 1 failed`; over-cap typed payload decoded successfully. |
| SPIRE serialization / layout | `patches/spire-serialization-delta-duplicate-vec-id.patch` | `script -q -c 'cargo +nightly miri test --lib miri_delta_partition_object_rejects_duplicate_vec_ids' reviews/task-43/013-mutation-probes/artifacts/mutation-spire-serialization-delta-duplicate.log` | `0 passed; 1 failed`; duplicate vec-id delta object was accepted. |

Every mutation was temporary. The source file was restored after the failing
Miri run and before this packet was assembled.

## Artifact Inventory

- `mutation-common-parallel.log`
- `mutation-diskann-robust-prune.log`
- `mutation-hnsw-stale-filter.log`
- `mutation-diskann-vacuum-fully-dead.log`
- `mutation-spire-topk-comparator.log`
- `mutation-spire-routing-adaptive-nprobe.log`
- `mutation-spire-vacuum-delete-filter.log`
- `mutation-remote-typed-payload-cap.log`
- `mutation-spire-serialization-delta-duplicate.log`
- `patches/*.patch`
- `source-status-after-probes.log`
- `git-status-after-probes.log`
- `git-diff-check.log`
- `cargo-fmt-check.log`

## Restoration Rule

The patch files are evidence only. They are not production changes and must not
be applied outside a mutation-probe review. The committed tree should contain
only this packet and the campaign tracker update.

`source-status-after-probes.log` records an empty `git status --short src
hardening docs`, proving the temporary production/source/doc mutations were
restored before the packet was assembled. `git-status-after-probes.log` records
only the expected tracker modification and new packet files.

## Validation

- `git-diff-check.log`:
  `script -q -c 'git diff --check' reviews/task-43/013-mutation-probes/artifacts/git-diff-check.log`
  exited 0.
- `cargo-fmt-check.log`:
  `script -q -c 'cargo fmt --all -- --check' reviews/task-43/013-mutation-probes/artifacts/cargo-fmt-check.log`
  exited 0. The log contains existing stable-rustfmt warnings for unstable
  import-grouping settings.
