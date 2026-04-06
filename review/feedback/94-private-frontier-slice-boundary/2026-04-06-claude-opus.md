# Feedback: Private Frontier Slice Boundary

Request:
- `review/94-private-frontier-slice-boundary.md`

**Reviewer:** Claude (Opus)
**Date:** 2026-04-06

## Response to Review Focus

### Is the snapshot helper the right narrow debug-facing seam?

**Yes.** `visible_frontier_snapshot` (scan.rs:419-421) returns a `Vec<ScanCandidate>` by value — a full copy of the frontier state at the point of call. This is the correct debug contract:
1. **No aliasing**: the snapshot is an independent copy, so debug code can't accidentally hold a reference that blocks mutation
2. **No raw-slice exposure**: `scan_debug.rs` never sees `&[ScanCandidate]` directly
3. **Stable across mutations**: debug code can snapshot, mutate, snapshot again, and diff

The helper delegates to `visible_frontier_ref(opaque).iter().collect()`, which goes through the `VisibleCandidateFrontierState::iter()` method. Clean chain.

`candidate_slot(opaque, index)` (scan.rs:415-417) provides the other debug entry point — positional access with a default fallback for out-of-bounds. This is used by `scan_debug.rs` at 13+ sites for slot-level assertions.

Together, `visible_frontier_snapshot` and `candidate_slot` are sufficient for all debug/test needs without exposing the raw slice.

### Does any remaining non-scan code depend on raw frontier-slice access?

**No.** Confirmed:
- `scan_debug.rs` uses only `visible_frontier_snapshot`, `candidate_slot`, and `current_candidate_frontier_head_tid` — no raw Vec or slice access
- `candidate_frontier_ref` (scan.rs:395-397) is private to `scan.rs` — the `fn` is not `pub(super)` (it returns `&[ScanCandidate]` but only for internal use within `scan.rs` and unit tests)

The boundary is real: the raw `candidates` field is private to `VisibleCandidateFrontierState`, and the raw-slice accessor is private to `scan.rs`.

### Should the next step move ownership or add more container behavior?

**Review 95 completes the encapsulation** by replacing `*mut Vec<ScanCandidate>` with `*mut VisibleCandidateFrontierState` in `TqScanOpaque`. After that, ownership transfer into `search.rs` becomes feasible. This is the right sequence: seal, then move.

## Additional Findings

No issues found. The private boundary makes the encapsulation enforceable rather than just conventional.
