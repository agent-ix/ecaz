# Request: Beam Candidates Through Runtime Consume

Commit: `1664deb`

Summary:
- Changes runtime frontier consumption, refill-after-consume, and direct bootstrap-result materialization in `src/am/scan.rs` to operate on shared `search::BeamCandidate<ItemPointer>` values.
- Keeps the scan-owned `active_candidate` field and debug surfaces on `ScanCandidate`, converting only at those boundaries.
- Updates `src/am/scan_debug.rs` to consume the new beam-shaped runtime result from the frontier consume helper without reaching through stale scan-local field assumptions.

Files:
- `src/am/scan.rs`
- `src/am/scan_debug.rs`

Why this matters:
- Before this slice, seeding/refill already used `BeamCandidate`, but the next runtime stage immediately converted back into `ScanCandidate` for consume, refill-after-consume, and direct materialization.
- That left the hot path split across two candidate representations for no real ownership reason.
- This slice pushes the shared search candidate type deeper into active execution, leaving `ScanCandidate` more clearly as a boundary/persistent-state shape rather than the transient frontier execution payload.

Review focus:
- Whether the new beam-candidate runtime flow is the right place to stop before tackling `active_candidate` itself
- Whether any debug or helper paths still assume consume/refill returns a scan-local candidate shape instead of the shared search candidate
- Whether the next step should convert `active_candidate` too, or instead focus on folding more visible-frontier ownership into `search.rs`
