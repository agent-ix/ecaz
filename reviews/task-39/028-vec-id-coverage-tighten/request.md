# Task 39 SpireVecId coverage tightening

## Summary

Closes two specific coverage gaps in `src/am/ec_spire/storage/vec_id.rs`
that follow the same boundary-equality pattern as the RaBitQ closeout
in packet 027:

1. `SpireVecId::global` only had a "just-too-big" rejection test
   (`vec![7; SPIRE_VEC_ID_MAX_BYTES]`); the "just-fits" boundary
   (`SPIRE_VEC_ID_MAX_BYTES - 1` payload bytes) was untested, so a
   `> → >=` mutation on the length guard at vec_id.rs:272 would
   survive.
2. `SpireVecId::local_sequence` had no test asserting `None` for a
   global vec_id; the early-return branch on the discriminator check
   (vec_id.rs:300-303) was structurally executed only via the local
   path.

Both tests are MIRI-friendly (no `unsafe`, no pgrx interaction) so they
flow into the hardening MIRI lane.

This is a narrow follow-up slice in the Task 39 coverage burndown
(handoff identifies `am/ec_spire/storage/vec_id.rs below 80%`).

## Code under review

- Commit: `ca10cef215d9fa818b659def7dcbdc57e2c354d8`
- Changed file: `src/am/ec_spire/storage/tests/vec_and_routing.rs`

## Validation

- `cargo test --manifest-path hardening/careful/Cargo.toml --lib
  careful_spire::storage::tests::miri` passed: 20 miri-prefixed
  storage tests, including the 2 new ones (artifact:
  `artifacts/vec-id-focused-tests.log`).
- No clippy / build delta beyond the two new test functions.

## Notes

- `cargo-mutants` cannot directly target ec_spire/storage files today
  because the careful shadow crate includes them via `include!`, not
  `#[path = "..."]`. Restructuring that wiring is outside the scope of
  this slice; a separate slice can convert the includes to path
  modules if the reviewer wants empirical mutation evidence.
- Remaining `vec_id.rs` gaps (private `SpireVecIdKind::decode` error
  path; `SpireLeafObjectColumns::row` overflow / out-of-bounds error
  branches) are reachable only through their callers in `leaf_v2.rs` /
  `leaf_v2_parts.rs`, so they belong with the leaf-V2 coverage slice
  rather than this one.
