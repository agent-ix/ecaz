# Review Request: Parallel Scan Linear Admission Window

Current head: `94b51fa`

Scope:
- `src/am/ec_hnsw/scan.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- The scan layer had a generic blocked-owner fallback, but linear fallback still
  treated every blocked staged row the same way.
- That meant a row blocked only because it lost the current admitted window
  could still go through the local-direct emit fallback, even though it had not
  survived the shared admission seam.
- The staged shared snapshot also needed an explicit discard path so dropping a
  staged loser would clear both local and shared duplicate-drain state.

What changed:
- Added `BlockedParallelScanDisposition` and
  `blocked_parallel_scan_disposition(...)` in the scan layer.
- `AdmissionWindow` blockers now map to `DropAndContinue`; foreign-owner
  blockers keep the existing local-direct emit fallback until the real handoff
  contract lands.
- Added `discard_active_parallel_scan_output(...)` to clear the active staged
  current result and republish an empty shared snapshot.
- Wired linear fallback tuple production to:
  - drop admission-window losers from staged local/shared state
  - continue searching locally for the next candidate
  - keep the explicit local-direct emit fallback only for foreign blockers
- Added focused scan tests for:
  - blocker disposition
  - discard helper clearing fallback state and shared snapshot

Why this matters:
- This is the first place where the new blocker taxonomy actually changes scan
  behavior instead of only being diagnostic metadata.
- It makes the staged linear path more faithful to the eventual shared merge
  semantics by not routing obvious admission losers around the merge seam.

Still intentionally deferred:
- the real multi-worker output handoff / ownership contract
- analogous graph-side handling beyond the existing staging fallback
- planner-visible parallel execution and `amcanparallel = true`

Validation:
- Passed:
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether dropping admission-window losers in linear fallback is the right
  staged behavior before the full handoff contract lands
- Whether the discard helper fully clears both local and shared staged state
