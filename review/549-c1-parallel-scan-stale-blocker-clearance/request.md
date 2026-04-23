# Review Request: Parallel Scan Stale Blocker Clearance

Current head: `319e497`

Scope:
- `src/am/ec_hnsw/scan.rs`
- `plan/tasks/18-parallel-index-scan.md`

Problem:
- Deferred and local-only fallback still treated retained foreign blocker
  metadata as authoritative even after the shared selected/admitted blocker
  generation had already disappeared.
- That meant a row could still drain through blocked-fallback accounting even
  though the foreign blocker was already gone.
- In practice this overstated blocked fallback and made stale blocker metadata
  survive longer than the real shared state.

What changed:
- Added a liveness check for retained foreign selected/admitted blockers when a
  real parallel scan state is bound.
- Local-only duplicate resolution now clears stale retained blocker metadata
  before the hidden row goes through its normal emit path.
- Deferred drain now clears stale retained blocker metadata before treating a
  row as blocked fallback.
- Same-element obsolete-drop still wins before stale-clear logic, and the old
  no-shared-state unit fixtures keep their conservative blocked semantics.
- Added focused coverage proving:
  - stale retained blocker metadata clears before deferred drain
  - stale retained blocker metadata clears before local-only emit bookkeeping
- Updated Task 18 notes to record the stale-blocker clearance seam.

Why this matters:
- Dead blocker generations no longer inflate deferred/local-only fallback
  counters.
- Rows whose foreign blocker has already disappeared now drain through their
  normal ready path instead of pretending to remain blocked.
- This narrows the remaining ownership-transfer gap without changing the still
  deferred contract for genuinely blocked unique outputs.

Still intentionally deferred:
- the final cross-worker ownership-transfer contract for genuinely blocked
  unique outputs
- planner-visible parallel execution and `amcanparallel = true`
- `n=2/4/8` correctness and measurement once the remaining seam lands

Validation:
- Passed:
  - `cargo test`
  - `cargo run -p ecaz-cli -- dev test pgrx --pg 18`
  - `cargo run -p ecaz-cli -- dev test pg18-preload-pgstat`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`

Review focus:
- Whether stale retained blocker metadata is now cleared at the right fallback
  boundaries
- Whether any remaining fallback path can still count or emit a row as blocked
  after the shared foreign blocker generation is already gone
