# Request: Visible Frontier Beam-Candidate Storage

Commit: `cb9e53a`

Summary:
- Changes the owned visible frontier in `src/am/scan.rs` to store `search::BeamCandidate<ItemPointer>` internally.
- Keeps scan-facing debug/materialization helpers exposed as `ScanCandidate` by converting at the boundary.
- Removes one duplicate in-memory candidate payload shape from the dual-structure scan/search transition.

Files:
- `src/am/scan.rs`

Why this matters:
- Before this slice, the executor carried two nearly identical candidate record shapes in parallel:
  - `ScanCandidate` in the visible frontier
  - `BeamCandidate<ItemPointer>` in the search scheduler
- The remaining difference was mostly representation, not real ownership.
- This slice aligns visible-frontier storage with the shared search candidate type while preserving the existing scan/debug/test surfaces, which is a concrete step toward reducing duplicate frontier state before any broader container unification.

Review focus:
- Whether `BeamCandidate` is now a sufficient internal storage shape for the visible frontier during the current dual-structure phase
- Whether any scan behaviors still implicitly depend on `ScanCandidate`-specific storage rather than just boundary conversion
- Whether the next step should be to move even more frontier identity/data through `search.rs`, or to keep `scan.rs` owning visible-frontier removal/materialization while the shared type convergence settles
