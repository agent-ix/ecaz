---
agent: claude-opus-4-7
role: reviewer
model: claude-opus-4-7
date: 2026-06-03
seq: 06
---

# Phase 3 Worker-Curve — Loop idle status

## Summary

The review loop has been running on this branch since 2026-06-02
and is now in dormant state. The last coder commit on Task 71 is
`167490db5 Update Task 71 worker curve suite reloptions`. Seven+
review ticks have passed without a new coder commit; the working
tree has not changed since 2026-06-02 23:11 (the failed pg_test
run after the routine.rs edit).

Bumping cadence to 3600s tick interval until the coder pushes
motion or the user nudges the loop.

## Outstanding items (all open, none resolved)

### Packet 001 — phase1-design (BLOCK — seq 01)
- No Amdahl bound on heap-scan share.
- Determinism / sample-set-invariance claim unjustified.
- Posting-flush dominance unanalyzed.

### Packet 002 — parallel-heap-ingest (REQUEST-CHANGES — seq 01)
- `# Safety` docs missing on unsafe fn surface
  (`src/am/ec_ivf/build_parallel.rs:485,494,787,798`).
- `IndexInfoView::as_mut` borderline anti-pattern B (cross-engine
  cleanup with HNSW).
- `IndexInfoView` / `build_index_info_inner` duplicated from HNSW
  — lift to `src/am/common/index_info.rs`.
- 2× heap-row memory peak / no HWM gate.
- Unused `nparticipantsdone` / `scanned_heap_tuples` counters.
- Missing parallel-vs-serial page-equivalence pg_test (Phase 2
  step 4 exit criterion).

### Packet 003 — worker-curve (BLOCK — seq 01–05)
- Pre-fix build-time matrix is flat-to-negative across
  10k/25k/50k/100k. Stop Condition triggered.
- Path B fix (parallel-scan callbacks added to ec_ivf routine)
  identified the right bug per seq 04. **Fix is uncommitted.**
- pg_test failed on cargo-pgrx install permissions; suite is the
  real validator. Suite has not been re-run with the fix.
- `parallel-workers-after.log` still 0 (pre-fix capture from
  2026-06-02 22:15).
- `manifest.md` lacks per-cell rows.
- `request.md` does not cite Task 31 baseline recall numbers.
- `allow_manifest_mismatch: true` not justified.
- 100k w=8 cell missing.
- All 100k / 50k / 25k / 10k load/recall/storage logs and
  `results.jsonl` remain untracked.

## What unblocks the loop

Any one of these resumes tight cadence (1200-1500s):

1. **Path B commit + suite re-run** with `workers_launched > 0`
   in the new `parallel-workers-after.log` and an updated
   build-time matrix.
2. **Packet 004 invoking the Stop Condition** with a clean
   write-up citing the pre-fix matrix and the post-fix
   confirmation that the parallel callbacks are wired but the
   curve is still a wash on M5.
3. **A direct user nudge** redirecting scope or priority.

Until one of the above lands, the loop holds at 3600s and will
not generate per-tick feedback files (would just duplicate seq
01–05 on identical data).
