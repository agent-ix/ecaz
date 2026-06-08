# Task 87 Packet 006: HNSW FullLut Structural CandidateBatch Route

## Summary

This packet asks for review of the HNSW structural slice for Task 87. It
routes newly loaded HNSW TurboQuant `FullLut` no-QJL 4-bit exact payloads
through the shared `CandidateBatch` abstraction during cached successor
expansion.

Code checkpoint under review:

- `c44c7fe6c3d886419174c1996e5dfb78d9cceb7f` - `Route HNSW FullLut scoring through CandidateBatch`

## Changes

- Defers scoring of newly loaded live exact payloads when
  `turboquant_exact_score_mode == FullLut`, preserving owned code bytes in
  `LoadedElementState::ExactPayload`.
- Adds a HNSW `FullLut` batch scorer that:
  - builds `CandidateBatch` entries over owned exact payload bytes;
  - delegates to `score_turboquant_no_qjl_4bit_batch`;
  - writes the existing HNSW score cache with negated inner products;
  - preserves per-candidate distance-count and score-cache miss accounting.
- Updates the non-binary cached successor loop to collect consecutive
  `FullLut` exact payloads into a batch and flush before scalar/grouped
  boundaries, preserving candidate output order for mixed paths.
- Adds a unit test proving batch scores match the scalar `FullLut` path
  and that scored candidates are cached.

This packet intentionally scopes HNSW to `TurboQuantExactScoreMode::FullLut`
only, per packet 002. `TiledLut`, `Int8Approx`, and generic exact scoring
remain on their current scalar paths.

## Validation

See `artifacts/manifest.md` for artifact metadata.

- `artifacts/cargo-test-hnsw-scan.log`
  - `cargo test --lib am::ec_hnsw::scan::tests --no-default-features --features pg18`
  - result: `74 passed; 0 failed`

## Review Focus

- Confirm `FullLut`-only routing matches the packet 002 HNSW scope.
- Confirm the owned `LoadedElementState::ExactPayload` bytes are an
  acceptable backing store for borrowed `CandidateBatch` entries.
- Confirm flushing before scalar/grouped boundaries preserves traversal
  candidate order for mixed successor lists.
- Confirm the score-cache and distance-count accounting remains consistent
  with the previous scalar path.
