# SPIRE Non-Test `unwrap()` / `expect()` Classification

Code checkpoint: `d216f142151d4b989c13e6d7b083e844e3d1d0c5`

The audit command was:

```text
rg -n '\.unwrap\(\)|\.expect\(' src/am/ec_spire --glob '!**/tests*'
```

The final inventory has 114 non-test hits. Two avoidable hits were replaced
with explicit control flow in the code checkpoint:

- `src/am/ec_spire/scan/relation.rs`: the remote-placement diagnostic now
  returns an internal consistency error if the pre-count and iterator disagree,
  instead of panicking while forming the user-facing error.
- `src/am/ec_spire/scan/candidates.rs`: bounded vec-id dedupe eviction now
  returns the default append outcome if a previously peeked live worst candidate
  is no longer poppable, instead of panicking.

## Category Summary

- Category (a), accepted invariant: 114 remaining hits.
- Category (b), replaceable with `?` or explicit error: 2 fixed by this
  checkpoint, 0 remaining.
- Category (c), hot-path panic risk on remote-supplied data: 0 remaining.

## Accepted Category (a) Groups

- Fixed-width binary decoders:
  - `src/am/ec_spire/storage/header.rs`
  - `src/am/ec_spire/storage/routing_delta.rs`
  - `src/am/ec_spire/storage/leaf_v2_parts.rs`
  - `src/am/ec_spire/storage/relation_store.rs`
  - `src/am/ec_spire/storage/top_graph.rs`
  - `src/am/ec_spire/storage/assignment.rs`
  - `src/am/ec_spire/meta/epoch.rs`
  - `src/am/ec_spire/meta/local_store.rs`
  - `src/am/ec_spire/meta/object_manifest.rs`
  - `src/am/ec_spire/meta/placement.rs`
  - `src/am/ec_spire/meta/placement_directory.rs`
  - `src/am/ec_spire/meta/root_control.rs`
  - `src/am/ec_spire/meta/snapshot.rs`

  These `try_into().expect(...)` calls convert fixed-width slices after the
  decoder has already checked the record length, magic/version, entry count,
  segment count, or row stride that bounds the subsequent slice windows. The
  `expect` messages name the exact field width invariant.

- Validated local row encoders:
  - `src/am/ec_spire/storage/assignment.rs`
  - `src/am/ec_spire/storage/vec_id.rs`
  - `src/am/ec_spire/storage/helpers.rs`

  These convert buffers after constructor or parser validation of vec-id,
  payload, gamma, and local sequence byte lengths. The accepted invariant is
  local encoding shape, not remote input trust.

- Bounded integer conversions:
  - `src/am/ec_spire/coordinator/diagnostics.rs`
  - `src/am/ec_spire/coordinator/snapshots.rs`
  - `src/am/ec_spire/coordinator/remote_candidates/contracts.rs`
  - `src/am/ec_spire/coordinator/remote_candidates/libpq_plan.rs`
  - `src/am/ec_spire/coordinator/remote_candidates/pipeline.rs`

  These convert non-negative session settings, in-process vector lengths,
  bounded row counts, or constant iteration counts into wider or platform
  integer types. The source range is already clamped or materially smaller
  than the target range.

- SPI, pgrx, and planner pointer invariants:
  - `src/am/ec_spire/scan/relation.rs`
  - `src/am/ec_spire/custom_scan/planner.rs`

  `f32::into_datum()` is expected to produce a Datum for a Postgres `float4`
  score. Planner pointer dereferences occur only after explicit null guards;
  the `expect` strings name the checked pointer invariant.

- Build, update, catalog, and validated-route invariants:
  - `src/am/ec_spire/build/recursive.rs`
  - `src/am/ec_spire/build/tuples.rs`
  - `src/am/ec_spire/update/types.rs`
  - `src/am/ec_spire/coordinator/snapshots.rs`
  - `src/am/ec_spire/scan/routing.rs`
  - `src/am/ec_spire/scan/candidates.rs`

  These remain accepted because the surrounding code has already resolved the
  catalog attribute, formed a draft-placement configuration, checked that the
  local node row exists for local stores, validated top-graph route membership,
  or checked that a retained vec-id exists before mutably retrieving it.

- Constant/default construction:
  - `src/am/ec_spire/quantizer/mod.rs`

  The default RaBitQ configuration is a local constant configuration that is
  validated once through the normal constructor.

## Final File Counts

```text
11 src/am/ec_spire/storage/top_graph.rs
11 src/am/ec_spire/storage/leaf_v2_parts.rs
10 src/am/ec_spire/storage/header.rs
 8 src/am/ec_spire/meta/object_manifest.rs
 7 src/am/ec_spire/meta/placement.rs
 7 src/am/ec_spire/meta/local_store.rs
 6 src/am/ec_spire/storage/relation_store.rs
 6 src/am/ec_spire/storage/assignment.rs
 5 src/am/ec_spire/storage/routing_delta.rs
 5 src/am/ec_spire/meta/root_control.rs
 5 src/am/ec_spire/meta/epoch.rs
 4 src/am/ec_spire/meta/placement_directory.rs
 3 src/am/ec_spire/custom_scan/planner.rs
 3 src/am/ec_spire/coordinator/snapshots.rs
 3 src/am/ec_spire/coordinator/remote_candidates/contracts.rs
 3 src/am/ec_spire/build/recursive.rs
 2 src/am/ec_spire/update/types.rs
 2 src/am/ec_spire/storage/vec_id.rs
 2 src/am/ec_spire/storage/helpers.rs
 2 src/am/ec_spire/scan/routing.rs
 2 src/am/ec_spire/build/tuples.rs
 1 src/am/ec_spire/scan/relation.rs
 1 src/am/ec_spire/scan/candidates.rs
 1 src/am/ec_spire/quantizer/mod.rs
 1 src/am/ec_spire/meta/snapshot.rs
 1 src/am/ec_spire/coordinator/remote_candidates/pipeline.rs
 1 src/am/ec_spire/coordinator/remote_candidates/libpq_plan.rs
 1 src/am/ec_spire/coordinator/diagnostics.rs
```
