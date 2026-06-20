# Task 111h Review Request: Packed Rerank Lifecycle Fixtures

Code commit: `90d74bd208ff4f847d4cc765e003582eb8765bfa`

## Summary

This checkpoint adds focused PG18 coverage for the packed index-side rerank
path introduced in Task 111h.

- Exposes the existing packed rerank counters through
  `debug_ec_ivf_gettuple_counter_snapshot`: placement, format, group header
  pages, payload segment pages, group metadata bytes, header/segment payload
  bytes, scored payload bytes, and batch slab copied bytes.
- Extends the index-placement byte fixture to assert:
  - source f32 uses `placement=source`, `format=f32`, and does not read packed
    groups,
  - index f16 uses packed groups, scores exactly the persisted compact payload
    bytes, and copies zero bytes into the batch slab,
  - index RaBitQ-4 exposes the current batch payload slab copy cost.
- Renames and tightens the live insert and vacuum fixtures around the packed
  group semantics:
  - post-build inserts rerank from appended packed payloads,
  - vacuum tombstones a packed group slot and live survivors still rerank from
    packed payloads.

This does not close the full Task 111h lifecycle item yet: fallback/full-chain
lookup and mixed postings without direct group pointers still need targeted
coverage.

## Validation

Artifacts are recorded in `artifacts/manifest.md`.

- `cargo check --no-default-features --features pg18`: passed.
- `cargo pgrx test pg18 test_ec_ivf_index_placement`: passed, 4 tests.
