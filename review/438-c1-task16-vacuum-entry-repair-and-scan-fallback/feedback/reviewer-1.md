## Feedback: Vacuum entry repair + scan fallback — ACCEPTED, fixes the packet-437 concurrency bug

Verified against:

- commit `be4c0a4` adding `LiveEntryCandidate` + `highest_level_live_entry_candidate(...)` in `src/am/shared.rs`
- `src/am/vacuum.rs` `repair_metadata_entry_point_after_vacuum(...)` wired after `finalize_fully_dead_elements(...)`
- `src/am/scan.rs` `initialize_scan_entry_candidate(...)` fallback path
- new pg tests `test_tqhnsw_vacuum_repairs_deleted_entry_point_metadata` and
  `test_tqhnsw_scan_falls_back_from_stale_entry_metadata` in `src/lib.rs`
- rerun of `scripts/vacuum_concurrency_scratch.sh --socket-dir /home/peter/.pgrx --duration 60`
  now reporting `vacuum concurrency harness passed`

### What's right

- **Fix is in the right layer.** This is generic AM hygiene — stale
  metadata entry-point under concurrent vacuum — not a TurboQuant
  scorer concern. Repair lives in the shared `shared.rs` /
  `vacuum.rs` / `scan.rs` lifecycle paths and covers all three
  storage descriptors (`TurboQuant`, `TurboQuantHotCold`,
  `PqFastScan`) in one sweep. Exactly the right blast radius.
- **Both sides of the race are closed.** Vacuum repairs the metadata
  after finalizing dead elements; scans also fall back if they
  happen to observe a stale entry point before vacuum gets there.
  Either path alone would leave a window; both together close it.
- **Symmetric on the vacuum side.** `repair_metadata_entry_point_after_vacuum(...)`
  early-returns on empty `finalize_tids`, and when no live elements
  remain it resets metadata to `INVALID` / `0` rather than leaving
  stale state behind. `max_level` is kept aligned with the repaired
  entry point — if this drifted, upper-layer seed selection would
  silently return wrong results.
- **pg tests target the exact failure shape.** The vacuum test
  proves metadata is repaired after the current entry is finalized;
  the scan test proves queries still return rows under stale
  metadata. Together they lock in both halves of the fix.
- **Live harness passed.** The same 60-second scratch concurrency
  rerun that produced `unexpected tqhnsw scan result count: 0` in
  packet `437` now completes cleanly with non-trivial
  insert/scan/vacuum iterations (`335 / 196 / 585+583`). The fix is
  validated against the shape of workload that discovered the bug.

### Concerns

1. **`highest_level_live_entry_candidate(...)` is an O(N_blocks)
   linear scan.** It walks every data block under `BUFFER_LOCK_SHARE`
   and decodes every tuple. That's fine at 50k — cheap in absolute
   terms — but it scales linearly with index size, and both the
   scan-side fallback and the vacuum-side repair call it. On a
   multi-million-row index, a scan that trips into the fallback
   could pay a meaningful hit. Worth naming in a comment that this
   is a **repair path**, not a steady-state lookup, so a future
   reader doesn't regress the fast path into using it.
2. **Scan-side fallback runs without any rate limit.** If metadata
   is stuck `INVALID` for some reason (e.g. vacuum hasn't run yet
   after a bulk delete that wiped the current entry), every query
   pays the linear scan. The vacuum repair should close this
   window, but this is a new tail-latency failure mode for scans
   under an unexpected metadata state. Probably not worth guarding
   now, but worth a log/metric so that if it ever fires under real
   load it's visible rather than silent.
3. **No `reachable_vs_reference_percent` regression check in the
   harness output.** The rerun reports `reachable_vs_reference_percent=116`
   which is fine, but the harness readout doesn't say what the
   pass/fail contract actually is. The current pass text is
   `vacuum concurrency harness passed` — worth naming the exact
   invariant the harness now enforces, so future regressions to
   this code path are caught by a clear contract rather than a
   manual readout comparison.
4. **TOCTOU between SHARE-locked candidate scan and metadata
   rewrite.** The scan picks a candidate under `BUFFER_LOCK_SHARE`,
   releases, then re-locks the metadata exclusive to write. If
   another vacuum in a parallel backend finalizes the chosen
   candidate in that window, metadata would get repointed to a
   dead TID, and the next scan would fall back again. That is
   self-healing (next vacuum runs the repair again) but worth
   naming — "metadata may still be stale immediately after
   repair; scan fallback carries the correctness load" — so
   nobody assumes the metadata post-vacuum is always live.
5. **Plan status update, if not already.** Packet `437`'s feedback
   asked that this bug be lifted out as a tracked task. With the
   fix landed, please make sure task 16's plan reflects that the
   blocker from packet `428` / `432` / `437` is now closed — and
   that the scratch concurrency contract is being continuously
   asserted (via the harness or a pg_test invariant), not just run
   once by hand.

### Questions for coder-1

1. **Is the scan-side fallback instrumented?** Under what log level
   does a scan notice that it took the `INVALID` fallback path?
   If it's silent, a persistent metadata bug could hide as mild
   query-latency weather with no visible signal.
2. **Does the vacuum test exercise the "no live remain" branch?**
   The repair code has a path that resets to `INVALID` / `0` when
   no live element is found. Worth confirming that branch is hit
   by at least one test, not just the "live replacement found"
   path.
3. **Did any existing V1 / V2 / PqFastScan integration tests
   silently rely on the old "dead entry stays in metadata"
   behavior?** Not expecting breakage — tests are green — but if
   there were any, they'd be worth naming so future test
   refactoring doesn't re-introduce the bad assumption.

### Call

Accepted. The V3 vacuum-concurrency bug surfaced in packet `437` is
a correctness regression against the A6 vacuum contract, and this
slice fixes it at the right layer with coverage on both sides of
the race. The fix is generic across storage descriptors, the harness
rerun passes on the same command that originally failed, and the
scope is right — no policy changes, no TurboQuant scorer drift.
Concerns `1`–`4` are hardening notes, not blockers.
