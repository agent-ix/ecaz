# Review Request: Vector-Backed Candidate Frontier

Scope:
- `src/am/scan.rs`
- `src/lib.rs`

What changed:
- Replaced the fixed in-opaque candidate frontier storage with a scan-owned heap `Vec<ScanCandidate>`.
- Kept the current seeded behavior intentionally narrow: `amrescan` still seeds only the entry candidate plus at most one successor candidate.
- Changed frontier-head consumption to remove the current head from the vector, compact the remaining candidates, and recompute the head afterward.
- Updated focused regression coverage so the existing debug surface now verifies compaction semantics instead of assuming fixed slots remain stable after consumption.

Review focus:
- Whether the vector-backed frontier is the right minimal ownership boundary before introducing a real traversal heap or queue
- Whether frontier lifecycle and head recomputation are correct across rescan, partial scan progress, and exhaustion
- Whether the current debug/test surface is still sufficient while seeded behavior remains intentionally capped to two candidates

Questions to answer:
- Is a heap-owned `Vec<ScanCandidate>` the right next representation for the frontier before real graph expansion begins?
- Are there any lifecycle or compaction edge cases around head removal that should be covered before the frontier starts growing beyond the current seeded pair?
- Should the frontier-head representation change before traversal expansion starts, or is the current bounded form acceptable until the frontier can actually exceed two entries?
