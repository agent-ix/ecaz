# Review Request: C1 ADR-030 V2 Experimental Build Gate

## Context

Packet `320` added a reusable builder flush-output seam for both:

1. current-format scalar build output
2. grouped v2 build output

The next narrow slice is to expose a real end-to-end alternate build lane behind an explicit
internal gate, while keeping the normal build path unchanged.

## Problem

We can now assemble grouped v2 build output, but there is still no end-to-end builder path that can
use it during `ambuild`.

That means there is still no way to run a real experimental grouped v2 rebuild without hand wiring
internal helpers.

## Planned Slice

Add a default-off experimental build gate that:

1. only activates when an explicit environment variable is set
2. only activates for source-backed builds
3. plans grouped v2 output and flushes it through the shared builder writer

This slice still excludes:

- no runtime grouped scan support
- no user-visible reloption or SQL surface
- no default build-path switch

## Implementation

Added a default-off internal build gate for experimental ADR-030 v2 rebuilds.

New internal constants:

- grouped search subvector size
- grouped training cap
- grouped k-means iteration count
- environment variable name:
  - `TQVECTOR_EXPERIMENTAL_ADR030_V2_BUILD`

New helpers:

- `experimental_grouped_v2_build_enabled()`
- `experimental_grouped_v2_flush_output(...)`

Build-path change:

- `flush_build_state(...)` now checks the env var and only switches to the grouped v2 builder lane
  when both conditions are true:
  1. the env var is set
  2. the build is source-backed via `build_source_column`

If either condition is false, the existing current-format scalar build path remains unchanged.

Grouped experimental behavior:

1. plan grouped v2 output from source-backed build state
2. assemble grouped v2 metadata and staged pages through the shared flush-output seam
3. flush through the same writer used by the current format

Test added:

- validates that the experimental grouped helper uses the intended default v2 parameters and emits
  grouped-format metadata

This is the first packet where ADR-030 v2 has a real end-to-end alternate rebuild lane inside
`ambuild`, even though it is still explicitly gated off by default.

## Measurements

This packet is still a build-path slice, so there are no new recall or latency measurements.

Known validation results for this attempt:

- `cargo test experimental_grouped_v2_flush_output_uses_default_v2_parameters --lib`: passed
- `cargo clippy --lib --tests -- -D warnings`: passed
- `cargo test`: passed
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`: passed
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`: passed

## Outcome

ADR-030 v2 now has a real experimental rebuild switch that can run end to end without disturbing the
default builder behavior.

What this de-risks:

1. the alternate grouped-v2 builder lane now exists in production build code, not only test seams
2. the default current-format build path remains stable when the gate is off
3. the next slice can focus on inspecting or validating real v2 on-disk output rather than wiring
   more builder plumbing

## Next Slice

The next narrow slice should validate the gated grouped-v2 rebuild output directly:

1. build an index under the internal gate on a source-backed fixture
2. inspect metadata and tuple tags from raw index pages
3. verify that the on-disk output is truly `v2 grouped` rather than only internally assembled
