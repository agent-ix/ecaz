## Feedback: Debug Heap-Backed Scan And Vacuum Repair

Read the packet and the described changes in
`src/am/scan_debug.rs` and `src/am/vacuum.rs`.

### What's right

- **Fixes a real production/debug call-shape mismatch.** Several
  debug helpers were opening only the index relation, skipping
  heap-open, and relying on whatever active snapshot was current.
  That is fine for helpers that only poke index-owned state, but
  once `404` made `pq_fastscan` default to heap-backed rerank,
  a debug helper that doesn't open the heap was literally testing
  a different path than production. The packet names this
  honestly rather than papering over it.
- **Shared helper is the right factoring.**
  `DebugHeapBackedScan` + `debug_begin_heap_backed_scan` /
  `debug_end_heap_backed_scan` consolidates the `index_open +
  table_open + index_beginscan` boilerplate into one place. That
  kills the "which debug helpers remembered to open the heap"
  footgun by construction.
- **Fresh latest snapshot everywhere.** `debug_push_latest_snapshot`
  means debug helpers no longer lean on stale-active-snapshot
  luck. Stale-snapshot reuse is the kind of bug that produces
  flaky tests that sometimes pass by happening to catch the right
  snapshot boundary — fixing it at the source is better than
  debugging individual flakes.
- **Vacuum helper now sets `heaprel`.** Without this,
  source-backed vacuum repair was being tested through a shape
  that would never match a real `ambulkdelete` / `amvacuumcleanup`
  call — the real callbacks receive `IndexVacuumInfo.heaprel` set.
  Correcting the debug helper to match closes the shape gap that
  earlier feedback on `409` had implicitly trusted.

### Concerns

1. **Every affected debug helper changes behavior silently for
   callers.** Any test that relied on the old reused-active-
   snapshot behavior (e.g., coordinating a debug scan with an
   outer transaction snapshot) will now get a different answer.
   Probably no such test exists, but the packet doesn't
   enumerate which callers were migrated to the new path, which
   makes "did we regress anything" a `cargo test` question, not
   a review question. A one-paragraph migration summary — "these
   N helpers routed through the new path; none of them had test
   callers that depended on the old snapshot semantics" — would
   make the review case cleaner.
2. **No test asserts the shape claim.** The packet's whole point
   is "debug helpers now match the real call shape." But there
   is no test that, e.g., calls both `debug_gettuple_scan_...`
   and the real `amgettuple` path on the same fixture and asserts
   they return the same result. Without that, the claim "these
   lanes match shape" is still an inspection-level claim, not a
   runtime-verified one.
3. **`debug_profile_ordered_scan_with_heap_fetch` change scope.**
   The packet says that helper now "forces a fresh latest
   snapshot instead of opportunistically reusing a stale active
   one." That is the right direction, but it is a semantic
   change to a profiling helper — if a profile caller was
   timing the helper's scan cost against a specific long-running
   transaction, the new snapshot push could change the measured
   cost. Probably fine for debug profiling, but worth one
   sentence confirming no real caller depends on the old
   behavior.
4. **Both of the previously-landed `409` guardrail tests depend
   on this.** `test_tqhnsw_storage_format_switch_rejects_vacuum_
   until_reindex` exercises `debug_vacuum_remove_heap_tids`. That
   test's correctness now depends on `heaprel` being populated.
   If `409`'s test had silently passed before this repair
   because the guardrail error fires before heap access, it
   would still pass now — but this packet is changing the
   fixture shape underneath that test. Worth naming the
   interaction explicitly so the merge reviewer isn't surprised.

### Observation

Quiet but important. The kind of debug-helper repair that would
not matter at all until someone ships a heap-backed runtime path
— and then it matters a lot, because every `#[pg_test]` that
uses these helpers was subtly drifting away from the real shape.
The right time to fix this was now, while tests are being added,
not later when a discrepancy surfaces as a mystery test flake.
