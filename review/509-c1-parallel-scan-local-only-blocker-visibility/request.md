# Review Request: Parallel Scan Local-Only Blocker Visibility

Current head: `b7d8751`

Scope:
- `src/am/ec_hnsw/scan.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- Packet 508 made stable foreign-owner fallback rows local-only between
  shared retries by clearing the coordinator result slot.
- That avoided stale shared publication, but it also hid the retained
  foreign-owner blocker from the shared worker snapshot during that
  local-only window.

What changed:
- `publish_parallel_scan_worker_slot_snapshot(...)` now publishes blocker
  metadata from either:
  - the current live owned-output blocker, or
  - the retained foreign blocker when local-only fallback is active
- The worker snapshot now keeps the blocker kind, owner slot, and blocker
  generation visible while the coordinator result slot remains cleared.
- The focused local-only fallback test now proves both sides of the contract:
  - worker snapshot stays active and retains the foreign blocker metadata
  - coordinator result slot stays unpublished during the local-only window

Why this matters:
- It keeps the staged ownership-handoff seam visible to shared diagnostics even
  when the row is intentionally suppressed from coordinator selection.
- That makes the local-only retry contract easier to reason about: the row is
  hidden from shared output ordering, but the blocker that caused the fallback
  is still externally visible in worker runtime state.

Feedback processed in this slice:
- Retrospective feedback for packets `481` through `489` was reviewed at the
  start of this turn.
- No new blocker was introduced for this packet; the cumulative follow-up that
  still stands is the LWLock-based serializer replacement before
  `amcanparallel = true`.

Still intentionally deferred:
- the real multi-worker output handoff / ownership transfer contract
- planner-visible parallel execution and `amcanparallel = true`
- the LWLock-based serializer replacement before real parallel enablement
- `n = 2/4/8` correctness coverage once the real multi-worker path lands

Validation:
- Passed:
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether keeping the retained foreign blocker in the worker snapshot while
  suppressing coordinator publication is the right staged boundary
- Whether the blocker kind/slot/generation is the right minimum shared
  visibility surface for the current local-only retry contract
