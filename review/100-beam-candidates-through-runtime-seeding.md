# Request: Beam Candidates Through Runtime Seeding

Commit: `ad6b45b`

Summary:
- Changes runtime bootstrap entry seeding, successor collection, and discovered-candidate seeding in `src/am/scan.rs` to operate on shared `search::BeamCandidate<ItemPointer>` values directly.
- Keeps `ScanCandidate` at the scan/debug/materialization boundary via `From` conversions instead of using it as the runtime seeding payload.
- Preserves existing frontier/debug behavior while reducing one more hot-path dependency on the scan-local candidate shape.

Files:
- `src/am/scan.rs`

Why this matters:
- The previous slice aligned visible-frontier storage with `BeamCandidate`, but runtime seeding/refill still manufactured `ScanCandidate` values first and converted them immediately.
- That kept the hot path conceptually more scan-local than it needed to be.
- This slice pushes the shared search candidate type further out into runtime bootstrap flow, which is a cleaner base for any later move of more frontier ownership toward `search.rs`.

Review focus:
- Whether the new `From`-based boundary between `BeamCandidate` and `ScanCandidate` is the right tradeoff for the current transition phase
- Whether any runtime/bootstrap path still unnecessarily constructs `ScanCandidate` values before crossing back into search-owned state
- Whether the next step should be to convert more consume/materialization plumbing to beam candidates, or instead to pull more visible-frontier responsibility directly behind `search.rs`
