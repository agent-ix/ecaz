# Request: Beam-Native Scan Test Fixtures

Commit: `34f0bd4`

Summary:
- Changes the remaining `src/am/scan.rs` unit-test frontier fixtures to construct `BeamCandidate<ItemPointer>` values directly through local helper functions.
- Removes the last `ScanCandidate { ... }` literals from the `scan.rs` test module without changing runtime code paths.
- Keeps `ScanCandidate` as an explicit boundary/debug type while making the scan test fixtures reflect the beam-native runtime ownership model.

Files:
- `src/am/scan.rs`

Why this matters:
- Recent slices already moved visible-frontier storage, active candidate state, head selection, and debug helpers onto beam-native structures.
- The remaining `ScanCandidate` literals in `scan.rs` tests were no longer exercising the hot path as it actually runs; they were just fixture churn converting immediately back into beam-native candidates.
- This slice makes the tests easier to read and narrows the remaining `ScanCandidate` surface toward true boundary/debug needs.

Review focus:
- Whether the new `tid`, `beam_candidate`, and `sourced_beam_candidate` helpers are the right minimal fixture seam for `scan.rs` tests
- Whether any remaining `ScanCandidate` usage in `scan.rs` still reflects a necessary boundary, rather than leftover fixture convenience
- Whether this cleanup makes the next structural move clearer: continue shrinking `ScanCandidate`, or shift to moving more result/frontier identity into `src/am/search.rs`
