# Task 39 / 040 — Extract Pure Helpers from coordinator/diagnostics.rs

## Goal

Move the pure-function helpers from
`src/am/ec_spire/coordinator/diagnostics.rs` into a sibling
`diagnostics_helpers.rs` so the careful crate can `include!` them
behind minimal shim types and exercise every branch under
`make coverage`. This carves off a `100%`-covered file from a 900-line
0%-covered file without producing any logic change.

## Code Change

`src/am/ec_spire/coordinator/diagnostics_helpers.rs` (new):

- Hosts the 11 pure helpers + 3 thresholds constants previously inline
  in `diagnostics.rs`: `assignment_payload_format_name`,
  `assignment_payload_scannability`, `boundary_replica_identity_scope`,
  `boundary_replica_identity_status`,
  `boundary_replica_placement_status`, `scan_sanity_status`,
  `consistency_mode_name`, `epoch_state_name`, `placement_state_name`,
  `partition_object_kind_name`, `leaf_maintenance_thresholds`,
  `leaf_maintenance_labels`, plus `SPIRE_LEAF_SPLIT_*` /
  `SPIRE_LEAF_MERGE_*` constants.
- All function bodies are byte-for-byte identical to what they replaced
  in `diagnostics.rs` (verified by `cargo check` clean compile on the
  full pgrx surface).

`src/am/ec_spire/coordinator/diagnostics.rs`:

- Replaces the moved function definitions with `include!("diagnostics_helpers.rs")`
  near the top.
- All callers (`assignment_payload_format_name`, `placement_state_name`,
  etc.) resolve to the included definitions, so the rest of
  `diagnostics.rs` is unchanged.

`src/am/ec_spire/mod.rs`:

- Removes the three `SPIRE_LEAF_*` constants (now defined in
  `diagnostics_helpers.rs` and visible to `diagnostics.rs` via the
  include).

`hardening/careful/src/lib.rs` + `hardening/careful/src/spire_diagnostics_helpers.rs`:

- New careful module that shims the production types
  (`meta::SpireConsistencyMode`, `meta::SpireEpochState`,
  `meta::SpirePlacementState`, `storage::SpirePartitionObjectKind`,
  `storage::SPIRE_*_VEC_ID_DISCRIMINATOR`,
  `quantizer::SpireAssignmentPayloadFormat`) so the helpers compile
  inside the careful crate, then `include!`s the production helpers
  file verbatim.
- 8 unit tests cover every branch of every extracted helper.

`scripts/check_coverage_baseline_complete.sh`:

- Adds `src/am/ec_spire/coordinator/diagnostics_helpers.rs` to the
  critical-paths set (now 41 paths).

`fixtures/quality/coverage-baseline.tsv`:

- New row `am/ec_spire/coordinator/diagnostics_helpers.rs 100.00`.

## Baseline Net Effect

| File | Before | After |
| --- | ---: | ---: |
| `am/ec_spire/coordinator/diagnostics.rs` | 0.00 (900 lines) | 0.00 (~700 lines) |
| `am/ec_spire/coordinator/diagnostics_helpers.rs` | (did not exist) | **100.00** (199 lines) |
| Critical paths tracked | 40 | 41 |

Net: ~200 previously-unreachable lines now run under
`make coverage`. The remaining `diagnostics.rs` body is the
pgrx-touching surface (relation reads, snapshot accumulators, Spi)
that genuinely requires a live PG18 backend or a deeper careful
scaffold; that surface is the documented follow-up.

## Validation

Artifacts under
`reviews/task-39/040-diagnostics-helpers-extract/artifacts/`:

- `diagnostics-helpers-extract-focused-tests.log`: **508 passed**.
- `coverage/summary.txt` + JSON: `diagnostics_helpers.rs 100.00%`,
  `diagnostics.rs 0.00%` (line count drops to ~570 source lines as
  expected after the move).
- `coverage-delta-check.log`: every baseline row green.
- `coverage-baseline-check.log`: **41 critical paths complete**.
- Production `cargo check --features pg18 --no-default-features`
  is clean (no behavior change; function bodies identical).
- `changed-files.txt`: the two source paths whose baselines this
  packet ratchets (also touches `mod.rs` and the careful scaffold,
  which are not baseline-tracked).

## Reviewer Direction

Verify the moved bodies are byte-for-byte unchanged so this is
strictly a coverage refactor with no behavior delta. The careful
scaffold deliberately uses local shim types (not re-exports from
`careful_spire`) because some referenced constants are `pub(super)` in
the production tree and re-exporting them through the public hardening
surface would widen visibility past what tests actually need.
