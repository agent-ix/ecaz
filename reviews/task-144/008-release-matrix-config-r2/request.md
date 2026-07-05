# Review Request: Task 144 Packet 008 - Release Matrix Config R2

## Scope

This packet addresses the actionable feedback from packets 006 and 007 before
running the 50k/100k Task 144 matrix.

Code checkpoint under review:

- `a7857ae9f bench: add suite truth-cache and SPIRE scan reporting`

Packet-local config/evidence:

- `artifacts/suite-task144-release-matrix-r2.json`
- `artifacts/suite-audit.log`
- `artifacts/suite-dry-run.log`
- `artifacts/dry-run-suite-manifest.json`
- `artifacts/cargo-test-ecaz-cli-suite.log`
- `artifacts/cargo-test-ecaz-cli-storage.log`

## Reviewer Feedback Addressed

Packet 006 F1: ratio axis was a single dead point.

- R2 config expands ratio pruning to `1.25 / 2.0 / 4.0 / 8.0`.
- `1.25` remains as the packet 007 over-pruning baseline.

Packet 006 F2 / packet 005 F1: closure axis and replica reporting were too thin.

- R2 config keeps `closure_e010_b8` and adds `closure_e025_b8` and
  `closure_e050_b8`.
- `bench storage` now emits SPIRE `mean_replicas_per_vector` from
  `ec_spire_index_health_snapshot`, so storage steps report replication factor
  directly rather than only bytes.

Packet 006 F3 / packet 007 reviewer note: truth cache was manual.

- R2 config adds one `recall` truth-cache step per scale.
- Suite audit now treats those cache files as produced artifacts and treats
  downstream `spire-pipeline` truth cache paths as inputs, so clean runs fail
  audit unless the dependency chain exists.

Packet 006 F4 / packet 007 reporting gap: bespoke sweep and AC metric.

- R2 config documents why `[8,16,32,64,96]` is used instead of the registered
  default `[8,16,24,32]`.
- Suite results now derive `spire_pipeline_row_scan` rows with
  `candidate_row_instances_percent` and `ready_row_instances_percent`.

## Validation

- `cargo test -p ecaz-cli suite`: 62 passed.
- `cargo test -p ecaz-cli storage`: 13 passed.
- `ecaz bench suite audit`: passed for 124 steps.
- `ecaz bench suite run --dry-run`: produced a 124-step dry-run manifest with
  3 truth-cache steps, 15 storage steps, and 90 `spire-pipeline` steps.

## Notes

This is not Task 144 closeout. It is the corrected runner/config slice that
unblocks the release 50k/100k evidence run without knowingly re-running the
under-sampled packet 006 matrix.
