# Task 50 Packet 008: SPIRE Active Anchor Seed

## Scope

- Task: `plan/tasks/50-unsafe-structural-reduction.md`.
- Slice: 3a, SPIRE active-epoch anchor seed.
- Code commit: `3e089ada3276c64d1bd8bcf221fcbb583ac759de`.

This packet introduces a small `SpireLiveIndexRelation` wrapper and `SpireActiveEpochAnchor` in `src/am/ec_spire/coordinator/snapshots.rs`.

The wrapper centralizes the repeated live SPIRE index relation invariants for:

- root/control reads;
- relation option reads;
- active epoch manifest loading;
- relation-backed object-store opening.

Two active-epoch loaders are intentionally kept distinct:

- local diagnostic snapshots use the existing `scan::load_relation_epoch_manifests` path, preserving local-store config and local heap delivery validation;
- coordinator fanout diagnostics keep the existing `load_relation_epoch_manifests_for_coordinator_fanout` path.

## Touched Surface

Converted these SPIRE snapshot helpers to consume the wrapper/anchor:

- `active_snapshot_diagnostics`
- `active_epoch`
- `index_allocator_snapshot`
- `index_options_snapshot`
- `index_writer_identity_snapshot`
- `index_level_parameter_snapshot`
- `index_scan_sanity_snapshot`
- `index_placement_snapshot`
- `remote_node_snapshot`

The packet also adds local type aliases around physical cleanup candidate tuple refs so current clippy no longer reports a touched-file `type_complexity` diagnostic after rustfmt.

## Unsafe Count

| File | Before | After | Delta |
| --- | ---: | ---: | ---: |
| `src/am/ec_spire/coordinator/snapshots.rs` | 62 | 52 | -10 |

This is a 16.1% reduction for the seed slice. It does not claim to complete the SPIRE 30% target; follow-on SPIRE slices should continue into hierarchy snapshots and DML/frontdoor surfaces.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench`: passed.
- `rustfmt --check src/am/ec_spire/coordinator/snapshots.rs`: passed.
- `git diff --check`: passed.
- `cargo fmt --all --check`: failed on existing repo-wide formatting drift outside the touched file.
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`: failed on existing repo-wide lint backlog; no diagnostics target `src/am/ec_spire/coordinator/snapshots.rs`.
- Focused runtime attempt: `cargo test --lib --no-default-features --features pg18,bench am::ec_spire::scan::tests::runtime_state::local_heap_delivery_gate -- --nocapture` built, then failed at process start with `undefined symbol: LockBuffer`.

Artifacts are under `reviews/task-50/008-spire-active-anchor-seed/artifacts/`.

## Review Notes

Please focus on whether the anchor split preserves the prior validation semantics:

- local snapshot helpers should continue to get local-store config validation through `scan::load_relation_epoch_manifests`;
- fanout-oriented helpers should continue using `load_relation_epoch_manifests_for_coordinator_fanout`;
- `object_store_set` should remain a thin wrapper around the existing relation-backed store opening, with lock mode still explicit at each call site.
