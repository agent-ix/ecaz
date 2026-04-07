# Request: Remove Dead Scan Candidate Boundary

Commit: `d688919`

Summary:
- Removes the unused `ScanCandidate` type from `src/am/scan.rs` along with its `Default` impl and beam-conversion helpers.
- Leaves the runtime, debug, and test surfaces on `BeamCandidate<ItemPointer>` and `CurrentScanResult`, with no behavior change.
- Confirms that the prior beam-native cleanup left `ScanCandidate` as dead boundary code rather than an active execution or debug contract.

Files:
- `src/am/scan.rs`

Why this matters:
- Recent slices moved visible-frontier storage, active candidate state, debug helpers, and test fixtures onto beam-native state.
- Keeping an unused scan-local candidate type around would make the ownership split look more ambiguous than it really is.
- Removing it makes the next real design question clearer: whether result identity should stay scan-local in `CurrentScanResult`, or start moving behind `src/am/search.rs`.

Review focus:
- Whether any intended boundary semantics were accidentally tied to `ScanCandidate` and should instead be made explicit elsewhere
- Whether `CurrentScanResult` is now the only remaining scan-local result carrier that matters for the execution path
- Whether the next structural step should target result/materialization ownership instead of more candidate-shape cleanup
