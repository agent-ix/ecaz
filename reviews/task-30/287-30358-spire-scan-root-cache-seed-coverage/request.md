# 30358 SPIRE Scan Root Cache Seed Coverage

## Request

Review the small unit-test coverage addition for scan root/control cache
seeding.

## Scope

- Extended `scan_opaque_refreshes_root_control_on_every_rescan_observation`.
- Asserted the scan opaque starts with no cached root/control state before the
  first observation.
- Updated Task 30 status.

## Behavior Covered

The existing test already verified every observation replaces the cached
root/control state, including same-epoch cursor changes and cross-epoch
changes. This slice makes the seed-from-empty branch explicit by asserting
`root_control = None` before the first observation and then checking the first
observed root/control state is stored.

## Validation

- `cargo fmt`
- `cargo test scan_opaque_refreshes_root_control_on_every_rescan_observation --no-default-features --features pg18`
- `git diff --check`
