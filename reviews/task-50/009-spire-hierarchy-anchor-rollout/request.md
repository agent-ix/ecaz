# Task 50 Packet 009: SPIRE Hierarchy Anchor Rollout

## Scope

- Task: `plan/tasks/50-unsafe-structural-reduction.md`.
- Slice: 3b, SPIRE hierarchy/read-path active-anchor rollout.
- Code commit: `5ede3fe2bb34abc03d1920bedb8c6464722ff74a`.

This packet reuses the Packet 008 `SpireLiveIndexRelation` and `SpireActiveEpochAnchor` helpers from `coordinator/snapshots.rs` in `src/am/ec_spire/coordinator/hierarchy_snapshots.rs`.

Converted active epoch/root-control/object-store reads in:

- `remote_search_candidates_result`
- `remote_search_coordinator_local_candidates_result`
- `remote_search_coordinator_local_result_summary`
- `remote_search_coordinator_local_summary_result`
- `index_top_graph_snapshot`
- `index_hierarchy_snapshot`
- `index_object_snapshot`
- `index_delta_snapshot`
- `index_scan_placement_snapshot`

The rollout preserves the prior loader split:

- local scan/object/delta/top-graph/scan-placement surfaces use `active_epoch_anchor`, which preserves `scan::load_relation_epoch_manifests` validation;
- coordinator fanout/local summary surfaces use `coordinator_fanout_anchor`, matching their previous fanout loader.

## Unsafe Count

| File | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_spire/coordinator/hierarchy_snapshots.rs` | 71 | 48 | -23 |

This is a 32.4% reduction, so this top-15 SPIRE file now meets Task 50's 30% per-file reduction target.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench`: passed.
- `rustfmt --check src/am/ec_spire/coordinator/hierarchy_snapshots.rs`: passed.
- `git diff --check`: passed.
- `cargo fmt --all --check`: failed on existing repo-wide formatting drift outside the touched file.
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`: failed on existing repo-wide lint backlog; no diagnostics target `src/am/ec_spire/coordinator/hierarchy_snapshots.rs`.
- Focused runtime attempt: `cargo test --lib --no-default-features --features pg18,bench am::ec_spire::coordinator:: -- --nocapture` built, then failed at process start with `undefined symbol: LockBuffer`.

Artifacts are under `reviews/task-50/009-spire-hierarchy-anchor-rollout/artifacts/`.

## Review Notes

Please focus on the loader-choice preservation in each converted function. The intended invariant is:

- local heap-delivery paths still reject remote placement directories through the full scan loader;
- coordinator fanout diagnostic paths still allow the fanout placement view;
- lock mode remains explicit at `object_store_set(..., AccessShareLock)` call sites.
