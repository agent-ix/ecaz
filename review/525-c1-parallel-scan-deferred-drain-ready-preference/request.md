# Review Request: Parallel Scan Deferred Drain Ready Preference

Current head: `2fa08f9`

Scope:
- `src/am/ec_hnsw/scan.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- Once the scan reached deferred-only drain, it still picked the best deferred
  row by score and locally emitted it if its foreign blocker remained live.
- That meant a still-blocked best row could force local fallback even when a
  lower-ranked deferred row was already ready to drain.

What changed:
- Refactored deferred drain around
  `restore_deferred_parallel_blocked_outputs(...)` plus an iterative
  `take_next_deferred_parallel_blocked_output(...)` loop.
- Live blocked deferred rows are now held aside while the drain keeps looking
  for another deferred row that can:
  - hand off through the shared seam
  - drop as obsolete
  - or drain locally without violating the current ownership gate
- Only if no deferred row is eligible does the staged path fall back to local
  emit of the remaining blocked row.
- Reworked the deferred-drain unit coverage onto the non-FFI helper seam so
  PG18 pgrx lanes do not trip `pgrx`'s multithreaded FFI guard.
- Added focused coverage for:
  - skipping an obsolete deferred row and draining the next eligible one
  - skipping a still-blocked best deferred row in favor of a ready next row
  - still making progress when the only remaining deferred row is blocked

Why this matters:
- It narrows one more ownership gap without pretending the final cross-worker
  transfer protocol is landed.
- Deferred-only drain now behaves more like “drain what is actually ready”
  instead of “force local fallback on the first blocked best row”.

Still intentionally deferred:
- full cross-worker ownership transfer instead of scan-local deferred fallback
- planner-visible parallel execution and `amcanparallel = true`
- `n=2/4/8` correctness and measurement after the remaining ownership seam
  lands

Validation:
- Passed:
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- Note:
  - the first `ecaz dev test pgrx --pg 18` attempt overlapped another long
    `cargo test` run and fanned out into the known broad `pgrx` harness
    failure mode; the serial rerun on the unchanged tree was green and is the
    result that counts for this checkpoint

Review focus:
- Whether the new deferred-drain ordering is the right staged contract before
  full ownership transfer exists
- Whether the held-aside blocked-row restore path is the right boundary for
  keeping progress while still preferring shared handoff and ready deferred work
