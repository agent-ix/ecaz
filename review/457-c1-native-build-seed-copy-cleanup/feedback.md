# Feedback: 457-c1-native-build-seed-copy-cleanup

Reviewed against head `a344ad9`.

## Verdict

Approved. Trivial and obviously correct.

## Review focus answer

> Is it still worth carrying these tiny behavior-preserving cleanups, or stop
> optimization slices and focus on final landing evidence?

Recommendation: **stop here on pure micro-cleanups** and shift the branch's
remaining packets to landing evidence — recall gate rerun on real corpus,
BUILD throughput comparison vs. the removed `hnsw_rs` baseline, and any final
merge-blocker checklist.

Reasoning:

- `BeamCandidate<usize>` being `Copy` means `seeds.clone()` was already a
  shallow copy — this packet saves a `Vec` allocation per upper-layer
  iteration, which is real but small and not on the hot path.
- The three recent slices (455/456/457) have progressively diminishing
  returns, and each additional micro-slice spends review/validation budget
  that's better applied to the final merge-readiness story now that `hnsw_rs`
  is already removed (commit `ab117ea`).
- If a real BUILD-time measurement later flags a specific hotspot, optimize
  then with evidence rather than by inspection.

## Notes

- Diff is literally `seeds.clone()` → `seeds.iter().copied()` at one call
  site. No behavioral change possible.
- No new test needed.

## Blockers

None.
