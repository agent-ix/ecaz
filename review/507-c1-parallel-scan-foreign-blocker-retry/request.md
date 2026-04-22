# Review Request: Parallel Scan Foreign-Blocker Retry

Current head: `48f9519`

Scope:
- `src/am/ec_hnsw/scan.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- After packets 505 and 506, blocked-owner state carried blocker kind and
  blocker generation, but scan-side foreign-owner blockers still always fell
  back to the staged local `KeepLocalEmit` path.
- That meant a worker could keep locally emitting the same row even after the
  foreign owner had already advanced to a newer generation.

What changed:
- Added a scan-local retained foreign-blocker record keyed by staged
  `element_tid`.
- `blocked_parallel_scan_disposition(...)` is now generation-aware:
  - `AdmissionWindow` still drops and continues.
  - first observation of a foreign-owner blocker retries the shared seam.
  - a repeated identical foreign-owner blocker for the same staged row falls
    back to local emit.
  - a changed foreign-owner generation for the same staged row reopens one
    retry against the shared seam.
- The retained blocker clears on:
  - shared owned-output consume
  - explicit staged-output discard
  - new linear materialization
  - parallel bind/clear
- Added focused unit coverage for:
  - first foreign-owner blocker retry
  - stable repeated foreign-owner blocker falling back to local emit
  - changed foreign-owner generation reopening retry
  - admission-window blockers still dropping immediately

Why this matters:
- It is the first scan-side use of blocker generation as control flow rather
  than only diagnostics.
- That keeps the staged local fallback from holding the same row forever once
  the foreign owner has already moved.

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
- Whether “retry once on changed foreign generation, then local fallback on a
  stable repeat” is the right staged control-flow contract before the final
  ownership handoff lands
- Whether keying the retained blocker by `element_tid` is the right local
  boundary for scan-side retry state
